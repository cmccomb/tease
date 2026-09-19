use crate::{
    bridge::{Request, Response},
    Teaser,
};
use serde::Deserialize;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    thread,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Submission {
    pub(crate) request_id: u64,
    inputs: Vec<String>,
}

impl Submission {
    pub(crate) fn parse(message: &str) -> Result<Self, String> {
        let submission: Self = serde_json::from_str(message)
            .map_err(|_| "Invalid request. Submit all fields again.".to_string())?;
        // Identifiers must round-trip exactly through JavaScript Number.
        if submission.request_id == 0 || submission.request_id > 9_007_199_254_740_991 {
            return Err("Invalid request identifier.".into());
        }
        Ok(submission)
    }
}

#[derive(Debug)]
pub(crate) enum Event {
    PageReady,
    SaveCsv(String),
    Ready(Result<(), String>),
    Finished {
        request_id: u64,
        response: Response,
        fatal: bool,
    },
    ProtocolError(String),
}

#[derive(Debug, PartialEq)]
enum Phase {
    Preparing,
    Idle,
    Running(u64),
    Failed,
    Closed,
}

pub(crate) struct Worker {
    sender: Option<SyncSender<Submission>>,
    closed: Arc<AtomicBool>,
    phase: Phase,
    last_request: u64,
}

impl Worker {
    pub(crate) fn spawn(
        demo: Teaser,
        notify: impl Fn(Event) -> bool + Send + 'static,
    ) -> std::io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel::<Submission>(1);
        let closed = Arc::new(AtomicBool::new(false));
        let worker_closed = Arc::clone(&closed);
        thread::Builder::new().name("tease-inference".into()).spawn(move || {
            let prepared = catch_unwind(AssertUnwindSafe(|| demo.prepare()));
            let session = match prepared {
                Ok(Ok(session)) => session,
                result => {
                    let message = match result {
                        Ok(Err(error)) => format!("Could not prepare model: {error}"),
                        _ => "Model initialization panicked. Restart the demo after fixing the model.".into(),
                    };
                    if !worker_closed.load(Ordering::Acquire) {
                        notify(Event::Ready(Err(message)));
                    }
                    return;
                }
            };
            if worker_closed.load(Ordering::Acquire) || !notify(Event::Ready(Ok(()))) { return; }
            while let Ok(job) = receiver.recv() {
                if worker_closed.load(Ordering::Acquire) { return; }
                let result = catch_unwind(AssertUnwindSafe(|| session.handle_request(Request { inputs: job.inputs })));
                let (response, fatal) = match result {
                    Ok(response) => (response, false),
                    Err(_) => (Response::Error {
                        field: None,
                        message: "Model prediction panicked. Restart the demo after fixing the model.".into(),
                    }, true),
                };
                if worker_closed.load(Ordering::Acquire)
                    || !notify(Event::Finished { request_id: job.request_id, response, fatal })
                    || fatal { return; }
            }
        })?;
        Ok(Self {
            sender: Some(sender),
            closed,
            phase: Phase::Preparing,
            last_request: 0,
        })
    }

    pub(crate) fn ready(&mut self, success: bool) -> bool {
        if self.phase != Phase::Preparing {
            return false;
        }
        self.phase = if success { Phase::Idle } else { Phase::Failed };
        true
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.phase == Phase::Idle
    }

    /// Reject duplicates while work is queued, running, or awaiting UI delivery.
    pub(crate) fn submit(&mut self, job: Submission) -> Result<bool, String> {
        if self.phase != Phase::Idle || job.request_id <= self.last_request {
            return Ok(false);
        }
        self.last_request = job.request_id;
        self.phase = Phase::Running(job.request_id);
        if self
            .sender
            .as_ref()
            .is_none_or(|sender| sender.try_send(job).is_err())
        {
            return Err("The inference worker stopped. Restart the demo.".into());
        }
        Ok(true)
    }

    pub(crate) fn finish(&mut self, request_id: u64, fatal: bool) -> bool {
        if self.phase != Phase::Running(request_id) {
            return false;
        }
        self.phase = if fatal { Phase::Failed } else { Phase::Idle };
        true
    }

    pub(crate) fn close(&mut self) {
        self.closed.store(true, Ordering::Release);
        self.phase = Phase::Closed;
        self.sender.take();
        // Never join a possibly blocked user callback on the UI thread.
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc, time::Duration};
    const TIMEOUT: Duration = Duration::from_secs(10);
    fn job(id: u64, value: &str) -> Submission {
        Submission::parse(&serde_json::json!({"request_id":id,"inputs":[value]}).to_string())
            .unwrap()
    }
    fn launch(demo: Teaser) -> (Worker, mpsc::Receiver<Event>) {
        let (tx, rx) = mpsc::channel();
        let worker = Worker::spawn(demo, move |event| tx.send(event).is_ok()).unwrap();
        (worker, rx)
    }
    fn ready(worker: &mut Worker, rx: &mpsc::Receiver<Event>) {
        assert!(matches!(
            rx.recv_timeout(TIMEOUT).unwrap(),
            Event::Ready(Ok(()))
        ));
        assert!(worker.ready(true));
    }

    #[test]
    fn only_one_job_runs_and_stale_completions_do_not_release_it() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let demo = Teaser::default().with_function(move |x| {
            started_tx.send(thread::current().id()).unwrap();
            release_rx.recv_timeout(TIMEOUT).unwrap();
            x[0]
        });
        let (mut worker, events) = launch(demo);
        ready(&mut worker, &events);
        assert!(worker.submit(job(1, "2")).unwrap());
        assert_ne!(
            started_rx.recv_timeout(TIMEOUT).unwrap(),
            thread::current().id()
        );
        assert!(!worker.submit(job(1, "3")).unwrap());
        assert!(!worker.submit(job(2, "3")).unwrap());
        assert!(!worker.finish(99, false));
        release_tx.send(()).unwrap();
        assert!(matches!(
            events.recv_timeout(TIMEOUT).unwrap(),
            Event::Finished {
                request_id: 1,
                response: Response::Success { .. },
                fatal: false
            }
        ));
        // Completion remains in flight until the UI acknowledges it.
        assert!(!worker.submit(job(2, "3")).unwrap());
        assert!(worker.finish(1, false));
        assert!(!worker.submit(job(1, "3")).unwrap());
        assert!(worker.submit(job(2, "4")).unwrap());
        started_rx.recv_timeout(TIMEOUT).unwrap();
        release_tx.send(()).unwrap();
        assert!(matches!(
            events.recv_timeout(TIMEOUT).unwrap(),
            Event::Finished { request_id: 2, .. }
        ));
        assert!(worker.finish(2, false));
    }

    #[test]
    fn non_send_model_is_created_once_on_the_worker_and_errors_are_recoverable() {
        let (factory_tx, factory_rx) = mpsc::channel();
        let demo = Teaser::default().with_predictor_factory(move || {
            factory_tx.send(thread::current().id()).unwrap();
            let count = Rc::new(Cell::new(0));
            Ok(move |values: &[crate::Value]| {
                count.set(count.get() + 1);
                if values[0].as_number()? < 0.0 {
                    return Err("Try a positive number.".into());
                }
                Ok(vec![crate::Value::Number(f64::from(count.get()))])
            })
        });
        let (mut worker, events) = launch(demo);
        ready(&mut worker, &events);
        assert_ne!(
            factory_rx.recv_timeout(TIMEOUT).unwrap(),
            thread::current().id()
        );
        for (id, value) in [(1, "-1"), (2, "2")] {
            assert!(worker.submit(job(id, value)).unwrap());
            match events.recv_timeout(TIMEOUT).unwrap() {
                Event::Finished {
                    request_id,
                    response,
                    fatal: false,
                } => {
                    assert_eq!(id, request_id);
                    if id == 1 {
                        assert!(matches!(response, Response::Error { .. }));
                    } else {
                        assert!(
                            matches!(response, Response::Success { values } if values == ["2.00"])
                        );
                    }
                    assert!(worker.finish(id, false));
                }
                other => panic!("{other:?}"),
            }
        }
        assert!(factory_rx.try_recv().is_err());
    }

    #[test]
    fn closing_during_inference_does_not_wait_or_deliver_a_result() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let demo = Teaser::default().with_function(move |x| {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(TIMEOUT).unwrap();
            done_tx.send(()).unwrap();
            x[0]
        });
        let (mut worker, events) = launch(demo);
        ready(&mut worker, &events);
        worker.submit(job(1, "2")).unwrap();
        started_rx.recv_timeout(TIMEOUT).unwrap();
        worker.close(); // Would deadlock if close tried to join the callback.
        assert!(!worker.finish(1, false));
        assert!(!worker.submit(job(2, "3")).unwrap());
        release_tx.send(()).unwrap();
        done_rx.recv_timeout(TIMEOUT).unwrap();
        assert!(matches!(
            events.recv_timeout(TIMEOUT),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn initialization_failures_and_panics_retire_the_worker() {
        let (mut worker, events) = launch(Teaser::default().with_predictor_factory(|| {
            Err::<fn(&[crate::Value]) -> Result<Vec<crate::Value>, String>, _>(
                "Missing model.".into(),
            )
        }));
        assert!(
            matches!(events.recv_timeout(TIMEOUT).unwrap(), Event::Ready(Err(message)) if message.contains("Missing model"))
        );
        assert!(worker.ready(false));
        assert!(!worker.submit(job(1, "1")).unwrap());
        let (mut worker, events) = launch(Teaser::default().with_function(|_| panic!("bad model")));
        ready(&mut worker, &events);
        worker.submit(job(1, "1")).unwrap();
        assert!(matches!(
            events.recv_timeout(TIMEOUT).unwrap(),
            Event::Finished { fatal: true, .. }
        ));
        assert!(worker.finish(1, true));
        assert!(!worker.submit(job(2, "1")).unwrap());
    }

    #[test]
    fn closing_during_preparation_suppresses_readiness() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let demo = Teaser::default().with_predictor_factory(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(TIMEOUT).unwrap();
            Ok(|values: &[crate::Value]| Ok(values.to_vec()))
        });
        let (mut worker, events) = launch(demo);
        started_rx.recv_timeout(TIMEOUT).unwrap();
        worker.close();
        release_tx.send(()).unwrap();
        assert!(matches!(
            events.recv_timeout(TIMEOUT),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert!(!worker.ready(true));
    }

    #[test]
    fn invalid_envelopes_are_rejected() {
        for message in [
            "{}",
            r#"{"request_id":0,"inputs":[]}"#,
            r#"{"request_id":9007199254740992,"inputs":[]}"#,
            r#"{"request_id":1,"inputs":[1]}"#,
            r#"{"request_id":1,"inputs":[],"extra":1}"#,
        ] {
            assert!(Submission::parse(message).is_err());
        }
    }
}

use crate::{
    bridge::Response,
    html,
    worker::{Event as WorkerEvent, Submission, Worker},
    Error, Teaser,
};
use std::{cell::RefCell, rc::Rc};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{WebView, WebViewBuilder};

fn display(webview: &WebView, script: &str) {
    if let Err(error) = webview.evaluate_script(script) {
        eprintln!("Could not update the demo: {error}");
    }
}

pub(crate) fn run(demo: Teaser) -> Result<(), Error> {
    let event_loop = EventLoopBuilder::<WorkerEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title(&demo.title)
        .with_inner_size(LogicalSize::new(1100.0, 760.0))
        .with_min_inner_size(LogicalSize::new(360.0, 400.0))
        .build(&event_loop)
        .map_err(|error| Error::Desktop(error.to_string()))?;
    let html = html::render(&demo);
    let worker_proxy = proxy.clone();
    let worker = Rc::new(RefCell::new(
        Worker::spawn(demo, move |event| worker_proxy.send_event(event).is_ok())
            .map_err(|error| Error::Desktop(error.to_string()))?,
    ));
    let ipc_worker = Rc::clone(&worker);
    let builder = WebViewBuilder::new()
        .with_html(html)
        .with_ipc_handler(move |request| {
            if request.body() == "ready" {
                let _ = proxy.send_event(WorkerEvent::PageReady);
                return;
            }
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Export {
                export_csv: String,
            }
            if let Ok(export) = serde_json::from_str::<Export>(request.body()) {
                if export.export_csv.len() <= 6 * 1024 * 1024 {
                    let _ = proxy.send_event(WorkerEvent::SaveCsv(export.export_csv));
                }
                return;
            }
            match Submission::parse(request.body()) {
                Ok(job) => {
                    let request_id = job.request_id;
                    if let Err(message) = ipc_worker.borrow_mut().submit(job) {
                        let _ = proxy.send_event(WorkerEvent::Finished {
                            request_id,
                            response: Response::Error {
                                field: None,
                                message,
                            },
                            fatal: true,
                        });
                    }
                }
                Err(message) => {
                    let _ = proxy.send_event(WorkerEvent::ProtocolError(message));
                }
            }
        });
    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let container = window
            .default_vbox()
            .ok_or_else(|| Error::Desktop("Window has no GTK container.".into()))?;
        builder.build_gtk(container)
    };
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window);
    let webview = webview.map_err(|error| Error::Desktop(error.to_string()))?;
    let mut page_ready = false;
    let mut initialization: Option<Result<(), String>> = None;
    event_loop.run(move |event, _, control_flow| {
        let _ = &window;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(WorkerEvent::SaveCsv(csv)) => {
                let message = match rfd::FileDialog::new()
                    .set_parent(&window)
                    .set_title("Export predictions")
                    .add_filter("CSV", &["csv"])
                    .set_file_name("tease-results.csv")
                    .save_file()
                {
                    Some(path) => match std::fs::write(&path, csv) {
                        Ok(()) => format!("CSV saved to {}", path.display()),
                        Err(error) => format!("Could not save CSV: {error}"),
                    },
                    None => "CSV export canceled.".into(),
                };
                let message = serde_json::to_string(&message).expect("string serialization");
                display(
                    &webview,
                    &format!("window.tease.downloadFinished({message});"),
                );
            }
            Event::UserEvent(WorkerEvent::PageReady) if !page_ready => {
                page_ready = true;
                if let Some(result) = &initialization {
                    let error = serde_json::to_string(&result.as_ref().err())
                        .expect("string serialization");
                    display(&webview, &format!("window.tease.ready({error});"));
                }
            }
            Event::UserEvent(WorkerEvent::Ready(result)) => {
                if worker.borrow_mut().ready(result.is_ok()) {
                    if page_ready {
                        let error = serde_json::to_string(&result.as_ref().err())
                            .expect("string serialization");
                        display(&webview, &format!("window.tease.ready({error});"));
                    }
                    initialization = Some(result);
                }
            }
            Event::UserEvent(WorkerEvent::Finished {
                request_id,
                response,
                fatal,
            }) => {
                if worker.borrow_mut().finish(request_id, fatal) {
                    display(&webview, &response.script(Some(request_id), fatal));
                }
            }
            Event::UserEvent(WorkerEvent::ProtocolError(message)) => {
                if worker.borrow().is_idle() {
                    display(
                        &webview,
                        &Response::Error {
                            field: None,
                            message,
                        }
                        .script(None, false),
                    );
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                worker.borrow_mut().close();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

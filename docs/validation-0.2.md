# 0.2.0 implementation validation: issues 1–6

Validated on 2026-09-19 from the local `v0.2.0` working tree, on macOS ARM64.
This records local implementation evidence, not a published release or GitHub CI results.
The changes are uncommitted and unpushed.

## Implemented scope

- #1: Wry 0.57 / Tao 0.37, published AutoML 0.3.1 and matching SmartCore 0.4.2,
  a development lockfile, Rust 1.88 baseline, and six CI jobs covering two Rust
  versions on Windows, macOS, and Linux.
- #2: separate component definitions and runtime values; a shared validated
  pipeline for numerical and typed/fallible callbacks; labeled choices and
  multiple numerical/text results.
- #3: structured messages, configuration/input/output validation, escaped text,
  and recoverable field/model errors.
- #4: dedicated inference worker, single in-flight request, request identifiers,
  disabled controls and progress status, stale-response rejection, and closure
  without joining user code. Initialization and unwinding prediction panics
  produce terminal errors; ordinary model errors permit resubmission.
- #5: real SmartCore regression and Iris classification, real published AutoML
  regression, and division/slow-model demonstrations with checked sample results.
- #6: API/migration/example documentation, changelog, matching license texts,
  an explicit package file list, isolated packaged-consumer verification,
  publish dry run, and a release/platform/distribution checklist.

`with_predictor_factory` creates non-Send models on the worker itself. For headless
use, `Teaser::prepare()` creates a reusable `Session` on the calling thread;
`Session::predict()` is synchronous. The migration guide explains the new ownership
requirements and shutdown behavior. No unsafe Send/Sync implementation is used.

## Automated results

| Check | Result |
| --- | --- |
| Rust 1.93.1: `cargo test --locked --all-targets` | 27 passed |
| Rust 1.88.0: `cargo test --locked --all-targets` | 27 passed |
| Doctests on both Rust versions | 9 passed each |
| Strict Clippy and rustdoc, both Rust versions | Passed |
| Rustfmt, workflow actionlint, whitespace checks | Passed |
| Node UI lifecycle tests | 5 passed |
| Rust 1.88 Windows MSVC target: `cargo check --locked --all-targets --target x86_64-pc-windows-msvc` | Passed from macOS; type checking only |
| `python3 scripts/check-package.py` | Passed: 32 archive files, isolated consumer builds and runs |
| `cargo publish --locked --dry-run --allow-dirty` | Passed; upload aborted by dry-run mode |

The Rust tests include 10 library tests, 11 public-API integration tests, and six
example tests. Worker tests use explicit synchronization to check thread ownership,
one-time non-Send initialization, duplicate/stale requests, recovery, panic retirement,
and closure during preparation or inference. The UI tests execute the shipped
JavaScript with a minimal DOM harness; they are not native-browser tests.

The package check inspects license/docs/source inclusion, excludes obsolete assets,
unpacks the archive outside the checkout, and builds/runs a new consumer against
that unpacked crate. The consumer checks numerical and typed callbacks, non-Send
factory initialization, and recovery after an error. Dependency build artifacts may
be cached; the consumer does not refer to the source checkout.

## Native macOS smoke checks

The final worker-based examples were tested in native Wry windows using temporary
app bundles:

- Iris: all three documented samples displayed their expected species.
- AutoML: the default sample displayed `206.12`.
- Division: zero displayed a model error; a subsequent blank divisor displayed
  a required-field error and focused that field. Correcting it to 4 and pressing
  Enter displayed `2.500` and `Calculated`, preserving numerator 10.
- Slow model: value 3 displayed `9.00` after a five-second delay. A subsequent
  window-close action during a pending prediction returned promptly; process
  termination was confirmed. Worker tests separately verify closure does not wait
  for the callback and suppresses late delivery.

Earlier checks on the first implementation batch also verified the Longley default
(`60.056`), its changed-slider result (`59.113`), typed dropdown results, range errors,
and the initial two-panel layout. Longley's default and the typed callback behavior
were rechecked by automated tests after moving inference to the worker.

## Pending evidence and release actions

Native Windows and Linux builds/tests/runtime checks remain pending. The Windows
cross-check does not establish native linking or WebView2 runtime behavior. No Linux
build ran because a running Linux environment was unavailable. The six CI jobs are
configured but have not run on this unpushed branch.

The crate is a local release candidate. Commit/push, remote CI, remaining native
platform checks, merge, tagging, publication, and verification of crates.io/docs.rs
are still pending. The final clean release commit must pass the package checks again
without `--allow-dirty`. Bare example binaries are not installers; signing and clean
target-machine distribution checks have not been performed. Plots and parameter
sweeps remain outside the 0.2.0 scope.

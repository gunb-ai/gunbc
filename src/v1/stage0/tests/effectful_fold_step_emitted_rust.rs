//! A fold whose step reaches an effectful callee must compile and run at the emitted-Rust
//! boundary. Before the effectful-step loop realization (v1.compiler.emit_rust
//! emit_rust_effectful_fold_loop) the step was handed to `Iterator::fold` as a synchronous
//! closure and the callee's `.await?` landed inside it: rustc E0728. Run with
//! cargo test -p v1-compiler --test effectful_fold_step_emitted_rust. Like its siblings this
//! target is compiled by all-target clippy and not executed by required CI
//! (gunbc.rung_drop rust_unit_tests_off_the_merge_path).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

// The step calls `scaled`, which reads its resource and is therefore effectful; the record
// accumulator mirrors gunbc.auth.approval_decision_store pending_escalations. The pure fold is
// the control that the closure realization is kept where the step is pure.
const SOURCE: &str = r#"module effectful_fold_probe

type FoldNet { base: Int }

type FoldTally { sum: Int, seen: Int }

fn scaled(n: Int) -> Int
  uses net: FoldNet
{
  net.base * n
}

fn tally(xs: List<Int>) -> FoldTally
  uses net: FoldNet
{
  fold(xs, init: FoldTally { sum: 0, seen: 0 }, f: (acc, x) => FoldTally { sum: acc.sum + scaled(n: x), seen: acc.seen + 1 })
}

fn pure_total(xs: List<Int>) -> Int {
  fold(xs, init: 0, f: (acc, x) => acc + x)
}
"#;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn unique_rlib(deps: &Path, name: &str) -> PathBuf {
    let prefix = format!("lib{name}-");
    let matches: Vec<_> = std::fs::read_dir(deps)
        .expect("read test dependency directory")
        .map(|entry| entry.expect("read dependency entry").path())
        .filter(|path| {
            let filename = path.file_name().unwrap().to_string_lossy();
            filename.starts_with(&prefix) && filename.ends_with(".rlib")
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "ambiguous or absent {name} artifact: {matches:?}"
    );
    matches[0].clone()
}

#[test]
fn an_effectful_fold_step_emits_buildable_and_correct_rust() {
    let result = compile_sources(
        Rc::new(
            vec![Rc::new(SourceFile {
                path: "effectful_fold_probe.dag".into(),
                content: SOURCE.into(),
            })]
            .into(),
        ),
        RenderTarget::Rust,
    );
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/effectful_fold_probe.rs")
        .collect();
    assert_eq!(
        modules.len(),
        1,
        "emission did not produce the specimen: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    let emitted = modules[0].content.to_string();
    assert!(
        emitted.contains("for __fold_elem in"),
        "the effectful step was not realized as a loop:\n{emitted}"
    );
    assert!(
        emitted.contains(".fold("),
        "the pure fold lost its closure realization:\n{emitted}"
    );
    let root = std::env::temp_dir().join(format!("gunbc-effectful-fold-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(
        scratch.0.join("effectful_fold_probe.rs"),
        &modules[0].content,
    )
    .expect("retain emitted bytes unchanged");
    // The emitted futures never suspend (the only effect is a resource field read), so one
    // poll with a no-op waker drives them to completion; a Pending is a harness refusal.
    std::fs::write(
        scratch.0.join("main.rs"),
        r#"pub use v1_compiler::*;
mod effectful_fold_probe;
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match f.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => v,
        std::task::Poll::Pending => panic!("emitted future suspended with no executor"),
    }
}
fn main() {
    use std::rc::Rc;
    use effectful_fold_probe::{FoldNet, pure_total, tally};
    let net = FoldNet { base: 10 };
    let xs = Rc::new(im::vector![1, 2, 3]);
    let t = block_on(tally(xs.clone(), &net)).expect("effectful fold refused");
    assert_eq!(t.sum, 60);
    assert_eq!(t.seen, 3);
    let empty = block_on(tally(Rc::new(im::vector![]), &net)).expect("empty fold refused");
    assert_eq!(empty.sum, 0);
    assert_eq!(empty.seen, 0);
    assert_eq!(pure_total(xs), 6);
}
"#,
    )
    .expect("write harness around emitted module");
    let executable = std::env::current_exe().expect("locate test executable");
    let deps = executable.parent().expect("test dependency directory");
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(scratch.0.join("main.rs"))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!(
            "v1_compiler={}",
            unique_rlib(deps, "v1_compiler").display()
        ))
        .arg("--extern")
        .arg(format!("im={}", unique_rlib(deps, "im").display()))
        .arg("--extern")
        .arg(format!("serde={}", unique_rlib(deps, "serde").display()))
        .arg("-o")
        .arg(scratch.0.join("specimen"))
        .output()
        .expect("invoke rustc on emitted specimen");
    assert!(
        output.status.success(),
        "emitted Rust refused: {}\n--- emitted module ---\n{emitted}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen");
    assert!(
        run.status.success(),
        "effectful fold behavior failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

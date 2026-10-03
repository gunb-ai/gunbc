//! A collection call taking a lambda is legal in a branch condition exactly as it is in value
//! position, and must emit, compile and run there. The emitter once refused every module with a
//! `filter` in an `if` condition (FilterInBranchCondition) on the premise that the guarded branch
//! was dropped; executing the projection showed the guard and both branches render intact, so the
//! wall refused correct code -- the approval broker's receipt parser among it. The specimen is that
//! shape in four spellings: pipe, free call, a filter inside a nested if used as the condition, and
//! the broker's own `count(filter(xs |> skip(n: 2), ...))` under `||`. Each function is exercised
//! on both arms, so a dropped or inverted branch goes red.
//! Run with cargo test -p v1-compiler --test filter_in_branch_condition_emitted_rust. Like its
//! sibling named_filter_emitted_rust, it is enrolled as an integration target compiled by
//! all-target clippy; required CI does not execute Rust integration tests.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module filter_guard_probe

fn pipe_guard(xs: List<Int>) -> Int {
  if (xs |> filter(x => x > 0) |> count) > 0 { 1 } else { 0 }
}

fn call_guard(xs: List<Int>) -> Int {
  if (filter(xs, x => x > 0) |> count) > 0 { 1 } else { 0 }
}

fn nested_guard(xs: List<Int>) -> Int {
  if (if (xs |> filter(x => x > 0) |> count) > 0 { true } else { false }) { 1 } else { 0 }
}

fn broker_guard(a: String, lines: List<String>) -> Int {
  if a == "" || count(filter(lines |> skip(n: 2), l => l != "")) != 0 { 0 } else { 1 }
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
fn filter_in_branch_condition_emits_buildable_and_correct_rust() {
    let result = compile_sources(
        Rc::new(
            vec![Rc::new(SourceFile {
                path: "filter_guard_probe.dag".into(),
                content: SOURCE.into(),
            })]
            .into(),
        ),
        RenderTarget::Rust,
    );
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/filter_guard_probe.rs")
        .collect();
    assert_eq!(
        modules.len(),
        1,
        "emission did not publish the specimen: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    let root = std::env::temp_dir().join(format!("gunbc-filter-guard-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(scratch.0.join("filter_guard_probe.rs"), &modules[0].content)
        .expect("retain emitted bytes unchanged");
    std::fs::write(
        scratch.0.join("main.rs"),
        r#"pub use v1_compiler::*;
mod filter_guard_probe;
fn main() {
    use std::rc::Rc;
    use filter_guard_probe::{broker_guard, call_guard, nested_guard, pipe_guard};
    assert_eq!(pipe_guard(Rc::new(im::vector![-1, 0, 3])), 1);
    assert_eq!(pipe_guard(Rc::new(im::vector![-1, 0])), 0);
    assert_eq!(call_guard(Rc::new(im::vector![5])), 1);
    assert_eq!(call_guard(Rc::new(im::vector![])), 0);
    assert_eq!(nested_guard(Rc::new(im::vector![2])), 1);
    assert_eq!(nested_guard(Rc::new(im::vector![0])), 0);
    let lines = |xs: &[&str]| Rc::new(xs.iter().map(|s| s.to_string()).collect::<im::Vector<String>>());
    assert_eq!(broker_guard("x".to_string(), lines(&["h", "", ""])), 1);
    assert_eq!(broker_guard("x".to_string(), lines(&["h", "", "", "y"])), 0);
    assert_eq!(broker_guard("".to_string(), lines(&["h", "", ""])), 0);
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
        "emitted Rust refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen");
    assert!(
        run.status.success(),
        "guard behavior failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

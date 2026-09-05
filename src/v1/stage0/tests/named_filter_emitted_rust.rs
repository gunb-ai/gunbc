//! Named predicates must compile at the emitted-Rust boundary, not merely resolve as .dag.
//! Run with cargo test -p v1-compiler --test named_filter_emitted_rust. This test remains
//! enrolled as an integration target; all-target clippy compiles the harness, but required CI
//! does not execute Rust integration tests. Direct execution is not scheduled coverage.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module named_filter_probe

type FilterProbeItem { value: Int }

fn keep_record(item: FilterProbeItem) -> Bool { item.value > 0 }
fn keep_scalar(item: Int) -> Bool { item > 0 }

fn records(items: List<FilterProbeItem>) -> List<FilterProbeItem> {
  items |> filter(keep_record)
}

fn scalars(items: List<Int>) -> List<Int> {
  items |> filter(keep_scalar)
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
fn named_filter_predicates_emit_buildable_and_correct_rust() {
    let result = compile_sources(
        Rc::new(
            vec![Rc::new(SourceFile {
                path: "named_filter_probe.dag".into(),
                content: SOURCE.into(),
            })]
            .into(),
        ),
        RenderTarget::Rust,
    );
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/named_filter_probe.rs")
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
    let root = std::env::temp_dir().join(format!("gunbc-named-filter-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(scratch.0.join("named_filter_probe.rs"), &modules[0].content)
        .expect("retain emitted bytes unchanged");
    std::fs::write(
        scratch.0.join("main.rs"),
        r#"pub use v1_compiler::*;
mod named_filter_probe;
fn main() {
    use std::rc::Rc;
    use named_filter_probe::{FilterProbeItem, records, scalars};
    let input = Rc::new(im::vector![
        Rc::new(FilterProbeItem { value: -1 }),
        Rc::new(FilterProbeItem { value: 2 }),
        Rc::new(FilterProbeItem { value: 0 }),
        Rc::new(FilterProbeItem { value: 3 }),
    ]);
    let output = records(input.clone());
    assert_eq!(output.iter().map(|item| item.value).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(input.len(), 4);
    assert_eq!(*scalars(Rc::new(im::vector![-1, 2, 0, 3])), im::vector![2, 3]);
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
        "filter behavior failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

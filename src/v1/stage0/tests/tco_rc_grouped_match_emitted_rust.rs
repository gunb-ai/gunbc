//! A tail-recursive match whose arms enumerate a nested variant under an Rc-shared field, with no
//! wildcard, must emit Rust that rustc accepts as exhaustive. The loop lowering used to render each
//! nested discrimination as a match guard, which rustc does not count toward exhaustiveness (E0004);
//! both lowerings now share v1.compiler.emit_rust rc_grouped_match_arm_strs
//! (gunbc.recurring_failure_mode tail_call_match_lowering_skips_rc_arm_grouping).
//! Run with cargo test -p v1-compiler --test tco_rc_grouped_match_emitted_rust. This test remains
//! enrolled as an integration target; all-target clippy compiles the harness, but required CI
//! does not execute Rust integration tests. Direct execution is not scheduled coverage.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module tco_rc_grouped_probe

type TcoProbeExpr =
  | TcoLit { value: Int }
  | TcoNeg { inner: TcoProbeExpr }
  | TcoPair { head: TcoProbeExpr, tail: TcoProbeExpr }

fn walk(e: TcoProbeExpr, acc: Int) -> Int {
  match e {
    TcoLit { value: v } => acc + v
    TcoPair { head: _, tail: t } => walk(e: t, acc: acc + 1)
    TcoNeg { inner: TcoLit { value: v } } => acc - v
    TcoNeg { inner: TcoNeg { inner: _ } } => acc + 100
    TcoNeg { inner: TcoPair { head: _, tail: _ } } => acc + 1000
  }
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
fn tail_call_match_groups_enumerated_nested_rc_arms() {
    let result = compile_sources(
        Rc::new(
            vec![Rc::new(SourceFile {
                path: "tco_rc_grouped_probe.dag".into(),
                content: SOURCE.into(),
            })]
            .into(),
        ),
        RenderTarget::Rust,
    );
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/tco_rc_grouped_probe.rs")
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
    let emitted = &modules[0].content;
    assert!(
        emitted.contains("loop"),
        "the specimen must exercise the tail-call lowering, not the value lowering:\n{emitted}"
    );
    let root = std::env::temp_dir().join(format!("gunbc-tco-rc-grouped-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(scratch.0.join("tco_rc_grouped_probe.rs"), emitted)
        .expect("retain emitted bytes unchanged");
    std::fs::write(
        scratch.0.join("main.rs"),
        r#"pub use v1_compiler::*;
mod tco_rc_grouped_probe;
fn main() {
    use std::rc::Rc;
    use tco_rc_grouped_probe::{walk, TcoProbeExpr};
    let lit = |v: i64| Rc::new(TcoProbeExpr::TcoLit { value: v });
    let neg = |e: Rc<TcoProbeExpr>| Rc::new(TcoProbeExpr::TcoNeg { inner: e });
    let pair = |h: Rc<TcoProbeExpr>, t: Rc<TcoProbeExpr>| Rc::new(TcoProbeExpr::TcoPair { head: h, tail: t });
    assert_eq!(walk(neg(lit(3)), 10), 7);
    assert_eq!(walk(neg(neg(lit(3))), 0), 100);
    assert_eq!(walk(neg(pair(lit(1), lit(2))), 0), 1000);
    assert_eq!(walk(pair(lit(1), pair(lit(9), neg(lit(5)))), 0), -3);
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
        "emitted Rust refused: {}\n{emitted}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen");
    assert!(
        run.status.success(),
        "grouped tail-call behavior failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

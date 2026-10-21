//! A NULLARY function passed as a VALUE must emit to Rust that compiles and computes what the
//! interpreter computes (DESIGN section 7: emitted is behaviourally equivalent to the seed).
//! The emitter decided "is this parameter an arrow" by counting the arrow's parameters, so
//! `f: fn() -> Int` fell to the value renderer as `Rc<dyn Fn() -> i64>` while the named value
//! `seven` arrived bare -- rustc E0308. Run with
//! cargo test -p v1-compiler --test nullary_fn_value_emitted_rust. Like its sibling
//! named_filter_emitted_rust, this is enrolled as an integration target that all-target clippy
//! compiles; required CI does not execute Rust integration tests.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module nullary_fn_value_probe

fn seven() -> Int { 7 }
fn eleven(x: Int) -> Int { x + 4 }

fn apply0(f: fn() -> Int) -> Int { f() }
fn apply1(f: fn(Int) -> Int, x: Int) -> Int { f(x) }

fn probe() -> Int { apply0(f: seven) * 100 + apply1(f: eleven, x: 3) }
"#;

fn source() -> Rc<im::Vector<Rc<SourceFile>>> {
    Rc::new(
        vec![Rc::new(SourceFile {
            path: "nullary_fn_value_probe.dag".into(),
            content: SOURCE.into(),
        })]
        .into(),
    )
}

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
fn nullary_fn_value_emits_buildable_rust_equal_to_the_interpreter() {
    let resolved = compile_to_resolved(source());
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let interpreted = match v1_interpreter::run(graph, resolved.source_indices.clone(), "probe") {
        Ok(Value::Int(n)) => n,
        other => panic!("interpreter did not produce an Int: {other:?}"),
    };
    assert_eq!(interpreted, 707, "interpreter control");

    let result = compile_sources(source(), RenderTarget::Rust);
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/nullary_fn_value_probe.rs")
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
    let root = std::env::temp_dir().join(format!("gunbc-nullary-fn-value-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(
        scratch.0.join("nullary_fn_value_probe.rs"),
        &modules[0].content,
    )
    .expect("retain emitted bytes unchanged");
    std::fs::write(
        scratch.0.join("main.rs"),
        "pub use v1_compiler::*;\nmod nullary_fn_value_probe;\nfn main() { print!(\"{}\", nullary_fn_value_probe::probe()); }\n",
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
        "emitted Rust refused:\n{}\n--- emitted ---\n{}",
        String::from_utf8_lossy(&output.stderr),
        modules[0].content
    );
    let run = Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen");
    assert!(
        run.status.success(),
        "specimen failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        interpreted.to_string(),
        "emitted result diverges from the interpreter"
    );
}

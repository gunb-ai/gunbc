//! A String ORDERING over an alias of String, and a BINDING ARM after an unguarded absent arm,
//! must each emit to Rust that compiles and computes what the interpreter computes (DESIGN
//! section 7). Before the repair the first refused at emission as an operand "whose declaration
//! could not be read" (compile_error!, the corpus String reaching the ordering operators with no
//! host-text-carrier arm in v1.compiler.emit_rust rust_operand_realization_of_type), and the second
//! bound the whole Option in `o => o` (rustc E0308), in ordinary matches and in the tail-call
//! lowering's own match rendering alike. Found on the native App Attest closure
//! (extdeps.standards.rfc_5280 x509_validity_at; x690_der der_unsigned_integer). The ordering rows
//! discriminate byte-lexicographic UTF-8 order, the interpreter's (Str, Str) arm: "Z" < "a" and
//! NOT "é" < "z". Run with cargo test -p v1-compiler --test string_order_present_binding_emitted_rust.
//! Like its siblings this is an integration target that all-target clippy compiles; required CI
//! does not execute Rust integration tests.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module order_binding_probe

type Stamp = String

fn before(a: Stamp, b: Stamp) -> Bool { a < b }
fn bit(b: Bool) -> Int { if b { 1 } else { 0 } }

fn first_or_zero(xs: List<Int>) -> Int {
  match xs |> first {
    null => 0
    o => o
  }
}

fn total_from(xs: List<Int>, i: Int, acc: Int) -> Int {
  match xs |> get(i) {
    null => acc
    x => total_from(xs: xs, i: i + 1, acc: acc + x)
  }
}

fn probe() -> Int {
  total_from(xs: [97, 5], i: 0, acc: 0) * 100000000
    + first_or_zero(xs: [97, 5]) * 100000
    + first_or_zero(xs: []) * 10000
    + bit(b: before(a: "2026-01-02T00:00:00Z", b: "2026-01-10T00:00:00Z")) * 1000
    + bit(b: before(a: "b", b: "a")) * 100
    + bit(b: before(a: "Z", b: "a")) * 10
    + bit(b: before(a: "é", b: "z"))
}
"#;

fn source() -> Rc<im::Vector<Rc<SourceFile>>> {
    Rc::new(
        vec![Rc::new(SourceFile {
            path: "order_binding_probe.dag".into(),
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
fn string_order_and_present_binding_emit_buildable_rust_equal_to_the_interpreter() {
    let resolved = compile_to_resolved(source());
    let graph = resolved.graph.as_ref().expect("resolved graph");
    let interpreted = match v1_interpreter::run(graph, resolved.source_indices.clone(), "probe") {
        Ok(Value::Int(n)) => n,
        other => panic!("interpreter did not produce an Int: {other:?}"),
    };
    assert_eq!(interpreted, 10209701010, "interpreter control");

    let result = compile_sources(source(), RenderTarget::Rust);
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/order_binding_probe.rs")
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
    let root = std::env::temp_dir().join(format!("gunbc-order-binding-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(
        scratch.0.join("order_binding_probe.rs"),
        &modules[0].content,
    )
    .expect("retain emitted bytes unchanged");
    std::fs::write(
        scratch.0.join("main.rs"),
        "pub use v1_compiler::*;\nmod order_binding_probe;\nfn main() { print!(\"{}\", order_binding_probe::probe()); }\n",
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

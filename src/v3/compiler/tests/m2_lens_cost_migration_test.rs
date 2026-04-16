use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, Dag, PortId};
use v3_compiler::emit_rust::emit_rust_module;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);
const GENERATED_LENS_HEADER: &str = "// AUTO-GENERATED from `src/v3/lenses/complexity.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("complexity.dag")
}

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read complexity.dag")
}

fn emit_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled complexity.dag");
    assert!(
        dag.diagnostics().is_empty(),
        "complexity.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../src/lens_cost_generated.rs")
}

fn format_rust_source(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .expect("write source to rustfmt");
    let output = child.wait_with_output().expect("wait for rustfmt");
    assert!(
        output.status.success(),
        "rustfmt failed on emitted lens module"
    );
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

fn deps_dir() -> PathBuf {
    std::env::current_exe()
        .expect("current test binary path")
        .parent()
        .expect("deps dir")
        .to_path_buf()
}

fn find_current_rlib(crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(deps_dir())
        .expect("read deps dir")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.starts_with(&prefix) && file_name.ends_with(".rlib") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    matches.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });
    matches
        .into_iter()
        .last()
        .expect("compiled rlib for current crate")
}

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_cost_lens_migration_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
}

fn compile_with_current_crate(src_path: &Path, bin_path: &Path) {
    let deps = deps_dir();
    let current_rlib = find_current_rlib("v3_compiler");
    let compile = Command::new("rustc")
        .arg("--edition=2021")
        .arg(src_path)
        .arg("-o")
        .arg(bin_path)
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!("v3_compiler={}", current_rlib.display()))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("invoke rustc");
    assert!(compile.success(), "rustc failed on emitted lens source");
}

fn roundtrip_cost(
    module_source: &str,
    program_source: &str,
    file_name: &str,
    bind_name: &str,
) -> usize {
    let wrapped = format!(
        "mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} }} \
         fn main() {{ \
           let dag = v3_compiler::compile_to_dag({program_source:?}, {file_name:?}).expect(\"compiles\"); \
           let bind = dag.nodes().iter().find_map(|node| match node {{ \
             v3_compiler::dag::Behavior::Bind(bind) if bind.name == {bind_name:?} => Some(bind.clone()), \
             _ => None \
           }}).expect(\"bind\"); \
           println!(\"{{}}\", emitted::cost_of(&dag, &bind.value)); \
         }}"
    );

    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.rs");
    let bin_path = tmp_dir.join("main_bin");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(wrapped.as_bytes()))
        .expect("write wrapped rust source");

    compile_with_current_crate(&src_path, &bin_path);

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled binary");
    assert!(run.status.success(), "compiled binary failed");
    String::from_utf8_lossy(&run.stdout)
        .trim()
        .parse()
        .expect("printed cost should be usize")
}

fn bind_value(dag: &Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn handwritten_cost(dag: &Dag, port: PortId) -> usize {
    match dag.port(port).produced_by {
        None => 0,
        Some(node_id) => match dag.node(node_id) {
            Behavior::Value(_) => 0,
            Behavior::Transform(t) => {
                1 + t
                    .inputs
                    .iter()
                    .map(|&input| handwritten_cost(dag, input))
                    .sum::<usize>()
            }
            Behavior::Branch(branch) => {
                let cond = handwritten_cost(dag, branch.input);
                let paths = branch
                    .paths
                    .iter()
                    .map(|path| handwritten_cost(dag, path.output))
                    .max()
                    .unwrap_or(0);
                1 + cond + paths
            }
            Behavior::Loop(loop_node) => {
                1 + handwritten_cost(dag, loop_node.source) + handwritten_cost(dag, loop_node.init)
            }
            Behavior::Bind(bind) => handwritten_cost(dag, bind.value),
        },
    }
}

fn handwritten_bind_cost(program_source: &str, file_name: &str, bind_name: &str) -> usize {
    let dag = compile_to_dag(program_source, file_name).expect("fixture compiles");
    handwritten_cost(&dag, bind_value(&dag, bind_name))
}

#[test]
fn complexity_dag_compiles_cleanly() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("complexity.dag should compile cleanly");
    assert!(
        dag.diagnostics().is_empty(),
        "complexity.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn complexity_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate lens_cost_generated.rs from complexity.dag"
    );
}

#[test]
fn complexity_dag_matches_handwritten_oracle_on_core_fixtures() {
    let module = emit_lens_module();
    let fixtures = [
        ("let x = 1", "literal.v3", "x"),
        ("let x = 1 + 2 + 3", "chained_transform.v3", "x"),
        (
            "let r = if 1 > 0 then 20 + 30 else 40 + 50 + 60",
            "branch_max.v3",
            "r",
        ),
        (
            "\
fn countdown(n: Int) -> Int =
  if n == 0 then 0 else countdown(n - 1)
",
            "recursive_fn.v3",
            "countdown",
        ),
        (
            "let total: Int = fold(map(singleton(1), |x| x + 1), 0, |acc, x| acc + x)",
            "nested_fold.v3",
            "total",
        ),
    ];

    for (source, file_name, bind_name) in fixtures {
        let expected = handwritten_bind_cost(source, file_name, bind_name);
        let actual = roundtrip_cost(&module, source, file_name, bind_name);
        assert_eq!(
            actual, expected,
            "compiled complexity.dag should match handwritten oracle on {file_name}"
        );
    }
}

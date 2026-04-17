use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, BindNode, NodeId, PortId};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::Dag;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);
const GENERATED_LENS_HEADER: &str =
    "// AUTO-GENERATED from `src/v3/lenses/unused_parameters.dag` via\n\
     // `emit_rust_module`. Regenerate instead of hand-editing.\n\n";

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read unused_parameters.dag")
}

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("unused_parameters.dag")
}

fn emit_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled lens source");
    assert!(
        dag.diagnostics().is_empty(),
        "unused_parameters.dag should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag).expect("emit compiled lens module");
    format_rust_source(&format!("{GENERATED_LENS_HEADER}{raw}"))
}

fn checked_in_generated_module() -> &'static str {
    include_str!("../src/lens_unused_parameters_generated.rs")
}

fn clone_call_count(source: &str) -> usize {
    source.match_indices(".clone(").count()
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

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_lens_migration_roundtrip_{}_{}",
        std::process::id(),
        id
    ))
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

fn compile_with_current_crate(src_path: &Path, bin_path: &Path) {
    let deps = deps_dir();
    let current_rlib = find_current_rlib("v3_compiler");
    let compile = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-D")
        .arg("warnings")
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

/// Compile the unused-parameters roundtrip harness once. `main` reads
/// `program_source` + `file_name` from argv so each fixture is a
/// single process spawn rather than a fresh rustc invocation.
fn build_roundtrip_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        "#[allow(warnings, clippy::all)] \
         mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} }} \
         fn render(dag: &v3_compiler::Dag, function: v3_compiler::dag::NodeId) -> String {{ \
           dag.nodes().iter().find_map(|node| match node {{ \
             v3_compiler::dag::Behavior::Bind(bind) if bind.id == function => Some(bind.name.clone()), \
             _ => None \
           }}).unwrap_or_else(|| format!(\"{{:?}}\", function)) \
         }} \
         fn main() {{ \
           let mut __args = std::env::args(); __args.next(); \
           let program_source = __args.next().expect(\"program_source arg\"); \
           let file_name = __args.next().expect(\"file_name arg\"); \
           let dag = v3_compiler::compile_to_dag(&program_source, &file_name).expect(\"compiles\"); \
           let mut rendered: Vec<String> = emitted::check(&dag).iter().map(|v| {{ \
             format!(\"{{}}:param[{{}}]\", render(&dag, v.function), v.parameter_index) \
           }}).collect(); \
           rendered.sort(); \
           println!(\"{{}}\", rendered.join(\"|\")); \
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
    bin_path
}

fn roundtrip_lens_render(bin_path: &Path, program_source: &str, file_name: &str) -> String {
    let run = Command::new(bin_path)
        .arg(program_source)
        .arg(file_name)
        .output()
        .expect("run compiled binary");
    assert!(
        run.status.success(),
        "compiled binary failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

#[derive(Debug, Clone)]
struct OracleUnusedParameter {
    function: NodeId,
    parameter_index: usize,
}

fn render_handwritten_oracle(program_source: &str, file_name: &str) -> String {
    let dag = compile_to_dag(program_source, file_name).expect("program compiles");
    render_handwritten_oracle_on_dag(&dag)
}

fn render_handwritten_oracle_on_dag(dag: &Dag) -> String {
    let mut rendered: Vec<String> = handwritten_unused_parameters(dag)
        .iter()
        .map(|violation| {
            let function_name = dag
                .nodes()
                .iter()
                .find_map(|node| match node {
                    v3_compiler::dag::Behavior::Bind(bind) if bind.id == violation.function => {
                        Some(bind.name.clone())
                    }
                    _ => None,
                })
                .unwrap_or_else(|| format!("{:?}", violation.function));
            format!("{function_name}:param[{}]", violation.parameter_index)
        })
        .collect();
    rendered.sort();
    rendered.join("|")
}

fn handwritten_unused_parameters(dag: &Dag) -> Vec<OracleUnusedParameter> {
    let mut violations = Vec::new();
    for node in dag.nodes() {
        let Behavior::Bind(bind) = node else {
            continue;
        };
        if bind.params.is_empty() {
            continue;
        }
        collect_unused_params(dag, bind, &mut violations);
    }
    violations
}

fn collect_unused_params(dag: &Dag, bind: &BindNode, out: &mut Vec<OracleUnusedParameter>) {
    let referenced = collect_referenced_ports(dag, bind.value);

    for (idx, &param_port) in bind.params.iter().enumerate() {
        if !referenced.contains(&param_port) {
            out.push(OracleUnusedParameter {
                function: bind.id,
                parameter_index: idx,
            });
        }
    }
}

fn collect_referenced_ports(dag: &Dag, root_port: PortId) -> HashSet<PortId> {
    let mut referenced: HashSet<PortId> = HashSet::new();
    let mut queue: Vec<PortId> = vec![root_port];

    while let Some(port) = queue.pop() {
        if !referenced.insert(port) {
            continue;
        }
        let Some(producer) = dag.port(port).produced_by else {
            continue;
        };
        match dag.node(producer) {
            Behavior::Value(_) => {}
            Behavior::Transform(t) => {
                for &input in &t.inputs {
                    queue.push(input);
                }
            }
            Behavior::Branch(b) => {
                queue.push(b.input);
                for path in &b.paths {
                    queue.push(path.output);
                }
            }
            Behavior::Loop(l) => {
                queue.push(l.source);
                queue.push(l.init);
                queue.push(l.bound.count);
                queue.push(behavior_output_port(dag.node(l.body)));
            }
            Behavior::Bind(b) => {
                queue.push(b.value);
            }
        }
    }

    referenced
}

fn behavior_output_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.output,
        Behavior::Transform(t) => t.output,
        Behavior::Branch(b) => b.output,
        Behavior::Loop(l) => l.output,
        Behavior::Bind(b) => b.value,
    }
}

#[test]
fn unused_parameters_dag_compiles_cleanly() {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("unused_parameters.dag should compile cleanly");
    assert!(
        dag.diagnostics().is_empty(),
        "unused_parameters.dag should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn unused_parameters_generated_module_matches_checked_in_snapshot() {
    let fresh = emit_lens_module();
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module().trim(),
        "checked-in generated module is stale; regenerate lens_unused_parameters_generated.rs from unused_parameters.dag"
    );
}

#[test]
fn unused_parameters_generated_module_clone_count_is_ratcheted() {
    const MAX_CLONE_CALLS: usize = 6;

    let fresh = emit_lens_module();
    let clone_calls = clone_call_count(&fresh);
    assert!(
        clone_calls <= MAX_CLONE_CALLS,
        "generated lens clone count regressed: observed {clone_calls}, ratchet allows at most {MAX_CLONE_CALLS}",
    );
}

#[test]
fn unused_parameters_dag_matches_rust_lens_on_core_fixtures() {
    let module = emit_lens_module();
    let fixtures = [
        ("fn add(a: Int, b: Int) -> Int = a + b", "used_all.v3"),
        ("fn first(a: Int, b: Int) -> Int = a", "single_unused.v3"),
        (
            "fn always_one(x: Int, y: Int, z: Int) -> Int = 1",
            "constant_body.v3",
        ),
        (
            "fn pick(a: Int, b: Int) -> Int = if a > 0 then a else b",
            "branch_body.v3",
        ),
        (
            "fn count(list: List<Int>) -> Int = match list { Empty => 0, Cons(payload) => 1 + count(payload.tail) }",
            "recursive_list.v3",
        ),
        (
            "fn content_upsert(content: Int, path: Int) -> Int = content + 0",
            "patterns_synthetic.v3",
        ),
    ];

    let bin_path = build_roundtrip_harness(&module);
    for (source, file_name) in fixtures {
        let rust_rendered = render_handwritten_oracle(source, file_name);
        let dag_rendered = roundtrip_lens_render(&bin_path, source, file_name);
        assert_eq!(
            dag_rendered, rust_rendered,
            "compiled .dag lens should match handwritten Rust oracle on {file_name}"
        );
    }
}

#[test]
fn unused_parameters_dag_self_analysis_reports_zero_findings() {
    let module = emit_lens_module();
    let bin_path = build_roundtrip_harness(&module);
    let rendered = roundtrip_lens_render(
        &bin_path,
        &lens_source(),
        lens_path().to_string_lossy().as_ref(),
    );
    assert_eq!(rendered, "");
}

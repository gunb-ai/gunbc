use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    Behavior, BindNode, BranchNode, LoopBound, LoopNode, Path as DagPath, Port, TransformNode,
    ValueNode,
};
use v3_compiler::emit::{emit as shared_emit, emit_module as shared_emit_module, EmitTarget};
use v3_compiler::emit_python::emit_python_module;
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::Dag;

static ROUNDTRIP_ID: AtomicUsize = AtomicUsize::new(0);

fn lens_source() -> String {
    std::fs::read_to_string(lens_path()).expect("read unused_parameters.dag")
}

fn lens_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lenses")
        .join("unused_parameters.dag")
}

fn emit_rust_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled lens source");
    emit_rust_module(&dag).expect("emit rust lens module")
}

fn emit_python_lens_module() -> String {
    let dag = compile_to_dag(&lens_source(), lens_path().to_string_lossy().as_ref())
        .expect("compiled lens source");
    emit_python_module(&dag).expect("emit python lens module")
}

fn emit_python_module_from_source(source: &str, file_name: &str) -> String {
    let dag = compile_to_dag(source, file_name).expect("compiled python module source");
    emit_python_module(&dag).expect("emit python module")
}

#[test]
fn emit_python_wrappers_match_shared_entrypoint() {
    let program_source = "\
fn double(x: Int) -> Int = x + x
let result: Int = double(21)
";
    let program_dag =
        compile_to_dag(program_source, "emit_python_wrapper_program_parity.v3").expect("compiles");
    let shared = shared_emit(&program_dag, EmitTarget::Python)
        .expect("shared emit")
        .text;
    let wrapper = v3_compiler::emit_python::emit_python(&program_dag).expect("wrapper emit");
    assert_eq!(
        shared, wrapper,
        "emit_python wrapper drifted from emit::emit"
    );

    let module_source = "\
fn double(x: Int) -> Int = x + x
";
    let module_dag =
        compile_to_dag(module_source, "emit_python_wrapper_module_parity.v3").expect("compiles");
    let shared_module = shared_emit_module(&module_dag, EmitTarget::Python)
        .expect("shared module emit")
        .text;
    let wrapper_module = emit_python_module(&module_dag).expect("wrapper module emit");
    assert_eq!(
        shared_module, wrapper_module,
        "emit_python_module wrapper drifted from emit::emit_module"
    );
}

#[test]
fn emit_python_uses_only_shared_schema_surface() {
    const PYTHON_SPEC: &str = include_str!("../../../spec/python.dag");
    const PYTHON_EMITTER: &str = include_str!("../../src/emit/python_target.rs");
    let forbidden = [
        "PythonTypeRealization",
        "PythonOperatorRealization",
        "PythonCallableRealization",
        "PythonCallableStrategy",
        "PythonTypeInstantiationRealization",
        "PythonPatternRealization",
        "PythonPatternStrategy",
        "PythonExpressionSyntax",
        "PythonCollectionOps",
        "PythonTypeApplicationSyntax",
    ];
    for needle in forbidden {
        assert!(
            !PYTHON_SPEC.contains(needle),
            "spec/python.dag still contains private Python scaffold `{needle}`"
        );
        assert!(
            !PYTHON_EMITTER.contains(needle),
            "emit/python_target.rs still contains private Python scaffold `{needle}`"
        );
    }
}

fn next_roundtrip_dir() -> PathBuf {
    let id = ROUNDTRIP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "v3_emit_python_roundtrip_{}_{}",
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

fn roundtrip_rust_lens_render(
    module_source: &str,
    program_source: &str,
    file_name: &str,
) -> String {
    let wrapped = format!(
        "mod emitted {{ use v3_compiler::dag::*; use v3_compiler::diagnostics::*; {module_source} }} \
         fn render(dag: &v3_compiler::Dag, function: v3_compiler::dag::NodeId) -> String {{ \
           dag.nodes().iter().find_map(|node| match node {{ \
             v3_compiler::dag::Behavior::Bind(bind) if bind.id == function => Some(bind.name.clone()), \
             _ => None \
           }}).unwrap_or_else(|| format!(\"{{:?}}\", function)) \
         }} \
         fn main() {{ \
           let dag = v3_compiler::compile_to_dag({program_source:?}, {file_name:?}).expect(\"compiles\"); \
           let mut rendered: Vec<String> = emitted::check(&dag).iter().map(|v| format!(\"{{}}:param[{{}}]\", render(&dag, v.function), v.parameter_index)).collect(); \
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

    let run = Command::new(&bin_path)
        .output()
        .expect("run compiled rust binary");
    assert!(run.status.success(), "compiled rust binary failed");
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn python_runtime_prelude() -> &'static str {
    r#"
from dataclasses import dataclass
import typing

NodeId = str
PortId = str
DeclarationId = str

@dataclass
class SourceSpan:
    file: str
    start: int
    end: int

@dataclass
class DagPort:
    id: PortId
    state: typing.Any
    produced_by: typing.Optional[NodeId]

@dataclass
class BranchPath:
    body: NodeId
    result_port: PortId
    pattern: typing.Any = None
    binding: typing.Any = None

class LoopBound:
    pass

@dataclass
class LoopBound_Cardinality(LoopBound):
    count: PortId

@dataclass
class LoopBound_Descent(LoopBound):
    cluster: str

@dataclass
class ValueNode:
    id: NodeId
    payload: typing.Any
    result_port: PortId
    span: SourceSpan

@dataclass
class TransformNode:
    id: NodeId
    target: typing.Any
    inputs: list[PortId]
    result_port: PortId
    span: SourceSpan

@dataclass
class BranchNode:
    id: NodeId
    input: PortId
    paths: list[BranchPath]
    result_port: PortId
    span: SourceSpan

@dataclass
class LoopNode:
    id: NodeId
    source: PortId
    init: PortId
    body: NodeId
    bound: LoopBound
    result_port: PortId
    span: SourceSpan

@dataclass
class BindNode:
    id: NodeId
    name: str
    result_port: PortId
    params: list[PortId]
    span: SourceSpan

class Behavior:
    pass

@dataclass
class Behavior_Value(Behavior):
    _0: ValueNode

@dataclass
class Behavior_Transform(Behavior):
    _0: TransformNode

@dataclass
class Behavior_Branch(Behavior):
    _0: BranchNode

@dataclass
class Behavior_Loop(Behavior):
    _0: LoopNode

@dataclass
class Behavior_Bind(Behavior):
    _0: BindNode

@dataclass
class Dag:
    declarations: list[typing.Any]
    nodes: list[Behavior]
    ports: list[DagPort]
    clusters: list[typing.Any]
"#
}

fn roundtrip_python_lens_render(
    module_source: &str,
    program_source: &str,
    file_name: &str,
) -> String {
    let dag = compile_to_dag(program_source, file_name).expect("program compiles");
    let serialized_dag = serialize_dag(&dag);
    let (future_import, module_source) = split_future_annotations(module_source);
    let wrapped = format!(
        "{}{}{}\n\ndag = {}\n\ndef render(dag: Dag, function: NodeId) -> str:\n    for node in dag.nodes:\n        if isinstance(node, Behavior_Bind) and node._0.id == function:\n            return node._0.name\n    return function\n\nrendered = [f\"{{render(dag, v.function)}}:param[{{v.parameter_index}}]\" for v in check(dag)]\nrendered.sort()\nprint(\"|\".join(rendered))\n",
        future_import,
        python_runtime_prelude(),
        module_source,
        serialized_dag
    );

    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("main.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(wrapped.as_bytes()))
        .expect("write wrapped python source");

    let run = Command::new("python3")
        .arg(&src_path)
        .output()
        .expect("run python3");
    assert!(
        run.status.success(),
        "python3 failed on emitted source:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn run_python_module(module_source: &str) {
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("module.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(module_source.as_bytes()))
        .expect("write python module source");

    let run = Command::new("python3")
        .arg(&src_path)
        .output()
        .expect("run python3");
    assert!(
        run.status.success(),
        "python3 failed on emitted module:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

fn split_future_annotations(module_source: &str) -> (&str, &str) {
    let future = "from __future__ import annotations\n\n";
    module_source
        .strip_prefix(future)
        .map(|rest| (future, rest))
        .unwrap_or(("", module_source))
}

fn serialize_dag(dag: &Dag) -> String {
    format!(
        "Dag(declarations=[], nodes=[{}], ports=[{}], clusters=[])",
        dag.nodes()
            .iter()
            .map(serialize_behavior)
            .collect::<Vec<_>>()
            .join(", "),
        dag.ports()
            .iter()
            .map(serialize_port)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn serialize_behavior(node: &Behavior) -> String {
    match node {
        Behavior::Value(value) => format!("Behavior_Value({})", serialize_value_node(value)),
        Behavior::Transform(transform) => {
            format!(
                "Behavior_Transform({})",
                serialize_transform_node(transform)
            )
        }
        Behavior::Branch(branch) => format!("Behavior_Branch({})", serialize_branch_node(branch)),
        Behavior::Loop(loop_node) => format!("Behavior_Loop({})", serialize_loop_node(loop_node)),
        Behavior::Bind(bind) => format!("Behavior_Bind({})", serialize_bind_node(bind)),
    }
}

fn serialize_value_node(node: &ValueNode) -> String {
    format!(
        "ValueNode(id={}, payload=None, result_port={}, span={})",
        py_debug(&node.id),
        py_debug(&node.result_port()),
        serialize_span(&node.span)
    )
}

fn serialize_transform_node(node: &TransformNode) -> String {
    format!(
        "TransformNode(id={}, target=None, inputs=[{}], result_port={}, span={})",
        py_debug(&node.id),
        node.inputs
            .iter()
            .map(py_debug)
            .collect::<Vec<_>>()
            .join(", "),
        py_debug(&node.result_port()),
        serialize_span(&node.span)
    )
}

fn serialize_branch_node(node: &BranchNode) -> String {
    format!(
        "BranchNode(id={}, input={}, paths=[{}], result_port={}, span={})",
        py_debug(&node.id),
        py_debug(&node.input),
        node.paths
            .iter()
            .map(serialize_branch_path)
            .collect::<Vec<_>>()
            .join(", "),
        py_debug(&node.result_port()),
        serialize_span(&node.span)
    )
}

fn serialize_branch_path(path: &DagPath) -> String {
    format!(
        "BranchPath(body={}, result_port={})",
        py_debug(&path.body),
        py_debug(&path.result_port())
    )
}

fn serialize_loop_node(node: &LoopNode) -> String {
    format!(
        "LoopNode(id={}, source={}, init={}, body={}, bound={}, result_port={}, span={})",
        py_debug(&node.id),
        py_debug(&node.source),
        py_debug(&node.init),
        py_debug(&node.body),
        serialize_loop_bound(&node.bound),
        py_debug(&node.result_port()),
        serialize_span(&node.span)
    )
}

fn serialize_loop_bound(bound: &LoopBound) -> String {
    match bound {
        LoopBound::Cardinality { count } => {
            format!("LoopBound_Cardinality(count={})", py_debug(count))
        }
        LoopBound::Descent { cluster } => {
            format!("LoopBound_Descent(cluster={})", py_debug(cluster))
        }
    }
}

fn serialize_bind_node(node: &BindNode) -> String {
    format!(
        "BindNode(id={}, name={:?}, result_port={}, params=[{}], span={})",
        py_debug(&node.id),
        node.name,
        py_debug(&node.result_port()),
        node.params
            .iter()
            .map(py_debug)
            .collect::<Vec<_>>()
            .join(", "),
        serialize_span(&node.span)
    )
}

fn serialize_port(port: &Port) -> String {
    let produced_by = port
        .produced_by
        .map(|id| py_debug(&id))
        .unwrap_or_else(|| "None".to_string());
    format!(
        "DagPort(id={}, state=None, produced_by={})",
        py_debug(&port.id()),
        produced_by
    )
}

fn serialize_span(span: &v3_compiler::diagnostics::SourceSpan) -> String {
    format!(
        "SourceSpan(file={:?}, start={}, end={})",
        span.file, span.byte_start, span.byte_end
    )
}

fn py_debug<T: std::fmt::Debug>(value: &T) -> String {
    let inner = format!("{value:?}");
    format!("{inner:?}")
}

// emit_python_lens_module compiles unused_parameters.dag, which uses
// recursive helpers that lower to `Behavior::Loop`. emit_python now
// fail-closes on Loop instead of silently rendering the body's result
// port. Re-enable once emit_python gains Loop emission (Lane 1e
// consolidation).
#[test]
#[ignore = "blocked on emit_python Behavior::Loop support; previously passed via silent loop-body collapse"]
fn emit_python_module_marks_ownership_as_skipped_for_gc_target() {
    let module = emit_python_lens_module();
    assert!(
        module.contains("# ownership skipped: GarbageCollected / LexicalScoping"),
        "expected ownership-skip trace in emitted Python module, got:\n{module}"
    );
}

#[test]
fn emit_python_module_defers_annotation_evaluation() {
    let module = emit_python_module_from_source(
        "\
type First = WrapSecond(Second) | FirstNone
type Second = WrapFirst(First) | SecondNone
",
        "recursive_types.v3",
    );
    assert!(
        module.starts_with("from __future__ import annotations\n"),
        "expected future-annotations import at module top, got:\n{module}"
    );
    run_python_module(&module);
}

#[test]
fn emit_python_module_qualifies_variant_class_names_by_parent_type() {
    let module = emit_python_module_from_source(
        "\
type First = Shared(Int) | FirstMissing
type Second = Shared(String) | SecondMissing
fn make_first(x: Int) -> First = Shared(x)
fn make_second(x: String) -> Second = Shared(x)
fn first_value(v: First) -> Int = match v { Shared(n) => n, FirstMissing => 0 }
fn second_value(v: Second) -> String = match v { Shared(s) => s, SecondMissing => \"\" }
",
        "variant_collision.v3",
    );
    assert!(
        module.contains("class First_Shared(First):"),
        "expected qualified runtime class for First.Shared, got:\n{module}"
    );
    assert!(
        module.contains("class Second_Shared(Second):"),
        "expected qualified runtime class for Second.Shared, got:\n{module}"
    );
    assert!(
        module.contains("return First_Shared(p0)"),
        "expected qualified constructor call for First.Shared, got:\n{module}"
    );
    assert!(
        module.contains("return Second_Shared(p0)"),
        "expected qualified constructor call for Second.Shared, got:\n{module}"
    );
    assert!(
        module.contains("isinstance(__match, First_Shared)"),
        "expected qualified match guard for First.Shared, got:\n{module}"
    );
    assert!(
        module.contains("isinstance(__match, Second_Shared)"),
        "expected qualified match guard for Second.Shared, got:\n{module}"
    );
    assert!(
        !module.contains("class Shared("),
        "unexpected unqualified runtime class survived in emitted module:\n{module}"
    );
}

/// E-5 / Lane 1 Stage 1c PR 3 pilot — unused match-arm payload
/// bindings never leak the source-level identifier into emitted
/// Python. `python_clean_emission.pattern_bindings =
/// NotApplicablePatternBinding` encodes the structural fact that
/// emit_python does not use Python's native `match` statement and
/// therefore has no pattern-site to emit a binding at; it
/// substitutes an extraction expression (`__match._0` / `__match`)
/// at every payload-port reference in the rendered arm body, and
/// unused references produce no identifier at all.
///
/// Before the pilot the emitter dropped the binding into
/// `arm_locals.names` unconditionally without consulting the
/// contract. After the pilot the dispatch reads
/// `python_clean_emission` via the typed `PatternBindingRuleVariants`
/// cache (Lane 1 Stage 1c PR 2.5) and rejects Python-invalid
/// variants at contract-parse time.
#[test]
fn emit_python_unused_payload_binding_never_leaks_identifier() {
    let module = emit_python_module_from_source(
        "type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(unique_value) => 0, Empty => 1 }",
        "python_clean_emission_unused.v3",
    );
    assert!(
        !module.contains("unique_value"),
        "unused payload leaked binding identifier into emitted Python, got:\n{module}"
    );
    assert!(
        module.contains("isinstance(__match, BoxedInt_Boxed)"),
        "expected qualified isinstance dispatch, got:\n{module}"
    );
    run_python_module(&module);
}

/// Complement to `emit_python_unused_payload_binding_never_leaks_identifier`:
/// when the arm body references the payload binding, the emitter
/// substitutes `__match._0` at the port reference. The source-level
/// identifier is still absent — Python's
/// `NotApplicablePatternBinding` rule never surfaces it, used or
/// unused. Proves the contract dispatch doesn't regress the
/// used-binding path.
#[test]
fn emit_python_used_payload_binding_substitutes_match_extraction() {
    let module = emit_python_module_from_source(
        "type BoxedInt = Boxed(Int) | Empty
fn use_payload(b: BoxedInt) -> Int = match b { Boxed(unique_value) => unique_value, Empty => 1 }",
        "python_clean_emission_used.v3",
    );
    assert!(
        !module.contains("unique_value"),
        "used payload leaked binding identifier into emitted Python, got:\n{module}"
    );
    assert!(
        module.contains("__match._0"),
        "expected used payload to render via __match._0 substitution, got:\n{module}"
    );
    run_python_module(&module);
}

#[test]
fn emit_python_named_single_field_payload_projects_from_match_value() {
    let module = emit_python_module_from_source(
        "type Point { x: Int y: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap(payload) => payload.inner.x, Empty => 0 }",
        "python_variant_payload_named_single.v3",
    );
    assert!(
        module.contains("__match.inner.x"),
        "expected named single-field payload access to project from __match, got:\n{module}"
    );
    assert!(
        !module.contains("__match.inner.inner"),
        "named single-field payload access must not double-project through the inner field, got:\n{module}"
    );
    run_python_module(&module);
}

#[test]
fn emit_python_named_multi_field_payload_projects_from_match_value() {
    let module = emit_python_module_from_source(
        "type Pair = Both { left: Int right: Int } | Empty
fn right_or_zero(p: Pair) -> Int = match p { Both(payload) => payload.right, Empty => 0 }",
        "python_variant_payload_named_multi.v3",
    );
    assert!(
        module.contains("__match.right"),
        "expected named multi-field payload access to project from __match, got:\n{module}"
    );
    assert!(
        !module.contains("payload.right"),
        "named multi-field payload access must not leak the source binding name, got:\n{module}"
    );
    run_python_module(&module);
}

/// E-5 / Lane 1 Stage 1c PR 4 — emitted Python passes
/// `python_clean_emission.post_emit_verifier` as invoked through
/// the shared harness (`python3 -m py_compile` +
/// `IgnoreVerifierOutput`). Reads the contract via
/// `parse_post_emit_verifier` so any future spec change (different
/// verifier, new args) picks up automatically without updating the
/// test body.
///
/// This is the sole py_compile roundtrip for the Python pilot —
/// there is no separate hardcoded-command version. `run_post_emit_verifier`
/// IS the only verifier-invocation authority the test suite uses, so
/// spec drift (e.g. flipping to `ruff --select=E` in the future)
/// shows up here without test-code edits.
///
/// Gated behind `#[ignore]` — CI sandboxes don't always carry
/// python3. Run locally:
///
///     cargo test -p v3-compiler --test m1_4_emit_python_test \
///         python_pilot_source_passes_post_emit_verifier_harness \
///         -- --ignored --nocapture
#[test]
#[ignore]
fn python_pilot_source_passes_post_emit_verifier_harness() {
    use v3_compiler::post_emit_verifier::{parse_post_emit_verifier, run_post_emit_verifier};
    let source = "type BoxedInt = Boxed(Int) | Empty
fn ignore_payload(b: BoxedInt) -> Int = match b { Boxed(unique_value) => 0, Empty => 1 }";
    let dag = compile_to_dag(source, "python_post_emit_verifier.v3").expect("source compiles");
    let spec = dag
        .python_clean_emission_spec()
        .expect("python_clean_emission cached");
    let binding = parse_post_emit_verifier(&dag, spec).expect("parse contract");
    let rendered = emit_python_module(&dag).expect("emits python module");
    let tmp_dir = next_roundtrip_dir();
    std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
    let src_path = tmp_dir.join("module.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(rendered.as_bytes()))
        .expect("write python source");
    run_post_emit_verifier(&binding, &src_path)
        .expect("python post_emit_verifier rejected pilot source — E-5 contract regression");
}

// Same Loop blocker as emit_python_module_marks_ownership_as_skipped_for_gc_target.
#[test]
#[ignore = "blocked on emit_python Behavior::Loop support; previously passed via silent loop-body collapse"]
fn emitted_python_lens_matches_emitted_rust_lens_on_reflected_programs() {
    let rust_module = emit_rust_lens_module();
    let python_module = emit_python_lens_module();

    let fixtures = [
        (
            "fn keep(a: Int, b: Int) -> Int = a",
            "unused_fixture.v3",
            "keep:param[1]",
        ),
        (
            "fn count_down(n: Int, marker: Int) -> Int = if n == 0 then 0 else count_down(n - 1, marker)",
            "loop_fixture.v3",
            "",
        ),
    ];

    for (source, file_name, expected) in fixtures {
        let rust_rendered = roundtrip_rust_lens_render(&rust_module, source, file_name);
        let python_rendered = roundtrip_python_lens_render(&python_module, source, file_name);
        assert_eq!(
            python_rendered, rust_rendered,
            "python and rust emitted lenses diverged for {file_name}"
        );
        assert_eq!(
            python_rendered, expected,
            "unexpected rendered unused-parameter set for {file_name}"
        );
    }
}

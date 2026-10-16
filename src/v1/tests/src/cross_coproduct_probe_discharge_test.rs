// #14152 review 81061 item (2): discharge measurement for the two known_red_probe rows
// in dag/gunbc/explicit_witness_admission.dag (~lines 170-184), whose dissolve_on reads:
//   "the shipped seed v1_compiler_infer.rs unify_generics returns UnifyGenericsAcc AND
//    compile of this probe is a typecheck refusal (not first-wins Accepted)".
//
// Both probe entries (dag/test/claim/constructor_tag_route_probe_red.dag and
// src/v2/test/claim/manual/constructor_tag_route_probe_red.dag) carry ONE body, so a
// single source is the witness for both rows; the rows differ only in module name.
//
// The exit condition is a property of COMPILING the probe, not of running it, so it is
// measured on the shipped seed binary through the same compile path the claim executor
// uses. claim_batch itself cannot run on the remote runner: the host budget has no
// absorbable fallback (cli_run/entry_resolve.rs refuses with HostBudgetUnreadable
// rather than accept a fabricated GUNBC_MEMORY_BUDGET_BYTES, per
// dag/gunbc/host/host_budget_source.dag), and the runner grants no cgroup write — both
// `+memory` in cgroup.subtree_control and mkdir under /sys/fs/cgroup are EPERM there,
// so the ci_spec.dag hosted_runner_memory_cgroup_bind_commands recipe is unavailable.
// That refusal is correct behaviour and is NOT worked around here; it is reported.
//
// §4b dissolution: when this refuses, the RED stays enrolled and the QUARANTINE goes.
use crate::helpers::compile_dag;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

// Same local helpers the sibling locate test defines; they are file-private there, not
// re-exported from helpers.
fn blocking_messages(source: &str) -> Vec<String> {
    let result = compile_dag(source);
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn compile_program_errors(source: &str) -> Vec<String> {
    blocking_messages(source)
}

fn has_generic_rebind(msgs: &[String]) -> bool {
    // Matched on the fold's own message SHAPE, not on a variable letter: the text is
    // `type variable <bind_name> bound as <shape> (first argument)` vs `<shape> (later
    // argument)` (v1.compiler.infer unify_generics, rebind-branch expected/got construction). Hardcoding "T" makes every row over a
    // differently-named generic vacuously green -- which is how the Box<Int> vs
    // Box<Bool> row read as a failure on both seeds while the fold was in fact refusing
    // it (measured at 2751b200: the message was present, under the name A).
    msgs.iter().any(|m| {
        m.contains("bound as") && m.contains("(first argument)") && m.contains("(later argument)")
    })
}

// Verbatim from both probe files, minus the module header's claim-fixture imports.
// P1 belongs to Pa and Q1 to Pb: same arity, same payload type, different tags — the
// probe's whole point is that they must not be conflated by one T.
fn probe_source() -> String {
    "module probe.ctor_tag.discharge

type Pa = P1 { a: Int } | P2
type Pb = Q1 { a: Int } | Q2

type ConstructorTag<T> = ConstructorTag { name: String }

fn tag_of<T>(v: T) -> ConstructorTag<T> {
  ConstructorTag { name: discriminant(v: v) }
}

fn same_arm<T>(a: T, b: T) -> Bool {
  tag_of(v: a) == tag_of(v: b)
}

fn witness() -> Bool {
  same_arm(a: P1 { a: 1 }, b: Q1 { a: 1 })
}
"
    .to_string()
}

// THE EXIT CONDITION, measured. Before the seed carried the fold this compiled and
// returned (first-wins bound T to Pa and moved on), so the probe was red because it
// returned false. It must now be red because the compiler REFUSED, which is a
// different kind of red and the one the row asked to be dissolved on.
#[test]
fn probe_one_t_two_coproducts_compiles_to_a_refusal() {
    let errors = compile_program_errors(&probe_source());
    assert!(
        !errors.is_empty(),
        "the probe must not compile clean; it was first-wins Accepted before the fold"
    );
    let msgs = blocking_messages(&probe_source());
    assert!(
        has_generic_rebind(&msgs),
        "the probe's refusal must be the unify_generics rebind refusal, not some other \
         blocking diagnostic; got {msgs:?}"
    );
}

// And the probe must still refuse for the RIGHT reason: the two arms are distinct
// types. If this accepted, the probe would be refusing for an unrelated defect and the
// discharge would be vacuous.
#[test]
fn probe_same_tag_side_by_side_is_still_accepted() {
    let src = probe_source().replace(
        "same_arm(a: P1 { a: 1 }, b: Q1 { a: 1 })",
        "same_arm(a: P1 { a: 1 }, b: P1 { a: 2 })",
    );
    let msgs = blocking_messages(&src);
    assert!(
        !has_generic_rebind(&msgs),
        "one T bound by two P1 applications must not refuse: {msgs:?}"
    );
}

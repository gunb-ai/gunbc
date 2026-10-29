//! FLOOR PROBE (node://adhoc-5fdefc60-2eb; found by bright-crane-219 on gunbc#15230).
//!
//! THE FLOOR: DESIGN section 4b's "names resolve" rung, read with section 5's fail-closed rule
//! and namespace-resolution-design.md section 13 -- a bare name's candidates are the binders on
//! the REFERENCING module's own chain, never the corpus-wide pool. A name declared in module A
//! must NOT resolve in module B when B never imports it and A is not an ancestor of B.
//!
//! THE DEFECT THIS PROBE PINS: the seed's global-bare pool answers a name that is declared
//! EXACTLY ONCE in the whole compile from ANY module in it. The homonym arm of the same lookup
//! already applies the chain test under the production policy (namespace_unique_on_chain_policy_
//! test.rs pins both its refusal and its unique-on-chain control); the UNIQUE arm carries no such
//! test, so the floor holds for two binders and falls for one. That asymmetry is the earliest
//! unjustified boundary, and it is what made the unimported names in
//! test.claim.gcp_iam_bootstrap_population_split_witness execute green.
//!
//! THE SHAPE: three files. `FloorProbeRecord` / `floor_probe_label` are declared ONLY in
//! probe.floor.a. probe.floor.b never imports probe.floor.a and is not its descendant. probe.floor.c
//! is the root that imports both, so probe.floor.b is in the compile and its bare occurrences are
//! resolved. The refusal must be typed and located at probe.floor.b's use site.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn error_diag_messages(sources: Vec<Rc<SourceFile>>) -> Vec<String> {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    resolved
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

/// probe.floor.a declares both names ONCE -- corpus-unique, so the pool's unique arm answers.
fn probe_a() -> Rc<SourceFile> {
    src(
        "probe_floor_a.dag",
        "module probe.floor.a\ntype FloorProbeRecord { tag: Int }\ndata floor_probe_label: String = \"hit\"\n",
    )
}

/// probe.floor.b: NO import of probe.floor.a, and `probe.floor.b` is not a descendant of it.
/// Both occurrences are bare: a record type in parameter position and a field access, plus a
/// bare reference to the `data` value.
fn probe_b(with_import: bool) -> Rc<SourceFile> {
    let header = if with_import {
        "module probe.floor.b\nimport probe.floor.a { FloorProbeRecord, floor_probe_label }\n"
    } else {
        "module probe.floor.b\n"
    };
    src(
        "probe_floor_b.dag",
        &format!(
            "{header}fn floor_probe_tag(x: FloorProbeRecord) -> Int {{ x.tag }}\nfn floor_probe_name() -> String {{ floor_probe_label }}\n"
        ),
    )
}

/// probe.floor.c is the root: it imports BOTH modules, which is what puts probe.floor.b's
/// unimported reference inside the compile at all.
fn probe_c() -> Rc<SourceFile> {
    src(
        "probe_floor_c.dag",
        "module probe.floor.c\nimport probe.floor.a { FloorProbeRecord }\nimport probe.floor.b { floor_probe_tag, floor_probe_name }\nfn floor_probe_go() -> Int {{ floor_probe_tag(x: FloorProbeRecord { tag: 1 }) }}\nfn floor_probe_show() -> String {{ floor_probe_name() }}\n",
    )
}

// ── RED: the floor this work item is about ───────────────────────────────────────────────

/// The 3-file probe. probe.floor.b references a corpus-unique name it never imports: the
/// compile must REFUSE with a typed, located diagnostic naming the unimported name, and must
/// not execute green.
#[test]
fn unimported_bare_type_reference_refuses() {
    let diags = error_diag_messages(vec![probe_a(), probe_b(false), probe_c()]);
    assert!(
        diags.iter().any(|m| m.contains("FloorProbeRecord")),
        "probe.floor.b never imports probe.floor.a, so `FloorProbeRecord` is unimported there; \
         the compile must refuse with a typed diagnostic naming the unimported name at its \
         located site -- the pool's corpus-unique arm must not answer across modules; got {diags:?}"
    );
}

#[test]
fn unimported_bare_data_value_reference_refuses() {
    let diags = error_diag_messages(vec![probe_a(), probe_b(false), probe_c()]);
    assert!(
        diags.iter().any(|m| m.contains("floor_probe_label")),
        "the same probe on the VALUE path: `floor_probe_label` is a `data` declared only in \
         probe.floor.a and referenced bare in probe.floor.b; the compile must refuse there too, \
         not bind it through the pool; got {diags:?}"
    );
}

// ── controls: the refusal is about the missing import, nothing else ──────────────────────

/// Control 1: the SAME three files with probe.floor.b importing probe.floor.a compile clean.
/// Discriminating: the two fixtures differ by one import line.
#[test]
fn imported_bare_reference_compiles_clean() {
    let diags = error_diag_messages(vec![probe_a(), probe_b(true), probe_c()]);
    assert!(
        diags.is_empty(),
        "with the import present every name resolves -- this must compile clean; got {diags:?}"
    );
}

/// Control 2: a bare name whose ONLY binder sits on the referencing module's ancestor chain
/// resolves -- the floor is unique-on-chain, not no-bare-names-anywhere. (Same shape as
/// namespace_unique_on_chain_policy_test.rs's unique_on_chain_fixture, on the corpus-unique arm.)
#[test]
fn chain_ancestor_bare_reference_compiles_clean() {
    let diags = error_diag_messages(vec![
        src(
            "probe_chain_mid.dag",
            "module probe.chain.mid\ntype ChainProbeRecord { tag: Int }\n",
        ),
        src(
            "probe_chain_leaf.dag",
            "module probe.chain.mid.leaf\nfn chain_probe_tag(x: ChainProbeRecord) -> Int { x.tag }\n",
        ),
    ]);
    assert!(
        diags.is_empty(),
        "`ChainProbeRecord` is declared in probe.chain.mid, an ancestor of the referencing \
         module probe.chain.mid.leaf -- unique ON CHAIN, so it must still resolve; got {diags:?}"
    );
}

/// Control 3: kernel builtins (map/filter/count/any/all) and the `none` literal stay bare and
/// admitted with no import. They are kernel surface, not pool entries; refusing them is NOT
/// this floor, and a fix that reddens them has cut too deep.
#[test]
fn kernel_builtins_and_none_stay_bare_and_admitted() {
    let diags = error_diag_messages(vec![src(
        "probe_kernel_surface.dag",
        "module probe.kernel.surface\nfn floor_probe_tally(xs: List<Int>) -> Bool {\n  let doubled = map(xs, x => x + x)\n  let evens = filter(doubled, x => x % 2 == 0)\n  count(evens) > 0 && all(evens, x => x >= 0) && any(evens, x => x == 0)\n}\nfn floor_probe_missing() -> String? { none }\n",
    )]);
    assert!(
        diags.is_empty(),
        "map/filter/count/any/all and the none literal are kernel-admitted bare; they must stay \
         admitted when the pool fallback is cut; got {diags:?}"
    );
}

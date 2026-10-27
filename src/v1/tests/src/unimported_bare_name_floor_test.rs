//! Floor probe (node://adhoc-5fdefc60-2eb, found by bright-crane-219 on #15230):
//! a bare name DECLARED in module A must NOT resolve in module B when B never
//! imports it. DESIGN §4b floor "names resolve" + §5 fail-closed +
//! namespace-resolution-design.md §13 (bare candidates are the binders on the
//! referencing module's OWN chain — never the whole corpus pool).
//!
//! The probe is the 3-file shape bright-crane-219 proved: `CrossModuleData` is
//! declared ONLY in probe.a; probe.b never imports it; probe.c (the root) imports
//! both. probe.b's bare `CrossModuleData` occurrences must refuse with a typed,
//! located diagnostic under the production policy — and the refusal must name
//! probe.b (where the unimported reference lives), not silently bind A's data.

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

fn probe_files() -> Vec<Rc<SourceFile>> {
    vec![
        src(
            "probe_a.dag",
            "module probe.a\ndata CrossModuleData { tag: Str }\n",
        ),
        src(
            "probe_b.dag",
            "module probe.b\nfn read_tag(x: CrossModuleData) -> Str { x.tag }\n",
        ),
        src(
            "probe_c.dag",
            "module probe.c\nimport probe.a { CrossModuleData }\nimport probe.b { read_tag }\nfn go() -> Str { read_tag(x: CrossModuleData { tag: \"hit\" }) }\n",
        ),
    ]
}

#[test]
fn unimported_bare_data_reference_refuses_under_production_policy() {
    let diags = error_diag_messages(probe_files());
    assert!(
        diags
            .iter()
            .any(|m| m.contains("CrossModuleData") && m.contains("probe.b")),
        "the 3-file probe must REFUSE: probe.b references CrossModuleData bare \
         without importing probe.a; the compile must carry a typed diagnostic naming \
         the unimported name at its located site; got {diags:?}"
    );
}

// ── positive controls ──────────────────────────────────────────────────────────────

/// Control 1: the SAME sources with the import present compile clean — the refusal
/// is about the missing import, not about the shape.
#[test]
fn imported_bare_data_reference_compiles_clean() {
    let diags = error_diag_messages(vec![
        src(
            "probe_a.dag",
            "module probe.a\ndata CrossModuleData { tag: Str }\n",
        ),
        src(
            "probe_b.dag",
            "module probe.b\nimport probe.a { CrossModuleData }\nfn read_tag(x: CrossModuleData) -> Str { x.tag }\n",
        ),
        src(
            "probe_c.dag",
            "module probe.c\nimport probe.a { CrossModuleData }\nimport probe.b { read_tag }\nfn go() -> Str { read_tag(x: CrossModuleData { tag: \"hit\" }) }\n",
        ),
    ]);
    assert!(
        diags.is_empty(),
        "with the import, every name resolves — must compile clean; got {diags:?}"
    );
}

/// Control 2: kernel builtins (map/filter/count/any/all) and the none literal stay
/// bare and admitted with no import — the floor fix must not touch them.
#[test]
fn kernel_builtins_and_none_stay_bare_and_admitted() {
    let diags = error_diag_messages(vec![src(
        "probe_builtins.dag",
        "module probe.builtins\nimport std.types { Bool, Int, List, Str }\nfn tally(xs: List<Int>) -> Bool {\n  let doubled = map(xs, x => x + x)\n  let evens = filter(doubled, x => x % 2 == 0)\n  count(evens) > 0 && all(evens, x => x >= 0) && any(evens, x => x == 0)\n}\nfn missing() -> Str? { none }\n",
    )]);
    assert!(
        diags.is_empty(),
        "map/filter/count/any/all + none are kernel-admitted bare; refusing them is \
         NOT this floor; got {diags:?}"
    );
}

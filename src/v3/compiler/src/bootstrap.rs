// Dag::new() bootstrap.
//
// Parses the production `dsl/std/*.dag` files (logic → bit → algebra →
// types) and lowers them into the freshly-created Dag so that the
// declaration table is primed with primitive types and algebraic
// structures before any user code runs. The four files are embedded via
// `include_str!` so bootstrap is hermetic at runtime and the declaration
// table stays in sync with the `.dag` source at build time.
//
// Before M1(2.6), bootstrap embedded narrower fixture strings and
// injected hardcoded `"+"`/`"-"`/... Arrow declarations to bridge
// user-code dispatch. Both are deleted: operators resolve via §8.9
// inhabitance walks in `infer.rs` (see `resolve_arrow`), and primitive
// types come from parsing the real files. This closes the FACTS FLOW
// FORWARD and SINGLE AUTHORITY concerns from PR #445's review.

use crate::dag::{ArrowBody, AtomPayload, Dag, Declaration, TypeConnective};
use crate::diagnostics::SourceSpan;
use crate::lower::{lower_into, resolve_pending_identifiers};
use crate::parse::parse;
use crate::tokenize::tokenize;

const LOGIC_DAG: &str = include_str!("../../../../dsl/std/logic.dag");
const BIT_DAG: &str = include_str!("../../../../dsl/std/bit.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../dsl/std/algebra.dag");
const INTEGER_DAG: &str = include_str!("../../../../dsl/std/integer.dag");
const FLOAT_DAG: &str = include_str!("../../../../dsl/std/float.dag");
const STRING_TYPE_DAG: &str = include_str!("../../../../dsl/std/string_type.dag");
const TYPES_DAG: &str = include_str!("../../../../dsl/std/types.dag");

pub(crate) fn bootstrap(dag: &mut Dag) {
    // Load order: `logic` → `bit` (needs Classical) → `algebra` (no deps)
    // → `integer`/`float` (need algebra + bit) → `string_type` (needs
    // algebra + types for Char, but the final sweep resolves the
    // cross-file forward ref) → `types` (needs integer for Int64).
    for (file, source) in &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
    ] {
        parse_and_lower_fixture(dag, source, file);
    }
    inject_realization_stub(dag);
    // Batch-final resolution for cross-file forward references (e.g.,
    // algebra.dag fields referencing `Bool` which types.dag declares).
    // Any identifier still unresolved after all four files load is a
    // genuine bootstrap bug and will surface as a diagnostic on the Dag
    // — at which point the panic below stops `Dag::new()` cold.
    resolve_pending_identifiers(dag);
    if !dag.diagnostics().is_empty() {
        panic!(
            "v3 bootstrap produced unresolved identifiers: {:?}",
            dag.diagnostics()
        );
    }
}

fn parse_and_lower_fixture(dag: &mut Dag, source: &str, file: &str) {
    let tokens = tokenize(source, file).unwrap_or_else(|diag| {
        panic!("v3 bootstrap tokenize failed in {file}: {diag:?}")
    });
    let module = parse(&tokens, file).unwrap_or_else(|diag| {
        panic!("v3 bootstrap parse failed in {file}: {diag:?}")
    });
    lower_into(dag, &module);
}

/// M1_DESIGN.md §6.5 realization smoke test scaffold. Constructs the minimal
/// set of declarations needed to exercise the `ArrowBody::ExternalRealization`
/// substrate path end-to-end:
///
///   1. A `Realization` meta-type declaration (empty Conj — shape placeholder).
///   2. A concrete realization instance `Int64_add_rust` whose `meta_tag` edge
///      points at the meta-type.
///   3. A named Arrow declaration `Int64_add` whose `body` is
///      `ExternalRealization(instance_id)` rather than `Pending`.
///
/// The substrate test `smoke_int_add_external_realization` walks this chain
/// and asserts inference accepts the declared Arrow without panicking. Per
/// M1_FOLLOWUPS.md, PR #444's spec has this data coming from a parsed
/// `dsl/extdeps/languages/rust.dag` stub — that requires parser support for
/// record literals and a `realization` item keyword, deferred to M1(2.6). The
/// Rust construction here is a placeholder that validates the substrate
/// shape; the M1(2.6) follow-up swaps it for fixture parsing without
/// substrate changes.
fn inject_realization_stub(dag: &mut Dag) {
    let span = SourceSpan::new("<bootstrap:realization>", 0, 0);

    let meta_type_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: meta_type_id,
        name: Some("Realization".to_string()),
        connective: TypeConnective::Conj {
            children: Vec::new(),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        span: span.clone(),
    });

    let instance_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: instance_id,
        name: Some("Int64_add_rust".to_string()),
        connective: TypeConnective::Conj {
            children: Vec::new(),
        },
        type_params: Vec::new(),
        meta_tag: Some(meta_type_id),
        inhabits: None,
        span: span.clone(),
    });

    let int_id = dag
        .declaration_by_name("Int")
        .expect("bootstrap: Int missing after fixtures")
        .id;
    let arrow_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: arrow_id,
        name: Some("Int64_add".to_string()),
        connective: TypeConnective::Arrow {
            inputs: vec![int_id, int_id],
            output: int_id,
            body: ArrowBody::ExternalRealization(instance_id),
        },
        type_params: Vec::new(),
        meta_tag: None,
        inhabits: None,
        span,
    });
}

// Re-export AtomPayload so future bootstrap work that adds Atom declarations
// directly doesn't hit an unused-import lint.
#[allow(dead_code)]
type _KeepAtomPayloadLive = AtomPayload;

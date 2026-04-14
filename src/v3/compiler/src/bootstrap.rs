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

/// M1_DESIGN.md §6.5 realization smoke test scaffold. Constructs the
/// minimal declaration chain needed to exercise the
/// `ArrowBody::ExternalRealization` substrate path end-to-end:
///
///   1. A `Realization` meta-type declaration (empty Conj, top-level
///      named — callers may refer to it as a type).
///   2. An **anonymous** concrete realization instance whose `meta_tag`
///      edge points at the meta-type. The instance is unreferenceable
///      by name so it stays out of `Dag::declaration_by_name`'s scan.
///   3. An **anonymous** Arrow declaration whose `body` is
///      `ExternalRealization(instance_id)` rather than `Pending`. The
///      Arrow's id is stashed in `Dag.realization_smoke_arrow` so the
///      substrate test can find it without a name lookup.
///
/// The typed-edge check from PR #445's review: before constructing the
/// `ExternalRealization`, assert the instance has a `Conj` connective
/// AND a `meta_tag` pointing at the `Realization` meta-type. Fails
/// construction (panic) if the chain is malformed — this is the narrow
/// shape guarantee `ArrowBody::ExternalRealization` needs without
/// introducing a full `RealizationId` newtype.
///
/// Per `src/v3/ROADMAP.md` M2, the Rust construction here is a
/// placeholder that validates the substrate shape; the follow-up swaps
/// it for fixture parsing (a `realization` item keyword + record
/// literal support) without substrate changes.
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
        name: None,
        connective: TypeConnective::Conj {
            children: Vec::new(),
        },
        type_params: Vec::new(),
        meta_tag: Some(meta_type_id),
        inhabits: None,
        span: span.clone(),
    });

    // Typed-edge check: verify the instance is realization-shaped
    // before encoding it in `ArrowBody::ExternalRealization`. Bootstrap
    // owns both sides so this is always true here — the check exists
    // as a self-documenting invariant and would catch any future drift
    // that tries to store a non-realization declaration in the body.
    assert_realization_shape(dag, instance_id, meta_type_id);

    let int_id = dag
        .declaration_by_name("Int")
        .expect("bootstrap: Int missing after fixtures")
        .id;
    let arrow_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: arrow_id,
        name: None,
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

    dag.set_realization_smoke_arrow(arrow_id);
}

/// Invariant check: a declaration used as the target of
/// `ArrowBody::ExternalRealization` must be a `Conj` whose `meta_tag`
/// edge points at the `Realization` meta-type. Bootstrap owns both
/// sides so this holds by construction; the assertion documents the
/// invariant and catches future drift.
fn assert_realization_shape(
    dag: &Dag,
    instance_id: crate::dag::DeclarationId,
    expected_meta: crate::dag::DeclarationId,
) {
    let decl = dag.declaration(instance_id);
    assert!(
        matches!(decl.connective, TypeConnective::Conj { .. }),
        "realization instance must be a Conj declaration"
    );
    assert_eq!(
        decl.meta_tag,
        Some(expected_meta),
        "realization instance's meta_tag must point at the Realization meta-type"
    );
}

// Re-export AtomPayload so future bootstrap work that adds Atom declarations
// directly doesn't hit an unused-import lint.
#[allow(dead_code)]
type _KeepAtomPayloadLive = AtomPayload;

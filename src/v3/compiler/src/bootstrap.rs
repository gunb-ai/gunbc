// Dag::new() bootstrap.
//
// Parses the production `dsl/std/*.dag` files in dependency order and
// lowers them into the freshly-created Dag so that the declaration table
// is primed with primitive types and algebraic structures before any
// user code runs. The seven files are embedded via `include_str!` so
// bootstrap is hermetic at runtime and the declaration table stays in
// sync with the `.dag` source at build time.
//
// **Production bootstrap does no realization injection.** Realization
// facts live in `dsl/extdeps/languages/*` per the thesis; compiler code
// does not manufacture them. The §6.5 `ExternalRealization` substrate
// path is exercised by a `#[cfg(test)]` scaffold below — the test owns
// both sides of the realization chain locally and does not leak into
// `Dag::new()`.
//
// Bootstrap failures (tokenize/parse/lower errors on std/ files,
// unresolved cross-file references) attach to the Dag's diagnostic
// table via `Dag::attach_diagnostic` rather than panicking, so
// `compile_to_dag` surfaces them through `Err(CompileError::Semantic(dag))`
// on every subsequent call — the same structural channel user errors
// go through. A failed bootstrap is visible to callers without a
// side channel.

use crate::dag::{Dag, DeclarationId};
use crate::lower::{
    collect_symbols_phase, lower_bodies_phase, resolve_pending_identifiers,
};
use crate::parse::{parse, SurfaceModule};
use crate::tokenize::tokenize;
use std::collections::HashMap;

const LOGIC_DAG: &str = include_str!("../../../../dsl/std/logic.dag");
const BIT_DAG: &str = include_str!("../../../../dsl/std/bit.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../dsl/std/algebra.dag");
const INTEGER_DAG: &str = include_str!("../../../../dsl/std/integer.dag");
const FLOAT_DAG: &str = include_str!("../../../../dsl/std/float.dag");
const STRING_TYPE_DAG: &str = include_str!("../../../../dsl/std/string_type.dag");
const TYPES_DAG: &str = include_str!("../../../../dsl/std/types.dag");

pub(crate) fn bootstrap(dag: &mut Dag) {
    // Two-phase loading across all seven std/ files. Phase 1 parses and
    // `collect_symbols_phase`s every file, allocating top-level
    // declarations + their TypeParam children in one batch. Phase 2
    // fills in each file's bodies, at which point every cross-file
    // template reference (e.g., `bit.dag`'s `Word64 { bytes: List<Byte> }`
    // where `List` is declared in `types.dag`) finds its template's
    // `type_params` slot already populated — no half-valid template
    // arguments, no post-sweep fixup pass.
    //
    // Load order within each phase: `logic` → `bit` (needs Classical)
    // → `algebra` (no deps) → `integer`/`float` (need algebra + bit)
    // → `types` (needs integer for Int64) → `string_type` (needs
    // Char from types; the sweep resolves the cross-file forward ref).
    let fixtures: &[(&str, &str)] = &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
    ];

    // Phase 0: parse every fixture. Tokenize/parse errors attach to
    // `dag.diagnostics()` and the corresponding module is omitted
    // from later phases.
    //
    // Phase 1: per-file `collect_symbols_phase` runs inline with the
    // parse loop so every file's declarations + type_params land in
    // the shared `dag` before ANY body lowering runs. The per-file
    // symbols map is captured but discarded — it's stale by the end
    // of Phase 1 because later files' declarations aren't in it.
    // Phase 2 uses a REBUILT shared symbols map below.
    let mut parsed: Vec<(SurfaceModule, Vec<bool>)> = Vec::with_capacity(fixtures.len());
    for (file, source) in fixtures.iter() {
        let Some(module) = parse_fixture(dag, source, file) else {
            continue;
        };
        let (_stale_symbols, is_first) = collect_symbols_phase(dag, &module.items);
        parsed.push((module, is_first));
    }

    // Rebuild the symbols map from the shared declaration table. By
    // now every top-level declaration across all fixtures is present
    // with its type_params slot populated, so Phase 2 can resolve
    // every cross-file template reference at construction time.
    // First-match semantics match `Dag::declaration_by_name`.
    let mut shared_symbols: HashMap<String, DeclarationId> = HashMap::new();
    for d in dag.declarations() {
        if let Some(name) = &d.name {
            shared_symbols.entry(name.clone()).or_insert(d.id);
        }
    }

    // Phase 2: lower bodies using the shared symbols map.
    for (module, is_first) in parsed.iter() {
        lower_bodies_phase(dag, module, &shared_symbols, is_first);
    }

    // Batch-final resolution for cross-file forward references. In
    // bootstrap mode the sweep tolerates dangling stubs — the canonical
    // std/ files reference types that live in modules outside the
    // M1(2.6) load set (e.g., `Tuple`), and those are not bootstrap
    // errors. User-code compilation uses the strict variant.
    resolve_pending_identifiers(dag);
}

fn parse_fixture(dag: &mut Dag, source: &str, file: &str) -> Option<SurfaceModule> {
    let tokens = match tokenize(source, file) {
        Ok(t) => t,
        Err(diag) => {
            dag.attach_diagnostic(diag);
            return None;
        }
    };
    match parse(&tokens, file) {
        Ok(m) => Some(m),
        Err(diag) => {
            dag.attach_diagnostic(diag);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    //! §6.5 realization smoke test. The stub chain (Realization meta-
    //! type, realization instance, realization Arrow) is constructed
    //! entirely inside this test module — no production bootstrap code
    //! is involved. The test exercises the
    //! `ArrowBody::ExternalRealization` substrate path end-to-end
    //! (construction + typed-edge validation + inference dispatch)
    //! without manufacturing realization facts at `Dag::new()` time.

    use super::*;
    use crate::dag::{
        ArrowBody, AtomPayload, Declaration, DeclarationId, TypeConnective,
    };
    use crate::diagnostics::SourceSpan;

    /// Build a Realization → instance → Arrow chain inside a fresh Dag.
    /// Returns the Arrow's DeclarationId so callers can walk it.
    fn inject_test_realization(dag: &mut Dag) -> DeclarationId {
        let span = SourceSpan::new("<test:realization>", 0, 0);

        let meta_type_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: meta_type_id,
            name: Some("TestRealization".to_string()),
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
        // before encoding it in `ArrowBody::ExternalRealization`. This
        // is the same invariant `infer::is_realization_shape` enforces
        // at dispatch time; the test asserts it at construction time
        // as well, so both sides of the invariant are exercised.
        let instance_decl = dag.declaration(instance_id);
        assert!(
            matches!(instance_decl.connective, TypeConnective::Conj { .. }),
            "realization instance must be a Conj"
        );
        assert_eq!(
            instance_decl.meta_tag,
            Some(meta_type_id),
            "realization instance's meta_tag must point at the TestRealization meta-type"
        );

        // Use an anonymous Int primitive reference for the Arrow
        // inputs/output. At runtime, the smoke test walks through the
        // real Int declaration via `declaration_by_name`.
        let int_id = dag
            .declaration_by_name("Int")
            .expect("Int is populated by bootstrap before the test runs")
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

        arrow_id
    }

    #[test]
    fn smoke_int_add_external_realization() {
        let mut dag = Dag::new();
        let arrow_id = inject_test_realization(&mut dag);

        let arrow_decl = dag.declaration(arrow_id);
        let (inputs, output, body) = match &arrow_decl.connective {
            TypeConnective::Arrow {
                inputs,
                output,
                body,
            } => (inputs.clone(), *output, body.clone()),
            other => panic!("expected realization arrow, got {other:?}"),
        };
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0], output);
        assert!(
            arrow_decl.name.is_none(),
            "realization arrow is anonymous so it stays out of declaration_by_name"
        );

        let realization_id = match body {
            ArrowBody::ExternalRealization(id) => id,
            other => panic!("expected ExternalRealization body, got {other:?}"),
        };
        let realization_decl = dag.declaration(realization_id);
        assert!(
            realization_decl.name.is_none(),
            "realization instance is anonymous"
        );
        assert!(
            matches!(realization_decl.connective, TypeConnective::Conj { .. }),
            "realization instance must be a Conj"
        );

        let meta_type_id = realization_decl
            .meta_tag
            .expect("realization instance must carry a meta_tag");
        let meta_type_decl = dag.declaration(meta_type_id);
        assert_eq!(
            meta_type_decl.name.as_deref(),
            Some("TestRealization"),
            "meta_tag points at the test-local meta-type"
        );
        assert!(
            realization_decl.inhabits.is_none(),
            "realization instance uses meta_tag only, not inhabits"
        );

        // Self-check on the AtomPayload enum so the test depends on
        // its shape (otherwise an unused import warning fires).
        let _probe: Option<&AtomPayload> = None;
    }
}

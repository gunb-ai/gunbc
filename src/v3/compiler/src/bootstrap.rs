// Dag::new() bootstrap.
//
// Parses four minimal fixture modules in dependency order (logic → bit →
// algebra → types) and lowers them into the freshly-created Dag so that the
// declaration table is primed with primitive types, algebraic structures,
// and named primitives (`Int`, `Bool`, `String`) before any user code runs.
//
// After fixture loading, a small set of primitive operator arrows (`+`, `-`,
// ..., `==`, ...) are injected as named Arrow declarations. These are the
// M1(2.5) bridge for user-code operator dispatch: user code emits
// `Call { target: "+" }` and inference looks up the "+" declaration. A full
// §8.9 inhabitance walk (resolve "+" by walking the Int inhabitance chain
// to OrderedRing.add) is deferred to M1(2.6).
//
// Per M1_DESIGN.md §8.6 Risk 2, these fixtures are narrower than the full
// `dsl/std/*.dag` files — the v3 parser doesn't yet handle `data`
// declarations, record literals, `match` expressions, or `module`/`import`
// directives that the production std files use. The fixtures only cover
// what the substrate test and M0 tests exercise.

use crate::dag::{
    ArrowBody, AtomPayload, Dag, Declaration, DeclarationId, TypeConnective,
};
use crate::diagnostics::SourceSpan;
use crate::lower::lower_into;
use crate::parse::parse;
use crate::tokenize::tokenize;

const LOGIC_SUBSET: &str = r#"
type Classical = True | False
"#;

const BIT_SUBSET: &str = r#"
type Bit = Classical
type Word8 { }
type Word16 { }
type Word32 { }
type Word64 { }
"#;

const ALGEBRA_SUBSET: &str = r#"
type Magma<T> {
  op: fn(T, T) -> T
}

type Semigroup<T> {
  op: fn(T, T) -> T
}

type Monoid<T> {
  op: fn(T, T) -> T
  identity: T
}

type Group<T> {
  op: fn(T, T) -> T
  identity: T
  inverse: fn(T) -> T
}

type Ring<T> {
  add: fn(T, T) -> T
  zero: T
  negate: fn(T) -> T
  mul: fn(T, T) -> T
  one: T
}

type OrderedRing<T> {
  add: fn(T, T) -> T
  sub: fn(T, T) -> T
  zero: T
  negate: fn(T) -> T
  mul: fn(T, T) -> T
  one: T
  div: fn(T, T) -> T
  eq: fn(T, T) -> Bool
  ne: fn(T, T) -> Bool
  lt: fn(T, T) -> Bool
  le: fn(T, T) -> Bool
  gt: fn(T, T) -> Bool
  ge: fn(T, T) -> Bool
}

type Ordering = Less | Equal | Greater
"#;

const TYPES_SUBSET: &str = r#"
type Int = OrderedRing<Word64>
type Bool = Classical
type String { }
"#;

pub(crate) fn bootstrap(dag: &mut Dag) {
    for (file, source) in &[
        ("<bootstrap:logic>", LOGIC_SUBSET),
        ("<bootstrap:bit>", BIT_SUBSET),
        ("<bootstrap:algebra>", ALGEBRA_SUBSET),
        ("<bootstrap:types>", TYPES_SUBSET),
    ] {
        parse_and_lower_fixture(dag, source, file);
    }
    inject_primitive_operators(dag);
    inject_realization_stub(dag);
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

/// Add primitive operator arrow declarations after the algebra fixtures
/// have populated `Int` and `Bool`. User-code operator calls target these
/// by name (`"+"`, `"-"`, ...) during lowering.
fn inject_primitive_operators(dag: &mut Dag) {
    let int_id = dag
        .declaration_by_name("Int")
        .expect("bootstrap: Int missing after fixtures")
        .id;
    let bool_id = dag
        .declaration_by_name("Bool")
        .expect("bootstrap: Bool missing after fixtures")
        .id;
    let span = SourceSpan::new("<bootstrap:operators>", 0, 0);

    for op in ["+", "-", "*", "/"] {
        push_operator_arrow(dag, op, &[int_id, int_id], int_id, span.clone());
    }
    for op in ["==", "!=", "<", "<=", ">", ">="] {
        push_operator_arrow(dag, op, &[int_id, int_id], bool_id, span.clone());
    }
}

fn push_operator_arrow(
    dag: &mut Dag,
    name: &str,
    inputs: &[DeclarationId],
    output: DeclarationId,
    span: SourceSpan,
) {
    let id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id,
        name: Some(name.to_string()),
        connective: TypeConnective::Arrow {
            inputs: inputs.to_vec(),
            output,
            body: ArrowBody::Pending,
        },
        meta_tag: None,
        inhabits: None,
        span,
    });
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
        meta_tag: None,
        inhabits: None,
        span,
    });
}

// Re-export AtomPayload so future bootstrap work that adds Atom declarations
// directly doesn't hit an unused-import lint.
#[allow(dead_code)]
type _KeepAtomPayloadLive = AtomPayload;

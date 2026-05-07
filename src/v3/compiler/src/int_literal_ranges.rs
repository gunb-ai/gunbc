//! Integer literal magnitude vs declared integer types — **Q1 consumer**.
//!
//! Authoritative modeling: [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md)
//! §Q1 — `BoundDeclaration = StaticBound(Interval<Int>) | PlatformDependent`, with asymmetric
//! match (target `Unbounded` universal-accept; target `ExactInterval` exact `lo`/`hi` equality at
//! the fold). This module implements only the **static** side needed for literal narrowing:
//! substrate range facts [`range_min_inclusive` / `range_max_inclusive`](../../../../dsl/extdeps/languages/rust/primitives.dag)
//! on [`rust_pilot_primitives`](crate::dag::Dag::rust_pilot_primitives) supply
//! `StaticBound(Interval<Int>)` as [`IntervalInt::ExactInterval`] (decimal endpoints + host
//! `BigInt` comparison; widened from `i128` per R3 Phase A so `u128::MAX` and any future wider
//! primitive is representable structurally). [`IntervalInt::Unbounded`] exists so Q1’s interval
//! algebra is representable when a
//! target declares an unbounded value domain (pilot `IntegerPrimitive` rows are all exact today).
//! [`PlatformDependent`] is out of scope for i64-bounded literal narrowing (deferred targets).
//!
//! ## Downstream consumers (range-facts + narrowing)
//!
//! | Location | Behavior |
//! | --- | --- |
//! | `infer::try_reconcile_int_literal_decision_set` | `let` / `data` pre-seed vs default `Int64` literal; in-range narrow; OOB → `MagnitudeOutOfRange`. |
//! | `infer::decide_transform` (calls) | Parameter-narrow type vs default-`Int` argument literal; narrow or OOB. |
//! | `infer::int_literal_implicit_bind_tolerated_for_expected` | Callable template binding when structural binding fails on int literal. |
//! | `lower` scalar literal lowering | Early reject for OOB literals before inference reunion. |
//!
//! ### R2 downstream audit ([`docs/briefs/r2-modeling-int-lit-magnitude-worker.md`](../../../../docs/briefs/r2-modeling-int-lit-magnitude-worker.md) §Slice 2)
//!
//! Post–[#1227](https://github.com/gunb-ai/gunbc/pull/1227) (**MethodEmitTemplate** Phase 1.5): no shared symbols with `emit_model.dag` method-template row lists
//! or `*_method_template_contracts` — those paths do **not** consult `rust_pilot_primitives` / this module.
//!
//! | Consumer | Range-facts / Q1 magnitude |
//! | --- | --- |
//! | `infer` (`try_reconcile_int_literal_decision_set`, `decide_transform`, `int_literal_implicit_bind_tolerated_for_expected`, transform/`PortUnion` narrow paths) | **Yes** — `integer_range_for_decl` + `magnitude_out_of_range_for_interval`. |
//! | `lower` (`lower_scalar_literal` / scalar literal outcome) | **Yes** — same facts for early OOB. |
//! | `emit::*` (e.g. `rust_target`) | **Indirect** — emits **resolved** shapes after inference; no parallel range-fact walk. |
//! | Method-template / emit-model carriers (**#1227**) | **N/A** — template contracts; unrelated to int-literal narrowing. |

use std::collections::HashSet;

use num_bigint::BigInt;

use crate::dag::{
    AtomPayload, Behavior, Dag, DeclarationId, FieldValue, LiteralBits, PortId, TypeConnective,
    ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::types::TypeShape;

/// Q1 `Interval<Int>` instance carried from String-decimal range facts (not `LiteralBits::Int`
/// widening — producer brief).
///
/// **Host repr:** `min` / `max` are arbitrary-precision `BigInt` so `u128::MAX` (and any future
/// wider-than-i128 primitive) is representable structurally without per-width host-repr variant
/// explosion. Per Director Path A RATIFIED at gunbc#1739 #issuecomment-4392731264 (R3 Substrate
/// Rust-primitive-full-coverage bundled brief). The previous narrow `i128` host-repr deferred the
/// `u128` row in `dsl/extdeps/languages/rust/primitives.dag` because `u128::MAX` exceeds `i128`
/// range; that gap is closed by the `BigInt` host repr here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IntervalInt {
    /// Closed interval — substrate `range_*_inclusive` facts for a fixed-width target primitive.
    ExactInterval {
        target_name: String,
        min_decimal: String,
        max_decimal: String,
        min: BigInt,
        max: BigInt,
    },
    /// Value-domain unbounded integer (e.g. arbitrary-precision target). Universal-accept for any
    /// i64-representable literal magnitude.
    ///
    /// **Dissolution trigger (when this variant is constructed from [`integer_range_for_decl`]):**
    /// a `rust_pilot_primitives` `IntegerPrimitive` row (or successor multi-target table) is
    /// authored for a target whose Q1 `BoundDeclaration` is `StaticBound(Unbounded)` at magnitude
    /// check — e.g. Python `int` per [`docs/design-emission-model.md`](../../../../docs/design-emission-model.md)
    /// fold example (T-Ground cross-target / language `primitives.dag` work, not this consumer).
    /// Until that producer exists, only [`IntervalInt::ExactInterval`] is returned from the pilot
    /// list; [`Unbounded`] remains for `contains_i64` / Q1 algebra completeness and unit tests.
    #[allow(dead_code)]
    Unbounded,
}

/// Decimal endpoints for a **fixed** integer target — the payload of [`MagnitudeOutOfRange`].
///
/// **API split:** only [`magnitude_out_of_range`] takes this type. When you have a full
/// [`IntervalInt`] from [`integer_range_for_decl`], call [`magnitude_out_of_range_for_interval`]
/// (it uses [`IntervalInt::exact_interval_facts`] and never passes an unbounded domain into the
/// magnitude diagnostic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactIntIntervalFacts {
    pub(crate) target_name: String,
    pub(crate) min_decimal: String,
    pub(crate) max_decimal: String,
}

impl IntervalInt {
    /// Reconciliation receives only literals that already fit [`LiteralBits::Int(i64)`]. Declared
    /// range facts may exceed `i64` (e.g. `u64::MAX`); literals above `i64::MAX` are rejected at
    /// tokenization until the deferred Int128 carrier lane lands.
    pub(crate) fn contains_i64(&self, value: i64) -> bool {
        let value = BigInt::from(value);
        match self {
            IntervalInt::Unbounded => true,
            IntervalInt::ExactInterval { min, max, .. } => *min <= value && value <= *max,
        }
    }

    /// Fixed-width pilot rows: `Some` for [`IntervalInt::ExactInterval`]; `None` for
    /// [`IntervalInt::Unbounded`] (no decimal range to quote in `MagnitudeOutOfRange`).
    pub(crate) fn exact_interval_facts(&self) -> Option<ExactIntIntervalFacts> {
        match self {
            IntervalInt::ExactInterval {
                target_name,
                min_decimal,
                max_decimal,
                ..
            } => Some(ExactIntIntervalFacts {
                target_name: target_name.clone(),
                min_decimal: min_decimal.clone(),
                max_decimal: max_decimal.clone(),
            }),
            IntervalInt::Unbounded => None,
        }
    }
}

pub(crate) enum IntegerRangeLookup {
    Found(IntervalInt),
    Missing,
    Invalid(Diagnostic),
}

/// Structural witness for routing integer literals: `IntegerAlgebra` and
/// `TargetCarrier` variant **payload type** ids (the `constructor` field on
/// `FieldValue::Variant`), derived from std `OrderedRing<C>` / `Semiring<C>`
/// by `DeclarationId` equality on template and carrier type declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntegerRoutingWitness {
    pub(crate) algebra_variant_ty: DeclarationId,
    pub(crate) carrier_variant_ty: DeclarationId,
}

/// Integration / structural tests: witness for `decl`'s resolved integer
/// instantiation (`OrderedRing` / `Semiring` + word carrier), if any.
pub(crate) fn integer_routing_witness_for_decl(
    dag: &Dag,
    decl: DeclarationId,
) -> Option<IntegerRoutingWitness> {
    integer_routing_witness_walk(dag, decl, 0)
}

/// `Nat` (`dsl/std/nat.dag`: `Semiring<Magnitude>`). Decimal literals narrow like nonnegative
/// fixed-width ints: \([0, i64::MAX]\) until a distinct magnitude literal carrier ships.
fn type_is_nat(dag: &Dag, mut decl: DeclarationId) -> bool {
    let Some(nat_id) = dag.declaration_by_name("Nat").map(|d| d.id) else {
        return false;
    };
    let Some(semiring_template) = dag.declaration_by_name("Semiring").map(|d| d.id) else {
        return false;
    };
    let Some(magnitude_id) = dag.declaration_by_name("Magnitude").map(|d| d.id) else {
        return false;
    };
    for _ in 0..32 {
        if decl == nat_id {
            return true;
        }
        let declaration = dag.declaration(decl);
        match &declaration.connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } if *template == semiring_template
                && arguments.len() == 1
                && arguments[0].value == magnitude_id =>
            {
                return true;
            }
            TypeConnective::Instantiation { template, .. } => decl = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => decl = *next,
            _ => return false,
        }
    }
    false
}

fn nat_decimal_literal_interval() -> IntervalInt {
    IntervalInt::ExactInterval {
        target_name: "Nat".to_string(),
        min_decimal: "0".to_string(),
        max_decimal: i64::MAX.to_string(),
        min: BigInt::from(0),
        max: BigInt::from(i64::MAX),
    }
}

pub(crate) fn integer_range_for_decl(dag: &Dag, decl: DeclarationId) -> IntegerRangeLookup {
    if type_is_nat(dag, decl) {
        return IntegerRangeLookup::Found(nat_decimal_literal_interval());
    }
    let Some(witness) = integer_routing_witness_walk(dag, decl, 0) else {
        return IntegerRangeLookup::Missing;
    };
    let Some(pilot) = dag.rust_pilot_primitives() else {
        return IntegerRangeLookup::Missing;
    };
    let Some(body) = pilot.value_body.as_ref() else {
        return IntegerRangeLookup::Missing;
    };
    let ValueBody::List(elements) = body else {
        return IntegerRangeLookup::Missing;
    };

    let integer_primitive_ctor = match rust_primitive_integer_variant_ty(dag) {
        Some(id) => id,
        None => {
            return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
                "bootstrap: RustPrimitive.IntegerPrimitive variant type is unavailable".to_string(),
                pilot.span.clone(),
            ));
        }
    };

    let mut matches: Vec<PilotIntegerMatch> = Vec::new();
    for element in elements {
        let FieldValue::Variant {
            constructor,
            payload,
        } = element
        else {
            continue;
        };
        if *constructor != integer_primitive_ctor {
            continue;
        }
        match pilot_integer_row(witness, payload, pilot.span.clone()) {
            Ok(Some(m)) => matches.push(m),
            Ok(None) => {}
            Err(diag) => return IntegerRangeLookup::Invalid(diag),
        }
    }

    if matches.is_empty() {
        return IntegerRangeLookup::Missing;
    }
    if let Some(row) = matches.iter().find(|row| row.range.is_none()) {
        return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
            format!(
                "malformed rust_pilot_primitives IntegerPrimitive row for routing witness {:?}; integer literal range narrowing is unavailable",
                (witness.algebra_variant_ty, witness.carrier_variant_ty)
            ),
            row.span.clone(),
        ));
    }
    if matches.len() > 1 {
        return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
            format!(
                "duplicate rust_pilot_primitives IntegerPrimitive rows for routing witness {:?}; integer literal range narrowing is ambiguous",
                (witness.algebra_variant_ty, witness.carrier_variant_ty)
            ),
            matches[1].span.clone(),
        ));
    }
    let row = matches.remove(0);
    match row.range {
        Some(range) => IntegerRangeLookup::Found(range),
        None => unreachable!("malformed matching rows are handled before duplicate detection"),
    }
}

struct PilotIntegerMatch {
    range: Option<IntervalInt>,
    span: SourceSpan,
}

fn integer_routing_witness_walk(
    dag: &Dag,
    decl: DeclarationId,
    depth: usize,
) -> Option<IntegerRoutingWitness> {
    if depth >= 32 {
        return None;
    }
    let declaration = dag.declaration(decl);
    match &declaration.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByName(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
            integer_routing_witness_walk(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            if arguments.is_empty() {
                return integer_routing_witness_walk(dag, *template, depth + 1);
            }
            let carrier = arguments.first()?.value;
            integer_instantiation_witness(dag, *template, carrier)
        }
        _ => None,
    }
}

fn integer_instantiation_witness(
    dag: &Dag,
    template: DeclarationId,
    carrier_decl: DeclarationId,
) -> Option<IntegerRoutingWitness> {
    let ordered_ring = dag.declaration_by_name("OrderedRing")?.id;
    let semiring = dag.declaration_by_name("Semiring")?.id;
    let algebra_variant_ty = if template == ordered_ring {
        disj_variant_payload_ty(dag, "IntegerAlgebra", "OrderedRingAlgebra")?
    } else if template == semiring {
        disj_variant_payload_ty(dag, "IntegerAlgebra", "SemiringAlgebra")?
    } else {
        return None;
    };
    let carrier_variant_ty = std_word_carrier_to_target_carrier_variant_ty(dag, carrier_decl)?;
    Some(IntegerRoutingWitness {
        algebra_variant_ty,
        carrier_variant_ty,
    })
}

fn disj_variant_payload_ty(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
) -> Option<DeclarationId> {
    let decl = dag.declaration_by_name(sum_name)?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return None;
    };
    variants
        .iter()
        .find(|v| v.label == variant_label)
        .map(|v| v.ty)
}

fn std_word_carrier_to_target_carrier_variant_ty(
    dag: &Dag,
    carrier_decl: DeclarationId,
) -> Option<DeclarationId> {
    let byte = dag.declaration_by_name("Byte")?.id;
    let word16 = dag.declaration_by_name("Word16")?.id;
    let word32 = dag.declaration_by_name("Word32")?.id;
    let word64 = dag.declaration_by_name("Word64")?.id;
    let word128 = dag.declaration_by_name("Word128")?.id;
    let label = if carrier_decl == byte {
        "ByteCarrier"
    } else if carrier_decl == word16 {
        "Word16Carrier"
    } else if carrier_decl == word32 {
        "Word32Carrier"
    } else if carrier_decl == word64 {
        "Word64Carrier"
    } else if carrier_decl == word128 {
        "Word128Carrier"
    } else {
        return None;
    };
    disj_variant_payload_ty(dag, "TargetCarrier", label)
}

fn rust_primitive_integer_variant_ty(dag: &Dag) -> Option<DeclarationId> {
    let rust_primitive = dag.declaration_by_name("RustPrimitive")?;
    let TypeConnective::Disj { variants } = &rust_primitive.connective else {
        return None;
    };
    variants
        .iter()
        .find(|v| v.label == "IntegerPrimitive")
        .map(|v| v.ty)
}

fn pilot_integer_row(
    witness: IntegerRoutingWitness,
    payload: &[FieldValue],
    default_span: SourceSpan,
) -> Result<Option<PilotIntegerMatch>, Diagnostic> {
    // IntegerPrimitive field order: target_name, algebra, carrier, range_*, is_copy, overflow
    if payload.len() < 5 {
        return Ok(None);
    }
    let FieldValue::Variant {
        constructor: algebra_ctor,
        ..
    } = &payload[1]
    else {
        return Err(malformed_integer_range_fact(
            "rust_pilot_primitives IntegerPrimitive `algebra` must be a variant value".to_string(),
            default_span.clone(),
        ));
    };
    let FieldValue::Variant {
        constructor: carrier_ctor,
        ..
    } = &payload[2]
    else {
        return Err(malformed_integer_range_fact(
            "rust_pilot_primitives IntegerPrimitive `carrier` must be a variant value".to_string(),
            default_span.clone(),
        ));
    };
    if *algebra_ctor != witness.algebra_variant_ty || *carrier_ctor != witness.carrier_variant_ty {
        return Ok(None);
    }

    let range = (|| {
        let min_decimal = literal_string(payload.get(3)?)?;
        let max_decimal = literal_string(payload.get(4)?)?;
        let min = min_decimal.parse().ok()?;
        let max = max_decimal.parse().ok()?;
        if min > max {
            return None;
        }
        Some(IntervalInt::ExactInterval {
            target_name: literal_string(payload.first()?)?,
            min_decimal,
            max_decimal,
            min,
            max,
        })
    })();

    Ok(Some(PilotIntegerMatch {
        range,
        span: default_span,
    }))
}

fn literal_string(value: &FieldValue) -> Option<String> {
    match value {
        FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
        _ => None,
    }
}

pub(crate) fn literal_int_at(dag: &Dag, port: PortId) -> Option<i64> {
    match dag.resolve_producer_opt(&port)? {
        Behavior::Value(value) => match &value.data {
            LiteralBits::Int(n) => Some(*n),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn int_literal_fits_expected_type(
    dag: &Dag,
    literal: i64,
    expected: DeclarationId,
) -> Result<Option<bool>, Diagnostic> {
    match integer_range_for_decl(dag, expected) {
        IntegerRangeLookup::Found(bound) => Ok(Some(bound.contains_i64(literal))),
        IntegerRangeLookup::Missing => Ok(None),
        IntegerRangeLookup::Invalid(diag) => Err(diag),
    }
}

/// Build [`Diagnostic::MagnitudeOutOfRange`] from **exact** decimal range facts only.
///
/// For a bound that may be [`IntervalInt::Unbounded`], use [`magnitude_out_of_range_for_interval`]
/// instead — this function does not accept [`IntervalInt`].
pub(crate) fn magnitude_out_of_range(
    literal: i64,
    expected: TypeShape,
    facts: ExactIntIntervalFacts,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::MagnitudeOutOfRange {
        literal: literal.to_string(),
        target: facts.target_name,
        range_min_inclusive: facts.min_decimal,
        range_max_inclusive: facts.max_decimal,
        expected,
        span,
        fixes: Vec::new(),
    }
}

/// OOB diagnostic when the only available model is [`IntervalInt`] (e.g. from
/// [`integer_range_for_decl`]). Unbounded domains never produce `MagnitudeOutOfRange` for
/// i64-bounded literals; if that combination appears, fail closed without `unreachable!`.
pub(crate) fn magnitude_out_of_range_for_interval(
    literal: i64,
    expected: TypeShape,
    bound: IntervalInt,
    span: SourceSpan,
) -> Diagnostic {
    match bound.exact_interval_facts() {
        Some(facts) => magnitude_out_of_range(literal, expected, facts, span),
        None => Diagnostic::ResolveError {
            name: "internal: integer literal failed range check but target has no exact interval facts"
                .to_string(),
            span,
            fixes: Vec::new(),
        },
    }
}

fn malformed_integer_range_fact(message: String, span: SourceSpan) -> Diagnostic {
    Diagnostic::MalformedIntegerRangeFact {
        message,
        span,
        fixes: Vec::new(),
    }
}

/// Bootstrap-only gate: walk **every** `IntegerPrimitive` row in
/// `rust_pilot_primitives` and fail closed if any row is structurally
/// ill-formed, range strings do not parse to `i128`, `min > max`, or
/// `(algebra, carrier)` witness pairs collide.
///
/// Call this once when constructing the extdeps-including bootstrapped `Dag`
/// so drift or corruption in the pilot list surfaces at `Dag::new()`,
/// not only when a particular std type is queried.
pub(crate) fn validate_rust_pilot_integer_primitives(dag: &mut Dag) {
    // T-Int128 Slice B1: pilot extended to 9 IntegerPrimitive rows (i8..i64,
    // i128, u8..u64). u128 lands in Slice B2 once `IntervalInt::ExactInterval`
    // widens past host i128.
    const EXPECTED_INTEGER_ROWS: usize = 9;
    const INTEGER_PRIMITIVE_FIELD_COUNT: usize = 7;

    enum PilotListSnapshot {
        List(Vec<FieldValue>),
        MissingBody,
        NotList,
    }

    // Authority file for span when the declaration is absent (extdeps fixture).
    const RUST_PILOT_PRIMITIVES_AUTHORITY: &str = "dsl/extdeps/languages/rust/primitives.dag";

    let (default_span, pilot_elements) = {
        let Some(pilot) = dag.rust_pilot_primitives() else {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "bootstrap: `rust_pilot_primitives` is missing from the extdeps fixture; \
                 integer range authority is unavailable (expected extdeps `primitives.dag` load)"
                    .to_string(),
                SourceSpan::new(RUST_PILOT_PRIMITIVES_AUTHORITY, 0, 0),
            ));
            return;
        };
        let sp = pilot.span.clone();
        let snap = match pilot.value_body.as_ref() {
            None => PilotListSnapshot::MissingBody,
            Some(ValueBody::List(els)) => PilotListSnapshot::List(els.clone()),
            Some(_) => PilotListSnapshot::NotList,
        };
        (sp, snap)
    };

    let elements: &[FieldValue] = match &pilot_elements {
        PilotListSnapshot::List(els) => els,
        PilotListSnapshot::MissingBody => {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "bootstrap: rust_pilot_primitives is missing a value body".to_string(),
                default_span,
            ));
            return;
        }
        PilotListSnapshot::NotList => {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "bootstrap: rust_pilot_primitives must be ValueBody::List".to_string(),
                default_span,
            ));
            return;
        }
    };

    let Some(integer_ctor) = rust_primitive_integer_variant_ty(dag) else {
        dag.attach_diagnostic(malformed_integer_range_fact(
            "bootstrap: RustPrimitive.IntegerPrimitive variant is unavailable".to_string(),
            default_span,
        ));
        return;
    };

    let Some(ord_algebra) = disj_variant_payload_ty(dag, "IntegerAlgebra", "OrderedRingAlgebra")
    else {
        dag.attach_diagnostic(malformed_integer_range_fact(
            "bootstrap: IntegerAlgebra.OrderedRingAlgebra is unavailable for pilot validation"
                .to_string(),
            default_span,
        ));
        return;
    };
    let Some(sem_algebra) = disj_variant_payload_ty(dag, "IntegerAlgebra", "SemiringAlgebra")
    else {
        dag.attach_diagnostic(malformed_integer_range_fact(
            "bootstrap: IntegerAlgebra.SemiringAlgebra is unavailable for pilot validation"
                .to_string(),
            default_span,
        ));
        return;
    };
    let mut allowed_carriers: HashSet<DeclarationId> = HashSet::new();
    for label in [
        "ByteCarrier",
        "Word16Carrier",
        "Word32Carrier",
        "Word64Carrier",
        "Word128Carrier",
    ] {
        if let Some(c) = disj_variant_payload_ty(dag, "TargetCarrier", label) {
            allowed_carriers.insert(c);
        }
    }
    if allowed_carriers.is_empty() {
        dag.attach_diagnostic(malformed_integer_range_fact(
            "bootstrap: no TargetCarrier word variant payload types for pilot validation"
                .to_string(),
            default_span,
        ));
        return;
    }

    let mut witnesses: HashSet<(DeclarationId, DeclarationId)> = HashSet::new();
    let mut integer_rows: usize = 0;

    for element in elements {
        let FieldValue::Variant {
            constructor,
            payload,
        } = element
        else {
            continue;
        };
        if *constructor != integer_ctor {
            continue;
        }
        integer_rows += 1;

        if payload.len() < INTEGER_PRIMITIVE_FIELD_COUNT {
            dag.attach_diagnostic(malformed_integer_range_fact(
                format!(
                    "rust_pilot_primitives IntegerPrimitive row has {} fields; expected {}",
                    payload.len(),
                    INTEGER_PRIMITIVE_FIELD_COUNT
                ),
                default_span.clone(),
            ));
            continue;
        }

        let FieldValue::Variant {
            constructor: algebra_ctor,
            ..
        } = &payload[1]
        else {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `algebra` must be a variant value"
                    .to_string(),
                default_span.clone(),
            ));
            continue;
        };
        if *algebra_ctor != ord_algebra && *algebra_ctor != sem_algebra {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `algebra` must be OrderedRingAlgebra or SemiringAlgebra (variant payload type id)".to_string(),
                default_span.clone(),
            ));
            continue;
        }

        let FieldValue::Variant {
            constructor: carrier_ctor,
            ..
        } = &payload[2]
        else {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `carrier` must be a variant value"
                    .to_string(),
                default_span.clone(),
            ));
            continue;
        };
        if !allowed_carriers.contains(carrier_ctor) {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `carrier` must be a word TargetCarrier (Byte/Word16/Word32/Word64/Word128) variant payload type id".to_string(),
                default_span.clone(),
            ));
            continue;
        }

        if literal_string(&payload[0]).is_none() {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `target_name` must be a string literal"
                    .to_string(),
                default_span.clone(),
            ));
            continue;
        }

        let (min_s, max_s) = match (literal_string(&payload[3]), literal_string(&payload[4])) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                dag.attach_diagnostic(malformed_integer_range_fact(
                    "rust_pilot_primitives IntegerPrimitive range bounds must be string literals"
                        .to_string(),
                    default_span.clone(),
                ));
                continue;
            }
        };

        if !matches!(payload[5], FieldValue::Literal(LiteralBits::Bool(_))) {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `is_copy` must be a bool literal"
                    .to_string(),
                default_span.clone(),
            ));
            continue;
        }

        if !matches!(&payload[6], FieldValue::Variant { .. }) {
            dag.attach_diagnostic(malformed_integer_range_fact(
                "rust_pilot_primitives IntegerPrimitive `overflow` must be a variant value"
                    .to_string(),
                default_span.clone(),
            ));
            continue;
        }

        let (min_n, max_n) = match (min_s.parse::<BigInt>(), max_s.parse::<BigInt>()) {
            (Ok(mn), Ok(mx)) => (mn, mx),
            _ => {
                dag.attach_diagnostic(malformed_integer_range_fact(
                    format!(
                        "rust_pilot_primitives IntegerPrimitive range [{min_s}, {max_s}] must parse as a decimal integer"
                    ),
                    default_span.clone(),
                ));
                continue;
            }
        };
        if min_n > max_n {
            dag.attach_diagnostic(malformed_integer_range_fact(
                format!(
                    "rust_pilot_primitives IntegerPrimitive range order invalid: min {min_s} > max {max_s}"
                ),
                default_span.clone(),
            ));
            continue;
        }

        if !witnesses.insert((*algebra_ctor, *carrier_ctor)) {
            dag.attach_diagnostic(malformed_integer_range_fact(
                format!(
                    "duplicate rust_pilot_primitives IntegerPrimitive (algebra, carrier) witness: ({algebra_ctor:?}, {carrier_ctor:?})"
                ),
                default_span.clone(),
            ));
        }
    }

    if integer_rows != EXPECTED_INTEGER_ROWS {
        dag.attach_diagnostic(malformed_integer_range_fact(
            format!(
                "rust_pilot_primitives must list exactly {EXPECTED_INTEGER_ROWS} IntegerPrimitive rows (pilot int scope); found {integer_rows}"
            ),
            default_span,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeShape;

    #[test]
    fn magnitude_out_of_range_accepts_only_exact_int_interval_facts() {
        let dag = Dag::new();
        let u8_decl = dag.declaration_by_name("UInt8").expect("UInt8").id;
        let d = magnitude_out_of_range(
            256,
            TypeShape::new(u8_decl),
            ExactIntIntervalFacts {
                target_name: "u8".to_string(),
                min_decimal: "0".to_string(),
                max_decimal: "255".to_string(),
            },
            SourceSpan::new("t.v3", 0, 0),
        );
        assert!(
            matches!(
                d,
                Diagnostic::MagnitudeOutOfRange {
                    ref literal,
                    ref target,
                    ..
                } if literal == "256" && target == "u8"
            ),
            "expected MagnitudeOutOfRange, got {d:?}"
        );
    }

    #[test]
    fn magnitude_out_of_range_unbounded_target_fails_closed_with_resolve_error() {
        let dag = Dag::new();
        let int_decl = dag.declaration_by_name("Int").expect("Int in bootstrap").id;
        let d = magnitude_out_of_range_for_interval(
            0,
            TypeShape::new(int_decl),
            IntervalInt::Unbounded,
            SourceSpan::new("t.v3", 0, 0),
        );
        assert!(
            matches!(d, Diagnostic::ResolveError { .. }),
            "expected fail-closed ResolveError, got {d:?}"
        );
    }

    #[test]
    fn interval_int_unbounded_accepts_all_i64_literals() {
        assert!(IntervalInt::Unbounded.contains_i64(i64::MIN));
        assert!(IntervalInt::Unbounded.contains_i64(i64::MAX));
    }

    #[test]
    fn int128_witness_matches_i128_pilot_row_constructors() {
        // T-Int128 Slice B1: signed Int128 -> i128 pilot row via Word128Carrier.
        let dag = Dag::new();
        assert!(
            dag.diagnostics().is_empty(),
            "bootstrap diagnostics: {:?}",
            dag.diagnostics()
        );
        let int128 = dag
            .declaration_by_name("Int128")
            .expect("Int128 in bootstrap")
            .id;
        let witness = integer_routing_witness_for_decl(&dag, int128).expect("Int128 witness");
        let pilot = dag.rust_pilot_primitives().expect("pilot");
        let ValueBody::List(elements) = pilot.value_body.as_ref().expect("list body") else {
            panic!("expected list");
        };
        let integer_primitive_ctor = rust_primitive_integer_variant_ty(&dag).expect("ctor");
        let mut matched = 0usize;
        for element in elements {
            let FieldValue::Variant {
                constructor,
                payload,
            } = element
            else {
                continue;
            };
            if *constructor != integer_primitive_ctor {
                continue;
            }
            let FieldValue::Variant { constructor: a, .. } = &payload[1] else {
                continue;
            };
            let FieldValue::Variant { constructor: c, .. } = &payload[2] else {
                continue;
            };
            if *a == witness.algebra_variant_ty && *c == witness.carrier_variant_ty {
                matched += 1;
            }
        }
        assert_eq!(
            matched, 1,
            "exactly one pilot IntegerPrimitive row for Int128 witness"
        );
    }

    #[test]
    fn int128_range_lookup_accepts_all_i64_literals() {
        // i128 row's range covers all i64 magnitudes; reconciliation passes
        // any i64 literal through `contains_i64`.
        let dag = Dag::new();
        let int128 = dag
            .declaration_by_name("Int128")
            .expect("Int128 in bootstrap")
            .id;
        match integer_range_for_decl(&dag, int128) {
            IntegerRangeLookup::Found(bound) => {
                assert!(bound.contains_i64(i64::MIN));
                assert!(bound.contains_i64(0));
                assert!(bound.contains_i64(i64::MAX));
            }
            IntegerRangeLookup::Missing => panic!("expected Found range for Int128, got Missing"),
            IntegerRangeLookup::Invalid(d) => {
                panic!("expected Found range for Int128, got Invalid: {d:?}")
            }
        }
    }

    #[test]
    fn uint8_witness_matches_u8_pilot_row_constructors() {
        let dag = Dag::new();
        assert!(
            dag.diagnostics().is_empty(),
            "bootstrap diagnostics: {:?}",
            dag.diagnostics()
        );
        let uint8 = dag
            .declaration_by_name("UInt8")
            .expect("UInt8 in bootstrap")
            .id;
        let witness = integer_routing_witness_for_decl(&dag, uint8).expect("UInt8 witness");
        let pilot = dag.rust_pilot_primitives().expect("pilot");
        let ValueBody::List(elements) = pilot.value_body.as_ref().expect("list body") else {
            panic!("expected list");
        };
        let integer_primitive_ctor = rust_primitive_integer_variant_ty(&dag).expect("ctor");
        let mut matched = 0usize;
        for element in elements {
            let FieldValue::Variant {
                constructor,
                payload,
            } = element
            else {
                continue;
            };
            if *constructor != integer_primitive_ctor {
                continue;
            }
            let FieldValue::Variant { constructor: a, .. } = &payload[1] else {
                continue;
            };
            let FieldValue::Variant { constructor: c, .. } = &payload[2] else {
                continue;
            };
            if *a == witness.algebra_variant_ty && *c == witness.carrier_variant_ty {
                matched += 1;
            }
        }
        assert_eq!(
            matched, 1,
            "exactly one pilot IntegerPrimitive row for UInt8 witness"
        );
    }
}

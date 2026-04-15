// Structural operator representation.
//
// **🟡 Scaffold — M1(2.7).** `OperatorKind` is a parse-time
// discriminator that bridges the surface operator grammar to
// structural algebra field dispatch in infer.rs. It is explicitly
// NOT terminal: the richer source (algebra field declarations
// in `std/algebra.dag`) already exists in the declaration graph,
// and the long-term shape has operators desugaring to plain
// algebra-field `Callable` calls with no parallel Rust enum.
//
// **Dissolution receipt — Q3/Q4 + R9 operator dispatch.** Before
// M1(2.7) this module owned `OPERATOR_FIELD_MAP`, a name-based
// bridge that infer.rs read twice: once to decide "is this target
// an operator?" (inspecting `AtomPayload::UnresolvedIdentifier(String)`)
// and once to decide "arithmetic or comparison?" (via a hardcoded
// `is_comparison_operator` string match). That shape put the
// operator dispatch fact in two places with the string as the
// discriminator.
//
// M1(2.7) replaced it with this structural coproduct. `OperatorKind`
// is a closed set of 10 binary operators split into `ArithmeticOp`
// (returns operand type) and `ComparisonOp` (returns Bool). The
// parser commits at parse time to a variant; downstream code
// dispatches on the variant, not on a string. In a second pass
// (ChatGPT review R9), infer.rs was rewired to walk the LHS type's
// algebra chain and read the operator's signature from the actual
// `std/algebra.dag` field (e.g., `OrderedRing.add`), rather than
// fabricating `(T, T) -> T` in Rust. The enum now acts as a
// typed *lookup key* into algebra declarations, not as a parallel
// authority that says what the signature should be.
//
// The terminal form (M2+) removes the enum: when the surface
// grammar adopts explicit algebra field access (e.g., writing
// `Int.add(a, b)` or a desugaring pass that rewrites `1 + 2` to
// that form at parse time), `SurfaceExpr::Operator`,
// `TransformTarget::Operator`, and this whole file dissolve.
// Operators become plain `TransformTarget::Callable` dispatches
// through the ordinary `resolve_arrow` walk.
//
// Consumers:
//   - `parse.rs` builds `SurfaceExpr::Operator { op, args, span }`
//     when it sees `+ - * / == != < <= > >=`.
//   - `lower.rs` lowers `SurfaceExpr::Operator` to a `TransformNode`
//     whose `target: TransformTarget::Operator(OperatorKind)`.
//   - `infer.rs::resolve_operator_arrow` uses
//     `OperatorKind::algebra_field_name()` as the lookup key into
//     the LHS type's algebra Conj; reads the field's Arrow from
//     the declaration graph; substitutes the receiver type param
//     to the source declaration.
//   - `lower.rs::descent_provable` checks for
//     `SurfaceExpr::Operator { op: ArithmeticOp::Sub, .. }` when
//     verifying structural descent on recursive self-calls. Same
//     discriminator-not-authority framing.
//
// 4-pattern check on `OperatorKind`:
// - Pattern 1 (fact placement): the signature fact lives in
//   `std/algebra.dag` algebra fields, not here. This enum is only
//   the parse→dispatch discriminator — no parallel facts.
// - Pattern 2 (variant-is-data): label-only; no data.
// - Pattern 3 (algebraic form): fails. The algebraic facts are
//   in the algebra declarations.
// - Pattern 4 (dimensional): fails.
//
// Verdict: **🟡 scaffold with named dissolution trigger.** Not
// terminal because the richer source exists in std/algebra.dag
// today. Dissolves when the M2+ parser / desugarer replaces
// `SurfaceExpr::Operator` with direct algebra-field `Call`s.

/// Arithmetic binary operators — the four variants of `+`, `-`,
/// `*`, `/`.
///
/// **🟡 Scaffold — inherits the outer `OperatorKind` receipt.**
/// Same dissolution trigger as `OperatorKind`: when the M2+ parser
/// desugars `a + b` to direct algebra-field `Call`s (or adds
/// explicit `Int.add(a, b)` syntax), this enum disappears along
/// with `SurfaceExpr::Operator` and `TransformTarget::Operator`.
/// No independent dissolution path — the three operator enums
/// rise and fall together.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): label-only; each variant is a
///   discriminator for `OperatorKind::algebra_field_name()` /
///   `symbol()`. The signature itself lives in
///   `std/algebra.dag`'s `OrderedRing<T>` `add`/`sub`/`mul`/`div`
///   fields, not here.
/// - Pattern 2 (variant-is-data): fails. No payloads; these are
///   tag-only variants.
/// - Pattern 3 (algebraic form): partial. The four arithmetic
///   ops partition the `Ring`-level primitive operations by
///   role. The set grows only if the algebra does.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: 🟡 scaffold inheriting `OperatorKind`'s trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Comparison binary operators — the six variants of `==`, `!=`,
/// `<`, `<=`, `>`, `>=`.
///
/// **🟡 Scaffold — inherits the outer `OperatorKind` receipt.**
/// Same dissolution trigger as `ArithmeticOp` and `OperatorKind`:
/// M2+ parser desugaring replaces `SurfaceExpr::Operator` with
/// direct algebra-field calls, and this enum disappears.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): label-only; the arm is a
///   discriminator for `OperatorKind::algebra_field_name()`
///   against `OrderedRing.{eq, ne, lt, le, gt, ge}` fields.
///   Signature and semantics live in `std/algebra.dag`.
/// - Pattern 2 (variant-is-data): fails. No payloads.
/// - Pattern 3 (algebraic form): partial. Six variants cover
///   the total-order relations on an ordered ring. `eq`/`ne`
///   come from equality; `lt`/`le`/`gt`/`ge` from the order.
///   Growing this set would require a new algebraic structure
///   in algebra.dag (Lattice gives `meet`/`join`, not `lt`).
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: 🟡 scaffold inheriting `OperatorKind`'s trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Top-level operator kind. Structural dispatch target for the
/// `TransformTarget::Operator` variant. The split between
/// `Arithmetic` and `Comparison` encodes the output-type rule:
/// arithmetic returns the operand type, comparison returns Bool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatorKind {
    Arithmetic(ArithmeticOp),
    Comparison(ComparisonOp),
}

impl OperatorKind {
    /// Translate a source symbol to a structural `OperatorKind`. Used
    /// at parse time to commit to the enum variant as early as
    /// possible, so downstream code never re-parses the symbol
    /// string. Returns `None` for non-operator identifiers.
    pub fn from_symbol(symbol: &str) -> Option<OperatorKind> {
        match symbol {
            "+" => Some(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            "-" => Some(OperatorKind::Arithmetic(ArithmeticOp::Sub)),
            "*" => Some(OperatorKind::Arithmetic(ArithmeticOp::Mul)),
            "/" => Some(OperatorKind::Arithmetic(ArithmeticOp::Div)),
            "==" => Some(OperatorKind::Comparison(ComparisonOp::Eq)),
            "!=" => Some(OperatorKind::Comparison(ComparisonOp::Ne)),
            "<" => Some(OperatorKind::Comparison(ComparisonOp::Lt)),
            "<=" => Some(OperatorKind::Comparison(ComparisonOp::Le)),
            ">" => Some(OperatorKind::Comparison(ComparisonOp::Gt)),
            ">=" => Some(OperatorKind::Comparison(ComparisonOp::Ge)),
            _ => None,
        }
    }

    /// Human-readable source symbol for this operator. Used by
    /// diagnostics when the compiler needs to display the operator
    /// to the user. Total inverse of `from_symbol`.
    pub fn symbol(self) -> &'static str {
        match self {
            OperatorKind::Arithmetic(ArithmeticOp::Add) => "+",
            OperatorKind::Arithmetic(ArithmeticOp::Sub) => "-",
            OperatorKind::Arithmetic(ArithmeticOp::Mul) => "*",
            OperatorKind::Arithmetic(ArithmeticOp::Div) => "/",
            OperatorKind::Comparison(ComparisonOp::Eq) => "==",
            OperatorKind::Comparison(ComparisonOp::Ne) => "!=",
            OperatorKind::Comparison(ComparisonOp::Lt) => "<",
            OperatorKind::Comparison(ComparisonOp::Le) => "<=",
            OperatorKind::Comparison(ComparisonOp::Gt) => ">",
            OperatorKind::Comparison(ComparisonOp::Ge) => ">=",
        }
    }

    /// The algebra field name this operator dispatches to. Operator
    /// resolution in `infer::resolve_operator_arrow` walks the source
    /// type's instantiation chain to an algebra Conj declaration,
    /// then looks up the field by this name to read the Arrow
    /// signature. The field→operator mapping is canonical: every
    /// algebra that supports a given operator declares a field with
    /// the corresponding name. `std/algebra.dag` declares direct
    /// `add`/`sub`/`mul`/`div`/`eq`/`ne`/`lt`/`le`/`gt`/`ge` fields
    /// on `OrderedRing<T>`; adding an operator to a new algebra
    /// means adding a field with the name below.
    ///
    /// Any derived-operator runtime semantics (sub = add + negate,
    /// lt = compare == Less, etc.) live in the realization layer.
    /// The field declaration here is the *signature* the compiler
    /// consumes, not the implementation.
    pub fn algebra_field_name(self) -> &'static str {
        match self {
            OperatorKind::Arithmetic(ArithmeticOp::Add) => "add",
            OperatorKind::Arithmetic(ArithmeticOp::Sub) => "sub",
            OperatorKind::Arithmetic(ArithmeticOp::Mul) => "mul",
            OperatorKind::Arithmetic(ArithmeticOp::Div) => "div",
            OperatorKind::Comparison(ComparisonOp::Eq) => "eq",
            OperatorKind::Comparison(ComparisonOp::Ne) => "ne",
            OperatorKind::Comparison(ComparisonOp::Lt) => "lt",
            OperatorKind::Comparison(ComparisonOp::Le) => "le",
            OperatorKind::Comparison(ComparisonOp::Gt) => "gt",
            OperatorKind::Comparison(ComparisonOp::Ge) => "ge",
        }
    }
}

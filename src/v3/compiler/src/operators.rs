// Operator symbol → algebra field name mapping.
//
// M1(2.6)'s §8.9 inhabitance walk resolves operator calls (`1 + 2`) by
// looking up the LHS type's inhabitance chain for a matching algebra
// field. This module owns the one remaining name-based bridge: the map
// from surface operator symbols to the `dsl/std/algebra.dag` field
// names they dispatch to.
//
// This bridge IS documented debt. It dissolves in M2+ once the surface
// grammar exposes algebra field access directly (`Int.add(a, b)`). Until
// then, it's a single localized constant — no parallel declarations,
// no bootstrap injection, no name-keyed HashMap scattered across
// infer.rs or lower.rs.
//
// The sweep in `lower::resolve_pending_identifiers` uses
// `is_operator_name` to skip these identifiers: they stay unresolved
// through lowering and get resolved at Transform-decide time via the
// inhabitance walk in `infer::resolve_operator_arrow`.

pub(crate) const OPERATOR_FIELD_MAP: &[(&str, &str)] = &[
    ("+", "add"),
    ("-", "sub"),
    ("*", "mul"),
    ("/", "div"),
    ("==", "eq"),
    ("!=", "ne"),
    ("<", "lt"),
    ("<=", "le"),
    (">", "gt"),
    (">=", "ge"),
];

pub(crate) fn operator_field_name(symbol: &str) -> Option<&'static str> {
    OPERATOR_FIELD_MAP
        .iter()
        .find(|(sym, _)| *sym == symbol)
        .map(|(_, field)| *field)
}

pub(crate) fn is_operator_name(name: &str) -> bool {
    operator_field_name(name).is_some()
}

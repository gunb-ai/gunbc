//! R2 closure-ledger ratchet for L6 `MissingEmissionPath` gaps (`docs/r2-closure-ledger.md`).
//!
//! Consumes [`crate::walker::CrossProductReport`] as tracked-gap receipts: every missing
//! cell must appear as a machine-readable key between ledger markers; ledger keys must
//! not drift from the live walker output at HEAD.

use std::collections::HashSet;

use v3_compiler::generated_full_bootstrap_dag;

use crate::check_l6_load_completeness;

const R2_CLOSURE_LEDGER: &str = include_str!("../../../../docs/r2-closure-ledger.md");

const L6_KEYS_BEGIN: &str = "<!-- L6_MISSING_EMISSION_PATH_KEYS_BEGIN -->";
const L6_KEYS_END: &str = "<!-- L6_MISSING_EMISSION_PATH_KEYS_END -->";

/// Structural debt receipt id authored next to the L6 gap key list (non-cell).
pub const L6_METHOD_TEMPLATE_PER_ROW_PROJECTION_ID: &str = "l6_method_template_per_row_projection";

fn parse_l6_missing_keys_between_markers(doc: &str) -> HashSet<String> {
    let start_idx = doc.find(L6_KEYS_BEGIN).expect(
        "`docs/r2-closure-ledger.md` must contain L6_MISSING_EMISSION_PATH key block start marker",
    );
    let rest = &doc[start_idx + L6_KEYS_BEGIN.len()..];
    let end_rel = rest.find(L6_KEYS_END).expect(
        "`docs/r2-closure-ledger.md` must contain L6_MISSING_EMISSION_PATH key block end marker",
    );
    let body = &rest[..end_rel];
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .map(String::from)
        .collect()
}

/// Ratchet: ledger keys ⟺ `CrossProductReport.missing` keys at bootstrap HEAD.
pub(crate) fn assert_closure_ledger_matches_l6_missing_emission_paths() {
    let dag = generated_full_bootstrap_dag();
    let report = check_l6_load_completeness(&dag);
    let expected: HashSet<String> = report
        .missing
        .iter()
        .map(|(cell, _)| cell.ledger_key(&dag))
        .collect();

    assert!(
        R2_CLOSURE_LEDGER.contains(L6_METHOD_TEMPLATE_PER_ROW_PROJECTION_ID),
        "closure ledger must receipt `{}` (coverage.rs per-row projection debt)",
        L6_METHOD_TEMPLATE_PER_ROW_PROJECTION_ID
    );

    let ledger_keys = parse_l6_missing_keys_between_markers(R2_CLOSURE_LEDGER);
    assert_eq!(
        ledger_keys, expected,
        "closure ledger L6 key block must exactly match walker missing cells (update \
         `docs/r2-closure-ledger.md` markers when coverage shifts)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_cross_product_missing_matches_closure_ledger_l6_keys() {
        assert_closure_ledger_matches_l6_missing_emission_paths();
    }
}

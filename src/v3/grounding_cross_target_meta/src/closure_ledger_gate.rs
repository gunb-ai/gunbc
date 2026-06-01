//! R2 closure-ledger ratchet for L6 `MissingEmissionPath` gaps.
//!
//! Consumes [`crate::walker::CrossProductReport`] as tracked-gap receipts: every missing
//! cell must appear as a machine-readable key between ledger markers; ledger keys must
//! not drift from the live walker output at HEAD.

use std::collections::HashSet;

use v3_compiler::generated_full_bootstrap_dag;

use crate::check_l6_load_completeness;

// Snapshot of the L6 key block from the retired `docs/r2-closure-ledger.md` ledger.
const R2_CLOSURE_LEDGER: &str = r#"| `l6_method_template_per_row_projection` | structural-debt |
<!-- L6_MISSING_EMISSION_PATH_KEYS_BEGIN -->
Atom_Value_Rust
Atom_Value_Python
Atom_Value_Go
Atom_Transform_Rust
Atom_Transform_Python
Atom_Transform_Go
Atom_Branch_Rust
Atom_Branch_Python
Atom_Branch_Go
Atom_Loop_Rust
Atom_Loop_Python
Atom_Loop_Go
Atom_Bind_Rust
Atom_Bind_Python
Atom_Bind_Go
Conj_Value_Rust
Conj_Value_Python
Conj_Value_Go
Conj_Transform_Rust
Conj_Transform_Python
Conj_Transform_Go
Conj_Branch_Rust
Conj_Branch_Python
Conj_Branch_Go
Conj_Loop_Rust
Conj_Loop_Python
Conj_Loop_Go
Conj_Bind_Rust
Conj_Bind_Python
Conj_Bind_Go
Disj_Value_Rust
Disj_Value_Python
Disj_Value_Go
Disj_Transform_Rust
Disj_Transform_Python
Disj_Transform_Go
Disj_Branch_Rust
Disj_Branch_Python
Disj_Branch_Go
Disj_Loop_Rust
Disj_Loop_Python
Disj_Loop_Go
Disj_Bind_Rust
Disj_Bind_Python
Disj_Bind_Go
Arrow_Value_Rust
Arrow_Value_Python
Arrow_Value_Go
Arrow_Transform_Rust
Arrow_Transform_Python
Arrow_Transform_Go
Arrow_Branch_Rust
Arrow_Branch_Python
Arrow_Branch_Go
Arrow_Loop_Rust
Arrow_Loop_Python
Arrow_Loop_Go
Arrow_Bind_Rust
Arrow_Bind_Python
Arrow_Bind_Go
Cardinality_Value_Rust
Cardinality_Value_Python
Cardinality_Value_Go
Cardinality_Branch_Rust
Cardinality_Branch_Python
Cardinality_Branch_Go
Cardinality_Loop_Rust
Cardinality_Loop_Python
Cardinality_Loop_Go
Cardinality_Bind_Rust
Cardinality_Bind_Python
Cardinality_Bind_Go
Instantiation_Value_Rust
Instantiation_Value_Python
Instantiation_Value_Go
Instantiation_Transform_Rust
Instantiation_Transform_Python
Instantiation_Transform_Go
Instantiation_Branch_Rust
Instantiation_Branch_Python
Instantiation_Branch_Go
Instantiation_Loop_Rust
Instantiation_Loop_Python
Instantiation_Loop_Go
Instantiation_Bind_Rust
Instantiation_Bind_Python
Instantiation_Bind_Go
<!-- L6_MISSING_EMISSION_PATH_KEYS_END -->
"#;

const L6_KEYS_BEGIN: &str = "<!-- L6_MISSING_EMISSION_PATH_KEYS_BEGIN -->";
const L6_KEYS_END: &str = "<!-- L6_MISSING_EMISSION_PATH_KEYS_END -->";

/// Structural debt receipt id authored next to the L6 gap key list (non-cell).
pub const L6_METHOD_TEMPLATE_PER_ROW_PROJECTION_ID: &str = "l6_method_template_per_row_projection";

fn parse_l6_missing_keys_between_markers(doc: &str) -> HashSet<String> {
    let start_idx = doc
        .find(L6_KEYS_BEGIN)
        .expect("closure ledger snapshot must contain L6 key block start marker");
    let rest = &doc[start_idx + L6_KEYS_BEGIN.len()..];
    let end_rel = rest
        .find(L6_KEYS_END)
        .expect("closure ledger snapshot must contain L6 key block end marker");
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
         embedded snapshot in closure_ledger_gate.rs when coverage shifts)"
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

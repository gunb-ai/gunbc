//! Lockstep ratchet: lane-local `EmissionDiagnostic` Rust mirrors ⊆ substrate sum (`diagnostics.dag`).
//!
//! **Authority:** substrate `type EmissionDiagnostic` in `src/v3/std/diagnostics.dag` — variants read from
//! [`v3_compiler::generated_full_bootstrap_dag`] via `TypeConnective::Disj` (same discipline as other
//! substrate sum projections in integration tests).
//!
//! **Mirrors (bridging until codegen):** each lane ships `include_str!` of its `diagnostic.rs` and extracts
//! `pub enum EmissionDiagnostic` variant names with a small line + brace-depth scanner — intentional
//! textual bridge, parallel to the closure-ledger gate’s `include_str!(../r2-closure-ledger.md)`.
//!
//! **Retirement:** this module deletes when `.dag → Rust enum codegen` consumes substrate
//! `EmissionDiagnostic` directly and lane-local mirrors are removed (mirrors’ docstrings already name this).

use std::collections::BTreeSet;

use v3_compiler::dag::{Dag, TypeConnective};
use v3_compiler::generated_full_bootstrap_dag;

/// Lane-local mirrors enumerated by relative path from `grounding_tests/src/` (`include_str!` bridging).
const MIRROR_CROSS_TARGET_META: &str =
    include_str!("../../grounding_cross_target_meta/src/diagnostic.rs");
const MIRROR_LIFETIME: &str = include_str!("../../grounding_lifetime/src/diagnostic.rs");
const MIRROR_COERCION_FOLD: &str = include_str!("../../grounding_coercion_fold/src/diagnostic.rs");

fn substrate_emission_diagnostic_variant_labels(dag: &Dag) -> BTreeSet<String> {
    let decl = dag
        .declaration_by_name("EmissionDiagnostic")
        .expect("bootstrap Dag must declare EmissionDiagnostic (diagnostics.dag)");
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|v| v.label.clone()).collect(),
        other => panic!("EmissionDiagnostic must be a Disj sum; got {other:?}"),
    }
}

fn extract_pub_enum_emission_diagnostic_body(src: &str) -> &str {
    let key = "pub enum EmissionDiagnostic";
    let idx = src
        .find(key)
        .unwrap_or_else(|| panic!("mirror source must declare `{key}`"));
    let after_key = &src[idx + key.len()..];
    let open_rel = after_key
        .find('{')
        .expect("EmissionDiagnostic enum must open with `{`");
    let open_idx = idx + key.len() + open_rel;
    extract_matching_brace_body(src, open_idx)
}

/// Returns inner slice for `{` at `open_idx`, excluding the outermost braces.
fn extract_matching_brace_body(full: &str, open_idx: usize) -> &str {
    let b = full.as_bytes();
    assert_eq!(b[open_idx], b'{', "expected `{{` at open_idx");
    let mut depth = 1usize;
    let body_start = open_idx + 1;
    let mut i = body_start;
    while i < b.len() && depth > 0 {
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            i += 2;
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() {
                    match b[i] {
                        b'\\' => i += 2,
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &full[body_start..i];
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    panic!("unbalanced braces while scanning EmissionDiagnostic enum body");
}

fn variant_labels_from_enum_body(body: &str) -> BTreeSet<String> {
    let mut depth = 0i32;
    let mut out = BTreeSet::new();
    for line in body.lines() {
        let depth_here = depth;
        if depth_here == 0 {
            if let Some(name) = variant_header_from_line(line) {
                out.insert(name);
            }
        }
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
    }
    out
}

fn variant_header_from_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("//") || line.starts_with("#[") {
        return None;
    }
    let end = line.find(|c| ['{', ','].contains(&c))?;
    let head = line[..end].trim();
    let name = head.split_whitespace().next()?;
    let mut ch = name.chars();
    let first = ch.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    Some(name.to_string())
}

fn mirror_variant_union() -> BTreeSet<String> {
    let mut u = BTreeSet::new();
    for src in [
        MIRROR_CROSS_TARGET_META,
        MIRROR_LIFETIME,
        MIRROR_COERCION_FOLD,
    ] {
        let body = extract_pub_enum_emission_diagnostic_body(src);
        u.extend(variant_labels_from_enum_body(body));
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_bootstrap_stack<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(f)
            .expect("spawn bootstrap stack test thread")
            .join()
            .expect("bootstrap stack test thread panicked")
    }

    #[test]
    fn lane_local_emission_diagnostic_mirrors_are_subset_of_substrate_sum() {
        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let substrate = substrate_emission_diagnostic_variant_labels(&dag);
            let mirrors = mirror_variant_union();

            let stray: Vec<_> = mirrors.difference(&substrate).cloned().collect();
            assert!(
                stray.is_empty(),
                "lane-local EmissionDiagnostic mirrors contain variants not in substrate sum — add them \
                 to `src/v3/std/diagnostics.dag` first, then mirror: {stray:?}\n\
                 substrate labels: {substrate:?}"
            );
        });
    }

    /// Negative control: mirror-side label absent from substrate must surface via the same
    /// `difference` path as production drift (proves the ratchet bites).
    #[test]
    fn subset_ratchet_detects_synthetic_mirror_only_variant() {
        const SYNTHETIC_MIRROR: &str = r"
// Synthesized — never ship; proves stray-label detection for lockstep reviews.
pub enum EmissionDiagnostic {
    MirrorOnlySyntheticVariantRatchetTest,
}
";

        with_bootstrap_stack(|| {
            let dag = generated_full_bootstrap_dag();
            let substrate = substrate_emission_diagnostic_variant_labels(&dag);
            let body = extract_pub_enum_emission_diagnostic_body(SYNTHETIC_MIRROR);
            let parsed = variant_labels_from_enum_body(body);
            let stray: Vec<_> = parsed.difference(&substrate).cloned().collect();

            assert!(
                stray.contains(&"MirrorOnlySyntheticVariantRatchetTest".to_string()),
                "expected synthetic mirror-only variant in stray set; got {stray:?}"
            );
            assert_eq!(
                stray.len(),
                1,
                "synthetic mirror must declare exactly one variant for this negative control"
            );
        });
    }
}

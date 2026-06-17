//! §5/§6 structural guards for body_producer infer perf (NOT wall-clock).
//! Complements discoverable `infer_perf_structural_guard_test.dag` (claim_batch CI floor).

use crate::helpers::read_v2_file;

fn live_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fn_live_body(source: &str, fn_name: &str) -> String {
    let live = live_source(source);
    let needle = format!("fn {fn_name}");
    let start = live
        .find(&needle)
        .unwrap_or_else(|| panic!("missing fn {fn_name}"));
    let after = &live[start..];
    let open = after
        .find('{')
        .unwrap_or_else(|| panic!("missing body for fn {fn_name}"));
    let mut depth = 0i32;
    for (i, ch) in after[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return after[..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("unclosed body for fn {fn_name}");
}

fn count_substring(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

#[test]
fn substitute_generics_entry_is_map_is_empty_only() {
    let src = read_v2_file("src/v1/04_infer.dag");
    let body = fn_live_body(&src, "substitute_generics");
    assert!(
        body.contains("if map_is_empty(subst)"),
        "substitute_generics entry must use O(1) map_is_empty"
    );
    assert!(
        !body.contains("map_keys(subst)"),
        "substitute_generics entry must not materialize map_keys(subst)"
    );
    assert!(
        src.contains("fn substitute_generics_apply"),
        "recursion must split into substitute_generics_apply"
    );
}

#[test]
fn substitute_generics_apply_does_not_recheck_subst_emptiness() {
    let src = read_v2_file("src/v1/04_infer.dag");
    let body = fn_live_body(&src, "substitute_generics_apply");
    assert!(
        !body.contains("map_is_empty(subst)") && !body.contains("map_keys(subst)"),
        "apply path must not re-check subst per node (subst invariant through recursion)"
    );
}

#[test]
fn target_model_vep_bundle_is_sharded_not_mega_closure() {
    let src = read_v2_file("src/v2/std/compilers/target_model.dag");
    let body = fn_live_body(&src, "decode_value_expression_projection_bundle");
    assert!(
        !body.contains("bind_outcome("),
        "VEP bundle entry must not inline bind_outcome (mega-closure regression)"
    );
    assert!(
        body.contains("decode_vep_chain_binding_ref(bundle: bundle)"),
        "VEP bundle must delegate to one-bind chain"
    );
}

#[test]
fn target_model_vep_parent_helpers_have_single_bind_outcome() {
    let src = read_v2_file("src/v2/std/compilers/target_model.dag");
    for name in [
        "decode_vep_parent_binding_ref",
        "decode_vep_parent_primitive_apply",
        "decode_vep_chain_binding_ref",
    ] {
        let body = fn_live_body(&src, name);
        assert_eq!(
            count_substring(&body, "bind_outcome("),
            1,
            "{name} must have exactly one bind_outcome (not a nested tower)"
        );
    }
}

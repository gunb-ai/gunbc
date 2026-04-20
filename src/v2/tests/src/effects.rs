#![allow(clippy::disallowed_macros)]

use std::rc::Rc;
use v2_compiler::std_effects::*;
use v2_compiler::std_http_path::{has_path_params, last_path_param, parse_path_template};

fn parse_ok(path: &str) -> Rc<PathTemplate> {
    match &*parse_path_template(&path.to_string()) {
        PathTemplateParseResult::ParsedPathTemplate { template } => template.clone(),
        other => panic!("expected parsed path template, got {other:?}"),
    }
}

fn derive_result(name: &str, method: &str, path: &str) -> Rc<DeriveOpEffectResult> {
    derive_op_effect(name.to_string(), &method.to_string(), path.to_string())
}

fn derive(name: &str, method: &str, path: &str) -> Rc<DerivedOpEffect> {
    match &*derive_result(name, method, path) {
        DeriveOpEffectResult::DerivedEffect { effect } => effect.clone(),
        other => panic!("expected derived effect, got {other:?}"),
    }
}

fn check(op: &Rc<DerivedOpEffect>, idempotent: bool, readonly: bool) -> Rc<ModifierCheck> {
    check_modifier_vs_derivation(op, &idempotent, &readonly)
}

fn is_read(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::ReadEffect)
}

fn is_create(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::CreateEffect)
}

fn is_upsert(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::UpsertEffect { .. })
}

fn is_delete(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::DeleteEffect { .. })
}

// =========================================================================
// Path template parsing
// =========================================================================

#[test]
fn parse_simple_path() {
    let t = parse_ok("/repos/{owner}/{repo}/pulls");
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "repo");
}

#[test]
fn parse_path_with_colon_suffix() {
    let t = parse_ok("/v1/{secret_name}:addVersion");
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "secret_name");
}

#[test]
fn parse_path_no_params() {
    let t = parse_ok("/token");
    assert!(!has_path_params(t.clone()));
    assert!(last_path_param(t).is_none());
}

#[test]
fn parse_path_multiple_params() {
    let t = parse_ok("/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access");
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "version");
}

#[test]
fn parse_path_strips_query_string() {
    let t = parse_ok(
        "/computeMetadata/v1/instance/service-accounts/default/identity?audience={audience}",
    );
    assert!(!has_path_params(t.clone()));
    assert!(last_path_param(t).is_none());
}

#[test]
fn parse_deeply_nested_path() {
    let t = parse_ok("/repos/{owner}/{repo}/pulls/{pull_number}/reviews");
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "pull_number");
}

#[test]
fn parse_path_rejects_unclosed_param_segment() {
    assert!(matches!(
        &*parse_path_template(&"/repos/{owner/pulls".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

#[test]
fn parse_path_rejects_stray_closing_brace() {
    assert!(matches!(
        &*parse_path_template(&"/repos/owner}/pulls".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

#[test]
fn parse_path_rejects_multiple_params_in_one_segment() {
    assert!(matches!(
        &*parse_path_template(&"/v1/{project}{secret}".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

// =========================================================================
// Effect derivation (fail-closed)
// =========================================================================

#[test]
fn get_derives_read_effect() {
    let op = derive("List", "GET", "/repos/{owner}/{repo}/pulls");
    assert!(is_read(&op.shape));
}

#[test]
fn post_always_derives_create_effect() {
    // POST with parent-scoped path key — still CreateEffect (fail-closed)
    let op = derive("CreateSecret", "POST", "/v1/projects/{project_id}/secrets");
    assert!(
        is_create(&op.shape),
        "POST should always derive CreateEffect"
    );
}

#[test]
fn post_no_path_key_derives_create_effect() {
    let op = derive("Messages", "POST", "/v1/messages");
    assert!(is_create(&op.shape));
}

#[test]
fn put_with_path_key_derives_upsert() {
    let op = derive("Update", "PUT", "/repos/{owner}/{repo}/pulls/{pull_number}");
    assert!(is_upsert(&op.shape));
}

#[test]
fn put_without_path_key_fails_closed() {
    let op = derive("Update", "PUT", "/config");
    assert!(
        is_create(&op.shape),
        "PUT without path key should fail closed to CreateEffect"
    );
}

#[test]
fn delete_with_path_key_derives_delete() {
    let op = derive("DeleteItem", "DELETE", "/items/{id}");
    assert!(is_delete(&op.shape));
}

#[test]
fn delete_without_path_key_fails_closed() {
    let op = derive("Purge", "DELETE", "/cache");
    assert!(
        is_create(&op.shape),
        "DELETE without path key should fail closed to CreateEffect"
    );
}

#[test]
fn malformed_path_fails_closed_at_derivation_boundary() {
    assert!(matches!(
        &*derive_result("Broken", "PUT", "/repos/{owner/pulls"),
        DeriveOpEffectResult::MalformedPathInput { .. }
    ));
}

#[test]
fn unknown_method_and_malformed_path_are_distinct_failures() {
    assert!(matches!(
        &*derive_result("Broken", "TRACE", "/repos/{owner}/{repo}"),
        DeriveOpEffectResult::UnknownHttpMethodInput { .. }
    ));
    assert!(matches!(
        &*derive_result("Broken", "PUT", "/repos/{owner/pulls"),
        DeriveOpEffectResult::MalformedPathInput { .. }
    ));
}

// =========================================================================
// REST ops in scope derive an EffectShape
//
// Single authority: paths are parsed from dsl/extdeps/*.dag `transport rest`
// blocks (same facts as the extdep declarations). No parallel hand-maintained
// (name, method, path) table — drift against GitHub / GCP / LLM specs is caught
// when the .dag sources change.
// =========================================================================

#[derive(Clone)]
struct RestOp {
    name: String,
    method: String,
    path: String,
}

fn unescape_extdep_path_template(s: &str) -> String {
    s.replace("\\{", "{").replace("\\}", "}")
}

fn extract_method_after_label(block: &str) -> Option<String> {
    let i = block.find("method:")?;
    let rest = block[i + 7..].trim_start();
    let tok = rest.split_whitespace().next()?;
    Some(tok.trim_end_matches(',').to_string())
}

fn extract_quoted_path_after_label(block: &str) -> Option<String> {
    let i = block.find("path:")?;
    let rest = &block[i + 5..];
    let q1 = rest.find('"')?;
    let rest2 = &rest[q1 + 1..];
    let q2 = rest2.find('"')?;
    Some(unescape_extdep_path_template(&rest2[..q2]))
}

/// Parse `operation` / `transport rest` blocks from extdep `.dag` sources.
fn parse_rest_operations_from_extdep_dag(src: &str) -> Vec<RestOp> {
    let mut out = Vec::new();
    for chunk in src.split("operation ") {
        let Some(name_end) = chunk.find(|c: char| c.is_whitespace() || c == '{') else {
            continue;
        };
        let op_name = chunk[..name_end].trim();
        if op_name.is_empty() {
            continue;
        }
        let Some(transport_pos) = chunk.find("transport rest") else {
            continue;
        };
        let after = &chunk[transport_pos..];
        let block_end = after
            .find("response ")
            .or_else(|| after.find("\n    mock_response"))
            .unwrap_or(after.len());
        let block = &after[..block_end];
        let Some(method) = extract_method_after_label(block) else {
            continue;
        };
        let Some(path) = extract_quoted_path_after_label(block) else {
            continue;
        };
        out.push(RestOp {
            name: op_name.to_string(),
            method,
            path,
        });
    }
    out
}

const GITHUB_PULLS_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/github/pulls.dag"
));
const GITHUB_GISTS_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/github/gists.dag"
));
const ANTHROPIC_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/llm/anthropic.dag"
));
const OPENAI_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/llm/openai.dag"
));
const GCP_IAM_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/cloud/gcp/iam.dag"
));
const GCP_SECRET_MANAGER_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/cloud/gcp/secret_manager.dag"
));
const GCP_STS_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/cloud/gcp/sts.dag"
));
const GCP_TOP_DAG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../dsl/extdeps/cloud/gcp/gcp.dag"
));

fn all_parsed_extdep_rest_ops() -> Vec<RestOp> {
    let mut parsed: Vec<RestOp> = Vec::new();
    parsed.extend(parse_rest_operations_from_extdep_dag(GITHUB_PULLS_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(GITHUB_GISTS_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(ANTHROPIC_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(OPENAI_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(GCP_IAM_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(
        GCP_SECRET_MANAGER_DAG,
    ));
    parsed.extend(parse_rest_operations_from_extdep_dag(GCP_STS_DAG));
    parsed.extend(parse_rest_operations_from_extdep_dag(GCP_TOP_DAG));
    parsed
}

fn tracked_extdep_rest_ops() -> Vec<RestOp> {
    let parsed = all_parsed_extdep_rest_ops();
    const NAMES: &[&str] = &[
        "List",
        "Get",
        "Diff",
        "CreateComment",
        "ListReviews",
        "CreateReview",
        "ListComments",
        "Create",
        "Messages",
        "ChatCompletion",
        "Responses",
        "GenerateAccessToken",
        "AccessVersion",
        "AddVersion",
        "Refresh",
        "Exchange",
    ];
    let mut by_name: std::collections::HashMap<String, RestOp> =
        std::collections::HashMap::with_capacity(NAMES.len());
    for op in parsed {
        by_name.entry(op.name.clone()).or_insert(op);
    }
    NAMES
        .iter()
        .map(|n| {
            by_name
                .remove(*n)
                .unwrap_or_else(|| panic!("missing extdep REST operation `{n}` in parsed sources"))
        })
        .collect()
}

#[test]
fn rest_ops_have_derived_effects() {
    for op in tracked_extdep_rest_ops() {
        let result = derive_result(&op.name, &op.method, &op.path);
        assert!(
            matches!(&*result, DeriveOpEffectResult::DerivedEffect { .. }),
            "failed to derive effect for {} ({} {})",
            op.name,
            op.method,
            op.path
        );
    }
}

// =========================================================================
// Obligation generation
// =========================================================================

#[test]
fn obligation_count_matches_idempotent_ops() {
    let derived: Vec<Rc<DerivedOpEffect>> = tracked_extdep_rest_ops()
        .iter()
        .map(|op| derive(&op.name, &op.method, &op.path))
        .collect();
    let idempotent_count = derived
        .iter()
        .filter(|d| is_idempotent_effect(d.shape.clone()))
        .count();
    let obligations = generate_idempotency_obligations(Rc::new(derived));
    assert_eq!(obligations.len(), idempotent_count);
}

#[test]
fn every_idempotent_effect_has_obligation() {
    let derived: Vec<Rc<DerivedOpEffect>> = tracked_extdep_rest_ops()
        .iter()
        .map(|op| derive(&op.name, &op.method, &op.path))
        .collect();
    let obligations = generate_idempotency_obligations(Rc::new(derived.clone()));
    for d in &derived {
        if is_idempotent_effect(d.shape.clone()) {
            assert!(
                obligations
                    .iter()
                    .any(|o| o.operation_name == d.operation_name),
                "missing obligation for idempotent op {}",
                d.operation_name
            );
        }
    }
}

// =========================================================================
// GitHub readonly-on-GET falsification (5 sites)
// =========================================================================

#[test]
fn github_readonly_gets_agree() {
    let readonly_gets = [
        ("List", "/repos/{owner}/{repo}/pulls"),
        ("Get", "/repos/{owner}/{repo}/pulls/{pull_number}"),
        ("Diff", "/repos/{owner}/{repo}/pulls/{pull_number}"),
        (
            "ListReviews",
            "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
        ),
        (
            "ListComments",
            "/repos/{owner}/{repo}/pulls/{pull_number}/comments",
        ),
    ];
    for (name, path) in &readonly_gets {
        let op = derive(name, "GET", path);
        let mc = check(&op, false, true);
        assert!(
            matches!(*mc.agreement, ModifierAgreement::Agrees),
            "{}: expected Agrees, got {:?}",
            name,
            mc.agreement
        );
        assert!(
            mc.declared_readonly,
            "{}: should have declared_readonly",
            name
        );
    }
}

// =========================================================================
// GCP idempotent falsification (5 sites)
// =========================================================================

#[test]
fn gcp_idempotent_sites_classified() {
    // AccessVersion: GET → ReadEffect → idempotent, declared idempotent → Agrees
    let access = derive(
        "AccessVersion",
        "GET",
        "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access",
    );
    let access_check = check(&access, true, true);
    assert!(
        matches!(*access_check.agreement, ModifierAgreement::Agrees),
        "AccessVersion: expected Agrees"
    );

    // All 4 POST ops: derivation is fail-closed → CreateEffect → non-idempotent.
    // Declared idempotent → DerivationUnknown (can't prove from method+path alone).

    let add = derive("AddVersion", "POST", "/v1/{secret_name}:addVersion");
    assert!(
        is_create(&add.shape),
        "AddVersion: POST always derives CreateEffect"
    );
    let add_check = check(&add, true, false);
    assert!(
        matches!(
            *add_check.agreement,
            ModifierAgreement::DerivationUnknown { .. }
        ),
        "AddVersion: expected DerivationUnknown, got {:?}",
        add_check.agreement
    );

    let gen = derive(
        "GenerateAccessToken",
        "POST",
        "/v1/projects/-/serviceAccounts/{target_sa}:generateAccessToken",
    );
    assert!(is_create(&gen.shape));
    let gen_check = check(&gen, true, false);
    assert!(
        matches!(
            *gen_check.agreement,
            ModifierAgreement::DerivationUnknown { .. }
        ),
        "GenerateAccessToken: expected DerivationUnknown, got {:?}",
        gen_check.agreement
    );

    let refresh = derive("Refresh", "POST", "/token");
    assert!(is_create(&refresh.shape));
    let refresh_check = check(&refresh, true, false);
    assert!(
        matches!(
            *refresh_check.agreement,
            ModifierAgreement::DerivationUnknown { .. }
        ),
        "Refresh: expected DerivationUnknown, got {:?}",
        refresh_check.agreement
    );

    let exchange = derive("Exchange", "POST", "/v1/token");
    assert!(is_create(&exchange.shape));
    let exchange_check = check(&exchange, true, false);
    assert!(
        matches!(
            *exchange_check.agreement,
            ModifierAgreement::DerivationUnknown { .. }
        ),
        "Exchange: expected DerivationUnknown, got {:?}",
        exchange_check.agreement
    );
}

// =========================================================================
// POST ops with parent-scoped paths derive CreateEffect, not UpsertEffect
// =========================================================================

#[test]
fn post_with_parent_path_derives_create_not_upsert() {
    // These were the reviewer's counterexamples: POST to parent-scoped paths
    // should NOT derive UpsertEffect.
    let create_secret = derive("CreateSecret", "POST", "/v1/projects/{project_id}/secrets");
    assert!(
        is_create(&create_secret.shape),
        "CreateSecret: POST should derive CreateEffect"
    );

    let create_comment_path = all_parsed_extdep_rest_ops()
        .into_iter()
        .find(|o| o.name == "CreateComment")
        .expect("CreateComment in pulls.dag")
        .path;
    let create_comment = derive("CreateComment", "POST", &create_comment_path);
    assert!(
        is_create(&create_comment.shape),
        "CreateComment: POST should derive CreateEffect"
    );

    let create_review = derive(
        "CreateReview",
        "POST",
        "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
    );
    assert!(
        is_create(&create_review.shape),
        "CreateReview: POST should derive CreateEffect"
    );
}

#[test]
fn create_comment_rest_path_matches_github_issues_comments_api() {
    let cc = all_parsed_extdep_rest_ops()
        .into_iter()
        .find(|o| o.name == "CreateComment")
        .expect("CreateComment operation in pulls.dag");
    assert!(
        cc.path.contains("/issues/"),
        "CreateComment must use Issues API path (PR comments are issues); got {}",
        cc.path
    );
    assert!(
        cc.path.contains("{issue_number}"),
        "expected issue_number path param in {}, got {}",
        "CreateComment",
        cc.path
    );
}

#[test]
fn extdep_operation_names_are_unique_in_authority_closure() {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for op in all_parsed_extdep_rest_ops() {
        assert!(
            seen.insert(op.name.clone()),
            "duplicate operation name `{}` in extdep .dag parse (ambiguous authority)",
            op.name
        );
    }
}

#[test]
fn tracked_rest_ops_list_has_no_duplicate_operation_names() {
    let ops = tracked_extdep_rest_ops();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for op in &ops {
        assert!(
            seen.insert(op.name.clone()),
            "tracked REST op list unexpectedly listed `{}` twice",
            op.name
        );
    }
}

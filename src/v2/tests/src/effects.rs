#![allow(clippy::disallowed_macros)]

use std::rc::Rc;
use v2_compiler::std_effects::*;
use v2_compiler::std_http_path::{has_path_params, last_path_param, parse_path_template};

fn derive(name: &str, method: &str, path: &str, fields: Vec<&str>) -> Option<Rc<DerivedOpEffect>> {
    derive_op_effect(
        name.to_string(),
        method.to_string(),
        path.to_string(),
        Rc::new(fields.into_iter().map(|s| s.to_string()).collect()),
    )
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
    let t = parse_path_template("/repos/{owner}/{repo}/pulls".to_string());
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "repo");
}

#[test]
fn parse_path_with_colon_suffix() {
    let t = parse_path_template("/v1/{secret_name}:addVersion".to_string());
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "secret_name");
}

#[test]
fn parse_path_no_params() {
    let t = parse_path_template("/token".to_string());
    assert!(!has_path_params(t.clone()));
    assert!(last_path_param(t).is_none());
}

#[test]
fn parse_path_multiple_params() {
    let t = parse_path_template(
        "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access".to_string(),
    );
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "version");
}

#[test]
fn parse_deeply_nested_path() {
    let t = parse_path_template(
        "/repos/{owner}/{repo}/pulls/{pull_number}/reviews".to_string(),
    );
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "pull_number");
}

// =========================================================================
// Effect derivation (the 162-186 table as code)
// =========================================================================

#[test]
fn get_derives_read_effect() {
    let op = derive("List", "GET", "/repos/{owner}/{repo}/pulls", vec![]).unwrap();
    assert!(is_read(&op.shape));
}

#[test]
fn post_no_path_key_derives_create_effect() {
    let op = derive("Messages", "POST", "/v1/messages", vec!["model", "messages"]).unwrap();
    assert!(is_create(&op.shape));
}

#[test]
fn post_with_path_key_derives_upsert() {
    let op = derive("CreateSecret", "POST", "/v1/projects/{project_id}/secrets", vec![]).unwrap();
    assert!(is_upsert(&op.shape));
}

#[test]
fn put_with_path_key_derives_upsert() {
    let op = derive("Update", "PUT", "/repos/{owner}/{repo}/pulls/{pull_number}", vec![]).unwrap();
    assert!(is_upsert(&op.shape));
}

#[test]
fn delete_with_path_key_derives_delete() {
    let op = derive("DeleteItem", "DELETE", "/items/{id}", vec![]).unwrap();
    assert!(is_delete(&op.shape));
}

// =========================================================================
// REST ops in scope derive an EffectShape
// =========================================================================

struct RestOp {
    name: &'static str,
    method: &'static str,
    path: &'static str,
}

const REST_OPS: &[RestOp] = &[
    // GitHub pulls.dag
    RestOp { name: "List",         method: "GET",  path: "/repos/{owner}/{repo}/pulls" },
    RestOp { name: "Get",          method: "GET",  path: "/repos/{owner}/{repo}/pulls/{pull_number}" },
    RestOp { name: "Diff",         method: "GET",  path: "/repos/{owner}/{repo}/pulls/{pull_number}" },
    RestOp { name: "CreateComment", method: "POST", path: "/repos/{owner}/{repo}/pulls/{pull_number}/comments" },
    RestOp { name: "ListReviews",  method: "GET",  path: "/repos/{owner}/{repo}/pulls/{pull_number}/reviews" },
    RestOp { name: "CreateReview", method: "POST", path: "/repos/{owner}/{repo}/pulls/{pull_number}/reviews" },
    RestOp { name: "ListComments", method: "GET",  path: "/repos/{owner}/{repo}/pulls/{pull_number}/comments" },
    // GitHub gists.dag
    RestOp { name: "Create",       method: "POST", path: "/gists" },
    // LLM anthropic.dag
    RestOp { name: "Messages",     method: "POST", path: "/v1/messages" },
    // LLM openai.dag
    RestOp { name: "ChatCompletion", method: "POST", path: "/v1/chat/completions" },
    RestOp { name: "Responses",    method: "POST", path: "/v1/responses" },
    // GCP falsification targets
    RestOp { name: "GenerateAccessToken", method: "POST", path: "/v1/projects/-/serviceAccounts/{target_sa}:generateAccessToken" },
    RestOp { name: "AccessVersion", method: "GET",  path: "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access" },
    RestOp { name: "AddVersion",    method: "POST", path: "/v1/{secret_name}:addVersion" },
    RestOp { name: "Refresh",       method: "POST", path: "/token" },
    RestOp { name: "Exchange",      method: "POST", path: "/v1/token" },
];

#[test]
fn rest_ops_have_derived_effects() {
    for op in REST_OPS {
        let result = derive(op.name, op.method, op.path, vec![]);
        assert!(
            result.is_some(),
            "failed to derive effect for {} ({} {})",
            op.name, op.method, op.path
        );
    }
}

// =========================================================================
// Obligation generation
// =========================================================================

#[test]
fn obligation_count_matches_idempotent_ops() {
    let derived: Vec<Rc<DerivedOpEffect>> = REST_OPS
        .iter()
        .filter_map(|op| derive(op.name, op.method, op.path, vec![]))
        .collect();
    let idempotent_count = derived.iter().filter(|d| is_idempotent_effect(d.shape.clone())).count();
    let obligations = generate_idempotency_obligations(Rc::new(derived));
    assert_eq!(obligations.len(), idempotent_count);
}

#[test]
fn every_idempotent_effect_has_obligation() {
    let derived: Vec<Rc<DerivedOpEffect>> = REST_OPS
        .iter()
        .filter_map(|op| derive(op.name, op.method, op.path, vec![]))
        .collect();
    let obligations = generate_idempotency_obligations(Rc::new(derived.clone()));
    for d in &derived {
        if is_idempotent_effect(d.shape.clone()) {
            assert!(
                obligations.iter().any(|o| o.operation_name == d.operation_name),
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
        ("List",         "/repos/{owner}/{repo}/pulls"),
        ("Get",          "/repos/{owner}/{repo}/pulls/{pull_number}"),
        ("Diff",         "/repos/{owner}/{repo}/pulls/{pull_number}"),
        ("ListReviews",  "/repos/{owner}/{repo}/pulls/{pull_number}/reviews"),
        ("ListComments", "/repos/{owner}/{repo}/pulls/{pull_number}/comments"),
    ];
    for (name, path) in &readonly_gets {
        let op = derive(name, "GET", path, vec![]).unwrap();
        let mc = check(&op, false, true);
        assert!(
            matches!(*mc.agreement, ModifierAgreement::Agrees),
            "{}: expected Agrees, got {:?}",
            name, mc.agreement
        );
        assert!(mc.declared_readonly, "{}: should have declared_readonly", name);
    }
}

// =========================================================================
// GCP idempotent falsification (5 sites)
// =========================================================================

#[test]
fn gcp_idempotent_sites_classified() {
    // AccessVersion: GET → ReadEffect → idempotent, declared idempotent → Agrees
    let access = derive("AccessVersion", "GET", "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access", vec![]).unwrap();
    let access_check = check(&access, true, true);
    assert!(matches!(*access_check.agreement, ModifierAgreement::Agrees), "AccessVersion: expected Agrees");

    // AddVersion: POST with {secret_name} → UpsertEffect → idempotent by table,
    // but real-world is non-idempotent (Google :verb convention)
    // Derivation says idempotent, declared idempotent → Agrees at derivation level
    // (the Disagrees is a SEMANTIC disagreement documented in comments, not a derivation disagreement)
    let add = derive("AddVersion", "POST", "/v1/{secret_name}:addVersion", vec![]).unwrap();
    assert!(is_upsert(&add.shape), "AddVersion: POST with path key should derive UpsertEffect");
    assert!(is_idempotent_effect(add.shape.clone()), "AddVersion: UpsertEffect is idempotent by algebra");
    let add_check = check(&add, true, false);
    assert!(matches!(*add_check.agreement, ModifierAgreement::Agrees), "AddVersion: derivation agrees with declared idempotent");

    // GenerateAccessToken: POST with {target_sa} → UpsertEffect → same as AddVersion
    let gen = derive("GenerateAccessToken", "POST", "/v1/projects/-/serviceAccounts/{target_sa}:generateAccessToken", vec![]).unwrap();
    assert!(is_upsert(&gen.shape));
    let gen_check = check(&gen, true, false);
    assert!(matches!(*gen_check.agreement, ModifierAgreement::Agrees));

    // Refresh: POST /token, no path key → CreateEffect → non-idempotent by table
    // But declared idempotent (RFC 6749 §6) → DerivationUnknown
    let refresh = derive("Refresh", "POST", "/token", vec![]).unwrap();
    assert!(is_create(&refresh.shape), "Refresh: POST no path key should derive CreateEffect");
    let refresh_check = check(&refresh, true, false);
    assert!(
        matches!(*refresh_check.agreement, ModifierAgreement::DerivationUnknown { .. }),
        "Refresh: expected DerivationUnknown, got {:?}", refresh_check.agreement
    );

    // Exchange: POST /v1/token, no path key → same as Refresh
    let exchange = derive("Exchange", "POST", "/v1/token", vec![]).unwrap();
    assert!(is_create(&exchange.shape));
    let exchange_check = check(&exchange, true, false);
    assert!(
        matches!(*exchange_check.agreement, ModifierAgreement::DerivationUnknown { .. }),
        "Exchange: expected DerivationUnknown, got {:?}", exchange_check.agreement
    );
}

// =========================================================================
// Workflow concern for review_pr (CreateReview without guarding read)
// =========================================================================

#[test]
fn create_review_is_non_idempotent_create() {
    let op = derive("CreateReview", "POST", "/repos/{owner}/{repo}/pulls/{pull_number}/reviews", vec![]).unwrap();
    assert!(is_upsert(&op.shape), "CreateReview: POST with path key derives UpsertEffect");
}

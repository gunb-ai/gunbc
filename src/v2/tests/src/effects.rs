#![allow(clippy::disallowed_macros)]

use std::rc::Rc;
use v2_compiler::std_effects::*;
use v2_compiler::std_http_path::{has_path_params, last_path_param, parse_path_template};

fn derive(name: &str, method: &str, path: &str) -> Option<Rc<DerivedOpEffect>> {
    derive_op_effect(name.to_string(), method.to_string(), path.to_string())
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
    let t = parse_path_template(&"/repos/{owner}/{repo}/pulls".to_string()).unwrap();
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "repo");
}

#[test]
fn parse_path_with_colon_suffix() {
    let t = parse_path_template(&"/v1/{secret_name}:addVersion".to_string()).unwrap();
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "secret_name");
}

#[test]
fn parse_path_no_params() {
    let t = parse_path_template(&"/token".to_string()).unwrap();
    assert!(!has_path_params(t.clone()));
    assert!(last_path_param(t).is_none());
}

#[test]
fn parse_path_multiple_params() {
    let t = parse_path_template(
        &"/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access".to_string(),
    )
    .unwrap();
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "version");
}

#[test]
fn parse_path_strips_query_string() {
    let t = parse_path_template(
        &"/computeMetadata/v1/instance/service-accounts/default/identity?audience={audience}"
            .to_string(),
    )
    .unwrap();
    assert!(!has_path_params(t.clone()));
    assert!(last_path_param(t).is_none());
}

#[test]
fn parse_deeply_nested_path() {
    let t = parse_path_template(&"/repos/{owner}/{repo}/pulls/{pull_number}/reviews".to_string())
        .unwrap();
    assert!(has_path_params(t.clone()));
    assert_eq!(last_path_param(t).unwrap(), "pull_number");
}

#[test]
fn parse_path_rejects_unclosed_param_segment() {
    assert!(parse_path_template(&"/repos/{owner/pulls".to_string()).is_none());
}

#[test]
fn parse_path_rejects_stray_closing_brace() {
    assert!(parse_path_template(&"/repos/owner}/pulls".to_string()).is_none());
}

#[test]
fn parse_path_rejects_multiple_params_in_one_segment() {
    assert!(parse_path_template(&"/v1/{project}{secret}".to_string()).is_none());
}

// =========================================================================
// Effect derivation (fail-closed)
// =========================================================================

#[test]
fn get_derives_read_effect() {
    let op = derive("List", "GET", "/repos/{owner}/{repo}/pulls").unwrap();
    assert!(is_read(&op.shape));
}

#[test]
fn post_always_derives_create_effect() {
    // POST with parent-scoped path key — still CreateEffect (fail-closed)
    let op = derive("CreateSecret", "POST", "/v1/projects/{project_id}/secrets").unwrap();
    assert!(
        is_create(&op.shape),
        "POST should always derive CreateEffect"
    );
}

#[test]
fn post_no_path_key_derives_create_effect() {
    let op = derive("Messages", "POST", "/v1/messages").unwrap();
    assert!(is_create(&op.shape));
}

#[test]
fn put_with_path_key_derives_upsert() {
    let op = derive("Update", "PUT", "/repos/{owner}/{repo}/pulls/{pull_number}").unwrap();
    assert!(is_upsert(&op.shape));
}

#[test]
fn put_without_path_key_fails_closed() {
    let op = derive("Update", "PUT", "/config").unwrap();
    assert!(
        is_create(&op.shape),
        "PUT without path key should fail closed to CreateEffect"
    );
}

#[test]
fn delete_with_path_key_derives_delete() {
    let op = derive("DeleteItem", "DELETE", "/items/{id}").unwrap();
    assert!(is_delete(&op.shape));
}

#[test]
fn delete_without_path_key_fails_closed() {
    let op = derive("Purge", "DELETE", "/cache").unwrap();
    assert!(
        is_create(&op.shape),
        "DELETE without path key should fail closed to CreateEffect"
    );
}

#[test]
fn malformed_path_fails_closed_at_derivation_boundary() {
    let op = derive("Broken", "PUT", "/repos/{owner/pulls");
    assert!(op.is_none(), "malformed path should not derive an effect");
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
    RestOp {
        name: "List",
        method: "GET",
        path: "/repos/{owner}/{repo}/pulls",
    },
    RestOp {
        name: "Get",
        method: "GET",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}",
    },
    RestOp {
        name: "Diff",
        method: "GET",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}",
    },
    RestOp {
        name: "CreateComment",
        method: "POST",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}/comments",
    },
    RestOp {
        name: "ListReviews",
        method: "GET",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
    },
    RestOp {
        name: "CreateReview",
        method: "POST",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
    },
    RestOp {
        name: "ListComments",
        method: "GET",
        path: "/repos/{owner}/{repo}/pulls/{pull_number}/comments",
    },
    // GitHub gists.dag
    RestOp {
        name: "Create",
        method: "POST",
        path: "/gists",
    },
    // LLM anthropic.dag
    RestOp {
        name: "Messages",
        method: "POST",
        path: "/v1/messages",
    },
    // LLM openai.dag
    RestOp {
        name: "ChatCompletion",
        method: "POST",
        path: "/v1/chat/completions",
    },
    RestOp {
        name: "Responses",
        method: "POST",
        path: "/v1/responses",
    },
    // GCP falsification targets
    RestOp {
        name: "GenerateAccessToken",
        method: "POST",
        path: "/v1/projects/-/serviceAccounts/{target_sa}:generateAccessToken",
    },
    RestOp {
        name: "AccessVersion",
        method: "GET",
        path: "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access",
    },
    RestOp {
        name: "AddVersion",
        method: "POST",
        path: "/v1/{secret_name}:addVersion",
    },
    RestOp {
        name: "Refresh",
        method: "POST",
        path: "/token",
    },
    RestOp {
        name: "Exchange",
        method: "POST",
        path: "/v1/token",
    },
];

#[test]
fn rest_ops_have_derived_effects() {
    for op in REST_OPS {
        let result = derive(op.name, op.method, op.path);
        assert!(
            result.is_some(),
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
    let derived: Vec<Rc<DerivedOpEffect>> = REST_OPS
        .iter()
        .filter_map(|op| derive(op.name, op.method, op.path))
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
    let derived: Vec<Rc<DerivedOpEffect>> = REST_OPS
        .iter()
        .filter_map(|op| derive(op.name, op.method, op.path))
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
        let op = derive(name, "GET", path).unwrap();
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
    )
    .unwrap();
    let access_check = check(&access, true, true);
    assert!(
        matches!(*access_check.agreement, ModifierAgreement::Agrees),
        "AccessVersion: expected Agrees"
    );

    // All 4 POST ops: derivation is fail-closed → CreateEffect → non-idempotent.
    // Declared idempotent → DerivationUnknown (can't prove from method+path alone).

    let add = derive("AddVersion", "POST", "/v1/{secret_name}:addVersion").unwrap();
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
    )
    .unwrap();
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

    let refresh = derive("Refresh", "POST", "/token").unwrap();
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

    let exchange = derive("Exchange", "POST", "/v1/token").unwrap();
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
    let create_secret =
        derive("CreateSecret", "POST", "/v1/projects/{project_id}/secrets").unwrap();
    assert!(
        is_create(&create_secret.shape),
        "CreateSecret: POST should derive CreateEffect"
    );

    let create_comment = derive(
        "CreateComment",
        "POST",
        "/repos/{owner}/{repo}/pulls/{pull_number}/comments",
    )
    .unwrap();
    assert!(
        is_create(&create_comment.shape),
        "CreateComment: POST should derive CreateEffect"
    );

    let create_review = derive(
        "CreateReview",
        "POST",
        "/repos/{owner}/{repo}/pulls/{pull_number}/reviews",
    )
    .unwrap();
    assert!(
        is_create(&create_review.shape),
        "CreateReview: POST should derive CreateEffect"
    );
}

#![allow(clippy::disallowed_macros)]

use std::collections::HashMap;
use std::rc::Rc;
use v2_compiler::std_effects::*;
use v2_compiler::std_http_path::{has_path_params, last_path_param, parse_path_template};
use v2_compiler::v2_compiler_parse::parse;
use v2_compiler::v2_compiler_tokenize::tokenize;
use v2_compiler::v2_std_core::{
    authored_name_at, build_newline_index, find_property, find_property_string, is_rest_transport,
    transport_method_key, transport_path_template_key, ExprData, NewlineIndex, Node,
};

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
// Single authority: tokenize + parse extdep `.dag` files, then read `transport
// rest` facts via `v2_std_core` (same transport property accessors as the
// compiler). No second text parser over raw source.
// =========================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
struct RestOp {
    service: String,
    name: String,
    method: String,
    path: String,
}

fn parse_extdep_module(relative_path: &str) -> (Rc<Node>, Rc<HashMap<String, Rc<NewlineIndex>>>) {
    let source = crate::helpers::read_v2_file(relative_path);
    let filename = std::path::Path::new(relative_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.dag");
    let tokens = tokenize(&source.to_string(), filename.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(
        filename.to_string(),
        build_newline_index(filename.to_string(), &source.to_string()),
    );
    let source_indices = Rc::new(source_indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "failed to parse {}: {}",
            relative_path,
            v2_compiler::v2_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result.module.clone().unwrap_or_else(|| {
        panic!("{} produced no module", relative_path);
    });
    (module, source_indices)
}

/// `method: GET` and similar use keyword / ident values; `path` is a string literal.
fn rest_method_or_path_string(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    find_property_string(props.clone(), prop_name.clone(), source_indices.clone()).or_else(|| {
        let n = find_property(props, prop_name, source_indices.clone())?;
        match (*n.expr_data).clone() {
            ExprData::ExprVar { .. } => {
                let s = authored_name_at(source_indices, &n);
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    })
}

fn collect_rest_ops_from_parsed_module(
    module: &Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<RestOp> {
    let mut out = Vec::new();
    fn walk(
        n: &Rc<Node>,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        service_ctx: Option<String>,
        out: &mut Vec<RestOp>,
    ) {
        let ctx_for_children = match &n.transport {
            Some(t)
                if !is_rest_transport(t.clone(), source_indices.clone()) && !n.name.is_empty() =>
            {
                Some(n.name.clone())
            }
            _ => service_ctx.clone(),
        };

        if let Some(t) = &n.transport {
            if is_rest_transport(t.clone(), source_indices.clone()) {
                let svc = service_ctx
                    .clone()
                    .expect("REST operation without enclosing service scope");
                let method = rest_method_or_path_string(
                    t.properties.clone(),
                    transport_method_key(),
                    source_indices.clone(),
                )
                .unwrap_or_else(|| {
                    panic!(
                        "missing method value on rest transport for {}::{}",
                        svc, n.name
                    )
                });
                let path = rest_method_or_path_string(
                    t.properties.clone(),
                    transport_path_template_key(),
                    source_indices.clone(),
                )
                .unwrap_or_else(|| {
                    panic!(
                        "missing path value on rest transport for {}::{}",
                        svc, n.name
                    )
                });
                out.push(RestOp {
                    service: svc,
                    name: n.name.clone(),
                    method,
                    path,
                });
            }
        }

        for c in n.children.iter() {
            walk(c, source_indices.clone(), ctx_for_children.clone(), out);
        }
    }
    walk(module, source_indices, None, &mut out);
    out
}

fn all_parsed_extdep_rest_ops() -> Vec<RestOp> {
    const FILES: &[&str] = &[
        "dsl/extdeps/github/pulls.dag",
        "dsl/extdeps/github/gists.dag",
        "dsl/extdeps/llm/anthropic.dag",
        "dsl/extdeps/llm/openai.dag",
        "dsl/extdeps/cloud/gcp/iam.dag",
        "dsl/extdeps/cloud/gcp/secret_manager.dag",
        "dsl/extdeps/cloud/gcp/sts.dag",
        "dsl/extdeps/cloud/gcp/gcp.dag",
    ];
    let mut parsed: Vec<RestOp> = Vec::new();
    for path in FILES {
        let (module, indices) = parse_extdep_module(path);
        parsed.extend(collect_rest_ops_from_parsed_module(&module, indices));
    }
    parsed
}

fn tracked_extdep_rest_ops() -> Vec<RestOp> {
    let parsed = all_parsed_extdep_rest_ops();
    const TRACKED: &[(&str, &str)] = &[
        ("github.Pulls", "List"),
        ("github.Pulls", "Get"),
        ("github.Pulls", "Diff"),
        ("github.Pulls", "CreateComment"),
        ("github.Pulls", "ListReviews"),
        ("github.Pulls", "CreateReview"),
        ("github.Pulls", "ListComments"),
        ("github.Gist", "Create"),
        ("llm.Anthropic", "Messages"),
        ("llm.OpenAI", "ChatCompletion"),
        ("llm.OpenAI", "Responses"),
        ("gcp.IAM", "GenerateAccessToken"),
        ("gcp.SecretManager", "AccessVersion"),
        ("gcp.SecretManager", "AddVersion"),
        ("oauth2.Google", "Refresh"),
        ("gcp.STS", "Exchange"),
    ];
    let mut by_key: std::collections::HashMap<(String, String), RestOp> =
        std::collections::HashMap::with_capacity(TRACKED.len());
    for op in parsed {
        by_key
            .entry((op.service.clone(), op.name.clone()))
            .or_insert(op);
    }
    TRACKED
        .iter()
        .map(|(svc, name)| {
            by_key
                .remove(&((*svc).to_string(), (*name).to_string()))
                .unwrap_or_else(|| {
                    panic!("missing extdep REST operation `{svc}::{name}` in parsed sources")
                })
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
fn extdep_service_operation_pairs_are_unique_in_authority_closure() {
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for op in all_parsed_extdep_rest_ops() {
        let key = (op.service.clone(), op.name.clone());
        assert!(
            seen.insert(key.clone()),
            "duplicate (service, operation) `{:?}` in extdep parse (ambiguous authority)",
            key
        );
    }
}

#[test]
fn tracked_rest_ops_list_has_no_duplicate_service_operation_keys() {
    let ops = tracked_extdep_rest_ops();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for op in &ops {
        let key = (op.service.clone(), op.name.clone());
        assert!(
            seen.insert(key.clone()),
            "tracked REST op list unexpectedly listed `{:?}` twice",
            key
        );
    }
}

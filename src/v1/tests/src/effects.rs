#![allow(clippy::disallowed_macros)]

use std::collections::HashMap;
use std::rc::Rc;
use v1_compiler::extdeps_uri_path::{parse_path_template, PathTemplateParseResult};
use v1_compiler::rest_transport_facts::{
    collect_rest_transport_operations, DeclaredRestTransportOp,
};
use v1_compiler::std_effects::*;
use v1_compiler::std_http_path::{has_path_params, last_path_param};
use v1_compiler::std_types::HttpMethod;
use v1_compiler::v1_compiler_parse::parse;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, NewlineIndex, Node};

fn parse_ok(path: &str) -> Rc<PathTemplate> {
    match &*parse_path_template(path.to_string()) {
        PathTemplateParseResult::ParsedPathTemplate { template } => template.clone(),
        other => panic!("expected parsed path template, got {other:?}"),
    }
}

fn ingest_rest_transport_method(method: &str) -> Option<HttpMethod> {
    match method {
        "GET" => Some(HttpMethod::GET),
        "POST" => Some(HttpMethod::POST),
        "PUT" => Some(HttpMethod::PUT),
        "PATCH" => Some(HttpMethod::PATCH),
        "DELETE" => Some(HttpMethod::DELETE),
        "HEAD" => Some(HttpMethod::HEAD),
        "OPTIONS" => Some(HttpMethod::OPTIONS),
        _ => None,
    }
}

fn method_ok(method: &str) -> HttpMethod {
    ingest_rest_transport_method(method)
        .unwrap_or_else(|| panic!("unknown REST transport HTTP method `{method}`"))
}

fn derive_result(name: &str, method: HttpMethod, path: &str) -> Rc<DeriveOpEffectResult> {
    derive_op_effect(name.to_string(), method, parse_ok(path))
}

fn derive(name: &str, method: HttpMethod, path: &str) -> Rc<DerivedOpEffect> {
    match &*derive_result(name, method, path) {
        DeriveOpEffectResult::DerivedEffect { effect } => effect.clone(),
        other => panic!("expected derived effect, got {other:?}"),
    }
}

fn check(op: &Rc<DerivedOpEffect>, idempotent: bool, readonly: bool) -> Rc<ModifierCheck> {
    check_modifier_vs_derivation(op.clone(), idempotent, readonly)
}

fn is_read(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::ReadEffect)
}

fn is_create(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::CreateEffect { .. })
}

fn is_create_always(shape: &EffectShape) -> bool {
    matches!(
        shape,
        EffectShape::CreateEffect {
            cause,
            ..
        } if matches!(cause.as_ref(), CreateCause::PostAlways)
    )
}

fn is_upsert(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::UpsertEffect { .. })
}

fn is_delete(shape: &EffectShape) -> bool {
    matches!(shape, EffectShape::DeleteEffect { .. })
}

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
        &*parse_path_template("/repos/{owner/pulls".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

#[test]
fn parse_path_rejects_stray_closing_brace() {
    assert!(matches!(
        &*parse_path_template("/repos/owner}/pulls".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

#[test]
fn parse_path_rejects_multiple_params_in_one_segment() {
    assert!(matches!(
        &*parse_path_template("/v1/{project}{secret}".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

#[test]
fn get_derives_read_effect() {
    let op = derive("List", method_ok("GET"), "/repos/{owner}/{repo}/pulls");
    assert!(is_read(&op.shape));
}

#[test]
fn post_always_derives_create_effect() {
    let op = derive(
        "CreateSecret",
        method_ok("POST"),
        "/v1/projects/{project_id}/secrets",
    );
    assert!(
        is_create(&op.shape),
        "POST should always derive CreateEffect"
    );
}

#[test]
fn post_no_path_key_derives_create_effect() {
    let op = derive("Messages", method_ok("POST"), "/v1/messages");
    assert!(is_create(&op.shape));
}

#[test]
fn put_with_path_key_derives_upsert() {
    let op = derive(
        "Update",
        method_ok("PUT"),
        "/repos/{owner}/{repo}/pulls/{pull_number}",
    );
    assert!(is_upsert(&op.shape));
}

#[test]
fn put_without_path_key_fails_closed() {
    let op = derive("Update", method_ok("PUT"), "/config");
    assert!(
        is_create(&op.shape),
        "PUT without path key should fail closed to CreateEffect"
    );
}

#[test]
fn delete_with_path_key_derives_delete() {
    let op = derive("DeleteItem", method_ok("DELETE"), "/items/{id}");
    assert!(is_delete(&op.shape));
}

#[test]
fn delete_without_path_key_fails_closed() {
    let op = derive("Purge", method_ok("DELETE"), "/cache");
    assert!(
        is_create(&op.shape),
        "DELETE without path key should fail closed to CreateEffect"
    );
}

#[test]
fn derivation_consumes_typed_method_and_path_template_without_parsing_strings() {
    let path = parse_ok("/repos/{owner}/{repo}");
    let result = derive_op_effect("TypedBoundary".to_string(), HttpMethod::PUT, path.clone());
    assert!(matches!(
        &*result,
        DeriveOpEffectResult::DerivedEffect { .. }
    ));
}

#[test]
fn method_and_path_string_failures_remain_at_surface_parsers() {
    assert!(ingest_rest_transport_method("TRACE").is_none());
    assert!(matches!(
        &*parse_path_template("/repos/{owner/pulls".to_string()),
        PathTemplateParseResult::MalformedPathTemplate { .. }
    ));
}

const GITHUB_PULLS: &str = "github.Pulls";

type RestOp = DeclaredRestTransportOp;

fn rest_transport_fingerprint(op: &RestOp) -> (String, String, String, String) {
    (
        op.service.clone(),
        op.name.clone(),
        op.method.clone(),
        op.path.clone(),
    )
}

fn parse_extdep_module(relative_path: &str) -> (Rc<Node>, Rc<HashMap<String, Rc<NewlineIndex>>>) {
    let source = crate::helpers::read_v2_file(relative_path);
    let filename = std::path::Path::new(relative_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.dag");
    let tokens = tokenize(source.to_string(), filename.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(
        filename.to_string(),
        build_newline_index(filename.to_string(), source.to_string()),
    );
    let source_indices = Rc::new(source_indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "failed to parse {}: {}",
            relative_path,
            v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result.module.clone().unwrap_or_else(|| {
        panic!("{} produced no module", relative_path);
    });
    (module, source_indices)
}

fn all_parsed_extdep_rest_ops() -> Vec<RestOp> {
    const FILES: &[&str] = &[
        "dag/extdeps/github/pulls.dag",
        "dag/extdeps/github/gists.dag",
        "dag/extdeps/llm/anthropic_rest.dag",
        "dag/extdeps/llm/openai_rest.dag",
        "dag/extdeps/cloud/gcp/iam.dag",
        "dag/extdeps/cloud/gcp/secret_manager.dag",
        "dag/extdeps/cloud/gcp/sts.dag",
        "dag/extdeps/cloud/gcp/gcp.dag",
    ];
    let mut parsed: Vec<RestOp> = Vec::new();
    for path in FILES {
        let (module, indices) = parse_extdep_module(path);
        let collected = collect_rest_transport_operations(&module, indices);
        assert!(
            collected.errors.is_empty(),
            "unexpected REST transport fact errors for {}: {:?}",
            path,
            collected.errors
        );
        parsed.extend(collected.ops);
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
    TRACKED
        .iter()
        .map(|(svc, name)| {
            let matches: Vec<&RestOp> = parsed
                .iter()
                .filter(|p| p.service == *svc && p.name == *name)
                .collect();
            match matches.as_slice() {
                [op] => (*op).clone(),
                [] => panic!("missing extdep REST operation `{svc}::{name}` in parsed sources"),
                _ => panic!(
                    "ambiguous extdep REST operation `{svc}::{name}`: {} matches (use full fingerprint)",
                    matches.len()
                ),
            }
        })
        .collect()
}

#[test]
fn rest_ops_have_derived_effects() {
    for op in tracked_extdep_rest_ops() {
        let result = derive_result(&op.name, method_ok(&op.method), &op.path);
        assert!(
            matches!(&*result, DeriveOpEffectResult::DerivedEffect { .. }),
            "failed to derive effect for {} ({} {})",
            op.name,
            op.method,
            op.path
        );
    }
}

fn op_lattice_read(name: &str) -> Rc<OperationEffect> {
    Rc::new(OperationEffect {
        operation_name: name.to_string(),
        shape: Rc::new(EffectShape::ReadEffect),
        evidence: Rc::new(IdempotencyEvidence::LatticeEffect {
            shape: Rc::new(EffectShape::ReadEffect),
        }),
    })
}

fn op_non_idempotent(name: &str) -> Rc<OperationEffect> {
    let shape = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::PostAlways),
    });
    Rc::new(OperationEffect {
        operation_name: name.to_string(),
        shape: shape.clone(),
        evidence: Rc::new(IdempotencyEvidence::NonIdempotent {
            shape,
            reason: "test".to_string(),
        }),
    })
}

#[test]
fn r3_f5_compose_effects_empty_yields_idempotent_composition() {
    let ops = Rc::new(vec![]);
    let v = compose_effects(ops);
    assert!(matches!(&*v, CompositionVerdict::IdempotentComposition));
}

#[test]
fn r3_f5_compose_effects_all_lattice_yields_idempotent_composition() {
    let ops = Rc::new(vec![op_lattice_read("a"), op_lattice_read("b")]);
    let v = compose_effects(ops);
    assert!(matches!(&*v, CompositionVerdict::IdempotentComposition));
}

#[test]
fn r3_f5_compose_effects_broken_by_first_non_idempotent_evidence() {
    let ops = Rc::new(vec![
        op_lattice_read("first"),
        op_non_idempotent("breaker"),
        op_non_idempotent("later"),
    ]);
    let v = compose_effects(ops);
    match &*v {
        CompositionVerdict::BrokenBy { first_breaker } => {
            assert_eq!(first_breaker.operation_name, "breaker");
        }
        CompositionVerdict::IdempotentComposition => {
            panic!("expected BrokenBy when NonIdempotent evidence is present");
        }
    }
}

#[test]
fn obligation_count_matches_idempotent_ops() {
    let derived: Vec<Rc<DerivedOpEffect>> = tracked_extdep_rest_ops()
        .iter()
        .map(|op| derive(&op.name, method_ok(&op.method), &op.path))
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
        .map(|op| derive(&op.name, method_ok(&op.method), &op.path))
        .collect();
    let obligations = generate_idempotency_obligations(Rc::new(derived.clone()));
    for d in &derived {
        if is_idempotent_effect(d.shape.clone()) {
            assert!(
                obligations.iter().any(|o| {
                    o.operation_name == d.operation_name && o.effect_shape == d.shape
                }),
                "missing obligation for idempotent op {} (match name + effect shape, not name alone)",
                d.operation_name
            );
        }
    }
}

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
        let op = derive(name, method_ok("GET"), path);
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

#[test]
fn gcp_idempotent_sites_classified() {
    let access = derive(
        "AccessVersion",
        method_ok("GET"),
        "/v1/projects/{project_id}/secrets/{secret}/versions/{version}:access",
    );
    let access_check = check(&access, true, true);
    assert!(
        matches!(*access_check.agreement, ModifierAgreement::Agrees),
        "AccessVersion: expected Agrees"
    );

    let add = derive(
        "AddVersion",
        method_ok("POST"),
        "/v1/{secret_name}:addVersion",
    );
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
        method_ok("POST"),
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

    let refresh = derive("Refresh", method_ok("POST"), "/token");
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

    let exchange = derive("Exchange", method_ok("POST"), "/v1/token");
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

#[test]
fn post_with_parent_path_derives_create_not_upsert() {
    let create_secret = derive(
        "CreateSecret",
        method_ok("POST"),
        "/v1/projects/{project_id}/secrets",
    );
    assert!(
        is_create(&create_secret.shape),
        "CreateSecret: POST should derive CreateEffect"
    );

    let create_comment_path = all_parsed_extdep_rest_ops()
        .into_iter()
        .find(|o| o.service == GITHUB_PULLS && o.name == "CreateComment")
        .expect("CreateComment in github.Pulls (pulls.dag)")
        .path;
    let create_comment = derive("CreateComment", method_ok("POST"), &create_comment_path);
    assert!(
        is_create(&create_comment.shape),
        "CreateComment: POST should derive CreateEffect"
    );

    let create_review = derive(
        "CreateReview",
        method_ok("POST"),
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
        .find(|o| o.service == GITHUB_PULLS && o.name == "CreateComment")
        .expect("CreateComment operation under github.Pulls (pulls.dag)");
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
fn create_if_absent_with_same_key_is_dedupable() {
    let key = Rc::new(KeySource::InputField {
        field: "id".to_string(),
    });
    let shape = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::CreateIfAbsent { key_source: key }),
    });
    assert!(is_idempotent_effect(shape.clone()));
    assert!(create_effect_is_dedupable(shape.clone()));
    assert!(create_double_init_collapsible(shape.clone(), shape));
}

#[test]
fn create_if_absent_with_different_keys_not_collapsible() {
    let a = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::CreateIfAbsent {
            key_source: Rc::new(KeySource::InputField {
                field: "a".to_string(),
            }),
        }),
    });
    let b = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::CreateIfAbsent {
            key_source: Rc::new(KeySource::InputField {
                field: "b".to_string(),
            }),
        }),
    });
    assert!(create_effect_is_dedupable(a.clone()));
    assert!(create_effect_is_dedupable(b.clone()));
    assert!(!create_double_init_collapsible(a, b));
}

#[test]
fn create_always_is_not_dedupable() {
    let shape = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::PostAlways),
    });
    assert!(!is_idempotent_effect(shape.clone()));
    assert!(!create_effect_is_dedupable(shape.clone()));
    assert!(!create_double_init_collapsible(shape.clone(), shape));
}

#[test]
fn create_if_absent_declared_idempotent_agrees_with_derivation() {
    let shape = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::CreateIfAbsent {
            key_source: Rc::new(KeySource::InputField {
                field: "id".to_string(),
            }),
        }),
    });
    let op = Rc::new(DerivedOpEffect {
        operation_name: "create_secret".to_string(),
        method: HttpMethod::POST,
        path_template: parse_ok("/secrets"),
        shape: shape.clone(),
    });
    let result = check_modifier_vs_derivation(op, true, false);
    assert!(
        matches!(*result.agreement, ModifierAgreement::Agrees),
        "create-if-absent with declared idempotent should agree with derivation, got {:?}",
        result.agreement
    );
}

#[test]
fn create_if_absent_declared_non_idempotent_disagrees_with_derivation() {
    let shape = Rc::new(EffectShape::CreateEffect {
        cause: Rc::new(CreateCause::CreateIfAbsent {
            key_source: Rc::new(KeySource::InputField {
                field: "id".to_string(),
            }),
        }),
    });
    let op = Rc::new(DerivedOpEffect {
        operation_name: "create_secret".to_string(),
        method: HttpMethod::POST,
        path_template: parse_ok("/secrets"),
        shape: shape.clone(),
    });
    let result = check_modifier_vs_derivation(op, false, false);
    assert!(
        matches!(*result.agreement, ModifierAgreement::Disagrees { .. }),
        "create-if-absent with declared non-idempotent should disagree with derivation, got {:?}",
        result.agreement
    );
}

#[test]
fn create_always_declared_idempotent_is_derivation_unknown() {
    let op = Rc::new(DerivedOpEffect {
        operation_name: "add_version".to_string(),
        method: HttpMethod::POST,
        path_template: parse_ok("/versions"),
        shape: Rc::new(EffectShape::CreateEffect {
            cause: Rc::new(CreateCause::PostAlways),
        }),
    });
    let result = check_modifier_vs_derivation(op, true, false);
    assert!(
        matches!(
            *result.agreement,
            ModifierAgreement::DerivationUnknown { .. }
        ),
        "PostAlways create with declared idempotent should be DerivationUnknown, got {:?}",
        result.agreement
    );
}

#[test]
fn extdep_rest_fingerprints_are_unique_in_authority_closure() {
    let mut seen: std::collections::HashSet<(String, String, String, String)> =
        std::collections::HashSet::new();
    for op in all_parsed_extdep_rest_ops() {
        let key = rest_transport_fingerprint(&op);
        assert!(
            seen.insert(key.clone()),
            "duplicate REST fingerprint `{:?}` in extdep parse (ambiguous authority)",
            key
        );
    }
}

#[test]
fn tracked_rest_ops_list_has_no_duplicate_fingerprints() {
    let ops = tracked_extdep_rest_ops();
    let mut seen: std::collections::HashSet<(String, String, String, String)> =
        std::collections::HashSet::new();
    for op in &ops {
        let key = rest_transport_fingerprint(op);
        assert!(
            seen.insert(key.clone()),
            "tracked REST op list unexpectedly listed `{:?}` twice",
            key
        );
    }
}

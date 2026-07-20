#![allow(clippy::disallowed_macros)]

use im::HashMap;
use std::collections::HashSet;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::cli_run::{collect_rest_transport_operations, DeclaredRestTransportOp};
use v1_compiler::extdeps_uri_path::{parse_path_template, PathTemplateParseResult};
use v1_compiler::std_effects::{
    derive_op_effect, generate_idempotency_obligations, is_idempotent_effect, DeriveOpEffectResult,
    EffectShape, HttpMethod,
};
use v1_compiler::std_http_path::PathTemplate;
use v1_compiler::v1_compiler_parse::parse;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, NewlineIndex, Node};

const AUTHORITY_FILES: &[&str] = &[
    "dag/extdeps/github/pulls.dag",
    "dag/extdeps/github/gists.dag",
    "dag/extdeps/llm/anthropic_rest.dag",
    "dag/extdeps/llm/openai_rest.dag",
    "dag/extdeps/cloud/gcp/iam.dag",
    "dag/extdeps/cloud/gcp/secret_manager.dag",
    "dag/extdeps/cloud/gcp/sts.dag",
    "dag/extdeps/cloud/gcp/gcp.dag",
];

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

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("effects_rest_transport_witness: {msg}");
    ExitCode::from(1)
}

fn parse_path(path: &str) -> Rc<PathTemplate> {
    match &*parse_path_template(path.to_string()) {
        PathTemplateParseResult::ParsedPathTemplate { template } => template.clone(),
        PathTemplateParseResult::MalformedPathTemplate { .. } => {
            panic!("expected parsed path template for `{path}`")
        }
    }
}

fn method_ok(method: &str) -> HttpMethod {
    match method {
        "GET" => HttpMethod::GET,
        "POST" => HttpMethod::POST,
        "PUT" => HttpMethod::PUT,
        "PATCH" => HttpMethod::PATCH,
        "DELETE" => HttpMethod::DELETE,
        "HEAD" => HttpMethod::HEAD,
        "OPTIONS" => HttpMethod::OPTIONS,
        other => panic!("unknown REST transport HTTP method `{other}`"),
    }
}

fn fingerprint(op: &DeclaredRestTransportOp) -> (String, String, String, String) {
    (
        op.service.clone(),
        op.name.clone(),
        op.method.clone(),
        op.path.clone(),
    )
}

fn parse_extdep_module(relative_path: &str) -> (Rc<Node>, Rc<HashMap<String, Rc<NewlineIndex>>>) {
    let path = workspace_root().join(relative_path);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file.dag")
        .to_string();
    let tokens = tokenize(source.clone(), filename.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), build_newline_index(filename, source));
    let source_indices = Rc::new(source_indices);
    let result = parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "failed to parse {}: {}",
            relative_path,
            v1_compiler::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .clone()
        .unwrap_or_else(|| panic!("{relative_path} produced no module"));
    (module, source_indices)
}

fn all_parsed_extdep_rest_ops() -> Vec<DeclaredRestTransportOp> {
    let mut parsed = Vec::new();
    for path in AUTHORITY_FILES {
        let (module, indices) = parse_extdep_module(path);
        let collected = collect_rest_transport_operations(&module, indices);
        assert!(
            collected.errors.is_empty(),
            "unexpected REST transport fact errors for {path}: {:?}",
            collected.errors
        );
        parsed.extend(collected.ops);
    }
    parsed
}

fn tracked_extdep_rest_ops(parsed: &[DeclaredRestTransportOp]) -> Vec<DeclaredRestTransportOp> {
    TRACKED
        .iter()
        .map(|(svc, name)| {
            let matches: Vec<&DeclaredRestTransportOp> = parsed
                .iter()
                .filter(|p| p.service == *svc && p.name == *name)
                .collect();
            match matches.as_slice() {
                [op] => (*op).clone(),
                [] => panic!("missing extdep REST operation `{svc}::{name}` in parsed sources"),
                _ => panic!(
                    "ambiguous extdep REST operation `{svc}::{name}`: {} matches",
                    matches.len()
                ),
            }
        })
        .collect()
}

fn main() -> ExitCode {
    let parsed = all_parsed_extdep_rest_ops();

    let mut seen = HashSet::new();
    for op in &parsed {
        let key = fingerprint(op);
        if !seen.insert(key.clone()) {
            return fail(format!(
                "duplicate REST fingerprint {:?} in extdep authority closure",
                key
            ));
        }
    }

    let tracked = tracked_extdep_rest_ops(&parsed);

    let mut tracked_seen = HashSet::new();
    for op in &tracked {
        let key = fingerprint(op);
        if !tracked_seen.insert(key.clone()) {
            return fail(format!(
                "tracked REST op list unexpectedly listed {:?} twice",
                key
            ));
        }
    }

    for op in &tracked {
        let result = derive_op_effect(op.name.clone(), method_ok(&op.method), parse_path(&op.path));
        if !matches!(&*result, DeriveOpEffectResult::DerivedEffect { .. }) {
            return fail(format!(
                "failed to derive effect for {} ({} {})",
                op.name, op.method, op.path
            ));
        }
    }

    let derived: Vec<_> = tracked
        .iter()
        .map(|op| {
            let result =
                derive_op_effect(op.name.clone(), method_ok(&op.method), parse_path(&op.path));
            match &*result {
                DeriveOpEffectResult::DerivedEffect { effect } => effect.clone(),
                _ => unreachable!(),
            }
        })
        .collect();

    let idempotent_count = derived
        .iter()
        .filter(|d| is_idempotent_effect(d.shape.clone()))
        .count();
    let obligations = generate_idempotency_obligations(Rc::new(derived.clone().into()));
    if obligations.len() != idempotent_count {
        return fail(format!(
            "obligation count {} != idempotent op count {idempotent_count}",
            obligations.len()
        ));
    }

    for d in &derived {
        if is_idempotent_effect(d.shape.clone())
            && !obligations
                .iter()
                .any(|o| o.operation_name == d.operation_name && o.effect_shape == d.shape)
        {
            return fail(format!(
                "missing obligation for idempotent op {}",
                d.operation_name
            ));
        }
    }

    let create_comment = tracked
        .iter()
        .find(|o| o.service == "github.Pulls" && o.name == "CreateComment")
        .expect("CreateComment in tracked ops");
    if !create_comment.path.contains("/issues/") {
        return fail(format!(
            "CreateComment must use Issues API path; got {}",
            create_comment.path
        ));
    }
    if !create_comment.path.contains("{issue_number}") {
        return fail(format!(
            "CreateComment expected issue_number path param; got {}",
            create_comment.path
        ));
    }
    match &*derive_op_effect(
        create_comment.name.clone(),
        method_ok(&create_comment.method),
        parse_path(&create_comment.path),
    ) {
        DeriveOpEffectResult::DerivedEffect { effect } => {
            if !matches!(effect.shape.as_ref(), EffectShape::CreateEffect { .. }) {
                return fail(format!(
                    "CreateComment POST on {} should derive CreateEffect, got {:?}",
                    create_comment.path, effect.shape
                ));
            }
        }
        _ => {
            return fail(format!(
                "failed to derive effect for CreateComment ({} {})",
                create_comment.method, create_comment.path
            ));
        }
    }

    ExitCode::SUCCESS
}

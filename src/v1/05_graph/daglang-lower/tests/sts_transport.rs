use daglang_lower::{lower_typed_project, ServiceOperationSpec};
use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::parser;
use daglang_typecheck::{typecheck_owned_module_graph, TypedProject};
use gunbc_exec::Executable;
use gunbc_ir::transport::{HttpMethod, TransportRequest};
use gunbc_ir::{SecretString, Value};
use gunbc_resolve::service_ops::GenericPrepareOp;
use std::collections::HashMap;
use std::path::PathBuf;

fn module_graph_from_sources(sources: &[(&str, &str)]) -> ModuleGraph {
    let modules = sources
        .iter()
        .map(|(path, source)| {
            let ast = parser::parse(source).expect("source should parse");
            let module_path = ast
                .module_path
                .as_ref()
                .map(|module| module.node.clone())
                .expect("module declaration is required");
            ResolvedModule {
                path: PathBuf::from(path),
                ast,
                module_path,
                dependencies: Vec::new(),
                source: source.to_string(),
            }
        })
        .collect::<Vec<_>>();
    let module_lookup = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect::<HashMap<_, _>>();
    let mut modules = modules;
    for module in &mut modules {
        module.dependencies = module
            .ast
            .imports
            .iter()
            .filter_map(|import| module_lookup.get(&import.node.path.as_dotted()).copied())
            .collect::<Vec<_>>();
    }
    ModuleGraph { modules }
}

fn typed_project_from_sources(sources: &[(&str, &str)]) -> TypedProject<'static> {
    typecheck_owned_module_graph(module_graph_from_sources(sources))
        .expect("typecheck should succeed")
}

fn sts_transport_module_source(body_fields: &str) -> String {
    [
        "module extdeps.cloud.gcp.sts\n",
        "type SubjectTokenType = Jwt | StsAccessToken | IdToken | Saml2\n",
        "type RequestedTokenType = RequestAccessToken | RequestIdToken\n",
        "type StsGrantType = TokenExchange {}\n",
        "type StsTokenExchange {\n",
        "  grant_type: StsGrantType\n",
        "  subject_token: Secret\n",
        "  subject_token_type: SubjectTokenType\n",
        "  audience: String\n",
        "  requested_token_type: RequestedTokenType?\n",
        "}\n",
        "type StsTokenResponse {\n",
        "  access_token: Secret\n",
        "}\n",
        "data token_exchange_grant_type_wire: String = \"urn:ietf:params:oauth:grant-type:token-exchange\"\n",
        "data jwt_subject_token_type_wire: String = \"urn:ietf:params:oauth:token-type:jwt\"\n",
        "data access_token_requested_token_type_wire: String = \"urn:ietf:params:oauth:token-type:access_token\"\n",
        "service gcp.STS {\n",
        "  config { endpoint: \"https://sts.googleapis.com\" }\n",
        "  operation Exchange {\n",
        "    input {\n",
        "      subject_token: Secret\n",
        "      audience: String\n",
        "    }\n",
        "    output {\n",
        "      access_token: Secret from \"access_token\"\n",
        "    }\n",
        "    idempotent\n",
        "    transport rest {\n",
        "      method: POST,\n",
        "      path: \"/v1/token\",\n",
        "      body: StsTokenExchange {\n",
        body_fields,
        "\n",
        "      }\n",
        "    }\n",
        "    response {\n",
        "      200 => StsTokenResponse\n",
        "    }\n",
        "  }\n",
        "}\n",
        "func run(subject_token: Secret, audience: String) -> { access_token: Secret } {\n",
        "  token = gcp.STS.Exchange(subject_token: subject_token, audience: audience)\n",
        "  return { access_token: token.access_token }\n",
        "}\n",
    ]
    .concat()
}

#[test]
fn sts_typed_body_emits_expected_oauth_urns_in_prepared_request() {
    let source = sts_transport_module_source(
        r#"        grant_type: TokenExchange {}
        subject_token: subject_token
        subject_token_type: Jwt
        audience: audience
        requested_token_type: RequestAccessToken"#,
    );
    let typed = typed_project_from_sources(&[("dsl/extdeps/cloud/gcp/sts.dag", &source)]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let prepare_node = dag
        .nodes
        .iter()
        .find(|node| node.id.0.contains("prepare_transport") && node.id.0.contains("Exchange"))
        .expect("prepare transport node for STS.Exchange should exist");

    let metadata = match &prepare_node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => op
            .service_call_metadata()
            .expect("service metadata should be preserved"),
        _ => panic!("expected opaque lowered node"),
    };

    let spec = metadata.spec.as_ref().expect("spec should be present");
    let ServiceOperationSpec::Rest(rest_spec) = spec else {
        panic!("expected REST spec");
    };
    let op = GenericPrepareOp {
        spec: ServiceOperationSpec::Rest(rest_spec.clone()),
    };

    let mut inputs = HashMap::new();
    inputs.insert(
        "subject_token".to_string(),
        Value::Secret(SecretString::new("tok123")),
    );
    inputs.insert(
        "audience".to_string(),
        Value::Str(
            "projects/123/locations/global/workloadIdentityPools/pool/providers/provider"
                .to_string(),
        ),
    );

    let outputs = op
        .execute(inputs)
        .expect("prepare op should build a request");
    match outputs.get("request") {
        Some(Value::Request(TransportRequest::Rest(request))) => {
            assert_eq!(request.method, HttpMethod::Post);
            assert_eq!(request.url, "https://sts.googleapis.com/v1/token");
            let body = request
                .body
                .as_ref()
                .expect("POST request should have a body");
            assert_eq!(
                body["grant_type"],
                "urn:ietf:params:oauth:grant-type:token-exchange"
            );
            assert_eq!(body["subject_token"], "tok123");
            assert_eq!(
                body["subject_token_type"],
                "urn:ietf:params:oauth:token-type:jwt"
            );
            assert_eq!(
                body["audience"],
                "projects/123/locations/global/workloadIdentityPools/pool/providers/provider"
            );
            assert_eq!(
                body["requested_token_type"],
                "urn:ietf:params:oauth:token-type:access_token"
            );
        }
        other => panic!("expected REST request, got {other:?}"),
    }
}

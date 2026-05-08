//! **Layer:** integration
//!
//! Brief #2219 receipt: one lowered endpoint-bearing fixture feeds the Shape B
//! OpenAPI 3.1 YAML projection and the canonical route projection used as the
//! interim backend exposure set. The cross-target equality test is interim until
//! a cross-target TestPredicate variant exists.

use std::collections::{BTreeMap, BTreeSet};

use v3_compiler::compile_to_dag;
use v3_compiler::omni_shape_b_openapi::{extract_rest_routes, project_openapi_yaml, RestRoute};

const OMNI_SERVICE_FIXTURE: &str = r#"
module t.openapi_demo

import std.types { GET, POST }
import std.effects { PathTemplate, LiteralToken, ParamToken }
import v3.std.services { RestEndpointBinding }

type DemoOperation {
  endpoint: RestEndpointBinding
}

type NotAServiceOperation {
  endpoint: String
}

type MimicEndpointBinding {
  method: HttpMethod
  path:   PathTemplate
}

type MimicServiceOperation {
  endpoint: MimicEndpointBinding
}

data omni_service_operations: List<DemoOperation> = [
  {
    endpoint: {
      method: GET,
      path: { tokens: [LiteralToken { text: "users" }] }
    }
  },
  {
    endpoint: {
      method: POST,
      path: { tokens: [LiteralToken { text: "users" }] }
    }
  },
  {
    endpoint: {
      method: GET,
      path: {
        tokens: [LiteralToken { text: "users/" }, ParamToken { name: "id" }]
      }
    }
  },
  {
    endpoint: {
      method: GET,
      path: {
        tokens: [
          LiteralToken { text: "orgs/" },
          ParamToken { name: "org" },
          LiteralToken { text: "/repos/" },
          ParamToken { name: "repo" }
        ]
      }
    }
  },
  {
    endpoint: {
      method: POST,
      path: {
        tokens: [
          LiteralToken { text: "secrets/" },
          ParamToken { name: "secret_name" },
          LiteralToken { text: ":addVersion" }
        ]
      }
    }
  },
  {
    endpoint: {
      method: GET,
      path: { tokens: [LiteralToken { text: "a-b" }] }
    }
  },
  {
    endpoint: {
      method: GET,
      path: { tokens: [LiteralToken { text: "a_b" }] }
    }
  }
]

data non_service_endpoint_rows: List<NotAServiceOperation> = [
  { endpoint: "not a route" }
]

data same_shape_non_service_rows: List<MimicServiceOperation> = [
  {
    endpoint: {
      method: GET,
      path: { tokens: [LiteralToken { text: "same-shape-but-not-service" }] }
    }
  }
]
"#;

fn compile_omni_service_fixture() -> v3_compiler::Dag {
    std::thread::Builder::new()
        .name("m1_5_openapi_fixture_compile".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            compile_to_dag(
                OMNI_SERVICE_FIXTURE,
                "m1_5_omni_shape_b_openapi_fixture.dag",
            )
            .expect("Shape B OpenAPI demo fixture compiles")
        })
        .expect("spawn larger-stack compile thread")
        .join()
        .expect("larger-stack compile thread completes")
}

fn expected_routes() -> BTreeSet<RestRoute> {
    BTreeSet::from([
        RestRoute {
            method: "GET".to_string(),
            path: "/users".to_string(),
            path_parameters: vec![],
        },
        RestRoute {
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
            path_parameters: vec!["id".to_string()],
        },
        RestRoute {
            method: "GET".to_string(),
            path: "/orgs/{org}/repos/{repo}".to_string(),
            path_parameters: vec!["org".to_string(), "repo".to_string()],
        },
        RestRoute {
            method: "POST".to_string(),
            path: "/secrets/{secret_name}:addVersion".to_string(),
            path_parameters: vec!["secret_name".to_string()],
        },
        RestRoute {
            method: "GET".to_string(),
            path: "/a-b".to_string(),
            path_parameters: vec![],
        },
        RestRoute {
            method: "GET".to_string(),
            path: "/a_b".to_string(),
            path_parameters: vec![],
        },
        RestRoute {
            method: "POST".to_string(),
            path: "/users".to_string(),
            path_parameters: vec![],
        },
    ])
}

fn openapi_yaml_routes(yaml: &str) -> BTreeSet<RestRoute> {
    let mut routes: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut current_path: Option<String> = None;
    let mut current_route: Option<(String, String)> = None;
    for line in yaml.lines() {
        if let Some(path) = line.strip_prefix("  '").and_then(|s| s.strip_suffix("':")) {
            current_path = Some(path.replace("''", "'"));
            current_route = None;
            continue;
        }
        for method in ["get", "post", "put", "patch", "delete", "head", "options"] {
            if line == format!("    {method}:") {
                let route = (
                    method.to_ascii_uppercase(),
                    current_path
                        .clone()
                        .expect("method appears under a path in emitted YAML"),
                );
                routes.entry(route.clone()).or_default();
                current_route = Some(route);
            }
        }
        if let Some(parameter) = line
            .strip_prefix("        - name: ")
            .or_else(|| line.strip_prefix("          name: "))
        {
            let route = current_route
                .as_ref()
                .expect("parameter appears under an operation");
            routes
                .get_mut(route)
                .expect("route exists before parameter")
                .insert(yaml_scalar_value(parameter));
        }
    }
    routes
        .into_iter()
        .map(|((method, path), parameters)| RestRoute {
            method,
            path,
            path_parameters: parameters.into_iter().collect(),
        })
        .collect()
}

fn yaml_scalar_value(value: &str) -> String {
    if let Some(single_quoted) = value
        .strip_prefix('\'')
        .and_then(|inner| inner.strip_suffix('\''))
    {
        return single_quoted.replace("''", "'");
    }
    if let Some(double_quoted) = value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
    {
        return double_quoted
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    value.to_string()
}

fn compile_omni_service_fixture_counted(count: &mut usize) -> v3_compiler::Dag {
    *count += 1;
    compile_omni_service_fixture()
}

#[test]
fn omni_layers_share_one_node_tree() {
    let mut compile_count = 0;
    let dag = compile_omni_service_fixture_counted(&mut compile_count);

    let _canonical_routes = extract_rest_routes(&dag).expect("canonical route projection extracts");
    let _openapi = project_openapi_yaml(&dag).expect("Shape B OpenAPI projects from shared DAG");

    assert_eq!(
        compile_count, 1,
        "backend and Shape B OpenAPI projections must consume the same compile_to_dag \
         result; recompiling per target would break the structural-fold receipt."
    );
}

#[test]
fn shape_b_openapi_projection_produces_3_1_yaml_for_rest_operations() {
    let dag = compile_omni_service_fixture();
    let yaml = project_openapi_yaml(&dag).expect("OpenAPI YAML projects");

    assert!(yaml.starts_with("openapi: 3.1.0\n"));
    assert!(yaml.contains("  '/users':\n"));
    assert!(yaml.contains("    get:\n"));
    assert!(yaml.contains("    post:\n"));
    assert!(yaml.contains("  '/users/{id}':\n"));
    assert!(yaml.contains("  '/orgs/{org}/repos/{repo}':\n"));
    assert!(yaml.contains("  '/secrets/{secret_name}:addVersion':\n"));
    assert!(yaml.contains("      operationId: get_a_x2D_b\n"));
    assert!(yaml.contains("      operationId: get_a_x5F_b\n"));
    assert_eq!(yaml.matches("  '/users':\n").count(), 1);
    assert!(yaml.contains("      parameters:\n        - name: \"id\"\n          in: path\n          required: true\n          schema:\n            type: string\n"));
    assert!(yaml.contains("        - name: \"org\"\n          in: path\n          required: true\n          schema:\n            type: string\n        - name: \"repo\"\n          in: path\n          required: true\n          schema:\n            type: string\n"));
    assert_eq!(openapi_yaml_routes(&yaml), expected_routes());
}

#[test]
fn openapi_routes_match_canonical_dag_routes_interim() {
    let dag = compile_omni_service_fixture();

    // Interim until cross-target TestPredicate exists: both projections are
    // extracted from the same compiled DAG and compared here in `tests/`.
    // Rust target does not yet expose a structured route projection, so this
    // shared DAG extraction is the canonical backend exposure set for Brief #1.
    let canonical_routes = extract_rest_routes(&dag).expect("canonical route projection extracts");
    let openapi_routes =
        openapi_yaml_routes(&project_openapi_yaml(&dag).expect("OpenAPI YAML projects"));

    assert_eq!(canonical_routes, expected_routes());
    assert_eq!(openapi_routes, canonical_routes);
}

#[test]
fn openapi_projection_ignores_non_service_endpoint_fields() {
    let dag = compile_omni_service_fixture();

    assert_eq!(
        extract_rest_routes(&dag).expect("canonical route projection extracts"),
        expected_routes(),
        "Only declarations whose list element type carries the canonical \
         services.dag RestEndpointBinding field should become OpenAPI routes."
    );
}

#[test]
fn openapi_projection_ignores_same_shape_non_service_endpoint_binding() {
    let dag = compile_omni_service_fixture();

    let routes = extract_rest_routes(&dag).expect("canonical route projection extracts");

    assert_eq!(routes, expected_routes());
    assert!(
        !routes
            .iter()
            .any(|route| route.path == "/same-shape-but-not-service"),
        "A user-authored endpoint record with the same method/path shape must not become \
         an OpenAPI route unless its field type is the canonical RestEndpointBinding."
    );
}

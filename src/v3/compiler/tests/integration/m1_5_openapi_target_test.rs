//! **Layer:** integration
//!
//! Brief #2219 receipt: one lowered endpoint-bearing fixture feeds both the
//! OpenAPI 3.1 YAML projection and the canonical route projection used as the
//! interim backend exposure set. The cross-target equality test is interim until
//! a cross-target TestPredicate variant exists.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use v3_compiler::compile_to_dag;
use v3_compiler::emit::openapi_target::{emit_openapi_yaml, extract_rest_routes, RestRoute};

static COMPILE_COUNT: AtomicUsize = AtomicUsize::new(0);

const OMNI_SERVICE_FIXTURE: &str = r#"
module t.openapi_demo

import std.types { GET, POST }
import std.effects { PathTemplate, LiteralToken, ParamToken }
import v3.std.services { RestEndpointBinding }

type DemoOperation {
  endpoint: RestEndpointBinding
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
        tokens: [LiteralToken { text: "users" }, ParamToken { name: "id" }]
      }
    }
  }
]
"#;

fn compile_omni_service_fixture() -> v3_compiler::Dag {
    COMPILE_COUNT.fetch_add(1, Ordering::SeqCst);
    std::thread::Builder::new()
        .name("m1_5_openapi_fixture_compile".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            compile_to_dag(OMNI_SERVICE_FIXTURE, "m1_5_openapi_target_fixture.dag")
                .expect("OpenAPI demo fixture compiles")
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
        },
        RestRoute {
            method: "GET".to_string(),
            path: "/users/{id}".to_string(),
        },
        RestRoute {
            method: "POST".to_string(),
            path: "/users".to_string(),
        },
    ])
}

fn openapi_yaml_routes(yaml: &str) -> BTreeSet<RestRoute> {
    let mut routes = BTreeSet::new();
    let mut current_path: Option<String> = None;
    for line in yaml.lines() {
        if let Some(path) = line.strip_prefix("  '").and_then(|s| s.strip_suffix("':")) {
            current_path = Some(path.replace("''", "'"));
            continue;
        }
        for method in ["get", "post", "put", "patch", "delete", "head", "options"] {
            if line == format!("    {method}:") {
                routes.insert(RestRoute {
                    method: method.to_ascii_uppercase(),
                    path: current_path
                        .clone()
                        .expect("method appears under a path in emitted YAML"),
                });
            }
        }
    }
    routes
}

#[test]
fn omni_layers_share_one_node_tree() {
    COMPILE_COUNT.store(0, Ordering::SeqCst);
    let dag = compile_omni_service_fixture();

    let _canonical_routes = extract_rest_routes(&dag).expect("canonical route projection extracts");
    let _openapi = emit_openapi_yaml(&dag).expect("OpenAPI target emits from shared DAG");

    assert_eq!(
        COMPILE_COUNT.load(Ordering::SeqCst),
        1,
        "Rust and OpenAPI projections must consume the same compile_to_dag \
         result; recompiling per target would break the structural-fold receipt."
    );
}

#[test]
fn openapi_emit_produces_3_1_yaml_for_rest_operations() {
    let dag = compile_omni_service_fixture();
    let yaml = emit_openapi_yaml(&dag).expect("OpenAPI YAML emits");

    assert!(yaml.starts_with("openapi: 3.1.0\n"));
    assert!(yaml.contains("  '/users':\n"));
    assert!(yaml.contains("    get:\n"));
    assert!(yaml.contains("    post:\n"));
    assert!(yaml.contains("  '/users/{id}':\n"));
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
    let openapi_routes = openapi_yaml_routes(&emit_openapi_yaml(&dag).expect("OpenAPI YAML emits"));

    assert_eq!(canonical_routes, expected_routes());
    assert_eq!(openapi_routes, canonical_routes);
}

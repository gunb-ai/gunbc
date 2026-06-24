//! Route-A periphery emitter fixes (measure-tower ride families), proven by execution on the
//! HostNative (`--emit-fresh`) path. Each fixture is named under `src/v1/...` so `compile_dag_named`
//! exercises the same HostNative emit as the assembled crate.
//!
//! Families witnessed here:
//! - E0599/E0277: a generic fn whose body requires `Clone` on a type param must emit the bound.
//! - length routing: `.length()` routes BY RECEIVER through the existing method->realization
//!   dispatch — collection receiver -> `count` (`.len() as i64`), String receiver ->
//!   `v1_rt::string_length` — never a bare `v1_rt::length` (which does not exist).

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit_host(path: &str, src: &str) -> String {
    compile_dag_named(path, src, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fn_body(emitted: &str, name: &str) -> String {
    let needle = format!("fn {name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fn `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[needle.len()..]
        .find("\npub fn ")
        .map(|i| i + needle.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

// ---- E0599/E0277: generic fn Clone bound -----------------------------------------------------

const GENERIC_CLONE_FIXTURE: &str = concat!(
    "module genclone.fixture\n",
    "import std.nat { Nat }\n\n",
    "fn cardinality<T>(xs: List<T>) -> Int {\n  fold(xs, init: 0, f: (acc, _) => acc + 1)\n}\n"
);

#[test]
fn generic_fn_emits_clone_bound() {
    let emitted = emit_host("src/v1/generic_clone_fixture.dag", GENERIC_CLONE_FIXTURE);
    let body = fn_body(&emitted, "cardinality");
    // The fold lowers to `.iter().cloned()`, which requires `T: Clone`; the emitted signature
    // must carry the bound or rustc raises E0277/E0599.
    assert!(
        body.contains("<T: Clone>"),
        "a generic fn whose body clones a type param must emit the `T: Clone` bound, got:\n{body}"
    );
}

// ---- length routing (receiver-keyed) ---------------------------------------------------------

const LENGTH_FIXTURE: &str = concat!(
    "module lengthroute.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Bar {\n  x: Nat\n}\n\n",
    "fn collection_len(xs: List<Bar>) -> Int {\n  xs.length()\n}\n\n",
    "fn string_len(s: String) -> Int {\n  s.length()\n}\n"
);

#[test]
fn collection_length_routes_to_count() {
    let emitted = emit_host("src/v1/length_route_fixture.dag", LENGTH_FIXTURE);
    let body = fn_body(&emitted, "collection_len");
    // A collection receiver routes `.length()` to the `count` realization (`.len() as i64`).
    assert!(
        body.contains(".len() as i64"),
        "collection `.length()` must route to the count realization, got:\n{body}"
    );
    // It must never emit the non-existent `v1_rt::length`.
    assert!(
        !body.contains("v1_rt::length"),
        "no bare v1_rt::length may be emitted, got:\n{body}"
    );
}

#[test]
fn string_length_routes_to_string_length() {
    let emitted = emit_host("src/v1/length_route_fixture.dag", LENGTH_FIXTURE);
    let body = fn_body(&emitted, "string_len");
    // A String receiver routes `.length()` to the `string_length` realization (char-count,
    // pass-by-ref) — the same FreeMonoid cardinality hom over a different carrier.
    assert!(
        body.contains("v1_rt::string_length(&"),
        "String `.length()` must route to v1_rt::string_length(&recv), got:\n{body}"
    );
    assert!(
        !body.contains("v1_rt::length"),
        "no bare v1_rt::length may be emitted, got:\n{body}"
    );
}

//! Gate-1 repr-mismatch secondaries (bold-fox-810): collection Vector/FreeMonoid and
//! Optional/Option construction grounding — the same native-form == modeled-form move as
//! the text-carrier root (#7131) and the numeric tower (#5428).

use crate::helpers::{compile_dag_named, compile_dag_target};
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_emit_rust::{
    is_host_freemonoid_vec_alias, is_host_optional_carrier_alias, rust_seed_host_container_base,
};
use v1_compiler::v1_compiler_infer_emit_info::RustCorpusRepr;

#[test]
fn faithful_freemonoid_vec_alias_eligible_in_faithful_repr() {
    assert!(
        is_host_freemonoid_vec_alias("FreeMonoid".to_string()),
        "FreeMonoid must ground to Vec in FaithfulFreeMonoid corpus repr"
    );
}

#[test]
fn faithful_container_base_grounds_list_to_vec() {
    let base =
        rust_seed_host_container_base("List".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        base.as_deref(),
        Some("Vec"),
        "List container base must ground to Vec in FaithfulFreeMonoid, got {base:?}"
    );
}

#[test]
fn faithful_freemonoid_container_base_grounds_to_vec() {
    let base =
        rust_seed_host_container_base("FreeMonoid".to_string(), RustCorpusRepr::FaithfulFreeMonoid);
    assert_eq!(
        base.as_deref(),
        Some("Vec"),
        "FreeMonoid container base must ground to Vec in FaithfulFreeMonoid, got {base:?}"
    );
}

#[test]
fn faithful_optional_carrier_alias_eligible() {
    assert!(
        is_host_optional_carrier_alias("Optional".to_string()),
        "modeled Optional coproduct must ground to native Option alias"
    );
}

#[test]
fn optional_coproduct_emits_native_option_alias() {
    let source =
        "module optsig.fixture\n\ntype Optional<T>\n  = Absent\n  | Present { value: T }\n";
    let emitted = compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("type Optional<T> = Option<T>"),
        "Optional coproduct must emit native Option alias, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("enum Optional"),
        "grounded Optional must not emit coproduct enum (got:\n{emitted})"
    );
}

#[test]
fn freemonoid_coproduct_emits_vec_alias() {
    let source = "module fmc.fixture\n\ntype FreeMonoid<T>\n  = Empty\n  | Snoc { prev: FreeMonoid<T>, item: T }\n";
    let emitted = compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("type FreeMonoid<T> = Vec<T>"),
        "FreeMonoid coproduct must emit native Vec alias, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("enum FreeMonoid"),
        "grounded FreeMonoid must not emit coproduct enum (got:\n{emitted})"
    );
}

#[test]
fn optional_applied_type_renders_option_in_signature() {
    let source = "module optsig.fixture\n\ntype Optional<T>\n  = Absent\n  | Present { value: T }\n\nfn maybe_int(flag: Bool) -> Optional<Int> {\n  if flag { Present { value: 1 } } else { none }\n}\n";
    let emitted = compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    let sig_start = emitted
        .find("fn maybe_int")
        .unwrap_or_else(|| panic!("fixture fn not emitted:\n{emitted}"));
    let sig = &emitted[sig_start..];
    let body_open = sig.find(" {").unwrap_or(sig.len());
    let sig_slice = &sig[..body_open];
    assert!(
        sig_slice.contains("Option<"),
        "Optional<Int> return must render Option<..> in signature, got:\n{sig_slice}"
    );
    assert!(
        !sig_slice.contains("Optional<"),
        "modeled Optional must not appear in grounded signature, got:\n{sig_slice}"
    );
}

fn faithful_emit(source: &str) -> String {
    compile_dag_named("test.dag", source, RenderTarget::Rust)
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

#[test]
fn faithful_freemonoid_empty_value_grounds_to_vec_literal() {
    let source = concat!(
        "module fmcval.fixture\n",
        "type FreeMonoid<T>\n  = Empty\n  | Cons { head: T, tail: FreeMonoid<T> }\n\n",
        "fn empty_list() -> FreeMonoid<Int> {\n  Empty\n}\n"
    );
    let body = fn_body(&faithful_emit(source), "empty_list");
    assert!(
        body.contains("vec![]"),
        "Empty must lower to vec![] once FreeMonoid grounds to Vec, got:\n{body}"
    );
    assert!(
        !body.contains("FreeMonoid::Empty"),
        "grounded Empty must not emit enum constructor, got:\n{body}"
    );
}

#[test]
fn faithful_freemonoid_cons_value_grounds_to_vec_push() {
    let source = concat!(
        "module fmcval.fixture\n",
        "type FreeMonoid<T>\n  = Empty\n  | Cons { head: T, tail: FreeMonoid<T> }\n\n",
        "fn one() -> FreeMonoid<Int> {\n  Cons { head: 1, tail: Empty }\n}\n"
    );
    let body = fn_body(&faithful_emit(source), "one");
    assert!(
        body.contains("__cons_v") && body.contains("insert(0,"),
        "Cons must lower to native Vec push-front, got:\n{body}"
    );
    assert!(
        !body.contains("Cons {"),
        "grounded Cons must not emit coproduct constructor, got:\n{body}"
    );
}

#[test]
fn faithful_freemonoid_match_lowers_to_vec_is_empty() {
    let source = concat!(
        "module fmcval.fixture\n",
        "type FreeMonoid<T>\n  = Empty\n  | Cons { head: T, tail: FreeMonoid<T> }\n\n",
        "fn len(xs: FreeMonoid<Int>) -> Int {\n",
        "  match xs {\n",
        "    Empty => 0\n",
        "    Cons { head: _, tail: _ } => 1\n",
        "  }\n",
        "}\n"
    );
    let body = fn_body(&faithful_emit(source), "len");
    assert!(
        body.contains("__fm.is_empty()"),
        "FreeMonoid match must use native Vec is_empty lowering, got:\n{body}"
    );
    assert!(
        !body.contains("FreeMonoid::Empty"),
        "grounded match must not emit enum patterns, got:\n{body}"
    );
}

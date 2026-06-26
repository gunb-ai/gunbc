//! Type-parameter names in .dag use the lowercase idiom (e.g. `type List<element>`,
//! `MachineWidth<bits>`, `Map<key,value>`).  When emitting Rust those names must be converted to
//! UpperCamelCase — Rust's `non_camel_case_types` lint fires on lowercase type parameters, and
//! with `-D warnings` that becomes a hard E0.
//!
//! Fix sites in `05_emit_rust.dag`:
//!   • `emit_type_params` / `emit_type_params_with_clone_bound` — the `<P…>` declaration list
//!   • `render_rust_decl_type` / `render_rust_alias_rhs_type` — the param use-site in field/ret types
//!   • `rust_phantom_marker_inner` — unused-param PhantomData payload
//!   • `emit_parametric_phantom_opaque_struct` — opaque-struct PhantomData marker
//!   • FreeMonoid Vec alias elem_csv — `type FreeList<element> = Vec<element>`
//!   • TypeVariable id emission (lines 151 / 474) — inferred type-variable names
//!
//! Each test has a positive assertion (pascal-cased name is present) and a negative control
//! (lowercase name is absent) so the gate goes red when the casing is wrong in either direction.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit(name: &str, source: &str) -> String {
    compile_dag_named(name, source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

// ── record type with a single lowercase type param ────────────────────────────

const RECORD_FIXTURE: &str = concat!(
    "module type_param_casing.record\n\n",
    "type Box<element> {\n",
    "  value: element\n",
    "  tag: Int\n",
    "}\n",
);

#[test]
fn record_type_param_emits_upper_camel() {
    let emitted = emit("src/v1/type_param_casing_record.dag", RECORD_FIXTURE);
    assert!(
        emitted.contains("Box<Element>"),
        "lowercase `.dag` type param `element` must emit as `Element` in Rust struct decl:\n{emitted}"
    );
    assert!(
        !emitted.contains("Box<element>"),
        "lowercase `element` must not appear as a Rust type param:\n{emitted}"
    );
}

// ── type alias with unused params → PhantomData payload ───────────────────────

const ALIAS_UNUSED_FIXTURE: &str = concat!(
    "module type_param_casing.alias_unused\n\n",
    "import std.constructors { Phantom }\n\n",
    "type Marker<bits> = Phantom\n",
);

#[test]
fn alias_unused_param_phantom_emits_upper_camel() {
    let emitted = emit(
        "src/v1/type_param_casing_alias.dag",
        ALIAS_UNUSED_FIXTURE,
    );
    assert!(
        emitted.contains("PhantomData<Bits>"),
        "unused alias param `bits` must emit as `Bits` inside PhantomData:\n{emitted}"
    );
    assert!(
        !emitted.contains("PhantomData<bits>"),
        "lowercase `bits` must not appear inside PhantomData:\n{emitted}"
    );
}

// ── multi-param record: all params pascal-cased ────────────────────────────────

const MULTI_PARAM_FIXTURE: &str = concat!(
    "module type_param_casing.multi\n\n",
    "type Pair<key, value> {\n",
    "  first: key\n",
    "  second: value\n",
    "}\n",
);

#[test]
fn multi_param_record_all_upper_camel() {
    let emitted = emit("src/v1/type_param_casing_multi.dag", MULTI_PARAM_FIXTURE);
    assert!(
        emitted.contains("Pair<Key, Value>"),
        "lowercase params `key, value` must emit as `Key, Value`:\n{emitted}"
    );
    assert!(
        !emitted.contains("Pair<key") && !emitted.contains("Pair<value"),
        "no lowercase type param must survive in the Rust struct decl:\n{emitted}"
    );
}

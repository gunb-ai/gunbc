//! Measure-tower E0614: a struct field whose type grounds to a host-native Copy scalar
//! (Nat/Int -> i64 under #5428's HostNative grounding) must NOT be dereferenced on field
//! access. The pre-fix emitter treated a `Nat` magnitude field as boxed — Nat is structurally
//! recursive (`Succ{prev:Nat}`), hence in `recursive_types`, and has no `is_copy` checkpoint —
//! so `field_access_field_is_boxed` returned true and the access emitted `(*x.count).clone()`,
//! which fails `E0614: i64 cannot be dereferenced` once Nat is grounded to a bare i64. This is
//! exactly the shape inside `std.measure` (`measure_add`'s `a.count + b.count`, etc.) where the
//! magnitude `M` is instantiated to `Nat`.
//!
//! The fix is GENERAL (not Measure-specific): `field_access_field_is_boxed` short-circuits to
//! `false` when the field type grounds to a host scalar (`rust_seed_host_numeric_alias`), the
//! corpus_repr-keyed access-tail of #5428.
//!
//! Discriminating pair — the SAME `Wrap<Nat>` field accessed under the TWO corpus reprs, so the
//! only thing that differs is whether Nat is grounded:
//!   - HostNative (seed `--emit-fresh`): Nat grounds to i64 -> access must NOT deref.
//!   - FaithfulFreeMonoid: Nat is the genuine boxed recursive coproduct -> access STILL derefs.
//!
//! The pipeline picks HostNative iff a source path contains "src/v1" (the seed marker), so the
//! two helpers below differ only in the fixture path — that is the load-bearing control proving
//! the no-deref rule is grounded(host)-KEYED, not a blanket removal.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

// `Wrap<M>` is a two-field generic product (the second field prevents single-field collapse),
// so `w.item` is a genuine stored-field access routed through `field_access_field_is_boxed`,
// with the magnitude `M` instantiated to the recursive-leaf `Nat` — the exact `Measure<_,_,Nat>`
// shape.
const FIXTURE: &str = concat!(
    "module wrapderef.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Wrap<M> {\n  item: M\n  tag: Int\n}\n\n",
    "fn get_nat(w: Wrap<Nat>) -> Nat {\n  w.item\n}\n"
);

fn emit_at(path: &str) -> String {
    compile_dag_named(path, FIXTURE, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

// Body of `fn <name>` up to the next top-level `pub fn`.
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
fn grounded_scalar_field_access_not_dereferenced() {
    // HostNative (src/v1 path): Nat grounds to i64 (Copy) -> `w.item` must NOT deref.
    let body = fn_body(&emit_at("src/v1/wrap_deref_fixture.dag"), "get_nat");
    assert!(
        !body.contains("(*"),
        "under HostNative a grounded-scalar (Nat->i64) field must NOT be dereferenced, got:\n{body}"
    );
}

#[test]
fn faithful_boxed_nat_field_access_still_dereferenced() {
    // Control: the SAME field under FaithfulFreeMonoid (non-seed path), where Nat is the genuine
    // boxed recursive coproduct, STILL derefs — proving the no-deref rule is grounded(host)-KEYED,
    // not a blanket removal that would also break legitimately-boxed fields.
    let body = fn_body(&emit_at("test.dag"), "get_nat");
    assert!(
        body.contains("(*"),
        "under FaithfulFreeMonoid a genuinely boxed Nat field must still deref, got:\n{body}"
    );
}

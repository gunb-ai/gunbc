//! Measure-tower E0560: a record literal that names a TYPE ALIAS to an Rc-shared applied
//! struct must be constructed through the canonical struct, not the alias. The `.dag` source
//! `Gibibyte { count: count }` (where `type Gibibyte = Measure<Memory, Gibi, Nat>`) emitted
//! `Gibibyte { count: count }`, but `Gibibyte` resolves to `Rc<Measure<...>>` — you cannot
//! struct-construct through an Rc alias, and the emitter-synthesized `_phantom` field was
//! missing. Result: `E0560: struct Rc<Measure<...>> has no field named count`.
//!
//! Faithful fix (emitter-faithfulness, reusing existing resolution): resolve the alias to its
//! canonical struct, emit `Rc::new(Measure { count: count, _phantom: PhantomData })` — matching
//! the struct def the emitter itself emits. Like the deref fix this is HostNative-keyed (the
//! type aliases only render as `Rc<Measure<...>>` under HostNative), so the fixture is named
//! under src/v1 to exercise the same emit as `--emit-fresh`.

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

// `Meas<Q, M>` mirrors `Measure`: `Q` is a phantom param (unused in fields) so the emitter
// synthesizes a `_phantom`, and the struct is Rc-shared. `Giga` is a type alias to an applied
// instantiation, exactly the `type Gibibyte = Measure<...>` shape.
const FIXTURE: &str = concat!(
    "module aliasctor.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Meas<Q, M> {\n  count: M\n}\n\n",
    "type Giga = Meas<Nat, Nat>\n\n",
    "fn mk(c: Nat) -> Giga {\n  Giga { count: c }\n}\n"
);

fn emit_host() -> String {
    compile_dag_named("src/v1/alias_ctor_fixture.dag", FIXTURE, RenderTarget::Rust)
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

// The fn body with its signature line dropped. The signature `fn mk(...) -> Giga {` itself
// contains `Giga {`, which would false-match a struct-literal substring search; the
// alias-vs-canonical distinction lives strictly in the body.
fn fn_body_no_sig(emitted: &str, name: &str) -> String {
    let body = fn_body(emitted, name);
    match body.find('\n') {
        Some(i) => body[i + 1..].to_string(),
        None => String::new(),
    }
}

#[test]
fn alias_ctor_resolves_to_canonical_struct_with_phantom_and_rc() {
    let emitted = emit_host();
    let body = fn_body(&emitted, "mk");
    assert!(
        body.contains("Meas {")
            && body.contains("_phantom: std::marker::PhantomData")
            && body.contains("Rc::new("),
        "an alias-to-Rc<struct> ctor must emit `Rc::new(Meas {{ count: c, _phantom: PhantomData }})`, got:\n{body}"
    );
    // Negative: the alias name must NOT be used as the struct-literal head (it resolves to
    // `Rc<Meas<...>>`, which has no fields). Check the signature-stripped body so `-> Giga {`
    // in the fn signature does not false-match.
    let body_no_sig = fn_body_no_sig(&emitted, "mk");
    assert!(
        !body_no_sig.contains("Giga {"),
        "the alias name must not be used as a struct-literal head, got:\n{body_no_sig}"
    );
}

// Direct-struct discriminating control (mandatory, per bright-stag's §3 over-peel oracle).
// The alias resolver runs only behind the `!tn_is_known_struct` guard: it reuses
// `peel_alias_once_for_field_access` (resolve the alias use-site to its Conj struct) then
// `find_unique_struct_name_by_fields` to recover the canonical NAME from the struct's field
// set. A NON-alias direct struct is a KNOWN struct (`tn_is_known_struct`), so it must short-
// circuit BEFORE that resolution and name the STRUCT (`Direct`) directly — never get re-derived
// to some other nominal. This control is exactly what catches a regression that dropped the
// guard and let a direct-struct construction fall into the alias resolution path.
const DIRECT_FIXTURE: &str = concat!(
    "module directctor.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Direct<Q, M> {\n  count: M\n}\n\n",
    "fn mkd(c: Nat) -> Direct<Nat, Nat> {\n  Direct { count: c }\n}\n"
);

#[test]
fn direct_struct_ctor_names_struct_not_first_field() {
    let emitted = compile_dag_named(
        "src/v1/direct_ctor_fixture.dag",
        DIRECT_FIXTURE,
        RenderTarget::Rust,
    )
    .files
    .iter()
    .map(|f| f.content.clone())
    .collect::<Vec<_>>()
    .join("\n");
    let body = fn_body(&emitted, "mkd");
    assert!(
        body.contains("Direct {"),
        "a direct (non-alias) struct ctor must name the STRUCT `Direct`, got:\n{body}"
    );
    // Over-peel guard: the first FIELD name must never become the struct-literal head — the
    // failure mode if a direct struct were routed through the alias name-recovery path.
    assert!(
        !body.contains("count {"),
        "a direct struct ctor must not over-peel to the first field name as the literal head, got:\n{body}"
    );
}

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

#[test]
fn alias_ctor_resolves_to_canonical_struct_with_phantom_and_rc() {
    let body = fn_body(&emit_host(), "mk");
    assert!(
        body.contains("Meas {")
            && body.contains("_phantom: std::marker::PhantomData")
            && body.contains("Rc::new("),
        "an alias-to-Rc<struct> ctor must emit `Rc::new(Meas {{ count: c, _phantom: PhantomData }})`, got:\n{body}"
    );
    // Negative: the alias name must NOT be used as the struct-literal head (it resolves to
    // `Rc<Meas<...>>`, which has no fields).
    assert!(
        !body.contains("Giga {"),
        "the alias name must not be used as a struct-literal head, got:\n{body}"
    );
}

// Direct-struct discriminating control (mandatory, per bright-stag's §3 over-peel oracle):
// the resolver reuses `resolved_type_name`, whose ELSE branch (inferred absent) peels to the
// FIRST CHILD = the first FIELD name, not the struct name. A NON-alias direct struct must
// therefore name the STRUCT (`Direct`), never its first field (`count`). It does so because the
// `!tn_is_known_struct` guard short-circuits before `resolved_type_name` is ever invoked on a
// known struct — so this control is exactly what catches a regression that dropped that guard
// and let the over-peel reach a direct-struct construction.
const DIRECT_FIXTURE: &str = concat!(
    "module directctor.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Direct<Q, M> {\n  count: M\n}\n\n",
    "fn mkd(c: Nat) -> Direct<Nat, Nat> {\n  Direct { count: c }\n}\n"
);

#[test]
fn direct_struct_ctor_names_struct_not_first_field() {
    let emitted = compile_dag_named("src/v1/direct_ctor_fixture.dag", DIRECT_FIXTURE, RenderTarget::Rust)
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
    // Over-peel guard: the first FIELD name must never become the struct-literal head — that is
    // exactly the `resolved_type_name` else-branch (first-child = field) failure mode.
    assert!(
        !body.contains("count {"),
        "a direct struct ctor must not over-peel to the first field name as the literal head, got:\n{body}"
    );
}

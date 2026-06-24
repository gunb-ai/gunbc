//! Measure-tower E0573: a VALUE enum-variant used in a TYPE-argument position must collapse to
//! the unit type `()` in the emitted Rust, never render as a bare type name. `Measure<Q, S, M>`
//! declares Q/S as type-params but the corpus instantiates them with `Quantity`/`Scale` VALUE
//! variants (`Measure<Time, S, Nat>`, `Measure<Memory, One, Nat>`). Rust cannot carry a value in a
//! type-arg slot: rendering the variant by name produced `error[E0573]: expected type, found
//! variant `Time`` wherever the variant's emitter-synthesized ZST marker was not in scope (e.g. a
//! consumer module like `std.realization_schedule`).
//!
//! The fix collapses a value-occupied type-arg slot to `()` (`is_value_variant_type_arg`), the same
//! lossy-but-sound move the emitter already makes for a Nat width LITERAL (`MachineWidth<8>` ->
//! `MachineWidth<()>`). The `.dag` authority keeps Q/S (they give a real type-level distinction
//! between a Time measure and a Length measure); only the Rust seed projection collapses the slot.
//!
//! Discriminating witness for the ACTUAL root cause: `Time` is a unit variant of MORE THAN ONE
//! coproduct in the real corpus (`Quantity`, `realization_measurement`, `realization_width`, ...),
//! so the prior "exactly one owner" detector (`is_phantom_unit_variant_type_arg`) failed to fire
//! and emitted the bare name. `OtherClock` below reproduces that ambiguity, so this test goes RED
//! under the exactly-one predicate and GREEN under the ">=1 owner" fix.
//!
//! Fixtures are named under `src/v1/...` so `compile_dag_named` exercises the same HostNative emit
//! as the assembled `--emit-fresh` crate.

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

// Mirrors the measure tower: a 3-param product whose first two params are phantom (used only in
// type-arg slots) and instantiated with `Quantity`/`Scale` value variants, plus a forwarded
// type-param `S` and a real magnitude type `Nat`. `OtherClock` makes `Time` an AMBIGUOUS variant
// (owned by two coproducts) -- the exact shape the exactly-one detector could not collapse. The
// `CostAccount` struct exercises the FIELD render path (which routes through the generic node
// renderer) in addition to the alias and fn-signature paths.
const FIXTURE: &str = concat!(
    "module measureunit.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Quantity = Time | Memory | Currency\n",
    "type OtherClock = Time | Tick\n",
    "type Scale = One | Micro\n\n",
    "type Measure<Q, S, M> {\n  count: M\n}\n\n",
    "type ByteSize = Measure<Memory, One, Nat>\n\n",
    "type CostAccount<S> {\n  t: Measure<Time, S, Nat>\n}\n\n",
    "fn time_measure<S>(count: Nat) -> Measure<Time, S, Nat> {\n  Measure { count: count }\n}\n"
);

#[test]
fn value_variant_type_args_collapse_to_unit() {
    let emitted = emit_host("src/v1/measure_unit_fixture.dag", FIXTURE);

    // Negative: a value variant must NEVER be emitted as a bare type-arg name. These are the exact
    // E0573 shapes (`expected type, found variant`), and `Time` is the AMBIGUOUS one.
    for variant in ["Memory", "One", "Time", "Currency"] {
        assert!(
            !emitted.contains(&format!("Measure<{variant}"))
                && !emitted.contains(&format!(", {variant},"))
                && !emitted.contains(&format!(", {variant}>")),
            "the `{variant}` value variant must not appear as a type arg, got:\n{emitted}"
        );
    }

    // Positive: the `ByteSize` alias's value-variant slots (`Memory`, `One`) both collapse to `()`.
    assert!(
        emitted.contains("Measure<(), ()"),
        "value-variant type-args (Memory, One) must collapse to `()`, got:\n{emitted}"
    );

    // Control (discriminating): a forwarded *type-param* `S` is NOT a value variant, so it must
    // survive uncollapsed -- the rule keys on value-occupancy, not blanket arg removal. Present in
    // both `time_measure`'s return type and `CostAccount`'s field type: `Measure<(), S, ...>`.
    assert!(
        emitted.contains("Measure<(), S,"),
        "a genuine type-param `S` must NOT collapse to `()`, got:\n{emitted}"
    );
}

// Control: a struct whose type-arg is a REAL type (not a value variant) must render that type
// normally -- proving the collapse is value-keyed, not a blanket `()`-ing of every generic arg
// (which would also wreck the algebra tower's `Magma<Int>`-shaped uses).
const REAL_ARG_FIXTURE: &str = concat!(
    "module realarg.fixture\n",
    "import std.nat { Nat }\n\n",
    "type Box<T> {\n  item: T\n}\n\n",
    "type NatBox = Box<Nat>\n"
);

#[test]
fn real_type_arg_does_not_collapse() {
    let emitted = emit_host("src/v1/real_arg_fixture.dag", REAL_ARG_FIXTURE);
    // `Box<Nat>` -> `Box<i64>` (Nat grounds to i64), never `Box<()>`.
    assert!(
        emitted.contains("Box<i64>"),
        "a real type arg (Nat->i64) must render normally, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("Box<()>"),
        "a real type arg must NOT collapse to `()`, got:\n{emitted}"
    );
}

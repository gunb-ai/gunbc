# Root B — primitive representation fork: live measurement and root cause (2026-08-16)

Session `eager-deer-389`, working the Root B lane of
[the self-host cargo refusal root partition](../plans/self-host-cargo-refusal-root-partition.md).

**Every number here was measured today** with the instrument that section names
(`gunbc compile --target rust`, then `cargo build` on the emitted crate), in this session's own
worktree. Nothing is carried forward from the July snapshots; where a July figure is quoted it is
labelled as such and is being superseded, not relied on.

## 1. The root cause, stated exactly

`v1.compiler.04_infer` `rust_corpus_repr` selects the Rust representation of the modeled
primitives from a single Boolean:

```
fn corpus_has_v1_seed_source_indices(modules: List<TypedModule>) -> Bool {
  modules |> any(m =>
    map_keys(m.type_env.source_indices) |> any(k => contains(k, "/v1/") || contains(k, "src/v1")))
}

fn rust_corpus_repr(has_v1_seed: Bool) -> RustCorpusRepr {
  if has_v1_seed { HostNative } else { FaithfulFreeMonoid }
}
```

`HostNative` grounds the numeric tower to the host — `v1.compiler.05_emit_rust`
`rust_seed_host_numeric_alias` renders `Nat` and `Int` as `i64`, gated on
`corpus_repr_is_host`. `FaithfulFreeMonoid` does not, so the modeled carrier survives into
emitted Rust.

**The v1 seed's closure contains `src/v1` sources, so the seed is emitted `HostNative` and
compiles. A v2 compiler module's closure contains none, so it is emitted `FaithfulFreeMonoid`
and every arithmetic, comparison and literal site against a modeled primitive refuses.**

The discriminator is a **path substring over the closure's source keys** — not a modeled fact
about the target. DESIGN §3 states the relevant law directly: a fact's home is its layer, not its
file; paths are discriminators, not gospel.

## 2. The executed receipt — one source module, two representations

The same file, `dag/std/nat.dag`, emitted twice.

Committed v1 seed (`src/v1/stage0/src/std_nat.rs`, `HostNative`):

```rust
pub type Nat = i64;

pub fn nat_compare(a: Nat, b: Nat) -> Ordering {
    if (a.clone() < b.clone()) { Ordering::Less } else { ... }
}
```

Emitted today into a pure-v2 closure
(`gunbc compile --source-root dag --source-root src/v2 --entry dag/std/nat.dag --target rust`,
`FaithfulFreeMonoid`):

```rust
pub type Nat = Rc<crate::std_algebra::CommutativeSemiring<Magnitude>>;

pub fn nat_compare(a: Nat, b: Nat) -> Ordering {
    if (a.clone() < b.clone()) { Ordering::Less } else { ... }
}
```

`cargo build --release --lib` on that crate, executed:

```
4 error[E0369]   binary operation `<` / `>` cannot be applied to type
                 `Rc<CommutativeSemiring<Magnitude>>`
1 error[E0432]   unresolved import `crate::std_types` (unrelated — namespace, not this root)
```

**This three-file closure is a ~30-second discriminating reproducer for Root B**, and is a far
cheaper instrument than a compiler-module probe (which takes tens of minutes). Recommended for
anyone testing a fix in this family.

## 3. The double-Rc row is not a second defect

The partition asked whether `Rc<Rc<CommutativeSemiring<Magnitude>>>` is a distinct wrapping bug.
It is not. Read the two emitted alias declarations from the same run:

```rust
// src/std_nat.rs
pub type Nat = Rc<crate::std_algebra::CommutativeSemiring<Magnitude>>;
// src/v2_std_integer.rs
pub type Int = GroupCompletion<Rc<Nat>>;
```

`Nat` already carries the `Rc`; the use site wraps it again. The double-Rc signature is the
single-Rc signature seen through one alias hop, so it dissolves with the same fix and must not be
sized as separate work.

## 4. Live sizing, and how it differs from July

`src/v2/compiler/06_translate.dag`, probed today. Emission succeeds (92 files) and `cargo`
refuses with 671 coded errors. Root B's share, by cause signature:

| signature | count |
|---|---:|
| E0369 `cannot multiply Rc<Rc<CommutativeSemiring<Magnitude>>> by ...` | 15 |
| E0369 `==` / `<` / `>` on `Rc<Rc<CommutativeSemiring<Magnitude>>>` | 10 |
| E0308 expected `bool` found `Bool` | 11 |
| E0369 `cannot multiply/divide` mixed single/double Rc `CommutativeSemiring<Magnitude>` | 5 |
| E0308 `i64` ↔ `Rc<Rc<CommutativeSemiring<Magnitude>>>` (both directions) | 5 |
| E0369 `<` on `Rc<v2_std_nat::Nat>`, `divide Rc<v2_std_nat::Nat> by {integer}` | 3 |
| E0369 `==` on `Rc<Measure<(), (), Rc<Rc<CommutativeSemiring<Magnitude>>>>>` | 2 |

**≈51 in this module**, against 57 total E0369 — so Root B is the *dominant* E0369 cause here,
while being only ~6% of this module's E0308. Concentrated in `src/std_measure.rs` (60 diagnostic
citations), which is a floor file shared by every closure, so the fix is closure-wide rather than
module-local.

Three honest differences from the July table, stated rather than smoothed:

- **The `found {integer}` rows are gone in this module.** July recorded 50 + 40 + 33 + 10
  occurrences of a modeled carrier against `{integer}`; today's equivalents read `i64` and number
  5. The mechanism is the same (a native numeric meeting a modeled carrier); the literal is now
  resolved to `i64` before the mismatch is reported.
- **`expected bool found Bool` is 11 here, not 60.** The July 60 was summed across seven modules'
  files, not one.
- **This is one module.** The July counts were closure-denominated across the canonical seven. A
  single module's histogram cannot be scaled to a corpus figure, and I am not doing so. The
  cross-module figure is owed and is listed as open below.

## 5. The `Bool` half is the same root with a second, independent defect on top

`Bool` *is* in the Rust checkpoint table (`dag/extdeps/languages/rust/types.dag`:
`{ dag_name: "Bool", target_type: "bool", ... }`), so a reference to `Bool` always renders as
native `bool` — while the declaration `type Bool = True | False` emits a Rust `enum Bool`. Every
site returning a modeled `Bool` into a rendered `bool` position refuses.

The host bridge that would reconcile them is real but is targeted by hard-coded name:

```
fn repr_grounding_supplemental_bool_host_bridge_target(module_path: String, name: String) -> Bool {
  module_path == "std.types" && name == "Bool"
}
```

Two consequences worth separating. First, `src/v2/std/logic.dag` declares its own
`type Bool = True | False`, so there are two `Bool` authorities and only one of them is bridged —
a §3 fork, and the bridge's own witness pins `module_path: "std.logic"` returning `false` as
*expected* behaviour. Second, `impl From<Bool> for bool` does not coerce on its own; a bridge
only helps where the emitter inserts the conversion. Both facts sit above the repr choice, so
**the `Bool` half will not fully dissolve with a repr fix alone** — unlike the numeric half.

## 6. What is NOT claimed

- No fix is proposed here, and none has landed. §7 records an experiment in progress.
- The corpus-wide size of Root B is **unmeasured**. One module is measured.
- Whether `HostNative` is the correct terminal answer is **not established**. It is the v1 seed's
  representation; the terminal question is whether the modeled primitives should be grounded to
  their host realization at emit for *every* closure, which is a modeling decision above this
  lane's authority, not a switch to flip.
- `emit_host` was probed in the same batch; its result is withheld because the probe overlapped a
  rebuild of the instrument (see §7) and I cannot attribute it cleanly. It will be re-run.

## 7. In progress — the discriminating experiment

Forcing `rust_corpus_repr` to `HostNative` unconditionally, in the generated seed only, then
re-emitting the pure-v2 `dag/std/nat.dag` closure and re-running `cargo`. The prediction Root B
makes is specific: `pub type Nat = i64` and the four E0369 go to zero.

Per the shared surface's working agreements this is a **probe, reverted afterwards** — the
emitter's authority is `src/v1/05_emit_rust.dag` and `src/v1/04_infer.dag`, and a real fix lands
there and regenerates. It is also null-controlled: the check is what the emitted text *becomes*,
not merely that a count dropped, because `HostNative` also changes the text carrier
(`FreeMonoid<Char>` → `String`) and could trade one wrong output for another.

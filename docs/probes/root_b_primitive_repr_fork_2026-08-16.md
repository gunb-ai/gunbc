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

## 6. The discriminating experiment — executed, and it refutes the obvious fix

`rust_corpus_repr` was forced to `HostNative` unconditionally **in the generated seed only**
(`src/v1/stage0/src/v1_compiler_infer.rs`), `gunbc` rebuilt, the probes re-run, and the patch
reverted. Working agreement 4: the emitter's authority is `src/v1/04_infer.dag` and
`src/v1/05_emit_rust.dag`; this was a probe and no fix is proposed from it.

**Instrument control first.** The rebuilt binary emits `pub type Nat = i64;` for the pure-v2
closure. Had it still emitted the modeled carrier, the patch never reached the binary and every
number below would be void rather than negative.

### 6.1 The minimal closure goes green

`dag/std/nat.dag`, pure-v2 closure: **4 E0369 + 1 E0432 → 0 errors, 0 warnings, cargo green.**

### 6.2 On a real compiler module the numeric half is eliminated — and the wall gets worse

`src/v2/compiler/06_translate.dag`, same probe, before and after:

| | baseline | `HostNative` forced |
|---|---:|---:|
| diagnostics citing `CommutativeSemiring<Magnitude>` | 342 | **0** |
| `expected bool found Bool` | 11 | **11** |
| total coded errors | 652 | **773** |

Three things follow, and only the first is comfortable.

**The cause is confirmed.** 342 → 0 is not a marginal shift; the repr switch *is* the mechanism
behind Root B's numeric half, established by execution on a real module rather than by reading.

**The `Bool` half is untouched — now executed, not reasoned.** 11 → 11 confirms §5: `Bool` is a
checkpoint entry plus a hard-coded bridge target, both above the repr choice. Emitting
`src/v2/std/logic.dag` under the forced binary reproduces `expected bool found Bool` × 11 directly.

**Flipping the switch is not the fix.** The total *rose* by 121. This is working agreement 6
firing exactly as written: one wrong output became a different wrong output.

### 6.3 What the increase is made of — and why it is progress, not noise

The new errors are two disjoint families, both meaningful:

- **~76 E0308 that the modeled carrier had been masking.** `expected i64 found Rc<i64>` (39) and
  `expected Rc<Measure<_, _, Rc<i64>>> found Rc<Measure<(), (), i64>>` (37). The carrier grounded
  correctly, and immediately exposed that `Nat` is still in `shared_types` and so is still
  `Rc`-wrapped after becoming a `Copy` scalar. This is the same wrapping machinery §3 identifies,
  now visible because the carrier error no longer fires first — DESIGN §5's absorbing structure,
  whose deficit frequency was zero *by construction* until the mask came off.
- **~87 E0425/E0433 missing type names** (`NodeOccurrenceIdentity`, `NodeKind`, …). These are not
  mysterious. `v1.compiler.05_emit_rust` `reference_derived_use_lines_note` states the rule in
  tree: import-bearing modules run reference-derived use-line synthesis **only when
  `corpus_repr_is_faithful`**; HostNative import-bearing modules get `[]`, because running the
  walk on the seed "adds spurious/wrong use-lines and breaks zero-drift seed regen."

### 6.4 The actual finding

**`RustCorpusRepr` is two independent facts fused into one two-valued enum.** It decides

1. how modeled primitives are realized (`Nat`/`Int` → `i64`, text carrier → `String`), and
2. whether namespace-derived use-lines are synthesized for import-bearing modules,

and the two want *opposite* settings for a pure-v2 closure: it needs the host numeric grounding
(1) **and** the faithful-branch use-line synthesis (2). No value of a two-valued enum can supply
both, which is why the seed compiles, the v2 corpus refuses, and forcing either arm merely
relocates the refusals. That is a §5 state-space conflation, and it sits underneath Root B rather
than beside it.

## 7. What is NOT claimed

- **No fix is proposed and none has landed.** The experiment establishes a cause and refutes the
  obvious remedy; it does not select the terminal design. Splitting `RustCorpusRepr` into its two
  axes is the shape the evidence points at, but which authority owns the split, and whether the
  numeric grounding belongs in the checkpoint table at all, are modeling decisions above this
  lane.
- **The corpus-wide size of Root B is unmeasured.** One module is measured, twice.
- **The 121 increase is characterized, not fully attributed.** The two families above account for
  the bulk; I have not reconciled every individual diagnostic.
- **The third closure-shape branch is a candidate, not an instance.**
  `05_emit_rust` `type_leaf_is_unbound_in_closure_scope` returns `true` on `Absent` — the same
  "narrower closure silently takes the defaulting arm" shape — but I have not executed anything
  showing it takes the wrong arm. By contrast `04_env` `source_tree_of` is explicitly **not** an
  instance: its own note records the 2026-07-11 ruling that tree only labels a dissolution
  partition and no longer decides refusal.
- **`emit_host` was probed and its result is discarded.** That run overlapped a rebuild of the
  instrument, so its `emit_fail` verdict is unattributable — it contradicts the banked receipt
  that this module emits 621 diagnostics, and a contaminated run is not evidence against a clean
  one. It needs re-running on a stable binary.

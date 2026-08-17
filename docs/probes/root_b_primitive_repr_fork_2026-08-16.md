# Root B — primitive representation fork: live measurement and root cause (2026-08-16)

Session `eager-deer-389`, working the Root B lane of
the self-host cargo refusal root partition (plan doc not present in the tree; the link that stood here pointed at `docs/plans/self-host-cargo-refusal-root-partition.md`, which no commit has ever added).

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
| algebra-carrier errors / **distinct sites** | 84 / **74** | **0 / 0** |
| `Measure<…>` carrier errors / distinct sites | 11 / 9 | 37 / 37 |
| `Rc<i64>` errors / distinct sites | 0 / 0 | 125 / 125 |
| unresolved-name (E0425/E0433/E0422) sites | 19 | **110** |
| `expected bool found Bool` | 11 | **11** |
| total error blocks | 693 | **807** |

> **CORRECTION (2026-08-16, same day).** An earlier revision of this table reported "342
> diagnostics citing `CommutativeSemiring<Magnitude>`". That figure was a raw `grep -c` over
> matching *lines*, which counts rustc's annotation and note lines as well as the error itself —
> it overstated the population by roughly 4x. Re-counted per error block, and at distinct
> `file:line:col` site grain to match how the corpus census is denominated, the same run gives
> **84 errors / 74 sites → 0**. The direction and the conclusion are unchanged and the elimination
> is still total; the magnitude was wrong and is corrected here rather than left to be found by
> whoever tried to reconcile it against a site-grain census.

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

## 8. The discriminating control for identity-keying — executed, and RED

`smart-ram-730` conditioned any identity-keying work on a control that proves *disambiguation*
rather than merely that the algebra carrier goes to zero: two types with the **same authored name
from different declaring modules**, rendered in one closure, each getting its own realization.
Without it, a green B1 is equally consistent with "the flag stopped firing".

The fixture is three lines, using the `Bool` pair already identified in §5:

```
module probe.bool_identity_collision

fn takes_std_bool(b: std.types.Bool) -> std.types.Bool { b }
fn takes_v2_bool(b: v2.std.logic.Bool) -> v2.std.logic.Bool { b }
```

Emitted to Rust today (`--source-root dag --source-root src/v2`, clean binary), **verbatim**:

```rust
pub fn takes_std_bool(b: bool) -> bool {
    b
}

pub fn takes_v2_bool(b: bool) -> bool {
    b
}
```

**Both render as native `bool`.** Two distinct declarations, in two distinct modules, fully
qualified at the reference site — and the emitter produces byte-identical signatures. `emit`
returned 0 blocking errors, so this is not a refusal: it is silent, confident, wrong output.

This is the thesis reduced to its smallest executable form. It is not that realization is *missing*
information — the reference is written `v2.std.logic.Bool`, the qualification survives to the
`Node`, and `Node.inferred`'s `Resolved` node carries an `ident_span` naming the declaring module.
It is that `render_named_type_base` throws that away by keying `coerce_primitive_type` on
`authored_name_at`, which reads **the source text at the identifier span**. A token cannot
distinguish two declarations; the identity sitting on the same node can.

**As a control it is discriminating in both directions**, which is what makes it worth landing
ahead of any fix: it is RED now for a reason that has nothing to do with `corpus_repr` (the flag
does not gate `Bool` at all — §6.2 measured `expected bool found Bool` at 11 → 11 under the flip),
and it can only go green if realization actually consults declaration identity. "The flag stopped
firing" cannot turn it green.

**Scope caution, from the §10 instrumentation.** Identity-keying disambiguates two same-named
*types*. It does **not** obviously repair a *variant* name arriving where a type was expected —
28 of the 34 `type_leaf_is_unbound_in_closure_scope` misses are the variant name `Absent`, and
`gentle-dove-833`'s independent sentinel set contains it too. Two of my three miss names are in
their six-name sentinel set (32 of my 34 firings), so those populations are entangled and neither
is downstream of the checkpoint keying. The identity fix must not be sized as if it covers them.

## 9. A third specimen at the same seam — and the ceiling on the whole mechanism

### 9.1 `Hash`: a declared RHS silently replaced

Found by `smart-ibex-716`. `src/v2/std/node.dag` declares `type Hash = Fnv1a64Structural`. Emitted
at `v2_std_node.rs` from an `03_ingest` closure, tree `5e1a73fa33`, via
`gunbc compile --target rust` → `cssl_assemble` → `cargo check`, one signature carries both
spellings of the same type:

```rust
pub fn bag_hash_digest(empty: Hash, xs: Rc<Vec<v1_rt::Hash>>) -> Hash
```

(Route quoted deliberately: a signature with no route behind it is what this program keeps getting
burned by.)

**An objection was raised against this specimen and then withdrawn, and it is worth recording
because it is the first thing a reviewer will ask.** The concern was that this is a *seed-prelude*
substitution — a collision with a runtime type that need not be in the compiled corpus at all, and
therefore out of reach of declaration-identity keying. Traced independently by `smart-ibex-716` and
by me to the same place, it is neither. `dag/extdeps/languages/rust/types.dag` carries a checkpoint
row `{ dag_name: "Hash", target_type: "v1_rt::Hash" }`, and the `is_type_alias_item` arm of
`05_emit_rust` reaches `rust_scalar_checkpoint_render_base(dag_name: item_text)` — `item_text` being
**the alias's own authored name** — whose `Present` arm emits the row's target and never renders the
declared RHS:

```
match rust_scalar_checkpoint_render_base(dag_name: item_text, corpus_repr: …) {
  Present { value: host } => "pub type <item_text> = <host>;"        // declared RHS DISCARDED
  Absent                  => "pub type <item_text> = <rendered RHS>;"
}
```

So the collision is with a **row in the realization table**, not with the prelude. The original
phrasing named where the target type *lives* rather than what performed the substitution.

The "colliding declarant is not in the closure" objection is answered structurally: kernel
declarations are not nameless. `00_core` `kernel_span` mints the file `<kernel:Hash>` and `hash_type`
carries it as its `ident_span`; `04_env` `source_tree_of` already treats `<kernel:` as its own tree.
The kernel side has a stable identity for a row to name without being in the closure.

This is **stronger evidence than `Bool`**, and the reason is worth stating: `bool` is a plausible
realization of a type spelled `Bool`, so a reader can read that row as a design choice. Nothing
reads `type Hash = Fnv1a64Structural` becoming `String` as a choice.

### 9.2 The ceiling — stated as part of the claim, not left for a reviewer

`smart-ibex-716`'s corpus partition, at the figures current as of 2026-08-16:

| | sites |
|---|---:|
| directly attributable to name-keying | **253** |
| exposed-but-not-caused | 110 |
| **not** attributable (shape, ownership, required traits) | ~1,470 |
| **corpus denominator (fixed, M=11)** | **1,883** |

**Name-keying is ~13.4% of the wall (253 / 1,883). It is not the wall.** Four independent structural
confirmations (`Bool`, the `Nat`/algebra carrier, `Hash`, and the `Witness` rows below) do not
change that denominator, and this receipt does not claim otherwise.

**The basis, so the attribution is checkable rather than flattering** (`smart-ibex-716`'s own
statement of its limits): eleven entry modules; **one head, no before/after**;
`unreachable_patterns` counted as errors because the crate denies them; denominator is distinct
`(file, line, col, code, signature)` sites.

**The denominator rule this receipt is measured under (program rule, 2026-08-16).** Diagnostic
totals inflate with the *number of entries probed*, because every entry re-counts the same shared
floor:

| entries probed (M) | distinct sites | summed | inflation |
|---:|---:|---:|---:|
| 7 | 1,874 | 5,156 | 2.75× |
| 11 | 1,883 | 7,846 | 4.17× |

So "N diagnostics across M modules" is largely a statement about M, and it misleads in **both**
directions: a wall that shrank after a fix is not evidence of the fix if M fell, and one that grew
is not evidence of regression if M rose. The fixed denominator is the eleven-module census —
**1,883 distinct sites**, same route, same head, distinct `(file, line, col, code, signature)`
grain. The summed figure is never the denominator.

**How this receipt complies, stated so a reader can check rather than trust.** The §6.2 before/after
is **M=1** — one entry, `src/v2/compiler/06_translate.dag`, same route and same head on both sides,
differing only in the forced switch. The delta is therefore attributable; what it is *not* is a
corpus share, and the 74 is never divided by anything here. Any share claim uses 1,883.

**Why the partition can be planned against now.** Going from seven entries to eleven added 2,690
diagnostics and **nine** new distinct sites, and every root size held to within one (B1 509, A 142,
C 167, K 132, D 116, T7 105). The marginal entry returns roughly two new defects, so this is close
to the whole wall rather than a sample of it — no broader census is owed before planning.

**Why the figure is 253 and not 286 — the 33 are counted by `vivid-wren-870`, by agreement.**
They are the `missing generics for enum Witness` sites, and the reasoning for putting them there is
mine: re-keying the relation would have made the row match the *right* declaration **and still
dropped `<T>`**, so the **deletion** is what closes them, not the key. `vivid-wren-870` deletes the
rows in gunbc#8341 and counts them; this receipt strikes them. One home for the arithmetic, not
two. (Historical note, for anyone reconciling against an earlier revision: this figure read 286
briefly, on the reasoning below.)

**The reasoning that made them a candidate for this lane.** 33 of Root D's 116 sites — `missing generics for enum
Witness` — come from this same table, which carries `{ dag_name: "Witness", target_type: "Witness" }`
plus a second row spelled `"witness"`. **A row stating a bare target type is a row claiming arity
zero for a generic declaration**, which `dag/std/types.dag` `container_type_arity` contradicts
directly (`"Witness": 1`). The other 73 sites of Root D are *not* name-keyed — the emitted alias is
already applied with its parameter list dropped at the declaration while use sites still supply an
argument — and belong to `vivid-wren-870`.

Two consequences I hold myself to. The 110 exposed-but-not-caused sites are **not** mine to annex:
§6.3 measured them appearing when the mask came off, which makes them evidence of ordering defects
underneath the carrier, not members of this population. And the corpus denominator is
`smart-ibex-716`'s eleven-module census (1,883), not my 74 — which came from `06_translate` because that is
where I probed. Both are site-grain, so they **compose; they do not average**.

## 10. Why every root here survived review — the diagnostic instruction

Recorded because it is the most useful thing anyone said about this wall, and because my own two
specimens are exactly its shape (`smart-ram-730`, 2026-08-16):

**Each root is a correct local answer to a question nobody asked at that site.** The checkpoint row
is not wrong about the seed's `Hash`. The closure switch is not wrong about a seed corpus. The
variant sentinel correctly records an ambiguity. The derive trigger is not wrong about enum
declarations. **Not one is a bug read alone — the defect lives in the relation between two sites
that are each individually right.**

The instruction that follows: **a reader looking for a wrong line will not find one.** Ask instead
which question each site is answering, and whether anyone asked it *there*. That is precisely why
`type Hash = Fnv1a64Structural` losing its RHS took a trace rather than a read — the row is a true
statement about one declaration being applied to a different one, and both halves look correct in
isolation.

## 11. Inherited from the Root D lane — a control, a trap, and a witness that pins a defect

`vivid-wren-870` landed the row deletion (gunbc#8341) and handed over three things.

**The null control this lane was missing, to be enrolled against any re-keying change.**
`dag/test/claim/root_d_checkpoint_scalar_declared_arity_witness_test.dag`
`w_arity_zero_checkpoint_scalar_still_strips_phantom_arguments`. It compiles a fixture importing
`std.integer { Int8 }` and asserts the emitted `std_integer.rs` **contains** `Compose<i64,` and
**does not contain** `i64<`. Both halves matter here: the first proves a genuine arity-0 row still
matches and still renders after re-keying, the second proves the phantom argument is still stripped
rather than surfacing as `i64<T>`. My own controls prove same-named types stop colliding; none of
them proves a legitimate scalar row still fires, which is exactly the way identity-keying could
regress silently.

Its stated limitation, carried rather than dropped: it is an **emit-text** claim, so if re-keying
changes the emitted spelling of `Int8` for an unrelated reason it reds on the text and not on the
property. Read the diff, not the boolean.

**On the one-line guard `vivid-wren-870` added** (`if is_container_type(leaf) then none`, before
the existing lookup): if re-keying lands, it becomes **redundant rather than wrong** — a guard
sourced from `container_type_arity` is a second, independent statement of the arity fact, not a
duplicate of the key. Their correction to my framing, which is worth keeping: gunbc#8341 removes a
wrong row and makes the wrong classification unreachable *from the arity authority we already
have*, but **neither the deletion nor the guard makes the underlying fact structural** — arity is
still asserted by table membership in two places rather than derived from the declaration once.
That derivation is this lane's terminal shape, not theirs. The guard stops being needed only then,
and it will not be removed silently as part of any re-keying change.

**A harness trap worth an hour if hit cold.** `compile_dag_rust_emit_check` compiles a *virtual
single-module* source through the witness-root index, and in that mode importing `v2.std.witness` —
or `v2.std.node` beneath it — refuses with hard diagnostics, so the check returns `false`
**indistinguishably from a failed assertion**. `v2.std.optional` imports fine, so it is not a
blanket `src/v2` limitation; which diagnostic was not chased. This lane will want fixtures over real
`std` types, so it will meet this directly.

That trap has a consequence for the Root D fixture that is **this lane's obligation**: its positive
fixture declares its own two-arm `Witness<C>` rather than importing the authority, which is
defensible only *because the table is spelling-keyed and the spelling is the load-bearing input*.
Re-keying makes that false, at which point the fixture must import the real declaration. The
dissolve-on is written into that witness note pointing here.

**A witness that currently pins a defect as expected behaviour.**
`dag/test/claim/root4_measure_missing_generics_witness_test.dag`
`w_zero_param_alias_list_param_unchanged` asserts the emitted text
`ClosedCarrierAlias<ProbeQuantity, ProbeScale, ProbeMagnitude>` for a **zero-parameter** alias —
the authored alias name carrying the resolved definition's arguments. Whoever fixes that class goes
red there, and should read it as **the witness pinning the defect**, not as a regression.

**Ownership resolved as: nobody, and that is the accurate answer rather than a dodge** (settled with
`vivid-wren-870`, 2026-08-17). It is not this lane's — nothing in it turns on two declarations
sharing a spelling. It is not theirs either: they scoped **half 1** out of gunbc#8341 deliberately
and did not diagnose it past the receipt, knowing that the renderer composes an authored leaf name
with a resolved node's children but **not** why the closure-shape branch gating the strip fails to
fire, having never executed the control that varies the flag. Their "probably yours" was proximity
to the checkpoint machinery, not mechanism, and is withdrawn.

**So half 1 (the ~73 applied-alias sites) currently has no owner, and this witness belongs to half
1 rather than to a lane.** Left unresolved deliberately: the drift being guarded against is real,
and "whichever lane touches it first" is exactly how a witness that pins a defect as expected
behaviour acquires a defender.

> **Warning owed to whoever picks up half 1, before they start:** an enrolled witness currently
> asserts the broken shape. The first thing a correct fix does is turn `w_zero_param_alias_list_param_unchanged`
> **red**, and it will look like a regression they caused. It is not — it is the witness pinning
> the defect.

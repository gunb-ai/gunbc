# v1_rt Witness/Diagnostic decouple — design note (Gate-1 root, 879 E0308 / 53%)

**Status:** LANDED (#7211, 2026-07-25). Stage 2+3 shipped atomically: native
`v1_rt::Witness<V>` deleted, `lookup` → `Option<V>`, `v2.std.optional` hoist +
`witness_from_optional` bridge, `Map.lookup` → `Optional<V>`, `05_emit_rust` emit
routing de-aliases modeled `Witness` off `v1_rt`, seed regen. **Remaining:** Stage 4
— Gate-1 E0308 burn-down probe receipt (§3; measurement, not structural work).

Reasoned serially per DESIGN.md's preamble: §1 fixes the blocker from receipts, §2 is vivid-
raven-588's domain-wrap half, §3 is the combined staged plan, §4 is scope fence + flags.
Owners: §1 tidy-boar-444, §2 vivid-raven-588 (co-designed, converged in this file).

---

## 0. The blocker (why the naive fix fails)

`src/v1/stage0/src/v1_rt.rs` and `src/v1/runtime_rust.dag`'s `rt_collection_ops()` (the string
template that *generates* `v1_rt.rs`) both declare a **native** `Witness<V>`:

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Witness<V> {
    Holds { value: V },
    Violates { diagnostic: String },   // <- String, not the modeled Diagnostic
}
pub fn lookup<V: Clone>(table: &HashMap<String, V>, key: String) -> Witness<V> { ... }
```

— a second, unmodeled `Witness` fork that (a) already diverges from the single modeled authority
`v2.std.witness.Witness<C> = Holds{value:C} | Violates{diagnostic: Diagnostic}`
(`src/v2/std/witness.dag:6-8`) on the `Violates` payload type (bare `String` vs. modeled
`Diagnostic`), and (b) is the direct mechanism behind Gate-1's two largest E0308 roots per the
2026-07-24 diagnosis (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`): WITNESS
(`Witness<Rc<X>>` expected, `Witness<_>` found — the generated code can't propagate a concrete
type argument through a Rust-native generic it doesn't fully control) and, pre-#7141's Diagnostics
fix, the DIAGNOSTICS carrier fork. Combined the two are **879 E0308 across the canonical seven
deep modules — 53% of the remaining Gate-1 wall.**

The tempting fix — point `v1_rt::Witness`'s `Violates.diagnostic` at the real modeled `Diagnostic`
— **fails and is rejected**:

- The bootstrap crate (`src/v1/stage0`, compiled standalone to produce the `v1-compiler` binary
  that self-hosts everything else) has **no real `v2_std_diagnostic`** — only a deliberately
  isolated toy stand-in, explicitly marked independent of the self-emitted artifact under test.
  This isolation exists to break a bootstrap circularity: the bootstrap cannot depend on the very
  `Diagnostic` model it self-emits and behaviorally tests. Pointing `v1_rt` at the real
  `Diagnostic` fails to resolve in the bootstrap build (E0433) and risks reintroducing exactly the
  circularity that isolation was built to prevent.
- Structurally it also violates the bootstrap isolation named above: `v1_rt` is host-boundary
  glue (raw `HashMap` lookup) compiled into the hand-seed bootstrap crate, while `Diagnostic`/
  `Witness` are modeled domain carriers the bootstrap deliberately does not depend on (§0). Wiring
  the primitive at `Witness`/`Diagnostic` reintroduces that upward dependency — the circularity §0
  was built to break.

## 1. The primitive-return-shape half (§1, tidy-boar-444)

### 1.1 The fix: `v1_rt::lookup` returns a primitive, not a carrier

`v1_rt::lookup` stops returning `Witness<V>` and returns a bare **`Option<V>`** — the host
primitive it actually is (a `HashMap::get` miss is a primitive fact, not yet a diagnosed one).
Concretely, in `rt_collection_ops()` (`src/v1/runtime_rust.dag:145-156`, the template that emits
`v1_rt.rs` verbatim into bootstrap *and* every target crate):

```rust
pub fn lookup<V: Clone>(table: &HashMap<String, V>, key: String) -> Option<V> {
    table.get(&key).cloned()
}
```

The native `Witness<V>` enum (`runtime_rust.dag:147-151`) is **deleted entirely** — not kept
alongside as a second carrier (that would just re-fork it under a different label; DESIGN §5's
"a scaffold with no dissolution trigger is a workaround," and here there is no reason to keep it
at all once nothing constructs it).

This is a primitive-shape change, not a new concept: `Option<V>` is exactly what `HashMap::get`
already returns before `.cloned()`; today's code manufactures a `Witness` (and a synthesized,
un-located `"lookup miss for key {}"` string diagnostic — itself a fabricated-plausible-output
smell, DESIGN §5) purely to answer "was it there," which `Option` already answers for free.

### 1.2 The compiler-front-end hook that must move with it

`lookup` is a Tier-2b compiler-recognized builtin, not user code — its return type is
**hardcoded** in the v1 compiler's own inference (`src/v1/04_infer.dag:988-1005`):

```
if func_name == "lookup" {
  witness_of_element(element: value_type)     -- <- becomes optional_of_element / equivalent
} else {                                       -- (map_get already does this: with_optional_cardinality)
  with_optional_cardinality(n: value_type)
}
```

and carries a distinct `LookupCallSemantics` tag (`04_infer.dag:2140-2143`) separate from
`map_get`'s `PlainCallSemantics`. Post-decouple, `lookup`'s inferred return type is
`witness_of_element` → an `Optional<V>`/`Option<V>`-shaped inference identical in kind to what
`map_get` already does (`with_optional_cardinality`) — **`lookup` and `map_get` converge onto the
same inferred-return-shape family**, which is itself a small §2/§3 dissolve (two builtins that
already meant almost the same thing stop needing two separate arms). Whether they fully merge to
one `LookupCallSemantics`/`PlainCallSemantics` tag or keep distinct tags for call-site diagnostics
is left to implementation, not decided here — it doesn't change any byte the fixed-point gate
checks.

### 1.3 Call-site wrap propagation — actual footprint, not the naive grep count

A raw `grep -rl v1_rt::Witness src/v1/stage0/src` returns 97 files / 227 references, but nearly
all of that is either (a) unused-import boilerplate carried by every module the emitter touches
(the generated `use crate::v1_rt::Witness;` prelude line, unused in ~70 of the 97 files — dies for
free once `05_emit_rust.dag` stops emitting it, §2.C) or (b) an unrelated same-named domain type
(`std_realization_schedule.rs`'s `WitnessKind`/`WitnessSpan`/`ScheduleWitnessEntry` — a modeled
execution-witness-span concept, not `v1_rt::Witness`, a naming coincidence not a shared carrier).

The **real** construction footprint — files that actually pattern-match or construct
`Holds{..}`/`Violates{..}` from a `v1_rt::lookup`/`v1_rt::Witness` value — is 18 `Holds{` sites +
15 `Violates{` sites across ~13 files (`extdeps_languages_rust_types.rs`, `std_algebra.rs`,
`std_types.rs`, `v1_compiler_compiler_tests_rust.rs`, `v1_compiler_complexity.rs`,
`v1_compiler_emit.rs`, `v1_compiler_emit_rust.rs`, `v1_compiler_infer_lookup.rs`,
`v1_compiler_infer_method.rs`, `v1_compiler_infer_patterns.rs`, `v1_compiler_infer.rs`,
`v1_compiler_infer_types.rs`, `v1_compiler_languages.rs`, `v1_compiler_parse.rs`,
`v1_compiler_runtime_rust.rs`, `v1_compiler_tokenize.rs`, `v1_std_core.rs`, plus `cli_run.rs`/
`compiler_tests.rs`/`pre_push.rs`/`v1_interpreter.rs` which construct `Witness` values of their
own domain types unrelated to `lookup`, not `v1_rt::Witness` specifically — those are unaffected).
This is the tractable footprint the migration actually needs to touch, each site becoming either
a direct `match table.get(...) { Some(v) => .., None => .. }`/`if let` over the new `Option<V>`,
or — where the site's *own* return type is itself the modeled `Witness<C>` — a call into §2.B's
`witness_from_optional` wrap helper at that boundary.

Because these are all **generated, frozen (`SeedRetained`) files**, not hand-authored — every one
is re-emitted from its `.dag` source by the *same* `05_emit_rust.dag` template machinery §2.C
changes — the actual migration work is: fix the emit template once (§2.C's routing change +
§1.1/§1.2's primitive-shape change upstream of it), then **regenerate** these ~13-17 files, not
hand-edit each one. Hand-editing would itself violate DESIGN §7 (the seed is a realization,
re-derived from `.dag`, not a hand-maintained parallel copy) — flagged explicitly so no
implementation PR is tempted to patch the `.rs` files directly.

### 1.4 Regen fixed-point / dual-compile implications

This is the crux of why the note is required before any edit: `v1_rt.rs` is compiled twice from
one template (`rt_collection_ops`) — once as `src/v1/stage0/src/v1_rt.rs` (committed, part of the
`v1-compiler` bootstrap binary) and once synthesized into every freshly emitted target crate
(§0). `regen_stage0`'s fixed-point gate (DESIGN.md's `regen_floor_skip_witness` / self-host
frontier machinery) requires these **byte-identical**. Because both are the *same template
function*, changing `rt_collection_ops()` once changes both emission sites atomically — the
byte-identity is preserved by construction, not by a second hand-sync step. The sequencing that
must hold for the gate to stay green through the change:

1. `04_infer.dag`'s `lookup` return-type inference (§1.2) and `runtime_rust.dag`'s
   `rt_collection_ops()` template (§1.1) change **together, in one PR** — an inference/template
   split-brain (old inferred `Witness<V>` type against a new `Option<V>`-returning primitive, or
   vice versa) would desync mid-migration and produce exactly the E0308 class this whole effort
   removes.
2. Every call site in `src/v1/stage0/src/*.rs` that constructs/matches `v1_rt::Witness` (§1.3's
   ~13-17 file footprint) is itself emitted by the same infra, so it is **regenerated**, not
   patched, as part of the same PR — the committed bootstrap crate and freshly emitted crates stay
   the same artifact family throughout, never a hand-diverged one.
3. Post-change, `regen_stage0` is run for real (byte-fold verification, not just `cargo check` —
   DESIGN §5's "compile-green ≠ executed" applies directly here) to confirm the fixed point holds
   with the new shape before the PR is considered done.
4. The bootstrap-crate/`Diagnostic` isolation named in §0 is **not touched** by this design — `lookup`
   returning `Option<V>` needs no `Diagnostic` in the bootstrap at all (that's precisely the point:
   the primitive stays diagnostic-free), so the circularity §0 named stays exactly as isolated as
   it is today. This design removes the reason anyone would have reached for the real `Diagnostic`
   from `v1_rt` in the first place, rather than finding a way around the isolation.

## 2. The domain-wrap + emit-routing half (§2, vivid-raven-588)

### 2.A Single modeled authority

`src/v2/std/witness.dag`'s `Witness<C> = Holds{value:C} | Violates{diagnostic: Diagnostic}`
becomes the *only* `Witness` in the corpus once §1 deletes the native one. Emitted Rust construct
sites use `Violates{diagnostic: Rc<Diagnostic>}` (the ordinary ownership-wrap for a modeled
product, per the existing `wrap_decision_predicate` family — no special-casing).

### 2.B Boundary wrap helper — generalizes the existing `collection.dag` precedent

The precedent already lives in `src/v2/std/collection.dag`: `optional_present_witness`
(`collection.dag:42-47`) and `list_nth`'s caller-supplied `absent: Diagnostic` parameter
(`collection.dag:97-114`) both already do "take a primitive absence, wrap it into modeled
`Witness<T>` with a constructed `Diagnostic` at the call boundary." This design generalizes that
shape into one single-authority helper (exact module home — `v2.std.collection` beside its
siblings, or `v2.std.witness` beside the type it constructs — is an implementation-time pick, not
load-bearing to this note):

```
fn witness_from_optional<T>(opt: Optional<T>, absent: Diagnostic) -> Witness<T> {
  match opt {
    Present { value: v } => Holds { value: v }
    Absent => Violates { diagnostic: absent }
  }
}
```

Every `v1_rt::lookup` call site whose *own* return type is the modeled `Witness<C>` (i.e. was
relying on `v1_rt::lookup` to hand it a `Witness` directly) now calls `v1_rt::lookup(...)` for the
raw `Option<V>`, then `witness_from_optional(opt: .., absent: <located Diagnostic>)` to produce the
modeled carrier — with the `Diagnostic` constructed at that call site (`reason`, `at: Locus`,
`correction: Unavailable{reason: ExternalContractUnknown}`), never fabricated generically the way
today's `format!("lookup miss for key {}", key)` string is. This mirrors `map_absent_diagnostic`/
`optional_absent_unwrap_diagnostic` (`collection.dag:34-40,62-68`) exactly — no new diagnostic
shape is invented.

### 2.C Emit routing — stop aliasing modeled `Witness` onto `v1_rt::Witness`

`05_emit_rust.dag` currently hard-routes every modeled `Witness` reference through the native
runtime type at three seams:

- `rust_normalize_witness_type_text` (`05_emit_rust.dag:492-494`): `replace(rendered, "witness<",
  "v1_rt::Witness<")` — every modeled `Witness<C>` type render becomes `v1_rt::Witness<C>`.
- The import-prelude synthesis (`05_emit_rust.dag:4236-4252`): unconditionally injects
  `use crate::v1_rt::Witness;` / `use crate::v1_rt::Witness::{Holds, Violates};` into any module
  whose type surface mentions `Witness`.
- The variant-path renderer (`05_emit_rust.dag:5591`): `if parent_leaf == "Witness" { concat(
  "v1_rt::Witness::", rust_name) }` — `Holds`/`Violates` construction sites render as
  `v1_rt::Witness::Holds`/`v1_rt::Witness::Violates`.

Post-decouple, all three drop the `v1_rt::` prefix and resolve to the modeled type's own emitted
module path (the ordinary `v2_std_witness::Witness<C>` a ctor turbofish already targets once
vivid-raven-588's parallel lane, #7195, lands modeled turbofish carrier resolution — see §4's
sequencing). This is a pure deletion at these three seams, not a new rendering rule: once nothing
constructs the native type (§1), nothing needs to alias to it.

### 2.D Net effect on the fork

Today the corpus carries genuinely two `Witness` carriers under one name (native `v1_rt::Witness`,
modeled `v2.std.witness.Witness`) — the textbook §3 nickname-fork DESIGN.md calls out ("a second
name for one concept... duplicates work at the meaning layer and everything derived from it").
§1+§2 together dissolve it to one: the WITNESS bucket (Gate-1's `Witness<Rc<X>>`-vs-`Witness<_>`
mismatch) disappears because there is only one `Witness` left to instantiate, and the DIAGNOSTICS
bucket's remaining `v1_rt::Witness.Violates{diagnostic: String}` vs. modeled
`Violates{diagnostic: Diagnostic}` split disappears with it — both roots collapse together, which
is why they were sized and staffed as one 879-count bucket rather than two.

## 3. Combined staged plan

**Execution (2026-07-25):** Stages 0–3 LANDED in #7211. Stage 4 remains open.

- **Stage 0 — this design note.** No `v1_rt.rs`/`runtime_rust.dag`/`04_infer.dag` edit. Sent to
  sharp-bee-290 for sign-off (load-bearing — bootstrap + regen fixed-point).
- **Stage 1 (parallel, already in flight, non-blocking on this note) — #7195**, vivid-raven-588:
  modeled turbofish carrier resolution at modeled `Witness` construction sites. Explicitly scoped
  to **not** touch `v1_rt` or emit routing (§2.C) — lands independently either before or after
  sign-off.
- **Stage 2 (one PR, requires sign-off) — the primitive+template change (§1.1, §1.2)**: delete
  native `Witness<V>` from `rt_collection_ops()`/`v1_rt.rs`, change `lookup`'s primitive return to
  `Option<V>`, move `04_infer.dag`'s `lookup` inference off `witness_of_element` onto the
  `Optional`/`Option` family alongside `map_get`.
- **Stage 3 (same PR as Stage 2, or immediately after) — domain wrap + emit routing (§2.B, §2.C)**:
  land `witness_from_optional`, delete the three `v1_rt::Witness` aliasing seams in
  `05_emit_rust.dag`, regenerate the ~13-17 real-usage `src/v1/stage0/src/*.rs` files (§1.3) plus
  the unused-import boilerplate across the rest of the 97-file surface.
- **Stage 4 — verify.** Real `regen_stage0` byte-fold (§1.4 point 3) confirming bootstrap ==
  freshly-emitted fixed point under the new shape; re-run the canonical-seven Gate-1 probe
  (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`'s method) to measure the
  879-count burn-down as the discriminating witness for this whole effort — a byte flipped in
  either the new `Option<V>` primitive or the `witness_from_optional` wrap should reintroduce a
  concrete, locatable E0308/E0433 (not silently pass), which is itself evidence the change is
  construction, not validation theater, per DESIGN §5.

Stage 2+3 are the load-bearing edit this note gates; Stage 1 (#7195) and Stage 0 (this note) carry
no such gate and are unblocked today.

## 4. Non-goals / scope fence

- **Not** touching the bootstrap/`Diagnostic` isolation named in §0 — this design's whole point is
  that `v1_rt` no longer needs `Diagnostic` at all, so the isolation is left exactly as-is.
- **Not** re-litigating `Witness<C>`'s modeled shape (`Holds`/`Violates`) — unchanged, only its
  native shadow is removed.
- **Not** a general Rc-ownership/wrap-decision redesign — `Violates{diagnostic: Rc<Diagnostic>}`
  uses the existing `wrap_decision_predicate` machinery as-is (including its known
  `Instantiation`-kind coverage gap tracked separately, `docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`'s
  Root 3/#6776 follow-up) — not re-scoped or re-fixed here.
- **Not** merging `lookup`/`map_get`'s call-semantics tags outright (§1.2) — left as an
  implementation-time judgment call, non-load-bearing either way.

## 5. Flags for sharp-bee-290 sign-off

- **FLAG A — primitive shape.** Confirm `v1_rt::lookup: Option<V>` (§1.1), native `Witness<V>`
  deleted outright (not kept as a second, unused carrier).
- **FLAG B — wrap-helper home.** `witness_from_optional`'s module home (§2.B) — `v2.std.collection`
  vs. `v2.std.witness` — either is fine; pick one so it isn't re-decided per call site.
- **FLAG C — Stage 1 (#7195) sequencing.** Confirm #7195 stays independent and doesn't need to land
  before/after Stage 2/3 in any strict order — this note assumes either order works since #7195
  scopes itself to modeled-ctor-site turbofish resolution only, not `v1_rt`/emit-routing.
- **FLAG D — Stage 2/3 PR granularity.** Whether the primitive/template change (§1.1-1.2) and the
  domain-wrap/emit-routing change (§2.B-2.C) land as one PR (this note's default, since both must
  move together for the fixed-point gate to stay green at every commit) or as two tightly
  sequenced PRs with an interim state accepted to be red on `regen_stage0` between them (not
  recommended — named as the alternative for completeness).

---

Related: [Gate-1 E0308 root classification, 2026-07-24](../probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md)
(names WITNESS Root 2 and the (addendum) landed DIAGNOSTICS fix this note's §2.D generalizes) ·
[Gate-1 diagnosis, 2026-07-23](../probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-23.md) (the
TEXT-carrier fix this note's §2.A/§2.D follow the same construction-grounding shape as).

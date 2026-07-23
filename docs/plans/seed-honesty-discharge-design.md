# Seed-honesty discharge — a real second compiler and a computed fixed-point digest

**Status:** design-first (model-before-implement). This document is the deliverable for
roadmap `5-seed-honesty`. It fixes the gap, names a concrete second-compiler candidate, and lays
out a staged plan. No load-bearing `bootstrap.dag` edit lands in the design PR; each stage below is
a separate, separately-signed implementation PR — `bootstrap.dag` is DESIGN.md-load-bearing (§7),
so every stage past this one carries a higher review bar.

Reasoned serially, per DESIGN.md's preamble: §1 fixes the gap from receipts; each later section is
a consequence, not a restatement.

---

## 1. The gap (from receipts, not assertion)

DESIGN.md §7 states the end-state directly: v2 "self-emits to a bit-identical fixed point," and
the honest way to retire the hand-written seed is "a pinned, content-addressed, reproducible-from-
`.dag`, v2-emitted bootstrap binary" — the rustc/GCC-style trust chain `SeedHonestyDischarge`/DDC
(Diverse Double-Compiling, the Thompson trusting-trust defense) exists to certify.
`src/v2/workflow/bootstrap.dag` models this machinery today, and it is well-shaped — `Witness<T>`
throughout, fail-closed `Violates` arms, a real `IndependentCompilerPair`/`DiverseCompilationAgreement`/
`SeedHonestyDischarge` type chain. But two of its load-bearing facts are not *facts* yet:

1. **`pinned_v4_fixed_point_hash` is self-referential, not computed.**
   ```
   data pinned_v4_fixed_point_hash: Hash = pinned_v4_fixed_point_hash   -- bootstrap.dag:30
   ```
   A `Hash` whose value is literally its own declaring atom. Nothing hashes anything. Every place
   that "checks" this digest (`bootstrap_hash_pin_projection_node`, `bootstrap_fixpt_witness`,
   `diverse_compilation_agreement_witness`) compares this atom against itself or against the other
   fixture atoms (`v2_stage0_hash`, `v2_stage1_hash`, `v2_stage2_hash`, `bootstrap.dag:27-29` — same
   pattern, same problem) via `==`. The comparison is real Node/Witness machinery; the *operands* are
   props, not digests.

2. **`independent_compiler_witness` is an opaque, unbacked `Symbol` — no second compiler exists.**
   ```
   fn independent_compiler_pair_witness(primary: Symbol, secondary: Symbol, evidence: Symbol) -> Witness<IndependentCompilerPair> {
     if (primary != secondary) && (evidence == ^independent_compiler_witness) { Holds { ... } }
     ...                                                                                  -- bootstrap.dag:558-568
   }
   ```
   and at the call site (`bootstrap.dag:709-713`):
   ```
   independent_compilers: independent_compiler_pair_witness(
     primary: ^v1_pipeline,
     secondary: ^independent_compiler_witness,
     evidence: ^independent_compiler_witness
   )
   ```
   `evidence` is checked against the same atom that names the (nonexistent) secondary compiler. The
   "evidence two independent compilers agreed" is satisfied by `^independent_compiler_witness ==
   ^independent_compiler_witness` — trivially true by construction, every time, regardless of
   whether any compilation ever ran. This is exactly DESIGN.md §5's named failure shape: *"a check
   that can be satisfied by editing the declaration while the realizer still lies."* There is no
   realizer here at all — no second compiler binary, no second compilation run, no comparison of
   real bytes.

Both gaps are honestly named already: roadmap `5-seed-honesty` — "modeled in `bootstrap.dag`
(`SeedHonestyDischarge`), no execution exists; needs a reproducible artifact to double-compile" —
and `bootstrap.dag`'s own scaffold disposition markers point at `content_hash` as the eventual
authority (see `v2_self_hosting.dag`: "placeholder hashes … dissolve-on T-15/T-20 `content_hash`
supplying real per-stage merkle digests"). This document is the concrete dissolution plan for
both, plus the one thing that has never had a proposal: **what the second compiler actually is.**

## 2. The digest half is *not* a new invention — it already exists elsewhere in the corpus

Before proposing anything new, apply DESIGN.md §2's DFS-the-concept-DAG discipline: does a real,
grounded digest of emitted source already exist? Yes — `src/v2/compiler/self_host.dag`, built for
the adjacent `5-real-fixpoint` milestone:

```
fn source_text_code_unit_digest(text: String) -> Hash { fold_list(xs: text, ... combine_hash ...) }  -- self_host.dag:96-104
fn canonical_emitted_bytes_digest(source: Medium<String>) -> Hash {
  combine_hash(a: symbol_identity_digest(sym: ^canonical_emitted_bytes), b: source_text_code_unit_digest(text: source.carried))
}                                                                                                      -- self_host.dag:106-111
```

This is a real, host-grounded, content-addressed digest over actual emitted bytes — not a
placeholder. `self_host_realized_comparison_gate.dag` (the #6009 interim gate) already drives it
end-to-end through a real shell execution (`ensure_gunbc_built` → `gunbc run --claim-run` →
byte-fold over `lib.rs`/`v1_rt.rs`), scoped today to a 2-file roster pending the full 92-file
`GENERATED_STAGE0_FILES` widen tracked by `5-real-fixpoint`.

**So `pinned_v4_fixed_point_hash` should not be a new fixture — it should *be*
`canonical_emitted_bytes_digest` applied to the real stage1 (or agreed stage1==stage2) emitted
source.** Minting a second digest mechanism here would be exactly the §2 "net concepts must not
grow by re-invention" failure. The design in §5 below reuses this machinery unchanged.

## 3. The second-compiler half — thesis and candidate

### 3.1 Why the obvious answers don't satisfy DDC

Diverse Double-Compiling defeats a Thompson-style trapdoor (a compiler that recognizes it is
compiling *itself* and reinserts a backdoor) by compiling the compiler's own source with a
**second, independently-implemented** compiler and checking agreement. Two candidates that look
free are not independent in the sense DDC needs:

- **v1_pipeline compiling itself again, or comparing stage1 vs stage2.** This is what
  `5-real-fixpoint` already proves (`self_host_fixpoint_witness`, `self_host.dag:142-158`) —
  useful and necessary, but it is v2 compiling v2 (a self-consistency check), or v1 compiling v2
  once (already modeled as `primary`). Neither introduces a compiler that doesn't already sit on
  the trust chain being audited. `bootstrap.dag`'s own model already requires `primary != secondary`
  (`bootstrap.dag:563`) — v1_pipeline is already spent as `primary`.
- **The v2-emitted TypeScript realization (Track T) as "secondary."** Tempting because it's a
  different *target* language, but it shares the *same* frontend/translate stage
  (`06_translate.dag`'s one `fold_node` catamorphism, DESIGN §4) as the Rust realization. A
  trapdoor planted in `05_emit`/`06_translate` would poison both targets identically — TS-vs-Rust
  output agreement proves emitter-target coverage, not compiler independence. This is a real,
  valuable receipt (Track T), just not a DDC witness.

Genuine independence requires an implementation that shares no code, and ideally no author-blind-
spot, with either `src/v1` (the hand-written Rust seed) or `src/v2` (the self-hosted `.dag`
pipeline, including all its emit targets).

### 3.2 Why a full independent `.dag` compiler is the wrong scope

Building a second production-grade compiler for the whole `.dag` language would itself violate §2
(duplicate ~154k LOC of frontend/backend work for a **trust-verification tool**, not a shipped
capability) and would never finish. But DDC does not need a production compiler — it needs a
compiler that can reproduce **exactly the closure `bootstrap.dag` already declares as the frozen
seed capability**, which is deliberately narrow:

```
fn bootstrap_seed_capability_inventory_root() -> Node { ... }   -- 4 constructs, bootstrap.dag:180-203
```
— `bootstrap_construct_source_language_subset`, `bootstrap_construct_target_model_subset`,
`bootstrap_construct_runtime_model_subset`, `bootstrap_construct_lowering_subset`. That subset is
exactly what a seed compiler needs to exist at all (parse the frozen source-language subset, emit
against the frozen target-model subset, run under the frozen runtime-model subset). It is already
scoped for minimality by the model that exists.

### 3.3 The candidate — `ddc_reference_compiler`

Propose a **new, minimal, independently-written reference compiler**, scoped *only* to
`seed_capability.constructs` (the same 4-construct closure, never more), whose job is single-
purpose: compile the bootstrap source snapshot
(`bootstrap_source_snapshot_root`/`BootstrapSourceSnapshot`) end to end, once, for DDC comparison.

- **Location:** `tools/ddc_reference_compiler/` — outside `src/v1`, `src/v2`, and `dag/`. It is
  trust-verification tooling, not a compiler stage or a std/extdeps model, so it sits beside the
  repo's other `tools/` (per DESIGN.md's layer discipline: its home is what it *is*, not a
  convenient path). It must not import anything from `src/v1` or `src/v2` — that is the whole
  point; a shared dependency is a shared blind spot.
- **Language:** deliberately **not** Rust and **not** one of v2's existing emit targets (Rust,
  TypeScript, Python, Go, …) that share test/emitter provenance with this repo's own corpus —
  pick a toolchain used nowhere else in the compiler stack (e.g. a from-scratch recursive-descent
  reader in a language chosen for being boring and auditable, not for reuse). The exact language
  choice is an open flag (§7) — it is a genuinely free choice as long as it shares no code with
  `src/v1`/`src/v2`.
- **Provenance discipline:** written directly from DESIGN.md §4's closed vocabulary (`Node`+`Edge`,
  6 connectives, 5 behaviors) and the frozen construct subset's grammar — **not** by reading
  `src/v1`'s parser/emitter source or `src/v2`'s `.dag` pipeline stages. This is the DDC assumption
  made explicit: if the reference compiler's author consulted the compiler-under-test's source
  while writing it, a shared bug (or shared backdoor) becomes possible and the independence claim
  is void. This must be a documented constraint on the PR that writes it, checkable by "did this
  PR's diff touch or import `src/v1`/`src/v2`" (mechanical, not just an honor rule).
- **Scope, explicitly bounded:** parse+emit+run exactly the frozen `SeedCapability` subset against
  `rust_target_model`'s bootstrap-relevant slice — not the general `.dag` language, not the
  full target-model bundle. LOC budget: low thousands, not tens of thousands — small enough that a
  human reviewer can read the whole thing in one sitting (that *is* the trust property DDC needs;
  a reference compiler too large to audit defeats its own purpose).

### 3.4 What agreement means

`bootstrap.dag`'s existing witness shape already decides this, and this design does not propose
changing it (§4 is explicit that the `Witness`/fail-closed shape is out of scope):
`diverse_compilation_agreement_witness` requires `primary.output_hash.digest ==
secondary.output_hash.digest` (`bootstrap.dag:581-583`) — **byte-identical digests**, full stop.
So agreement here is **byte-identical emitted source for the frozen subset, plus identical corpus
verdicts** — not a looser "behavioral" standard. Concretely: `v1_pipeline` and
`ddc_reference_compiler` each independently compile `bootstrap_source_snapshot_root`'s underlying
source, both targeting the same frozen backend representation; both binaries are then run against
`bootstrap_manual_corpus_harness`'s fixed test entry (`self_host.dag`'s corpus runner is already
wired for this shape); agreement is **identical corpus verdicts** (pass/fail per witness) **and**
identical `canonical_emitted_bytes_digest` over each compiler's emitted source for the frozen
subset.

This is a real constraint on `ddc_reference_compiler`'s scope (§3.3): it is not free to make
independent codegen choices for the frozen subset — it must emit source that is byte-identical to
`v1_pipeline`'s output for that closure, which is achievable only because the frozen subset is
small and its target representation is fixed (`rust_target_model`'s bootstrap-relevant slice),
not because codegen freedom is being waived. If a future stage wants to allow independently-chosen
codegen (e.g. a reference compiler targeting a different backend entirely), that requires first
widening `bootstrap.dag`'s witness shape to a semantic/behavioral equality — a load-bearing change
this design does not propose and which would need its own sign-off.

## 4. Non-goals / scope fence

- **Not** replacing or extending `5-real-fixpoint` (full-roster stage1==stage2 self-consistency) —
  that milestone is a precondition this design consumes, not redoes.
- **Not** a general-purpose second `.dag` compiler, and not a path toward TypeScript (Track T)
  joining the fixed point — Track T is a real, separate, valuable receipt but is not a DDC witness
  (§3.1).
- **Not** claiming DDC is an absolute defense. It is a probabilistic one (§6).
- **Not** touching `bootstrap.dag`'s existing `Witness`/fail-closed shape — that machinery is
  already well-formed; only its currently-fabricated *inputs* (the two gaps in §1) change.

## 5. Staged plan (each stage = one signed PR; strictly ordered)

- **Stage 0 — this design PR (non-load-bearing).** This document + a roadmap link. No behavior
  change to `bootstrap.dag`. *This is the PR this session opens.*

- **Stage 1 — dissolve the digest placeholder (depends on `5-real-fixpoint`, per the existing
  roadmap edge `5-seed-honesty → 5-real-fixpoint`).** Replace
  `data pinned_v4_fixed_point_hash: Hash = pinned_v4_fixed_point_hash` with a function that calls
  `canonical_emitted_bytes_digest` (§2) over the real stage1 (== stage2, once `5-real-fixpoint`
  widens the roster) emitted `Medium<String>`. Same treatment for `v2_stage0_hash`/`v2_stage1_hash`/
  `v2_stage2_hash` (`bootstrap.dag:27-29`) — each becomes the real digest of its stage's actual
  emitted bytes, not a same-named atom. **Open question for the operator (not decided here):**
  whether Stage 1 can start *before* `5-real-fixpoint`'s full-roster widen completes, by scoping the
  bootstrap fixed-point digest to exactly the frozen `SeedCapability` subset's emitted bytes (a
  strictly smaller closure than the full 92-file roster) rather than waiting on the full self-host
  fixed point. Flagged, not assumed — the existing roadmap edge is honored by default.
  **Discriminating witness:** perturb one byte of the stage1 emitted source → the digest changes and
  `bootstrap_fixpt_witness` goes `Violates`; identical bytes → `Holds`. (Today, no perturbation can
  move the result — the atom always equals itself.)

- **Stage 2 — build `ddc_reference_compiler` (§3.3).** New tool, new location, scoped to the frozen
  4-construct closure. Reviewed explicitly for zero `src/v1`/`src/v2` imports (mechanical grep gate,
  not just review attention). Proven by compiling the frozen bootstrap source snapshot and running
  it against the same fixed corpus entry v1 already runs (`bootstrap_manual_corpus_harness`).

- **Stage 3 — real DDC execution + typed evidence.** A workflow mirroring
  `self_host_realized_comparison_gate.dag`'s pattern (a `Disposition`-scaffolded shell program,
  `ensure_gunbc_built`, real `gunbc run --claim-run`): build/pin `ddc_reference_compiler` at a
  content-addressed commit, run it and `v1_pipeline` over the same frozen source, compute
  `canonical_emitted_bytes_digest` over each output, compare per §3.4, and produce a real
  `Witness<DiverseCompilationAgreement>` from that execution — not from atom equality.
  `IndependentCompilerPair.evidence`/`DiverseCompilationRun.compiler` stop being bare `Symbol`s
  satisfied by naming themselves; they become `DeclarationRef`-style pins (per
  `std.decl_ref.DeclarationRef`, already used for this exact "point at a real realized binding"
  shape in `self_host_realized_comparison_gate.dag`'s `Scaffold.bind`) into the pinned reference-
  compiler artifact. **Discriminating witness:** flip one byte of `ddc_reference_compiler`'s output
  → `diverse_compilation_agreement_witness` goes `Violates` (today: impossible to construct a red
  case, since `evidence` never reads a real comparison).

- **Stage 4 — wire `bootstrap_seed_honesty_discharge` to Stage 3's real witness.** The chain is
  `IndependentCompilerPair` → `DiverseCompilationAgreement` → `SeedHonestyDischarge`
  (`bootstrap.dag:558-604`, `602-619`); Stage 3 already routes `evidence`/`compiler` through real
  `DeclarationRef` pins, so `independent_compiler_pair_witness` no longer needs (and Stage 4
  deletes) the `evidence == ^independent_compiler_witness` self-check at `bootstrap.dag:563` — its
  `Holds`/`Violates` verdict now falls out of the pins actually resolving, not an atom comparing
  itself. The **top-level** `bootstrap_seed_honesty_discharge` binding (`bootstrap.dag:694-716`) is
  what changes to consume Stage 3's executed `Witness<DiverseCompilationAgreement>` as an argument,
  replacing the inline fabricated `diverse_compilation_agreement_witness(...)` call built from bare
  atoms. `SeedHonestyDischarge` becomes provable by execution for the first time.

Stage 1 can proceed in parallel with Stages 2–3 (independent halves of the same gap); Stage 4
requires both.

## 6. Fail-closed guarantees (§5) and honest limitations

- Every stage's discriminating witness is a **perturbation that must go red** — per DESIGN.md §5,
  a check satisfiable only by editing the declaration (today's state, both gaps) is validation
  theater, not construction. Stage 1/3's witnesses are chosen specifically to prove the checks are
  now sensitive to real bytes, not just internally consistent atoms.
- **DDC is not an absolute defense — say so plainly.** Per DESIGN.md §5's "the word 'never' is the
  trap": this discharges a *specific, named* threat (a Thompson trapdoor surviving because only one
  compiler ever built the seed), not compiler correctness in general, and not collusion between
  `v1_pipeline`'s and `ddc_reference_compiler`'s authors. The provenance discipline in §3.3
  (reference compiler written without consulting `src/v1`/`src/v2`) is the mitigation, not a proof —
  this is honestly a "ratchet forever" residue (§5's three-way classification), not a wall.
- Reserve real independent review (a different author for `ddc_reference_compiler` than for
  `src/v1`/`src/v2`, if available) as the strongest available mitigation for the collusion risk;
  flagged as an operator decision (§7), not assumed.

## 7. Flags for operator sign-off

- **FLAG A — second-compiler candidate shape.** Confirm the scoped `ddc_reference_compiler`
  approach (§3.2–§3.3: independently-written, scoped to the frozen `SeedCapability` closure, zero
  shared code with `src/v1`/`src/v2`) over the alternatives ruled out in §3.1 (self-recompilation,
  TS-target comparison). Needs sign-off because `bootstrap.dag` is DESIGN.md-load-bearing.
- **FLAG B — implementation language for the reference compiler.** Open choice (§3.3); needs a
  decision, not a default, since the only hard constraint is "shares no code/toolchain with
  `src/v1`/`src/v2`."
- **FLAG C — Stage 1 sequencing relative to `5-real-fixpoint`.** Whether the bootstrap fixed-point
  digest can be grounded on the narrower frozen-subset closure ahead of the full 92-file roster
  widen, or must wait on `5-real-fixpoint` completing per the existing roadmap edge (§5, Stage 1).
- **FLAG D — reference-compiler authorship.** Whether a distinct author/reviewer (not the `src/v1`/
  `src/v2` maintainers) should write or independently re-derive `ddc_reference_compiler`, to
  strengthen the collusion-risk mitigation named in §6.

Related: [RED-job counted-divergence ratchet — shape note](regen-divergence-ratchet-shape.md) (#6350's two-job split: `regen_verify` + the self-host staleness arm in a non-required RED job, divergences counted not absorbed) · [post-zero regen gate placement](post-zero-regen-gate-placement.md) (where the enforcing gate lands once divergence pins at zero).

Historical audit record: [regen divergence 31 vs 32 — reconciliation](regen-divergence-31-vs-32-reconciliation.md).

# v4 Leaf-Model Verification: How Do We Know Our Models Are Right?

> **Status:** SCOPING DRAFT — operator sign-off requested on §10 before dispatch.
> **Date:** 2026-05-30
> **Author:** PM May 29 (session `nimble-dove-733`)
> **Sibling to:** `docs/planning/v4-ci-overhaul-2026-05-30.md` (this doc is in the same PR #3959).
> **Trigger:** Operator 2026-05-30: *"When we model something, how do we gain any confidence that our models are 'right' or 'working'? — this is sort of an integration challenge on the leaf models, but it's important to think about up front because, w/o it, our architecture will get very painful/sharp (a lot of errors basically) — EVERY leaf model has zero verification right now — those are where this system really can't verify much currently — I would start with the language files suite as a guinea pig."*

---

## §1. Provocation

The substrate has dozens of leaf models — language files (rust.dag, python.dag, go.dag, ...), format files (json.dag, sql.dag, ...), primitive type substrates (integer.dag, float.dag, ...), lens substrates (cost.dag, complexity.dag, ...). Each declares facts about target reality. **None of them are verified.** Consumers trust their claims; if the model is wrong, downstream output is wrong silently.

The operator's concern: *"our architecture will get very painful/sharp without [verification]"*. The pain has been visible: 7951 rustc errors on full-corpus emit (per `docs/audit/v4-rustc-error-catalog-2026-05-29.md`) are partly model-claims-not-matching-reality at the language-file level. We've been debugging emit when in many cases the upstream model itself is unverified.

This doc scopes a **per-leaf-model verification framework** with language files as the guinea pig — highest value (foundation for emit), highest tractability (claims are concrete and exercisable against the real target).

---

## §2. The problem in one sentence

**Per-leaf-model claims are authored without a mechanism to verify they hold against the target they model.**

Concrete examples (rust.dag at 2026-05-30 HEAD):
- Claim: `i32 inhabits OrderedRing<Int32>` — never exercised against rustc + ord trait.
- Claim: rust's interval spec for `i32` is `int32_interval_spec` — never compared against actual rustc behavior on overflow.
- Claim: surface spelling for `i32` is "i32" — visually obvious, never asserted.

For each: the model says X, the target IS Y, and **we have no test that fails if X ≠ Y**.

Same shape for python.dag, go.dag, etc. Same shape for format files (json.dag claims about JSON parse semantics — never asserted). Same shape for primitive substrate (integer.dag claims about Nat / Int / OrderedRing — laws never exercised on emit).

---

## §3. Why this matters for the architecture

Three compounding effects when leaf models are unverified:

**E1. Errors compound silently.** If rust.dag's claim about `i32` is wrong by 1 bit (e.g., reports a width incorrectly), every emit consuming that claim produces wrong code. The wrong code may compile in some cases and break in others. The bug looks like an emit bug; debugging finds emit; "fixes" emit by adding a workaround; next time rust.dag's claim breaks differently, repeat.

**E2. Architecture sharpness.** Every consumer of a leaf model either (a) defensively re-verifies the model's claims (parallel authority, P2 violation) or (b) trusts blind (silent-failure mode). Both are unsustainable at N leaf models × M consumers scale.

**E3. The substrate-rich / activation-poor pattern at leaf-model granularity.** PR #3938 §3 diagnosed this at the standards level (1 of 17 standards gated). The same pattern is here at finer grain: every leaf model is rich substrate, zero verification activation.

The operator's framing nails the consequence: *"our architecture will get very painful/sharp."* It already is.

---

## §4. What "validate arbitrary emission of N models" means

For each leaf model M, the verification framework answers:

1. **What does M claim?** Enumerate the model's declared facts (typed claims, inhabitance assertions, primitive specs, grammar productions, etc.).
2. **Can we generate a minimal fixture per claim?** For claim C, produce a fixture F(C) — smallest possible artifact that exercises C against the real target.
3. **Does the target agree?** Run F(C) against the real target (rustc, pyright, go vet, python, etc.); capture verdict.
4. **Verdict tally per model.** Aggregate: N claims declared / K PROVEN / L FALSIFIED / M NOT_CHECKED.

This is essentially **L4 (emit-runs-matches-eval) discipline applied at LEAF MODEL granularity instead of full programs**.

It is also the natural consumer of the §10.0 two-axis disposition vocabulary from PR #3938:
- `ship_disposition: PROVEN` ⟺ fixture exercised + target agreed + falsification probe attempted
- `ship_disposition: GAP` ⟺ fixture exercised + target disagreed (the model claim is wrong)
- `engineering_state: CENSUS_NOT_RUN` ⟺ no fixture generated for this claim yet

---

## §5. Per-leaf-model verification framework (3-layer, mirrors CI overhaul)

**Layer A — Claim authority.** Each leaf model's claims live in the model itself (declared facts) OR in a sibling `test/claim/language_model/<model>.dag` directory of CLAIM declarations that reference the model. Claim authoring follows the existing `TestClaim` discipline.

**Layer B — Fixture generator.** For each claim type, a generator emits the minimal fixture that exercises the claim against the target. Generators live with the testgen substrate (T-19, `lens/testgen.dag`) and produce target-source-string OR target-AST-graph artifacts.

**Layer C — Verification runner.** For each (claim, fixture) pair, the runner:
1. Materializes the fixture (writes to a tempfile, builds an AST, etc.).
2. Invokes the real target (rustc / pyright / go vet / python / lean / swift / ...).
3. Captures target's verdict (compile pass/fail, runtime output, diagnostic text).
4. Compares vs the model's expected — emits `Verdict<ClaimSubject>` per quick-tern-735's PR #3961 verdict-surface contract.
5. Aggregates: per-claim verdict → per-model `VerificationReport<Model>`.

**Acceptance contract per leaf model:**
```
verified(model M) ⟺ every claim C in M has a fixture F(C) with target verdict PROVEN
                    AND a falsification probe attempted (target verdict FALSIFIED for a deliberately-wrong claim)
```

---

## §6. Why language files are the guinea pig

Operator framing: *"I suspect they are the highest value / hard value targets."* Concretely:

- **Foundation for emit.** Every emit target consumes a language model. Wrong language model → wrong emit. Verifying language files first short-circuits the largest class of emit bugs.
- **Claims are concrete.** "i32 has surface spelling 'i32'" is literally exercisable by emitting `let x: i32 = 0;` and running rustc. No interpretation gap.
- **Target is real.** rustc / pyright / etc. ARE the reality. Verification = the target compiles or doesn't.
- **Surface is bounded.** 18 language files at HEAD: cpp, dag, ecmascript, english, fidelity, go, java, kotlin, lean, llvm_ir, machine_code, ptx, python, rust, swift, typescript, verilog, wasm. (Of these, dag/english/fidelity are not standard programming languages — separate handling.)
- **High-value bugs are here.** SG-1 (Symbol/Atom value emission, 2978 errors) is partly a language-file modeling issue. Verifying rust.dag first surfaces this class before the SG-1 worker touches anything.

**Proposed Phase 1 fixture set**: rust.dag only. Same fixture-first discipline as the correctness ladder (PR #3938 §7). Once rust.dag verification works end-to-end, widen to python.dag, go.dag, etc. — each language is a separate fixture lane.

---

## §7. Worked example — rust.dag (3 representative claims)

**Claim R1**: rust.dag declares Rust's `i32` primitive type.
- **Fixture**: emit Rust source `pub fn r1_test() -> i32 { 0i32 }`.
- **Target exercise**: `rustc --edition 2024 r1_test.rs --crate-type lib` (or equivalent).
- **Expected verdict**: PROVEN if rustc compiles clean.
- **Falsification probe**: emit `pub fn r1_test() -> i32 { "string" }`; verify rustc rejects with E0308 type mismatch. If rustc accepts, model is wrong about i32's value space.

**Claim R2**: rust.dag declares `i32 inhabits OrderedRing<Int32>` (per algebra inhabitance).
- **Fixture**: emit Rust source that uses i32 with comparison + addition: `pub fn r2_test(a: i32, b: i32) -> bool { (a + b) < a }`.
- **Target exercise**: rustc compile + run with `r2_test(-1, -1)` expected to return `true`.
- **Expected verdict**: PROVEN if compile + runtime behavior matches OrderedRing<Int32> semantics.
- **Falsification probe**: emit code that depends on i32 inhabiting some other algebra (e.g., a Boolean lattice); verify rustc rejects OR the runtime behavior diverges.

**Claim R3**: rust.dag declares Symbol projects to `String` in Rust (per the SG-1 worksheet's tentative target realization).
- **Fixture**: emit `pub fn r3_test() -> /* Symbol */ String { "loop_bound_edge".to_string() }`.
- **Target exercise**: rustc compile + verify the function returns a String at runtime.
- **Expected verdict**: PROVEN if compile + runtime behavior matches.
- **Falsification probe**: emit Symbol as a wrapper struct (`pub struct Symbol(pub String); pub fn r3_test() -> Symbol { Symbol("foo".to_string()) }`); verify rustc behavior changes. If both forms behave identically, the model's choice between alias-vs-newtype is undetectable — which is itself information about whether the claim is meaningful.

For rust.dag at HEAD, the inventory is roughly:
- 13+ primitive type claims (int8/16/32/64/128, uint8/16/32/64/128, bool, char, etc.)
- 4 algebra inhabitance claims (OrderedRing<Int*>, ApproximateField<Float*>, BooleanAlgebra<Bool>, ...)
- Grammar productions per Rust Reference (T-4.17 wave 1 + 2)
- Lex rules per Rust Reference
- Symbol/Bool/Char target realizations (pending SG-1)
- 94 Symbol-tagged catalog entries (rust_std_projection_*, rust_surface_spelling_*, etc. — per keen-heron-687's pre-dispatch finding)

Verification surface estimate: ~50–100 fixtures for rust.dag alone. Bounded, exercisable, and entirely within current testgen + multi-target check infrastructure once wired.

---

## §8. Integration with the CI overhaul (PR #3959)

Leaf-model verification fits cleanly as Upsert<T> CI steps:

```dag
data verify_rust_dag_r1: CiUpsertStep<VerificationReport> = upsert {
  inputs: [
    FileGlob { glob: "src/v4/extdeps/languages/rust.dag" }
    LensOutputRef { lens: TestgenLens, ports: [rust_r1_fixture] }
  ],
  verify: cached_verification_holds(rust_r1_claim),
  create: run_target_verification(rust_r1_fixture, rustc_target),
  resolve: latest_verification_report(rust.dag, r1)
}
```

Then:
- A PR touching `src/v4/extdeps/languages/rust.dag` triggers all rust.dag claim re-verifications.
- A PR touching a single rust primitive (via finer-grained FileGlob or SubstrateNodeSet) triggers only that primitive's claim re-verifications.
- The verify-first phase short-circuits unchanged claims to cached `PROVEN`.
- Each verification produces a `Verdict<ClaimSubject>` per quick-tern-735's contract.

**The CI overhaul (PR #3959 sibling doc) + the leaf-model verification framework are mutually reinforcing**: the overhaul provides the cheap minimal-CI runner; verification provides the things to run that produce confidence. Without verification, the minimal CI runs minimal *nothing-useful*. Without minimal CI, verification at scale is unrunnable.

---

## §9. Manager ownership

| Concern | Primary | Secondary |
|---------|---------|-----------|
| Claim authoring discipline (what counts as a verifiable claim per model) | **Modeling DFS** (proud-pike-680) | Target Realization (keen-heron-687) |
| Fixture generation (testgen extension for language-file claims) | **Runtime/TestClaim** (quick-tern-735) — T-19 testgen substrate | Compiler Spine (smart-stag-871) — generators consume compiler substrate |
| Verification runner (invoke real target, capture verdict) | **Runtime/TestClaim** (quick-tern-735) | Compiler Spine (T-22/T-38 alignment) |
| CI wiring (each fixture is a CiUpsertStep) | **Compiler Spine** (smart-stag-871) | Runtime/TestClaim |
| Fixture-rung mapping (where in the 9-rung ladder do language-file verifications sit?) | **Ladder/Fixture** (keen-crab-361) — most are rung 1 (target type-check) + rung 4 (emit runs and matches) | — |
| Verification disposition + close-receipt (per-model `VerificationReport` becomes a receipt) | **Close/Receipt** (sharp-otter-407) | Self-host/Release (downstream consumer for predicates 2-5) |

**Does NOT need a new manager lane** — fits within existing §11 architecture from PR #3938.

**Critical DFS gate (Modeling DFS):** the claim authoring discipline needs a worksheet — what makes a claim "verifiable", what's the canonical claim shape (probably `LanguageModelClaim<Subject, ExpectedShape>` parameterized over the subject type and the target expectation), what's the falsification probe form. Worker briefs cannot dispatch until this worksheet is approved (same §10.0 rule from PR #3938).

---

## §10. Open questions for operator

**D-LMV-1.** Accept the per-leaf-model verification framework (§5) as architecture?

*Proposed: accept.* Without it, the leaf-model layer remains the substrate-rich/activation-poor failure mode at finest granularity.

**D-LMV-2.** Accept language files (`src/v4/extdeps/languages/*.dag`) as the Phase 1 guinea pig?

*Proposed: accept.* Highest value (foundation for emit) + highest tractability (claims exercise against real targets) per operator framing.

**D-LMV-3.** Phase 1 fixture choice: **rust.dag only** (single-fixture-first per §7 of PR #3938) OR **all 8+ language files in parallel**?

*Proposed: rust.dag only.* Same discipline as PR #3938 §7: prove the framework end-to-end on one fixture before widening. If rust.dag verification works, widening to python.dag/go.dag/etc. is mechanical.

**D-LMV-4.** Where do verification claims live?
- (a) **In the model itself** (model self-declares verifiable claims as inline data)
- (b) **In a sibling `test/claim/language_model/<model>.dag` directory** (claims reference model but live separately)
- (c) **Generated by testgen** (testgen enumerates claims from the model's declared facts; no manual authoring)

*Proposed: start with (b), evolve toward (c).* (a) couples claim authoring with model authoring (probably fine for self-declarations but limits external claim authoring). (b) is the current TestClaim discipline. (c) is the eventual zero-floor target.

**D-LMV-5.** Confirm **Runtime/TestClaim + Compiler Spine** as primary owner pair?

*Proposed: yes.* Runtime/TestClaim owns testgen + runner; Compiler Spine owns CI wiring. No new manager lane.

**D-LMV-6.** Integration sequencing — does leaf-model verification dispatch BEFORE / AFTER / IN PARALLEL with CI overhaul Phase 1.5 (CiUpsertStep<T> substrate)?

*Proposed: in parallel, with verification's CI wiring depending on Phase 1.5 landing.* Verification framework design (this doc) is independent of Upsert<T> substrate; integration uses Upsert<T> when it lands.

---

## §11. What this doc is NOT

- **Not a redesign of testgen (T-19).** T-19 substrate exists; this doc proposes extending it for language-file claim generation.
- **Not a redefinition of the §10.0 disposition vocabulary.** Per-model `VerificationReport` consumes the existing PROVEN/GAP/NOT-CHECKED axis.
- **Not a critique of leaf models.** Leaf-model claims are real substrate work — the absence of verification is the gap, not the models themselves.
- **Not a substitute for cross-target equivalence** (rung 5 of the 9-rung ladder per PR #3938). Per-leaf-model verification is rung 1 (target type-check) + rung 4 (emit runs and matches) discipline at finer granularity. Cross-target equivalence is a separate rung.

---

## §12. Related artifacts

- **`docs/planning/v4-ci-overhaul-2026-05-30.md`** (sibling in this PR) — the minimal-CI substrate that runs verifications cheaply.
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §6, §7, §10 — the 9-rung ladder + DFS worksheet discipline this verification framework consumes.
- `src/v4/extdeps/languages/*.dag` — the 18 leaf models that Phase 1 targets (`rust.dag` first per §6).
- `src/v4/lens/testgen.dag` — T-19 testgen substrate that fixture generators extend.
- `src/v4/test/claim/` — existing TestClaim discipline; language-file claims would live under `test/claim/language_model/`.
- PR #3961 (verdict surface contract by quick-tern-735) — `Verdict<A>` shape that per-claim verdicts consume.
- `docs/audit/v4-rustc-error-catalog-2026-05-29.md` — the 7951-error catalog that motivated this scoping; many entries are partly leaf-model-claim-wrong issues, not pure emit bugs.

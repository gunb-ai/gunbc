# R3 Launch-Claim Coverage Analysis

**Status**: PROPOSAL — review pending from Director (zesty-bear-812 #828) + Research PM (loyal-swift-270 in `gunb-ai/ctrl#339`).

**Purpose**: Single source of truth for the launch-claim coverage analysis surfaced through the cross-program adversarial-test-of-R3 (research PM's Round 1+2 viability research) + Brian's framing reframes (Framing C; substrate-vs-application convention split). Captures what's been ratified, what's open, what's load-bearing for R3 release.

**Why this doc exists**: substantial cross-program signal accumulated across 12+ inbox exchanges between PM, Director, and research PM. Brian flagged that tracking it across that surface is hard. This doc consolidates findings + decisions so nothing drops between R3 close and launch.

## 1. Origin

Research PM (loyal-swift-270, `gunb-ai/ctrl`) ran a viability-research project on gunbc/daglang per Brian's directive. Round 1+2 surfaced:

- Round 1: gunbc fits failure-cluster economics at the *adoption* level (Wing/Ballerina/Pony/Crystal pattern) — but this conflated technical-claim with adoption-economics.
- Round 2: reframed around *technical-advantage demonstration*. Building demos that falsify or confirm gunbc's claim to exponential advantage over traditional languages.
- Round 3 (in flight): adversarial-test of R3 design + prioritization against the sharpened claim. Phase 3 invariant-mapping landed at 6/21 = 28.6% Class B→A upgrade rate (within informed prior 5–30%). **Adversarial gap analyses against four target systems** — LLVM (`gunb-ai/ctrl#365`, merged; 0/35 As after adversarial review), Kubernetes (`gunb-ai/ctrl#366`, merged; 0/18), Discord/Elixir (`gunb-ai/ctrl#367`, merged; 0/18), PyTorch (`gunb-ai/ctrl#368`, open; 6/19) — provide convergent empirical grounding for §4 exhaustivity argument and §5 Class A/B/C measurement target. The four PRs converge on a substantive thesis recommendation (see §6d Tier 4 "out of scope, declared").

Brian's reframe through this work surfaced three clarifications:

1. **Technical claim is sui-generis**, separate from adoption-economics comparison. No language has attempted "no intra-program bugs by construction" before. Different question from "how does a new language get adopted."
2. **Program IS intent declaration** (Framing C). gunbc validates intent soundness; cannot validate intent-vs-want — that's the human-specification gap, ineliminable.
3. **Convention preferences split into substrate-level commitments + application-level user-declarable conventions** — a structural distinction that sharpens the claim further.

This doc captures the resulting structural picture as of 2026-05-04.

## 2. The launch claim (sharpest framing as of 2026-05-04)

Two layers, decoupled:

**Layer 1 — Intent soundness (structurally validated by construction)**:
> *gunbc validates that the declared intent is structurally consistent: type-correct, algebraically valid, invariant-preserving, terminating, total. The program IS the intent declaration; gunbc's compile-time discipline checks that intent is sound across all structural dimensions.*

**Layer 2 — Intent-vs-want (behavioral analyses surface mismatch)**:
> *gunbc cannot validate that the declared intent matches what the user actually wanted — no language can; that's the human-specification gap. But because everything is modeled as data, gunbc provides integration tests + property-based testgen + behavioral analyses that detect intent-vs-want mismatch faster and more comprehensively than any alternative.*

Both layers together, in honest framing:

> *We validate intent soundness completely **within the authored substrate**. We cannot validate intent-vs-want — no language can. We provide intent-vs-want analysis beyond alternatives.*

The "within the authored substrate" qualifier is load-bearing per the four-PR adversarial-gap-analysis portfolio (PyTorch surfaced multiple intent-soundness properties — precision-parametric algebra; non-smooth subdifferential selectors; allocator-state-machine modeling — that the substrate cannot yet express, not because they're outside the thesis but because the carriers aren't authored). Three structural classes worth distinguishing:

- **Today-banked substrate**: Tier 1 (R1) + L1/L7 + CX (R1+R2 closed). Intent-soundness validation already cashes for properties expressible against this substrate.
- **Thesis-supported but not yet authored**: e.g., post-R3 phantom-parameters, `Algebra<Precision>`, `Determinism` lens, bounded-atom-creation lens — surfaced across the four adversarial PRs. Within thesis scope; substrate carriers pending. See §6e.
- **Outside thesis** (intent-vs-want; truly empirical/statistical residuals; out-of-scope categories per §6d Tier 4 recommendation).

This is the strong claim, and it's defensible because it's honest about both the human-specification gap (which compiler-people read as epistemic seriousness) AND the substrate-extension trajectory (which prevents collapsing in-flight substrate work into already-banked claims). The structural claim survives intact; the behavioral-analysis claim is sui-generis.

## 3. Convention preference split (refinement)

Conventions divide into two structurally-distinct classes:

### 3a. Substrate-level commitments (built into gunbc; not user-declarable-away)

These are *structural invariants of gunbc itself*, not preferences:

- **No partial functions** — Tier 2: all partial functions made total. A developer who *prefers* nullable-return-style cannot have it; the substrate rules it out.
- **No aliased mutation** — ownership lens (banked at L1).
- **All recursion bounded** — CX gate (R1).
- **Errors as Diagnostics** — fail-closed discipline (`feedback_fail_closed_discipline` C-8).
- **Parallelism is the default** — sequential execution requires data-dependency justification.
- **No metadata markers** — types are structural, not labeled.
- ...etc.

These can't be opted out of. A program either expresses intent compatible with substrate commitments, or it doesn't compile.

### 3b. Application-level invariants (user-declarable via `apply_lens`)

These are project-author-declared invariants that gunbc enforces across declared scopes. `apply_lens` is the substrate-supported authoring surface; the carrier is `EnforcedApplication<Output, Budget>` per `docs/design-lens-application-surface.md` with `SectionRef::DeclarationScope | NodeScope` for scoping.

Examples of application-level invariants:

- **Complexity-contract enforcement**: *"this function never exceeds O(N log N)"* via `apply_lens(complexity, fn, Enforce { budget: O(N log N), ... })`
- **CRDT cost-basis contracts**: *"this merge function is bounded by O(M)"* via `apply_lens(cost, fn, Enforce { ... })` with cost-axis Lens<C>
- **Memory-peak contracts**: *"this transform never peaks above K"* via memory-peak Lens<C>
- **Cross-iteration parallelism opt-in**: *"this loop has independent iterations"* via parallelism Lens<C>
- **Custom user-defined lens enforcement**: any `Lens<Output>` the user authors over substrate facts (e.g., SSA-form for an LLVM-pipeline `.dag` program; project-specific deterministic-execution invariant)

For these, the project author either:
- **Authors the invariant explicitly** via `apply_lens(my_invariant, scope, Enforce {...})` — gunbc enforces across the scope and emits Diagnostics on violation
- **Authors as `Introspect`-only** for inspection without enforcement (lens output available downstream; no Diagnostic on "violation")

**Default policy is RESOLVED** (not an open question — see §6c for authority cite): unannotated functions get synthesized `IntrospectApplication<ComplexitySummary>` only; no inferred `Enforce`; explicit Enforce requires explicit user authoring with declared budget.

**What §3b does NOT cover** (worth disambiguating since the original framing conflated these):

- **Code-style conventions** (builder vs constructor patterns; module organization; naming patterns; error-propagation phrasing). These aren't `apply_lens` use-cases — they're either compiler-emission choices (structurally determined) or user-program-shape choices (the user authors the code in the shape they want). gunbc doesn't synthesize style-convention defaults; the substrate or the user's program determines the shape directly.

## 4. Five-category intra-program bug partition

The research PM's adversarial test produced this candidate-exhaustive partition. Each category maps to a specific R3 lane or banked surface:

| # | Category | What it covers | gunbc surface |
|---|---|---|---|
| 1 | **Structural / type-theoretic** | Type mismatches, field typos, non-exhaustive matches, bare container types, circular dependencies, stale imports, cross-target drift | **Tier 1** — banked since R1 (impossible-to-write) |
| 2 | **Algebraic correctness** | Operations violating declared algebra (commutativity, associativity, identity, distributivity, ring axioms, lattice meet/join) | **L7 algebraic-inhabitance witnesses** — `T-Verification-L4-L7-Direct` (R3); per-(algebra, inhabitant, law) exhaustive coverage |
| 3 | **Invariant preservation** | SSA-form, dominance, type-soundness, observable-behavior, PHI-well-formedness, freeze, etc. | **`apply_lens` enforce** — `T-Lens-Application-Surface` (R3 cascade); compile-time fail-closed |
| 4 | **Termination / totality** | Unbounded recursion, partial functions, division by zero, integer overflow, OOB, force-unwrap, aliased mutation | **CX gate + Tier 2 + ownership** — banked since R1+L1 |
| 5 | **Behavioral equivalence at boundaries** | Build/linker/sanitizer/heuristic invariants; cross-target/cross-mode behavioral mismatch | **5th R3 gate** `integration_testgen_demonstrated_on_at_least_one_domain` (RATIFIED) |

The partition's structural argument: **if Categories 1-4 cover all intra-program bug shapes within bounded-iteration closed-system scope (Layer 1 — intent soundness), and Category 5 covers boundary equivalence via testgen (Layer 2 — intent-vs-want behavioral analyses), then the strong claim cashes.**

The "100% back it up" empirical question: **is 1+2+3+4 actually exhaustive over intra-program bug shapes within bounded-iteration closed-system scope?**

That's measurable. See §5.

**Category 5 sharpening (per K8s + PyTorch adversarial PRs)**: Cat 5 covers **axis derivation** — the structural facts that define the integration-testgen surface (build dependencies; linker section layout; sanitizer contracts; heuristic monotonicity). It does **NOT** claim exhaustive **cell sampling** across vendor-runtime cross-products (e.g., "every kernel × every accelerator × every driver × every CUDA version"). Cell sampling is empirical-residual territory; that's an explicit non-claim, not a gap. The 5th gate's `integration_testgen_demonstrated_on_at_least_one_domain` requires demonstrating axis derivation works on at least one domain; cell-sampling saturation is post-R3 ecosystem work bounded by the substrate's expressive surface.

**The bounded-iteration-closed-system qualifier**: across the four-PR portfolio, the qualifier surfaces as load-bearing. LLVM (local thesis strong; toolchain/global limits genuinely outside) + Kubernetes (matrix-cell-sampling problem) + Discord/Elixir (OTP territory disclaimed by THESIS line 23) + PyTorch (data-dependent numerical instability + cross-process coordination) converge on naming the boundary explicitly. See §6d for the Tier 4 recommendation.

## 5. Three-class behavioral partition (for exhaustive-coverage demo)

For a chosen `.dag`-scope program, intra-program bugs partition into three classes:

- **Class A — caught by soundness validation**: structural / algebraic / invariant-preserved / termination violations. Compile-time fail-closed. Maps to Categories 1-4.
- **Class B — intent-vs-want mismatch**: program is sound but user wanted different behavior. **No language catches this. gunbc's contribution: behavioral analyses surface the mismatch faster + more comprehensively than alternatives.**
- **Class C — boundary / integration**: cross-target, cross-mode, build/linker/sanitizer/heuristic. Caught via testgen. Maps to Category 5.

The launch claim cashes if:
- **Class A is high** (intent soundness is structurally complete)
- **Class B's behavioral-analysis surface is provably better than alternatives** (intent-vs-want analysis exceeds what other languages provide)
- **Class C is demonstrated** (5th R3 gate ratified)

Measuring this partition on a moderate-complexity `.dag`-scope program is the **exhaustive-coverage demo target** — the right framing for the demo work that ctrl is reshaping (see §9 outstanding work).

## 6. Open questions (need disposition before R3 close / launch lock)

### 6a. §3c invariant-list policy (open-ended vs closed)

**Where**: design-coord thread `gunb-ai/gunbc#1586` anchor 5 (carrier-general lens enforcement).

**Question**: should the §3c invariant list (currently 5: SSA-form / dominance / type-soundness / observable-behavior / PHI-well-formedness) be closed at R3, or open-ended structurally?

**PM recommendation**: **open-ended structurally** with ≥N demonstrated at launch as the closure criterion. Per `feedback_construction_over_ratchets` and `feedback_parallel_representation_debt`, closing the list creates a parallel-representation ratchet. Open-ended substrate accommodates user-declared invariants naturally per Framing C.

**Status**: PM recommendation posted; awaiting Substrate Mgr (jolly-ram-908 #1130) + Director response in-thread.

### 6b. Tier 3 cpp/ extdeps scope (translator-rewrite in/out)

**Where**: `gunb-ai/gunbc#828` PM routing at [comment 4367640264](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4367640264).

**Question**: does "C++ as bidirectional language" include re-authoring xlscc-translator-equivalent in `.dag`, or C++ ingestion only?

**Citation impact**: 6 upgrades (28.6%) vs 5 upgrades (23.8%) in Phase 3 launch-narrative.

**Status**: routed to Brian via Director; pending response.

### 6c. Default policy for `apply_lens` invariant enforcement — RESOLVED (per existing lane authority)

**Status**: **RESOLVED**, not open. Cited here as a resolved fact for completeness; included to prevent reopening at the launch-claim-coverage layer.

**Resolution**: per `docs/r3-structure.md:40, 148` + `docs/design-lens-application-surface.md` §3.2 + §5.1 + §8.3 (resolved at e9d67113e):

> *Default policy for complexity contracts: user-driven. Unannotated functions get synthesized `Introspect`-only applications — no implicit baseline, no inferred `Enforce`. Enforcement requires explicit user authoring of `apply_lens(complexity, fn, Enforce { ... })`.*

The synthesized `IntrospectApplication<ComplexitySummary>` exists fold-pass-only (not stored in source) and ensures the lens-fold walk is total over function declarations without requiring explicit per-function authoring.

This generalizes to any `apply_lens` invariant: **default for unannotated scope is `Introspect`-only synthesis (or no synthesis at all for non-default lenses); explicit `Enforce` requires explicit user authoring with declared budget.** Per `feedback_state_space_vs_behavioral_invariants` + design doc §2-3, the carrier shape (`EnforcedApplication<Output, Budget>` vs `IntrospectApplication<Output>` as separate top-level carriers) makes Enforce-without-budget structurally unrepresentable; `apply_lens` use without explicit Enforce can't accidentally activate enforcement.

**Note on the original framing of this section** (kept for traceability): an earlier draft of this doc treated default-behavior as an open question and named three options (structurally-derived; project-config; arbitrary-but-consistent). That framing was a P1 violation — the question is resolved at the lane authority. This entry now cites the resolved policy directly. The "code-style conventions" examples that earlier-draft §3b conflated (builder vs constructor; module organization; etc.) are clarified in §3b as out-of-scope-for-`apply_lens`; not part of this resolved-policy authority.

### 6d. Tier 4 "out of scope, declared" — pending Director ratification (NEW; convergent across four adversarial PRs)

**Where**: not yet routed; surfaced through research PM PR review of this doc.

**Recommendation**: add an explicit **Tier 4 — out of scope, declared** section to `THESIS.md` naming the boundaries gunbc *does not claim*. Today THESIS reads as unbounded; explicit boundaries make the surrounding claims more credible (compiler-people read scoped-with-named-boundaries as epistemically serious; unbounded-by-omission as marketing).

**Convergent boundaries from the four-PR adversarial portfolio**:

- **Kernel-implementation correctness** — verified-compilation territory (CompCert, F\*, Lean). gunbc provides structural facts those tools consume; pure Transform-preservation proof is their domain.
- **Vendor-runtime cross-product cell-sampling** — exhaustive coverage across hardware × driver × runtime version cells. gunbc derives the axes structurally; saturating cells is empirical-residual.
- **Data-dependent numerical stability** — emergent training dynamics; algorithm-stability-as-empirical-property. Adjacent to but distinct from `Algebra<Precision>` substrate work (§6e).
- **Cross-process coordination** — distributed-systems consensus; pi-calculus / TLA+ territory. gunbc local thesis strong; cross-process formal proof is its own discipline.
- **OTP-style fault tolerance** — Erlang/Elixir-native. THESIS line 23 already disclaims; recommend making the disclaimer Tier-4-explicit.

**Why this matters for launch**:
- Compiler-people reading the launch material will probe boundaries adversarially. Explicit Tier 4 naming defangs the probe (we already declared the boundary).
- The §2 strong claim ("validates intent soundness completely within the authored substrate; provides behavioral analysis beyond alternatives") is more defensible against adversarial framing if Tier 4 is explicit.
- Per `feedback_construction_over_ratchets`, Tier 4 isn't a heuristic patch — it's a structural commitment that the closed-system principle's *closure* is bounded; bounded closure with explicit non-claims survives review better than implicit unbounded closure.

**Decision needed (Director / Brian)**: ratify Tier 4 addition to `THESIS.md` (or thesis-mapping doc). If ratified, recommend folding into the operational-equivalence-stance amendment cycle (§7b) — both are thesis-doc additions surfaced by the adversarial review.

**Status**: pending Director routing; recommend separate focused PM-to-Director ask after this doc lands.

### 6e. `Algebra<Precision>` substrate need — pending Substrate Mgr review (NEW; surfaced from PyTorch adversarial PR)

**Where**: not yet routed; surfaced through research PM PR review.

**Question**: PyTorch adversarial gap analysis (`gunb-ai/ctrl#368`) surfaced a substrate need: precision-parametric algebra. Adjacent to but stronger than post-R3 phantom-parameter wrappers. Would let users declare "this `Float` operation is precision-parametric over {f16, f32, f64}; algebraic laws hold within precision class; cross-precision conversion has typed cost."

**Implications**:
- Within thesis scope (algebra-with-typed-parameter is consonant with the `Algebra<C>` framing)
- Substrate carriers pending — not in current R3 plan
- Touches `T-Numeric-Construction` (refined integer types; Magnitude / ApproximateField work) but extends the carrier shape

**Status**: pending Substrate Mgr (jolly-ram-908 #1130) review of substrate-extension scope. Could be R3 substrate-completion work absorbed under §187 standing protocol if classified as substrate-gap; otherwise post-R3 ecosystem.

### 6f. Verification Mgr domain selection for 5th gate

**Where**: Director-deferred to Verification Mgr (fierce-ferret-556 #1276) per ratification at [#828 comment-4367830997](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4367830997).

**Question**: which domain (build-system / heuristic-cost-function / sanitizer-contract / linker-layout) hosts the worked TestClaim demonstration for `integration_testgen_demonstrated_on_at_least_one_domain`?

**Director's framing** (offered to Verification Mgr; not pre-selected):
- **heuristic-cost-function**: closest to gunbc's existing cost-lens work (natural extension via SymbolicCost semiring + `Lens<C>`)
- **build-system**: broadest launch-narrative visibility
- **sanitizer-contract**: demo-friendly + concrete

**PM read shifted under Framing C**: **build-system** is the strongest demonstration choice for the unprecedented claim. Build-system bugs in conventional languages are usually classed as "system" or "tooling" issues, not language issues — gunbc's claim that build is intra-program (because workflow) is sui-generis. Demonstrates behavioral analysis on a domain other languages don't try to validate at all.

**Status**: pending Verification Mgr selection.

## 7. Ratified R3 decisions surfaced through this analysis

These have been ratified through the cross-program coordination chain (research PM → PM → Director → Brian) and are load-bearing for R3 close + launch:

### 7a. 5th closure gate on T-Tests-As-Data-Completeness

**Gate name**: `integration_testgen_demonstrated_on_at_least_one_domain`

**Closure criterion**: ≥1 worked `TestClaim` demonstrating that domain's invariants are testgen-coverable.

**Sizing**: absorbs into T-Tests-As-Data-Completeness lane scope; demo work runs parallel-track per `r3-structure.md:187` standing protocol.

**Authority**: Director-ratified at [#828 comment-4367830997](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4367830997); Brian-authorized via *"i would make these sorts of integration testgen an R3 gate if needed."*

**TODO**: r3-structure.md amendment PR adding the 5th gate (queued for after #1586 design-doc lands; folds into the same amendment cycle).

### 7b. Operational-equivalence thesis-stance

**Stance**:
> *"Operational equivalence is verified-by-execution via Tier 3 (L4/L5 differential), not formal proof. Formal-proof composition with verified-compilation toolchains (CompCert, F*, Lean) is post-R3 ecosystem work — gunbc provides the structural facts that those tools consume; pure Transform-preservation proofs are their domain, not gunbc's."*

**Authority**: Director-ratified at [#828 comment-4367830997](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4367830997).

**TODO**: folds into queued `r2-r3-thesis-mapping.md` disposition-row amendment (or THESIS.md addition) pending #1586 design-doc landing.

### 7c. Cascade-slip notification protocol

**Trigger**: material cascade-slip on `T-Lens-Application-Surface` (proxy: >~3 months gap between cascade-stage close and next stage opening, OR explicit substrate-blocker visibly invalidating timeline).

**Channel**: Director → PM (#846) → ctrl (`gunb-ai/ctrl#339`).

**Director-side commitment**: cascade health added to ongoing Substrate-monitoring via #1130; if Substrate flags an explicit cascade-blocker OR Director observes material gap accumulation in cadence, Director routes through PM.

**Authority**: Director-ratified at [#828 inbox-message-4367089130](https://github.com/gunb-ai/gunbc/issues/828).

### 7d. Iterative-pressure-test framing for ctrl pressure-test substrate gaps

**Mechanism**: ctrl pressure-test discoveries route to Substrate Mgr (jolly-ram-908 #1130) per `r3-structure.md:187` standing protocol — already Director-locked 2026-04-28; carried through 2026-05-02 expansion. Substrate Mgr absorbs gaps as substrate-completion work, not as new lanes.

**Status**: standing protocol; no new ratification required.

### 7e. Bidirectional language-spec architecture (parallel-track / post-R3)

**Decision**: bidirectional ingestion/emission work pushes forward as parallel-track or post-R3, NOT as an R3 lane. Cross-program design-coordination thread `gunb-ai/gunbc#1586` is the venue for architectural convergence.

**Six anchor questions** in the thread:
1. Bidirectional file naming convention (`parse.dag` vs `ingest.dag`)
2. Substrate sharing across consumers (LLVM IR ↔ ORCv2 ↔ DWARF; LLVM ↔ XLS)
3. Shape B emit pattern as production precedent
4. xlscc subset definition for C++ bidirectional extdeps
5. Carrier-general vs complexity-only `lens_enforcement_carrier_landed` (covers §3c question)
6. Versioning architecture (per Brian's OQ #2 on version-modeling-in-extdeps)

**Authority**: Director-ratified at [#828 comment-4366929544](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4366929544).

**Status**: thread open; convergence in flight; OQ #8 = Reading B (hold launch for v1) confirmed.

## 8. Implications for R3 release / launch positioning

### 8a. R3 close vs launch decoupling (per OQ #8 = Reading B)

Public launch is no longer aligned with R3 close. Launch is gated specifically on `T-Lens-Application-Surface` cascade dispatching:
- T-E-P-Producer-Broadening
- T-Lens-Behavioral-Parity (4 sub-slices: complexity / cost / parallelism / effect_enumeration)
- T-Lens-Application-Surface
- v1 carriers land

Multi-stage substrate cascade. R3 close horizon may be different from launch horizon; both are structurally gated.

**Implication for R3 release planning**: the "R3 close" milestone and "public launch" milestone are now distinct. Some R3 lanes (e.g., T-V2-Retirement, T-Numeric-Construction-after-cascade) may close after public launch.

### 8b. Honest framing strengthens the claim

The Framing C reframe (intent soundness vs intent-vs-want) makes the launch claim defensible:

> *"We validate intent soundness completely. We cannot validate intent-vs-want — no language can. We provide intent-vs-want analysis beyond alternatives."*

The "we cannot validate intent-vs-want" admission is what makes the rest of the claim land. Compiler-people read this as epistemic seriousness; the over-claim ("no intra-program bugs") would have been read as marketing.

### 8c. Behavioral analyses are now load-bearing for the strong claim

Pre-Framing-C: 5th R3 gate (`integration_testgen_demonstrated_on_at_least_one_domain`) was framed as "covering boundary equivalence" — adjacent to the main claim.

Post-Framing-C: **the 5th gate is the structural commitment that gunbc delivers on the "behavioral analyses beyond other languages" claim**. Without demonstration, that part of the strong claim doesn't cash. With demonstration, the claim is honest: *"here's intent-vs-want analysis that no other language gives you."*

This makes Verification Mgr's domain selection (§6f) materially more important for launch positioning.

### 8d. Cross-program impact on ctrl's Phase 4 launch narrative

Phase 4 (launch-narrative reframing) drafting can advance against:
- ✅ Cat 1 (algebraic preservation) — structurally covered
- ✅ Cat 2 (operational equivalence) — explicit empirical-via-testgen + post-R3-ecosystem-integration stance ratified
- ✅ Cat 4 (testgen integration) — 5th gate ratified
- ✅ Cat 5 (termination/totality) — structurally covered
- ⏳ Cat 3 — pending §3c policy (#1586). (Default-policy on `apply_lens` per §6c is RESOLVED; only §3c invariant-list policy on #1586 remains.)
- ⏳ OQ #4 — pending Director answer

Recommend: ctrl drafts Phase 4 against most-likely-disposition (open-ended §3c + translator-rewrite-included on OQ #4) with explicit conditional-citation language; final lock is a quick pass when both pending pieces land.

## 9. Outstanding work tracking

### 9a. ctrl-side dispatches

- **vivid-dove-240 re-task**: from "find LLVM bugs" (retrospective citation) to "exhaustive coverage demo on chosen `.dag`-scope program with Class A/B/C partition measurement, ideally measured on both A/B/C bug-shape axis AND substrate-supported / proof-supported / empirical-residual evidentiary axis (per orthogonal-axis reframe in flight)." Pending ctrl-side dispatch.
- **`.dag`-scope program selection**: purpose-built moderate complexity, ~100-300 LOC, all five Behaviors + multiple algebras + user invariants + effects + at least one Class B example. Pending ctrl-side decision.
- **Phase 4 launch-narrative draft**: partial draft against ratified categories; final lock pending §6a + §6b + §6d resolution.
- **Three-bucket / two-axis viability synthesis artifact** (NEW; pending Brian's confirmation of orthogonal-axis reframe per `gunb-ai/ctrl#339`): consolidates the four adversarial findings (LLVM/K8s/Discord/PyTorch) into a single research artifact with the substrate-supported / proof-supported / empirical-residual partition + Tier 4 boundary recommendation. This artifact is what makes vivid-dove-240's re-task target measurable — without it, the Class A/B/C measurement on a chosen `.dag`-scope program lacks the partition framework to compare against external systems. Will dispatch on Brian's confirm.

### 9b. gunbc-side dispatches

- **Verification Mgr domain selection** for 5th gate (§6f): pending.
- **r3-structure.md amendment**: lane scope addition for 5th gate (§7a). Queued for after #1586 design-doc lands; folds into amendment cycle.
- **`r2-r3-thesis-mapping.md` / `THESIS.md` amendment**: operational-equivalence thesis-stance (§7b) + bidirectional disposition-row from §7e + **Tier 4 "out of scope, declared" addition (§6d) if Director ratifies**. Queued for after #1586 design-doc lands.
- **#1586 §3c policy convergence** (§6a): Substrate Mgr + Director response in-thread.
- **Director response on OQ #4 cpp/ scope** (§6b): pending.
- ~~#1586 default-behavior on application-level conventions (§6c)~~ — **N/A**: RESOLVED at the lane authority + design doc; not an open question. See §6c.
- **Director ratification on Tier 4 "out of scope, declared"** (§6d, NEW): focused PM-to-Director ask after this doc lands. Convergent across four adversarial PRs; strongest single thesis-edit recommendation from leverage research.
- **Substrate Mgr review of `Algebra<Precision>`** (§6e, NEW): substrate-extension scope question; could be R3 substrate-completion work absorbed under §187 standing protocol, or post-R3.

### 9c. Cross-program signal channels (active)

- **PM ↔ Director** via `gunb-ai/gunbc#828` ↔ `#846`
- **PM ↔ Substrate Mgr** via `gunb-ai/gunbc#1130` ↔ `#846`
- **PM ↔ Research PM** via `gunb-ai/ctrl#339` ↔ `#846`
- **Coordination thread** at `gunb-ai/gunbc#1586` (multi-participant)
- **Cascade-slip protocol** ratified for Director → PM → ctrl

## 10. Review asks

### 10a. Director (zesty-bear-812 #828)

Confirm:
- §7 ratifications match your understanding (5th gate, operational-equivalence stance, cascade-slip protocol, iterative-pressure-test framing, bidirectional architecture coordination)
- §6 open questions — anything that should be re-routed differently or that has guidance you can give now
- §8 implications — anything additional load-bearing for R3 release that's missing from this doc
- Surface anything I've missed in the cross-program coordination chain

### 10b. Research PM (loyal-swift-270 in `gunb-ai/ctrl#339`)

Confirm:
- §2 Framing C captures the framing as you understand it post-Brian-reframes
- §3 substrate-vs-application convention split matches your sharpening
- §4 5-category bug partition matches your Phase 3 invariant-mapping framework
- §5 Class A/B/C partition is the right exhaustive-coverage demo target
- §9a outstanding ctrl-side work captures what your dispatches need to do
- Surface anything else from the adversarial-test that this doc misses

### 10c. PM (deep-wolf-155, this session)

Will respond to review feedback in-thread; will author the queued amendments (r3-structure.md 5th gate addition; r2-r3-thesis-mapping.md disposition rows for operational-equivalence stance + bidirectional architecture) once #1586 design-doc lands and §6 open questions resolve.

## Cross-references

- **Director ratifications**: `gunb-ai/gunbc#828` comment-4366929544 (bidirectional architecture); comment-4367089130 (cascade-slip protocol); comment-4367830997 (5th gate + operational-equivalence stance)
- **Research PM thread**: `gunb-ai/ctrl#339` (full coordination history)
- **Four-PR adversarial-gap-analysis portfolio** (empirical grounding for §4 + §5 + §6d):
  - LLVM gap analysis: `gunb-ai/ctrl#365` (merged; 0/35 As after adversarial review)
  - Kubernetes gap analysis: `gunb-ai/ctrl#366` (merged; 0/18; matrix-cell-sampling boundary)
  - Discord/Elixir gap analysis: `gunb-ai/ctrl#367` (merged; 0/18; OTP territory disclaimed)
  - PyTorch gap analysis: `gunb-ai/ctrl#368` (open; 6/19; surfaced `Algebra<Precision>` need + data-dependent numerical instability boundary)
- **Coordination thread**: `gunb-ai/gunbc#1586` (bidirectional language-spec architecture; six anchor questions)
- **Substrate Mgr signal**: `gunb-ai/gunbc#1130` comment-4367070285 (consumer-requirement reframing for `lens_enforcement_carrier_landed`)
- **Thesis grounding for §2 framing**: `docs/thesis/what-else-falls-out.md:6-20` §"Frontend/backend agnosticism"; `THESIS.md:155-405` thesis claims list
- **R3 lane structure**: `docs/r3-structure.md` (16 lanes + 1 standing program = 17 active surfaces; 9 standing R3 managers)
- **Lens application surface**: `docs/design-lens-application-surface.md` (substrate carriers for §3b application-level invariants; §3.2/§5.1/§8.3 default policy authority cited from §6c)
- **Tests-as-data design**: `docs/design-tests-as-data-completeness.md` (substrate for Category 5 + 5th gate work)
- **R3 structural framework feedback memories**: `feedback_construction_over_ratchets`, `feedback_parallel_representation_debt`, `feedback_calibration_vs_oscillation`, `feedback_verify_thesis_claims`

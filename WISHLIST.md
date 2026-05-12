# WISHLIST

Operator scratchpad of forward-looking wishes per milestone. **Wish-tier, not implementation.** When a wish lands a worker brief / §1.8 gate / lane scope, it shrinks to a one-line pointer; when it closes, it moves to the retrospective for that milestone. Detail lives in the canonical authority docs (`THESIS.md`, `ROADMAP.md`, `r3-program-plan.md`, `docs/design-*.md`, `docs/audit/*.md`) — this doc does NOT duplicate them.

**Authority**: operator-authored / operator-ratified. Mgrs/PMs surface wishes via inbox/discussion, not direct edits.

**Dual-rep discipline**: if you find yourself writing implementation detail here, the wish is too narrow — it belongs in a brief or a program plan. WISHLIST entries are the **what we want**, not the **how**.

---

## R1 — closed

**Capacity-layer baseline**: `.dag` substrate with TestClaim runner (DB-15 schema), pure-bootstrap baseline (T-PB-A non-test census + T-PB-B tests-as-data), v2-oracle parity demos, R1C closure-Mgr lane sweep.

Receipts: `docs/r2-structure.md` §"Transition mechanics"; ROADMAP §"R1 Closure Manager"; `docs/design-pure-bootstrap.md` (since superseded by `design-pure-bootstrap-zero.md`).

---

## R2 — closed 2026-04-25 (PR #1275)

**Substrate + runtime + grounding**: lens-primitive carrier substrate (T-Substrate-Lens-Primitive); evaluator-as-data design + first execution (T-Evaluator); Grounding completeness (Tier 1 Rust + Python + Go target primitives structurally modeled); cascade-promotion of Pure Bootstrap to Zero (LIVE 2026-04-25 — supersedes ≤5-floor framings); `ExecuteCommand` PB-Runtime boundary for class-5 boundary tests.

Receipts: `docs/r2-structure.md`; `docs/r2-closure-ledger.md`; `docs/design-pure-bootstrap-zero.md`.

---

## R3 — current

**The thesis-validation milestone.** R3 is where we prove that the substrate + runtime + grounding (R1+R2) actually compose into a self-hosting compiler with zero hand-Rust + verifiable predicates + retired bridges + closed substrate-gap classes.

R3 ships when the 96 R3-load-bearing §1.8 gates are GREEN + `r3_debt_paydown_zero_remaining` GREEN. Detail: `docs/r3-program-plan.md` §1.

**Wishes within R3 scope** (all in flight; pointer-only):
- PB-0: 0 hand-Rust + 0 TESTING residual (gates #8, #84) → `docs/design-pure-bootstrap-zero.md`
- v2 retired (gates #41, #42) → T-V2-Retirement lane
- Bridges to zero (gate #36 + per-bridge #31-#35) → T-Bridge-Retirement lane
- Pattern A NYI predicates executable (TC1/TC2/TC3 + RustDagIso + SymbolicCost) → T-V-L4-L7-Direct lane
- 5 substrate-gap classes closed (#60-#64) → cross-lane
- Multi-target emission with cross-target consistency (#15, #51, #52) → T-V-L5-Corpus + T-Free-Consequences-Demonstration

**Wishes I want from R3 close beyond gates** (audit-tier, not gate-additions):
- An honest read on **what we're losing** during emission/lowering — does the unified-DAG claim hold under self-host pressure, or are there hidden per-target couplings? (R3-close audit point per operator framing 2026-05-10.)
- Confidence that the **architecture is symmetric enough for R4 omni-ingestion** — i.e., extdeps that work for emit also work (with detail extensions) for ingest. R3 doesn't have to deliver this; R3 has to NOT preclude it.
- An explicit **emission-bias leak audit** — score every `dsl/extdeps/*` file + every substrate field for consumer-neutrality (no hidden emission-only assumptions that would preclude R4.A omni-ingestion or R4.C low-level emission targets). Substrate-Mgr-authored audit; ratified by Director. R3-close-or-fast-follow; the concrete output is a consumer-neutrality scorecard + flagged leak sites. (Wishlist ratification 2026-05-12 per operator agreement on the "algebra + projection" framing.)
- A **clean cementing pattern** generalizable beyond R3 — every lens has a cementing test against frozen v2-oracle (gate #87). Pattern should outlive R3 as the standard discipline.
- **Free-consequences demonstrations** (gates #43-#52) — auto-parallelism, auto-memoization, cross-target optimization fall out structurally from lens composition. These are the "thesis cashes" of R3. Want them visceral, not just passing.
- **Operational maturity of the Mgr machine** — 5 standing R3 Mgrs (PB / Substrate / Grounding / Verification / Debt-Paydown) + auto-spawn cascade + ratchet discipline. R3 is also a stress test of whether this org structure scales to R4+.

---

## R4 — upcoming

**The architecture-validation milestone.** R4 is where the unified-DAG claim gets falsified or confirmed by inverse use cases — ingestion (read substrate) and structural query (interrogate substrate). If R3 proves we can build a self-hosting algebra + projection engine in `.dag`, R4 proves the substrate is actually neutral + queryable in the way we claim.

**Framing correction (operator 2026-05-12)**: "compiler" is the legacy name for what gunbc actually is. The more accurate framing is **algebra + projection engine**. The gesture is:

> **LHS** = an input that says "x then y then z" (a typed substrate declaration — the dependency graph)
> **RHS** = a provable version of the LHS (a projection)
> The projection could be: the SAME language re-expressed; ANOTHER language; a witness; a query answer; an affected-set; a diagnostic; a cementing-receipt; or NONE-of-the-above (no emission at all).

**Emission is one projection — it is OPTIONAL.** The architecture's value does NOT depend on always emitting code. Many high-value use cases consume **non-emission outputs**: "does this program satisfy property X" (verification witness), "what's affected if I change Y" (affected-set lens), "give me the structural facts" (introspect lens), "is X structurally equivalent to Y" (parity receipt), and so on. The "compiler outputs are NOT just code" pitch is a load-bearing architectural feature, not a side property.

R4.A (omni-ingestion), R4.B (queries-as-data via Introspect lens), and R4.C (low-level emission targets) are three dimensions of stress-testing this framing.

### R4.A — Omni-ingestion

**Wish**: any language we can emit, we can also ingest. User writes Python, Rust, Go (or whatever subset we support) → gunbc parses + lowers to canonical IR → all the gunbc benefits apply (lens composition, multi-target emission, query). Compiler stays neutral; per-language knowledge lives in extdeps (same extdeps that drive emission, possibly with symmetric-extension entries for ingest).

**Why it matters**:
- Lower entry barrier — you don't have to learn `.dag` to use gunbc
- Architectural falsifier — if any per-target hidden coupling exists in emission, ingestion-via-same-extdeps will fail to invert. R4.A is the proof that R3's compiler-neutrality claim is a *property*, not just an assertion.
- Composes with R4.B — once you can ingest, queries work on existing codebases not just gunbc-authored ones.

**Open questions**:
- Symmetric extdeps: how much extra detail does ingest need vs emit? (Surfaced as work; not blocker.)
- Scope: all 3 emission targets (Rust/Python/Go), or one as proof-of-concept first? Different first-target choices have different costs (Rust = hardest test of architecture; Python = biggest user-experience win).
- Information loss: ingest may not be lossless (Python's dynamic dispatch ≠ Rust's static dispatch); what's the contract for "we ingested your code well enough"?

### R4.B — Introspect-lens saturation + tooling-consumer adapters (user-facing: "queries-as-data")

**LOCKED 2026-05-11 — no `Query` substrate carrier**: the lens vs query distinction is **user-surface terminology**, not substrate. There is only **lens** as a substrate type, with `EnforcedApplication` (compile-time obligation) and `IntrospectApplication` (read-only fact) config variants. "Query" is the user gesture for invoking an Introspect-config lens through a tooling surface (CLI / agent / IDE). R4.B is therefore a saturation lane for Introspect-config lens variants + tooling-consumer adapters — NOT a new substrate type. Avoids the coproduct trap of nicknaming similar concepts (per `feedback_coproduct_dissolution`). Authority: operator ratification at gunbc#846 (2026-05-11); design rationale at [`docs/design-affected-set-lens.md`](docs/design-affected-set-lens.md) §0.

**Wish**: the `.dag` substrate is mechanically queryable — given a program graph, ask arbitrary structural questions and get answers. The lens infrastructure that powers Verification (Enforce config) ALSO powers querying (Introspect config) — same substrate, different config + consumer. IDE / LLM agent / refactoring tool / build system reads Introspect lens output as a query answer.

**Why it matters**:
- LLM agents today spend disproportionate effort understanding program structure (dependency-tracking, refactoring impact, type relationships) — that's CPU work the LLM is doing on GPU. gunbc as the first language designed-from-the-start for LLM-tractable queries.
- Vertical-integration advantage: most languages lose info at every layer transition (source → IR drops source structure; IR → machine code drops type info). gunbc keeps the DAG live across all layers, so queries that span layers ("what's slow at runtime accounting for target realization cost?") are *composed lenses over one substrate*, not a multi-system join. This is the unique-to-gunbc claim worth surfacing as a feature.
- Composes with R4.A — combined, you get **structural queries on ANY codebase you ingested**, not just gunbc-authored ones. That's the competitive feature against existing dev tools (which all reverse-engineer per-language).

**Stress-test use cases** (each exercises a different substrate slice):
1. **Performance bottleneck on realistic workflow** — complexity lens + realization cost + workflow execution. Closest to existing T-CostLens-Composition work.
2. **Refactoring impact** — "what breaks if I change X" via dep-graph traversal. Highest LLM-agent value; tests T-E-P descent-evidence completeness.
3. **Effect-shape query** — "what side effects does this code path produce." Tests effect_enum lens generality after Cluster F-β.
4. **Coverage gap** — "what code paths aren't covered by tests." Tests test-as-data + path-traversal substrate.
5. **Affected-set lens (fine-grained build system)** — query "I changed X; what Y is *actually* affected?" with structural strict-narrower-than-transitive-downstream semantics. Buck2/bazel-style fine-grained dep management falls out of pure-substrate for free. Pre-R3-close working prototype dispatched at [gunbc#2699](https://github.com/gunb-ai/gunbc/issues/2699); design doc + 5 worked examples at [`docs/design-affected-set-lens.md`](docs/design-affected-set-lens.md). Operator framing 2026-05-11: "I changed this code, so I should change Y code — i.e. not just downstream, but actually affected at an atomic level — since everything is pure, changing upstream usually doesn't matter too much; it usually changes interfaces or certain edge cases."

**Sequencing** (operator framing 2026-05-10; RESOLVED 2026-05-11): R4.A and R4.B both land in R4. Stress-test the use cases first; design follows requirements. ~~Core query concept may be expressible as `lens` with a different consumer disposition (rather than a new substrate type) — confirm via stress test before deciding.~~ **RESOLVED via affected-set stress test (2026-05-11)**: query IS lens (Introspect config); no new substrate carrier. See top-of-section LOCKED note + `docs/design-affected-set-lens.md` §0.

**Connection to R3**: a lot of substrate is being built under R3 lane names that turns out to be R4.B foundation (T-E-P descent evidence, T-Lens-Self-Application, T-CostLens-Composition reading realization cost). R4.B is partly a *lens over R3 work* asking "did we accidentally build the right substrate, or do we have gaps?"

### R4.C — Low-level emission targets

**Wish (operator 2026-05-12)**: extend the emission projection set to include **machine code**, **assembly**, and **LLVM IR text** as native emission targets — same Lens<C> shape as the existing Rust / Python / Go target codegen, just at lower levels in the toolchain stack.

**Orthogonal-to-LLVM framing** (explicit): gunbc is **NOT built on LLVM** and is **NOT competing with LLVM**. The substrate is gunbc-native; the projection to LLVM IR is an **interop output**, not an internal dependency. (The dropped `docs/r4-c-compiler-and-llvm-in-dag-program-plan.md` was about IMPORTING LLVM IR + C as substrate via bidirectional extdeps — multi-month effort, currently out of scope per operator 2026-05-12. R4.C is the much narrower wish: emit LLVM IR text as one more target alongside Rust / Python / Go, no ingestion / no LLVM-as-internal-engine claim.)

**Why these matter**:
- **Machine code / assembly** = lowest-friction emission for systems-software / firmware / embedded targets. Demonstrates the substrate is **target-class-neutral** — gunbc compiles down equally to managed runtimes AND bare metal, with all per-target knowledge in extdeps and no per-target leak in the substrate.
- **LLVM IR text** = interop with the entire LLVM-consuming ecosystem (object files, linking, optimization passes, JIT engines). Gives gunbc-authored programs the ability to be consumed by existing LLVM tooling without claiming to be inside LLVM. Same posture as "we emit Python" — the consumer is the existing Python interpreter ecosystem; we don't write a Python runtime.
- **Architectural falsifier**: same substrate, three more emission targets → demonstrates the projection-engine framing is structurally sound. If we have to add target-specific carriers to std/ to make machine-code emission work, that's a leak (emission-bias) that R3-close-audit should catch.

**Open questions**:
- **Target-realization cost composition**: how does machine-code emission compose with the cost lens? Realization cost should fall out of substrate composition (per T-CostLens work in R3) — same lens, different target. If it doesn't, that's a R3 cost-lens-generality gap.
- **ABI / linkage substrate**: machine code requires calling conventions, register allocation, debug info, relocations. Modeled in `dsl/extdeps/*` (preferred — per-target knowledge in extdeps, not std/) or does something need to live in std/?
- **LLVM IR fidelity**: which LLVM IR version, which subset, which dialect? Probably pin similar to existing extdeps version pinning (e.g., `dsl/extdeps/llvm/v20/ir/` shape, per the dropped C-compiler plan's promotion target — but without the ingest direction).
- **Slicing**: R4.C as a single program or per-target (machine + assembly + LLVM IR each their own slice)? Probably per-target since each has different extdeps weight (assembly = small ABI-only; machine code = full ABI + relocations; LLVM IR = LLVM dialect modeling).
- **Sequencing vs R4.A/B**: R4.C may need to wait until R4.A omni-ingestion settles (the emission-bias audit clears) so we know the substrate is consumer-neutral before adding 3 more emission consumers. Or it could go in parallel as a non-blocker since it's strictly more projections of the same substrate.

**Composes with R4.B**: once `.dag` substrate compiles to machine code, queries-as-data can include "what's the actual binary cost of this function?" — that's a composed lens over cost-of-machine-code-emission, which falls out structurally if R4.C lands. Bridging cost-lens to *realized cost on actual hardware* is a real-world thesis cash beyond the symbolic cost story.

---

## R5+ — speculative

Furthest-out wishes; placeholder until R3/R4 read clearer.

- **Real applications shipped on gunbc** — gunbc-authored production code, not just self-host. First-party use case (one of: ML pipeline / build system / web service / data tool). Demonstrates the architecture beyond compiler-as-the-only-program.
- **Deep IDE integration** — go-to-definition / find-references / refactor-impact-preview as queries against R4.B substrate. Should fall out of R4.B; R5 = the user-facing surface.
- **Self-applying compiler optimizations via LLM queries** — agent reads gunbc's own substrate, identifies bottlenecks, proposes restructuring. Compiler optimizing itself via the same query infrastructure available to users. (Highly speculative; depends on R4.B maturity + LLM-agent reliability.)
- **Cross-language refactoring** — "rename this concept everywhere it appears across my Python + Rust + Go codebase" via R4.A ingest + R4.B query + R3 emit. This is the long-tail story for what omni-architecture buys.

---

## Doc discipline (how this evolves)

- **Wishes flow through**: new wish → first appears as full description here → if picked up, shrinks to one-line pointer → if delivered, moves to retrospective for that milestone → eventually deletes (or stays as historical anchor for the milestone it closed).
- **No dual-rep**: if a wish has a `docs/briefs/*` or `docs/r3-program-plan.md` row, this doc carries the *aspiration* (one-line + pointer); detail lives there.
- **Retrospectives stay brief**: 3-5 lines per milestone + canonical receipt links. Anyone wanting detail reads `r2-closure-ledger.md` or equivalent.
- **Operator authority**: scratchpad-style edits welcome; this doc is allowed to be incomplete, speculative, contradictory across versions. The canonical docs hold the rigor; this one holds the direction.

---

**End of wishlist.**

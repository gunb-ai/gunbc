# v4 — Design Decisions Ledger

The single record of v4's design ratifications. Two states:

- **RATIFIED** — operator-decided and binding. Encoded in the cited authority
  location(s). For de-prosed `.dag` files, terse headers are boundary indexes;
  this ledger carries the ratified decision prose.
- **PROPOSED** — a recommended resolution awaiting your ratify-or-redirect.
  Nothing here is decided unilaterally. The recommendation is my read,
  with the tradeoff stated honestly so you can overrule it cheaply.

How to use this PR: ratify or redirect each PROPOSED item (review comment
or merge-with-notes). On ratification, each item is encoded into the cited
authority location(s) and moves to Part 1.

---

## Part 1 — RATIFIED (this session, 2026-05-15)

| ID | Decision | Encoded in |
|----|----------|-----------|
| **A1** | One recursive type (Node); 6 connectives + 5 behaviors are orthogonal flat-discriminant axes; behaviors structural not lens-derived; recursive generics are Node-underlying (Instantiation + name-ref); regular recursion admissible, non-regular polymorphic recursion a STOP absent a decidable bound | this row + `std/node.dag` declarations |
| **A2** | Termination is Tier-1; `Loop` = bounded recursion total-by-construction; descent evidence implicit-on-sub-Node or explicit ranking; checker not discoverer; the bound IS the cost-lens datum; residual boundary stated | this row + `std/node.dag`/`std/cardinality.dag` declarations |
| **A3** | Retirement = a reproduction predicate, not a count; it is an early-surfacing amplifier, NOT un-gameable (Trusting-Trust); seed trust is a named axiom; culture + operator-ratification is the actual enforcement | `STRUCTURE.md` §7, `workflow/bootstrap.dag`, `TASKS.md` T-5/T-15 |
| **A4** | Folded into A3 — a 7th connective is the same machine: deviation changes the reproduction → conspicuous signal → STOP. Not "the substrate refuses by construction" | `STRUCTURE.md` §7, `workflow/bootstrap.dag` |
| **U1** | One homomorphism-with-cost carrier; the back half is five phases (Find/Realize/Measure/Compare) not five engines; no-engine discipline (empty search → helpful Diagnostic, never fabrication). **U1's homomorphism-with-cost carrier IS the THESIS core-theme *derived homomorphism*** — the structure-preserving cross-target map the compiler derives, never an authored adapter (see `docs/thesis/the-derived-homomorphism.md`). | this row + `std/algebra.dag` declarations |
| **C2** | Synthesis = compare(derived cost, the relation's structurally-derived lower bound) via a closed `LowerBoundTechnique` set; NOT a (naive→better) rule library; honest worked examples preserved | `lens/synthesis.dag`, `TASKS.md` T-17 |
| **C7-REPORT** | `std/report.dag` owns the advisory, non-fail-closed carrier family for C7/T-17: `Report`, closed `ReportReason`, and typed `ReportSuggestion`. Reports are distinct from fail-closed `Diagnostic`/`Witness<C>` output; advisory lenses emit report sets directly, and the only advisory→blocking bridge is explicit user opt-in through `apply_lens(..., Enforce { ... })`. | this row + `std/report.dag` boundary index + `TASKS.md` T-17/T-23 |
| **U2** | `complexity.dag` *consumes* `cost.dag`'s `SymbolicCost`, never re-derives; **cost is TOTAL over the closed kernel** — no per-feature opt-in, so a future addition cannot bypass cost (the no-eternal-maintenance property) | `lens/cost.dag` + `lens/complexity.dag` headers |
| **C3** | `normalize` dissolves exactly the 4 bounded sugar forms (`service/fn/type/operation`) into Node; not identity, not open-ended; new sugar = STOP | `compiler/03_normalize.dag` + `TASKS.md` T-8 |
| **C1** | `infer` is the bounded *Find* phase — bounded structural unification over the finite (A1/A2) space; empty ⇒ Diagnostic, never fabricated coercion | this row; `compiler/04_infer.dag` is a boundary index only |
| **B3** | The type signature is the single effect authority; the effect lens reads it; coordination effect kind must be derived from the signature. Until the T-13 effect lens can read coordination effects from signatures, `extdeps/coordination.dag` may carry one explicitly yellow `CoordinationEffectKind` scaffold; effect kind must never be selected by a family of wrapper types or empty marker carriers. | `lens/effect.dag` + `extdeps/coordination.dag` |
| **C4** | On-disk emitted artifacts (`ci.yml`, trampoline) are committed==emit(source) checked projections, not editable authority (same machine as A3) | `workflow/ci.dag` + `STRUCTURE.md` + `workflow/bootstrap.dag` |
| **C5** | Ingest = emit⁻¹ on each model's lossless core; outside ⇒ fail-closed Diagnostic. Guaranteed ASAP via testgen `bidirectional_roundtrip` (Phase-1.5, gated per-PR via the B1 hash from first language-model commit) | `extdeps/languages/*` + `lens/testgen.dag` + C5 note |
| **C5-fidelity** | Round-trip fidelity is **parameterized by the model, not a fixed "trivia" category** (operator-corrected 2026-05-15). Each surface feature, per language, has an explicit disposition in the model: **Modeled** (∈ F → Node-bearing; both `ingest∘emit=id` and `emit∘ingest=id` for that feature — e.g. Python indentation IS block structure; comments IF modeled) \| **Declared-normalized** (deliberately not in F; `emit∘ingest` canonicalizes it — Go/C++ insignificant whitespace; a *declared* loss, reviewable, never silent) \| **Fail-closed** (encountered but neither → Diagnostic, no-engine). `emit∘ingest=id` holds restricted to F; "lossless core" ≡ F; round-trip fidelity = model completeness, boundary declared not assumed. F is drawn from the spec's own meaning-vs-lexical distinction (see L-2) | `extdeps/languages/*` headers + `TASKS.md` T-4 + C5 |
| **L-2** | **Model the language SPECIFICATION, not libraries** (operator-ratified 2026-05-15). `extdeps/languages/*` models the versioned upstream spec (grammar+type-system+semantics; the anchor IS that spec by construction — cf. the verilog→IEEE-1364 / spice→SPICE3f5 fix). Libraries (std/crates/packages) are NOT modeled — a library is just a program in the modeled language = `Node` like any other. This makes F principled (the spec defines meaning-vs-lexical, not our judgment) and maximally general. **Frameworks rule (BINDING — cascade-closure of L-2, resolves 3250029061; no PROPOSED residue):** a `frameworks/*` model is admitted IFF the framework constitutes a sub-language with its own non-host-expressible rules (React Rules-of-Hooks / effect-lifecycle / reconciliation) — that IS a "language specification" under L-2, so L-2 *already* decides this; a framework that is only a library API surface is a library, which L-2 already says is NOT modeled (it is just `Node`). Not a new fork — the logical closure of the ratified L-2 principle; T-4.7 has binding spec-vs-library authority | `extdeps/languages/*` + `extdeps/frameworks/*` headers + `TASKS.md` T-4 |
| **K-1** | Kernel identifiers are OPAQUE `Symbol`s (equality only, no content accessor); spelling lives in a boundary table, never kernel/lens-readable — structurally forecloses the v2 string-heuristic channel | this row + `std/node.dag` declarations |
| **IR-1** | `InferredTree` is NOT a new type — it is `Node` + a FROZEN flat `InferredFacts` coordinate (resolved_type, inhabits, cost, descent); **cardinality and effects are NOT fields** — both already carried by `resolved_type`'s Node structure (the Cardinality connective / the signature), caching = second authority P2; a 5th field is a STOP; modeled up front, not emergent | this row; `compiler/04_infer.dag` is a boundary index only |
| **B2-OMNI** | parse is one ingestion instance; `ingest`/`emit` are parameterized boundaries over declarative LanguageModels; Node is the universal pivot ⇒ O(N+M) not O(N×M); `.dag` is language #1, never hardcoded | this row; `compiler/00_compile.dag`/`01_tokenize.dag`/`02_parse.dag`/`05_emit.dag` are boundary indexes only |
| **L-1** | Verilog/SPICE/English added as **parallel B2-OMNI falsification probes** (T-4.9/4.10/4.11), maximally diverse on purpose. Forks resolved: English = boundary-honesty probe (Shape B emit + fail-closed ingest TestClaim), **not** a language model; SPICE = `extdeps/formats/` (netlist is a data format); Verilog = the IN-B concurrency validation (a needed 6th behavior = C1 escalation, by design caught early) | `TASKS.md` T-4.9/4.10/4.11 + `extdeps/languages/verilog.dag`, `extdeps/formats/spice.dag`, `test/claim/boundary/english_ingest_fail_closed.dag` |
| **L-3** | **Down-the-stack** B2-OMNI probes added (operator-ratified 2026-05-15, parallel up front): T-4.12 `extdeps/languages/llvm_ir.dag` (SSA IR — generalize down the stack; F ~all-structural), T-4.13 `extdeps/languages/machine_code.dag` (bottom of stack; no cosmetics; disassembly = the extreme fail-closed/no-engine test), T-4.14 `extdeps/languages/ptx.dag` (CUDA SIMT — IN-B bet vs the 5 behaviors). Validates the model across the **full target spectrum** (source→IR→machine code) — concretizes C5-fidelity ("we don't necessarily output a binary; F is model+intent-relative"). **Forks DISSOLVED (cascade-closure, resolves 3250390911; no PROPOSED residue):** neither was a genuine open fork — each is forced by an already-ratified parent, and the alternative *contradicts* a ratified decision. (1) machine_code = ONE model parameterized by an `Isa` — **forced by B2-OMNI** (parameterize-not-enumerate / O(N+M); per-ISA files ARE the N×M trap B2-OMNI forbids). (2) CUDA = model **PTX** — **forced by L-3's own ratified down-the-stack-IR scope**: PTX is the SIMT *IR* layer (the down-the-stack probe T-4.14 exists to test); CUDA-C++ is just another Shape A *source* language already covered by general modeling, so it would not be a down-the-stack probe at all. Per P5 the fork dissolves into already-ratified coordinates, not a new decision | `TASKS.md` T-4.12/4.13/4.14 + `extdeps/languages/{llvm_ir,machine_code,ptx}.dag` |
| **L-4** | **PROOF-1's prover model lands as `extdeps/languages/lean.dag`** (B-2, side-session 2026-05-15 — operator-ratified). PROOF-1 (#3158, merged) was framed "no new file" and explicitly deferred its prover model to "realized when a lean/coq model lands"; this IS that model. B2-OMNI **structurally requires** a declarative `LanguageModel` file to emit to any target, so PROOF-1's "emit to a lean model" needs exactly this file — adding it is the closed-tree extension operator-ratified here (STRUCTURE.md invariant #1). **ONE prover first — Lean** — scoped to the first theorem class **termination** (A2 structural descent ≈ Lean well-founded recursion / `termination_by`). **Coq is the DEFERRED second-prover falsification probe** ("does the model generalize across provers?" — O(1) per B2-OMNI, added only when a theorem class needs it; same probe posture as L-1/L-3; a future operator-ratified extension, never a silent TODO). Per L-2 the Lean *spec* is modeled, not Mathlib. STRUCTURE count 63→64 .dag | `extdeps/languages/lean.dag` (new) + `STRUCTURE.md` languages enumeration/count + `TASKS.md` PROOF-1/T-15 note; cross-ref DECISIONS PROOF-1 / B2-OMNI / L-2 |
| **L-5** | **`extdeps/languages/dag.dag` admitted — gunbc's native `.dag` LanguageModel** (operator **Option A**, C1 closed-tree extension 2026-05-16, relay merry-ibex-337). `.dag` is B2-OMNI **language #1** (never hardcoded into the generic `01_tokenize` / `02_parse` walkers); lexical + syntactic authority is declarative **`Node` grammar-data** here, same `extdeps/languages/*` bundle shape as the other Shape-A models. L-2 **spec-first** without an external URL: anchor is repo-owned **`docs/v3-spec.md`** plus the v4 closed tree (`STRUCTURE.md`, `TASKS.md` T-6/T-7). PARSE-1 bodiless-`fn` is a **grammar production in data**, not walker code. STRUCTURE count **64→65** `.dag`. | `extdeps/languages/dag.dag` (new) + `STRUCTURE.md` languages line + total; cross-ref `compiler/01_tokenize.dag` / `compiler/02_parse.dag` + B2-OMNI + PARSE-1 + **DECISIONS §CP-1b** |
| **V-HDR** | **Lane B strict `.dag` header discipline (interim to modeling-discipline.md #3226, operator 2026-05-17).** In-file comments on Lane-B PR `.dag` files are limited to line-1 path + four one-line fields: `Scope` / `Owns` (carrier names only) / `Consumes` / `Status` — no `HEADER RECONCILE` blocks, no multi-line Practice-4 ledgers, no Brief/Anchor essays in those files. **Measured target:** `//` lines stay under 20% of file lines per primary file. **Rationale relocation (D5 intent preserved):** operator decision I (`TestClaim.input` / `expected` as `Node` / `Outcome<Node>`) and api-review #3183 `kind`×`expected` pairing tension — narrative receipts belong in **git commit messages** for this PR, not in `std/verification.dag` headers. **Nat manual `TestClaim.classification`:** stub-era claims use **Tier1+Unit** when `kind`/`input`/`expected` match structurally (STRUCTURE §248 — tier×layer is not label-derived); `claim_nat_mul_associativity` shares the same stub surface as `claim_conj_equals_refl`, so it carries the same classification until T-22 supplies law-bearing `Node` shape. Do not stamp the `Compiles` stubs as L7 integration. **T-19:** `lens/testgen.dag` materializes `TypeConstructionSubject` / `TestgenConcept` / subject carriers as real types (M1); C5-1/C5-3 round-trip prose stays in **C5** / **C5-fidelity** / **C5-3** rows here (not duplicated in testgen comments). | `std/verification.dag`, `lens/testgen.dag`, `lens/coverage.dag`, `test/claim/manual/{connective_anchors,nat_law_anchors,t19_manual_anchor_manifest}.dag` + this row; cross-ref **D5** **I** **C5-1** **C5-3** |
| **LB-P4-3212** | **Lane B coproduct emoji ledger (PR #3212, swift-ram-178; modeling-discipline.md via PR #3234 — Practice 4, Practice 9 item 4).** Each touched N≥2 sum in Lane-B scope carries exactly one in-file line immediately under the `type` name: `//` + one of 🟢 🟡 🔴 + ` coproduct dissolution — DECISIONS.md LB-P4-3212` (no narrative blocks). **🟢 `AssertKind`** — four assert kinds; dissolution = single `Outcome<Node>` pairing authority (`std/diagnostic.dag` **I** / THESIS). **🟢 `TestgenTier` / `TestgenLayer`** — closed 3×3 grid; dissolution = `TestClaim.classification` sole authority (TASKS T-19 P2). **🟢 `ImpossibleBugClass`** — six THESIS impossible-bug demos only; dissolution = closed coverage keys, no string extension. **🟢 `TypeConstructionSubject`** — `ConnectiveKernel { connective: Connective }` vs `BehaviorValueSubject { behavior: Behavior }`; connective **and** behavior substrate axes flow structurally (TASKS T-19 bootstrap: six connectives + five behaviors); still **no** live manual value-behavior `TestClaim` until T-22 slice (TASKS §T-19 Phase-1.5). **🟡 `T19ManualAnchorKey`** — twelve live anchors + absent; Phase-1.5 P2 join (`TestClaim.t19_anchor` ↔ `t19_manual_anchor_manifest.dag` rows; TASKS §T-19 Phase-1.5); **M2 dissolution** = cross-file load + literal validation → fold manifest membership into reflection / `Symbol` spellings (TASKS §T-19 retirement). **🟡 `TestgenConcept`** — five scheduling arms; dissolution = **`T19ManualAnchorKey`** tag-equality (`TestClaim.t19_anchor` ↔ manifest membership; TASKS §T-19 P2) + variant-prefix routing interim, M2 single discriminant or reflection (TASKS §T-19). | `std/verification.dag`, `lens/testgen.dag` tags + this row; cross-ref **V-HDR** **#3234** |
| **QRY-1** | **RETRACTED & reframed (operator-corrected 2026-05-15).** "query = user-defined lens" is wrong: user-vs-kernel is PROVENANCE, not a structural distinction (feedback_naming_is_aliasing) — a decorative name distinguished only by who-authored-it must not exist ("a real distinction, or not exist"). There is NO query concept/noun/subsystem. The REAL axis is **scope/parameterization**, already modeled with no new noun: (1) `SectionRef` (DeclarationScope\|NodeScope) — applying a lens AT a scope; (2) parameterized lens signatures — the **affected-set lens (T-21) `(Dag,Diff)->Witness<ReExecFrontier>` is the archetype**: "the affected set of THIS change" = `affected_set(dag,diff)`, already first-class. Lenses, read at scopes and/or parameterized. No `query.dag`. The B2-OMNI "one read over the Node pivot, all ingested languages" property holds for ANY lens, not a "query" | `lens/application.dag` header + THESIS §1.5 + `lens/affected_set.dag` (T-21) |
| **PAR-1** | Parallelism (T-13) is a THESIS free consequence, a target-AGNOSTIC lens reporting the independence **FACT** (`Witness<ParallelismMap>`); WHETHER/HOW to realize it is target-DEPENDENT — a cost-derived decision (U1/U2 per-target) at emit(T-11)/eval(T-22), NOT the lens. independence = structural-disjointness ∧ effect-independence (consumes `lens/effect.dag`, B3 — Node-disjoint is necessary not sufficient) ∧ fold-combiner associativity-by-inhabitance (consumes `std/algebra.dag` — parallel-reduce is an algebra fact, not a heuristic). fact ≠ spawn-decision; no annotation, no scheduling heuristic. **User-facing contract:** NOT a "parallelize my code" command (forbidden — engine-shaped from the user's seat); a lens the user READS (same UX as complexity/cost); control = structure not a knob; default introspective with the realize-decision derived+inspectable per target (e.g. honestly "no — CPython GIL"); guarantee via `apply_lens(parallelism, Enforce)` (T-23, uniform opt-in-depth). 4 worked examples embedded | `lens/parallelism.dag` header + `TASKS.md` T-13 |
| **B1** | One content-addressing scheme: a `std`-level Merkle catamorphism `content_hash : Node -> Hash` over the **canonical-form clause** (a SEPARATE explicit node.dag authority — NOT subsumed by A1, which only covers recursion/axis-closure/generics/termination). `Hash` is an opaque digest (no content accessor, K-1-style). One scheme consumed by T-15/T-20/T-21 + A3 + C5 + structural build/CI artifact caching (a cache key IS `content_hash` of the input subgraph — see `lens/affected_set.dag`). NOT workflow-level | this row + `std/node.dag` `Hash` declaration; the canonical-form clause remains ledger authority until executable declarations land |
| **PARSE-1** | Bodiless `fn` (signature-only / contract-as-checked-signature) is **not needed in the v2-bridge era** (operator-ratified 2026-05-15) — verified by constructive pre-flight: frozen v2's parser **rejects** bodiless `fn` (`expected LBrace`) but cleanly parses the full ratified kernel shape **with bodies** (40k-line `src/v2` corpus + kernel probe, zero syntax errors; the 65 src/v2 errors are 100% post-parse import/cycle). So bridge-era substrate contracts stay **comment-prose** (current scaffold form) and the fail-closed parse-class gate is SAFE — v2 is never asked to parse bodiless fn. **Forward requirement (the conditional that makes the deferral acceptable):** v4's OWN `.dag` grammar (T-7) MUST admit a bodiless signature declaration as a first-class production — graduating the "immutable I/O contract for the worker" from comment-prose to **compiler-enforced** (filled body must conform to the frozen signature; divergence = fail-closed type error, structural per no-annotations). Supportable by construction (v4's own grammar; nothing external constrains it); per B2-OMNI it is a grammar-DATA production, not hardcoded in the walker. v4-self-parser-era only — never through the v2 bridge | this row + T-20 pre-flight note; `compiler/02_parse.dag` is a boundary index only |
| **D1** | The structural-delta vocabulary over `Node` (side-session 2026-05-15; **shape refined** per codex P2 REQUEST_CHANGES on PR #3162). An `Edit` replaces the subtree at a `Path` with a `Node` (term-rewriting `t[s]p`). `Path = List<Symbol>`: each step descends the `Named` edge keyed by that opaque `Symbol` — flat & finite, NOT a new recursive type (A1); resolution = `Symbol` equality ⇒ K-1-clean. Name is **`Path`** (the route), not `Position` (operator pick); "position/occurrence" (JSON Pointer, RFC 6901, member-name tokens only) retained as anchor. **No positional/ordinal selector** (P2 finding 1): an `Int` ordinal admits negative/illegal indices = the representable-illegal-state node.dag designed out at `EdgeLabel`; positional-connective children are reached by replacing the enclosing `Named` subtree (the safe over-fire #3157 blesses); surgical positional addressing is a future operator-ratified extension (consumes-nothing root has no ordinal primitive — no kernel `Nat`; recursive Peano = A1 STOP). **`Diff` = ORDERED `List<Edit>` applied SEQUENTIALLY** (P2 finding 2): no set / pairwise-independence / "parallel positions" invariant, no `diff_well_formed`/`paths_independent` — any `List<Edit>` is a valid rewrite program; a non-resolving Edit ⇒ fail-closed Diagnostic at apply time (operational, not a type-level illegal state); `apply_diff` is a fold, all-or-nothing. **DATA TYPES in `std/node.dag`** (B1/`Hash` precedent: a structural delta over `Node` is substrate-tier; corrects the initial "owner = T-21" read). **OPERATIONS** (`subterm_at`/`apply_diff` → `Result<Node, Diagnostic>`) in **`lens/application.dag`**, FORCED: `std/diagnostic.dag` already `Consumes std/node.dag` (`Correction = Suggested(Node)`) so a node.dag op returning `Result<_,Diagnostic>` is a substrate cycle — the node.dag "consumers fail-close" discipline (`node_well_formed` precedent). `apply_diff` consumer = CLI / test harness / user app code. Two refinements of the in-session ratification were encoded via PR #3162 review (the normal confirm/redirect channel — not an open fork): (a) the data/op split (operations in `application.dag`, forced by the diagnostic→node cycle); (b) the P2-driven shape change — no positional selector (Practice-4 dissolved; receipt in `std/node.dag`) and sequential `Diff` (no independence predicate). Operator redirect, if any, is via the PR like any change | this row + `std/node.dag` Path/Edit/Diff declarations + `lens/application.dag` operation contract + `lens/affected_set.dag` cross-ref |
| **B-4** | affected_set R2 minimality contract (side-session 2026-05-15). For `lens/affected_set.dag` (T-21) to compute a *minimal* re-exec frontier, `compiler/04_infer.dag` resolution must be **NODE-PRECISE**: a use-site name-reference `Atom` binds to the **exact declaration `Node`**, never merely its module/scope (**carrier corrected** per a P2/Practice-3 review finding on #3162). The use→def fact's named forward carrier is the post-resolution opaque **`Symbol`** (K-1) on the name-ref `Atom`, **NOT `resolved_type`** (that is the node's *type* — many declarations share a type, so it is lossy for binding identity; citing it was the P2/Practice-3 error). AUTHORITY = `compiler/03_resolve.dag` (it owns identifier→declaration binding; resolution canonicalizes a use-site `Atom`'s `Symbol` to equal its binder's `Symbol`, so binding = `Symbol` equality — the K-1-granted op, node.dag `name_occurrences`). `04_infer` is **preserve-only** (must not alter/drop the resolved `Symbol`; no 5th `InferredFacts` field — IR-1-safe). `lens/affected_set.dag` **derives** the graph by `Symbol`-equality traversal over the resolved tree (same "derive, don't store" as `content_hash`/D1). Coarse (module/scope) resolution leaves affected_set *correct but maximally over-firing* — so node-precision is a **precision INVARIANT on T-9**, the dominant gate on R2 minimality, not a T-21 concern. Distinct axis from #3157's hash-granularity "over-fire = safe" tolerance (content identity vs dependency-edge granularity). No new type/field; IR-1-safe | this row + `compiler/03_resolve.dag`; `compiler/04_infer.dag` is a boundary index only |
| **B-5** | affected_set R1 usefulness contract (side-session 2026-05-15). For purity-aware skipping to be *useful* (not merely correct), effect must be a **per-`Arrow`-Node fact** derived from that node's signature (the B3 authority) and **never aggregated to a coarser unit** (one ambient `IO`, per-module effect set) — a coarse aggregate is the parallel taxonomy B3 forbids (P2) *and* degrades R1 to "re-run ~everything" (still correct/fail-closed, useless). A **granularity invariant**, not effect-lens strength (effects are READ off the signature per B3 ⇒ strong by construction; the only risk is coarse modeling — this forbids collapsing the per-effect-KIND distinction the signature already carries). No new field. = #3153 Ex.R1, now encoded. **Corollary (effects-as-parameters, 2026-05-16):** effects are threaded as explicit parameters (DI-by-construction); a mock-injected unit test's B-4 edge points at the mock `Node`, so affected_set skips it and re-runs only real-carrier (integration) tests *automatically* — "affected" is consumer-relative with no unit/integration mode in the lens. Ambient/global effects re-couple the cone = the anti-pattern this forbids. Honest boundary: a re-run test with changed behavior may have a stale expected value; the lens surfaces "re-run", never rewrites the expectation (no-engine) | `lens/effect.dag` B-5 block (+corollary) + `docs/design-affected-set-lens.md` worked examples |
| **#3153-subsumed** | `v4-affected-set-design` (#3153, no worker) is **superseded by this branch** (operator 2026-05-15): `docs/design-affected-set-lens.md` remains the prose design authority for the affected-set lens (design framing + 5 worked examples + dimension-aware/fail-closed exclusion discipline); `lens/affected_set.dag` owns only the terse machine-readable T-21 carrier/read surface under the strict de-prose rule. The branch-specific Ex.R1/Ex.R2/Ex.R3 labels are not a second prose authority; they are reconciled here to the now-*encoded* reciprocal contracts — Ex.R1→B-5 (`effect.dag`), Ex.R2→B-4 (`04_infer.dag`), Ex.R3→B-3 (the drafted #3157 B1-CANON point-3 correction). Dogfood-reality is tracked by the T-21 scaffold trigger rather than copied into the `.dag` file. IRT reconciliation (operator 2026-05-17): the eventual `AffectedSetRead.read` body must prune at unchanged/pure `content_hash` boundaries, expand the frontier as a pure parallel fold over the DAG, and make TestClaim result reuse the cache-dual keyed by the input subgraph `content_hash`; a whole-program traversal scaffold is rejected. Practice-4 coproduct classification: `ExclusionProof` is 🟡 scaffold; named trigger = promote or dissolve when the T-21 read body and `.dag` TestClaims prove the exclusion evidence cases. `FrontierDecision` is 🟢 terminal: it is the per-`ExclusionKey` outcome coproduct that makes rerun-vs-excluded mutually exclusive while leaving node identity solely on the key path resolved against the input `Dag`. #3153 carried no new file/type; close it as superseded | `docs/design-affected-set-lens.md` + `lens/affected_set.dag` carrier surface |
| **PROOF-1** | **External trust-discharge for A3** (operator-ratified 2026-05-15; *not the north star — placed & designed early like AGENT-1*). gunbc's structural evidence (A2 termination descent, the algebra-homomorphism epistemic chain, cost, effects) is EMITTED as a machine-checkable proof term in an external prover (Lean/Coq); their small independently-audited KERNEL checks it. Converts A3's named seed-trust axiom into an INTERSUBJECTIVE check against an anchor gunbc does not own. Does NOT make gunbc un-gameable — the external kernel + the evidence→proof-term faithfulness are the NEW named axioms; it MOVES trust to a stronger anchor (same Trusting-Trust honesty A3 observes). Constraints: the lens EXPORTS witnesses gunbc already has, the prover only KERNEL-CHECKS, never searches (no-engine + A2 "checker not discoverer"); discharges only what is structurally GROUNDED (ungroundable ⇒ fail-closed Diagnostic — inherits gunbc's honesty boundary). Realized as (evidence read) ⊕ (B2-OMNI emit to a `lean`/`coq` language model — ordinary targets). The PROOF-1 *framing* (#3158) added **NO new file** — it was framing-only and explicitly DEFERRED the prover model ("realized when a lean/coq model lands"); that model **subsequently lands as `extdeps/languages/lean.dag` per B-2/L-4** (operator-ratified closed-tree extension — see the **L-4** row). No contradiction: framing = no file; the deferred model's later arrival is the L-4 extension | `STRUCTURE.md` §7 (A3 home) + `workflow/bootstrap.dag` A3 block + `TASKS.md` T-15 note; cross-ref A2 / AGENT-1 / B2-OMNI / **L-4** |
| **C5-2** | **emit-locality** (operator-ratified 2026-05-16) — `emit` is *structurally local*: a subtree's emitted form depends only on that subtree + a bounded declared context, so a D1 `apply_diff` at Path p re-emits ONLY p's subtree; everything outside p is unchanged by construction. **Locality is Node-relative, NOT file-relative** (operator code-as-data steer: the substrate anchors no semantic on the file concept — a "file" is a convenience projection only; stability = content_hash of the emitted *subtree*, never file bytes). Non-local emit decisions are DECLARED normalizations at the smallest *enclosing Node scope* (never "file scope" — there is no file in the model). Load-bearing: AGENT-1/D1 "faithful re-emit" is unprovable from global `ingest∘emit=id_Node` (C5-1) alone — locality is the stronger separate property; the C5↔D1 hinge | this row; `compiler/05_emit.dag` is a boundary index only; cross-ref C5 / C5-fidelity / D1 |
| **T-9** | **infer keystone contracts** (operator-ratified 2026-05-16). **A — Find decidable by construction** (closes C1): the homomorphism Find recurses structurally on Node (A2 implicit descent ⇒ the inferencer terminates); the per-node candidate set is EXACTLY the closed declared inhabitances in `std/*` (enumerable, never synthesized/open), so "no homomorphism" = "exhausted the finite declared set" = a *decidable* fail-closed Diagnostic, not a non-terminating search. Synthesizing candidates = left the decidable fragment = STOP. **B — single-file discipline is a scope boundary**: v2's 12-file split was env/scope/lookup/method-resolution tangling; in v4 those are 03_resolve's job (B-4: binding = canonical `Symbol`), so `04_infer` owns ONLY {C1 Find + cardinality propagation + diagnostic precision}. A worker writing scope/env/binding/lookup here re-imported the 12-file pressure → STOP (belongs in 03_resolve). The single file holds because the scope is bounded, not willed | this row; `compiler/04_infer.dag` is a boundary index only; cross-ref C1 / IR-1 / A1 / A2 / B-4 |
| **C5-1** | **The C5 round-trip law is node-level** (operator-ratified 2026-05-16). A `bidirectional_roundtrip` TestClaim asserts `ingest∘emit = id_Node` via B1 `content_hash(N) == content_hash(ingest(emit(N)))` — the *meaning* round-trips; it does NOT assert text-byte identity (`emit∘ingest = id_text`). Lexical skin (`0x1F`/`31`, whitespace, comments) is allowed to canonicalize. **Text-level fidelity is a DEFERRED, ORTHOGONAL future layer**: a concrete-syntax/trivia sidecar *outside* the Node and *outside* `content_hash`, partial (ingest-originated Nodes only) — it never changes C5-1 or `content_hash`. The C5-fidelity Declared-normalized per-feature dispositions are its forward hook; adopting node-level + the disposition table preserves text-level as a zero-rework future add. The non-orthogonal route (lexical choices stored in the Node) is rejected (breaks α-/rename-invariance, dual authority) | DECISIONS **C5** / **C5-fidelity** / **C5-3** (round-trip **law prose**; **V-HDR**) + `lens/testgen.dag` (`TestgenConcept` **`BidirectionalRoundtrip`** typed substrate; prior Lane‑B header essay **retired**) |
| **C5-3** | **F = the spec-provable meaning core; spec silence ⇒ out of F** (operator-ratified 2026-05-16; conservative/fail-safe). If the language spec does not *unambiguously* place a feature on the meaning side, it is NOT in F — Declared-normalized (emit canonicalizes; recorded per-feature) or Fail-closed (ingest needs semantics it lacks ⇒ Diagnostic, never a guess). The **spec decides; the modeler has no discretion**. Consequence: a language's lossless core may be strictly smaller than its spec (e.g. C++ most-vexing-parse → out, Fail-closed), declared per-feature **up front** in the C5-fidelity disposition table, never discovered at round-trip-test time. Sharpens the pre-existing C5 / C5-fidelity rows | DECISIONS **C5** / **C5-fidelity** / **C5-3** (F-core **prose**; **V-HDR**) + `extdeps/languages/*` (C5-fidelity dispositions) + `lens/testgen.dag` (typed **C5** scheduling substrate only; same header discipline as **C5-1** row) |
| **AGENT-1** | The **non-text agent surface** (operator-ratified 2026-05-15 — *include & design early; explicitly NOT the north star*; "a very important/exclusive feature" but a NATURAL CONSEQUENCE, not a new subsystem). A non-text client (LLM/agent) interacts with the program **as data, never files**: READ = `apply_lens` at `SectionRef` + `affected_set(dag,diff)` (the affected lens replaces file-reading exploration); WRITE = the **D1** `apply_diff:(Node,Diff)->Result<Node,Diagnostic>` (the ratified D1 operation — NOT a total `->Node`; an agent-supplied Diff is external/possibly-stale input ⇒ FAIL-CLOSED typed Diagnostic, all-or-nothing, INVARIANTS P3 — AGENT-1 uses D1's op, does not restate a total one) entering the B2-OMNI pipeline at **`core`** (a SIBLING of `ingest`, NOT an ingest — the change is already Node: no source text, no LanguageModel, no parse), content-addressed (B1), not a text patch; BACK = the `affected_set` lens read in whatever shape T-21/`lens/affected_set.dag` DECLARES — AGENT-1 is a CLIENT, defers to that single authority (P2) and coins NO return noun (`Witness<ReExecFrontier>` was an ungrounded AGENT-1 noun, P1/P2 — removed) + a faithful re-emit (C5-1 node-level hash-checked + C5-2 emit-locality + C4 committed==emit); GATE = `apply_lens(_,Enforce)` fail-closed (agent earns no special trust). It is a **CLIENT** of application.dag + affected_set + B2-OMNI + C5/C4 — NO new authority/noun/query subsystem (QRY-1 + feedback_closed_system_design). Owned by T-23 so it is a contract, not an accident; realized only when T-23 composes T-21 + B2-OMNI + C5/C4 (scaffold today — nothing runs yet) | `lens/application.dag` header (AGENT-SURFACE block) + this row + `TASKS.md` T-23 note; `compiler/00_compile.dag` is a boundary index only |
| **I** | **Fail-closed success/failure beside `Diagnostic` (M6 / P2).** v4's **single** substrate coproduct for "produced value vs fail-closed reject carrying gunbc's `Diagnostic`" is **`Outcome<ok>`** with variants `Produced { value: ok }` and `Rejected { diagnostic: Diagnostic }` in `std/diagnostic.dag` (operator-ratified 2026-05-16). `TASKS.md` / `STRUCTURE.md` **`Result<…, Diagnostic>` prose** names the **same semantic surface** — it **denotes `Outcome<…>`**, not a second parallel `.dag` `Result<ok, err>` type. v4 **does not** declare a generic `Result<ok, err>` in `std/` alongside `Outcome`; introducing one would duplicate M6's authority unless this ledger is amended. Variant/field names are **binding** for downstream mirrors. A future *different* error payload (non-`Diagnostic`) is a new substrate decision (STOP), not an unratified shadow carrier. | `std/diagnostic.dag` (`Outcome<T>`) + this row; `TASKS.md` / `STRUCTURE.md` (prose only) |
| **D2** | **[SUPERSEDED by D2-REV — see the D2-REV row below + the "D2 REVERSAL" section]** **Primitive-inhabitance FORM — the `extdeps` resolver shape** (operator-ratified 2026-05-16; resolves the loyal-koi-702 STOP-2/STOP-3 escalated via the T-4 mgr). A language/format `extdeps` file is a **resolver** for the abstract `std/` carriers — the `infra(abstract)→gcloud/aws(concrete)` pattern (operator-cited prior art: gunb.ai `tools/extdeps/cloud_extdeps`, where the abstract `InfraScopeType` is declared ONCE and each provider supplies a translation, never a re-declaration). **D2a — three thin per-primitive declarations, NO re-declared algebra inhabitance:** (1) **alias-identity** to the `std/` carrier (`type RustI32 = Int32`) — the machine-readable-inhabitance form (`List<T> = FreeMonoid<T>` precedent); the algebra inhabitance flows through the alias, so "see through the labels" is literal `Node` identity; (2) a **grounding map** (surface spelling — `i32`); (3) **operation-semantics carriers** (`OverflowDisposition`, NaN/Inf handling) — genuine per-language facts, siblings to the alias. A per-language `data <lang>_<prim>: OrderedRing<<lang>Prim,…>` inhabitance is the parallel `OrderedRing<Word*>` substrate **INVARIANTS P1:42 forbids** (`numeric_aliases_align_to_refinements`) — superseded; #3174's `RustScalar` Disj registry + `IrToRust<>` carrier dissolve into this shape. **D2b — fn-value form:** the integer chain is fully CONSTRUCTIVE (Peano `Nat` → `GroupCompletion` → `AbelianGroup` → `Compose<MachineWidth>`) — its operation `Arrow` bodies are sub-DAGs of L1 behaviors (THESIS:203, the user-function case), so D2b's primitive question never fires for `integer.dag`. A genuinely-irreducible spec primitive (IEEE-754 float arithmetic) flows through the SAME single external-primitive path THESIS:203 already defines — `Transform → FunctionRef → Arrow`, where the `Arrow`'s `body` is *a realization declaration in `extdeps/`* (THESIS:203's primitive case). D2b coins **NO new carrier**: no `Symbol`+`Anchor` data node referenced directly by the inhabitance — that would be the second external-primitive path P2 forbids. The `ApproximateField` inhabitance holds `FunctionRef`s to the op `Arrow`s; for an IEEE-754 primitive the `Arrow`'s `body` realization (anchored to the spec §) is the per-target `extdeps/languages/*` resolver (the D2.2 resolver), not a std-homed carrier. The `Arrow` HAS a body (the `extdeps/` realization), so it is not a bodiless `fn` and PARSE-1 is untriggered; NOT a kernel/A1 extension (no new connective/behavior). **D2.2 — coercion** = the U1 homomorphism realized by the generic B2-OMNI emit fold reading the resolver data; NOT a hand table (the `IrToRust<>` per-target string table INVARIANTS P1 forbids), NOT a per-language emit lens (B2-OMNI/B4: one generic emit, target-specificity is declarative T-11 data). **D2.3** — compound/generic coercion composes for FREE via the `Node` catamorphism (A1's single recursive type; `List<Int32>` = `Instantiation` recursed); no per-instantiation rows, not blocked on the Functor hierarchy (a cost/algebra-lens concern). **D2.4** — the round-trip guarantee is structural-on-F per **C5-1**; TestClaims document it, never establish it. **D2.5** — overflow/UB/arbitrary-precision are DISTINCT `std/` carriers selected by the alias target + the per-language `OverflowDisposition` carrier, never a re-declared semantics. **Scope — D2 covers per-primitive inhabitance form ONLY:** it does NOT decide LanguageModel-grammar facts (C++ most-vexing-parse, TypeScript structural typing, etc.) — those are owned by C5-3 / B2-OMNI / B4 per their own cascade; a grammar-slice worker must not import D2 reasoning into them. | superseded by D2-REV; historical authority was this row + `TASKS.md` T-4 brief amendment + `extdeps/languages/*` headers; `std/integer.dag`/`std/float.dag` are boundary indexes only |
| **D3** | **ApproximateField<Float> machine-readable witness — encoding deferred (INVARIANTS P1/P3)** (operator spine 2026-05-16 for float algebra **shape** via **D2b** + Tier-2 **D2** intrinsic `Symbol` nan identities; **amended** after codex REQUEST_CHANGES on **PR #3191**). `std/float.dag` **does not** publish `data float_approximate_field: ApproximateField<Float>` while v2-bridge constructive `fn` bodies at this site would fabricate IEEE finite `+`/`*` or binary64 `one` (left-operand hooks or wrong-bias `Succ` on the Peano exponent carrier = known-wrong machine-readable authority — INVARIANTS P1 modeling faithfulness + P3 fail-closed honesty). Re-introduce the data row when **T-4 `extdeps/languages/*`** supplies grounded THESIS:203 **Arrow** bodies **and/or** v4 admits bodiless primitive signatures (**PARSE-1**). **Does ship:** `Float` carrier + Tier-2 `Outcome` ordered compare + intrinsic `data <id>: Symbol = <id>` for nan rejection (cross-ref **D2**, **I**, **PARSE-1**). | `std/float.dag` (carrier + Tier-2 compare + `Symbol` data; **no** `data float_approximate_field` until grounded) + this row |
| **D4** | **[SUPERSEDED by D2-REV — see the D2-REV row below + the "D2 REVERSAL" section]** **GroundingMap-home — the shared `extdeps/languages/resolver.dag` registry** (operator-ratified 2026-05-16; co-drafted; completes **D2**). The D2 resolver's *shared* types live in ONE new file `extdeps/languages/resolver.dag` — NOT re-declared per language (parallel-declaration P2 forbids; `feedback_import_not_redeclare_carriers`) and NOT homed in `rust.dag` (which would make one language file a substrate authority for the other four, and balloon as future shared resolver types land). `type GroundingMap<IRCarrier> { spelling: String }` is its first resident. **Admission rule (STRICT):** `resolver.dag` hosts ONLY *structurally-language-INVARIANT* resolver types — a type whose structure provably cannot vary across the modeled language specs (a convergence carrier). Spec-varying types stay per-language (`OverflowDisposition` etc. — D2.5 / L-2). Test before admitting: can you prove the structure invariant across the specs? Future shared resolver types (e.g. the C5-fidelity `Modeled \| Declared-normalized \| Fail-closed` disposition classifier) land here without a fresh placement decision. Closed-tree extension: `STRUCTURE.md` `.dag` count +1. Per-language files each add a one-line `import … resolver { GroundingMap }` + their per-primitive `data <lang>_<prim>_grounding` instances (the grounding-row fan-out — canonical `rust.dag` first, then the 4 others). | new `extdeps/languages/resolver.dag` + `STRUCTURE.md` count/enumeration + `extdeps/languages/*` import lines + grounding instances; this row; cross-ref **D2** |
| **D5** | **Operator-driven frozen-header / frozen-contract reconcile is sanctioned in-PR** (operator-ratified 2026-05-16). "Preserve-verbatim" on a frozen scaffold header forbids *unsanctioned worker contract-drift* — it does NOT forbid *sanctioned reconciliation*. When an operator-tier action (a BLOCKING finding, an operator ruling) legitimately moves a worker's body past its frozen header or I/O contract, the header/contract is **reconciled to match in the SAME PR**, with the reconcile receipt — citing the driving operator action — recorded in the **commit message**. **Amended 2026-05-17 (operator-ratified):** the receipt is a commit-message entry, *not* an in-file `HEADER RECONCILE` block — the in-file-block form is superseded by the no-prose directive / `docs/modeling-discipline.md` Practice 9. D5's intent — a conscious, reviewer-visible reconcile receipt — is preserved: it lives in the commit message + `git blame`. **Discriminator:** an operator-tier action drove the body change ⇒ reconcile the header/contract in-PR, flagged; NO operator action drove it ⇒ the body must conform to the frozen header (preserve-verbatim holds, the worker cannot move the contract). Covers BOTH header-doc drift (stale `Owns`/`Consumes` bullets after a sanctioned refactor) AND frozen-contract changes (operator BLOCKS an I/O contract as P2/§2-insufficient). Already operating: `integer.dag` #3190 (`Consumes` reconciled), #3209 finding 2 (`WaitProcessResult`). Does NOT permit free worker header edits — operator-driven only. | this row + the reconcile receipt recorded in the **commit message** (the in-file `HEADER RECONCILE` block form — `integer.dag` #3190 precedent — is retired by Practice 9, 2026-05-17); refines the frozen-scaffold-header / preserve-verbatim discipline |
| **D2-REV** | **D2 / D4 REVERSED — fact-bundle modeling supersedes alias-identity** (operator-ratified 2026-05-17, **out-of-band** — the verbatim operator-quote provenance is in the "D2 REVERSAL" section below). An `extdeps` primitive is modeled as a fact-bundle grounded from its own spec, never a bare `std/` alias; deduplicate to a `std/` carrier only on *proven* coincidence; the structural fact-density / hollow-alias gate is `T-30`, a hard prerequisite of T-4. This is a one-line index pointer — the full record (root cause, phased plan, impact map, "coincide" definition) is the **"D2 REVERSAL + FACT-BUNDLE RESEED"** section below, which is the authority. Supersedes the **D2** and **D4** rows above. | the **"D2 REVERSAL + FACT-BUNDLE RESEED"** section of this file (authority); `#3224` executes Phase 1; `docs/modeling/grounding-worked-examples.md` companion |
| **P4-3208** | **Practice-4 ledger home for #3208 Lean/machine-code/C++-bool model coproducts** (operator-directed strict de-prose 2026-05-17). `.dag` files keep only terse anchors/tags; detailed dissolution ledgers live here until modeling-discipline reconciles Practice-4 vs Practice-9. See the "P4-3208 coproduct ledger" section below. | this row + "P4-3208 coproduct ledger"; `extdeps/languages/{lean,machine_code,cpp}.dag` keeps terse carrier declarations only |
| **OS-1** | **OS extdep coproduct ledgers and refinement-scaffold disposition for #3209** (operator-audit regime, 2026-05-17). Practice-4 dissolution ledgers for `file_system.dag` / `process.dag` are architectural decision records, not source comments; the `.dag` files keep only terse tags/anchors. POSIX numeric and byte-string refinement gaps remain tracked scaffolds in #3209, not expanded fact bundles; named trigger is **T-25-core refinement substrate / T-30 fact-density gate**. | this row + "OS-1 — #3209 coproduct dissolution and scaffold record" below; terse tags in `extdeps/file_system.dag` and `extdeps/process.dag` |
| **T29-ABI** | **C++ ABI / target data-model slice** (T-29, operator-ratified 2026-05-17; manager brief 2026-05-18). `extdeps/cpp_abi.dag` is the single authority for C++ implementation-defined fundamental-width choices consumed by `cpp.dag`; C++ primitive fact-bundles parameterize over `CppTargetProfile<abi_model,data_model>` instead of guessing LP64/ILP32 constants. `CppIntegerWidth`, `CppPlainCharSignedness`, `CppDataModelFamily`, and `CppAbiModel` remain closed coproducts: each is an externally named alternative, not simultaneous coordinates. Common ILP32/LP64/LLP64/ILP64 rows are type-level data-model facts parameterized by `plain_char_signedness`; the data-model family never defaults that independent implementation-defined fact. | `extdeps/cpp_abi.dag`, `extdeps/languages/cpp.dag`, `STRUCTURE.md`; cross-ref T-29 in `TASKS.md` |

---

## De-Prosed Substrate — Coproduct & Scaffold Classification Ledgers
<a id="coordination-coproduct-ledgers"></a>

These classification ledgers are the authoritative coproduct/scaffold entries
for the de-prosed `.dag` files touched by T-31(b). The terse `.dag` headers
remain boundary indexes until the named substrate support lands.

| File | Declaration(s) | Class | Receipt |
|------|----------------|-------|---------|
| `std/algebra.dag` | `FreeMonoid<T>` | Green coproduct | Closed inductive finite-sequence carrier: `Empty | Cons { head, tail }`. Terminal because finiteness is structural in the spine, not a side fact or length field. |
| `std/algebra.dag` | `Ordering` | Green coproduct | Closed total-order trichotomy: `Less | Equal | Greater`. Terminal because exactly one comparison result holds; numeric ranks may be derived projections, not the carrier. |
| `std/cardinality.dag` | `DescentEvidence`, `Multiplicity` | Green coproducts | `DescentEvidence` is the closed per-edge descent observation sum. `Multiplicity = Bounded | Unbounded { step: TerminationProof }` keeps the bounded/unbounded distinction closed. |
| `std/cardinality.dag` | `Never` | Green atomic | Uninhabited type — zero inhabitants; grounds empty / bottom types in external language specs (e.g. Rust `!`) on a single std/ authority. |
| `std/cardinality.dag` | `Unit` | Green atomic | Singleton type — exactly one inhabitant; shared cardinality-1 authority for unit-shaped facts (opaque `type Unit` in the v2-bootstrap-compatible surface; same semantic role as a nominal unit value). |
| `std/cardinality.dag` | `TerminationProof` | Green proof record | Encodes lexicographic descent by structure: `non_increasing: List<RankingDimension>` plus mandatory `strict: RankingDimension`; no stored `DescentEvidence` field may stand in for the strict witness. |
| `std/cardinality.dag` | `Never`, `Unit` | Green opaque atoms | Canonical 0- and 1-inhabitance anchors in shared vocabulary: `Never` (empty), `Unit` (singleton). Rust `!` / `()` name **external** encodings of those facts, not identities proved in this row. **D2-REV:** per-language never/unit primitives (e.g. `NeverScalar`) remain **spec fact-bundles** in `extdeps/`; this ledger does **not** assert a bare alias into `std/`—deduplication to these carriers only on **structurally evidenced coincidence** authored elsewhere. Opaque atoms carry no fields; `std/` must not mint a parallel second empty/unit authority. |
| `std/cardinality.dag` | `NonZeroNat` | Green proof record | Structural strictly-positive natural: `prev: Nat` denotes `Succ { prev }` (same witness field name as `Nat`’s successor case) — excludes `Zero` by construction (no separate ordering predicate). Use for positive counts/widths (e.g. “at least one”) without kernel-ambient `Int` width payloads. |
| `std/cardinality.dag` | `NatLeWitness`, `UpperBoundedNat` | Green proof bundle | Inductive witness for `value ≤ inclusive_max` on Peano `Nat`: `LeqZero { upper }` proves `Zero ≤ upper`; `LeqSucc { inner }` lifts `a ≤ b` to `Succ a ≤ Succ b`. `UpperBoundedNat` carries **only** `{ le: NatLeWitness }`; `nat_le_witness_lower` / `nat_le_witness_upper` derive the endpoints from the witness spine (no independent value/max fields that could disagree with `le`). `upper_bounded_nat_value` / `upper_bounded_nat_inclusive_max` project those derived endpoints. **`nat_le_witness_well_formed` / `upper_bounded_nat_well_formed` are vacuous on any inhabited witness** (always `true` by induction over the spine): uniform predicate hooks only—downstream must not treat them as substantive runtime validators that recover facts absent from the witness. |
| `std/collection.dag` | `NonEmptyList<T>` | Yellow refinement scaffold | `NonEmptyList<T> = List<T> where non_empty` is the canonical future shared shape over the `List<T> = FreeMonoid<T>` carrier, not a second head/tail inhabitant family. It is not exported from `std/collection.dag` yet because `src/v3/compiler/src/lower.rs` lowers registered `non_empty` predicates through the placeholder path rather than a Bool body tied to `free_monoid_non_empty`. Bounded use: consumers must not rely on this alias to reject empty lists yet. Trigger: bootstrap lowerer synthesizes/enforces `non_empty` for `List` carriers through the landed FreeMonoid predicate, then `std/collection.dag` may publish `NonEmptyList<T>` and downstream non-empty-list sites may rewrite to it. |
| `std/collection.dag` | `Set<T>` finite-cardinality refinement | Yellow refinement scaffold | Documented: bare `Set<T> = PointwisePower<T>` is an arbitrary subset/characteristic-function carrier and does not encode finiteness. Bounded use: consumers needing finite subsets must wait on the refinement, not infer finiteness from `Set<T>`. Trigger: cardinality/enumerability substrate (`Multiplicity` plus an enumerability `Witness`) lands and adds `FiniteSet<T>` or equivalent. |
| `std/diagnostic.dag` | `Extent.ByteRange` | Yellow value-refinement scaffold | Documented: `ByteRange { start: Int, end: Int }` is an honest bridge carrier because current substrate syntax cannot exclude negative offsets or `start > end`. Bounded use: producers must validate textual spans before constructing diagnostics that rely on byte-range validity. Trigger: bounded/non-negative ordered span carrier or equivalent substrate refinement. |
| `std/diagnostic.dag` | `Extent`, `Locus`, `NoCorrectionReason`, `Correction`, `Outcome<T>` | Green coproducts | `Extent` is terminal as whole-file vs byte-range, with the raw-offset gap isolated in the ByteRange yellow row. `Locus` is the closed set of diagnostic pointing sites. `NoCorrectionReason` is the exhaustive no-fix partition. `Correction` is the typed show-correct-code sum, not `Option<Node>`. `Outcome<T>` is the two-case produced vs fail-closed rejected carrier beside `Diagnostic`. |
| `std/float.dag` | `FloatSpecial`, `FloatBody` | Green coproducts | `FloatSpecial` is the closed IEEE non-finite partition (`QuietNan`, `SignalingNan`, `PosInfinity`, `NegInfinity`). `FloatBody` is the finite vs non-finite split for IEEE semantics, with `NonFinite` carrying the closed `FloatSpecial` tag. |
| `std/float.dag` | `Float32`, `Float64`, Nat exponent/significand fields | Yellow width/value scaffold | Documented: nominal float wrappers and `Nat` fields do not type-enforce binary32/binary64 exponent/significand widths or interchange validity. Bounded use: producers must respect nominal widths and Grounding/interchange checks; this file must not publish a fabricating `ApproximateField<Float>` witness. Trigger: bounded refinement substrate and/or grounded IEEE primitive Arrow bodies. |
| `std/integer.dag` | `GroupCompletion<M>` | Yellow constrained-inhabitance scaffold | Documented: group completion denotationally requires a commutative-monoid witness, but current syntax cannot express constrained generic parameters. Bounded use: authored use is `GroupCompletion<Nat>`, backed by `nat_additive_commutative_monoid`. Trigger: v4 lands constrained generic parameters or inhabitance bounds. |
| `std/logic.dag` | `Bool` | Green coproduct | Closed two-valued logic carrier: `True | False`. Terminal as the irreducible Boolean partition; Boolean algebra data is a structure over this carrier, not a second carrier. |
| `std/text.dag` | `Char` | Yellow value-refinement scaffold | Documented: `Char = Nat` is an honest bridge carrier for Unicode scalar values because current substrate syntax cannot express `0..=0x10FFFF` excluding surrogate range `0xD800..=0xDFFF`. Bounded use: producers/ingesters must validate scalar-value membership. Trigger: bounded non-contiguous value refinement or equivalent character scalar carrier. |
| `std/machine.dag` | `Byte`, `Word8`, `Word16`, `Word32`, `Word64`, `Word128` | Yellow fixed-cardinality scaffold | Documented: `bits: List<Bit>` and `bytes: List<Byte>` fields are intentionally unrefined list carriers. Bounded use: no consumer may treat list length as proven width without an explicit validator, witness, or grounding-side check. Trigger: fixed-cardinality list/field refinement substrate. |
| `std/network.dag` | `HttpMethod` (`T-26-HttpMethod`) | Green coproduct | RFC 9110 method token set plus RFC 5789 `PATCH`; closed boundary carrier consumed by OpenAPI / coordination / wire-contract models. |
| `std/network.dag` | `Rfc3986AbnfProduction`, `UriScheme`, `UriUserInfo`, `UriUserInfoSlot`, `UriHost`, `UriPort`, `UriPortSlot`, `UriAuthority`, `UriPath`, `UriHierPart`, `UriQuery`, `UriQuerySlot`, `UriFragment`, `UriFragmentSlot`, `Url`, `UriRelativePart`, `UriRelativeReference`, `UriReference`, `NetworkAddress` | Green modeled facts | RFC 3986 §3/§4.1/§4.2 carriers model current-substrate facts now: `Url` is scheme + hier-part + optional query/fragment; `UriAuthority` decomposes to optional userinfo + host + optional port; `UriPort` carries `Nat`; `UriScheme` carries a normalized spelling; every component carries its ABNF production data; `UriReference` includes absolute and relative references for consumers such as OpenAPI URL fields. |
| `std/network.dag` | `UriUserInfoSlot`, `UriPortSlot`, `UriQuerySlot`, `UriFragmentSlot` (`T-26-UriComponentPresence`) | Green coproducts | RFC 3986 optional component presence is modeled structurally: userinfo, port, query, and fragment are present/absent slots, so constructors do not fabricate empty component values. |
| `std/network.dag` | `UriHierPart` (`T-26-UriHierPart`) | Green coproduct | RFC 3986 §3 hier-part alternatives are modeled as the closed choice among authority path, absolute path, rootless path, and empty path. |
| `std/network.dag` | `UriRelativePart` (`T-26-UriRelativePart`) | Green coproduct | RFC 3986 §4.2 relative-part alternatives are modeled as the closed choice among authority path, absolute path, no-scheme path, and empty path. |
| `std/network.dag` | `UriReference` (`T-26-UriReference`) | Green coproduct | RFC 3986 §4.1 URI-reference is the closed choice between an absolute URI and a relative reference; OpenAPI URL fields consume this carrier where relative references are permitted. |
| `std/network.dag` | RFC 3986 validated component refinements | Yellow `feature:T-25-core` scaffold | Residual only: fail-closed validation that a component spelling satisfies its carried ABNF production. Bounded use: constructors/ingesters must validate before constructing component values. Trigger: T-25-core base-type plus fail-closed validation shape and T-30 structural checker; dissolve under #3244 plan-binding by replacing producer-side convention with checked refinement witnesses. |
| `std/nat.dag` | `Nat` | Green coproduct | Closed Peano inductive natural number carrier: `Zero | Succ { prev: Nat }`. Terminal because the recursive structure is the natural-number definition; arithmetic data rows are structures over it. |
| `std/node.dag` | `Connective`, `Behavior`, `NodeKind`, `EdgeLabel`, `EdgeDiscipline` | Green coproducts | `Connective` is the closed set of six type connectives; additions are substrate-extension stops. `Behavior` is the closed set of five L1 computation behaviors. `NodeKind` is the binary type/computation split. `EdgeLabel` is named vs positional child addressing. `EdgeDiscipline` is the closed classifier derived from connectives. |
| `std/node.dag` | `Path` prior step sum | Green dissolved-away receipt | Positional path steps are deliberately dissolved: `Path` is `List<Symbol>` over named edges only. The removed path-step sum is not retained; positional addressing is subsumed by replacing the enclosing named subtree until a future ratified extension changes that shape. |
| `std/witness.dag` | `Witness<C>` | Green coproduct | Closed fail-closed proof/read carrier: `Holds { value } | Violates { diagnostic }`. Terminal because every read either carries the witnessed value or a diagnostic explaining the failed witness. |
| `extdeps/coordination.dag` | `FrameworkBinding` | Green coproduct | Closed endpoint framework coordinate: an endpoint is either hosted by a named framework or not framework-hosted. Terminal because a bare optional would hide absence, and a flat record would make missing-present combinations representable. |
| `extdeps/coordination.dag` | `ExchangePattern` | Green coproduct | Closed messaging-pattern coordinate: request-reply, fire-and-forget, stream, and publish-subscribe are mutually exclusive exchange topologies at the wire-contract layer. Settlement and replica convergence are separate coordinates, so async pubsub and streaming-with-convergence remain representable. |
| `extdeps/coordination.dag` | `SettlementGuarantee` | Green coproduct | Closed settlement coordinate: a contract either settles immediately or carries a structural `SettleBound`. Terminal because an optional bound would allow boundedness to be skipped, while making settlement a coordinate avoids compressing it into exchange topology. |
| `extdeps/coordination.dag` | `ConsistencyGuarantee` | Green coproduct | Closed replica-convergence coordinate: a contract either has no replica-convergence obligation at this layer or carries a structural `ConvergeBound`. Terminal because consistency is independent of exchange topology and settlement timing. |
| `extdeps/coordination.dag` | `CoordinationEffectKind` | Yellow effect-kind scaffold | Documented: coordination effect kind is currently one explicit coproduct fact on `CoordinationBind` because `lens/effect.dag` cannot yet derive HTTP/queue/stream/pubsub kind from the bound node's type signature. Bounded use: producers must set `CoordinationEffectKind` to match the bound signature; consumers must treat it as provisional and may not introduce parallel wrapper families or empty effect marker carriers. Gate: `feature:T-13-effect-lens-coordination-signature` plus `consumer:T-16-deployment-endpoint-partition` (`src/v4/TASKS.md` T-13/T-16). Dissolve-on-arrival: when T-13 effect-lens substrate reads coordination effect kind from the type signature, remove `CoordinationEffectKind` and have coordination consumers derive effect kind from the signature authority. |
| `extdeps/coordination.dag` | `BindRef` membership | Yellow resolver-membership scaffold | Documented: `BindRef { identity: Symbol }` is an opaque reference to an intended L1 `Bind` node; current syntax does not encode a typed membership witness proving the symbol resolves to a Bind node whose signature carries the coordination effect. Bounded use: producers must only emit `BindRef` values interned against actual Bind nodes and set `CoordinationEffectKind` to match that node's signature; consumers must validate Bind resolution before treating coordination effect typing as complete and may not treat the raw symbol as proof of Bind membership. Gate: `feature:T-13-effect-lens-coordination-signature` plus `consumer:T-16-deployment-endpoint-partition` and `feature:T-25-core-validation-boundary` (`src/v4/TASKS.md` T-13/T-16/T-25-core). Dissolve-on-arrival: when T-13/T-16 run on the T-25-core fail-closed validation substrate, replace this row with a typed Bind reference, constructor-validated `CoordinationBind`, or equivalent membership witness so non-Bind or unresolved targets are unrepresentable. |
| `extdeps/coordination.dag` | `NetworkAddress` | Yellow value-refinement scaffold | Documented: `NetworkAddress { identity: Symbol }` is an opaque boundary identity until the single std boundary-carrier authority lands. Bounded use: consumers may compare identity only and producers must intern deployment-boundary addresses. Gate: `feature:T-26-std-network-address` (`src/v4/TASKS.md` T-26 std boundary carriers; PR/task: T-26). Dissolve-on-arrival: when T-26 lands the spec-grounded `std` `NetworkAddress`/URI authority, update `coordination.dag` to consume/alias that carrier and retire this local identity wrapper. |
| `extdeps/coordination.dag` | `WireContractFacts.from/to` endpoint membership | Yellow deployment-membership scaffold | Documented: `WireContractFacts` carries `EndpointRef` values while `DeploymentUnit` owns `endpoints: List<Endpoint>`; current syntax does not encode a scoped membership-and-uniqueness witness tying those refs to exactly one endpoint in that list. Bounded use: producers must only emit contracts whose `from` and `to` refs each resolve to exactly one endpoint in the enclosing `DeploymentUnit.endpoints`; consumers that require closed-world endpoint membership must validate presence and uniqueness before treating the contract as deployment-complete. Gate: `consumer:T-16-deployment-endpoint-partition` plus `feature:T-25-core-validation-boundary` (`src/v4/TASKS.md` T-16/T-25-core). Dissolve-on-arrival: when the T-16 coordination consumer is implemented on the T-25-core fail-closed validation substrate, replace this row with a constructor-validated `WireContractFacts` or deployment-scoped membership/uniqueness witness so absent or multiply-owned endpoint targets are unrepresentable. |
| `extdeps/coordination.dag` | `LanguageRef` / `FrameworkRef` membership | Yellow registry-membership scaffold | Documented: `LanguageRef { identity: Symbol }` and `FrameworkRef { identity: Symbol }` are opaque refs embedded in `Endpoint.language` and `FrameworkBinding.HostedByFramework`, while current syntax does not encode a scoped membership-and-uniqueness witness tying those refs to declared language/framework models. Bounded use: producers must intern refs against declared T-4 language models and T-4.7 framework models; consumers may compare identity only and must validate presence and uniqueness before treating endpoint language/framework resolution as complete. Gate: `consumer:T-16-language-framework-reference-resolution` plus `feature:T-4-language-model-registry`, `feature:T-4.7-framework-model-registry`, and `feature:T-25-core-validation-boundary` (`src/v4/TASKS.md` T-16/T-4/T-4.7/T-25-core). Dissolve-on-arrival: when the T-16 coordination consumer is implemented on the T-4/T-4.7 registries and T-25-core fail-closed validation substrate, replace this row with resolved/validated language and framework references or scoped membership/uniqueness witnesses so dangling or multiply-owned refs are unrepresentable. |
| `extdeps/languages/go.dag` | (Practice-4 sum carriers + D2 partial) | Green coproduct family / records | Per merge-base `92cb26402` 🟢 blocks (see **Part 6 · CP-3229-GREEN-TERMINAL**). De-prose 2026-05-18: in-file prose removed; `GoCost` / `GoIntegerOverflowDisposition` / D2 deferrals indexed here, not in body comments. |
| `extdeps/languages/python.dag` | (Practice-4 sum carriers + cost record) | Green coproduct family / records | Same as go row; merge-base had three 🟢 sum ledgers (see **Part 6 · CP-3229-GREEN-TERMINAL**). **Heuristic vs content (Practice 9):** mechanical `//`-line share may sit modestly above the reviewer’s ~20% *heuristic* while the file still meets **content** compliance (mandated path + four-line header + `// Anchor:` + one-line 🟢 tag per coproduct only). That is not a license to pad with blank lines to game the ratio; additional non-`//` lines should come from real substrate (e.g. more carriers/imports), not whitespace inflation. |
| `extdeps/languages/rust.dag` | (Practice-4 sum carriers + D2 resolver) | Green coproduct family / records | Same bulk **CP-3229-GREEN-TERMINAL** receipt; `PubInPath` semantic scaffold and `RustCost` raw-`Int` bridge remain producer obligations per Part 6 / substrate tables, not narration in the `.dag` body. |
| `extdeps/languages/typescript.dag` | `TsEcma262NumericPrimitiveKind`, `TsEcma262PrimitiveOperationSemantics`, D2 resolver scaffolds | Green coproduct family / yellow D2 deferrals | De-prose 2026-05-18: in-file prose removed; carrier declarations are byte-identical. `TsEcma262NumericPrimitiveKind` is 🟢 terminal because ECMA-262/TypeScript exposes exactly one numeric primitive kind per value: `number` or `bigint`; the partition is not a bool proxy, a dimensional product, or a parameterized width family, and algebraic facts land through future std numeric aliases rather than collapsing this boundary classifier. `TsEcma262PrimitiveOperationSemantics` is 🟢 terminal because resolver rows choose one ECMA primitive-semantics track among IEEE-754 `Number`, ToInt32/ToUint32 bitwise `Number`, and exact `BigInt`; this is not Rust overflow policy and not a width-indexed family. D2a(2): the `{spelling}` `GroundingMap` shape is **retired** under ruled-B — there is no "shared P2 home" pending because the shape is removed, not relocated (cf. INVARIANTS §P2; the `GroundingMap`→`resolver.dag` relocation obligation is obsolete for the removed twin). A local `GroundingMap` type and `{spelling}`-style `ts_*_grounding: GroundingMap{…}` rows remain **forbidden** (the hollow form). The **ruled-B decl-ref** `ts_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` is a *different, sanctioned* shape (single std-authority reference, not a `GroundingMap` row) — declared this PR, **P2/E-6 staging** (consumer-gated); single-authority disposition is the **TS-D2 §** below. The Bool D2a(1) alias `TsBoolean = Bool` is **retired** (A-vs-B = B / ruled-B) — removed from `typescript.dag`, **not** in the remaining-scaffold set; single-authority disposition is the **TS-D2 §** below (machine-readable grounding *declaration* `ts_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` — decl-ref shared-authority, present + v2-parse-verified; **P2/E-6 STAGING**, not landed authority — no same-PR consumer, canonical fold specified-not-realized; `grounding-worked-examples.md` §0). Non-Bool D2a(1) alias rows remain 🟡 tracked scaffolds: `TsNumber = Float64` waits on std/float `Float64` + `ApproximateField`, `TsBigInt = Int` waits on unbounded integer/BigInt alignment, `TsString = String` waits on UTF-16/std text refinement policy, and `symbol`/`null`/`undefined` wait on the LanguageModel/nominal-runtime substrate. D2a(3) per-primitive instance rows remain deferred only on top-level nullary sum-variant `data` body validation (Class-5-Gap-3); record-structural disposition rows are safe; the retired D2a(2) `{spelling}`-`GroundingMap` shape is superseded by the ruled-B decl-ref grounding above (no pending `GroundingMap` authority decision — the shape is removed). D2b IEEE-754 `number` Arrow bodies remain deferred to the bundled T-4 grammar and std/float ApproximateField lane. |

### Coordination coproduct receipts

`FrameworkBinding` is 🟢 GREEN. Fact placement fails because every endpoint
consumer needs the hosting coordinate. Variant-is-data fails because a boolean
plus optional framework would make present-without-framework and hidden absence
representable. Algebraic form is not applicable; framework hosting is not an
algebra carrier. Dimensional decomposition fails because hosted vs unhosted is
exclusive at one endpoint. Parameterized-family reduction fails because the
hosted arm carries `FrameworkRef` and the unhosted arm is nullary.

`ExchangePattern` is 🟢 GREEN. Fact placement fails because emitter,
simulator, and verification consumers all read the same exchange topology.
Variant-is-data fails because independent booleans would permit contradictory
topologies. Algebraic form is not applicable; the variants are messaging
patterns, not algebraic operations. Dimensional decomposition has already been
performed: settlement and consistency are separate carriers, while the exchange
topology itself remains one-of. Parameterized-family reduction fails because
the four variants are named topology alternatives, not copies over a declared
index family.

`SettlementGuarantee` is 🟢 GREEN. Fact placement fails because bounded
settlement is shared by simulator and verification consumers. Variant-is-data
fails because an optional bound would allow a bounded claim with no bound.
Algebraic form is not applicable; settlement timing is not a richer algebra
carrier. Dimensional decomposition has already been performed: settlement is
orthogonal to exchange topology and replica convergence. Parameterized-family
reduction fails because immediate settlement and bounded settlement have
different payload shapes.

`ConsistencyGuarantee` is 🟢 GREEN. Fact placement fails because convergence
obligations are shared by distributed simulation and verification consumers.
Variant-is-data fails because an optional convergence bound would permit a
convergence claim without evidence. Algebraic form is not applicable here;
future CRDT or replica algebras may refine implementations, but this carrier is
the wire-contract convergence obligation. Dimensional decomposition has already
been performed: convergence is orthogonal to exchange and settlement.
Parameterized-family reduction fails because the no-convergence and
bounded-convergence arms have different payload shapes.

`CoordinationEffectKind` is 🟡 YELLOW. Fact placement fails because the
effect lens cannot yet read coordination effect kind from the bound node's
signature. Variant-is-data fails as a terminal claim: the four arms are a
provisional effect-kind partition, not independent booleans or payload facts.
Algebraic form is not applicable; this is an effect-kind read, not an operation
algebra. Dimensional decomposition fails because the temporary coordinate is
exactly the effect kind. Parameterized-family reduction failed in the prior
wrapper-family shape: four empty marker carriers and four near-identical bind
wrappers were a B3 parallel taxonomy and #3273 nominalization issue. Named
trigger: `feature:T-13-effect-lens-coordination-signature`; dissolve when
T-13 reads HTTP/queue/stream/pubsub kind from the type signature and T-16
consumes that derived fact.

### PR-3252-cardinality-std — `std/cardinality.dag` `Never` / `Unit` (Practice 9 receipt)

**Cross-ref:** Part 1 de-prosed substrate table rows for `Never` / `Unit`; **`§PR-3252-extdeps-deferrals`** for `go.dag` / `python.dag` / `rust.dag` pointer discipline.

- **P1 inhabitance:** `Never` (zero inhabitants) and `Unit` (cardinality-1 authority) are the shared std facts for external empty / unit types. **Do not conflate** with bounded non-negative **numeric** refinement (cost axes, LLVM width payloads, tuple-`()` substrate, etc.) — those remain on **T-25-core + `nat`/`integer`** and the **SL-3229** dissolution family already indexed in this file.

- **`Unit` opaque (not `= MkUnit`):** v2 v4-bootstrap (`v2-compiler compile --source-root src/v4 --target dag`) does **not** resolve `MkUnit` as a type name for a single-variant `|` sum; **`type Unit`** opaque stays within the v2-compat subset without changing semantic authority.

## CP-1b — `03_resolve` / `extdeps/languages/dag` scaffold (Practice 9)

**Receipt (2026-05-17):** Rationale that briefly lived in `.dag` body comments is indexed here per `docs/modeling-discipline.md` Practice 9 (substrate workers stay de-prosed).

1. **`dag_language_model_wave1_void_canonical_symbols`:** The four `data dag_c3_surface_sugar_*` mints are substrate-declared self-canonical `Atom` identities for resolve when no binder row exists; the characteristic-function `Set` is the LM module’s authority for that closed prelude slice.

2. **`dag_language_model_canonical_symbols` / wave-1 void (B2-OMNI; codex #13790):** `dag_language_model_canonical_symbols(lm)` must **use** the passed `DagLanguageModel`. **Wave-1 void *shape*** — typed constructors `VoidLexRules` and `VoidGrammar` — is **necessary but not sufficient** for admitting the native `.dag` C3 prelude `Set`: the same incidental shape could satisfy a *non*-native scaffolded LM and wrongfully inherit native `dag_c3_surface_sugar_*` keywords (INVARIANTS P1/P2; modeling-discipline Practices 3 & 5). **Predicate disposition — 🟡 YELLOW:** `dag_language_model_is_wave1_void_shape` is a bounded wave-1 structural query over the typed language-model mode partition, not a generic grammar walk; it dissolves when T-6/T-7 fold the void model into the first modeled lex/grammar root or when the language model exposes a canonical mode query. **`DagLanguageModel` carries `language_identity: Symbol`:** only `dag_language_model_surface_id` (minted in `extdeps/languages/dag.dag`, wired by `dag_language_model_wave1_void`) arms the four `data dag_c3_surface_sugar_*` slice via `dag_language_model_wave1_void_canonical_symbols()`; any other identity at wave-1 void shape yields an **empty** `Set` (fail-closed) until that language’s own extractor lands. `dag_language_model_is_wave1_void_shape` remains the structural leg; identity is the model-specific authority leg. **Consumer contract (`compiler/03_resolve.dag`, `resolve_atom`):** after binder lookup misses, the canonical `Set` is consulted only as an opt-in self-canonical prelude shortcut; an **empty** `Set` leaves that branch inert, so unresolved `Atom` identities become `ResolveRejected` with `unbound_symbol_diagnostic`—not a silent success path (external api-review receipt, 2026-05-18).

3. **`resolve_bind_*` (arity-3 `Bind`):** Slot-0 binder is a **declaration** handle (bare `Atom` → `canonical_atom`, not `resolve_atom`). Slot-1 initializer resolves in the **outer** scope only (binder not visible). Slot-2+ resolve under `ScopeFrame` locals carrying the binder.

**Coproduct ledger (Practice 4):** New `N≥2` sum types in `compiler/01_tokenize.dag` / `compiler/02_parse.dag` / `compiler/03_normalize.dag` / `compiler/03_resolve.dag` carry the required in-file 🟡 tag; classification / dissolution pattern / named triggers are recorded here (not in the `.dag` body prose).

4. **`SurfaceSugarKind` (`compiler/03_normalize.dag`):** **Classification — 🟡 YELLOW (scaffold):** T-8 normalize seam; not a terminal substrate primitive while the stage is scaffold. **Dissolution pattern — enumerated axis (closed N=4):** names the four C3 surface-sugar connectives aligned with the `data dag_c3_surface_sugar_*` identities in `extdeps/languages/dag.dag` (THESIS/C3; not a foreign-label consumer coproduct). **Named triggers:** `classify_sugar`; feeds `normalize_node` / `malformed_sugar_diagnostic`.

5. **`SugarClassification` (`compiler/03_normalize.dag`):** **Classification — 🟡 YELLOW (scaffold).** **Dissolution pattern — partition:** `Sugar` vs `NotSugar` splits the normalize walk between C3-shaped sugar and pass-through `Node` kinds. **Named triggers:** `classify_sugar`; `normalize_node`; `normalize_edge`.

6. **`NormalizeChildrenResult` (`compiler/03_normalize.dag`):** **Classification — 🟡 YELLOW (scaffold).** **Dissolution pattern — success-or-Diagnostic carrier:** `NormalizedChildren` vs `NormalizeChildrenRejected` threads child recursion fail-closed without a second authority. **Named triggers:** `normalize_children`; `normalize_edge` / `normalize_node` folds.

7. **`Scope` (`compiler/03_resolve.dag`):** **Classification — 🟡 YELLOW (scaffold).** **Dissolution pattern — structured lexical environment:** `ScopeRoot` (module `Namespace`) vs `ScopeFrame` (locals + outer) for K-1 use→def binding (B-4; Part 1 **B-4** row). **Named triggers:** `resolve_node`; `resolve_atom`; `resolve_bind_*` family.

8. **`ResolveResult` (`compiler/03_resolve.dag`):** **Classification — 🟡 YELLOW (scaffold).** **Dissolution pattern — success-or-Diagnostic:** `ResolvedNode` vs `ResolveRejected` on single-node resolution. **Named triggers:** `resolve_node`; `resolve_atom`.

9. **`ResolveChildrenResult` (`compiler/03_resolve.dag`):** **Classification — 🟡 YELLOW (scaffold).** **Dissolution pattern — success-or-Diagnostic:** `ResolvedChildren` vs `ResolveChildrenRejected` on edge-list resolution. **Named triggers:** `resolve_children`; `resolve_arrow_domain_named_params`; folds using `EdgeResolveAcc` / `BindEdgeAcc`.

10. **`test/claim/manual/resolve_compile_anchor.dag` (Tier-1 compile anchor):** **Why:** codex #13724 — keep `resolve` / `dag_language_model_canonical_symbols` / wave-1 `DagLanguageModel` on the v2 `compile --source-root src/v4` graph so resolver/LM edges cannot silently rot pre-T-22. **What:** imports `resolve` + `dag_language_model_wave1_void` / `dag_c3_surface_sugar_service`; `anchor_resolve_wave1_service_atom_via_canonical_symbols` builds minimal `Conj` → `Atom(service)` and calls `resolve` (exercises CP-1b items 1–2 + `resolve_atom` canonical-set fallback, including **`language_identity == dag_language_model_surface_id`** per codex #13790). **Runtime:** `v2-compiler run` on v4 `TestClaim`s remains **deferred until T-22** (`test/v2_run_preflight/MOVE1_COVERAGE.txt`); follow-up — promote to a real `TestClaim` asserting `Produced` on the resolved `Atom`, then add `Bind` / multi-edge cases.

11. **`compiler/01_tokenize.dag` — `LexRules` / wave-1 E0 (T-6 / B2-OMNI; PR #3284 → T-8 spine update #3311):** lexical authority is the typed coproduct `LexRules = VoidLexRules | ModeledLexRules { root: LexRuleSet }`, with rule rows carried by `LexRule` payloads rather than ad-hoc `Node` variant tags. Each `TokenRule.pattern` is a bounded `LexPattern` regular-language carrier (`EmptyPattern`, literal/char/class atoms, sequence, choice, optional, repeat), not a raw `Node` bridge. Wave-1 E0 void lex is `VoidLexRules`. `ModeledLexRules` fail-closes with `tokenize_lexical_walk_not_realized` until the P3 lexical walk primitive lands; no hand-rolled walker is introduced. Rationale lives here per Practice 9; `01_tokenize.dag` carries no in-file rationale beyond the mandated terse header (path + Scope/Owns/Consumes/Status + optional `Anchor:`) and the one-line tags on the tokenize coproducts.

12. **`std/node.dag` — `is_empty_conj_root` (P5 / INVARIANTS §Progress Is Dissolution; codex PR #3284):** **Not a net-new `is_*` verifier** — this is the **landed R1 receipt** from `docs/audit/dissolution-inventory.md` §1.0: one structural query replaces three literal duplicate predicates (`compiler/01_tokenize.dag`, `compiler/02_parse.dag`, `extdeps/languages/dag.dag`). **Interim steady-state:** shared empty-`Conj`-root shape leg for wave-1 void LM / E0–G0 stubs until T-6/T-7 generic walks consume grammar-as-data without re-deriving this `match`. **Forward dissolution (named triggers):** (a) `fold_node` / `NodeFold` in `std/node.dag` is **landed** (dissolution-inventory **P5** substrate — PR #3297; `node_well_formed` is already a consumer). Optional follow-up: fold this discriminant into that catamorphism / shared query surface if a ratified consumer wants to shed the dedicated `match`. Remaining **P5** debt is the **`compiler/03_resolve.dag` binding-harvest cascade** (inventory §2.3), not absent substrate. (b) `feature: compiler pipeline-stage lex-walk + parse-walk substrate` (inventory **P3**, TASKS T-6/T-7) — grammar-as-data carries void-book as constructor-shaped facts so consumers discriminate on `NodeKind` / carriers instead of calling `is_empty_conj_root`. **Audit anchor:** inventory §1.0 row **R1** (status = landed PR #3284). **Api-review receipt (PR #3274):** duplicate-collapse / modeling-discipline Practice 5 single authority — not a parallel verification predicate. **INVARIANTS.md §P5 — Dispatch-Discipline Mechanisms (b)** SG-0 `EXPECTED_HAND_AUTHORED_*` census gates **new** `src/v3/compiler/tests/**` hand-Rust paths; they do **not** apply to this v4-only substrate `fn`. **Exhaustiveness:** `NodeKind` is the closed `TypeNode | ComputationNode` pair today; a third top-level variant would require updating every `match n.kind` in `std/node.dag`, including this helper, in the same substrate edit that extends `NodeKind`.

13a. **`LexCharClass` (`compiler/01_tokenize.dag`):** **Classification — 🟡 YELLOW (P3-gated declarative seam).** **Dissolution pattern — closed character-class vocabulary:** variants name the lexical character classes the future T-6 walk consumes. **Named triggers:** `feature:T-6`; consumed by `LexPattern.CharClassPattern`.

13b. **`LexPattern` (`compiler/01_tokenize.dag`):** **Classification — 🟡 YELLOW (P3-gated declarative seam).** **Dissolution pattern — regular-language grammar carrier:** atoms plus sequence/choice/optional/repeat declare lexical productions as data; the current tokenizer does not walk them. **Named triggers:** `feature:T-6`; `consumer:compiler/01_tokenize.tokenize`; future P3 lexical walk consumes this carrier directly.

13c. **`LexRule` (`compiler/01_tokenize.dag`):** **Classification — 🟡 YELLOW (P3-gated declarative seam).** **Dissolution pattern — token-class-to-pattern row:** pairs an emitted token class with a typed lexical pattern; no stringly or raw-`Node` pattern payload remains. **Named triggers:** `feature:T-6`; `consumer:compiler/01_tokenize.tokenize`.

13d. **`LexRules` (`compiler/01_tokenize.dag`):** **Classification — 🟡 YELLOW (P3-gated declarative seam).** Wave-1 E0 encodes void lex as `VoidLexRules`; modeled lex data enters through `ModeledLexRules { root: LexRuleSet }`, `LexRule` rows, and bounded `LexPattern` payloads. **Dissolution pattern — stage-mode partition:** P3 owns the lexical walk primitive that consumes `LexRuleSet`; until that lands, `ModeledLexRules` rejects with `tokenize_lexical_walk_not_realized`. **Named triggers / gates:** `feature:T-6` (`TASKS.md` — `compiler/01_tokenize.dag` realizes the generic lexical walk over declarative `LanguageModel.lex`); `feature:T-7` (`TASKS.md` — `compiler/02_parse.dag` realizes the paired parse walk; grammar-side companion to T-6). **Dissolve-on-arrival:** when P3/T-6 supplies the generic lexical walk, it consumes `LexRuleSet` / `LexPattern` directly; any derived `Node` projection remains optional-only if a ratified self-hosting milestone needs that view, never a second authority. **Practice 9 receipt:** forward rationale and process tracking for this gate live only here.

### P9-COST — LLVM Instruction Cost Table Authority

`src/v4/lens/cost.dag` owns `llvm_instruction_cost(LlvmInstruction) -> Int` as the P9 cost-of-instruction model fact from `docs/audit/dissolution-inventory.md`; `src/v4/extdeps/languages/llvm_ir.dag` owns the LLVM instruction shape only. This preserves one cost authority while T-12 fills the total `Node -> Witness<SymbolicCost>` fold.

### PREFIX-T-23-registry — `lens/registry.dag` / `LensIdV0` + `LensModulePathV0` (Lane A freeze-pin; 2026-05-18)

**Cross-ref:** operator pin `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` §3 (human table mirror; `.dag` amend-first); `STRUCTURE.md` P2-staging tree line; INVARIANTS §P5(b) row + `v4_lens_registry_dag_smoke_test.rs` parse witness (same **staging posture** as T-30 `fact_density.dag`, not a §P2-landed primitive until a **generated** consumer reads `LensRegistryEntryV0`).

- **MODELING.md M4 — 🟢 terminal:** `LensIdV0` is the **closed sum** of ratified PREFIX lens keys (`Complexity` \| `Cost` \| … \| `StructuralResolution`). **`lens_id` is not `String`** — illegal / unlisted IDs are **unrepresentable** in `LensRegistryEntryV0` at the type level (P2 illegal-states discipline at substrate). **Rejected pattern:** `lens_id: String` + string literals for “closed” IDs (constructible typos; stringly dispatch). **Amend discipline:** extending the closed set requires operator-signed pin amend **and** a new `LensIdV0` variant (or superseding type revision) in the same `.dag` authority wave.
- **Practice 4 (coproduct dissolution) — 🟢 terminal:** `LensModulePathV0 = Bound { path: String } | Unbound` is the admitted sum for “ratified v4 lens module path” vs “operator has not closed a `v4.lens.*` home yet.” **Rejected pattern:** overload `path: String` with a reserved sentinel inside the same field (coproduct-as-magic-string; consumers would string-compare instead of matching variants). **Rejected pattern:** conflate the two placeholder lenses by collapsing both to one `Unbound` row — `lens_id` remains the row key; `Unbound` is the correct shared arm for distinct pending modules.
- **P2 / Practice 5 (single authority):** Until a **generated** harness reads the `data lens_registry_v0_*` rows mechanically, the registry is **P2-staging** only (INVARIANTS §P2 “When a boundary counts as landed”). The freeze pin is process authority for closed `LENS_ID` names; it does **not** substitute for the §P2 generated-consumer proof.
- **Dissolve-on-arrival:** delete or shrink the hand `compile_to_dag` smoke and this ledger bullet’s staging clauses when a generated `.dag` / emitted consumer is the sole reader of `LensRegistryEntryV0` / `LensIdV0` / `LensModulePathV0` in the compiler substrate (same dissolution style as the T-30 `SourceSpecReadFact` bullet above).

14. **`GrammarExpr` (`compiler/02_parse.dag`):** **Classification — 🟡 YELLOW (scaffold):** T-7 grammar-as-data production alternatives; not a terminal substrate primitive while the generic syntactic walk is still absent. **Dissolution pattern — typed declarative grammar family:** the five current variants (`Terminal`, `Sequence`, `Choice`, `Optional`, `Repeat`) carry syntax-production structure directly, replacing the rejected variant-as-`Symbol`-under-`Conj` encoding and making the production shape well-formed by construction. `Nonterminal` is intentionally absent until T-7 lands a same-authority production environment; this scaffold therefore cannot encode arbitrary symbolic nonterminal references whose closure is not validated. **Named triggers:** `feature:T-7-parse-syntactic-walk`; `consumer:compiler/02_parse.parse`; derived `grammar_expr_to_node` projection only. **Dissolve-on-arrival:** when the T-7 parse walker consumes the typed grammar model directly, keep the expression family as the grammar authority, add keyed production references only with same-authority start/nonterminal closure checks, and remove any projection-only `Node` dependency that is no longer needed by self-hosting.

15. **`Grammar` (`compiler/02_parse.dag`):** **Classification — 🟡 YELLOW (P3-gated declarative seam):** wave-1 boundary between the realized void G0 parser path and modeled grammar roots awaiting the T-7 walk. **Dissolution pattern — stage-mode partition:** `VoidGrammar` preserves the current empty-grammar acceptance/rejection behavior; `ModeledGrammar { root: GrammarRoot }` carries the typed grammar root without reintroducing hand validators or string tags. `extdeps/languages/dag.dag` now carries this typed `Grammar` directly in `DagLanguageModel.grammar`, not a raw `Node` grammar alias. `GrammarRoot.start` is the start `GrammarProduction` itself, not a free `Symbol`, so the current root cannot name a missing start production; the future T-7 production environment may reintroduce keyed lookup only with same-authority closure checks for start and nonterminal references. `GrammarProduction.emitted: Node` is a projection placeholder, not the final parse-tree authority; T-7 must replace or constrain it through the same typed production environment rather than letting arbitrary `Node` remain an escape hatch. **Named triggers:** `feature:T-7-parse-syntactic-walk`; `consumer:compiler/02_parse.parse`; `consumer:extdeps/languages/dag.dag` grammar model. **Dissolve-on-arrival:** when the syntactic walker is realized, fold the void path into a first modeled `GrammarRoot` (or equivalent typed empty-language model) so `parse` no longer branches on a scaffold mode coproduct and production emission is typed rather than an unconstrained raw `Node`.

---

## P4-3208 coproduct ledger

For #3208's `lean.dag`, `machine_code.dag`, and the `CppBool` surface in
`cpp.dag`, each listed coproduct was
checked against the Practice-4 five-pattern ledger:

1. **Fact placement** — can this distinction move to one consumer instead
   of substrate?
2. **Variant-is-data** — can this be a product/record with data fields?
3. **Algebraic form** — is this already an algebraic inhabitance/law fact?
4. **Dimensional** — is this one coordinate that should be a field, not a
   sum?
5. **Parameterized family** — is this one `F<X>` family rather than a
   closed coproduct?

### `src/v4/extdeps/languages/lean.dag`

| Carrier | Classification | Ledger result |
|---------|----------------|---------------|
| `LeanDeclarationKind` | 🟢 GREEN terminal | All five dissolution patterns fail: declaration kind is shared by grammar/emit/proof export; `Symbol` would be stringly; not algebra; one declaration has one kind; payload/validation rules are heterogeneous. |
| `LeanLevel` | 🟢 GREEN terminal | All five dissolution patterns fail: sort/constant/universe validation share the carrier; payloads are heterogeneous; Lean universe syntax is the external fact; one resolved level is zero/succ/max/imax/param; constructors are not one `F<X>`. Unresolved metavariables fail closed outside this carrier. |
| `LeanBinderInfo` | 🟢 GREEN terminal | All five dissolution patterns fail: parser/emitter/elaboration consume binder visibility; `Bool` cannot encode the four visibilities; not algebra; one binder has one visibility; closed nullary alternatives are not one family. |
| `LeanTermForm` | 🟢 GREEN terminal | All five dissolution patterns fail: type checking/emission/proof export share the classifier; payloads are heterogeneous and a flat record admits invalid mixtures; Lean term grammar is external; one term has one form; constructors are heterogeneous. |
| `LeanTerminationBy` | 🟢 GREEN terminal | All five dissolution patterns fail: proof export/termination checking/emission distinguish structural vs well-founded hints; structural carries a decreasing parameter while well-founded carries a measure; not algebra; one hint is structural or well-founded; payloads are heterogeneous. |
| `LeanTerminationClause` | 🟢 GREEN terminal | All five dissolution patterns fail: termination export/parser/emitter distinguish `termination_by` vs `decreasing_by`; payloads are heterogeneous; Lean clause syntax is external; one clause occurrence has one form; roles differ. |
| `LeanDefinitionTermination` | 🟢 GREEN terminal | All five dissolution patterns fail: declaration-level clause cardinality is shared; `List<LeanTerminationClause>` admits duplicates/order drift and a flat record admits optional holes; syntax cardinality is external; one declaration has one of four presence states; payload combinations are heterogeneous. |
| `LeanProofArtifact` | 🟢 GREEN terminal | All five dissolution patterns fail: proof exporter/emitter share the classifier; theorem and terminating-definition artifacts carry different required fields; not algebra; one artifact is theorem proof or terminating-definition proof; payloads are heterogeneous. |
| `LeanFidelityFeature` | 🟢 GREEN terminal | All five dissolution patterns fail: C5 boundary and generic walker share feature classes; `Symbol` would make the boundary open; not algebra; one boundary decision names one feature; feature classes are heterogeneous. |
| `LeanFidelityDisposition` | 🟢 GREEN terminal | All five dissolution patterns fail: modeled/normalized/fail-closed is the shared C5 disposition; non-modeled variants carry feature payloads and a flat record admits impossible combinations; not algebra; one feature has one disposition; payloads are heterogeneous. |
| `LeanIntKind` | 🟢 GREEN terminal | Same dissolution posture as `RustIntKind` (`rust.dag`, DECISIONS.md Part 6 / CP-3229): signed vs unsigned is a closed external alternative for fixed-precision scalars, not a hideable coordinate; not algebra; not one `F<X>` family over identical payloads. |
| `LeanIntWidth` | 🟢 GREEN terminal | Same dissolution posture as `RustIntWidth` (`rust.dag`): closed width ladder for fixed-precision scalars; `Pointer` names platform-sized facts separately from fixed bit widths; not algebra; not a `Symbol` tag. |
| `LeanScalar` | 🟢 GREEN terminal | Same dissolution posture as `RustScalar` `IntScalar` / `BoolScalar` (`rust.dag`): kind×width bundles fixed-precision facts read from the Lean 4.20 anchor. **A-vs-B = B (ruled-B):** the prior kernel `LeanBool = Bool` + `lean_bool_grounding` (spelling/coincidence assertion) is **retired** — a bare D2 alias asserts identity instead of demonstrating it (the hollow form the machine-readable-inhabitance bar forecloses). Lean `Bool`'s **classifier fact** is the `LeanScalar.BoolScalar` variant; the *machine-readable grounding edge* declaration `lean_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` (decl-ref, single std authority) is present + v2-parse-verified — **P2/E-6 STAGING**, not landed authority (no same-PR consumer; canonical fold specified-not-realized; `docs/modeling/grounding-worked-examples.md` §0). `lean.dag`'s per-file `GroundingMap` twin is retired with it (no spelling-only resolver rows post-D2-REV; cf. `machine_code.dag` / INVARIANTS §P2). **Signed** fixed-precision surface types from the anchor (`Int8`…`Int64`, `ISize`) inhabit `IntScalar { kind: Signed, width: … }` (use the `Pointer` width arm for `ISize` per anchor); **unsigned** (`UInt8`…`UInt64`, `USize`) inhabit `IntScalar { kind: Unsigned, width: … }`. Per-name std aliases + spelling-only `GroundingMap` rows are **omitted** (reversed-D2 hollow); unbounded `Nat`/`Int` are deferred until explicit fact-bundles. |

### `src/v4/extdeps/languages/machine_code.dag`

| Carrier | Classification | Ledger result |
|---------|----------------|---------------|
| `Isa` | 🟢 GREEN terminal | All five dissolution patterns fail: encoding/register/operand rules dispatch on ISA; `Symbol` would make the closed spec set stringly; not algebra; one ISA value has one versioned spec authority; ISA authorities are not one width-indexed carrier. Version authorities: `X86_64Isa.revision` names the Intel SDM revision, `Aarch64Isa.revision` names the Arm A-profile Architecture Reference Manual revision (`https://developer.arm.com/architectures/cpu-architecture/a-profile/docs`), and `RiscV64Isa.revision` names the RISC-V ISA manual release (`https://github.com/riscv/riscv-isa-manual/releases`). |
| `ByteOrder` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit reads byte order uniformly; `Bool`/`Symbol` erases order meaning; not algebra; one field has one order; two alternatives are not one `F<X>`. |
| `InstructionLength` | 🟢 GREEN terminal | All five dissolution patterns fail: tokenization/decoding consumes the instruction boundary; fixed and variable forms carry different payloads; not algebra; one decode point has one length discipline; payloads are heterogeneous. |
| `InstructionWord` | 🟡 TRACKED scaffold (`feature:fixed-cardinality-width-carrier`) | Fact placement fails; variant-is-data is partial because the bounded byte-count / Nat-indexed instruction-width primitive is missing. Named arrival: TASKS.md T-3 scalar/numeric stack lands the fixed-cardinality width carrier in `std/machine.dag` or `std/cardinality.dag`. Dissolve-on-arrival follow-up: replace this enum with that carrier and update the P4-3208 tag/row to 🟢, or remove it if no coproduct remains. |
| `RegisterClass` | 🟢 GREEN terminal | All five dissolution patterns fail: operand legality/cost/register consumers share register class; width alone cannot distinguish classes; integer/float aliases ground elsewhere but register class is ISA surface; one occurrence has one class; register semantics are heterogeneous. |
| `IndexScale` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit/address calculation share scale legality; an unconstrained numeric width tag admits invalid factors; not algebra; one scaled index has one factor; closed factors are not one family. |
| `AddressingMode` | 🟢 GREEN terminal | All five dissolution patterns fail: operand decode and memory-effect lenses share the classifier; variants have different required fields and a flat record admits invalid mixtures; not algebra; one operand occurrence has one address form; payloads are heterogeneous. |
| `Operand` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit/operand legality branch on operand shape; payloads are heterogeneous; instruction operand syntax is ISA surface; one operand is register/immediate/memory; payloads differ by category. |
| `ImmediateOperandTarget` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/branch-target analysis/emit distinguish literal and target immediates; `Bool`/`Symbol` erases target role; not algebra; one immediate slot has one target class; alternatives are closed. |
| `MemoryOperandConstraint` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit/memory effects share addressing-form constraints; each form carries different register/displacement constraints and a flat record admits impossible mixtures; not algebra; one memory slot constrains one form; payloads are heterogeneous. |
| `InstructionOperandSlot` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit/operand legality read production-slot classifier; slots carry category-specific constraints; not algebra; one slot accepts one operand category in this L-2 model; payloads are heterogeneous. |
| `OperandBinding` | 🟢 GREEN terminal | All five dissolution patterns fail: decode/emit/operand legality consume category-specific binding; variants carry different slot refs and payloads; not algebra; one decoded operand is register/immediate/memory; payloads are heterogeneous. |
| `MachineFidelityFeature` | 🟢 GREEN terminal | All five dissolution patterns fail: disassembly/emit/C5 diagnostics read feature classes; `Symbol` would make the boundary open; not algebra; one boundary decision names one class; feature classes are heterogeneous. |
| `MachineFidelityDisposition` | 🟢 GREEN terminal | All five dissolution patterns fail: modeled/normalized/fail-closed is the shared C5 disposition; non-modeled variants carry feature payloads and a flat record admits impossible combinations; not algebra; one feature has one disposition; payloads are heterogeneous. |
| `MachineIntegerOverflowBehavior` | 🟢 GREEN terminal | All five dissolution patterns fail: integer operation semantics/lowering read overflow behavior; `Bool` cannot distinguish wrap/trap/undefined; wrapping relates to fixed-width carrier behavior but trap/undefined are target operation semantics; overflow is separate from flag writes; alternatives are closed. |
| `FlagWriteDisposition` | 🟢 GREEN terminal | All five dissolution patterns fail: condition-code dependency consumers read this coordinate; `Bool` cannot distinguish architectural flags vs predicate-register writes; not algebra; one instruction has one flag-write disposition; alternatives are closed. |
| `MachineIntKind` | 🟢 GREEN terminal | Same dissolution posture as `RustIntKind` / `LeanIntKind`: ISA-modeled signed vs unsigned is a closed alternative for fixed-width machine integers, not a coordinate; not algebra; consumers branch uniformly. |
| `MachineIntWidth` | 🟢 GREEN terminal | Same dissolution posture as `RustIntWidth` / `LeanIntWidth` with `Bits128` extension: closed width ladder for ISA fixed-width integers through 128-bit class; not algebra; not a `Symbol` width tag. |
| `MachineScalar` | 🟢 GREEN terminal | Same dissolution posture as `RustScalar` `IntScalar` (`rust.dag`): kind×width bundles ISA scalar facts (**D2-REV**). Per-width `MachineI*` / `MachineU*` std aliases + spelling-only `GroundingMap` rows are **omitted** (reversed-D2 hollow). `AddressingMode` **signed** displacement slots use `MachineSignedDisplacement32` / `MachineSignedDisplacement64` — **width-only** records over encoding-class witnesses (`MachineDisplacement32WidthClass` / `MachineDisplacement64WidthClass` with singleton `DisplacementEncodingBits32 {}` / `DisplacementEncodingBits64 {}`); signedness is **nominal** in the carrier name (no `MachineIntKind` field — cannot spell `Unsigned` on these SDM signed-displacement paths). |
| `MachineDisplacement32WidthClass` | 🟢 GREEN terminal | Singleton encoding-class witness for 32-bit-class SDM displacement fields (`DisplacementEncodingBits32 {}` — empty record so v2’s `type` parser treats this as a sum arm, not a bare-alias RHS); not a bare `Int32` alias. |
| `MachineDisplacement64WidthClass` | 🟢 GREEN terminal | Singleton witness for 64-bit absolute linear-address displacement (`DisplacementEncodingBits64 {}`); same v2-parse shape as the 32-bit class. |
| `MachineSignedDisplacement32` | 🟢 GREEN terminal | D2-REV: SDM / ISA **signed** 32-bit-class displacement — single `width` witness; not isomorphic to the 64-bit absolute carrier; not a free `MachineIntKind` product. |
| `MachineSignedDisplacement64` | 🟢 GREEN terminal | D2-REV: **signed** 64-bit-class absolute linear-address displacement — single `width` witness; same posture as `MachineSignedDisplacement32`. |

### `src/v4/extdeps/languages/cpp.dag`

Non-Bool C++ scalar ladder in this slice: **partially forwarded** by **T29-CPP-SCALAR-FORWARD** below. The live slice now carries the C++ core integer scalar fact-bundle shape over `CppIntegerWidth`; remaining non-integer scalar refinement (`float` / `double` / `long double`, text/code-unit details, and per-spelling target-profile selection rows) stays tracked under 🟡 `feature:t4-cpp-scalar-ladder` (T-4 Phase-3 cpp-slice + T-25-core); dissolve on the owning cpp-slice arrival.

| Carrier | Classification | Ledger result |
|---------|----------------|---------------|
| `CppBool` | 🟡 staging (canonical-B decl-ref declared + v2-parse-verified; P2/E-6 consumer-gated — no same-PR consumer, canonical fold specified-not-realized); classifier = `CppScalar.BoolScalar` | **A-vs-B = B (operator-ratified, ruled-B).** The prior `CppBool = Bool` + `cpp_bool_grounding { spelling: \"bool\" }` shape is **retired**: a bare D2 alias plus a `{spelling}` label asserts identity instead of demonstrating it — the hollow form the machine-readable-inhabitance bar forecloses. The machine-readable grounding edge *declaration* `cpp_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` — a decl-ref referencing the single std authority (ISO C++ [basic.fundamental]: `bool` is exactly `true`/`false`, standard ops — nothing above the primitive to model) — is present + v2-parse-verified (0 diagnostics, v2 bootstrap gate). **P2/E-6: STAGING, not landed authority** — no same-PR consumer reads it; the canonical/coincidence fold consumer is specified-not-realized (`docs/modeling/grounding-worked-examples.md` §0). `cpp.dag`'s per-file `{spelling}` `GroundingMap` twin is retired (no spelling-only resolver rows post-D2-REV; cf. `machine_code.dag` / INVARIANTS §P2). |
| `CppIntegerSurfaceSpelling` / `CppTwosComplement` / `CppIntegerOverflowDisposition` / `CppScalar` | 🟢 terminal for the currently modeled C++ core integer fact-bundle shape; remaining per-spelling profile selection rows stay 🟡 under `feature:t4-cpp-scalar-ladder` | **T29-CPP-SCALAR-FORWARD.** This is the post-`#3338` forward reconciliation of `#3277`: preserve the operator-directed T-29 value (C++ integer widths come from the target ABI/data model) without restoring the stale `type CppBool = Bool` / spelling-map shape. `CppScalar` carries C++ scalar classifier facts as data: integer surface spelling, `CppTargetProfile`, `CppTargetDataModel`, ABI-selected `CppIntegerWidth`, signed two's-complement representation for signed integer scalars, and overflow/range disposition; `BoolScalar` carries the bool classifier while the canonical-B decl-ref grounding remains `cpp_bool_grounding`. The shape is a fact bundle, not a hollow alias: no `type CppInt = Int`, no `type CppBool = Bool`, and no local `{spelling}` `GroundingMap`. Signed and unsigned integer scalar arms are split so an unsigned scalar cannot carry the signed representation field by construction; later per-spelling rows must select a width from `CppTargetProfile` / `CppTargetDataModel` rather than guessing LP64/ILP32 constants in `cpp.dag`. |

## OS-1 — #3209 Coproduct Dissolution And Scaffold Record

This section preserves the Practice-4 ledger content moved out of
`src/v4/extdeps/file_system.dag` and `src/v4/extdeps/process.dag` during
the strict de-prose pass. The source files keep only path/scope/owns/
consumes/status, anchor lines, and terse concept/scaffold tags.

### Coproduct Classifications

All listed coproducts are **YELLOW, first-consumer-confirmed**. The
named first-consumer trigger is the T-4.5 OS realization seam unless a
more specific trigger is stated.

- `PathComponent`: POSIX `.` and `..` component semantics are distinct
  from ordinary filename spellings. Trigger: filesystem path parsing /
  resolution realization. Ledger: fact placement fails because component
  kind is consumed uniformly by path resolution; variant-is-data fails
  because current/parent are semantic facts and named spelling is
  payload; algebraic form is not applicable; dimensional fails because
  a component is exactly current, parent, or named; parameterized family
  fails because variants are not identical payloads over an enumerable
  set.
- `RelativePathHead`: non-current relative descendants can begin with
  parent traversal or named component. Trigger: filesystem path parsing /
  resolution realization. Ledger: fact placement fails because
  first-component admissibility is consumed by relative path resolution;
  variant-is-data fails because parent traversal and named payload are
  different facts; algebraic form is not applicable; dimensional fails
  because a non-current relative descendant head is parent or named;
  parameterized family fails because variants are not identical payloads
  over an enumerable set.
- `RelativePath`: current directory, current-prefixed descendant with a
  non-current continuation, and non-current relative descendant are
  distinct POSIX relative-path facts. Trigger: filesystem path parsing /
  resolution realization. Ledger: fact placement fails because the
  partition is consumed uniformly by path operations; variant-is-data
  fails because current-prefixed and non-current descendants carry
  different leading facts while sharing the non-current continuation
  head; algebraic form is not applicable; dimensional fails because a
  relative path is current directory, current-prefixed descendant, or
  non-current descendant; parameterized family fails because variants
  are not identical payloads over an enumerable set.
- `Path`: absolute-vs-relative is a POSIX root fact. Trigger:
  filesystem path parsing / resolution realization. Ledger: fact
  placement fails because root-ness is consumed by path resolution and
  operation realization; variant-is-data fails because a flat
  `absolute: Bool` duplicates the partition as a flag; algebraic form
  is not applicable; dimensional fails because a path is absolute or
  relative; parameterized family fails because variants are not projected
  over a separate payload family.
- `FileKind`: v4's bounded filesystem subset observes regular file,
  directory, or symlink. Trigger: first `file_kind` realization. Ledger:
  fact placement fails because file kind is consumed uniformly by
  readers/listing/link handling; variant-is-data fails because the
  variant is the file-kind fact and readlink target is a separate POSIX
  observation; algebraic form is not applicable; dimensional fails
  because one directory entry has one kind in this subset; parameterized
  family fails because variants are not mechanically identical payloads.
- `FileKindResolutionPolicy`: the boundary declares stat-like follow or
  lstat-like no-follow symlink observation. Trigger: `file_kind`
  realization. Ledger: fact placement fails because resolution policy is
  a request-boundary fact; variant-is-data fails because each variant is
  a distinct observation mode without shared payload; algebraic form is
  not applicable; dimensional fails because one request follows symlinks
  or does not; parameterized family fails because variants are not a
  projection over an independently modeled payload family.
- `Termination`: POSIX wait status is exit or signal. Trigger:
  fork/exec/wait realization. Ledger: fact placement fails because
  terminal status is consumed uniformly by wait/capture carriers;
  variant-is-data fails because exit and signal carry different payloads
  and a flat record admits impossible combinations; algebraic form is
  not applicable; dimensional fails because a terminal process ended by
  exit or signal; parameterized family fails because variants are not
  identical payloads.
- `CaptureSource`: capture can drain from a live spawned child or a
  waited child whose pipe authority was preserved. Trigger: capture
  realization. Ledger: fact placement fails because capture source state
  is consumed uniformly by capture; variant-is-data fails because each
  variant carries the lifecycle process authority and current or waited
  resource authority; algebraic form is not
  applicable; dimensional fails because one request uses live or
  post-wait authority; parameterized family fails because variants carry
  different lifecycle carriers.

### Refinement Scaffold Disposition

The following are intentionally tracked as YELLOW refinement scaffolds in
#3209 rather than expanded into structural value-refinement or witness
carriers: `ProcessId = Int`, `ExitCode = Int`, `SignalNum = Int`,
`NamedPathComponent { bytes: PosixByteString }`, `PosixArgument`,
`PosixEnvironmentName`, and `PosixEnvironmentValue`. The named trigger is
**T-25-core refinement substrate / T-30 fact-density gate**. Expanding
them now would be fact-bundle-rework-class work, outside the NOT-D2-held
de-prose scope.

## Part 2 — RATIFIED 2026-05-15 (cascade-closures — now in Part 1)

> **All three (U2, C3, C1) were ratified and are encoded — see Part 1.**
> The detailed rationale is retained below as the decision record.
> U2 carries the operator sharpening: cost/complexity is **total over
> the closed kernel** — a future language addition *cannot* bypass it
> (no per-feature opt-in); that is the no-eternal-maintenance property.

These are not new decisions — they are the logical closure of a RATIFIED
parent. Each is confirm-or-redirect, not a fresh fork.

### U2 — `cost.dag` and `complexity.dag` are two projections, not two engines

- **Why (cascade of U1):** "coercion cost = complexity" + the one-carrier
  rule mean these cannot be independent engines.
- **Recommended:** `complexity.dag` *consumes* `cost.dag`'s `SymbolicCost`
  and projects it to an asymptotic `ComplexityBound`; it never
  independently re-derives cost. Plus: verify the `SymbolicCost` lattice
  axioms at T-12 ratification (`UnknownCost`-as-top must satisfy the
  lattice laws — the `AbelianGroup<Nat>` precedent).
- **Tradeoff:** none of consequence; the alternative (two cost
  derivations) is the parallel-authority bug U1 forbids.
- **Encodes into:** `lens/cost.dag` + `lens/complexity.dag` headers
  (the consume-not-re-derive contract).

### C3 — `normalize` (T-8) dissolves exactly the 4 bounded sugar forms

- **Why (cascade of A1):** THESIS confirms `service / fn / type /
  operation` are the bounded surface sugar over the substrate.
- **Recommended:** `03_normalize.dag` dissolves exactly those 4 sugar
  forms into Node trees. It is **not** identity and **not** open-ended.
  Any new sugar form is a substrate extension = STOP.
- **Tradeoff:** none; this closes the "what does normalize do in a
  no-sugar language" tension — there *is* bounded sugar, exactly 4.
- **Encodes into:** `compiler/03_normalize.dag` header + `TASKS.md` T-8.

### C1 — T-9 infer is the bounded *Find* phase of the one homomorphism

- **Why (cascade of A1 + A2 + U1):** A1 forbids a separate recursive
  `InferredNode`; A2 makes termination Tier-1; U1 makes infer the *Find*
  phase over the one carrier.
- **Recommended:** the homomorphism search is **bounded structural
  unification** over the (finite, by A1/A2) candidate space — not an
  open search. Empty result ⇒ a helpful Diagnostic (`feedback_no_engine`),
  never a fabricated coercion. The v2 12-file split cannot recur
  (its cause — a separate recursive inferred type — is structurally gone).
- **Tradeoff:** restricts inference to the decidable structural fragment;
  that is the decidability invariant, not a limitation.
- **Encodes into:** this section + `TASKS.md` T-9;
  `compiler/04_infer.dag` is a boundary index only.

---

## Part 3 — open forks

> **Status 2026-05-15 — Part 3 CLOSED.** All open forks ratified and
> encoded (see Part 1): B1 (std-level fold — operator confirmed),
> B2→B2-OMNI, B3, B4→IR-1, C4, C5. The per-header encoding pass is
> complete. node.dag-contract items: **K-1** (opaque Symbol — modeled) +
> the **canonical-form clause** (now **B1-CANON** — operator-DESIGNED &
> ratified in the node.dag header 2026-05-15, NOT a worker-relay choice:
> a SEPARATE authority, NOT subsumed by A1, B1 depends on it; the
> 6-point contract is fixed) + **`content_hash = merkle_fold ∘
> canonical`** which T-1 IMPLEMENTS to the B1-CANON contract (it does
> not design it). Nothing in Part 3 remains PROPOSED.

### B1 — One content-addressing scheme (ELEVATED: load-bearing for A3)

- **Tension:** T-15 fixed-point hash, T-20 pinned bootstrap binary, T-21
  unchanged-subgraph detection, **and the A3 reproduction/surfacing
  check** all need content-addressing. Three schemes = drift.
- **Recommended:** a single canonical content-hash over the canonical
  Node structure. **Correction (3248138046):** A1 does NOT make Node
  canonical — A1 ratifies recursion / axis-closure / generics /
  termination only. Canonical form is a SEPARATE explicit authority —
  the **canonical-form clause**, now **B1-CANON**,
  operator-DESIGNED & ratified in this ledger 2026-05-15
  (pulled back from worker delegation because every consumer rests on it).
  B1 *depends on* that clause; it is not subsumed by A1. B1-CANON fixes
  the total deterministic normal form as a 6-point contract: (1)
  post-normalize domain; (2) α-equivalence is a `merkle_fold` property,
  NOT a canonical-Node rewrite (no De Bruijn Node carrier — A1; the
  fold hashes a bound occurrence binder-relatively, a free Symbol by
  opaque K-1 identity; the α-fact lives in the fold's `Hash` codomain,
  representable — review-corrected 2026-05-16); (3) **child order is
  unconditional
  `children`-list order** (B-3 — an amendment 2026-05-16 to this
  2026-05-15 B1-CANON clause, not a separate ratification) — the
  commutative-quotient is structurally foreclosed (no order on opaque
  `Hash`; algebra unreachable from the consumes-nothing root, the D1
  cycle class), the correct closure of a decidable system not a defect;
  T-2/std/algebra.dag unaffected (point 3 was mislayered, not
  deficient); over-fire on a permuted record is safe, the unsafe
  direction (missed change) is foreclosed by list order being total;
  (4) recursion by-reference, name-ref hashed by Symbol identity,
  never inlined (totality on `type T=…T…`); (5) structural catamorphism,
  decidable, Merkle ⇒ shared sub-Nodes hash once (dedup free); (6) no
  incidental ordering. T-1 IMPLEMENTS `content_hash`; it does not design
  it. Defined **once**, consumed by T-15/T-20/T-21 + the A3 check.
- **Tradeoff / settled:** *where it lives* — `std`-level: a pure fold
  over the canonical Node, not `workflow`-level (operator-confirmed).
- **Encodes into:** `std/node.dag` (canonical-form clause) + the chosen
  hash-operation owner header + cross-refs in T-15/T-20/T-21.

### B2 — Grammar is declarative data; the parser is a generic walker

- **Tension:** if grammar is a procedural recognizer, tokenize/parse are
  hand-shaped and gameable.
- **Recommended:** grammar lives as **declarative production data** in
  `extdeps/languages/*.dag` (T-4); `01_tokenize`/`02_parse` (T-6/T-7) are
  generic walkers driven by typed `LexRules` / `Grammar` data, not
  per-language procedural code or stringly `Node` variant tags. Derived
  `Node` projections are optional views, not the authority.
- **Tradeoff:** a declarative-grammar substrate is more work to design
  than a hand recognizer, but a hand recognizer is the gameable path and
  re-fragments per language.
- **Encodes into:** `extdeps/languages/*` headers + `01_tokenize.dag` +
  `02_parse.dag` + `TASKS.md` T-6/T-7.

### B3 — The type signature is the single effect authority

- **Tension:** effects appear in three places (the effect lens T-13,
  coordination effect-types T-4.8, `unenumerated_effects.dag`).
- **Recommended:** the **type signature is the one effect authority**.
  The effect lens *reads* effects from the signature. Coordination effect
  kind is either derived from that signature or, until T-13 can read it,
  a single yellow scaffold fact; it is never a family of empty marker
  carriers or wrapper-selected effect types.
- **Tradeoff:** requires coordination's effect-types to be expressed as
  signature facts the lens reads, not a standalone enum — slightly more
  modeling, but it's the single-authority discipline (same class as the
  AssertKind/testgen reconciliation already done).
- **Encodes into:** `lens/effect.dag` + `extdeps/coordination.dag` +
  cross-ref to `test/claim/impossible_bug/unenumerated_effects.dag`.

### B4 — There is no separate IR; "IR" is InferredTree; the seam is T-11

- **Tension:** T-10 emit / T-22 eval / T-11 specialization — where does
  target-specificity enter, and is there an IR type?
- **Recommended:** **no separate IR type** (A1: no new recursive type).
  The "target-agnostic IR" *is* InferredTree (Node + inferred
  discriminant). emit (T-10) and eval (T-22) both consume it;
  target-specificity enters **only** at T-11 per-target realization
  tables (the homomorphism's realization per target, read from
  `extdeps/languages/*`).
- **Tradeoff:** none vs A1 — this is the consistent reading; the
  alternative (a distinct IR type) is the bounded-kernel violation.
- **Encodes into:** `compiler/05_emit.dag` + `compiler/05_eval.dag` +
  `TASKS.md` T-10/T-11/T-22.

### C4 — On-disk emitted artifacts are checked projections, not authority

- **Tension:** `ci.yml` must exist on disk (GitHub reads it) yet
  `feedback_no_generated_code_on_disk` forbids editable generated code.
- **Recommended:** same machine as A3. Emitted artifacts are build-dir
  transient where possible. Where one must be committed (`ci.yml`), the
  committed file is guarded by a **committed == emit(source)**
  reproduction check (the A3 early-surfacing machine): divergence is
  conspicuous and STOP-routed. The `.dag` is authority; the on-disk file
  is a *checked projection*, never editable authority.
- **Tradeoff:** this is the honest reconciliation (no-generated-code =
  "no editable generated *authority*", not "no generated bytes on
  disk"); same shape as A3, so it's a confirm of an already-ratified
  pattern more than a new fork.
- **Encodes into:** `workflow/ci.dag` + `STRUCTURE.md` (the C4 pattern) +
  `workflow/bootstrap.dag` (the trampoline).

### C5 — Ingest is emit's structural inverse on the lossless core only

- **Tension:** the R4 arbitrary-ingestion roundtrip claim needs ingest to
  be `emit⁻¹`, or RoundTrips is hope not construction.
- **Recommended:** ingest and emit are the two directions of **one**
  language-model homomorphism (T-4 direction-agnostic). `RoundTrips`
  holds by construction **on the lossless core**. Outside it (a target
  feature with no Node equivalent), ingest is **partial → a fail-closed
  Diagnostic** (`feedback_no_engine`: can't ingest losslessly ⇒ surface,
  never a lossy guess).
- **Tradeoff:** this *bounds* the arbitrary-ingestion claim honestly
  (lossless core + fail-closed boundary) rather than overclaiming total
  invertibility. That honesty is the point, not a weakness.
- **Encodes into:** `extdeps/languages/*` + `extdeps/formats/*` headers +
  a C5 note in `v4-close-interrogation.md`.

---

## Part 4 — Lower-cascade (resolve before the owning task; non-blocking)

Recommended defaults; not blocking T-1 or the spine. Confirm in passing
or defer to the task.

- **T-4.6 JSON number model** — recommend **arbitrary-precision by
  default, opt-in IEEE-754** (preserves big-int roundtrip; RoundTrips is
  load-bearing per C5).
- **T-4.7 Server/Client Components** — recommend **effect-typed, not a
  structural distinction** (consistent with B3; SC/CC differ by effect).
- **T-4.8 effect-type-set** — **the question dissolves under B3
  (correction, 3248138059).** B3 made the type signature the *single*
  effect authority; a terminal HTTP/queue/stream/pubsub enum would be
  exactly the parallel effect taxonomy B3 forbids (P2). So there is no
  terminal effect-type enum to close: `CoordinationEffectKind` is only a
  yellow scaffold until `lens/effect.dag` reads the effect kind from the
  signature. Non-default; resolved by B3.
- **T-18 bounded-coverage** — coverage is over the **finite substrate
  cross-product**; infinite inhabitant domains are sampled by declared
  generators, never enumerated (keeps coverage decidable).
- **T-20 v2-interpreter pre-flight** — **a check, not a decision**: audit
  frozen v2's `run` opcode support against what `bootstrap.dag` needs
  *before* T-20 starts. If v2 can't interpret a needed construct, that's
  a STOP (the seed is frozen — can't grow it).
  - **Parse-surface sub-check RESOLVED 2026-05-15 (see PARSE-1):**
    constructive pre-flight run. Result: frozen v2's parser accepts the
    full ratified kernel shape *with fn bodies* (40k-line `src/v2`
    corpus + kernel probe, zero syntax errors — the 65 `src/v2` errors
    are 100% post-parse unresolved-import/derived-cycle, NOT syntax) but
    **rejects bodiless `fn`** (`expected LBrace`). Operator ratified that
    bodiless fn is not needed pre-self-parse, so this is **not a STOP**:
    bridge contracts stay comment-prose; the fail-closed parse-class
    gate over `src/v4` is safe. Residual T-20 surface = the `run`
    opcode/eval audit (unchanged), not the parser.

### T-4.6 — Practice-4 closed sums on external vocabularies (Lane C formats)

Rubric (`docs/modeling-discipline.md` “For reviewers” §4): large closed
sums keyed to an external spec carry an explicit 🟢/🟡/🔴 disposition and
a named YELLOW trigger in this ledger (Practice 9 — not in `Scope:`).

| ID | Carrier / slice | Disposition | YELLOW trigger (omit if GREEN) | Encoded in |
|----|------------------|-------------|--------------------------------|------------|
| **T-4.6-CSV** | RFC 4180 row/document carriers (`CsvRow` Peano depth = per-row field arity) + **`CsvRfc4180*` product witness records** (comma / DQUOTE / **CRLF record separator** / **TEXTDATA vs HTAB** outside unquoted fields / header / `text/csv` registration anchor — no new coproducts; witnesses state **normative CRLF** between records, not LF-as-record-boundary tolerance; **MIME `charset` is transport-layer** — not an RFC 4180 §2 witness in this slice); **singleton `CsvRfc4180*` witness types** are **inhabitance-only** (no Bool-encoded RFC negation); optional **ending CRLF** after the last record (named on **`CsvRfc4180LastRecordEndingLineBreakOptional`**) is **not** **trailing-COMMA** tolerance (comma after the last field is **forbidden**, RFC 4180 §2) | 🟡 | Cross-row/header-vs-body arity mesh is **operator-pending** until a ruling + substrate hook (TASKS.md T-4.6); until then validation may fail-closed without a claimed total mesh. | `extdeps/formats/csv.dag` |
| **T-4.6-JSK** | `JsonSchemaAdditionalKeywordKey` — closed sum for Draft **2020-12** keywords exercised by this slice, grounded in the **default dialect meta-schema** (`https://json-schema.org/draft/2020-12/schema`), which composes the Core, Applicator, Unevaluated, Validation, meta-data, format-annotation, and content vocabularies (see `$vocabulary` therein). Typed `instance_*` slots intentionally favor **Core**-shaped modeling; the sum carries applicator/validation/meta/content keywords from that same meta-schema bundle, including Appendix A transitional **`definitions`** / **`dependencies`**. | 🟡 | Any ratified draft-2020-12 vocabulary or meta-schema change adds/renames/removes a keyword this enum carries ⇒ extend **JsonSchemaAdditionalKeywordKey** + amend **T-4.6-JSK** in the same change-set. | `extdeps/formats/json_schema.dag` |
| **T-4.6-OAS** | OpenAPI 3.1 document/object carriers in `openapi.dag` | 🟡 | Upstream OAS 3.1 normative or meta-schema change forces a modeled fixed field, deferred-key family, or path-template wire rule to migrate — ship with operator-visible reconcile (D5). | `extdeps/formats/openapi.dag` |

#### T-4.6-P4 — Per-coproduct classification ledger (PR #3205)

Authoritative Practice-4 ledger rows: dissolution stance + named YELLOW
trigger (if YELLOW). Each coproduct in the cited `.dag` carries **only**
the in-file `// <emoji> coproduct dissolution — DECISIONS.md <ID>` tag
(operator directive 2026-05-17, vivid-carp-207).

| ID | Coproduct | Disp | Ledger (dissolution / trigger) |
|----|-----------|------|--------------------------------|
| **T-4.6-P4-CsvHeaderPresence** | `CsvHeaderPresence` | 🟡 | Header present vs absent; cross-row arity couples to **T-4.6-CSV** mesh (operator-pending). |
| **T-4.6-P4-CsvRow** | `CsvRow` | 🟡 | Peano spine = per-row field count; mesh with **T-4.6-CSV** / **T-4.6-P4-CsvHeaderPresence**. |
| **T-4.6-P4-JsonSchemaCoreTypeName** | `JsonSchemaCoreTypeName` | 🟡 | Draft 2020-12 `type` keyword closed vocabulary for this slice; spec keyword churn extends the sum + amends this row. |
| **T-4.6-P4-JsonSchemaTypeSlot** | `JsonSchemaTypeSlot` | 🟡 | Optional / single / union type shapes over **T-4.6-P4-JsonSchemaCoreTypeName**; **`JsonSchemaTypeUnion`** uses **`first` + `rest:Set`** so the Draft 2020-12 `type` array is **non-empty** by construction. **`first` ∈ `rest`** or duplicate wire members ⇒ **parse/Diagnostic** (T-4 brief). |
| **T-4.6-P4-JsonSchemaItemsSlot** | `JsonSchemaItemsSlot` | 🟡 | `items` absent vs schema subtree; validation/type-derivation for `items` keywords deferred (T-4.6 header). |
| **T-4.6-P4-JsonSchemaPropertiesSlot** | `JsonSchemaPropertiesSlot` | 🟡 | `properties` absent vs map; key uniqueness is `Map` + parse/Diagnostic (collection.dag). |
| **T-4.6-P4-JsonSchemaRequiredSlot** | `JsonSchemaRequiredSlot` | 🟡 | `required` absent vs `Set<String>`; set semantics vs instance `properties` keys validated at ingest (T-4 brief). |
| **T-4.6-P4-JsonSchema** | `JsonSchema` | 🟡 | `true` / `false` / instance shell; instance keyword surface split across fixed fields + **T-4.6-JSK** map — full validation deferred (T-4.6). |
| **T-4.6-P4-OpenApiOpenapiObjectDeferredKey** | `OpenApiOpenapiObjectDeferredKey` | 🟡 | Fixed-field deferrals on root OpenAPI Object; new OAS fixed field ⇒ extend sum + **T-4.6-OAS**. |
| **T-4.6-P4-OpenApiOperationDeferredKey** | `OpenApiOperationDeferredKey` | 🟡 | Operation Object deferred members; OAS adds/reorders modeled deferrals ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiPathItemDeferredKey** | `OpenApiPathItemDeferredKey` | 🟡 | Path Item deferred members + `$ref` slot; OAS Path Item churn ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiPathItemOrReference** | `OpenApiPathItemOrReference` | 🟡 | Path Item inline vs **`$ref`**; used for **`webhooks`** map values and **`components.pathItems`** — **not** for root **`paths`** patterned map (**Paths** values are **`OpenApiPathItem`**; Path Item **`$ref`** is **`OpenApiPathItemDeferredKey`**, **T-4.6-OAS**). |
| **T-4.6-P4-OpenApiComponentsDeferredKey** | `OpenApiComponentsDeferredKey` | 🟡 | Components deferred map keys (fixed fields **not** modeled here — e.g. **`pathItems`** lives on **`OpenApiComponents.path_items`** / **T-4.6-P4-OpenApiComponentsPathItemsSlot**); Components object spec churn on *these* keys ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiComponentsMapKey** | `OpenApiComponentsMapKey` | 🟡 | Nominal key for **`components.schemas`** / **`components.pathItems`** map entries; OAS Components fixed-field key regexp `^[a-zA-Z0-9\.\-_]+$` is **parse/Diagnostic** (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiMediaTypeDeferredKey** | `OpenApiMediaTypeDeferredKey` | 🟡 | Media Type deferred members; encoding/example surface churn ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiHeaderDeferredKey** | `OpenApiHeaderDeferredKey` | 🟡 | Header Object deferred members (style / explode / example / examples); excludes query-only **`allowEmptyValue`**; **`deprecated`** is **`OpenApiHeaderDeprecatedSlot`** on **`OpenApiHeaderObject`**; **`content`** is first-class on **`OpenApiHeaderPayloadSlot`** (XOR with **`schema`**). OAS Header churn on these deferrals ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiLinkDeferredKey** | `OpenApiLinkDeferredKey` | 🟡 | Link Object deferred members; Link spec churn ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiLinkTarget** | `OpenApiLinkTarget` | 🟡 | OAS Link Object **`operationId` XOR `operationRef`** — exactly one target identity; carried as `OpenApiLinkObject.target` (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiHttpMethod** | `OpenApiHttpMethod` | 🟢 | OAS Path Item operation field key set is the closed eight verbs `get`…`trace`; it intentionally excludes HTTP `CONNECT` even though shared `std/network.dag` `HttpMethod` includes it. |
| **T-4.6-P4-OpenApiOptionalString** | `OpenApiOptionalString` | 🟢 | Present/absent optional string field idiom over OAS optional strings. |
| **T-4.6-P4-OpenApiHeaderRequiredSlot** | `OpenApiHeaderRequiredSlot` | 🟢 | Required boolean slot on Header Object (spec-bounded two-way). |
| **T-4.6-P4-OpenApiHeaderDeprecatedSlot** | `OpenApiHeaderDeprecatedSlot` | 🟢 | Header Object **`deprecated`** optional boolean (Parameter-parity fixed field on **`OpenApiHeaderObject`**, **T-4.6-OAS**). |
| **T-4.6-P4-OpenApiHeaderPayloadSlot** | `OpenApiHeaderPayloadSlot` | 🟡 | Header Object **`schema` XOR `content`** (OAS 3.1 mutual exclusion); **`content`** arm carries exactly one media-type entry via **`OpenApiHeaderContentSingle`** (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiHeaderOrReference** | `OpenApiHeaderOrReference` | 🟡 | Inline Header vs `$ref`; reference resolution + Reference Object parity triggers **T-4.6-OAS**. |
| **T-4.6-P4-OpenApiLinkOrReference** | `OpenApiLinkOrReference` | 🟡 | Inline Link vs `$ref`; same reference substrate as **T-4.6-P4-OpenApiHeaderOrReference**. |
| **T-4.6-P4-OpenApiContactSlot** | `OpenApiContactSlot` | 🟡 | Info Object `contact` optional subtree; OAS Contact Object churn ⇒ extend Contact fields + reconcile row. |
| **T-4.6-P4-OpenApiLicenseUrlOrIdentifierSlot** | `OpenApiLicenseUrlOrIdentifierSlot` | 🟡 | License Object `url` XOR `identifier` (+ neither) — mutual exclusion modeled; `url` consumes `UriReference` because OAS URL fields admit relative references unless a field overrides that rule. OAS License churn ⇒ amend row. |
| **T-4.6-P4-OpenApiLicenseSlot** | `OpenApiLicenseSlot` | 🟡 | Info `license` optional subtree; couples to **T-4.6-P4-OpenApiLicenseUrlOrIdentifierSlot**. |
| **T-4.6-P4-OpenApiMediaTypeSchemaSlot** | `OpenApiMediaTypeSchemaSlot` | 🟡 | Media Type `schema` present vs absent; couples to `OpenApiSchema`. |
| **T-4.6-P4-OpenApiResponseContentSlot** | `OpenApiResponseContentSlot` | 🟡 | Response `content` optional map; media-type map churn ⇒ amend modeling + row. |
| **T-4.6-P4-OpenApiHeadersSlot** | `OpenApiHeadersSlot` | 🟡 | Response `headers` optional map of **T-4.6-P4-OpenApiHeaderOrReference**. |
| **T-4.6-P4-OpenApiLinksSlot** | `OpenApiLinksSlot` | 🟡 | Response `links` optional map of **T-4.6-P4-OpenApiLinkOrReference**. |
| **T-4.6-P4-OpenApiResponseOrReference** | `OpenApiResponseOrReference` | 🟡 | Inline Response vs `$ref`; Responses object value shape (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiStatusDigit** | `OpenApiStatusDigit` | 🟢 | Decimal digit spine for explicit HTTP status codes (0–9). |
| **T-4.6-P4-OpenApiStatusHundreds** | `OpenApiStatusHundreds` | 🟢 | Hundreds class spine (`1xx`…`5xx`) for explicit status codes. |
| **T-4.6-P4-OpenApiResponseKeyWildcardKind** | `OpenApiResponseKeyWildcardKind` | 🟢 | OAS wildcard response key buckets `1XX`…`5XX` (uppercase `X` wire discipline in model). |
| **T-4.6-P4-OpenApiResponseKeyNonDefault** | `OpenApiResponseKeyNonDefault` | 🟡 | Explicit HTTP status vs wildcard response keys (excludes wire key **`default`**); used for **`OpenApiResponses.primary_response.status_key`** and **`additional_by_status`** map keys (**T-4.6-OAS**). OAS Responses key grammar churn ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiResponsesDefaultSlot** | `OpenApiResponsesDefaultSlot` | 🟡 | Optional **`default`** response arm split out from **`OpenApiResponses.primary_response`** / **`additional_by_status`** (**T-4.6-OAS**); spec churn ⇒ extend sum + row. |
| **T-4.6-P4-OpenApiResponseKey** | `OpenApiResponseKey` | 🟡 | `default` / explicit / wildcard key forms + overlaps per Responses Object (**T-4.6-OAS**). **`OpenApiResponses`:** `Map` forbids duplicates **within** `additional_by_status`; **`primary_response.status_key` ∉ keys(`additional_by_status`)** is **parse/Diagnostic** (T-4 brief). |
| **T-4.6-P4-OpenApiPathsSlot** | `OpenApiPathsSlot` | 🟡 | Root `paths` present vs absent channel (**T-4.6-OAS**); empty vs missing distinguished structurally. |
| **T-4.6-P4-OpenApiComponentsSchemasSlot** | `OpenApiComponentsSchemasSlot` | 🟡 | `components.schemas` map present vs absent; keys are **T-4.6-P4-OpenApiComponentsMapKey**; schema registry churn (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiComponentsPathItemsSlot** | `OpenApiComponentsPathItemsSlot` | 🟡 | OAS 3.1 **`components.pathItems`** present vs absent; keys are **T-4.6-P4-OpenApiComponentsMapKey**; values are **T-4.6-P4-OpenApiPathItemOrReference** (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiComponentsSlot** | `OpenApiComponentsSlot` | 🟡 | `components` subtree present vs absent at document root coordinate (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiWebhooksSlot** | `OpenApiWebhooksSlot` | 🟡 | `webhooks` map present vs absent at document root coordinate (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiDocumentRoot** | `OpenApiDocumentRoot` | 🟡 | OAS §3.1: document **MUST** expose at least one of **`paths`**, **`components`**, or **`webhooks`** — encoded as three mutually exclusive obligation-first carriers + optional sibling slots (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiSchemaOasMemberKey** | `OpenApiSchemaOasMemberKey` | 🟡 | OAS Schema Object fixed members **outside** the **`core: JsonSchema`** authority carried here — extend sum + row when OAS adds a modeled OAS-only schema field (**T-4.6-OAS**). |
| **T-4.6-P4-OpenApiSpecificationExtensionKey** | `OpenApiSpecificationExtensionKey` | 🟡 | OAS Specification Extensions: wire keys **MUST** begin **`x-`**; reserved **`x-oai-`** / **`x-oas-`** families are **disjoint arms** vs generic vendor keys (**T-4.6-OAS**). Keys not starting **`x-`**, or **`OpenApiSpecExtensionVendor`** with **`after_x_hyphen`** whose **`x-` +** form still prefixes **`x-oai-` / `x-oas-`** (mis-route) ⇒ **parse/Diagnostic** (T-4 brief). **Wire recon (ingest):** **`OpenApiSpecExtensionOaiReserved`** ⇒ literal **`x-oai-`** concatenated with **`remainder`**; **`OpenApiSpecExtensionOasReserved`** ⇒ **`x-oas-`** + **`remainder`**; **`OpenApiSpecExtensionVendor`** ⇒ **`x-`** + **`after_x_hyphen`**. |
| **T-4.6-P4-OpenApiSchema** | `OpenApiSchema` | 🟡 | Schema Object wire surface = **`core: JsonSchema`** + **`openapi_schema_oas_members`** + **`openapi_schema_specification_extensions`** (keys = **T-4.6-P4-OpenApiSpecificationExtensionKey**) + **`openapi_schema_deferred_keyword_members`** (`Map<String, JsonValue>`) for **custom vocabularies**, unenumerated JSON Schema keywords, and forward-compatible keys **not** absorbed by the first three partitions (**T-4.6-OAS**). Duplicate keys across partitions or reserved-wire mis-routing ⇒ **parse/Diagnostic** (T-4 brief). |
| **T-4.6-P4-OpenApiOpenapiField3_1** | `OpenApiOpenapiField3_1` | 🟡 | OAS §4.1 `openapi` semver for the **3.1 feature line**: **`OpenApiOpenapi31Patch { patch: Nat }`** encodes wire **`3.1.{patch}`** (any patch this `Nat` can spell — aligns with “tooling SHOULD accept all **`3.1.*`**”). New **`3.2.*`** / major lines ⇒ extend this sum + **T-4.6-OAS** row (same change-set). |

`JsonSchemaAdditionalKeywordKey` uses row **T-4.6-JSK** (same coproduct).
**`OpenApiOpenapiField3_1`:** optional Practice-9 item-4 **DECISIONS cite** on the `type` line in `openapi.dag`; **N ≥ 2** coproducts still require the emoji dissolution tag + matching **§T-4.6-P4** row.

---

## D2 REVERSAL + FACT-BUNDLE RESEED — RATIFIED (2026-05-17)

> Operator-declared and **RATIFIED 2026-05-17**. This section
> **supersedes D2** (and its downstream D4). The operator declared D2
> wrong, ratified the fact-bundle principle, and dispatched the Phase-1
> execution (PR #3224) — so the decision is ratified, not awaiting a
> merge vote. This PR (#3223) is the ledger encoding of that ratified
> decision; #3224 executes Phase 1 against it.

### What reversed

D2 ratified the **alias-identity inhabitance form**: a language `extdeps`
file declares each primitive as `type <Lang>X = StdX` (e.g.
`type RustI32 = Int32`), the algebra "flowing through the alias." D4
homed the shared D2-resolver type (`GroundingMap`) in
`extdeps/languages/resolver.dag`.

The operator declared D2 **wrong** (verbatim: *"D2 is wrong — i was
misunderstanding the purpose of this alias"*). A bare alias is not
modeling: it asserts an *unproven* identity between our carrier and an
external language's, while reading **zero facts** from that language's
spec. `type RustI32 = Int32` models nothing about Rust.

### Root cause — an enforcement gap, not a missing principle

`MODELING.md` **M1** ("types decompose into facts") and **M3** ("extdeps
model specs") already mandate fact-modeling. The principle was not
missing. Two failures let the alias through:

1. **`INVARIANTS.md` P1:42** (`numeric_aliases_align_to_refinements`)
   forbids a *parallel `OrderedRing<Word*>` substrate* — do not
   re-declare the algebra per language. D2 read this as "therefore
   bare-alias the whole carrier," conflating *algebra duplication*
   (correctly forbidden) with *carrier fact-modeling* (wrongly dropped).
2. **A hollow alias is invisible to every structural gate.**
   `type RustX = StdX` is structurally minimal — nothing for a reviewer
   or checker to flag. The api-review rubric (`INVARIANTS.md` +
   `docs/modeling-discipline.md`) checks duplicate authority and coproduct
   dissolution; it has no **fact-density** check. Aliasing passes review
   *because* it is hollow.

### The replacement — fact-bundle modeling

Operator-stated principle (2026-05-17), governing the reseed:

- **Model the facts. Always.** Deduplicate two models into one **only**
  when their identity is **proven** — identity is an evidenced claim,
  never an assumed default.
- **Invent or reuse — a bare alias is neither.** To model an external
  primitive you either *invent* a fact for it (read from that system's
  spec) or *reuse* a `std/` fact (with cited evidence of identity).
- **External defaults to separate; internal defaults to reuse.**
  `extdeps/` models systems we do not control and do not fully know →
  default to separate, honest, accumulated modeling; reuse `std/` only
  on evidenced identity. Internal code (compiler layers, our own
  substrate) → we control both sides, identity is usually *known* →
  default to reuse `std/`, still naming the evidence. Keeping
  `RustSignedness` separate until Rust's spec is checked is honest
  modeling, not a duplicate-authority violation.

### "Coincide" — defined once

Two models **coincide** when their **canonical `Node` groundings are
structurally equal** — equal under `node.dag`'s B1-CANON contract
`content_hash = merkle_fold ∘ canonical`. This is the *single* definition
of the word across v4: the U1 homomorphism, the T-9 coercion fold, and the
`docs/modeling/grounding-worked-examples.md` companion all use exactly it.
(The T-9 coercion fold is the **verification facet** of the derived
homomorphism — the fold that checks a candidate map preserves structure;
see `docs/modeling-discipline.md` "The three facets" and
`docs/thesis/the-derived-homomorphism.md`.)
Three consequences pin it down:

- **Mechanical, not free-form semantic equivalence.** Coincidence is
  decided by a zip-fold over two groundings plus a content-hash compare —
  decidable by construction (U1 no-engine discipline, T-9 / C1 Find is
  decidable over the closed declared candidate set). It is never a
  judgement that two things "mean the same"; it is structural equality of
  canonical `Node`s.
- **Requires a shared vocabulary.** Two groundings are comparable only if
  both are expressed in the *same* `std/` primitive vocabulary (authority:
  `std/algebra.dag` plus the T-3 shared-fact vocabulary — signedness,
  representation, the numeric stack). Facts are *sourced* independently —
  each from its own spec — but must be *expressed* in one vocabulary, or
  two structurally-different `Node` trees would denote the same fact and
  the mechanical compare would wrongly report "not equal." The shared
  vocabulary is what makes the fold both mechanical *and* complete.
- **Proven coincidence is the ONLY licence to deduplicate.** A fact-bundle
  collapses to a `std/` carrier only where coincidence is *proven*; a bare
  alias asserts coincidence without proving it — the D2 error.

This definition *reconciles* the existing U1 and T-9 decision rows rather
than rewriting them: both already mandate the no-engine, decidable form;
"coincide" is now the named term for the structural equality they assume.

### The phased plan

- **Phase 0 — stop the bleeding. DONE 2026-05-17.** `#3219`
  (`resolver.dag`, the D2-resolver encoding-half) closed; `#3208`
  (lean/machine_code, D2-resolver-shaped) held. Forward-fix, not revert.
- **Phase 1 — the doc keystone (rubric + enforcement).** Amend
  `INVARIANTS.md` P1:42 so "aligns to refinements" means *the language
  carrier is a fact-bundle coinciding with the std carrier*, not a bare
  alias. Promote `MODELING.md` M1 to a named first-class rule. Add a new
  Practice to `docs/modeling-discipline.md` — "model facts, not aliases"
  — with a worked good example (fact-bundle) and bad example (bare
  alias); since the api-review rubric reads that doc, a documented
  bad-example arms every review against the hollow alias. That is the
  *first* enforcement tier — but it is reviewer-applied, i.e.
  convention-level until a structural/API gate exists. Phase 1 ships that
  documented bad-example: the convention-tier enforcement that the
  `docs/modeling-discipline.md` api-review practice checks. The stronger
  *structural* tier — a generated **fact-density / hollow-alias gate** —
  is **its own foundation task, `T-30`** (a sibling of P1-KEYSTONE, NOT
  folded into `docs/modeling-discipline.md`), and it is a **hard
  prerequisite of T-4**: the per-language fact-bundle rework does not
  begin until the structural gate exists.
  This resolves a real gap by a dependency edge, not a phase argument. D2
  slipped through *two* failures (see root cause above): an **inverted**
  `P1:42` — misread as *licensing* the alias — and the **absence** of any
  bad-example for review to fail against. Phase 1 corrects both — it
  re-points `P1:42` and adds the bad-example, net-new versus the pre-D2
  state. But a documented bad-example is *convention* enforcement, and
  convention is exactly what let D2 through; reworking every per-language
  file under convention-tier-only enforcement would re-expose that
  surface. So the structural gate is **not** a loose follow-up and **not**
  sequenced by phase number — it is `T-30`, with a hard edge `T-30 → T-4`,
  landing *before* the per-language reseed by construction. The documented
  bad-example is the **interim floor only**: it holds the line until
  `T-30` lands; it is explicitly *not* the enforcement the per-language
  rework runs under.
  Reverse the D2/D4 rows. Rewrite the
  `TASKS.md` T-4 authoring contract.
- **Phase 2 — rework the entire T-## program** around fact-bundle
  modeling, first-class, plus the `std/` substrate axes the model needs
  that do not exist yet: signedness, representation, and an
  **exact-real carrier plus a physical-quantity carrier** — surfaced by
  the SPICE worked example, where a node voltage is an exact real value
  carrying a physical quantity (volts) that the approximate `f64` carrier
  cannot ground without declared loss. (Rational is at most a narrower
  inhabitant of the exact-real carrier, not the carrier itself.) The
  structural fact-density / hollow-alias gate is **not** a Phase-2 item —
  it is `T-30`, a hard prerequisite of T-4 (see Phase 1).
- **Phase 3 — execution.** Per-task / per-language / per-compiler-layer
  modeling rework, Rust exemplar first, under the Phase-1 rubric.

### Impact-map disposition (the close/revert/rework list)

`extdeps/languages` files and the merged D2 commits were classified
(grep-verified). The unwind is **forward-fix / supersession**, never
`git revert` of the merged commits (#3195 D2, #3216 D4/D5) — that would
clobber surviving language vocabulary and the `std/` numeric core.

- **Orthogonal — untouched:** the `std/` numeric core
  (`Int`/`UInt`/`Compose`/algebra witnesses/`Float`), `llvm_ir`/`ptx`
  carriers, the format models, `react`, `coordination`,
  `process`/`file_system`.
- **D2-encoding — rework (vocabulary preserved):** the 5 merged language
  files (`rust`/`python`/`go`/`cpp`/`typescript`); `verilog`/`llvm_ir`/
  `ptx` prose-only; `INVARIANTS.md` P1:42 (the doc root); `DECISIONS.md`
  D2/D4; `TASKS.md` T-4 contract; the `std/integer.dag` /
  the historical D2 row in this file (NOT the core).
- **Active PRs:** `#3219` closed; `#3208` held.

#### #3208 held dissolution dispositions

- **Primitive alias rows:** 🟡 bound to the post-D2-REV fact-bundle
  rework. Trigger: the T-4 fact-bundle rework executes after T-30 and
  the remaining T-4 feeders clear.
- **Shared C5 fidelity disposition carrier and disposition rows:** 🟡
  bound to the same post-D2-REV shared-C5-fidelity / fact-bundle rework.
  This covers `LeanFidelityDisposition` / `MachineFidelityDisposition`
  consolidation and replacing the removed M1(2.8)-opaque `fn lean_feature_disposition` /
  `fn machine_feature_disposition` maps with **structural** per-feature disposition
  authority (data rows or shared carrier) once the shared C5 substrate lands. Trigger:
  the T-4 fact-bundle rework executes after T-30 and the remaining T-4
  feeders clear.
- **Phantom `IRCarrier` parameters:** 🟢 not a dissolution-class finding;
  dead-generic-parameter polish only, with no walker, predicate, carrier,
  or emit-template obligation.

### TS-D2 — `typescript.dag` D2→fact-bundle dissolution disposition

Authoritative ledger entry cited by the in-file coproduct-dissolution
one-liners in `extdeps/languages/typescript.dag` on `main`
(`// 🟢 coproduct dissolution — DECISIONS.md TS-D2`). Per-language
accountability row for the `typescript` entry of the **D2-encoding —
rework** list above (the 5th merged D2 language file). Row established
2026-05-18; describes current `origin/main` state.

- **In-file carrier — LANDED.** PR **#3251**
  (`23061e284 typescript: regate D2 scaffolds to fact bundles (#3251)`,
  ancestor of `origin/main`) strict-de-prosed and re-gated
  `typescript.dag`. On `main` the file is now a terse 36-line scaffold:
  header `Status: 🟡 gated — feature: T-4 fact-bundle Phase-3 rework
  after T-3/T-29/T-30/T-25-core`; the D2 `GroundingMap` facet and the
  heavy D2 header prose are gone. #3251 authored **no** fact-bundles and
  made **no** `DECISIONS.md` edit — it added the two in-file `TS-D2`
  pointers that this row resolves.
- **🟢 dissolved (terminal) — the two coproducts.**
  `TsEcma262NumericPrimitiveKind` (variants `TsNumberPrimitive`,
  `TsBigIntPrimitive`) and `TsEcma262PrimitiveOperationSemantics`
  (variants `TsNumberIeee754Binary64Semantics`,
  `TsNumberBitwiseInt32Uint32Semantics`,
  `TsBigIntExactUnboundedSemantics`) are **TypeScript-native
  ECMA-262 vocabulary** — closed enumerations sourced from the ES2025
  spec, not `std/` alias re-declarations and not a parallel
  `OrderedRing` algebra (INVARIANTS P1:42). They are terminal: nothing
  dissolves further, hence the in-file `🟢`.
- **A-vs-B = B (ruled-B): `TsBoolean = Bool` retired.** The prior
  `type TsBoolean = Bool` kernel-ambient alias bridge is **removed** — a
  bare D2 alias asserts identity instead of demonstrating it (the hollow
  form the machine-readable-inhabitance bar forecloses). TypeScript
  `boolean`'s machine-readable grounding edge *declaration*
  `ts_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra`
  (decl-ref, single std authority) is present + v2-parse-verified —
  **P2/E-6 STAGING**, not landed authority (no same-PR consumer;
  canonical fold specified-not-realized;
  `docs/modeling/grounding-worked-examples.md` §0), not the bare alias.
  The still-unmodeled `number` / `string` / `symbol` /
  `null` / `undefined` primitive facts (the numeric/text `std/` carriers
  the header marks "fact-bundle-gated") remain **🟡 gated D2→fact-bundle
  debt** — not yet fact-modeled — and are unaffected by this row.
- **Named trigger / dissolve-on-arrival:** upstream P4 node
  `node://adhoc-71ec74f4-080` ("T-4 fact-bundle Phase-3 rework
  `extdeps/languages`", `crisp-crab-858`) is to author the remaining
  TypeScript primitive fact-bundles — `number` (binary64 +
  NaN/Infinity/disposition facts), `string` (UTF-16 code-unit facts),
  `symbol` / `null` / `undefined` (nominal/runtime facts). Any reuse of
  `std/float.dag` / `std/integer.dag` / `std/text.dag` must cite
  **proven coincidence** evidence ("Coincide — defined once", above),
  never a bare alias.
- **Feeder gates (hard T-4 prereqs, per Phase 1 / `TASKS.md` T-4):**
  T-3, **P1-KEYSTONE — CLEARED** (`#3226` merged @`77b9e7d72`), T-29,
  T-30, T-25-core. The Phase-3 reseed does **not** begin until all clear.
- **Dissolution:** the 🟡 clause is removed (or replaced by a
  coincidence-evidenced bundle reference) when node `…71ec74f4-080`
  lands and the TypeScript primitive fact-bundles merge; the 🟢 clause
  and the in-file pointers persist as the terminal record.

### Worked examples

`docs/modeling/grounding-worked-examples.md` (this PR) demonstrates the
fact-bundle/grounding model on one complex type per target — spanning
`machine_code` (pure bits) → `rust` → `verilog` (4-valued logic) →
`spice` (continuous) → `lean` (dependent types) → `english` (natural
language, mostly fail-closed). Each: the model (carrier + meaning) and
step-by-step coercion. Plan-only; the remaining mainstream
language/format targets fan out against the same template.

### T-30 — `std/fact_density.dag` encoding note (2026-05-17)

- **Header discipline (PR #3227):** the on-disk file keeps a terse machine
  header only: path line; then **contiguous** `Scope:` / `Owns:` /
  `Consumes:` / `Status:` (Practice 9 four-line header); **then** at most
  one `Anchor:` URL line per owned carrier (anchor **after** that block,
  not between `Owns:` and `Consumes:`).
  Process receipts, Practice pointers, and bootstrap-collision narrative
  live in commit messages, this ledger, or `TASKS.md` — not in the `.dag`
  comments.
- **P2 / Practice 5 (single authority):** Until a **generated** `.dag` (or emitted) consumer reads `SourceSpecReadFact`, this module does **not** satisfy INVARIANTS §P2 “**When a boundary counts as landed**” — the hollow-alias predicate’s interim authority remains the Rust mirror (`src/v3/compiler/src/v4_hollow_alias_gate.rs`). `STRUCTURE.md` lists `fact_density.dag` as **P2-staging**, not co-equal with landed `std/` primitives. The on-disk file is intentionally a **Practice-9 terse header** only: the `Scope:` line is one machine line; this bullet carries the former multi-line header narrative (no generated `.dag`/std consumer yet; do **not** treat `SourceSpecReadFact` as a landed substrate primitive in docs until INVARIANTS §P2’s generated-consumer proof exists).
- **Nominal witness:** `SourceSpecReadFact` is a body-less nominal marker
  for spec-read anchoring (not a numeric alias). A richer `Node` payload
  is deferred until `compile_to_dag` can prepend `std/node.dag` without
  colliding on v3-bootstrap top-level names; dissolution ships in the
  same change set as that bridge.

### Status

**RATIFIED 2026-05-17** (operator, out-of-band — the operator declared D2
wrong [the verbatim operator quote is in *What reversed* above, this same
section], ratified the fact-bundle principle, and dispatched the Phase-1
execution). Phase 1 is executing now as **PR #3224** (TASKS.md +
DECISIONS.md re-sequencing). This PR (#3223) encodes the ratified
decision into the ledger; #3224 is stacked on it. TASKS.md consuming the
D2-reversal as ratified is therefore correct, not premature.

---

## Part 5 — Ratification flow

For each PROPOSED item: ratify (👍 / "confirm") or redirect ("instead,
…"). On ratification I encode it into the cited file header(s) — the
same way A1/A2/A3/U1/C2 were encoded — and promote it to Part 1 here.
Quick confirms (Part 2) are cascade-closures of already-ratified
parents; redirecting one means reopening its parent. Part 3 are genuine
forks; B1's "where it lives" is the one pick I most need from you.

Nothing here is encoded into substrate until you ratify it. This file
PROPOSES; it does not decide.


---

## Part 6 — PR #3229 substrate Practice-4 / P5 ledger relocation (RATIFIED in-PR 2026-05-17)

> **Authority:** Operator reconciliation (still-hawk-102 relay, 2026-05-17). Strict de-prose on `session/lively-seal-825` removed post-`module` `//` blocks from the five allowlisted v4 substrate files. **Sum-coproduct** Practice-4 dissolution receipts (🟢/🟡/🔴) and **enumerated non-coproduct tracked scaffolds** (lexeme bridges, raw-`Int` cost axes, LLVM operand residuals, integer where-clause gap, …) that were *authoritative review records* must **not** live only in `docs/v4-dag-rationale.md` (explicitly non-authoritative). They are **relocated here** — the same home style as sibling ledger rows (#3208 / #3209 precedent: ratified `DECISIONS.md` + one-line live pointer on coproducts + explicit `// Ledger:` slug inventory).
>
> **Merge-base object** for verbatim recovery of removed `//` lines: **`92cb26402eeb21471acb6ac47559cbae3b52afdb`**. Live `.dag` files keep **declarations** + terse headers + **`// Ledger: …` pointer** (generated by `scripts/strict_deprose_dag.py`).
>
> **In-file one-liners (operator directive 2026-05-17; `modeling-discipline.md` PR #3234):** each sum coproduct (`type` with N≥2 `|` variants) in the five allowlisted `.dag` files carries exactly one `// 🟢|🟡|🔴 coproduct dissolution — DECISIONS.md Part 6 · <SL-3229-*|CP-3229-*>` line **on** the coproduct. Full five-pattern ledgers, #3200 yellow footers, Wave-A2 list deferrals, and **non-coproduct** scaffold bridges live **only** in the subsections below (not duplicated in live bodies). `scripts/strict_deprose_dag.py` **fails** unless every slug listed on a live `// Ledger:` line exists as a `###` row in this Part.

### CP-3229-VERIFY — `std/integer.dag`

**Verified:** merge-base `92cb26402:src/v4/std/integer.dag` contains **no** `Coproduct dissolution` / `TRACKED 🟡` / `🟢 GREEN` Practice-4 checkpoint blocks on sum coproducts (search-empty). Removed body `//` text was **D2 / modeling prose** already superseded at the ledger level by **DECISIONS.md `D2-REV`**. **`GroupCompletion<M>` constrained-inhabitance** is **not** a coproduct dissolution receipt; verbatim merge-base text is relocated at **`SL-3229-INTEGER-GROUP-COMPLETION`** below (not omitted).

### CP-3229-NAT-LE-WITNESS — `std/cardinality.dag` `NatLeWitness` (Practice-4 terminal sum)

**Authority:** PR #3310 P1 cardinality refinement (api-review codex blocking receipt, 2026-05-18).

**Disposition:** **🟢 GREEN terminal** — inductive witness `NatLeWitness = LeqZero { upper: Nat } | LeqSucc { inner: NatLeWitness}` encodes `a ≤ b` on Peano `Nat`; the impossible `Succ _ ≤ Zero` case is **structurally absent** (illegal states unrepresentable).

**Five-pattern ledger (Practice 4):**

1. **Fact placement** — FAILS: witness is shared `std/cardinality.dag` vocabulary, not a single downstream consumer projection.
2. **Variant-is-data / flat record** — FAILS: `LeqSucc` is inductive structure, not a tagless product encoding of `≤`.
3. **Algebraic law carrier** — FAILS: not an algebra table; principled via `Nat` + witness spine only.
4. **Dimensional decomposition** — FAILS: one coproduct axis, not orthogonal coordinates.
5. **Parameterized indexed family** — FAILS: closed two-variant sum, not `F<X>` over an open index.

**Live substrate tag:** `// 🟢 coproduct dissolution — DECISIONS.md Part 6 · CP-3229-NAT-LE-WITNESS.` immediately precedes `type NatLeWitness` in `src/v4/std/cardinality.dag`.

**Cross-ref:** Part 1 table row `NatLeWitness` / `UpperBoundedNat`; **`§PR-3252-cardinality-std`** for `Never` / `Unit` inhabitance context. **Not** part of merge-base **`### CP-3229-GREEN-TERMINAL`** bulk table (that table enumerates strict-de-prose merge-base files only); this subsection is the **reachable** Practice-4 receipt for `NatLeWitness`.

### SL-P7-NAT-IS-ZERO-VPRED — `std/nat.dag` `nat_is_zero` (predicate-dissolution interim)

**Authority:** operator CORE relay (`still-hawk-102` → `jolly-ibex-599` → P7 lane, 2026-05-18).

**Disposition:** **🟡 gated** — `fn nat_is_zero` is a **Practice 10 predicate-dissolution** interim: hand `match` derives the `Nat = Zero | Succ { … }` **variant discriminant** (`Zero => true; Succ => false`) where the canonical surface is a **substrate-derived `Zero`-variant projection** once the discriminant-predicate machinery exists. **Not 🟢 terminal** while a named dissolve target exists.

**`feature:` gate:** coproduct **variant-discriminant predicate** substrate (generated / compiler-owned discriminant API — canonical `is_<Ctor>` / projection family).

**Plan bind:** internal work node **`node://adhoc-2145db6b-69a`** (lane bucket for that substrate).

**Dissolve-on-arrival:** replace `nat_is_zero`’s **body** with the substrate-derived **Zero discriminant** when **`node://adhoc-2145db6b-69a`** closes. **Explicitly forbidden interim “fix”:** rewriting into a **`nat_cata`** / generic fold carrier as **fold laundering** (operator).

**Live substrate tag:** one-line **`// 🟡 gated — …`** immediately precedes `fn nat_is_zero` in `src/v4/std/nat.dag`; header **`// Ledger: … SL-P7-NAT-IS-ZERO-VPRED`**.

**PR receipt:** gunbc **#3255** (P7) + on-thread **#3244** disposition history.

### SL-P7-NAT-COMPARE-VPRED — `std/nat.dag` `nat_compare` (predicate-dissolution interim)

**Authority:** PR #3310 (api-review codex blocking receipt, 2026-05-18) — binary `Nat` **total-order trichotomy** (`Ordering`) promoted from `float.dag` into `std/nat.dag` as shared substrate.

**Disposition:** **🟡 gated** — `fn nat_compare` is a **Practice 10 predicate-dissolution** interim: hand `match` performs **paired Peano spine unwinding** for `Less | Equal | Greater` with **no** consumption yet of a substrate-declared **binary** fold / generated order query (distinct from unary `nat_cata`). **Not 🟢 terminal** while the discriminant / systematic-compare substrate bucket is open.

**`feature:` gate:** coproduct **variant-discriminant predicate** substrate (**`node://adhoc-2145db6b-69a`**, same lane bucket as `SL-P7-NAT-IS-ZERO-VPRED`).

**Dissolve-on-arrival:** replace `nat_compare`’s **body** with the substrate-owned **canonical Nat compare** (generated lex-compare / `OrderedSemiring<Nat>` witness consumer — whichever the `node://adhoc-2145db6b-69a` closure lands) **without** `nat_cata` / extra hand `match` **fold laundering** as stand-in authority (operator).

**Live substrate tag:** one-line **`// 🟡 gated — …`** immediately precedes `fn nat_compare` in `src/v4/std/nat.dag` (parallel interim glyph to `nat_is_zero`, with **binary** `dissolve-on-arrival` text). File header **`// Ledger: … SL-P7-NAT-IS-ZERO-VPRED, SL-P7-NAT-COMPARE-VPRED`**.

**PR receipt:** gunbc **#3310** (this PR).

### SL-P6-FREEMONOID-IS-EMPTY-VPRED — `std/algebra.dag` `free_monoid_is_empty` (predicate-dissolution interim)

**Authority:** PR #3249 cursor review artifact #14097 plus Practice 10's post-#3258 blocking disposition rule.

**Disposition:** **🟡 gated** — `fn free_monoid_is_empty` is a **Practice 10 predicate-dissolution** interim: hand `match` derives the `FreeMonoid<T> = Empty | Cons { … }` **variant discriminant** (`Empty => true; Cons => false`) where the canonical surface is a **substrate-derived `Empty`-variant projection** once the discriminant-predicate machinery exists. **Not 🟢 terminal** while a named dissolve target exists.

**`feature:` gate:** coproduct **variant-discriminant predicate** substrate (generated / compiler-owned discriminant API — canonical `is_<Ctor>` / projection family).

**Plan bind:** internal work node **`node://adhoc-2145db6b-69a`** (same substrate bucket as `SL-P7-NAT-IS-ZERO-VPRED`).

**Dissolve-on-arrival:** replace `free_monoid_is_empty`'s **body** with the substrate-derived **Empty discriminant** when **`node://adhoc-2145db6b-69a`** closes. **Explicitly forbidden interim “fix”:** rewriting into a **`free_monoid_fold`** / generic fold carrier as **fold laundering**; under strict evaluation that also turns a discriminant query into a full-spine walk.

**Live substrate tag:** one-line **`// 🟡 gated — …`** immediately precedes `fn free_monoid_is_empty` in `src/v4/std/algebra.dag`; header **`// Ledger: … SL-P6-FREEMONOID-IS-EMPTY-VPRED`**.

**PR receipt:** gunbc **#3249** rework after review artifact **#14097**.

### SL-3229-INTEGER-GROUP-COMPLETION — `GroupCompletion<M>` constrained-inhabitance gap

Verbatim `//` lines from merge-base `integer.dag` (lines **124–132**):

```text

// CONSTRAINED-INHABITANCE GAP (tracked scaffold). Denotationally the
// Grothendieck group completion requires a commutative monoid. v4 does
// not yet have a parametric where-clause syntax such as
// `<M> where M : CommutativeMonoid<_>`. The only authored use here is
// `GroupCompletion<Nat>`, and nat.dag already declares Nat's additive
// `CommutativeMonoid<Nat>` instance, so the mathematical precondition
// holds for the actual construction. Dissolution trigger: when v4 lands
// constrained generic parameters / inhabitance bounds, tighten
// `GroupCompletion<M>` to require the commutative-monoid witness.

```

**Dissolution trigger:** v4 lands constrained generic parameters / inhabitance bounds tightening `GroupCompletion<M>` (merge-base text).

### SL-3229-LLVM-WIDTH — raw-`Int` width payload scaffold (`LlvmType` family)

**Authority / closure:** PR #3310 P1 cardinality refinement (api-review codex receipt, 2026-05-18).

**Disposition:** **🟢 GREEN terminal** for the merge-base **raw `Int` width payload** class on `LlvmType` — live `llvm_ir.dag` uses `v4.std.cardinality.NonZeroNat` for `IntegerType.bits` and `VectorType.count`, and `v4.std.nat.Nat` for `ArrayType.count` / `PointerType.address_space`, closing the signed / zero-bit illegal state the merge-base `Int` payloads admitted. **Bounded use:** LangRef-range tightness beyond “positive width / non-negative count” stays producer-side until any stronger witness the operator schedules separately.

Verbatim `//` lines from merge-base `llvm_ir.dag` (lines **276–297**), immediately preceding `type LlvmType`:

```text

// RAW-Int WIDTH RESIDUAL — TRACKED 🟡 SCAFFOLD (isolated, the exact
// std/diagnostic.dag `Locus`(🟢)/`Extent.ByteRange`(🟡) precedent). The
// SUM is 🟢 terminal (the categories are genuine), but the numeric
// payloads `IntegerType.bits` / `VectorType.count` / `ArrayType.count` /
// `PointerType.address_space` are kernel-ambient `Int`, which
// STRUCTURALLY ADMITS illegal LLVM values (a 0-bit / negative-width
// integer, a negative element count) — modeling-discipline.md §2
// (illegal states unrepresentable) forbids it, and no bounded-natural
// substrate is consumable yet (std/cardinality.dag refinement +
// std/nat.dag are parallel Wave-A1, NOT in this file's whitelist). It is
// a TRACKED scaffold declared with the three bridge properties, verbatim
// the merged diagnostic.dag discipline (PR #3161):
//   (a) scaffold doc — THIS note; the raw `Int` widths are provisional,
//       not the terminal bounded-width shape.
//   (b) bounds on use — a producer MUST construct only LangRef-valid
//       widths (`bits >= 1`; `count >= 0`; `address_space >= 0`). Until
//       the substrate enforces it the invariant is producer-side; an
//       out-of-range width is a producer bug, not a legal value.
//   (c) dissolution trigger — when std/cardinality.dag's refinement
//       substrate lands (its own T-3 entry), these refine to bounded
//       non-negative widths type-enforced, and this residual
//       reclassifies 🟢 (the SUM is already 🟢; only the payload moves).
```

**Dissolution trigger (merge-base archive):** when `std/cardinality.dag` refinement substrate lands (T-3), bounded non-negative widths become type-enforced; residual reclassifies 🟢 per merge-base note. **Checkable landing (width payloads):** satisfied on `LlvmType` as documented in **Disposition** above; **`SL-3229-LLVM-OPS`** remains the separate operand-relation scaffold.

### SL-3229-LLVM-OPS — operation-specific operand constraints

Verbatim `//` lines from merge-base `llvm_ir.dag` (lines **1117–1170**):

```text

// ─────────────────────────────────────────────────────────────────────
// OPERATION-SPECIFIC OPERAND CONSTRAINTS — consolidated TRACKED 🟡
// SCAFFOLD (the merged diagnostic.dag `Extent.ByteRange` three-bridge
// precedent — a CHECKABLE trigger, not a prose-only disposition).
//
// Several instructions carry LangRef operand-relation constraints that
// the broad `LlvmValue` / `LlvmType` carriers do NOT type-enforce, e.g.:
//   - `icmp`/`fcmp`: both operands same type; `icmp` int/ptr, `fcmp`
//     float (or same-shaped vector); result is `i1`/vector-of-`i1`.
//   - `getelementptr`: indices are integer-typed; struct indices are
//     `i32` constants; result type is the GEP-walked type.
//   - `select`: condition is `i1` (or vector-of-`i1` matching operands);
//     the two arms share the result type.
//   - `Binary`: `lhs`/`rhs`/result share one type; float opcodes need
//     float operands, integer opcodes integer operands.
//   - `insert/extractelement`, `shufflevector`: element/index/mask type
//     and length relations.
// (Atomic-ordering op-specific validity is NO LONGER in this residual:
// Load/Store/Fence/AtomicRmw/CmpXchg orderings were ALL structurally
// fixed via per-operation subset enums — `LoadOrdering`/`StoreOrdering`/
// `FenceOrdering`/`AtomicRmwOrdering`/`CmpXchgSuccessOrdering`/
// `CmpXchgFailureOrdering`. Verified against LLVM 18: the historical
// `cmpxchg` "failure not stronger than success" cross-field rule was
// REMOVED in LLVM ≈3.9, so NO cross-field atomic-ordering refinement
// remains — all are single-subset, structurally unrepresentable-when-
// invalid, not a substrate-gap.)
// These are NOT collapsed-away facts — they are real LangRef invariants
// the model currently leaves PRODUCER-SIDE because the substrate to
// type-encode an operand-relation refinement (a dependent/refined
// typed-value carrier) is std/cardinality.dag's refinement substrate,
// which is parallel Wave-A1 and NOT in this file's whitelist.
// Introducing a constrained carrier here would be exactly the
// improvise-past-not-ready-substrate failure mode; the sanctioned path
// (operator-ratified, merged in diagnostic.dag) is the tracked scaffold
// with the three bridge properties:
//   (a) scaffold doc — THIS note; the broad operands are provisional,
//       not the terminal operand-relation-refined shape.
//   (b) bounds on use — a producer MUST construct only LangRef-valid
//       operand-type relations (same-type binary/compare; i1 select
//       cond; integer GEP indices; …). Until the substrate enforces it
//       the invariant is producer-side; a violating instruction is a
//       producer bug, not a legal value.
//   (c) dissolution trigger — CHECKABLE, not prose: when
//       std/cardinality.dag's refinement substrate lands (its own T-3
//       entry — a concrete, observable file/feature-landing event, the
//       SAME trigger the merged diagnostic.dag `Extent.ByteRange` 🟡
//       uses), these operand relations refine to type-enforced
//       constraints and this residual reclassifies 🟢. The trigger is a
//       substrate-landing fact a reviewer can check, not a vague
//       "later".
// This converts root-cause #3's "broad carriers + prose-only
// dispositions" into a single tracked residual with a checkable trigger,
// without improvising refinement substrate that is not yet ratified.

```

**Dissolution trigger:** same checkable landing as `SL-3229-LLVM-WIDTH` (`std/cardinality.dag` refinement).

### SL-3229-PTX-DIM3 — `Dim3` kernel-ambient `Int` axis scaffold

**Authority / closure:** PR #3310 P1 cardinality refinement plus follow-up
receipt PR (2026-05-18).

**Disposition:** **🟢 GREEN terminal for the merge-base raw `Int` axis
payload class.** Live `ptx.dag` uses `Nat` for `Dim3.x` / `y` / `z`, so
negative dimension or coordinate axes are no longer representable by this
carrier. **Residual remains 🟡:** `Nat` still admits zero and does not encode
PTX-version-specific per-axis maxima, so `SL-3229-PTX-DIM3` remains a live
ledger slug on `ptx.dag` for the positive / bounded-axis refinement. Named
trigger: the same T-3 bounded-natural refinement substrate must supply a
positive bounded-axis witness before this row can close fully.

Verbatim `//` lines from merge-base `ptx.dag` (lines **1059–1090** — section header + 🟡 three-bridge note):

```text

  | PredicateScalar


// ─────────────────────────────────────────────────────────────────────
// Dim3 — the PTX 3-D dimension/index carrier: an x/y/z triple of
// kernel-launch dimensions or coordinate indices. Conj record (no
// Practice-4 ledger — only coproducts dissolve).
//
//   - x, y, z       the three-axis components
//
// 🟡 SCAFFOLD (same bridge as std/diagnostic.dag's `Extent.ByteRange`).
// With only kernel-ambient `Int` available at the T-4.14 phase,
// `{ x: Int, y: Int, z: Int }` STRUCTURALLY ADMITS illegal states
// (negative dims; zero dims; dims exceeding the PTX max per axis —
// 65535 for x of gridDim, 1024 for thread count of blockDim, etc.).
// modeling-discipline.md §2 (illegal-states-unrepresentable) forbids
// admitting them; no bounded-Cardinality refinement substrate exists
// yet to exclude them. It is a TRACKED scaffold, declared with the
// three bridge properties (the precedent diagnostic.dag established
// 2026-05-15):
//   (a) scaffold doc — THIS note; the carrier is provisional, not
//       the terminal bounded-axis shape.
//   (b) bounds on use — a producer MUST construct only valid PTX
//       dimensions per the pinned PTX-ISA version's per-axis limits.
//       Until the substrate enforces it the invariant is producer-
//       side; a Dim3 violating it is a producer bug, not a legal
//       value.
//   (c) dissolution trigger — when std/cardinality.dag's refinement
//       substrate lands (T-3, P4 decidability), `x` / `y` / `z`
//       become refined bounded-positive-Int axes (positive and
//       within the PTX-version-pinned per-axis maxima), and this
//       scaffold dissolves.
```

**Dissolution result:** P1's `Nat` carrier landing closes only the
merge-base negative-axis illegal state. The zero-axis and
PTX-pinned-maximum illegal states remain live under `SL-3229-PTX-DIM3`
until the named positive bounded-axis witness lands.

### SL-3229-PTX-COST — raw-`Int` PTX cost axes (`PtxCost`)

**Authority / closure:** PR #3310 P1 cardinality refinement plus follow-up
receipt PR (2026-05-18).

**Disposition:** **🟢 GREEN terminal for the merge-base raw `Int` cost
payload class.** Live `ptx.dag` uses `Nat` for `PtxCost.instruction_cost`
and `PtxCost.occupancy_cost`, so negative cost is structurally
unrepresentable.

Verbatim `//` lines from merge-base `ptx.dag` (lines **1287–1328** — `PtxCost` + 🟡 three-bridge note):

```text

// ─────────────────────────────────────────────────────────────────────
// PtxCost — the per-target cost-realization carrier shape for the
// PTX `LanguageModel`. Conj record (no Practice-4 ledger).
//
// This declares the SHAPE the U1 spine reads when computing PTX-side
// realization cost — both components per the scaffold header's
// "per-instruction + occupancy cost on a PTX target" claim:
//
//   - instruction_cost   per-instruction throughput cost (the U1
//                        per-target-op cost projection — the `map`
//                        side of `Homomorphism<…, …, PtxScalar>`
//                        carries this; cost is read FROM the map's
//                        structure, never stored as a parallel field
//                        per algebra.dag's U1 no-engine discipline)
//   - occupancy_cost     per-kernel occupancy cost (registers-per-
//                        thread × threads-per-block, shared-memory-
//                        per-block) — the SIMT-specific cost
//                        contribution beyond per-instruction; how
//                        many warps the SM can co-schedule
//
// Both Int carriers are scaffold (same 🟡 bridge as Dim3): cost is a
// non-negative quantity, but without std/cardinality.dag refinement
// negative-cost is structurally representable. Producer-side
// invariant pending the cardinality substrate landing. PTX-ISA-
// version-specific cost tables (e.g. SASS instruction latencies per
// release) sit in the inhabitance walk that T-4's LanguageModel
// shape parameterizes, not in this carrier itself.
type PtxCost {
  instruction_cost: Int
  occupancy_cost: Int
}

```

**Dissolution result:** P1's `Nat` carrier landing type-enforces
non-negative cost axes; no live `SL-3229-PTX-COST` ledger slug remains in
`ptx.dag`.

### SL-3229-VERILOG-NONEMPTY — shared `List<T>` spec-non-empty Wave-A2 deferral

Verbatim `//` lines from merge-base `verilog.dag` (lines **2156–2311**):

```text

// 🟡 TRACKED-SCAFFOLD: spec-non-empty list fields (relayed
// disposition from T-4 mgr vivid-carp-207 msg_30a7ec7c, 2026-05-16).
//
// Twenty-six sites in this file carry `List<T>` payloads where IEEE
// 1364-2005 requires AT LEAST ONE entry — the spec-illegal empty
// state is structurally representable, the same INVARIANTS P2
// illegal-states-unrepresentable concern codex / openai-pro
// correctly flagged. The MOTIVATION is real and spec-grounded. The
// CANONICAL FIX is NOT a worker-minted `NonEmptyList<T>` generic
// carrier:
//
//   - `docs/coercion-design.md` RQ-3 (ratified): "refinements of
//     base containers, NOT separate inhabitant families.
//     `NonEmptyList<T> = List<T> where non_empty`. Separate
//     inhabitants would grow the authority surface." A hand-rolled
//     `NonEmptyList<T> { head: T, tail: List<T> }` IS a separate
//     inhabitant family and breaks the ratified Vec<T> coercion.
//   - `std/collection.dag` (merged on main) DEFERS its entire
//     finite-cardinality / refinement layer to "Wave-A2 — TRACKED
//     SCAFFOLD (🟡)". The `List<T> where non_empty` refinement IS
//     that deferred layer.
//   - This file's frozen scaffold header neither Owns nor Consumes
//     `std/collection.dag`. Minting a parallel non-empty carrier in
//     a leaf language probe would be a 5-language parallel-
//     declaration precedent — the v3 cross-layer spine-drift this
//     T-4 directive exists to prevent.
//
// SANCTIONED DEFERRAL (the deferral middle-path, mirroring
// collection.dag's own Wave-A2 scaffold + the ptx #3170 / spice #3168
// inhabitance-deferral pattern):
// - Each of the twenty-six spec-non-empty sites keeps `List<T>` for now.
// - Each site carries a back-pointer comment to THIS shared note.
// - The empty-list illegal state is a PRODUCER-SIDE invariant
//   pending the refinement substrate landing.
// - DISSOLUTION TRIGGER: when `std/collection.dag` Wave-A2 lands the
//   non-empty refinement, the twenty-four sites adopt
//   `List<T> where non_empty` uniformly in one sweep — single-
//   authority preserved.
//
// The twenty-six spec-non-empty sites (back-pointers to this note):
//   (sites 25-26 below — VerilogModule per-style header_ports)
//   1. NetDeclaration.NonTriregIdentifierForm.identifiers (§A.2.1.3)
//   2. NetDeclaration.NonTriregAssignForm.assignments     (§A.2.1.3)
//   3. NetDeclaration.TriregIdentifierForm.identifiers    (§A.2.1.3)
//   4. NetDeclaration.TriregAssignForm.assignments        (§A.2.1.3)
//   5. VariableDeclaration.{Reg,Integer,Real,Time,RealTime}Variable.declarators
//                                                    (§A.2.4 / §7.4)
//   6. ParameterDeclaration.assignments              (§A.2.1.1)
//   7. LocalParameterDeclaration.assignments         (§A.2.1.1)
//   8. SpecparamDeclaration.assignments              (§A.2.1.1)
//   9. ModuleParameters.parameters                   (§12.1)
//  10. AlwaysBlockSensitivity.ExplicitEventList.items (§9.7.2)
//  11. ParameterOverrides.OrderedParameterOverrides.actuals (§12.1.3)
//  12. ParameterOverrides.NamedParameterOverrides.overrides (§12.1.3)
//  13. ContinuousAssign.assignments                  (§6.1)
//  14. ModuleInstantiation.instances                 (§A.4.1.1)
//  15. PortConnections.PositionalConnections.actuals (§A.4.1.1)
//  16. PortConnections.NamedConnections.connections  (§A.4.1.1)
//  17. DefparamDeclaration.assignments               (§A.2.5)
//  18. EventDeclaration.event_names                  (§A.2.1.4)
//  19. GenvarDeclaration.genvar_names                (§A.2.1.1)
//  20. UdpInstantiation.instances                    (§A.5.2)
//  (Former site 21 PortDeclaration.names was DISSOLVED at sha
//  cb028c7d+ — PortDeclaration itself dissolved into AnsiPortDeclaration
//  per openai-pro REQUEST_CHANGES; the §A.2.1.2 list_of_port_identifiers
//  fact is now uniformly carried by AnsiPortDeclaration's per-direction
//  variants at sites 21-24 below.)
//  21. AnsiPortDeclaration.AnsiInputPort.names           (§A.1.3)
//  22. AnsiPortDeclaration.AnsiOutputNetPort.names       (§A.1.3)
//  23. AnsiPortDeclaration.AnsiOutputVariablePort.names  (§A.1.3)
//  24. AnsiPortDeclaration.AnsiInoutPort.names           (§A.1.3)
//  25. VerilogModule.NonAnsiVerilogModule.header_ports (§12.3.4)
//  26. VerilogModule.AnsiVerilogModule.header_ports    (§12.3.4)
//
// Sites previously numbered 7-9 added 2026-05-16 per codex
// REQUEST_CHANGES on PR #3176 at sha f87fdf189; site previously 10
// added 2026-05-16 per codex BLOCKING on PR #3176 at sha bcc6b5f89
// (ContinuousAssign restructured to carry drive_strength + delay +
// list_of_net_assignments per §6.1, with the non-empty assignments
// list deferred under the same RQ-3 disposition); site previously 11
// added 2026-05-16 per codex BLOCKING on PR #3176 at sha 2bf0024ba
// (ModuleInstantiation restructured to carry shared module_name +
// parameter_overrides + non-empty instances per §A.4.1.1 multi-
// instance form). Sites 1-4 (formerly the single site
// `NetDeclaration.declarators`) added 2026-05-16 per openai-pro
// REQUEST_CHANGES on PR #3176 at sha 6f587e6c — NetDeclaration
// dissolved into the IEEE 1364-2005 §A.2.1.3 four-way
// {trireg?}×{form} structural Disj making spec-impossible states
// (drive_strength on the identifier-only form, charge_strength on
// a non-trireg form, mixed init/non-init declarator lists,
// identifier-only-with-initializer, assignment-form-without-
// initializer) unrepresentable; the four per-variant
// list_of_net_decl_assignments / list_of_net_identifiers fields
// inherit the §A.2.1.3 spec-non-empty constraint and ride the same
// RQ-3 deferral. Sites 15-20 added 2026-05-16 per openai-pro
// REQUEST_CHANGES on PR #3176 at sha 1eb241a2 (inline BLOCKING
// findings at lines 2156 / 2315 / 2604): PortConnections gained the
// NoPortConnections nullary variant — the §A.4.1.1 empty `()`
// connection-list form is now structurally distinct from non-empty
// ordered/named lists (P2 representation-duality dissolution); the
// two non-empty inner lists ride the same RQ-3 deferral.
// DefparamDeclaration / EventDeclaration / GenvarDeclaration each
// gained a non-empty list of per-entry sub-records (new
// DefparamAssignment Conj for §A.2.5 list_of_defparam_assignments;
// `event_declarators: List<EventDeclarator>` for §A.2.1.4
// list_of_event_identifiers (later sha 4ccc6267 added per-identifier
// dimensions via EventDeclarator); `genvar_names: List<Symbol>` for
// §A.2.1.1 list_of_genvar_identifiers) — restoring the spec
// statement-boundary lost by the prior singular-entry shape. UdpInstantiation refactored to the
// same shared-header + non-empty-instance-list pattern as
// ModuleInstantiation, per §A.5.2 `udp_identifier [drive_strength]
// [delay2] udp_instance { , udp_instance } ;` — shared
// (udp_name, drive_strength, delay) clause + per-instance
// (instance_name, body_lexeme) UdpInstance entries. Codex's /
// openai-pro's motivation is spec-correct in each case; the FORM
// is the deferred refinement, not a verilog-local mint. Site 21
// added 2026-05-16 per openai-pro REQUEST_CHANGES on PR #3176 at
// sha cba903a3e (inline BLOCKING at line 1429): PortDeclaration
// reshape from singular-name to list-form mirrors the Defparam/
// Event/Genvar list-form precedent under §A.2.1.2 list_of_port_
// identifiers; the non-empty list rides the same RQ-3 deferral.
// Sites 22-24 added 2026-05-16 per openai-pro REQUEST_CHANGES on
// PR #3176 at sha 4ccc6267 (Layer-Model BLOCKING on
// AnsiPortDeclaration per-single-name carrier): the three ANSI port
// direction variants reshape from `name: Symbol` singletons to
// non-empty name-lists (AnsiInputPort/AnsiInoutPort use
// `List<Symbol>` per §A.1.3 list_of_port_identifiers; AnsiOutputPort
// uses `List<AnsiOutputPortDeclarator>` per §A.1.3
// list_of_variable_port_identifiers — output variable-ports admit
// per-port initializers via the existing VariableInitializerClause
// Disj). All three new non-empty lists ride the same RQ-3 /
// std/collection.dag Wave-A2 deferral; no verilog-local
// NonEmptyList mint. Sites 25-26 added 2026-05-16 per openai-pro
// REQUEST_CHANGES on PR #3176 at sha 7709c9968 (inline BLOCKING at
// line 3525 — VerilogModule header_ports List representation-
// duality): the no-ports module form (`module M; ... endmodule`)
// was representable as either NonAnsi{header_ports=[]} or
// Ansi{header_ports=[]} (P2 representation-duality), AND empty
// non-ANSI header with body port_declaration items admitted a spec-
// illegal §12.3.4 state (body port_declarations reference header
// port names; no header ports → no body port_declarations).
// Resolution: VerilogModule gained a NoPortsModule style-neutral
// variant (3-way Disj) with `items: List<AnsiModuleItem>` (no body
// port_declaration variants representable). The remaining
// NonAnsiVerilogModule + AnsiVerilogModule variants now have
// spec-non-empty header_ports — sites 25-26 — riding the same
// RQ-3 deferral.


// ─────────────────────────────────────────────────────────────────────
// ParameterAssignment — one (name, default-value) pair in a
// parameter_declaration's list_of_param_assignments (IEEE 1364-2005
// §A.2.4 / §3.10). A single declaration like `parameter WIDTH = 8,
// DEPTH = 16;` carries TWO ParameterAssignments. Conj record (no
// Practice-4 ledger).
//
```

**PR #3272 extension:** P8's constant-expression carrier adds sites 27-29:
`ConstantFunctionCall.arguments` (§A.8.2 `constant_function_call`),
`ConstantSystemFunctionCall.arguments` (§A.8.2 `constant_system_function_call`),
and `ConstantConcatenation.expressions` (IEEE 1364-2005 §A.8.1
`constant_concatenation ::= { constant_expression { , constant_expression } }`).
They ride the same Wave-A2 `List<T> where non_empty` deferral; no
Verilog-local non-empty carrier is introduced.

**Dissolution trigger:** `std/collection.dag` Wave-A2 `List<T> where non_empty` (29 total sites after PR #3272: merge-base sites 1-26 plus PR #3272 sites 27-29).
Upstream has partially landed the canonical predicate substrate (`non_empty`
backed by `free_monoid_non_empty`), but the `NonEmptyList<T> = List<T> where
non_empty` alias is not exported and consumer rewrites remain blocked until
`src/v3/compiler/src/lower.rs` synthesizes/enforces the registered `non_empty`
predicate for `List` carriers through the landed FreeMonoid predicate. Until
then, the merge-base `List<T>` sites and PR #3272 sites 27-29 stay producer-side
invariants rather than API-enforced refinements.

### SL-3229-VERILOG-D3200 — #3200 consumer-independent 🟡 coproducts (first-consumer decomposition)

Merge-base `verilog.dag` Practice-4 headers marked **🟡 YELLOW** under the **#3200** consumer-independent rule (namable richer source axes + first meaning-consumer decomposition), **distinct** from the Wave-A2 `List<T> where non_empty` ledger (`SL-3229-VERILOG-NONEMPTY`).

**Live coproduct one-liners cite this slug:** `NonTriregNetKind`, `VariableDeclaration`, `OutputPortAnsiVariableTypeKind`, `ParameterTypeKind`, `PrimitiveGateKind` (merge-base `92cb26402` — verbatim five-pattern ledgers + `#3200 RE-SCOPE` footers recoverable via `git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/verilog.dag`).

**Dissolution trigger:** first meaning-consumer owes the structural decomposition named in each carrier’s merge-base footer (D2 / synthesis / elaboration consumers — not a verilog-local mint).

### SL-3229-VERILOG-VECTOR-RANGE — CLOSED: `VectorRange` constant_expression endpoints

Closed by the P8 Verilog sub-grammar landing: `src/v4/extdeps/languages/verilog.dag`
now declares `ConstantExpression` and supporting constant-expression carriers,
and `VectorRange` stores `msb: ConstantExpression` / `lsb: ConstantExpression`
instead of raw lexeme strings. Historical merge-base context (lines **585–627**)
is retained below for auditability:

```text

// ─────────────────────────────────────────────────────────────────────
// VectorRange — a packed bit-vector index range `[msb:lsb]`
// (IEEE 1364-2005 §3.10 / §A.8.3). Conj record (no Practice-4 ledger —
// only coproducts dissolve).
//
// Per spec §A.8.3 grammar:
//
//   range ::= [ constant_expression : constant_expression ]
//
// where `constant_expression` admits integer literals, constant-
// identifier references (e.g. `WIDTH`, parameter names), arithmetic
// (`WIDTH-1`, `1<<N`), conditional, and function-call sub-forms per
// the full §A.8.3 / §A.8.4 productions. The prior `{ msb: Int, lsb:
// Int }` shape collapsed the constant_expression value-space into a
// literal-only set, making spec-legal parameterized declarations
// like `reg [WIDTH-1:0]` and `wire [(N+1)*8-1:0]` structurally
// unrepresentable (openai-pro REQUEST_CHANGES on PR #3176 2026-05-16
// at line 594). The bounded-Cardinality refinement trigger that
// previous scaffold cited was wrong-shape for the same reason: an
// integer-range refinement can never capture parameterized
// expressions; the canonical resolution is the constant_expression
// sub-grammar owned by the bundled T-4 LanguageModel.
//
//   - msb_lexeme, lsb_lexeme     the most- / least-significant
//                                bit-position constant_expressions
//                                as verbatim lexemes (`"7"`, `"0"`,
//                                `"WIDTH-1"`, `"(N+1)*8-1"`, etc.);
//                                sub-grammar deferred to the bundled
//                                T-4 LanguageModel via the
//                                established body_lexeme precedent.
//
// 🟡 SCAFFOLD (same bridge as the lexeme-of-expression precedent
// used throughout this file for body_lexeme / rhs_lexeme /
// default_lexeme):
//   (a) scaffold doc — THIS note; the lexeme carrier is provisional.
//       The terminal shape parses each lexeme into a
//       constant_expression AST.
//   (b) bounds on use — a producer MUST construct only lexemes that
//       satisfy IEEE 1364-2005 §A.8.3 constant_expression syntax;
//       arbitrary strings outside that production are producer-side
//       bugs until the LanguageModel parser enforces the constraint.
//   (c) dissolution trigger — when the bundled T-4 LanguageModel
//       productions for constant_expression land (the same place
//       body_lexeme / rhs_lexeme dissolve to), msb_lexeme /
//       lsb_lexeme are parsed into constant_expression ASTs and the
//       lexeme scaffold dissolves.
type VectorRange {
  msb_lexeme: String
  lsb_lexeme: String
}

```

**Dissolution result:** P8 landed; no live `SL-3229-VERILOG-VECTOR-RANGE`
slug remains on `verilog.dag`'s `// Ledger:` line.

### SL-3229-VERILOG-COST — raw-`Int` Verilog cost axes (`VerilogCost`)

**Authority / closure:** PR #3310 P1 cardinality refinement plus follow-up
receipt PR (2026-05-18).

**Disposition:** **🟢 GREEN terminal for the merge-base raw `Int` cost
payload class.** Live `verilog.dag` uses `Nat` for
`VerilogCost.gate_cost` and `VerilogCost.area_cost`, so negative cost is
structurally unrepresentable.

Verbatim `//` lines from merge-base `verilog.dag` (lines **4423–4456** — `VerilogCost` + 🟡 cardinality / non-negative cost scaffold):

```text

// ─────────────────────────────────────────────────────────────────────
// VerilogCost — the per-target cost-realization carrier shape for the
// Verilog `LanguageModel`. Conj record (no Practice-4 ledger).
//
// This declares the SHAPE the U1 spine reads when computing Verilog-
// target realization cost — the scaffold header's "Cost realization:
// per-construct cost on a Verilog target" claim:
//
//   - gate_cost     per-construct gate-count cost (an XOR, AND, OR
//                   primitive's gate equivalent; the U1 per-target-
//                   op cost projection — the `map` side of
//                   `Homomorphism<…, …, VerilogBits>` carries this;
//                   cost is read FROM the map's structure, never
//                   stored as a parallel field per algebra.dag's U1
//                   no-engine discipline)
//   - area_cost     per-construct area cost (in technology-dependent
//                   units — gate equivalents in a generic process,
//                   LUT count on FPGA, transistor count on ASIC;
//                   the technology-specific cost table sits in the
//                   inhabitance walk that the T-4 LanguageModel
//                   shape parameterizes when it lands, not in this
//                   carrier itself)
//
// Both Int carriers are 🟡 scaffold (same bridge as VectorRange and
// ptx Dim3): cost is a non-negative quantity, but without
// std/cardinality.dag refinement the negative-cost state is
// structurally representable. Producer-side invariant pending the
// cardinality substrate landing.
type VerilogCost {
  gate_cost: Int
  area_cost: Int
}

```

**Dissolution result:** P1's `Nat` carrier landing type-enforces
non-negative Verilog cost axes; no live `SL-3229-VERILOG-COST` ledger slug
remains in `verilog.dag`.

### SL-3229-FLOAT-NOMINAL — nominal width / interchange list-length scaffold

Verbatim `//` lines from merge-base `float.dag` (lines **104–144** — modeling notes through D3 cross-ref):

```text

// ─────────────────────────────────────────────────────────────────────
// Modeling notes (read before validating against the anchor)
//
// WIDTH NOMINALS — `Float32` / `Float64` are distinct nominal wrappers
// around ONE shared semantic carrier `FloatBody` (IEEE-754 *semantic*
// classification: finite sign + biased exponent + trailing significand
// as Peano `Nat`, vs non-finite tags). Binary32/binary64 **bit layout**
// (interchange formats) remain the authority of `Word32` / `Word64` in
// std/machine.dag — Grounding maps `Float*` ↔ interchange words; this
// file does not duplicate width-correct bit lists (same 🟡 list-length
// scaffold class as machine.dag's `Word*` fields).
//
// P2 NOMINAL WRAPPERS — `Float32` / `Float64` add **no** fields beyond
// `body: FloatBody` and therefore attach **no** structural witness tying
// `Finite.biased_exponent` / `Finite.trailing_significand` to binary32
// (8+23) vs binary64 (11+52) bit counts. A `Float32` value whose `body` is
// `Finite { … }` with exponent/significand magnitudes that could not decode
// from any `Word32` interchange pattern is **still representable** here —
// that hole is the same honest deferral class as **std/machine.dag**'s
// `Word*` / `Byte` list-length vs nominal-width **"LIST FIELD LENGTH vs
// NOMINAL WIDTH — TRACKED SCAFFOLD (🟡)"** block (single P2 authority for
// interchange bit-vectors: `Word32` / `Word64`, not a shadow width engine
// re-authored inside float.dag). Producer-side discipline + bounded
// refinement / dissolution named there close the gap.
//
// NAT FIELDS — `biased_exponent` / `trailing_significand` are honest
// unrefined `Nat` carriers: nominal IEEE field *widths* (8+23 binary32,
// 11+52 binary64) are NOT type-enforced here — the same honest deferral
// pattern as machine.dag's `List<Bit>` width scaffold. Producers must
// respect nominal widths; the dissolution trigger is the same bounded
// refinement substrate named in machine.dag's modeling notes.
//
// APPROXIMATEFIELD INHABITANCE — DECISIONS.md **D3**: a machine-readable
// `data …: ApproximateField<Float>` witness is **not** encoded in this file.
// Shipping left-operand `add`/`mul`, a non–bias-1023 `one`, or wiring those
// into inhabitance data would publish INVARIANTS P1/P3-violating substrate
// authority (codex REQUEST_CHANGES, PR #3191). Re-introduce the data row only
// when T-4 `extdeps/languages/*` supplies grounded Arrow bodies and/or v4
// admits bodiless primitive signatures (**PARSE-1**). Tier-2 compare and
// carrier-local `float_body_compare_semantic_total` remain honest scaffolds,
// not interchange-grounded IEEE `totalOrder`.
```

**Dissolution trigger:** bounded refinement substrate in `std/machine.dag` notes / Wave-A2 (merge-base cross-ref).

### PR-3252-extdeps-deferrals — Practice‑9 prose home (cardinality P1 slice)

**Authority:** still-hawk-102 / operator (2026‑05‑18) — live `extdeps/languages/*.dag` pointers only; rationale here.

- **`go.dag` / `GoNever`:** `std/cardinality.dag` `Never` is landed. This slice’s `GoScalar` is the six‑variant go1.26 predeclared scalar partition; there is **no** modeled bottom primitive in that closed set, so **no** `type GoNever = Never` D2a row. A row lands only after an operator‑ratified `GoScalar` extension (or an explicit encoding that bottom is control‑flow‑only without a scalar carrier).

- **`python.dag` / singletons:** `Unit` lands the shared cardinality‑1 authority for the three spec singleton kinds. Per‑singleton D2a rows remain **deferred** — but **under ruled-B the `{spelling}` `GroundingMap` home is retired**: grounding is a fold-discharged structural coincidence, never an asserted alias/spelling-map (`docs/modeling/grounding-worked-examples.md` §0). When the singleton carrier lands it grounds structurally (via `Unit` / a `PythonScalar` variant), not through a `GroundingMap` row. `python.dag`'s former `PyBool = Bool` is removed; the machine-readable grounding edge *declaration* `py_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` (decl-ref shared-authority, the truth facet) is present + v2-parse-verified — **P2/E-6 STAGING**, not landed authority (no same-PR consumer; canonical fold specified-not-realized). Python's data-model fact that `bool` is an `int` subtype is the build-up above the primitive — carried by `PythonScalar.BoolScalar` (first iteration; §0).

- **`rust.dag` / `RustNever`:** **A-vs-B = B (ruled-B).** The bare D2a(1) `type RustNever = Never` and the D2a(2) `{spelling}` `GroundingMap` shape are **retired**: identity is a claim discharged by the fold, never an asserted alias/spelling-map (the hollow form the machine-readable-inhabitance bar forecloses). Rust `!` / `bool`'s **classifier fact** is `RustScalar.NeverScalar` / `RustScalar.BoolScalar` (an honest surface enumeration). The **machine-readable grounding edge** *declarations* `rust_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` (decl-ref, single std authority) and `type RustNever = Never` (direct primitive identity — `!` is uninhabited, nothing above the primitive) are present + v2-parse-verified — **P2/E-6 STAGING**, not landed authority (no same-PR consumer; canonical fold specified-not-realized) — `docs/modeling/grounding-worked-examples.md` §0. The per-file `GroundingMap` twin is retired from `rust.dag` (cf. `machine_code.dag` post-D2-REV; INVARIANTS §P2). Numeric cost/width refinement (distinct from inhabitance `Never`/`Unit`) stays on its existing **T‑25 / nat / integer** triggers — unchanged by this row.

### SL-3309-PYTHON-SCALAR-RESEED — `extdeps/languages/python.dag` scalar-tower fact-bundle reseed

**Authority:** v4 T-4 Phase-3 fact-bundle rework (DECISIONS.md `D2-REV`); operator A-vs-B ruling **(2)** (2026-05-19, relayed via `vivid-carp-207`) — `#3309`'s Practice-8 dissolution carries unique value over the `Numeric`-tower wrapper `#3338` retained, so it is forward-ported onto post-`#3338` main.

**Disposition:** **🟢 terminal** — `PythonScalar` is reseeded from the `Numeric { tower: PythonNumericTower } | Singleton { kind }` form (as `#3338` landed it on main) into a flat coproduct `IntScalar | FloatScalar | ComplexScalar | BoolScalar | Singleton { kind: PythonSingletonKind }`. This dissolves two hollow defects `#3338` retained: the single-field `Numeric { tower }` wrapper (Practice-8 hollow prong — a wrapper adding no field of its own) and the bare `PythonNumericTower` enum (a classifier reading zero spec facts). The flat coproduct mirrors `rust.dag` `RustScalar`'s *structure* (a flat scalar coproduct with nullary variants), not its facts.

**Why the numeric variants are nullary (Python ≠ Rust).** `RustScalar` carries `kind` / `width` sub-carriers because Rust's integer/float primitives genuinely range over signedness and width (`i8`…`u128`, `f32`/`f64`). Python's numeric tower has **zero intra-kind variance**: exactly one `int` (arbitrary precision), one `float` (IEEE-754 binary64), one `complex` (pair of binary64) — Python Language Reference §3.2 (the standard type hierarchy). Per the CORE standing instruction — `rust.dag` is a *shape* reference, never a fact-template; a byte-symmetric copy asserting a non-fact is wrong — Python's numeric variants carry no fabricated `kind` / `width` carrier. A single-inhabitant carrier is also ungrammatical in v4 (`type X = Y` parses as an alias), and a free `kind × width` product over-generates invalid combinations.

**Bool grounding unchanged.** The canonical-B grounding mechanism `#3338` landed — `data py_bool_grounding: BooleanAlgebra<Bool> = bool_boolean_algebra` plus its `v4.std.logic` / `v4.std.algebra` imports — is preserved **byte-exact**; this reseed touches neither the data row nor its imports. Python's data-model fact that `bool` is an `int` subtype, formerly anchored at `PythonNumericTower.BoolLevel`, is re-anchored to the flat `PythonScalar.BoolScalar` variant.

**Ratchet authority:** this `###` row is the post-reseed Practice-4 authority for the live `PythonScalar` coproduct. `PythonSingletonKind` is unchanged from merge-base and retains `CP-3229-GREEN-TERMINAL`.

**PR receipt:** gunbc `#3309` forward-port (fresh PR onto post-`#3338` main).

### SL-3229-T4-FORMAT-T6T7 — T-4.6 format parse/emit bodies (compiler pipeline P3)

**Gate (live cite, Practice 9):** two adjacent in-file cites on each T-4.6 format carrier (`json.dag` / `yaml.dag` / `toml.dag`, …) so the parse trigger (P3) and emit trigger (T-10) are independently auditable — parse-only work does not imply emit-half closure, and vice versa:

- `🟡 gated — feature: dissolution-inventory §1.1 P3 — T-6 lexical-walk + T-7 parse-walk compiler pipeline-stage substrate (TASKS T-6/T-7) — DECISIONS.md Part 6 · SL-3229-T4-FORMAT-T6T7 (parse half).`
- `🟡 gated — feature: T-10 emit pipeline-stage substrate (TASKS T-10) — DECISIONS.md Part 6 · SL-3229-T4-FORMAT-T6T7 (emit half).`

**Named arrival:** B2-OMNI generic `tokenize` / `parse` over declarative `LanguageModel` lex + grammar `Node` data (`compiler/01_tokenize.dag`, `compiler/02_parse.dag`; TASKS.md **T-6**, **T-7**; L-5 `extdeps/languages/dag.dag`). The T-4.6 format files (`json.dag`, `yaml.dag`, `toml.dag`, …) **do not** host hand-rolled Char-stream parsers or emitters: `json_parse`/`json_emit` (and mirrors) land as **structural walks** composed on that pipeline substrate once realized beyond Wave-1 E0/G0 stubs. Emit stays fail-closed `Outcome<String>` (corrected seam #3, PR #3184 / msg_c7704bd6, INVARIANTS P3); inverse projection mates **`compiler/05_emit.dag` (TASKS.md T-10)** as the grammar-directed emit half of the same bidirectional seam — not a parallel string-templated “backend.”

**Dissolution trigger:** **Parse half:** T-6 and T-7 carry realized lex-walk + parse-walk bodies for grammar-as-data (dissolution-inventory §1.1 **P3**). **Emit half:** `compiler/05_emit.dag` is realized under **TASKS.md T-10** (grammar-directed inverse projection). Follow-up PR wires format **parse** bodies only after the **parse** trigger is satisfied; wires format **emit** bodies only after the **emit** trigger is satisfied — a parse-only landing does **not** dissolve the emit half (and vice versa). `std/text.dag` `Char` / `String` ↔ `List<Char>` decomposition (T-3) remains upstream **text** substrate for the walk’s stream spine — cite **P1** rows separately where the backlog conflated “numeric / refinement” deferrals (`SL-3229-LLVM-WIDTH` family), not mixed into vague “operations scaffold” prose.

**Roll-up:** `docs/audit/dissolution-inventory.md` §1.1 row **P3** (`compiler/01_tokenize.dag tokenize`, `compiler/02_parse.dag parse`, plus the parser-side in-file backlog this row concretizes).

### SL-3229-T4-FORMAT-TOML-DATETIME — `TomlDatetime` temporal interpretation (substrate gap)

**Gate (live cite, Practice 9):** `🟡 gated — feature: v4 temporal substrate for RFC 3339 datetime value interpretation (TOML §Date-Time four sub-kinds)`

**Named arrival:** `TomlDatetime` in `toml.dag` carries a verbatim RFC 3339 **lexeme**; **structured clock/calendar instant** interpretation requires v4 **`std/` temporal carrier + operations** in the **named scheduled file** **`src/v4/std/datetime.dag`** (module `v4.std.datetime` once authored; see **`TASKS.md` T-3** roster — file absent until landing PR). **Consumer / wiring owner:** **`TASKS.md` T-4.6** (`extdeps/formats/*`, including `toml.dag` operation(s) that consume the temporal facts). **This gate is NOT `SL-3229-LLVM-WIDTH`:** that row documents **LLVM LangRef raw-`Int` width payloads** on `LlvmType`; citing it for datetime was **wrong gate attribution** (Practice 4 accurate `feature:` naming, Practice 5 single authority).

**Dissolution trigger:** **`src/v4/std/datetime.dag`** is present in-tree with ratified temporal carriers + operations (**T-3**), and **T-4.6** wires typed interpretation (e.g. `toml_datetime_value : TomlDatetime -> Outcome<…>` over the four sub-kinds); this row closes.

**Roll-up:** **`TASKS.md` T-3** + **T-4.6**; **orthogonal** to dissolution-inventory **P1** cardinality-width family (`SL-3229-LLVM-WIDTH`, `SL-3229-PTX-COST`, …).

### SL-3229-JSON-UNIQUE-NAMES — JSON object unique-name profile

**Gate (live cite, Practice 9):** `🟡 gated — feature: JSON unique-name object profile parse/emit validation`

**Named arrival:** `JsonObject { members: Map<String, JsonValue> }` intentionally models a **unique-name JSON object profile**, not the full RFC 8259 §4 byte-stream space where duplicate names are permitted but receiver behavior is unpredictable. The profile is grounded in RFC 8259 §4's unique-name recommendation plus RFC 7493 (I-JSON) §2.3; a duplicate-name input must be rejected by the T-4.6 parser instead of silently last-wins collapsing into the `Map`.

**Dissolution trigger:** T-4.6 wires `json_parse : String -> Outcome<JsonValue>` so duplicate object member names are a typed `Rejected { diagnostic }`, and `json_emit : JsonValue -> Outcome<String>` continues to emit only the unique-name profile represented by `Map<String, JsonValue>`.

**Bounded use:** consumers may treat produced `JsonObject` values as duplicate-free by construction, but must not claim this carrier represents all RFC 8259 duplicate-name byte streams.

### SL-3229-TOML-TABLE-SYNTAX — inline-table vs table construction syntax

**Gate (live cite, Practice 9):** `🟡 gated — feature: TOML inline-table/table construction syntax collapse`

**Named arrival:** TOML inline table syntax and standard table syntax can denote the same TOML logical table value. `TomlInlineTable` and `TomlTable` currently have the same payload shape because the frozen scaffold named both; that same-payload discriminant is not terminal value substrate. It is a tracked construction-syntax distinction pending the operator header/contract reconcile that collapses value-level table representation to one table carrier while leaving parse/emit free to preserve or choose surface syntax.

**Dissolution trigger:** T-4.6 reconciles the TOML value model so inline-vs-standard table syntax is handled by parser/emitter construction facts, not by two value variants with identical `Map<String, TomlValue>` payloads. Until then, consumers must not infer semantic value difference from the two variants.

**Bounded use:** `TomlInlineTable` / `TomlTable` are acceptable only as scaffold-bound syntax provenance; no downstream model may treat them as two TOML value kinds.

### SL-3229-YAML-CANONICAL-KEYS — YAML mapping key canonical uniqueness

**Gate (live cite, Practice 9):** `🟡 gated — feature: YAML §3.2.1.3 canonical mapping-key uniqueness`

**Named arrival:** YAML 1.2.2 §3.2.1.3 defines mapping-key uniqueness by tag plus canonical content. `YamlMapping { entries: Map<YamlValue, YamlValue> }` only deduplicates by the current `YamlValue` structural representation, and `YamlInt` / `YamlFloat` preserve lexemes while numeric canonicalization is deferred. Therefore same-tag, canonically equal scalar keys with distinct lexemes remain representable in the carrier until parser-side canonical-key validation lands.

**Dissolution trigger:** T-4.6 parser work resolves tags, canonicalizes scalar keys using the text/numeric substrate, and rejects same-tag canonical duplicates as typed diagnostics before producing `YamlValue`; a later structural normalization may make the canonical key relation carrier-enforced.

**Bounded use:** consumers may not assume `YamlMapping` is YAML §3.2.1.3 duplicate-free from the `Map<YamlValue, YamlValue>` type alone; that guarantee belongs to the deferred parser/canonicalization gate.

### CP-3229-GREEN-TERMINAL — 🟢 GREEN five-pattern ledgers (bulk)

Merge-base `92cb26402` **Practice-4** `// Coproduct dissolution … 🟢 GREEN (terminal). Ledger — five patterns attempted:` blocks were adjacent to carriers (verbatim per-carrier text **only** in the merge-base object):

| File | GREEN ledger blocks (merge-base `rg` count) |
|------|-----------------------------------------------|
| `src/v4/extdeps/languages/llvm_ir.dag` | 29 |
| `src/v4/extdeps/languages/verilog.dag` | 41 |
| `src/v4/extdeps/languages/ptx.dag` | 13 |
| `src/v4/extdeps/languages/go.dag` | 10 |
| `src/v4/extdeps/languages/python.dag` | 3 |
| `src/v4/extdeps/languages/rust.dag` | 8 |
| `src/v4/extdeps/formats/json.dag` | 1 |
| `src/v4/extdeps/formats/yaml.dag` | 1 |
| `src/v4/std/float.dag` | 2 |

### CP-3229-VERILOG-CONSTEXPR-TERMINAL — 🟢 Verilog P8 constant-expression coproducts

PR #3272 adds Verilog constant-expression sum carriers not present in
merge-base `92cb26402`: `ConstantUnaryOperator`, `ConstantBinaryOperator`,
`ConstantRangeExpression`, `ConstantSelect`, `ConstantPrimary`, and
`ConstantExpression`.

Practice-4 terminal ledger: all of these are closed, spec-grounded enumerations
from IEEE 1364-2005 §A.8.3 / §A.8.4. They are not user-extensible vocabulary,
not coordinates of a product, and not consumer-local policy. Their variants
partition the standard's constant-expression grammar operators, selectable
constant-primary references, primary forms, and recursive expression forms.
The expression recursion is the grammar's own recursion; it does not add a
new substrate behavior or a second expression authority. Non-coproduct
records introduced with them (`ConstantFunctionCall`,
`ConstantSystemFunctionCall`, `ConstantConcatenation`,
`ConstantMultipleConcatenation`, `VectorRange`) are structural payload records,
not Practice-4 sums.

**Recovery:**

```bash
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/llvm_ir.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/verilog.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/ptx.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/go.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/python.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/languages/rust.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/formats/json.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/extdeps/formats/yaml.dag
git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:src/v4/std/float.dag
```

**Ratification:** strict de-prose did not alter `type`/`data` shapes; it removed `//` ledger text only. This subsection **re-anchors** the merge-base 🟢 GREEN classifications until a future ratified edit changes them. The format entries above are the authoritative recovery home for the live one-line `JsonValue` / `YamlValue` coproduct tags in `extdeps/formats/{json,yaml}.dag`; `TomlValue` is intentionally excluded because `SL-3229-TOML-TABLE-SYNTAX` supersedes its old green receipt with a 🟡 table-syntax collapse gate.

### CP-3229-RED-PRACTICE4 — 🔴 Practice-4 coproduct dissolution (allowlist)

Merge-base `92cb26402` **may** mark a sum coproduct **🔴** in the Practice-4 header (stop-signal / fail-closed disposition). Live substrate one-liners use **`// 🔴 coproduct dissolution — DECISIONS.md Part 6 · CP-3229-RED-PRACTICE4.`** — **not** `CP-3229-GREEN-TERMINAL` (that slug is **🟢 GREEN** bulk recovery only). Verbatim 🔴 five-pattern ledgers recover from the merge-base object the same way as 🟢 carriers; this row exists so the tag map never mislabels red as “green terminal.”

**Recovery:** `git show 92cb26402eeb21471acb6ac47559cbae3b52afdb:`*path* on the five allowlisted `.dag` files; search `Coproduct dissolution` + `🔴` in the recovered `//` text.


## Part 7 — Practice-4 coproduct classification ledger (PR #3213, still-hawk-102 Option-1)

> Worker-authored provisional, operator-ratified on audit (still-hawk-102
> Option-1, 2026-05-17). Scope: coproducts introduced by **PR #3213** in
> non-allowlisted **workflow** files — distinct from **Part 6 / #3229**,
> which relocates Practice-4 receipts for the five strict-de-prose
> substrate files (`SL-3229-*` / `CP-3229-*`). The coproduct carries the
> one-line in-file tag `// 🟡 coproduct dissolution — DECISIONS.md
> LB-P4-3213` (modeling-discipline.md Practice 4 / Practice 9 general form).

| ID | Coproduct / classification / dissolution-patterns-tried / trigger | Home |
|---|---|---|
| **LB-P4-3213** | `CiCommand` (`LintCommand \| TestCommand \| IgnoredTestCommand{test_name} \| BootstrapStageCompile{produces:Symbol} \| ShellCommand{command:String}`) — faithful PORT of v3 `dsl/gunbc/ci.dag` `CICommand` (still-hawk-102 fork-2 directive; not imported). **Single-authority (P2/Practice 5) — RESOLVED in-PR (openai-pro #3213 13971):** the bootstrap seed action is NOT restated in `ci.dag`; the `v2_compile_src_v4` job uses `BootstrapStageCompile{produces: v4_stage0_binary}`, a typed machine-readable reference imported from `v4.workflow.bootstrap` — `BootstrapPlan.seed` is the sole authority for the seed stage. **Structurally ENFORCED (P2/P3/Practice 5/6), not prose (openai-pro #3213 14006; operator BLOCKING inline #3213 ci.dag:168):** `ci_pipeline_well_formed` consumes the bootstrap authority `bootstrap_stage_output(plan: Outcome<BootstrapPlan>, s)` (owned by `v4.workflow.bootstrap`), which pattern-matches the canonical `bootstrap_plan` Outcome itself: fail-closed (`Rejected ⇒ false` — if the canonical bootstrap plan is Rejected, NO `BootstrapStageCompile` can satisfy the CI gate, so CI cannot be `Produced` while bootstrap is `Rejected`, INVARIANTS P3) and validates `produces` against the *validated plan's actual stage outputs* (`bp.seed/self0/self1.produces`), not a static symbol set — the validated `bootstrap_plan` is the sole authority (P2). Any out-of-plan or plan-Rejected payload routes to `ci_bootstrap_authority_violation`; a dangling payload cannot reach `Produced`. `BootstrapStageCompile` is 🟢 (a real cross-module authority edge, boundary-enforced, not deferred command-shape). The remaining 🟡 below is ONLY the `ShellCommand{String}` raw-argv command-shape decomposition, which is orthogonal and CORE-deferred to the consumer lane. **🟡 YELLOW (scaffold) — valid plan-bound, NOT "no change needed" (anti-#3250).** **Gate kind = `consumer:`** — the gate is the **first meaning-consumer** of the typed-command shape (deferred ci.yml projection / `select_jobs` / T-22 eval), which is **currently deferred-by-brief**, so the consumer-gate remains **CLOSED** and the #3244 gate-open→🔴 chain does **not** fire here. **Landed migration target:** `extdeps/process.dag::Command{program,args,env}` is **LANDED (#3209)** — it is the typed-command *feature/target*, NOT the meaning-consumer; the future consumer consumes typed `Command` **directly**, so **no parallel carrier is needed** and `#3213` does **NO migration** and **NO local `CiCommand` parse**. **Dissolution plan (complete #3244 plan-binding):** named consumer (ci.yml projection / `select_jobs` / T-22 eval) + landed target (`process.dag::Command`, #3209) + owning deferred lane — when that consumer lane is built it consumes `Command`, and `ShellCommand{command:String}`'s `String` dissolves there into typed `program/args/env`; the consumer owes the decomposition, not a local parse. **5 dissolution patterns tried:** (1) fact-placement FAILS (uniform command consumer, not scattered); (2) variant-is-data FAILS (heterogeneous payloads — `test_name` vs raw `command`; collapsing loses structural-intent-vs-raw-shell, the carrier's point); (3) algebraic N/A (not an algebra carrier); (4) dimensional FAILS (exactly-one-intent, not orthogonal axes); (5) parameterized-family FAILS (not `F<X>` over a declared set). Terminal-as-coproduct but 🟡 (not 🟢) because the consumer-gate is closed and the landed richer source (`process.dag::Command` #3209 / v3-F12) is the named decomposition target. | `src/v4/workflow/ci.dag` (`type CiCommand`) |

### LB-P10-3213 — Practice-10 hand-rolled `List` operation dissolution ledger

> Operator-flagged merge gate (still-hawk-102 via Lane B, 2026-05-18):
> per-file hand-rolled `List` ops with duplicate-across-files are a
> Practice-10 tell and must not merge as silent debt. Dispositions use
> the **#3244 unified Dissolution dispositions** vocabulary (🔴
> dissolve-now / 🟢 terminal / 🟡 gated `feature:`). Procedure result:
> `std/collection.dag` (T-3; `git ls-tree` HEAD = 18 lines, only
> `List`/`Set`/`Map` type aliases) declares **zero** derived `List`
> operations ⇒ **zero 🔴** (nothing to dissolve into in-PR); every
> generic primitive is **🟡 gated `feature:`**, owner **T-3
> `std/collection.dag`** (the FreeMonoid-derived List-op surface;
> `fold`/`map`/`count`/`concat` are language substrate primitives, used
> directly — not hand-rolled, out of scope). In-file tag:
> `// 🟡 List-op dissolution (Practice 10) — DECISIONS.md LB-P10-3213`.

| ID | Helper(s) — *duplicate-across-files = Practice-10 tell* | Disposition | Missing `std/collection.dag` op (gate kind `feature:`, owner T-3) · dissolve-on-arrival obligation |
|---|---|---|---|
| **LB-P10-3213-MEMBER** | `bs_member` (bootstrap.dag) ∥ `ci_member` (ci.dag) — **duplicate** | 🟡 gated `feature:` | `member(x: T, xs: List<T>) -> Bool` (membership/contains). On arrival: replace both call-sites with the std op; **delete both hand-rolled helpers**. |
| **LB-P10-3213-ANY** | `ci_symbol_resolves`, `ci_blocked` (ci.dag) | 🟡 gated `feature:` | `any(p: fn(T) -> Bool, xs: List<T>) -> Bool` (existential). On arrival: re-express as `any(...)`; delete the helpers. |
| **LB-P10-3213-ALL** | `ci_all_job_ids_unique`, `ci_all_gate_ids_unique`, `ci_all_needs_resolve`, `ci_all_gate_jobs_resolve` (ci.dag) | 🟡 gated `feature:` | `all(p: fn(T) -> Bool, xs: List<T>) -> Bool` (universal). On arrival: the four predicates become `all(...)` compositions; delete the bespoke folds. |
| **LB-P10-3213-COUNTIF** | `ci_id_occurrences` ∥ `ci_gate_id_occurrences` (ci.dag) — **duplicate** | 🟡 gated `feature:` | `count_if(p: fn(T) -> Bool, xs: List<T>) -> Int` (predicate count). On arrival: both occurrence-folds collapse to one `count_if`; delete both. |
| **LB-P10-3213-SETEQ** | `bs_list_eq` (bootstrap.dag) | 🟡 gated `feature:` | `set_eq(a: List<T>, b: List<T>) -> Bool` (membership-symmetric / multiset equality). On arrival: replace with std `set_eq`; delete helper. |
| **LB-P10-3213-FILTER** | `ci_eliminate_pass` (ci.dag) | 🟡 gated `feature:` | `filter(p: fn(T) -> Bool, xs: List<T>) -> List<T>`. On arrival: the keep-blocked pass becomes `filter(...)`; delete helper. |
| **LB-P10-3213-FIND** | `ci_job_needs` (ci.dag) | 🟡 gated `feature:` | `find`/lookup-first — honest shape `find(p: fn(T) -> Bool, xs: List<T>) -> Witness<T>` (per TASKS.md:235 `Map`/`PartialFunction` honesty; gated also on `witness.dag`/Wave-A2). On arrival: replace lookup-fold; delete helper. |
| **LB-P10-3213-KAHN** | `ci_kahn_fixpoint`, `ci_acyclic` (ci.dag) | 🟢 **terminal** | **Not** a reusable collection primitive: Kahn topological-elimination cycle-detection over the job graph — domain well-formedness model content, a *peer* of `ci_pipeline_well_formed` / `bootstrap_plan_well_formed` (which the gate does not ask to dissolve). Consumer-independent; no `std/collection.dag` op to dissolve into. (`fold`-as-bounded-counter is the P4 decidability idiom.) Its generic sub-primitives (`ci_member`/`ci_job_needs`/`ci_eliminate_pass`/`ci_blocked`) dissolve via the rows above; the Kahn *composition* stays. |

**Note (out of dissolution scope, recorded for completeness):** `bs_diagnostic` / `ci_diagnostic` are `Diagnostic` constructors, not `List` operations. No 🔴 in this ledger — `std/collection.dag` currently has no derived-op surface to dissolve into; the gate opens when T-3 `std/collection.dag` lands the List-op surface, at which point every 🟡 row above is a dissolve-on-arrival merge obligation.

### LB-T22-3213 — bootstrap-stage rejection-family negative-coverage plan-bound 🟡

> CORE ruling (still-hawk-102, 2026-05-18, horn (i)): the
> `BootstrapStageCompile` single-authority seam is **ADDRESSED-BY-CONSTRUCTION** —
> `ci_pipeline_well_formed` is a pure structural predicate over the modeled
> `CiPipeline`; an out-of-set `BootstrapStageCompile.produces` cannot satisfy
> the gate (deterministically routes to `Rejected{ci_bootstrap_authority_violation}`,
> a modeled `Outcome` variant — no imperative side-channel / stringly exception).
> Verified in code by `bold-hawk-201` @ `6353d695e`. Horn (ii) — in-PR
> executable negative harness — REJECTED (T-22-in-#3213 = brief violation;
> hand-rolled harness = parallel test mechanism, anti-pattern).

**🟡 plan-bound (NOT "no change needed" — anti-#3250).** The enforcement is structural and fail-closed *now*; what is deferred is the executable *demonstration*. **Arrival:** T-22's executable `TestClaim` runner lands (`compiler/05_eval.dag`; brief defers the executable TestClaim lane to the T-22 named trigger). **Follow-up (dissolves this 🟡):** add negative `TestClaim`(s) for the CI bootstrap-stage rejection family — dangling `BootstrapStageCompile.produces` + siblings (duplicate job/gate id, dangling `needs`, dangling gate job, dependency cycle) — exercising `ci_pipeline_well_formed`'s `Rejected` branches. **Bilateral binding:** the same obligation is recorded in `src/v4/TASKS.md` T-22 scope text (neither side is a vague "T-22 will cover"). In-file tag: `// 🟡 negative-coverage plan-bound (T-22) — DECISIONS.md LB-T22-3213`. | `src/v4/workflow/ci.dag` (`ci_pipeline_well_formed` rejection family) |

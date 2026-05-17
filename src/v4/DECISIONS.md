# v4 — Design Decisions Ledger

The single record of v4's design ratifications. Two states:

- **RATIFIED** — operator-decided and binding. Encoded in the cited file(s).
  This ledger is just the index; the file headers are authority.
- **PROPOSED** — a recommended resolution awaiting your ratify-or-redirect.
  Nothing here is decided unilaterally. The recommendation is my read,
  with the tradeoff stated honestly so you can overrule it cheaply.

How to use this PR: ratify or redirect each PROPOSED item (review comment
or merge-with-notes). On ratification, each item is encoded into the
cited file headers — the same flow the RATIFIED items already went
through — and moves to Part 1.

---

## Part 1 — RATIFIED (this session, 2026-05-15)

| ID | Decision | Encoded in |
|----|----------|-----------|
| **A1** | One recursive type (Node); 6 connectives + 5 behaviors are orthogonal flat-discriminant axes; behaviors structural not lens-derived; recursive generics are Node-underlying (Instantiation + name-ref); regular recursion admissible, non-regular polymorphic recursion a STOP absent a decidable bound | `std/node.dag` header |
| **A2** | Termination is Tier-1; `Loop` = bounded recursion total-by-construction; descent evidence implicit-on-sub-Node or explicit ranking; checker not discoverer; the bound IS the cost-lens datum; residual boundary stated | `std/node.dag` header |
| **A3** | Retirement = a reproduction predicate, not a count; it is an early-surfacing amplifier, NOT un-gameable (Trusting-Trust); seed trust is a named axiom; culture + operator-ratification is the actual enforcement | `STRUCTURE.md` §7, `workflow/bootstrap.dag`, `TASKS.md` T-5/T-15 |
| **A4** | Folded into A3 — a 7th connective is the same machine: deviation changes the reproduction → conspicuous signal → STOP. Not "the substrate refuses by construction" | `STRUCTURE.md` §7, `workflow/bootstrap.dag` |
| **U1** | One homomorphism-with-cost carrier; the back half is five phases (Find/Realize/Measure/Compare) not five engines; no-engine discipline (empty search → helpful Diagnostic, never fabrication) | `std/algebra.dag` header |
| **C2** | Synthesis = compare(derived cost, the relation's structurally-derived lower bound) via a closed `LowerBoundTechnique` set; NOT a (naive→better) rule library; honest worked examples preserved | `lens/synthesis.dag`, `TASKS.md` T-17 |
| **U2** | `complexity.dag` *consumes* `cost.dag`'s `SymbolicCost`, never re-derives; **cost is TOTAL over the closed kernel** — no per-feature opt-in, so a future addition cannot bypass cost (the no-eternal-maintenance property) | `lens/cost.dag` + `lens/complexity.dag` headers |
| **C3** | `normalize` dissolves exactly the 4 bounded sugar forms (`service/fn/type/operation`) into Node; not identity, not open-ended; new sugar = STOP | `compiler/03_normalize.dag` + `TASKS.md` T-8 |
| **C1** | `infer` is the bounded *Find* phase — bounded structural unification over the finite (A1/A2) space; empty ⇒ Diagnostic, never fabricated coercion | `compiler/04_infer.dag` header |
| **B3** | The type signature is the single effect authority; the effect lens reads it; coordination effect-types are carriers over it, not a parallel taxonomy | `lens/effect.dag` + `extdeps/coordination.dag` |
| **C4** | On-disk emitted artifacts (`ci.yml`, trampoline) are committed==emit(source) checked projections, not editable authority (same machine as A3) | `workflow/ci.dag` + `STRUCTURE.md` + `workflow/bootstrap.dag` |
| **C5** | Ingest = emit⁻¹ on each model's lossless core; outside ⇒ fail-closed Diagnostic. Guaranteed ASAP via testgen `bidirectional_roundtrip` (Phase-1.5, gated per-PR via the B1 hash from first language-model commit) | `extdeps/languages/*` + `lens/testgen.dag` + C5 note |
| **C5-fidelity** | Round-trip fidelity is **parameterized by the model, not a fixed "trivia" category** (operator-corrected 2026-05-15). Each surface feature, per language, has an explicit disposition in the model: **Modeled** (∈ F → Node-bearing; both `ingest∘emit=id` and `emit∘ingest=id` for that feature — e.g. Python indentation IS block structure; comments IF modeled) \| **Declared-normalized** (deliberately not in F; `emit∘ingest` canonicalizes it — Go/C++ insignificant whitespace; a *declared* loss, reviewable, never silent) \| **Fail-closed** (encountered but neither → Diagnostic, no-engine). `emit∘ingest=id` holds restricted to F; "lossless core" ≡ F; round-trip fidelity = model completeness, boundary declared not assumed. F is drawn from the spec's own meaning-vs-lexical distinction (see L-2) | `extdeps/languages/*` headers + `TASKS.md` T-4 + C5 |
| **L-2** | **Model the language SPECIFICATION, not libraries** (operator-ratified 2026-05-15). `extdeps/languages/*` models the versioned upstream spec (grammar+type-system+semantics; the anchor IS that spec by construction — cf. the verilog→IEEE-1364 / spice→SPICE3f5 fix). Libraries (std/crates/packages) are NOT modeled — a library is just a program in the modeled language = `Node` like any other. This makes F principled (the spec defines meaning-vs-lexical, not our judgment) and maximally general. **Frameworks rule (BINDING — cascade-closure of L-2, resolves 3250029061; no PROPOSED residue):** a `frameworks/*` model is admitted IFF the framework constitutes a sub-language with its own non-host-expressible rules (React Rules-of-Hooks / effect-lifecycle / reconciliation) — that IS a "language specification" under L-2, so L-2 *already* decides this; a framework that is only a library API surface is a library, which L-2 already says is NOT modeled (it is just `Node`). Not a new fork — the logical closure of the ratified L-2 principle; T-4.7 has binding spec-vs-library authority | `extdeps/languages/*` + `extdeps/frameworks/*` headers + `TASKS.md` T-4 |
| **K-1** | Kernel identifiers are OPAQUE `Symbol`s (equality only, no content accessor); spelling lives in a boundary table, never kernel/lens-readable — structurally forecloses the v2 string-heuristic channel | `std/node.dag` header (T-1 worker encodes) |
| **IR-1** | `InferredTree` is NOT a new type — it is `Node` + a FROZEN flat `InferredFacts` coordinate (resolved_type, inhabits, cost, descent); **cardinality and effects are NOT fields** — both already carried by `resolved_type`'s Node structure (the Cardinality connective / the signature), caching = second authority P2; a 5th field is a STOP; modeled up front, not emergent | `compiler/04_infer.dag` header |
| **B2-OMNI** | parse is one ingestion instance; `ingest`/`emit` are parameterized boundaries over declarative LanguageModels; Node is the universal pivot ⇒ O(N+M) not O(N×M); `.dag` is language #1, never hardcoded | `compiler/00_compile.dag` + `01_tokenize`/`02_parse`/`05_emit` headers |
| **L-1** | Verilog/SPICE/English added as **parallel B2-OMNI falsification probes** (T-4.9/4.10/4.11), maximally diverse on purpose. Forks resolved: English = boundary-honesty probe (Shape B emit + fail-closed ingest TestClaim), **not** a language model; SPICE = `extdeps/formats/` (netlist is a data format); Verilog = the IN-B concurrency validation (a needed 6th behavior = C1 escalation, by design caught early) | `TASKS.md` T-4.9/4.10/4.11 + `extdeps/languages/verilog.dag`, `extdeps/formats/spice.dag`, `test/claim/boundary/english_ingest_fail_closed.dag` |
| **L-3** | **Down-the-stack** B2-OMNI probes added (operator-ratified 2026-05-15, parallel up front): T-4.12 `extdeps/languages/llvm_ir.dag` (SSA IR — generalize down the stack; F ~all-structural), T-4.13 `extdeps/languages/machine_code.dag` (bottom of stack; no cosmetics; disassembly = the extreme fail-closed/no-engine test), T-4.14 `extdeps/languages/ptx.dag` (CUDA SIMT — IN-B bet vs the 5 behaviors). Validates the model across the **full target spectrum** (source→IR→machine code) — concretizes C5-fidelity ("we don't necessarily output a binary; F is model+intent-relative"). **Forks DISSOLVED (cascade-closure, resolves 3250390911; no PROPOSED residue):** neither was a genuine open fork — each is forced by an already-ratified parent, and the alternative *contradicts* a ratified decision. (1) machine_code = ONE model parameterized by an `Isa` — **forced by B2-OMNI** (parameterize-not-enumerate / O(N+M); per-ISA files ARE the N×M trap B2-OMNI forbids). (2) CUDA = model **PTX** — **forced by L-3's own ratified down-the-stack-IR scope**: PTX is the SIMT *IR* layer (the down-the-stack probe T-4.14 exists to test); CUDA-C++ is just another Shape A *source* language already covered by general modeling, so it would not be a down-the-stack probe at all. Per P5 the fork dissolves into already-ratified coordinates, not a new decision | `TASKS.md` T-4.12/4.13/4.14 + `extdeps/languages/{llvm_ir,machine_code,ptx}.dag` |
| **L-4** | **PROOF-1's prover model lands as `extdeps/languages/lean.dag`** (B-2, side-session 2026-05-15 — operator-ratified). PROOF-1 (#3158, merged) was framed "no new file" and explicitly deferred its prover model to "realized when a lean/coq model lands"; this IS that model. B2-OMNI **structurally requires** a declarative `LanguageModel` file to emit to any target, so PROOF-1's "emit to a lean model" needs exactly this file — adding it is the closed-tree extension operator-ratified here (STRUCTURE.md invariant #1). **ONE prover first — Lean** — scoped to the first theorem class **termination** (A2 structural descent ≈ Lean well-founded recursion / `termination_by`). **Coq is the DEFERRED second-prover falsification probe** ("does the model generalize across provers?" — O(1) per B2-OMNI, added only when a theorem class needs it; same probe posture as L-1/L-3; a future operator-ratified extension, never a silent TODO). Per L-2 the Lean *spec* is modeled, not Mathlib. STRUCTURE count 63→64 .dag | `extdeps/languages/lean.dag` (new) + `STRUCTURE.md` languages enumeration/count + `TASKS.md` PROOF-1/T-15 note; cross-ref DECISIONS PROOF-1 / B2-OMNI / L-2 |
| **L-5** | **`extdeps/languages/dag.dag` admitted — gunbc's native `.dag` LanguageModel** (operator **Option A**, C1 closed-tree extension 2026-05-16, relay merry-ibex-337). `.dag` is B2-OMNI **language #1** (never hardcoded into the generic `01_tokenize` / `02_parse` walkers); lexical + syntactic authority is declarative **`Node` grammar-data** here, same `extdeps/languages/*` bundle shape as the other Shape-A models. L-2 **spec-first** without an external URL: anchor is repo-owned **`docs/v3-spec.md`** plus the v4 closed tree (`STRUCTURE.md`, `TASKS.md` T-6/T-7). PARSE-1 bodiless-`fn` is a **grammar production in data**, not walker code. STRUCTURE count **64→65** `.dag`. | `extdeps/languages/dag.dag` (new) + `STRUCTURE.md` languages line + total; cross-ref `compiler/01_tokenize.dag` / `compiler/02_parse.dag` + B2-OMNI + PARSE-1 |
| **QRY-1** | **RETRACTED & reframed (operator-corrected 2026-05-15).** "query = user-defined lens" is wrong: user-vs-kernel is PROVENANCE, not a structural distinction (feedback_naming_is_aliasing) — a decorative name distinguished only by who-authored-it must not exist ("a real distinction, or not exist"). There is NO query concept/noun/subsystem. The REAL axis is **scope/parameterization**, already modeled with no new noun: (1) `SectionRef` (DeclarationScope\|NodeScope) — applying a lens AT a scope; (2) parameterized lens signatures — the **affected-set lens (T-21) `(Dag,Diff)->Witness<ReExecFrontier>` is the archetype**: "the affected set of THIS change" = `affected_set(dag,diff)`, already first-class. Lenses, read at scopes and/or parameterized. No `query.dag`. The B2-OMNI "one read over the Node pivot, all ingested languages" property holds for ANY lens, not a "query" | `lens/application.dag` header + THESIS §1.5 + `lens/affected_set.dag` (T-21) |
| **PAR-1** | Parallelism (T-13) is a THESIS free consequence, a target-AGNOSTIC lens reporting the independence **FACT** (`Witness<ParallelismMap>`); WHETHER/HOW to realize it is target-DEPENDENT — a cost-derived decision (U1/U2 per-target) at emit(T-11)/eval(T-22), NOT the lens. independence = structural-disjointness ∧ effect-independence (consumes `lens/effect.dag`, B3 — Node-disjoint is necessary not sufficient) ∧ fold-combiner associativity-by-inhabitance (consumes `std/algebra.dag` — parallel-reduce is an algebra fact, not a heuristic). fact ≠ spawn-decision; no annotation, no scheduling heuristic. **User-facing contract:** NOT a "parallelize my code" command (forbidden — engine-shaped from the user's seat); a lens the user READS (same UX as complexity/cost); control = structure not a knob; default introspective with the realize-decision derived+inspectable per target (e.g. honestly "no — CPython GIL"); guarantee via `apply_lens(parallelism, Enforce)` (T-23, uniform opt-in-depth). 4 worked examples embedded | `lens/parallelism.dag` header + `TASKS.md` T-13 |
| **B1** | One content-addressing scheme: a `std`-level Merkle catamorphism `content_hash : Node -> Hash` over the **canonical-form clause** (a SEPARATE explicit node.dag authority — NOT subsumed by A1, which only covers recursion/axis-closure/generics/termination). `Hash` is an opaque digest (no content accessor, K-1-style). One scheme consumed by T-15/T-20/T-21 + A3 + C5 + structural build/CI artifact caching (a cache key IS `content_hash` of the input subgraph — see `lens/affected_set.dag`). NOT workflow-level | `std/node.dag` — the `Hash` type lands here (re-homed beside `Symbol` by the primitive.dag-decomposition; `Hash` is a content identity, not a scalar); the **B1-CANON block** is the canonical-form clause — operator-DESIGNED & ratified in-header 2026-05-15 (pulled back from worker delegation because every consumer rests on it); the canonical-form clause + `content_hash = merkle_fold ∘ canonical` fold land in `std/node.dag`; T-1 IMPLEMENTS that contract, it does not design it |
| **PARSE-1** | Bodiless `fn` (signature-only / contract-as-checked-signature) is **not needed in the v2-bridge era** (operator-ratified 2026-05-15) — verified by constructive pre-flight: frozen v2's parser **rejects** bodiless `fn` (`expected LBrace`) but cleanly parses the full ratified kernel shape **with bodies** (40k-line `src/v2` corpus + kernel probe, zero syntax errors; the 65 src/v2 errors are 100% post-parse import/cycle). So bridge-era substrate contracts stay **comment-prose** (current scaffold form) and the fail-closed parse-class gate is SAFE — v2 is never asked to parse bodiless fn. **Forward requirement (the conditional that makes the deferral acceptable):** v4's OWN `.dag` grammar (T-7) MUST admit a bodiless signature declaration as a first-class production — graduating the "immutable I/O contract for the worker" from comment-prose to **compiler-enforced** (filled body must conform to the frozen signature; divergence = fail-closed type error, structural per no-annotations). Supportable by construction (v4's own grammar; nothing external constrains it); per B2-OMNI it is a grammar-DATA production, not hardcoded in the walker. v4-self-parser-era only — never through the v2 bridge | `compiler/02_parse.dag` header + T-20 pre-flight note |
| **D1** | The structural-delta vocabulary over `Node` (side-session 2026-05-15; **shape refined** per codex P2 REQUEST_CHANGES on PR #3162). An `Edit` replaces the subtree at a `Path` with a `Node` (term-rewriting `t[s]p`). `Path = List<Symbol>`: each step descends the `Named` edge keyed by that opaque `Symbol` — flat & finite, NOT a new recursive type (A1); resolution = `Symbol` equality ⇒ K-1-clean. Name is **`Path`** (the route), not `Position` (operator pick); "position/occurrence" (JSON Pointer, RFC 6901, member-name tokens only) retained as anchor. **No positional/ordinal selector** (P2 finding 1): an `Int` ordinal admits negative/illegal indices = the representable-illegal-state node.dag designed out at `EdgeLabel`; positional-connective children are reached by replacing the enclosing `Named` subtree (the safe over-fire #3157 blesses); surgical positional addressing is a future operator-ratified extension (consumes-nothing root has no ordinal primitive — no kernel `Nat`; recursive Peano = A1 STOP). **`Diff` = ORDERED `List<Edit>` applied SEQUENTIALLY** (P2 finding 2): no set / pairwise-independence / "parallel positions" invariant, no `diff_well_formed`/`paths_independent` — any `List<Edit>` is a valid rewrite program; a non-resolving Edit ⇒ fail-closed Diagnostic at apply time (operational, not a type-level illegal state); `apply_diff` is a fold, all-or-nothing. **DATA TYPES in `std/node.dag`** (B1/`Hash` precedent: a structural delta over `Node` is substrate-tier; corrects the initial "owner = T-21" read). **OPERATIONS** (`subterm_at`/`apply_diff` → `Result<Node, Diagnostic>`) in **`lens/application.dag`**, FORCED: `std/diagnostic.dag` already `Consumes std/node.dag` (`Correction = Suggested(Node)`) so a node.dag op returning `Result<_,Diagnostic>` is a substrate cycle — the node.dag "consumers fail-close" discipline (`node_well_formed` precedent). `apply_diff` consumer = CLI / test harness / user app code. Two refinements of the in-session ratification were encoded via PR #3162 review (the normal confirm/redirect channel — not an open fork): (a) the data/op split (operations in `application.dag`, forced by the diagnostic→node cycle); (b) the P2-driven shape change — no positional selector (Practice-4 dissolved; receipt in `std/node.dag`) and sequential `Diff` (no independence predicate). Operator redirect, if any, is via the PR like any change | `std/node.dag` header + Practice-4 receipt + Path/Edit/Diff decls + `lens/application.dag` header (operation contract) + `lens/affected_set.dag` cross-ref |
| **B-4** | affected_set R2 minimality contract (side-session 2026-05-15). For `lens/affected_set.dag` (T-21) to compute a *minimal* re-exec frontier, `compiler/04_infer.dag` resolution must be **NODE-PRECISE**: a use-site name-reference `Atom` binds to the **exact declaration `Node`**, never merely its module/scope (**carrier corrected** per a P2/Practice-3 review finding on #3162). The use→def fact's named forward carrier is the post-resolution opaque **`Symbol`** (K-1) on the name-ref `Atom`, **NOT `resolved_type`** (that is the node's *type* — many declarations share a type, so it is lossy for binding identity; citing it was the P2/Practice-3 error). AUTHORITY = `compiler/03_resolve.dag` (it owns identifier→declaration binding; resolution canonicalizes a use-site `Atom`'s `Symbol` to equal its binder's `Symbol`, so binding = `Symbol` equality — the K-1-granted op, node.dag `name_occurrences`). `04_infer` is **preserve-only** (must not alter/drop the resolved `Symbol`; no 5th `InferredFacts` field — IR-1-safe). `lens/affected_set.dag` **derives** the graph by `Symbol`-equality traversal over the resolved tree (same "derive, don't store" as `content_hash`/D1). Coarse (module/scope) resolution leaves affected_set *correct but maximally over-firing* — so node-precision is a **precision INVARIANT on T-9**, the dominant gate on R2 minimality, not a T-21 concern. Distinct axis from #3157's hash-granularity "over-fire = safe" tolerance (content identity vs dependency-edge granularity). No new type/field; IR-1-safe | `compiler/04_infer.dag` B-4 header block + `lens/affected_set.dag` Consumes cross-ref |
| **B-5** | affected_set R1 usefulness contract (side-session 2026-05-15). For purity-aware skipping to be *useful* (not merely correct), effect must be a **per-`Arrow`-Node fact** derived from that node's signature (the B3 authority) and **never aggregated to a coarser unit** (one ambient `IO`, per-module effect set) — a coarse aggregate is the parallel taxonomy B3 forbids (P2) *and* degrades R1 to "re-run ~everything" (still correct/fail-closed, useless). A **granularity invariant**, not effect-lens strength (effects are READ off the signature per B3 ⇒ strong by construction; the only risk is coarse modeling — this forbids collapsing the per-effect-KIND distinction the signature already carries). No new field. = #3153 Ex.R1, now encoded. **Corollary (effects-as-parameters, 2026-05-16):** effects are threaded as explicit parameters (DI-by-construction); a mock-injected unit test's B-4 edge points at the mock `Node`, so affected_set skips it and re-runs only real-carrier (integration) tests *automatically* — "affected" is consumer-relative with no unit/integration mode in the lens. Ambient/global effects re-couple the cone = the anti-pattern this forbids. Honest boundary: a re-run test with changed behavior may have a stale expected value; the lens surfaces "re-run", never rewrites the expectation (no-engine) | `lens/effect.dag` B-5 block (+corollary) + `lens/affected_set.dag` Ex.R1/Ex.R1' |
| **#3153-subsumed** | `v4-affected-set-design` (#3153, no worker) is **superseded by this branch** (operator 2026-05-15): its 4 worked-examples + adversarial R1/R2/R3 + dogfood-reality are folded into `lens/affected_set.dag`'s header as the T-21 design contract, reconciled to the now-*encoded* reciprocal contracts — Ex.R1→B-5 (`effect.dag`), Ex.R2→B-4 (`04_infer.dag`), Ex.R3→B-3 (the drafted #3157 B1-CANON point-3 correction). #3153 carried no new file/type; close it as superseded | `lens/affected_set.dag` WORKED EXAMPLES header section |
| **PROOF-1** | **External trust-discharge for A3** (operator-ratified 2026-05-15; *not the north star — placed & designed early like AGENT-1*). gunbc's structural evidence (A2 termination descent, the algebra-homomorphism epistemic chain, cost, effects) is EMITTED as a machine-checkable proof term in an external prover (Lean/Coq); their small independently-audited KERNEL checks it. Converts A3's named seed-trust axiom into an INTERSUBJECTIVE check against an anchor gunbc does not own. Does NOT make gunbc un-gameable — the external kernel + the evidence→proof-term faithfulness are the NEW named axioms; it MOVES trust to a stronger anchor (same Trusting-Trust honesty A3 observes). Constraints: the lens EXPORTS witnesses gunbc already has, the prover only KERNEL-CHECKS, never searches (no-engine + A2 "checker not discoverer"); discharges only what is structurally GROUNDED (ungroundable ⇒ fail-closed Diagnostic — inherits gunbc's honesty boundary). Realized as (evidence read) ⊕ (B2-OMNI emit to a `lean`/`coq` language model — ordinary targets). The PROOF-1 *framing* (#3158) added **NO new file** — it was framing-only and explicitly DEFERRED the prover model ("realized when a lean/coq model lands"); that model **subsequently lands as `extdeps/languages/lean.dag` per B-2/L-4** (operator-ratified closed-tree extension — see the **L-4** row). No contradiction: framing = no file; the deferred model's later arrival is the L-4 extension | `STRUCTURE.md` §7 (A3 home) + `workflow/bootstrap.dag` A3 block + `TASKS.md` T-15 note; cross-ref A2 / AGENT-1 / B2-OMNI / **L-4** |
| **C5-2** | **emit-locality** (operator-ratified 2026-05-16) — `emit` is *structurally local*: a subtree's emitted form depends only on that subtree + a bounded declared context, so a D1 `apply_diff` at Path p re-emits ONLY p's subtree; everything outside p is unchanged by construction. **Locality is Node-relative, NOT file-relative** (operator code-as-data steer: the substrate anchors no semantic on the file concept — a "file" is a convenience projection only; stability = content_hash of the emitted *subtree*, never file bytes). Non-local emit decisions are DECLARED normalizations at the smallest *enclosing Node scope* (never "file scope" — there is no file in the model). Load-bearing: AGENT-1/D1 "faithful re-emit" is unprovable from global `ingest∘emit=id_Node` (C5-1) alone — locality is the stronger separate property; the C5↔D1 hinge | `compiler/05_emit.dag` C5-2 header block + Owns line; cross-ref C5 / C5-fidelity / D1 |
| **T-9** | **infer keystone contracts** (operator-ratified 2026-05-16). **A — Find decidable by construction** (closes C1): the homomorphism Find recurses structurally on Node (A2 implicit descent ⇒ the inferencer terminates); the per-node candidate set is EXACTLY the closed declared inhabitances in `std/*` (enumerable, never synthesized/open), so "no homomorphism" = "exhausted the finite declared set" = a *decidable* fail-closed Diagnostic, not a non-terminating search. Synthesizing candidates = left the decidable fragment = STOP. **B — single-file discipline is a scope boundary**: v2's 12-file split was env/scope/lookup/method-resolution tangling; in v4 those are 03_resolve's job (B-4: binding = canonical `Symbol`), so `04_infer` owns ONLY {C1 Find + cardinality propagation + diagnostic precision}. A worker writing scope/env/binding/lookup here re-imported the 12-file pressure → STOP (belongs in 03_resolve). The single file holds because the scope is bounded, not willed | `compiler/04_infer.dag` T-9 header block; cross-ref C1 / IR-1 / A1 / A2 / B-4 |
| **C5-1** | **The C5 round-trip law is node-level** (operator-ratified 2026-05-16). A `bidirectional_roundtrip` TestClaim asserts `ingest∘emit = id_Node` via B1 `content_hash(N) == content_hash(ingest(emit(N)))` — the *meaning* round-trips; it does NOT assert text-byte identity (`emit∘ingest = id_text`). Lexical skin (`0x1F`/`31`, whitespace, comments) is allowed to canonicalize. **Text-level fidelity is a DEFERRED, ORTHOGONAL future layer**: a concrete-syntax/trivia sidecar *outside* the Node and *outside* `content_hash`, partial (ingest-originated Nodes only) — it never changes C5-1 or `content_hash`. The C5-fidelity Declared-normalized per-feature dispositions are its forward hook; adopting node-level + the disposition table preserves text-level as a zero-rework future add. The non-orthogonal route (lexical choices stored in the Node) is rejected (breaks α-/rename-invariance, dual authority) | `lens/testgen.dag` C5-1/C5-3 block + DECISIONS C5 / C5-fidelity |
| **C5-3** | **F = the spec-provable meaning core; spec silence ⇒ out of F** (operator-ratified 2026-05-16; conservative/fail-safe). If the language spec does not *unambiguously* place a feature on the meaning side, it is NOT in F — Declared-normalized (emit canonicalizes; recorded per-feature) or Fail-closed (ingest needs semantics it lacks ⇒ Diagnostic, never a guess). The **spec decides; the modeler has no discretion**. Consequence: a language's lossless core may be strictly smaller than its spec (e.g. C++ most-vexing-parse → out, Fail-closed), declared per-feature **up front** in the C5-fidelity disposition table, never discovered at round-trip-test time. Sharpens the pre-existing C5 / C5-fidelity rows | `lens/testgen.dag` C5-1/C5-3 block + `extdeps/languages/*` C5-fidelity dispositions |
| **AGENT-1** | The **non-text agent surface** (operator-ratified 2026-05-15 — *include & design early; explicitly NOT the north star*; "a very important/exclusive feature" but a NATURAL CONSEQUENCE, not a new subsystem). A non-text client (LLM/agent) interacts with the program **as data, never files**: READ = `apply_lens` at `SectionRef` + `affected_set(dag,diff)` (the affected lens replaces file-reading exploration); WRITE = the **D1** `apply_diff:(Node,Diff)->Result<Node,Diagnostic>` (the ratified D1 operation — NOT a total `->Node`; an agent-supplied Diff is external/possibly-stale input ⇒ FAIL-CLOSED typed Diagnostic, all-or-nothing, INVARIANTS P3 — AGENT-1 uses D1's op, does not restate a total one) entering the B2-OMNI pipeline at **`core`** (a SIBLING of `ingest`, NOT an ingest — the change is already Node: no source text, no LanguageModel, no parse), content-addressed (B1), not a text patch; BACK = the `affected_set` lens read in whatever shape T-21/`lens/affected_set.dag` DECLARES — AGENT-1 is a CLIENT, defers to that single authority (P2) and coins NO return noun (`Witness<ReExecFrontier>` was an ungrounded AGENT-1 noun, P1/P2 — removed) + a faithful re-emit (C5-1 node-level hash-checked + C5-2 emit-locality + C4 committed==emit); GATE = `apply_lens(_,Enforce)` fail-closed (agent earns no special trust). It is a **CLIENT** of application.dag + affected_set + B2-OMNI + C5/C4 — NO new authority/noun/query subsystem (QRY-1 + feedback_closed_system_design). Owned by T-23 so it is a contract, not an accident; realized only when T-23 composes T-21 + B2-OMNI + C5/C4 (scaffold today — nothing runs yet) | `lens/application.dag` header (AGENT-SURFACE block) + `compiler/00_compile.dag` B2-OMNI back-ref + `TASKS.md` T-23 note |
| **I** | **Fail-closed success/failure beside `Diagnostic` (M6 / P2).** v4's **single** substrate coproduct for "produced value vs fail-closed reject carrying gunbc's `Diagnostic`" is **`Outcome<ok>`** with variants `Produced { value: ok }` and `Rejected { diagnostic: Diagnostic }` in `std/diagnostic.dag` (operator-ratified 2026-05-16). `TASKS.md` / `STRUCTURE.md` **`Result<…, Diagnostic>` prose** names the **same semantic surface** — it **denotes `Outcome<…>`**, not a second parallel `.dag` `Result<ok, err>` type. v4 **does not** declare a generic `Result<ok, err>` in `std/` alongside `Outcome`; introducing one would duplicate M6's authority unless this ledger is amended. Variant/field names are **binding** for downstream mirrors. A future *different* error payload (non-`Diagnostic`) is a new substrate decision (STOP), not an unratified shadow carrier. | `std/diagnostic.dag` (`Outcome<T>`) + this row; `TASKS.md` / `STRUCTURE.md` (prose only) |
| **D2** | **[SUPERSEDED by D2-REV — see the D2-REV row below + the "D2 REVERSAL" section]** **Primitive-inhabitance FORM — the `extdeps` resolver shape** (operator-ratified 2026-05-16; resolves the loyal-koi-702 STOP-2/STOP-3 escalated via the T-4 mgr). A language/format `extdeps` file is a **resolver** for the abstract `std/` carriers — the `infra(abstract)→gcloud/aws(concrete)` pattern (operator-cited prior art: gunb.ai `tools/extdeps/cloud_extdeps`, where the abstract `InfraScopeType` is declared ONCE and each provider supplies a translation, never a re-declaration). **D2a — three thin per-primitive declarations, NO re-declared algebra inhabitance:** (1) **alias-identity** to the `std/` carrier (`type RustI32 = Int32`) — the machine-readable-inhabitance form (`List<T> = FreeMonoid<T>` precedent); the algebra inhabitance flows through the alias, so "see through the labels" is literal `Node` identity; (2) a **grounding map** (surface spelling — `i32`); (3) **operation-semantics carriers** (`OverflowDisposition`, NaN/Inf handling) — genuine per-language facts, siblings to the alias. A per-language `data <lang>_<prim>: OrderedRing<<lang>Prim,…>` inhabitance is the parallel `OrderedRing<Word*>` substrate **INVARIANTS P1:42 forbids** (`numeric_aliases_align_to_refinements`) — superseded; #3174's `RustScalar` Disj registry + `IrToRust<>` carrier dissolve into this shape. **D2b — fn-value form:** the integer chain is fully CONSTRUCTIVE (Peano `Nat` → `GroupCompletion` → `AbelianGroup` → `Compose<MachineWidth>`) — its operation `Arrow` bodies are sub-DAGs of L1 behaviors (THESIS:203, the user-function case), so D2b's primitive question never fires for `integer.dag`. A genuinely-irreducible spec primitive (IEEE-754 float arithmetic) flows through the SAME single external-primitive path THESIS:203 already defines — `Transform → FunctionRef → Arrow`, where the `Arrow`'s `body` is *a realization declaration in `extdeps/`* (THESIS:203's primitive case). D2b coins **NO new carrier**: no `Symbol`+`Anchor` data node referenced directly by the inhabitance — that would be the second external-primitive path P2 forbids. The `ApproximateField` inhabitance holds `FunctionRef`s to the op `Arrow`s; for an IEEE-754 primitive the `Arrow`'s `body` realization (anchored to the spec §) is the per-target `extdeps/languages/*` resolver (the D2.2 resolver), not a std-homed carrier. The `Arrow` HAS a body (the `extdeps/` realization), so it is not a bodiless `fn` and PARSE-1 is untriggered; NOT a kernel/A1 extension (no new connective/behavior). **D2.2 — coercion** = the U1 homomorphism realized by the generic B2-OMNI emit fold reading the resolver data; NOT a hand table (the `IrToRust<>` per-target string table INVARIANTS P1 forbids), NOT a per-language emit lens (B2-OMNI/B4: one generic emit, target-specificity is declarative T-11 data). **D2.3** — compound/generic coercion composes for FREE via the `Node` catamorphism (A1's single recursive type; `List<Int32>` = `Instantiation` recursed); no per-instantiation rows, not blocked on the Functor hierarchy (a cost/algebra-lens concern). **D2.4** — the round-trip guarantee is structural-on-F per **C5-1**; TestClaims document it, never establish it. **D2.5** — overflow/UB/arbitrary-precision are DISTINCT `std/` carriers selected by the alias target + the per-language `OverflowDisposition` carrier, never a re-declared semantics. **Scope — D2 covers per-primitive inhabitance form ONLY:** it does NOT decide LanguageModel-grammar facts (C++ most-vexing-parse, TypeScript structural typing, etc.) — those are owned by C5-3 / B2-OMNI / B4 per their own cascade; a grammar-slice worker must not import D2 reasoning into them. | `std/integer.dag` + `std/float.dag` headers (D2 blocks) + `TASKS.md` T-4 brief amendment + `extdeps/languages/*` headers (workers encode the resolver shape); cross-ref INVARIANTS P1 / A1 / B2-OMNI / U1 / C5-1 / IR-1 |
| **D3** | **ApproximateField<Float> machine-readable witness — encoding deferred (INVARIANTS P1/P3)** (operator spine 2026-05-16 for float algebra **shape** via **D2b** + Tier-2 **D2** intrinsic `Symbol` nan identities; **amended** after codex REQUEST_CHANGES on **PR #3191**). `std/float.dag` **does not** publish `data float_approximate_field: ApproximateField<Float>` while v2-bridge constructive `fn` bodies at this site would fabricate IEEE finite `+`/`*` or binary64 `one` (left-operand hooks or wrong-bias `Succ` on the Peano exponent carrier = known-wrong machine-readable authority — INVARIANTS P1 modeling faithfulness + P3 fail-closed honesty). Re-introduce the data row when **T-4 `extdeps/languages/*`** supplies grounded THESIS:203 **Arrow** bodies **and/or** v4 admits bodiless primitive signatures (**PARSE-1**). **Does ship:** `Float` carrier + Tier-2 `Outcome` ordered compare + intrinsic `data <id>: Symbol = <id>` for nan rejection (cross-ref **D2**, **I**, **PARSE-1**). | `std/float.dag` (carrier + Tier-2 compare + `Symbol` data; **no** `data float_approximate_field` until grounded) + this row |
| **D4** | **[SUPERSEDED by D2-REV — see the D2-REV row below + the "D2 REVERSAL" section]** **GroundingMap-home — the shared `extdeps/languages/resolver.dag` registry** (operator-ratified 2026-05-16; co-drafted; completes **D2**). The D2 resolver's *shared* types live in ONE new file `extdeps/languages/resolver.dag` — NOT re-declared per language (parallel-declaration P2 forbids; `feedback_import_not_redeclare_carriers`) and NOT homed in `rust.dag` (which would make one language file a substrate authority for the other four, and balloon as future shared resolver types land). `type GroundingMap<IRCarrier> { spelling: String }` is its first resident. **Admission rule (STRICT):** `resolver.dag` hosts ONLY *structurally-language-INVARIANT* resolver types — a type whose structure provably cannot vary across the modeled language specs (a convergence carrier). Spec-varying types stay per-language (`OverflowDisposition` etc. — D2.5 / L-2). Test before admitting: can you prove the structure invariant across the specs? Future shared resolver types (e.g. the C5-fidelity `Modeled \| Declared-normalized \| Fail-closed` disposition classifier) land here without a fresh placement decision. Closed-tree extension: `STRUCTURE.md` `.dag` count +1. Per-language files each add a one-line `import … resolver { GroundingMap }` + their per-primitive `data <lang>_<prim>_grounding` instances (the grounding-row fan-out — canonical `rust.dag` first, then the 4 others). | new `extdeps/languages/resolver.dag` + `STRUCTURE.md` count/enumeration + `extdeps/languages/*` import lines + grounding instances; this row; cross-ref **D2** |
| **D5** | **Operator-driven frozen-header / frozen-contract reconcile is sanctioned in-PR** (operator-ratified 2026-05-16). "Preserve-verbatim" on a frozen scaffold header forbids *unsanctioned worker contract-drift* — it does NOT forbid *sanctioned reconciliation*. When an operator-tier action (a BLOCKING finding, an operator ruling) legitimately moves a worker's body past its frozen header or I/O contract, the header/contract is **reconciled to match in the SAME PR**, with the reconcile receipt — citing the driving operator action — recorded in the **commit message**. **Amended 2026-05-17 (operator-ratified):** the receipt is a commit-message entry, *not* an in-file `HEADER RECONCILE` block — the in-file-block form is superseded by the no-prose directive / `docs/modeling-discipline.md` Practice 9. D5's intent — a conscious, reviewer-visible reconcile receipt — is preserved: it lives in the commit message + `git blame`. **Discriminator:** an operator-tier action drove the body change ⇒ reconcile the header/contract in-PR, flagged; NO operator action drove it ⇒ the body must conform to the frozen header (preserve-verbatim holds, the worker cannot move the contract). Covers BOTH header-doc drift (stale `Owns`/`Consumes` bullets after a sanctioned refactor) AND frozen-contract changes (operator BLOCKS an I/O contract as P2/§2-insufficient). Already operating: `integer.dag` #3190 (`Consumes` reconciled), #3209 finding 2 (`WaitProcessResult`). Does NOT permit free worker header edits — operator-driven only. | this row + the reconcile receipt recorded in the **commit message** (the in-file `HEADER RECONCILE` block form — `integer.dag` #3190 precedent — is retired by Practice 9, 2026-05-17); refines the frozen-scaffold-header / preserve-verbatim discipline |
| **D2-REV** | **D2 / D4 REVERSED — fact-bundle modeling supersedes alias-identity** (operator-ratified 2026-05-17, **out-of-band** — the verbatim operator-quote provenance is in the "D2 REVERSAL" section below). An `extdeps` primitive is modeled as a fact-bundle grounded from its own spec, never a bare `std/` alias; deduplicate to a `std/` carrier only on *proven* coincidence; the structural fact-density / hollow-alias gate is `T-30`, a hard prerequisite of T-4. This is a one-line index pointer — the full record (root cause, phased plan, impact map, "coincide" definition) is the **"D2 REVERSAL + FACT-BUNDLE RESEED"** section below, which is the authority. Supersedes the **D2** and **D4** rows above. | the **"D2 REVERSAL + FACT-BUNDLE RESEED"** section of this file (authority); `#3224` executes Phase 1; `docs/modeling/grounding-worked-examples.md` companion |

---

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
- **Encodes into:** `compiler/04_infer.dag` header + `TASKS.md` T-9.

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
  termination only. Canonical form is a SEPARATE explicit node.dag
  authority — the **canonical-form clause**, now **B1-CANON**,
  operator-DESIGNED & ratified in the `std/node.dag` header 2026-05-15
  (pulled back from worker delegation — every consumer rests on it).
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
  **generic Node-tree builders driven by that data**, not per-language
  procedural code. Productions are Node trees (consistent with A1).
- **Tradeoff:** a declarative-grammar substrate is more work to design
  than a hand recognizer, but a hand recognizer is the gameable path and
  re-fragments per language.
- **Encodes into:** `extdeps/languages/*` headers + `01_tokenize.dag` +
  `02_parse.dag` + `TASKS.md` T-6/T-7.

### B3 — The type signature is the single effect authority

- **Tension:** effects appear in three places (the effect lens T-13,
  coordination effect-types T-4.8, `unenumerated_effects.dag`).
- **Recommended:** the **type signature is the one effect authority**.
  The effect lens *reads* effects from the signature; coordination's
  `HttpEffect`/`QueueEffect` are typed carriers *over* that, not a
  parallel effect taxonomy/enum.
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
  effect authority; a closed `HttpEffect | QueueEffect | …` enum would be
  exactly the parallel effect taxonomy B3 forbids (P2). So there is NO
  effect-type enum to "close": `HttpEffect`/`QueueEffect` are typed
  CARRIERS read from the signature by `lens/effect.dag`, not an
  enumerated axis. Non-default; resolved by B3.
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
  `std/float.dag` D2 header prose (NOT the core).
- **Active PRs:** `#3219` closed; `#3208` held.

### Worked examples

`docs/modeling/grounding-worked-examples.md` (this PR) demonstrates the
fact-bundle/grounding model on one complex type per target — spanning
`machine_code` (pure bits) → `rust` → `verilog` (4-valued logic) →
`spice` (continuous) → `lean` (dependent types) → `english` (natural
language, mostly fail-closed). Each: the model (carrier + meaning) and
step-by-step coercion. Plan-only; the remaining mainstream
language/format targets fan out against the same template.

### Status

**RATIFIED 2026-05-17** (operator, out-of-band — the operator declared D2
wrong [the verbatim operator quote is in *What reversed* above, this same
section], ratified the fact-bundle principle, and dispatched the Phase-1
execution). Phase 1 is executing now as **PR #3224** (TASKS.md +
DECISIONS.md re-sequencing). This PR (#3223) encodes the ratified
decision into the ledger; #3224 is stacked on it. TASKS.md consuming the
D2-reversal as ratified is therefore correct, not premature.

---

## T-4.7 — `extdeps/frameworks/react.dag` Practice 4 / INVARIANTS P1 receipts

**Binding:** `docs/modeling-discipline.md` Practice 9 — coproduct **classification** (🟢 / 🟡 / N/A per carrier), five-pattern **ledger**, and named **triggers** live in this file, not as multi-line prose in `react.dag`. The substrate carries **one** `// Anchor:` + **one** `// Practice-4: DECISIONS.md#…` cite per coproduct. **TASKS authority:** `src/v4/TASKS.md` T-4.7 (incl. `ReactNode` partial-slice posture).

### react-p4-optref

**Carrier:** `ReactOptRef` in `src/v4/extdeps/frameworks/react.dag`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟢 GREEN (terminal).** Abbreviated five-pattern ledger:

1. Fact placement — FAILS: shared optional-ref discriminant across hooks/elements.
2. Variant-is-data — FAILS: nullary absent vs record-present differ structurally.
3. Algebraic form — N/A: not a std/algebra decomposition target at this boundary.
4. Dimensional — FAILS: mutually exclusive ref-present vs ref-absent at call site.
5. Parameterized family — FAILS: not one generic F<X> over a separate hook-id set.

**Terminal:** irreducible optional ref handle arity pinned to react.dev call signatures.

### react-p4-effect-cleanup-site

**Carrier:** `ReactEffectCleanupSite`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟢 GREEN (terminal).** Abbreviated five-pattern ledger:

1. Fact placement — FAILS: shared cleanup-shape classifier for all effect hooks.
2. Variant-is-data — FAILS: NoCleanup vs CleanupBody carry different structure.
3. Algebraic form — N/A: cleanup arity is API-surface, not an algebra carrier here.
4. Dimensional — FAILS: exclusive “no cleanup return” vs “cleanup function ref”.
5. Parameterized family — FAILS: not mechanical F<X> over an external enumerated set.

**Terminal:** irreducible react.dev effect cleanup return shape (omit vs present function).

### react-p4-hook-inline-deps

**Carrier:** `ReactHookInlineDependenciesArgument`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟢 GREEN (terminal).** Abbreviated five-pattern ledger:

1. Fact placement — FAILS: deps arity is a shared inline classifier for several hooks.
2. Variant-is-data — FAILS: omitted vs present-list are different input shapes.
3. Algebraic form — N/A: dependency array is not a std/algebra re-export.
4. Dimensional — FAILS: exclusive omit-deps vs explicit dependency list at call site.
5. Parameterized family — FAILS: not one F<DepsSite> copy of a separate closed set.

**Terminal:** irreducible optional dependency-array arity (react.dev useEffect family, …).

### react-p4-module-surface

**Carrier:** `ReactModuleSurface`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟢 GREEN (terminal).** Abbreviated five-pattern ledger:

1. Fact placement — FAILS: module surface is shared metadata on function components.
2. Variant-is-data — FAILS: client vs RSC are distinct authoring/runtime surfaces.
3. Algebraic form — N/A: module kind is boundary classification, not std/algebra.
4. Dimensional — FAILS: exclusive client module vs RSC module per pinned react.dev split.
5. Parameterized family — FAILS: two named surfaces, not F<ModuleKind> over a larger set.

**Terminal:** irreducible react.dev / RSC module-surface partition for components.

### react-p4-hook-site

**Carrier:** `ReactHookSite` in `src/v4/extdeps/frameworks/react.dag`.

**Classification (home of record — design-r4 §12 + Phase‑1.5):** **🟡 YELLOW (scaffold / deferred-on-custom-promotion).** Per `docs/design-r4-phase-1-5-hookkind-custom-react-substrate-canvas.md` §2.1, **`ReactHookSite` including `CustomHook` stays 🟡** until operator-ratified promotion — the Phase‑1.5 canvas names **how** `CustomHook` may gain fields without parallel lifecycle sums and **explicitly does not** “close” the carrier to 🟢. **Dissolution** still flows through semver / hooks-index substrate edits and/or extending `CustomHook` in the **same PR** as consumer proof (see that canvas §3).

**Practice 4 / INVARIANTS P1 — five-pattern ledger (abbreviated):** why the react.dev **Hooks-index built-in arms** are not cheaply **🔴 RED**-dissolvable here; **not** a second authority that overrides §12 **🟡** on the **whole** sum:

1. Fact placement — FAILS: roster is the shared substrate discriminant for hook consumers.
2. Variant-is-data — FAILS: each **built-in** arm carries a distinct per-API record (not one tag×payload).
3. Algebraic form — N/A: built-in hooks are not copies of std/algebra intro forms here.
4. Dimensional — FAILS: exactly one hook API (or `CustomHook`) per modeled call site.
5. Parameterized family — FAILS: heterogeneous payloads — not one F<HookId> projection.

**Pinned slice:** **18** react.dev **Hooks**-index `use*` arms + boundary **`CustomHook { implementation_ref }`** (= **19** `ReactHookSite` arms, T‑4.7). The **`use(resource)`** API is **`ReactUseCallSite`**, not folded into `ReactHookSite` (react.dev: **use** is not a Hook).

**Why the whole `Disj` remains 🟡:** `CustomHook` is **opaque** at this layer by default (canvas §2); §12 binds the **entire** coproduct until promotion closes the custom boundary.

### react-p4-use-call-site

**Carrier:** `ReactUseCallSite` (single-arm sum).

**Practice 4 §4:** N/A — single-arm coproduct (`UseResource` only); the §4 checkpoint applies to coproducts with **N ≥ 2** variants (`docs/modeling-discipline.md` §4 / INVARIANTS P1).

### react-p4-element

**Carrier:** `ReactElement`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟢 GREEN (terminal).** Abbreviated five-pattern ledger:

1. Fact placement — FAILS: element-kind is shared across render-tree consumers.
2. Variant-is-data — FAILS: Host vs Composite vs Fragment carry different record shapes.
3. Algebraic form — N/A: not decomposable into std/algebra carriers at this boundary.
4. Dimensional — FAILS: exactly one createElement/fragment partition per value.
5. Parameterized family — FAILS: three structurally distinct carriers, not F<ElementKind>.

**Terminal:** irreducible react.dev element partition (host / composite / fragment children).

### react-p4-node

**Carrier:** `ReactNode`.

Coproduct dissolution (Practice 4 / INVARIANTS P1 / modeling-discipline.md §4) — **🟡 YELLOW (scaffold).** T-4.7 **partial** `createElement` children slice (`Element` | `Text` only). Full JSX / react.dev child lattice is **owned elsewhere** — see `src/v4/TASKS.md` T-4.7 “ReactNode (Practice‑4 🟡)” + `docs/design-r4-phase-1-5-hookkind-custom-react-substrate-canvas.md` (not exhaustive for “all React children”).

**Trigger + obligation:** **Dissolution = operator-ratified PR** extending or retiring this sum; consumers must not treat the Disj as terminal closure of React child modeling.

**Five-pattern note:** Patterns 1–5 do not justify closing the full child lattice inside this substrate file; deferral is explicit **task authority** (TASKS T-4.7), not consumer-independent GREEN.

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

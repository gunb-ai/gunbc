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
| **QRY-1** | **RETRACTED & reframed (operator-corrected 2026-05-15).** "query = user-defined lens" is wrong: user-vs-kernel is PROVENANCE, not a structural distinction (feedback_naming_is_aliasing) — a decorative name distinguished only by who-authored-it must not exist ("a real distinction, or not exist"). There is NO query concept/noun/subsystem. The REAL axis is **scope/parameterization**, already modeled with no new noun: (1) `SectionRef` (DeclarationScope\|NodeScope) — applying a lens AT a scope; (2) parameterized lens signatures — the **affected-set lens (T-21) `(Dag,Diff)->Witness<ReExecFrontier>` is the archetype**: "the affected set of THIS change" = `affected_set(dag,diff)`, already first-class. Lenses, read at scopes and/or parameterized. No `query.dag`. The B2-OMNI "one read over the Node pivot, all ingested languages" property holds for ANY lens, not a "query" | `lens/application.dag` header + THESIS §1.5 + `lens/affected_set.dag` (T-21) |
| **PAR-1** | Parallelism (T-13) is a THESIS free consequence, a target-AGNOSTIC lens reporting the independence **FACT** (`Witness<ParallelismMap>`); WHETHER/HOW to realize it is target-DEPENDENT — a cost-derived decision (U1/U2 per-target) at emit(T-11)/eval(T-22), NOT the lens. independence = structural-disjointness ∧ effect-independence (consumes `lens/effect.dag`, B3 — Node-disjoint is necessary not sufficient) ∧ fold-combiner associativity-by-inhabitance (consumes `std/algebra.dag` — parallel-reduce is an algebra fact, not a heuristic). fact ≠ spawn-decision; no annotation, no scheduling heuristic. **User-facing contract:** NOT a "parallelize my code" command (forbidden — engine-shaped from the user's seat); a lens the user READS (same UX as complexity/cost); control = structure not a knob; default introspective with the realize-decision derived+inspectable per target (e.g. honestly "no — CPython GIL"); guarantee via `apply_lens(parallelism, Enforce)` (T-23, uniform opt-in-depth). 4 worked examples embedded | `lens/parallelism.dag` header + `TASKS.md` T-13 |
| **B1** | One content-addressing scheme: a `std`-level Merkle catamorphism `content_hash : Node -> Hash` over the **canonical-form clause** (a SEPARATE explicit node.dag authority — NOT subsumed by A1, which only covers recursion/axis-closure/generics/termination). `Hash` is an opaque digest (no content accessor, K-1-style). One scheme consumed by T-15/T-20/T-21 + A3 + C5 + structural build/CI artifact caching (a cache key IS `content_hash` of the input subgraph — see `lens/affected_set.dag`). NOT workflow-level | `std/node.dag` — the `Hash` type lands here (declared by the primitive.dag-decomposition PR, beside `Symbol`; `Hash` is a content identity, not a scalar); the canonical-form clause + `content_hash` fold also `std/node.dag` (T-1 worker encodes the clause + fold per relay) |
| **PARSE-1** | Bodiless `fn` (signature-only / contract-as-checked-signature) is **not needed in the v2-bridge era** (operator-ratified 2026-05-15) — verified by constructive pre-flight: frozen v2's parser **rejects** bodiless `fn` (`expected LBrace`) but cleanly parses the full ratified kernel shape **with bodies** (40k-line `src/v2` corpus + kernel probe, zero syntax errors; the 65 src/v2 errors are 100% post-parse import/cycle). So bridge-era substrate contracts stay **comment-prose** (current scaffold form) and the fail-closed parse-class gate is SAFE — v2 is never asked to parse bodiless fn. **Forward requirement (the conditional that makes the deferral acceptable):** v4's OWN `.dag` grammar (T-7) MUST admit a bodiless signature declaration as a first-class production — graduating the "immutable I/O contract for the worker" from comment-prose to **compiler-enforced** (filled body must conform to the frozen signature; divergence = fail-closed type error, structural per no-annotations). Supportable by construction (v4's own grammar; nothing external constrains it); per B2-OMNI it is a grammar-DATA production, not hardcoded in the walker. v4-self-parser-era only — never through the v2 bridge | `compiler/02_parse.dag` header + T-20 pre-flight note |
| **D1** | The structural-delta vocabulary over `Node` (side-session 2026-05-15). `Diff` = a set of `Edit`s; an `Edit` replaces the subtree at a `Path` with a `Node` (term-rewriting `t[s]p`); `Path = List<EdgeSelector>` (`ByName`/`ByPosition`, mirroring `EdgeLabel`) — flat & finite, NOT a new recursive type (A1: `Node` stays the sole recursion). Name is **`Path`** (the route), not `Position` (operator pick — a position is a coordinate; a path is how you get there); "position/occurrence" (JSON Pointer, RFC 6901) is retained only as the anchor citation. `ByPosition`'s index is a navigation query against live `children` order — NOT the stored-edge-index anti-pattern (P2, distinct concern). Resolving a Path needs only `Symbol` equality ⇒ K-1-clean, not K-1-exempt. **DATA TYPES live in `std/node.dag`** (B1/`Hash` precedent: a structural delta over `Node` is substrate-tier, not a lens's private type; corrects the initial "owner = T-21" read). **OPERATIONS** (`subterm_at`/`apply_diff` → `Result<Node, Diagnostic>`; `paths_independent`/`diff_well_formed` → `Bool`) live in **`lens/application.dag`**, FORCED: `std/diagnostic.dag` already `Consumes std/node.dag` (`Correction = Suggested(Node)`), so a node.dag op returning `Result<_,Diagnostic>` is a substrate cycle — the node.dag "consumers fail-close" discipline (the `node_well_formed` precedent). Well-formed `Diff` = pairwise-independent Paths ("parallel positions") ⇒ `apply_diff` is deterministic-total or fail-closed; the INVARIANT is named in `std/node.dag`, the deciding predicate in `application.dag`. `apply_diff` consumer = CLI / test harness / user app code (not a meta-work dispatcher). **PROPOSED sub-point for operator eye:** the data/op split is a refinement of the ratified "homed in node.dag" — confirm or redirect in PR review | `std/node.dag` header + EdgeSelector/Path/Edit/Diff decls + `lens/application.dag` header (operation contract) + `lens/affected_set.dag` cross-ref |

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
> complete. **THREE** node.dag-contract items the T-1 worker encodes per
> relay (it owns that file): **K-1** (opaque Symbol), the **canonical-form
> clause** (total deterministic normal form — a SEPARATE authority, NOT
> subsumed by A1; B1 depends on it), and **`content_hash`** (the Merkle
> fold over that form). Nothing in Part 3 remains PROPOSED.

### B1 — One content-addressing scheme (ELEVATED: load-bearing for A3)

- **Tension:** T-15 fixed-point hash, T-20 pinned bootstrap binary, T-21
  unchanged-subgraph detection, **and the A3 reproduction/surfacing
  check** all need content-addressing. Three schemes = drift.
- **Recommended:** a single canonical content-hash over the canonical
  Node structure. **Correction (3248138046):** A1 does NOT make Node
  canonical — A1 ratifies recursion / axis-closure / generics /
  termination only. Canonical form is a SEPARATE explicit node.dag
  authority — the **canonical-form clause** — which the T-1 worker
  encodes alongside K-1 and `content_hash`. B1 *depends on* that clause;
  it is not subsumed by A1. The clause must define a total deterministic
  normal form (structural child order; opaque-Symbol identity per K-1; no
  incidental ordering) so the fold is well-defined. Defined **once**,
  consumed by T-15/T-20/T-21 + the A3 check.
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

## Part 5 — Ratification flow

For each PROPOSED item: ratify (👍 / "confirm") or redirect ("instead,
…"). On ratification I encode it into the cited file header(s) — the
same way A1/A2/A3/U1/C2 were encoded — and promote it to Part 1 here.
Quick confirms (Part 2) are cascade-closures of already-ratified
parents; redirecting one means reopening its parent. Part 3 are genuine
forks; B1's "where it lives" is the one pick I most need from you.

Nothing here is encoded into substrate until you ratify it. This file
PROPOSES; it does not decide.

# v4 — XL Task Plan

The XL tasks below define "v4 done" (the count is intentionally NOT stated — it drifts as scope is ratified; the close gate is "every task in this plan," never a hardcoded number — see T-15). Each task is a bounded modeling unit; each produces a typed pure function in declared files; each is honestly hard to game because the work IS the decisions.

**Sizing discipline** (per operator directive 2026-05-15): all tasks are XL by default. Relative sizing (S / M / L / XL within the XL bracket) is used only when conveying scope-risk explicitly. **No timelines, no day estimates** — discuss only technical decisions.

## Execution graph

Phases were a coarse proxy — they bucketed tasks by rough wave but buried
the one load-bearing fact: the **critical path**. This graph is a
**critical-path + parallel-fill** model. Two properties drive scheduling,
and they are INDEPENDENT:
- **schedulable** — every dependency is met; the task *can* start.
- **prioritized** — the task is on the critical path; it *sets
  time-to-done*.

A low-dependency leaf is schedulable early but is never *prioritized*
over a critical-path task. Resource the critical path ruthlessly;
schedule everything else the instant its dependencies clear, on any free
worker.

### Critical path — sets time-to-done

```
T-1 → T-2 → T-3 → T-6 → T-7 → T-8 → T-9 → T-10 → T-11 → T-16 → T-15
```

`T-1 → T-2 → T-3` is serial and unavoidable — the substrate foundation.
After T-3 the spine is the serial compiler pipeline
`T-6 → T-7 → T-8 → T-9 → T-10`, then emit specialization + the omni demo,
closing at T-15.

```
  T-1   std/node.dag                     [BLOCKS: all]
  T-2   std/algebra.dag                  [needs T-1]
  T-3   std/* supporting (11 files)      [needs T-1, T-2; OWNS the full shared-fact vocabulary — signedness, representation, the numeric stack — see T-3 detail]
  T-6   compiler/01_tokenize.dag         [needs T-3]
  T-7   compiler/02_parse.dag            [needs T-6]
  T-8   compiler/03_normalize.dag + 03_resolve.dag   [needs T-7; T-28 module-graph substrate is bundled here]
  T-9   compiler/04_infer.dag            [needs T-8, T-2, T-3, and T-4 — T-4 enters from the side branch below, not the spine]
  T-10  compiler/05_emit.dag + 00_compile.dag       [needs T-9, T-4]
  T-11  emit per-target specialization (extends T-10 across all 5 Shape A targets)   [needs T-10]
  T-16  Full-stack omni-emission demo: ONE .dag → Rust+C++ backend
        + SQL DDL schema (Shape-B, via T-4.6 sql.dag — Theme-A #4)
        + React/TS frontend + OpenAPI wire contract
        [needs T-4, T-4.5, T-4.6, T-4.7, T-4.8, T-10, T-11]
        (T-4.8 coordination.dag is load-bearing — T-16 uses it for
        endpoint partitioning; facts must flow forward from the
        coordination substrate into the flagship demo)
  T-15  bin/main.dag + bootstrap glue + self-host fixed-point validation
        + PROOF-1: the external trust-discharge for A3 — a lens =
          (evidence read: A2 descent, the coercion-fold chain, cost,
          effects) ⊕ (B2-OMNI emit to a lean/coq language model). The
          prover KERNEL-CHECKS gunbc's exported witnesses (never
          searches — no-engine + A2). Framing in STRUCTURE.md §7 +
          DECISIONS.md PROOF-1 (no new file there). Prover model now
          landed: `extdeps/languages/lean.dag` (B-2 / DECISIONS L-4,
          operator-ratified) — Lean first, scoped to the termination
          theorem class; Coq is the deferred second-prover probe.
          PROOF-1 is realized when the lens framework composes with
          that model (composition, not a new subsystem).
```

### Side branch — `{P1-KEYSTONE, T-30, T-29, T-25-core} → T-4 → T-9` (watch item)

```
{P1-KEYSTONE, T-30, T-29, T-25-core} → T-4 → T-9
```

T-9 needs T-4 (the language fact-bundles) in addition to T-8. This branch
carries slack against the `T-6→T-7→T-8` pipeline branch **only if its
feeders start immediately**. The D2 reversal CHANGED T-4's dependency set
— the old alias model needed almost nothing; fact-bundle modeling needs
the shared vocabulary (`T-3`, itself on the critical path) plus
**four feeders that are not on the critical path and gate `T-4 → T-9`**:
P1-KEYSTONE, T-30, T-29, T-25-core. Those four are **watch items**, not
slack-having parallel fill — if any slips, the side branch goes critical.
T-4 is no longer a schedule-anytime leaf — see T-4.

```
  P1-KEYSTONE   the Phase-1 doc keystone — NOT a T-## task. The
                `INVARIANTS.md` P1:42 amend, the `MODELING.md` M1
                promotion, and the `docs/modeling-discipline.md`
                fact-bundle Practice + worked good/bad examples
                (DECISIONS.md "D2 REVERSAL + FACT-BUNDLE RESEED",
                Phase 1). It is the rubric every fact-bundle task is
                authored and reviewed against; T-4 cannot start before
                it lands.
  T-30  std/ structural fact-density / hollow-alias gate — a generated
                checker that fails closed on a hollow alias (a carrier
                that reads zero spec facts). A hard prerequisite of T-4:
                the per-language fact-bundle rework does not begin under
                convention-tier-only enforcement — convention is what let
                D2 through. Sibling of P1-KEYSTONE; see T-30 detail.
  T-29  extdeps C++ ABI / target data-model — C++ integer widths are
                implementation-defined; the cpp fact-bundle cannot ground
                them without it. Low-dependency (needs only T-3
                machine/width) but a hard T-4 prerequisite — see T-29.
  T-25-core  std/ value-predicate refinement substrate (the core half) —
                base type + fail-closed validation obligation; T-4's
                refinement-bearing carriers need it. Sits near T-3 — see
                T-25. (T-25-tail, the predicate prover, is post-T-9
                parallel fill, not a feeder.)
  T-4   extdeps/languages/{rust,python,go,cpp,typescript}.dag
        [needs T-3, P1-KEYSTONE, T-29, T-30, T-25-core — see T-4]
```

### Parallel fill — schedule the instant deps clear

These have **slack** against the critical path: schedule each as soon as
its dependencies clear, on any free worker; as long as it finishes before
its consumer needs it, it adds nothing to time-to-done. (The per-task
`[needs]` contracts are the **authoritative** dependency record; this
critical-path / side-branch / parallel-fill grouping is the *derived*
scheduling view of those same edges — one authority, two views, never a
second set of facts.)

```
Substrate / extdeps fan-out:
  T-4.5 extdeps/{process,file_system}.dag                      [needs T-3, T-25-core]
  T-4.6 extdeps/formats/* (7 files: json/yaml/csv/toml/json_schema/openapi/sql)  [needs T-25-core, T-26]
  T-4.7 extdeps/frameworks/react.dag    [needs T-4 (typescript)]
  T-4.8 extdeps/coordination.dag         [needs T-4, T-4.7]
  T-4.9  extdeps/languages/verilog.dag   [needs T-1, T-2; B2-OMNI falsification probe — concurrency vs the 5 behaviors]
  T-4.10 extdeps/formats/spice.dag       [needs T-1; B2-OMNI falsification probe — LanguageModel generality (no control flow)]
  T-4.11 test/claim/boundary/english_ingest_fail_closed.dag  [needs T-1, T-3 std/verification.dag; boundary-honesty probe — TestClaim/AssertKind bind after verification.dag fill, not parallel to Wave-A2 scaffold]
  T-4.12 extdeps/languages/llvm_ir.dag   [needs T-1, T-2; B2-OMNI probe — generalize DOWN the stack (SSA IR)]
  T-4.13 extdeps/languages/machine_code.dag  [needs T-3 machine + T-4 LanguageModel shape; B2-OMNI probe — bottom of stack; disassembly = extreme fail-closed]
  T-4.14 extdeps/languages/ptx.dag       [needs T-1, T-2; B2-OMNI + IN-B probe — SIMT data-parallel vs the 5 behaviors]
  T-5   REMOVED 2026-05-15 (operator-ratified) — work-direction meta-layer
        cut; only workflow/bootstrap.dag (T-20) + workflow/ci.dag (T-24) remain

Test + bootstrap substrate (schedule early — every later task benefits):
  T-19  lens/testgen.dag                 [needs T-1, T-2, T-3]
        Produces TestClaim corpus from substrate; manual TestClaims in
        test/claim/manual/ serve as anti-regression contract until
        T-19 implementation lands. Every later task benefits from
        testgen-derived test coverage instead of hand-authoring.
  T-20  workflow/bootstrap.dag           [needs T-1; grows incrementally]
        Bootstrap orchestration AS DATA (seed-once → self-host →
        fixed-point). v2 interprets it. Scaffold-early (the parse-
        viability step is the existing CI gate); full self-host
        content lands as the pipeline matures. T-15 consumes it for
        fixed-point validation. NOT a build.rs/shell (that = the v3
        regression door).
  T-21  lens/affected_set.dag            [needs T-1, T-2, T-3]
        Incremental re-exec frontier (operator: "wanted very early").
        Structural authority that replaces scripts/detect-affected-
        components.sh. Consumed by T-24 (ci) + eval (skip pure
        unchanged subgraphs).
  T-24  workflow/ci.dag                  [needs T-21, T-20]
        CI pipeline AS DATA; .github/workflows/ci.yml derived. Closes
        v3's gate-#98 gap (hand-authored CI YAML). Consumes T-21 for
        job selection — the shell bridge dissolves once both land.

Interpreter + lens dimensions (each needs T-9):
  T-22  compiler/05_eval.dag             [needs T-9]
        The interpreter — THE PRIMARY execution path (THESIS:225).
        Sibling of emit (same InferredTree input). workflow/bootstrap.dag
        + TestClaim eval + lens dry-run all compose over it.
  T-12  lens/complexity.dag + lens/cost.dag      [needs T-9]
  T-13  lens/{parallelism,effect,ownership,idempotency}.dag   [needs T-9]
  T-17  lens/synthesis.dag + std/report.dag  (cross-algorithm complexity, C7;
         XL scope, research-tier risk)              [needs T-12 for current-complexity input]
  T-18  lens/coverage.dag  (meta-lens: L6/L7/impossible-bug/testgen coverage
         discipline; STRUCTURAL not exhaustive-fixture per TESTING.md)
                                                    [needs T-3, T-4, T-12, T-13]
  T-23  lens/application.dag  (apply_lens surface — opt-in depth + the ONLY
         advisory→fail-closed bridge; load-bearing for §1.5 user-defined
         dimensions + §6.2 audience duality + C7 Report→Diagnostic)
         + AGENT-1: also owns the non-text AGENT-SURFACE contract — agent
           reads lenses + submits the D1 apply_diff:(Node,Diff)->
           Result<Node,Diagnostic> (enters B2-OMNI at `core`, a sibling
           of ingest, NOT an ingest; fail-closed on external/stale Diffs
           per INVARIANTS P3 — NOT a total `->Node`), gets the
           affected_set lens read in whatever shape T-21 declares
           (AGENT-1 coins no return noun — no Witness<ReExecFrontier>)
           + faithful re-emit (C5-1/C5-2/C4), fail-closed via
           apply_lens(Enforce). A client/composition of T-21 + B2-OMNI
           + C5/C4 — no new file, no new authority (DECISIONS.md
           AGENT-1; lens/application.dag header).
                                                    [needs T-1, lens framework]

Close-the-loop + late substrate:
  T-14  test/claim/* + test/fixture/* (port load-bearing TestClaims from v3)
  T-25-tail  the T-25 predicate prover — erases proven refinements (a pure
        optimization); after T-9. (T-25-core is NOT here — it is a
        side-branch feeder of T-4; see the side branch above.)
  T-26  std/ boundary carriers (HttpMethod / URL / NetworkAddress port)
        [needs T-3] — feeds T-4.6 (openapi's HttpMethod/Url) and the T-16
        wire contract; genuine slack (T-4.6 itself has slack).
  (T-28 module-graph substrate is bundled into T-8 — it is critical-path
   work inside T-8, not a standalone parallel-fill item; see T-28. T-29
   and T-25-core are side-branch feeders of T-4, listed in the side
   branch above, not here. T-27 versioning/edition lattice — DROPPED,
   ruled orthogonal to v4;
   see T-27 tombstone.)
```

## Task definitions

### T-1: std/node.dag — substrate root

**File**: `src/v4/std/node.dag`
**Why first**: every other file consumes this. Get this right; the rest follows.

**Modeling decisions**:
- Exact shape of the 6 connectives (do they share a common base, or are they truly disjoint?)
- Encoding of the 5 L1 behaviors (sum-type vs separate types?)
- C1 stop-signal mechanism (how does the substrate refuse a 7th connective by construction?)

**Reference**:
- v2: `src/v2/00_core.dag` — prior approximation
- v3: `dsl/std/` directory — substrate refinement attempts (not all honest)
- `THESIS.md` "Substrate shape" section + `docs/thesis/the-substrate-two-coordinated-shapes.md`

---

### T-2: std/algebra.dag — algebraic primitives

**File**: `src/v4/std/algebra.dag`
**Why critical**: the epistemic chain roots here. Without this, codegen has no walk path.

**Modeling decisions**:
- Inhabitance declaration shape (relation? predicate? typeclass-style?)
- Composition: how do Sum/Product algebras compose for the cost lens?
- Free constructions: FreeMonoid<T> as primitive vs derived?

**Reference**:
- v3: `dsl/std/algebra.dag` (study; expected substantive)
- `THESIS.md` "Epistemic stacking" section

---

### T-3: std/* supporting (cardinality, witness, diagnostic, collection, verification + the scalar/numeric stack)

**File**: 11 files in `src/v4/std/` — `cardinality`, `witness`, `diagnostic`,
`collection`, `verification`, plus the **scalar/numeric stack** (`logic`,
`nat`, `machine`, `integer`, `float`, `text`) that replaced the deleted
`primitive.dag` — see `STRUCTURE.md` §"Scalar/numeric concept decomposition".
**Why bundled**: smaller individually, all interrelated, foundation for everything.

**Shared-fact vocabulary — T-3 owns it (D2-reversal scope, operator-ratified
2026-05-17).** T-3 explicitly owns the full **shared-fact vocabulary** every
fact-bundle grounds into — not only `MachineWidth` and the numeric stack but
the axes the D2-reversal consumer audit found do not exist yet:
**`Signedness`**, **`Representation`**, and the remaining per-fact carriers a
language model coincides against. A per-language fact-bundle (T-4) cannot be
authored until this vocabulary exists — so T-3 is on the **critical path** and
every T-4 slice blocks on it. The exact-real / physical-quantity carriers (the
SPICE gap — see `DECISIONS.md` "D2 REVERSAL + FACT-BUNDLE RESEED", Phase 2) are
part of this vocabulary. Each axis is a real modeled fact, placed in the
appropriate scalar/numeric file (`machine`, `integer`, `float`) by DFS to its
concept-DAG home (M9) — never minted per-language.

**Dependency order within T-3** (the scalar/numeric stack is a cluster, not
flat — dispatch in waves):
- `diagnostic`, `cardinality` need only `node.dag`.
- `logic`, `nat`, `collection`, `witness`, `verification` need `algebra.dag` (T-2) or `diagnostic`.
- `machine` needs `logic` + `nat`; `text` needs `nat` + `algebra.dag` (T-2, FreeMonoid); `integer` needs `nat` + `machine` + `algebra.dag` (T-2, OrderedRing/AbelianGroup); `float` needs `machine` + `algebra.dag` (T-2, ApproximateField). Every scalar file except `machine` consumes `algebra.dag` — none of the scalar/numeric cluster is dispatchable before T-2.
- **`collection.Map<K, V>` → `witness.dag` (Wave-A2)** — Map is split out of the Wave-A1 collection.dag PR per operator-ratified Option A 2026-05-16; the honest `PartialFunction<K, V>` shape is `Map<K, V> { lookup: fn(K) -> Witness<V> }` (duplicate keys structurally unrepresentable). Lands in a follow-up `collection.dag` PR after `witness.dag` merges; tracked in `src/v4/std/collection.dag`'s "Deferred to Wave-A2 — TRACKED SCAFFOLD" header note (named dissolution trigger: `witness.dag` lands).

**Modeling decisions per file** (see file headers for specifics).

**Reference**:
- v3 mirrors of each (study for design, audit for honesty)
- TestClaim schema: import directly from `dsl/std/verification.dag:38`

---

### T-4: extdeps/languages/{rust,python,go,cpp,typescript}.dag

**File**: 5 files in `src/v4/extdeps/languages/` (operator-ratified 2026-05-15: cpp + typescript added; cpp subsumes C subset; Go retained)
**Why bundled**: identical structural shape per language; the SHAPE is the work. Each file declares the language MODEL (grammar + types + semantics) — direction-agnostic; emit AND ingest are operations against the same model.

**Dependencies — re-gated by the D2 reversal (operator-ratified 2026-05-17).** T-4 is no longer a schedule-anytime Phase-1 leaf: `[needs T-3, P1-KEYSTONE, T-29, T-30, T-25-core]`. The old alias model needed almost nothing — a bare alias reads no facts. Fact-bundle modeling needs T-3's shared-fact vocabulary (signedness/representation/numeric stack), the `P1-KEYSTONE` modeling-discipline rubric (the doc against which every bundle is authored and reviewed), the `T-30` structural fact-density / hollow-alias gate (the per-language rework does not run under convention-tier-only enforcement — see T-30), `T-25-core` (the refinement substrate — a language fact-bundle that grounds a refinement-bearing carrier needs the base-type + fail-closed-validation shape), and — for the cpp slice — the T-29 C++ ABI / target data-model. T-4 sits on the `{P1-KEYSTONE, T-30, T-29, T-25-core} → T-4 → T-9` side branch — its four feeders are watch items, not slack-having parallel fill; see the execution graph. The D2 reversal *changing this dependency set* is the single most consequential planning edit of the reseed.

**Authoring contract (operator-ratified 2026-05-15; D2 bullet superseded 2026-05-17):**
- **Model the SPECIFICATION, not libraries (L-2).** Model the versioned upstream spec (Rust Reference, ECMAScript/TS Handbook, IEEE 1364, …) — the anchor IS that spec. Do NOT model std/crates/packages: a library is just a program in the modeled language = `Node`. Modeling libraries is infinite, non-general, the wrong layer.
- **Declare every surface feature's disposition (C5-fidelity).** For each feature: `Modeled` (∈ F, Node-bearing, round-trips both ways — e.g. Python indentation IS block structure) | `Declared-normalized` (deliberately not in F; `emit∘ingest` canonicalizes — Go/C++ insignificant whitespace; a *declared*, reviewable loss, never silent) | `Fail-closed` (encountered but neither → Diagnostic, no-engine). F = the spec's own meaning-vs-lexical distinction, not worker judgment. Round-trip fidelity = declared model completeness.
- **A language file FACT-BUNDLES each primitive (fact-bundle reseed — operator-ratified 2026-05-17, supersedes D2).** For each primitive the file authors a **fact-bundle**: the facts read from that language's *own spec* — width, signedness, representation, overflow / NaN-Inf disposition, surface spelling — each a real modeled carrier grounding into the shared `std/` vocabulary (T-3). It does NOT bare-alias to the `std/` carrier: `type RustI32 = Int32` models *nothing about Rust* — it asserts an unproven identity while reading zero facts. A bundle deduplicates against a `std/` carrier ONLY where the identity is **proven** — a compiler-verified coincidence of the language bundle with the `std/` bundle, cited as evidence. `extdeps/` models systems we do not control: default to separate, honest modeling; reuse `std/` only on evidenced identity. A per-language `OrderedRing<<lang>Prim, …>` re-declaration is still the parallel-*algebra* substrate INVARIANTS P1:42 forbids — model the facts, never a duplicate algebra and never a hollow alias. See `DECISIONS.md` "D2 REVERSAL + FACT-BUNDLE RESEED" and `docs/modeling-discipline.md`.

**Modeling decisions**:
- Per-language primitive **grounding** (fact-bundle, per DECISIONS.md "D2 REVERSAL + FACT-BUNDLE RESEED"): the bundle of spec-read facts for each primitive — width / signedness / representation / overflow disposition / surface spelling — grounding into the shared `std/` vocabulary (T-3). Libraries such as `std::vector` are NOT modeled per L-2 — they are ordinary `Node`s. Deduplicate to a `std/` carrier only on proven identity; never a bare alias, never a re-declared algebra inhabitance (INVARIANTS P1:42)
- Per-language realization cost shape
- Grammar encoding: declarative production data — the **bidirectional relation** (concrete syntax ⟷ Node), read as ingest (partial, many→one, fail-closed off F) and emit (the chosen canonical section); NOT a procedural recognizer. The ingest reading MUST be unambiguous, or ambiguity ⇒ Diagnostic (never "parser picks one" = fabrication). Syntax needing semantic feedback to parse (C++ most-vexing-parse, `<` template-vs-less-than) is a STOP/escalation, not silently absorbed.
- Type system: nominal (Rust, Java) vs structural (TypeScript, Go), or both (C++)

**Reference**:
- v2: `src/v2/languages.dag`
- v3: `dsl/extdeps/languages/` (audit each for honesty)

---

### T-4.5: extdeps/process.dag + extdeps/file_system.dag

**File**: 2 files in `src/v4/extdeps/`
**Why bundled**: both are OS-interaction substrate; both are required for v4 to function as a self-hosting compiler (read source files, write emitted files, ExecuteCommand for boundary tests per THESIS facet 3).
**Why anchored**: each file carries a `# Anchor:` to its canonical reference (Wikipedia/POSIX). Reviewers validate the modeling against the reference — no invented vocabulary.

**Modeling decisions**:
- `process.dag`: how to model parent/child relationships? Signal handling depth (full POSIX signal set vs minimal {SIGTERM, SIGKILL, SIGINT})? Pipe model for capture (live-streaming vs buffered)?
- `file_system.dag`: AbsolutePath vs RelativePath as Disj sum or refinement on Path? Symlink target as recursive Path or opaque? Read failure modes (NotFound vs PermissionDenied vs IOError) as distinct Diagnostic `reason` name-references (`Symbol`, per std/diagnostic.dag — `reason` is an opaque name-reference, not a closed enum).

**Reference**:
- Anchors in file headers (Wikipedia: Process, Wikipedia: File system, POSIX File and Directory Operations)
- v2 / v3 had ad-hoc I/O sprinkled across files — v4 consolidates per substrate-cohesion discipline

---

### T-5: REMOVED — work-direction meta-layer cut (operator-ratified 2026-05-15)

T-5 ("workflow/* — recursive-flex": `brief.dag`, `worker_output.dag`,
`cycle.dag`, `retirement.dag`, `doc_anchor.dag`) is **deleted**. Rationale
(operator-ratified): modeling gunbc's *own work-direction* as `.dag` data
is not used by the project; the compiler model self-justifies (rationale
emergent from composition), so no meta-layer is needed to narrate it.
Tombstone retained (not silently dropped) so the decision is on record.

What survives in `workflow/`, as standalone tasks — these are *compiler
build infrastructure*, not the work-direction meta-layer:
- **`workflow/bootstrap.dag` → T-20** (the bootstrap chain; load-bearing
  for the anti-regression guarantee, STRUCTURE.md invariant 7).
- **`workflow/ci.dag` → T-24** (CI pipeline as data).

THESIS facet 4 / STRUCTURE.md invariant 6 narrowed correspondingly.
The 5 deleted files were never filled (scaffolds only); nothing in the
substrate imported them, so the cut is a pure scope reduction.

---

### T-6: compiler/01_tokenize.dag

**I/O**: `FreeMonoid<Char> -> Result<TokenStream, Diagnostic>`

**Modeling decisions**:
- Character class encoding (predicate fn vs enum vs charset)
- Whitespace/comment handling (preserve vs discard)
- Token boundary discipline

**Reference**:
- v2: `src/v2/01_tokenize.dag`
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-2

---

### T-7: compiler/02_parse.dag

**I/O**: `TokenStream -> Result<ParseTree, Diagnostic>`

**Modeling decisions**:
- Grammar productions as Node trees vs separate parser substrate
- Error recovery (single Diagnostic vs continued)
- ParseTree shape (layout-preserving?)

**Reference**:
- v2: `src/v2/02_parse.dag`
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-3

---

### T-8: compiler/03_normalize.dag + 03_resolve.dag

**I/O**: `ParseTree -> NormalizedTree -> ResolvedTree`

**Modeling decisions**:
- Surface sugar dissolution rules (service/fn/type -> Node tree)
- Identifier binding strategy (scope chain vs flat namespace)

**Reference**:
- v2: `src/v2/03_normalize.dag`, `src/v2/03_resolve.dag`

---

### T-9: compiler/04_infer.dag

**I/O**: `ResolvedTree -> Result<InferredTree, Diagnostic>`

**This is the file v2 split into 12 files (`04_*`).** v4's discipline: this is ONE file. Pressure to split = substrate design escalation, not a worker decision.

**Modeling decisions**:
- **The coercion fold** (rescoped 2026-05-17, D2-reversal — supersedes "algebra-homomorphism search algorithm"). Coercion is a **mechanical zip-fold** (a catamorphism) over two groundings — not a search, not research. It walks both canonical `Node` groundings in parallel and compares; `node.dag`'s B1-CANON contract `content_hash = merkle_fold ∘ canonical` already specifies the hard half (the canonical-form fold). Per `DECISIONS.md` U1 / C1 / T-9 the Find is **decidable by construction** over the closed declared candidate set — empty ⇒ Diagnostic, never a fabricated coercion. Name it the *coercion fold*; never an "engine" or "search algorithm".
- **The coercion quality tag** — the coercion result is the ratified `Outcome` carrier (`std/diagnostic.dag`), and **quality and outcome are distinct axes**. A *successful* coercion is `Outcome::Produced` carrying the coerced value **plus a closed quality tag**: `Identity` (groundings coincide) | `Exact` (related, total, lossless) | `Lossy` (related, with a *declared* accepted-loss). A coercion that cannot be derived is `Outcome::Rejected { diagnostic }` — the audit's missing fourth *outcome*, "can't prove ⇒ fail-closed". Fail-closed is **not** a fourth quality value: it is the `Rejected` branch of `Outcome`. The quality tag attaches only to `Produced`; never collapse the success-quality axis and the success/failure axis into one flat enum.
- **The composition rule** — when two *successful* coercions compose, their quality tags compose by a closed lattice (`Identity` is the unit; `Lossy` absorbs `Exact` and `Identity`; `Exact ∘ Exact = Exact`). If either coercion is `Rejected`, the composition is `Rejected` — `Outcome` short-circuits on the failure branch (the standard bind), so the failure axis needs no lattice entry. This is the audit's missing composition lattice; it lives here in T-9, not in a new task.
- `type AlgebraRef = Symbol` — `04_infer.dag`'s IR-1 `InferredFacts.inhabits` names `AlgebraRef`; it is a `Symbol` name-reference to the algebra inhabitance (the `Diagnostic.reason` cross-declaration idiom, K-1), not a type `std/algebra.dag` declares. Declared here (Theme-A audit #2).
- Cardinality propagation
- Diagnostic precision when inference fails

**Reference**:
- v2: `src/v2/04_*.dag` (12 files — read AS the cautionary tale on substrate inflation)
- v3 L2.5 design: PB-5 infer model (PR #3085)

---

### T-10: compiler/05_emit.dag + compiler/00_compile.dag — emission + orchestrator

**I/O**:
- `emit: (InferredTree, TargetSpec) -> Result<TargetSource, Diagnostic>`
- `compile: (Source, TargetSpec) -> Result<TargetSource, Diagnostic>` (orchestrator)

**Modeling decisions**:
- Target-agnostic IR shape
- How target spec drives concrete emission (interpreter vs codegen)
- Orchestrator: monadic `Result` chaining vs early-return pattern

**Reference**:
- v2: `src/v2/05_emit.dag`, `src/v2/compile.dag`
- v3 L2.5 design: PB-emit model (`docs/r3-retirement-modeling-emit-rs.md`)

---

### T-11: emit per-target specialization

**Why separate from T-10**: T-10 is the orchestrator; T-11 is the per-target translation tables that populate emit's behavior across **all five Shape-A targets — rust/python/go/cpp/typescript** (matching T-4's language set and the execution-graph critical-path line; T-16 depends on the full set).

**Modeling decisions**:
- Per-target translation rules — **as grammar-as-data, never string templates** (no-templating principle, operator 2026-05-17). The per-target "translation tables" are the declarative bidirectional grammar relation (concrete-syntax ⟷ `Node`, the canonical non-templated form — see T-4 "Grammar encoding"), NOT fill-in-the-holes string templates. A string-template emit path is the emit-side D2 hollow alias: an artifact the compiler cannot ground and check. STOP if a translation rule cannot be expressed as grammar-data.
- Target-specific optimizations (or absence thereof)

---

### T-12: lens/complexity.dag + lens/cost.dag

**I/O**: `Node -> Witness<ComplexityBound>`, `Node -> Witness<SymbolicCost>`

**Modeling decisions**:
- Complexity class encoding
- SymbolicCost lattice shape (per `docs/audit/sub-value-relation-bounded-lattice-claim.md`)
- Composition with Sum/Product algebra

---

### T-13: lens/{parallelism,effect,ownership,idempotency}.dag

**I/O**: `Node -> Witness<...>` per lens

**Modeling decisions per lens** (see file headers).

---

### T-14: test/claim/* + test/fixture/* — TestClaim corpus

**Files**: `src/v4/test/claim/*` directories (6 impossible_bug + algebra_laws + diagnostic_correction + future categories) + `src/v4/test/fixture/*`
**Operator-ratified additions 2026-05-15**: scaffolds for all 6 R1+R2+ impossible-bug classes already present (`test/claim/impossible_bug/{suboptimal_complexity,idempotency_contract,transport_type_drift,nested_optional_flatten,unenumerated_effects,unhandled_diagnostic_paths}.dag`); diagnostic_correction/ + algebra_laws/ directories ready for fill-in.

**Why bundled**: TestClaim corpus is one cohesive workstream; the coverage lens (T-18) enforces completeness structurally.

**Modeling decisions**:
- TestClaim shape per concern (input/expected/falsification triple)
- Demonstration vs verification — impossible-bug TestClaims are demos for the thesis claim; algebra-laws are testgen-derived; diagnostic_correction is end-to-end demos
- Fixture corpus shape (per-stage vs end-to-end?)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

**Why**: test infra port + fixture authoring. TestClaim data lives here.

**Modeling decisions**:
- Fixture corpus shape (how many fixtures? per-stage vs end-to-end?)
- TestClaim coverage discipline (every Diagnostic path covered)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

---

### T-15: bin/main.dag + self-host fixed-point validation (the anti-regression gate)

**Why last**: validates the whole stack. v4 compiles itself, produces bit-identical output, ships.

**Reframe (operator 2026-05-15)**: T-15's `BitIdentical` assertion is not just a self-host check — it IS the structural anti-regression guarantee. The v4 binary is a content-addressed release artifact; its fixed-point hash is pinned; any change rebuilds and must reproduce the exact hash or CI goes red. "Off Rust" is cashed here: the only editable authority is `.dag`; Rust cannot regress because none is authored and the binary hash is structurally locked. Consumes `workflow/bootstrap.dag` (T-20) for the orchestration.

**Modeling decisions**:
- `bin/main.dag` trampoline shape (1-line `include!()`; 0-floor per design-pure-bootstrap-zero.md:210)
- Fixed-point check: stage1-emitted == stage2-emitted (NOT stage0==stage1 — stage0 is v2-emission-style)
- Content-addressing scheme for the pinned binary hash
- CI gate shape: rebuild-from-.dag-via-frozen-seed must reproduce pinned hash

**Falsification probe — what "bit-identical self-host failure" looks like as TestClaim**:

```
data t_15_self_host_fixed_point: TestClaim {
  kind: Equals,   // Equals over B1 content_hash — see the Theme-A note below
  label: "v4 compiler is a fixed point — iteration N matches iteration N+1",
  input: content_hash(compile(src/v4/compiler/*.dag, target=Rust)),       // emitted Rust source, iteration N+1
  expected: <pinned content_hash of iteration N's emitted Rust source>    // same artifact, iteration N
}
```

**`BitIdentical` is a property name, not an `AssertKind`** (Theme-A audit, 2026-05-17): the probe's `kind` is `Equals` over the B1 `content_hash` of the two stage outputs — `verification.dag`'s closed `AssertKind` `{Equals, Diagnostic, Compiles, RoundTrips}` is sufficient; **no 5th kind**. The word "BitIdentical" elsewhere in this task denotes that *property*, never a substrate type.

Failure modes the probe MUST catch (each enumerable, each testable):
- **Non-determinism**: HashMap-iteration-order dependency in emit → different bytes between compilations
- **Hidden state**: global/static/ambient capability used in compiler logic → bytes vary with build environment
- **Test-double leakage**: mock or test scaffold loaded at compile-time → bytes differ when test toolchain absent
- **Substrate drift**: worker silently changed a substrate type without ratification → bytes differ from N to N+1

Once T-15 lands and stays green, all four failure modes are impossible-by-construction. A CI gate runs `cargo test t_15_self_host_fixed_point` per-PR on the v4 affected-set.

**Definition of v4-done**:
- **Every other scheduled task in this plan complete** — *every* task
  this plan schedules, T-15 itself excepted. This is deliberately **not
  an enumeration**: an explicit list goes stale — T-25 / T-26 / T-28 /
  T-29 / T-30 were scheduled after an earlier draft of this gate, and
  T-27 was dropped. The close gate is "the whole plan minus T-15",
  resolved against the plan as it stands at close time — never a
  hardcoded list or count that can omit in-scope work.
- v4 compiles `src/v4/compiler/*.dag` end-to-end
- v4 emits Rust source that compiles to a binary
- That binary, run on `src/v4/compiler/*.dag`, produces bit-identical output
- TestClaim suite passes
- Hand-authored Rust is **not the editable authority** — proven by REPRODUCTION, not a count (A3): rebuild-from-(.dag + frozen-pinned seed)-only reproduces the pinned hash; the seed's own hash matches its pin. (The old "count = 0" phrasing was the gameable v3 proxy — replaced. The machine-emitted trampoline is build-dir-transient, never authority.) The check is an early-surfacing amplifier run per-PR on the affected set, not an un-gameability claim.

### T-4.6: extdeps/formats/* (json/yaml/csv/toml/json_schema/openapi/sql)

**File**: 7 files in `src/v4/extdeps/formats/` (operator-ratified 2026-05-15: arbitrary ingestion via direction-agnostic format models; `sql.dag` added 2026-05-17 — Theme-A #4 fork (a))
**Why bundled**: identical structural shape per format; each file declares the format MODEL (data structure + parse/emit operations).

**Modeling decisions**:
- Recursive vs iterative parsing strategy (per-format)
- Number model (RFC 8259 §6 for JSON: arbitrary-precision OR IEEE-754; v4 default + opt-in)
- Schema-to-type derivation (json_schema.dag): given a schema, generate corresponding `.dag` types via `schema_to_type` operation
- Anchor/Alias resolution (yaml.dag): YAML's structure-sharing must resolve before producing typed value
- Dialect handling (csv.dag): delimiter/quote/escape/line-terminator parameterization
- **SQL DDL (`sql.dag`, Theme-A #4 (a))**: model the relational/DDL surface of the versioned ISO SQL spec (L-2 — the spec, not a dialect's library). T-16 emits its schema artifact *through* this grounded model so the DDL is coercion-checked against the domain types and stays inside T-16's shared `Node` tree — never a string-templated printout (no-templating principle). Dialect variance is `Declared-normalized`/`Fail-closed` per C5-fidelity, same as csv dialects.

**Scope**: M-L (medium-to-large; seven files but each is bounded by its anchored spec)

---

### T-4.7: extdeps/frameworks/react.dag

**File**: `src/v4/extdeps/frameworks/react.dag` (operator-ratified 2026-05-15: React framework substrate; coupled with T-16 full-stack demo)
**Why solo**: framework substrates are conceptually rich (Component / Hook / Effect / Lifecycle); React is the load-bearing first.

**Modeling decisions**:
- Hook-as-substrate: `HookKind = Builtin(BuiltinHook) | Custom(Node)`; `BuiltinHook` = the COMPLETE react.dev built-in set (no "..." — see react.dag header); custom hooks are Node composition per Rules-of-Hooks, not a new kind
- Effect lifecycle modeling (Mount / Unmount / DependencyChange / EveryRender)
- Rules-of-Hooks discipline (lens-checkable: no Hooks in conditionals — surface as Diagnostic)
- Component composition (props-down, events-up; structural propagation through Node tree)
- Server Components vs Client Components distinction (or unified via effect typing — effects intrinsic to the type signature, not an annotation)

**Scope**: L (large — substrate decisions cascade across full-stack demo T-16)

**Reference**:
- Anchor in file header (https://react.dev/reference/react)
- `docs/design-r4-full-stack-omni-emission-canvas.md` — 5-Q canvas (consult, do not block)

---

### T-4.8: extdeps/coordination.dag

**File**: `src/v4/extdeps/coordination.dag` (operator-ratified 2026-05-15 IN-B: Bind composition + effect typing — effects intrinsic to the type signature, NOT an annotation layer; NO 6th L1 behavior)
**Why solo**: multi-program coordination is the most consequential effect-typing in v4 — discipline matters.

**Modeling decisions**:
- Endpoint shape (NetworkAddress + LanguageRef + optional FrameworkRef)
- DeploymentUnit = collection of Endpoints + WireContracts between them
- WireContract = typed interface between two endpoints + CoordinationSemantics
- CoordinationSemantics = Sync | Async(SettleBound) | Stream | PubSub | EventuallyConsistent(ConvergeBound) (closed enum — operator-ratified C1 closure per node.dag discipline; non-immediate-settlement variants carry their bound as a STRUCTURAL field per operator fork 2026-05-15, read deterministically by the testgen simulator arm — see coordination.dag header)
- Effect-typing: HttpEffect, QueueEffect, StreamEffect, PubSubEffect — each is a typed parameter to Bind
- Failure-at-boundary modeling (composes with std/diagnostic.dag — no silent partial-failure)
- Idempotency at endpoint (composes with lens/idempotency.dag)

**Scope**: L (large — substrate decisions affect every distributed-app demo)

**Discipline**: NO 6th L1 behavior. If during work the temptation surfaces to add a `Coordinate` behavior to `std/node.dag`, STOP and escalate. The IN-B decision (operator 2026-05-15) is binding — coordination IS Bind composition + effect typing (effects intrinsic to the type signature, NOT an annotation layer).

---

### T-4.9 … T-4.14 — architecture stress probes (operator-ratified 2026-05-15)

**Parallel** tasks (need the B2-OMNI `LanguageModel` contract; independent of
each other once their named substrate inputs exist). Their value is that they
are **maximally diverse on purpose** — each is a *falsification probe* for the
B2-OMNI O(N+M) claim. If adding one is genuinely O(1) (one declarative model,
instantly cross-composing through the Node pivot), B2-OMNI is empirically
validated; if any forces a core/pipeline change, B2-OMNI is leaking — surfaced
now, before it is load-bearing. T-4.9-4.11 span the *upper* stack (HDL /
netlist / NL boundary); T-4.12-4.14 span *down* the stack (IR / machine code /
GPU) — together they validate the model across the **full target spectrum**:
source (F may include cosmetics by intent) → IR (F structural) → machine code
(F = encoding, no cosmetics). Long-held v2 intent, de-deferred per the
frontload-the-hard-cases discipline.

**Prerequisite correction (T-4.13).** `machine_code.dag` is not T-1-only.
It consumes two authorities that must exist before the probe is fillable:
`src/v4/std/machine.dag` for Byte/Word/MachineWidth and T-4's canonical
`LanguageModel`-as-data shape. The machine-code worker must consume those
models; it must not invent a local byte/word scaffold or a private language
model shape to keep the probe "parallel."

#### T-4.9: `extdeps/languages/verilog.dag`
- **Stress axis**: hardware **concurrency** vs the 5 L1 behaviors. This is the **IN-B validation probe** — if Verilog (`always @(posedge clk)`, continuous assignment) cannot be modeled as effect-typed `Bind` composition without a 6th `Concurrent` behavior, that is a **C1 stop-signal escalation**, and catching it early is the entire point.
- **Clear win**: one `.dag` FSM → simulable Verilog + a Rust reference model, same Node, zero translator.
- **Scope**: L (substrate-validating; concurrency model is the risk).

#### T-4.10: `extdeps/formats/spice.dag`
- **Stress axis**: is the format/`LanguageModel` abstraction *actually* general, or secretly programming-language-shaped? A SPICE netlist has **no control flow** — components + a connection graph.
- **Clear win**: one `.dag` circuit declaration → a SPICE netlist that simulates (omni-emission reaches analog hardware).
- **Placement (operator-ratified fork)**: `extdeps/formats/` — a netlist is a data format, not a programming language (sibling of csv/json), Shape B.
- **Scope**: M-L.

#### T-4.11: `test/claim/boundary/english_ingest_fail_closed.dag`
- **Framing (operator-ratified fork)**: English is **NOT a language model** (no formal grammar). It is a **boundary-honesty probe**, not `extdeps/languages/english.dag`.
- **Sequencing**: TestClaims here use `std/verification.dag`'s closed `TestClaim` / `AssertKind` vocabulary — that file is T-3 Wave-A2 substrate, still a scaffold until filled. T-4.11 is sequenced **behind** `verification.dag` (same graph edge as T-19 testgen): do not treat English as consuming an unfilled verification scaffold in parallel; wait for the T-3 authority, then bind claims.
- **Stress axis**: the C5 lossless-core boundary at its extreme, and the no-engine thesis made visible.
- **Clear win**: (a) Shape B emit — `.dag` → English docs (≈ T-16's existing Markdown artifact, no new substrate); (b) the honest win — `ingest(English prose)` → a precise Diagnostic, **never a fabricated parse**. The architecture refusing to lie *is* the demonstrable result.
- **Scope**: M (diagnostic + compile-boundary substrate exists; **TestClaim schema lands with T-3 verification** — the probe's claim instances follow).

#### T-4.12: `extdeps/languages/llvm_ir.dag`
- **Stress axis**: does the model generalize **down** the abstraction stack? SSA form / dominance / phi is structurally unlike a source AST. F is ~all-structural (LLVM IR has negligible cosmetic surface — the clean contrast point for C5-fidelity).
- **Clear win**: `.dag → LLVM IR` (LLVM lowers to machine code) + `LLVM IR → Node` ingest — the down-stack half of O(N+M).
- **Anchor**: LLVM Language Reference Manual, pinned release (L-2).
- **Scope**: L.

#### T-4.13: `extdeps/languages/machine_code.dag`
- **Stress axis**: the **bottom of the stack** — no cosmetic surface at all (the limit test for C5-fidelity), and **disassembly is the extreme fail-closed case** (most byte runs are not valid instructions; a disassembler that guesses = the no-engine violation made visible).
- **Shape (operator-ratified L-3)**: ONE `machine_code.dag` parameterized by an `Isa` model. Per-ISA files would reintroduce the N×M trap B2-OMNI forbids.
- **Prerequisites**: `src/v4/std/machine.dag` filled through the T-3 scalar/numeric stack (`logic` + `nat` -> `machine`) and T-4's canonical `LanguageModel` data shape. A local `Byte`/`Word` copy or private language-model schema is invalid.
- **Anchor**: the ISA spec (Intel 64 SDM / Arm ARM), pinned revision (L-2).
- **Scope**: L.

#### T-4.14: `extdeps/languages/ptx.dag` (CUDA)
- **Stress axis**: the **SIMT data-parallel execution model** vs the 5 L1 behaviors — the IN-B bet again (like Verilog's concurrency, but data-parallel). A needed 6th `Parallel`/`Kernel` behavior = C1 escalation, by design caught early.
- **Fork (PROPOSED — confirm)**: model **PTX** (the spec'd IR — clean, general, captures SIMT directly, parallel to llvm_ir; recommended) vs CUDA-C++ as a `cpp.dag` extension (entangled; the C++ surface is not where the stress is).
- **Anchor**: NVIDIA PTX ISA spec, pinned version (L-2).
- **Scope**: L.

---

### T-16: Full-stack omni-emission demo

**Output**: ONE `.dag` program → multi-language multi-endpoint application
**Operator framing 2026-05-15**: "consider pipeline emission i.e. 'backend program using react in the frontend (and say rust/C++ in the backend)' — i suggest we frontload this style of work — this is exactly what we keep deferring"

**Deliverable**: a single .dag file declaring a TODO-app-class application that emits:
- Rust backend (+ optionally C++ backend variant)
- React/TypeScript frontend
- OpenAPI wire contract between backend and frontend
- SQL DDL for persistence
- Markdown docs

All 5 artifacts share ONE Node tree (per gate #28 omni_layers_share_one_node_tree); coherence is structural, not test-checked.

**Modeling decisions**:
- How does the .dag file express endpoint partitioning (which fragment runs where)? (uses extdeps/coordination.dag's Endpoint + DeploymentUnit)
- Wire contract derivation (does it auto-derive from shared types, or is it explicitly declared?)
- Cross-target consistency: same domain types in Rust + TypeScript — tested via L5

**Scope**: XL (extra-large — this is the visceral cash of the omni-emission thesis)

**Why this task is the v4-flagship demo**: per operator "this is exactly what we keep deferring" — v4 fronts loading it because it forces the substrate decisions (T-4.7 React, T-4.8 coordination) to be made well, not as afterthoughts.

---

### T-17: lens/synthesis.dag + std/report.dag — cross-algorithm complexity (C7)

**File**: 2 files (operator-ratified 2026-05-15 IN: cross-algorithm complexity synthesis lens)
**Why bundled**: the synthesis lens is the consumer of the Report advisory carrier; both must land together for the lens to have anything to emit.

**Scope**: **XL** — research-tier risk **collapsed by the C2 reframe** (operator-ratified 2026-05-15). No undecidable equivalence engine, no unbounded rule library. Bounded to encoding the closed `LowerBoundTechnique` set + a few honest worked examples. STOP-and-escalate still applies if a relation class demands a 6th technique.

**Modeling decisions** (C2-reframed):
- **No semantic equivalence, no pattern library.** Synthesis reads the user's **declared** I/O relation (its contract/type — declared, never inferred; this dissolves the Rice-undecidability collision). It does NOT prove two programs equivalent and does NOT match against a `(naive→better)` catalogue (engine-shaped + unbounded — `feedback_no_engine`).
- **`LowerBoundTechnique` = closed set, enumerated up front**: `DecisionTree | AlgebraicRank | AdversaryCommunication | InformationTheoretic | ReductionConditional`. Each is a *general algebraic property over a relation class*, encoded once; adding the Nth algorithm adds ZERO entries.
- Synthesis = `compare(cost-lens-derived cost of the user's realization, the declared relation's lower bound derived via the technique set)`. No applicable technique ⇒ helpful Diagnostic (honest, never fabricated — `feedback_no_engine`).
- Report carrier shape (`std/report.dag`): closed-enum `ReportReason` disjoint from Diagnostic's `reason` name-reference (`Symbol`); advisory by construction; opt-in fail-closed via `apply_lens(synthesis, Enforce { ... })`.
- **Honest worked examples (for the worker later — illustrations of the technique→relation→lower-bound→compare flow, NOT a rule catalogue):**
  - *Sorting* — relation: "ordered permutation under a comparison oracle". Technique: `DecisionTree` ⇒ ≥ n! leaves ⇒ Θ(n log n). User Θ(n²) ⇒ Report the gap. (Merge-sort never named — the provable gap to optimum is surfaced, not a fix.)
  - *Matrix multiply* — relation: bilinear form. Technique: `AlgebraicRank` ⇒ naive n³ is rank-suboptimal vs n^ω. (Strassen never named; ω is open — the model surfaces structural suboptimality, refuses to fabricate an optimal.)
  - *Substring search* — relation: match positions over an n-length input. Technique: `InformationTheoretic` ⇒ Ω(n) (input must be read once); naive re-reads ⇒ Report the unforced re-scan. (KMP never named.)

**Reference**:
- `docs/r4-carve-out-routing.md` C7 — Director-tier design scope spec
- `lens/complexity.dag` — current-complexity input
- THESIS.md correctness dimensions §1.1 — complexity dimension parent
- INVARIANTS C-8 — fail-closed discipline (Report is the IS-NOT-fail-closed branch)

---

### T-19: lens/testgen.dag — producer of TestClaim corpus from substrate

**File**: `src/v4/lens/testgen.dag` (operator-ratified 2026-05-15: testgen as substrate fold; scheduled early — parallel fill — so the test corpus exists before the compiler stages need it)
**Why early**: per operator "i want testgen to be working fairly early — for the compiler itself". Scheduling it early (parallel fill, deps clear after T-3) means T-6+ tasks consume testgen-derived TestClaims rather than hand-authoring.
**Why solo**: testgen is a producer with cross-cutting consumption of every substrate file; one cohesive home.

**Modeling decisions**:
- Generator<C> generic carrier shape — one lens, parameterized over substrate concept type
- Per-substrate-kind testgen rules (see file header for the 5 categories: type-construction / algebra-law / diagnostic-exhaustiveness / lens-applicability / bidirectional-roundtrip)
- **TestClaim.classification** (`TestClassification`: Tier×Layer) on every produced claim — canonical field in `std/verification.dag` (STRUCTURE §248); testgen stamps the same axes on each emitted claim as on its scheduling `Generator<C>`. Tier1/2/3 (correctness) × Unit/Integration/Boundary (test layer)
- Bootstrap path: hand-authored TestClaims in `test/claim/manual/` are the contract testgen must satisfy; coverage lens (T-18) enforces produced ⊇ manual

**Scope**: L (large — substrate-traversal across every concept; cross-cutting consumption)

**Bootstrap pragma** (per operator: "manual authoring is fine as well"):
- After T-1 (`std/node.dag`) lands: hand-author 5-10 TestClaims in `test/claim/manual/` covering type-construction for the 6 connectives + 5 behaviors. Validates schema + shape immediately.
- After T-2 (`std/algebra.dag`) lands: hand-author algebra-law TestClaims for at least Magma/Monoid.
- After T-19 implementation: testgen produces same set programmatically; manual claims become regression anchors.

**Phase-1.5 scaffolding — forward dissolution (INVARIANTS P2)**:
- **`t19_manual_anchor_manifest.dag` — slug join table (P2):** rows are **`slug: String` only** — stable keys bijective with live `TestClaim` data in `connective_anchors.dag` ∪ `nat_law_anchors.dag`. **`TestClaim.classification`** (`TestgenTier`×`TestgenLayer`) and **`TestClaim.kind`** (`AssertKind`) are the **only** authority for tier/layer/assert form; the manifest does **not** duplicate them as `Int` (codex #3212). Testgen scheduling arm (`type_construction` vs `algebra_law` …) is implied by slug prefix until M2 can read claims or carry a single non-duplicated discriminant if needed.
- **`BehaviorValueSubject` (L1 `Value` in `type_construction`):** modeled in `lens/testgen.dag` subject carriers; **no** live manual `TestClaim` yet ⇒ **no** manifest row until one lands (same P2 bijection rule).
- **Retirement (same band as M2 unblock):** migrate `slug: String` → `Symbol` (or fold rows into `Set<ManualTestgenAnchor>` / reflection over the live claim corpus) once M2 cross-file loading + literal validation land; never reintroduce parallel `Int` encodings of axes already on `TestClaim`.
- **`lens/testgen.dag` — M2 materialization band:** subject folds / eval wiring land with cross-file M2 load; first change set that enables peer imports must keep declarations consistent with this file's live `type` carriers.

**Reference**:
- TESTING.md §141 "Test layers (target ratios)" — Unit ~75% / Integration ~15% / Boundary ~10%
- THESIS.md §168-182 — correctness Tier 1/2/3
- THESIS.md §348-368 — "Tests are structural data"

---

### T-20: workflow/bootstrap.dag — bootstrap orchestration AS DATA

**File**: `src/v4/workflow/bootstrap.dag` (operator-ratified 2026-05-15: the "off Rust, can't regress" load-bearing file)
**Why early**: the parse-viability step (v2 indexes src/v4) is needed from day 1 — it's the existing CI gate. The full self-host chain content grows incrementally as the pipeline matures. T-15 consumes the completed file for fixed-point validation.
**Why solo**: bootstrap orchestration is its own concern — it's the file that makes "compiler as data" structurally true rather than aspirational.

**Modeling decisions**:
- BootstrapPlan step sequence: seed (v2→stage0) / self0 (stage0→stage1) / self1 (stage1→stage2) / fixpt (assert stage1==stage2 BitIdentical)
- How v2's `run` interpreter executes this (the workflow is data v2 interprets, not Rust v2 compiles)
- Content-addressing of the pinned v4-stage-final binary
- Fail-closed on any step (compose with std/diagnostic.dag)

**Scope**: L (large — load-bearing for the entire anti-regression guarantee)

**The non-negotiable discipline**: this file is the ONLY bootstrap authority. A worker reaching for `build.rs` or `bootstrap.sh` has reintroduced editable Rust/script authority = the v3 regression door = STOP signal. v2 interprets this `.dag`; v2 is the frozen external seed (in `src/v2/`, outside `src/v4/`), touched exactly once per fresh bootstrap.

**Reference**:
- THESIS.md §223-226 — meta-process modeling ("Bootstrap ... modeled as .dag workflows")
- `docs/design-pure-bootstrap-zero.md` §"N=0 runtime boundary"
- STRUCTURE.md §"Bootstrap chain" + closed-system invariant 7

---

### T-18: lens/coverage.dag — meta-lens for coverage discipline

**File**: `src/v4/lens/coverage.dag` (operator-ratified 2026-05-15: structural coverage enforcement, not exhaustive fixtures)
**Why solo**: coverage discipline is its own concern — meta over the other lenses. One file owns the unified mechanism.

**Modeling decisions**:
- Coverage<C> generic carrier shape — one lens, parameterized over coverage concern (L6 form×target / L7 algebra×law×inhabitant / impossible-bug class enum / testgen type×inhabitant)
- Substrate read for each concern: derive EXPECTED set from substrate authority (not hand-enumerated)
- Comparison shape: actual TestClaim corpus vs expected derived set; emit Diagnostic per missing
- Composition with testgen: testgen produces TestClaims; coverage lens checks they cover the expected combinatorics
- Per operator: "make the target clear so we cannot bypass it this time" — the coverage lens MUST be structurally derived from substrate; cannot be opted-out, cannot be narrowed without substrate change

**Scope**: L (large — substrate-meta lens; multiple coverage concerns)

**Reference**:
- TESTING.md hermetic + behavior-driven discipline
- THESIS L6 §181 + L7 §182 + impossible-bug §370-413
- memory: feedback_no_textual_enforcement_bridges (coverage is structural, not grep-enforced)

---

### T-21: lens/affected_set.dag — incremental re-exec frontier

**File**: `src/v4/lens/affected_set.dag` (operator-ratified 2026-05-15: "something i wanted to get working very early on")
**Why early**: load-bearing for incremental cross-run execution AND it is the structural replacement for `scripts/detect-affected-components.sh` (the interim shell bridge currently gating v2/v3/v4 CI selection).

**Modeling decisions**:
- `affected_set: (Dag, Diff) -> Witness<ReExecFrontier>` shape
- Diff representation (file-set? node-set? structural-delta over the Dag?)
- Purity-aware skipping: an unchanged pure subgraph is incrementally skippable; what makes a subgraph "unchanged" structurally?
- Composition with `compiler/05_eval.dag` (skip) and `workflow/ci.dag` (job selection)
- Structural caching is the **dual** of the affected set — the same mechanism. A build/exec artifact's cache key is `content_hash` (B1) of its input subgraph: the affected set names what re-runs, a cache restores what doesn't. Caching is not a separate system. The cache backend (GHA `actions/cache`, a remote build cache, a local memo table) is just an emission target of the hash.

**Scope**: L (large — load-bearing for incremental execution + CI dissolution)

**Reference**: THESIS §205-210 free consequences (incremental cross-run) + v4-close-interrogation.md §2.5.F + memory: feedback_no_textual_enforcement_bridges

---

### T-22: compiler/05_eval.dag — the interpreter (PRIMARY execution path)

**File**: `src/v4/compiler/05_eval.dag` (operator-raised 2026-05-15: "what about the interpreter")
**Why load-bearing**: THESIS:225 — `dag run` is THE primary execution path. eval is not an afterthought to emit; it is the default. Sibling of `05_emit.dag` (same `InferredTree` input; eval executes, emit projects to target languages).

**Modeling decisions**:
- `eval: (InferredTree, Inputs) -> Result<Value, Diagnostic>` shape
- Bounded-execution enforcement (INVARIANTS P4 — no unbounded loops; how does the evaluator structurally refuse non-termination?)
- The shared substrate three consumers compose over: `workflow/bootstrap.dag` (interpreted, not compiled), TestClaim evaluation, lens dry-run
- Concept-unification (THESIS:188): interpreter runtime = language spec = transport spec — eval reads the same `extdeps/languages/*.dag` carriers emit does

**Scope**: XL (extra-large — THE primary execution path; bootstrap + tests + dry-run all depend on it)

**Reference**: THESIS:225 + concept-unification THESIS:188 + STRUCTURE.md §"Bootstrap chain" (v2's eval seeds; v4's eval takes over)

---

### T-23: lens/application.dag — apply_lens surface (opt-in depth)

**File**: `src/v4/lens/application.dag` (closes prior-audit BLOCKING GAP 1)
**Why load-bearing**: `apply_lens(<lens>, Enforce { ... })` is referenced by `report.dag`, `synthesis.dag`, and the C7 advisory→blocking bridge — but had no substrate home until now. It is simultaneously: §1.5 user-defined-dimensions surface, §6.2 audience-duality opt-in-depth mechanism, and the ONLY advisory→fail-closed path.

**Modeling decisions**:
- `EnforcedApplication<Output, Budget>` vs `IntrospectApplication<Output>` carrier shapes (v3 T-Lens-Application-Surface precedent: two separate carriers, NOT a sum — per r3-structure.md:40)
- `SectionRef = DeclarationScope | NodeScope` (where a lens attaches)
- The advisory→fail-closed conversion: how `Enforce { }` turns a lens's `Set<Report>` into fail-closed Diagnostics (the single explicit bridge per `std/report.dag` discipline)
- Default policy: a function with no `apply_lens(<lens>, Enforce { ... })` declaration gets synthesized Introspect-only (no implicit Enforce) per THESIS:307-321 opt-in depth. `apply_lens` is a first-class declaration (a Node), not an annotation — absence of the declaration, not absence of a tag, is the default trigger.

**Scope**: L (large — connective tissue for three thesis claims)

**Reference**: THESIS:95-101 + THESIS:307-321 + r3-structure.md:40 (v3 precedent) + std/report.dag discipline

---

### T-24: workflow/ci.dag — CI pipeline AS DATA

**File**: `src/v4/workflow/ci.dag` (closes prior-audit BLOCKING GAP 2)
**Why load-bearing**: THESIS:223-226 — "adding a CI gate = editing one .dag file." v3's gate #98 `ci_yml_hand_authority_dissolved` was an open R3 gap precisely because CI YAML stayed hand-authored. v4 must not reproduce it.

**Modeling decisions**:
- `CiPipeline { jobs, gates }` shape
- `.github/workflows/ci.yml` as DERIVED Shape-B artifact (.dag walks CiPipeline, emits YAML)
- Affected-set-driven job selection consuming `lens/affected_set.dag` (T-21) — this is what dissolves `scripts/detect-affected-components.sh`
- Structural cache keys: a cacheable job's `actions/cache` key is `content_hash` (B1) of its input subgraph, not a hand-authored `hashFiles(...)` glob. The interim `hashFiles(...)` keys in the committed `ci.yml` (e.g. the v2-compiler-binary cache) are manual approximations, replaced by emitted content-hashes when `ci.yml` is emitted from this file.
- The bootstrap interaction: CI runs `workflow/bootstrap.dag` (T-20)

**Scope**: L (large — closes the v3 hand-authored-CI gap; dissolves the shell bridge)

**Reference**: THESIS:223-226 + v4-close-interrogation.md §3.2 + v3 gate #98 (the gap not to reproduce)

---

## Summary

Every task in this plan is a bounded, modeling-load-bearing pure function (the count is intentionally unstated — it drifts as scope is ratified; see T-15's drift-proof close gate). Gaming surface is structurally bounded because adding files / splitting files / reaching outside declared substrate all require operator escalation. Per zero-deferrals: "I'll just do this for now" is forbidden — STOP and escalate.

If a task hits an unmodelable case or escalations pile up, that's a substrate-design signal — STOP, re-model, do not paper over.

The release is when v4-done. Not before, not after.

---

## Theme-A planning-debt closure — consumer-DFS audit (2026-05-17)

The adversarial consumer→`std/` DFS audit traced every planned consumer's
substrate needs against the merged `std/` plus the scheduled plan. It
found 12 items where a consumer needs substrate the plan never scheduled,
or a doc promises what the tree lacks. Per the operator framing — at this
stage the only debt v4 can introduce is *missed-planning* debt
(worker-discretion debt is foreclosed by STOP + settled contracts) — this
section closes that debt: every row below now points somewhere. The
**new-PROPOSED-task forks (T-25…T-29) were RATIFIED by the operator
2026-05-17** as part of the D2-reversal Phase-1 execution: zero
planning-level deferrals — every "(b) rule out-of-v4" escape was killed
and the schedule fork taken. T-25 (decomposed core + prover tail), T-26,
T-28 (bundled into T-8), and T-29 are **SCHEDULED** and join T-15's
close-gate plan; **T-27 was DROPPED** (ruled orthogonal to v4). The one
remaining fork — `#4 — T-16 SQL DDL` — was **RESOLVED by the operator
2026-05-17**: fork (a), SQL modeled as a checked `extdeps` Shape-B format
(scheduled by extending T-4.6). Every Theme-A fork is now disposed.

**Dissolved (no new substrate — wording/clarification):**
- **#1 `BitIdentical`** — = `Equals` over B1 `content_hash`; no 5th
  `AssertKind`. Encoded in the T-15 probe above.
- **#2 `AlgebraRef`** — `04_infer.dag`'s IR-1 `InferredFacts.inhabits`
  names `AlgebraRef`; `std/algebra.dag` declares no such type. It is a
  `Symbol` name-reference to the algebra inhabitance (the
  `Diagnostic.reason` cross-declaration idiom, K-1). Disposition: T-9
  declares `type AlgebraRef = Symbol` (or the IR-1 header states the
  identity) — a clarification in existing T-9 scope, not a new task.

**New PROPOSED tasks (the "missing substrate" Theme-A gaps):**

### T-25 — std/ value-predicate refinement substrate  [SCHEDULED]
**Gap:** `PositiveInt` (PID), `NonNegativeInt` (exit code), non-empty
`String` (paths/keys), `NonEmptyList` (`AbsolutePath`), and a general
`where`-clause / phantom-bound on records — needed by T-4.5
process/file_system, T-4.6 json/toml, rust.dag and others; no substrate
exists (`integer.dag` explicitly states "no `where`-clause").
**Disposition — SCHEDULED (operator ruling 2026-05-17; no rule-out). DECOMPOSE into core + tail:**
- **T-25-core** — a refinement modeled as a **base type + a fail-closed
  validation obligation**: a refined value is its base carrier plus a
  named validation that must discharge at a constructor boundary, failing
  closed (Diagnostic) when it cannot. This IS the audit's missing fourth
  coercion *outcome* — "can't prove ⇒ fail-closed" (T-9's `Outcome::Rejected`
  branch — the failure outcome, not a quality tag). `docs/coercion-design.md`
  Category 6 already designed this shape: chain to the base carrier + a
  validation at a named constructor boundary. T-25-core sits **near T-3**
  (the cardinality area) and is a **hard prerequisite** of the extdeps
  tasks that ground refinement-bearing carriers — **T-4** (the per-language
  fact-bundles, e.g. rust.dag), **T-4.5** (process/file_system — `PositiveInt`
  PID, `NonNegativeInt` exit code, `NonEmptyList` `AbsolutePath`), and
  **T-4.6** (the format models — non-empty json/toml keys). Those tasks
  carry T-25-core in their `[needs]`.
- **T-25-tail** — the **predicate prover** that *erases* a refinement once
  its predicate is proven (a pure optimization: a proven refinement need
  not re-validate downstream). Placed **after T-9**; never dropped.
**Independent sub-bug — decoupled from the T-25 schedule:**
`file_system.dag`'s header `Consumes` cites `std/collection NonEmptyList`,
a type `collection.dag` does not declare. This is a dangling-`Consumes`
bug independent of T-25 — the header cites a non-existent type today,
before any refinement substrate lands — so it is NOT gated on T-25. It is
routed standalone to the `file_system.dag` owner (T-4.5 / PR #3209) to
correct the header; the dangling `Consumes` stands on the PR-head tree
until that PR lands. (`NonEmptyList` itself, once T-25-core lands, is a
`List` refinement — `List<T> where non_empty` — not a separate carrier.)

### T-26 — std/ boundary carriers (net-address / URL / HttpMethod)  [SCHEDULED]
**Operator ruling 2026-05-17 — SCHEDULED; the port disposition below stands (no fork).**
**Gap:** `HttpMethod` and `URL` already have a single authority in the
reference tree — `dsl/std/types.dag` (`HttpMethod` = the RFC 9110 enum;
`Url` = a `String` refinement). They are not yet ported to v4 `std/`, so
v4 consumers (`openapi.dag` references `HttpMethod`; the T-16 wire
contract) have no carrier to `Consume`. `NetworkAddress` appears only in
`coordination.dag` prose — DFS the concept DAG (M9) for an existing
authority before minting.
**Disposition:** **port** `HttpMethod` / `Url` into v4 `std/` from the
`dsl/std/types.dag` authority — RFC 9110 / the URL spec are genuine
shared facts, so the home is `std/`, not a new `extdeps` file; **create**
a spec-grounded `NetworkAddress` carrier in `std/` if M9 finds none.
Consumers (`openapi.dag`, `coordination.dag`, T-16) `Consume` the single
`std/` authority. Minting a parallel `extdeps` carrier would be the very
P2 violation this task names (INVARIANTS P2 / M9).

### T-27 — extdeps version / semver / edition lattice  [DROPPED]
**Operator ruling 2026-05-17 — ruled orthogonal, out of v4 entirely.**
Versioning / spec-drift is a property of how external specs are *consumed
over time* — not of the compiler's projection / coercion mechanics. It is
not a v4 task and gets no substrate. Where a model genuinely needs a spec
edition at a point in time, that pin lives in the model's `# Anchor:` as a
fixed-edition reference (the existing anchor convention); nothing schedules
a semver / ordering lattice. Tombstoned here so the T-2# numbering stays
stable; the original gap text is intentionally removed — it described a
task that will not exist.

### T-28 — std/ module-graph substrate  [SCHEDULED]
**Gap:** `03_resolve` cross-file binding and `rust.dag`'s `PubInPath`
visibility both need a module-tree + an ancestor-relation `Witness`; no
substrate exists, no scheduled task. (This is the substrate side of the
Theme-B "module-loading" dependency.)
**Disposition — SCHEDULED (operator ruling 2026-05-17).** Schedule a
`std/` module-graph carrier, **bundled into T-8** (the
`03_normalize`/`03_resolve` work — `03_resolve` is the primary consumer).
Not a standalone task: the module-tree + ancestor-relation `Witness` land
inside the T-8 resolver scope.

### T-29 — extdeps C++ ABI / target data-model  [SCHEDULED]
**Gap:** `cpp.dag`'s fact-bundle grounding of `int`/`long`/… into the
`std/` numeric vocabulary is undefined without an ABI data-model — C++
integer widths are implementation-defined (LP64 / ILP32 / …), so the
width fact is not a constant of the language but of the target ABI.
**Disposition — SCHEDULED (operator ruling 2026-05-17; no fork).** Schedule
an `extdeps` ABI / target-data-model slice that the `cpp.dag` fact-bundles
parameterize over (LP64 / ILP32 / …). Low-dependency — it needs only T-3's
machine / width vocabulary, otherwise a leaf — but it is **NOT parallel
fill**: T-29 is a **side-branch feeder of T-4** (a hard prerequisite of
T-4's cpp slice — the cpp fact-bundle cannot ground implementation-defined
integer widths without it; hence the `T-4 [needs … T-29]` edge). It is a
**watch item** — schedulable the instant T-3's `machine` lands, and it
*should* be scheduled then, because if it slips the `{P1-KEYSTONE, T-30,
T-29, T-25-core} → T-4 → T-9` side branch goes critical. Low-dependency
≠ low-priority.

### T-30 — std/ structural fact-density / hollow-alias gate  [SCHEDULED]
**Operator ruling 2026-05-17 (codex 13403, via the D2-reversal Phase-1
resolution).** A generated structural checker — a pure function
`Node -> Outcome` — that **fails closed on a hollow alias**: a carrier
that reads zero facts from its source spec (`type RustI32 = Int32` and
its kind). It is the *structural* enforcement tier the D2-reversal root
cause named missing — "a hollow alias is invisible to every structural
gate" — and it makes the hollow alias *impossible*, not merely
review-discouraged.
**Why a task, and where it sits.** T-30 is its own foundation task — a
sibling of `P1-KEYSTONE`, **not** folded into `docs/modeling-discipline.md`
(that doc carries the *convention*-tier bad-example; T-30 is the
*structural* tier). It is a **hard prerequisite of T-4**: `T-30 → T-4`.
The per-language fact-bundle rework does **not** begin under
convention-tier-only enforcement — convention is exactly what let D2
through, and reworking every per-language file that way would re-expose
the same surface. The `docs/modeling-discipline.md` bad-example is the
*interim floor* only; T-30 is the enforcement the reseed actually runs
under. Sequenced by the dependency edge, not by phase number (see
`DECISIONS.md` "D2 REVERSAL + FACT-BUNDLE RESEED", Phase 1).
**Modeling decisions:** what counts as a *fact* (the carrier decomposes
into ≥1 spec-read fact beyond a bare std alias); how the gate reads
fact-density off a `Node` carrier; the kernel-ambient exemption (`Bool`
and the other kernel-ambient atoms are legitimately atomic — not hollow);
the Diagnostic shape on a fail-closed hollow alias.

**Scope / clarification dispositions:**
- **#4 — T-16 SQL DDL — RESOLVED (operator 2026-05-17): fork (a).** SQL is
  modeled as a **checked `extdeps` Shape-B format** — a sibling of
  `csv`/`json`/`yaml` (SQL has a versioned ISO spec, L-2-admissible).
  T-16 emits the DDL **through** that grounded format model, so the schema
  is coercion-checked against the domain types and cannot drift — it stays
  inside T-16's gate #28 (all omni-layers share one `Node` tree). Fork (b)
  — a string DDL printout — is **ruled out**: an artifact the compiler
  cannot ground and check is the templating smell (the emit-side
  equivalent of the D2 hollow alias; see the no-templating principle).
  Scheduled by **extending T-4.6** — `extdeps/formats/sql.dag` joins the
  formats bundle as a 7th file. (Not a standalone task: SQL DDL is a
  format sibling of the other six; bundling matches how they are scoped.)

  **Single SQL authority — `sql.dag` is a PORT, not a second authority**
  (briansrls + codex blocking review, #3224, 2026-05-17). A v3 SQL
  authority already exists: `dsl/extdeps/sql/migration.dag` models
  migration-script shape + ordering constraints (`SqlMigrationOperationKind`,
  `SqlMigrationStep`/`Script`/`EmissionTarget`), and `dsl/extdeps/
  transports/sql.dag` carries the transport execution config. v4
  `extdeps/formats/sql.dag` **ports / consumes / reconciles** that
  existing SQL split — into the v4 checked-Shape-B-format shape — it is
  **not** a freshly-authored parallel SQL model. Authoring a second
  generated-SQL authority beside `migration.dag` violates INVARIANTS P2
  (single authority) and extdeps fidelity (the spec already has a modeled
  home). T-4.6's `sql.dag` brief states the port explicitly: every SQL
  fact the v3 files already model is carried forward, not re-invented;
  net-new modeling is only what the v3 contract did not cover, and the v3
  files are retired into the v4 one (no dual representation left
  standing).
- **#9 — `LanguageModel` / `TargetModel` named type.** `00_compile.dag`
  prose (B2-OMNI) is parameterized over "declarative LanguageModels" but
  no `type LanguageModel` is declared. Disposition: T-6/T-10 either
  declare the carrier type, or the B2-OMNI header states formally "a
  LanguageModel IS a `Node` — no separate type." A naming-clarity fix in
  existing scope, not a new task.
- **#12 — ExecuteCommand TestClaims.** THESIS facet 3 names
  `ExecuteCommand`-based `TestClaim`s; v4 models the boundary via a
  simulator `Node` + the closed 4 `AssertKind`s. Disposition:
  confirm-only — T-19/T-14 verify `ExecuteCommand`-shaped TestClaims are
  expressible via `process.dag` + `eval` with no lost predicate surface
  vs v3; if a gap surfaces, escalate. No planning edit pending the
  confirm.

**THESIS-vs-tree dispositions (encoded in `THESIS.md`, this PR):**
- **#10 — timing lens.** THESIS facet 4 listed a *timing* lens; the
  closed `lens/` tree has none. Disposition: timing is a projection of
  the **cost** lens (cost is the time/complexity dimension — U2), not a
  separate lens — THESIS facet 4 edited accordingly.
- **#11 — Swift.** THESIS Shape-A lists Swift; no `swift.dag`.
  Disposition: **not a gap** — THESIS:217 is an illustrative capability
  claim (Shape-A costs one language spec), not a v4 task commitment;
  TASKS.md T-4 scopes the v4 languages. No edit; confirmed.

Net: every Theme-A row now points to a scheduled task, a wording fix, a
scope ruling, or a confirmed non-gap. The operator ratified the T-25…T-29
disposition forks and resolved the `#4 — T-16 SQL DDL` fork (fork (a) —
SQL as a checked `extdeps` Shape-B format) on 2026-05-17 (D2-reversal
Phase-1 execution). Theme-A missed-planning debt is **closed** — no fork
remains open.

### T-31 — de-prose / de-templating backward sweep  [SCHEDULED]
**Operator-confirmed 2026-05-17 (D2-reversal Phase-1 execution).** The
no-prose and no-templating principles are operator-ratified, but they
currently get only *forward* enforcement: `P1-KEYSTONE` makes new work
compliant. The *backward* sweep had no task home — every already-merged
`.dag` file (`node.dag`, `algebra.dag`, the whole `std/` stack, the
`extdeps/` files) carries large prose headers today. T-31 is that home.

It has two parts:

- **(a) The rework rider — no separate PR.** Every fact-bundle / T-##
  rework PR de-proses and de-templates the files it already touches, *in
  the same edit*. A file is never touched twice — a rework PR that
  modifies a file's body and leaves a stale prose header behind is
  incomplete. This is a rider on the existing rework tasks (T-4 and the
  D2-impact rework), not a task that dispatches its own work.
- **(b) The mop-up task.** A standalone pass over already-merged `.dag`
  files that *no other rework PR will touch* — settled `std/` files,
  compiler stages not otherwise changing. Without this, those files keep
  their prose indefinitely.

**KEEP / REMOVE / RELOCATE** is the per-line classification:
- **KEEP** — the structured header *contract* (a file's `Consumes` /
  `Owns` / `Scope` lines, behavior tables, the machine-readable parts a
  reviewer and the substrate depend on).
- **REMOVE** — rationale prose, narrative motivation, design-history
  asides: anything a reader does not need to *use* the file.
- **RELOCATE** — long-form rationale worth keeping moves to
  `src/v4/DECISIONS.md` or a `docs/` subdoc. The `.dag` file keeps **no
  pointer**: per Practice 9 a file's comments are only the four allowed
  classes (file-path line, terse header, per-carrier `// Anchor:`,
  optional one-line concept tag), and a `see docs/X` pointer is not one
  of them. The relocated rationale is discoverable at its destination,
  not linked from the carrier.

**Load-bearing files** (`node.dag`, `STRUCTURE.md`-named substrate,
the four pipeline stages) de-prose **carefully**: KEEP the structured
header contract intact — REMOVE only the rationale prose around it. When
in doubt on a load-bearing file, KEEP.

**Where it sits.** Parallel fill, **not** critical path — it does not
block the T-4 gates and is not blocked by them. It cites
`docs/modeling-discipline.md` (the no-prose / no-templating rules) once
that doc lands, but need not block on it: the KEEP/REMOVE/RELOCATE
classification is settled. The rider (a) starts with the first rework
PR; the mop-up (b) dispatches immediately as parallel fill.

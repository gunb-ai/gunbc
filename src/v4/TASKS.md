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
T-1 → T-2 → T-3 → T-6 → T-7 → T-8 → T-9 → T-10 → T-11 → T-16 ─┐
                                            └─ T-36 ───────────────┴→ T-15
```

`T-1 → T-2 → T-3` is serial and unavoidable — the substrate foundation.
After T-3 the spine is the serial compiler pipeline
`T-6 → T-7 → T-8 → T-9 → T-10`, then two parallel T-15 gates:
`T-11 → T-16` (full-stack omni-emission demo) and `T-36` (ingest round-trip
fidelity claim, needs only T-10). Both must be complete before T-15.

```
  T-1   std/node.dag                     [BLOCKS: all]
  T-2   std/algebra.dag                  [needs T-1]
  T-3   std/* supporting (11 files)      [needs T-1, T-2; OWNS the full shared-fact vocabulary — signedness, representation, the numeric stack — see T-3 detail]
  T-6   compiler/01_tokenize.dag         [needs T-3]
  T-7   compiler/02_parse.dag            [needs T-6]
  T-8   compiler/03_normalize.dag + 03_resolve.dag   [needs T-7; T-28 module-graph substrate is bundled here]
  T-9   compiler/04_infer.dag            [needs T-8, T-2, T-3, and T-4 — T-4 enters from the side branch below, not the spine]
  T-10  compiler/05_emit.dag + 00_compile.dag       [needs T-9, T-4, T-23]
  T-11  emit per-target specialization (extends T-10 across all 5 Shape A targets)   [needs T-10]
  T-16  Full-stack omni-emission demo: ONE .dag → Rust+C++ backend
        + SQL DDL schema (Shape-B, via T-4.6 sql.dag — Theme-A #4)
        + React/TS frontend + OpenAPI wire contract
        [needs T-4, T-4.5, T-4.6, T-4.7, T-4.8, T-10, T-11]
        (T-4.8 coordination.dag is load-bearing — T-16 uses it for
        endpoint partitioning; facts must flow forward from the
        coordination substrate into the flagship demo)
  T-36  Omni ingest demo: `.dag` source → Node → emit → source (round-trip)
        [needs T-6, T-7, T-8, T-9, T-10; dag.dag lex/grammar data from T-6/T-7]
        One executable claim: parse a known `.dag` program, emit it back,
        assert bit-identical. Validates `ingest = emit⁻¹` (C5) is not
        just a property claim but a checked, executable fact before T-15.
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

### Side branch — `{P1-KEYSTONE, T-30, T-29, T-25-core, T-33, T-19, T-21} → T-4 → T-9` (watch item)

```
{P1-KEYSTONE, T-30, T-29, T-25-core, T-33, T-19, T-21} → T-4 → T-9
```

T-9 needs T-4 (the language fact-bundles) in addition to T-8. This branch
carries slack against the `T-6→T-7→T-8` pipeline branch **only if its
feeders start immediately**. The D2 reversal CHANGED T-4's dependency set
— the old alias model needed almost nothing; fact-bundle modeling needs
the shared vocabulary (`T-3`, itself on the critical path) plus
**seven feeders that are not on the critical path and gate `T-4 → T-9`**:
P1-KEYSTONE, T-30, T-29, T-25-core, T-33, T-19, T-21. Those seven are **watch items**,
not slack-having parallel fill — if any slips, the side branch goes
critical. T-4 is no longer a schedule-anytime leaf — see T-4.

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
  T-33  std/model_core.dag — shared substrate factoring (Option C).
                LanguageModel (T-4) and concrete runtime extdeps (T-34)
                both consume it;
                T-4's fact-bundle authoring cannot ground primitives + algebra
                inhabitance + laws against ModelCore until the carrier file
                exists. Low-dependency (needs only T-1, T-2, T-3) but a hard
                T-4 prerequisite — see T-33.
  T-19  lens/testgen.dag — **LanguageBehaviorEquivalence corpus emission**
                authority for T-4 conformance (see T-19 detail). Low-
                dependency (needs only T-1, T-2, T-3) but a hard T-4
                prerequisite for the per-language conformance deliverable.
  T-21  lens/affected_set.dag — **incremental re-test selection** over
                T-4 conformance claims (IRT-1; see T-21 detail). Low-
                dependency (needs only T-1, T-2, T-3) but a hard T-4
                prerequisite for the per-language conformance deliverable.
  T-4   extdeps/languages/{rust,python,go,cpp,typescript}.dag
        [needs T-3, P1-KEYSTONE, T-29, T-30, T-25-core, T-33, T-19, T-21 — see T-4]
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
  T-4.5 extdeps/{posix,file_system}.dag                        [needs T-3, T-25-core]
  T-4.6 extdeps/formats/* (7 files: json/yaml/csv/toml/json_schema/openapi/sql)  [needs T-25-core, T-26]
  T-4.7 extdeps/frameworks/react.dag    [needs T-4 (typescript)]
  T-4.8 extdeps/coordination.dag         [needs T-4, T-4.7]
  T-4.9  extdeps/languages/verilog.dag   [needs T-1, T-2; header Consumes: std/node.dag; std/nat.dag (Nat); B2-OMNI falsification probe — concurrency vs the 5 behaviors]
  T-4.10 extdeps/formats/spice.dag       [needs T-1; B2-OMNI falsification probe — LanguageModel generality (no control flow)]
  T-4.11 test/claim/boundary/english_ingest_fail_closed.dag  [needs T-1, T-3 std/verification.dag; boundary-honesty probe — TestClaim/AssertKind bind after verification.dag fill, not parallel to Wave-A2 scaffold]
  T-4.12 extdeps/languages/llvm_ir.dag   [needs T-1, T-2; B2-OMNI probe — generalize DOWN the stack (SSA IR)]
  T-4.13 extdeps/languages/machine_code.dag  [needs T-3 machine + T-4 LanguageModel shape; B2-OMNI probe — bottom of stack; disassembly = extreme fail-closed]
  T-4.14 extdeps/languages/ptx.dag       [needs T-1, T-2; B2-OMNI + IN-B probe — SIMT data-parallel vs the 5 behaviors]
  T-5   REMOVED 2026-05-15 (operator-ratified) — work-direction meta-layer
        cut; only workflow/bootstrap.dag (T-20) + workflow/ci.dag (T-24) remain
  (T-4.15 protocols substrate is NOT in this "instant parallel fill" block —
   see "Close-the-loop + late substrate" below. It is scheduled-but-deferred:
   file authoring activates when omni-stack glue work activates, per P4
   "Out of scope for the initial single-target compiler.")

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
  T-24  workflow/ci.dag                  [needs T-21, T-20, T-10, T-23]
        CI pipeline AS DATA; .github/workflows/ci.yml derived. Closes
        v3's gate-#98 gap (hand-authored CI YAML). Consumes T-21 for
        job selection — the shell bridge dissolves once both land.
        Consumes T-10/T-23 for lens verdict via `run_required_lens_gates`
        (T-24 schedules; T-10 owns the orchestrator gate surface).

Interpreter + lens dimensions (each needs T-9):
  T-22  compiler/05_eval.dag             [needs T-9, T-34]
        The interpreter — THE PRIMARY execution path (THESIS:225).
        Sibling of emit (same InferredTree input). workflow/bootstrap.dag
        + TestClaim eval + lens dry-run all compose over it.
        T-34 superseded 2026-05-21 (Option C) — eval consumes decomposed
        runtime carriers plus a concrete runtime extdep; eval cannot be
        authored before the runtime carriers exist.
  T-12  lens/complexity.dag + lens/cost.dag      [needs T-9]
  T-13  lens/{parallelism,effect,ownership,idempotency,structural_resolution}.dag   [needs T-9]
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
  T-4.15 extdeps/protocols/{rest,graphql,grpc}.dag — transport substrate
        (Ratified P4)
        [needs T-3, T-26; scheduled-but-deferred — file authoring activates
        with omni-stack glue work per P4 ("Out of scope for the initial
        single-target compiler. Architecture must not preclude, but no
        implementation in the initial single-target compiler"). Language-
        orthogonal: transport declares its own wire-format type system;
        LanguageModel ⊗ TransportModel composition is the future-omni-stack-
        expansion's responsibility (beyond T-16's current OpenAPI scope —
        T-16's `[needs]` does NOT include T-4.15 today), not this
        substrate's. Single-authority for the activation gate lives in
        the task body's "Out of scope for the initial single-target
        compiler" section — see T-4.15.]
  (T-33 std/model_core.dag is NOT in this "late substrate" bucket —
   see the side-branch feeders block at the top of this file. T-33 is a
   watch-item T-4 prerequisite, not slack; the side branch goes critical
   if T-33 slips.)
  T-34  std/runtime.dag + extdeps/runtimes/*.dag — runtime substrate (Option C)
        [needs T-33] — std/ owns abstract runtime carriers; extdeps/runtimes/
        owns concrete runtime bundles over the same ModelCore. Carries runtime
        value representation, primitive operation interpretation, execution
        semantics, resource / effect boundary.
        Consumed by T-22 (eval) and the MVP-B route.
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

**Deliverable (gate for T-4 Wave 2b)**: In addition to the typed algebra structures (`OrderedRing<T>`, `ApproximateField<T>`, `BooleanAlgebra<T>`, etc.), T-2 must export a **Node constructor for each algebra type** — e.g. `ordered_ring_node(inhabitant: Node) -> Node`, `approximate_field_node(inhabitant: Node) -> Node` — so that T-4 Wave 2b and runtime extdeps (T-34) can populate `AlgebraInhabitanceDecl.algebra` with a grounded, walkable Node rather than a bridge Symbol atom. Each constructor returns a **`Conj` + named-edge fact-bundle** (see the ratified shape below) — not an `Instantiation` connective or bare-Atom wrapper. T-4 Wave 2b is explicitly gated on this deliverable. Language files using bare-Atom bridge symbols as placeholders for algebra references (`rust_model_core_bridge_std_ordered_ring_representable_integer`, etc.) will fail-closed at T-9 (`infer_algebra_ref_ungrounded`) until these constructors exist and are used.

**Modeling decisions**:
- Inhabitance declaration shape (relation? predicate? typeclass-style?)
- Composition: how do Sum/Product algebras compose for the cost lens?
- Free constructions: FreeMonoid<T> as primitive vs derived?
- **Node constructor shape (operator-ratified 2026-05-25):** `Conj` + named `Edge { label: Named { name: … }, target: … }` children — the same fact-bundle carrier M1 / Practice 8 names for spec-read facts. **Not** an `Instantiation` connective and **not** a bare Atom with unstructured children. Positional-only `Conj` leaves are insufficient: named edges are required for fact-density (T-30), content-hash stability (B1), and T-9's coercion fold to walk the algebra reference. Reference pattern: `extdeps/languages/rust.dag` inhabitant/facts nodes (`rust_named_edge`, `rust_facts_*_node`).

**Reference**:
- v3: `dsl/std/algebra.dag` (study; expected substantive)
- `THESIS.md` "Epistemic stacking" section

---

### T-3: std/* supporting (cardinality, witness, diagnostic, collection, verification + the scalar/numeric stack)

**File**: 11 files in `src/v4/std/` today — `cardinality`, `witness`, `diagnostic`,
`collection`, `verification`, plus the **scalar/numeric stack** (`logic`,
`nat`, `machine`, `integer`, `float`, `text`) that replaced the deleted
`primitive.dag` — see `STRUCTURE.md` §"Scalar/numeric concept decomposition".
**Scheduled (named carrier home, checkable arrival):** **`src/v4/std/datetime.dag`**
(module `v4.std.datetime` once authored) — RFC 3339 / clock-calendar instant
facts for format-layer consumers; **absent from the tree until** the Wave-A2+
landing PR; dissolution paired with `DECISIONS.md` Part 6 ·
`SL-3229-T4-FORMAT-TOML-DATETIME` (T-4.6 `toml.dag` wires ops against this file).
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
part of this vocabulary. **Temporal / RFC 3339 structured instants** (format
lexemes → clock facts; T-4.6 consumers — `DECISIONS.md` Part 6 ·
`SL-3229-T4-FORMAT-TOML-DATETIME`) are **T-3-owned `std/` vocabulary** with the
**concrete scheduled home** `src/v4/std/datetime.dag` (see **File** above), same
scheduling envelope as other Wave-A2+ shared facts. Each **numeric /
physical-quantity** axis is a real modeled fact, placed in the
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

**Dependencies — re-gated by the D2 reversal (operator-ratified 2026-05-17) + Ratified Q1 (operator-ratified 2026-05-20) + conformance deliverable (operator-ratified 2026-05-25).** T-4 is no longer a schedule-anytime Phase-1 leaf: `[needs T-3, P1-KEYSTONE, T-29, T-30, T-25-core, T-33, T-19, T-21]`. The old alias model needed almost nothing — a bare alias reads no facts. Fact-bundle modeling needs T-3's shared-fact vocabulary (signedness/representation/numeric stack), the `P1-KEYSTONE` modeling-discipline rubric (the doc against which every bundle is authored and reviewed), the `T-30` structural fact-density / hollow-alias gate (the per-language rework does not run under convention-tier-only enforcement — see T-30), `T-25-core` (the refinement substrate — a language fact-bundle that grounds a refinement-bearing carrier needs the base-type + fail-closed-validation shape), `T-33` (the `std/model_core.dag` shared substrate that LanguageModel extends per Ratified Q1 — primitives + algebra inhabitance + laws ground against ModelCore, not re-declared per-language), and — for the cpp slice — the T-29 C++ ABI / target data-model. **T-19 + T-21** gate the per-language **`LanguageBehaviorEquivalence` conformance corpus** (T-19 emits; T-21 selects re-runs — see ownership rows under T-19/T-14/T-21). T-4 sits on the `{P1-KEYSTONE, T-30, T-29, T-25-core, T-33, T-19, T-21} → T-4 → T-9` side branch — its feeders are watch items, not slack-having parallel fill; see the execution graph. The D2 reversal *changing this dependency set* is the single most consequential planning edit of the reseed; T-33 is the 2026-05-20 Q1 addition; T-19/T-21 is the 2026-05-25 conformance addition.

**Note on T-33 edge scope.** Adding T-33 to `[needs …]` is a graph-edge update — it records that LanguageModel cannot be authored before ModelCore exists. It does **not** restructure the T-4 fact-bundle authoring contract (the body text above) to be expressed in terms of "LanguageModel extends ModelCore". That re-expression is its own commit train, after T-33 lands.

**Authoring contract (operator-ratified 2026-05-15; D2 bullet superseded 2026-05-17):**
- **Model the SPECIFICATION, not libraries (L-2).** Model the versioned upstream spec (Rust Reference, ECMAScript/TS Handbook, IEEE 1364, …) — the anchor IS that spec. Do NOT model std/crates/packages: a library is just a program in the modeled language = `Node`. Modeling libraries is infinite, non-general, the wrong layer.
- **Declare every surface feature's disposition (C5-fidelity).** For each feature: `Modeled` (∈ F, Node-bearing, round-trips both ways — e.g. Python indentation IS block structure) | `Declared-normalized` (deliberately not in F; `emit∘ingest` canonicalizes — Go/C++ insignificant whitespace; a *declared*, reviewable loss, never silent) | `Fail-closed` (encountered but neither → Diagnostic, no-engine). F = the spec's own meaning-vs-lexical distinction, not worker judgment. Round-trip fidelity = declared model completeness.
- **A language file FACT-BUNDLES each primitive (fact-bundle reseed — operator-ratified 2026-05-17, supersedes D2).** For each primitive the file authors a **fact-bundle**: the facts read from that language's *own spec* — width, signedness, representation, overflow / NaN-Inf disposition, surface spelling — each a real modeled carrier grounding into the shared `std/` vocabulary (T-3). It does NOT bare-alias to the `std/` carrier: `type RustI32 = Int32` models *nothing about Rust* — it asserts an unproven identity while reading zero facts. A bundle deduplicates against a `std/` carrier ONLY where the identity is **proven** — a compiler-verified coincidence of the language bundle with the `std/` bundle, cited as evidence. `extdeps/` models systems we do not control: default to separate, honest modeling; reuse `std/` only on evidenced identity. A per-language `OrderedRing<<lang>Prim, …>` re-declaration is still the parallel-*algebra* substrate INVARIANTS P1:42 forbids — model the facts, never a duplicate algebra and never a hollow alias. See `DECISIONS.md` "D2 REVERSAL + FACT-BUNDLE RESEED" and `docs/modeling-discipline.md`.
- **Per-language conformance test (operator-ratified 2026-05-25):** Each language slice ships a **`LanguageBehaviorEquivalence` TestClaim corpus** proving the fact-bundle round-trips against a frozen behavior snapshot — an **explicit T-4 deliverable**, not a post-close follow-on. **Single-authority split:** T-4 owns the per-language **model facts** under test; **T-19** owns corpus **emission** (`testgen_emit_language_behavior_equivalence_claim`; incremental regen is **IRT-2 / T-19**); **T-14** owns the **`test/claim/generated/`** file home (`language_behavior_equivalence.dag` is the receipt pattern); **T-21** owns **incremental re-test selection** (**IRT-1** frontier pruning; **IRT-4** cache reuse for unaffected claims) over that corpus. T-4 does not hand-author generated claims — it closes when testgen rows exist for each language slice. **`[needs T-19, T-21]`** (also reflected in the graph edge above).

**Ingest support by language (scheduled — operator-ratified 2026-05-24):**
Each language model activates bidirectional ingest when its lex/grammar DATA is filled. `dag.dag` is the first-class language — T-6/T-7 fill its lex/grammar schema AND the dag.dag data in the same commit trains. Other languages follow in T-4 Wave 2a (distinct from Wave 2b type-deepening):
```
  dag (dag.dag)      : ingest activated by T-6 + T-7 fill — CRITICAL PATH
  rust (rust.dag)    : lex/grammar shape landed (Wave 1); 🟡 gate dissolves on T-10
                       bidirectional round-trip; Wave 2a = confirm/extend ingest data
  python / go / cpp / typescript : Wave 2a lex/grammar data fill (after T-4 + T-6/T-7 schema)
  format models (json/yaml/csv/toml/sql): these model DATA FORMAT shapes AND their
      syntax grammar. Ingest of format text (e.g. `{"k": 1}` → Node) requires lex/grammar
      data in extdeps/formats/ (same pattern as dag.dag/rust.dag). Grammar wave for
      formats is T-4.6 Wave 2 — scheduled after T-6/T-7 define the LexRules/Grammar schemas.
      `[needs T-6, T-7]`
```
**T-4 Wave 2a** (lex/grammar data per language): extend DATA on each language model after
T-6/T-7 schema lands. Scheduled post-T-6/T-7. `[needs T-6, T-7, T-4]` (T-4 = fact-bundle authoring; Wave 2a adds lex/grammar data on top).
**T-4 Wave 2b** (type deepening): inhabitance + algebra laws + effects + partiality per language.
Scheduled in parallel with Wave 2a. `[needs T-4, T-33, T-2 Node constructors]` — T-4 = fact-bundle authoring; T-2 Node constructors = algebra.dag exports `ordered_ring_node() -> Node` etc., required so `AlgebraInhabitanceDecl.algebra` can be a grounded Node rather than a bridge Symbol atom. Wave 2b CANNOT ship while any bridge-Symbol algebra reference exists in any language file — these fail-closed at T-9 (`infer_algebra_ref_ungrounded`). The bridge symbols in rust.dag (e.g. `rust_model_core_bridge_std_ordered_ring_representable_integer`) are scaffolds that dissolve when T-2 Node constructors land and this wave replaces them.

**Modeling decisions**:
- Per-language primitive **grounding** (fact-bundle, per DECISIONS.md "D2 REVERSAL + FACT-BUNDLE RESEED"): the bundle of spec-read facts for each primitive — width / signedness / representation / overflow disposition / surface spelling — grounding into the shared `std/` vocabulary (T-3). Libraries such as `std::vector` are NOT modeled per L-2 — they are ordinary `Node`s. Deduplicate to a `std/` carrier only on proven identity; never a bare alias, never a re-declared algebra inhabitance (INVARIANTS P1:42)
- Per-language realization cost shape
- Grammar encoding: declarative production data — the **bidirectional relation** (concrete syntax ⟷ Node), read as ingest (partial, many→one, fail-closed off F) and emit (the chosen canonical section); NOT a procedural recognizer. The ingest reading MUST be unambiguous, or ambiguity ⇒ Diagnostic (never "parser picks one" = fabrication). Syntax needing semantic feedback to parse (C++ most-vexing-parse, `<` template-vs-less-than) is a STOP/escalation, not silently absorbed.
- Type system: nominal (Rust, Java) vs structural (TypeScript, Go), or both (C++)

**Reference**:
- v2: `src/v2/languages.dag`
- v3: `dsl/extdeps/languages/` (audit each for honesty)

---

### T-4.5: extdeps/posix.dag + extdeps/file_system.dag  [SUBSTRATE LANDED]

**Status:** Both files landed. `posix.dag` models the POSIX process substrate
(ProcessId/ExitCode/SignalNum via T-25-core refinement substrate — PR #3507;
CapturedOutputPipes stdout/stderr axes, ProcessTable opaque OS resource).
`file_system.dag` models file-system resource + modeled effects (ModeledFileEffects
per Practice 11 companion; legacy FileSystemOperations P5-bridge dissolves when
testgen transitions to ModeledFileEffects). TestClaims in `test/claim/manual/process_numeric_refinements.dag`
and `test/claim/manual/posix_output_capture.dag`.
**File**: 2 files in `src/v4/extdeps/`
**Why bundled**: both are OS-interaction substrate; both are required for v4 to function as a self-hosting compiler (read source files, write emitted files, ExecuteCommand for boundary tests per THESIS facet 3).
**Why anchored**: each file carries a `# Anchor:` to its canonical reference (Wikipedia/POSIX). Reviewers validate the modeling against the reference — no invented vocabulary.

**Modeling decisions (resolved)**:
- `posix.dag`: parent/child via `ProcessIdentity.parent: ProcessId`; signal handling minimal (`SignalNum` + `Termination` sum — no named SIGTERM/SIGKILL constants per T-4.5 modeling decision); pipe capture buffered via `CapturedOutputPipes { stdout, stderr: CapturedByteStream }`.
- `file_system.dag`: `FilesystemPath = Absolute | Relative` (Disj sum); symlink target as `FileKind` discriminant (`FileKindResolutionPolicy = FollowSymlinks | DoNotFollowSymlinks`); read failure modes as opaque `Symbol` reason-references per std/diagnostic.dag.

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

**Role:** Lexical half of generic `ingest` (00_compile B2-OMNI): walker over `LanguageModel` **lex** `LexRules` data — grammar-as-data, not hardcoded `.dag` classes (N×M STOP). Wave-2+ = extend **data**, not walker.

**Merged `00_compile.dag` ingest (literal `Owns` today):** `ingest: (Source, LanguageModel) -> Result<Node, Diagnostic>` — authoritative composed ingest spelling on the orchestrator file (per DECISIONS.md item **I**: `Result<…, Diagnostic>` prose denotes the same fail-closed surface as `Outcome<…>` from `std/diagnostic.dag`; do not “fix” TASKS to one carrier spelling without changing **`00_compile.dag` in the same commit train**).
**Merged `01_tokenize.dag` (literal `Owns` today):** `tokenize(text: String, file: Symbol, rules: LexRules) -> Outcome<TokenStream>` with `LexRules = VoidLexRules | ModeledLexRules { root: LexRuleSet }` and `TokenRule.pattern: LexPattern` (`String` + `file: Symbol` is the concrete source slot inside `ingest` today; read `LexRules` as the lexical projection of the `LanguageModel` bundle per Theme-A #9 — not a second authority).
*Theme-A #9 resolved:* `src/v4/compiler/07_target_carriers.dag` is the
single carrier authority: `type LanguageModel = Node`. Read
**`LexRules` / `Grammar` as lexical and syntax projections** on that
same grammar-as-data `Node`, not as second authorities beside the
conceptual `(Source, LanguageModel)` spelling.

**Modeling decisions**:
- The lexical-rule **data schema** on the `LanguageModel` — what a
  declarative lexical production is, as typed `LexRule` / `LexPattern`
  payload data (the structural shape the walker recognizes by constructor
  + child structure, never by `Symbol` spelling).
- Whitespace/comment handling expressed *in that data*, not as walker logic.
- The walker's structural recognition contract (E0 and successors).

**Reference**:
- merged: `src/v4/compiler/01_tokenize.dag` (B2-OMNI, E0 contract)
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-2

---

### T-7: compiler/02_parse.dag

**Role:** Syntactic half of `ingest`: walker over `Grammar` `Node`; grammar = bidirectional concrete-syntax ⟷ `Node`; parse forward, emit (T-10) inverse (`ingest = emit⁻¹`, C5). G0+ = **data** on model.

**I/O**: `(TokenStream, Grammar) -> Outcome<ParseTree>` — `ParseTree = Node` (A1); matches **`compiler/02_parse.dag` `Owns`** (`parse: (TokenStream, Grammar) -> Outcome<ParseTree>`).
*Ingest tie-in (`00_compile.dag` `Owns` today):* composed `ingest` still closes as **`Result<Node, Diagnostic>`** (see T-6); this stage keeps **`Outcome<ParseTree>`** in **`02_parse.dag`** until a ratified rename train retires the split spelling across **`00_compile.dag` + `01_tokenize` + `02_parse` together**.
*Theme-A #9 resolved:* Same projection reading as T-6 — `Grammar` is the
syntax-side projection of the landed `LanguageModel = Node` authority in
`src/v4/compiler/07_target_carriers.dag`.

**Modeling decisions**:
- The grammar **production data schema** as `Node` — a declarative
  production *is* one direction of the bidirectional relation; both
  directions must be expressible from the same data.
- Error recovery (single `Diagnostic` vs continued) — fail-closed.
- That the walker stays generic — no per-language branch in-body.

**Reference**:
- merged: `src/v4/compiler/02_parse.dag` (B2-OMNI, G0 contract)
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-3

---

### T-8: compiler/03_normalize.dag + 03_resolve.dag

**Role:** First two `core` transforms after `ingest` on `00_compile`: **normalize**, then **resolve** (`resolve ∘ normalize` under standard `∘`, matching the composite line below). T-9 **infer** follows on the resolved tree; the full `core` chain on parse output is `infer ∘ resolve ∘ normalize` (never `normalize ∘ resolve ∘ infer`). Causal `Node` transforms — C3 desugar; K-1 resolve; derived facts carried **once** on returned `Node`.

**I/O (pivot truth):** `ParseTree` / `NormalizedTree` / `ResolvedTree` are aliases on the universal **`Node`** pivot (A1); composite normalize→resolve is **`resolve ∘ normalize`** on that pivot (standard `∘`: normalize first, then resolve).
*Merged seam (CP-1b, literal headers today):* `normalize: ParseTree -> Result<NormalizedTree, Diagnostic>`, `resolve: NormalizedTree -> Result<ResolvedTree, Diagnostic>` in `03_normalize.dag` / `03_resolve.dag`.
*Do not drift:* **Carrier is `Node`; the seam types are parse→normalize scaffolding** — keep the header aliases and `Result<…, Diagnostic>` until CP-1b closes; do not delete or flatten signatures early chasing “purity.”

**Modeling decisions**:
- The 4 sugar forms and their dissolution **as structural rewrites on
  `Node`** (C3) — single-authority for the desugared form.
- `Symbol` binding (K-1): the use→def fact is *derived and carried
  forward at the resolve boundary itself*, not supplied out-of-band — the
  resolve stage contract is "identifier binding to declarations".
- **Declared-binding bridge:** resolve currently infers declaration-ness from
  structural position. This dissolves when `DeclaredBinding` lands in
  `std/node.dag` or `std/binding.dag`, `03_normalize.dag` stamps it on
  declaration nodes, and `03_resolve.dag` consumes that typed fact
  exclusively.
- The sugar-name authority is the `LanguageModel`'s, consumed — never
  re-minted in this stage (single-authority, Practice 5).
- **CP-1b bucket C (native `.dag` LM):** `DagLanguageModel` carries
  `canonical_symbols: Set<Symbol>` as declared data; resolve reads that field
  only—no inferring the native C3 prelude `Set` from void lex/grammar shape
  inside `03_resolve.dag` (per CP-1b/T-8 dispatch row above and merged
  T-8 closeout PR #3436).
- **CI emit-wall bridge (tracked):** when Ubicloud SIGTERMs v2 emit after a clean resolve,
  `scripts/v4-bootstrap-resolve-posture-gate.sh` is the sole bridge authority (structured receipt +
  `V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE=1`); dissolves when a typed resolve-only compiler gate
  lands or emit reaches `compiled:` on standard-8 without host SIGTERM.

**Reference**:
- v2: `src/v2/03_normalize.dag`, `src/v2/03_resolve.dag`

---

### T-9: compiler/04_infer.dag

**I/O**: `ResolvedTree -> Result<InferredTree, Diagnostic>`

**This is the file v2 split into 12 files (`04_*`).** v4's discipline: this is ONE file. Pressure to split = substrate design escalation, not a worker decision.

**Modeling decisions**:
- **The coercion fold** (rescoped 2026-05-17, D2-reversal — supersedes "algebra-homomorphism search algorithm"). Coercion is a **mechanical zip-fold** (a catamorphism) over two groundings — not a search, not research. It walks both canonical `Node` groundings in parallel and compares; `node.dag`'s B1-CANON contract `content_hash = merkle_fold ∘ canonical` already specifies the hard half (the canonical-form fold). Per `DECISIONS.md` U1 / C1 / T-9 the Find is **decidable by construction** over the closed declared candidate set — empty ⇒ Diagnostic, never a fabricated coercion. Name it the *coercion fold*; never an "engine" or "search algorithm".
- **The coercion quality tag** *(rescoped 2026-05-20, operator-direct strict amendments — `Lossy` stripped + `Outcome` shape reconciled to ratified Q11 two-variant)*. The coercion result is the ratified `Outcome` carrier (`std/diagnostic.dag`), and **quality and outcome are distinct axes**. A *successful* coercion is `Outcome::Accepted { value, diagnostics: Diagnostics }` carrying the coerced value **plus a closed quality tag**: `Identity` (groundings coincide) | `Exact` (related, total, lossless). **`Lossy` is REMOVED from the quality tag** — a candidate target inhabitant that would lose information is NOT a valid structure-preserving homomorphism; the correct categorical answer is `Outcome::Rejected { diagnostics: NonEmptyDiagnostics }`, not a success-with-warning. Lossy operations exist as **explicit user-declared source operations** (e.g., `floor`, `int_truncate`, `widening_cast`) modeled like any other operation in `std/integer.dag` / `std/float.dag`; they are not a coercion-time category. This eliminates the success-with-warning failure mode at the substrate level (P3 fail-closed enforcement). A coercion that cannot be derived is `Outcome::Rejected { diagnostics: NonEmptyDiagnostics }` where each diagnostic in the list carries a `CoercionMismatchKind` payload: `CoercionMismatchKind = NoTargetCandidate | AmbiguousTargetCandidate | StructuralMismatch | WouldLoseInformation`. Fail-closed is **not** a quality value: it is the `Rejected` branch of `Outcome`. The quality tag attaches only to `Accepted`; never collapse the success-quality axis and the success/failure axis into one flat enum.
- **The composition rule** *(rescoped 2026-05-20 — simplified lattice post-`Lossy` strip)*. When two *successful* coercions compose, their quality tags compose by a closed lattice: `Identity ∘ Identity = Identity`; `Identity ∘ Exact = Exact`; `Exact ∘ Identity = Exact`; `Exact ∘ Exact = Exact`. **No `Lossy` entry in the lattice** — any candidate that would be lossy is `Rejected` before reaching composition. If either coercion is `Rejected`, the composition is `Rejected` — `Outcome` short-circuits on the failure branch (the standard bind), so the failure axis needs no lattice entry. This is the audit's composition lattice in its simplest form; it lives here in T-9, not in a new task.
- `type AlgebraRef { algebra: Node, witness: Node }` — `04_infer.dag`'s `InferredFacts.grounding` carries canonical grounding; `inferred_facts_algebra_ref` / `algebra_ref_from_grounding` project a typed boundary coordinate for algebra inhabitance while the full algebra authority is still pending. Declared here (Theme-A audit #2).
- Cardinality propagation
- Diagnostic precision when inference fails

**Reference**:
- v2: `src/v2/04_*.dag` (12 files — read AS the cautionary tale on substrate inflation)
- v3 L2.5 design: PB-5 infer model (PR #3085)

---

### T-10: compiler/05_emit.dag + compiler/00_compile.dag — emission + orchestrator

**Dependencies:** `[needs T-9, T-4, T-23]` — spine infer/emit inputs plus **T-23** for the Enforce application substrate consumed by `apply_compile_lens` / `run_required_lens_gates` (see lens gate orchestration below; graph edge matches T-24's T-10 consume).

**emit is `ingest` inverted, not a codegen backend.** emit is the **emit
boundary** of the OMNI pivot — `ingest = emit⁻¹` over the **same**
bidirectional relation (00_compile.dag C5). **`TargetModel`** is the
Shape-A emit parameter in **`00_compile.dag`** and in **`05_emit.dag`
`Owns` / B2-OMNI** (`emit: (InferredTree, TargetModel) -> …`). **P2
boundary honesty:** **`05_emit.dag` is not yet internally consistent** — the
frozen scope line still says "target spec" and **`Consumes` still lists
`TargetSpec`**, while the **`Owns` / B2-OMNI / signature** spelling already
moved to **`TargetModel`**. That is **one carrier, dual names in-flight** on
the T-10 scaffold — not a second authority; reconcile the scope prose +
**`Consumes`** line in the same CP-1b/T-10 close-out train that touches
`05_emit.dag`. TASKS tracks the **typed `Owns` I/O** here. Emission *applies
the target language's declarative grammar in the inverse direction* (`Node`
→ concrete syntax). The orchestrator `compile = emit ∘ core ∘ ingest`
composes the three; `run = eval ∘ core ∘ ingest` is the sibling execution
path. The IR is the universal `InferredTree` — there is no "target-agnostic
IR shape" decision, the pivot already is target-agnostic.

**No-templating constraint (operator 2026-05-17).** emit goes **through**
the grounded `TargetModel`'s grammar-as-data, run inverted — **never** a
string template or print routine. A string-templated emit path is the
emit-side D2 hollow alias: an artifact the compiler cannot ground and
coercion-check. The grammar-as-data *is* the emitter. (This constraint
governs T-10's emit boundary as well as T-11's per-target tables — STOP
if any emission step cannot be expressed as inverse grammar-data.)

**I/O**:
- `emit: (InferredTree, TargetModel) -> Result<TargetSource, Diagnostic>` — the emit
  boundary, the U1 Realize phase, inverse of `ingest` (matches `00_compile.dag` and
  `05_emit.dag` **`Owns`** contract headers; see the `TargetSpec` / scope-line caveat above).
- `compile: (Source, TargetModel) -> Result<TargetSource, Diagnostic>` — the orchestrator,
  `emit ∘ core ∘ ingest`.

**`Result` vs `Outcome` (literal alignment, api-review):** **Ground in merged headers, not TASKS invention:** `compiler/00_compile.dag` **`Owns`** spells **`Result<…, Diagnostic>`** for `ingest` / `core` / `emit` / `eval` today; `compiler/01_tokenize.dag` and `compiler/02_parse.dag` **`Owns`** still spell **`Outcome<…>`** on `tokenize` / `parse`. Stage files **import** `Outcome` (and `Diagnostic`) from **`std/diagnostic.dag`**, which **`Owns`** the declared **`Outcome<T>`** carrier — there is no parallel `.dag` `Result<ok, err>` type (per DECISIONS.md item **I**). TASKS quotes **`Result`** here only where **`00_compile.dag` / `05_emit.dag` `Owns`** do (emit/compile bullets above); do not “standardize” orchestrator prose to `Outcome<Source>` **without** changing **`00_compile.dag` in the same commit train**.

**Modeling decisions**:
- How the `TargetModel`'s grammar drives emission **as the inverse walk**
  of the same relation parse (T-7) applies forward — the bidirectional
  relation is authored once, consumed in both directions.
- `TargetModel.authority_source_text` / `*_source_literal` are fixed-point
  anchors only. The bridge dissolves when no emit path reads
  `authority_source_text`, or a P3 gate in `05_emit.dag` / `06_translate.dag`
  blocks reads outside fixed-point contexts.
- The orchestrator as function composition (`emit ∘ core ∘ ingest`),
  fail-closed error propagation on the `Result` / `Rejected` branch (same
  discipline as `Outcome` in `std/diagnostic.dag`).
- `00_compile.dag` `LanguageModel` / `TargetModel`: declare the carrier
  type, or state formally "a model IS a `Node`" (Theme-A audit #9).
- **Lens gate orchestration (operator-ratified 2026-05-25):** `compiler/00_compile.dag` owns **`run_required_lens_gates`**, **`LensGateWitness`**, **`apply_compile_lens`**, and **`validate_then_compile`** — the compile-orchestrator lens verdict surface. T-24 schedules via `LensCiCommand`; T-23 supplies the Enforce application substrate consumed by `apply_compile_lens`. T-24 does **not** re-implement lens pass/fail.

**Tracked scaffolds (this lane):**
- **`feature:W-T-10-mvp1-exact-zip-closure`** — `src/v4/extdeps/languages/rust.dag`
  `rust_mvp1_i32_zip_fold_closure_edges`: MVP-1 Rust add-fn zip-fold coercion
  catalog append to `rust_mvp1_declared_inhabitants_root`. **Dissolve-on:**
  `find_witness` / declared-inhabitants enumeration covers the i32 fact-bundle
  plus emitted add-fn subtree without hand-appended fixture closure edges.
  **Forbidden steady state:** parallel i32 fact-bundle `data` nodes as a second
  coercion authority.
- **`feature:W-T-10-mvp1-inferred-tree-grounding`** —
  `src/v4/test/claim/manual/mvp1_rust_add_translate.dag`
  `mvp1_rust_canonical_grounding_for`: hand-enumerated per-node canonical
  grounding for the MVP emit/translate fixture. **Dissolve-on:** infer-authored
  `InferredTree.facts` replace hand-enumerated MVP fixture grounding.
  **Forbidden steady state:** generic Atom→i32 inhabitant algebra-evidence
  across unrelated nodes.

**Reference**:
- merged: `src/v4/compiler/00_compile.dag` (B2-OMNI), `05_emit.dag` (C5)
- v3 L2.5 design: PB-emit model (`docs/r3-retirement-modeling-emit-rs.md`)

---

### T-11: emit per-target specialization

**Why separate from T-10**: T-10 is the orchestrator; T-11 is the per-target translation tables that populate emit's behavior across **all five Shape-A targets — rust/python/go/cpp/typescript** (matching T-4's language set and the execution-graph critical-path line; T-16 depends on the full set).

**Modeling decisions**:
- Per-target translation rules — **as grammar-as-data, never string templates** (no-templating principle, operator 2026-05-17). The per-target "translation tables" are the declarative bidirectional grammar relation (concrete-syntax ⟷ `Node`, the canonical non-templated form — see T-4 "Grammar encoding"), NOT fill-in-the-holes string templates. A string-template emit path is the emit-side D2 hollow alias: an artifact the compiler cannot ground and check. STOP if a translation rule cannot be expressed as grammar-data.
- **Layout-in-literals bridge:** Rust, Java, TypeScript, Swift, and WASM
  still encode inter-token layout inside `LiteralPattern.text` (for example
  `"fn "`, `" + "`, `" { "`). This is not a terminal lex authority: it
  dissolves when T-6 supplies `TokenLayout` / `TriviaPolicy`, T-11 strips
  layout from token spellings, and `token_sequence_to_source` interleaves
  spellings with layout from that carrier.
- Target-specific optimizations (or absence thereof)

---

### T-12: lens/complexity.dag + lens/cost.dag

**I/O**: `Node -> Witness<ComplexityBound>`, `Node -> Witness<SymbolicCost>`

**Modeling decisions**:
- Complexity class encoding
- SymbolicCost lattice shape (per `docs/audit/sub-value-relation-bounded-lattice-claim.md`)
- Composition with Sum/Product algebra

---

### T-13: lens/{parallelism,effect,ownership,idempotency,structural_resolution}.dag

**I/O**: `(InferredTree, List<DependencyView>) -> Witness<...>` per lens — each
`*_witness(tree, dependencies)` projects over `dependency_lens` output; facts at
usage sites come from `tree.facts.lookup`, not row payload (Practice 11).
`structural_resolution` also exports `at(tree: InferredTree)` for registry/dry-run
entry (wires `dependency_lens(root: tree.root)` internally).

**Modeling decisions per lens** (see file headers).

---

### T-14: test/claim/* + test/fixture/* — TestClaim corpus  [CORPUS FILLED]

**Status:** Corpus filled **PR #3467**. All 6 impossible-bug classes landed
(`suboptimal_complexity`, `idempotency_contract`, `transport_type_drift`,
`nested_optional_flatten`, `unenumerated_effects`, `unhandled_diagnostic_paths`).
`algebra_laws/nat_semiring.dag` and `diagnostic_correction/show_correct_code.dag` filled.
Manual claims in `test/claim/manual/` (connective_anchors, nat_law_anchors, process_numeric_refinements,
refinement_nonempty_list, posix_output_capture, and others). Execution deferred to T-22 runner.
**Files**: `src/v4/test/claim/*` directories (6 impossible_bug + algebra_laws + diagnostic_correction + future categories) + `src/v4/test/fixture/*`
**Operator-ratified additions 2026-05-15**: scaffolds for all 6 R1+R2+ impossible-bug classes already present (`test/claim/impossible_bug/{suboptimal_complexity,idempotency_contract,transport_type_drift,nested_optional_flatten,unenumerated_effects,unhandled_diagnostic_paths}.dag`); diagnostic_correction/ + algebra_laws/ directories ready for fill-in.

**T-4 conformance file home (operator-ratified 2026-05-25):** T-14 owns the **`test/claim/generated/`** corpus directory — including `language_behavior_equivalence.dag` (T-19 emission receipt). T-4 references rows in that corpus as its conformance deliverable; T-14 owns the file tree, T-19 owns generation mechanics.

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

**`BitIdentical` is a property name, not an `AssertKind`** (Theme-A audit, 2026-05-17): the probe's `kind` is `Equals` over the B1 `content_hash` of the two stage outputs — `verification.dag`'s closed `AssertKind` `{Equals, DiagnosticAssert, Compiles, RoundTrips}` is sufficient; **no 5th kind**. The word "BitIdentical" elsewhere in this task denotes that *property*, never a substrate type.

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

**Substrate cross-locks (checkable, not prose-only):** `toml.dag` **TomlDatetime**
value semantics (RFC 3339 / TOML §Date-Time four sub-kinds) dissolve on
**`src/v4/std/datetime.dag`** landing under **T-3** + typed ops wired on **T-4.6**
(`toml_datetime_value`, …) — authority `DECISIONS.md` Part 6 ·
`SL-3229-T4-FORMAT-TOML-DATETIME`.

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
- Hook-as-substrate: **per-arm `ReactHookSite`** (design-r4 §4 Q2 / canvas:76) — each **Hooks-index** built-in is its own coproduct arm with only that call’s signature fields (`ReactHookInlineDependenciesArgument` only where react.dev admits **omit-vs-present** deps; **`UseMemo` / `UseCallback`** carry **required** `dependencies: List<ReactCrossDeclRef>` per pinned signature + design-r4 canvas `List<Reference>`; `ReactOptRef` / `ReactOptValue` / cleanup / refs elsewhere as admitted). Roster matches `react.dag`: **18** stable `use*` APIs under the react.dev **Hooks** index + boundary arm **`CustomHook { implementation_ref: ReactCrossDeclRef }`** (= **19** `ReactHookSite` arms). The separate **`use(resource)`** API (react.dev/reference/react/use — **not** a Hook; placement rules differ, including **loops/conditionals**) is **`ReactUseCallSite`** (`UseResource` arm), **not** folded into `ReactHookSite`, so Rules-of-Hooks / P2 placement consumers can discriminate. See `react.dag` (in-file gate `// 🟡 coproduct dissolution — consumer:react-custom-hook-structural-classifier` on `ReactHookSite`) and `docs/design-r4-phase-1-5-hookkind-custom-react-substrate-canvas.md` §3 (named consumer = a v4 emit or lens pipeline that branches on structural classes of custom hooks before lowering to `Node`; dissolve-on-arrival = same-PR `CustomHook`-arm extension + `TestClaim` row) for R4 Phase‑1.5 Custom posture + design-r4 §12 **🟡** on the whole `ReactHookSite` sum (including `CustomHook`).
- Effect lifecycle: **no** standalone `Mount | Unmount | …` trigger coproduct — phases read only from the three effect arms (`UseEffect` / `UseLayoutEffect` / `UseInsertionEffect`) via **required** `setup_ref: ReactCrossDeclRef` + `ReactHookInlineDependenciesArgument` + `ReactEffectCleanupSite` (react.dev requires the setup function at the call site; design-r4 §4 dissolution receipt; INVARIANTS P1/P2).
- **Authority fork (tracked, non-terminal):** `ReactCrossDeclRef` stands in for `std/node.dag::Symbol` / L1 K-1 only while `v2-compiler` and isolated `compile_to_dag` lack merged `import v4.std.node { Symbol }` (M1(2.7)). **Dissolution** = land the import-merge + bundle alignment in that milestone and retire the fork; do **not** add parallel cross-decl ref carriers elsewhere in extdeps.
- **Optional carriers (P1 nominal split — Practices 2 & 8):** **`ReactOptRef`** = optional **ref-handle** slots (`element.ref`, hook ref operands). **`ReactOptKey`** = optional **`key`** on createElement-returned element objects (not ref semantics). **`ReactOptValue`** = optional **non-ref** hook operands (e.g. `useDeferredValue` `initialValue`, `useActionState` `permalink`, lazy `init`, optional `getServerSnapshot`, `useDebugValue` `format`, `useOptimistic` `reducer`). Each is its own sum with distinct variant/field names over **`ReactCrossDeclRef`** edges so unrelated react.dev facts are not compressed into one “optional ref” placeholder (see inline `// 🟢` marks on `ReactOptKey` / `ReactOptValue` / `ReactOptRef` in `react.dag`).
- **`ReactCreateElementChild` (Practice‑4 🟡; not TS `ReactNode`):** T‑4.7 **partial** `createElement` **children** slice in `react.dag` (`Element` | `Text` only — **not** numbers, arrays, portals, `null`, …). The substrate name **avoids** the TS / react.dev **`ReactNode`** identifier (that name denotes the full child lattice); **`ReactCreateElementChild`** scopes the model to this T‑4.7 partial slice only so consumers are not handed an authoritative-but-incomplete mirror of the lattice. Full lattice is **Owned ELSEWHERE** — promotion / dissolution discipline is recorded here, plus the inline gate `// 🟡 coproduct dissolution — consumer:react-child-lattice-arm-discriminator` on `ReactCreateElementChild` in `react.dag` (Practice 9 per `docs/modeling-discipline.md`: no multi-line Practice prose in the `.dag` file, emoji-tag one-liners only). **Dissolution gate (Practice‑4 / P5 single authority):** named consumer is `react-child-lattice-arm-discriminator` — the first v4 lens or emit pipeline that requires discriminating child arms beyond `Element | Text` (numbers, arrays, portals, `null`, ...); dissolve-on-arrival = that consumer PR extends `ReactCreateElementChild` in the **same commit train** and demonstrates the new arm against a `TestClaim` row in `src/v4/test/claim/manual/` — never a silent “terminal” closure, never a parallel `ReactNode` carrier authored elsewhere.
- **`children: List<ReactCreateElementChild>` (P2 / Practice 3):** `ReactHostElement`, `ReactCompositeElement`, and **`Fragment`** carry **`children`** typed as **`List<ReactCreateElementChild>`** so text vs nested element facts are discriminated **at the field** (single authority at the boundary), not via a second parallel **`List<ReactCrossDeclRef>`** for the same react.dev children slot.
- Rules-of-Hooks discipline (lens-checkable: no Hooks in conditionals — surface as Diagnostic)
- Component composition (props-down, events-up; structural propagation through Node tree)
- Server Components vs Client Components distinction (or unified via effect typing — effects intrinsic to the type signature, not an annotation)

**Scope**: L (large — substrate decisions cascade across full-stack demo T-16)

**Reference**:
- Per-carrier `// Anchor:` URLs in `react.dag` (React **19.2.0** pin; see release + react.dev index)
- `docs/design-r4-full-stack-omni-emission-canvas.md` — 5-Q canvas (consult, do not block)

---

### T-4.8: extdeps/coordination.dag

**File**: `src/v4/extdeps/coordination.dag` (operator-ratified 2026-05-15 IN-B: Bind composition + effect typing — effects intrinsic to the type signature, NOT an annotation layer; NO 6th L1 behavior)
**Why solo**: multi-program coordination is the most consequential effect-typing in v4 — discipline matters.

**Modeling decisions**:
- Endpoint shape (NetworkAddress + LanguageRef + optional FrameworkRef)
- DeploymentUnit = collection of Endpoints + WireContracts between them
- WireContract = typed interface between two endpoints + `WireContractFacts` +
  `CoordinationBind`.
- Drift note (PR #3207): the older `CoordinationSemantics` /
  `HttpEffect` / `QueueEffect` wording is superseded by the decomposed
  `ExchangePattern` + `SettlementGuarantee` + `ConsistencyGuarantee` facts
  on `WireContractFacts`, plus `CoordinationEffectKind` on
  `CoordinationBind`.
- WIRECONTRACT-OBLIGATION-TABLE-T4.8: `CoordinationEffectKind` is tracked over
  `Http` / `Queue` / `Stream` / `PubSub`; each arm has an executable
  `CoordinationEffectObligation` row mapping it to required exchange,
  settlement, and consistency facts. The label bridge dissolves when
  `CoordinationBind` references canonical obligation rows directly.
- Effect-typing: the effect kind is intrinsic to `CoordinationBind`, not a
  separate annotation layer.
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
- **Wave-0 / Practice 9 (header hygiene):** The live `verilog.dag` preamble stays **terse** — path line, the four-line `Scope` / `Owns` / `Consumes` / `Status` block, `// Anchor`, `// Ledger`, and mandated coproduct one-liners only (`docs/modeling-discipline.md` §9). Do not add extra `//` rationale paragraphs to the `.dag` file; falsification narrative lives **here** and under `DECISIONS.md` **L-1**, not as parallel prose authority in the substrate header. **Boundary (P2):** the **authoritative import surface** is the header `Consumes: std/node.dag; std/nat.dag (Nat)` — not the execution-graph tag `[needs T-1, T-2]` (that tag names **substrate schedule prerequisites**: `node` + `algebra` must exist before the probe is fillable). T-2 facts reach Verilog through `std/nat.dag` (which **Consumes** `std/algebra.dag`), not by importing `algebra.dag` directly in this file.
- **Clear win**: one `.dag` FSM → simulable Verilog + a Rust reference model, same Node, zero translator.
- **Owns D3200 dissolution arrival**: `SL-3229-VERILOG-D3200` gates on this task, specifically the Verilog `LanguageModel` axis rework that decomposes `NonTriregNetKind`, `VariableDeclaration`, `OutputPortAnsiVariableTypeKind`, `ParameterTypeKind`, and `PrimitiveGateKind` against their merge-base richer-source axes. This is a T-4.9-owned arrival, not the mainstream T-4 language fact-bundle task.
- **Scope**: L (substrate-validating; concurrency model is the risk).

#### T-4.10: `extdeps/formats/spice.dag`
- **Stress axis**: is the format/`LanguageModel` abstraction *actually* general, or secretly programming-language-shaped? A SPICE netlist has **no control flow** — components + a connection graph.
- **Clear win**: one `.dag` circuit declaration → a SPICE netlist that simulates (omni-emission reaches analog hardware).
- **Placement (operator-ratified fork)**: `extdeps/formats/` — a netlist is a data format, not a programming language (sibling of csv/json), Shape B.
- **Scope**: M-L. **Status**: LANDED via PR #3168 (`src/v4/extdeps/formats/spice.dag`); any pre-D2-reversal / pre-Practice-10-A1 fact-bundle rework stays gated by the A1-invariant decision and is not Wave-0 fill for this already-landed probe.

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
- **PTX path (operator-ratified / evidenced):** model **PTX** (the spec'd IR — clean, general, captures SIMT directly, parallel to `llvm_ir.dag`). The CUDA-C++-as-`cpp.dag` extension alternative is the **rejected** fork (entangled; the C++ surface is not where the stress is). **IN-B probe receipt** is **DECISIONS.md L-3** (PTX falsification posture; `ptx.dag` file header is intentionally domain-neutral). **Derived scheduling / completion state** for this slice is authoritative in **`docs/briefs/r4-program-dispatch-plan.md` §2** (Status column: **PASS (IN-B)**; **Blocked-on** `LANDED` — table-accuracy with main; *LANDED* ≠ every TASKS Scope:L tail dissolved — see that row’s rework-obligation / keystone blast-radius notes in the dispatch plan).
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

**T-4 conformance owner (operator-ratified 2026-05-25):** T-19 owns **`LanguageBehaviorEquivalence` corpus emission** — `testgen_emit_language_behavior_equivalence_claim` writes rows into `test/claim/generated/language_behavior_equivalence.dag` (file home: T-14). T-4's per-language conformance deliverable closes when testgen rows exist for each language slice; T-4 does not hand-author generated claims.

**Modeling decisions**:
- Generator<C> generic carrier shape — one lens, parameterized over substrate concept type
- Per-substrate-kind testgen rules — closed six-way **`TestgenConcept`** in `lens/testgen.dag` (variants `TypeConstruction` / `AlgebraLaw` / `DiagnosticExhaustiveness` / `LensApplicability` / `BidirectionalRoundtrip` / `LanguageBehaviorEquivalence`; LBE pairs `TypeConstructionSubject` × target `language: Symbol` with `FrozenLanguageBehaviorSnapshot` + `LanguageBehaviorIoMock` I/O mock; generated corpus in `test/claim/generated/language_behavior_equivalence.dag` runs through `run_test_claim` / `run_test_claim_assert`; CI gate `scripts/check_testgen_activation.py`)
- **TestClaim.classification** (`TestClassification`: Tier×Layer) on every produced claim — canonical field in `std/verification.dag` (STRUCTURE §248); testgen stamps the same axes on each emitted claim as on its scheduling `Generator<C>`. Tier1/2/3 (correctness) × Unit/Integration/Boundary (test layer)
- Bootstrap path: hand-authored TestClaims in `test/claim/manual/` are the contract testgen must satisfy; coverage lens (T-18) enforces produced ⊇ manual

**Incremental Re-Test requirement set — held (IRT-2, IRT-3; see T-21 for the full IRT-1..4 set + rationale).**
- **IRT-2.** testgen MUST be incremental: on a change, fold only the affected subgraph and regenerate TestClaims for affected nodes — never re-fold the whole corpus.
- **IRT-3 (with T-22).** Preserve arbitrary-node TestClaim-input granularity: a TestClaim's `input: Node` may bind ANY node subgraph (incl. `program ∘ generated-input`); testgen must not restrict generated inputs to whole-function / whole-module units.

**Scope**: L (large — substrate-traversal across every concept; cross-cutting consumption)

**Bootstrap pragma** (per operator: "manual authoring is fine as well"):
- After T-1 (`std/node.dag`) lands: hand-author 5-10 TestClaims in `test/claim/manual/` covering type-construction for the 6 connectives + 5 behaviors. Validates schema + shape immediately.
- After T-2 (`std/algebra.dag`) lands: hand-author algebra-law TestClaims for at least Magma/Monoid.
- After T-19 implementation: testgen produces same set programmatically; manual claims become regression anchors.

**Phase-1.5 scaffolding — forward dissolution (INVARIANTS P2)**:
- **`manual_anchor_manifest.dag` — P2 join (single authority):** `ManualAnchorKey` is defined **once** in `std/verification.dag` (closed 12 live anchors + **`ManualAnchorAbsent`** for claims outside this set). Manifest rows and manual **`TestClaim`** literals use the **same discriminant values** — mechanical join is **tag equality** on `ManualAnchorKey` (no parallel `String` slug tables). **`TestClaim.classification`** and **`TestClaim.kind`** remain sole authority for tier×layer and assert form. Testgen scheduling arm (`type_construction` vs `algebra_law` …) is implied by variant name prefix until M2 can read claims or carry a single non-duplicated discriminant if needed.
- **`TestClaimCoproductVariant` — 🟡 `feature:testclaim-coproduct-reflection` gate (coproduct-exhaustiveness):** the current generated `DiagnosticExhaustiveness` slice needs a typed `omitted_variant` key, but v4 cannot yet project the arm-key set directly from the canonical `TestClaim` coproduct. This is a tracked T-19 bridge, not a terminal source of truth. **Owning follow-up:** land the T-19 coproduct-reflection primitive/consumer that reads the `TestClaim` variant set structurally and emits per-arm generated TestClaims. **Dissolve-on-arrival:** in that same follow-up, delete `TestClaimCoproductVariant`, make `GeneratedCoproductExhaustiveness` consume the reflected arm key, and keep the generated corpus witness proving every reflected arm schedules/emits from the canonical `TestClaim` type.
- **`ClaimAnchorKey` — 🟡 `feature:t19-claim-anchor-split` gate (claim-anchor-unification):** the generated corpus shares `TestClaim.anchor` with manual claim rows; a single typed union (`ManualClaimAnchor | GeneratedClaimAnchor`) bridges this because .dag cannot yet split the field per-corpus. This is a tracked T-19 bridge, not a terminal source of truth. **Owning follow-up:** when T-19 Phase-2 defines separate corpus types for generated vs manual `TestClaim` rows (so `anchor` is no longer a shared field), the union dissolves. **Dissolve-on-arrival:** in the PR that separates manual/generated claim types, delete `ClaimAnchorKey`, `ManualClaimAnchor`, and `GeneratedClaimAnchor` wrappers, and update all `TestClaim` field declarations accordingly.
- **`TestgenOracleBasis` — 🟡 `feature:t19-generator-oracle-basis-carrier` gate (wishlist-dispatch oracle basis):** the wishlist dispatch ledger needs a typed oracle-basis witness while some `TestgenConcept` arms are still pending generated corpora, but this witness must not become a parallel category authority. **Owning follow-up:** the T-19 follow-up that replaces each pending wishlist arm with its category-specific generated corpus owns the carrier split. **Dissolve-on-arrival:** delete `TestgenOracleBasis` from `testgen_category_wishlist.dag` when those rows are replaced, and carry oracle basis through the concrete generator helper or emitted `TestClaim` shape for each landed category.
- **Nat algebra-law expression nodes — 🟡 `feature:t19-nat-expression-node-encoding` gate:** the generated AlgebraLaw Nat corpus must not export or consume per-operation/per-value mirror symbols from `v4.std.nat`; sample terms route through canonical `Nat` constructors and `nat_add` / `nat_mul` results. The remaining local expression-shape tags in `algebra_law_conformance.dag` are a bounded T-19 bridge until a canonical Nat-expression `Node` encoder can reflect function application identity directly. **Dissolve-on-arrival:** replace those local tags with the canonical encoder and keep the generated corpus ratchet proving it uses the modeled Nat constructors/functions.
- **AlgebraLaw Nat corpus receipt (this PR):** `testgen_category_wishlist.dag` records dispatch; `algebra_law_conformance.dag` is the generated corpus; v3 hand-Rust integration tests in `v4_test_bootstrap_infra_closeout_test.rs` assert corpus parse + oracle rows. SG-0 path: Rust test assertions at integration-test band, no new hand-Rust primitive (corpus is `.dag`-only; Rust side is test infrastructure, not a new v2/v3 primitive).
- **`BehaviorValueSubject` (L1 `Value` in `type_construction`):** `BehaviorValueSubject { behavior: Behavior }` in `lens/testgen.dag` — carries the **five-behavior** axis structurally (pairs with `ConnectiveKernel { connective: Connective }` for the **six connectives** per bootstrap pragma above). **No** live manual `TestClaim` yet ⇒ **no** manifest row until one lands (same P2 bijection rule).
- **Retirement (same band as M2 unblock):** fold manifest membership into reflection / `Symbol` spellings once M2 cross-file loading + literal validation land; never reintroduce parallel `Int` encodings of axes already on `TestClaim`. **`ManualAnchorAbsent`:** use on **`TestClaim`** rows not in the twelve-anchor manifest until a broader substrate generalizes membership.
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
- **L6 (THESIS L6 / §181):** `L6CoverageKey` is **`{ subject: TypeConstructionSubject, language: Symbol }`** — form × **target language identity** (not form-only). Canonical **gunbc `.dag`** surface token: **`dag_language_model_surface_id`** in `extdeps/languages/dag.dag` (K-1); other `extdeps/languages/*` models add their own `Symbol` anchors when L6 expands past language #1.
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

**T-4 conformance consumer (operator-ratified 2026-05-25):** T-21 owns **incremental re-test selection** (**IRT-1** — `test_claim_evaluation_touches_rerun_frontier` / `ci_select_from_affected_set` name the rerun frontier; **IRT-4** — unaffected claims reuse cached results) over T-4's `LanguageBehaviorEquivalence` corpus. T-21 does not emit claims (**IRT-2** remains T-19 testgen-only); T-19 owns emission (see T-19 detail).

**Modeling decisions**:
- `affected_set: (Dag, Diff) -> Witness<ReExecFrontier>` shape
- Diff representation (file-set? node-set? structural-delta over the Dag?)
- Purity-aware skipping: an unchanged pure subgraph is incrementally skippable; what makes a subgraph "unchanged" structurally?
- Composition with `compiler/05_eval.dag` (skip) and `workflow/ci.dag` (job selection)
- Structural caching is the **dual** of the affected set — the same mechanism. A build/exec artifact's cache key is `content_hash` (B1) of its input subgraph: the affected set names what re-runs, a cache restores what doesn't. Caching is not a separate system. The cache backend (GHA `actions/cache`, a remote build cache, a local memo table) is just an emission target of the hash.

**Incremental Re-Test requirement set — IRT-1..4 (operator directive, briansrls, 2026-05-17; held at author + review time).** The architecture supports precise, proven, fast incremental re-testing — a change re-runs only the proven-affected TestClaims, in parallel, the rest from cache. A naive scaffold-fill silently loses this. Four held requirements, anchored across T-19/T-21/T-22, cross-referencing each other:
- **IRT-1 (affected_set — this task).** The `read` body MUST prune at unchanged/pure boundaries so cost is O(affected-region), never O(whole-program); reuse cached `content_hash`es (O(1) change-check per subgraph); be a parallel frontier-expansion (a pure fold over the DAG). Analysis cost scales with the *change* size, not the program size.
- **IRT-2 (testgen — T-19).** Incremental: fold the affected subgraph, regenerate TestClaims only for affected nodes; never re-fold the whole corpus on a change.
- **IRT-3 (TestClaim input granularity — T-22 + T-19).** Arbitrary-node granularity preserved: `eval` must evaluate ANY node subgraph bound into a TestClaim's `input: Node` (incl. `program ∘ generated-input`); never silently restrict to whole-function / whole-module units.
- **IRT-4 (result caching — this task + T-24).** A TestClaim's result is cached keyed by the `content_hash` of the **whole TestClaim node** — its complete evaluation subgraph, which transitively includes the `input` field AND the oracle (predicate/`AssertKind`/`expected`), evaluator, resources, and extdeps. Keying by the `input` subgraph alone violates P2 Boundary Discipline — the cache-key boundary must carry every fact the result depends on — and would reuse a stale result when the oracle changes while the input is unchanged. Because B1 `content_hash` is a merkle fold over the canonical `Node`, hashing the TestClaim node naturally covers all of it. Unaffected TestClaims' results are REUSED, not recomputed — the concrete form of "caching is the dual of the affected set" (modeling decision above).

*Rationale.* These four are what make "affected-set + node-level TestClaim + content-hash caching ⇒ a one-line change re-runs only the handful of proven-downstream-affected tests, in parallel" real. The minimality is *proven* — excluded tests carry `ExclusionReceipt`s — so "affected tests pass ⇒ the change is coherent downstream" is sound, not heuristic.

**Scope**: L (large — load-bearing for incremental execution + CI dissolution)

**Reference**: THESIS §205-210 free consequences (incremental cross-run) + v4-close-interrogation.md §2.5.F + memory: feedback_no_textual_enforcement_bridges

---

### T-22: compiler/05_eval.dag — the interpreter (PRIMARY execution path)

**File**: `src/v4/compiler/05_eval.dag` (operator-raised 2026-05-15: "what about the interpreter")
**Why load-bearing**: THESIS:225 — `dag run` is THE primary execution path. eval is not an afterthought to emit; it is the default. Sibling of `05_emit.dag` (same `InferredTree` input; eval executes, emit projects to target languages).

**Modeling decisions**:
- `eval: (InferredTree, runtime carriers, Inputs) -> Result<RuntimeValue, Diagnostic>` shape (Option C supersession 2026-05-21 — eval interprets primitives against decomposed runtime carriers plus a concrete runtime extdep; see T-34)
- Bounded-execution enforcement (INVARIANTS P4 — no unbounded loops; how does the evaluator structurally refuse non-termination?)
- The shared substrate three consumers compose over: `workflow/bootstrap.dag` (interpreted, not compiled), TestClaim evaluation, lens dry-run
- Concept-unification (THESIS:188): under Option C, eval reads **runtime carriers** (T-34) for primitive interpretation, execution semantics, and runtime value representation — NOT LanguageModel. LanguageModel is for ingest grammar + emit serialization. Concrete runtimes live under `extdeps/runtimes/` and consume `ModelCore` (T-33) for primitive-type / algebra-inhabitance / laws / effect / partiality facts. The earlier "eval reads the same `extdeps/languages/*.dag` carriers emit does" framing predates Q1 and is superseded
- **`BehaviorValueSubject` naming vs eval slice (T-19 → T-22):** The identifier anchors the T-19 **L1 `Value`** placement under `type_construction` (T-19 Phase-1.5); the payload is still the **closed** `Behavior` sum (all five behaviors—not Value-only structurally). When T-22 binds testgen/type-construction to execution, re-audit the name with the real consumer: either keep it as the Value-slice carrier of `Behavior` facts or rename (e.g. to `BehaviorSubject`) if the eval path is behavior-wide with no residual Value reading; fold the decision into the T-22 close gate so names cannot silently drift from semantics.

**Incremental Re-Test requirement set — held (IRT-3; see T-21 for the full IRT-1..4 set + rationale).**
- **IRT-3 (with T-19).** `eval` MUST evaluate ANY node subgraph bound into a TestClaim's `input: Node` — including a `program ∘ generated-input` composite — at arbitrary-node granularity; it must never silently restrict TestClaim evaluation to whole-function / whole-module units. Node-level evaluability is what lets the affected set (T-21) name re-runs at node precision.

**Bilateral binding — #3213 negative-coverage obligation (DECISIONS.md LB-T22-3213).** ci.dag's `ci_pipeline_well_formed` enforces bootstrap-stage single-authority fail-closed by construction (`ci_all_commands_authority_ok` / `bootstrap_stage_output`). T-22 Wave-0 now carries the executable demonstration in `test/claim/workflow/pipeline_rejections.dag`: six workflow-pipeline `TestClaimRun<CiPipeline>` rows cover dangling `BootstrapStageCompile.produces`, duplicate job/gate id, dangling `needs`, dangling gate job, and dependency cycle, and the paired pass-gate rows pass each run through `workflow_pipeline_pass_gate` so non-`Pass` verdicts become modeled non-pass coverage verdicts carrying the full run fact. Remaining yellow scope is the source-side plan-bound receipt plus the typed-input bridge until real B1 `content_hash` facts and Node projection / typed TestClaim input facts replace the placeholder cache hashes and make the pipeline subject a first-class `TestClaim.input` subgraph. This binding is bilateral with DECISIONS.md `LB-T22-3213`, `T22-EVAL-TYPED-INPUT-BRIDGE`, and `T22-EVAL-PASS-RECEIPT`.

**Scope**: XL (extra-large — THE primary execution path; bootstrap + tests + dry-run all depend on it)

**Reference**: THESIS:225 + concept-unification THESIS:188 + STRUCTURE.md §"Bootstrap chain" (v2's eval seeds; v4's eval takes over)

---

### T-23: lens/application.dag — apply_lens surface (opt-in depth)

**File**: `src/v4/lens/application.dag` (closes prior-audit BLOCKING GAP 1)
**Why load-bearing**: `apply_lens(<lens>, Enforce { ... })` is referenced by `report.dag`, `synthesis.dag`, and the C7 advisory→blocking bridge — but had no substrate home until now. It is simultaneously: §1.5 user-defined-dimensions surface, §6.2 audience-duality opt-in-depth mechanism, and the ONLY advisory→fail-closed path.

**Compile lens enforcement consumer (operator-ratified 2026-05-25):** T-23 owns the **`apply_lens` / Enforce** application substrate. `compiler/00_compile.dag` **`apply_compile_lens`** (T-10) invokes lenses in **`CompileLensEnforce`** mode — consuming T-23's advisory→fail-closed bridge. T-10 owns orchestrator wiring; T-23 owns lens application semantics.

**Modeling decisions**:
- `EnforcedApplication<Output, Budget, Projected>` (references `EnforceableLens<Output, Budget, Projected>` — bundled lens + enforcement) vs `IntrospectApplication<Output>` carrier shapes (v3 T-Lens-Application-Surface precedent: **two separate top-level carriers**, NOT a sum — per `docs/design-lens-application-surface.md` §2 + `src/v4/DECISIONS.md` Part 4 **T-23-PIN**; the historical two-parameter `EnforcedApplication<Output, Budget>` sketch is **retracted** here)
- `SectionRef = DeclarationScope | NodeScope` (where a lens attaches)
- The advisory→fail-closed conversion: how `Enforce { }` turns a lens's `Set<Report>` into fail-closed Diagnostics (the single explicit bridge per `std/report.dag` discipline)
- Default policy: a function with no `apply_lens(<lens>, Enforce { ... })` declaration gets synthesized Introspect-only (no implicit Enforce) per THESIS:307-321 opt-in depth. `apply_lens` is a first-class declaration (a Node), not an annotation — absence of the declaration, not absence of a tag, is the default trigger.

**Scope**: L (large — connective tissue for three thesis claims)

**Reference**: THESIS:95-101 + THESIS:307-321 + r3-structure.md:40 (v3 precedent) + std/report.dag discipline

---

### T-24: workflow/ci.dag — CI pipeline AS DATA

**File**: `src/v4/workflow/ci.dag` (closes prior-audit BLOCKING GAP 2)
**Why load-bearing**: THESIS:223-226 — "adding a CI gate = editing one .dag file." v3's gate #98 `ci_yml_hand_authority_dissolved` was an open R3 gap precisely because CI YAML stayed hand-authored. v4 must not reproduce it.

**Dependencies:** `[needs T-21, T-20, T-10, T-23]` — T-21 for affected-set job/claim selection; T-20 for bootstrap interaction; **T-10 + T-23** for lens verdict (T-24 schedules; T-10 owns `run_required_lens_gates`; T-23 owns Enforce application substrate).

**Modeling decisions**:
- `CiPipeline { jobs, gates }` shape
- `.github/workflows/ci.yml` as DERIVED Shape-B artifact (.dag walks CiPipeline, emits YAML)
- **CI/YAML authority bridge:** T-24 is not closed while committed YAML and
  v3 string ratchets can act as parallel authorities. It dissolves when the
  generator emits checked YAML from `ci.dag`, the hand-authored YAML is
  deleted, and the v3 string ratchets become `TestClaim`s over the generated
  output.
- Affected-set-driven job selection consuming `lens/affected_set.dag` (T-21) — this is what dissolves `scripts/detect-affected-components.sh`
- **Lens verdict gate (operator-ratified 2026-05-25) — single-authority split:** CI pass/fail for lens enforcement is **`run_required_lens_gates`** in `compiler/00_compile.dag` (`Outcome<List<LensGateWitness>>`) — **owned by T-10**, not build exit code alone. **T-24** owns **`LensCiCommand { required_lenses: … }`** scheduling in `workflow/ci.dag` (which lens set runs in CI). **T-23** owns the Enforce application substrate consumed by T-10's `apply_compile_lens`. Semantic verdict = lens-gate rejection diagnostics, structurally separate from "compile succeeded."
- Structural cache keys: a cacheable job's `actions/cache` key is `content_hash` (B1) of its input subgraph, not a hand-authored `hashFiles(...)` glob. The interim `hashFiles(...)` keys in the committed `ci.yml` (e.g. the v2-compiler-binary cache) are manual approximations, replaced by emitted content-hashes when `ci.yml` is emitted from this file.
- The bootstrap interaction: CI runs `workflow/bootstrap.dag` (T-20)

**Scope**: L (large — closes the v3 hand-authored-CI gap; dissolves the shell bridge)

**Reference**: THESIS:223-226 + v4-close-interrogation.md §3.2 + v3 gate #98 (the gap not to reproduce)

---

## Watch — `docs/audit/coproduct-anemia-inventory.md` (v4 coproduct sum census)

**P5 dissolution (repo-checkable).** This row is the **stable** completion gate for the inventory artifact — not a dashboard session handle.

**Closes when all hold:**

1. `docs/audit/coproduct-anemia-inventory.md` enumerates **274** corpus sums; the **full** corpus table carries **no** `DEFERRED` placeholder in **grounding-axis columns (A)(B)(C)** (columns **8–10**) for any row, and **no** `DEFERRED` in **coproduct-modeling columns 4–7** for any row — every row has substantive hand-authored modeling notes read off the cited `.dag` fragment in cols **4–7**, per the Method bar in that file.
2. The inventory’s **Method** and **Summary** sections describe completion in terms of **this structural condition** (and, where useful, stable intent in `ROADMAP.md` / `MODELING.md` / `INVARIANTS.md` plus the **no parallel comment-ledgers** standing rule in `docs/modeling-discipline.md` — **not** a resurrected standalone `DECISIONS.md` path), not a session nickname as the sole dissolution trigger.
3. The landing PR updates **this bullet** with the merge SHA (or the operator folds the obligation elsewhere and edits this section accordingly).

Until (1)–(3): the partial table with `DEFERRED` cells is **honest interim state**; reviewers use the checkpoint batch + exemplars as the depth bar, not as “done.”

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
- **#2 `AlgebraRef`** — `04_infer.dag`'s `InferredFacts.grounding` plus
  `algebra_ref_from_grounding` names `AlgebraRef`; `std/algebra.dag` declares no such type yet. It is a
  typed boundary coordinate over the algebra and witness nodes until the
  full algebra inhabitance authority lands. Disposition: T-9 declares the
  carrier — a clarification in existing T-9 scope, not a new task.

**New PROPOSED tasks (the "missing substrate" Theme-A gaps):**

### T-25 — std/ value-predicate refinement substrate  [SUBSTRATE LANDED]
**Status:** Both T-25 components are landed on `main`.
- **T-25-core** — `src/v4/std/refinement.dag` (`Validation<B>`, `Refined<B>`, `refine`, `refined_base`).
  Landed **PR #3354**. Consumed by posix.dag (ProcessId/ExitCode/SignalNum per PR #3507).
  **Approach (operator-ratified 2026-05-25):** Strengthen the home type before reaching for a refinement. `posix.dag` ProcessId/ExitCode is the reference pattern: `ProcessId` is a nominal wrapper (`{ refined: Refined<Int> }`) with a `value > 0` lower bound from the POSIX spec; `ExitCode` adds a `0–255` upper bound. Neither carries width or signedness fields in the current substrate — those would be the *strengthening* step a future T-4 home-type upgrade would add. The principle: use `Refined<B>` for externally-specified boundary values only (POSIX defines these bounds, not the modeler). A refinement on an under-modeled home type does not fix the home; it adds a second layer of opacity over the gap.
- **T-25-tail** — predicate prover (`constraint_satisfaction` + `exact_structural_equality_zip_fold`
  semantics) in `src/v4/std/find_witness.dag` and `src/v4/std/constraint_satisfaction_predicate.dag`;
  dissolves identity-MVP scaffolds. Landed **PR #3531**.
**Residual (not T-25):** RFC 3986 validated-component refinements in
`src/v4/std/network.dag` are now **unblocked** (T-25-core gate open); authoring is
follow-on to T-26 (`feature:network-validated-components`), not gated on T-25-core.
`NonEmptyList` witness in `src/v4/test/claim/manual/refinement_nonempty_list.dag`
(acceptance witness, T-22 exec) is similarly unblocked.
**Independent sub-bug (resolved):** `file_system.dag` header dangling-`Consumes`
(cited `std/collection NonEmptyList` before T-25-core landed) is corrected — the header
no longer cites a non-existent type.

### T-26 — std/ boundary carriers (net-address / URL / HttpMethod)  [SUBSTRATE LANDED]
**Operator ruling 2026-05-17 — disposition unchanged (no fork); authority lives in `std/`.**
**Status:** `src/v4/std/network.dag` is the v4 **single authority** for `HttpMethod`,
structured RFC 3986 URI carriers (`Url`, `UriReference`, …), and
`NetworkAddress { authority: UriAuthority }`. `extdeps/coordination.dag` and
`extdeps/formats/openapi.dag` consume this module per M9 / DECISIONS Part 1
(`std/network.dag` rows + coordination `NetworkAddress` dissolution row).
**Residual (not T-26):** RFC 3986 validated-component refinements in `std/network.dag`
are now unblocked (T-25-core gate open); tracked as `feature:network-validated-components`
(T-26 follow-on). OpenAPI path verbs stay
`OpenApiHttpMethod` (OAS eight-verb closed set vs broader `HttpMethod`) per
DECISIONS **T-4.6-P4-OpenApiHttpMethod**.

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

### T-28-B — Extract module graph admission from `03_resolve.dag`  [SCHEDULED]
**Gap:** `03_resolve.dag` currently exposes `resolve_with_graph` /
`namespace_from_tree_and_graph`, so the K-1 resolver has a
`ModuleGraph`-shaped cross-file surface even though it does not load files
and the graph path remains gated. This keeps module admission policy in
the resolver layer.
**Disposition — SCHEDULED (T-28 follow-up, bundled with T-8).** Move
graph-to-namespace projection into a separate module-resolution stage that
consumes `std/module_graph.dag` and calls
`compiler/03_resolve.resolve_with_namespace`. `03_resolve.dag` remains
single-tree K-1 resolution only: `resolve(tree, lm)` and
`resolve_with_namespace(tree, namespace)`.
**Boundary:** the new stage receives the fully loaded `ModuleGraph` plus
the subject `ModulePath` / tree, enforces import / visibility / ambiguity
rules, and produces the exact `Namespace` admitted for that subject module.
It must not flat-fold `graph.entries`; module paths remain authoritative
until admission is complete.
**Move out of `03_resolve.dag`:** `ModuleGraph` import,
`namespace_from_tree_and_graph`, `resolve_with_graph`, and header
ownership / consume claims for `ModuleGraph`.
**Dissolve gate:** once the external stage owns module admission and emits
a `Namespace`, delete the T-28 graph gate from `03_resolve.dag`;
cross-file resolution enters through the new stage, not through a third
`Scope` arm or a resolver-local graph fold.

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

### T-30 — std/ structural fact-density / hollow-alias gate  [SUBSTRATE LANDED]
**Status:** Landed **PR #3359**. `src/v4/lens/fact_density.dag` is the substrate authority
(`carrier_spec_fact`, `SourceSpecReadFact`, kernel-ambient exemption). Hand-Rust bootstrap
mirror at `src/v3/compiler/src/v4_hollow_alias_gate.rs` (P5(b) interim; dissolves when
generated `.dag` checker runs during bootstrap). TestClaims in `src/v4/test/claim/lens_fact_density/`.
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
**Mechanism (operator-ratified 2026-05-25):** The T-30 hollow-alias gate runs as a lens verdict via `run_required_lens_gates` (affected-lens `TestClaim` driven by T-21 affected-set frontier), **not** as a separate lint tool or a `ci.dag` build step. The lens verdict IS the enforcement gate; `cargo build` success is the M1 floor minimum, not the T-30 authority.

**Interim bootstrap mirror (not the final generated checker).** The north
star remains a **generated** structural `Node → Outcome` gate in v4
`.dag` authority. While the v4→v3 `compile_to_dag` bootstrap cannot yet
prepend `std/node.dag` without name collisions (see `DECISIONS.md` T-30
`fact_density.dag` encoding note), the Practice-8 predicate lands as the
**P5(b) bounded pattern**: (1) a **hand Rust mirror**
(`src/v3/compiler/src/v4_hollow_alias_gate.rs` + hermetic unit tests), (2)
a **body-less nominal witness** in `src/v4/std/fact_density.dag`, and (3) a
**retired `compile_to_dag` smoke** harness (`v4_std_fact_density_dag_smoke_test.rs`,
**dissolved** with **#3338**; see INVARIANTS §P5(b) / `v4:` bootstrap gate).
**`EXPECTED_HAND_AUTHORED_{NON_TEST,TEST}`** literals and matching
**`INVARIANTS.md` §P5(b)** rows under **T-PB-A** / **T-PB-B** name the
**dissolution** trigger when the generated checker is the authority; the
dissolved smoke harness is **not** reintroduced as an interim ratchet. The
**`.dag` witness (2)** is a **P2-staging nominal** at `src/v4/std/fact_density.dag` (INVARIANTS §P2 / Practice 5: **parse/compile proof only** — **no** generated `.dag` consumer yet; **not** a landed substrate primitive alongside `node.dag` / `diagnostic.dag`). It is **enumerated honestly** in **`STRUCTURE.md`** (separate from the 14 landed `std/` primitives) and is **named** in the same **§P5(b) rows** (and `DECISIONS.md`) as the T-30 witness file — it is **not** a third **SG-0 hand-Rust** census path (SG-0 inventories **Rust** only). This slice is **sequencing**,
not semantic deferral of the gate's *definition* (the three-part rule +
kernel-ambient exemption is already pinned in tests and docs).

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
- **#9 — `LanguageModel` / `TargetModel` named type — RESOLVED.**
  `src/v4/compiler/07_target_carriers.dag` is the single carrier
  authority: `type LanguageModel = Node`, with `TargetModel` owned there
  as the target bundle. The earlier open-item fork ("declare the carrier"
  vs "a model IS a `Node`") is settled as the latter through the named
  carrier alias, so downstream T-32 work must consume this authority
  rather than reopen Theme-A #9.
- **#12 — ExecuteCommand TestClaims.** THESIS facet 3 names
  `ExecuteCommand`-based `TestClaim`s; v4 models the boundary via a
  simulator `Node` + the closed 4 `AssertKind`s. Disposition:
  confirm-only — T-19/T-14 verify `ExecuteCommand`-shaped TestClaims are
  expressible via `posix.dag` + `eval` with no lost predicate surface
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

### T-33 — std/model_core.dag — shared substrate factoring  [SCHEDULED]
**Operator-ratified 2026-05-21 (Option C runtime split).** The shared base
substrate that both `LanguageModel` (T-4) and concrete runtime extdeps
(T-34) consume. `ModelCore` is the categorical floor for primitive-type
and algebra-inhabitance declarations to stay consistent across language
and runtime targets without duplicate authority.

**Dependencies — `[needs T-1, T-2, T-3]`.** Numeric and string vocabulary
(T-3 scalar/numeric stack); algebra carriers (T-2); the Node substrate
root (T-1). Low-dependency — no upstream-feeder watch items — but a hard
T-4 / T-34 prerequisite per the side-branch graph at the top of this
file.

**File:** `src/v4/std/model_core.dag` — does not exist on main; this task
is its first authoring.

**Carrier shape (per `docs/design-v4-compiler-homomorphism.md` §"`ModelCore`"):**
- **Primitive types** — each a Practice-8 fact-bundle (width, signedness,
  range, encoding, …) grounding in the T-3 shared-fact vocabulary.
- **Algebra inhabitance declarations** — which primitives inhabit which
  algebras (consumed by the T-9 coercion fold).
- **Laws / proof obligations** — associativity, commutativity, etc., for
  the coercion fold's preservation checks.
- **Effect semantics** — what effects each primitive operation declares
  (Open Q10 in the design doc).
- **Partiality semantics** — which operations are partial; how partiality
  is discharged or surfaced.

**Why a sibling of T-3, not a fold-in.** T-3 already owns the shared-fact
vocabulary (Signedness / Representation / the numeric stack); ModelCore is
one layer up — the named carrier that *bundles* a primitive's facts +
inhabitance + laws + effects under a single substrate type. It consumes
T-3 vocabulary; it does not duplicate it. Authored as its own file so
LanguageModel and concrete runtime extdeps both have a single named carrier
to consume (rather than each re-declaring the bundle shape).

**Dependencies — what this PR DOES vs DOES NOT touch in T-4.** This PR
adds T-33 to T-4's authoritative dependency contract:
`[needs T-3, P1-KEYSTONE, T-29, T-30, T-25-core, T-33]`. A schedule-edge
update recording that LanguageModel cannot be authored before ModelCore
exists; single-authority for the dependency fact lives in T-4's `[needs
…]` line, not in this T-33 prose. What this PR does NOT do: re-express
T-4's fact-bundle *authoring contract* (the body prose under T-4) in
terms of "LanguageModel consumes ModelCore". That authoring reframe is
its own commit train, after T-33 lands. The Q1 ratification established
the SHAPE; the schedule edge belongs in `[needs …]`; the authoring
reframe is a separate edit.

**Reference:** `docs/design-v4-compiler-homomorphism.md` §"`ModelCore`" +
§"Ratified Q1 supersession — option C runtime split".

---

### T-33-Q10 — std/model_core.dag effect / partiality carriers  [SCHEDULED]
**Owner:** ModelCore substrate follow-up. **Authority:** `docs/design-v4-compiler-homomorphism.md`
§"Open Q10 — Partiality and effects in `ModelCore`".

Lands the ModelCore-owned effect / resource / partiality carrier names
currently deferred by Q10: `EffectSignature`, `ResourceAccess`, and any
ratified companion witnesses (`CommutationWitness`, `ConflictWitness`, or
their replacement shape). This is the concrete dissolve-on-arrival owner
for any pre-merge T-34 forward declaration of `EffectSignature` /
`ResourceAccess`.

**Trigger:** first primitive operation in `std/` that declares non-trivial
effect or partiality, or the first consumer that needs to validate runtime
resource/effect boundaries against those facts. When this task lands,
`src/v4/std/runtime.dag` must replace any local forward declarations with
imports from `v4.std.model_core`.

---

### T-34 — std/runtime.dag + extdeps/runtimes/*.dag — runtime substrate  [SCHEDULED]
**Operator-ratified 2026-05-21 (Option C).** The former `HostModel`
umbrella is decomposed. Abstract runtime carriers live in
`src/v4/std/runtime.dag`; concrete runtime fact-bundles live in
`src/v4/extdeps/runtimes/*.dag` and consume those carriers plus
`ModelCore` (T-33). Runtime-side concerns (allocation, execution model,
runtime values, resource limits) remain categorically different from
emit-side concerns (grammar, serialization), but concrete runtimes are
external dependency models rather than a `std/` umbrella abstraction.

**Files:**
- `src/v4/std/runtime.dag` — abstract runtime carriers only.
- `src/v4/extdeps/runtimes/v4_evaluator.dag` — concrete v4 evaluator
  runtime bundle.

**Carrier shape (per Option C):**
- **Runtime value representation** — how a substrate primitive is
  represented at runtime.
- **Primitive operation interpretation** — Transform → runtime function
  call; Branch → runtime branch choice; Value → runtime literal allocation.
- **Execution semantics** — call/return, allocation, control transfer.
- **Resource / effect boundary** — FFI, exceptions, async / concurrency
  model, identity / reference semantics.

**Dependencies — `[needs T-33]`.** Abstract runtime carriers import
`ModelCore` directly. T-33-Q10 effect/resource refinement remains the
dissolve-on-arrival owner for richer operation/resource facts; until then
`std/runtime.dag` references the landed `PrimitiveOperationRef` and a
minimal `ResourceAccess { resource: Node }` carrier rather than declaring a
second ambient capability table.

**Why extdeps, not a runtime umbrella.** Q1c ("eval =
translate-to-machine-code + execute") is still rejected because direct
evaluation involves runtime values, process state, and failure modes.
Option C keeps that split while removing the misleading `Host*` `std/`
umbrella: `std/` states the abstract runtime contract, and each concrete
runtime is an external system model under `extdeps/runtimes/`.

**Consumer:** T-22 (`compiler/05_eval.dag`) — eval consumes the runtime
carriers it needs plus a concrete runtime extdep. The MVP-B route
(`eval(runtime)`) depends on this carrier set landing.

**Reference:** `docs/design-v4-compiler-homomorphism.md` §"Runtime
carriers (option C)" + §"Ratified Q1 supersession — option C runtime split".

---

### T-35 — virtual module-loader + AgentStore (filesystem-free ingest)  [SCHEDULED]

**Operator-ratified 2026-05-26.** Eliminates filesystem I/O from the
compile path by replacing file reads with an agent-supplied AST store.
This is the **ingest-side** infrastructure that T-23/AGENT-1 composes
over — T-35 owns the no-filesystem entry point; T-23 owns the non-text
AGENT-SURFACE contract (lens reads, `apply_diff`, structured output).
These are complementary, not overlapping: T-35 is a prerequisite of the
T-23 round-trip workflow.

**Scope — two pieces (ingest side only):**

1. **Virtual module-loader.** The module-admission stage (T-28-B) is
   replaced with an agent-supplied store: a content-addressed map from
   `ModulePath` to `Node` (pre-parsed `.dag` AST). The stage reads from
   the store rather than the filesystem. Agents write to the store; the
   compiler reads from it. No filesystem I/O anywhere in the compile path.
   `🟡 gate: dissolve-on T-28-B — module-admission stage must be extracted
   from 03_resolve.dag before the virtual loader can replace it.`

2. **AgentStore carrier.** A mutable, content-addressed carrier in
   `src/v4/std/agent.dag`:
   `AgentStore { entries: Map<ModulePath, Node> }` with operations
   `store_insert`, `store_lookup`, `store_delete`. The store is the
   agent's write surface; the virtual module-loader is the compiler's read
   surface. Agents populate the store and invoke `compile_with_store` — the
   filesystem-free entry point alongside existing `compile_ingest_staging`
   in `00_compile.dag`.

**Authority boundary — what T-35 does NOT own:**
The non-text AGENT-SURFACE (structured compiler output — `InferenceResult`,
`DiagnosticSet`, lens reads, `apply_diff`) belongs to T-23/AGENT-1, which
already declares this authority (see T-23 entry above and
`lens/application.dag` header mark; "no new file, no new authority"). A
worker dispatched from T-35
must not define new output types or a structured output mode — that work
goes in T-23's scope. T-35 workers stop and escalate if they feel pressure
to define a new agent output surface.

**Files:**
- `src/v4/std/agent.dag` — new file; `AgentStore` carrier only.
- `src/v4/compiler/00_compile.dag` — add `compile_with_store` entry point
  (filesystem-free compile path; output type is the existing `Outcome<TargetSource>`).

**Dependencies — `[needs T-28-B]`.**
T-28-B is the only hard prerequisite: the module-admission stage must be
extracted before the virtual loader can replace it. T-9 and T-10 are
prerequisites of T-23/AGENT-1's round-trip workflow, not of T-35's
ingest-side scope.

**What this is NOT:**
- Not a new language feature — no new `.dag` syntax.
- Not a runtime evaluator — `AgentStore` is compile-time, not runtime.
- Not T-34 (runtime substrate).
- Not T-23/AGENT-1 — T-35 does not define the agent output surface
  (InferenceResult, DiagnosticSet, apply_diff). Those live in T-23.

**Sequencing.** Post-M3. Dispatch after T-28-B confirms merged.
Unblocks: T-23/AGENT-1 round-trip workflows; IDE integration; automated
`.dag` authoring agents.

---

### T-4.15 — extdeps/protocols/{rest,graphql,grpc}.dag — transport substrate  [SCHEDULED]
**Operator-ratified 2026-05-20 (PR #3437, P4 — "Glue derivation is
composed homomorphism, orthogonal to the compiler").** Transport
semantics substrate — the carrier shape that a **future omni-stack
expansion** (beyond T-16's current OpenAPI-based wire-contract scope)
glue derivation composes over. Inter-module marshaling is structurally a
**composed homomorphism**: source → wire-format model → target. Same
primitive (the coercion fold), applied twice through a shared transport
model. **Note on the T-16 edge:** T-16's authoritative `[needs]` does
NOT include T-4.15 — T-16's current scope is the OpenAPI demo (via
T-4.6 `openapi.dag` + T-4.8 `coordination.dag`'s WireContract), not the
full REST/GraphQL/gRPC transport substrate. Co-scheduling T-4.15 with
T-16 would assert a consumer edge T-16 does not actually carry in its
current scope.

**Files:** `src/v4/extdeps/protocols/{rest,graphql,grpc}.dag` — directory
does not exist on main; this task is its first authoring. The
**`extdeps/protocols/`** slot is named verbatim in the design doc as
**"Currently missing"** substrate.

**Modeling decisions:**
- Per-transport carrier shape — the structural facts each transport
  declares (REST: method / path / status / body-type negotiation;
  GraphQL: schema / query / mutation / subscription; gRPC: service /
  method / message types / streaming kinds).
- Where to ground against T-26 boundary carriers (HttpMethod / Url /
  NetworkAddress) vs. authoring fresh transport-specific carriers.
- Bidirectional read: same substrate consumed by ingest (parse a wire
  message into Node) and emit (project a Node into wire-format text) —
  composed homomorphism, NOT a string template (no-templating
  principle).
- Relationship to T-4.8 `coordination.dag`'s `WireContract` /
  decomposed `WireContractFacts` + `CoordinationBind` shape (HTTP / REST is
  the immediate consumer in omni-stack scenarios).

**Dependencies — `[needs T-3, T-26]`. Language-orthogonal per P4.**
Numeric and string vocabulary from T-3 for wire-format primitives;
HttpMethod / Url / NetworkAddress from T-26 for REST grounding. **NOT
T-4.** The transport substrate declares its OWN wire-format type system
(REST: structured HTTP bodies + headers; gRPC: protobuf wire format with
its own primitive/message types; GraphQL: GraphQL type system).
LanguageModel bindings happen downstream at *composition time* — a
**future-expanded omni-stack glue derivation** (beyond T-16's current
OpenAPI scope; not part of T-16's current `[needs]`) composes
`LanguageModel(source) ∘ TransportModel ∘ LanguageModel(target)` via the
coercion fold, P4's "applied twice through a shared transport model."
Co-locating language-binding facts on this substrate would collapse the
shared transport model back into language-specific concerns — the
opposite of the P4 framing.

**Scope discipline — L-2 holds.** Model the versioned transport SPEC
(IETF RFCs for REST / HTTP semantics, the GraphQL spec, the gRPC HTTP/2
spec) — NOT specific implementations (Axum / FastAPI / Apollo / Tonic).
Libraries are downstream programs written in the modeled language; they
are ordinary `Node`s, never modeled as targets.

**Out of scope for the initial single-target compiler.** The design doc
names this explicitly: "Glue derivation / omni-stack — orthogonal
substrate (P4). Architecture must not preclude, but no implementation in
the initial single-target compiler." This task is scheduled because the
substrate slot is named; the file authoring waits until omni-stack glue
work activates *beyond T-16's current OpenAPI scope* (a future expansion;
T-16's `[needs]` does NOT list T-4.15 today), not part of the critical-path single-target
MVP.

**Related substrate slot (NOT bundled here).** The design doc also names
**`std/system.dag` or equivalent — module decomposition substrate.
Currently informal** as a sibling P4 substrate gap. That is a separate
follow-on task (not scheduled here; tracked when glue derivation work
activates).

**Reference:** `docs/design-v4-compiler-homomorphism.md` §"P4 — Glue
derivation is composed homomorphism, orthogonal to the compiler" +
§"What's NOT in scope for this design" (the `extdeps/protocols/` row).

---

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

### T-32 — minimum never-hand-edited bootstrap seed  [SCHEDULED]
**Operator directive 2026-05-17 (briansrls).** Reduce the bootstrap seed
to the absolute minimum, in a form that never needs a hand-edit, ever.

**The success metric — two bars, both must hold:**
- **Seed size** — the seed is the smallest it can be.
- **Zero ongoing hand-edits** — once frozen, the seed is never
  hand-edited again, for any v4 language change.

A bloated seed that is never hand-edited still **fails**; a tiny seed
that needs an edit on every language change still **fails**. The metric
is size **and** no-hand-edit, together — not either alone.

**This is NOT "retire v2".** "Off v2" is a *gameable* success metric —
v3 proved it: v3 retired v2, cleared "off v2", and was no better,
because v3's own Rust seed stayed bloated. v2-retirement is at most an
incidental outcome of hitting the real metric, never the goal. Frame
every milestone of this task by seed-size + no-hand-edit, never by
"v2 removed".

**Phase 1 — DEFINITION (design-first; operator-reviewed; gates all
reduction work).** Before any reduction, produce an operator-reviewed
definition of exactly what "minimum never-hand-edited seed" means. The
open design question — posed in `src/v3/SELF_HOSTING.md` ("how small can
the seed parser be?") and never resolved:

- The seed's only job is to bootstrap v4 once from source.
- "Never hand-edit, ever" requires the seed **decoupled from v4's
  evolving language surface** — v4's `.dag` grows arbitrarily; the seed
  never changes.
- That holds if the seed compiles a **frozen, pinned bootstrap
  snapshot**: seed → frozen-snapshot → stage-N → live v4, the seed and
  the snapshot both pinned forever.
- Tension — **language drift**: if live v4 outgrows the snapshot's
  compiler, the chain needs intermediate stages walking the language
  forward, *or* the snapshot's frozen language subset must stay inside
  what the seed handles.
- So: minimum seed = the smallest compiler for the snapshot's **frozen
  language subset**; never-hand-edited = that subset is frozen so the
  seed never grows. **The real question Phase 1 must answer: what IS
  that frozen subset, and how small can it be.**

**Phase-1 design content — [`docs/design-bootstrap-fact-model.md`](../../docs/design-bootstrap-fact-model.md).**
The Phase-1 definition is worked out in that design doc: the layer model
(every layer — compiler / seed / snapshot / target / runtime — a fact
model, the seed as a *projection* `emit(snapshot, target, runtime)`),
the comprehension boundary modeled as a frozen sub-model + a `footprint`
fold + a `⊆`-`Witness` gate, the bootstrap circularity as a fixed point
whose bit-identical witness proves *reproduction* — not seed honesty;
path-independence holds only for benign entry paths (§5) — and the **two**
honest floors it bottoms out at: the permanent physical axiom and the
seed-honesty axiom, the latter *discharged* by diverse double-compilation
rather than dissolved (§6).
**Phase 1's deliverable is the operator-ratified layer model in that
doc**; the seed-reduction work (Phase 2+) is dispatched against it. A
worker picking up T-32 Phase 1 reads `docs/design-bootstrap-fact-model.md`
as the brief.

**Sequencing.** Design-first — do **not** dispatch any seed-reduction
work until the operator reviews and ratifies the Phase-1 definition.
Parallel fill — adjacent to T-15 (self-host fixed-point gate) and T-20
(`workflow/bootstrap.dag`); **not** on the pipeline critical path.

---

### T-36 — Omni ingest demo: round-trip fidelity claim  [SCHEDULED]

**File**: `src/v4/test/claim/round_trip/dag_ingest_round_trip.dag` (new)
**Why**: `ingest = emit⁻¹` (C5, THESIS §B2-OMNI) is the central bidirectionality property.
T-6 fills the tokenizer, T-7 fills the parser — but without a checked executable claim the property remains aspirational. T-36 closes that gap: one worked claim that the full round-trip holds on a known `.dag` program.

**Scope:** ONE TestClaim:
- Input: a short, self-contained `.dag` program (hand-authored, committed as fixture in `test/fixture/`)
- Forward pass: `ingest(source, dag_language_model()) -> Node` (T-6 lex → T-7 parse → T-8/T-9 normalize/resolve/infer)
- Inverse pass: `emit(node, dag_language_model()) -> String`
- Claim: the emitted string is identical to the original source (or identifies exactly which declared-normalized differences apply per C5-fidelity)
- Assert kind: `EqualsClaim` (lhs: original source, rhs: re-emitted source; bit-identical) or `RoundTripClaim` if the claim is fidelity-up-to-declared-normalization; Tier1, Integration layer

**Dependencies**: `[needs T-6, T-7, T-8, T-9, T-10; dag.dag lex/grammar data from T-6/T-7 fill]`

**Modeling decisions:**
- Input fixture selection: a program that exercises enough of the dag surface to be non-trivial but is fully within the T-6/T-7 grammar fill scope — ideally a TestClaim definition itself (self-referential, closed-form)
- Normalization budget: what exactly is "bit-identical"? Comment stripping? Whitespace normalization? The normalization disposition is a C5 `Declared-normalized` fact in the `dag.dag` language model — the T-36 fixture must reference or assert the expected canonical output derived from that model, not author new C5 facts in the fixture header (single authority: facts flow from the language model, not from the test)
- Fail-closed: if ingest cannot represent any part of the input — ambiguity, unsupported syntax — the claim must produce a Diagnostic, not silently pass

**Sequencing:** dispatch after T-10 merges (T-8/T-9/T-10 are prerequisites for the executable round-trip; fixture authoring may begin after T-6/T-7 as prep). Unblocks T-15 (self-host fixed-point validation needs a working round-trip before the fixed-point loop is meaningful).

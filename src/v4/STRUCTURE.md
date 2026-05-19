# v4 — Structural File Tree (closed system)

This is the **closed file tree** for v4. Every file is enumerated below. New files require explicit operator ratification (substrate extension = stop signal). The discipline that v3 lacked: file-tree-as-substrate-categorization, cost-of-change = 1.

## Why v4 exists

v2 proved 1-residual hand-Rust is achievable but had modeling gaps. v3 had richer modeling ambition but accumulated 192 hand-Rust files plus paper-shrink debt because the work-direction substrate was prose, not data. v4 is the synthesis: **v2's residual discipline + v3's modeling depth + recursive-flex applied from day 1.**

## File tree

```
src/v4/
  STRUCTURE.md           # this file
  CULTURE.md             # the working agreement + reading map (read first)
  BRIEF_TEMPLATE.md      # the worker brief shape (immutable across tasks)
  TASKS.md               # the XL task plan (count drift-proof; see T-15)

  std/                   # substrate primitives (15 landed + 1 P2-staging witness; see note on `fact_density.dag`)
    node.dag             # 6 type connectives + 5 L1 behaviors (substrate root)
    algebra.dag          # Magma/Monoid/BoolAlgebra/FreeMonoid (structures only)
    cardinality.dag      # cardinality refinement, P4 decidability
    witness.dag          # Witness<C> — fail-closed lens reads, no Option::None
    diagnostic.dag       # structural Diagnostic { reason, at, correction }
    logic.dag            # Bool — classical two-valued logic (Boolean algebra)
    nat.dag              # Nat — natural numbers (Peano); numeric-tower base
    machine.dag          # Byte/Word*/MachineWidth/PointerWidth — machine repr
    integer.dag          # Int + fixed-width ints (Nat projected onto a width)
    float.dag            # Float — IEEE-754 floating-point (rounding-aware algebra, not exact Field)
    text.dag             # Char (Unicode code point) + String (FreeMonoid<Char>)
    network.dag          # HttpMethod / Url / NetworkAddress boundary carriers
    collection.dag       # bounded containers
    verification.dag     # TestClaim schema + Tier×Layer classification (v4-fresh; studied v3/dsl)
    report.dag           # advisory carrier (NOT fail-closed Diagnostic); used by synthesis lens
    fact_density.dag     # P2-staging only (INVARIANTS §P2): T-30 `compile_to_dag` parse witness — **not** a landed std primitive until a **generated** `.dag` consumer reads `SourceSpecReadFact`; hollow-alias authority today is the private Rust mirror module `v4_hollow_alias_gate` in `v3-compiler`. See `TASKS.md` T-30.

  extdeps/               # external system contracts (23 files)
    cpp_abi.dag          # C++ ABI / target data-model (LP64/LLP64/ILP32/ILP64)
    languages/           # language models (direction-agnostic — emit AND ingest)
      dag.dag            # gunbc `.dag` — B2-OMNI language #1 (C1 extension 2026-05-16; relay merry-ibex-337)
      rust.dag
      python.dag
      go.dag
      cpp.dag            # C++ (subsumes C subset); ISO/IEC 14882
      typescript.dag     # TypeScript + ECMAScript
      verilog.dag        # Verilog HDL (IEEE 1364-2005; extdeps language model; header Consumes: std/node.dag; std/nat.dag (Nat); schedule edges in TASKS extdeps fan-out)
      llvm_ir.dag        # LLVM IR (T-4.12 — B2-OMNI probe; down-the-stack SSA)
      machine_code.dag   # ISA-parameterized (T-4.13 — bottom of stack; disasm fail-closed)
      ptx.dag            # CUDA/PTX (T-4.14 — B2-OMNI+IN-B probe; SIMT data-parallel)
      lean.dag           # Lean 4 (B-2 — PROOF-1 prover target; termination first; Coq deferred)
    frameworks/          # framework substrates (UI / server / data)
      react.dag          # React: Component/Hook/Effect (frontload per operator 2026-05-15)
    formats/             # data format models (direction-agnostic)
      json.dag
      yaml.dag
      csv.dag
      toml.dag
      json_schema.dag
      openapi.dag
      spice.dag          # SPICE netlist (T-4.10 — B2-OMNI probe; Shape B, no control flow)
    process.dag          # OS process model (POSIX/SUS)
    file_system.dag      # OS file system model (POSIX file/directory operations)
    coordination.dag     # multi-program: Endpoint/DeploymentUnit/sync/async/stream/pubsub
                         # (effect-typed carriers over existing 5 L1 behaviors;
                         # NO 6th behavior per IN-B decision 2026-05-15)

  compiler/              # pipeline orchestrator + 6 stages (7 files)
    00_compile.dag       # orchestrator: (Source, TargetSpec) -> Result<TargetSource, Diagnostic>
    01_tokenize.dag      # FreeMonoid<Char> -> TokenStream
    02_parse.dag         # TokenStream -> ParseTree
    03_normalize.dag     # ParseTree -> NormalizedTree (sugar dissolution)
    03_resolve.dag       # NormalizedTree -> ResolvedTree (symbol binding)
    04_infer.dag         # ResolvedTree -> InferredTree (types/algebra/cardinality)
    05_emit.dag          # InferredTree + TargetSpec -> TargetSource (omni-emission projection)
    05_eval.dag          # InferredTree + Inputs -> Value (THE PRIMARY execution path,
                         # THESIS:225; sibling of emit — eval executes, emit projects)

  lens/                  # dimensions (13 files, parallel after compiler)
    complexity.dag
    cost.dag             # Tier 1 + Tier 2 textbook (α(n)/log*/log log/sub-exp); UnknownCost floor
    parallelism.dag
    effect.dag
    ownership.dag
    idempotency.dag
    synthesis.dag        # cross-algorithm complexity (C7; advisory via Report carrier)
    coverage.dag         # meta-lens — L6/L7/impossible-bug/testgen coverage discipline (structural)
    testgen.dag          # producer side — reads substrate, emits TestClaim corpus (Phase 1.5)
    affected_set.dag     # incremental re-exec frontier; replaces detect-affected shell (Phase 1.5)
    affected_set_examples.dag # expected affected-frontier example values
    application.dag      # apply_lens surface — opt-in depth + ONLY advisory→fail-closed bridge
    registry.dag         # P2-staging only (INVARIANTS §P2): PREFIX T-23 v0 `LensIdV0` × `LensModulePathV0` rows — **not** landed single authority until a **generated** consumer reads `LensRegistryEntryV0`; `v4_lens_registry_dag_smoke_test.rs` is **parse + inference cleanliness** only (same posture as `fact_density.dag`). Operator pin §3 human mirror; amend `.dag` first.

  workflow/              # meta-process as data (2 files; the v3-derived
                         # work-direction substrate — brief/worker_output/
                         # cycle/retirement/doc_anchor — was cut
                         # 2026-05-15, operator-ratified: the project does
                         # not model its own work-direction; the compiler
                         # model self-justifies, no meta-layer narrates it)
    bootstrap.dag        # bootstrap orchestration AS DATA (seed-once → self-host
                         # → fixed-point); v2 interprets it; no build.rs/shell
    ci.dag               # CI pipeline AS DATA; .github/workflows/ci.yml is derived
                         # (THESIS:226 — adding a CI gate = editing this one file)

  bin/
    main.dag             # emits main.rs trampoline (0-floor compliant)

  test/
    claim/               # TestClaim data — no hand-Rust tests
      impossible_bug/    # the R1+ impossible-bug class demos
        idempotency_contract.dag
        nested_optional_flatten.dag
        suboptimal_complexity.dag
        transport_type_drift.dag
        unenumerated_effects.dag
        unhandled_diagnostic_paths.dag
      diagnostic_correction/
      algebra_laws/
      manual/            # hand-authored anti-regression anchors (Phase 1.5)
        connective_anchors.dag
        nat_law_anchors.dag
        t19_manual_anchor_manifest.dag  # T-19 manifest — `T19ManualAnchorKey` membership rows
        resolve_compile_anchor.dag  # resolve + wave-1 canonical `Set` compile anchor (#3225; T-22 defers `v2 run`)
      boundary/          # boundary-honesty probes
        english_ingest_fail_closed.dag  # T-4.11 — fail-closed ingest, no fabrication
    fixture/             # canonical input programs
```

**Total: 75 .dag files + 5 docs + 5 .gitkeep = 85 files.** (Per invariant
#1 the enumeration above — not the count — is authoritative; the count is
a checksum, updated on every operator-ratified file addition/removal.
**Reconciliation (2026-05-17, PR #3225 / review #13750):** the prior printed
total (`65`) lagged the live tree at **67** `.dag` files — the line had not
yet absorbed the 2026-05-16 `extdeps/languages/dag.dag` +1 (and other
intervening operator-ratified edits). **#3225** adds **`test/claim/manual/resolve_compile_anchor.dag`**; **#3212** adds **`test/claim/manual/t19_manual_anchor_manifest.dag`**. **69** `.dag`, matching `find src/v4 -name '*.dag'`.
**Earlier operator-ratified deltas already in the tree (audit trail only — not re-applied by #3225):**
+1 .dag 2026-05-16: `extdeps/languages/dag.dag` — operator-ratified C1 closed-tree
extension (Option A, relay merry-ibex-337). −5 .dag 2026-05-15: work-direction
meta-layer cut, operator-ratified. **2026-05-17 (PR #3212):** enumerate
`test/claim/manual/*` (4) + `test/claim/impossible_bug/*` (6); checksum **65→69** `.dag`.)
**2026-05-18 (T-26):** add `std/network.dag` for shared
`HttpMethod` / `Url` / `NetworkAddress` boundary carriers; checksum
**69→70** `.dag`.
**2026-05-18 (T-29):** add `extdeps/cpp_abi.dag` for the operator-ratified
C++ ABI / target data-model feeder; checksum **70→71** `.dag`.
**2026-05-18 (T-30):** add `std/fact_density.dag` P2-staging parse witness;
checksum **71→72** `.dag`.
**2026-05-18 (PREFIX / T-23 v0):** add `lens/registry.dag` (`LensIdV0` + `LensModulePathV0` registry twin of
`docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` §3); checksum **72→73** `.dag`.
**P2-staging** (INVARIANTS §P2) until a generated consumer reads the rows — paired
`v4_lens_registry_dag_smoke_test.rs` receipt (parse witness only; same discipline as `fact_density.dag`).
**2026-05-19 (#3349):** add `lens/affected_set_examples.dag` as expected affected-frontier example values;
checksum **74→75** `.dag`.

## Scalar/numeric concept decomposition

`std/primitive.dag` is **deleted.** It lumped five unrelated concepts
(`Int`/`Float`/`String`/`Char`/`Bool`) under the label "primitive" — but per
THESIS epistemic stacking the genuine primitives are the six connectives
(`node.dag`) and the algebra roots (`algebra.dag`); `Int`/`Bool`/`String`/`Char`
are *compositions* that attach by inhabitance, not primitives. It is replaced
by six concept-located files, each anchored to a real external concept
(Wikipedia / spec) — the same anchored-concept discipline `extdeps/` uses:

- `std/logic.dag` — `Bool`, classical two-valued logic
- `std/nat.dag` — `Nat`, natural numbers (Peano)
- `std/machine.dag` — `Byte`/`Word*`/`MachineWidth`/`PointerWidth`, machine representation
- `std/integer.dag` — `Int` + fixed-width ints (a `Nat` projected onto a machine width)
- `std/float.dag` — `Float`, IEEE-754 (a sign/exponent/mantissa bit-record
  inhabiting a rounding-aware algebra — *not* an exact `Field`, and *not*
  opaque: fully grounded, only its algebra is weakened)
- `std/text.dag` — `Char` (Unicode code point) + `String` (`FreeMonoid<Char>`)
- `std/network.dag` — network boundary carriers (`HttpMethod`, `Url`,
  `NetworkAddress`) shared by OpenAPI / coordination / wire contracts

Each declares its own inhabitance (the inhabiting type owns its grounding —
INVARIANTS P2); `algebra.dag` owns the algebra *structures* only.

**`Hash` re-homing.** PR #3150's B1 entry slated the `Hash` content-address
digest for `std/primitive.dag`. With that file deleted, `Hash` — which is
not a scalar — re-homes to `std/node.dag`: this PR declares the opaque
`Hash` type there (beside `Symbol`, the other K-1-opaque substrate-root
identity) with `node.dag` as the authority carrier. The `content_hash`
fold and the canonical-form clause stay the T-1 closeout, per B1.

**Kernel-ambient types.** `String`, `Int`, `Bool`, `Char`, `List`, `Map` are
provided by the v2 seed and are usable in any `.dag` file *without an import*.
This relaxes only the import edge — not single-authority: the v4 substrate
file that *models* each type (`text.dag` for `String`/`Char`, `integer.dag`
for `Int`, `logic.dag` for `Bool`, `collection.dag` for `List`/`Map`) remains
its sole authority. A file's header `Consumes` line lists a scalar file only
when it needs that type's *modeled* facts (algebra, inhabitance, totalization)
— not when it merely needs the raw kernel value (e.g. a `String` message
label). This is why several headers note "String … is kernel-ambient — no
import".

## Anchor convention

Every v4 `.dag` file carries an `# Anchor:` line in its header. The anchor is
a citation to canonical knowledge that grounds the file's modeling — typically
a Wikipedia article, a language specification, a POSIX standard, or an
authoritative gunbc doc (THESIS.md, MODELING.md, INVARIANTS.md, memory entries).

The discipline:
- **Reviewers can validate the model against the anchor.** If `extdeps/process.dag`
  models a "Process" but the structure doesn't match what
  https://en.wikipedia.org/wiki/Process_(computing) says a process is, the
  reviewer surfaces it.
- **Workers ground modeling in shared facts.** Per `feedback_modeling_philosophy`
  and `feedback_epistemic_stacking`: every concept attaches to an explicit
  ontology rooted in canonical knowledge — no opaque names, no invented
  vocabulary disconnected from established meaning.
- **For external dependencies (`extdeps/`)** the anchor is mandatory and
  external (Wikipedia / spec). For internal substrate (`std/`, `compiler/`,
  `lens/`, `workflow/`) the anchor may be internal (THESIS, MODELING) when
  the concept is gunbc-specific, or external when it grounds in standard
  CS / math.
- **One anchor per file.** Multiple references for the same concept can be
  joined on one line; the file represents one cohesive concept and should
  have one canonical anchor for that concept.

## Zero-deferrals discipline (operator directive 2026-05-15)

v4 has NO deferrals. Any decision that would require a workaround, or
push the decision to a follow-up phase, is a **HARD STOP — escalate
to operator**.

This applies at every tier:
- **Worker tier**: hitting an unmodelable case, ambiguous substrate, or
  "I'll just do this for now" temptation → STOP, file an inbox message
  to operator with the decision shape, do not work around. The brief's
  ESCALATION TRIGGERS section is binding.
- **Audit tier**: every disposition is PROVEN, NOT-IN-V4 (with named
  reason), or OPERATOR-DECISION-REQUIRED. There is no R4-DEFERRED,
  no fast-follow, no canvas-blocked. See [`docs/v4-close-interrogation.md`](../../docs/v4-close-interrogation.md)
  §0 disposition vocabulary.
- **Substrate tier**: if a substrate decision is ambiguous, the file
  is NOT scaffolded until the operator decides. No "we'll figure out
  the modeling during the worker task."

There is no "v5 / v6 / R5". v4 is the shipping version. If something
is needed but not in v4, it goes through a fresh operator
scope-expansion decision (a v4 amendment) — not a deferral to a
future phase that doesn't exist.

The discipline exists because v3 failed exactly here: deferrals
created drift, drift created gaming, gaming required operator
intervention to catch. Zero-deferrals removes the drift surface at
its source.

## Architectural commitments (ratified during PR #3147 review)

These are substrate-level decisions that constrain every worker's modeling
freedom. They are structural, not process-discipline. Per-task briefs
reference this section when dispatching workers.

1. **`TypeNode` and `Behavior` are CLOSED enums** (per C1 stop-signal,
   THESIS:202). Adding a 7th type connective or 6th behavior requires
   explicit operator ratification of substrate extension. The closure is
   enforced in the substrate itself (Disj sum-type declaration in
   `std/node.dag`), not by review process — the compiler reads the closed
   enum and refuses to compile any program that synthesizes outside it.

2. **Tier 2 partial-op totalization lives with each scalar type, in its
   own file** (per THESIS:175-176). A scalar's partial operations declare
   their totalization shape (`Result`-return / `Witness`-return /
   refinement-precondition) in the same file as the type itself: integer
   divide / modulo in `std/integer.dag`, NaN-producing ops in
   `std/float.dag`, indexed access / force-unwrap where the indexed type
   lives. No separate "totalization registry."

3. **`Diagnostic` schema carries a typed `correction`** (per THESIS:103-105
   "show the correct code"). Schema:
   `Diagnostic { reason: Symbol, at: Locus, correction: Correction }`
   where `Correction = Suggested(Node) | Unavailable(NoCorrectionReason)`.
   The "show the correct code" promise is structural — every Diagnostic
   site answers the fix question, and answers it with a type, not a
   convention. "No fix" is the `Unavailable` case carrying a structural
   `NoCorrectionReason`, never a bare `None`. The suggested fix is a `Node`
   (the bounded kernel's only recursive type — a subtree of Nodes IS a
   Node; no separate `NodeFragment` alias). WHERE it applies is the
   Diagnostic's own `at: Locus`, not a field of the fix.

4. **Every TestClaim slots into one (Tier × Layer) cell** (per THESIS §168-182
   correctness tiers + TESTING.md §141 test layers — two orthogonal axes).
   - **Correctness Tier** (when bug is caught): Tier1 (compile-time) | Tier2 (runtime-totalized) | Tier3 (runtime-observed L4-L7)
   - **Test Layer** (where test runs, with target ratios per TESTING.md): Unit ~75% | Integration ~15% | Boundary ~10%
   Testgen output respects ratio targets; coverage lens verifies completeness
   across the (Tier × Layer × Substrate) cross-product. Workers writing
   manual TestClaims declare both axes in the claim metadata.

5. **Concept unifications are structurally enforced** (per THESIS §184-188).
   THESIS commits to four named unifications. Each lives in a single substrate
   file declared via the `// Unifies:` header field. Adding a parallel carrier
   for any unified concept is a substrate extension = STOP signal. The four:
   - **`coercion = emission`** — owned by `compiler/05_emit.dag`. No separate
     coercion engine; coercion logic lives in emission rules.
   - **`coercion cost = complexity`** — owned by `lens/complexity.dag`. The
     cost of converting between representations IS a complexity-lens read;
     no `CoercionCost` carrier.
   - **`language spec = transport spec = interpreter runtime`** — owned by
     `extdeps/languages/*.dag`. ONE substrate carrier per language for all
     three roles; different lenses read different facts from the same data.
   - **`idempotency + cancellation + redundancy = algebraic simplification`** —
     owned by `lens/idempotency.dag`. Three named runtime concerns are ONE
     mechanism; no `lens/cancellation.dag` or `lens/redundancy.dag`.

6. **Emission is mechanical; algebra-enforcement is the primary job**
   (per THESIS:13 "causal engine ... before emission becomes a mechanical
   translation" + THESIS:196 "the epistemic chain IS the emission algorithm;
   every emitter special case is evidence of an ungrounded concept upstream"
   + THESIS:441 "Emits ... as mechanical translation").
   The compiler's LOAD-BEARING work is validating the epistemic chain:
   `compiler/04_infer.dag` algebra-homomorphism search + `lens/*` +
   `std/algebra.dag` grounding. `compiler/05_emit.dag` is mechanical
   projection of that validated chain. An emitter special-case is a STOP
   signal — it means a concept is ungrounded upstream; fix the grounding
   in `std/algebra.dag` or the epistemic chain (`04_infer.dag`), NOT the
   emitter. A worker on T-10 writing `if target == X` special-cases has
   found an upstream grounding gap; escalate. Declared via `// Primary:`
   header field in `04_infer.dag` (owns enforcement) and `05_emit.dag`
   (mechanical projection).

## The closed-system invariants

These are non-negotiable across all v4 work:

1. **No new files without operator ratification.** Every file in the tree is enumerated above. A worker proposing a new file is reporting a substrate gap — surface it, do not unilaterally add.
2. **No hand-Rust except the trampoline.** `bin/main.dag` emits a 1-line `include!()` trampoline (per `design-pure-bootstrap-zero.md:210`). Everything else is .dag.
3. **No file-splitting without operator ratification.** Each file is a typed pure function. If a worker thinks `04_infer.dag` should be five files, that's a substrate-design question, not a worker decision.
4. **Cost-of-change = 1.** Adding a new type/expression/transport edits exactly one file. If a change ripples, the substrate is wrong.
5. **Tests are TestClaim data.** Zero hand-Rust tests. Test surface lives in `test/claim/`.
6. **Meta-process as data (narrowed 2026-05-15, operator-ratified).** `workflow/` is now exactly `bootstrap.dag` (the bootstrap chain — load-bearing per invariant 7) and `ci.dag` (CI pipeline) — both `.dag` data, never `build.rs`/shell/hand-authored YAML. The v3-derived **work-direction substrate** (briefs / worker-outputs / cycles / retirement / doc-anchors modeled as data — the original "recursive-flex implement-before-compiler" claim) is **retracted**: the project does not model its own work-direction, and the compiler model self-justifies (rationale emergent from composition), so no meta-layer narrates it. Lens self-application to gunbc's own build/CI pipeline survives via `bootstrap.dag`/`ci.dag`; modeling our own *process* does not.
7. **`.dag` is the sole editable authority; Rust is never authority.**
   "Off Rust" means: no Rust is editable authority — not "no Rust exists
   anywhere" (the CPU always has a host; the seed is always *some*
   compiler). Three sub-invariants make this structural: (a) zero
   hand-Rust in `src/v4/` — closed file tree forbids adding it, .dag-only
   scaffold means none exists to regress; (b) emitted Rust is transient
   build-dir output, never committed; (c) bootstrap orchestration is
   `workflow/bootstrap.dag` (data, interpreted by frozen v2), never a
   `build.rs`/shell. The v4 binary is a content-addressed artifact
   reproducible from `.dag` via the frozen seed; its fixed-point hash is
   pinned.

   **A3 (operator-ratified 2026-05-15) — honest framing of this guarantee.**
   This is NOT un-gameable. Any in-repo enforcement artifact (the pin, CI,
   the seed) is editable by whoever can commit; there is no structural
   bottom (Ken Thompson, "Trusting Trust" — you cannot prove from inside
   the system that the seed is honest). The guarantee is therefore
   *un-hideable*, not *impossible-to-violate*: the reproduce-from-`.dag`-
   through-frozen-seed check is an **early-surfacing amplifier** — run
   per-PR on the affected set (T-21/T-24), it makes any divergence (extra
   Rust authority, an edited/grown seed, a 7th connective) **loud,
   immediate, and impossible to do by accident**, routing it to operator
   ratification the same commit. The seed's trust is a **named axiom**
   (built in the open, pinned at a known-good point), not a proof —
   stating it honestly is `feedback_no_engine` applied to our own claims.
   Actual enforcement is the operator-ratification spine + the STOP-and-
   escalate culture + removing the incentive to game (large honest tasks,
   no proxy ratchet); structure makes defection conspicuous, not
   impossible. (A4 — "what refuses a 7th connective" — is the SAME machine:
   deviation changes the reproduction → conspicuous signal → STOP → human
   judgment. Not "the substrate refuses.") This surfacing structure is in
   force from scaffold time; the tasks fill behavior *under* it.

   **PROOF-1 (operator-ratified 2026-05-15) — the external trust-
   discharge for A3.** A3 is honest that the guarantee is *un-hideable*,
   not *impossible-to-violate*, and that seed trust is a named axiom.
   PROOF-1 is the mechanism that SHRINKS that trust surface: gunbc's
   structural evidence — A2 termination descent, the algebra-homomorphism
   epistemic chain, the cost bound, the effect facts — is EMITTED as a
   machine-checkable proof term in an external proof assistant
   (Lean/Coq); their small, independently-audited KERNEL checks it. This
   converts "trust gunbc's internal checker + the frozen seed" into an
   INTERSUBJECTIVE check against a trust anchor gunbc does not own. It
   does NOT make gunbc un-gameable — the external kernel + the
   faithfulness of the evidence→proof-term emission are the NEW named
   axioms; PROOF-1 *moves* trust to a stronger anchor, it does not
   eliminate it (the same `feedback_no_engine`/Trusting-Trust honesty A3
   itself observes). Hard constraints: (i) the lens EXPORTS witnesses
   gunbc ALREADY HAS (the descent evidence, the homomorphism) — the
   external prover only KERNEL-CHECKS, it never SEARCHES for a proof (a
   searching export would violate no-engine + A2 "checker not
   discoverer"); (ii) it discharges only what is structurally GROUNDED —
   ungroundable concepts remain fail-closed Diagnostics; PROOF-1 inherits
   gunbc's honesty boundary, it does not paper over it. Realized as a
   lens = (evidence read) ⊕ (B2-OMNI emit to a `lean`/`coq` language
   model — ordinary emit targets, no new subsystem); NO new file (the
   closed-tree invariant holds). Primary export source = A2; an agent may
   REQUEST it (AGENT-1).

8. **On-disk emitted artifacts are checked projections, not authority
   (C4, operator-ratified 2026-05-15).** Some emitted artifacts must live
   on disk because an external tool reads them from the repo (GitHub
   reads `.github/workflows/ci.yml`; the bootstrap trampoline). These are
   NOT a violation of "no generated code on disk" — that rule means *no
   editable generated **authority***, not *no generated bytes*. A
   committed emitted artifact is a **`committed == emit(source)` checked
   projection**, guarded by the SAME machine as invariant 7 / A3: if the
   committed file diverges from `emit(.dag source)`, CI is red and the
   divergence is conspicuous and STOP-routed. The `.dag` is the
   authority; the on-disk file is a checked shadow, never hand-edited.

## Bootstrap chain

The chain IS a file: `workflow/bootstrap.dag` (orchestration as data; v2
interprets it via `v2-compiler run`). NOT a `build.rs` or shell script —
those would reintroduce editable Rust authority (the v3 regression door).

```
stage −1  v2 binary (from src/v2/'s committed 1-residual Rust — the SEED,
            outside src/v4/, frozen + CI-gated, touched EXACTLY ONCE)
              compiles src/v4/*.dag → Rust (v2 emission style)
                                    → rustc → v4-stage0 binary
stage 0   v4-stage0 compiles src/v4/*.dag → Rust (v4's OWN emission style)
                                    → rustc → v4-stage1 binary
stage 1   v4-stage1 compiles src/v4/*.dag → Rust → rustc → v4-stage2 binary
fixpt     assert stage1-emitted == stage2-emitted  (BitIdentical)
          — fixed point is stage1==stage2, NOT stage0==stage1
            (stage0 is v2-emission-style; stage1+ is v4-style)
```

- **Seed used once**: v2 produces v4-stage0 from a Rust-only environment.
  After stage0 exists, v4 compiles itself; v2 is never in the loop again.
  Identical to gcc-needs-a-C-compiler-once / rustc-was-seeded-via-OCaml.
- **Language constraint**: v4 .dag stays in v2-syntax-compatible subset
  until v4 self-compiles. New syntax lands only after fixed point.
- **Emitted Rust is transient** (option (a)): build-dir only, never
  committed, never editable authority. `.dag` is sole authority.
- **The v4 binary is a content-addressed release artifact**: pinned at
  the fixed-point hash. Day-to-day, people edit `.dag` and run the
  shipped binary — zero Rust touched. Reproducibility = rebuild from
  `.dag` via frozen v2 seed, must reproduce the exact pinned hash.
  T-15's `BitIdentical` TestClaim IS the anti-regression mechanism: a
  drift = hash mismatch = CI red.

## Relationship to v2 and v3

- **`src/v2/`** (restored 2026-05-15 for honest comparison): historical reference. v2's `04_*` cluster (12 files) is studied as the cautionary tale on substrate inflation in the type-checking layer; v4 collapses this to a single `04_infer.dag` and treats any pressure to split as a substrate-design escalation.
- **`src/v3/`**: not deleted. Sources of importable design (`dsl/std/verification.dag` TestClaim schema, L2.5 design docs in `docs/r3-path-b-*.md`, lens framework conceptual design). Importable items are explicitly cited in TASKS.md per task. v3's hand-Rust is NOT imported under any circumstance.

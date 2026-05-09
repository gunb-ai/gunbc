# R3 Cluster M Phase 1 — Substrate Canvas Dispatch Asks (2026-05-09)

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068)
**Authority**: PM-tier dispatch coordination per Director ratification at gunbc#846 #issuecomment-4412309986 (Director answered Ask 2 with "Substrate Mgr standing authority"). Sequencing structure lives at [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §3 — this brief is a light-touch dispatch surface.

---

## §0. Scope

Two parallel substrate canvases — Phase 1 of Cluster M sequencing per (γ) hybrid dispatch shape:

1. **#85** `forall_exists_quantifier_substrate_landed` — substrate canvas authoring at `docs/briefs/r3-substrate-cluster-m-quantifier-canvas-2026-05-09.md` (or analogous path under Substrate Mgr discretion)
2. **#86** `program_generator_carrier_landed` — substrate canvas authoring at `docs/briefs/r3-substrate-cluster-m-program-generator-canvas-2026-05-09.md` (or analogous path)

Both canvases parallel-authorable; no mutual substrate dep. T-WAD Slice 2 ratified ~30 min ago (2026-05-09; #1955 spawn) is the substrate-leads-runtime sequencing precedent — same shape applies here.

## §1. Director-ratified disposition (cite-and-execute)

- **Dispatch shape**: (γ) hybrid — Substrate canvas authors → Verification discipline → Verification bulk-port coordinator
- **Authoring authority**: Substrate Mgr standing authority (no Director ratification needed for canvas drafting; ratification surfaces on substrate-shape questions in the canvases)
- **Pattern precedent**: T-WAD Slice 2 (canvas-then-worker) per `feedback_dispatch_dont_preask_symmetric` + `feedback_canvas_recommendations_are_preliminary`

## §2. Substantive guidance per canvas

### §2.1 #85 — ForAll/Exists quantifier substrate canvas

Surface for Director ratification (per `feedback_substrate_principle_audit` 6-question audit):

- **Carrier shape**: how does `ForAll<P, T>` / `Exists<P, T>` embed in Node/Conj/Disj/Cardinality/Bit per `feedback_compiler_is_dag_processor`?
- **Predicate body type**: closed-DSL term (decidable per `feedback_decidability_invariant`) or runtime-evaluation against ProgramGenerator?
- **Naming canvas**: `ForAllPrograms<C>`, `ForAllInhabitants<T>`, `Exists<P>` — grep `dsl/std/` first per `feedback_grep_substrate_before_naming_ratification`
- **Adjacency**: `BinaryDimensionReportEquals` (Pattern-A) is the precedent; quantifier substrate generalizes per-(algebra, inhabitant) iteration
- **Pass-condition wiring**: how does `every_rust_test_ports_to_dag_or_generated` count quantifier-driven tests?

### §2.2 #86 — ProgramGenerator carrier canvas

Surface for Director ratification:

- **Carrier shape**: `ProgramGenerator<C>` as `.dag` data — what does it produce? Concrete `Dag` instances? `Node` trees? Constrained by what predicate?
- **Composition with #85**: `ForAllPrograms<P, ProgramGenerator<C>>` semantics
- **Examples / fixtures**: representative test cases the generator must cover (≥2 algebraic constructs per §1.6 demonstration discipline minimum bar)
- **Naming canvas**: `ProgramGenerator<C>`, `ProgramShapeFamily<S>`, `TestProgramSeed` — grep `dsl/std/`
- **Adjacency**: existing fixture patterns in `tests/fixtures/`; do those become structured `.dag` data via this carrier?

## §3. Dispatch trigger

Substrate Mgr (warm-wolf-698) dispatches canvas-authoring under standing authority once PR #2361 lands (sequencing plan ratified by Director ratification at #846 #issuecomment-4412309986). Workers can be queued ahead of merge per pre-authored brief discipline.

## §4. Receipt

- Each canvas surfaces shape questions to Director for ratification of carrier shape + naming
- On ratification: Substrate Mgr authors worker briefs for the substrate-introduction PR(s) — these are Phase 1 close
- §1.8 ledger Status moves DECLARED → CONSUMER_LANDED on substrate carrier landing

## §5. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md): Cluster M is **critical-path-load-bearing** for PB-0 closure. Phase 1 (this brief) gates Phase 2 (#87) gates Phase 3 (#84 bulk-port). Total Cluster M close target: 4-8 weeks per sequencing plan §6; fits 8-12 week R3 window with parallel dispatch per operator "staffing is not a concern" directive.

---

**End of brief.**

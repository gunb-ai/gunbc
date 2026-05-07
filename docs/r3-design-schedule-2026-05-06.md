# R3 Design Schedule — 2026-05-06

**Status**: live PM-tier dispatch matrix per Brian directive 2026-05-06 (chat): *"can we schedule all the design now?"*

**Authority hierarchy**:
- [`docs/r3-structure.md`](r3-structure.md) — architectural archive (lane defs / 95-gate §"Acceptance" / Mgr structure)
- [`docs/r3-program-plan.md`](r3-program-plan.md) — forward-looking dependency graph + escalation register + §1.8 canonical 95-gate ledger
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — bridge-inventory framework
- **THIS DOC** — per-Mgr design queue + dispatch matrix; sequencing + worker-pinning where determinable; cross-lane coordination triggers

**Purpose**: schedule per-lane design work in parallel with Brian/Director scope-calibration decisions. Mgrs do not wait for all decisions to resolve — design dispatches in flight as escalations resolve.

**Discipline**: each design item names — owner Mgr / cross-lane prerequisite / scope / dispatch trigger / closure predicate (which §1.8 ledger gate(s) it advances). Status updates flow through §1.8 ledger Status column.

---

## §1. Substrate Mgr (quick-crab-830, gunbc#1739)

**Lane scope** (per `r3-structure.md` §"Manager structure"): T-Numeric-Construction + T-Anthropic-Wire + T-CostLens-Composition + T-E-P-Producer-Broadening + T-Lens-Behavioral-Parity (cross-program) + T-Lens-Application-Surface (cross-program) + T-Workflow-As-Data + 2 Substrate-owned bridges.

**Workers idle (per Substrate Mgr canvas D)**: loyal-wolf-828 + valiant-ant-72 fully idle; proud-lynx-311 / smart-ram-167 / valiant-ibex-312 holding on #1794-cascade.

### S1 — Q-Class-2-Chain-Break gap-test surface (CLOSED 2026-05-06: option (a) RATIFIED)

**Scope (delivered)**: option (a) narrowed gap-test surfaced — function-valued `data` + evaluator consumption (lens-behavior out of scope) traces to GREEN without requiring T-LBP COMPLETE. Substrate fact under test: "function-valued data is first-class".

**Closure outcome**: **Director ratified option (a) 2026-05-06** at [gunbc#828 #issuecomment-4385329180](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4385329180). Q1 RATIFIED (narrowing strips the non-substrate axis); Q2 RATIFIED (#61 RED → DECLARED YELLOW; single-prereq-blocked on T-E-P-Producer-Broadening Phase 1 + E6-G0d, both finite-trace; E6-G0d landed at #1813); Q3 N/A. Worker brief authored at `docs/briefs/r3-substrate-s1-gap-test-representative-worker.md`; dispatch single-prereq-blocked on T-E-P P1 / S10.

**Cascade impact**: §1.8 gate #61 reframes RED → DECLARED YELLOW. Independent of Q-LBP-R3-Closeability outcome (S2 ratified separately as option (b) — chain-break dissolution does not require T-LBP COMPLETE).

**S1 closed** — worker dispatch follows T-E-P P1 / S10 cascade (post-#1782 → quick-koi-190).

### S2 — T-LBP scope-calibration canvas (CLOSED 2026-05-06: option (b) RATIFIED)

**Scope (delivered)**: Substrate canvas surfaced per-lens × per-sub-slice blocker matrix (16 cells); option (a)/(b)/(c) shape recommendations with feasibility analysis per blocker matrix. Canvas at `docs/briefs/r3-substrate-s2-t-lbp-scope-calibration-canvas.md` (landed via PR #1782 commit set).

**Closure outcome**: **Director ratified option (b) 2026-05-06** at [gunbc#828 #issuecomment-4385329180](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4385329180) (zesty-bear-812 Director session; cross-relayed via deep-wolf-155 propagation PR Q-LBP option-b paydown).
- **Q1 (option a) REJECTED**: option (a) full T-LBP scope requires landing 4c caller-side effect-set pinning carrier inside R3 — substrate-fact-introduction-without-confirmed-bridge-consumer; circular against R3 close per `INVARIANTS.md` P1.
- **Q2 (option b) RATIFIED**: T-LBP narrows R3 scope to **complexity + cost lenses only** (both share T-E-P producer dependency; closing simultaneously is critical-path fastest). Carved to R4 per `docs/r4-carve-out-routing.md` C1+C2+C3: parallelism lens, effect_enumeration lens, register zero-proxy/zero-stub narrowed to in-R3 lenses.
- **Q3** N/A.

**Cascade impact** (per ratification):
- §1.8 gate #79 (`complexity_lens_behaviorally_complete`) — IN R3
- §1.8 gate #80 (`cost_lens_behaviorally_complete`) — IN R3
- §1.8 gate #81 (`parallelism_lens_behaviorally_complete`) — R4-CARVED (C1)
- §1.8 gate #82 (`effect_enumeration_lens_behaviorally_complete`) — R4-CARVED (C2)
- §1.8 gate #83 (`lens_capability_register_zero_proxy_zero_stub`) — NARROWED scope IN R3 (C3): in-R3 lenses only
- §1.8 gate #61 (Class 2 `substrate_gap_function_valued_data_closed`) — chain-break dissolved (Q-Class-2-Chain-Break option (a) RATIFIED separately): RED → DECLARED YELLOW; single-prereq-blocked on T-E-P P1 + E6-G0d (E6-G0d landed at #1813)
- T-LAS gate `opt_in_iteration_parallelism_via_lens_application_demonstrated` — carves alongside C1 parallelism lens
- T-LSA stays in R3 (timing-lens via T-WAD; not T-LBP scope)
- T-WAD stays in R3 (timing-lens carrier separate from T-LBP)

**S2 closed** — no further dispatch on this line item. Cascade work (T-E-P P1 / S10 dispatch on quick-koi-190; complexity + cost lens consumer wiring) follows separately.

### S3 — `MachineConstraint<C>` carrier design (Brian directive 2026-05-06)

**Scope**: design substrate carriers for **algebra × machine-constraints interaction modeling** per Brian directive: concrete types like `i64` emerge as products of independent constraint axes (algebra side × machine `MachineWidth<64>`), NOT primary entities.

**P1 axiom-violation paydown (2026-05-06; Brian directive)**: prior framing `Int = AbelianGroup<Nat>` is **structurally false** — Nat is a commutative monoid under +, not the carrier of an Abelian group (no additive inverses in {0,1,2,...}). Per ratified self-correction at [gunbc#1739 issuecomment-4385432940](https://github.com/gunb-ai/gunbc/issues/1739#issuecomment-4385432940) (cross-ref: Director-side memory entry `feedback_verify_algebraic_axioms_in_ratification` logs the discipline going forward). Two mathematically sound options pending design selection (worker judgment per `feedback_compositional_not_templating`):
- **Option A** — `Int` as canonical AbelianGroup primitive (terminal, no Nat parameter). Severs the constructive Nat→Int edge; `Int` is just-ℤ, not derived.
- **Option B** — `Int ≡ GroupCompletion<CommutativeMonoid<Nat>>` (Grothendieck). Algebra-faithful; preserves compositional Nat→Int edge; introduces `GroupCompletion<M>` as new substrate (P1 procedure required: name second consumer beyond Int OR justify single-consumer via P1 carve-out).

Prior `AbelianGroup<Nat>` references in r3-structure.md, r3-program-plan.md, this doc are **DEFERRED-shorthand** until design selection lands. **T-Numeric-Construction further dispatch HELD** (Brian directive 2026-05-06) until algebra-side selection ratified. S3 (`MachineConstraint<C>`) interaction modeling proceeds in parallel — once algebra-side resolves, the interaction `Int<N> = Int × MachineWidth<N>` semantics carries through identically under either option (Int participates as AbelianGroup carrier in both).

**Q-MachineConstraint-Carrier scope RATIFIED 2026-05-06** (Brian directive: "universal substrate, ratify defaults" + clarifying inline 2026-05-06 on sub-decisions 1, 2, 5, 6). Per `r3-program-plan.md` §10.3 Q-MachineConstraint-Carrier row, the 6 sub-decisions are:

1. **Axes in R3 scope**: `MachineWidth<bits>` only. `RegisterClass<R>` / `EndianMode<E>` / alignment / signedness-as-axis deferred post-R3 absent a closure-gate forcing them. **Discipline note (Brian directive)**: this is a separation-of-facts/concerns exercise — algebra is one axis of fact, machine constraint is another, target lowering (e.g., Rust emission) is a *projection / coercion* over both. Integer modeling is the canonical exercise for this discipline; getting comfortable with the separation is the broader R3 modeling work.
2. **Interaction substrate shape**: **REST-API-style typed-contract interactions between models** (Brian directive 2026-05-06: *"these models interact a la rest apis interacting — scalable / no dual representations (.dag generated)"*). Each model (algebra, machine-constraint) declares its interface in `.dag`; the interaction substrate (composition, projection, coercion) is **`.dag`-generated**, not hand-coded. **Hard constraint: no dual representations** — any Rust counterpart must be generated from `.dag`, never hand-maintained (per `feedback_isomorphism_or_generation_for_mirrors` + `feedback_no_generated_code_on_disk`). Closed-enumeration lookup-maps and parallel Rust mirrors are both rejected (bridges).
3. **Type-level spelling**: `Int<64>` parses/elaborates as **`Compose<Int, MachineWidth<64>>`** parametrically — first slot is the **algebraic concept** (the fully-applied carrier+witness composite, e.g., `Int = AbelianGroup<GroupCompletion<Nat>>` per #1466 already on main), second slot is the machine-constraint axis. **Critical correction (per codex BLOCKING 2026-05-06)**: prior phrasing `Compose<AbelianGroup, MachineWidth<64>>` was wrong — `AbelianGroup<T>` is a *witness shape* generic over carrier `T`, not a carrier constructor (per `dsl/std/algebra.dag:148-150`: *"T = GroupCompletion<M> is the carrier; AbelianGroup<T> carries op/identity/inverse over that carrier"*). Putting bare `AbelianGroup` in `Compose<...>` slot-1 composes the witness rather than the integer concept; correct form is `Compose<Int, MachineWidth<64>>` where `Int` IS the carrier+witness composite. Same pattern for: `UInt<64>` = `Compose<UInt, MachineWidth<64>>` (where `UInt = CommutativeMonoid<Nat>` per #1818), `Real<64>` = `Compose<Real, MachineWidth<64>>` (where `Real = ApproximateField<Rational>`), `Nat<8>` = `Compose<Nat, MachineWidth<8>>`. Surface intuition is the literal product `Int × MachineWidth<64>`; substrate spelling is the parametric `Compose<concept, MachineConstraint>`. **Distinction (per claude review observation)**: the *elaboration* of a particular `Compose<Int, MachineWidth<64>>` term is parser/elaborator behavior; the **interaction substrate** (composition/projection/coercion machinery operating *on* such Compose terms) is what's `.dag`-generated per sub-decision 2 — not the `Compose<...>` *type itself*. S3 worker brief should preserve this distinction.
4. **Approximate-algebra layering**: algebra-side approximation (`Real = ApproximateField<Rational>`) and machine-side approximation (`MachineWidth<64>`) stay as independent axes that compose. `Real<64>` = `Compose<Real, MachineWidth<64>>` carries both layers explicitly per S8's named-axiom-relaxation discipline (the algebra-side approximation is internal to `Real`'s definition; the machine-side approximation is the second slot).
5. **Demonstration breadth — NOT a target** (Brian directive 2026-05-06: *"3 is not a target, we shouldn't be 'targeting' modeling, we are just doing our best job to faithfully represent the concepts"*). The 3-pair demonstration (`Int<64>` / `Real<64>` / `Nat<8>`) is the **minimum existence proof** that the substrate carries the concept faithfully. **Faithful representation of the concept is the closure criterion**; pair-counting is not. Once the parser handles generic interaction syntax + the substrate carries the algebra+machine separation, all valid pairs work by construction. Closure-gate predicate phrasing should reflect "concept is faithfully modeled" not "≥3 pairs land".
6. **Target-specificity — universal faithful representation, cost lens as the discriminator** (Brian directive 2026-05-06 elaborated 2x): `MachineConstraint<C>` is **universal substrate** — every target carries machine-constraint facts as substrate. **Universality of faithful representation** (Brian directive 2026-05-06: *"in all cases we should be able to represent the structure with literal bits/objects or whatever — the cost would be enormous — in those cases, we would look for alternatives like better modeling/libraries/int & 0xFF"*): every target language CAN faithfully represent every concept; literal-bits / direct-object construction is always available as the floor (universal but expensive). **The cost lens (T-CostLens-Composition) is the discriminator** that orders the faithful-representation alternatives by per-primitive realization cost: `cost_lens_reads_target_realization` reads the target language spec's per-primitive realization cost; `coercion_cost_equals_complexity_by_construction` makes "the choice between faithful representations IS the cost-lens application" structural-not-conventional. **Grounding selects the lowest-cost faithful representation** per target by reading the cost lens output. For Python u8: cost-tier 1 = `numpy.uint8` (low cost when numpy available; native operations align), cost-tier 2 = `ctypes.c_uint8` (always-available, medium cost), cost-tier 3 = `int & 0xFF` discipline (universal floor, high cost). All three are faithful (carry the 8-bit-natural-number structure); cost lens orders them; Grounding picks tier-1 when available, falls back through tiers as needed. **Faithful representation > target-conditioned omission** AND **cost lens > heuristic-or-bridge selection**.

   **Implication for `docs/design-numeric-construction.md:194-198`**: that doc's existing Python row (`Nat<N>` for any N → Python `int` with "size hint informs runtime checks but doesn't change carrier") is **NOT a faithful representation** — it drops the width-structure to opaque metadata; the cost lens cannot read what isn't there. Reframing routes to **T-Numeric-Construction / Grounding lane paydown**: replace the size-hint-on-int row with a **cost-tiered table** of faithful representations (numpy.uint8 / ctypes.c_uint8 / int-and-mask-discipline) so the cost lens reads structural facts, not metadata. Not bundled here per scope; tracked-not-silent and routed to the cost-lens-bearing lane.

**S3 dispatch unblocked on machine-constraint side** (independent of T-Numeric-Construction algebra-side Option A vs B selection per PR #1815 — interaction semantics carries through under either). **Sub-decision 2's `.dag`-generated discipline** means S3 worker brief must specify the interaction substrate as `.dag`-declared with any Rust counterpart generated, not hand-authored.

**Dispatch trigger**: now (foundational; gates Class 1 closure).
**Closure predicate**: §1.8 gate #60 `substrate_gap_parser_grammar_closed`. **Pass = concept faithfully modeled** (per sub-decision 5 above): substrate carries algebra and machine-constraint as independent axes; interaction substrate is `.dag`-generated; parser handles generic `Compose<...>` interaction syntax; concrete types are derived consequences (not primary entities). The 3-pair set (`Int<64>` / `Real<64>` / `Nat<8>`) lowering without v2-fallback is **minimum existence-proof evidence**, NOT the closure target — once the substrate carries the separation faithfully, all valid pairs work by construction. **§1.4 conjunctive-closure rule still binds** (per claude review observation): "concept faithfully modeled" replaces pair-counting as the success criterion but does NOT relax the conjunctive form (representative gap-test executes AND class-bridge enumeration = 0); see `r3-structure.md` §"Acceptance" + `r3-program-plan.md` §1.4 for the conjunctive rule. Sub-decision 5 narrows the *what counts as Pass*, not *how many things must hold*.
**Cross-lane**: T-Numeric-Construction provides algebra side (in flight); MachineConstraint<C> is new substrate.

### S4 — Workflow* family carriers (Class 4)

**Existing ontology audit prerequisite (per codex BLOCKING 2026-05-06)**: `dsl/extdeps/github/actions.dag` (218 lines) already declares `Workflow`, `WorkflowTrigger` (sum: `Push | PullRequest | Schedule | WorkflowDispatch | WorkflowCall`), `Job`, `Step`, `MatrixStrategy`, `RunnerSpec`, `WorkflowPermissions`, `ConcurrencySpec`, `DispatchInput`, etc. The S4 worker brief MUST audit `extdeps.github.actions` first and either (a) extend / refine the existing carriers via T-Workflow-As-Data lens-consumption-shape additions (preferred per `feedback_audit_adjacent_authority_first` + `feedback_parallel_representation_debt`), or (b) explicitly dissolve `extdeps.github.actions` with a migration path before introducing parallel carriers. **Names below are audit targets / preliminary additions, NOT a fresh ontology** — they reuse `extdeps.github.actions` types where they map cleanly and propose new types only where the lens-shape (e.g., observation-driven `WorkflowObservationAnchor` per Substrate Mgr design stance at gunbc#1130 comment-4374109666) doesn't exist yet.

**Scope (audit + delta against `extdeps.github.actions`)**:
- `WorkflowTrigger` — already exists; potential refinement only (e.g., typed `Cron<Schedule>` over current `Schedule { cron: String }`)
- `WorkflowStep` — already exists as `Step`; reuse name
- `WorkflowMatrix<Axes>` — already exists as `MatrixStrategy`; potential parametric refinement
- `RunnerResource<C>` — already exists as `RunnerSpec`; reuse or refine
- `Workflow<Trigger, Steps, Resources>` — already exists as `Workflow`; potential parametric refinement only
- `WorkflowSecret<Name>` — NEW (no equivalent in `extdeps.github.actions`); provider-typed, opaque-at-rest, scoped-by-step
- `WorkflowObservationAnchor` + observation/measurement carriers — NEW (Shared External Attachment Pattern per Substrate Mgr design stance; not in current actions.dag)

**Dispatch trigger**: post-T-Lens-Behavioral-Parity COMPLETE (per `r3-structure.md` §"Dependency on R2"; lens consumption needs lenses COMPLETE) **AND** post-`extdeps.github.actions` audit-and-delta receipt (Substrate Mgr to surface audit before worker dispatch).

**Audit-and-delta receipt (2026-05-06):** Landed on [gunbc#1771](https://github.com/gunb-ai/gunbc/issues/1771#issuecomment-4391856805) (quick-ferret-413); tracked assignment [#1873](https://github.com/gunb-ai/gunbc/issues/1873) **CLOSED**. The conjunctive trigger’s audit half is satisfied; **T-LBP COMPLETE** remains the remaining prerequisite before S4 substrate dispatch fires.

**Closure predicate**: §1.8 gates #53 (workflow_substrate_carriers_landed), #54 (timing_lens_carrier_landed), #55 (shared_external_attachment_pattern_documented), #56 (ci_workflow_modeled_as_dag), #62 (substrate_gap_file_ingestion_closed), #63 (substrate_gap_workflow_scheduling_closed).

### S5 — Variant-aware projection metadata carrier

**Scope**: typed REST response projection carrier for coproduct response bodies. Per Substrate canvas C1 + Anthropic #1702 + Grounding G5.

**Dispatch trigger**: gates #1702 re-dispatch + 3 follow-up paydown PRs (Anthropic / OpenAI ChatCompletion / OpenAI Responses).
**Closure predicate**: §1.8 gates #29-#30 (T-Anthropic-Wire 2 gates) + Q-Anthropic-Variant-Aware closure-scope ratification (Director — carrier-only vs 3 paydowns; default = all 3 per Brian no-post-R3-deferral).

### S6 — `EmissionPathProjection` carrier (L6)

**Scope**: per `docs/briefs/r3-substrate-l6-per-row-projection-routing-decision.md` Option 2 — `EmissionPathProjection` keyed by `MethodTemplateContractKey`, `List<EmissionCell>`, target on the key, carrier empty first.

**Dispatch trigger**: now (non-Evaluator-gated; can fire parallel to T-E-P-Producer-Broadening per B4).
**Closure predicate**: gates Grounding L6 row population (post-merge); cross-lane handoff to Grounding Mgr per #1745.

### S7 — PR-F (BoundDeclaration consumer + Rust ReferenceModel<T>)

**Scope**: substrate carrier landing per Substrate canvas B3 + Grounding G1.

**Dispatch trigger**: post-#1782 merge (Substrate Mgr 2026-05-06).
**Worker pin** (per Q-PR-F bandwidth-aware routing + Substrate Mgr partition 2026-05-06): **loyal-wolf-828**; brief shape = T-E-P-Producer-Broadening adjacent precedent (Substrate Mgr authors brief; valiant-ant-72 reserved for S3 `MachineConstraint<C>` implementation post-design).
**Closure predicate**: unblocks Grounding T-Ground-Rust Phase 1 (`u128` / `isize` / `usize` / walker arms / pilot mirror).

### S8 — `ApproximateField<F>` Float migration (T-NumericConstruction)

**Scope**: Float32/Float64 migration from `Field<Word*>` to `ApproximateField<F>` per Substrate canvas B2 + Grounding G2; plus Real/base-carrier convention for `F`.

**Dispatch trigger**: **parallel with S3** (per Substrate Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074769 — `MachineConstraint<C>` and `ApproximateField<F>` are independent axes: machine width vs algebra approximation). Both Mgr-tier design now; cross-reference at brief-landing.
**Closure predicate**: unblocks Grounding Rust float rows + Class 1 Real<N> demonstration; advances §1.8 gates #17-#24 (T-Numeric-Construction 8 gates).

### S9 — T-Numeric-Construction Mgr-tier brief authoring (B2 ratified)

**Scope**: comprehensive T-Numeric-Construction worker brief covering 13 in-scope types (Int / UInt / Float direct + 10 Int-inherited refinements: Char / EpochMs / Duration / Milliseconds / Seconds / RetryCount / HttpStatus / Port / PositiveInt / NonNegativeInt) + Slice 2 Nat-alignment migration.

**Dispatch trigger**: now (parallel-author authorized per B2).
**Closure predicate**: §1.8 gates #17-#24.

### S10 — T-E-P-Producer-Broadening dispatch (foundational)

**Scope**: brief drafted in PR #1782 (`docs/briefs/r3-t-e-p-producer-broadening-worker.md`); broaden per-call DescentEvidence / CallPattern / SubValueRelation producer coverage.

**Dispatch trigger**: post-#1782 merge (per B3 RATIFIED).
**Worker pin** (per Substrate Mgr 2026-05-06): **quick-koi-190** (currently on #1799 termination-contract; T-E-P consumes descent-evidence, natural follow-on).
**Closure predicate**: §1.8 gates #76-#78 (3 T-E-P gates) + foundational for cascade T-LBP → T-LAS → T-WAD → T-LSA.

### S11 — Slice C of #1795 follow-up (B1 ratified)

**Scope**: Slice C residual covers ~10 files (per smart-ram-167 enumeration); paired prose+regen edits per #1795 path-(a) precedent.

**Dispatch trigger**: post-#1795 (Slice A) + #1801 (Slice B) merge per B1 RATIFIED prose+regen bundling (Substrate Mgr 2026-05-06 — cascade-clearance gated).
**Worker pin** (per Substrate Mgr 2026-05-06): **smart-ram-167** (Slice B precedent owner; pattern-familiar).
**Closure predicate**: ROADMAP :425 row PARTIAL → Retired (per Substrate canvas C3); Q-Slice-C-Retirement-Receipt resolution.

### S12 — F2 + F8 doc-sharpening PR (B6 ratified)

**Scope**: F2 (`.v3` filename-suffix grammar dispatch) + F8 (bootstrap load-order/exclusion authority); single bundled Mgr-tier doc-sharpening PR per Substrate canvas B6.

**Dispatch trigger**: post-:425 close (sequenced after S11).
**Closure predicate**: ROADMAP F2/F8 rows retire.

### Substrate demonstration gates (per §1.6 minimum bar)

Per Substrate Mgr partition response 2026-05-06: **demonstration gate scope folded into parent worker brief Acceptance bullets**, NOT separate dispatches. Each gate becomes an Acceptance bullet on the parent lane's worker brief.

| Gate (§1.8 row) | Parent brief | Demonstration scope (Acceptance bullet) |
|---|---|---|
| #67 `numeric_construction_demonstration` | T-Numeric-Construction worker brief (S9) | end-to-end `Int<32>` + `Real<64>` round-trip |
| #68 `anthropic_wire_demonstration` | T-Anthropic-Wire / variant-aware brief (S5 follow-on) | full request/response cycle vs deterministic mock |
| #70 `cost_lens_demonstration` | T-CostLens-Composition brief | ≥2 algebra-instances + ≥1 recursive call + observable cost-bound |
| #72 `e_p_producer_demonstration` | T-E-P-Producer-Broadening worker brief (S10) | call-site produces full descent evidence at runtime |
| #73 `lens_behavioral_parity_demonstration` | T-LBP scope-calibrated brief (post-S2) | each lens demonstrates + matches frozen v2-oracle cementing-test snapshot (per openai-pro F2 — frozen, NOT live v2) |

---

## §2. Verification Mgr (cool-owl-579, gunbc#1740)

**Lane scope**: T-V-L4-L7-Direct + T-V-L5-Corpus + T-Free-Consequences-Demonstration + T-Tests-As-Data-Completeness + T-Lens-Self-Application + cross-program portion of T-Lens-Behavioral-Parity + cross-program portion of T-Lens-Application-Surface + `bridge_retirement_ledger_zero` audit gate.

**Worker partition** (per Verification Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074816):

| Track | Worker | Items | Status |
|---|---|---|---|
| **A — executable / ledger** | bold-crane-790 (#1748) | V1 (TC1 hold pending Q-PAFS + EVAL-3) + V6 (active) | V6 ACTIVE; V1 standby |
| **B — corpus / demos / data** | cool-heron-521 (#1766) | V2 + V4 + V5 (all post-R2-Evaluator-gated) | Prep now (design + skeleton); closure waits triggers |
| **C — Mgr-reserved / cross-lane** | cool-owl-579 (this lane) | V3 (post-cascade) + V7 (hold) | V3 sequenced per §8; V7 design-first only |

### V1 — Pattern-A executable cluster (TC1 first; Q-PAFS Path A ACCEPTED 2026-05-06)

**Scope**: 4 NEW Pattern-A executable gates (DimensionReport-typed cluster) need consumer infrastructure here in V1:
- TC1 (`tc1_eta_equivalence_executable`): static representative via E6-G1.a (PM-default; Q-Pattern-A-First-Slice-Subscope) — runtime prereq from Evaluator EVAL-3
- TC2 (`tc2_church_rosser_executable`): second strategy/input order + strategy-keyed report
- TC3 (`tc3_pattern_a_second_mover_executable`): Descent execution proof (E5) + eval-step producer
- RustDagIsomorphism (`rust_dag_isomorphism_executable`): shape-report producers

**Note (per codex BLOCKING 2026-05-06)**: the framework adds **5 NEW Pattern-A executable gates total** at PR #1809 (per `r3-program-plan.md:88`). The 5th — `symbolic_cost_expr_equals_executable` (§1.8 gate **#40**) — is **SymbolicCost-typed** (not DimensionReport-typed) and belongs to **T-CostLens-Composition** lane, not V1's TC cluster (per `r3-program-plan.md:755`: V1's DimensionReport-unblock fixes TC1/TC2/TC3 cluster but does NOT automatically fix SymbolicCostExprEquals — different predicate family, distinct runner work). Tracked separately under T-CostLens-Composition.

**Dispatch trigger**: **TC1 first slice (Path A) DISPATCH UNBLOCKED 2026-05-06** — Q-PAFS / Q-Pattern-A-First-Slice-Subscope / Q-EVAL-Lens-Fold-First-Slice ACCEPTED per Brian directive ("approved path A countersign"). TC1 static representative via E6-G1.a; Verification Mgr authors V1 worker brief; runtime prereq satisfied as Evaluator E3 (E6-G1.a static lens fold) lands in same release step.
**Closure predicate**: §1.8 gates #11-#14 (4 Pattern-A executable, DimensionReport-typed) + cascades to BridgeLedgerZero net-shrink (gate #36). 5th gate (#40 `symbolic_cost_expr_equals_executable`, SymbolicCost-typed) closes via T-CostLens-Composition lane.

### V2 — L4 corpus + L7 exhaustive coverage

**Scope**: l4_emit_eval_match (gate #9) + l7_algebraic_laws_witnessed (gate #10) — currently DECLARED (skeleton/staged). Closure bar = exhaustive per-(algebra, inhabitant, law) coverage per `r3-structure.md` 2026-05-02 fold-in.

**Dispatch trigger**: post-R2-Evaluator landed + Shape A grounding for L4 corpus.
**Closure predicate**: gates #9-#10 reach CONSUMER_LANDED + PASSING.

### V3 — T-Lens-Self-Application stronger demo (Y-2 RATIFIED)

**Scope**: per Research PM Y-2 — strengthen `lens_self_application_demonstrated` to tie lens output to emission gate (CI workflow → TestClaim → fails → gunbc emission refuses). Demonstrates 4 gates simultaneously + WEDGE-CORE-CLAIM Part B opportunistically.

**Dispatch trigger**: post-T-Workflow-As-Data + T-Lens-Application-Surface (per cascade).
**Closure predicate**: §1.8 gates #57-#59 (3 T-Lens-Self-Application gates) + 4-fold demo overlap (Self-App + Workflow-As-Data + Tests-As-Data + integration_testgen).

### V4 — T-Tests-As-Data-Completeness lane work

**Scope**: every Rust test ports to `.dag` TestClaim or generated target-language test code; ForAll/Exists quantifier substrate; ProgramGenerator carrier; cementing test discipline.

**Dispatch trigger**: post-R2-Evaluator (test-execution runtime needed).
**Closure predicate**: §1.8 gates #84-#87 (4 T-Tests-As-Data gates) + #74 (tests_as_data_demonstration).

### V5 — T-Free-Consequences-Demonstration (10 gates)

**Scope**: 10 demo gates — auto-parallelism × 3 + auto-loop-parallelism × 3 + auto-memoization × 2 + cross-target-optimization × 2.

**Dispatch trigger**: post-R2-Evaluator + T-CostLens-Composition (cost-related claims).
**Closure predicate**: §1.8 gates #43-#52.

### V6 — `bridge_retirement_ledger_zero` audit gate (ACTIVE)

**Scope**: unified ledger reports 0 named identity bridges remaining. 5 sub-bridges retire (2 Substrate-owned + 3 PB-owned).

**Dispatch trigger**: ongoing (audit cadence). **Worker** (per Verification Mgr 2026-05-06): bold-crane-790 — ACTIVE.
**Closure predicate**: gate #36 reaches PASSING.

**Doc / grep alignment**: §3 **P4** (warm-ant-877) + [`docs/briefs/r3-v-bridge-retirement-ledger-zero-audit.md`](briefs/r3-v-bridge-retirement-ledger-zero-audit.md) §"Grep hygiene" maintain SB5/6-style enumeration receipts on the same three DAG paths (`bridge_ledger.dag`, `r3_bridge_retirement_ledger_zero.dag`, `verification.dag`). PB and Substrate retire evidence; Verification owns the ledger-zero gate — appendix hygiene complements execution audits without shifting gate ownership.

### V7 — ValueBody isomorphism gate design (Q-ValueBody-Isomorphism)

**Scope**: per Verification canvas V6 + `feedback_isomorphism_or_generation_for_mirrors` — Rust↔.dag mirror conformance gate. Verification test-only pressure; Substrate Mgr authority on mirror.

**Dispatch trigger**: pending Director scope-decision (in-R3 vs post-R3 carry).
**Closure predicate**: NEW gate `value_body_isomorphism_gate_active` if added; design first.

**Verification Mgr PRE-AUTH worker brief:** [`docs/briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md`](briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md) — working consumer name `value_body_substrate_mirror_isomorphism_executable` (§1.8 enumeration **after** Q-row ratification; **INVARIANTS** §P2).

---

## §3. PB Mgr (neat-bear-351, gunbc#1742)

**Lane scope**: T-LensProducer-Retirement + T-FixedPoint + T-Tier3-Dissolution + T-V2-Retirement + 3 PB-owned bridges.

**Worker partition** (per PB Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385075315):

| Schedule row | Worker / inbox | Dispatch posture |
|---|---|---|
| **P1** T-LensProducer-Retirement | sleek-eagle-514 (#1768) — PR #1805 path-1 + sub-briefs gunbc#828 (Sub1/2/3) | Roadmap intent: Sub1–Sub3 retirement stack (`lens_apply` / `lens_testgen` / `regen_lens`). **Live dispatch:** Sub1 **parked** until Item 4 / Row 4 / canonical-lens prerequisites — PB Mgr STOP on [#1768](https://github.com/gunb-ai/gunbc/issues/1768). |
| **P1** parallel doc spine | zesty-ram-316 (#1769) — PR #1806 regen_lens audit + Sub2/Sub3 brief threads | post-R2-Evaluator + PB-1 bin-shim pattern; parallel doc-spine on P1 |
| **P4** bridge appendix / cross-links | warm-ant-877 (#1770) — grep / ledger hygiene against `bridge_ledger.dag` / `r3_bridge_retirement_ledger_zero.dag` / `verification.dag` | Lockstep with §2 **V6** `bridge_retirement_ledger_zero` audit cadence (bold-crane) + P1/P2 sequencing; `include_str!` post-T-FixedPoint per Q-Bridge-Retirement-Sequencing-Authority |
| **P5** F2 + F8 doc-sharpening | PB Mgr coordinates consumer-side with Substrate S12 owner | No duplicate PR; PB-named co-author OR comment-only on Substrate's PR (PM ratify if explicit co-author shape needed) |
| **P2** T-FixedPoint | HOLD until P1 + SG-0 zero per F1 | No worker spawn that pretends LP is done; TC3 text stays proposal-side |
| **P3** T-V2-Retirement | HOLD on broad ~79 .rs sweep until P2 + LP + T-Numeric-Construction `Int<N>` clear | Q-V2-Retirement-Boundary-Matrix split visible (PB vs Grounding vs Debt-Paydown) |

*Schedule rows summarize roadmap intent and stable sequencing; **worker inbox STOP/park** states supersede “next slice” wording when PB Mgr accepts them (Sub1 example above).*

**§2.2 sequencing authority — HARD DAG (PM ratification 2026-05-06)**: per PB Mgr ask + r3-structure.md §"Lane structure" → T-FixedPoint row "R2-close dependency: SG-0 zero from T-LensProducer-Retirement" — sequencing is **hard DAG, not staffing parallelism**. T-FixedPoint cannot complete until T-LP-Retirement completes (SG-0 zero is structural precondition; not just resource sequencing). Plan §2.2 sequence is canonical authority. Per r3-structure.md T-FixedPoint row: "SG-0 zero from T-LensProducer-Retirement" is named explicit dependency.

### P1 — T-LensProducer-Retirement (3 hand-Rust files retired)

**Scope**: lens_apply.rs + lens_testgen.rs + regen_lens.rs retired via PB-Runtime + PB-1 patterns.

**Dispatch trigger**: post-R2-Evaluator + PB-1 generated bin-shim pattern.
**Closure predicate**: §1.8 gates #5-#8 (3 retirement gates + sg0_non_test_zero) + cascade to T-FixedPoint + Class 5 (#64).

### P2 — T-FixedPoint completion

**Scope**: `compiler.dag` self-compile fixed-point; bit-identical stage0 + emitted artifacts.

**Dispatch trigger**: post-T-LensProducer-Retirement → SG-0 zero (corrected sequencing per PB Mgr F1).
**Closure predicate**: §1.8 gate #16 reaches CONSUMER_LANDED at R3 horizon (already CONSUMER_LANDED at R1).

### P3 — T-V2-Retirement (post-FP+LP cascade)

**Scope**: ~79 .rs + ~32 .dag files in src/v2/ retired; workspace member removed; bootstrap routes through PB-Runtime trampoline only.

**Dispatch trigger**: post-T-FixedPoint + T-LensProducer-Retirement + T-Numeric-Construction `Int<N>` (for parser path).
**Closure predicate**: §1.8 gates #41-#42 + #71 (v3_self_host_demonstration).
**Cross-lane**: per Q-V2-Retirement-Boundary-Matrix — PB owns v2 directory + test-consumer + PB-Runtime trampoline; Grounding owns emit-shim + Coercion-Fold; Debt-Paydown owns drift items.

### P4 — 3 PB-owned bridges

**Scope**:
- canonical lens-name dispatch retires alongside T-LensProducer-Retirement
- include_str! side channels retire post-T-FixedPoint (per Q-Bridge-Retirement-Sequencing-Authority)
- patch_lower_helpers_* residual post-Tier-2

**Dispatch trigger**: per per-bridge prerequisite chain.
**Closure predicate**: contributes to §1.8 gate #36 (bridge_retirement_ledger_zero) + #69 (bridge_retirement_demonstration).

#### P4 — Verification V6 alignment + grep hygiene (PR #1804 SB5/6 + PR #1810 schedule)

**Cross-lane contract** ([gunbc#828](https://github.com/gunb-ai/gunbc/issues/828), PR [#1810](https://github.com/gunb-ai/gunbc/pull/1810) §3 table): PB (and Substrate, for Substrate-owned bridges) **lands retirement evidence** in owning programs; Verification **audits** unified `bridge_retirement_ledger_zero` via production `BridgeLedgerZero` + integration harness (**§2 V6**, bold-crane-790). Doc work here does not substitute for gate execution — it keeps PM compile passes **grep-aligned** with live ledger + predicate surfaces.

**Authoritative paths** (same triple as Phase-2 SB5/6 bridge appendix on PR [#1804](https://github.com/gunb-ai/gunbc/pull/1804)):

- `src/v3/std/bridge_ledger.dag` — row truth for the five named bridges + statuses
- `src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag` — fixture wiring for `bridge_retirement_ledger_zero`
- `src/v3/std/verification.dag` — `BridgeLedgerZero` and adjacent verification predicates that ride the harness

**Repeatable enumeration gate** (squash-merge PR numbers from subjects; per-PR file receipt):

```bash
git log origin/main --format='%s' -- \
  src/v3/std/bridge_ledger.dag \
  src/v3/compiler/tests/fixtures/r3_bridge_retirement_ledger_zero.dag \
  src/v3/std/verification.dag

gh pr view <N> --repo gunb-ai/gunbc --json files,title
```

**Sequencing (ratified HARD-DAG)**: PB-owned `include_str!` side-channel retirement stays **post-T-FixedPoint** per Q-Bridge-Retirement-Sequencing-Authority — ledger rows remain **honestly Open** until structural receipts land; schedule ordering does not relax `BridgeLedgerZero` truth.

### P5 — F2 + F8 doc-sharpening PR (per S12 + B6)

**Cross-lane**: Substrate-led; PB consumer-side coordination.

### PB demonstration gates

| Gate (§1.8 row) | Demonstration scope |
|---|---|
| #65 `tier3_dissolution_demonstration_executes` | Tier3-mirror-consumer `.dag` runs end-to-end via Evaluator |
| #66 `lens_producer_retirement_executable_witness` | DEFERRED to Row-4 receipts per PB Mgr F3 |
| #69 `bridge_retirement_demonstration` | typed-identity-surface in production code |
| #71 `v3_self_host_demonstration` | bootstrap PB-Runtime trampoline runs end-to-end |

---

## §4. Evaluator Mgr (merry-gull-128, gunbc#1743)

**Lane scope**: R3 Evaluator continuation work — runtime prereqs for Pattern A predicates + lens runtime execution + constructor/field-call execution per E6 G0a/G0b/G0c/G0d work.

**Worker partition** (per Evaluator Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385081532):

| Item | Status | Routing |
|---|---|---|
| **E1** E6-G0d constructor runtime execution | **DISPATCHED 2026-05-06** | valiant-carp-10 (#1767 #issuecomment-4385079490). Scope: evaluator-only `src/v3/compiler/src/lib.rs`; brief authority = #1784 G0d worker brief |
| **E2** E5 Descent termination contract consumer | HELD | Trigger = Substrate carrier `descent_execution_proof` landing via quick-koi/quick-crab path; sharp-ibex #1799 remains STOP/audit, not consumer wiring yet |
| **E3** E6-G1.a static lens fold | **DISPATCH UNBLOCKED 2026-05-06** | Q-PAFS / Q-EVAL-Lens-Fold-First-Slice ACCEPTED (Path A) per Brian directive ("approved path A countersign"); Evaluator Mgr authors E3 worker brief + dispatches. Lands in same release step as Verification V1 (TC1 first slice). |
| **E4** E6-G1.b generic dispatch | HELD | Trigger = post-G1.a + post-Substrate X1.b S1/S3 |
| **E5** X1.b S1 TransformDispatch coordination | **DONE** | Cross-lane status update sent to Substrate Mgr (quick-crab-830) at #1739 #issuecomment-4385080388 |

**Additional state notes**:
- #1784 G0d brief at head `61c90a65`: `fmt` / `ci` / `v3` green; `self_host_ratchet` in progress post-main merge. Does NOT block E1 dispatch — brief content stable + approved; final merge waits on check.
- #1799 E5 STOP packet: `fmt` / `ci` / `v3` green; `self_host_ratchet` in progress. Held semantically behind Substrate termination contract.
- warm-dove #1778: passing/held; no new code work assigned (existing PR needs Director/PM disposition).

No additional PM/Director ratification needed from Evaluator for E1/E5. **E3 dispatch unblocked 2026-05-06** — Q-PAFS Path A ACCEPTED per Brian directive; Evaluator Mgr authors E3 worker brief and dispatches in same release step as Verification V1.

### E1 — E6-G0d constructor runtime execution (RATIFIED Mgr executes)

**Scope**: authorize evaluator implementation slice in `src/v3/compiler/src/lib.rs` for non-Arrow constructor `Callable`, using existing `Value::RecordValue` / `Value::VariantValue`. Brief at `docs/briefs/r3-pr-e6-g0d-constructor-runtime-execution-worker.md` (#1784 ready).

**Dispatch trigger**: now (Evaluator Mgr executes per Q-EVAL-G0d-Dispatch RATIFIED).
**Closure predicate**: unblocks Pop A constructor behavior + honest `Witness`/`DimensionReport` construction + static lens fold.

### E2 — E5 Descent termination contract consumer (RATIFIED post-Substrate carrier)

**Scope**: `eval_loop` consumes `descent_execution_proof` token from substrate carrier authored via quick-koi-190 (already authorized through quick-crab).

**Dispatch trigger**: post-Substrate carrier landing.
**Closure predicate**: unblocks TC3 (gate #13) + any lens fold traversing Descent loops.

### E3 — E6-G1.a static lens fold (Pattern A first slice — Q-PAFS Path A ACCEPTED 2026-05-06)

**Scope**: single static top-level `data ... : Lens<C>` representative under G1.a. Per Q-Pattern-A-First-Slice-Subscope (TC1-static-rep first, ratified by Brian directive 2026-05-06).

**Dispatch trigger**: **DISPATCH UNBLOCKED 2026-05-06** — Q-PAFS / Q-EVAL-Lens-Fold-First-Slice ACCEPTED (Path A: G1.a static representative). Evaluator Mgr authors E3 worker brief and dispatches; lands in same release step as Verification V1 (TC1 first slice).
**Closure predicate**: §1.8 gate #11 (tc1_eta_equivalence_executable).

### E4 — E6-G1.b generic dispatch (post-G1.a)

**Scope**: generic `fold_lens<C>` for non-static lens applications; cascades from G1.a + X1.b S1/S3.

**Dispatch trigger**: post-G1.a + post-Substrate X1.b S1/S3 carrier.
**Closure predicate**: cascades to TC1-generic + TC2 + TC3 (gates #11-#13).

### E5 — X1.b S1 TransformDispatch coordination

**Scope**: cross-lane status update on E6-G0c → unblocks Substrate X1.b dispatch.

**Dispatch trigger**: cross-lane coord with Substrate Mgr.
**Closure predicate**: Q-X1b-S1-Dispatch-Coord resolution.

---

## §5. Grounding Mgr (bold-ferret-748, gunbc#1745)

**Lane scope**: L6 CrossTarget-Meta + T-Ground-Rust + Coercion-Fold retirement + emit shim consumption + F10 cleanup + Anthropic re-dispatch.

**Worker partition** (per Grounding Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385080863):

| Item | Status | Worker / trigger |
|---|---|---|
| **G1** L6 row population | HELD | Trigger = Substrate S6 `EmissionPathProjection` carrier landing |
| **G2** T-Ground-Rust full coverage | HELD | Trigger = Substrate S7 PR-F + S8 Float migration / Real base-carrier for float rows; #1783 remains draft as dispatch-guide staging artifact |
| **G3** Coercion-Fold scratch retirement | HELD | Trigger = executable LanguageSpec projection |
| **G4** F10 `install_hint` cleanup | **DONE / NO-OP 2026-05-06** | silent-badger-711 (#1774) verified shape already present at HEAD `cde245713f89a08e11c4242e4bb1cd98e098a881`; no diff, no PR needed; `cargo test -p v2-compiler-tests` 482 passed / 0 failed. Closure signal at [gunbc#846 #issuecomment-4387302181](https://github.com/gunb-ai/gunbc/issues/846#issuecomment-4387302181). |
| **G5** Anthropic #1702 re-dispatch | HELD | Trigger = Substrate S5 variant-aware projection metadata carrier + Q-Anthropic-Variant-Aware closure-scope ratification |

No additional PM/Director ratification needed for G4. For G1/G2/G3/G5, schedule already names sufficient upstream triggers; Grounding dispatches as soon as triggers land. Grounding lane is largely consumer of Substrate work — most items HELD on Substrate cascade.

### G1 — L6 row population (post-EmissionPathProjection)

**Scope**: per `docs/briefs/r3-l6-emission-path-projection-substrate-worker.md` consumer-side; populate L6 projection rows + convert `coverage.rs`.

**Dispatch trigger**: post-Substrate S6 (EmissionPathProjection carrier landing).
**Closure predicate**: L6 cross-product fold gate.

### G2 — T-Ground-Rust full coverage (post-PR-F + post-Float)

**Scope**: Phase 1 (`u128` / `isize` / `usize` / walker arms / pilot mirror) post-PR-F; Float rows post-ApproximateField<F>.

**Dispatch trigger**: post-Substrate S7 (PR-F) + S8 (Float).
**Closure predicate**: full Rust primitive grounding + L5 readiness.

### G3 — Coercion-Fold scratch retirement (post-LanguageSpec projection)

**Scope**: `LanguageSpecProjection::ScratchIntExamples` + `TargetInhabitance` retirement in `src/v3/grounding_coercion_fold`.

**Dispatch trigger**: post-Substrate/LanguageSpec projection executable.
**Closure predicate**: Q-Coercion-Fold-Scratch closure.

### G4 — F10 install_hint cleanup (DONE / NO-OP 2026-05-06)

**Scope (verified)**: `dsl/extdeps/tools.dag` `install_hint` join semantics fix (per Substrate canvas G8).

**Closure outcome**: **DONE / NO-OP at HEAD `cde2457`** per Grounding Mgr (silent-badger-711) closure signal at [gunbc#846 #issuecomment-4387302181](https://github.com/gunb-ai/gunbc/issues/846#issuecomment-4387302181). Grep-verified at `dsl/extdeps/tools.dag:91-92`:
- `install_hint` already implements `sources |> map(s => one_hint(source: s)) |> join(separator: " | ")`
- `one_hint` (line 79) is the single per-source rendering authority
- No sentinel-fold / `acc == ""` / `fold(init: "")` pattern present

Verification: `cargo test -p v2-compiler-tests -- --nocapture` 482 passed / 0 failed / 55 ignored.

**G4 closed** — no follow-up Grounding worker dispatch pending.

### G5 — Anthropic #1702 re-dispatch (post-variant-aware projection)

**Scope**: re-dispatch Anthropic Messages 200 content-block work post-Substrate variant-aware projection metadata carrier.

**Dispatch trigger**: post-Substrate S5 + Q-Anthropic-Variant-Aware closure-scope ratification.
**Closure predicate**: §1.8 gates #29-#30 (T-Anthropic-Wire) + #68 demonstration.

---

## §6. Debt-Paydown Mgr (quiet-otter-416, gunbc#1744)

**Lane scope**: ROADMAP debt-row retirement + velocity-tripwire enforcement + closure-receipt cadence + per-PR discipline rule.

**Partition** (per Debt-Paydown Mgr partition response 2026-05-06 at gunbc#846 #issuecomment-4385074935):

| Item | Status | Next step |
|---|---|---|
| **DP1** Q-Drift-Reconcile single PR | DISPATCH-NOW | Single Debt-Paydown worker thread; scope = one reconciliation PR + §1.8 row retirements feeding `r3_debt_paydown_zero_remaining` |
| **DP2** SG-0 PR-window CI gate | **IN-FLIGHT at PR #1807** | scripts/check-pr-sg0-net-shrink-discipline.sh + workflow + template + ROADMAP; closure target = §1.8 gate #75 |
| **DP3** Velocity tripwire | CONTINUOUS | Standing cadence slice (P5 ratio surfacing); recurring Mgr report + Director queue pings on threshold trips; no single landed event |
| **DP4** Closure-receipt cadence | CONTINUOUS | Lane ops / receipts discipline; feeds `r3_debt_paydown_zero_remaining` Pass surface |
| **DP5** #1566 rollup hygiene | HOLD pending DRAFT close | Deferred worker; no idle burn until trigger clears |

No §6 items currently Director-blocked; §7 CP items stay out-of-lane unless they become prerequisites.

### DP1 — Q-Drift-Reconcile single PR

**Scope**: 3 drift items single-PR reconciliation:
- declaration_by_name ROADMAP↔ledger drift (ROADMAP says retired by #1638; ledger marks Open)
- #1499 transitional fence ledger-row gap
- CollectionOps / StringOps / MapOps stale ledger refresh

**Dispatch trigger**: now (RATIFIED-by-default; single Debt-Paydown worker).
**Closure predicate**: 3 ledger rows retire; advances `r3_debt_paydown_zero_remaining` predicate.

### DP2 — SG-0 PR-window net-shrink CI gate landing

**Scope**: integrate `scripts/check-pr-sg0-net-shrink-discipline.sh` into CI workflow; self-test passes.

**Dispatch trigger**: now (RATIFIED-by-default per §1 closure-gate set).
**Closure predicate**: §1.8 gate #75 (pr_anticipation_discipline_ci_active).

### DP3 — Velocity tripwire reporting cadence

**Scope**: per `INVARIANTS.md §P5` — surface introduction:dissolution PR ratio readings to Director on cadence.

**Dispatch trigger**: now (continuous reporting).
**Closure predicate**: ongoing operational; not a single closure event.

### DP4 — Closure-receipt cadence

**Scope**: per-tracked-debt-row receipt before R3 close; aggregation feeds `r3_debt_paydown_zero_remaining` predicate.

**Dispatch trigger**: continuous.
**Closure predicate**: aggregation in `r3_debt_paydown_zero_remaining` Pass surface 2.

### DP5 — #1566 rollup hygiene (DRAFT)

**Scope**: per Debt-Paydown Mgr canvas — DRAFT rollup hygiene.

**Dispatch trigger**: pending DRAFT close.
**Closure predicate**: ledger-row retirement.

---

## §7. Cross-program / Director-tier decisions

### CP1 — Q-LBP-R3-Closeability scope-calibration (Brian/Director)

**Scope**: T-Lens-Behavioral-Parity scope decision (a accept / b reframe to 1-2 lenses for R3 / c carve to R4 with substrate-gap routing).

**Dispatch trigger**: post-Substrate S2 (LBP scope-calibration canvas).
**Closure predicate**: cascades to Q-Class-2-Chain-Break + Class 2 closure path + critical-path timeline (Research PM R-2 Q-Timeline-Risk-Alternates).

### CP2 — Q-Tier4-Inclusion (Brian directive)

**Scope**: Tier 4 in-scope vs declared-out per ctrl#1608 §6d. Brian directional framing 2026-05-06: "bring more into R3 now while we're planning" (B3).

**Dispatch trigger**: PM surfaces tradeoff (timeline impact + work scope) for Brian review.
**Closure predicate**: §1 closure criteria + §1.8 ledger expansion if included.

### CP3 — Q-WEDGE-A framing + Director engagement

**Scope**: PM provides 1-2 sentence framing on WEDGE-CORE-CLAIM Part A (ctrl#444 build-orchestration as free consequence) + thesis-edit scope. Then Director ratifies.

**Dispatch trigger**: PM authors framing first; Director engages second.
**Closure predicate**: thesis-edit gate inclusion in §1 if ratified.

### CP4 — Q-Class-6-Substrate-Extension-Lens (Brian/Director scope)

**Scope**: 3 ctrl-side substrate proposals (SchemaPreservation / ResourceCap / TotalResult) — Class 6 inclusion in R3 vs declared-out-of-scope.

**Dispatch trigger**: pending Research PM sharper framing per Director HOLD disposition.
**Closure predicate**: §1 + §4.6 expansion if included; or explicit out-of-scope-by-design.

### CP5 — PR #1794 merge (closes Q-Self-Host-Ratchet-Timeout)

**Scope**: PR #1794 (quick-ferret-413) bumps `self_host_ratchet` timeout-minutes 30 → 60. Director-cleared CLOSED-by-state disposition.

**Dispatch trigger**: now (Director cadence merge).
**Closure predicate**: Q-Self-Host-Ratchet-Timeout closes.

---

## §8. Sequencing summary

**Critical path** (per `r3-program-plan.md` §6):

```
T-E-P-Producer-Broadening (S10; foundational)
   ↓
T-Lens-Behavioral-Parity (Substrate + Verification cross-program)
   ↓
[ T-Lens-Application-Surface PARALLEL T-Workflow-As-Data ]
   ↓
T-Lens-Self-Application (V3; demonstration)
```

**Parallel longest single-lane**: T-V2-Retirement (P3) — gated on T-FixedPoint (P2) + T-LensProducer-Retirement (P1) + T-Numeric-Construction `Int<N>` (S9 + S3 cascading).

**Verification-internal path**: T-V-L4-L7-Direct (V2) → T-V-L5-Corpus.

**Bottleneck escalations** (R3 close horizon):
- **Q-LBP-R3-Closeability** (CP1) — gates the LBP cascade; Substrate canvas (S2) prereq
- **Q-Class-6** (CP4) — potential 6th substrate-gap class adding ~1 lane equivalent
- **Q-Tier4-Inclusion** (CP2) — fundamental scope decision

---

## §9. Status update cadence

- **Daily**: Mgr-internal dispatch tracking; lane-owning Mgr updates §1.8 Status column as gates ratchet DECLARED → CONSUMER_LANDED → PASSING
- **Weekly Monday**: PM compiles §3 lane status from Mgr per-lane reports
- **Weekly Wednesday**: closure-gate progress check
- **Weekly Friday**: bridge-ledger receipt cadence

**Cross-Mgr coord**: blockers route to Director via standard cross-Mgr queue (per `r3-structure.md` §"Manager structure"). Cross-program ctrl-side dependencies route to Research PM (gunb-ai/ctrl#339).

---

## §10. References

- [`docs/r3-structure.md`](r3-structure.md) — architectural archive (95 closure gates; 18 lanes; 9 Mgrs)
- [`docs/r3-program-plan.md`](r3-program-plan.md) — forward-looking dependency graph + escalation register + §1.8 canonical 95-gate ledger
- [`docs/audit/r3-debt-sweep-2026-05-06.md`](audit/r3-debt-sweep-2026-05-06.md) — bridge-inventory framework
- Per-Mgr canvas threads: gunbc#1739 (Substrate) / #1740 (Verification) / #1742 (PB) / #1743 (Evaluator) / #1744 (Debt-Paydown) / #1745 (Grounding)
- Director inbox: gunbc#828
- Research PM inbox: gunb-ai/ctrl#339

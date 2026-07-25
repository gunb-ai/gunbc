# Enforcement intent — ask once, compile forever

*Status: draft for operator review (2026-07-02). Extends the standing threads "model §1's axioms + enforce the syllogism" and the intent-linearity draft; does not supersede either. Quantitative companion: [dependency-fidelity design](dependency-fidelity-design.md) — the same "ask once, compile forever" spine turned into a measured coverage law (CI-green ⟺ declared ≡ witnessed across an affected-set-scoped, mutation-adequate coverage set).*

## 1. The displaced cost (why this is on-dial)

The operator has repeatedly expressed one standing intent — *enforce complexity / avoid objective quadratics / make lenses real, repo-wide* — and the system keeps treating each utterance as a fresh local request. The recurring manual join is:

> what is enforced · where · over which corpus · by which lens · is it live · does it reach v1 · does it apply to itself?

That join is **redundant governance work** (§2): paid again on every new lens, gate, PR, and roadmap node. It is the meta-instance of the very bug we chase in the object code — an intent that must be re-discovered because it was never made load-bearing. The deliverable's denominated benefit (§6) is *this displaced cost*: the operator stops performing the missing meta-lens.

## 2. DFS first — what already exists (do NOT re-mint)

Per §2/§3, before minting vocabulary we map onto the concept DAG. The carriers the naive proposal would fork **already exist**:

- **Decidability boundary** — `v2.lens.common.construction_justification :: ConstructionJustification` (`WallNow` / `WallAfterGrounding` / `RatchetForever`). Reuse verbatim; do not add a parallel `declared_boundary` enum.
- **Lens contract (V0)** — `v2.lens.registry :: LensRegistryEntryV0 { lens_id, module_path: Bound | Unbound }`. This is the seed of `LensContract`; **extend it**, don't fork a new type.
- **Scope / subject set** — the per-lens `subject_roster` pattern (`v2.lens.idempotency.subject_roster`, `v2.lens.ownership.subject_roster`, …). Reuse the pattern; the scope of an intent is a `subject_roster` expression.
- **Self-application / fractal** — `v2.lens.intent_linearity` + `dsl/gunbc/plans/self_applying_lenses.dag` + `axiom_syllogism_lens.dag`. The "apply the lens to the intent expression itself" layer is already being modeled here; the enforcement-intent gate **consumes** it, it does not re-invent it.
- **Inertness backstop** — `dsl/test/claim/inert_lens_hygiene_witness_test.dag` (DESIGN §6's executable "wired-or-deleted" check, already live over the corpus). The meta-gate **strengthens** this from "is the lens wired at all?" to "does the lens satisfy the *declared scope* and self-apply?"

**Net: this is an extension, not a greenfield build.** That materially shrinks the work and keeps single-authority intact.

## 3. The one genuinely missing thing: the join

Two additions, both minimal:

### 3a. `LensContract` = extend `LensRegistryEntryV0`
Add to the existing registry entry the fields that make "is it enforced?" decidable:

```
type LensContract {                        // grows LensRegistryEntryV0, one concept
  lens_id: LensIdV0
  module_path: LensModulePathV0            // already present
  claimed_scope: SubjectRoster             // reuse the subject_roster carrier
  mode: EnforcementMode                    // Advisory < AuditOnly < Blocking (§12)
  consumer: ConsumerKind                   // FloorGate | MergeAdmission | … | None (§12); "referenced by synthesis" == None
  red_control: Present { witness: QualifiedName } | Absent
  self_application: SelfApplies | Exempt { reason: String }
  boundary: ConstructionJustification      // reuse — WallNow / WallAfterGrounding / RatchetForever
}
```

### 3b. `StandingIntent` — the operator's durable directive (the only new authority)
A row, not a doc paragraph — the mark on the carrier is the authority (§6):

```
type StandingIntent {
  id: String                               // "complexity.repo-wide"
  property: LensIdV0                        // REUSE the existing lens-property coproduct (src/v2/lens/registry.dag:10 — Complexity | Cost | …); do NOT mint a PropertyClass nickname (§3)
  desired_scope: SubjectRoster              // default = WHOLE CORPUS (see §5)
  required_subjects: List<QualifiedName>    // the receipts that MUST be covered
  default_mode: EnforcementMode            // Advisory < AuditOnly < Blocking (§12)
  required_consumer: ConsumerRequirement   // AnyLiveConsumer | OneOf | Exact (§12) — NOT hardwired to FloorGate
  self_application_required: Bool
  fallback_when_unavailable: Refuse | Unknown | AuditOnlyWithReason  // never silently OK (§5)
}
```

`StandingIntent` references the property and subject carriers rather than restating them; it is the single home for "what the operator wants," so a future PR can refine the *implementation* but cannot silently narrow the *intent*.

**Admission rule — `StandingIntent` is durable governance, not a preference bucket.** A row is admissible only if it (a) *recurs* across PRs, (b) has a *named displaced cost* (§6), (c) has a *scope*, (d) names an *enforcement mechanism class*, and (e) *can produce a receipt*. "make this cleaner" / "prefer better names" fail (a) and (e); the five initial rows pass. This keeps the carrier from degenerating into a global lint dump.

## 4. The meta-gate (decidable — a wall now)

```
enforcement_intent_gate(intent: StandingIntent, contracts: List<LensContract>, receipts: CoverageReceipts) -> Witness<Holds>
```

For each `StandingIntent` it proves, all fail-closed (`fallback_when_unavailable` on any miss — never green):

1. **claimed** — some `LensContract` names `intent.property` with `enforcement_mode_satisfies(contract.mode, intent.default_mode)` (§12 order; no overloaded `>=`).
2. **scope ⊇** — `scope_satisfaction(contract.claimed_scope, intent.desired_scope)` is `ScopeCovers`, or a typed `ScopeNarrowed { reason }` whose reason kind the intent allows (§12; set-containment on declared roots — decidable).
3. **subjects ⊇** — the *discovered* subject set (from the `CoverageReceipt`, not the contract's claim) includes every `required_subject` (membership over the live discovery — decidable; this is the leg that fails today on `src/v1/04_infer.dag::build_type_env`).
4. **live consumer** — `consumer_satisfies(contract.consumer, intent.required_consumer)` (§12) — NOT hardwired to `FloorGate`, so merge-admission / pre-push / periodic-actuator / deploy-readback consumers count; `None` = the coverage-by-illusion failure. Reuses the inert-lens backstop.
5. **red control** — the `CoverageReceipt`'s `red_control_status == RedControlPassed` — the witness actually went red under perturbation (§5 green-by-execution). A contract that self-declares `Present` but whose receipt is `RedControlFailedToFlip`/`NotRun` still reds.
6. **self-application** — the lens module is in its own property's subject set, or carries an explicit `Exempt { reason }`.

Every leg is decidable set-containment or liveness → the meta-gate is a `WallNow`, not a ratchet. It is itself classified by `ConstructionJustification`, and (§7) it is subject to itself.

## 5. Flip the default: whole-corpus unless explicitly narrowed

The operator's "objective things, enforce on the entire corpus — why not" becomes a *default*, not a per-lens opt-in:

- `StandingIntent.desired_scope` defaults to **whole corpus** (`["dsl", "src/v1", "src/v2"]`, `DagSource`).
- Scope containment is a typed `ScopeSatisfaction` (§12): `ScopeCovers` | `ScopeNarrowed { missing, reason: NarrowingReason }` | `ScopeMissing`. A `LensContract` narrower than a `StandingIntent` it answers must carry a typed `NarrowingReason` (`BootstrapBlocked` / `TypeReflectionUnavailable` / `ExternalRuntimeOnly` / `ExplicitOperatorExemption`) — a Blocking intent **Refuses** unless the reason kind is on its allow-list; an AuditOnly intent **Reports**. Not a stringly escape hatch.
- This inverts the permissive interface: silence no longer means "v2 only." Under-scope is a failing receipt, not a default.

## 6. The fractal layer (reuse, don't re-invent)

Your point that the lens must apply to the *intent expression*, not just the code, is §7 recursion and is already seeded in `intent_linearity` + `self_applying_lenses`. The enforcement-intent gate makes it a **required leg** (§4.6), across five occurrences of the same property:

1. the program analyzed · 2. the lens implementation · 3. the subject producer · 4. the enforcement registry · 5. the dispatch manager's acceptance template.

Concretely for complexity: if `complexity_lens`'s subject producer recomputes closure per lens (O(n²) over the corpus), the enforcement of the complexity intent *itself violates the complexity intent* — and the self-application leg reds. Same case, one layer up.

## 7. Sequencing (so the corpus can still merge)

The **meta relationship** is Blocking now — it is decidable and cheap: scope-must-not-narrow, lens-must-be-live, must-self-apply. The **object complexity rule** ships `AuditOnly` over the whole corpus first (it will surface the 10 `06_translate.dag` + 3 `glob_discovery_law.dag` quadratic-suspect sites, several harmless-by-magnitude), then flips per-rule to `Blocking` as the false-positive magnitude tiers land. Blocking the object rule whole-corpus on day one would red everything; Blocking the *coverage* on day one is exactly what we want.

## 8. Tasks (refined; each reuses an existing carrier)

1. **standing-intent-carrier** — `StandingIntent` type + the `complexity.repo-wide`, `lenses.must-be-live`, `lenses.self-apply-or-exempt` rows. Reuses `LensIdV0` (the existing lens-property coproduct — NOT a new `PropertyClass`), `SubjectRoster`, `ConstructionJustification`.
2. **lens-contract-extend** — grow `LensRegistryEntryV0` → `LensContract`; classify `complexity_lens` precisely (fixture-only vs live-syntactic; blocking vs advisory); record claimed scope + discovered subject counts. `build_type_env` / former `flatten` present-or-absent **by execution**.
3. **enforcement-intent-gate** — the join (§4). RED controls: absent-`src/v1` reds; whole-corpus-claim-but-scans-only-`src/v2` reds; blocking-without-consumer reds; blocking-without-red-control reds; audit-only passes only in `AuditOnly`.
4. **complexity-bad-shape-wall (R1)** — the first real wall: accumulator-in-copied-port (`map_merge` overlay / `list_append` left), with the v1 fixtures and the `merge_envs` negative test; floor-enrolled *through* the enforcement-intent gate so it cannot be inert.

Do 1–3 before adding many object rules, else task 4 is just one more possibly-inert lens.

## 9. Honest limits

- **Decidable (Blocking):** scope containment, subject coverage, consumer liveness, red-control presence, self-application. These are repo policy, not undecidable.
- **Count-witnessed residue:** magnitude / boundedness of a growing side (is the suspect fold actually bounded?) — a size fact the graph does not carry. Count-witness, never a wall-clock threshold.
- **Genuinely off-substrate:** the v1 interpreter's own quadratic is Rust (`v1_interpreter.rs`), not Nodes — structurally uncoverable by a Node lens; report as a clean negative, not a gap.

## 10. The sentence for the manager

> The operator has repeatedly asked for complexity enforced repo-wide. Your first job is **not** to add another complexity check — it is to make that standing intent impossible to lose: extend the registry into a contract, model the intent as a row, and gate the relationship fail-closed. A PR that narrows complexity from whole-corpus to fixture-only, adds a lens without a consumer, claims "blocking" without a red control, or omits `src/v1` without an explicit bootstrap exemption must go red.

## 11. Operator sign-off refinements (2026-07-02)

Signed off with these deltas:

- **Wording:** completion attaches to *a mechanism claiming enforcement*, not to every lens. Not all lenses answer a standing directive (some are local experiments / advisory diagnostics); only ones that *claim* enforcement (`repo-wide`, `blocking`, `complete`, `self-applying`) are held to a `StandingIntent`.
- **Anti-overcomplication rule (the wall that keeps the walls honest):** *no lens may claim `repo-wide`, `blocking`, `complete`, or `self-applying` unless the enforcement gate can independently prove the claim.* This is the meta-gate turned on the vocabulary of the claims themselves.
- **Two model-quality StandingIntents** — the DFS/anemia/consolidation habit is the *same governance bug on the single-authority axis* rather than the runtime-complexity axis (§2 deep-reduction, §3 single-authority). Add:

  ```
  model.anemia.repo-wide          // a String leaf hiding named parts is anemic (decompress→map→reduce)
  single-authority.consolidation  // two names for one concept fail unless one is explicitly
                                   //   realization / transport / external-source WITH direction
  ```

  `concept-dfs-before-minting` is an *implementation rule under* `single-authority.consolidation`, not (yet) its own top-level intent. The lens must **suggest a consolidation target**, not merely warn — "what existing concept should this map to? where is the single authority? is this a realization/transport/policy variant?" Live seed corpus: the `extdeps.shell` fork (main-red incident), the JSON-emitter fork, the hostname-surface split, `cli_run` BFS vs `module_graph` closure authority.

- **Rollout (so the corpus still merges, without a weak gate):** three tiers, not one flip.
  1. `StandingIntent` rows: **live**.
  2. `enforcement_intent_gate`: **Blocking on contract *consistency*** now — a claim that is internally impossible fails immediately, independent of any dependency: Blocking-with-no-consumer → fail; whole-corpus-claim-scanning-only-v2 → fail; self-apply-required-but-absent → fail; AuditOnly-with-a-declared-scope-gap → report, not block.
  3. Object rules (R1/R2/R3, anemia, consolidation): **AuditOnly** over the whole corpus until the receipts are understood, then per-rule flip to Blocking as false-positive magnitude tiers land.

  This is confirmed by silent-ferret's #6162 characterization: `complexity.repo-wide` already reds today on **three contract legs** — claimed_scope (4 exemplars + 3 synthetic, not repo-wide), consumer (witness-only, no FloorGate), self_application (false) — with **no `decl_facts` dependency**. The gate's first honest RED needs nothing new; the whole-corpus *subject-discovery* leg waits on the `decl_facts(roots)` reflection builtin (#5966 / gunbc#5364), and the gate's job is precisely to red on that gap rather than hide it.

- **Task order (manager):** 1 `StandingIntent` rows · 2 `LensContract` extension · 3 `enforcement_intent_gate` · 4 `complexity.repo-wide` first proof · 5 `model.anemia.repo-wide` + `single-authority.consolidation` next proof · 6 *only then* R1/R2 object rules. Do 1–3 before adding object rules, else task 6 is one more possibly-inert lens.

## 12. Type refinements (operator review 2026-07-02)

Concrete carriers so the gate consumes **receipts, not prose**. These supersede the sketchy inline enums in §3–§5.

### Consumer — not hardwired to the floor

```
type ConsumerKind
  = FloorGate | MergeAdmission | PrePushHook | PeriodicActuator
  | RoadmapDriftGate | DeployReadback | WitnessOnly | None

type ConsumerRequirement                       // lives on StandingIntent
  = AnyLiveConsumer
  | OneOf { acceptable: List<ConsumerKind> }
  | Exact { kind: ConsumerKind }

fn consumer_satisfies(actual: ConsumerKind, required: ConsumerRequirement) -> Bool
```

Rationale: the roadmap already has load-bearing enforcement *off* the floor — merge-admission freshness, pre-push doc/drift hooks, periodic converge, deploy read-back, generated-artifact gates. A gate that knows only `FloorGate` would reject valid enforcement or pressure everything into the floor. `complexity.repo-wide` sets `required_consumer = Exact { FloorGate }`; a merge-admission intent may set `OneOf`/`AnyLiveConsumer`.

### Enforcement mode — model the order, no overloaded `>=`

```
type EnforcementMode = Advisory | AuditOnly | Blocking     // Advisory < AuditOnly < Blocking
fn enforcement_mode_satisfies(actual: EnforcementMode, required: EnforcementMode) -> Bool
```

REDs: required `Blocking` + actual `AuditOnly` → red · required `AuditOnly` + actual `Advisory` → red · required `Advisory` + actual `AuditOnly` → green.

### Scope satisfaction — typed narrowing, not a stringly hatch

```
type ScopeSatisfaction
  = ScopeCovers
  | ScopeNarrowed { missing: SubjectRoster, reason: NarrowingReason }
  | ScopeMissing

type NarrowingReason
  = BootstrapBlocked { blocker: String }           // e.g. src/v1 pending self-host
  | TypeReflectionUnavailable { blocker: String }   // e.g. decl_facts(roots) not yet landed
  | ExternalRuntimeOnly { reason: String }          // e.g. the interpreter's off-substrate Rust loop
  | ExplicitOperatorExemption { signoff: String }
```

Rollout: `Blocking` intent + `ScopeNarrowed` → Refused unless the reason kind is on the intent's allow-list; `AuditOnly` intent + `ScopeNarrowed` → Report. `complexity.repo-wide`'s v1-reach gap carries `TypeReflectionUnavailable { blocker: "decl_facts(roots) #5966/gunbc#5364" }` today.

### CoverageReceipt — the gate reads receipts, never self-declared claims

```
type CoverageReceipt {
  contract_id: LensIdV0
  discovered_subjects: SubjectRoster             // what the live producer ACTUALLY found
  consumer_observed: ConsumerKind
  consumer_receipt_ref: ReceiptRef
  red_control_status: RedControlStatus
  self_application_status: SelfApplicationStatus
  probed_at: Timestamp                           // supply via args; no argless clock in-substrate
}

type RedControlStatus
  = RedControlPassed | RedControlMissing | RedControlNotRun | RedControlFailedToFlip
```

The gate consumes `CoverageReceipt`, never the `LensContract`'s self-declared claim: a contract claiming `red_control: Present` whose receipt says `RedControlFailedToFlip` still reds. This is §5 "green by execution, not by declaration" — applied to the gate itself (§7).

### StandingIntent admission (restated as a check)

A candidate row must satisfy all five: recurs across PRs · named displaced cost (§6) · has a scope · names a mechanism class · can produce a receipt. Fail any → it is a preference, not a `StandingIntent`. This is itself the anti-purity-trap guard (§6) turned on the governance carrier.

## 11. Candidate intake — the open queue (2026-07-25, management lane; operator-acknowledged capacity gap)

**The honest state, recorded so the roadmap stops implying otherwise:** the operator's own words — "I am missing a ton of lens enforcement, and I'm honestly not sure when I can add any of it." Decidable wall candidates are accumulating faster than enforcement capacity exists to land them, and until a candidate becomes a `StandingIntent` row with a live gate, **its ruling binds only the sessions that were briefed on it**. The discriminating receipt, same day as the ruling it violates: the `MoneyRate<P>` phantom family was ruled the specimen defect in the morning census, and by evening two independent sessions had re-minted onto it (#7202 added three new phantom markers in `std/measure.dag`; a second lane consumed the pattern fresh). A ruling without a wall has a half-life of hours.

Each row below passes the §StandingIntent admission test (recurs · displaced cost · scope · mechanism class · receipt). **None has scheduled capacity unless named.** The un-shelve rule is §6's: a candidate is pulled when its class recurs with a priced cost — recurrence receipts should be appended here, dated, so the queue self-prioritizes.

| Candidate | Class | Receipt | Capacity |
|---|---|---|---|
| Bodyless type reached in a construction position | wall (compile refusal + declared-abstract frontier) | `GroupCompletion`/`FieldOfFractions` found by 4,000-error probe instead of a compile error | **owned** — Gate-1 adjacent lane, design-note-first |
| Phantom-without-structure (bodyless type used only as a type argument) | lens → wall | `MoneyRate<P>` family; re-minted twice within hours of its ruling (2026-07-25) | queued |
| Uncited literal constant in std | lens (greppable) → StandingIntent | std sweep 2026-07-25: `billing_month_as_hour_count()=730`, POSIX exit codes in `std.process`, 8 unit-definition constants | queued |
| Hand count beside the structure it counts | wall (derive, never state) | `node_superset_field_count: Int = 18` | queued |
| Emit-tag census must be bidirectional (new un-rostered `[tag]` family reds) | lens, arms with the P3 re-census | `[floor-drain]`/`[gate-warm-cost]` born past the census snapshot, invisible to the one-directional witness | queued (rides #7162's P3 end state) |
| `*_failure_receipt` invocation parity (every companion invocable; a broken hook reds) | wall in claim_executor | two instances, one day: `witness_admission_…`/`design_register_lift_parity_…` both "no main function found" | queued (small; filed on #7162/#7199) |
| Interaction-totality family census (every component state × stateful channel in-family or `ConstantByLaw{DeclarationRef}`) | census → StandingIntent | the press-fill green flash (frontend round 5) | **owned** — composition session slice 1 |
| Fact-family → entity edge (a cited fact family must reference its subject entity row) | lens over extdeps | GitLab pricing/SEC facts correlated only by filename prefix (2026-07-25) | queued |
| Output policy installed by construction at every gunbc entrypoint (uninstalled → refuse, never fallback-Full) | wall | merge-admission stamp ran with no policy installed → ShellTrace fell back to Full → the raw-argv emoji dump (2026-07-25) | queued (interim per-entrypoint install in flight on #7162) |
| Undeclared-edge incidents — a use no derived view can see (bare ref bound by pool coincidence · string-path dep · workflow-invoked entry closure compiled by no gate) | one law, per-kind mechanism: derive-the-view-from-use, or refuse-the-undeclared | deploy red ~6h on #7233 (`srv3_boot_once_cd`, run 30173364541) · #7196 stale witness rode main green (no source-ref edge) · #6775's 705-error regression (string-hidden dep) — see import-strip diagnosis §15 | **partially owned**: bare refs = keen-wolf (a)+(b); string paths = SourceRef child; derived workflow-entry roster = day-one CI-registry row; affected-set `NoDeclaredSourceRefs→refuse` queued behind declarations |
| Fallback arms — catch-all-that-answers (`_ =>` yielding a success-shaped value over an open set), compiled default tables, default-on-absent (operator ruling 2026-07-25: "fallbacks almost always represent correctness issues"; the §5 hard-reject made mechanical) | lens (classify every `_ =>` arm: refuses ✓ · completes-closed-total ✓ · answers-on-open ✗, typed+located) → wall on zero-false-positive | one week's bijection: `_ => outcome_accepted` wrap-decision bypass (46–65 E0308/module) · `Product(<anon>)` fabrication (strip + deploy) · `ExpectSuccess` default (hid the real red) · policy fallback tables (shell-echo revert + log dump) · `NoDeclaredSourceRefs => false` · `Err(NoMainFunction) => ""` swallow · `sccache --show-stats` auto-start · `tail`+`exit 0` over dead unit. Measured superset 2026-07-25: 2,586 `_ =>` arms / 423 .dag files · 392 self-named fallback mentions / 124 files · 167 in 23 seed files · 408 seed `unwrap_or` · 7 shell `\|\| true` residues | W0 (expectation required-arg, policy-by-construction) + W1 arms owned by named lanes; **W2 census lens child unowned**; W3 wall rides lens-first→refusal promotion |

**Pricing note (§6):** the queue's aggregate displaced cost is no longer speculative — today alone it charged: one regen red + relocation round-trip (phantom re-mint), one operator log-review round (policy fallback), and two broken failure receipts discovered only because humans read logs. The first three rows are greppable-in-an-afternoon lens shapes; their cost is not the lens but the `StandingIntent` plumbing (§8 tasks 1–3), which is why §8's ordering — carrier + contract + gate before object rules — remains the real bottleneck and the thing to fund first when capacity exists.

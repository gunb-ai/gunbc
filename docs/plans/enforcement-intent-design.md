# Enforcement intent — ask once, compile forever

*Status: draft for operator review (2026-07-02). Extends the standing threads "model §1's axioms + enforce the syllogism" and the intent-linearity draft; does not supersede either.*

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
  mode: Blocking | AuditOnly | Advisory
  consumer: FloorGate | WitnessOnly | None // "referenced by synthesis" == None
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
  property: PropertyClass                   // reuse the lens property vocabulary (Complexity, …)
  desired_scope: SubjectRoster              // default = WHOLE CORPUS (see §5)
  required_subjects: List<QualifiedName>    // the receipts that MUST be covered
  default_mode: Blocking | AuditOnly | Advisory
  self_application_required: Bool
  fallback_when_unavailable: Refuse | Unknown | AuditOnlyWithReason  // never silently OK (§5)
}
```

`StandingIntent` references the property and subject carriers rather than restating them; it is the single home for "what the operator wants," so a future PR can refine the *implementation* but cannot silently narrow the *intent*.

## 4. The meta-gate (decidable — a wall now)

```
enforcement_intent_gate(intent: StandingIntent, contracts: List<LensContract>, receipts: CoverageReceipts) -> Witness<Holds>
```

For each `StandingIntent` it proves, all fail-closed (`fallback_when_unavailable` on any miss — never green):

1. **claimed** — some `LensContract` names `intent.property` with `mode >= intent.default_mode`.
2. **scope ⊇** — that contract's `claimed_scope` covers `intent.desired_scope` (set-containment on declared roots — decidable).
3. **subjects ⊇** — the *discovered* subject set includes every `required_subject` (membership over the live discovery — decidable; this is the leg that fails today on `src/v1/04_infer.dag::build_type_env`).
4. **live consumer** — `consumer == FloorGate` (reuses the inert-lens backstop; `None` = the coverage-by-illusion failure).
5. **red control** — `red_control == Present` and that witness goes red under perturbation (§5 green-by-execution + discriminating red).
6. **self-application** — the lens module is in its own property's subject set, or carries an explicit `Exempt { reason }`.

Every leg is decidable set-containment or liveness → the meta-gate is a `WallNow`, not a ratchet. It is itself classified by `ConstructionJustification`, and (§7) it is subject to itself.

## 5. Flip the default: whole-corpus unless explicitly narrowed

The operator's "objective things, enforce on the entire corpus — why not" becomes a *default*, not a per-lens opt-in:

- `StandingIntent.desired_scope` defaults to **whole corpus** (`["dsl", "src/v1", "src/v2"]`, `DagSource`).
- A `LensContract` whose `claimed_scope` is **narrower** than a `StandingIntent` it answers must carry an explicit `Narrowed { reason }` (e.g. a bootstrap exemption for `src/v1`), else the gate **Refuses**.
- This inverts the permissive interface: silence no longer means "v2 only." Under-scope is a failing receipt, not a default.

## 6. The fractal layer (reuse, don't re-invent)

Your point that the lens must apply to the *intent expression*, not just the code, is §7 recursion and is already seeded in `intent_linearity` + `self_applying_lenses`. The enforcement-intent gate makes it a **required leg** (§4.6), across five occurrences of the same property:

1. the program analyzed · 2. the lens implementation · 3. the subject producer · 4. the enforcement registry · 5. the dispatch manager's acceptance template.

Concretely for complexity: if `complexity_lens`'s subject producer recomputes closure per lens (O(n²) over the corpus), the enforcement of the complexity intent *itself violates the complexity intent* — and the self-application leg reds. Same case, one layer up.

## 7. Sequencing (so the corpus can still merge)

The **meta relationship** is Blocking now — it is decidable and cheap: scope-must-not-narrow, lens-must-be-live, must-self-apply. The **object complexity rule** ships `AuditOnly` over the whole corpus first (it will surface the 10 `06_translate.dag` + 3 `glob_discovery_law.dag` quadratic-suspect sites, several harmless-by-magnitude), then flips per-rule to `Blocking` as the false-positive magnitude tiers land. Blocking the object rule whole-corpus on day one would red everything; Blocking the *coverage* on day one is exactly what we want.

## 8. Tasks (refined; each reuses an existing carrier)

1. **standing-intent-carrier** — `StandingIntent` type + the `complexity.repo-wide`, `lenses.must-be-live`, `lenses.self-apply-or-exempt` rows. Reuses `PropertyClass`, `SubjectRoster`, `ConstructionJustification`.
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

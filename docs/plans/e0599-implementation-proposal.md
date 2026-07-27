# E0599 implementation proposal (read-only, Phase 0)

**Session:** swift-bee-52 · **Status:** re-chartered by loyal-boar-481 (msg_7b6ac246); **rework** per loyal-boar-481 (msg_c95ef6e2) — **docs-only; no emitter/model implementation**  
**Prerequisite receipts:** #7283 merged (`docs/probes/e0599_diagnosis_2026-07-26.md`, `docs/probes/e0599_canonical_seven_census_2026-07-26.tsv`); loyal-raven-94 E0277 census (`docs/probes/e0277_trait_bound_census_2026-07-26.md`); gate1 Root-4 taxonomy (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md` §Root 4); **#7304 merged** at `18f71cf6` — `measure_missing_generics` closed (8→0; causal receipt: materialization_carriers 328/18→320/17, `05_eval` 431→427); **Gate 1 remains open**.

**Management boundary:** Phase 0 forbids broad emitter edits until this consumer graph is accepted. **#7304 is landed — not a future boundary.** Next authorized work after acceptance: **P-fn model-only authority**, then **one** `emit_fn_def` consumer flip (arm (a) only). P-derive and P-optional are separate later work.

---

## 1. What is proven vs not proven

| Claim | Status | Receipt |
|---|---|---|
| Canonical-seven E0599 totals stable at 635, Δ=0 vs #7275 | **Proven** | `e0599_canonical_seven_census_2026-07-26.tsv` |
| Six census root families (R1–R6) with mechanistic descriptions | **Proven** | same TSV §root_family_rollup |
| Root-4 **`measure_missing_generics`** family closed | **Proven** | #7304 @ `18f71cf6` (8→0) |
| Gate 1 (trait-derive completeness / repr grounding) closed | **Not proven** | #7304 closed only the Measure overlay slice |
| R1+R2+R3 share one *mechanism* (missing `Clone` on generic `T` and container uses) | **Diagnosis hypothesis** | diagnosis §Root 1–3; not yet closed by execution |
| R1+R2+R3 and E0277 Family 1 share **one predicate** | **Not proven — do not claim** | loyal-raven-94 census: E0277 Family 1 sites are **struct-declaration derives**; E0599 R1–R3 sites are **fn-body** `.clone()` / `.is_empty()` / `.iter()` on generic receivers |
| One predicate moves both buckets on one stamped binary | **Not demonstrated** | joint gate explicitly on hold (msg_c66ee135) |
| Body-lowering exposes **some** ingested fn operations as substrate `Transform` | **Proven (fixture scope)** | `wave1_gate1_normalized_add_yields_arrow_transform_holds` green by execution (`body_lowering/wave1_gate1_normalize_add_helpers.dag`); manual `body_lowering_projection_call` — postfix `helper.step(…)` → `Transform` |
| Body-lowering exposes **all** canonical-seven R1/R2/R3 method sites as typed `TraitBoundSiteMethodInvoke` in v1 emit-module closure | **Not proven — do not claim** | no executed witness over v1 emit-module fn bodies; see §3 P-fn coverage audit |

**No E0599 root family is closed.** Build + `regen_stage0 --verify` green on worktree; E0599 error counts unchanged.

---

## 2. Root-family → emission-site consumer graph

### 2.1 E0599 families (fn-body / method-resolution surface)

| Family | Count | Share | Emission consumer (where rustc sees the defect) | Modeled authority today |
|---|---:|---:|---|---|
| **R1** missing_method `clone` on type param | 261 | 41% | `emit_fn_def` signature missing `T: Clone` that the **modeled** fn already requires | **Partial** — `v1_generic_params_needing_clone_bound` (structural rules on substrate `Node`s only; incomplete vs census) |
| **R2** bounds_unsatisfied `is_empty`/`iter` on `im::Vector` carriers | 168 | 27% | Same — modeled collection witness requires `Clone` on element type param | **None** (witness row not yet authored) |
| **R3** bounds_unsatisfied `clone` on `Outcome`/`Option`/`im::Vector` | 161 | 25% | Same | **None** |
| **R4** GlobalBare* on `std::option::Option` | 24 | 4% | **R4-A only (census):** match-pattern emission mis-parents `GlobalBare*` variants on native `Option` scrutinee — see §3 P-optional | `v2.std.symbol_index.GlobalBareBindingState` + `emit_variant_pattern` / `emit_rust_expr_match` (`05_emit_rust.dag:5689` / `:6596`) |
| **R5** `as_deref` on `()` | 12 | 2% | Projection site with unit receiver | separate tail; defer |
| **R6** misc cache/verdict `clone` | 9 | 1% | Named carrier structs | defer |

**R1+R2+R3 = 590 (93%)** — all observed at **generic fn/item bodies and their signatures**, not at `#[derive]` on struct declarations (per full 7/7 census keyed by message text).

### 2.2 E0277 families (trait-bound / derive surface) — loyal-raven-94

| Family | Approx share | Emission consumer | Overlap with E0599 R1–R3? |
|---|---:|---|---|
| **F1** `T`/`U`/`A`/`B`: `Clone` at **struct-declaration derive** sites | ~26–30/module | `emit_type_def_from_connective` → plain `emit_type_params` + `#[derive(Debug, Serialize, Deserialize, …)]` (`05_emit_rust.dag` ~4609–4615) | **Same missing Clone fact, different site class** — NOT the fn-only predicate |
| **F2** `Node` / `EnvironmentBindingKey` missing `Hash`/`Eq` | 5–6/module | Map/set key usage without derive on carrier | None |
| **F3** serde/Debug missing on `*Interpreter`, `CommutativeSemiring<Magnitude>` | dominant in emit_host / materialization_carriers | `trait_derive_emit` arm (c) — `#[derive]` without per-target bound overrides | None for R1–R3 counts |
| **Residual** 56 tree-wide | `Debug` derive variant + two unrelated one-offs | same as F3 + arithmetic one-off | None |

**Critical distinction (loyal-raven review 43338):** Family 1 is **not** fixed by adding `T: Clone` to the struct's own generic parameter list. `im::Vector<A>` needs `Clone` only in specific **impl** bound lists (`Debug`, `Serialize`, `Deserialize`). Correct shape: per-derive-target bounds (`#[serde(bound(...))]` overrides; hand `Debug` impl or per-impl bound emission).

### 2.3 Trait-derive lane (Gate-1 #7174 — open; measure overlay landed)

`src/v1/trait_derive_emit.dag` + `dag/std/trait_derive_shape.dag` + `v2.compiler.trait_derive_completeness` name three arms (canonical surfaces in §3.0):

| Arm | Scope | Current reach | Clean-salvage target |
|---|---|---|---|
| **(a)** clone bounds on generic params | fn signatures | `v1_generic_params_needing_clone_bound` → `emit_fn_def` only (`05_emit_rust.dag:5172`) — **scaffold** | `TraitBoundWitness` accessor (model-only first) → **one** consumer flip at `emit_fn_def` |
| **(b)** supplemental `impl` blocks for coproduct-native arithmetic | `GroupCompletion`, kernel int carriers | `trait_derive_completeness_gate` + `repr_grounding_derive_traits_for_collection_witness` | E0369 coproduct ops (consume gate, not new predicate walks) — **deferred** |
| **(c)** serde/Debug/Ord `#[derive]` on named structs/enums | struct/enum declarations | capability table → `v1_emit_struct_from_capability_table` | P-derive container-contract rows — **deferred** |

---

## 3. Proposed predicates (separate — do not merge)

### §3.0 Canonical surfaces (predicate/walker dissolution)

DESIGN requires facts-flow-forward **and** that Node-shape classification not accumulate hand-matched predicates. Implementation must **consume** one of:

| Surface | Authority | Precedent |
|---|---|---|
| **General trait-bound accessor** | New `TraitBoundWitness` coproduct + `required_trait_bound_witnesses_for_fn_decl(fn_node) -> List<TraitBoundWitness>` (`target_model.dag`) — grounded **modeled operations** in fn bodies (`TraitBoundSiteMethodInvoke { method, receiver }`), **not** type occurrence alone | (new — P-fn home) |
| **Collection repr witness** | `target_collection_witness_from_node` → `Outcome<RequiredTraitWitness>` — **unchanged**; models constraints on `TargetRepresentationChoice` only (Ord/Hash/Eq today). Collection facts **refine** general `TraitBoundWitness` at repr-choice sites; do not broaden `RequiredTraitWitness` for fn-wide Clone | Ord/Hash/Eq collection witnesses |
| **Completeness gate** | `trait_derive_completeness_gate` / `trait_derive_completeness_gate_for_collection_witness` (`trait_derive_completeness.dag`) — calls `repr_grounding_derive_completeness_predicate` **only inside** the gate; emit consumes the gate verdict | Gate-1 sub-wall #2 (#7174) |
| **Capability table lookup** | `repr_grounding_derive_shape_has_trait(shape, trait)` + `record_derive_traits_*` list builders (`trait_derive_shape.dag`) — answers **which traits an elem shape may derive**, not per-impl bound lists | struct `#[derive]` attr *selection* today |
| **Container implementation contract** | `TargetCollectionRealization` / `target_collection_realization_lookup_in_catalog` (`target_model.dag`) — keyed by **source carrier + emitted representation**, carries per-trait supplemental bound facts | collection repr choice + `RequiredTraitWitness` constraints today |

**Prohibited in new work:** extending hand-matched substrate walks (including growing `v1_type_param_needs_clone_bound` or ad-hoc `repr_grounding_derive_completeness_predicate` call sites); grammar-production-spine or emitted-Rust syntax walkers as substitutes for canonical operation surfaces.

**Bounded interim disposition** for arm (a) until accessor lands:

| Field | Value |
|---|---|
| **Owner** | `v2.compiler.trait_derive_completeness` + `std.trait_derive_shape` |
| **Lane** | Gate-1 trait-derive-completeness (#7174); v1 `trait_derive_emit` is realization-only |
| **Dissolve trigger** | `TraitBoundWitness` + `required_trait_bound_witnesses_for_fn_decl` ships in `target_model`; v1 deletes `v1_generic_params_needing_clone_bound` body-classification; `emit_fn_def` reads witness list only. `RequiredTraitWitness` stays collection-repr-only |

### Predicate P-fn: `generic_fn_modeled_clone_witness` (E0599 R1–R3 candidate)

**Intent:** When a **modeled** generic function's body contains an operation that **requires** `Clone` on type parameter `T` (or `A`/`U`/`B`/`R`) — a grounded `TraitBoundWitness` naming the **specific modeled operation** (e.g. `.clone()` on a type-param receiver, `.clone()` on `Outcome<T>`/`Option<T>`/`im::Vector<T>`, `.is_empty()`/`.iter()` on a collection receiver) — emit `T: Clone` on the fn's generic/`where` clause. Emission **consumes** the witness list; it does not infer requirements from downstream Rust or from type occurrence alone.

**Anti-pattern (explicitly out of scope):** (1) scanning emitted function bodies for method names; (2) encoding all fn-bound facts as `TargetCollectionWitness*` inside `RequiredTraitWitness`; (3) witnessing Clone from return-type or field-type occurrence alone; (4) inventing a wholesale new body-producer lane or syntax walker when the missing fact is typed receiver→type-param evidence on method invokes.

#### P-fn body-lowering coverage audit (executed receipts)

Current `main` already has `v2.compiler.body_producer_forward` + `v2.compiler.body_lowering_fold` (postfix projection → `Transform`, infix → `Transform`). **Do not claim** "bodies exist only as grammar production spines."

| Surface | Status | Receipt |
|---|---|---|
| Infix fn body (`x + y`) → `Transform` | **Proven** | `wave1_gate1_normalized_add_yields_arrow_transform_holds` — green by execution |
| Postfix cross-decl call (`helper.step(a,b)`) → `Transform` | **Proven** | `manual/body_lowering_projection_call.dag` — `body_lowering_projection_lowers_to_transform` |
| Unconsumed fn_decl shapes | **Wrapper-retained** | `body_lower_wrapper_retained_shell` (`body_lowering_fold.dag`) — counted frontier, not a syntax walker substitute |
| `.clone()` / `.is_empty()` / `.iter()` on generic receivers in **v1 emit-module closure** with typed `TraitBoundSiteMethodInvoke { method, receiver, type_param }` evidence | **Not proven** | **Missing source fact for −590 claim** — no executed witness that canonical-seven R1/R2/R3 sites in `04_infer`…`emit_module` fn bodies expose method-invoke + receiver→type-param linkage in normalized substrate. Model-only `TraitBoundWitness` authority can land; **predicted −590 is hypothesis until this gap is receipted or ruled.** |

**If implementation cannot name the missing fact without inventing it:** return to loyal-boar-481 for a new ruling before claiming census movement.

| Field | Value |
|---|---|
| **Modeled home (proposed)** | New **`TraitBoundWitness`** coproduct in `target_model` (`FnTypeParamBoundClone { type_param, site: TraitBoundSiteMethodInvoke { method, receiver } }`). **`required_trait_bound_witnesses_for_fn_decl(fn_node) -> List<TraitBoundWitness>`** as `fold_node` query (`TraitBoundWitnessFold`) over substrate behaviors where they exist; refuse when operation evidence absent. **`RequiredTraitWitness` unchanged** |
| **Production consumers (sequenced)** | **(1) model-only:** accessor + witnesses in `target_model`; **(2) one flip:** `v1.compiler.trait_derive_emit` arm (a) → `v1_emit_type_params_with_clone_bounds` ← witness list → `emit_fn_def` (`05_emit_rust.dag:5172`) only |
| **Does NOT add** | `TargetCollectionWitnessClone` inside `RequiredTraitWitness`; new Node-shape predicates in v1; extensions to `v1_type_param_needs_clone_bound`; emitted-Rust body scans |
| **Predicted effect** | E0599 −590 (R1+R2+R3) — **hypothesis** pending coverage audit above |
| **RED (refusal)** | `fn f<T>() -> Outcome<T>` with no `.clone()` invoke → emit refuses `T: Clone` |
| **Rollback boundary** | Revert `TraitBoundWitness` fold algebra; census TSV byte-for-key unchanged |

**Interim disposition:** `v1_generic_params_needing_clone_bound` stays counted scaffold until accessor + single `emit_fn_def` flip land.

**Fence:** `src/v1/05_emit_rust.dag` owned by witty-wolf-289 — wiring routes through `trait_derive_emit` imports only.

### Predicate P-derive: `per_derive_target_bound_completeness` (E0277 F1/F3 + derive arm c) — **deferred**

**Intent:** Per-derived-impl minimal bounds via container implementation contract (`TargetCollectionRealization` / sibling catalog), not elem-shape capability table alone. **Separate later work** — do not batch with P-fn.

| Field | Value |
|---|---|
| **Modeled home (proposed)** | Container-contract rows keyed by `(source_carrier, emitted_representation, derive_trait)`; trait eligibility stays in `std.trait_derive_shape` gate |
| **Production consumers** | `v1_emit_struct_from_capability_table` → contract lookup → serde `bound(...)` / per-impl overrides |
| **Predicted effect** | E0277 Family 1 + F3 — not E0599 R1–R3 |

### Predicate P-optional: `global_bare_binding_state_emit_grounding` (E0599 R4 / Slice B) — **deferred**

**Two separate carriers — do not conflate (review 43687):**

| Carrier | Authority | Role |
|---|---|---|
| **`Optional<GlobalBareBindingState>`** | `SymbolIndex.global_bare: Map<Symbol, GlobalBareBindingState>`; `map_get` at `symbol_index.dag:130` | **Map-read result** — absent/present binding state for a bare symbol |
| **`GlobalBareLookup`** | `symbol_index_global_unique_lookup` → `GlobalBareHit \| GlobalBareLookupAmbiguous \| GlobalBareLookupUnbound` (`symbol_index.dag:40`, `:129`) | **Lookup coproduct** — reference-resolution verdict; **not** an `Option` envelope and **not** interchangeable with map-read emission |

**R4-A — census defect (match-pattern / map-read emission):** `GlobalBareUnique` / `GlobalBareAmbiguous` are variants of **`GlobalBareBindingState`** (`symbol_index.dag:36`), not modeled `Optional`. Today's bug: emit renders `Some(Optional::GlobalBareUnique { … })` on a native `std::option::Option<_>` scrutinee (diagnosis §Root 3). **Correct R4-A shape:** when the modeled site is a map-read yielding `Present(GlobalBareBindingState::GlobalBareUnique { … })`, emit `Some(GlobalBareBindingState::GlobalBareUnique { … })` — outer `Option` is Rust's map envelope; inner variants are `GlobalBareBindingState` only.

| Field | Value |
|---|---|
| **Modeled home** | `v2.std.symbol_index.GlobalBareBindingState` |
| **Production consumer (R4-A, named)** | `emit_variant_pattern` / `emit_variant_pattern_rc_aware` (`05_emit_rust.dag:5689` / `:5946`) called from `emit_rust_expr_match` (`:6596`) when variant parent resolution attaches `GlobalBare*` to modeled `Optional`/native `Option` instead of `GlobalBareBindingState` |
| **R4-B (separate — not census R4)** | `GlobalBareLookup` lowering at reference sites (`v1.compiler.infer_env` / `04_lookup.dag` / `symbol_index_global_unique_lookup`) — ambiguity/unbound refusals; **different consumer, different fix** |
| **Anti-pattern** | Treating `GlobalBareLookup` as `Option<GlobalBareBindingState>`; lowering R4-A and R4-B as one shared carrier |
| **Predicted effect** | E0599 −24 (R4-A only) |

**Not one root:** P-fn, P-derive, and P-optional are three predicates with disjoint primary consumers.

---

## 4. Sequencing (proposal only — awaiting authorization)

```
Phase 0 (now)     → this document + management acceptance of consumer graph
Phase 1 (done)    → #7304 measure_missing_generics overlay (@18f71cf6; 8→0; NOT Gate 1)
Phase 2           → P-fn model-only: `TraitBoundWitness` + `required_trait_bound_witnesses_for_fn_decl` in target_model
Phase 3           → one consumer flip: arm (a) → `emit_fn_def` reads witness list (`05_emit_rust.dag:5172` via trait_derive_emit)
Phase 4           → P-derive (separate; later)
Phase 5           → P-optional R4-A/R4-B (separate; later; do not conflate carriers)
Phase 6           → tails R5/R6 if still above noise floor
```

**Do not start Phase 3 before Phase 2** (model before emit flip). **Do not batch** P-derive or P-optional with P-fn. Interim arm (a) stays on counted `v1_generic_params_needing_clone_bound` scaffold until Phase 3.

---

## 5. Verification protocol (when authorized)

Each landed predicate carries its **own** receipt:

1. Fresh `gunbc` + `cssl_assemble` build; stamp sha256 + git sha in TSV header.
2. Canonical-seven probe (`PROBE_KEEP_LOG_DIR`, `e0599_census_extract.sh` / `.dag` authority).
3. Parallel E0277 extraction **only when testing a predicate claimed to move E0277**.
4. Compare to `refresh_canonical_seven_2026-07-26.tsv` + family-level rollup sections.
5. RED controls: both directions per §3 tables.

---

## 6. Recommendation

| Action | Verdict |
|---|---|
| Close work item | **No** — diagnosis complete, zero E0599 roots closed |
| Accept Phase 0 consumer graph | **Pending** loyal-boar-481 sign-off after this rework |
| Next authorized artifact | P-fn model-only `TraitBoundWitness` authority (Phase 2), then single `emit_fn_def` flip (Phase 3) |

---

*Dissolve-on: when each predicate lands with green receipt, fold measured targets from this proposal into `dag/tools/e0599_probe_census.dag` notes and retire duplicate prose here.*

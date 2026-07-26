# E0599 implementation proposal (read-only, Phase 0)

**Session:** swift-bee-52 · **Status:** re-chartered by loyal-boar-481 (msg_7b6ac246) — **no code until authorized**  
**Prerequisite receipts:** #7283 merged (`docs/probes/e0599_diagnosis_2026-07-26.md`, `docs/probes/e0599_canonical_seven_census_2026-07-26.tsv`); loyal-raven-94 E0277 census (`docs/probes/e0277_trait_bound_census_2026-07-26.md`); gate1 Root-4 taxonomy (`docs/probes/gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md` §Root 4).

**Management boundary:** Phase 0 forbids broad emitter edits until this consumer graph is accepted. **First implementation boundary after authorization:** Root-4 clean salvage (`v1.compiler.trait_derive_emit` / `std.trait_derive_shape`), not a blanket Slice-A fn-body PR.

---

## 1. What is proven vs not proven

| Claim | Status | Receipt |
|---|---|---|
| Canonical-seven E0599 totals stable at 635, Δ=0 vs #7275 | **Proven** | `e0599_canonical_seven_census_2026-07-26.tsv` |
| Six census root families (R1–R6) with mechanistic descriptions | **Proven** | same TSV §root_family_rollup |
| R1+R2+R3 share one *mechanism* (missing `Clone` on generic `T` and container uses) | **Diagnosis hypothesis** | diagnosis §Root 1–3; not yet closed by execution |
| R1+R2+R3 and E0277 Family 1 share **one predicate** | **Not proven — do not claim** | loyal-raven-94 census: E0277 Family 1 sites are **struct-declaration derives**; E0599 R1–R3 sites are **fn-body** `.clone()` / `.is_empty()` / `.iter()` on generic receivers |
| One predicate moves both buckets on one stamped binary | **Not demonstrated** | joint gate explicitly on hold (msg_c66ee135) |

**No root family is closed.** Build + `regen_stage0 --verify` green on worktree; error counts unchanged.

---

## 2. Root-family → emission-site consumer graph

### 2.1 E0599 families (fn-body / method-resolution surface)

| Family | Count | Share | Emission consumer (where rustc sees the defect) | Modeled authority today |
|---|---:|---:|---|---|
| **R1** missing_method `clone` on type param | 261 | 41% | `emit_fn` signature missing `T: Clone` that the **modeled** fn already requires | **Partial** — `v1_generic_params_needing_clone_bound` (structural rules on substrate `Node`s only; incomplete vs census) |
| **R2** bounds_unsatisfied `is_empty`/`iter` on `im::Vector` carriers | 168 | 27% | Same — modeled collection witness requires `Clone` on element type param | **None** (witness row not yet authored) |
| **R3** bounds_unsatisfied `clone` on `Outcome`/`Option`/`im::Vector` | 161 | 25% | Same | **None** |
| **R4** GlobalBare* on `std::option::Option` | 24 | 4% | Match lowering: modeled `Optional::GlobalBare*` arms reach native `Option` pattern syntax | namespace-resolution / coproduct grounding lane (not Root-4 trait derive) |
| **R5** `as_deref` on `()` | 12 | 2% | Projection site with unit receiver | separate tail; defer |
| **R6** misc cache/verdict `clone` | 9 | 1% | Named carrier structs | defer post–Root-4 |

**R1+R2+R3 = 590 (93%)** — all observed at **generic fn/item bodies and their signatures**, not at `#[derive]` on struct declarations (per full 7/7 census keyed by message text).

### 2.2 E0277 families (trait-bound / derive surface) — loyal-raven-94

| Family | Approx share | Emission consumer | Overlap with E0599 R1–R3? |
|---|---:|---|---|
| **F1** `T`/`U`/`A`/`B`: `Clone` at **struct-declaration derive** sites | ~26–30/module | `emit_type_def_from_connective` → plain `emit_type_params` + `#[derive(Debug, Serialize, Deserialize, …)]` (`05_emit_rust.dag` ~4609–4615) | **Same missing Clone fact, different site class** — NOT the fn-only predicate |
| **F2** `Node` / `EnvironmentBindingKey` missing `Hash`/`Eq` | 5–6/module | Map/set key usage without derive on carrier | None |
| **F3** serde/Debug missing on `*Interpreter`, `CommutativeSemiring<Magnitude>` | dominant in emit_host / materialization_carriers | `trait_derive_emit` arm (c) — `#[derive]` without per-target bound overrides | None for R1–R3 counts |
| **Residual** 56 tree-wide | `Debug` derive variant + two unrelated one-offs | same as F3 + arithmetic one-off | None |

**Critical distinction (loyal-raven review 43338):** Family 1 is **not** fixed by adding `T: Clone` to the struct's own generic parameter list. `im::Vector<A>` needs `Clone` only in specific **impl** bound lists (`Debug`, `Serialize`, `Deserialize`). Correct shape: per-derive-target bounds (`#[serde(bound(...))]` overrides; hand `Debug` impl or per-impl bound emission).

### 2.3 Root-4 lane (management first boundary)

`src/v1/trait_derive_emit.dag` + `dag/std/trait_derive_shape.dag` already name three arms:

| Arm | Scope | Current reach | Clean-salvage target |
|---|---|---|---|
| **(a)** clone bounds on generic params | fn signatures | `v1_generic_params_needing_clone_bound` → `emit_fn_def` only (`05_emit_rust.dag:5172`) | Extend `RequiredTraitWitness` / `trait_derive_completeness` rows from **modeled** fn generic usage; emit consumes witness list — **never** scan emitted Rust |
| **(b)** supplemental `impl` blocks for coproduct-native arithmetic | `GroupCompletion`, kernel int carriers | `repr_grounding_derive_completeness_predicate` + `rust_supplemental_impls_group_completion` | E0369 / coproduct operator surface (gate1 Root 4 sub-wall 2) |
| **(c)** serde/Debug/Ord `#[derive]` on named structs/enums | struct/enum declarations | capability table → `rust_trait_derive_attr_from_traits` | E0277 F3 (+ F1 serde/Debug limbs) with **per-derive-target** bounds |

**Root-4 clean salvage** = land (b)+(c) completeness and the **derive-site** half of (a) with fail-closed predicates, without opening the fenced `05_emit_rust.dag` ownership consolidation (#7296 / witty-wolf-289).

---

## 3. Proposed predicates (separate — do not merge)

### Predicate P-fn: `generic_fn_modeled_clone_witness` (E0599 R1–R3 candidate)

**Intent:** When a **modeled** generic function's signature/capabilities require `Clone` on type parameter `T` (or `A`/`U`/`B`/`R`) — because a `RequiredTraitWitness` row or `trait_derive_completeness` gate says the param is used as a collection element, return type, or other grounded witness site — emit `T: Clone` on the fn's generic/`where` clause. Emission **consumes** the witness list; it does not infer requirements from downstream Rust.

**Anti-pattern (explicitly out of scope):** scanning emitted function bodies (or any target-language text) for `.clone()`, `.is_empty()`, or `.iter()` calls and retroactively adding bounds. That inverts facts-flow-forward and duplicates authority at the wrong layer.

| Field | Value |
|---|---|
| **Modeled home (proposed)** | Extend `v2.std.compilers.target_model.RequiredTraitWitness` (or sibling row beside `TargetCollectionWitnessOrd`/`Hash`/`Eq`) with a `Clone` witness variant; classify generic-param sites in `v2.compiler.trait_derive_completeness` over substrate `Node`s (same forward read as existing `v1_type_param_needs_clone_bound`, but complete + v2-authoritative) |
| **Production consumers** | `v1.compiler.trait_derive_emit` arm (a) → `v1_generic_params_needing_clone_bound` / `v1_emit_type_params_with_clone_bounds` → `emit_fn_def` (`05_emit_rust.dag:5172`) |
| **Does NOT consume** | Emitted Rust text; `emit_type_def_from_connective` struct/enum derive path (that is P-derive) |
| **Predicted effect** | E0599 −590 (R1+R2+R3); per-module clone/is_empty/iter histogram → 0 |
| **E0277 collateral** | **Unknown — do not claim** until stamped binary proves Family 1 is not the dominant remaining E0277 mass |
| **RED (green direction)** | Canonical-seven probe: R1+R2+R3 rows absent from `e0599_canonical_seven_census_*.tsv` |
| **RED (refusal direction)** | Fixture: modeled fn declares generic `T` with **no** `Clone` witness in source → emit refuses to fabricate `T: Clone` on the signature |
| **Rollback boundary** | Revert witness rows + trait_derive_emit arm (a) extension; census TSV must match pre-fix receipt byte-for-key totals |

**Fence:** `src/v1/05_emit_rust.dag` owned by witty-wolf-289 — P-fn wiring must route through existing `trait_derive_emit` imports or wait for operator sequencing.

### Predicate P-derive: `per_derive_target_bound_completeness` (E0277 F1/F3 + Root-4 arm c)

**Intent:** For each `ReprGroundingDeriveTrait` × elem shape row, emit the **minimal bound list each derived impl actually needs** (e.g. `T: Clone + Serialize` for serde on `Vector<T>` fields), never a blanket struct-level `T: Clone`.

| Field | Value |
|---|---|
| **Modeled home (proposed)** | Extend `std.trait_derive_shape` capability table + `repr_grounding_derive_completeness_predicate`; realization in `trait_derive_emit` arm (c) + serde bound override emission |
| **Production consumers** | `emit_type_def_from_connective` derive attr path; `record_derive_traits_*` builders; `rust_trait_derive_attr_from_traits` |
| **Predicted effect** | E0277 Family 1 + F3 serde/Debug rows; **not** E0599 R1–R3 unless those errors also appear at derive sites (census says they do not) |
| **RED (green)** | `e0277_trait_bound_census` Family 1 `T: Clone` rows burn; serde 3:1 ratio rows on `*Interpreter` absent |
| **RED (refusal)** | Struct with field type whose impl needs `Clone` but source `.dag` explicitly marks param non-cloneable → typed refusal, no blanket struct bound |
| **Rollback boundary** | Revert capability-table rows + emit attr overrides; E0277 per-module totals return to 603 baseline |

### Predicate P-optional: `modeled_optional_match_grounding` (E0599 R4 / Slice B)

**Intent:** Lower `Optional::GlobalBareUnique|GlobalBareAmbiguous` to native match only when receiver is modeled `Optional`, not `std::option::Option`.

| Field | Value |
|---|---|
| **Modeled home** | namespace-resolution / coproduct emit lane (not `trait_derive_emit`) |
| **Predicted effect** | E0599 −24; GlobalBare* rows absent from census |
| **Coordination** | Separate from Root-4; do not batch with P-fn or P-derive |

**Not one root:** P-fn, P-derive, and P-optional are three predicates with disjoint primary consumers. **Joint E0599+E0277 claim requires:** one stamped binary where **the same predicate** moves both buckets — currently **not satisfiable** with P-fn alone per loyal-raven census.

---

## 4. Sequencing (proposal only — awaiting authorization)

```
Phase 0 (now)     → this document + management acceptance of consumer graph
Phase 1 (first)   → Root-4 clean salvage: P-derive (+ arm b arithmetic completeness)
Phase 2           → P-fn modeled Clone witness on fn signatures (separate PR, separate receipt)
Phase 3           → P-optional (R4) when namespace lane ready
Phase 4           → tails R5/R6 if still above noise floor
```

**Do not start Phase 2 before Phase 1 acceptance** — avoids repeating the "one root" overclaim and respects the `05_emit_rust.dag` fence (P-derive routes through `trait_derive_emit.dag`, not seed emit bulk edits).

---

## 5. Verification protocol (when authorized)

Each landed predicate carries its **own** receipt, not a shared joint gate unless explicitly re-approved:

1. Fresh `gunbc` + `cssl_assemble` build; stamp sha256 + git sha in TSV header.
2. Canonical-seven probe (`PROBE_KEEP_LOG_DIR`, `e0599_census_extract.sh` / `.dag` authority).
3. Parallel E0277 extraction (loyal-raven method) **only when testing a predicate claimed to move E0277**.
4. Compare to `refresh_canonical_seven_2026-07-26.tsv` + family-level rollup sections.
5. RED controls: both directions per §3 tables.

---

## 6. Recommendation

| Action | Verdict |
|---|---|
| Close work item | **No** — diagnosis complete, zero roots closed |
| Rename / re-charter | **Yes** → "E0599 Root-4 + per-family implementation" |
| Next authorized artifact | Management sign-off on this consumer graph, then Root-4 clean salvage PR scoped to **P-derive** (+ arm b), not Slice-A fn-body blast |

---

*Dissolve-on: when each predicate lands with green receipt, fold measured targets from this proposal into `dag/tools/e0599_probe_census.dag` notes and retire duplicate prose here.*

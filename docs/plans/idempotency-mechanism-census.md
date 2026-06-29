# Idempotency mechanism — corpus census (DESIGN-ONLY)

> **Status: DESIGN-ONLY — awaiting operator shape-sign.** READ-ONLY inventory; **no migrations** until signed.
> Companion: [idempotency-mechanism-design.md](idempotency-mechanism-design.md)
> Re-verified against live tree 2026-06-29 (session crisp-swift-764). Re-run receipt before acting on counts.

---

## 0. Census scope

**In scope:** Effect idempotency — `EffectShape` / `EffectKind`, derivation, modifier checks, idempotency witnesses, de-fork surface.

**Out of scope (named, not merged):**

| concept | touch count | note |
| --- | ---: | --- |
| Algebraic idempotency (`v2.lens.idempotency`) | 7 `.dag` modules | Dependency-graph law witness — separate concept |
| Round-trip idempotency (emit/ingest) | 2 sites | `type_decl_emit_derive_idempotent_holds`, `parse_emit_round_trip_idempotency` |
| Workflow dedup (`idempotency_key`) | 1 type field | `gunbc/workflow/types.dag` — orthogonal transport |
| Lattice idempotence (algebra) | 1 v1 test | `lattice_idempotence_on_non_parameterized_variants` — unrelated |

---

## 1. Receipt — fork surface (the blocker)

| authority | file | LOC (approx) | axis | consumers |
| --- | --- | ---: | --- | --- |
| **dsl** | `dsl/std/effects.dag` | 251 | operation-kind flat enum + `is_idempotent_effect: Bool` + `IdempotencyEvidence` | v1 seed (`std_effects.rs`), `v1/effect_derivation.dag`, `dsl/std/realization.dag`, extdeps HTTP derivation |
| **v2 duplicate** | `src/v2/std/effects.dag` | 412 | idempotency-class wrapper (`IsIdempotent`/`IsBreaking`) + node projections + samples | `v2.lens.testgen`, generated conformance, idempotency_contract (indirect) |
| **shared names, divergent bodies** | 6 symbols | — | `EffectShape`, `KeySource`, `CreateCause`, `key_source_eq`, `create_effect_is_dedupable`, `create_double_init_collapsible` | **cannot repoint without this design** (`dsl-v2-defork-audit` grounding cluster) |

**v2-only additions (migrate with unified authority):**

- `KeylessFallback { method: Symbol }` on `CreateCause` / `BreakingCreateCause` — absent from dsl; merge into dsl `CreateCause` at P1.
- `IdempotentOperationRef`, `ComposableIdempotentOperationSubject`, `IdempotentOperationSubject` — testgen sample machinery.
- Node projection fns (`idempotent_shape_node`, `idempotent_operation_apply_twice`, …) — realization layer; move to compiler or dissolve when test claims ingest `EffectKind` directly.

**dsl-only (survives as authority):**

- `derive_effect_shape`, `derive_op_effect`, `OperationEffect`, `compose_effects`
- `ModifierCheck`, `check_modifier_vs_derivation` (`declared_idempotent: Bool` — **dissolve target**)
- `IdempotencyTestObligation`, `generate_idempotency_obligations`
- `CompositeKey` on `KeySource` (v2 uses `PathParam` | `InputField` only)

---

## 2. Region census

### 2.1 `dsl/std` — operation authority (P1 target)

| artifact | kind | idempotency role | post-sign action |
| --- | --- | --- | --- |
| `effects.dag` | types + fns | core authority | extend: `EffectKind`, `IdempotencyClass`, `BreakingReason`; dissolve `Bool` |
| `realization.dag` | `independence_from_effect_shapes` | reads `EffectShape` pairs | repoint to `EffectKind` |
| `behavioral.dag` | (if present) | check cross-ref at sign | no change expected |

### 2.2 `src/v2/std` — duplicate (P2 delete)

| artifact | refs | post-sign action |
| --- | ---: | --- |
| `effects.dag` | 81 internal matches | **delete** after repoint |
| Importers: `v2.lens.testgen`, `v2.test.claim.generated.idempotent_operation_conformance` | 2 direct | repoint → `dsl/std/effects` |

### 2.3 `src/v2/lens` — algebraic idempotency (P4 consumer)

| module | status | role |
| --- | --- | --- |
| `idempotency.dag` | wired, advisory | `IdempotencyVerdict`, `idempotency_witness` — **algebraic**, not effect-shape |
| `idempotency/write_effect.dag` | floor manual receipt | `EffectDependsOn` → `RequiresAlgebraWitness` |
| `idempotency/subject_roster.dag` | 1 subject | lens family eval |
| `idempotency/family_receipt.dag` | 1 runtime row | family eval |
| `idempotency/sg_claims_test.dag` | construction justification | `WallAfterGrounding` |
| `registry.dag` | enrolled | `LensRegistryEntryV0` Idempotency |
| `testgen.dag` | 13 refs | emits `idempotent_operation` claims |

**Do not migrate lens modules at P1–P2** — they consume effect facts at P4.

### 2.4 `src/v2/test/claim` — witnesses (keep green through migration)

| file | tier | discriminating? | post-sign |
| --- | --- | --- | --- |
| `impossible_bug/idempotency_contract.dag` | Unit | **yes** — violation + repetition | update imports only |
| `generated/idempotent_operation_conformance.dag` | floor #5434 | **yes** — 4 ops + label-only skip | repoint samples |
| `generated_conformance_floor_test.dag` | floor | enrollment | unchanged structure |
| `discrimination_gate/discrimination_roster.dag` | discrimination | enrolls idempotency_contract | unchanged |
| `impossible_bug/release_demos_test.dag` | release | `IdempotencyContract` class | unchanged |
| `manual/emit_ingest_type_decl_round_trip_test.dag` | manual | round-trip (out of scope) | no change |

### 2.5 `src/v2/workflow` — lens family gate

| file | role |
| --- | --- |
| `lens_idempotency_family_eval_test.dag` | advisory family eval + empty-deps control |

### 2.6 `src/v1` seed — dissolution queue (P5)

| file | matches | notes |
| --- | ---: | --- |
| `stage0/src/std_effects.rs` | 56 | GENERATED mirror of `dsl/std/effects` |
| `stage0/src/v1_compiler_effect_derivation.rs` | 6 | re-export shims |
| `effect_derivation.dag` | 6 | v1 compiler import of dsl authority |
| `tests/src/effects.rs` | 48 | integration tests — migrate to `*_test.dag` before delete |
| `tests/src/pipeline.rs` | 3 | includes `parse_emit_round_trip_idempotency` (out of scope) |
| `tests/src/parse.rs` | 1 | keyword string `"idempotent"` |

### 2.7 `dsl/extdeps` — service modifiers (P3)

| file | `idempotent` modifier sites |
| --- | ---: |
| `cloud/gcp/gcp.dag` | 1 |
| `cloud/gcp/iam.dag` | 1 |
| `cloud/gcp/iam_admin.dag` | 3 |
| `cloud/gcp/secret_manager.dag` | 1 |
| `cloud/gcp/serviceusage.dag` | 1 |
| `cloud/gcp/sts.dag` | 1 |
| **total** | **8 operations** |

Plus `languages/dag/syntax.dag`: `idempotent` in `dag_keyword_set` (keyword surface — **no change at sign**, semantics refactor at P3).

### 2.8 `dsl/gunbc` — workflow + plans

| file | role |
| --- | --- |
| `workflow/types.dag` | `Signal.idempotency_key` — **orthogonal**, no migration |
| `plans/dsl_v2_defork_audit.dag` | documents effects fork — update on sign |
| `plans/testgen_oracle.dag` | floor enrollment row |
| `plans/self_applying_lenses.dag` | idempotency = RatchetForever (algebra law) |
| `plans/emission_ingestion_inverse.dag` | cites `EffectShape` authority |
| `test_node_wall_clock_ratchet.dag` | legacy test name reference |

### 2.9 Docs + DESIGN pointers

| file | role |
| --- | --- |
| `DESIGN.md` §4 | "idempotency dissolved from Bool into EffectShape variant" |
| `docs/plans/dsl-v2-defork-audit.md` | effects grounding-cluster row |
| `docs/plans/determinism-mechanism-design.md` | cites `EffectShape` pattern as analog |
| `docs/plans/seed-shrink-census.md` | de-fork sequencing context |

---

## 3. `Bool` flag residue (construction targets)

| site | representation | dissolve to |
| --- | --- | --- |
| `is_idempotent_effect(shape) -> Bool` | dsl | `idempotency_class_of(kind) -> IdempotencyClass` |
| `IdempotencyEvidence` enum | dsl | `IdempotencyClass` + optional witness data |
| `declared_idempotent: Bool` | `check_modifier_vs_derivation` | `OperationModifiers` surface → `IdempotencyClass` projection |
| `idempotent` keyword on operations | extdeps syntax | unchanged keyword; semantics from derived class |
| `ModifierAgreement` | dsl | `Agrees \| Disagrees \| DerivationUnknown` on `IdempotencyClass` |

---

## 4. Consumer dependency graph (migration order)

```
P0  design + census + operator sign
         │
P1  dsl/std/effects unified (EffectKind + IdempotencyClass + witnesses)
         │
P2  delete v2/std/effects.dag + repoint testgen/conformance
         │
P3  extdeps modifier refactor (8 GCP ops + check_modifier)
         │
P4  infer #3468 bundle → v2.lens.idempotency construction
         │
P5  v1 effects.rs → *_test.dag + std_effects.rs GENERATED collapse
```

**Hard gates:**

- P2 blocked on P1 + operator sign.
- P3 blocked on P1 (modifier check needs `IdempotencyClass`).
- P4 blocked on #3468 seam decision (FLAG 1 — coordinate with determinism).
- P5 blocked on per-module floor witnesses (seed-shrink §5).

---

## 5. Operator sign checklist

Before any implementation PR:

- [ ] **Shape-sign** `EffectKind` + `IdempotencyClass` + `BreakingReason` algebra (§4.1 sketch)
- [ ] **FLAG 1:** `InferredFacts` bundle vs side-map (#3468 — joint with determinism)
- [ ] **FLAG 2:** `EffectKind` vs `EffectShape` export naming at de-fork
- [ ] **Confirm** three-concept disambiguation (effect / algebraic / round-trip)
- [ ] **Confirm** `Signal.idempotency_key` stays orthogonal
- [ ] **Confirm** `idempotent` keyword semantics = projection, not new keyword
- [ ] **Assign** jolly-cat-29 emitter lane for P1–P2 sequencing

---

## 6. Dissolution trigger

Delete this census when:

1. `dsl-v2-defork-audit` `effects` row reads **mechanical repoint complete**;
2. grep for `is_idempotent_effect` and `IdempotencyEvidence` returns zero;
3. `src/v2/std/effects.dag` is deleted;
4. floor witnesses green on unified authority without import shims.

Until operator sign, this file is the sole migration map — **no consumer edits**.

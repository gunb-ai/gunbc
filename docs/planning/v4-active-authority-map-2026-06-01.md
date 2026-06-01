# v4 Active authority map (Modeling DFS Arbiter)

> **Status:** LIVING — updated by Modeling DFS Arbiter (`proud-fox-405`) as worksheets ratify.
> **Date:** 2026-06-01
> **Purpose:** Prevent parallel carriers for the same concept across Rust / Go / Python / TS RCA lanes (operator directive post-#4137 §11.2).
> **Companion:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11; per-class worksheets under `docs/planning/`.

---

## How to read this map

| Column | Meaning |
| ------ | ------- |
| **Concept** | Shared modeling fact (not a lane name) |
| **Canonical home** | Single `std/` / `extdeps/` / compiler file that owns the type or row shape |
| **Per-target rows** | Where language-specific realization lives (`rust.dag`, `go.dag`, …) |
| **§8 state** | `APPROVED` = impl dispatchable; `PENDING` = worksheet in review; `BLOCKED` = prerequisite |
| **Forbidden** | Parallel authorities RCA managers must not invent |

---

## Cross-target substrate (shared carriers)

| Concept | Canonical home | Per-target rows | §8 state | Forbidden |
| ------- | -------------- | ----------------- | -------- | --------- |
| **TargetAtomRealization** | `src/v4/std/target_model.dag` | `rust.dag`, `go.dag`, `python.dag`, `typescript.dag` | Rust APPROVED (#4099); Go APPROVED (#4149); TS APPROVED (#4169, after type-expr impl) | `GoAtomRealization`, duplicate carrier in `go.dag` |
| **TargetTypeExpressionProjection** | `src/v4/std/target_model.dag` | same extdeps | Rust APPROVED (#4124); Go APPROVED (#4149); TS APPROVED (#4169, dispatch before atom) | Name-keyed `Outcome`/`Witness` tables in emit |
| **TargetCollectionRealization** | `src/v4/std/target_model.dag` | same extdeps | SG-5/6 APPROVED; **SG-COLLECTION-PROJECTION** adjudication APPROVED (#4170) — extend carrier; worksheet §8 PENDING (`witty-moth-199`); impl BLOCKED until worksheet §8 | `Vec<Rc<T>>` shim in emit without monoid→Vec row; parallel `TargetCollectionBoundaryProjection` |
| **TargetTraitEligibility** *(provisional name)* | `src/v4/std/target_model.dag` or `TargetCollectionRealization` field | Rust `Ord`/operator eligibility rows; collection rows first | DRAFT — stable-band RCA routes collection subset to SG-5; non-collection waits remeasure | Adding `derive(Ord)` / trait impls / operand unwraps by emitted type name |
| **TargetFunctionSignatureRealization** | `src/v4/std/target_model.dag` | function-boundary rows in extdeps language files | SG-1b APPROVED; E0308-A routes here | Per-function return-type patches (`String => Symbol`, guessed aliases) |
| **TargetUseSiteOwnershipRealization** | `src/v4/std/target_model.dag` | Rust SG-RC use-site rows first; other target rows as needed | SG-RC APPROVED; E0308-B routes here | Name-keyed `Rc::new` / `Box::new` / `.clone()` insertion tables |
| **TargetBundleEdge / RC layering** | `v4.std.target_model` (`TargetBundleEdge`) | emit consumes via `target_bundle_edge_*` | SG-RC APPROVED (#4100) | Conflating with collection projection |
| **Module graph / export surface** | `03_name_resolve.dag` + `05_emit_rust.dag` (M1) | follow-on `TargetModuleExportSurface` in `target_model` | SG-8 §8 APPROVED (#4143) | Per-error `pub use` patch tables |
| **LeafModelClaimId + fixtures** | `src/v4/std/leaf_model_verification.dag` | claim files under `test/claim/language_model/` | Framework APPROVED; per-target claims PENDING | Parallel claim modules outside std IDs |
| **TargetCompileVerdict** | `leaf_model_verification.dag` | `target_diagnostic_*` + `leaf_model_toolchain_*` symbols | LANDED; Go/TS leaf-model compile-bound (#4149, #4169) | `TargetGoCompileVerdict`, `TargetTypeScriptCompileVerdict` |
| **TargetRuntimeExerciseVerdict** | `leaf_model_verification.dag` (**additive**, shared) | Go R2b + TS R2b (#4149, #4169); Python L1/L2 (#4170 worksheet B/C) | APPROVED worksheets; substrate landing in impl PRs | Per-target `Target*RuntimeVerdict` types |
| **TargetPythonExerciseVerdict** | `leaf_model_verification.dag` | Python L0 only | LANDED (#4117) | Replacing L1/L2 runtime with stdout grep |
| **TargetStaticAnalysisInvocation** | `leaf_model_verification.dag` (**additive**, shared) | pyright/mypy profiles (`extdeps/typecheckers/`); later `tsc --noEmit` | APPROVED (#4170 worksheet A); substrate landing PENDING | CI-only pyright without modeled profile; extending `TargetPythonExerciseVerdict` for static |
| **TargetStaticAnalysisVerdict** | `leaf_model_verification.dag` (**additive**, shared) | per-tool diagnostic namespaces | APPROVED (#4170 worksheet A); substrate landing PENDING | Python-only static verdict vocabulary |
| **BootstrapEvaluatorCorpusRuntimeEval** | `testclaim_corpus_runner.dag` + consume `05_eval.dag` | `v4_evaluator_runtime_id` pin | APPROVED (#4143); impl PENDING (`neat-hawk-413`) | Authoring-time `data run_*` as runtime pass |
| **CiUpsertStep / TestClaim corpus CI** | `src/v4/workflow/ci.dag` | host scripts dissolve-on-arrival | P5 Layer 2 APPROVED (#4115) | New `ci.yml` shell without Upsert row |
| **Go leaf-model CiUpsertStep rows** | `src/v4/workflow/ci.dag` | Go R1/R2a/R2b/R3-external runner steps | APPROVED worksheet; CI Manager coordinates row registration | `scripts/v4-leaf-model-go-*.sh` wired only in YAML |
| **Go L1 compiler-slice receipt** | `src/v4/workflow/ci.dag` + leaf/toolchain receipt rows | `go_l1_nat_semiring_rung2` (`phase1/nat_semiring`) | APPROVED worksheet; gated on Go L0 PROVEN | Claiming full Go self-compile or using manual `go build` as L1 receipt |
| **P5 strict runtime gate** | `BootstrapEvaluatorCorpusRuntimeEval` (above) | `scripts/v4-testclaim-corpus-eval.sh` amend | Worksheet APPROVED; gate OPEN until impl | `authoring_time_verdict_surface` as P5 GREEN |
| **TargetTypeExpressionProjection residual coverage** | existing `TargetTypeExpressionProjection` in `src/v4/std/target_model.dag` | aliases, cached statics, function signatures, constructor results, closure annotations | DRAFT — SG-2 residual worksheet; no new carrier | Name-keyed generic arity table (`Outcome`, `Witness`, `TestClaimRun`, etc.) |

---

## Operator enforcement holds (cross-lane)

| Hold | Enforced by | Violation signal |
| ---- | ----------- | ---------------- |
| No Go/TS shell in `ci.yml` without `CiUpsertStep` row | CI Manager + Arbiter | New `scripts/v4-leaf-model-{go,ts}-*` wired only in YAML |
| No TS atom impl before TS `TargetTypeExpressionProjection` row | Arbiter dispatch order | Atom worker before type-expr §8 |
| No Python L2 as stdout/stderr string shell compare | Python worksheet B + Arbiter | `grep`/`diff` on process output as final receipt |
| No SG-2 residual carrier-name special-casing | Arbiter | New `if carrier == "Outcome"` in translate before SG-RC+SG-2 remeasure |
| No E0308 broad worker | Rust RCA + Arbiter | Implementation brief titled "fix E0308" or PR acceptance based on error-count reduction |
| No stable-band SG-3 blob | Rust RCA + Arbiter | Broad `E0277`/`E0560`/`E0573` patch lane without a named single-authority fact |

---

## Strict dispatch order (per lane)

### Go (gentle-lynx-68, #4145)

```text
1. go_target_atom_realization_* rows (shared TargetAtomRealization)
2. go_target_type_expr_projection_* rows (shared TargetTypeExpressionProjection)
3. leaf_model R1/R2a/R2b/R3-external claims (TargetCompileVerdict + TargetRuntimeExerciseVerdict)
4. CiUpsertStep rows (leaf-model CI worksheet)
5. L1 compiler slice (nat_semiring rung2 recommended)
```

### TypeScript alpha (fierce-fox-719, #4147 on main — §8 ratification follow-up)

```text
1. ts_type_expression_projection() — BEFORE atom
2. ts_target_atom_realization_* 
3. ts algebra inhabitance widening
4. leaf_model R2a/R2b/R3-external (shared verdict carriers — NOT TargetTypeScript*Verdict)
5. grammar-inverse TestClaims (alpha, optional parallel)
```

### Python (witty-ram-95)

```text
L0: CLOSED (#4117)
L1/L2 worksheets §8: APPROVED (#4170) — A static → B runtime → C cross-target parity
Impl: crisp-bat-22 (static), crisp-crab-696 (runtime), eager-bee-889 (parity) after substrate rows land
```

### Rust (vivid-lynx-81)

```text
SG-COLLECTION-PROJECTION: adjudication APPROVED (#4170) — extend TargetCollectionRealization; witty-moth-199 authors §10.0 worksheet before impl
SG-2 residual coverage: DRAFT; extend existing TargetTypeExpressionProjection consumer coverage, do not add carrier
E0308 fanout: route P0 slices to SG-1b / SG-RC / SG-COLLECTION-PROJECTION; no broad E0308 worker
SG-8 impl: authorized (#4143)
Other residuals: see #4148 routing worksheets (not impl without §8)
```

---

## Worksheet §8 queue (Arbiter)

| Priority | Item | PR / owner | Arbiter state |
| -------- | ---- | ---------- | ------------- |
| 1 | SG-COLLECTION-PROJECTION §10.0 worksheet | `witty-moth-199` under vivid-lynx-81 | **ready-for-worksheet-author** — adjudication CLOSED (#4170); impl blocked until worksheet §8 |
| 2 | SG-2 residual consumer coverage | vivid-lynx-81 | **DRAFT** — no new carrier; extend approved SG-2 consumption |
| 3 | E0308 stratified routing | vivid-lynx-81 | **DRAFT** — P0 fanout only; collection slice blocked on SG-COLLECTION worksheet |
| 4 | Stable rustc bands routing | vivid-lynx-81 | **DRAFT** — attach bands to SG-5/SG-8/SG-RC or hold |
| 5 | Go L0/L1 bundle (5 worksheets) | #4149 MERGED | **CLOSED** |
| 6 | TS strict-order (5 worksheets) | #4169 MERGED | **CLOSED** |
| 7 | Python L1/L2 (A/B/C + D frame) | #4170 | **CLOSED** — §8 ratified; A dispatchable; B/C blocked on Runtime/TestClaim |

---

## Amendment log

| Date | Change | Arbiter |
| ---- | ------ | ------- |
| 2026-06-01 | Initial map; P5 + SG-8 APPROVED; Go/TS verdict carrier consolidation | proud-fox-405 |
| 2026-06-01 | Go five worksheets §8 APPROVED on #4149; map Go rows → APPROVED | proud-fox-405 |
| 2026-06-01 | Post-#4149 reconciliation: R2b `go_facts_int64` anchor; SG-1 §3.1 dual-name (`go_target_atom_realization_*` + `go_atom_realization_*`) | zesty-otter-480 |
| 2026-06-01 | TS five worksheets §8 APPROVED on #4169; shared verdict carriers (no TargetTypeScript*Verdict) | proud-fox-405 |
| 2026-06-01 | Added Rust RCA/E0308/stable-band emerging shared authorities; kept draft rows non-dispatchable | eager-deer-177 |
| 2026-06-01 | Python L1/L2 worksheets §8 APPROVED (#4170); static/runtime shared carriers; SG-COLLECTION adjudication (extend TargetCollectionRealization) | proud-fox-405 |

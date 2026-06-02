# v4 Grounding / Spine Round-Robin Worksheet (RR-G)

> **Status:** RATIFIED — Branch G design closure (W2.2 + W2.5).
> **PR:** #4287 (`session/bright-dove-710`).
> **Work item:** `node://adhoc-b57d9f5d-ee8` — Grounding/Spine Mgr.
> **Gate:** substrate schema before G.1 per-language population; spine claims before G.3.5 harness.

## §10.0-adapted worksheet

```text
Migration class:        G0-GROUNDING-SCHEMA + G3-SPINE-CLAIMS (substrate authority)
Representative failure:  Per-language aliases in extdeps/ without a keyed fact-bundle registry;
                         SG-1/2/5 evidence shapes duplicated beside target_model.dag;
                         compiler spine stages lack T-38 claim rows (normalize/resolve/infer/translate).
Immediate local patch:   Hand-enumerate grounding in each RCA manager PR without G.0 schema;
                         Rust host scripts as sole spine proof.
Why forbidden:           P2 single-authority — TargetModel + model_core already own realization
                         carriers; grounding.dag must index them, not fork parallel taxonomies.
DFS path:
  G0 schema:
    - src/v4/std/grounding.dag — PerLanguageFactBundleKey, HollowAliasGovernanceBar,
      GroundingEvidenceSchema, PerTargetGroundingReceipt
  G0.2 enforcement (existing):
    - src/v4/lens/fact_density.dag — T-30 NoFact / NamedFieldFacts classifier
  G0.3 evidence home (existing):
    - src/v4/std/target_model.dag — SG-1 / SG-1b / SG-2 / SG-5 catalogs
  G3 spine claims (this PR: first two):
    - src/v4/test/claim/claim_pipeline/normalize.dag — G3.1
    - src/v4/test/claim/claim_pipeline/resolve.dag — G3.2
  G3 remainder (follow-on):
    - claim_pipeline/infer.dag, claim_pipeline/translate.dag — after G3.1-3.2 land
    - F.13 spine harness — consumes G3.5 aggregated receipt
Deepest unsound boundary:
  Declaring `type LangFoo = Bar` without a PerLanguageFactBundleEntry trace — hollow alias
  passes shape checks until T-30 fires at compile time.
Systemic fix:
  G.0 registry keys (subject_carrier × target × fact_axis; per-key fact_value only) + SG evidence schema variants
  (SG-1 / SG-1b / SG-2 / SG-5) keyed to target_model bundle edges; G.3 CompilesClaim/EqualsClaim rows per spine stage.
Non-goals:
  - Populating full G.1 Rust/Python/Go/TS fact bundles (RCA mgr charters).
  - G.2 SG executable claims (needs A.1 runner + G.1 data).
  - 06_translate audit / TargetModel yellow gates (Branch C; sunny-owl).
Falsification probe:
  fact_density hollow-alias claims reject NoFact roots; spine normalize claim fails if
  normalize rejects well-formed C3 service sugar ParseTree.
Metric allowed only as secondary:
  Count of extdeps aliases with registry entries (G.1 ratchet); not a merge gate for G.0.
```

## §1 G.0 schema placement (W2.2)

| Row | Deliverable | Authority |
|-----|-------------|-----------|
| G0.1 | `PerLanguageFactBundleKey` (`subject_carrier` + `target` + `fact_axis`) + entry `fact_value`; registry `by_key: Map<…, Node>` via fail-closed `insert_per_language_fact_bundle_entry` | `v4.std.grounding` → `primitive_fact_bundle_for_entry` → `model_core` |
| G0.2 | `HollowAliasGovernanceBar` (governance; T-30 enforces) | `v4.std.grounding` → `v4.lens.fact_density` |
| G0.3 | `GroundingEvidenceSchema` (Sg1/Sg1b/Sg2/Sg5 variants carry `source_carrier`) | `v4.std.grounding` → `v4.std.target_model` |
| G0.4 | `PerTargetGroundingReceipt` (`EmitHostRunReceipt`; target via `host_run.target`) | `v4.std.grounding` → `v4.std.host_run` |

**Wave-2-design blocker lifted for:** G.1 RCA dispatches, G.3.3–3.4 infer/translate claims, G.2 verification matrix.

## §2 G.3 spine closure (W2.5 — partial)

| Row | Claim file | Stage | Status |
|-----|------------|-------|--------|
| G3.1 | `claim_pipeline/normalize.dag` | `v4.compiler.normalize` | ✅ this PR |
| G3.2 | `claim_pipeline/resolve.dag` | `v4.compiler.resolve` | ✅ this PR |
| G3.3 | `claim_pipeline/infer.dag` | `v4.compiler.infer` | follow-on |
| G3.4 | `claim_pipeline/translate.dag` | `v4.compiler.translate` | follow-on |
| G3.5 | spine harness (aggregates 4 claims) | F.13 consumer | after G3.3–3.4 |

## §3 Cross-branch consumers

- **Branch C:** `PerTargetGroundingReceipt` for per-target host verification receipts.
- **Branch G.1:** populate `PerLanguageFactBundleRegistry.by_key` via `insert_per_language_fact_bundle_entry` per RCA manager (one `fact_value` per keyed `fact_axis`).
- **Branch G.2:** SG-1/2/5 executable claims use `GroundingEvidenceSchema`.
- **Branch A.2:** corpus folders `claim_pipeline_*` activate under T-38B when runner rows land.

## §4 Test plan

- `cargo test -p v3-compiler v4_std_grounding_dag_smoke` — parse surface for `grounding.dag`
- M1 v4 emit (CI) — full `src/v4` compile including new std + claim modules
- T-38 manual corpus — spine claims compile; execution deferred until family activation

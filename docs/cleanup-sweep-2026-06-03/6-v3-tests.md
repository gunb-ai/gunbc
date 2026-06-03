# Cleanup-sweep #6: v3-tests — concerning-patterns catalog

**Scope:** `src/v3/grounding_*` crates (6) + `src/v3/compiler/tests` (~262 `.rs`/`.dag` files).  
**Session:** crisp-bee-539 · **Date:** 2026-06-03 · **Mode:** read-only audit (freeze); no code changes.  
**Slice size:** ~262 files (brief cited ~233; delta is `tests/fixtures/` + `tests/dag/`).

## Slice — overall scariness: 🟡

Bulk is yellow-gated v4 parse/string smokes and one **W3** host-transport cluster (`emit_host_bridge`); not load-bearing pipeline stages. Grounding crates are smaller but carry private digest towers and a pilot mirror. Many `bridge_*` tests are **retirement ratchets** (anti-bridge), not active bypasses.

**Escalation candidate:** `v4_workflow_ci_runner_dag_smoke_test.rs` alone is ~2734 LOC — consider a dedicated megaharness slice if synthesis needs file-level granularity.

---

## Per cluster / file (worst-first)

### 🔴 Megaharness cluster — `tests/integration/v4_*` (26 smokes + 4 anchors)

| Path / cluster | | Bridges | Towers | Note |
|---|---|---|---|---|
| `v4_workflow_ci_runner_dag_smoke_test.rs` | 🔴 | v4 `.dag` via `include_str!`; parse-only (no cross-module merge); P5 `Dissolve-on:` → T-PB-B | `BTreeSet`/`str::contains` CI claim rosters; `content_hash(...)` string pins in source; manual Upsert/receipt persistence asserts | Largest hand-Rust CI ratchet until `.dag` TestClaimRun + emitted `ci.yml` |
| `v4_compiler_emit_translate_smoke_test.rs` | 🔴 | v4 emit/translate `.dag`; tokenize/parse gate | closed-vocab surface scans, substring authority | Peer dissolution lane (T-22 / Shape-B) |
| `v4_emit_host_harness_test.rs` | 🔴 | **W3** — substrate `transport_not_wired`; real work via `emit_host_bridge` + `tools/emit_host_runner`; 15+ v4 `include_str!` | per-law×target host matrix; DAG string-literal escape/extract; MVP-2 stdout parity outside `.dag` TestClaim | **Highest-risk live bridge** |
| `v4_test_bootstrap_infra_closeout_test.rs` | 🔴 | v4 testgen/bootstrap/cli/eval bundle; parse-only T-22 | `BTreeMap`/`BTreeSet` inventories; director-locked lists; logic migrated from deleted `check_t19_testgen_activation.py` | Bootstrap-infra hand tower |
| Remaining 22× `v4_*smoke_test.rs` | 🟡 | same template: `include_str!(../../../../v4/...)`, `tokenize_for_test`/`parse_for_test` | `str::contains`, set diffs, occasional `content_hash` substring pins | Homogeneous; dissolve as a **class** not file-by-file |

**Non-test bridge (SG-0 non-test ratchet):** `src/v3/compiler/src/emit_host_bridge.rs` — W3 executable host transport until T-22 substrate eval dispatches host rows.

### 🟡 Grounding crates — `src/v3/grounding_*`

| Path / cluster | | Bridges | Towers | Note |
|---|---|---|---|---|
| `grounding_tests/src/stratum_a.rs` | 🟡 | uses `generated_full_bootstrap_dag()` only (avoids mis-pathed `include_str!` to std) | **`RowFingerprint` + `list_digest_from_fingerprints`**; closed field tables; `EXPECTED_STRATUM_A_ROW_COUNTS`; `LANGUAGE_SPEC_COLLECTION_OPS_CONTRACT_WITNESSES` | Hand-rolled Projection/content_hash around `MethodTemplateContract` rows |
| `grounding_engine/src/lib.rs` | 🟡 | **`RUST_PILOT_PRIMITIVES` mirror** until retirement | `BTreeMap`/`BTreeSet` enum-shape walk; sorted multiset `target_name` parity | Hand-rolled registry enumeration |
| `grounding_cross_target_meta/src/walker.rs` | 🟡 | **`fallback_targets`** when `language_spec_targets` missing | hardcoded `rust/python/go_shape_a_target` name lookup | L6 scaffold; bypasses data-driven targets |
| `grounding_coercion_fold/` | 🟢 | none (consumer only; SG-0 no compiler edits) | fold over declared `TargetIntegerTypeInhabitance` | Modeled-path consumer; `FoldNotImplemented` fail-closed |
| `grounding_tests/src/stratum_b.rs` | 🟢 | readiness checklist only | `StratumBPrerequisite` closed enum | Explicit scaffold; no production fold asserts |
| `grounding_pilot/` | 🟢 | OrderedRing→`i64` fallback noted in-code | pilot primitive tables | Source for engine mirror |
| `grounding_lifetime/` | 🟢 | none | axes/program extraction | Sibling-dep smoke only in `grounding_tests` |

### 🟡 Shared test infra — `tests/integration/common/`

| Path | | Bridges | Towers | Note |
|---|---|---|---|---|
| `common/mod.rs` | 🟡 | **`RustcHarness`** (hand `rustc` spawn; `RUSTC_BOOTSTRAP` strip); `run_on_larger_stack` 32MiB | `require_fixture_cost_*` duplicate `CostLookup` interpretation | Boundary + emit tests depend on this |
| `common/cached_compile.rs` | 🟢 | `OnceLock` compile cache (perf) | none | Legitimate amortization |
| `common/r1_gates_bridge.rs` | 🟡 | `compile_to_dag` on `r1_gates.dag` for ExecuteCommand gates | none | Small fixture bridge |

### 🟡 Boundary cluster — `tests/boundary/v4_leaf_model_*` (10 files)

| Cluster | 🟡 | v4 claim `.dag` + **`RustcHarness`** + shell scripts | hand `unescape_dag_string_literal` / fixture extract from `.dag` text | Until T-22 `run_target_verification` |

### 🟡 Governance meta — `tests/integration/sg0_census_test.rs`

| Path | 🟡 | documents `emit_host_bridge`, `emit_rust_bin_shim`; 204 test + 63 non-test hand-authored paths | filesystem walk + expected-set diff | Not a product bridge — PB bankruptcy inventory |

### 🟢 Bridge retirement / substrate carriers (lower concern)

| Cluster | | Note |
|---|---|---|
| `bridge_ledger_carrier_test.rs` | 🟢 | Tests **modeled** `bridge_ledger.dag` carrier — not a bypass |
| `canonical_lens_bridge_ratchet_test.rs` | 🟢 | Zero-residual for **retired** `test_runner` lens-name dispatch |
| `bridge_lower_helpers_patch_zero_residual_test.rs` | 🟢 | Zero-residual for retired `patch_lower_helpers` |
| `method_template_projection_emit_shim_coherence_test.rs` | 🟢 | Gap-4 **retirement** — shim must be absent |
| ~134 other SG-0 integration tests (m1/m2/R3/substrate carriers) | 🟢/🟡 | Lower bridge density than v4 cluster; mostly structural Dag asserts |

---

## Concerning patterns (unmodeled repetitive work)

Focus: recurring hand-rolled towers and bridges that should collapse to **one modeled surface**.

### 1. v4 parse-surface smoke class (26+ files)

- **Pattern:** `include_str!(../../../../v4/...)` + `tokenize_for_test` / `parse_for_test` + `str::contains` / `BTreeSet` roster of claim names, module paths, or projection strings; yellow `Dissolve-on:` headers pointing at T-PB-B / T-22 / A15 Shape-B.
- **Where it recurs:** All `tests/integration/v4_*smoke_test.rs`; anchors `v4_workflow_ci_runner_*`, `v4_compiler_emit_translate_*`, `v4_test_bootstrap_infra_closeout_*`.
- **One shared dissolve:** `.dag` **`TestClaim` execution** (`TestClaimRun`) + **`verification.dag` / workflow eval** producing receipts; retire hand-Rust string inventories when claims run from substrate and CI surface is Shape-B emitted `ci.yml`.

### 2. W3 host-transport bridge (emit-host lane)

- **Pattern:** Substrate rows `transport_not_wired` / fail-closed; real compile+run via `emit_host_bridge.rs` + `tools/emit_host_runner`; harness builds per-law×target matrix and compares MVP-2 five-byte stdout outside `.dag` TestClaim.
- **Where it recurs:** `src/v3/compiler/src/emit_host_bridge.rs` (non-test); `v4_emit_host_harness_test.rs`; related `v4_emit_host_eval_dispatch_test.rs`; boundary leaf tests via `RustcHarness`.
- **One shared dissolve:** **T-22 substrate eval** dispatch for `run_emit_host_rust/python/go` assembling `EmitHostRunReceipt` from typed host facts; delete `emit_host_bridge` when `.dag` owns transport.

### 3. Hand-Rust digest / fingerprint tower (grounding)

- **Pattern:** Ad-hoc `RowFingerprint` struct, per-row string projection, `list_digest_from_fingerprints` (sorted multiset hash), lockstep digests across two `generated_full_bootstrap_dag()` runs.
- **Where it recurs:** `grounding_tests/src/stratum_a.rs` (primary); diagnostic variants in `grounding_tests/src/diagnostic.rs`.
- **One shared dissolve:** Substrate **`Projection`** + **`content_hash`** over `MethodTemplateContract` list rows (registry-as-data); tests assert canonical hash equality, not private concat rules.

### 4. Pilot mirror + multiset parity (grounding engine)

- **Pattern:** Duplicate authority — bootstrap Dag walk compared to `v3_grounding_pilot::RUST_PILOT_PRIMITIVES`; sorted multiset `target_name` equality; local enum-shape extraction (`BTreeMap`/`BTreeSet`).
- **Where it recurs:** `grounding_engine/src/lib.rs`; pilot tables in `grounding_pilot/src/lib.rs`.
- **One shared dissolve:** Substrate **enumeration / `TotalMap`** over landed primitive declarations; single read path, retire pilot mirror crate surface.

### 5. DAG-text fixture extraction + rustc harness (boundary)

- **Pattern:** Parse `.dag` source as text to extract string-literal fixtures; hand `unescape_dag_string_literal`; `RustcHarness::compile` + `Command::new("rustc")` with env hygiene (`RUSTC_BOOTSTRAP` removal).
- **Where it recurs:** `tests/boundary/v4_leaf_model_*` (10 files); shared `tests/integration/common/mod.rs`; legacy m1 emit boundary tests.
- **One shared dissolve:** Modeled **`run_target_verification`** + typed **fixture carriers** in `.dag` (not text scrape); host receipts from substrate eval.

### 6. CI / bootstrap string ratchets (megaharness)

- **Pattern:** `str::contains` on full `.dag` source for claim names, `content_hash(n: …)` spellings, gate wiring strings; `BTreeMap`/`BTreeSet` structural inventories; script logic re-homed from deleted Python checks.
- **Where it recurs:** `v4_workflow_ci_runner_dag_smoke_test.rs`, `v4_test_bootstrap_infra_closeout_test.rs`, parts of `v4_compiler_emit_translate_smoke_test.rs`.
- **One shared dissolve:** Same as (1) — **TestClaimRun receipts** + modeled **CI selection / receipt persistence** (`CiSelectionReceipt`, Upsert projection nodes) evaluated, not substring-guarded.

### 7. Registry-as-data in Rust (governance + rosters)

- **Pattern:** Large `const EXPECTED_*` tables, filesystem walks, set-difference ratchets; director-locked row counts; collection-ops witness tables in Rust.
- **Where it recurs:** `sg0_census_test.rs`; `stratum_a.rs` witness tables; v4 smokes with director-locked category lists.
- **One shared dissolve:** **Generated inventory** from build graph or modeled `HandAuthoredInventory` / registry declarations; SG-0 stays a diff against derived facts, not parallel lists.

### 8. `language_spec_targets` fallback (cross-target meta)

- **Pattern:** When substrate metadata missing, hardcode `rust/python/go_shape_a_target` declaration ids into `ShapeATarget` list.
- **Where it recurs:** `grounding_cross_target_meta/src/walker.rs` (`fallback_targets`).
- **One shared dissolve:** Fail-closed on missing targets only, or substrate **`language_spec_targets`** as sole authority (no name-keyed fallback path in steady state).

---

## Recurring patterns (top 5)

1. **v4 parse-surface smokes** — `include_str!` + tokenize/parse without full v4 merge or TestClaimRun.
2. **W3 host-transport** — `emit_host_bridge` + harness while substrate stays `transport_not_wired`.
3. **Hand-Rust digests** — `RowFingerprint` / list digests; pilot multiset mirrors.
4. **Shared test scaffolds** — `RustcHarness`, `cached_compile`, `run_on_larger_stack`, fixture cost helpers.
5. **Bridge retirement ratchets** — ledger/zero-residual tests (lower fear; guard dissolution).

---

## Missing-substrate map

| Hand-rolled tower | Shared derived-op surface that dissolves it |
|---|---|
| Stratum A `RowFingerprint` / `list_digest_from_fingerprints` | `Projection` + `content_hash` on `MethodTemplateContract` rows |
| v4 smokes `str::contains` / `BTreeSet` claim rosters | `.dag` `TestClaim` execution + `TestClaimRun` receipts |
| `emit_host_harness` host-process matrix | T-22 eval dispatch for `run_emit_host_*`; delete `emit_host_bridge` |
| `grounding_engine` vs `RUST_PILOT_PRIMITIVES` | Substrate enumeration / `TotalMap` over primitives |
| `fallback_targets` in L6 walker | `language_spec_targets` only (fail-closed) |
| `RustcHarness` + boundary fixture extractors | `run_target_verification` + typed `.dag` fixture carriers |
| `sg0_census_test` expected-path sets | Generated census or modeled `HandAuthoredInventory` |
| `common/require_fixture_cost_*` | Single substrate `CostLookup` interpretation accessor |

---

## What is *not* concerning here

- **`bridge_ledger_carrier_test`**, **`canonical_lens_bridge_ratchet`**, **`bridge_lower_helpers_patch_zero_residual`** — ratchet **modeled retirement** or zero residual, not active bypass.
- **`grounding_coercion_fold`** — consumes declared projection facts; explicit `FoldNotImplemented` for undeclared cases.
- **`cached_compile`** — performance amortization with outcome-kind preservation, not semantic duplication.

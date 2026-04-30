# R2 — Evaluator: `test_runner.rs` test-predicate authority audit & ratchet

**Status:** AUDIT (docs-only). **Scope:** inventory the bespoke Rust predicate/producer surfaces in `src/v3/compiler/src/test_runner.rs` without changing code. **Does not** duplicate Worker C’s PR-B.2/3/4 runner-extension bundle brief — treat [PR #1315](https://github.com/gunb-ai/gunbc/pull/1315) and [`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) (authoritative copy on that PR branch until merged; may not exist in every checkout of `main` yet) as the **consumer** of this inventory; this document names **why** parallel Rust authority exists and **where** it should dissolve.

**Primary code anchor:** This PR is **docs-only** — it does not modify Rust. The audited implementation lives on the merge target (**`main`**): `TestRunner::run_claim` in `src/v3/compiler/src/test_runner.rs` (e.g. ~1497+) dispatches `TestPredicate` variant labels to Rust evaluators (`Compiles` through `MockBackedInvariant`); unknown labels return `NotYetImplemented`.

**Cross-reads (required by dispatch; paths below are **tracked on `main`** and are **not** added or edited by this docs-only PR):**

- [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — Evaluator program scope, PR-A–PR-E cadence, PB-Runtime convergence, R3 lane gates.
- [`ROADMAP.md`](../ROADMAP.md) — §"Scheduled cleanups: LensOutputEquals runner and R1 gate fixtures" (PR #764 checklist); §"Post-merge debt (2026-04-25 reflective + exploratory analyses)" → **Filename / sentinel bridges in `test_runner.rs`** and **B4 bridge-retirement queue** (identity-carrier sequencing). § here means prose gloss for `ROADMAP.md` **`###` headings** / list rows on `main` (merge-base snapshot ≈ **L87**, **L454**, **L476+** — lines drift with edits).
- **Not found at authoring time:** `ROADMAP.md` has no subsection titled `### Post-merge debt (2026-04-30 analyses)` — use the 2026-04-25 post-merge block above for the closest ledger alignment on `test_runner` bridges.

**In-tree scope (merge readiness):** The inventory of Rust predicate/producer behavior is grounded only in **`src/v3/compiler/src/test_runner.rs`** as present on `main`. **`docs/briefs/r2-evaluator-manager.md`** and **`ROADMAP.md`** are in-tree cross-reads (headings quoted above resolve under those paths). **[PR #1315](https://github.com/gunb-ai/gunbc/pull/1315)** and **`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`** name the **bundle consumer** for forward dissolution coordination; that brief may be absent on `main` until PR #1315 merges — it is **not** evidence this audit depends on out-of-tree artifacts for its factual claims about the current runner.

---

## 1. Problem statement (Director discipline)

`test_runner.rs` is the **host implementation** of `.dag` `TestClaim` evaluation today. Each new `match` arm adds a second place (alongside future PB-Runtime / eager evaluator paths) where test truth can live. Without an explicit inventory, the file becomes a **parallel test-predicate authority** — exactly the correction this audit addresses.

**Intent:** make the surface **auditable** and attach each arm to a **named dissolution lane** (PR-B eager evaluator, PR-B.2/3/4 runner debt, PB-Runtime-generated tests, T-LensProducer-Retirement, T-FixedPoint, T-TestGen, substrate identity carriers, etc.).

---

## 2. Inventory by category

Below, **Rust authority surface** means what the runner does in-process that a pure `.dag` evaluator would eventually own, mirror, or replace.

### 2.1 `ExecuteCommand`

| Field | Content |
| --- | --- |
| **Rust authority surface** | Parse `(command, List<String> args, expect_exit_code)` from lowered payload; require **clean compile** of `claim.source`; spawn host process with policy (timeouts, stdio nulling, namespace/bootstrap helpers, shell `-c` guards). Large supporting API: `evaluate_execute_command_exit_code`, `ExecuteCommandHostOutcome`, unshare/bootstrap stages. |
| **Why it exists** | PB-Runtime boundary (PR #792 / ROADMAP T-PB-B): express **bounded** external toolchain checks in CI as data (`verification.dag` schema). The compiled `Dag` is **not** passed to `std::process::Command` — only the predicate payload drives the child (commented in-file). |
| **Dissolution target** | **PB-Runtime / T-PB-B** — keep the *semantic* of bounded host execution as `.dag`-first; **PR-B eager evaluator** only where the program itself must *interpret* command construction (today: payload-only). Longer term: structural spawn receipts if the substrate models the boundary without duplicating policy in a second Rust fork. |
| **Retirement** | `.dag` claims can be evaluated without maintaining a second copy of spawn policy in `test_runner.rs` **or** the file shrinks to a thin FFI to one generated/declared authority. Count of bespoke ExecuteCommand policy branches **non-increasing** once PB-Runtime owns the vocabulary. |

### 2.2 Release deferral markers — `ReleaseDeferredClaim`

| Field | Content |
| --- | --- |
| **Rust authority surface** | Hard-coded fixture path `r1_release_acceptance.dag` only; validates three `DeclarationRef` fields against **fixture-local** role types (`R1GateMarker`, `TargetLaneMarker`, `ReleaseAuthorityDoc`) and file-span checks. |
| **Why it exists** | Concession-encoding pattern from R1 closure (see `r2-evaluator-manager.md` “R1 test-infrastructure precedent”): structural proof that a deferral row cites real markers — **shape check**, not execution of deferred work. |
| **Dissolution target** | **R2 Release / concession ledger** consumers; eventual **PB-Runtime TestClaim** suite if deferral claims become fully data-driven without path locks. |
| **Retirement** | Fixture-path sentinel removed; marker validation lives in one authority (generated query or single validator module keyed off substrate roles, not string path). |

### 2.3 Substrate research deferred claims — `SubstrateResearchDeferredClaim`

| Field | Content |
| --- | --- |
| **Rust authority surface** | Same shape as release deferrals but fixture locked to `tc1_substrate_lens_eta_equivalence_deferred.dag`; roles: `Tc1ResearchGateMarker`, `SubstrateLensPrimitiveTargetLaneMarker`, `LambdaCalculusGroundingAuthorityDoc`. |
| **Why it exists** | TC1 / R2 substrate research lane: fail-closed proof that deferred claims are declared in the **one** authorized TC1 fixture (constants at top of `test_runner.rs`). |
| **Dissolution target** | **T-Substrate** research closure + **Evaluator** readiness when TC1 content promotes; **PR-B.2/3/4 bundle** may fold TC-shaped deferrals into the same dissolution hooks as other runner extensions (see [PR #1315](https://github.com/gunb-ai/gunbc/pull/1315)). |
| **Retirement** | No runner hard-coded path list; deferral validity is structural (role + graph) or evaluated by substrate-carried witness. |

### 2.4 PB census resolution — `CensusBoundCheck`, `CensusSubsetCount`, `RatchetZero`, and census-backed pieces of `GeneratedFromDag`

| Field | Content |
| --- | --- |
| **Rust authority surface** | **SG-0 text mining:** `include_str!` of `sg0_census_test.rs` parsed by string markers (`EXPECTED_HAND_AUTHORED_*` slices) via `sg0_string_slice_constant_entries`. `CensusSubsetCount` filters paths with **hard-coded** `is_lens_producer_census_path` (three files). `RatchetZero` counts substring presence for three `infer_helpers.dag` type names via `INFER_HELPERS_SOURCE` text scan. `GeneratedFromDag` intersects claim-declared paths with `GENERATED_FILES` **and** reuses SG-0 hand-authored test list counts. |
| **Why it exists** | R1C-D “census-as-`.dag`” — ratchet **compiler std** / lens-producer / generated-file sets without silent drift; predates full structural census queries in the evaluator. |
| **Dissolution target** | **T-PB-A / PB-Runtime generated tests** — census facts should eventually come from declared tables or generated queries, not parallel string parsing. **T-LensProducer-Retirement** for lens-producer file enumeration. Compiler–std thesis ratchet (`ROADMAP.md`) for positive-set types. |
| **Retirement** | Runner does not `include_str!`+parse Rust sources for counts; census lists are **one** `.dag` or generated artifact consumed by both SG-0 tests and `TestClaim` evaluation. Lens-producer subset predicate reads substrate/registry, not a three-path `matches!`. `RatchetZero` retires when the three workaround types dissolve per compiler–std consolidation. |

### 2.5 Canonical lenses, host mirrors, and identity bridges — `LensOutputEquals`, `DifferentialEquals`, helpers

| Field | Content |
| --- | --- |
| **Rust authority surface** | **`R1_CANONICAL_*` `include_str!`** for `named_function_count` and `complexity.dag` bytes. **Name-keyed dispatch:** `lens_decl.name == "cost_of"` → `lens_cost::cost_of` + `Lookup<Int>` deferred to Int literal compare; `named_function_count` → compile canonical lens `Dag` instead of fixture body. **`lane_e_host_*`:** host forward fold duplicating emit order for `v3_program_cost` in `DifferentialEquals`; oracle side uses generated `cost_of`. `eval_algebraic_law_for_claim_program` + fixed witness triples. `reflect_program_dag_nodes_in_file` + `ProgramInput` / `ProgramOutputBind` string bridging. Role checks via `decl_inhabits_named_role` by **type name** lookup in fixture `Dag`. |
| **Why it exists** | T-LensAPI / T-LaneE receipts (PR #764): D1 `apply_lens_declaration` cannot run full `cost_of` / loops yet; need parity between host fold and generated lens. P2 single authority for lens tests still blocked on M1(2.8) and structural `DeclarationRef` for executable lens identity. |
| **Dissolution target** | **PR-E lens application** + **PR-B body evaluator** (when `Loop` / strategy land): replace `lane_e_host_*` with `apply_lens_declaration` on canonical `cost_of`. **T-LensProducer-Retirement** for producer enumeration; **R3 T-FixedPoint** is adjacent for pipeline convergence claims, not necessarily this lens block. **B4 identity-carrier** queue (`ROADMAP.md`) for `include_str!` and name dispatch. |
| **Retirement** | No `include_str!` side channels; no name-string lens dispatch; `DifferentialEquals` generalizes beyond the single cost pairing or defers to `NotYetImplemented` until the evaluator supports declared lineages. Host cost fold deleted. |

### 2.6 Fixed-point checks — `FixedPointConverges`

| Field | Content |
| --- | --- |
| **Rust authority surface** | Only accepts literal keys `default_fixed_point_source` + `pipeline_stage_snapshots`; runs `compile_stage_snapshots` twice and compares with `compare_stage_snapshots`. |
| **Why it exists** | **T-FixedPoint** / pipeline regen convergence: prove deterministic stage snapshots without manual diff tests. |
| **Dissolution target** | **T-FixedPoint** lane — structural fixed-point theory in `.dag`; runner is a thin witness until the evaluator can consume pipeline-stage equality claims directly. |
| **Retirement** | Predicate accepts arbitrary declared compile targets / snapshot refs supported by substrate; Rust stops hard-coding allowed string pairs. |

### 2.7 Generated-from-Dag — `GeneratedFromDag` (file-set half)

| Field | Content |
| --- | --- |
| **Rust authority surface** | Authoritative intersection with `crate::generated_files::GENERATED_FILES` + SG-0 `expected_hand_authored_test` census (see §2.4). |
| **Why it exists** | Enforce “tests live under generated paths” discipline for PB/runtime-generated tests migration. |
| **Dissolution target** | **PB-Runtime generated tests** — same list should be emitted from one `.dag` manifest; runner consumes declared facts only. |
| **Retirement** | No duplicate `GENERATED_FILES` vs census cross-check in Rust; one structural source. |

### 2.8 Structural / compile-graph predicates (lower “bespoke” risk, still Rust-centralized)

These are **still** implemented only in Rust today; they are “Day-1” predicates per ROADMAP but remain part of the runner’s authority surface until PB-Runtime can evaluate them structurally.

| Predicate | Rust authority surface (summary) | Dissolution target | Retirement |
| --- | --- | --- | --- |
| `Compiles` | `compile_to_dag` success/failure | PR-B + PB-Runtime TestClaim eval | Evaluator compiles as library call inside `.dag` runtime |
| `FailsWithDiagnostic` | Diagnostic matching vs `DiagnosticReference` | Same | Structured diagnostic compare in data |
| `OutputEquals` | Renders first `data` value via `render_value_body` | Same | Same |
| `PortHasState` | Inspects `PortState` after compile | PR-A carriers + evaluator | Exposed as structural witness |
| `DeclarationHasRefinement` | Checks lowered refinement edge | Substrate query | Same |
| `CostBounded` | `cost_of` on bind vs bound | PR-E + `cost_of` via evaluator | No duplicate cost oracle |
| `AlgebraicLaw` | Currently `Associativity` only; witness triples | PR-D/E witness + substrate laws | Declared law metadata vs sample triples |
| `MockBackedInvariant` | `apply_lens_declaration` ×2 + `requires` validation | T-TestGen / DB-15 | Full mock semantics in `.dag`; empty-`requires` stays `NotYetImplemented` by design today |

---

## 3. Ratchet recommendation

1. **Freeze** new `match` arms in `run_claim` unless the PR **names** a dissolution hook (adjacent ROADMAP / manager brief row / [PR #1315](https://github.com/gunb-ai/gunbc/pull/1315) bundle item) — same discipline as `docs/r2-structure.md` P5 for hand-Rust under `src/v3/`.
2. **Prefer count- and path-pinned ratchets only with an explicit “decreasing” story:** census / generated-file / positive-set checks should transition from **pinned integers** to **monotone decrease** (or zero-only) when the **next substrate or evaluator carrier** lands — never silent increases.
3. **PB-runtime / Evaluator split:** host-only policy (`ExecuteCommand`) stays explicitly **PB-Runtime debt**; graph semantics (`Compiles`, lens apply, cost lineage) tracks **PR-B / PR-E** — avoid solving both in one ad-hoc arm.
4. **When [`r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) is on `main`:** add a one-line cross-reference from that brief back to **this inventory** as the authoritative categorization; do not fork the tables here.

---

## 4. Named gates (smallest blocking increments)

Docs-only audit — **no implementation request.** If a future worker needs a **single** place to open the substrate/evaluator seam:

- **Gate A — Identity:** B4 / `DeclarationRef` carriers replace name + `include_str!` bridges (`ROADMAP` B4 queue item 2–3).
- **Gate B — Cost lineage:** PR-A.3 + D1 `Loop` sufficient for `apply_lens_declaration(cost_of)` → delete `lane_e_host_*` (`r2-evaluator-manager.md` PR-A.3 blocker audit).
- **Gate C — Census:** SG-0 lists emitted or queried from one artifact consumable by both integration tests and runner (retire string parsing of `sg0_census_test.rs`).

Until then, **stop** at the smallest of A/B/C rather than growing new runner branches without hooks.

---

## 5. Related

- [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md)
- [`ROADMAP.md`](../ROADMAP.md) — Scheduled cleanups (LensOutputEquals / PR #764), Post-merge debt 2026-04-25 (`test_runner` / B4 rows)
- [PR #1315](https://github.com/gunb-ai/gunbc/pull/1315) — PR-B.2/3/4 runner-extension bundle; consumer brief [`docs/briefs/r2-pr-b-2-runner-extension-bundle.md`](r2-pr-b-2-runner-extension-bundle.md) (merge with PR; may be absent on `main` until that PR lands)

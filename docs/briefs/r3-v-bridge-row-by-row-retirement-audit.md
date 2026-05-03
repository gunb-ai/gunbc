# R3 Verification - Bridge Row-By-Row Retirement Audit

**Status:** AUDIT RECEIPT - docs-only. This extends the bridge-retirement ledger-zero ratchet with per-open-row retirement paths; it does not flip any `BridgeLedgerRow.status`.

**Scope:** the four rows currently `Open` in `src/v3/std/bridge_ledger.dag`:

- `bridge_source_span_file_participation_retired`
- `bridge_canonical_lens_name_patching_residual`
- `bridge_include_str_side_channels_retired`
- `bridge_exact_string_patching_residual_retired`

## Contract Restatement

Verification owns the unified ledger and the decreasing-open-count ratchet; owner programs own the work that makes a row retire. The ledger-zero witness shape in [`r3-v-bridge-retirement-ledger-zero-witness-shape.md`](r3-v-bridge-retirement-ledger-zero-witness-shape.md) keeps this separation explicit: natural owners flip rows from `Open` to `Retired`, and Verification observes the structural ledger.

This receipt therefore records, for each open row:

1. the live bridge surface,
2. the typed carrier or carrier family that replaces the bridge,
3. the owner program that must land the retirement work, and
4. the blocking gap that keeps the row open.

The audit follows `INVARIANTS.md` P5 dispatch discipline for string/path/name identity bridges (`span.file ==`, `include_str!`, sentinel/name routing, exact-string patches) and the T-Bridge-Retirement distribution map in [`docs/r3-structure.md`](../r3-structure.md).

## Row Audits

### `bridge_source_span_file_participation_retired`

| Field | Audit |
|---|---|
| **(a) Bridge surface** | This row is a family of production participation/inclusion decisions keyed on source path. The live surfaces are: `src/v3/compiler/src/lens_apply.rs` filters reflected program behavior with `behavior_source_file(b) == source_file` in `reflect_program_dag_nodes_in_file`, and `behavior_source_file` reads each node's `span.file`; `src/v3/compiler/src/lower.rs` has `span.file == "dsl/std/types.dag"` in `lower_type_alias_refinements_phase`, `DIMENSION_STD_AUTHORITY_FILE` checks for the `Dimension` authority file, and `declaration_name_preference_rank(&span.file)` / `declaration_name_preference_rank(&d.span.file)` for duplicate-authority preference; `src/v3/compiler/src/emit.rs` reads `SourceFilteringBinding` structurally but still applies it to `decl.span.file` and `bind.span.file` through `source_filtering.excludes`. `docs/r3-structure.md` T-Bridge-Retirement names these exact surfaces as the current open state. |
| **(b) Substrate carrier consumer** | The replacement is not a single carrier yet. Existing partial structural surfaces include `SourceFiltering` / `ShapeATargetSourceFiltering` and `SourceFilteringBinding` in `src/v3/std/emit_model.dag` / `src/v3/compiler/src/emit.rs`, `Dag::post_bootstrap_declaration_append_begin` in `src/v3/compiler/src/dag.rs` for post-bootstrap identity without consulting `span.file`, and `BranchEmitParticipation` for the already-named user-match emit participation class. Remaining carriers need to land as typed module/compilation-unit identity for lens reflection, typed authority carriers for lower-time std/documentation-only decisions, structural declaration-source authority for duplicate resolution, and emit-scope participation carriers that let emit decide inclusion without raw file paths. |
| **(c) Owner-program-of-retirement** | Substrate owns the typed identity surface. `docs/r3-structure.md` line item T-Bridge-Retirement assigns `SourceSpan.file` participation checks to **Substrate**; the ledger file records owner `R3` only because Verification owns the row accounting and ratchet. The lane anchor is `docs/r3-structure.md` T-Bridge-Retirement plus the ROADMAP `lens-fold-file-path-semantics` authority cited by `src/v3/std/bridge_ledger.dag`. |
| **(d) Gap analysis** | **(i) Carrier needs to land first** and **(ii) consumer pattern needs naming**. Partial deletion would leave parallel participation authorities in lens reflection, lower, and emit. Retirement requires a mapped carrier family per consumer class, then migration of every production inclusion decision to those carriers. The source filtering consumer already has a structured policy declaration, but it still consumes `span.file`; that is not enough to retire the row. |

### `bridge_canonical_lens_name_patching_residual`

| Field | Audit |
|---|---|
| **(a) Bridge surface** | The residual bridge is canonical-lens text/name authority in `src/v3/compiler/src/test_runner.rs`. Two canonical lens byte constants use `include_str!`: `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` and `R1_CANONICAL_COMPLEXITY_LENS`. The `LensOutputEquals` path dispatches on `lens_decl.name.as_deref() == Some("cost_of")` and `Some("named_function_count")`, compiles `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` by path for the named-function-count case, and still relies on name-keyed lens identity instead of a cross-Dag typed lens body identity. The residual is documented in [`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md). |
| **(b) Substrate carrier consumer** | Existing carrier: `DeclarationRef` fields on `LensOutputEquals` in `src/v3/std/verification.dag` provide fixture-local typed edges. Missing carrier/consumer: a typed lens-registry or equivalent cross-Dag lens identity carrier, or PB-Runtime interpreter-as-data that resolves executable lens bodies from structural identity rather than canonical string names / `include_str!` text. The disposition doc explicitly rejects replacing the current `include_str!` bridge with another string registry. |
| **(c) Owner-program-of-retirement** | PB Manager / PB-Runtime owns the main path, adjacent to T-LensProducer-Retirement. `docs/r3-structure.md` maps canonical lens-name dispatch to **PB Manager**; [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md) names PB-Runtime structural dispatch as the dissolution path for the canonical-lens bridge. A substrate-introduced typed lens-registry carrier would be Substrate-owned if chosen, but the currently named owner lane is PB-Runtime. |
| **(d) Gap analysis** | **(i) Carrier / PB-Runtime producer needs to land first**. Fixture-local `DeclarationRef` is not enough because canonical lens bodies live in a different compilation unit / Dag. Strict retirement needs either PB-Runtime interpreter-as-data to make the runner consume the structural lens body authority, or an explicitly routed Substrate lens-registry carrier. Until then, the current include/name arms remain the live bridge. |

### `bridge_include_str_side_channels_retired`

| Field | Audit |
|---|---|
| **(a) Bridge surface** | The load-bearing open site is `src/v3/compiler/src/pipeline_authority.rs`. `ordered_pipeline_stages` reads stage order structurally from `PipelineStageBinding` rows, but the module comment records that prior drift checks used `include_str!("../pipeline.dag")` or `std::fs::read_to_string` plus span slicing and were rejected for this lane. The `pipeline_compile_body_remains_unparsed_blocking_structural_retirement` test still finds `compile` by `decl.name == "compile" && decl.span.file == PIPELINE_AUTHORITY_FILE` and asserts the body is `ArrowBody::Unparsed`; that test preserves the open blocker instead of proving retirement. |
| **(b) Substrate carrier consumer** | Existing carrier: `PipelineStageBinding` declarations in `src/v3/compiler/pipeline.dag`, consumed by `ordered_pipeline_stages`, are the sole runtime ordering authority for stage order today. Missing carrier: a lowered structural compile-body witness, or a single authored carrier that represents both the compile orchestrator body and stage binding order. The target replacement must make compile-body-vs-binding drift checkable without file IO or embedded source text. |
| **(c) Owner-program-of-retirement** | PB Manager owns the compiler-internal bootstrap / pipeline-authority path. `docs/r3-structure.md` maps `include_str!` side channels such as `pipeline_authority.rs` to **PB Manager**, with PR #1171 as the current disposition authority; `src/v3/std/bridge_ledger.dag` records owner `pipeline_authority` and authority `PR #1171`. |
| **(d) Gap analysis** | **(i) Carrier needs to land first**. The stage-order binding carrier exists, but the compile orchestrator body still lowers to `ArrowBody::Unparsed`, so there is no structural Dag fact for the body order to compare against. Full retirement waits for derivation / lowering to expose the compile body or a unified authored carrier. |

### `bridge_exact_string_patching_residual_retired`

| Field | Audit |
|---|---|
| **(a) Bridge surface** | This is an umbrella row for exact-string patch classes outside the already-retired lower-helper post-process slice. The current load-bearing class named by prior audits is `src/v3/compiler/src/bootstrap.rs::patch_kernel_bool_boolean_algebra_inhabits`: it looks for `Bool` by `d.name.as_deref() == Some("Bool") && d.span.file == "dsl/std/types.dag"` and hand-wires `Declaration.inhabits` for `BooleanAlgebra<Bool>`. Other exact-string scans remain separate bridge surfaces when they have their own receipts; for example `src/v3/compiler/src/test_runner.rs` still has `INFER_HELPERS_SOURCE.contains(&format!("type {name}"))`, but the ledger row's open rationale is specifically "other exact-string patching classes" beyond the closed lower-helper slice. |
| **(b) Substrate carrier consumer** | Existing target fact shape: `Declaration.inhabits` / `TypeConnective::Instantiation` can already represent the Bool inhabits BooleanAlgebra relationship after bootstrap. Missing authoring carrier: the source-level `type Bool inhabits BooleanAlgebra<Bool> = True | False` declaration in `dsl/std/types.dag` (or the equivalent v3-authoritative substrate home) once the v2 parser / bootstrap path can accept that syntax. For broader residual classes, each exact-string patch needs its own typed fact or declaration authority rather than an umbrella string matcher. |
| **(c) Owner-program-of-retirement** | PB Manager / PB Tier-2 owns the patch retirement path. `docs/r3-structure.md` maps patch residuals to **PB Manager** and the ledger records owner `PB-Tier-2`. The `bootstrap.rs` comment ties this class to PB-Bootstrap-Process and the `docs/design-pure-bootstrap-zero.md` lane. |
| **(d) Gap analysis** | **(i) Carrier/source authoring needs to land first** and **(iii) cross-program coordination remains unresolved** for any class whose typed fact requires v2 retirement or substrate syntax support. The Bool patch has a clear dissolution trigger in code: delete the patch when `dsl/std/types.dag` can author the inhabits declaration directly. The umbrella row should not retire from the lower-helper zero ratchet alone; it retires only after each remaining exact-string patch class has its own retirement receipt or is split into a separately tracked row. |

## Cross-Row Routing

| Row | Natural owner | Structural blocker class | Routing signal |
|---|---|---|---|
| `bridge_source_span_file_participation_retired` | Substrate | Carrier family + consumer naming | Route typed identity surface work through T-Bridge-Retirement / lens-fold file-path semantics; do not accept partial string deletion while lower/emit/lens reflection still consult paths. |
| `bridge_canonical_lens_name_patching_residual` | PB-Runtime, with possible Substrate carrier ask | Cross-Dag lens executable identity | Route through T-LensProducer-Retirement / PB-Runtime interpreter-as-data, or split a Substrate lens-registry carrier brief before implementation. |
| `bridge_include_str_side_channels_retired` | PB Manager / pipeline authority | Lowered compile-body witness | Route through PB compiler-internal bootstrap / derivation work; `PipelineStageBinding` alone is insufficient for compile-body drift detection. |
| `bridge_exact_string_patching_residual_retired` | PB Tier-2, with Substrate/v2 syntax dependency for Bool inhabits | Direct authored facts replacing patches | Route Bool inhabits authoring through PB bootstrap/v2-retirement coordination; split other exact-string patch classes when discovered rather than absorbing them into the lower-helper receipt. |

## Verification Consumption

This audit gives the decreasing-open-count ratchet a structural path to decrease:

- the current bound stays a count ratchet, not proof of row-level progress;
- each future owner PR can cite the row's retirement trigger and either flip the row or leave it open with a narrowed gap;
- Verification should reject row retirement if a PR removes only one textual surface while another production participation/name/path surface remains live.

## Per-PR Receipt

Debt found + routed: per-row routing recorded inline; net new substrate carrier asks, if any, are routed via the named owner programs. This PR does not close a Debt-Paydown row directly and does not change the `BridgeLedger` data.

## Test Plan

- `git diff --check`

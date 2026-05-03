# Bridge-Retirement Audit — SourceSpan / File-Identity Family

Status: **audit packet only** (no retirements). Anchors
`docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row 76 (`B4 bridge-retirement
queue`), family **A** — **SourceSpan / compilation-unit file string** semantics
in the v3 substrate (`src/v3/compiler/src/{dag,lower,infer,emit,lens_apply,
bootstrap,pipeline_authority,lens_testgen,test_runner}.rs`) plus **`include_str!`
cousins** that feed **`compile_to_dag`** (canonical lens bytes, lens `#[cfg(test)]`
loads) **or** otherwise stamp the same **`SourceSpan.file`** story as an
on-disk module load. **Excluded:** `INFER_HELPERS_SOURCE` — embedded text used
only for **substring presence** ratchets (`test_runner.rs::compiler_std_positive_set_ratchet_count`),
not for lowering; see §Out of family.

Verification (`fierce-ferret-556`, issue `#1276`) owns per-row retirement PRs
after this packet lands.

## Family boundary (and deliberate overlaps)

**In family:** Any **control-flow or partitioning** keyed on `span.file` /
`Behavior::* .span.file` / the synthetic `compile_to_dag(_, logical_name)`
string carried into those spans, plus **hard-coded path literals** that select
bootstrap or authority declarations when a typed `DeclarationRef` (or module
id) would be the steady-state identity.

**Out of family (listed only as siblings, not audited as rows here):**

- **Lens-name / string-dispatch arms** in `test_runner.rs` (`lens_decl.name ==
  Some("…")`, generic name-keyed lookups) — tracked under
  `bridge_canonical_lens_name_patching_residual` / `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`.
  They **overlap** canonical-lens `include_str!` entries (below): same PRs may
  touch both, but the name-keyed class is not SourceSpan-native.
- **Exact-string patching** (lower-helper patch class, SG-6, etc.) — separate B4
  queue item; not `span.file` keyed.
- **Ordinary provenance:** `parse_generated.rs` stamping `SourceSpan::new(self.file,
  …)` from the parser file handle is **not** a bridge; generated
  `bootstrap_std_generated.rs` span literals are data, not lookup keys.
- **`PROGRAM_INPUT_SENTINEL`** — **retired** structurally (`ProgramInputRole` /
  `src/v3/std/verification.dag` carriers per `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`);
  do not resurrect as an open SourceSpan row.
- **`INFER_HELPERS_SOURCE`** (`test_runner.rs`, `include_str!(…/infer_helpers.dag)`) —
  **not** in this family: the runner never runs **`compile_to_dag`** on these bytes for
  claims; it only runs **`.contains` substring** checks in
  `compiler_std_positive_set_ratchet_count`. `canonical_lens_bridge_ratchet_test.rs`
  documents that this is **outside Category A** canonical-lens `include_str!`
  surface (lines `:235-238`: not `R1_CANONICAL_*_LENS`, not lens-name dispatch).
  Map that debt under **exact-string / text-ratchet** discipline (see
  `bridge_exact_string_patching_residual_retired` umbrella in
  `docs/briefs/r3-v-bridge-row-by-row-retirement-audit.md`), not under
  `bridge_source_span_file_participation_retired` / this SourceSpan/file audit row.

**Fuzzy edge (documented, not escalated):** `include_str!` of canonical lens bytes
is both **file-path identity** (this family) and **canonical-lens residual**
(ledger row + `bridge_ledger.dag`). Retirement sequencing should explicitly pick
a lead owner when a PR touches both.

**Historical note:** B4 §0.4 (`lens_apply.rs` `algebra.dag` / `ends_with` fold-skip)
is **not present at HEAD**; fold step recovery is structural (`std_list_fold_decl`,
`find_fold_step_bind_via_instantiation`, tests `fold_step_lookup_requires_template_formal_edge`).
Do not budget a retirement row for the old path suffix.

## Ledger / bridge_ledger mapping

| `src/v3/std/bridge_ledger.dag` row | Relation to this family |
| --- | --- |
| `bridge_source_span_file_participation_retired` | **Primary** umbrella — this audit enumerates the open participation sites feeding that row’s prerequisites. |
| `bridge_include_str_side_channels_retired` | **Overlap** — `include_str!` entries that compile to a parallel `Dag` with real `SourceSpan.file` values (`test_runner.rs`, `pipeline_authority.rs` commentary, regen hosts). |
| `bridge_canonical_lens_name_patching_residual` | **Sibling** — co-retirement risk with `R1_CANONICAL_*` / `INFER_HELPERS_SOURCE` rows below. |

## Enumeration — per-entry shape

Columns: **(a)** declaration / anchor site · **(b)** consumer count (approximate
`rg` at HEAD, `src/v3/compiler/src` unless noted) · **(c)** structural retirement
shape · **(d)** sibling / blocker.

| # | Bridge entry | (a) Declaration / anchor | (b) Consumers | (c) Retirement shape | (d) Sibling / blocker |
| --- | --- | --- | --- | --- | --- |
| 1 | **`dsl_std_render_repeat_string_decl_id` authority via `span.file.ends_with`** | `lower.rs` (`dsl_std_render_repeat_string_decl_id`, `REPEAT_STRING_AUTHORITY_SUFFIXES`) | 1 (`try_lower_repeat_string_string_data` call path) | `DeclarationId` (or `DeclarationRef`) wired to the single canonical `repeat_string` data decl; delete suffix table. | Duplicate `dsl/std` vs bootstrap excerpt convergence (ROADMAP T-P0 narrative); `declaration_by_name` still finds `repeat_string` today. |
| 2 | **Kernel `Bool` bootstrap patch lookup** | `bootstrap.rs` `patch_kernel_bool_boolean_algebra_inhabits` (`BOOL_TYPES_FILE`, `span.file ==`) | 1 (+ tests in same file) | Express `Bool` `inhabits` in `dsl/std/types.dag` when v2 accepts syntax; delete patch + file gate. | v2 `dsl/` parse authority (commented dissolution at site). |
| 3 | **`Dimension` phantom / surface validation gated on authority file** | `lower.rs` `DIMENSION_STD_AUTHORITY_FILE`, `validate_dimension_phantom_surface_item`, `attach_dimension_phantom_parameter` | 3 direct file compares + unit tests in `lower.rs` | Substrate marker (`meta_tag` / nominal module stamp) on the std `Dimension` record; delete `src/v3/std/dimensions.dag` path string. | Depends on stable `Dimension` home in std (not file-string). |
| 4 | **`error_primitives` authority file gate** | `infer.rs` + `emit.rs` `ERROR_PRIMITIVES_AUTHORITY_FILE` (`decl.span.file ==` / `!=`) | 4 sites (2 infer, 1 emit gate, const) | Import-graph or `DeclarationRef` to error-primitive decl set; no bare path compare. | Bootstrap ordering / duplicate std mirrors. |
| 5 | **`dsl/std/types.dag` type-alias refinement placeholder** | `lower.rs` `lower_type_alias_refinements_phase` (`span.file == "dsl/std/types.dag"`) | 1 phase (all `where` aliases in that file) | Resolved Bool-level helpers **or** `meta_tag` for doc-only refinements **or** PB-1 authority list not keyed by string (see comment `lower.rs:829-835`). | PB-1 diagnostic-empty bootstrap gate; may interact with #2 when `types.dag` grows `inhabits`. |
| 6 | **Pipeline authority file guard** | `bootstrap.rs` (`span.file == PIPELINE_AUTHORITY_FILE`); `pipeline_authority.rs` (`decl.span.file == PIPELINE_AUTHORITY_FILE` in stage binding walk) | 2 + structural `PipelineStageBinding` consumers | Typed pipeline-stage graph only; drop file compare when compile arrow is lowered (`bridge_include_str_side_channels_retired` prerequisite). | Same structural gap as ledger `bridge_include_str_side_channels_retired` (compile body still `ArrowBody::Unparsed`). |
| 7 | **`reflect_program_dag_nodes_in_file` + `behavior_source_file`** | `lens_apply.rs` (`behavior_source_file`, `reflect_program_dag_nodes_in_file`) | 7 references across `src/v3` (runner + tests); **all** lens folds using file partition | `CompilationUnitId` / `ModuleId` (or declaration-owned “authored here” bit) on `Behavior` nodes; filter by id, not string. | Stamping must happen at lower/bind creation; touches `compile_to_dag` contract. |
| 8 | **`fold_lens_over_reflected_program`** | `lens_apply.rs` (wraps #7 + `apply_lens_declaration`) | 3 unit tests in `lens_apply.rs` + indirect runner use | Absorbs #7; same carrier. | Blocked on #7. |
| 9 | **`TestClaim` / runner `claim.file_name` as logical compilation unit** | `test_runner.rs` (`compile_to_dag(&claim.source, &claim.file_name)`, `find_bind(…, &claim.file_name)`, `decl.span.file == claim.file_name`) | **11** `compile_to_dag(&claim.source, &claim.file_name)`; additional `find_bind` / reflect call sites | `TestClaim` carries `DeclarationRef` / role-tagged program root; runner does not invent parallel filenames for identity. | **B4.1** `DeclarationRef` migration; overlaps **lens-name** family for lens selection. |
| 10 | **Deferred-claim fixture file constants** | `test_runner.rs` `RELEASE_ACCEPTANCE_FIXTURE`, `TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE` + `decl.span.file !=` / `==` checks | 2 claim kinds, ~6 guard branches | Structural “claim must be declared in fixture X” via `DeclarationRef` to fixture module, not path string. | #9. |
| 11 | **`SourceFilteringBinding::excludes` on `span.file`** | `emit.rs` (`excludes`, `normalize_source_filter_path`); emit walkers (`emit.rs`, `rust_target.rs`, `python_target.rs`) | **6** filter call sites (bind/decl walks) | Emit indexes keyed by module / declaration participation; prefix table becomes derived from structural “emit this decl” witness, not `span.file` string at walk time (may still read **data** from `.dag` lists — that part is already structural). | Target `ShapeATargetSourceFiltering` remains prefix-shaped in `.dag`; changing that is spec work. |
| 12 | **`BOOTSTRAP_FIXTURE_PATH_KEYS` lockstep slice** | `bootstrap.rs` const array + `dag.rs` `bootstrap_fixture_virtual_paths` compare | Regen host (`bootstrap_regen_fresh.rs`) + bootstrap init | Single authority: substrate list only; Rust side is generated or `include!` from `.dag` with no hand-maintained path literals. | `bridge_ledger` B4.4 narrative — partially landed via `extdeps_bootstrap_fixtures.dag` but Rust slice remains a second source. |
| 13 | **Canonical lens `include_str!` → `compile_to_dag` with real paths** | `test_runner.rs` `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS`, `R1_CANONICAL_COMPLEXITY_LENS`; `lens_apply.rs` **tests** `include_str!("../../lenses/…")`; `build.rs` splice for user-authored lens gate (same pattern) | Runner lens paths + ratchet `canonical_lens_bridge_ratchet_test.rs` **Category A** (`R1_CANONICAL_*_LENS` only — `INFER_HELPERS_SOURCE` **explicitly excluded** there) | Lens bytes resolved through `program_dag` / typed lens registry; delete parallel `Dag` keyed by duplicated path strings. | **Strong overlap** with `bridge_canonical_lens_name_patching_residual` and PB-Runtime interpreter-as-data gate. **Do not** conflate with `INFER_HELPERS_SOURCE` (out of family — §Out of family). |
| 14 | **`declaration_name_preference_rank` + `Dag::declaration_by_name`** | `dag.rs` (`declaration_name_preference_rank`, `declaration_by_name`) | **`declaration_by_name(` ~188** matches under `src/v3/compiler/src` | Delete rank; require single-authority modules; fail-closed on duplicate top-level names after `dsl/std` ↔ `src/v3/std` convergence (ROADMAP checklist). | **Root blocker** for most name-keyed substrate; duplicates must converge first. |
| 15 | **`collect_symbols` duplicate-authority mirror** | `lower.rs` `collect_symbols` (same rank helper as `dag.rs`) | Every `lower_into` module | Delete together with #14; one policy. | #14. |
| 16 | **`bootstrap_regen_fresh.rs` duplicate `declaration_name_preference_rank`** | `bootstrap_regen_fresh.rs` | Fresh regen symbol merge | Delete when #14/#15 delete; keep one implementation. | #14. |
| 17 | **`lens_testgen` std duplicate pick (`std_preference_rank`, `is_bootstrapped_std_file`, `substrate.dag` skip)** | `lens_testgen.rs` (~520–535, ~777) | Lens testgen output only | Consume unified module identity / `DeclarationRef` picks; delete second rank table. | #14/#15 (same duplicate-authority story). |
| 18 | **`lens_testgen` `verification.dag` special-case** | `lens_testgen.rs` (`decl.span.file == "src/v3/std/verification.dag"`) | 1 predicate arm | Typed “skip harness-only decl” flag or `DeclarationRef` set membership. | #17. |
| 19 | **Diagnostics correction file consistency** | `diagnostics.rs` (`correction.span.file != file`) | Low (fix struct consumers) | Corrections carry authoritative `SourceSpan`; drop cross-file string compare if spans are always normalized to compilation unit. | Minor; independent. |

**Not counted as separate rows:** integration tests under
`src/v3/compiler/tests/**` that **assert** `span.file == "<fixture>"` (they are
**witnesses** of the bridge, not additional declaration sites). SG-0 / census
tests that **forbid** new `span.file` bridges are enforcement, not bridges.

## Consumer count methodology

Approximate HEAD counts (2026-05-03 worktree):

- `declaration_by_name\(` in `src/v3/compiler/src/**/*.rs` → **188** matches.
- `compile_to_dag\(&claim\.source, &claim\.file_name\)` in `test_runner.rs` → **11**.
- `reflect_program_dag_nodes_in_file` in `src/v3/**/*.rs` → **7**.

## Leaf-first retirement order (dependency-respecting)

Retire **narrow / leaf** bridges first; **root rank + `declaration_by_name`**
last. Numbers refer to the table above.

1. **#19** — diagnostics file compare (no substrate dependents).
2. **#1** — `repeat_string` suffix authority (single lowering niche).
3. **#4** — `error_primitives` file gate (localized infer/emit).
4. **#3** — `Dimension` authority file gate.
5. **#2** — kernel `Bool` patch file gate (after v2 syntax prerequisite or parallel substrate plan).
6. **#5** — `types.dag` refinement placeholder (may interact with #2; land ordering inside same PB-1 wave if both touch `types.dag` bootstrap).
7. **#6** — pipeline `PIPELINE_AUTHORITY_FILE` guards (tied to structural compile-body / stage witness).
8. **#18** then **#17** — `lens_testgen` special cases (test-only blast radius).
9. **#12** — bootstrap fixture path key slice vs substrate virtual paths (shrinks regen drift).
10. **#11** — emit `SourceFiltering` application to `span.file` (requires emit-index redesign; can parallel with #7–#10 only where safe).
11. **#7** + **#8** — reflection / fold-over-reflected-program file partition (lens API).
12. **#9** + **#10** — runner `claim.file_name` + deferred fixture constants.
13. **#13** — canonical `include_str!` side channels (coordinate with lens-name residual PRs).
14. **#16** — delete duplicate rank in `bootstrap_regen_fresh.rs` when landing #14/#15.
15. **#15** + **#14** — `collect_symbols` + `declaration_by_name` preference scaffold (requires ROADMAP duplicate-module convergence).

## STOP / PING criteria (audit outcome)

**Not triggered:** Family boundary fuzziness is confined to documented overlaps
with **canonical-lens / `include_str` ledger rows** and **lens-name** residual;
no substrate discovery blocked completing this enumeration.

**Substrate work already known (not a STOP):** rows **#14–#16** require the
published duplicate-module convergence program before the rank scaffold can
delete; this is expected debt, not a new audit failure.

## Receipt — ledger row 76

**Disposition:** Audit packet landed as this file — **family A (SourceSpan /
file identity + `include_str!` cousins on the compile path)** for
`B4 bridge-retirement queue` (`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:76`).
Per-row retirement PRs remain with Verification / bridge owners per queue policy.

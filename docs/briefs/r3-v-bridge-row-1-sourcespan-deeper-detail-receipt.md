# R3 Verification - Bridge Row 1 SourceSpan Deeper-Detail Receipt

**Status:** AUDIT RECEIPT - docs-only. This receipt does not flip any
`BridgeLedgerRow.status` and does not close a Debt-Paydown row directly.

**Row:** `bridge_source_span_file_participation_retired`.

**Primary input:** `docs/briefs/bridge-retirement-audit-sourcespan-family.md`
(`royal-newt-846` family A packet). This receipt consumes that packet's
19-entry enumeration and leaf-first order; it does not re-author the audit
packet. Entry numbers below refer to the packet's enumeration table (§
`Enumeration - per-entry shape`) and retirement order (§
`Leaf-first retirement order`).

**Boundary input:** `docs/briefs/bridge-retirement-audit-include-str-family.md`
(`crisp-newt-163` family B packet), used only for overlap coordination with
`include_str!` / canonical-lens / source-text patching rows.

**Prior row audit:** `docs/briefs/r3-v-bridge-row-by-row-retirement-audit.md`
classified row 1 as a carrier-family and consumer-naming gap. This receipt
turns that row-level classification into a per-entry roadmap.

## Verification Consumption Rule

Verification should reject a PR that claims to retire one SourceSpan/file
entry unless it demonstrates that **all production participation surfaces named
for that entry in the family A packet are gone or converted to typed carrier
consumption**. Removing only the most visible `span.file` comparison is not
enough when sibling call paths still use the same file identity for lowering,
reflection, emit inclusion, runner lookup, duplicate-authority preference, or
fixture/module selection.

At review time, each retirement PR must cite the entry number and show:

1. every packet-listed participation surface for that entry was removed or made
   structural;
2. the replacement carrier consumer is present in production code, not only in a
   test fixture;
3. overlapping family B / canonical-lens work is either untouched or explicitly
   scoped so the later row 2/3/4 receipts do not count the same deletion twice;
4. if the carrier does not exist, the PR leaves the entry open and routes the
   ask through Substrate via T-Bridge-Retirement instead of claiming retirement.

## Cross-Packet Boundary Discipline

Family A owns retirement of decisions keyed on `SourceSpan.file`,
`Behavior.*.span.file`, synthetic `compile_to_dag(_, logical_name)` file
identity, and hard-coded path literals that stand in for declaration or module
identity. Family B owns `include_str!` / source-text patching side channels.

The family B overlap table and dispatch callouts name BR-07, BR-14, BR-15,
BR-16, BR-18, BR-19, and the BR-A appendix. Mapping back to the family A
packet:

| Family A entry | Family B overlap | Boundary rule |
| --- | --- | --- |
| #2 kernel `Bool` bootstrap patch lookup | BR-19 | Row 1 counts only the `span.file == "dsl/std/types.dag"` authority gate. The exact source-text / post-parse patching receipt owns the patching row closure. |
| #7 reflection by source file and #8 fold-over-reflected-program | BR-07 / BR-A | Row 1 counts file-partitioned reflection and fold participation. Canonical lens byte inclusion, lens-name dispatch, and fixture source channels remain for row 2/3/4 receipts. |
| #9 runner `claim.file_name` and #10 deferred fixture file constants | BR-A | Row 1 counts logical file-name identity introduced by runner compilation and lookup. Hermetic fixture byte inclusion is not counted as row 1 unless it feeds production participation by file identity. |
| #11 emit `SourceFilteringBinding::excludes` on `span.file` | BR-14 / BR-16 | Row 1 counts production emit inclusion/exclusion by declaration or binding file. Family B counts emitter/spec source embeds and whole-file text assertions. |
| #12 bootstrap fixture path-key slice | BR-15 | Row 1 counts the lockstep path-key authority. Family B counts the integration-test `include_str!` bootstrap corpus and generated/static-table source channels. |
| #13 canonical lens `include_str!` into `compile_to_dag` | BR-07 / BR-08 / BR-17 | Row 1 counts the real-path file identity stamped into the extra Dag and consumed by reflection/fold paths. Family B and the canonical-lens row count byte inclusion and name-dispatch identity. |
| No direct family A retirement entry | BR-18 | Raw `lower.rs` / `builder.rs` source-text proof is family B text-mining debt. It becomes row 1 only if a PR also changes a production `SourceSpan.file` participation surface. |

Entries #14, #15, and #16 are also adjacent to family B's generated/static
table work, but the family B STOP table does not list them as explicit overlap
callouts. Treat them as duplicate-authority/name-preference SourceSpan work for
row 1 unless a PR also edits generated include/static table authority.

## Per-Entry Receipts

Rows follow the family A packet's leaf-first retirement order.

| Order | Entry | Verification-side reviewer rule | Carrier consumer required | Cross-row impact |
| --- | --- | --- | --- | --- |
| 1 | #19 diagnostics correction file consistency | A retirement PR must remove correction validity that compares `correction.span.file` to a separate file string, and must prove diagnostics still carry a normalized authoritative `SourceSpan`. | Existing `SourceSpan` / diagnostic correction carrier if normalized at construction; otherwise needs a new correction-authority invariant routed to Substrate via T-Bridge-Retirement. | None called out by family B. |
| 2 | #1 `repeat_string` suffix authority | Delete `REPEAT_STRING_AUTHORITY_SUFFIXES` and the `span.file.ends_with` authority path; the lowering niche must not retain any suffix fallback for the canonical data declaration. | `DeclarationId` or `DeclarationRef` for the canonical `repeat_string` data declaration, consumed by `try_lower_repeat_string_string_data`. | None called out by family B. |
| 3 | #4 `error_primitives` authority file gate | Remove every infer/emit branch that admits or rejects error primitives by `ERROR_PRIMITIVES_AUTHORITY_FILE`; no production gate may continue reading `decl.span.file` for this authority. | Import-graph membership or a `DeclarationRef` set for error primitive declarations; if absent, route a typed error-primitive authority carrier to Substrate. | None called out by family B. |
| 4 | #3 `Dimension` authority file gate | Remove `DIMENSION_STD_AUTHORITY_FILE` checks from phantom/surface validation and attachment; unit tests must not assert the path string as authority. | A nominal module stamp, `meta_tag`, or equivalent typed std `Dimension` authority marker consumed by lower. If the stable carrier is undecided, keep the entry open and route to Substrate. | None called out by family B. |
| 5 | #2 kernel `Bool` bootstrap patch lookup | Retirement must delete the `BOOL_TYPES_FILE` file gate and all tests that prove the patch by file identity; `Bool` authority cannot remain name-plus-file. | Direct authored `Bool inhabits BooleanAlgebra<Bool>` fact in `dsl/std/types.dag` once the parser/bootstrap path accepts it, consumed through `Declaration.inhabits`. Until then, route via PB/Substrate T-Bridge-Retirement. | Overlaps family B BR-19 and the exact-string patching row. Row 1 claims only the file-authority deletion; patch-row closure is separate. |
| 6 | #5 `dsl/std/types.dag` type-alias refinement placeholder | Remove the special `span.file == "dsl/std/types.dag"` phase gate for alias refinements; all aliases in that file must not receive privileged treatment by path. | Resolved Bool-level helper facts, doc-only refinement `meta_tag`, or a PB-1 authority list keyed structurally rather than by path. If the carrier choice is still open, route to Substrate/PB rather than retiring. | May land near #2, but family B has no separate overlap callout beyond the Bool patch. |
| 7 | #6 pipeline authority file guard | Remove both bootstrap and `pipeline_authority.rs` comparisons against `PIPELINE_AUTHORITY_FILE`; stage binding walks must not locate authority by file string. | Typed `PipelineStageBinding` graph plus a structural compile-body/stage witness when available. Existing stage order is partial; compile-body witness remains the blocker. | Adjacent to `bridge_include_str_side_channels_retired`, but family A packet states `pipeline_authority.rs` has no active include side channel. Do not double-count with the row 3 receipt. |
| 8a | #18 `lens_testgen` `verification.dag` special-case | Delete the `decl.span.file == "src/v3/std/verification.dag"` skip arm; generated test selection must not special-case harness-only declarations by file. | Typed harness-only declaration flag or `DeclarationRef` set membership consumed by `lens_testgen`. | No explicit family B overlap. |
| 8b | #17 `lens_testgen` std duplicate pick | Delete `std_preference_rank`, `is_bootstrapped_std_file`, and `substrate.dag` skip behavior as participation authorities; testgen must consume unified identity choices. | Unified module identity or `DeclarationRef` selection surface shared with duplicate-authority retirement. | No explicit family B overlap, though generated/static table PRs may be adjacent. |
| 9 | #12 `BOOTSTRAP_FIXTURE_PATH_KEYS` lockstep slice | Remove the hand-maintained Rust path-key slice as an authority. A retirement PR must show bootstrap fixture virtual paths come from one substrate/generator authority, not a second Rust list. | Substrate fixture list or generated Rust include from `.dag` as single authority, consumed by bootstrap and regen. | Adjacent to family B BR-04/BR-05 static table work; row 1 counts path-key identity, while row 3 counts embedded source side channels. |
| 10 | #11 `SourceFilteringBinding::excludes` on `span.file` | All emit walkers must stop applying structured filtering policy to `decl.span.file` / `bind.span.file`; no target inclusion decision may remain path-prefix based at walk time. | Emit indexes keyed by module/declaration participation or a structural "emit this declaration" witness. Existing `SourceFilteringBinding` is not enough while it consumes file strings. | Overlaps family B BR-16 if a PR also touches Shape-A spec `include_str!` filtering authority. Count only the production emit file-participation move here. |
| 11a | #7 `reflect_program_dag_nodes_in_file` + `behavior_source_file` | Delete file-partitioned reflection over `Behavior.*.span.file`; all lens folds that select program nodes by source file must use typed compilation-unit/module identity. | `CompilationUnitId`, `ModuleId`, or declaration-owned "authored here" identity stamped during lower/bind creation and consumed by reflection. | Overlaps BR-07/BR-08/BR-A when canonical lenses or runner fixtures feed folded Dags. Row 1 owns the reflection partition; row 2/3/4 own byte/name channels. |
| 11b | #8 `fold_lens_over_reflected_program` | The wrapper cannot be considered retired while it delegates to file-partitioned reflection; unit and runner use must call the structural reflection carrier. | Same carrier as #7, consumed through the lens application API. | Same as #7. |
| 12a | #9 `TestClaim` / runner `claim.file_name` logical compilation unit | Remove runner identity that invents file names via `compile_to_dag(&claim.source, &claim.file_name)`, `find_bind(..., &claim.file_name)`, or `decl.span.file == claim.file_name`; claim execution must identify program/lens roots structurally. | `TestClaim` `DeclarationRef`, role-tagged program root, or equivalent typed root identity consumed by runner lookup/reflection. | Overlaps family B BR-A and canonical-lens entries when fixture bytes are compiled into extra Dags. Do not count fixture-byte deletion here unless file identity consumption is also removed. |
| 12b | #10 deferred-claim fixture file constants | Remove `RELEASE_ACCEPTANCE_FIXTURE`, `TC1_SUBSTRATE_LENS_ETA_DEFERRED_FIXTURE`, and related file equality/inequality guards as claim authority. | Structural "claim declared in fixture/module" identity, preferably via `DeclarationRef` to the fixture module or claim root. | Same runner/fixture boundary as #9. |
| 13 | #13 canonical lens `include_str!` into `compile_to_dag` with real paths | A row-1 retirement claim must show canonical lens extra-Dag file identity no longer feeds reflection/fold participation. Merely deleting one `include_str!` const is insufficient while lens-name dispatch or path-stamped Dags remain. | Typed lens registry, `program_dag` lens body identity, or PB-Runtime interpreter-as-data consumed by the runner/lens application path. If the carrier shape is chosen by PB, route there and leave row 1 open until production consumption changes. | Strong overlap with family B BR-07/BR-08/BR-17 and `bridge_canonical_lens_name_patching_residual`. Row 1 counts file-stamped participation only; byte inclusion and name dispatch close elsewhere. |
| 14 | #16 duplicate rank in `bootstrap_regen_fresh.rs` | Delete the duplicate `declaration_name_preference_rank` implementation in regen only when the shared duplicate-authority policy has been structurally replaced; no regen-only rank table may remain. | Unified declaration/module identity policy consumed by fresh regen symbol merge. | Adjacent to family B BR-04/BR-05 if generated static tables move in the same PR; avoid counting static table deletion as row 1. |
| 15a | #15 `collect_symbols` duplicate-authority mirror | Remove lower-time duplicate preference from `collect_symbols`; symbol collection must fail closed or consume typed authority rather than ranking by `span.file`. | Same unified declaration/module identity carrier as #14, consumed by lower. | Adjacent to generated/static table work but not an explicit family B STOP overlap. |
| 15b | #14 `declaration_name_preference_rank` + `Dag::declaration_by_name` | Delete the root preference rank and broad name-keyed lookup policy only after duplicate top-level names converge or have typed disambiguation. Retirement must prove no production code still relies on preferred file path to resolve a name. | Single-authority modules, `DeclarationRef`-first lookup, or typed duplicate-resolution carrier. This is a root Substrate/PB convergence ask via T-Bridge-Retirement. | Root blocker for many name-keyed bridges; keep separate from row 2 canonical lens-name dispatch unless a PR touches both name classes explicitly. |

## Substrate Routing Notes

Known carrier asks that remain routed through Substrate / T-Bridge-Retirement:

- typed compilation-unit or module identity for behavior/reflection and runner
  participation (#7, #8, #9, #10, #13);
- typed std authority markers for `Dimension`, error primitives, alias
  refinements, and similar lower-time decisions (#3, #4, #5);
- emit participation indexes that do not consume raw file paths (#11);
- unified declaration/module identity for duplicate authority and name
  preference retirement (#14, #15, #16, #17);
- fixture/path-key single authority for bootstrap and regen (#12).

Known PB/Substrate coordination remains for #2 and #5 because source-level
authoring / v2 syntax support determines when the replacement facts can be
authored directly.

## Per-PR Receipt

Debt found + routed: per-entry retirement roadmap recorded inline; net new
substrate carrier asks, if any, routed to Substrate via T-Bridge-Retirement
lane.

This receipt does not close a Debt-Paydown row directly. Closure happens only
when Substrate/PB ships the actual retirement PRs and `bridge_ledger.dag` flips
the relevant row from `Open` to `Retired`.

## Test Plan

- `git diff --check`

# XL3 wave 2 — the importers of `std.declaring_identity_spelling`: reference list, and what the population really is

Read-only preparation. No run was executed for this document or for the claim source on this branch.
Shape and rules come from `docs/plans/2026-11-01-xl3-identity-witness-shape.md` (wave 1, same branch);
the witness itself is `src/v2/test/claim/long/xl3_wave2_declaring_identity_witness.dag` and it IMPORTS
wave 1's `ExpectedBinding`, `read_of`, `declaring_paths`, `table_holds` rather than restating them.

## What was asked, and where the numbers come from

Wave 2 was scoped as "its 15 importers", the count zesty-badger-599's note carries from
`git grep -lw declaring_identity_spelling` (excluding the declaring module). That count is reproduced
exactly, and then read module by module. Everything below is one command at one revision:

    git grep -lw declaring_identity_spelling 345d6077fc4a -- | grep -v '^dag/std/declaring_identity_spelling.dag$'
    -> 15 files (population)

Per-file reference sites, same revision, excluding `import` rows (an import row is a different cut),
comment lines, and the `module` line — the script and its rule are in this branch's history; the
result, with line numbers, is the table below. `git grep -c` was not used because it counts the
import block and comments, which inflates every row.

## Population: 15 files, of which 3 carry a reference site

| file | reference sites to a declaring identity of `std.declaring_identity_spelling` |
| --- | --- |
| `src/v2/compiler/declaring_identity_spelling_census.dag` | **15 sites, 6 names** — `DeclaringIdentitySpelling` 64,68 · `IdentityAmbiguous` 69,116 · `IdentityOwnerless` 70,117 · `IdentityUnbound` 71,118 · `declaring_identity_spelling_resolved` 96 · `declaring_identity_spelling_unresolvable` 97,104,112,135,150,160 |
| `src/v2/test/claim/declaring_identity_spelling/verdict_fold.dag` | **26 sites, 12 names** — `DeclaringIdentitySpelling` 17 · `resolved_path_verdict` 20,26 · `ResolvedSpelling` 21,34,43 · `SpelledShorter` 22,31,35,44 · `ResolvedAuthoredNotOnCanonicalPath` 23,32,37,46 · `UnresolvableSpelling` 24,38,48 · `SpelledCanonically` 27,36,40,45 · `declaring_identity_spelling_resolved` 29 · `declaring_identity_spelling_unresolvable` 33 · `IdentityAmbiguous` 47 · `IdentityUnbound` 52,61 · `IdentityOwnerless` 56 |
| `src/v2/test/claim/declaring_identity_spelling/production_ingest.dag` | **7 sites, 5 names** — `ResolvedSpelling` 46,58 · `UnresolvableSpelling` 51,63 · `IdentityUnbound` 94 · `IdentityAmbiguous` 105 · `IdentityOwnerless` 116 |
| `dag/gunbc/compiler_frontend_program_status.dag` | **0** — the only match is line 1272, where `IdentityUnbound` sits INSIDE the `why:` string of a `NativeWallBlocked` row. A name inside a message is not a reference; XL3 must not touch it. |
| `dag/gunbc/source_layout_namespaces.dag` | **0** — lines 470 and 557 are namespace-registration strings (`"v2.test.claim.declaring_identity_spelling.production_ingest"`, `"...claim_fixture"`): a path, not a name. |
| `src/v2/test/fixture/declaring_identity_spelling/{call_only,cross_consumer,cross_provider,field_vs_parameter,matched_type_vs_value,shorter,value_outside_call}.dag` | **0** — matched because the DIRECTORY is named for the module. Their content is fixture source for the census cut; no mention of any of the fifteen export names. |
| `dag/test/probe/declaring_identity_spelling_unresolvable_probe.dag` | **0** — same: the token is in its own module name. |
| `src/v2/workflow/floor_pure_producer_share.dag` | **0** — same. |
| `artifacts/rung-drops/required_lanes_do_not_resolve_product_layer_modules_2026-09-20.txt` | **not a module** — an artifact record naming the module in prose. Listed as population, not respellable. |

So the respellable population is 3 modules; the other 12 are name-token matches. The claim module
carries this as an arm (`wave2_matched_by_name_only_carries_no_declaring_identity_holds`) instead of
leaving it as prose, and the design note's "15 importers" stands as the POPULATION, not as the cut list.

## Witness table, keyed by declaring path

All rows are declaring path `std.declaring_identity_spelling.<leaf>`; counts are the cited mention
counts. Full rows in the claim source, one table per touched module:

| declaring path | census | production_ingest | verdict_fold | wave total |
| --- | --- | --- | --- | --- |
| `.DeclaringIdentitySpelling` | 2 | — | 1 | 3 |
| `.IdentityAmbiguous` | 2 | 1 | 1 | 4 |
| `.IdentityOwnerless` | 2 | 1 | 1 | 4 |
| `.IdentityUnbound` | 2 | 1 | 2 | 5 |
| `.ResolvedSpelling` | — | 2 | 3 | 5 |
| `.UnresolvableSpelling` | — | 2 | 3 | 5 |
| `.SpelledCanonically` | — | — | 4 | 4 |
| `.SpelledShorter` | — | — | 4 | 4 |
| `.ResolvedAuthoredNotOnCanonicalPath` | — | — | 4 | 4 |
| `.resolved_path_verdict` | — | — | 2 | 2 |
| `.declaring_identity_spelling_resolved` | 1 | — | 1 | 2 |
| `.declaring_identity_spelling_unresolvable` | 6 | — | 1 | 7 |
| `.authored_is_suffix_of_canonical` | 0 | 0 | 0 | **0 — see the dead export below** |
| `.declaring_identity_spelling_declared_binding_frontier_dissolve_on` | 0 | 0 | 0 | 0 (dissolution row, not a reference) |

**Two findings worth the wave owner's attention.**
(a) `authored_is_suffix_of_canonical` has **no site in the three wave-2 modules**; its only mention in
the tree is inside its own declaring module —
`git grep -nw authored_is_suffix_of_canonical 345d6077fc4a --` returns exactly two lines, the
declaration (`dag/std/declaring_identity_spelling.dag:69`) and a **same-module call at line 80**. That
line falls in wave 1's subject, and wave 1's eight-row table carries no
`std.declaring_identity_spelling.authored_is_suffix_of_canonical` row. Either a same-module reference
mints no occurrence record (plausible, and the run would show it), or wave 1's table is missing a row.
Flagged to the wave owner rather than patched, since the table is theirs.
(b) The wave-2 tables therefore hold 12 of the module's 15 exports; the other three (that fn, the
dissolution `data`, and — in the census/production_ingest modules — the verdict/type names they do not
name) simply have no site to respell.

## The ingest the witness builds

29 modules read from disk at 1696954b82e (`read_of` = `filesystem_read` through
`extdeps.communication.medium`, as in wave 1 — no fixture source): the declaring module, the three
touched importers, and the union of their import rows —
`extdeps.communication.medium`, `extdeps.filesystem.filesystem_io`, `std.algebra`, `std.decl_ref`,
`std.live_tree`, `std.optional`, `std.repair_input_origin`, `std.types`,
`v2.compiler.{normalized_tree, program_assembly, reference_site_collector, repair_input_origin_roster,
resolution_provenance, source_authority, symbol_index_fill}`, `v2.extdeps.languages.dag`,
`v2.std.{artifact, cross_tree.import_model, diagnostic, language_model, namespace_alias, node,
qualified_name, symbol_index}`, `v2.test.claim.provenance.cross_file_provenance`. Every path was
checked to exist at that revision (`git cat-file -e`), which is how the two `extdeps.*` modules were
found at `dag/extdeps/...` rather than `src/extdeps/...`.

## Unknowns — what a first run owes, stated rather than resolved

1. **Counts vs records.** The counts are source mentions; the resolver mints one occurrence record per
   reference it reads. An unread arm of a record, or a mention inside a construct the resolver treats
   as one record, changes a number. Same posture as wave 1: first run at the base corrects the table.
2. **Type cases.** Eight of the twelve rows name type CASES (`IdentityUnbound`, `SpelledShorter`, …).
   Whether a case binds to its own declaring path (`.IdentityUnbound`) or to its enclosing type
   (`.UnresolvableCause`) is not settled by reading; the run answers it. If cases fold into their type,
   three rows merge into two and the counts move — a table correction, not a cut change.
3. **Transitive providers.** Only each module's own import rows are ingested (wave 1's choice). If the
   resolver refuses a subject for a provider two hops out, the fix is more `read_of` rows, not a
   narrower claim.
4. **Scope of the table.** These rows key ONLY the wave-defining module. XL3 will also respell the
   other unqualified references in the three modules (their `std.algebra` heads like `fold_list` and
   `Empty`, `v2.std.diagnostic` constructors, `v2.std.node` heads). A wave-2 witness that pinned the
   whole module would carry those too; deciding whether wave 2 is "the identity of the module it is
   named for" or "every reference in the touched module" is the wave owner's call, and the shape
   supports either without a fork.
5. **`authored_is_suffix_of_canonical` has no site** (above): unknown whether a route outside the grep
   reaches it.
6. **The probe/fixture/namespace matches**: pinned as non-reference by reading. If XL3's cut also
   respells name-matches inside strings, `dag/gunbc/compiler_frontend_program_status.dag:1272` and the
   two `source_layout_namespaces.dag` rows become in-scope and this document is wrong about them — a
   decision to make before the cut, not after.

## What was NOT done

No run of any kind (the claim is written to be run at base and at head, and the native-route command is
recorded in the branch's receipts elsewhere). No edit to any touched module. No claim that the
population is complete: it is `git grep -lw` at one revision, read file by file.

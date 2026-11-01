# XL-3 wave 2 -- every reference in the modules the wave touches

Status: **written, never run.** Nothing in this note or in
`src/v2/test/claim/long/xl3_wave2_declaring_identity_witness.dag` has been executed: no claim run, no
resolution, no emitted binary. The tables are pinned from READING (cited commands below), and the first
run may correct a count -- see "What a run owes".

Wave 1 (`docs/plans/2026-11-01-xl3-identity-witness-shape.md`,
`src/v2/test/claim/long/xl3_wave_declaring_identity_witness.dag`, zesty-badger-599, on
`plan/xl3-declaring-identity-witness`) witnesses ONE module's identities: the declaring module
`std.declaring_identity_spelling` itself. Wave 2 is its importers. This note is the read-only prep: the
reference list per module, the witness table keyed by declaring path, and the claim source plus its
wrong-declaration arm, unrun.

## Base and branch

* Prepared on `plan/sunny-bee-153-xl3-wave2-declaring-identity-witness`, based on
  `plan/xl3-declaring-identity-witness` at **1696954b82e** (the branch tip, not the 39f08591dde1 commit
  the design note was authored at -- ruling by tidy-ant-251: basing on the tip avoids forking `read_of`
  and the ingest shape). `plan/xl3-declaring-identity-witness` moved after that: 93cd69197b00
  ("read through the Filesystem.Read effect") and a559332688d (P3 respell-row-kind design note) are
  merged into this branch; 93cd69197b00 also renamed wave 1 --
  `xl3_declaring_identity_spelling_witness.dag` -> `xl3_wave_declaring_identity_witness.dag` and
  `2026-10-31-xl3-declaring-identity-spelling-witness.md` -> `2026-11-01-xl3-identity-witness-shape.md`.
  This file and the witness use the CURRENT names; wave 1 stays the authority for the shape and is
  IMPORTED, not forked.
* Everything is counted at one revision: **345d6077fc4a** (the merge-base of this branch with main at
  the time of writing), so every number below can be re-derived by the cited command.

## Population (how it was found, and what it is not)

Command:

    git grep -lw declaring_identity_spelling 345d6077fc4a --

Population: **15 files** at 345d6077fc4a (whole tree, `-w` on the module word). Split:

| kind | count | files |
|---|---|---|
| the declaring module itself | 1 | `dag/std/declaring_identity_spelling.dag` |
| carries `import std.declaring_identity_spelling` (the wave's sites) | 3 | `src/v2/compiler/declaring_identity_spelling_census.dag`, `src/v2/test/claim/declaring_identity_spelling/production_ingest.dag`, `src/v2/test/claim/declaring_identity_spelling/verdict_fold.dag` |
| the word appears, NO import row | 11 | `artifacts/rung-drops/required_lanes_do_not_resolve_product_layer_modules_2026-09-20.txt`, `dag/gunbc/compiler_frontend_program_status.dag`, `dag/gunbc/source_layout_namespaces.dag`, `dag/test/probe/declaring_identity_spelling_unresolvable_probe.dag`, the 7 files under `src/v2/test/fixture/declaring_identity_spelling/`, `src/v2/workflow/floor_pure_producer_share.dag` |

Verified by `for f in <those 15>; do git show 345d6077fc4a:$f | grep -c '^import std.declaring_identity_spelling'; done`:
exactly the 3 files above return 1; every other file returns 0. The 11 name-only files mention the words
in string literals, path/roster entries, comments or file names (e.g.
`dag/gunbc/source_layout_namespaces.dag:470,557` are namespace strings;
`dag/gunbc/compiler_frontend_program_status.dag:1272` is a message string), so an import-row edit has
no edge to move there. That property is asserted, not assumed -- the witness's last test reads those
files and refuses if any contains an import row for the wave module.

## Scope ruling, restated

tidy-ant-251: wave 2 is **every reference occurrence in the touched modules**, not only the references
to `std.declaring_identity_spelling`. Reason: an import edit can change what an UNCHANGED spelling
resolves to, so a witness that pinned only the wave-defining module's identities could stay GREEN while
the wave moved something else. The tables below therefore pin, for each of the 3 site-bearing modules,
every reference whose home is given by an import row.

## Witness tables, keyed by declaring path

Count convention (same as wave 1's): a name's occurrence count is its non-comment, non-import-row
mentions in the module body, computed by the script recorded in this note
(`/tmp/w2gen2.py` on the author's machine -- reproduced by the commands in "Reproducing the counts").
Positions that are NOT a reference to a declaring identity are excluded and listed separately below:
record field labels (`site:`), `^symbol` atoms, and positions bound by a `let`/parameter binder.
Counts are UNVERIFIED against the resolver: `tree.occurrence_resolutions` mints one row per occurrence
the resolver records, and the reading does not know which mentions the resolver mints.

### 1. `v2.compiler.declaring_identity_spelling_census` (`src/v2/compiler/declaring_identity_spelling_census.dag`)

| declaring path | occurrences |
|---|---|
| `std.algebra.Empty` | 1 |
| `std.algebra.FreeMonoid` | 3 |
| `std.algebra.fold_list` | 1 |
| `std.algebra.list_snoc_item` | 1 |
| `std.declaring_identity_spelling.DeclaringIdentitySpelling` | 2 |
| `std.declaring_identity_spelling.IdentityAmbiguous` | 2 |
| `std.declaring_identity_spelling.IdentityOwnerless` | 2 |
| `std.declaring_identity_spelling.IdentityUnbound` | 2 |
| `std.declaring_identity_spelling.declaring_identity_spelling_resolved` | 1 |
| `std.declaring_identity_spelling.declaring_identity_spelling_unresolvable` | 6 |
| `std.repair_input_origin.RepairInputDeclared` | 1 |
| `std.repair_input_origin.RepairInputIdentityAbsent` | 1 |
| `std.repair_input_origin.RepairInputIdentityAmbiguous` | 1 |
| `std.repair_input_origin.RepairInputTargetWithoutDeclarationIdentity` | 1 |
| `v2.compiler.normalized_tree.normalized_tree_roots_to_binding_sources` | 1 |
| `v2.compiler.program_assembly.module_roots_from_source_root_ingest` | 1 |
| `v2.compiler.reference_site_collector.UnreadableModuleRoot` | 1 |
| `v2.compiler.reference_site_collector.collect_reference_sites` | 1 |
| `v2.compiler.repair_input_origin_roster.carrier_provenance` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceAmbiguous` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceOwnerless` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceResolved` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceUnbound` | 1 |
| `v2.compiler.resolution_provenance.ResolutionProvenance` | 1 |
| `v2.compiler.resolution_provenance.answer_site` | 1 |
| `v2.compiler.resolution_provenance.module_roster_from_roots` | 1 |
| `v2.compiler.resolution_provenance.resolve_reference_sites` | 1 |
| `v2.compiler.source_authority.SourceRootIngest` | 1 |
| `v2.compiler.symbol_index_fill.symbol_index_fill_module_roots` | 1 |
| `v2.std.diagnostic.Outcome` | 1 |
| `v2.std.diagnostic.bind_outcome` | 1 |
| `v2.std.diagnostic.outcome_accepted` | 1 |
| `v2.std.language_model.LanguageModel` | 1 |
| `v2.std.namespace_alias.ModuleBindingSource` | 1 |
| `v2.std.namespace_alias.module_binding_source_roots` | 1 |
| `v2.std.symbol_index.empty_symbol_index` | 1 |

Providers read into its ingest (the modules its import rows name, one level deep): `dag/std/algebra.dag`, `dag/std/declaring_identity_spelling.dag`, `dag/std/repair_input_origin.dag`, `src/v2/compiler/normalized_tree.dag`, `src/v2/compiler/program_assembly.dag`, `src/v2/compiler/reference_site_collector.dag`, `src/v2/compiler/repair_input_origin_roster.dag`, `src/v2/compiler/resolution_provenance.dag`, `src/v2/compiler/source_authority.dag`, `src/v2/compiler/symbol_index_fill.dag`, `src/v2/std/diagnostic.dag`, `src/v2/std/language_model.dag`, `src/v2/std/namespace_alias.dag`, `src/v2/std/node.dag`, `src/v2/std/symbol_index.dag`

### 2. `v2.test.claim.declaring_identity_spelling.production_ingest` (`src/v2/test/claim/declaring_identity_spelling/production_ingest.dag`)

| declaring path | occurrences |
|---|---|
| `extdeps.communication.medium.Lossless` | 1 |
| `extdeps.communication.medium.Medium` | 1 |
| `std.algebra.Cons` | 12 |
| `std.algebra.Empty` | 4 |
| `std.algebra.FreeMonoid` | 3 |
| `std.algebra.fold_list` | 5 |
| `std.algebra.is_empty` | 1 |
| `std.declaring_identity_spelling.IdentityAmbiguous` | 1 |
| `std.declaring_identity_spelling.IdentityOwnerless` | 1 |
| `std.declaring_identity_spelling.IdentityUnbound` | 1 |
| `std.declaring_identity_spelling.ResolvedSpelling` | 2 |
| `std.declaring_identity_spelling.UnresolvableSpelling` | 2 |
| `std.live_tree.LiveTreeDisposition` | 1 |
| `std.live_tree.ReadsLiveTree` | 1 |
| `std.optional.Absent` | 32 |
| `std.optional.Optional` | 10 |
| `std.optional.Present` | 28 |
| `std.optional.optional_absent` | 2 |
| `std.optional.optional_present` | 1 |
| `std.types.List` | 1 |
| `v2.compiler.declaring_identity_spelling_census.DeclaringIdentitySpellingCensus` | 11 |
| `v2.compiler.declaring_identity_spelling_census.spelling_census_from_ingest` | 2 |
| `v2.compiler.normalized_tree.normalized_tree_roots_to_binding_sources` | 1 |
| `v2.compiler.program_assembly.module_roots_from_source_root_ingest` | 1 |
| `v2.compiler.reference_site_collector.collect_reference_sites` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceAmbiguous` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceOwnerless` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceResolved` | 1 |
| `v2.compiler.resolution_provenance.ProvenanceUnbound` | 1 |
| `v2.compiler.resolution_provenance.reference_site_scope_path` | 1 |
| `v2.compiler.resolution_provenance.resolve_reference_provenance` | 1 |
| `v2.compiler.source_authority.DagSourceReadWitness` | 4 |
| `v2.compiler.source_authority.SourceRootIngest` | 3 |
| `v2.compiler.symbol_index_fill.symbol_index_fill_module_roots` | 1 |
| `v2.extdeps.languages.dag.dag_language_model` | 3 |
| `v2.std.artifact.Artifact` | 1 |
| `v2.std.artifact.SourceFile` | 1 |
| `v2.std.cross_tree.import_model.V2Tree` | 1 |
| `v2.std.diagnostic.Accepted` | 3 |
| `v2.std.diagnostic.Rejected` | 3 |
| `v2.std.namespace_alias.ModuleBindingSource` | 3 |
| `v2.std.namespace_alias.module_binding_source_roots` | 1 |
| `v2.std.node.Symbol` | 2 |
| `v2.std.qualified_name.qualified_name_last_segment` | 1 |
| `v2.std.qualified_name.qualified_name_snoc` | 1 |
| `v2.std.qualified_name.qualified_name_to_dotted_string` | 4 |
| `v2.std.symbol_index.empty_symbol_index` | 1 |
| `v2.std.symbol_index.symbol_index_lookup` | 1 |
| `v2.test.claim.provenance.cross_file_provenance.cfp_index` | 1 |
| `v2.test.claim.provenance.cross_file_provenance.cfp_own_module_site` | 1 |
| `v2.test.claim.provenance.cross_file_provenance.cfp_roster` | 1 |

Providers: `dag/extdeps/communication/medium.dag`, `dag/extdeps/filesystem/filesystem_io.dag`, `dag/std/algebra.dag`, `dag/std/declaring_identity_spelling.dag`, `dag/std/live_tree.dag`, `dag/std/optional.dag`, `dag/std/types.dag`, `src/v2/compiler/declaring_identity_spelling_census.dag`, `src/v2/compiler/normalized_tree.dag`, `src/v2/compiler/program_assembly.dag`, `src/v2/compiler/reference_site_collector.dag`, `src/v2/compiler/resolution_provenance.dag`, `src/v2/compiler/source_authority.dag`, `src/v2/compiler/symbol_index_fill.dag`, `src/v2/extdeps/languages/dag.dag`, `src/v2/std/artifact.dag`, `src/v2/std/cross_tree/import_model.dag`, `src/v2/std/diagnostic.dag`, `src/v2/std/namespace_alias.dag`, `src/v2/std/node.dag`, `src/v2/std/qualified_name.dag`, `src/v2/std/symbol_index.dag`, `src/v2/test/claim/provenance/cross_file_provenance.dag`

### 3. `v2.test.claim.declaring_identity_spelling.verdict_fold` (`src/v2/test/claim/declaring_identity_spelling/verdict_fold.dag`)

| declaring path | occurrences |
|---|---|
| `std.decl_ref.DeclarationRef` | 1 |
| `std.decl_ref.decl_ref` | 1 |
| `std.declaring_identity_spelling.DeclaringIdentitySpelling` | 1 |
| `std.declaring_identity_spelling.IdentityAmbiguous` | 1 |
| `std.declaring_identity_spelling.IdentityOwnerless` | 1 |
| `std.declaring_identity_spelling.IdentityUnbound` | 2 |
| `std.declaring_identity_spelling.ResolvedAuthoredNotOnCanonicalPath` | 4 |
| `std.declaring_identity_spelling.ResolvedSpelling` | 3 |
| `std.declaring_identity_spelling.SpelledCanonically` | 4 |
| `std.declaring_identity_spelling.SpelledShorter` | 4 |
| `std.declaring_identity_spelling.UnresolvableSpelling` | 3 |
| `std.declaring_identity_spelling.declaring_identity_spelling_resolved` | 1 |
| `std.declaring_identity_spelling.declaring_identity_spelling_unresolvable` | 1 |
| `std.declaring_identity_spelling.resolved_path_verdict` | 2 |
| `std.live_tree.LiveTreeDisposition` | 1 |
| `std.live_tree.SubstrateInputsOnly` | 1 |
| `v2.std.qualified_name.QualifiedName` | 5 |
| `v2.std.qualified_name.qualified_name_from_dotted_string` | 5 |

Providers: `dag/std/decl_ref.dag`, `dag/std/declaring_identity_spelling.dag`, `dag/std/live_tree.dag`, `src/v2/std/qualified_name.dag`

## References the tables do NOT pin, and why

The ruling is coverage of every occurrence; pinning is narrower, deliberately, and the gap is named.

**Same-module references** (a name declared in the module itself; the wave's import-row edit cannot
re-point them, since their home is the module's own declaration namespace, not an import):

* census: `DeclaringIdentitySpellingCensus` 4, `spelling_census_from_ingest` 1, `spelling_census_over_roots` 2, `spelling_from_answer` 2
* production_ingest: `Xl3FixtureCensuses` 3, `Xl3PathCensus` 4, `a_bare_parameter_type_reaches_and_a_bare_field_type_does_too` 1, `a_bare_value_mention_outside_a_call_is_bound_holds` 1, `a_bare_value_mention_outside_a_call_reaches_the_census_holds` 1, `a_callee_name_reaches_the_census_holds` 1, `a_kernel_type_position_name_reaches_the_census_holds` 1, `a_qualified_cross_module_mention_reaches_the_census_holds` 1, `a_qualified_field_type_mention_reaches_the_census_holds` 1, `a_qualified_type_position_mention_reaches_the_census_holds` 1, `a_user_type_position_name_reaches_the_census_today` 1, `an_unbound_type_position_name_is_unresolvable_today` 1, `authored_is` 18, `authored_unbound` 3, `call_only_path` 3, `census_of` 10, `census_of_read` 2, `census_unreadable_roots_empty` 5, `cross_consumer_path` 2, `cross_provider_path` 2, `enrolled_same_module_site_binds_on_the_resolution_path_holds` 1, `field_vs_parameter_census` 2, `field_vs_parameter_path` 3, `fixture_read` 9, `ingest_fill_does_not_index_the_same_module_data_name_today` 1, `marker_site_fill_lookup_is_present` 2, `matched_census` 4, `matched_imported_kernel_type_position_reaches_the_census_holds` 1, `matched_type_vs_value_path` 3, `matched_user_type_position_mention_reaches_the_census_today` 1, `matched_value_only_mention_reaches_the_census_today` 1, `path_census` 7, `qualified_cross_module_spelling` 1, `qualified_field_type_spelling` 1, `qualified_type_position_spelling` 1, `read_for_path` 5, `shorter_path` 3, `two_module_census` 4, `two_module_usable_class_probe` 4, `unresolvable_path` 3, `value_outside_call_ingests_holds` 1, `value_outside_call_path` 7, `value_outside_call_roots` 3, `xl3_fixture_censuses` 4, `xl3_fixture_reads` 2
* verdict_fold: `bare_authored_is_spelled_shorter_holds` 1, `fold_bare_marker` 2, `fold_canonical_marker` 5, `fold_declaring` 2, `fold_fixture_module` 1, `fold_off_path` 2, `fold_partial_marker` 2, `fold_resolved` 3, `full_path_is_spelled_canonically_holds` 1, `partial_qualification_is_spelled_shorter_holds` 1, `resolved_off_canonical_path_is_its_own_arm_holds` 1, `unbound_is_unresolvable_holds` 1

**Names with no import row and no binder in the module** -- their home is whatever the resolver's
non-import row (an ambient/prelude row) names, and I could not pin that home by reading. These are
exactly the shared-leaf case the RED arm probes, so they are named rather than guessed:

* census: `declaring` 2, `ingest` 1, `lm` 1, `sites` 1, `sources` 2, `spelling` 7, `target` 2, `unreadable_roots` 1
* production_ingest: `Bool` 21, `String` 18, `artifact` 1, `authored` 6, `by_path` 1, `c` 2, `cause` 2, `census` 48, `consumer` 2, `content` 1, `file_path` 1, `filesystem_read` 1, `found` 2, `name` 2, `normalized` 2, `path` 7, `provider` 2, `rows` 2, `sites` 1, `spelling` 5, `two_module` 1, `unit` 2, `unreadable_roots` 1, `xl3_call_only` 2, `xl3_cross_consumer` 1, `xl3_cross_provider` 1, `xl3_field_vs_parameter` 2, `xl3_matched_type_vs_value` 2, `xl3_shorter` 2, `xl3_unresolvable` 2, `xl3_value_outside_call` 5
* verdict_fold: `Bool` 5, `String` 1, `authored` 1, `cause` 2, `verdict` 4

**Not references at all**: record field labels (`site:`, `cause:`, `spelling:` ...), `^symbol` atoms
(`^IdentityUnbound`), and `let`/parameter-bound positions. The reading counts them separately and drops
them from the tables; a wave edit that turns a label into a head, or a head into a label, is issue #1's
binding-position class, which the tables do not claim to cover.

## The wrong-declaration arm (RED), and why this shape

Option (i) -- a shared-leaf `Bool` -- is what tidy-ant-251 ruled on, and it is what the witness uses:
`src/v2/std/logic.dag:4` re-exports `Bool` from `std.types`, so `v2.std.logic.Bool` and `std.types.Bool`
share a leaf. A wave that rebound a bare `Bool` to the re-export instead of to the declaring home would
be invisible to any table keyed by LEAF NAME, and visible to one keyed by DECLARING PATH: the witness
asserts the real resolution paths REFUSE the table row `v2.std.logic.Bool` = 1.

Option (ii) -- a same-leaf twin for an identity name -- was ruled not needed on the statement that no
identity leaf has a twin at 345d6077fc4a. **That statement does not hold for one leaf, and the witness
carries the twin anyway.** `IdentityAmbiguous` is ALSO a variant of `RoadmapIdentityResolution` in
`dag/gunbc/roadmap/roadmap_identity.dag:132`:

    git grep -nwE 'IdentityAmbiguous' 345d6077fc4a -- '*.dag'

returns, besides the census/claim modules that use it:
`dag/gunbc/roadmap/roadmap_identity.dag:132` (`| IdentityAmbiguous { slug: String, matches: Int }`) and
`dag/test/claim/roadmap/roadmap_identity_witness.dag`, plus `dag/std/declaring_identity_spelling.dag`
(the wave module's own variant). The roadmap module does NOT import `std.declaring_identity_spelling`
(it is not in the 15-file population), so it is not a wave-2 site -- but a wave that inserts an import
can bring a colliding leaf in, and that is precisely the mis-binding the RED arm has to detect. So each
subject's wrong table carries a SECOND ROW, `gunbc.roadmap.roadmap_identity.IdentityAmbiguous` at that
module's own bare-use count (census 2, production_ingest 1, verdict_fold 1), and the assertion is that
the real resolution paths refuse it. One arm, two rows: the `Bool` row probes a re-export home, the
`IdentityAmbiguous` row probes a same-leaf twin in an unrelated module. For the other identity leaves the
"no twin" reading does hold: for each of
`DeclaringIdentitySpelling ResolvedSpelling SpelledCanonically SpelledShorter IdentityUnbound
IdentityOwnerless UnresolvableSpelling ResolvedAuthoredNotOnCanonicalPath declaring_identity_spelling_resolved
declaring_identity_spelling_unresolvable resolved_path_verdict`,
`git grep -nwE '<leaf>' 345d6077fc4a -- '*.dag'` puts the declaration head in
`dag/std/declaring_identity_spelling.dag` alone.
## Retention check for the census module

`src/v2/compiler/declaring_identity_spelling_census.dag` is one of the 3 site-bearing modules, so wave 2
would edit it. It survived the census retirement: `git show --name-only 15172708ae1f` (#15428) lists 16
files whose names contain "census" and NONE containing "declaring_identity". So it stays in wave 2 unless
swift-ibex-475 lists it; the one-line ask was sent. If it is on that list, the wave-2 population shrinks
to 2 site-bearing modules and this note's table 1 is dropped rather than edited.
**Answer received (swift-ibex-475, msg_5963d334): it is NOT on the removal list and not in flight** --
`v2.compiler.declaring_identity_spelling_census` is outside their starting set (the named labels in
`gunbc.instrument_targets`), no census-removal PR of theirs touches it, and it is cited by
`dag/gunbc/compiler_frontend_program_status.dag` (`DeclaringIdentitySpellingCensus` rows) plus a
production_ingest claim, so removing it would be a later slice they have not judged. So table 1 stands.

## What a run owes (unrun, and expected to refuse first)

1. The effect fix is in: zesty-badger-599's 93cd69197b00 replaced the unbound `filesystem_read` in wave
   1's `read_of` with the real effect, `Filesystem.Read(path: path).content`, and this branch merges it.
2. **The same unbound name is still in a wave-2 subject**: `src/v2/test/claim/declaring_identity_spelling/production_ingest.dag:80`
   calls `filesystem_read(path)`, and it is passed on as a value at lines 155/255. `git grep -n
   'fn filesystem_read' 345d6077fc4a` finds no DAG declaration of that name. So the production_ingest
   subject will refuse at resolution the way wave 1's first run did. That is zesty-badger-599's file to
   fix (finding sent), NOT patched here.
3. Every subject's closure reaches `src/v2/std/qualified_name.dag`, which at 345d6077fc4a calls the
   unbound `string_length` (line 304; `#15507`, wise-moth-707, is the egress and it is parked behind the
   heal grant folded into #15509). A run of any table here is therefore expected to report
   `resolve_reason_unbound_symbol ... "string_length"` before a single count is read. Measured
   elsewhere on the same class: the `//v2/test/claim/compiler/...` fold label on head 26ffb1382c7 refused
   5/5 at prepare with exactly that chain (receipt in PR #15487).
4. So the first run's deliverable is the refusal chain VERBATIM (symbol, file, byte range, the node
   occurrence it is attached to), then -- once #15507 and the `filesystem_read` fix are in the tested
   head -- the per-row counts, with any count the resolver disagrees with corrected IN THIS TABLE and the
   correction recorded here.

## Reproducing the counts

    REV=345d6077fc4a
    git show $REV:src/v2/compiler/declaring_identity_spelling_census.dag   # and the other two subjects
    # import rows -> the (name -> declaring module) map; every non-comment, non-import-row mention of a
    # mapped name is one occurrence of that declaring path.

The script that produced the tables is described in the commit message of this branch's witness commit;
it reads only `git show` output at $REV, so the tables are reproducible without a checkout of the wave.

## Unknowns

* Which mentions the RESOLVER mints as occurrence records (a record-field label, a type position, a
  `data` name) -- the counts would move; the table is the hypothesis to be corrected, not a claim.
* Whether the resolver needs more than the one-level provider list to resolve a subject (wave 1 used 5
  providers; the census module names 15). If it refuses on an unbound symbol INSIDE a provider, widen
  the ingest to the transitive import closure -- the closure is computable from the same `module` +
  `import` rows, and the widening must be recorded here rather than silently done.
* `module std.types` (and a few other names) is declared by more than one file at 345d6077fc4a -- e.g.
  `dag/std/types.dag` and `fixtures/class_b_trim/narrow_pool/dag/std/types.dag`. The provider paths here
  take the non-fixture path. A run that admits two files for one module name would be a different
  failure and must be reported as such.
* The home of the un-imported `Bool`/`String` uses (see "Names with no import row") is unpinned.
* Population completeness: `git grep -lw declaring_identity_spelling` finds files whose text carries the
  word. A file that imports the module through a path not spelled with that word (there is none in the
  layout at 345d6077fc4a by inspection of `dag/gunbc/source_layout_namespaces.dag`) would not be found
  by it. No completeness claim is made beyond that population.

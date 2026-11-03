# XL3 per-wave identity witness: shape and first wave (prep, NOTHING RUN)

Assignment: tidy-ant-251, approved sleek-owl-397. Fits calm-eagle-670's Part C template step 3 (gunbc-private #268, `docs/plans/2026-10-31-partC-wave-acceptance-template.md`). No census, no population gate; the source is written and has never been executed.

## What XL3 changes, and what the witness proves

XL3 (`NamespaceCutStage` `StageXL3DeclaringIdentitySpelling`) respells each reference as its fully qualified declaring identity (`Bool` becomes `std.types.Bool`). Resolution is meant to be unchanged. The witness proves "unchanged": it pins, per touched module, the declaring path each named reference binds to, reads the table back through the production resolver, and is run at base (bare spellings) and at head (qualified spellings). The same answer at both is the evidence; a respelling that rebound a reference changes a count or introduces a path outside the table.

## Shape (one claim module per wave, `src/v2/test/claim/long/`)

1. Real acquisition of the touched module plus its import providers via `module_roots_from_source_root_ingest` (the `declaring_identity_spelling/production_ingest` form), `ReadsLiveTree`.
2. Production resolver: `v2.compiler.name_resolve resolve_with_admission_policy`, then read `ResolvedTree.occurrence_resolutions` (`BoundToDeclaration { ResolvedToDeclaration { path } }`). This is the same resolution record the resolver mints, not a second resolver and not the retired `occurrence_resolution_census`.
3. A pinned table keyed by DECLARING PATH (never by leaf name) with an occurrence count.
4. GREEN: table holds. RED: the same producer against a table naming a same-leaf wrong declaration (`v2.std.logic.Bool` for `std.types.Bool`) must be refused, so the claim discriminates by declaration identity.
5. Lives under `long/` because from-source claims exceed the 72.3k new-witness eval budget (see `name_resolve/test_code_reference_wall` header); run with `gunbc run --claim-run --entry <file>` at base and at head, not as a floor witness. The supplied-root form of `name_resolve_cross_tree_resolution` does not fit alone: its inputs are synthetic roots, so it cannot name a real module's references. Its pairing obligation (an inhabitance claim) is what this from-source claim discharges for the wave.

Known route limit (measured by `std.declaring_identity_spelling` header): the ingest-fed spelling collector does NOT see qualified/dotted references in any position, names inside calls, or bare type names in field position. That is why this witness reads the resolver's own occurrence map instead of `spelling_census_from_ingest`: the collector cannot see the post-XL3 spellings at all.

## First wave: module `std.declaring_identity_spelling`

Candidate set, cited. Command: `git grep -nw -E "Bool|DeclarationRef|decl_ref|DissolutionCondition|retires_dissolution|is_prefix_of|list_reverse|QualifiedName" 345d6077fc4a -- dag/std/declaring_identity_spelling.dag`, filtered to non-comment, non-import lines. Revision 345d6077fc4a. Population: one 108-line module, five import sources.

| declaring path | mentions |
|---|---|
| std.types.Bool | 1 |
| std.decl_ref.DeclarationRef | 2 |
| std.decl_ref.decl_ref | 1 |
| std.dissolution.DissolutionCondition | 1 |
| std.dissolution.retires_dissolution | 1 |
| std.algebra.is_prefix_of | 1 |
| std.algebra.list_reverse | 2 |
| v2.std.qualified_name.QualifiedName | 10 |

Unknowns, not claimed away: (a) the counts are text counts and the resolver may mint a different number (for example type-position vs value-position records, or an Arrow's two slots, see the `duplicate_binding_records` note in `v2.compiler.resolve`); the first run may correct a count, and the table must then be re-pinned from the resolver AT BASE and reviewed, never from head; (b) whether `std.types.Bool` resolves to a declaration or to a kernel symbol (`ResolvedToKernelSymbol`) is unestablished, in which case the Bool row is read from the kernel arm instead; (c) importers of the module (15 files by `git grep -lw declaring_identity_spelling`) are a separate wave; (d) the Admission as written (empty imports, `ImportScoped`) is a guess at what the module needs to resolve and may need the providers' import rows.

## What a run owes (not done here)

Base run, head run, both green; RED run answering true (the refusal); a count correction, if any, recorded with its base evidence. A run must be a named unit on the serial srv1 queue.

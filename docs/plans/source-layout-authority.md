# Source layout authority: one storage rule, a name-only compiler, and the end of whole-tree ingestion

**Status:** plan for execution. The operator rulings of 2026-10-10 are recorded in §3; open decisions, each with a recommendation, are in §8. No code lands from this document alone.

**Registration:** not yet a row in `gunbc.plan_governance` `plan_governance_rows`. The change that adopts this plan adds the row (proposed: governed by `v2-compiler-architecture`, role `CompilerStageAuthority`, stage "source acquisition: layout authority, storage rule and the name-only compile boundary") and regenerates the derived banners.

**Supersedes, as to the storage half:** `docs/plans/module-identity-storage-binding-design.md` and `docs/plans/ingest-manifest-source-ref-carrier-design.md`. Both were deleted in the 2026-08-28 plan cleanup (gunb-ai/gunbc#9635); their last versions are at commits `dbf227cb7f4a` and `628ebe126bec`. §2.5 says what still governs from them.

**About the numbers:** every count here is a one-off measurement at `e3939ea`, taken by throwaway scripts during planning, not by an owned instrument. The P1 sweep instrument re-derives them; the change that lands P1–P2 deletes the numbers here rather than updating them (DESIGN §6: name the instrument, never transcribe its output).

---

## 1. Summary

- The v2 self-host step reads, tokenizes and indexes every `.dag` file in the repository (8,316) in order to emit a closure of roughly 256. About half of the seed's resident memory on that step is whole-tree structures.
- It has to, today, for three reasons: a module's file can only be found by reading every file's `module` line; bare references (names used without an import) are resolved by asking which module, anywhere, declares the name; and everything outside the closure is fed in on purpose so that a parse error anywhere fails the step.
- v1 and v2 disagree about what a name means. v2 binds a name only if it is declared, imported by name, or fully qualified, and refuses everything else. v1 also binds names through transitive imports, unlisted names and a whole-repository fallback. v2's rule is the rule; v1 is not changed.
- The fix is a layering that was designed on 2026-07-18 and then lost: the compiler works with module names only, and a storage layer — the only code that knows paths — maps names to files.
- This plan makes that storage layer a top-down authority: a declared namespace layout plus one rule, module `a.b.c` ↔ `<root(a)>/b/c.dag`, with every file brought into line in one sweep. A file outside the layout is not compilable.
- Then: fix the bare references in the self-host closures and lock v2's rule in on the required job; switch v2's source acquisition to loading modules by name, on demand, and delete the three acquisition mechanisms that exist today; decide how far to take the seed's memory before v2 compiles itself.

## 2. How we got here

This plan came out of one planning conversation that started from the memory audit of the self-host step. Each subsection is one link of that chain.

### 2.1 The memory question

The memory composition audit of the self-host step (`GUNBC_MEMORY_COMPOSITION=1`, the seed's readout; receipt at `7792ce6f013` in the instrument's PR) found the seed's resident set split roughly in half: whole-tree structures (the token pool, the resolve index and name census, the parsed heads of every module) and a closure-scale working set (closure ASTs, resolution and typecheck environments, emission buffers) that the instrument does not release and so does not measure. The slot's peak sits under the 22 GiB envelope with a margin that shrinks as the repository grows. gunb-ai/gunbc#13674 (release the token pool before resolution) lowers the seed's resident set but leaves the whole-tree index in place.

The question was why the step that emits only the v2 compiler reads the whole repository at all.

### 2.2 Why the seed reads everything

The emission transaction (`v1_compiler.cli_run` `compile_emission_over`) builds one index over both source roots, loads the entry's closure (`load_sources_for_entry_with_pool`), and passes every module outside the closure in as census-only input (`compile_clean_pipeline_options_for_sources`) — deliberately: its own comment says this is what makes a parse break anywhere under the roots fail the step. It resolves and emits only the closure, which an import-and-dotted-reference walk puts at about 256 files.

Three facts make the whole-tree read load-bearing:

1. **Module names do not locate files.** A module's identity is its `module` line, not its path, and 42% of files do not sit where their name would put them (Appendix A). Finding one module therefore means reading every header.
2. **Bare references are reverse lookups.** A name used without an import (`Arrow`, `fold_list`) carries no module. The seed answers "which module, anywhere, declares this name?" from a census of everything: the loader's bare-reference channel for files with no import lines (`visit_bare_reference_providers`) and the resolver's whole-pool fallback (`v1.compiler.infer_env` `global_bare_lookup_candidates`).
3. **The whole-tree parse is also a check.** Since 2026-10-09 it is the only thing on the required path that touches files outside the two emitted closures.

v2 has the same shape. Its CLI `emit` path parses and censuses every file, then derives the closure from references (`v2.compiler.self_host.closure_emission` `closure_ingest`). The v2 native test lane is the exception: it scans each file's `import` lines as text and parses only the import closure (`v2.compiler.compile` `native_lane_source_facts`, `native_lane_closure_ingest`, accepted by `native_lane_ingest_matches_closure`). That scan is a second, partial parser, and it is only correct on code that has no bare references.

### 2.3 v1 and v2 bind names differently

| Reference | v1 seed | v2 |
|---|---|---|
| declared in the same module | binds | binds |
| listed in `import M { x }` | binds | binds |
| written fully qualified | binds | binds |
| in M but not in the `{ }` list | binds | refuses |
| exported by something M itself imports | binds | refuses |
| not imported, declared once among the loaded modules | binds | refuses |
| unimported variant in a `match` pattern | binds | refuses |
| `Present` / `Absent` / `List` with no import | binds (kernel) | refuses |

v1: an import exposes the imported module's whole flattened export set, inherited names included (`v1.compiler.infer` `direct_import_export_precedence_note`); a name declared by exactly one loaded module binds from anywhere, and a name declared by several binds to the unique one on the referencing module's ancestor chain (`global_bare_lookup_candidates`); import lists are enforced only as a non-blocking advisory for types (`UnlistedImportUse`, severity `SeverityNonError`) and, for functions, for exactly one name (`v1.compiler.infer_env` `bare_free_call_requires_listed_import`, true for `trim` only).

v2: an import becomes one binding row per listed name (`v2.extdeps.languages.dag` `import_binding_rows_from_decl_node`); a name not bound on the reference's own chain refuses, and nothing searches the repository (`v2.compiler.resolve` `lookup_symbol_index_atom_identity`: "nothing follows an exhausted chain"); an unimported constructor in a pattern refuses too.

Consequences:

- Code the seed accepts and emits as v2 is not valid v2. An estimate (Appendix A) puts about 480 such references in about 105 of the closure's files: mostly unimported `v2.std.node` variants in patterns, about 29 `Present`/`Absent`, about 30 `List`/`Map`, and builtins v2 does not provide implicitly (`filter`, `any`, `count`). v2 cannot resolve its own source until they are fixed, which blocks generation two regardless of this plan.
- The bare-reference debt roster (`v2.workflow.floor_unimported_bare_provider_debt_roster`, 271 active pairs) tracks a narrower class over the whole tree; one pair falls inside the closures. Its gate ran in the floor lane, deleted on 2026-10-09, so nothing enforces it now.

### 2.4 The design existed

On 2026-07-18 the operator directed `docs/plans/module-identity-storage-binding-design.md` (gunb-ai/gunbc#6856). Its recorded intent:

> we should not depend on literal files; we should depend on nodes/modules, where a module is a collection of nodes and may be represented by ≥0 files at parse time … more ideally, code references modules and never deals with the bidirectional mapping at all.

and a second directive the same day: decide what happens to the graph first, then make the files represent it faithfully — "the direction of travel is a system that no longer transacts with files at all."

Its model: a module is a node (a containment subtree); a file is one storage realization of it; the path⇄module binding has one home, `v2.compiler.source_authority`; paths appear only at host boundaries, as a typed `SourceRef` (path, source root, content hash). Phase 1 landed on 2026-07-19 (gunb-ai/gunbc#6882 and follow-ups): `ModuleStorageBinding`, `ModuleStorageIndex`, `SourceRef`, and `module_storage_bindings_for_source_roots` with a host manifest. Both design documents were deleted on 2026-08-28 (gunb-ai/gunbc#9635) as stale off-topic plans. The code still cites them (`v2.compiler.source_authority` `module_storage_binding_authority_note`, `source_ref_authority_note`), and none of the three compile paths uses the binding authority.

### 2.5 What carries forward, and what changes

Carries forward: a module is a node and a file is a storage realization of it; one authority for the mapping, homed in `v2.compiler.source_authority`; `SourceRef` at host boundaries; delta-first as the direction of travel; and the namespace design's ruling that a module's position is never inferred from where a file happens to sit (`docs/plans/namespace-resolution-design.md` §10).

Changes: the July design derived the mapping by parsing (bottom-up). This plan declares it (top-down): a layout and one rule, from which every file's location follows. The `module` line becomes a checked copy until the seed retires. The July design's markdown and Rust write-back phases are out of scope here.

## 3. Rulings (operator, 2026-10-10)

- **R1.** v2's binding rule is the rule: a name is declared, imported by name, or fully qualified, or it refuses. Any file v2 resolves must satisfy it; there is no exemption list.
- **R2.** v1's resolver is not changed to match. The hard error is v2's, executed on the seed-generation step by the binary the seed builds.
- **R3.** One storage convention and no dual representations. A dual representation that cannot be removed yet is declared and bounded.
- **R4.** The model does not assume one file per module.
- **R5.** Storage rule: module `a.b.c` lives at `<root(a)>/b/c.dag`, where `root(v2) = src/v2` and `root(a) = dag/a` for every other top-level namespace. So `v2.compiler.resolve` → `src/v2/compiler/resolve.dag` and `std.types` → `dag/std/types.dag`. Paths follow names, never the reverse.
- **R6.** Modules stored where their name does not put them are fixed. Where a folder carries structure the name lacks, the folder goes into the name (e.g. `gunbc.x` stored in `dag/gunbc/host/` becomes `gunbc.host.x`).
- **R7.** The migration is done up front, in one sweep. No exception list outlives the cutover.
- **R8.** Work anchors on the v2 native process. v1 changes are limited to what the cutover mechanically requires.

## 4. Target design

### 4.1 Layers

- **Storage layer.** The only code that knows paths. Given a module name it returns the module's source and a `SourceRef`; given a namespace it can enumerate its modules. It owns the layout, the rule, the reads and their verification.
- **Compiler.** Works with module names only. It starts from the entry module, parses it, and requests every module its imports and qualified references name. The closure is whatever got requested; nothing computes it up front.
- **Whole-population jobs.** "Does every file parse?", "run every claim": explicit requests to the storage layer to enumerate the layout — never a side effect of compiling one entry, as they are today.

### 4.2 The layout authority

Two parts, kept apart because one is a language fact and the other a repository fact (DESIGN §3: interface, realization and policy are three facts):

- **Generic, in `v2.compiler.source_authority`:** the layout types, the rule, `module_source(name)` (rule → read → verify → `SourceRef`), namespace enumeration, and the layout check.
- **Repository data, in a new import-free module** (proposed `gunbc.source_layout`): the namespace-root map (`v2` → `src/v2`; every other top-level namespace `a` → `dag/a`) and one row per declared namespace — the package grain, about 560 rows — each with a kind. It has no imports so that it is the first and smallest thing any run reads.
- **Generated files** stay in `gunbc.generated_artifact_registry`, which already names each file's producer and commit policy. The check consults it; nothing restates it.

Kinds: `Authored` (hand-maintained bodies; the skeleton is generated); `Generated` (bytes from a producer, never hand-edited); and, if the sweep instrument finds the need, `Fixture` (test inputs that are deliberately malformed or compiled as their own root, excluded from the rule and from compiles).

Adding a namespace is a row plus a generated folder. Adding a module is a file at the rule's position whose `module` line the skeleton generator writes. Nothing else declares a module, so no list of modules duplicates the files (O1).

`src/v1` (the seed's own `.dag` sources) is outside the layout and retires with the seed.

### 4.3 The rule and lookup by name

`path(a.b.c) = root(a) + "/b/c.dag"`.

A qualified reference such as `v2.std.diagnostic.None` is split into module and member by probing whether `path(prefix)` exists, longest prefix first: an existence check per candidate, not a scan.

### 4.4 The layout check

Run by the binary the `v2-native-cli` step already builds, as one more check on the required `witnesses` job — no new job, and judged natively as the 2026-10-09 ruling requires of verdicts. It lists the declared roots and reads each file's first line. Its refusals are typed and located:

- `UndeclaredNamespace` — a file in a folder that no namespace row declares;
- `NotAtRulePosition` — a file whose `module` line names a module the rule places elsewhere;
- `ModuleLineMissing`;
- `DuplicateModule`;
- `RegistryArtifactMissing` — a registered generated file that is absent.

This is the one whole-tree read left, and it reads headers, not bodies. It only partly replaces the incidental parse-everything check (O5).

### 4.5 Closure by demand

Entry module → parse → requested names (imports and qualified references) → `module_source` → parse → repeat until nothing new is requested. This is the "parse-then-derive" mechanism `docs/plans/namespace-cut-replacement-plan.md` already names as the terminal closure assembly.

A bare name is not requestable: it names no module. Under R1 it refuses anyway. v2's "declared in X" hint on that refusal is the one remaining reverse lookup; it moves to the storage layer and runs only when something has already failed.

The ingest receipt keeps the native lane's identity join: the files read and parsed must equal the requested closure exactly, so a hidden whole-tree prepass is a refusal, not a slowdown.

### 4.6 The one dual representation that remains

The `module` line. The seed reads it to locate modules, so it stays until the seed retires — written by the skeleton generator and checked against the rule. After that it is deleted and the rule alone carries identity. Every other copy goes: path strings in code become module references (P2), and no exception list survives the cutover (R7).

### 4.7 Not one file per module

The model admits a module with no file (produced declarations, already representable as `ModuleStorageProvenance` `ProducedByBehavior`) and several namespaces in one file (nested blocks, `docs/plans/namespace-resolution-design.md` §10 step 2). The data stays one file per module while the seed compiles it, because the seed requires every module to be declared by exactly one file.

### 4.8 Out of scope

- Changing v1's resolver (R2).
- The July design's markdown and Rust write-back surfaces.
- Non-`.dag` files beyond what `gunbc.generated_artifact_registry` already covers. Extending the layout to every file in the repository, so that any unknown file fails, is a natural later step.
- Deleting the `module` line before the seed retires.

## 5. Plan

The phases land in order; P2 is the single cutover sweep (R7).

### P0 — Record and register

- Record R1–R8 in a typed carrier (DESIGN §4c: a ruling belongs in a carrier, not only in prose).
- Add this plan's `gunbc.plan_governance` row and regenerate the banners.
- Amend DESIGN §3 through `gunbc.design_document` and regenerate DESIGN.md. Today §3 says the `std`/`extdeps`/`compiler`/`workflow` folders are "browsing conventions" and that "paths are discriminators, not gospel"; R5–R6 make a module's folder part of its identity. The amendment states the storage rule as §3's single authority for the name⇄path mapping and says exactly what this plan says, nothing wider: acyclicity stays the import graph's only structural law, and the rule fixes where a file is stored, not which way imports may point.

### P1 — Layout authority and check

There is one way to process the layout, and it refuses when the tree does not satisfy it. No report mode exists: the check lands refusing, in the same change as P2's sweep that makes the tree satisfy it.

- Declare the layout types, the rule and the check in `v2.compiler.source_authority`. Declare the **target** layout in `gunbc.source_layout`: the namespace rows as they will be after P2's renames.
- Add a `layout` verb (or equivalent) to the v2 native CLI — a new verb is one more arm of `v2.cli.compile_cli` `CliVerb` — and run it on the required job. It refuses with the typed, located reasons of §4.4, and holds otherwise.
- The move-and-rename map is computed by the P2 sweep instrument from the rule and the tree (Appendix B gives the procedure), re-derived on every run and never committed as data. It is an input to the sweep, not an output of the check.
- Evidence: fixture trees for each refusal (undeclared folder, misplaced file, missing `module` line, duplicate module) refuse with their typed reason; a conforming fixture passes; the real tree passes after the sweep.

### P2 — The cutover sweep (lands with P1, one change, regenerable)

Produced by a re-runnable instrument that computes the map from the rule and the pre-sweep tree, so the change can be regenerated after merging main instead of being fixed up by hand. The precedent is `tools.source_reference_repoint`, which was re-run after main merges on gunb-ai/gunbc#13388.

0. **Collisions refuse first.** Before any edit, the instrument computes every target path and every new module name, and refuses the whole sweep on any collision: two sources mapped to one target, a target that already exists and is not itself moving away, or a new module name already declared anywhere (`.dag` names are corpus-global). A collision is a decision, never a suffix or a merge.
1. **Module renames** (folder into name, R6) through `tools.source_reference_repoint`, extended to take the whole rename map if it does not already. It rewrites imports, qualified-name prefixes and `DeclarationRef` module paths. The tool itself is a `tools.*` module that the sweep renames, so run it from the pre-sweep tree.
2. **File moves and renames** to the rule's positions.
3. **Path strings.** Data rows whose value is a `.dag` path (193); `.dag` paths in Rust (760 mentions in 67 files, including the seed's `NATIVE_COMPILE_ENTRY` and `REPORT_ENTRY`); paths in `gunbc.instrument_targets` rows; paths in CI workflows (82 mentions — regenerate them from their producers). Where a path is load-bearing, replace it with a module reference resolved through the layout (the July `SourceRef` intent). Where it is an argument to the seed's CLI, rewrite it.
4. **Strings the repoint tool does not touch:** labels derived from module names (`//v2/test/...`), and prose citations of renamed modules in notes, comments and `gunbc.design_document`. Rewrite them from the same map. Module names are dotted and distinctive, but review the diff.
5. **Seed mirrors.** Ten renamed modules have mirror files in the seed's stage0 tree (among them `extdeps.cargo`, `extdeps.cargo_version`, `gunbc.reference_derived_candidate`). Rename the mirror files and their Rust paths in the same change (O3).
6. **Regenerate** `.gitattributes`, the workflows, and every generated artifact whose producer reads a moved path. No CI step compares generated artifacts with their producers since 2026-10-09 (`gunbc.rung_drop` `v1_required_lanes_withdrawn`), so this is verified locally and said so in the change.
7. **Seed emission is unchanged.** v1 binds a bare name by the referencing module's ancestor chain (`v1.compiler.infer_env` `global_bare_lookup_candidates`), and the renames move modules onto new chains before P3 removes the bare references. So a rename can change which declaration the seed picks, or make a unique pick ambiguous, with no error. The sweep is admitted only if the seed's emission of both self-host closures is the same before and after, modulo the rename map; any other difference stops the sweep.

Evidence: the check holds on the real tree, and no exception list exists; the seed-emission comparison of step 7 holds; both required steps are green; re-running the instrument after merging main changes nothing; a branch that adds a misplaced file turns the required job red.

### P3 — Bare references in the self-host closures

- Get the exact list from v2 itself. The built CLI's `emit` on the real entry already counts member refusals (`CliEmitted` `refused_count`). If its output does not list every unbound name, or later-stage refusals hide resolve ones, add a resolve-only verb that does.
- Fix each by adding an import entry or qualifying the name. `Present`/`Absent`: import `std.optional` (O2). `filter`/`any`/`count`: import their declaring module, or follow the planned retirement of `v2.std.algebra` `filter`/`any` (gunb-ai/gunbc#13541 closed unfinished and has to be redone).
- Lock it in: the built binary resolves both closures on the required job with zero unbound or ambiguous names. A reintroduced bare name turns the job red.
- Retire the debt roster, its gate `v2.workflow.floor_unimported_bare_provider_debt`, and — as v1 maintenance — the seed's `unimported_bare_provider_gate`. All have been inert since 2026-10-09. The population outside the closures stays in the declared drop `v1_required_lanes_withdrawn`, not in a new roster.

### P4 — Acquire by name (the consolidation)

- v2's compile paths — the CLI `emit`, the native lane, and the callers of `v2.compiler.program_assembly` — get sources only through `module_source` and closure by demand.
- Deleted in the same change (DESIGN §3: cut over at the root): the parse-everything phase of `closure_ingest`; the native lane's text scan and import walk (`native_lane_source_facts`, `native_lane_closure_modules`); the use of `source_root_read`'s whole-root walk as compile acquisition; and the discovered binding (`module_storage_bindings_for_source_roots` and its host manifest emitter), whose rows the rule now produces.
- Whole-population jobs become explicit enumerations of the layout.
- Evidence: the ingest receipt's identity join holds for every compile; a malformed module outside the closure changes neither the verdict nor the set of files read; the deleted producers have no remaining consumers.
- Consequence for memory: v2's own compiles read only the modules they request, so their memory scales with the closure, not the repository.

### P5 — The seed's memory

The seed keeps its whole-tree index until v2 compiles itself. Options (O6):

- (a) accept it until then;
- (b) land gunb-ai/gunbc#13674;
- (c) run the seed on a scratch root holding only the closure P4 computes — no v1 change — and confirm once that the emitted crate is identical to a whole-tree run. This also measures the closure-scale half of the seed's memory directly, which the audit could not.

Measure with the existing memory composition instrument on srv1.

### P6 — When the seed retires

Delete the `module` lines and the check's header agreement. The rule alone carries identity.

## 6. Order and dependencies

P0 → P1+P2 (one change) → P3 → P4. P3 follows P2 because P2 renames the modules that P3's import lists name. P4 follows P3 because loading by name cannot request a bare name. P5 can start at any time; option (c) needs P4. P6 follows the seed's retirement.

## 7. Risks

- **Conflicts.** P2 touches thousands of files. Land it in a quiet window. Because an instrument produces it, a conflict is resolved by re-running, not by hand.
- **Strings outside the repoint tool's reach** go stale silently. P2.3 and P2.4 cover them from the same map; a census of path and module-name strings comes first.
- **Seed edits.** Path constants and ten mirrors change in v1. They are admitted under `gunbc.v1_maintenance_standing` `v1_seed_standing` because the self-host build needs them; nothing else in v1 changes.
- **Generated artifacts** are not compared with their producers in CI; regenerate them deliberately.
- **References from outside this repository** (other repositories, saved links, roadmap rows) to moved paths or renamed modules. Census them.
- **Fixtures** that are deliberately malformed or compiled as their own root need the `Fixture` kind; the sweep instrument finds them, and the check refuses any it misses.
- **The incidental parse-everything check goes away** with P4 and with option (c) of P5. O5 decides whether to replace it with an explicit whole-population job.

## 8. Open decisions

- **O0. Collisions found by P2 step 0**, if any, are decided one by one before the sweep runs.
- **O1. What the layout declares.** Namespaces only — recommended: about 560 rows, and modules are files at rule positions. Or every module — about 8,300 rows that restate the file list.
- **O2. `T?` constructors.** Recommended: no implicit scope; a module that constructs or matches `Present`/`Absent` imports `std.optional`, consistent with R1. This answers the question left open in `gunbc.recurring_failure_mode` `bare_name_resolved_only_by_the_flat_namespace`, and it is the binding the Optional de-fork's remaining steps (dropping the `none` literal; one Optional authority on both routes) build on. Those steps have no PR yet.
- **O3. Seed-mirrored modules in the rename set.** Rename them and their mirrors (recommended: keeps folders and names consistent), or move only their files.
- **O4. Names that are deliberate top-level layers rather than missing folders** (Appendix B, group D). `product.*` stored under `dag/gunbc/product/`: recommended to move the files to `dag/product/`, keeping the layer DESIGN §3b cites (`product.fabric.selection`, `product.capacity.lease`). `tools.*` stored in `dag/gunbc/instruments/`: recommended to rename to `gunbc.instruments.*`, matching the instrument labels `//gunbc/instruments:<target>`.
- **O5.** Whether to keep "every file parses" as an explicit whole-population job once nothing parses everything as a side effect.
- **O6.** Which seed memory option of P5.

## 9. Relationship to existing plans and authorities

- `docs/plans/namespace-resolution-design.md` — §10 (a module's position is declared, never inferred from storage; the `module` header is placement sugar that ends as a projection) is satisfied by declaring the rule; §13's unique-on-chain resolution is R1.
- `docs/plans/namespace-cut-replacement-plan.md` — its "parse-then-derive" closure mechanism is §4.5; stage XL-3 (every reference spelled as its declaring identity) overlaps P3.
- `docs/plans/demand-engine-program.md` — §1's unruled next derivation ("an authority cheaper than full ingestion … a source-root manifest … the module dependency closure of the requested targets") and M3a's context-assembly note are P4.
- `docs/plans/replacement-migration-doctrine.md` — P2 and P4 are replacement migrations; each deletes what it replaces in the same change.
- The memory composition instrument and gunb-ai/gunbc#13674 — P5.
- `gunbc.generated_artifact_registry` — unchanged; consulted by the check.

---

## Appendix A — Measurements (one-off, at `e3939ea`)

Method: a script reading each `.dag` file's `module` line under `dag/` and `src/v2/` and comparing the file's path with R5's rule. Deleted by the change that lands P1–P2.

| | files |
|---|---|
| `.dag` files | 8,316 (`dag/` 6,626; `src/v2/` 1,690) |
| at the rule's position | 4,829 — 58% (`src/v2/` alone: 37%) |
| A — same folder, different file name | 1,500 |
| B — module name deeper than its folder | 2 |
| C — folder deeper than the module name | 1,110 (22 of them shaped `x/x.dag`) |
| D — different branch | 875 |
| folders holding `.dag` files | 588 |
| namespaces that directly contain modules | 557 |

Other one-off counts:

- Modules named `gunbc.<leaf>`: 1,290, of which 675 sit in topic folders (`machine_intake/`, `host/`, `roadmap/`, `fleet/`, `runner/`, `fabric/`, `ci/`, `namespace/`, `bmc/`, …).
- Data rows whose whole value is a `.dag` path: 193.
- Self-host closures, approximated by an import and dotted-reference walk from `v2.compiler.compile` and `v2.cli.compile_cli`: about 256 files, 18 of them import-free.
- References in those closures that v2 would refuse, by pattern match with known false positives: about 480, in about 105 files.

## Appendix B — The move-and-rename procedure

For each file with `module M` at path `P`, let `R = path(M)`:

- **A — same folder as R:** rename the file to R. This includes the numbered stage files (`src/v2/compiler/00_compile.dag` → `src/v2/compiler/compile.dag`).
- **B — R's folder lies below P's:** move the file down to R.
- **C — P's folder lies below R's:** the folder carries structure, so rename the module to include it (R6): `gunbc.x` at `dag/gunbc/host/x.dag` → `gunbc.host.x`. Exception: `x/x.dag`, where the folder only repeats the module's own name → move the file up to R.
- **D — different branch:** decided by group. The largest groups and the proposed outcome:

| Group (module prefix @ folder) | files | Proposed |
|---|---|---|
| `v2.test.manual` @ `src/v2/test/claim/manual/` | 162 | rename to `v2.test.claim.manual.*` |
| `tools` @ `dag/gunbc/instruments/` | 93 | rename to `gunbc.instruments.*` (O4) |
| `v2.test.long` @ `src/v2/test/claim/long/` | 57 | rename to `v2.test.claim.long.*` |
| `v2.test.execution` @ `src/v2/test/claim/execution/` | 54 | rename to `v2.test.claim.execution.*` |
| `v2.test.emit` @ `src/v2/test/claim/emit/` | 53 | rename to `v2.test.claim.emit.*` |
| `product.printed_chassis` @ `dag/gunbc/product/printed_chassis/` | 52 | move to `dag/product/printed_chassis/` (O4) |
| `product` @ `dag/gunbc/product/` | 36 | move to `dag/product/` (O4) |
| `test.claim` @ `src/v2/test/claim/parse/` | 26 | rename to `v2.test.claim.parse.*` |

Smaller groups follow the same reading; for example `gunbc.test.*` and `dag.test.*` modules under `dag/test/` are renamed to `test.*`, joining their 2,100-odd `test.claim.*` siblings. The P2 sweep instrument computes the complete table; collisions refuse under P2 step 0. Anything not covered by A–C or by a confirmed group outcome stops the sweep for a decision; it is never guessed.

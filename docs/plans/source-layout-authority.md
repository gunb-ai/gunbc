# Source layout authority: one storage rule, a name-only compiler, and the end of whole-tree ingestion

**Status:** plan for execution, decisions closed. The operator rulings of 2026-10-10 are in §3. The plan lands as one PR (§5).

**Registration:** not yet a row in `gunbc.plan_governance` `plan_governance_rows`. The change that adopts this plan adds the row (proposed: governed by `v2-compiler-architecture`, role `CompilerStageAuthority`, stage "source acquisition: layout authority, storage rule and the name-only compile boundary") and regenerates the derived banners.

**Supersedes, as to the storage half:** `docs/plans/module-identity-storage-binding-design.md` and `docs/plans/ingest-manifest-source-ref-carrier-design.md`. Both were deleted in the 2026-08-28 plan cleanup (gunb-ai/gunbc#9635); their last versions are at commits `dbf227cb7f4a` and `628ebe126bec`. §2.5 says what still governs from them.

**About the numbers:** every count here is a one-off measurement at `e3939ea`, taken by throwaway scripts during planning, not by an owned instrument. The C0 generator re-derives them; the cutover PR deletes the numbers here rather than updating them (DESIGN §6: name the instrument, never transcribe its output).

---

## 1. Summary

- The v2 self-host step reads, tokenizes and indexes every `.dag` file in the repository (8,316) in order to emit a closure of roughly 256. About half of the seed's resident memory on that step is whole-tree structures.
- It has to, today, for three reasons: a module's file can only be found by reading every file's `module` line; bare references (names used without an import) are resolved by asking which module, anywhere, declares the name; and everything outside the closure is fed in on purpose so that a parse error anywhere fails the step.
- v1 and v2 disagree about what a name means. v2 binds a name only if it is declared, imported by name, or fully qualified, and refuses everything else. v1 also binds names through transitive imports, unlisted names and a whole-repository fallback. v2's rule is the rule; v1 is not changed.
- The fix is a layering that was designed on 2026-07-18 and then lost: the compiler works with module names only, and a storage layer — the only code that knows paths — maps names to files.
- This plan makes that storage layer a top-down authority: a declared namespace layout plus one rule, module `a.b.c` ↔ `<root(a)>/b/c.dag`, with every file brought into line in one sweep. A file outside the layout is not compilable.
- In the same PR: fix bare references in every cut-over route, switch v2's acquisition to loading by name on demand, and delete the three acquisition mechanisms that exist today. The seed keeps its whole-tree read until it retires.

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
- **R4.** The model does not assume one file per module. This storage realization does: every file-backed top-level module has exactly one `.dag` file; produced modules stay `ModuleStorageProvenance` `ProducedByBehavior` and get no source file; nested namespaces are contents of their owning file. Multi-file source modules are not realized by this cutover.
- **R5.** Storage rule: module `a.b.c` lives at `<root(a)>/b/c.dag`, where `root(v2) = src/v2` and `root(a) = dag/a` for every other top-level namespace. Paths follow names, never the reverse.
- **R6.** Modules stored where their name does not put them are fixed. Where a folder carries structure the name lacks, the folder goes into the name (`gunbc.x` in `dag/gunbc/host/` becomes `gunbc.host.x`).
- **R7.** The migration is one PR. The epochs of §5 are stages of one branch; none merges alone, and no intermediate state is an accepted repository state.
- **R8.** Work anchors on the v2 native process. v1 changes are limited to what the cutover mechanically requires.
- **R9.** The seed keeps its whole-tree read until it retires; no scratch-root bridge. In exchange, v2 is aggressive about what it consumes: a v2 compile reads, parses and retains only the modules it requests, builds no whole-tree index on any path (success or refusal), and its read set is checked against the requested closure exactly.
- **R10.** `tools.*` is renamed to `gunbc.instruments.*`; any member that is not an instrument stops the sweep for a decision.
- **R11.** The binding-repair population is every module reached by any compile route cut over to load by name, not only the two self-host closures. C0 measures it and the plan's executor states its size before C2.

Decisions recorded with the rulings (no open decisions remain):

| | Decision |
|---|---|
| Layout grain | Namespace rows only: one row per namespace that directly contains modules; parents derived. No per-module rows. |
| `T?` constructors | No implicit scope; `Present`/`Absent` require `import std.optional`. |
| Other bare builtins | Import their current declaring module. Not combined with the unfinished `filter`/`any` retirement. |
| Seed mirrors | Renamed with their modules; no old-name aliases. |
| `product.*` | Files move to `dag/product/`; names unchanged. |
| Parse-all | Kept, explicit, required (a step on `witnesses`, not a job), streaming: one module at a time, verdict only, no index. |
| Failure-path hint | The repository-wide "declared in X" lookup is deleted from compilation. |
| Fixtures | No exemption kind. A fixture that is a module obeys the rule; malformed non-module bytes live outside governed roots. |
| `module` line removal | Happens with seed retirement; not a phase of this plan. |

## 4. Target design

### 4.1 Layers

- **Storage layer.** The only code that knows paths. Given a module name it returns the module's file, or a typed refusal; given a namespace it enumerates its modules. It owns the layout, the rule, the reads and their verification.
- **Compiler.** Works with module names only. It starts from the entry module and requests every module its imports and qualified references name. The closure is whatever was requested.
- **Whole-population jobs** (layout check, parse-all, run every claim): explicit enumerations of the layout, never a side effect of compiling one entry.

### 4.2 The layout authority

- **Generic, in `v2.compiler.source_authority`:** layout types, the rule, file-backed lookup (`module_file(name)`), namespace enumeration, the layout check. It is expressed over a logical `SourceRootRef` and a relative path, never a host path.
- **Repository policy, in a new import-free module** (`gunbc.source_layout`): the namespace-root map and the namespace rows, each `Authored` or `Generated`. It is supplied at the entry boundary, not imported by the generic compiler.
- **Runtime binding:** the native CLI binds each `SourceRootRef` to the host root it is given, through the filesystem handler it already binds. No new Rust harness.
- **Generated files** stay in `gunbc.generated_artifact_registry`. The check joins both ways: every `.dag` in a `Generated` namespace is registered, every registered `.dag` artifact exists at its rule position, and no file is both authored and generated.

Lookup chain: module name → `SourceRootRef` + relative path → bound host root → exact read → `module` line agrees with the name → `SourceRef`.

### 4.3 Lookup by name

A qualified reference is split into module and member by probing prefixes, longest first. Each probe is tri-state: `ModuleFilePresent`, `ModuleFileAbsent`, `ModuleFileRefused` (unreadable, wrong kind, transport failure). Only `Absent` falls back to a shorter prefix; `Refused` refuses the reference. No prefix present is an unknown module.

Cross-tree direction is preserved at this layer: today's grounding (`v2.std.cross_tree.import_model`, `tree_fundamentality_order`; v2→dag admitted, dag→v2 denied) moves from the whole-tree ingest into the storage lookup, so loading by name cannot lose it.

### 4.4 The layout check

Run by the binary the `v2-native-cli` step builds, as a step on the required `witnesses` job. It has one mode: hold or refuse. Refusals are typed and located: `UndeclaredNamespace`, `NotAtRulePosition`, `ModuleLineMissing`, `DuplicateModule` (including case-fold duplicates), `RegistryArtifactMissing`, `GeneratedFileUnregistered`. It reads headers, not bodies.

### 4.5 Closure by demand

Entry module → parse → requested names → `module_file` → parse → repeat. This is the "parse-then-derive" mechanism of `docs/plans/namespace-cut-replacement-plan.md`. A bare name is not requestable and refuses under R1; the diagnostic names the spelling, the local module and its visible imports, and nothing scans the repository. The ingest receipt asserts files read = requested closure exactly.

### 4.6 The one dual representation that remains

The `module` line, read by the seed. It stays until the seed retires, written by the skeleton generator and checked against the rule.

### 4.7 Out of scope

Changing v1's resolver; the July design's markdown and Rust write-back; non-`.dag` files beyond the registry; the seed's memory (R9; gunb-ai/gunbc#13674 is independent work); deleting the `module` line.

## 5. Plan: one branch, one PR

**The epochs below are implementation stages within one cutover branch and one PR. They do not land independently, and no state between them is an accepted repository state. The only mergeable head contains the final layout, repaired v2 bindings, demand acquisition on every v2 compile route, and deletion of every superseded acquisition route.** The move map is a construction input used on the branch and after merging main; main receives only the refusing checks.

### C0 — Authority and migration generator

- Record R1–R11 and the decision table in a typed carrier (DESIGN §4c); add the `gunbc.plan_governance` row (governed by `v2-compiler-architecture`, role `CompilerStageAuthority`) and regenerate banners.
- Amend DESIGN §3 through `gunbc.design_document`: the storage rule is §3's single authority for the name⇄path mapping. Acyclicity stays the import graph's only structural law; the rule fixes where a file is stored, not which way imports point.
- Add the generic layout and exact-read types, and `gunbc.source_layout` declaring the **target** layout.
- Build the deterministic move/rename generator (extending `tools.source_reference_repoint`, run from the pre-sweep tree). It refuses the whole sweep on any collision — two sources to one target, a target that exists and is not moving away, case-fold equal targets, or a new module name already declared — and on any occurrence it cannot classify. It distinguishes typed module references from real file paths; a `.dag`-looking string is not automatically a module reference.
- Measure the R11 population: the closures of every compile route being cut over.

### C1 — Generated repository sweep

One generated commit, re-runnable after merging main:

- module renames: imports, qualified names, `DeclarationRef`s, labels (`//v2/test/...`), typed module identities, prose citations in `.dag`, Rust and `docs/`;
- file moves to the rule's positions;
- load-bearing paths repointed to module references through the layout; seed CLI arguments and constants (`NATIVE_COMPILE_ENTRY`, `REPORT_ENTRY`) rewritten;
- the ten seed mirrors renamed with their Rust paths;
- workflows, `.gitattributes` and every registered generated artifact regenerated (no CI step compares these since `gunbc.rung_drop` `v1_required_lanes_withdrawn`; the PR says so).

### C2 — Binding repair

- Run the real v2 resolver over the R11 population; add imports or qualification until zero unbound and zero ambiguous names remain.
- Delete the inert debt roster `v2.workflow.floor_unimported_bare_provider_debt_roster`, its gate, and the seed's `unimported_bare_provider_gate`, once the required native steps enforce R1.

### C3 — Acquisition replaced at the root

- Entry APIs take root bindings and an entry module, not a whole-tree `SourceRootIngest`. CLI `emit`, the native lane and `v2.compiler.program_assembly` callers all load by request.
- Deleted: the parse-everything phase of `closure_ingest`; `native_lane_source_facts` and `native_lane_closure_modules`; compile-time use of `source_root_read`'s recursive walk; `module_storage_bindings_for_source_roots` and its host manifest emitter (consumers enumerated by name first); the failure-path reverse lookup. No selector, no fallback.
- Inside the branch only: a differential of old reference-derived closure vs new demand-derived closure on both real entries, run before the old route is deleted.

### C4 — Final adjudication (the PR head must show)

1. Every governed `.dag` file is at its rule position; no exception roster exists.
2. Explicit population parse holds; a malformed module outside a compile closure fails it but changes neither that compile's verdict nor its read set.
3. Every R11 route resolves with zero unbound or ambiguous names.
4. Compile reads equal the requested closure exactly; no whole-tree index is built on any v2 path, including refusals (R9).
5. Probe controls: both `a.b` and `a.b.c` present → longest wins; `a.b.c` absent → `a.b`; `a.b.c` unreadable → refuse; none → unknown module; case-fold collision refuses; `module` line disagreeing with the request refuses.
6. Cross-tree control: a dag→v2 import still refuses through the storage lookup.
7. Seed emission of both self-host closures is identical before and after the sweep, modulo the rename map (v1 binds bare names by ancestor chain, which renames move).
8. The seed and all renamed mirrors build; both native entry closures emit and build.
9. The migration generator and every artifact generator are idempotent on the final head.
10. Deleted producers have no consumers; no compiler API takes the whole-repository ingest.

## 6. Risks

- **Conflicts.** The PR touches thousands of files; conflicts are resolved by re-running C1 after merging main, not by hand.
- **Strings outside the generator's classification** refuse the sweep rather than going stale.
- **Seed edits** (path constants, ten mirrors) are admitted under `gunbc.v1_maintenance_standing` `v1_seed_standing`; nothing else in v1 changes.
- **Outside consumers.** gunbc-private imports gunbc modules through its composed root and breaks on the renames; census it and land its repoint alongside. Census other external references (roadmap rows, saved links).
- **R11's size** may be most of the tree; that is the ruling, not a reason to narrow it.

## 7. Relationship to existing plans and authorities

- `docs/plans/namespace-resolution-design.md` — §10 (a module's position is declared, never inferred from storage; the `module` header is placement sugar that ends as a projection) is satisfied by declaring the rule; §13's unique-on-chain resolution is R1.
- `docs/plans/namespace-cut-replacement-plan.md` — its "parse-then-derive" closure mechanism is §4.5; stage XL-3 overlaps C2.
- `docs/plans/demand-engine-program.md` — §1's unruled next derivation ("an authority cheaper than full ingestion … a source-root manifest … the module dependency closure of the requested targets") and M3a's context-assembly note are C3.
- `docs/plans/replacement-migration-doctrine.md` — the whole PR is one replacement migration; it deletes what it replaces.
- gunb-ai/gunbc#13674 — independent of this plan (R9).
- `gunbc.generated_artifact_registry` — unchanged; consulted by the check.

---

## Appendix A — Measurements (one-off, at `e3939ea`)

Method: a script reading each `.dag` file's `module` line under `dag/` and `src/v2/` and comparing the file's path with R5's rule. Deleted by the cutover PR.

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
| `tools` @ `dag/gunbc/instruments/` | 93 | rename to `gunbc.instruments.*` (R10) |
| `v2.test.long` @ `src/v2/test/claim/long/` | 57 | rename to `v2.test.claim.long.*` |
| `v2.test.execution` @ `src/v2/test/claim/execution/` | 54 | rename to `v2.test.claim.execution.*` |
| `v2.test.emit` @ `src/v2/test/claim/emit/` | 53 | rename to `v2.test.claim.emit.*` |
| `product.printed_chassis` @ `dag/gunbc/product/printed_chassis/` | 52 | move to `dag/product/printed_chassis/` |
| `product` @ `dag/gunbc/product/` | 36 | move to `dag/product/` |
| `test.claim` @ `src/v2/test/claim/parse/` | 26 | rename to `v2.test.claim.parse.*` |

Smaller groups follow the same reading; for example `gunbc.test.*` and `dag.test.*` modules under `dag/test/` are renamed to `test.*`, joining their 2,100-odd `test.claim.*` siblings. The C0 generator computes the complete table; collisions refuse the whole sweep (C0). Anything not covered by A–C or by a confirmed group outcome stops the sweep for a decision; it is never guessed.

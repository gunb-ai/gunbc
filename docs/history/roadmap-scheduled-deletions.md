## Scheduled deletions — scaffolds with named dissolution triggers

**Discipline:** every scaffold in the live substrate lives here with an explicit dissolution trigger, the upstream work it's blocked on, and its enforcement path. When the trigger fires, a PR deletes the scaffold AND removes the row. Unscheduled scaffolds are violations of the scaffold-boundary invariant.

**Relationship to "Active deferrals":** deferrals name work that is in flight; scheduled deletions name artifacts that will disappear. A deferral may REFERENCE a scheduled deletion (e.g., "this sub-stage dissolves `ArrowBody::Pending` per the scheduled-deletions row"), and a scheduled deletion may reference a design blocker (DB-NN) that unlocks structural enforcement.

### Enforcement paths — three kinds

Each scheduled deletion names one enforcement path. **Grep over source code is not an enforcement path** — see §"Grep is not an enforcement path" below.

1. **Structural lens (preferred).** The substrate already carries the fact; a lens walks user DAGs and reports instances. Example: `ArrowBody::Pending` is a substrate variant; a lens walks `d.nodes` and fires on any `Pending` body reachable from user-range roots. Fits the existing `lens_unused_parameters.dag` / `lens_provenance.dag` shape. Writable today.

2. **Needs substrate amendment (DB-NN).** The substrate does not yet expose the fact the lens would need. A design blocker proposes the amendment; the lens becomes writable after the DB lands. The scheduled-deletion row carries a `Needs DB-NN` enforcement marker until the DB lands, then flips to a live lens path.

3. **Compiler-source ratchet (temporary).** For scaffolds that live in the hand-written Rust compiler and can't be lensed until `compiler.dag` self-hosts, a narrow source-level ratchet scoped to `src/v3/compiler/` is acceptable as temporary enforcement. Dissolves automatically when `compiler.dag` self-hosts and the same lens can walk the compiler's own DAG. The row explicitly marks this as temporary.

### Table

| Scaffold | Dissolution trigger | Upstream blocker | Enforcement |
|---|---|---|---|
| `ArrowBody::Pending` | M3 ratchet | Every realization arrow bound to `ExternalRealization` | **Lens** — writable now; walk `d.nodes` for Arrow declarations with `body = Pending` reachable from user-range roots |
| `ArrowBody::Unparsed` (**case 1** — `FnExternalBody` parse lag in std/) | M2+ parser surface | Match / pipe / lambda / block-body parsing so `FnExternalBody` lowers away | **Lens** — user-range + applicable std/ per R14; ratchet fires when block bodies become `SurfaceExpr` |
| `ArrowBody::Unparsed` (**case 2c** — `pipeline.dag` `fn compile` ordering text) | Structural pipeline-order carrier | First-class ordered stage list (or successor substrate) supersedes `compile` body-span parsing | **Not** the M2 grammar milestone — dissolution per [design-fn-external-body-reconciliation.md](docs/design-fn-external-body-reconciliation.md) case 2c; `pipeline_compile_order_stage_names` is the reader today |
| `ArrowBody::Unparsed` (**DB-14 accessor interim**, pre–E-9) | E-9 bootstrap materialization | **Deferral: E-9 substrate accessor bootstrap rewrite** below — `ExternalRealization(marker)` on `Arrow.body` | Clears with that deferral (not the case-1 lens) |
| `ValueBody::Unparsed` | M2+ parser surface | Record / map / list literal parsing | **Lens** — writable now; walk `data` declarations |
| `TransformTarget::Operator` | M2+ parser surface | Operator desugar into algebra-field calls | **Lens** — writable now; walk Transform targets |
| `LogicalOp` / `OperatorKind::Logical` | M2+ parser surface + Bool operator grounding | Logical operators lower to direct `BooleanAlgebra.{meet,join}` declaration refs (or equivalent explicit callable lowering) instead of a dedicated operator-family shim | **Compiler-source / substrate receipt** — keep the adjacent scaffold ledger on `dag.rs` + `substrate.dag` carriers until the trigger lands |
| User-range `ResolvedByName` AtomPayload (DB-17 new variant — post-landing, any user-range reference produced via name fallback rather than structural walk) | M2 module scoping | Cross-module structural resolution | **Live substrate consumer** — `lens_structural_resolution::name_keyed_references` walks declaration atoms for `ResolvedByName`; user/bootstrap boundary remains caller policy until that fact is reflected, and M2 module scoping still dissolves the variant entirely |
| Compiler-internal `declaration_by_name` call sites (bootstrap `substrate_markers` initialization in `dag.rs:1616+`, pipeline-authority wiring in `bootstrap.rs`/`pipeline_authority.rs`, emitter algebra lookups in `emit_go.rs`/`emit_python.rs`) | Self-hosting (most cases) OR specific per-site substrate amendments (e.g., substrate_markers becoming typed edges) | Depends on class — self-hosting for emitter/pipeline sites, specific substrate amendment for marker caches | **Compiler-source ratchet** (temporary, dissolves at self-hosting for most sites) — these are compiler-internal caches/wiring, NOT user-range resolution fallbacks; **DB-17 does not cover them** |
| `Node.name` field (v3 substrate) | **Closed in code** — direct reads migrated; stale docs can be deleted opportunistically | — | **Docs cleanup only** |
| `encoding_meet` / `encoding_join` (Rust fns) | Track 8 Phase 2 (user-defined generic emission) | User-defined generic emission for `Lattice<Encoding>` instance | **Compiler-source ratchet** (temporary; becomes lens-able when compiler.dag self-hosts and emission-generated code replaces these hand-written fns) |

### Notes on specific rows

- **`ArrowBody::Pending` lens-name-filter follow-up (#518) is cleared on PR #548.** The remaining anonymous operator-fallback site was reclassified to `ArrowBody::NoBody`, and `lens_structural_resolution` now keys on surviving `Arrow(Pending)` structurally rather than through a `Declaration.name` proxy. The scheduled-deletions row above stays live until the variant itself is removed; this follow-up only cleared the false-positive/false-negative gap around anonymous arrows.
- **`ArrowBody::Unparsed` is three dissolution stories, not one.** DB-16 / PR #524: **case 1** (parse lag, M2 grammar, lens ratchet) is separate from **`pipeline.dag` `compile` (case 2c)** — ordering text read by `pipeline_authority` until a structural pipeline-order fact supersedes span extraction — and from **DB-14 accessors** (interim `Unparsed` until the **E-9** deferral lands). The M2 milestone deletes case-1 uses; it does **not** by itself delete `compile`’s span authority or accessor interim encoding.
- **`declaration_by_name` is a helper name, not a single debt class.** The function at `dag.rs:1459` has 83 call sites that split into distinct classes with separate dissolution paths. [DB-17 (reference-resolution provenance)](docs/design-reference-resolution-provenance.md) narrows its scope to **only the user-range AtomPayload fallback class** (lowering produces `ResolvedByName(id)` when a structural walk falls back to name lookup). DB-17's lens walks user-range AtomPayloads; compiler-internal call sites (bootstrap substrate_markers in `dag.rs`, pipeline authority wiring, emitter algebra lookups in `emit_go.rs`/`emit_python.rs`) are a separate compiler-source class that dissolves at self-hosting (or via per-site substrate amendments — e.g., substrate_markers becoming typed edges rather than name-keyed caches). Keying the scheduled deletion to the helper name conflates these.
- **`Node.name` cluster**: closed in the current v3 compiler/substrate. The old generic `Node.name` carrier is gone; surviving `name` fields are declaration names and `BindNode.name`, which are distinct facts. Some older design docs still describe the deletion as in-flight; treat those as stale prose, not live debt.
- **`keyword_to_name` (recon outcome 2026-04-17, no row added):** the bare `keyword_to_name` was renamed to `tok_keyword_to_name` during v2 Phase 0 parser restructure — see `src/v2/parser-design.md:403-408`. The new name still carries the scaffold (parser-side keyword-name logic that duplicates facts from the tokenizer's `SyntaxSpec`), but it lives in `src/v2/02_parse.dag:455` and `src/v2/stage0/src/v2_compiler_parse.rs:1321` — **v2 code**, not v3. Grep confirms zero equivalents in `src/v3/`. V2 is the reference-implementation / test oracle per `ROADMAP.md` §"Sketch vs Oracle framing"; v2 scaffolds dissolve when v3 supersedes v2 entirely, not individually. The v3 Scheduled Deletions list tracks v3-scope scaffolds only.

### Grep is not an enforcement path

Per the compiler-as-dependency-analyzer framing: grep over source text cannot distinguish a real user-range violation from a comment, test fixture, bootstrap path, alias, helper indirection, or trait-dispatched call. It matches strings; the compiler analyzes a graph. Using grep to enforce "the system should be structural" uses a heuristic to enforce the ban on heuristics — the discipline defeats itself on the first move.

Every time a grep gate is proposed over source code, the correct question is: **what substrate fact would make this a lens?** That fact might need a DB; if so, the grep is a signal for the DB, not a substitute for it.

**Narrow exception:** the banked-dissolutions ratchet in `docs/post-l15-phase-plan.md` operates on *documentation text* (lane docs can't restate DB-rejected shapes), not on system behavior. Docs don't have a resolved DAG; they're text. Grep over docs for design-consistency is legitimate. System-level scaffolds get structural enforcement.

### How the scheduled-deletions discipline works

1. **Adding a scaffold.** The PR that introduces a scaffold opens a row here with: scaffold name (file:line or type path), dissolution trigger, upstream blocker, enforcement path (one of the three kinds above).
2. **Enforcement-path classification happens in the same PR.** Structural lens → write the lens or file `lens_TBD` naming the fact to query. Needs DB → file the DB design doc (or reference an existing one). Compiler-source ratchet → explicit; dissolves at self-hosting.
3. **Dissolution.** When the trigger fires, a PR deletes the scaffold AND removes the row. No lingering row after deletion; audit-traceable via git history.
4. **Reviewer gate.** A PR that introduces a scaffold without a row here — or with an enforcement path classified as "grep source code" — is blocked.


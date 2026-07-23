# Live-read witness classification — closing the masking-class root

> **Status:** P0–P2 landed (2026-07-14) — `v2.lens.live_read_classification` + fixture witnesses green by execution; P1 G2 call-reachability lens; P2 floor axis (iv) `runtime_data_dependency_touched` (#6630) with **G1-only module-closure interim** in `cli_run.rs` (`runtime_data_dependency_touched_via_carrier_closure`). **P3 (OFFLINE re-enrollment) remains open** — §8a/§8b require G2 path-intersection wired into floor admission plus carrier memoization before exclusion rows dissolve; G1-only axis (iv) over-selects on carrier-importing entries and would worsen CI cost vs the OFFLINE ruling. Model-before-implement authority for how runtime tree/host reads become visible to affected-set attribution. Companion to [affected-set precompute pruning](affected-set-precompute-pruning.md), [witness cost locality](../src/v2/lens/witness_cost_locality.dag) (`v2.lens.witness_cost_locality`), and [v1 run-stability throughline](v1-run-stability-throughline.md) §1.

---

## 1. The displaced cost (why this design)

Per-PR floor selection shrinks the witness corpus to the diff's affected set. That only earns trust when **every input a witness actually reads** is an edge the selection machinery can see. Today it cannot see runtime reads: a witness entry may declare `SubstrateInputsOnly`, sit outside the import closure of a changed file, and still **cold-scan the whole dag entry tree** or **filesystem_read arbitrary paths** at eval time. Selection predict-skips; the witness goes red on the next cold sweep — or worse, stays green until a human notices.

The cost is not "a few missed witnesses." It is:

- **Safety:** corpus-denominated reds that never fire on the PR that caused them (#6530, #6533 receipts below).
- **Cost:** witnesses that *should* skip on roster-target-only diffs cannot, and witnesses that *should* run on carrier-module diffs are selected intermittently — the `accumulator_copy_roster_gate` and `lever_a_local_receipt_witness` EX-notes document both directions of the same structural hole.
- **Complexity:** hand-maintained exclusion rows (`gunbc.ci_layer_roots` `witness_exclusion_substrings`) and operator-ruled OFFLINE witnesses are the parallel ledger this design dissolves.

---

## 2. The masking class — named, receipted

**Definition (masking class):** a witness whose evaluation reaches the live host tree or corpus **as data at runtime** through a carrier (`filesystem_read`, `module_declaration_facts`, or the `decl_facts` reflection family), where that reach is **not an import edge** and therefore **invisible to import-closure attribution**.

The falsifier cold sweep restored by M1 (#6528) immediately caught three latent corpus reds that per-PR selection had never run for the diffs that broke them — the lane's premise, receipted in [v1-run-stability-throughline.md](v1-run-stability-throughline.md) §1:

| Receipt | What broke | Why selection missed it |
|---------|------------|-------------------------|
| **#6530 (×2)** | (1) #6520 fold-API dissolution left 3 manual body-lowering claim modules unresolvable; (2) #6506 added `git.Core.Toplevel` to the git mock corpus without extending the totality consumer | Diffs touched modules outside those witnesses' import closures; no runtime-read edge, but also no import-closure path — **missing selection edge** class |
| **#6533** | `frontier_blocker_class_matches` in `self_host/frontier_probe_types.dag` landed unrostered — whole-corpus red on the next cold sweep | Wave 2 frontier probe PR's diff did not enroll the nfr roster witness; selection gap, **third instance of the masking class** (comment at `cli_run.rs` NON_FOLD_RESIDUE_ROSTER backfill) |

These are not the same failure mode as a lying `live_tree_disposition` declaration (that is the falsifier's lying-declaration divergence class). They are **honest witnesses whose runtime data dependencies are structurally absent from the selection graph**.

### 2a. The two OFFLINE witnesses (EX-note dissolution targets)

Both are operator-ruled OFFLINE in `gunbc.ci_layer_roots.dag` with identical structural cause and paired dissolve-on triggers:

| Witness | Runtime read | Selection failure mode |
|---------|--------------|------------------------|
| `accumulator_copy_roster_gate*_test.dag` | `roster_gate.dag:ingest_findings` → `filesystem_read(path)` per roster target | Intermittently selected on lens/std-closure PRs (cost); **never fires** on roster-target-only PRs (no enforcement) |
| `lever_a_local_receipt_witness_test.dag` | `witness_touched_path_dispositions_hold` → `compile_clean_shard_entry_paths()` → `module_declaration_facts` per scoped row | Selected after unrelated `cli_run.dag` PRs (85m+ timeout / OOM); skip-before-resolve cannot bound cost honestly |

**Shared dissolve-on (from EX-notes):** live-read classification **walks the import closure** for carrier modules; plus carrier-specific memoization (`resolved-graph memo` for roster gate; `single-roster memo` for lever_a). Re-enrollment is conscious through the floor owner's lane after both land.

---

## 3. What exists today — and the blind spot each layer leaves

### 3a. `LiveTreeDisposition` (entry-file declaration)

Authority: `v2.std.live_tree` — each witness entry declares `data live_tree_disposition: LiveTreeDisposition = ReadsLiveTree | SubstrateInputsOnly`; undeclared = `ReadsLiveTree` = never predict-skip.

This closed the **host-scaffold text classifier** fork (#6224/#6479) but at **entry grain only**:

- A declaration asserts the whole entry's input domain, not *which* carrier or *which* paths.
- Machine-stamped `SubstrateInputsOnly` rows inherit the deleted entry-text classifier's blind spot (`live_tree_disposition_stamp_provenance` in `v2.std.live_tree`): a live read hidden behind an import was invisible to the stamp.
- Enforcement for lying declarations = nightly falsifier; **no structural check** that declaration matches call reachability.

### 3b. Import-closure `entry_file_touched` (module-graph grain)

Authority: `v2.lens.module_graph.entry_affected_by_touched_paths` / host twin `entry_file_touched_via_import_closure` in `cli_run.rs`.

This axis answers: "did any touched **file** fall inside this entry's **import** closure?" It does **not** answer: "does this entry's **evaluation** read touched files via `filesystem_read` or `module_declaration_facts`?"

### 3c. Node-frontier selection (#6543)

`overlapping_data_items` + `rerun_frontier_nodes_for_entry` fires when a **`data` decl** referenced by the witness changes. This covers **static** data-item references discovered from resolved graphs. It does **not** cover:

- Dynamic path reads (`filesystem_read(path: roster_target)` where `path` is a parameter).
- Whole-corpus scans (`module_declaration_facts(pool_roots)` returning every module).

### 3d. `witness_cost_locality` (ambient-read carrier roster)

Authority: `v2.lens.witness_cost_locality` — module-grain import-closure BFS over `ambient_read_carrier_modules_v0` + pipeline stages.

**Already does the import-closure walk** for carrier *modules* (the `witness_cost_locality_from_facts` / `import_closure` fixpoint). Census 2026-07-11 (732-entry roster): falsified `enforcement_live_witness_test` as Local because corpus read flows through `v2.std.decl_index.decl_facts` two hops away — proving the transitive case matters.

**Gaps (documented in `witness_cost_locality_precision_frontier`):**

1. Module grain over-approximates (types-only imports flag as carriers).
2. Roster is hand-maintained grep receipt, not derived from call reachability.
3. Verdict is validation-tier — does not gate floor admission yet.
4. Does not add **data-dependency edges** from runtime reads to specific paths/modules.

---

## 4. The law

> **A witness's selection-eligible input domain = its import closure ∪ its proven runtime-read closure.**

Where **proven runtime-read closure** is the fixpoint over:

1. **Carrier reachability** — import-closure walk from the entry module hits a declared live-read carrier module.
2. **Carrier invocation shape** — for each reached carrier, the classified call pattern determines what host state is read (corpus-wide scan vs bounded path set).

Fail-closed defaults:

- Carrier reachability without resolvable invocation shape → `ReadsLiveTree` scheduling class (never predict-skip), same as undeclared disposition.
- Provenance gap on facts needed to classify → typed `Refused`, never widen to run-all or narrow to skip.

This is the same §5 discipline as `entry_file_touched_via_import_closure`'s refuse-when-entry-absent-from-facts arm — applied to the data-dependency half selection is missing.

---

## 5. Carrier taxonomy (single authority)

One closed coproduct; no per-witness bespoke grep.

```text
LiveReadCarrier
  = FilesystemReadPath { path_pattern: PathPattern }     // static or roster-derived
  | ModuleDeclarationFactsScan { pool_roots: PoolRoots } // whole entry-tree walk
  | DeclFactsReflection { home: ModulePath }             // v2.std.decl_index / decl_facts family
```

**Declared carrier home modules** (v0 roster — evolves into derived registry):

| Carrier variant | Authority module(s) | v0 roster row |
|-----------------|---------------------|---------------|
| `FilesystemReadPath` | any module calling `filesystem_read` | per grep receipt + `v2.lens.complexity_accumulator_copy.roster_gate` |
| `ModuleDeclarationFactsScan` | `v2.lens.module_graph`, `tools.dag_compile_clean_shard_roster`, `dag/tools/dag_compile_clean_scope` | `module_declaration_facts` / `module_declaration_facts_live` consumers |
| `DeclFactsReflection` | `v2.std.decl_index` | `decl_facts` reflection home |

The roster **must not** be a second parallel list beside `witness_cost_locality.ambient_read_carrier_modules_v0` at steady state — this design subsumes that list into `LiveReadCarrier` rows with invocation shape, and `witness_cost_locality` becomes a projection (DataBreadth evidence) of the classification verdict.

---

## 6. Classification grades (precision ladder)

Mirrors `live_tree_disposition_stamp_provenance`'s dissolve-at chain:

| Grade | Input | Output | Consumer |
|-------|-------|--------|----------|
| **G0 — entry declaration** | `data live_tree_disposition` row | `ReadsLiveTree` / `SubstrateInputsOnly` | Floor never-skip policy (live today) |
| **G1 — module closure** | Import-closure BFS + carrier roster | `CouplesToAmbient` / `Local` (cost locality) | Receipt + scheduled-lane routing (live today, validation-tier) |
| **G2 — call reachability** | fn-arrow `DependencyView` over lowered bodies | per-fn carrier invocations with path patterns | **This design's implementation target** |
| **G3 — path envelope** | evaluated path arguments / roster literals | `BoundedPathSet` / `CorpusScan` / `Unknown` | InputEnvelope + skip-before-resolve cost bound |

**G2 is the masking-class fix:** an entry that imports `v2.lens.complexity_accumulator_copy.roster_gate` gets `FilesystemReadPath` edges for each roster literal even though the witness entry file contains no `filesystem_read` text.

**G3 is the lever_a / roster_gate cost fix:** `ModuleDeclarationFactsScan` classifies as `CorpusScan` → never predict-skip unless the witness declares `InputEnvelope = DeclaredCorpusInput` and routes to the scheduled lane per `witness_cost_locality.per_pr_admission_law`.

---

## 7. Selection integration — a fourth axis, not a fallback

Today `floor_witness_run_disposition` has three skip axes (`touches_frontier`, `function_edited`, `entry_file_touched`) plus the `reads_live_tree` never-skip overlay.

**Add axis (iv) `runtime_data_dependency_touched`:**

```text
RunWitness when:
  touches_frontier
  || function_edited
  || entry_file_touched
  || runtime_data_dependency_touched   // NEW
```

Where `runtime_data_dependency_touched` is true when any touched path intersects the witness entry's **G2 classified read closure** (path patterns, corpus-scan flag, or decl-facts home module file).

Relationship to existing axes:

- **Implies `reads_live_tree` scheduling** when classification is `CorpusScan` or envelope is `EnvelopeUndeclared` with carrier evidence — but does not replace the declaration (declaration remains the falsifier's lying-detection authority).
- **Strictens `entry_file_touched`** — import closure is necessary but not sufficient for hermetic witnesses; the runtime axis catches cross-file reads *behind* carrier imports.
- **Orthogonal to node-frontier** — node-frontier handles static `data` item graph references; this axis handles dynamic host reads the graph does not encode as nodes.

Precompute-skip guard (consumer 2 in [affected-set-precompute-pruning.md](affected-set-precompute-pruning.md)) must include axis (iv) in the "no witness will run" predicate before skipping whole-tree mock precompute.

---

## 8. Dissolution: OFFLINE witnesses and hand exclusion rows

### 8a. `accumulator_copy_roster_gate`

**Re-enrollment criteria (both required):**

1. G2 classification derives `FilesystemReadPath` edges for roster targets from `roster_gate.dag` through import closure.
2. Resolved-graph memoization receipt lands (duplicate-work thread) so per-target resolve is not serial corpus cost.

**Selection behavior after:** roster-target-only diff → axis (iv) fires → witness runs. Lens-module-only diff without roster target changes → honest skip.

### 8b. `lever_a_local_receipt_witness`

**Re-enrollment criteria (both required):**

1. G2 classification marks `module_declaration_facts` consumers as `CorpusScan` through import closure (including `dag/tools/dag_compile_clean_scope`).
2. Single-roster memo in `tools.dag_compile_clean_scope` lands (one `compile_clean_shard_entry_paths()` per disposition batch, not per row).

**Selection behavior after:** `cli_run.dag` PRs without shard-roster relevance → honest skip; shard roster or `dag_compile_clean_scope` changes → run.

### 8c. Hand exclusion rows

`gunbc.ci_layer_roots` `witness_exclusion_substrings` for these witnesses dissolve when G2+G3 are wired into floor admission — same trigger as `witness_cost_locality_precision_frontier` ("subsuming the hand exclusion rows and the entry-text live-scan classifier").

---

## 9. Implementation phases

Phases are ordered; each closes at a named consumer tier.

### P0 — Model carriers + G2 shape (substrate only)

- Add `LiveReadCarrier` + `LiveReadClassification` types to `v2.std.live_tree` (or adjacent single-authority module — not a second disposition type).
- Add `live_read_carriers_v0` registry rows with invocation-shape metadata (migrating `ambient_read_carrier_modules_v0`).
- Lens: `live_read_classification_from_facts(entry_module, facts) -> LiveReadClassification` using import closure + carrier registry (G1 superset of G2 module reachability).
- Witness: fixture proofs for the two-hop carrier case (enforcement_live pattern) and roster_gate pattern.

**Accept:** unit fixtures green; RED when carrier import edge removed from fixture facts.

### P1 — G2 call-reachability upgrade

- Feed fn-arrow `DependencyView` (`fn_arrow_decl_facts_live`) to resolve **which** carrier call sites execute (not just module imports).
- Derive `FilesystemReadPath` patterns from literal path arguments where statable; `Unknown` → fail-closed never-skip.
- Re-stamp machine `SubstrateInputsOnly` rows from G2 verdict; lying stamps become falsifier divergences.

**Accept:** `accumulator_copy_roster_gate` entry classifies as `FilesystemReadPath` with roster-derived patterns by execution; discriminating RED when import of `roster_gate` removed.

### P2 — Floor admission consumes axis (iv)

- Extend `affected_set_floor_runner.dag` + `cli_run.rs` discovery skip with `runtime_data_dependency_touched`.
- Wire into precompute-skip predicate (consumer 2).
- Promote `witness_cost_locality` from validation-tier to admission gate for `CorpusScan` without `DeclaredCorpusInput`.

**Accept:** harness on real merged diffs — roster-target touch selects roster gate witness; unrelated lens PR does not. Precompute-skip receipt on scoped diff.

### P3 — Re-enroll OFFLINE witnesses

**Blocked on:** G2 path-intersection (not G1 module-closure alone) wired into `cli_run.rs` floor skip + carrier memoization receipts (§8a resolved-graph memo; §8b single-roster memo). P2's production axis (iv) is the G1-only `runtime_data_dependency_touched_via_carrier_closure` interim — fail-closed-safe (never under-selects) but over-selects on carrier-importing witnesses, so re-enrollment before G2 wiring would run ~15m roster-gate / 85m+ lever_a witnesses on most `.dag`-touching PRs.

- Remove `witness_exclusion_substrings` rows for `accumulator_copy_roster_gate` and `lever_a_local_receipt_witness`.
- Restore discovery enrollment; verify CI floor receipts within step budget.
- Delete dissolve-on EX-note rows or mark dissolved.

**Accept:** both witnesses discovery-enrolled; falsifier 8-window divergence rate unchanged; lever_a local recipe still passes as offline fallback during ramp.

---

## 10. Acceptance witnesses (green-by-execution)

| ID | Proves | RED control |
|----|--------|-------------|
| **LR-W1** | Two-hop carrier import (`entry → mid → decl_facts home`) classifies as runtime read | Remove `mid` import edge → classification drops to Local → skip mispredict |
| **LR-W2** | `roster_gate` import classifies `FilesystemReadPath` for roster literals | Remove `roster_gate` import → roster-target diff does not select witness |
| **LR-W3** | `dag_compile_clean_scope` import classifies `ModuleDeclarationFactsScan` | Touch only `cli_run.rs` → lever_a witness does not run |
| **LR-W4** | Full floor harness: axis (iv) agrees with cold-run necessity on real diff corpus | Planted missing-edge diff → typed divergence, not silent skip |
| **LR-W5** | Re-enrolled roster gate: roster-target edit runs witness; std-only edit skips | Swap fixture paths in discrimination harness |

---

## 11. Relationship to sibling designs

| Doc / module | Relationship |
|--------------|--------------|
| [affected-set-precompute-pruning.md](affected-set-precompute-pruning.md) | Axis (iv) extends the three-axis skip model; precompute-skip guard gains a fourth conjunct |
| [affected-set-differential-falsifier.md](affected-set-differential-falsifier.md) | Falsifier remains the audit for lying declarations; this design removes the **structural** missing-edge class |
| [bounded-input-cost-envelope-scheduling.md](bounded-input-cost-envelope-scheduling.md) | `CorpusScan` → `DeclaredCorpusInput` → scheduled lane; never cost-driven selection |
| [v1-run-stability-throughline.md](v1-run-stability-throughline.md) | Falsifier restoration receipts (#6530) are the safety proof this design must not regress |
| `v2.lens.witness_cost_locality` | G1 module-closure layer; retires `ambient_read_carrier_modules_v0` as a hand roster when G2 lands |

---

## 12. Explicit non-goals (this capture)

- No new host builtins — classification reads existing `module_declaration_facts` / fn-arrow facts.
- No absorbing fallback — a classification `Unknown` refuses or never-skips; it does not run the whole corpus "to be safe."
- No replacement of `LiveTreeDisposition` declarations — G2 re-derives them; declarations stay the author-facing contract and falsifier lying-detection surface.
- No skip-before-resolve implementation in this lane — axis (iv) must be computable from facts **before** witness-entry resolve (same constraint as `entry_file_touched_via_import_closure`; roster memo is the cost enabler).
- No fn-grain LayerDepth gating changes — `witness_cost_locality` receipt-only LayerDepth rule stays until call-graph reachability is independently proven.

---

## 14. Hand-Rust scaffold receipt (P1 G2 marshal)

**Authority:** `v2.std.fn_index::fn_arrow_skeleton_g2_marshal_host_scaffold_dissolution_trigger`

**Seed-retained site:** `src/v1/stage0/src/coproduct_reflection.rs` — `marshal_string_literal_atom`, `hoist_call_arg_string_literal_edges`, and callee/literal atoms on `marshal_generic` for `ExprCall` nodes. These expand the fn-arrow skeleton corpus-wide so G2 can stat string-literal path arguments and callee chains from `fn_arrow_decl_facts_live()`.

**Dissolution trigger:** fn-arrow body projection becomes a modeled substrate fold (the #5364 self-host corridor for `eval_fn_arrow_decl_facts_live`) emitting equivalent skeleton `Node` facts without hand-Rust marshal expansion in `coproduct_reflection.rs`.

**Witness:** `v2.test.claim.live_read_classification_test::g2_fn_arrow_marshal_host_scaffold_receipt_is_checkable` (Scaffold disposition row is checkable by execution).

---

## 15. Open sub-threads

- **Path pattern static analysis depth** — how many `filesystem_read` path arguments are literal at G2 vs require `Unknown` never-skip.
- **Namespace-only resolution terminal step** — import edges become `container.member` references; carrier registry and G2 must be edge-source-agnostic (same constraint as `dependency_edge_source_migration_note` in `module_graph.dag`).
- **NON_FOLD_RESIDUE_ROSTER backfill class** — #6533's unrostered landing is a sibling symptom (roster not tied to selection); G2 does not replace nfr roster enrollment — track separately, but the masking-class diagnosis is shared.

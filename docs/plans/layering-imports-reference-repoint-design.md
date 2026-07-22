# Layering-imports CI gate — repoint onto reference-derived edges

**Status:** Phase 1 landed (transitional producer). Phases 2–3 remain open.

**Lane:** namespace-only resolution terminal step (import deletion) · CI floor `LayeringImportsGate`.

**Parent:** [namespace-only resolution design](namespace-resolution-design.md) §8 step 5 (delete `import` grammar) and §PR-5b (reference-derived dependency-edge lane).

**Priced failure:** §5 coverage-by-illusion — when `import` syntax is deleted, the gate's fact producer (`extract_import_paths` text scan in `cli_run.rs:layer_import_facts`) emits zero rows for every file. The lens verdict (`layering_imports_clean_holds`) trivially greens; upward `std → extdeps` and `* → v2.compiler` violations become invisible.

---

## 1. What the gate enforces today

The layer DAG (DESIGN §3): `std ← extdeps ← compiler ← workflow`; imports point toward `std`.

`v2.lens.layering_imports` is a pure verdict over `LayerImportFact` rows:

| field | meaning today |
| --- | --- |
| `layer` | `LayerPrefixStd` or `LayerPrefixExtdeps` — implied by **which scan root** contained the file |
| `path` | repo-relative `.dag` path of the **importer** |
| `import_module` | target module string from an `import …` line |

Violations (unchanged by this repoint):

1. **std → extdeps** — `is_std_layer(layer) && is_std_importing_extdeps(import_module)`
2. **any → compiler** — `is_forbidden_compiler_import(import_module)` (`v2.compiler` prefix)

Host producer: `layer_import_facts(std_roots, extdeps_roots)` walks each root's `.dag` files and calls `extract_import_paths` (substring scan). Registered as the `layer_import_facts` interpreter builtin; `layer_import_facts_live` is the witness-facing alias.

CI enrollment: `LayeringImportsGate` → `layering_imports_gate_passes` → `clean_tree_test.dag` + five scanner perturb receipts under `src/v2/test/claim/layering_imports/scanner/`.

---

## 2. Why import-syntax scanning cannot survive namespace terminal step

[namespace-resolution-design.md](namespace-resolution-design.md) Rule 1 end-state: the reference is the **sole** representation of usage; `import …` becomes a parse error (step 5). PR-5b already strips import lines from workflow/extdeps subtrees while the host keeps compile alive via `extend_with_bare_reference_closure` and `reference_resolution_facts`.

The reference-derived edge machinery is **landed** for module-graph selection:

- `reference_resolution_facts` — parse-front-end reference collection + resolution (`cli_run.rs:17808`)
- `reference_edges_as_import_facts(edges, strict)` — projects into the same row channel as import facts; `strict = true` drops `AmbiguousBare` (selection / hygiene tier; median closure 96 vs 1136 at `false`, measured 2026-07-14)
- Import-bearing files intentionally emit **no** reference edges (pass 2 skips them; import facts own those edges exactly)
- Import-less files fall back to reference edges only

`layer_import_facts` did **not** follow that repoint. It remains import-syntax-only. At terminal step the gate's subject disappears.

---

## 3. Target shape — one producer, two tiers, edge-source-agnostic lens

**Principle (§3 single authority):** `LayerImportFact` rows are **layer-scoped dependency edges** — the lens stays unchanged; only the host projection changes. Same contract already used by `build_module_graph_facts_live` selection adjacency and `floor_lens_hygiene` `path_imports` union.

### 3.1 Transitional producer (pre-import-deletion)

For each `.dag` under `std_roots ∪ extdeps_roots`:

```
if file has import lines:
  facts += import_syntax_facts(file)          // current behavior
else:
  facts += reference_strict_facts(file)       // reference_edges_as_import_facts(..., true)
```

Union is keyed by `(path, target_module)`; import-syntax wins on collision (should not arise — reference producer skips import-bearing files by construction).

### 3.2 Terminal producer (post-import-deletion)

```
facts = reference_strict_facts(all files in std_roots ∪ extdeps_roots)
```

Import-syntax arm deletes with the grammar.

### 3.3 Strict tier only

Layering is a **selection/hygiene** consumer (like affected-set selection and inert-lens reach), not a loader. The producer uses `reference_edges_as_import_facts(..., /* strict */ true)` exclusively. `AmbiguousBare` edges are dropped, not widened (§5: refusal, never absorb).

Loader-tier consumers (`extend_with_bare_reference_closure`, compile-clean import adjacency) stay on import-only or `strict = false` — **do not conflate tiers** (documented fork surface at `cli_run.rs:17611`).

---

## 4. Named model changes

### 4.1 Keep stable (lens / gate verdict)

| artifact | disposition |
| --- | --- |
| `v2.lens.layering_imports` verdict fns | **unchanged** — `is_layering_violation`, `layering_imports_clean_holds`, `layering_imports_violation_count` |
| `LayeringImportsGate` CI enrollment | **unchanged** — transport + scanner roster stay wired |
| Violation predicates | **unchanged** — std→extdeps, *→compiler |

### 4.2 Producer / fact projection (host seam — primary work)

| change | from | to |
| --- | --- | --- |
| `layer_import_facts()` edge source | `extract_import_paths` per file | transitional union per §3.1; terminal reference-only per §3.2 |
| `reference_resolution_facts` pool | (n/a) | **`witness_layer_roots` pool** + `std_roots ∪ extdeps_roots` importer scope — qualified targets like `v2.compiler.*` resolve against the full witness-layer census, same split as module-graph selection adjacency |
| `layer` field assignment | scan-root constant (`LAYER_STD` / `LAYER_EXTDEPS`) | **`layer_prefix_from_dotted_qualified_name(importer_module)`** — authority: `v2.std.cross_tree.resolution`; **host mirror** (`CLI_RUN_LAYER_PREFIX_FROM_DOTTED_MODULE_SCAFFOLD_MARKER`) until builtin routes through `.dag` |
| `import_module` field semantics | import-line target | **resolved target module** (reference or import — same string shape) |
| Importer module resolution | implicit from path prefix | `extract_module_path(content)` per file (already required by reference producer) |

**Not a new type:** `LayerImportFact` name is retained for consumer stability (`realization_vocabulary_containment`, `meta_exec_confinement`, lens_unit fixtures). Optional follow-on: rename field `import_module` → `target_module` when all call sites migrate — cosmetic, not blocking.

**Not a new builtin:** reuse `reference_resolution_facts` + `reference_edges_as_import_facts` inside `layer_import_facts`; no third host entry point.

### 4.3 `.dag` surface (minimal)

| artifact | change |
| --- | --- |
| `layer_import_facts_live` | doc/note only — documents transitional vs terminal producer contract |
| `construction_justification` on `v2.lens.layering_imports` | stays **`WallAfterGrounding { dissolves_to: SingleAuthority }`** through Phase 1–2 (transitional dual-arm producer); flips to **`WallNow`** at Phase 3 when import-syntax arm deletes |

### 4.4 Coupled consumers (same producer — migrate together)

These call `layer_import_facts_live` / `layer_import_facts` and inherit the repoint **for free** when the host builtin changes:

| consumer | risk |
| --- | --- |
| `v2.lens.realization_vocabulary_containment` | low — predicates filter by `path` + `import_module`; reference-sourced rows are the intended subject after PR-5b strip |
| `v2.lens.meta_exec_confinement` | low — same |
| `floor_lens_hygiene` `path_imports` | **already unions** `reference_edges_as_import_facts(..., true)` — should **deduplicate** against the updated `layer_import_facts` to avoid double-counting during transition; track in Phase 1 |

### 4.5 Witness / RED-control migration

| artifact | change |
| --- | --- |
| `src/v2/test/claim/layering_imports/clean_tree_test.dag` | stays — still calls `layer_import_facts_live`; greens when producer is correct |
| Scanner fixtures (`layering_scan/*/plant.dag`) | **Phase 2:** add **import-less** sibling fixtures per scenario (`import` line deleted; violation expressed as `container.member` reference). Keep import-syntax fixtures until terminal step (transitional producer must still detect them) |
| `coverage_domain_equivalence_test.dag` | extend rows for reference-sourced perturb receipts |
| `lens_unit/*_fixture.dag` | **unchanged** — synthetic `LayerImportFact` rows test verdict logic directly (class-a); independent of producer |

**Discriminating witness (required in Phase 1):** extend `cli_run.rs` `reference_only_edge_divergence_test` pattern — a fixture `std` module with **no** `import` line and a qualified `v2.compiler.*` reference must yield `layering_imports_violation_count == 1` through `layer_import_facts_live`. Proves the gate survives import deletion.

---

## 5. Phases (receipt + dissolution)

### Phase 0 — scoping (this document)

*Receipt:* operator sign-off on model changes §4 and phase boundaries.

*Dissolve-on:* Phase 1 PR opened.

### Phase 1 — repoint producer (transitional union)

1. Implement §3.1 in `layer_import_facts()` with `layer_prefix_from_dotted_qualified_name` for `layer`.
2. Add discriminating witness (§4.5).
3. Reconcile `floor_lens_hygiene` `path_imports` dedup.
4. Run clean tree + five scanner receipts green (import-syntax fixtures still valid).

*Receipt:* `layering_imports_gate_passes` green; new reference-only RED fixture reds on revert.

*Dissolve-on:* PR-5b import strip reaches `std`/`extdeps` subtrees (or first stripped module in gate scan roots).

### Phase 2 — reference-native perturb fixtures

1. For each of the five scanner scenarios, add an import-less `plant_ref.dag` twin.
2. Extend `coverage_domain_equivalence_test.dag` rows.
3. Scanner claims run both arms during transition; import-syntax arm deletes at terminal step.

*Receipt:* scanner detects violation on import-less fixture alone.

*Dissolve-on:* namespace terminal step (import grammar deleted).

### Phase 3 — terminal producer

1. Delete import-syntax arm from `layer_import_facts()`.
2. Delete import-syntax scanner fixtures and `extract_import_paths` call sites **in this producer only** (`extract_import_paths` remains elsewhere until full import deletion).
3. Flip `construction_justification` to `WallNow`.

*Receipt:* gate green on corpus with zero `import` lines in `dag/std`, `dag/extdeps`, `src/v2/std`, `src/v2/extdeps`.

*Dissolve-on:* `symbol_index_fill` exact reference edges land — then `reference_resolution_facts` parse approximation deletes per its existing `Dissolve-on` comment (`cli_run.rs:17580`). Layering gate inherits the swap via the same `layer_import_facts` seam (no second migration).

---

## 6. Non-goals (this lane)

- **Changing violation rules** — no new layers, no workflow-layer scanning (out of scope today and after repoint).
- **Symbol-index reference edges** — tracked on SymbolIndex lane; this design consumes whatever `layer_import_facts` is fed until then.
- **Self-applying lens retrofit** — `layering_imports` is C2 per [self-applying-lenses.md](self-applying-lenses.md); construction wall, not produce-and-apply.
- **Renaming `LayerImportFact` globally** — defer; field semantics broaden, name stays for churn control.
- **`strict = false` tier for layering** — refused; would reintroduce the 2026-07-14 selection blow-up.

---

## 7. Dependencies and sequencing

| dependency | relationship |
| --- | --- |
| `reference_resolution_facts` scaffold | **ready** — landed, interpreter-registered |
| `reference_edges_as_import_facts(strict)` | **ready** — tier contract documented |
| `layer_prefix_from_dotted_qualified_name` | **ready** — `v2.std.cross_tree.resolution` |
| PR-5b import-line strip | **parallel** — Phase 1 must land before strip reaches gate scan roots, or gate goes blind on those files |
| Namespace terminal step 5 | **gates** Phase 3 — import grammar deletion |
| `realization_vocabulary_containment` / `meta_exec_confinement` | **coupled** — inherit producer; no separate migration |

**Sequencing rule:** Phase 1 (transitional union) lands **before** import strip touches `std`/`extdeps` scan roots. Phase 3 lands **with** namespace terminal step, not before.

---

## 8. Open / to-verify

- **Cross-tree `dag/std` vs `src/v2/std` modules:** `layer_prefix_from_dotted_qualified_name` handles `v2.*` and bare `std`/`extdeps` prefixes — confirm no mis-layering on `dag/std/foo.dag` declaring `module std.bar` (if any exist).
- **Perf:** `layer_import_facts` currently cheap (text scan). Reference path parses each file — measure against 5s fast-lane budget (`gunbc_ci_fast_lane_rule_note`). If over budget, scope scan to gate roots only (already the case) and rely on `REFERENCE_EDGE_CACHE`.
- **Test modules in std roots:** fixtures under `src/v2/test/fixture/layering_scan/` use `v2.test.*` modules; `layer_prefix_from_qualified_name` maps `v2.test` → `LayerPrefixCompiler`. Confirm this does not exclude them from std-root scans incorrectly (today scan root assigns `LayerPrefixStd` regardless of module name — **behavior change** to verify).

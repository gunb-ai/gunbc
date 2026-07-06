# Compiler algorithm survey — raw stage catalogs (2026-07-04)

> **Superseded note (2026-07-06):** the `flatten_visible_bindings` and `merge_envs` rows below (verdict SUSPICIOUS/latent — "loaded gun") were **deleted outright in #6331** — one-level visibility by construction, so that gun is removed. This survey is preserved verbatim as a dated snapshot; its `04_env.dag` line anchors for those two functions predate the deletion.

**Provenance:** eight parallel read-only survey agents, one per stage group, each instructed with the resolver reference lesson (request-major redundancy + per-unit whole-env scan) and asked to catalog every pass with unit, asymptotic class in named scaling variables, memoization story, and a verdict (OPTIMAL / ACCEPTABLE / SUSPICIOUS / PATHOLOGICAL), citing file:line anchors. Run 2026-07-04 on branch `claude/ci-witness-opt-in-10h2he` at the post-engine-PR state (constructor-owner ruling landed, suite 656/0/85 green).

**Status of claims:** these are READ-verdicts — asymptotic classes argued from code with anchors, not yet measured. The four highest-leverage claims were spot-verified against the live tree by the synthesizer (see the roadmap's Survey results section). Every PATHOLOGICAL row still owes a corpus-scale measurement before its fix lands (roadmap §B method, step 3).

**Synthesis:** `docs/plans/post-engine-pr-roadmap.md` § Survey results. This file is the raw receipt.

---

## Parse / lex (src/v1/02_parse.dag + parse-table memo)

I now have the complete picture. Here is the audit catalog.

---

# gunbc parser audit — `src/v1/02_parse.dag` + parse-table memo machinery

**Scaling variables:** `N` = files/modules per run · `M` = tokens per module (∝ file size) · `A` = operators/keywords in the grammar spec (fixed constant) · `k` = lookahead span of a disambiguation scan.

**Headline:** the v1 parser is a clean recursive-descent + Pratt parser with *no* exponential backtracking — but it accesses its token stream through `skip(list, pos) |> first`, and the interpreter's `skip` **materializes the whole tail of the token list on every access**. That turns every peek into O(M) and the whole per-file parse into **O(M²)**. This is the classic "cursor over a list, re-scanned from the head each step" pathology, and it is almost certainly the "legacy CYK floor ≈ 63 s" cost the memo tests benchmark against. The cross-file *redundancy* axis, by contrast, is already correct: each file parses exactly once per run.

## Catalog

| Pass / algorithm | Anchor | Computes / over what unit | Asymptotic | Memo / sharing | Verdict | Fix direction |
|---|---|---|---|---|---|---|
| **TokenStream random access** (`token_stream_first` / `peek` / `advance`) | `02_parse.dag:79,82,86` → `skip` builtin `v1_interpreter.rs:3187`, `first` `:3172` | Reads token at current `pos`; `skip(all,pos) |> first` copies the entire `M−pos` tail into a fresh RRB vector then discards all but the front. Per token access. | **O(M−pos) per access → O(M²) per module** (time + allocation) | None. Recomputed from the head on *every* peek; `pos` is just an int carried in `TokenStream{all,pos}`. | **PATHOLOGICAL** | Index directly: `all.get(pos)` (RRB get is O(log M)), or carry a real cursor. Drops parse to O(M log M). |
| **Parse driver** (recursive-descent + Pratt): `parse_module`/`parse_items_acc`/`parse_expr_bp`/`parse_expr_loop` | `02_parse.dag:856, 916, 3254, 3261` | Walks tokens once, O(M) parse *steps*, no re-parse of alternatives (Pratt binding-power, commits as it goes). | O(M) steps — but each step ≥1 token access → **inherits O(M²)** | Structure is fine; it's the *victim* of the access cost above. | ACCEPTABLE structurally / PATHOLOGICAL in practice via access | Fix the accessor; driver needs no change. |
| **`scan_for_fat_arrow_after_braces`** (lambda-vs-record lookahead) | `02_parse.dag:3983`, called `:3957` | Scans forward token-by-token tracking brace depth to disambiguate. Per ambiguous `{`-construct. | O(k) steps × O(M−pos)/step → **up to O(M²) per site**, ×(sites/module) | None | **SUSPICIOUS→PATHOLOGICAL** (same root cause, compounds) | Same accessor fix makes each scan O(k). |
| **`infix_bp` / `find_operator_bp` / `find_operator_binop`** | `02_parse.dag:3326, 3316, 3333` | `filter`+`count`+`first` over `dag_syntax_spec.operators` per operator token to get binding power / BinOp. Per operator token. | O(A) per token, rebuilds a filtered list each call → O(A·M)/module, A constant | Recomputed per use-site; operator table is derivable-once & constant | SUSPICIOUS (minor; A small) | Precompute a `symbol → {bp, binop}` map once at grammar load; O(1) lookup. |
| **Item / import accumulation** (`parse_items_acc`, `parse_imports_acc`) | `02_parse.dag:916, 901`; `list_push` `v1_interpreter.rs:3067` over `List = Rc<RrbVector>` `:331` | `list_push(acc, item)` per item. | O(log items) per push (RRB clone O(1) + push_back O(log n)) → O(items·log items) | — | **ACCEPTABLE** (not the quadratic it would be on a `Vec`; RRB saves it) | none |
| **Per-file parse-cache** (`MultiEntryIndex.parse_cache`) | `cli_run.rs:954` field, `:1091` lookup, `:1105` parse, `:1108` insert; shared index `process_shared_index:902` | Parse result keyed by `source.path`; index is process-shared per `(thread, source_roots)`. | Each file parsed **once per run** regardless of how many entries include it | Memoized per path, shared across all entries of the union closure | **OPTIMAL** (redundancy axis handled — graph-major already) | none |
| **Cold front-end** (`front_end_sources`) | `v1_compiler_compile.rs:2217`, parse `:2246` | Pre-interns all files then folds parse over them; each file parsed once. | O(Σ M) — once per file, no re-parse | one-shot (no cross-call cache, but no redundancy either) | ACCEPTABLE | none |
| **ParseTable packrat memo intrinsic** (`parse_table_lookup`/`insert`) — the "parse-table memo machinery" | `v1_interpreter.rs:2492` dispatch, `:2460` key, `:650` struct | Memoizes parse result keyed `(grammar_digest, token_stream_digest, position, production)`. Used by the **v2** parser, *not* `02_parse.dag`. | Turns re-parse of identical (stream, position, production) into O(1) → linearizes v2 parse; amortizes across repeat ingest | Correct packrat scoping (per grammar × token stream) | **OPTIMAL** design | Minor: `keepalive` Vec (`:652`) pushes 3 clones per insert, never cleared — memory grows with memo size, not a time bug. |

## Three worst offenders (evidence)

1. **TokenStream access is O(M) per peek → O(M²) per module.** `src/v1/02_parse.dag:79` `token_stream_first = skip(xs: stream.all, n: stream.pos) |> first` and `:87` `token_stream_peek = skip(..., pos+offset) |> first`. The `skip` builtin at `src/v1/stage0/src/v1_interpreter.rs:3187` is `items.iter().skip(n).cloned().collect::<Vec<_>>()` — it clones and reallocates the entire `M−pos`-element tail on every call, then `first` (`:3172`) throws all but the head away. There are **263** `token_stream_*` call sites in the parser (grep count) and the driver takes O(M) steps, so total ≈ Σ_pos O(M−pos) = **O(M²)** time *and* allocation per file. This is the dominant per-unit pathology and the direct analogue of the reference lesson's flatten+scan-per-literal. Fix: index by position (`all.get(pos)`, RRB O(log M)) → O(M log M).

2. **`scan_for_fat_arrow_after_braces` compounds the same cost.** `src/v1/02_parse.dag:3983` disambiguates lambda-vs-record by walking the stream one `token_stream_advance`+`first` at a time — each hop is O(M−pos) because of offender #1, so a single k-token lookahead is O(k·M), and it fires once per ambiguous `{`. Same fix eliminates it (each hop → O(log M)).

3. **`find_operator_bp` / `find_operator_binop` / `infix_bp` re-filter the operator spec per token.** `src/v1/02_parse.dag:3316/3333/3326` run `ops |> filter(...) |> count/first` over `dag_syntax_spec.operators` for every operator token — a derivable-once, constant-size fact re-derived at each use site (O(A) per token, allocating a filtered list each time). Minor because A is a small grammar constant, but it's the textbook "per-use-site recomputation of a precompute-once index." Fix: build a `symbol → {bp, binop}` map once when the grammar spec loads.

**Redundancy axis is clean:** `parse_cache` (`cli_run.rs:954`, keyed by `source.path`) plus the process-shared `MultiEntryIndex` (`process_shared_index`, `cli_run.rs:902`) guarantee each file is parsed once per run across the whole union closure — no request-major re-parse. The v2 `ParseTable` packrat memo (`v1_interpreter.rs:2492`, keyed by grammar+stream+position+production) is a correct linearizing design. So the only real problem here is the **per-module O(M²) token accessor** (offenders #1/#2); everything else is optimal or a small constant-factor cleanup.

---

## Resolve (src/v1/03_resolve.dag + 04_resolve.dag)

I have everything needed. Both files are fully analyzed, the map primitive costs are confirmed (`map_merge` = `base.extend(overlay)` → copies overlay; `map_values`/`map_keys` = full clone-to-Vec), the `TypeEnv` shape is confirmed (local `str_bindings` + `parents: List<TypeEnv>` + `ancestry_str_bindings`), and the resolve pass is confirmed graph-major (once per item via `analyze_item`).

---

# Efficiency Audit — `src/v1/03_resolve.dag` & `src/v1/04_resolve.dag`

**Scaling variables:** N = modules · M = nodes/item-size per module · E = visible-env binding count (local + ancestry + all parents) · P = parent-env count (≈ import depth D) · A = arms/variants/fields per type · Eg = import edges · U = use-sites of a given generic/alias · K = kernel-type-set size (constant).

**Confirmed primitive costs** (`stage0/src/v1_rt.rs`): `map_get` O(1)+val-clone · `map_insert` O(1) (owned) · `map_merge(base,overlay)` = clone-base + extend → **O(|base|+|overlay|)** · `map_values`/`map_keys` → **O(|map|)** clone-to-Vec. `authored_name_at` = map_get + substring (cheap, non-scaling but very hot).

## Catalog

| Pass (anchor) | Computes / unit | Asymptotic | Memoization | Verdict |
|---|---|---|---|---|
| `resolve_modules` orchestrator `03:41` | Prebuilds `module_index` + `export_sets` once, resolves each module vs. index; single fold over N | O(ΣM) = O(corpus) | index & export_sets built **once**, passed in | **OPTIMAL** — correct graph-major shape (the *fixed* form of the lesson's request-major resolver) |
| `export_sets` build `03:49` / `get_exported_names` `03:177` | Export name-set per module | O(ΣM + N·K) | per module once; but `map_keys(kernel_type_set)` re-materialized every module (`03:183`) | **ACCEPTABLE** (hoist the constant kernel keys) |
| `resolve_import` `03:130` | Import target + missing-export check / per import | O(imports·names), each lookup O(1) | vs. prebuilt index | **OPTIMAL** |
| `check_duplicate_modules` `03:201` | Dup detection / per module | O(N) map-dedup | — | **OPTIMAL** |
| `topological_sort` + `kahn_drain` `03:241/302` | Kahn topo-order / per module+edge | O((N+Eg)·log N); rebuilds in-degree by re-filtering imports, re-scans queue neighbors 2× per layer | recomputed each call | **ACCEPTABLE** (2× edge-scan + unused `fuel` guard are constant slop) |
| `resolve_item_types` `04:948` | Full item resolve (params/ret/uses/body/anno/children) / per item | O(item nodes)×per-node cost | **once per item** (`analyze_item` `04_infer:5894`) | **ACCEPTABLE**, but resolves `item.inferred` structure *and* re-resolves authored children `04:1023-1050` → same coproduct/product walked twice (SUSPICIOUS sub-part) |
| `resolve_expr_types` `04:778` | Expr-tree type-attach / per expr node | O(expr size), 1 visit/node | structural, no shared substructure | **OPTIMAL** |
| `resolve_node_bounded` `04:369` | Type-expr resolution, depth-capped at 100 (`04:370`) | per-node O(1) **except** generic/alias branch | none at use-site | **SUSPICIOUS→PATHOLOGICAL** (see #1,#2,#3) |
| `collect_unit_variant_phantom_matches` `04:81` (via `lookup_unit_variant_phantom_type` `04:69`) | Nullary-variant-as-type fallback / **per unresolved leaf** | **O(E·(P+A))** per call | **none** — flattens whole visible env every call | **PATHOLOGICAL** |
| `resolve_generic_use_decl` `04:233` | Which decl a generic name binds / per use-site | O(D) ancestry fold (shadow-retry), else O(1) | none; recomputed 2-3×/node | **SUSPICIOUS** |
| `substitute_type_slots` `04:279` | Slot→arg substitution over decl body / per generic use-site | O(decl-body size) full tree rebuild | **none** across use-sites | **SUSPICIOUS** (part of #2) |
| `resolve_nominal_alias_rhs` `04:348` | Parametric-alias RHS resolve / per alias decl | **O(2^nesting)** — recurses children *and* calls `resolve_node(n)` which re-descends same children (`04:350`+`04:353`) | none | **SUSPICIOUS** (exponential in type-nesting depth; small in practice, bounded by alias count) |
| `missing_generic_args_diagnostics` `04:586` | Arity diag / per field & param | re-invokes `is_user_generic_use_site`→`resolve_generic_use_decl` | recomputed | **SUSPICIOUS** (folds into #3) |
| `peel_nominal_alias_identity` `04:142` | Nominal-brand preservation / per alias identity | up to 3× `resolve_node_bounded` on resolved/target (`04:150-163`) | none | **ACCEPTABLE** (bounded fan-out, minor) |

## The 3 worst offenders

**1. `collect_unit_variant_phantom_matches` — `04_resolve.dag:81` (PATHOLOGICAL).**
This is the reference lesson's per-unit pathology, verbatim. Every unresolved leaf that reaches the fallback at `04:569` runs:
```
let direct_bindings = fold(env.parents, init: env.str_bindings, f: (acc, parent) =>
    map_merge(parent.str_bindings, acc))          // O(P·E): clones every parent's str_bindings
fold(direct_bindings |> map_values, init: [], ... unit_variant_in_coproduct(...))  // O(E) scan × O(A) variant-fold each
```
`map_merge` clones the base map each fold step (`v1_rt.rs:213` `base.extend`), so the flatten is O(P·E); then `map_values` materializes all E bindings and each is folded over its A variants → **O(E·(P+A)) per lookup**. Fires per nullary-variant-as-type use-site (and per genuinely-unresolved name before it errors) → literals × env-size × variant-expansion, matching the measured 60-min blowup shape. **Fix:** build a `Map<variant_name → List<owner_coproduct>>` index once per env (invert the visible set), or memoize the flattened visible-binding set on the env; the lookup becomes O(1)/O(#matches).

**2. Use-site generic & parameterized-alias re-expansion — `04_resolve.dag:461-502` + `substitute_type_slots:279` (REDUNDANCY).**
Every occurrence of a generic type or parameterized alias re-runs the full `substitute_type_slots` tree-rebuild over the decl body (`04:483`/`04:492-494`) and then re-resolves the expanded target via `resolve_node_bounded(depth+1)` (`04:486`). Nothing is keyed on the instantiation, so `Optional<Int>` appearing at U field/param sites does U full expansions + U re-resolutions of an identical result. This is the "unit of caching is wrong" axis at the type level. **Fix:** memoize resolved expansion on `(decl identity, resolved arg-type identities)` — a per-env instantiation cache — so each distinct instantiation expands once.

**3. Redundant `resolve_generic_use_decl` / `is_user_generic_use_site` recomputation — `04_resolve.dag:461→463`, `270`, `586` (REDUNDANCY / derivable-once fact).**
For a generic use-site with args, `resolve_node_bounded` calls `is_user_generic_use_site` (`04:461`, which internally calls `resolve_generic_use_decl`) and then immediately calls `resolve_generic_use_decl` **again** at `04:463`; the same classification is recomputed a third time per field/param inside `missing_generic_args_diagnostics` (`04:587`→`resolve_field:616`/`resolve_param:650`/item-child loops `04:1027,1038`). Each call is an O(D=P) ancestry `fold(env.parents …)` in the kernel-shadow retry case (`04:239-246`). The decl a name binds to is a derivable-once fact per node. **Fix:** compute the decl/classification once per node and thread it through the arity-diag + expansion paths (hoist out of `missing_generic_args_diagnostics`).

## Note on the assigned specifics
- **`resolve_type_node` recursion depth:** bounded — `resolve_node_bounded` hard-caps at `depth > 100` (`04:370`) emitting an `InternalError`; every structural descent passes `depth + 1`. Safe against runaway recursion; the cost risk is *fan-out per node* (offenders #1-#3), not depth.
- **Per-use-site re-resolution of the same type reference:** confirmed present — offenders #2 and #3 both re-derive the same instantiation/decl per use-site with no use-site cache. The env's stored `resolved` bindings are reused, but use-site *expansion* is not.
- **`03_resolve.dag` is clean:** it is already graph-major (build-index-once, resolve-each-once) — the corrected form of the lesson's quadratic request-major resolver. Only constant-factor slop (kernel-keys re-materialized per module; 2× edge-scan in Kahn; unused `fuel` guard).

---

## Infer — env construction (04_env.dag + env/rewire passes of 04_infer.dag)

Confirmed the picture. `flatten_visible_bindings`/`merge_envs` are cold in the `.dag` corpus (only reference each other; `merge_envs` is imported but not invoked on the hot path), matching the `flatten_visible_parent_recurses == 0` witness. The per-module resolver `resolve_env_bindings` (6290) does not call flatten. Here is the catalog.

---

# Efficiency Audit — `04_env.dag` + env/rewire passes in `04_infer.dag`

Scaling vars: **N** = modules (~89) · **M** = local items/nodes per module · **B** = local type bindings per module · **C** = transitive *visible-name closure* size per module (grows with import depth **D**) · **I** = direct imports per module · **E** = local type-dep edges.

## Outer schedule (context)
`typecheck_modules` (04_infer.dag:6510) is already **graph-major**: topological single pass, `module_index` accumulates typed modules, `parent_index` is passed by reference. Each module is typechecked once and parent `TypeEnv`s are shared by reference. So the resolver's *request-major* redundancy is **not** re-introduced at the outer loop — the cost lives in per-module cache materialization and the corpus-wide rewire pass.

## Catalog

| Pass / algo | Anchor | Unit / what it computes | Asymptotic | Memo / sharing | Verdict | Fix direction |
|---|---|---|---|---|---|---|
| **rewire import str-binding identity** | 04_infer.dag:6613 (`rewire_type_env_import_str_binding_identity`) → :6593 `rewire_inherited_str_binding` | per **inherited name** per **module**: rewrites each visible binding to its canonical exporter | **Θ(N²·C·B)** — per name it calls `global_type_exporter_count` (6556, scans all N modules × B bindings) **and** `local_authority_for_name` (6562, same whole-corpus scan) | index built once per pass (6614) but exporter/authority lookups recomputed from scratch **per (module,name)** | **PATHOLOGICAL** | precompute one `name → {exporter modules, canonical binding}` index in a single O(corpus) pass; each rewire becomes O(1) |
| **per-module cache = full closure** | 04_infer.dag:5689 `union_parent_type_env_caches` (:5491) + cache built :5743; `all_deps_map` :5703 | per **module**: materializes the *entire transitive import closure* deps_map + str_bindings + cycle_set_str as a fresh map | union fold over I imports × `merge_type_env_cache` (each 4× `map_merge` of C-sized maps) → **Θ(I·C)/module**; corpus **Θ(N·C) … Θ(N²·M)** by import shape | **nothing shared by reference** — `merge_type_env_cache` (04_env.dag:53) clones via `map_merge`; every module holds its own O(C) copy of the shared core | **SUSPICIOUS→PATHOLOGICAL** | store only the **local delta** per module + resolve through parent-cache chain (references), not a re-materialized closure; or intern one shared closure map |
| **flatten_visible_bindings** | 04_env.dag:217 | whole **parent DAG**: flatten all visible bindings | un-memoized recursion: re-flattens each shared parent **once per path** → exponential on diamond imports; each step whole-map `map_merge` | **no memo** on env identity | **SUSPICIOUS (latent/cold)** | currently avoided on hot path by precomputed `ancestry_str_bindings` (witness `flatten_visible_parent_recurses==0`); if re-enabled, memoize per-env |
| **merge_envs** | 04_env.dag:225 | one-shot merge of a list of envs | Σ flatten (calls the exponential fn) + `map_merge` + re-intern every merged binding (:237) → O(total bindings) atop flatten's blowup | none; rebuilds `bindings_by_ident` from scratch | **SUSPICIOUS (latent/cold)** — imported (04_infer:86) but not invoked on hot path | if used, drop the flatten dependency / memoize |
| **merge_type_env_cache** | 04_env.dag:53 | primitive: 4× `map_merge` | O(\|base\|+\|overlay\|)=O(C) per call | fresh map each call (correct for a primitive) | **ACCEPTABLE** (primitive) — the *caller* (union loop) is the problem | — |
| **detect_type_cycles_kahn** | 04_cycle.dag:99 (via :5704) | per **module**, over **local** names only (`bindings: local_str_bindings`, C-sized `deps_map` only `map_get`-probed) | Kahn O((V+E)·log) with V,E = local; corpus Σ O(M log M) | recomputed per module (correct — local fact) | **ACCEPTABLE** — correctly per-unit-bounded, not closure-wide | — |
| **rewire parent links** | 04_infer.dag:6682 / 6732 | per module: rebuild `parents` / func parents from import index | O(N) index + Σ O(I); `shared_kernel` found once & shared by ref | parent envs shared by reference (good); `type_env_for_import` only re-filters for `std.types` importers | **ACCEPTABLE** | (minor: cache the `std.types` filtered env once) |
| **build_type_env local scaffolding** | 04_infer.dag:5588–5714 | per module: kernel seed + local bindings + inductive fields | kernel re-seeded **every module** (5595–5619) ≈ O(kernel) const; local folds O(M) | kernel bindings rebuilt per module, not shared | **ACCEPTABLE** (bounded const) — but redundant; hoist kernel env once | build kernel `TypeEnv` once (cf. `compiler_kernel_type_env` at :6655) and share |

## Three worst offenders (evidence)

**1. `rewire_type_env_import_str_binding_identity` — PATHOLOGICAL, Θ(N²·C·B).** 04_infer.dag:6617 maps over all N modules; 6622 folds over `inherited_keys` (the module's whole visible closure, ~C names); each name enters `rewire_inherited_str_binding` (6593) which calls **`global_type_exporter_count(modules, name)`** (6556: `filter(modules, …)` over all N modules, each `module_exports_type_name` filtering B bindings) **and** **`local_authority_for_name(modules, name)`** (6562: another all-N-modules × B-bindings scan). This is the reference "whole-corpus scan per literal" pathology, one level up: whole-corpus scan **per visible name per module**. A single global `name→exporters` index (one O(corpus) pass) collapses it to O(N·C + corpus).

**2. Per-module full-closure `TypeEnvCache` — SUSPICIOUS→PATHOLOGICAL (the KEY QUESTION).** Answer: **yes, whole-map merges are recomputed per module and nothing is shared by reference.** `union_parent_type_env_caches` (5491) folds `merge_type_env_cache(acc, parent.type_env_cache)` over imports; each parent cache *already holds its full transitive closure* (built identically at 5743 with `deps_map: all_deps_map`, `str_bindings: visible_str_bindings`), and `merge_type_env_cache` (04_env.dag:53) `map_merge`-clones rather than referencing. So every module re-materializes an O(C) copy of the shared stdlib core: Θ(I·C) per module, and N redundant copies of the common closure across the corpus (Θ(N·C), degrading toward Θ(N²·M) on deep chains). The `TypeEnv.parents` list *is* shared by reference (5628/5736) — good — but the parallel `TypeEnvCache` (deps_map/str_bindings/cycle_set_str) is fully re-cloned. Fix: keep only the local delta and resolve through the parent chain.

**3. `flatten_visible_bindings` — SUSPICIOUS (latent).** 04_env.dag:217–223: recursion over `env.parents` with `map_merge(flatten_visible_bindings(parent), acc)` and no memo — on a diamond import DAG the same parent subtree is re-flattened once per path (exponential), each step merging whole maps. Currently **cold**: build_type_env precomputes `ancestry_str_bindings` and `lookup_binding_by_name` is fail-closed with no flatten fallback (witness `flatten_visible_parent_recurses == 0`, 04_env.dag:36); only `merge_envs` (itself uninvoked on the hot path) routes through it. It is a loaded gun, not a firing one — memoize per-env identity before any hot-path use.

Note: the emit-stage `--target dag` slowdown is not in these files — this scope is the typecheck/`reconcile` path (04_infer.dag:6755). Offender #1 fires during `reconcile` on every full-corpus compile.

---

## Infer — core (04_infer.dag inference passes)

I have enough evidence across all assigned passes. Here is the catalog.

---

# Efficiency audit — `src/v1/04_infer.dag` (inference core)

**Scaling variables:** N = #modules · M = module size (items/nodes) · E = visible-env bindings · A = arms/fields per match/record · K = imported names per import · D = import/alias-chain depth · T = type-node subtree size.

**Baseline established (kills the reference-lesson's first pathology):** name lookups here are O(1) HashMap gets, not env scans. `lookup_type_by_name`→`lookup_binding_by_name` (`src/v1/04_env.dag:79`) is `str_bindings` → `ancestry_str_bindings` → `intern+bindings`, all `map_get`; the `flatten_visible_bindings` fallback was reverted (note at `04_env.dag:36`). Constructor-owner lookup is now the O(1) binding-edge (`variant_owner_node`, authority note `04_infer.dag:490`). So the *per-literal whole-env flatten+scan* the lesson describes is already gone from this file. The remaining offenders are (a) a graph-major redundancy in import-variant binding and (b) accidental-quadratic / latent-exponential inner loops.

## Catalog

| Pass / anchor | Computes over unit | Asymptotic | Memo/sharing | Verdict | Fix direction |
|---|---|---|---|---|---|
| `infer_expr` dispatch `04_infer.dag:1338` | match on `expr_data`, one visit per expr node; post-order recursion | O(1)/node dispatch → O(nodes) per body | none needed (tree walk) | **OPTIMAL** as dispatch | — (but see match branch) |
| `ExprMatch` two-pass arm re-inference `04_infer.dag:1983` (pass1) + `2023-2063` (pass2) | per match, per arm | pass2 does `arm_infer_results \|> enumerate \|> filter(ap==idx)` per arm ⇒ **O(A²)** positional index (`:2027`); re-inference recurses into nested matches ⇒ worst-case **2^(match-nesting depth)** | none — arm bodies re-`infer_expr`'d, `annotate_pattern_parent_enums`+`extend_scope_with_pattern_node` re-run per arm (`:2036-2037`) | **SUSPICIOUS** (accidental-quadratic + latent exponential, gated by "type resolves only via outer expected") | zip the two parallel lists instead of enumerate+filter (A²→A); unify once / memoize node inference instead of full re-infer |
| `infer_record_lit` ladder `04_infer.dag:2863` | per record-literal use-site | field template re-derived every use-site via `record_lit_instantiated_fields`/`_expected_fields`/`_alias_struct_fields`; per field-init `struct_fields \|> filter \|> first` ⇒ **O(A²)** (`:2912`); `field_in_any_variant_named` walks all arms × `expand_type_for_field_access` per field-init in coproduct-construction case (`:2841-2851`, `:2905`) | none — per-decl field list recomputed per literal | **SUSPICIOUS** (A² + per-use-site rederivation of a per-decl fact) | index `struct_fields` by name into a map; memoize field template per type decl |
| `record_lit_instantiated_fields` `04_infer.dag:607` | per generic record use-site | fold over `decl.params` + map over template fields, each `substitute_generics` O(T) | none (args differ per use-site — inherent) | **ACCEPTABLE** | — |
| `expand_type_for_field_access` / `expand_alias_chain_for_field_access` `04_infer.dag:2807` / `2752` | per field-access base type | alias-chain walk bounded by `seen` set ⇒ O(D_alias); per hop O(1) lookup + `substitute_generics` O(T) | `seen` map prevents cycles; recomputed per field-access use-site | **ACCEPTABLE** (chain depth small; parallel to `04_resolve` — flagged for dissolution at `:2796`) | — |
| `substitute_generics(_apply)` `04_infer.dag:5405` | per type subtree | O(T), with `rc_ptr_eq` short-circuit to share unchanged subtrees | structural sharing via ptr-eq | **OPTIMAL** | — |
| `annotate_pattern_parent_enums` `04_infer.dag:1030` | per arm pattern, recursive over field bindings | per binding: `resolve_pattern_subject`+`lookup_variant_in_type`+`lookup_field_in_variant` (`find_child_named`=O(arms/fields)); bounded by pattern size | none; **run twice per arm** by the two-pass above | **ACCEPTABLE** (doubled by match finding) | fold once (fix the match two-pass) |
| `extend_scope_with_pattern_node` `04_infer.dag:1169` | per arm, fold over field bindings + nested patterns | per binding: lookups + `derive_field_provenance`→`inductive_fields_for` which **recurses `env.parents`** O(D) (`04_env.dag:157`); `concat(acc.diagnostics,…)` in fold ⇒ O(bindings²) diag accumulation | none | **ACCEPTABLE** (D-factor per binding; bindings small) | build provenance index once; accumulate diags without repeated concat |
| `check_match_exhaustiveness` `04_patterns.dag:279` | per match | `covered_set` fold O(A) + `variant_names` filter O(V) | none needed | **OPTIMAL** | — |
| `build_local_variants` / `bind_coproduct_item_arms` `04_infer.dag:6022` / `6010` | per module | fold items × arms = O(M) once/module | once per module | **OPTIMAL** | — |
| `build_imported_variants` + `owner_of_exported_arm` + `exported_coproduct_item` `04_infer.dag:6079` / `6033` / `6056` | per (importing module × imported name) | for **every** specific name (incl. non-variant names) calls BOTH followers, each `tm.items \|> filter(Disj) \|> filter(child==name)` = O(M_target·A), then follows re-export chain O(D). Over corpus ⇒ **O(N·K·D·M_target)** | **none** — per-exporter arm→owner / enum→item index rebuilt on every importer's every name | **PATHOLOGICAL / SUSPICIOUS** (request-major; the reference-lesson redundancy shape) | precompute per-exporting-module `Map<arm→owner>` + `Map<enum→item>` once (with re-export resolution baked in); binder does O(1) `map_get` |
| `build_module_context` `04_infer.dag:6106` orchestrator; `typecheck_modules` fold `:6510` | per module, N-fold | drives all the above once/module; passes accumulating `parent_index` | no cross-module variant-index cache | **SUSPICIOUS** (carrier for the redundancy above) | attach per-module export index to `TypedModule` at typecheck, reuse from `parent_index` |

## 3 worst offenders (evidence)

**1. `build_imported_variants` + re-export followers — whole-corpus redundancy (graph-major inversion).** `04_infer.dag:6079-6104`. For each resolved import, for **each** specific name it calls *both* `exported_coproduct_item` and `owner_of_exported_arm`, and each (`:6037-6052`, `:6060-6075`) does `tm.items |> filter(Disj) |> filter(any child == name) |> first` — a full scan of the exporting module's item list — and on miss walks the specific-name re-export chain, re-scanning at every hop (depth D). Crucially this fires for *every imported name*, and most imports are ordinary functions/types that are not variant arms, so the common case is a full Disj-scan + chain-walk that returns nothing. `build_imported_variants` runs once per module inside the `typecheck_modules` fold (`:6510-6537`) with no per-exporter index cached in `TypedModule` or `parent_index`. Net **O(N·K·D·M_target)**: if a large module (e.g. core) is imported by all N modules pulling K names each, its M items are re-scanned N·K times. This is exactly the reference lesson's request-major redundancy. Fix: compute `Map<arm→owner>`+`Map<enum→item>` once per exporting module (re-export resolved), store on `TypedModule`, binder does O(1) `map_get` per name.

**2. `ExprMatch` two-pass arm inference — accidental O(A²) + latent 2^depth.** `04_infer.dag:2023-2063`. When `unified_arm_type` resolves, pass 2 rebuilds each arm; to fetch the pass-1 result for arm `idx` it does `arm_infer_results |> enumerate |> filter(ap => ap.first == idx) |> map(...)` (`:2027`) — an O(A) positional lookup inside a per-arm map ⇒ **O(A²)** for a list it already holds in parallel order with `arm_nodes`. Worse, when re-inference triggers it calls `infer_expr` on the arm body again (`:2043`); if that body is itself a match whose types only resolve given the now-known expected type, its own pass 2 fires ⇒ worst-case **2^(nesting depth)** re-inference, unmemoized. Gated by "arm body type unresolved on pass 1 but unified type resolved," so not always hot, but unbounded in principle. Fix: `zip(arm_nodes, arm_infer_results)` (A²→A); replace re-inference with a unify/constraint step or memoize per-node inference.

**3. `infer_record_lit` — per-use-site field-template rederivation + O(A²) field match.** `04_infer.dag:2863-2955`. The struct's field list is a per-decl fact, but every `T { … }` literal recomputes it through the `record_lit_instantiated_fields → record_lit_fields_from_expected → record_lit_expected_fields → record_lit_alias_struct_fields` ladder (`:2864-2891`), including alias expansion and generic substitution. Then for **each** field-init it does `struct_fields |> filter(sf => name==fi_name) |> first` (`:2912`) ⇒ **O(A²)** in field count, and `field_in_any_variant_named` (`:2905`→`:2841`) walks every arm of the variant owner calling `expand_type_for_field_access` per arm in the coproduct-construction path. Fix: index `struct_fields` by name into a map (A²→A) and memoize the non-generic field template per type declaration so repeated literals of the same type reuse it.

**Adjacent (outside assigned set, worth flagging):** `04_infer.dag:6563` / `:6570` in the str-bindings rewire post-pass do `filter(modules, m => filter(m.type_env.bindings |> map_values, b => b.name==name)…)` — an **O(N·E)** whole-corpus binding scan per name lookup; same graph-major-index fix applies.

---

## Emit (--target dag path + rust emit)

I have everything needed. Here's my analysis.

---

# Emit-stage efficiency audit — `--target dag` (prime suspect) + rust emit

**Key structural finding:** `--target dag` does **not** go through `emit_rust`/`05_emit_rust.dag` at all. `emit_artifact` (compile.dag:147) dispatches `Dag => emit_dag_artifact` (compile.dag:151), a wholly separate path in `compile.dag` + `dag_collect.dag` + `dag_collect_support.dag`. The rust path is comparatively healthy; the 20 minutes lives in the dag **node-collection** phase, and it is a genuine **O(M²)** blowup that I confirmed in the generated Rust.

## Catalog

| # | Pass / anchor | Computes over units | Asymptotics | Memo/sharing | Verdict |
|---|---|---|---|---|---|
| **D1** | `collect_dag_nodes` → `dag_collect_insert` (dag_collect.dag:157, :117) | dedup+order **every node** in the whole closure | **O(M²)** in total nodes M | accumulator `Rc<DagCollectAcc>` threaded through fold; **whole `seen` map + `order` list deep-cloned per node** | **PATHOLOGICAL** |
| **D2** | `dag_node_surface_fingerprint_rec` (dag_collect_support.dag:98), called via `dag_node_fingerprint` at dag_collect.dag:120 | full **subtree hash per insert-call** | **O(M·D)**, worse w/ sharing (D=depth) | none; recomputed per **edge**, even for already-`seen` nodes; children re-hashed at every ancestor | **PATHOLOGICAL** |
| D3 | `build_dag_key_to_id` (compile.dag:223) | id per node | O(M) | bare `Rc<HashMap>` moved into `rc_map_insert` → unique → in-place | OPTIMAL |
| D4 | `dag_graph_source_indices` (compile.dag:229) | merge per-module `source_indices` | O(N·C) cheap-Rc copies (C=closure) | move-threaded merge, but each module carries its whole import-closure's indices | ACCEPTABLE (mild closure redundancy) |
| D5 | `serialize_dag_nodes_table`/`serialize_node_record`/`serialize_node_ref` (compile.dag:777,741,233) | JSON string per node/ref | O(output) | native `Vec::push`+`join`; `dag_node_key` on real spans skips the fingerprint | ACCEPTABLE |
| D6 | `dag_emit_ref_errors` (compile.dag:219) | ref-target check per edge | O(edges) map_get | key_to_id read-only | ACCEPTABLE |
| R1 | `emit_rust` orchestration (05_emit_rust.dag:1607) | corpus → per-module files | O(corpus) | **graph-major**: `build_emit_graph_info`, `build_ownership_results`, `build_shared_types`, `build_module_index`, `build_module_export_sets`, `build_data_item_index` all built **once**, then `modules |> map(emit_module_full)` | OPTIMAL |
| R2 | `build_shared_types` (05_emit_rust.dag:1554) | shared-type set from summaries | O(#types) | once per corpus, carried in `emit_info` | ACCEPTABLE |
| R3 | `render_rust_type` family / `render_rust_applied_type_shared` / `render_rust_decl_type` (148,513,519) | recursion per **type node** | O(type structure) | not memoized, but type exprs are small/bounded | ACCEPTABLE |
| R4 | per-use-site `type_name_is_rust_importable_in_module` etc. (05_emit_rust.dag:1944,1915,1931) | O(1) `typed_module_by_name` (module_index) + scan that module | O(module size) per use-site; **rebuilds `module_emit_scope(tm)` per call** | index built once (good); scope re-derived per call (not) | SUSPICIOUS (minor) |
| R5 | `needs_box_wrapping` (05_emit_rust.dag:3192) | per type node | O(1)-bounded recursion | set lookups O(1) | OPTIMAL |
| R6 | unified expr emitters `emit_unified_typed_expr` + `05_emit.dag` shared layer (2390, whole file) | per **expr node**, single walk | O(total output); `concat`=`push_str`, `join` native | registry/scope O(1) lookups | ACCEPTABLE |

`emit_go`/`emit_python` (05_emit_go.dag:125, 05_emit_python.dag:117) share the `05_emit.dag` unified emitters; they call `emit_*_module(tm)` per module rebuilding single-module `build_emit_graph_info([tm])` — per-module (not corpus-shared like rust) but still **O(corpus)**, not quadratic. Same acceptable shape.

## The 3 worst offenders (with evidence)

**1. `dag_collect_insert` — O(M²) accumulator deep-clone. This is the 20 minutes.**
The `DagCollectAcc` struct is `Rc`-wrapped and its `seen: Map` / `order: List` fields are themselves `Rc`-wrapped. You cannot move a field out of an `Rc`, so codegen emits `.clone()` on each field while the old handle is still live. Generated proof — `v1_compiler_dag_collect.rs:226-231`:
```rust
seen:  v1_rt::rc_map_insert(inner.seen.clone(),  pending.key.clone(), pending.fp.clone()),
order: v1_rt::rc_list_push (inner.order.clone(), pending.anchor.clone()),
```
`inner.seen.clone()` bumps the refcount to ≥2 (inner still needed for `.order`/`.collision_errors`), so `rc_map_insert`'s `Rc::make_mut` (v1_rt.rs:350-358) sees a shared Rc and **clones the entire growing HashMap**; same for `rc_list_push` on `order`. One full copy of the size-k collection per node → Σk = **O(M²)**. Contrast the healthy sibling `build_dag_key_to_id` (`v1_compiler_compile.rs:533`): `rc_map_insert(acc, …)` — `acc` passed **by move**, unique refcount, in-place, O(M). Fix direction: **thread the accumulator as three bare (unwrapped, move-only) locals instead of an `Rc`-wrapped struct** (or build `seen`/`order` with a single owned mutable pass), so `make_mut` stays unique.

**2. `dag_node_surface_fingerprint_rec` — unmemoized whole-subtree rehash, per edge.**
`dag_collect_insert` computes `let fp = dag_node_fingerprint(anchor)` (dag_collect.dag:120) **unconditionally, before** the `seen` check — so it runs once per *incoming edge*, including for nodes already collected. Each call (dag_collect_support.dag:98-103) recursively hashes the node's entire children+params subtree with no memo, so every node's fingerprint is recomputed once per ancestor and once per referencing edge → **O(M·D)** and up. Fix direction: **memoize fingerprint on the node key** (compute once, store in `seen`'s value or a side map) and only compute it in the `Absent`/new branch.

**3. `dag_graph_source_indices` closure-redundant merge (compile.dag:229) / R4 per-use-site `module_emit_scope` rebuild (05_emit_rust.dag:1944).** Secondary. The merge re-copies each module's full import-closure source-index set (O(N·C), cheap Rc values — measurable but small at N=89); the rust-emit importability checks re-derive `module_emit_scope(tm)` on every use-site instead of caching per module. Fix direction: **precompute one corpus-wide source-index map / cache `module_emit_scope` per module in the module_index.**

## One command to confirm

Profile the dag emit and read the top frames:
```
perf record -g --  <stage0 compiler binary>  --target dag <89-module corpus>  &&  perf report --stdio | head -40
```
Expected hot symbols: `hashbrown`/`<HashMap as Clone>::clone` and `Rc::make_mut` under `v1_compiler_dag_collect::dag_collect_insert`, plus `dag_node_surface_fingerprint_rec` — with `serialize_*` far below. (Cheaper corroboration without perf: time `--target dag` on the first 20/40/60/89 modules; O(M²) shows ~4× time per module-count doubling, which typecheck's 21s linear baseline will not.)

---

## Interpreter (src/v1/stage0/src/v1_interpreter.rs)

## Catalog — `src/v1/stage0/src/v1_interpreter.rs` (tree-walking evaluator for the self-hosted compiler)

Scaling variables: **S** = eval-steps executed (dynamic program work), **d** = dynamic call/recursion depth, **A** = fields/args per node, **B** = builtin+method arm count (~89 + ~34), **L** = string/collection length. This is an AST interpreter, so most costs are *per-eval-step constant factors*; the axis that matters is (a) whether any per-step factor hides a whole-structure walk, and (b) whether any structure grows with **d**.

| Pass / algo | Anchor | Units | Asymptotic | Memo/sharing | Verdict |
|---|---|---|---|---|---|
| eval_expr / eval_expr_inner dispatch | 1336 / 1363 | per node/step | O(1)/step but `(*node.expr_data).clone()` + `ctx.si()` Rc-clone **every** step | none (re-done each step) | SUSPICIOUS const — match on `&*expr_data`; lift `si()` into the one arm that needs it |
| Env repr + `extend` + `lookup` | 538–576; used 1328, 5988 | per call / per var read | `extend`=O(new bindings), no parent copy (good); **but chain depth = dynamic call depth**, so `lookup` = O(**d**) probes; ≈O(**d²**) over a depth-**d** recursion | frame shared by Rc; chain *rebuilt taller* each nested call | **PATHOLOGICAL/SUSPICIOUS** — `call_function` extends the **caller** env, not a fixed base |
| eval_var | 1460 | per var node | Symbol cached per node-ptr (good); fast path `env.lookup` inherits O(**d**); slow path registry+`data_cache` memoized | var_sym_cache + data_cache keyed on Rc-ptr | ACCEPTABLE (modulo Env) |
| call_function | 1259 | per user call | O(params) bind build; param-name list cached per fn-ptr; re-calls `ctx.sym(pname)` a few times | param_name_cache per fn Rc-ptr | ACCEPTABLE |
| **match_pattern** | 1899 | **per arm-test, per field** | re-slices each field name from **source** (`authored_name_at`: hashmap lookup + String alloc) every eval; `fields_get` linear O(A) | **none** — unlike eval_var/eval_call, names here are NOT cached | **SUSPICIOUS** — memoize field name/Symbol on node-ptr |
| eval_call dispatch | 2275 | per call | func_name cached (good); **arg names re-sliced per arg per call** (`arg_name_at`, uncached); 7 cheap bridge `contains`; then `eval_builtin` runs on **every** user call | partial (func_name only) | SUSPICIOUS const — cache arg names; precompute builtin-id |
| **eval_builtin** | 4985 | per builtin **and** per user call | `match name:&str` over ~89 arms, default `_=>Ok(None)`; O(**B**) str-compares before real fn lookup | none | SUSPICIOUS const-factor dispatch |
| eval_method_call + eval_algebra_method | 2595 / 2957 | per method call | method name re-sliced per call (uncached); ~34-arm str match; `fields_get` O(A) | none | SUSPICIOUS const |
| List HOFs: map/filter/fold/flat_map/sort_by/join/concat | 2972–3248, 5093 | per collection op | `im_rc::Vector` = O(1) clone, O(log L) index/split; ops single-pass O(L); `free_monoid_to_vec` materializes an O(L) Vec copy | structural sharing | ACCEPTABLE/OPTIMAL (no accidental quadratic from list clone) |
| Value clone traffic | 325 (enum), clones throughout | per bind/read/return | all heap payloads Rc-shared **except `Value::Str(String)`** → deep O(L) copy on every clone/bind/read | Rc for List/Map/Record/Variant/Closure; **not** for Str | SUSPICIOUS const-factor — use `Rc<str>` |
| CanonKey / value_hash (map keys) | 192 / 228 | per map insert/lookup | deep structural hash O(key size), no cached hash | none | ACCEPTABLE (inherent) |

Note: registry/module iterations at 1051, 1216, 1231 are one-time setup (O(N·M) once), not per-step — fine. No per-eval-step whole-corpus scan exists; function *calls* hit `ctx.lookup_fn` (hashmap) before env, so they don't walk the chain.

## Three worst offenders (evidence)

**1. Env chain grows with dynamic recursion depth — the only superlinear defect.** `call_function` (1328) and `apply_closure` (5988) do `Env::extend(env, bindings)` where `env` is the **caller's** env (threaded from `eval_call` 2401 / `run_in_context` 1190), not a fixed global/definition base. So at recursion depth **d** the parent-linked chain is **d** frames deep (verified: `run_in_context`→`call_function(...&env)`→body `eval_call`→`call_function(...&call_env)`→`Env::extend(call_env,…)`). `Env::lookup` (567) walks the chain, and `eval_var` (1503) calls `env.lookup` **before** the global-registry fallback (1509) — so every bare reference to a data-item/global inside a deep recursion walks all **d** frames (finds it at the root under `eager_data_env`, or misses to root then hits registry) → O(**d**) per reference, ≈O(**d²**) per recursion. This is the classic "inner walk over something that should be bounded" and is the prime in-file suspect for the `--target dag` emit blowup (emit = deep recursive AST descent). *Fix: invoke top-level `Value::Fn` bodies against a fixed global/definition base env (true lexical scoping), bounding chain depth to lexical nesting, not dynamic depth.*

**2. Un-memoized per-eval source-slicing of node names on the hottest paths.** `match_pattern` re-derives every field name via `field_binding_name_at`→`authored_name_at` (v1_std_core 1192) on **every** arm evaluation (2000, 2016, 2046, 2079, 2110, 2127) — each is a `map_get` hashmap lookup + `source_text_at` char-slice + `String` allocation (plus spurious `text.clone()==""` allocs). Same uncached slicing for method names (`expr_method_name_at`, 2596), field access (2661), and call arg names (`arg_name_at`, 2294). The name of an immutable AST node is derivable-once, and the codebase already proves the pattern (var_sym_cache 1470, call_func_name_cache 2278, param_name_cache 1276 keyed on `Rc::as_ptr`) — these three sites are just the un-memoized siblings. Cost is bounded (per identifier length) but multiplied by S·A on the pattern-match/method paths that dominate functional-compiler workloads. *Fix: cache name/Symbol on the node Rc-ptr exactly like the existing caches (or resolve at parse time).*

**3. Per-call mega string-dispatch + Str deep-clone traffic.** Every user function call falls through `eval_builtin`'s ~89-arm `match name:&str` (4985, default `_=>Ok(None)` at 5967) and, for methods, `eval_algebra_method`'s ~34-arm match (2957) before reaching the real body — O(**B**) string comparisons per call where a builtin-id/enum resolved once onto the call node would be O(1). Compounding it: `Value::Str(String)` (330) is the only Value payload not Rc-shared, so string values are deep-copied on every `env.lookup(...).clone()` (1504), bind, and return — in string-building emit this is O(total bytes) traffic, and it amplifies any interpreted `acc = concat(acc, piece)` fold (5093 allocates a fresh String each call) into program-level O(N²). *Fix: precompute a builtin/method discriminant on the call/method node at resolve time (O(1) dispatch); switch `Value::Str` to `Rc<str>` so string values share like every other heap value.*

---

## Floor / claim host (cli_run.rs + claim_executor.rs)

# gunbc floor/claim algorithm efficiency catalog

Scope: `src/v1/stage0/src/cli_run.rs` (floor/claim paths) + `src/v1/stage0/src/bin/claim_executor.rs` (plan eval). Scaling vars: **N** modules in corpus, **F** dag files, **E** import edges (whole corpus), **C** entry-closure size, **D** import depth, **W** shard/spawn width, **R** rows/claims.

## Catalog

| # | Pass (anchor) | Computes / unit | Asymptotic | Memo / reuse boundary | Verdict | Fix direction |
|---|---|---|---|---|---|---|
| 1 | `import_closure_from_facts` (cli_run.rs:571) — the engine under `resolve_transitively`→`load_sources_for_entry_with_index` | Entry's transitive import closure, per **entry** | **~O(D·R·E)** string-normalizations + O(D·C·E)/join per entry; invoked ≥2× per distinct entry | **NONE** — recomputed every call; no closure memo | **PATHOLOGICAL** | Prebuild adjacency (`path→imports`, `module→path`, pre-normalized) once per `ModuleGraphFactsLive`; worklist BFS → O(C+closure-edges) |
| 2 | Per-shard index build (cli_run.rs:5008) + per-claim cold resolve (`run_shared_entry_claims`→`resolve_entry_graph`, claim_executor.rs:455 → cli_run.rs:857) | Whole-tree module index + facts + typed cache, per **shard** / per **claim-entry-group** | O(W·(F+E)) rebuilds; shared std/spec prefix re-typechecked W× (shards) / per-unit (claims) | Cold cache **per shard thread / per claim unit**; NOT via `process_shared_index` | **SUSPICIOUS** (request/unit-major, the reference lesson only half-fixed) | Freeze typed cache into a `Send`/Arc snapshot shared across shard+claim threads (S2b/Arc frontier) |
| 3 | `precompute_whole_tree_published_mock_keys` (cli_run.rs:1603) | ~13 `PublishedMockCase` keys; builds own whole-tree index+facts, Strict-resolves declarer closures, per **invocation** | O(F+E) index + Σ declarer O(D·C·E) [inherits #1] + Strict typecheck | Own index+facts (not shared); result cloned to every shard+row; gated by `skip_precompute` | **SUSPICIOUS** (was RSS-dominant ~1.46 GiB) | Reuse `process_shared_index`; select declarers via facts, not a fresh index |
| 4 | `build_floor_lens_hygiene_graph` / `discover_floor_corpus_rows` (cli_run.rs:3464/3631) + naming hygiene | Read every file, extract module/imports, SIDECAR rule scans, build path maps, per **file (whole corpus)** | O(F·L_file + Rules·F) | Once per invocation (and once pre-plan in claim_executor.rs:1763) | **ACCEPTABLE** (linear read) — but a *separate* whole-tree read from `build_module_index` (redundant I/O axis) | Single corpus read feeding both hygiene + index |
| 5 | `check_floor_filename_hygiene` (cli_run.rs:3409) | Basename `__` check, per **filename** | O(F) filenames, no reads | Once per invocation | **OPTIMAL** | — |
| 6 | `inert_lens_modules` (cli_run.rs:3812) | Lens reachability, per **module/edge** | O(V+E) worklist BFS | Once per discovery | **OPTIMAL** (correct worklist — the counter-example to #1) | — |
| 7 | `reconcile_with_typed_cache` (cli_run.rs:1230) | Typecheck+accumulate, per **module in closure** | O(distinct modules) typecheck (memoized); accumulation linear | `typed_module_cache`/`parse_cache` keyed by module/path, once per index; `rc_map_merge`/`rc_map_insert`/`rc_list_push` on **unaliased** accumulators = O(1) amortized (verified: not the quadratic-merge trap) | **ACCEPTABLE** | (minor) `rewire_*` + `build_emit_graph_info` re-walk full closure each resolve, unmemoized |
| 8 | `shard_row_indices_by_entry` (cli_run.rs:5048) | Group rows by entry, round-robin to W | O(R) | per invocation | **OPTIMAL** (note: round-robins by group count, not closure weight → load-skew, not asymptotic) | — |
| 9 | Plan eval: `batches_from_plan` / `group_batch_units` / `run_walk` (claim_executor.rs:234/296/1315) | Materialize batches, coalesce same-entry claims, schedule | O(total runnables); grouping O(batch) hashmap | `entry_to_unit` coalesces to one resolve/entry; `walk_memo` across batches | **OPTIMAL** scheduling — but the resolve *substrate* it dispatches to is #2 | route units through per-thread shared index |

## 3 worst offenders (evidence)

**1. `import_closure_from_facts` — cli_run.rs:571 — PATHOLOGICAL, per-entry whole-corpus fixpoint.**
Non-worklist fixpoint bounded by `let fuel = nodes.len()` (= N passes). Each pass does `let mut next = reached.clone()`, then for every already-reached importer scans **all** edges — calling `workspace_relative_repo_path(&edge.path)` (allocates a `String` via `.replace`+`strip_prefix`) *on every edge, every importer, every pass* — and for each matching edge scans **all** nodes to resolve the target, with an O(R) `next.iter().any` dedup. This is the exact reference pathology ("inner loop scanning something whole-corpus" + "re-computation of derivable-once facts": the adjacency and the normalized paths are rebuilt every pass). It is invoked **≥2× per distinct entry** in discovery (`load_sources_for_entry_with_index` at cli_run.rs:5135 for the subject digest, again inside `resolve_entry_with_parse_cache` at cli_run.rs:1071) and once per SharedClaims/precompute-declarer — none memoized. Notably `resolve_transitively_bfs_legacy` (cli_run.rs:687) right above it *is* a proper O(V+E) worklist BFS; the live facts-based path regressed the closure walk to a naive join.

**2. Per-shard / per-claim cold resolve substrate — cli_run.rs:5008 and claim_executor.rs:455→cli_run.rs:857 — SUSPICIOUS, request/unit-major redundancy.**
The main-thread discovery was fixed to share one index (`process_shared_index`, S1), but each shard thread calls `build_multi_entry_index(&roots)` fresh (cli_run.rs:5008) and every `SharedClaims` unit resolves through **cold** `resolve_entry_graph` (`build_module_index` + `build_module_graph_facts_live` per call). Consequence: the whole-tree file walk/read/parse runs ~W× (plus once per claim entry-group), and the shared std/spec prefix re-typechecks with a cold `typed_module_cache` each time — the "resolve the same closure N times because the caching unit is per-request" shape, only partially retired. `run_walk`'s comment (claim_executor.rs:1321) confirms the cross-thread share is deliberately absent (Rc `!Send`).

**3. `precompute_whole_tree_published_mock_keys` — cli_run.rs:1603 — SUSPICIOUS, redundant whole-tree substrate + inherited #1.**
Builds a private `build_module_index(&dag_roots)` + `build_module_graph_facts_live` (whole-tree walk, separate from the already-built shared index), then `resolve_transitively(declarers, …)` loops ~13 declarers each paying the full `import_closure_from_facts` fixpoint (#1), then a Strict whole-closure typecheck. Runs once per `run_discovery_corpus_with_options` (gated by `skip_precompute`), historically the RSS peak (~1.46 GiB per the in-code comment). Frequency is bounded, but the substrate build is pure-duplicate of `process_shared_index` and the declarer loop multiplies offender #1.

**Direct answer — what still runs whole-tree per invocation / per shard / per claim:**
- Per **invocation**: naming-hygiene whole-tree read (#4), `precompute_whole_tree_published_mock_keys` index+facts+resolve (#3), plus the first `process_shared_index` build.
- Per **shard**: full `build_multi_entry_index` (whole-tree read ×~4: `build_module_index` + `build_module_path_index`×2 + import scan) and a cold typed cache (#2).
- Per **claim entry-group**: cold `resolve_entry_graph` = whole-tree index+facts rebuild + per-entry `import_closure_from_facts` fixpoint (#1, #2).
- Per **entry, ≥2×**: the `import_closure_from_facts` whole-corpus edge+node fixpoint (#1) — the single highest-leverage fix.

---

## v2 interpreted stages (src/v2/compiler/0*.dag)

I have enough to produce the catalog. Let me confirm one more critical helper — the `find_named_child` full body and whether infer's `solve_constraints` walks the node subtree — but I already have decisive evidence. Key confirmations:

- `Map` (collection.dag:49-84) is a **closure chain**: `lookup` is O(K) in #inserts; absent lookups traverse the whole chain.
- `list_snoc_item` = `list_append(left, [item])` = **O(len(left))** → fold-accumulation is O(n²).
- `fold_node` (node.dag:178) runs `algebra.init(n)` at **every** node.
- infer's `InferredTree.facts` (04_infer.dag:365) is backed by `lookup_inferred_facts_in_entries` = **linear O(entries) scan per lookup**.
- `termination_proof_witness_for_node` (cardinality.dag:260) is itself a **full `fold_node` over the subtree**, called per-node inside infer's catamorphism.

---

# Efficiency Catalog — v2 interpreted self-compiler stages (`src/v2/compiler/0*.dag`)

Scaling vars: **N**=modules in corpus, **M**=nodes in one module tree, **A**=atom/use-site occurrences (~O(M)), **E**=binding/namespace size (~O(M)), **C**=node arity (children), **D**=tree depth, **P**=grammar productions (corpus-constant), **T**=tokens in module, **L**=serialized output length. All stages run **interpreted (~100× constant)**, so every super-linear term below is amplified.

Foundational primitive costs (drive everything): `Map` = closure chain → **lookup O(E)**; `list_snoc_item`/`list_append` → **O(len)** ⇒ fold-with-snoc = **O(n²)**; `fold_node` init runs per node.

| # | Pass / algorithm | file:line | Computes / unit | Asymptotic | Memo/sharing | Verdict | Fix direction |
|---|---|---|---|---|---|---|---|
| 1 | **parse packrat memo table** | 02_parse.dag:728,774 | memoize prod×position | **dead** — `let _memo = parse_table_insert(...)` result **discarded**, table never threaded through `ParseExprResult`; lookup (744) always hits empty table | never — every prod re-parsed at every position | **PATHOLOGICAL** | thread updated table out of `parse_expr`/thunks (state-monad), or key a real memo by (pos,prod) → true packrat O(T·P) |
| 2 | parse position key | 02_parse.dag:317,736 | `len(all)-len(toks)` per nonterminal | **O(T)** per prod-attempt → O(T²+) overall | recomputed each call | **PATHOLOGICAL** | carry an integer index in the parse cursor; don't recompute via `length` |
| 3 | grammar validation (nullable/left-corner/undefined/dup) | 02_parse.dag:823,908,942,979 | pure fn of fixed grammar; nullable **O(P²)**, left-corner closure ~**O(P³)** (`grammar_lookup_production` is O(P) filter inside 3 nested folds) | recomputed **once per module** ⇒ **N×O(P³)** | not memoized across N | **SUSPICIOUS** (cross-module redundancy) | validate once per grammar digest (they already compute `grammar_digest` — cache on it) |
| 4 | `grammar_lookup_production` | 02_parse.dag:214 | `filter` productions by name, take[0] | **O(P)** linear per call; called per nonterminal parse | none | **SUSPICIOUS** | precompute name→production index Map once |
| 5 | **normalize** bottom-up rebuild | 03_normalize.dag:133,53 | rebuild tree; per node `list_snoc_item(shell.children,…)` | single pass O(M), but per-node child rebuild **O(C²)** → O(Σ C²) | one walk (good) | **SUSPICIOUS** | build children with prepend+reverse (O(C)) not snoc |
| 6 | **resolve namespace lookup** | 03_resolve.dag:200,261,790 | resolve each atom via `lookup_chain`→`map_get` on closure-chain `bindings` | **O(E)** per atom × **A** atoms = **O(M²)** per module (+O(D) scope frames) | namespace built once (179), but stored as O(E)-lookup closure chain | **PATHOLOGICAL** | back `Namespace.bindings` with an indexed/hashed map (O(1)/O(log) lookup) — this is the reference-lesson shape, interpreted |
| 7 | resolve child folds | 03_resolve.dag:333,529,561,654 | accumulate resolved edges via `list_snoc_item` (+`length(done)` at 341) | **O(C²)** per node | — | **SUSPICIOUS** | prepend+reverse; track index/len in acc |
| 8 | **infer termination proof** | 04_infer.dag:298,302 → cardinality.dag:260 | per node, `termination_proof_witness_for_node(n)` = **full `fold_node` over subtree(n)** | nested catamorphism: **O(Σ subtree(n)) = O(M²)** (O(M·D)) | none — re-walked per node | **PATHOLOGICAL** | compute descent proofs compositionally in the *same* bottom-up fold (combine child proofs in `step`) |
| 9 | **infer facts map** | 04_infer.dag:346,365,323 | `facts.lookup` = `lookup_inferred_facts_in_entries` linear scan | **O(M)** per lookup; used on every branch/match node (518,676,715) | map is a linear list wrapped in a closure | **PATHOLOGICAL** | key entries by `occurrence_id` in an indexed map |
| 10 | infer entry accumulation | 04_infer.dag:272,1018,352 | `concat_inferred_facts_entries`/`list_snoc_item` merge child entry-lists up tree | **O(M²)** (repeated append up the tree) | — | **SUSPICIOUS** | accumulate with a balanced/diff structure or emit into indexed map directly |
| 11 | infer `solve_constraints` per node | 04_infer.dag:307 → constraints.dag:111 | per-node candidate search over graph rooted at n | ≥O(subtree(n)) per node → compounds #8 | none | **SUSPICIOUS** | fold constraints once; share candidate sets |
| 12 | **translate emit driver** | 06_translate.dag:3555,3522,1270,398,375 | per node `coerce_grounded_node`→`canonical_grounding_for_node`→`facts.lookup` (linear, #9) | **M nodes × O(M) lookup = O(M²)** (≥2× via reject re-lookup 3536) | facts map is linear-scan (from infer) | **PATHOLOGICAL** (the measured >20-min emit) | index `facts` by occurrence_id; or thread facts positionally through the same fold |
| 13 | translate double walk | 06_translate.dag:3833–3860,3724 | `fold_node` raw pre-pass, then full `translate_node_mvp1_fold` | 2× full O(M) walks (+#12 per node) | raw result thrown away if relation-row miss | **SUSPICIOUS** | fuse the relation-row check into one pass |
| 14 | translate child rebuild | 06_translate.dag:3895,3908,3702 | `list_snoc_item(parent.children,…)` per step | **O(C²)** per node | — | **SUSPICIOUS** | prepend+reverse |
| 15 | **serialize to source string** | 06_translate.dag:814,931,1076 | `list_append(left: partial, right: source)` accumulate output chars | **O(L²)** in output length | — | **PATHOLOGICAL** (`--target dag`) | build with a rope/right-fold or accumulate reversed chunks, join once |
| 16 | serialize measure budgets | 06_translate.dag:227,238,256,206 | `node_subtree_count` (`fold_node`) per serialize entry (+rules budget each) | O(subtree) per entry; bounded variants thread `remaining` (OK) | rules-budget recomputed per measure call | **ACCEPTABLE** (watch) | compute rules budget once per target |

---

## 3 worst offenders (evidence)

**1. `06_translate` emit — O(M²) per-node facts lookup + O(L²) serialization → the measured >20 min.**
Every node's fold-init (`translate_mvp1_fold_init`, 3555) calls `coerce_grounded_node` (1270) → `canonical_grounding_for_node` (397) → `tree.facts.lookup(node)`. That `.lookup` is `lookup_inferred_facts_in_entries` (04_infer.dag:323), a **linear fold over all entries**. `fold_node` runs init at all **M** nodes ⇒ **M × O(M) = O(M²)**, doubled on the reject branch (3536 re-looks-up). Then `serialize_concrete_syntax_tokens_to_source_string` (814) and `target_serialize_relation_row_from_model_bounded` (967/989) grow the output with `list_append(left: partial, …)` ⇒ **O(L²)** in serialized length. Two independent quadratics on the emit path; ×100 interpretation ⇒ minutes for an 89-module corpus. *Fix:* index `InferredTree.facts` by `occurrence_id` (O(1)); accumulate serialized text reversed and join once.

**2. `04_infer` — O(M²) nested catamorphism (termination proof re-walked per node), plus quadratic entry plumbing.**
The stage is one `fold_node` (1066) — good shape — but its per-node init `infer_node_facts` (298) calls `infer_descent_witness_for_node` → `termination_proof_witness_for_node` (cardinality.dag:260) which is **itself a full `fold_node` over the node's subtree**. Summed over all nodes = **O(Σ subtree(n)) = O(M²)**. Compounded by `concat_inferred_facts_entries`/`list_snoc_item` merging child entry-lists up the tree (272/1018 ⇒ O(M²)) and linear `lookup_inferred_facts_in_entries` on every branch/match node (518,676,715). This directly answers the "one fold, 7 stages" claim: the catamorphism *shape* is reused, but infer **re-walks each subtree per node** and re-scans accumulated entries — it does not share substructure. *Fix:* fold descent proofs compositionally in the existing bottom-up `step`; emit facts into an indexed map instead of a snoc'd list.

**3. `03_resolve` closure-chain namespace — O(M²) per module (the reference-lesson shape, interpreted) — tied with `02_parse`'s dead packrat memo.**
`build_program_namespace` (179) harvests every atom into `bindings`, a `Map` that is a **chain of O(M) `lookup` closures** (collection.dag:75). Then `resolve_atom` (261) → `lookup_chain` (200) → `map_get` walks that chain **O(E)≈O(M)** for each of **A≈O(M)** atom occurrences ⇒ **O(M²)** per module. Same defect as the reference resolver, just via the closure-Map primitive rather than an env flatten. *Co-worst — `02_parse`:* the packrat table is **inert**: `parse_nonterminal_memoized_core` (774) does `let _memo = parse_table_insert(...)` and **discards it** — the table is never threaded out of `parse_expr`/thunks, so lookup (744) always sees the initial empty table and every production re-parses at every position (backtracking recursive descent, exponential worst case), with `parse_current_position` (317) adding an O(T) `length` subtraction per attempt. *Fixes:* back `Namespace.bindings` with an indexed map; thread the parse memo table through results (state-monad) so writes survive.

Cross-cutting: (a) `list_snoc_item`/`list_append` accumulation is quadratic in **every** stage's child-rebuild fold (#5,7,10,14) — replace snoc-fold with prepend+reverse; (b) grammar validation (#3) and canonical-symbol/set construction are **corpus-constant work recomputed per module** (N× redundant) — hoist/memoize on the already-computed `grammar_digest`.

---

# The scope rank-view — resolve a scope instead of materializing one

The seed builds a fresh set of name-keyed maps for every claim scope it enters. `[floor-scope-cost]`
already names the correction in the tree, beside the number it prices: *"a scope that is a view
rather than a rebuild costs nothing to enter."* This is the design for that view.

It is a **replacement migration** (DESIGN §3): the per-scope materialized maps and the rank-view
answer the same semantic question — *which declaration does this name resolve to, in this scope* —
and one of them is intended to disappear. The model lands first, alone; the reader cutover follows
in its own PR, at the root, with no interval in which both representations are live authorities.

## 1. What is materialized today, and what of it is scope-dependent

`cli_run` `claim_scope_for` computes the scope's module `order` (own module, then the compiler's
precedence-ordered import closure, then the reference closure) and then builds two things over it:

- the scope `item_registry` — a union of each scoped module's own registry, folded in `order`,
  first write wins, with the authored-region rule deciding which collisions count as ambiguous;
- `v1.interpreter` `build_scope_indexes_with_module_order` — `fn_nodes` (a bare slot per name and a
  qualified slot per declaration), `ambiguous_bare_function_names`, `file_module_paths`,
  `file_import_bindings`, `service_ops`.

gunbc#9809 removed the module-local half of that derivation: `ModuleScopeFragment` derives each
module's contribution once per prepared subject rather than once per scope containing it. What
survives is exactly the **fold** — the part that is not module-local, because it applies the scope's
precedence to fragments the subject already holds.

The observation this lane rests on is that the surviving fold stores a *decision* that is cheaper to
compute than to store. Each entry of each per-scope map is the answer to "of the modules declaring
this key, which one does this scope rank first". That answer is a function of two things the process
already has: the corpus-wide set of declarers, and the scope's own ordering of modules. Materializing
it once per scope is DESIGN §2 redundancy in its plainest form — 660 copies of a table derived from
one corpus and 660 short lists — and DESIGN §3's second-authority problem, because from that point on
the map, not the order, is what a reader consults.

## 2. The view

Two carriers replace the maps.

**Corpus-wide, one per prepared subject.** A `name -> declarers` index, each declarer carrying the
declaring module's identity alongside the declaration node, in a stable corpus order. This is the
fragment population of #9809 inverted from per-module lists into per-name lists; it introduces no
new derivation and no new authority, only a different join order over facts the subject already
carries.

**Per scope.** A `module -> rank` map, where rank is the module's position in `order`, plus the
`authored_region` boundary already computed there. Its size is the scope's module count (mean 427,
max 878 at the measured run), not its item count.

**Lookup.** Resolution intersects the two: take the key's declarers, keep those whose module has a
rank in this scope, and select by the slot's polarity. Nothing is materialized at scope entry beyond
the rank map, and every answer is derived at the ask.

## 3. The polarity finding — the slots do not agree, and a uniform cutover is silently wrong

The brief names the rule as "minimum rank". That is correct for the slot the prize is denominated in
and **not uniform across the maps being replaced**, which is the first thing this model exists to
record:

| slot | seed rule | rank rule |
|---|---|---|
| `fn_nodes` bare | `or_insert` under `module_order` | lowest rank wins |
| scope `item_registry` | first-write-wins over `order` | lowest rank wins |
| `file_import_bindings` | `or_insert` over the fold | lowest rank wins (key is module-local, so no scope can discriminate) |
| `fn_nodes` qualified | unconditional `insert` | exactly one declarer; **admission-gated** (§4) |
| `file_module_paths` | unconditional `insert` | highest rank wins |
| `service_ops` | unconditional `insert` | **highest rank wins** |

`service_ops` is the inversion that matters: a service operation key claimed by two scoped modules is
today the *last* module in precedence order, and a rank-view that resolved it by minimum rank would
silently redispatch it. The tree already carries the receipt for why this key is dangerous — the
`std.resources` `Filesystem` / `extdeps.filesystem` `Filesystem` pair, whose registry-merge collision
once dropped a service's operations into "unknown service operation" at runtime.

`file_module_paths` is last-write-wins over a key that is module-local in practice (one module per
file), so no current input discriminates the polarity. The model files it at the seed's polarity
rather than at the one a reader would guess, and names what would discriminate it, because "no input
reaches it" is a ceiling to record, not a licence to pick either arm.

## 4. The hard constraint, and the rung it puts at risk

On record for this lane: *a scope's qualified `fn_nodes` must never resolve a name outside the
scope's admitting closure — a claim must not call a module its closure never admitted.*

Today that holds **by construction and incidentally**: the qualified slot exists only because a
scoped module wrote it, so an out-of-scope declaration has no key in the map. The rank-view removes
that construction. The corpus-wide index holds every declarer in the subject, so an unguarded
qualified lookup would answer from a module the scope never admitted — a strictly new failure, and
one that fails in the direction that looks like success.

So the migration's obligation is not to add a filter but to keep the guarantee structural: resolution
must have no arm that yields a declaration without having consulted the rank map, so an unadmitted
answer has no constructor. Concretely, the resolver returns only a declarer obtained *through* rank
membership; there is no accessor that reaches the corpus index and returns a node. DESIGN §3's rule
for a replacement migration is the same point stated generally: the minimum Y is not the smallest
thing that executes the happy path, it must preserve every required refusal.

The model records this as a §4b subject with its previous rung, its target rung and the evidence that
would establish either — not as prose, so that a cutover which reaches the prize while lowering the
rung is a red row rather than an unnoticed regression.

## 5. What the view does not make free — the enumerating readers

Point lookups and membership tests become O(declarers of the key). Four readers instead **enumerate**
the scope registry, and for them the rank-view trades a cheap iteration over a scope-sized map for an
iteration over a corpus-sized one:

- `v1.interpreter` `build_initial_env` — per claim, over data items;
- `v1.interpreter` `eval_data_initializer_values` — over data items;
- `v1.interpreter` `resolve_published_mock_keys` — over data items;
- `cli_run` `roster_entry_registry_cache` — over every key.

The first three filter to `DataItem` immediately, so the view serves them from a kind-partitioned
corpus name list and iterates a fraction of the corpus rather than all of a scope. The fourth wants
every in-scope name and is the one row where the view is not obviously cheaper; it is named here so
the cutover measures it rather than discovering it, and its call frequency is the fact the migration
PR must carry.

This is the design's honest boundary: the prize is scope *entry*, and a reader that enumerates a
scope pays at the ask what it no longer pays at entry. A cutover that quotes the entry saving without
this column would be reporting half a ledger.

## 5b. The migration census — every site, its demand, and its cut order

The brief sizes the reader migration at "~126 `item_registry` sites". That is the grep count over the
token across `src/`, and it is not the migration population: **most of those occurrences are a
different carrier's field of the same name.** Filing them separately is the first thing the census
does, because a cutover scoped to the grep would edit the typecheck layer and the emitter for no
reason, and a cutover that ignored the difference would edit the wrong authority.

### What is not in the population (112 of 126)

| kind | where | count | why it is not the subject |
|---|---|---|---|
| `TypedModule.item_registry` | emitters, `coproduct_reflection`, `owned_data`, `data_initializer_identity`, `v1_compiler_compile` | ~18 | the per-module registry — the authority the corpus index is *derived from*, and the one the fragment population of #9809 already reads. Untouched. |
| typecheck-layer `scope`/`fn_scope`/`lam_scope`/`local`/`state`/`dep_state`/`ctx` registries | `v1_compiler_infer` | ~30 | `v1_compiler_infer`'s own threaded registry at compile time. A different carrier with the same field name; it never sees a claim scope. |
| field declarations | `v1_compiler_infer`, `v1_compiler_infer_items`, `v1_interpreter` | 8 | type declarations, not reads. |
| empty-map struct initializers | test fixtures and empty contexts across 9 files | ~12 | `Rc::new(HashMap::new())`; nothing is read. |
| `item_registry_keys` | `v1_compiler_artifact`, `v1_compiler_compile` | 5 | a different field: the serialized key list of an artifact. |
| doc-comment mentions | `cli_run`, `v1_interpreter`, `coproduct_reflection` | ~9 | prose. |
| `build_qualified_item_registry` and its duplicate marker | `v1_compiler_emit_rust` | ~10 | the emitter's own qualified overlay, with its own fail-closed refusal. A separate authority on a separate key. |

### The population (14 readers, 4 construction sites)

Demands, and the arm each falls in. The fourth arm is one the reader census did not originally
carry and this sweep found:

**Provenance test** — the reader looks the name up *in order to read `info.module_name`*, and decides
a builtin dispatch from it. These are the sites where a polarity error does not surface as a missing
name but as a **different function executing**, so they are the ones the qualified-admission RED and
the ambiguity line must both be exercised against:

| site | decides |
|---|---|
| `v1_compiler.v1_interpreter` `try_witness_evaluation_dispatch` | whether a call is the witness-evaluation authority's own |
| `v1_compiler.v1_interpreter` `is_v4_bridge_family` | whether a name is one bridge family's, by declaring module |
| `v1_compiler.v1_interpreter` `is_v2_std_collection_map_grounded_fn` | whether a call takes the primitive map grounding |
| `v1_compiler.cli_run` `definer_module_for_name` | the declaring module of a name — on a `ResolvedGraph` argument, so whether it is in the population depends on which graph its callers pass, and that is the one row this census leaves open for the cutover to close |

**Point lookup** — `v1_compiler.v1_interpreter` `eval_var` (the registry slow path behind the env
lookup) and `eval_data_item_value`. Both already re-resolve through `lookup_fn_from` immediately
after, so under the view they collapse into one resolution rather than two.

**Membership test** — five sites, all `contains_key` guarding a call into a modeled function:
`call_test_claim_fn_bool`, the three `call_floor_kernel_would_skip` / `call_floor_row_would_skip` /
`call_floor_row_precompute_would_skip` guards, and `rerun_frontier_nodes_for_entry`. Each
becomes `resolve(name).is_some()`.

**Enumeration** — the four readers of §5.

**Construction** — `v1_compiler.cli_run` `claim_scope_for`'s registry fold (with its `winner_of` /
`ambiguous` bookkeeping) and `v1_compiler.v1_interpreter` `build_scope_indexes_with_module_order`.
These are the root: they are deleted, not migrated.

Plus one site that looked like the hardest and is not. `v1_compiler.v1_interpreter`
`InterpContext::resolved_graph` hands out a whole `ResolvedGraph` built from the scope map, which
would force materialization to survive the cut for any consumer of it. **It has no consumer** — no
call site anywhere in `src/`. It is deleted with the maps rather than being the reason to keep them.

### Cut order

1. The resolver and the rank map land, and the **provenance tests and point lookups** move first —
   they are the sites that read the winner rather than merely its presence, so they are where a
   polarity or admission error is loudest, and moving them first means the discriminating REDs run
   against the arm most likely to be wrong.
2. The **membership tests**, which are the same resolution asked for less.
3. The **kind-filtered enumerators**, which need the corpus kind partition to exist.
4. `roster_entry_registry_cache`, last, and only after its call frequency is measured — it is the one
   reader the view does not obviously make cheaper.
5. The **construction sites and `resolved_graph`** are deleted in the same motion, which is what
   makes this a root cut rather than a leaf-first refinement. Nothing in steps 1–4 may be merged with
   the fold still standing: a surviving map is the attractor, and readers understanding both
   representations is the parallel-authority state the migration exists to avoid.

## 6. The scope population — 660 and 1,155 come from one instrument at two revisions

The brief asks that the delta between 1,155 and 660 be reconciled rather than assumed. Both are
readings of the **same instrument**: the required floor's
`floor: N scope construction(s) for M distinct scope(s)` line, whose `M` is the distinct-scope count.

- **660** is a current reading, from a named required-floor job on the #9809 branch, where
  `[floor-scope-split] constructions=660` and `M=660` agree — constructions equal distinct scopes,
  so the per-claim rebuild is already gone.
- **1,155** is a *transcription* of an earlier reading of that same line, sitting in the
  `PreparedScopeIndexes` doc comment in `v1.interpreter` inside the sentence that records the
  per-claim-rebuild finding ("9,573 reconstructions of maps that only 1,155 distinct scopes can
  differ in"). Its companion figure, 9,573 constructions, is the one that has since been repaired to
  equal the distinct count.

So the delta is not two instruments disagreeing and not a scope-universe difference: it is one
instrument read at two revisions, with the older reading copied into prose. Whether `M` fell from
1,155 to 660 because the corpus moved, because the claim manifest moved, or because an intervening
lane narrowed the closure is **not decidable from either figure**, and this design does not guess —
the question is settled by re-deriving `M` on the current tree, which is precisely the "before" half
of §7's paired run. Naming that as the answer rather than picking one of the two numbers is DESIGN
§6's rule applied to the reconciliation itself.

The transcription is the mechanism that made a stale reading look like a rival present-tense
measurement. **This PR deletes it**, replacing the literal pair in the `PreparedScopeIndexes` doc
comment with the name of the line that produces it. The number is not updated — updating it
re-commits the defect at a fresher value. The model keeps the row, because the reconciliation is a
finding about how the two figures were produced and survives the comment that carried one of them.

## 7. Acceptance

Paired required-floor runs, before and after, same instruments:

- identical claim population, results, verdict and digests;
- identical scope identities (the `hash_combine` fold over `order` — the rank map is derived from the
  same `order`, so a moved identity means the closure moved, not the view);
- `[floor-bare-name-ambiguity]` **byte-identical between the pair**. The line moves if and only if
  bare-name precedence forks, which is the exact failure a rank-view can introduce.
- `[floor-scope-split]` as the prize measurement: the `indexes` and `registry` terms go to
  approximately the rank-map build, and `order` is untouched.

The oracle is the paired run, never a literal (DESIGN §5): the relayed figures for the ambiguity line
were measured on a pre-#9809 corpus and are re-baselined by the "before" half of the pair, not
asserted.

## 8. Order of work

1. **This PR — the model only.** `gunbc.scope_rank_view` and its executing claim. No reader moves.
2. The rank-view carriers land beside the fold, with the equivalence law executing over a fixture
   corpus whose two entries rank one colliding bare name oppositely — the discriminating RED that
   `scope_fragment_memo_equivalence` established the shape of for #9809.
3. The root cut: the materialized maps are deleted and every reader is fixed forward onto the view in
   one motion. Not leaf-first — a surviving map is DESIGN §3's attractor, and readers that understand
   both representations are the parallel-authority state the brief forbids.

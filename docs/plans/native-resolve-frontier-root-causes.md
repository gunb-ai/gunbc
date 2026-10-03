# Native resolve frontier: root causes of the unbound and ambiguous refusals

**Status:** investigation output (lane smart-moth-115, under sharp-raven-357). This plan fixes nothing.
It ranks root causes by how many modules each one unblocks, and recommends first PRs.

**Receipt.** Every count this plan relies on comes from one run, not from this prose (DESIGN §6):
`gunbc test //gunbc/instruments:v2-native-census`, dispatched as instrument-dispatch run
**37151899762**. The run was at `7f947a1b`, which is main at `a0e8a12a` plus #13141, the change
that made the label dispatchable. Its artifact `instrument-dispatch-receipt/stdout.txt` holds one
`census_residual` row per independently refused occurrence. Each row carries its full diagnostic
chain: the advisories, then the fatal, whose `NodeLocus` contains the refused node and so the
spelled symbol. The ranking below is derived by reading that artifact, as follows:

1. **Fatal and symbol.** Take each row's fatal reason, its advisory reasons, and the atom
   identities under the fatal's node. A `Conj` node is a list of atoms, so it is flattened.
2. **Root.** Classify each (row, symbol) pair into a root using the rules in the table.
3. **Blocker sets.** For each module, collect the set of roots that block it.
4. **Unblocked counts.** For each root, count:
   - "alone": the modules whose blocker set is exactly that root;
   - "greedy cumulative": the modules unblocked as roots are fixed in the order given.

The derivation script and the counts it printed at this receipt are in the PR that introduced
this file. They are a snapshot of that receipt. Rerun the dispatch to refresh them; don't edit
them here.

**Why the M0 figures are not used.** The figures in the brief come from the
type-declaration-use-census receipt (run 37119024146). That run exited 2 with an empty stdout.
Its log names each unreached module and its cause only, with no symbol and no location, so no
chain can be re-derived from it. The census above is the receipt this plan stands on.

## The chain, re-derived once (DESIGN §6b)

Under the seed, a reference resolves through three mechanisms:

- **Host builtin table.** Names like `join`, `count`, `concat`, `none`, `to_string` and
  `parse_int` come from the seed's host builtin table.
- **Corpus-wide `global_bare` lookup.** Any spelling with a unique declaration anywhere in the
  corpus resolves.
- **Import lists.**

v2 resolves only through the lexical chain. Namespace-resolution-design §13 deleted
`global_bare` as a mechanism, and `v2.compiler.resolve` `lookup_symbol_index_atom_identity`
states that deletion. Imports are transmuted into binding rows at the importing position
(`v2.extdeps.languages.dag` `import_binding_rows_from_decl_node`). The same design rules that
**builtins bind at root**, so that they are unique on every chain.

Almost every refusal sits at one of three boundaries:

- **The design premise has no carrier.** "Builtins bind at root" has nothing to bind:
  - `std.primitives` names the builtins only as **strings** (cost contracts and a name list);
  - the `std.languages` `CollectionOps` / `StringOps` / `MapOps` records carry **per-target
    emission templates** whose field names coincide with the builtin names, and these are not
    declarations of the operations;
  - so no `.dag` declaration of the builtin callables exists for the root to bind.
- **The corpus still leans on the deleted `global_bare` tier.** Many modules never import names
  they use.
- **The selection step counts bindings, not meanings.** `v2.std.symbol_index`
  `symbol_index_lexical_collect` gathers every binding row on the whole ancestor chain, and
  `symbol_index_lexical_selection` decides none/one/many by `length`. When one declaration is
  reached by two routes, that reads as two candidates.

## Ranked roots

The rank order is the greedy order over the receipt: each row is the root that unblocks the most
further modules once the roots above it are fixed. The "alone" ordering differs, and it is noted
where it matters for sequencing.

| # | Root (row predicate) | Earliest unjustified boundary | Class | Repair class | Owner |
|---|---|---|---|---|---|
| R2 | **The builtin vocabulary has no bindable declaration.** The fatal is unbound, and the symbol is in the `std.primitives` name rows or is an `*Ops` template field: `concat`, `join`, `count`, `map`, `filter`, `length`, `string_contains`, `to_string`, `starts_with`, `substring`, `parse_int`… This root also takes the `elsewhere`/`several` advisories that point at `std.languages.StringOps.*` / `CollectionOps.*`. Those pointers are template fields, not declarations. | No `.dag` carrier declares the builtin callables (name and signature) for "builtins bind at root" to bind; the roster is host Rust and string rows. | (b), with a modeling prerequisite | **Model first:** declare the builtin callable roster as declarations in `std` (signatures already partly exist as `std.algebra` op rows, see `gunbc.rung_drop` `concat_binary_signature_exempt_from_arg_binding`). **Then** bind that roster at the root of v2's namespace. Separately, stop the symbol index offering template-record fields as declaration candidates in the unbound oracle. | none found; namespace-resolution lane is the natural home |
| R4 | **A declared name is used without an import (`global_bare` dependence).** The fatal is unbound with the `elsewhere`/`several` advisory pointing at a real declaration: `Present`, `Absent`, `List`, `Node`, `LiveTreeDisposition`, `NonEmptyStr`, `PartialFunction`, `Outcome`… The same root covers bare variant constructors (e.g. `OrthogonalReady`, declared in `std.orthogonal_geometry`). Those get no advisory because the oracle doesn't index variants. | Corpus source. The seed's `global_bare` tier resolved these, and §13 deleted it. | (c) | Corpus migration: add the import, or graft by containment per `gunbc.namespace_cut_landing_order` `current_landing_order`. It's mechanical per module, and the receipt's advisory names the declaration. #13048 is a precedent for one name (`List`). The `several` names (`Absent`, `Present`, `List`) also need a choice between real homonyms; see R6. | namespace-cut program (`gunbc.namespace_cut_landing_order`) |
| R7 | **Declared nowhere in the index (non-builtin).** It splits by re-derivation into: | | | | |
| R7a | …the declaring file was refused at ingest, e.g. `extdeps.languages.typescript.program`, `extdeps.filesystem.filesystem_io`, `extdeps.git`, `v2.compiler.compile`. | The front-end refusal of the declaring file. Fixing that file is what unblocks its importers. | (e) cascade | Fix the root file's front-end refusal: `parse_g0_tokens_remain`, `body_lowering_reason_service_realization_unreachable`, `body_lowering_reason_uses_clause_unmodeled`. | census lane step C (#13120, silent-stag-648) ranks these roots by fan-out |
| R7b | …the first segment of an absolute qualified path: `gunbc.`, `extdeps.`, `git.`, `github.` | Same boundary as R2: the root namespace binds no package heads. | (b) | The same root binding as R2, covering top-level namespaces. | as R2 |
| R7c | …the residue: names declared in accepted files that the index does not bind at the reference (`Run`, `WitnessBin`, `identity`, `subject`, `host`…). | **Not yet re-derived.** These may be fields, parameters or locals reached from the wrong scope. | (e) unclassified | A targeted read per name is owed before any repair is chosen. | — |
| R3 | **`none` is unbound.** | The Optional `none` literal is a seed kernel value with no v2 binding. | (b) | Declare it with the Optional carrier (`v2.std.optional`) or bind it at root with R2. | as R2 |
| R1 | **Ambiguity where both candidates are one declaration.** The fatal is ambiguous and every `resolve_ambiguous_competing_declaration` names the same path. The routes are own import + ancestor-module import, or own import + an ancestor that declares the name. | `v2.std.symbol_index` `symbol_index_lexical_selection` counts binding rows rather than declaration identities. | (b), **resolver ambiguity** | v2 repair: two routes to one declaration are one meaning. Key the candidate set by declaration path before the none/one/many trichotomy. Keep ambiguity for distinct paths, as the zero-ambiguity ruling requires. A separate design question, not decided here: should an import row bind only the importing module's body, rather than its whole namespace subtree? | namespace-resolution lane; no session found holding it |
| R5 | **An effect/resource name is unbound** (`Filesystem`, `Read`, `Write`, `shell`, `Env`, `Exec`, `Clock`). | The `uses` clause has no v2 carrier: `gunbc.recurring_failure_mode` `uses_clause_has_no_carrier`, `gunbc.rung_drop` `network_requirement_unrepresented_after_uses_cut`. | (c) | Waits on D13's resource-keyed DependencyDemand carrier (`gunbc.plans.demand_engine_program`). | D13 / demand-engine program |
| R6 | **Ambiguity between distinct homonyms**: `std.types.List` vs `v2.std.collection.List`, `std.constructors.Optional` vs `v2.std.optional.Optional`, `Cardinality`, `Unit`. Per-module `extdeps_external_authority_anchor` declarations also become visible down a containment chain. | Corpus: the two-std forks, and an anchor name minted once per module. | (a) / (d) | Fix in source. The two-std defork (docs/plans/namespace-flip-last-28-root-a-two-std-defork.md; #13048's `List` class). For the anchor, it needs a ruling on whether a per-module anchor should be visible to contained modules at all. | two-std defork lane |
| R8 | **Other fatals:** `binder_hides_visible_value`, `realized_declaration_mismatch` (`fold_list`), `construct_tag_not_a_constructor`, anonymous record/map expected-type refusals, `float_literal_not_lowered`. | Per-cause; small populations. | mixed | Read each after R1–R6, because most of these modules also carry R2/R4 rows. | — |

**What the ordering says.** Single-root-per-module counts are small: most refused modules carry
several roots at once, so no single fix unblocks a large population on its own. The roots compose.
R2, R4 and R7a together are the wall, and the rest is a tail.

R1 is the exception. Most of its modules have it as their **only** blocker, so one v2 change
unblocks them on its own. That makes it the cheapest first PR even though it ranks fifth greedily.

## In the native tests' import closure

The population is the import closure of every `v2.test.*` module, so the tests plus everything
they import. Every root R1–R8 has members in it. The ordering inside it matches the corpus-wide
ordering: R4, R2, R7, R3, R1, R5. Over a thousand non-test modules in that closure (mostly `extdeps.*`) are
refused, and they are what the tests import.

calm-boar-904 (the N7 manager) is told directly. Whether "native tests" means all of `v2.test.*`
or N7's narrower roster is N7's to state. The same derivation runs against either population.

## Recommended first PRs

1. **R1 (v2, one function).** In `v2.std.symbol_index`, collapse lexical candidates by declaration
   path before `symbol_index_lexical_selection`. Controls:
   - **RED:** the same name imported by a module and by its ancestor module resolves to its one
     declaration.
   - **Positive control:** two distinct declarations still refuse with both candidates.

   This lane does not decide whether ancestor imports should be visible to contained modules. That
   is a follow-up question for the namespace-resolution design.
2. **R2 / R3 / R7b, model first.** A model PR declaring the builtin callable roster and `none` in
   `std`, with signatures, consumed by its own executing control. Then the resolve PR that binds
   that roster, and the top-level namespace heads, at root. The model PR comes first, because
   binding a host string table at root would cement the seed's roster in v2.
3. **R4 (corpus).** A per-subtree import migration driven by the receipt's
   `resolve_unbound_name_is_declared_elsewhere` advisory paths, sequenced under
   `gunbc.namespace_cut_landing_order` rather than as an independent sweep. The `several` names
   wait on R6's defork choice.
4. **R7a.** Hand the cascade roots to the census lane's step C once #13120 lands. Its fan-out
   ranking is the right instrument for them, and this plan builds no second one.
5. **R7c.** A short re-derivation of the residue names, before anything is built for them.

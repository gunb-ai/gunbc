# Lowering occurrence projection (XL-2 prerequisite `LoweringOccurrenceProjection`): design

Status: design only. No compiler behaviour changes in the PR that lands this file. Tracked by
`gunbc.recurring_failure_mode.lowering_rebuilds_an_authored_atom_without_its_occurrence`, carrier arm
`gunbc.compiler_frontend_program_status` `LoweringOccurrenceProjection`.

## 1. What was measured, and how

The subject is the pinned stratified sample,
`v2.compiler.reference_conservation_census` `reference_conservation_stratified_sample_paths`
(315 paths), measured on main `69e0bb7566e` in 13 batches of 25 paths. The whole sample in one
process is too much memory; a batch peaks at about 9.8 GB RSS.

The census (`reference_conservation_census_for_paths`) reports `locus_erased` per module as a
count. It cannot say which lowering function rebuilt an atom, because nothing on the node records
that. The per-site split was therefore taken with a **one-off, uncommitted instrumentation**:

- In a throwaway worktree, each `occurrence_id: OccurrenceSynthetic` literal and each
  `node_synthetic(` call in the normalize route was replaced by a per-site constant, in
  `src/v2/compiler/*.dag`, `src/v2/extdeps/languages/dag.dag`, `src/v2/std/grammar.dag`, and the
  production `v2.std` callers of `node_synthetic` (`qualified_name`, `node`, `fold_assembly`,
  `compilers/body_lowering`, `runtime`, `anti_unification`, `type_binder`, `inhabitance`). That is
  131 sites. The constant is an `OccurrenceProjected` whose id is the site number. The census treats
  `OccurrenceProjected` exactly like `OccurrenceSynthetic` (`reference_conservation`
  `minted_occurrence_of` answers `Absent` for both), so the counts should be unchanged. That was
  checked on a 4-module control: all 144 census lines, summaries and drop rows, are byte-identical
  between the instrumented and uninstrumented trees.
- A scratch probe re-ran the census's declaration-scoped spelling pool and recorded which pool
  entry each locus-erased atom consumed: a rebuilt atom and the site tag on it, or a `Named` edge
  label. It tried atoms before labels. It also put two more kinds of atom into the pool: an atom
  whose minted occurrence belongs to a non-atom node (**wrong source: an enclosing shell**), and an
  atom whose minted occurrence belongs to a different authored atom.

The instrumentation is not an entry point and is not committed, because it only exists as edits to
the compiler. §5 below makes the lasting instrument part of the build. The recipe can be repeated
from this description; the probe module and the tagging script are about 250 lines.

### Totals (census, sample, main `69e0bb7566e`)

| disposition | atoms | note |
|---|---:|---|
| authored | 30,195 | |
| conserved | 5,392 (17.9%) | since #11985, type-position atoms keep their occurrence |
| **locus_erased** | **6,340 (21.0%)** | **this carrier's subject** |
| dropped: `refusal-not-at-or-above` | 14,787 (49.0%) | all 96 modules that normalize refuses |
| dropped: `absent` | 3,472 (11.5%) | the 206 modules normalize accepts |
| refused | 204 | |
| (channels) import / test marker | 5,671 / 495 | not in `authored` |

Of the 315 modules, 206 normalize, 96 are refused at normalize, 7 are refused at parse, and 6 are
unreadable: six `dag/gunbc/namespace/transition_admission/*` files on the pinned list have been
deleted since the pin.

The brief's figure of "1,390 of 30,020 conserved" is stale. It was taken at `fec339d561`. The
difference is mostly type atoms, which #11985 now carries through `body_lower_type_atom_node` via
`node_lowered_from`. **The RFM row's claim that `body_lower_type_atom_node` builds with
`node_synthetic` is no longer true.** This PR corrects it.

### Locus-erased atoms by lowering site, ranked

The probe placed 5,856 of the 6,340. The other 484 are module-header segments: the census matches
these against the module's qualified name (`header_pool`), and the probe does not model that pool,
so this row is **reconciled by subtraction, not measured directly**.

| rank | site (function that builds the node) | how the locus is lost | atoms | share | XL-2 population? |
|---:|---|---|---:|---:|---|
| 1 | `body_lower_field_init_edge` (record-literal field names, 1,332); `body_lower_field_pattern_edge` (field names in patterns, 432) | the name becomes a `Named` edge label | 1,764 | 27.8% | **no**: a field name |
| 2 | declaration names of `data` (407), `fn` (294), `test fn` (272), `type` (79): the module body's `Named` edges | label | 1,052 | 16.6% | **no**: a binder |
| 3 | `v2.std.node_query` `construct_tag_edge`, called with `source:` the enclosing captured shell, from `body_lower_try_record_literal` (record tag `Uri { .. }`, 657) and `body_lower_pattern_suffix_lowered` (pattern constructor `Present { .. }`, 353) | the atom exists but carries **the shell's** occurrence, not the tag token's | 1,010 | 15.9% | **yes**: a constructor or type reference |
| 4 | `v2.std.qualified_name` `qualified_name_spine_node`, reached from `body_lowering_fold` `body_lower_qualified_name_spine_node` | one `node_synthetic` atom per segment; the input is `List<Symbol>`, so the segment nodes are already gone | 988 | 15.6% | **yes** for the head segment and type paths; tail field projections are not |
| 5 | module-header segments (namespace graft) | header pool | 484 | 7.6% | **no**: the module's own name |
| 6 | parameter names (265), field declarations (313), variant declarations (236), named-argument labels (47) | label | 861 | 13.6% | **no**: binders and labels |
| 7 | `body_lower_let_expr` let binders | atom on an enclosing shell's occurrence | 98 | 1.5% | **no**: a binder |
| 8 | other labels: primary-position labels (45), postfix field projections (23), pattern qualified names (3) | label | 71 | 1.1% | mixed, at most 48 |
| 9 | `body_lower_method_operator_atom` (`.first` and similar method selectors) | `node_synthetic` | 7 | 0.1% | no: resolved through the receiver's type |
| — | other wrong-source atoms (postfix 5, arg 1) | shell occurrence | 6 | 0.1% | — |

**The reading that matters:**

- Only **three sites** (ranks 3 and 4) rebuild a *reference*. Together they cover **about 2,000
  atoms, 31% of the locus-erased atoms**. The other ~69% are binders and labels, which are not in
  the XL-2 population (§3).
- `node_synthetic` in the lowering fold itself (`body_lowering_fold`) accounts for **7** atoms. The
  brief's picture, where lowering rebuilds nodes as `node_synthetic` inside `body_lowering_fold`, is
  true for one site only: the qualified spine, whose `node_synthetic` is in `v2.std.qualified_name`.
  The single largest reference site is an atom that **is** lowered from a source, just the wrong
  one.
- Locus erasure is not the largest loss on this sample. `dropped` is 18,259 atoms, and 14,787 of
  them are in modules that normalize refuses outright. Those belong to the other `xl2_prerequisites`
  arms (list literals, lambda arguments, caret and `as` operands, else-less `if`). They are **not**
  this carrier's subject, but they weigh more on how exact the XL-2 residual is.

## 2. #11985's mechanism, and the one this carrier reuses

#11985 added `v2.std.node` `node_lowered_from(kind, children, source: Node)`. It builds the new
node with `source.occurrence_id`, taken whole. A lowered node stands for the same authored text at
the same position, so it keeps that occurrence. A source that had no authored origin gives a node
that still has none, so the constructor can only preserve attribution and never invent it.

**`OccurrenceProjected { caused_by }` is not the right carrier, and the RFM row's ceiling text is
wrong to name it.** `std.occurrence_identity` reserves `OccurrenceProjected` for a node minted in a
*different allocator*, meaning an insertion with no counterpart in the base, whose cause must be a
`ScopedOccurrenceRef`. It states outright that "a carried-forward node gets no arm here". Lowering
within one graph owns its source's occurrence, so using `OccurrenceProjected` there would create a
second identity for a fact `OccurrenceMinted` already carries. That is the parallel identity the
brief rules out.

**The one mechanism:** every lowering site that rebuilds an authored atom builds it with
`node_lowered_from(source: <the authored terminal node itself>)`. The reused type is
`std.occurrence_identity` `NodeOccurrenceIdentity`, the `OccurrenceMinted` arm, carried unchanged.
No new type, arm, or index is added. The measurement shows two ways a site gets this wrong, and the
same mechanism fixes both:

1. **The source is the enclosing shell, not the terminal** (rank 3). `construct_tag_edge(tag:
   Symbol, source: captured)` has a source, but it is the whole captured literal. The fix passes the
   tag's own terminal node. That means `body_lower_record_literal_tag_optional` and the
   pattern-suffix reader return the tag **node** instead of `Symbol`, and the tag atom is built with
   `node_lowered_from(source: tag_node)`.
2. **The input no longer holds the nodes** (rank 4). `qualified_name_spine_node(qn: QualifiedName)`
   receives `List<Symbol>`. By then there is nothing to carry, and `node_synthetic` is the honest
   answer. Re-deriving the chain (DESIGN §6b) puts the earliest unjustified boundary at the reader
   that turns segment nodes into symbols (`body_lower_postfix_chain_read` and its `segments`), not
   at the spine builder. The fix carries the segment terminals to the builder, and the builder makes
   each segment atom with `node_lowered_from(source: segment)`.

   `v2.std.qualified_name` `declaration_reference_node`, the marked carrier that resolve mints, keeps
   the occurrence on its marker only, and does so on purpose. That is a separate post-resolve
   decision and this design leaves it alone.

**Climbing from mitigation to construction.** Once each reference site takes a `Node` source, the
constructors that only take a `Symbol` leave the lowering surface. A lowered atom is then built by a
constructor that *requires* its authored source, and "rebuilt without its occurrence" cannot be
written on the lowering route, with no lens needed. `body_lower_type_atom_node(sym, source)` already
has this shape. The remaining writable path is `node_synthetic` used with an `Atom` kind inside
lowering, and the last build PR closes it for reference positions.

## 3. Binders and edge labels: not in the XL-2 population

The XL-2 residual is the set of **resolve refusals after the typed import strip**
(`gunbc.compiler_frontend_program_status` `ExactOccurrenceResidualCensus`). XL-5's waves rewrite
those references. An atom is in the population only if removing imports can make resolve refuse
*at it*.

- **A binder** (the name of a `fn`, `data`, or `type`, a variant declaration, a parameter, a field
  declaration, a `let` name) introduces a name. Stripping imports cannot make it refuse, and no wave
  rewrites it.
- **A field or argument label** (`Uri { uri: .. }`, `f(path: ..)`, `R { value: v } =>`) resolves
  against the declaration of its type or callee. When that declaration is imported and stripped,
  resolve refuses **at the type or callee reference** (rank 3 or 4), not at the label. A namespace
  wave renames modules and declarations; it never renames a field or a parameter.
- **Module-header segments** are the module's own name. The graft consumes them, not resolve.

So `EdgeLabel` (`v2.std.node` `Named { name: Symbol }`) does **not** need an occurrence for XL-2.
Adding one would change a substrate type that participates in structural equality and content
hashing, for a population XL-2 never reads.

There is one real caveat. An occurrence-exact rename of a *declaration*, which XL-5 is not, would
need binder occurrences. That is a separate future carrier and should be modelled when a consumer
exists, not before (DESIGN §3c).

Two consequences for the tracking rows, both corrected in this PR:

- The RFM trigger "`locus_erased` reads zero on the stratified sample" is **over-scoped**: about
  69% of the count is binders and labels that no XL-2 fix will or should move. The capability is
  stated as: *every reference-position atom that lowering rebuilds carries its authored terminal's
  occurrence*.
- Deleting the declaration-scoped spelling pool in `v2.compiler.reference_conservation` needs a
  second thing: binders must be told apart from references **by a grammar fact**, namely the atom's
  role in its production, instead of by a spelling match. Until then the pool can shrink to binders
  and labels, but it cannot be deleted. That work is PR 3 below.

## 4. Rollout order and the regression check

In order of share among in-population atoms:

1. **Constructor tags: `construct_tag_edge` sources** (about 1,010 atoms, 50% of the in-population
   total). Change one shared constructor and its two call sites: record literal and pattern suffix.
2. **Qualified spines: segment nodes reach `qualified_name_spine_node`** (about 988 atoms, 49%).
   This is larger: the chain reader's result type changes, and so does a `v2.std.qualified_name`
   constructor that other callers use.
3. **Census and closure.** Split the census's `locus_erased` into a typed binder/label channel
   (counted, like `import_channel`) and a reference-position count, using the grammar role of the
   atom. Take the `Symbol`-only atom constructors off the lowering route. Shrink the spelling pool.

**The check: one controlled-fixture control per site, not a ratchet over the live sample.** Each
build PR brings a fixture module that exercises its site. A claim runs
`reference_conservation_of_subject` on it and asserts that the site's reference atoms are
`conserved`. The number is grounded in the controlled fixture, so it is a real oracle (DESIGN §5).
The red is run before the fix: on the base, the same claim reports the atom as `locus_erased`. The
positive control is a fixture atom that already conserves, such as a type atom (#11985).

A conserved-share floor over the pinned sample is **not** recommended. That literal would be copied
from the current tree, which makes it a change detector. It is also not a closed universe: six of
its 315 paths have already been deleted. The sample census remains the observation that ranks the
sites, and PR 3 lets it report the split without the probe.

## 5. Size

- **In-population sites to change: 3** (the record-tag call, the pattern-constructor call, and the
  qualified spine), in **2 constructors**: `construct_tag_edge` and `qualified_name_spine_node`.
  Readers that change type: `body_lower_record_literal_tag_optional`, the pattern-suffix tag reader,
  and `body_lower_postfix_chain_read`.
- **Out-of-population sites, recorded but not changed:** 8 label and binder encodings, the
  let-binder atom, and the method-selector atom.
- **PRs: 3**, as in §4, each with its site fixture and a pre-fix red. PR 2 is the largest because
  it changes a `v2.std.qualified_name` signature, so it has to enumerate that constructor's other
  callers first. An optional fourth PR could cover the 7-atom method-selector site, but it is not in
  the population.

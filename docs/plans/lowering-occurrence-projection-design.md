# Lowering occurrence projection (XL-2 prerequisite `LoweringOccurrenceProjection`): design

Status: design only. No compiler behaviour changes in the PR that lands this file. Tracked by
`gunbc.recurring_failure_mode.lowering_rebuilds_an_authored_atom_without_its_occurrence`, carrier arm
`gunbc.compiler_frontend_program_status` `LoweringOccurrenceProjection`.

## 1. What was measured, and how

**The instrument for the totals** is the census:
`v2.compiler.reference_conservation_census` `reference_conservation_census_for_paths`, run over
`reference_conservation_stratified_sample_paths` in batches of 25 paths. The whole sample in one
process does not fit in memory. Its per-module summary is the authority for `conserved`,
`locus_erased`, `dropped` and the channels, and this document does not copy those numbers. The
figures that were measured when this design was written are a dated receipt in the PR that landed
it, not a fact kept here.

**The per-site split has no committed instrument yet, on purpose.** Nothing on a lowered node says
which function built it, so the census cannot attribute an erased atom to a site. The split in the
PR receipt came from a one-off instrumentation:

- Each synthetic-minting site on the normalize route was tagged with a site id carried in
  `OccurrenceProjected`, which the census treats exactly like `OccurrenceSynthetic`.
- A probe recorded which pool entry each locus-erased atom consumed: a rebuilt atom with its tag,
  a `Named` edge label, or an atom lowered from the wrong source.

That tagging is an edit to the compiler, so it cannot be an entry point, and it was not committed
(DESIGN §6: a one-off is not an instrument). **Nothing below depends on its numbers.** §3's
exclusion rests on grammar role alone, and §4's order uses only a coarse size comparison. PR 3 of
§4 makes the split re-derivable without tagging: the census reports each erased atom's grammar role
and its encoding (label, rebuilt atom, or wrong-source atom) as typed dispositions.

### The sites, by grammar role and encoding

Each row names the function that builds the node and how the locus is lost. The classification
is structural, from reading the lowering: which grammar role the atom has, and whether the
lowering encodes it as a label or as a rebuilt atom.

| site | how the locus is lost | grammar role | XL-2 population? |
|---|---|---|---|
| `v2.std.node_query` `construct_tag_edge`, called from `body_lower_try_record_literal` (record tag `Uri { .. }`) and `body_lower_pattern_suffix_lowered` (pattern constructor `Present { .. }`) | the atom exists but is lowered from the **enclosing captured shell**, so it carries the shell's occurrence instead of the tag token's | constructor or type reference | **yes** |
| `v2.std.qualified_name` `qualified_name_spine_node`, reached from `body_lowering_fold` `body_lower_qualified_name_spine_node` | every segment atom is built with `node_synthetic`; the `occurrence_id` parameter reaches only the spine's root, and the input is already `List<Symbol>`, so there is no segment node to carry | dotted reference or type path | **yes** for the head segment and type paths; tail field projections are not |
| `body_lower_field_init_edge`, `body_lower_field_pattern_edge` | the name becomes a `Named` edge label | field name | no |
| the module body's `Named` edges for `data`, `fn`, `test fn` and `type` names; parameter lists; field declarations; variant declarations; named-argument labels | label | binder or label | no |
| `body_lower_let_expr` let binders | atom lowered from an enclosing shell | binder | no |
| module-header segments (namespace graft; the census matches them against its `header_pool`) | header pool | the module's own name | no (see §3; to be proven in PR 3) |
| `body_lower_method_operator_atom` (`.first` and similar method selectors) | `node_synthetic` | method selector, resolved through the receiver's type | no |

**The reading that matters, and it holds without the counts:**

- Only two constructors rebuild a *reference*. Everything else in the table is a binder or a
  label.
- The largest reference site does not use `node_synthetic` at all. Its atom **is** lowered from a
  source, just the wrong one.
- In the one-off measurement, `node_synthetic` inside `body_lowering_fold` itself was a negligible
  site. The loss is in two `v2.std` constructors and in the readers that feed them.
- On the pinned sample, `dropped` is larger than `locus_erased`. The census shows this directly:
  most of it is in modules that normalize refuses, which belong to other `xl2_prerequisites`
  arms, and the rest is atoms `absent` from modules that normalize accepts.

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

- The RFM trigger "`locus_erased` reads zero on the stratified sample" is **over-scoped**: most of
  that count is binders and labels, which no XL-2 fix will or should move. The capability is
  stated as: *every reference-position atom that lowering rebuilds carries its authored terminal's
  occurrence*.
- Deleting the declaration-scoped spelling pool in `v2.compiler.reference_conservation` needs a
  second thing: binders must be told apart from references **by a grammar fact**, namely the atom's
  role in its production, instead of by a spelling match. Until then the pool can shrink to binders
  and labels, but it cannot be deleted. That work is PR 3 below.

## 4. Rollout order and the regression check

In order of size and risk:

1. **Constructor tags: `construct_tag_edge` sources.** Change one shared constructor and its two
   call sites: record literal and pattern suffix. It goes first because it is the smallest change.
   In the one-off measurement it was about the same size as the spine site.
2. **Qualified spines: segment nodes reach `qualified_name_spine_node`.** This is larger: the chain
   reader's result type changes, and so does a `v2.std.qualified_name` constructor that other
   callers use.
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

# Anonymous record literals: the tag-absent construct, elaborated from the expected type

Status: PR1 of two (the model and its consumer census). PR2 is the cut. Owner lane: sleek-fox-423,
parent gentle-koi-724, reviewer neat-boar-16. This builds on the construct carrier of
`docs/plans/construct-tag-reference-design.md` (gunbc#12701, lively-eagle-657) and does not restate
it.

## Symptom and the chain (DESIGN 6b)

A headless brace literal, `data x: T = { f: v }` or a nested `f: { g: w }`, file-refuses at native
ingest with `body_lowering_reason_unsupported_form`. A refused file drops its declarations from the
native index, so every importer then fails as unbound. The survey (lively-eagle-657, quiet-hawk-702
probe on the #12420 tree) names seven files.

The slice, read from the grammar forward:

- **Grammar.** `v2.extdeps.languages.dag` `dag_grammar_primary_expr_core` has ONE headless-brace
  alternative, `{ field_init_list }`. `dag_grammar_field_init_expr` keys each item on EITHER a
  binding name OR a string literal. The record literal `{ f: v }` and the map literal
  `{ "k": v }` are the same production. Only the lexical kind of each key tells them apart.
- **Lowering.** `v2.compiler.body_lowering_fold` `body_lower_try_record_literal` needs a head tag
  from `body_lower_record_literal_tag_optional`. A headless brace has none, so the primary is
  unreadable and refuses at the primary (the XL-2 refusal arm,
  `unrecognized_primary_expression_lowers_to_its_first_atom_at_v2_body_lowering`). This refusal is
  CORRECT at this link. Lowering runs before resolve and before infer, so it cannot know the
  constructor. The earliest unjustified boundary is therefore not lowering. It is the construct
  carrier, which has no tag-absent case, together with the absence of any stage that carries a
  declared type DOWN into the expression it types.
- **No stage pushes a declared type down.** v2 infer is a bottom-up fold. Its declared-type
  judgments run AFTER a body is synthesised: `infer_arrow_body_inhabits_declared_return` compares a
  derived body type with a declared return (cause `arrow_body_does_not_inhabit_declared_return`),
  and only for a return the language join denotes (`dag_binding_denotation`). Call arguments are
  judged the same way (`position_direct_call_argument`). A tag-elided construct cannot be
  synthesised at all, because its tag is what types it. So its tag must be supplied BEFORE infer's
  fold reaches it, by a stage that walks top-down with the declared type as context.
- **The seed is not an oracle for the choice.** v1 chooses the struct in its Rust emitter by
  matching field names: `v1.compiler.emit_rust` `anonymous_record_struct_candidates`, and it
  refuses `AmbiguousAnonymousRecordLiteral` ("shape matches N structs -- add a nominal type").
  That is the selection this lane is forbidden to make. So seed-vs-native agreement on the chosen
  tag is not evidence. The oracle is content equality with the authored headed form (control 1).

## The model

**The carrier.** Form B is the tag-ABSENT case of the one construct carrier of #12701. It is not a
new node kind. After #12701 a construct is `Conj { <construct_tag_marker>: <reference>, fields.. }`.
The tag-absent construct is `Conj { <construct_tag_marker>: <construct_tag_elided_marker>, fields.. }`.

- The tag edge is always present, so every construct reader keys on the same first edge and asks
  only whether its target is a reference or the elision marker. The alternative, a construct with
  no tag edge, is a bare `Conj` of Named edges. That is also the anonymous record TYPE
  (`body_lower_anonymous_record_type_optional`, `-> { success: Bool }`) and the shape every
  "unmarked Conj" reader already means something else by. Presence-by-absence would be a second,
  implicit representation. It is refused for the same reason #12701 keeps `construct_tag_marker`
  distinct from `declaration_reference_marker`.
- `construct_tag_elided_marker` is a structural core marker declared beside `construct_tag_marker`
  in `v2.std.node`, in the same form, and counted once in the #12473 `EdgeLabel` census. It
  carries no spelling and no guess.
- **Key kind is recorded, not interpreted.** Each field edge keeps its authored key kind. A name key
  is `Named { name }`, as today. A string key has no field-edge form yet (see Scope). Lowering does
  not decide "record" versus "map". It records what was written.

**Where elaboration happens: in resolve, at the construct-tag writer, with an expected-type
context threaded down from declared types.** (Revised after the first review. The first draft
placed it in infer "before synthesis". Infer is a bottom-up fold, so that would have been a second,
top-down pass beside it.)

Choosing the tag reads exactly two facts, and both are NAME facts that resolve already has:

- The expected type's HEAD resolves to a declaration. Resolve resolves every type expression.
- Whether that declaration is a single record, and, for nesting, its fields' declared types. Resolve's
  `ResolveContext.namespace.symbol_index` returns the declaration node through `symbol_index_lookup`,
  and `v2.std.type_binder` `type_decl_view` reads its shape.

Resolve is also, after #12714, the ONE writer of construct tags: `resolve_construct_walk` rewrites
the authored tag spine into `declaration_reference_node(path)`. Writing the elided tag anywhere else
would give one fact, "the declaration this construct constructs", two writers (§3). So the elided
case is a second arm of that same writer, with the reference derived from the expected type rather
than from an authored spine. Infer then sees only ordinary resolved constructs, and checks them
with its existing construct typing and declared-return judgment.

**The expectation is a parameter, not ambient context.** An expectation (`ResolveExpectation`: the
authored type expression, and the position its names are read at) is handed, as an explicit
argument, by exactly the four position rules below to the one child each rule owns
(`resolve_expected_edge`). It is never a `ResolveContext` field. Every other walk goes through
`resolve_node_walk`, which has no expectation to give, and a headless literal there refuses. That
makes review condition 3 structural rather than a clearing discipline: a field on the context would
have been inherited by every child walk that copies the context (a call's arguments, for one), and
each would have had to remember to clear it.

**The expectation is set ONLY from an AUTHORED type annotation** (ruling, gentle-koi-724): a data
declaration's declared type, a fn's declared return, a record field's declared type, or the element
type of an annotated list. Resolve never infers, unifies or propagates a type it computed. Where no
annotation reaches an elided construct, it refuses located (`resolve_anonymous_record_no_expected_type`)
and never guesses. A record field's declared type is an annotation authored on the record
declaration, so nesting stays within this rule. **Syntactic peel only** (review condition 1, neat-boar-16, binding both arms). The expectation is peeled
syntactically from the authored annotation. Where reaching a closed record head would need
inference, type-variable instantiation, alias unfolding beyond resolve's existing lookup, or any
head that is not a closed record (or `Map`, for the map arm), the construct REFUSES located. There
is no unification and no substitution in resolve. A record field whose declared type is one of the
record's own binders (`top: T` of `BoundedLattice<T>`) therefore gives no record head, and a headless
literal there refuses. The four positions:

- **data initializer**: `data x: T = e` lowers to `Arrow(<empty domain>, T, T, body: e)`, so the
  body edge of an Arrow is walked with its declared return as the expectation;
- **declared return**: the same Arrow rule covers `fn f(..) -> T { e }`, reaching `e`'s tail
  expression through blocks and `let .. in` bodies (the value position, not the statements);
- **record field**: under a construct resolved to record `R` (authored OR elaborated), field `f`'s
  init is walked with the declared type of `R.f` as the expectation, as authored (no substitution). This is how nesting works: `compile_stage_memo`'s
  `key_derivation: { .. }`, and `host_transport`'s `RuntimePrimitive { value: { .. } }`;
- **list element**: under a list literal whose expected head is `List` (through its alias
  authority), each element is walked with the element type argument as the expectation.
  `dag/std/algebra`'s 88 `AlgebraFieldTemplate` rows need this.

At a tag-elided construct, the elided arm of the writer:

1. Takes the expectation. If there is none, refuse `resolve_anonymous_record_no_expected_type`.
2. Takes its HEAD's resolved declaration, through resolve's existing lookup only. Type arguments
   (`BoundedLattice<DescentEvidence>`) never choose the constructor, and they are not substituted
   anywhere. If the head does not resolve, the head's own resolve refusal stands, and no second
   cause is minted for it. A head that is a type binder, or that reaches a record only through
   alias unfolding the lookup does not already do, refuses
   `resolve_anonymous_record_expected_type_not_record`.
3. Requires the declaration to be a single record (`type_decl_view` `PlainTypeDecl` whose member is a
   product), or `Map` (the MAP arm, which this lane refuses located; see Scope). Anything else,
   whether a multi-variant coproduct, a primitive, or `List`, refuses
   `resolve_anonymous_record_expected_type_not_record`.
4. Writes `declaration_reference_node(path)` of that declaration through `resolved_reference_node`,
   the same carrier the authored arm writes.
5. Walks the fields under the record-field rule above.

The choice reads the expectation only. The field set is never an input to steps 1-3.

**Resolve's choice is not proof** (review condition 2). Infer CHECKS the elaborated construct
against the declared type exactly as it checks an authored tag: field names and field value types
through its existing construct typing, and the declared return through
`infer_arrow_body_inhabits_declared_return`. A wrong field or a wrong value type refuses in infer, at
the field.

**Refusal causes** (typed, located at the literal's `{`, each with an ownership row in
`v2.workflow.compile_door_cause_ownership`, per
`refusal_producer_lands_without_its_ownership_row`):

- `resolve_anonymous_record_no_expected_type`: the construct sits in a position with no declared
  type. Examples: a `let` without an annotation, a call argument, a match-arm body, a list literal
  that itself has no expected type. These are not guessed. Call arguments stay here on purpose:
  their formal's declared type is a fact about the CALLEE, whose resolution this position does not
  own. Widening to them is a named follow-up, not an implicit one.
- `resolve_anonymous_record_expected_type_not_record`: the expected head resolves to something other
  than a single record or `Map`.
- `map_literal_construction_not_modeled`: the expected head is `Map`. This is the MAP arm, and it
  plugs into the same writer.

A tag-elided construct that survives resolve is unwritable downstream. The emit, eval and target_model
readers gate on a reference target, and the elision marker is not one. So a missed elaboration
refuses at their existing gates, and cannot emit an anonymous struct.

## As built in PR2 (gunbc#12740)

The model above was refined while the cut was reviewed. Each refinement below is the authority that
the PR2 code cites by section name.

### One reader of the construct tag

`v2.std.node_query` `construct_tag_reading` answers
`ConstructTagAuthored { reference } | ConstructTagElided | NotAConstruct`. Every reader (body
lowering's core-substrate test, resolve's construct-tag writer, the claims) matches on that one
answer. `construct_tag_optional` is its projection. The elision is carried as an EDGE LABEL
(`construct_tag_elided_marker`) on the tag's target, the mechanism the tag marker itself uses. A
marker carried as an atom's identity did not survive to resolve. And body lowering must count an
elided construct as core substrate, or its fold re-lowers the literal to its first atom.

### The string key is a grammar production

`dag_grammar_field_init_expr` is a choice between a name alternative and a production of its own,
`^dag_production_field_init_string_key` (emitted `^dag_surface_field_init_string_key`), whose key is
the class-stamped string terminal that gunbc#12759 decodes. So the key's kind is a fact of the parse
tree, read by its production identity (`body_lower_field_init_is_string_keyed`), and never recovered
from a spelling. A record literal refuses a string key at the item (`field_init_unlowered`). The
occurrence-role row of the new production reads its names as references, and the field-init reader
answers `NoNameHere` for it. The map arm (gunbc#12758) dispatches on the same identity. Admitting a
quoted key whose DECODED text names a declared field is a declared frontier. It is a follow-up in the
record arm, and it also carries the quoted-key round trip; the first lane that admits quoted keys
(gunbc#12758) carries that round trip for maps.

### The head resolves the way the annotation resolves

An annotation written in the literal's own module resolves its head through `resolve_atom` in the
literal's context, the same route the annotation itself takes, so the two cannot bind different
declarations. A field type is written in the record's own module, so it is read at the record's path
through the index's lexical lookup.

### The list-element rule

A list literal passes its element type to its elements only when the annotation's head reaches the
declaration infer types list literals with (`v2.std.list_introduction`
`list_introduction_head_path`, `std.algebra.FreeMonoid`).

### Pure-renaming aliases

`List` is an alias (`type List<element> = FreeMonoid<element>`), so the list rule needs one kind of
alias to be followed. Exactly one kind is followed: a PURE RENAMING, `type A<p1..pn> = H<p1..pn>`
(the aliased expression is a head applied to exactly the alias's own binders, in declared order) or
`type A = H`. Following one substitutes nothing, because the annotation's arguments stay in the
positions they were written.

- Purity is decided ONCE, on the declaration (`v2.std.type_binder` `type_alias_renaming`). It is
  recorded when the index records the declaration (`v2.std.symbol_index` `renaming_aliases`,
  `symbol_index_renaming_alias_at`), and resolve reads the recorded fact, never re-deriving it at a
  use.
- Chains are followed transitively, each renamed head read at the alias's own position. A revisited
  path refuses located (`resolve_anonymous_record_alias_cycle`).
- A reordering, constant or partially applied alias is not a renaming. It is the declaration the head
  names, so the literal refuses.

## Consumers (producers, readers, rewriters)

Stated as a delta on #12701's census. Every row there still applies; the rows here are the ones
the elided case adds.

| Module | Symbol | Role | Change |
|---|---|---|---|
| v2.std.node | `construct_tag_elided_marker` | vocabulary | New structural core marker beside `construct_tag_marker`. |
| v2.std.node_query | `construct_node` | producer | Takes the tag target as the reference OR the elision marker (one parameter; a typed `ConstructTag = Authored { reference } \| Elided` at the call boundary so a caller cannot pass an arbitrary node). |
| v2.std.node_query | `construct_tag_optional` | reader | Answers the reference only. For an elided tag it answers Absent. `construct_tag_is_elided` is the one reader of the elided case. |
| v2.std.node | `arrow_body_record_construct_conforms`, `classify_arrow_body_form` | well_formed gate | Admits the elided marker ONLY before resolve. After resolve the well_formed gate refuses it. |
| v2.compiler.body_lowering_fold | `body_lower_try_record_literal` | producer | A headless `{ field_init_list }` whose every key is a name lowers to `construct_node(Elided, ..)`. The field inits go through the existing `body_lower_field_init_edge`. |
| v2.compiler.resolve | `ResolveExpectation`, `resolve_expected_edge`, `resolve_arrow_body_or_named_edge`, `resolve_record_field_edge`, `resolve_list_literal_expected` | position rules | The expectation parameter and the four rules that hand it to one child each. `ResolveContext` is unchanged. |
| v2.compiler.resolve | `resolve_construct_walk`, new `resolve_construct_walk_with_tag`, `resolve_elided_construct_walk` | writer | The tag is walked once before the fields, and its resolved path is handed to the field rule, so a headless literal nested under an authored head elaborates. The elided arm writes the tag from the expectation (steps 1-5). In `resolve_node_walk`, an elided construct refuses `resolve_anonymous_record_no_expected_type`. |
| v2.compiler.resolve | `resolve_pattern_node_walk` | reader | A tag-elided PATTERN does not arise (the pattern grammar requires a head). No change. |
| v2.compiler.infer | Conj gather arms | reader | No change. A tag-elided construct cannot reach infer, because resolve either writes its tag or refuses. |
| v2.workflow.compile_door_cause_ownership | three rows | ownership | One row per refusal cause. |
| v2.test.claim.namespace_xl0.reference_conservation_accepted_drops | `a_map_literal_value_refuses_at_the_literal_holds` | control | Today it asserts `unsupported_form` for `Map<String, Bool> = { "k": .. }`. UNCHANGED by this lane: a string-keyed item still refuses at lowering here. It moves with the map arm (gunbc#12734, jolly-boar-246), which gives string keys an edge form and must then read the RESOLVE outcome. |
| v2.compiler.eval, v2.std.compilers.target_model, emit | construct gates | reader | No change. They already refuse a non-reference tag after #12701. PR2 adds a control proving an elided construct cannot reach them. |

## Scope: which of the seven Form B admits

THE READING THAT SET THIS LANE'S SCOPE, not a standing fact. It comes from a one-off probe on main
7f144278c57: each file's source was run through the parse and normalize route of
`v2.test.claim.namespace_xl0.reference_conservation` `conservation_subject`, and the first fatal was
rendered through the parse's `SpanIndex`. Only the FIRST fatal per file is shown, so a file can have
later blockers. The standing instrument is the #12550 base-vs-head native census, run on PR2's head.
It re-derives which of these files are admitted, and PR2 cites that census rather than this table
(DESIGN §6: name the instrument, never transcribe its output).

| File | First fatal refusal (line) | Form | Form B admits it? |
|---|---|---|---|
| dag/std/algebra | 359, `{ name: "filter", .. }` | the body of `fn collection_filter_shape() -> AlgebraFieldTemplate`; 88 more rows of the same record inside list literals | yes: declared return, and list element |
| dag/std/primitives | 17, `data char_at_contract: PrimitiveContract = {` | data initializer | yes |
| dag/std/termination | 45, `data .. : BoundedLattice<DescentEvidence> = {` | data initializer, generic, newline-separated fields | yes, by HEAD lookup: `BoundedLattice` resolves to a single record. Its type argument is not substituted and chooses nothing, and none of its field inits is headless. The newline separator is to be confirmed in PR2. |
| dag/extdeps/realization/compile_stage_memo | 39, `key_derivation: {` | record field under a headless data initializer (lowering refuses the inner brace first) | yes: data initializer plus record field |
| dag/extdeps/realization/parse_table_memo (since deleted with the parse memo carrier, operator ruling R2 2026-10-08) | 50, `key_derivation: {` | same | yes |
| dag/std/types | 5, `data kernel_type_set: Map<String, Bool> = {` | MAP literal | **no: MAP arm.** Reaches the elided-tag writer in resolve and refuses located `map_literal_construction_not_modeled`, as a declared population (below). |
| src/v2/std/host_transport | about 119, `reason: ^emit_host_runtime_row_..` | **caret symbol literal** (`body_lowering_reason_caret_symbol_not_lowered`), not Form B | **not by Form B alone.** Its first blocker is the caret form. Its Form B site (`RuntimePrimitive { value: { .. } }`, record field) is admitted by this lane, but the file stays refused until caret symbols lower. That is #12420 (vivid-ant-536), so PR2 lists it as depending on #12420. |

**The map arm (ruling, gentle-koi-724).** This lane builds the ONE elided-tag writer (in resolve) and its
RECORD arm. At that same site, a headless brace whose expected head is `Map` takes the MAP arm, which
refuses located `map_literal_construction_not_modeled`, with its ownership row. It is not left as
`unsupported_form`, and there is no second site later: map construction plugs into this arm.

For lowering to reach the site, a string-keyed item needs a field-edge form. It lowers to the same
tag-elided construct, with each item carried as a `map_literal_entry_marker` edge to a
`(key, value)` pair. Lowering still does not pick map or record; a construct that mixes name keys
and string keys refuses at lowering, located at the first key of the other kind.

The refusing population is DECLARED, as a `gunbc.rung_drop` row. Population: `dag/std/types` plus
every other map-literal file the base census locates, named. Restoration trigger: map construction
is modeled, so a `Map<K, V>` expected type elaborates the literal to a value that inhabits it, linked
to the open question on the XL-2 row
(`unrecognized_primary_expression_lowers_to_its_first_atom_at_v2_body_lowering`).

If `dag/std/types` staying refused still blocks the native seven transitively, the map arm is on the
critical path; the parent dispatches it.

**The seed.** The seed choosing by field names is not an oracle. If it chooses WRONGLY for a real
file, meaning the chosen struct differs from the declared type, that is a silent seed defect, and PR2
files a `gunbc.recurring_failure_mode` receipt for it.

## PR2 (the cut) and its controls

1. `data x: T = { f: v }` lowers, resolves and elaborates to a node whose `content_hash` equals that
   of `data x: T = T { f: v }`. Content equality with the headed form. The authored-tag token occurrence
   is the one expected difference. `v2.std.node` `content_hash` folds only kind, edge labels and
   children, so it should fall outside the hash. PR2 confirms this by execution. If it does not, the
   control compares the occurrence-erased projection, and says so.
2. A tag-elided literal with no expected type (a `let` without an annotation) refuses
   `resolve_anonymous_record_no_expected_type`, located at its `{`.
3. The expected type is a two-constructor coproduct: refuses `resolve_anonymous_record_expected_type_not_record`.
4. Fields that do not match the chosen record refuse at the field, through the existing construct
   typing, and not as "no constructor".
5. **Mutation: choose by field names.** Two records share the literal's field set. The expected
   type names the one declared SECOND. The control asserts the declared type's path is chosen. A
   field-name chooser picks the first or refuses as ambiguous, so it goes red either way.
6. Nesting: `host_transport`'s form, and a two-level `compile_stage_memo` form, each equal to their
   hand-headed form.
7. **Infer still checks** (review condition 2): elaboration succeeds, and infer then refuses a field
   value of the wrong type, located at the field.
8. **The expectation does not leak** (review condition 3): an anonymous literal passed as a call argument
   under an annotated data declaration refuses `resolve_anonymous_record_no_expected_type`. A second
   case puts it in a match-arm body under an annotated fn return.
9. **Syntactic peel** (review condition 1): a headless literal at a record field declared as the
   record's own binder (`top: T`) refuses `resolve_anonymous_record_expected_type_not_record`.
10. An elided construct cannot pass the post-resolve well_formed gate or the target_model gate
   (a discriminating pair with the headed form).
11. Evidence: the #12550 base-vs-head native census shows each Form B file of the table admitted,
   with every other outcome change explained. Seed claims show no true->false.

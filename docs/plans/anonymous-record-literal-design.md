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

Resolve gains an expected-type context: `ResolveContext.expected`, an `Optional` resolved type
expression. It is set at four positions and cleared everywhere else, so a construct reached through
any other edge sees Absent.

**`expected` is set ONLY from an AUTHORED type annotation** (ruling, gentle-koi-724): a data
declaration's declared type, a fn's declared return, a record field's declared type, or the element
type of an annotated list. Resolve never infers, unifies or propagates a type it computed. Where no
annotation reaches an elided construct, it refuses located (`resolve_anonymous_record_no_expected_type`)
and never guesses. A record field's declared type is an annotation authored on the record
declaration, so nesting stays within this rule. **Syntactic peel only** (review condition 1, neat-boar-16, binding both arms). `expected` is peeled
syntactically from the authored annotation. Where reaching a closed record head would need
inference, type-variable instantiation, alias unfolding beyond resolve's existing lookup, or any
head that is not a closed record (or `Map`, for the map arm), the construct REFUSES located. There
is no unification and no substitution in resolve. A record field whose declared type is one of the
record's own binders (`top: T` of `BoundedLattice<T>`) therefore gives no record head, and a headless
literal there refuses. The four positions:

- **data initializer**: `data x: T = e` lowers to `Arrow(<empty domain>, T, T, body: e)`, so the
  body edge of an Arrow is walked with `expected = ` its declared return;
- **declared return**: the same Arrow rule covers `fn f(..) -> T { e }`, reaching `e`'s tail
  expression through blocks and `let .. in` bodies (the value position, not the statements);
- **record field**: under a construct resolved to record `R` (authored OR elaborated), field `f`'s
  init is walked with `expected = ` the declared type of `R.f`, as authored (no substitution). This is how nesting works: `compile_stage_memo`'s
  `key_derivation: { .. }`, and `host_transport`'s `RuntimePrimitive { value: { .. } }`;
- **list element**: under a list literal whose expected head is `List` (through its alias
  authority), each element is walked with `expected = ` the element type argument.
  `dag/std/algebra`'s 88 `AlgebraFieldTemplate` rows need this.

At a tag-elided construct, the elided arm of the writer:

1. Takes `expected`. If it is Absent, refuse `resolve_anonymous_record_no_expected_type`.
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

The choice reads `expected` only. The field set is never an input to steps 1-3.

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
| v2.compiler.resolve | `ResolveContext` | context | Gains `expected: Optional<Node>`. It is set by the four position rules above and cleared by every other child walk (`resolve_ctx_with_scope` and siblings clear it by default, so a new edge kind cannot inherit it silently). |
| v2.compiler.resolve | `resolve_construct_walk` | writer | Gains the elided arm (steps 1-5). The authored arm also sets field expectations, so a headless literal nested under an authored head elaborates. |
| v2.compiler.resolve | `resolve_arrow_node_in`, the list-literal walk | position rules | Set `expected` for the body edge and for list elements. |
| v2.compiler.resolve | `resolve_pattern_node_walk` | reader | A tag-elided PATTERN does not arise (the pattern grammar requires a head). No change. |
| v2.compiler.infer | Conj gather arms | reader | No change. A tag-elided construct cannot reach infer, because resolve either writes its tag or refuses. |
| v2.workflow.compile_door_cause_ownership | three rows | ownership | One row per refusal cause. |
| v2.test.claim.namespace_xl0.reference_conservation_accepted_drops | `a_map_literal_value_refuses_at_the_literal_holds` | control | Today it asserts `unsupported_form` for `Map<String, Bool> = { "k": .. }`. It moves to `map_literal_construction_not_modeled` at the elided-tag writer in resolve. It must read the RESOLVE outcome, because the literal now clears lowering, so the `.normalized` read would turn green and the claim false. It stays enrolled as the map arm's refusal control. |
| v2.compiler.eval, v2.std.compilers.target_model, emit | construct gates | reader | No change. They already refuse a non-reference tag after #12701. PR2 adds a control proving an elided construct cannot reach them. |

## Scope: which of the seven Form B admits

MEASURED, on main 7f144278c57. Each file's source ran through the same parse and normalize route as
`v2.test.claim.namespace_xl0.reference_conservation` `conservation_subject`. The fatal diagnostic was
rendered through the parse's `SpanIndex`. Only the FIRST fatal refusal is reported per file, so a
file can have later blockers beyond its first.

| File | First fatal refusal (line) | Form | Form B admits it? |
|---|---|---|---|
| dag/std/algebra | 359, `{ name: "filter", .. }` | the body of `fn collection_filter_shape() -> AlgebraFieldTemplate`; 88 more rows of the same record inside list literals | yes: declared return, and list element |
| dag/std/primitives | 17, `data char_at_contract: PrimitiveContract = {` | data initializer | yes |
| dag/std/termination | 45, `data .. : BoundedLattice<DescentEvidence> = {` | data initializer, generic, newline-separated fields | yes, via generic instantiation. The newline separator is to be confirmed in PR2. |
| dag/extdeps/realization/compile_stage_memo | 39, `key_derivation: {` | record field under a headless data initializer (lowering refuses the inner brace first) | yes: data initializer plus record field |
| dag/extdeps/realization/parse_table_memo | 50, `key_derivation: {` | same | yes |
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
8. **`expected` does not leak** (review condition 3): an anonymous literal passed as a call argument
   under an annotated data declaration refuses `resolve_anonymous_record_no_expected_type`. A second
   case puts it in a match-arm body under an annotated fn return.
9. **Syntactic peel** (review condition 1): a headless literal at a record field declared as the
   record's own binder (`top: T`) refuses `resolve_anonymous_record_expected_type_not_record`.
10. An elided construct cannot pass the post-resolve well_formed gate or the target_model gate
   (a discriminating pair with the headed form).
11. Evidence: the #12550 base-vs-head native census shows each Form B file of the table admitted,
   with every other outcome change explained. Seed claims show no true->false.

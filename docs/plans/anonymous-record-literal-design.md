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
  carrier, which has no tag-absent case, together with typing, which has no site that checks a body
  against its declared type.
- **Typing has no check site.** `v2.std.inhabitance` `DeclaredTypePosition` declares
  `PositionDeclaredReturn`, but nothing constructs it. `v2.compiler.infer` builds only
  `position_direct_call_argument` obligations. v2 infer synthesises bottom-up and never pushes a
  declared type into a body. So "typing, which checks against the declared type" does not exist
  in v2 yet. Form B is the first consumer that needs it. The obligation vocabulary already has a
  home for it (`DeclaredTypeObligation`, `declared_type_inhabitance`). The v1 seed carries the
  missing positions by name (`src/v1/04_infer.dag` `PositionDataInitializer`,
  `PositionVariantPayload`).
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

**Where elaboration happens: at a declared-type check site in infer, before synthesis.** This is
the one place where both facts meet: the expected type, which only a declared position carries,
and resolved declarations, which infer's facts carry. The check site is a `DeclaredTypeObligation`
at a named position. A position is where a declared type meets a produced expression:

- `PositionDataInitializer`: `data x: T = e`. New arm, v1 vocabulary.
- `PositionDeclaredReturn`: `fn f(..) -> T { e }`. The arm exists. This change gives it a
  constructor.
- `PositionRecordField`: the expected type of a field init is the declared type of that field on
  the (authored or elaborated) constructor. This is how nesting works. `host_transport`'s
  `RuntimePrimitive { value: { .. } }` elaborates the inner literal from `RuntimePrimitive.value`'s
  declared type, and `compile_stage_memo`'s `key_derivation: { .. }` elaborates from the outer
  record's field. New arm.
- `PositionListElement`: the expected type of a list literal's element is the element type of the
  list's own expected type (`List<T>` gives `T`). `dag/std/algebra` needs this: its 88
  `AlgebraFieldTemplate` rows are headless literals inside list literals. New arm, v1 vocabulary.
  A list literal with no expected type passes none down, so its headless elements refuse
  `infer_anonymous_record_no_expected_type`.

At each such site, before the body is synthesised, a tag-elided construct at the root of `e` is
elaborated. Elaboration recurses through field inits under `PositionRecordField`. It is:

1. Resolve the expected type's HEAD to a declaration. Type arguments (`BoundedLattice<DescentEvidence>`)
   do not choose the constructor; they flow into the field expected types by the existing
   instantiation (`TypeVariableInstance`). A transparent alias is chased through the existing
   alias-identity authority, not re-implemented.
2. The declaration must have exactly ONE record-shaped constructor: a `type R { .. }` record. The
   single-variant coproduct form is admitted only if the existing record/coproduct classification
   already names it one constructor.
3. Replace `construct_tag_elided_marker` with `declaration_reference_node(path)` of THAT
   declaration, through `resolved_reference_node`. This is the same carrier resolve writes for an
   authored head.
4. Synthesis then proceeds on an ordinary resolved construct. Field names are checked against the
   chosen constructor by the existing construct typing (#12701's infer migration). A surplus or
   missing field refuses there, at the field.

The choice reads the expected type only. The field set is never an input to step 1 or step 2. Its
only role is step 4, which CHECKS a choice already made.

**Refusal causes** (typed, located at the literal's `{`, each with an ownership row in
`v2.workflow.compile_door_cause_ownership`, per
`refusal_producer_lands_without_its_ownership_row`):

- `infer_anonymous_record_no_expected_type`: a tag-elided construct reached synthesis without
  passing through a check site. Examples: a call argument whose formal is a type variable not yet
  instantiated, a `let` without an annotation, a match-arm body. These are not guessed.
- `infer_anonymous_record_expected_type_not_record`: the expected head resolves to a coproduct with
  more than one constructor, a primitive, or `Map`/`List` while the literal has name keys.
- `infer_anonymous_record_expected_type_unresolved`: the expected head does not resolve. This is
  distinct from the above so that a missing import is not reported as a shape error.

A tag-elided construct that survives infer is unwritable downstream. The emit, eval and target_model
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
| v2.std.node | `arrow_body_record_construct_conforms`, `classify_arrow_body_form` | well_formed gate | Admits the elided marker ONLY pre-infer. The post-infer well_formed gate refuses it. |
| v2.compiler.body_lowering_fold | `body_lower_try_record_literal` | producer | A headless `{ field_init_list }` whose every key is a name lowers to `construct_node(Elided, ..)`. The field inits go through the existing `body_lower_field_init_edge`. |
| v2.compiler.resolve | `resolve_node_walk`, `resolve_pattern_node_walk` Conj arm | rewriter | The elided tag edge is passed through untouched (there is nothing to resolve). Field values walk as today. |
| v2.std.inhabitance | `DeclaredTypePosition` | vocabulary | Add `PositionDataInitializer`, `PositionRecordField` and `PositionListElement`. `PositionDeclaredReturn` gains its constructor. |
| v2.compiler.infer | new `infer_elaborate_expected_construct` | elaborator | Steps 1-4 above, called from the data-initializer, declared-return, record-field and list-element check sites. It is the only writer that replaces the elision marker. |
| v2.compiler.infer | Conj gather arms | reader | A tag-elided construct reached here refuses `infer_anonymous_record_no_expected_type` (it is never typed as a bare product). |
| v2.workflow.compile_door_cause_ownership | three rows, plus `map_literal_construction_not_modeled` | ownership | One row per refusal cause. |
| v2.test.claim.namespace_xl0.reference_conservation_accepted_drops | `a_map_literal_value_refuses_at_the_literal_holds` | control | Today it asserts `unsupported_form` for `Map<String, Bool> = { "k": .. }`. It moves to `map_literal_construction_not_modeled` at the check site. It must read the INFER outcome, because the literal now clears lowering and the `.normalized` read would go green-then-false. It stays enrolled as the map arm's refusal control. |
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
| dag/std/types | 5, `data kernel_type_set: Map<String, Bool> = {` | MAP literal | **no: MAP arm.** Reaches the check site and refuses located `map_literal_construction_not_modeled`, as a declared population (below). |
| src/v2/std/host_transport | about 119, `reason: ^emit_host_runtime_row_..` | **caret symbol literal** (`body_lowering_reason_caret_symbol_not_lowered`), not Form B | **not by Form B alone.** Its first blocker is the caret form. Its Form B site (`RuntimePrimitive { value: { .. } }`, record field) is admitted by this lane, but the file stays refused until caret symbols lower. |

**The map arm (ruling, gentle-koi-724).** This lane builds the ONE declared-type check site and its
RECORD arm. At that same site, a headless brace whose expected head is `Map` takes the MAP arm, which
refuses located `map_literal_construction_not_modeled`, with its ownership row. It is not left as
`unsupported_form`, and there is no second check site later: map construction plugs into this arm.

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
   `infer_anonymous_record_no_expected_type`, located at its `{`.
3. The expected type is a two-constructor coproduct: refuses `..._expected_type_not_record`.
4. Fields that do not match the chosen record refuse at the field, through the existing construct
   typing, and not as "no constructor".
5. **Mutation: choose by field names.** Two records share the literal's field set. The expected
   type names the one declared SECOND. The control asserts the declared type's path is chosen. A
   field-name chooser picks the first or refuses as ambiguous, so it goes red either way.
6. Nesting: `host_transport`'s form, and a two-level `compile_stage_memo` form, each equal to their
   hand-headed form.
7. An elided construct cannot pass the post-infer well_formed gate or the target_model gate
   (a discriminating pair with the headed form).
8. Evidence: the #12550 base-vs-head native census shows each Form B file of the table admitted,
   with every other outcome change explained. Seed claims show no true->false.

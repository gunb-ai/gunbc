# The dag target emits by reading `dag_grammar_root` backward

Status: **implemented** in gunbc#12878 (`v2.std.grammar` `grammar_emit_parse_tree`, `v2.extdeps.languages.dag` `dag_emit_parse_tree`) (work item adhoc-da997c00-fa0). This page states how emit selects a
production for a Node shape and how each terminal's token class is recovered. It was written as the
model before code and accepted, and the implementation follows it except where a section below says
otherwise.

## The defect being replaced

DESIGN §4 says there is one grammar, read in both directions. For the v2 dag target that is
currently false (measured by tidy-lark-233 on main `cc2fefacac`):

- `v2.extdeps.languages.dag` `dag_translation_rules_node` carries one row, the `fn add` fixture.
- `token_class_emit_transforms` is `target_model_emit_transforms_empty`.
- The `emit_row_*` functions have no consumer outside the module.
- `data m: Int = 1` refuses with `v2.compiler.ingest` `parse_tree_atom_token_class_not_recoverable`.
- The only declaration round trip,
  `v2.test.execution.emit_ingest_type_decl_round_trip`, runs on
  `dag_type_decl_structural_formal_productions` and its own lexer
  (`dag_type_decl_structural_lex_rules`). That is a second, hand-authored grammar for one
  language, which is a parallel authority under DESIGN §3.

`gunbc.dag_grammar_fork_census` rosters the root of the problem: two production tables. One is the
reachable `dag_grammar_root`, which is `GrammarExpr` with choice, repeat, optional and
nonterminals. The other is the unreachable flat `FormalProduction` token spines, and every
relation row is grounded on them. The backward machinery in `v2.std.grammar`
(`grammar_relation_row_backward_selection`, `resolve_emitted_productions`) can only read the
second table.

## The subject: which Node emit starts from

Emit's input is the **parse-tree Node that ingest's parse produces**. That is the surface Node whose
production wraps are stamped by `v2.compiler.parse` `parse_wrap_production_captured`. It is not the
core Node that body lowering produces from it.

The grammar's own `emitted` values are surface atoms (`^dag_surface_fn_decl`, and so on). So
"one grammar read backward" relates source text to *this* Node. Inverting body lowering is a
different relation with a different authority, and it is out of scope here (see the open
questions). The round-trip control is therefore:

    parse(text) = N
    emit(N) = text'
    parse(text') = N'

and it asserts `v2.std.node` `content_hash(N) == content_hash(N')`. `content_hash` does not cover
occurrence identity, so source spans can legitimately differ.

## How emit selects a production: read the stamp, then invert each `GrammarExpr` arm

**Production selection is a lookup, not a search.** Every production capture in the parse tree is
the Conj `{grammar_production_identity_node_projection: production.emitted,
grammar_production_captured_node_projection: captured}`. Emit reads the identity edge and looks up
the one production in `dag_grammar_root` whose `emitted` is that identity. That lookup is total over
the closed production list. A missing production is the typed refusal below.

There is no shape-guessing at production grain, and no second index. `emitted` atoms are unique
per production. The implementation checks that once, when it builds the index from the
`GrammarRoot`, and refuses a duplicate there.

**Within a production, emit walks the `GrammarExpr` and the captured Node together.** Each arm is
the exact inverse of what its parse arm in `v2.compiler.parse` builds:

| `GrammarExpr` arm | What parse captured | What emit does |
|---|---|---|
| `Sequence { left, right }` | Conj `{sequence_left: cap1, sequence_right: cap2}` (`parse_stamp_derived_conj`) | emit `left` over `cap1`, then `right` over `cap2`, and concatenate the token spines |
| `Repeat { element }` | right-folded Conj spine of `sequence_left`/`sequence_right`, ending in empty Conj (`parse_repeat_captured_node`) | unfold the spine and emit `element` over each item |
| `Optional { element }` | the element's capture, or `grammar_empty_node()` | empty Conj gives no tokens; otherwise emit `element` |
| `Choice { left, right }` | the winning arm's capture, untagged | **ordered:** try `left` backward; if it refuses, try `right`; if both refuse, refuse |
| `Expect { element, reason }` | the element's capture (an Expect is a label, never a cut) | emit `element` |
| `Nonterminal { production }` | a production wrap | select by stamp as described above, and check the stamp names `production` |
| `Terminal` / `LiteralTerminal` | an atom, or a literal payload node | emit one token (next section) |

Ordered backward choice mirrors parse's ordered forward choice. If the left arm accepts a shape,
its spine re-parses through the left arm first.

There is one place where a shape alone could mislead: a `StampClass` atom and a `StampLexeme`
atom are both childless atoms. An identifier spelled exactly like a token-class symbol could
therefore satisfy the wrong arm. That is not a case to guess about. The round-trip control is what
falsifies it. The walk resolves the collision one way: a `StampLexeme` position reads a childless
atom as its lexeme first.

None of this needs a per-production emit row. The rows ARE the `dag_grammar_root` productions. A
new production gets emit for free, and a new target language is still rows, not an edit to the
walk. The walk is generic over `GrammarExpr`, so it belongs in `v2.std.grammar` beside the forward
reader, not in `extdeps/languages/dag.dag`.

## How each terminal's token class is recovered: from the grammar position, never from the atom

`parse_tree_atom_token_class_not_recoverable` exists because today's reverse path asks the *atom*
for its class. A `StampLexeme` atom carries the interned lexeme, so its class cannot be read back
from it. Walking the grammar alongside the tree removes the question. At every terminal, emit
already holds `Terminal { token_class, stamp }` or `LiteralTerminal { token_class, lexeme }`, so
the class is a fact of the grammar position.

The **spelling** of the token then comes from one of two places.

**Fixed-text classes**, such as keywords and punctuation, are spelled by the class's
`TokenRule { pattern: LiteralPattern }` in `dag_lex_rules`. That is the lexer's own row, so there
is no second spelling table. A `LiteralTerminal` is spelled by its `lexeme`.

**Value-carrying classes** are spelled by `v2.extdeps.languages.dag`
`dag_emit_class_terminal_spelling`. Each arm is the inverse of the parse-side decoder that #12759 and
its predecessors stamped. These arms do **not** go into `TargetModel` `token_class_emit_transforms`:
that map transforms one spelling into another, whereas these arms start from a decoded value. The
map stays empty for the dag target.

| class | captured Node (parse side) | emit transform |
|---|---|---|
| `dag_token_ident` (`StampLexeme`, `StampBinding`) | atom whose identity is the interned lexeme | the interned spelling |
| `dag_token_int_literal` | `dag_int_literal_node_from_magnitude` payload | the magnitude in decimal, the inverse of `dag_int_literal_node_from_lexeme` |
| `dag_token_float_literal` | `float_literal_lexeme_field` payload | the stored lexeme, verbatim |
| `dag_token_string_literal` | `string_literal_value_field`, the decoded value (#12759) | quote-frame the value, and **escape it from the same `dag_string_escapes` rows** the decoder reads; a scalar with no printable row is written `\u{H..H}` |
| caret symbol | `dag_token_caret` then an ident (`dag_grammar_caret_symbol_expr`) | not a transform: a fixed token, then the ident transform |
| quoted field key | `dag_grammar_field_init_string_key_expr`'s class-stamped string terminal | not a transform: the string-literal row (the key and the literal are one terminal since #12759) |

For strings, the encoder and the decoder read one table. Decode composed with encode is the
identity on values. Encode composed with decode is not the identity on spellings, and does not need
to be: the round trip is on the Node.

**Literal payloads are read through the model's own readers:**
`dag_int_literal_magnitude_int_from_node`, `dag_string_literal_value_optional`, and
`dag_float_literal_lexeme_optional`. Lowering uses the same readers. A literal is an `Atom` that
carries a payload edge, and `v2.std.node_query` `named_child_lookup` searches only `Conj` roots, so
it cannot be used to read one.

**Token separation.** Emit writes one space between adjacent tokens. This is **not** source-faithful
layout: `module rt.x` comes back as `module rt . x`. The one exception is layout the grammar
itself requires. An `AfterLineBreak` row (gunbc#12773) matches only after a line break, so the walk
yields `GrammarEmitLineBreak` before its element, and the dag spelling writes a newline there.
A `RefuseOnMatch` row derives no tree, so backward it is an arm that never fits. Whitespace is a
`TriviaRule`, so this cannot change the Node. Annotations (DESIGN §4c) are erased from the parse
tree and are not emitted. A target that needs layout is a separate projection.

## Typed refusals

Each refusal is located with `node_locus` at the node being emitted. The generic ones come from
`v2.std.grammar`; the spelling one comes from the dag model.

**Hard refusals.** These are never retried by an enclosing choice:

- `grammar_emit_production_unknown`: the stamp names no production in the root. This is the
  "missing emit row" red.
- `grammar_emit_projection_ambiguous`: a capture carries two projection edges of one name. Parse
  mints each projection edge once, so this is a malformed tree, not an arm that does not fit.

**Arm mismatches.** An enclosing ordered choice tries its next arm; if no arm fits, the last
mismatch is the refusal:

- `grammar_emit_production_stamp_absent`: a nonterminal position holds a capture with no
  production stamp. This is neat-boar-16's condition (3): a production is never guessed.
- `grammar_emit_nonterminal_stamp_mismatch`: the stamp names a different production than the
  position expects.
- `grammar_emit_sequence_shape_mismatch`, `grammar_emit_terminal_not_atom`,
  `grammar_emit_terminal_class_mismatch`: the capture is not what the arm's parse builds.

**Index refusals.** These refuse the grammar itself:

- `grammar_emit_production_emitted_not_atom`
- `grammar_emit_production_emitted_duplicate`

**Spelling refusal.** `dag_emit_token_class_untransformable`: a class with neither a fixed lexeme nor
a value inverse. It is located at the terminal.

The atom-collision case named above (a class atom versus a lexeme atom) is not a separate refusal. A
`StampLexeme` position reads a childless atom as its lexeme first, and the round-trip control is the
falsifier.

There is no fallback arm anywhere. A refusal never widens to "emit the atom's spelling" (DESIGN §5).

## The cut

This is one change, per DESIGN §3 on replacement migrations: delete first, then fix forward.

1. **Add** the generic backward walk to `v2.std.grammar` (`grammar_emit_parse_tree`), and the dag
   spelling to `v2.extdeps.languages.dag` (`dag_emit_parse_tree`). The string encoder shares
   `dag_string_escapes`.
2. **Not done here; this is the declared frontier.** `dag_translation_rules_node` still enrolls only
   the `fn add` row. It serves the core route, which needs the inverse of body lowering before it
   can read this walk (see "Rulings and declared frontier").
3. **Delete, in the same change:** the hand-authored declaration grammar. That means
   `dag_type_decl_structural_formal_productions`,
   `dag_type_decl_productions_only_translation_rules`, `dag_type_decl_structural_lex_rules`,
   `dag_type_decl_structural_lex`, `dag_type_decl_structural_target_model`, and their only consumer,
   `v2.test.execution.emit_ingest_type_decl_round_trip`. Its roster entries in
   `v2.workflow.floor_grandfathered_roster` and `gunbc.witness.witness_deferral_freeze` go with it.

   **Correction, measured while cutting.** The flat `dag_formal_production_fn_*` table and
   `v2.compiler.ingest` `parse_atom_frontier_class` are **not** part of this route, so this change
   does not delete them:
   - They serve the cross-language forward bridge `parse_tree_to_emitted_node`, which Python and
     TypeScript also use.
   - They serve `v2.compiler.body_producer_forward`.
   - They serve the core-route emit, `compile_dag_source_to_target_text`.

   That is the core -> surface frontier above. They are retired when that item lands, against the
   roster in `gunbc.dag_grammar_fork_census`, which keeps rostering them until then.
4. **Replace** `emit_ingest_type_decl_round_trip` with the coverage round trip over the real
   grammar.

**Controls.** Round trip on the parse-tree Node for each of these fixtures:

- `data m: Int = 1`
- a fn declaration
- a record type
- a coproduct
- `match`
- `let`
- a list
- a map literal
- a quoted key
- string escapes (`\"`, `\\`, `\n`, `\u{e9}`, `\x41`)
- a caret symbol

Quoted keys add more controls (from gentle-koi-724, for #12740 and #12758):

- **The key's kind follows production identity, not spelling.** `{"Node": true}` must emit quoted,
  through `^dag_surface_field_init_string_key`, and `{Node: true}` must emit bare. Both are
  fixtures.
- **An escaped key round-trips.** `"a\"b"` and a key containing a `\u{..}` scalar re-escape to
  an equivalent spelling and re-ingest to the identical node.
- **Two mutants must go red:** emitting the raw lexeme, and emitting a quoted key in bare form.
- **The tree covered is the surface tree.** This is the pre-elaboration map literal, not resolve's
  elaborated `map_from_entries(..)`. Quoted keys in record literals still refuse at lowering
  (#12740's frontier), so they are parse-tree fixtures only.

There is also one red: a root with one production removed must refuse with
`grammar_emit_production_unknown` at the right locus. Both the reds and the positives run through the
real `parse_module` route, not a supplied tree. This route is the inhabitance claim.

## Rulings and declared frontier

**Subject (neat-boar-16's ruling).** Emit's subject is the **parse-tree Node**. The control is
text -> parse tree -> text -> parse tree, and the two trees must be identical. The core Node is not
the subject.

**A missing stamp refuses.** A tree that lacks the production stamps emit needs is refused, typed
and located, as `grammar_emit_production_stamp_absent`. Emit never guesses a production from the
shape.

**Declared frontier: core -> surface.** This emitter is a parse-tree emitter. It is **not** full
dag emission. To emit a program that was constructed or lowered rather than parsed, emit needs the
inverse of body lowering: from a core Node back to a production-stamped surface tree, which this
walk then reads. That inverse belongs to the lowering's own authority and is a separate work item.
Its named consumer is gen-2 self-host emission: the built CLI emitting a closure (DESIGN §7). Until
that item lands, nothing on that route may cite this walk as dag emission.

**Open: 02_parse's prepared grammar.** I still need its owner to confirm that the projection-edge
shapes in the table above are a stable contract.

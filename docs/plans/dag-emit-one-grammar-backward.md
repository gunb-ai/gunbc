# The dag target emits by reading `dag_grammar_root` backward

Status: **model, before code** (work item adhoc-da997c00-fa0). This page states how emit selects a
production for a Node shape and how each terminal's token class is recovered. Nothing here is
implemented yet. The implementing change follows only once this model is accepted.

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
falsifies it, and if a fixture hits it, emit refuses with `dag_emit_choice_arm_ambiguous` at the
node.

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

**Value-carrying classes** are spelled by `token_class_emit_transforms`. These are the dag model's
rows, each the inverse of the parse-side decoder that #12759 and its predecessors stamped:

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

**Token separation.** Emit writes one space between adjacent tokens. Whitespace is a
`TriviaRule`, so this cannot change the Node. Annotations (DESIGN §4c) are erased from the parse
tree and are not emitted. A target that needs layout is a separate projection.

## Typed refusals

Each refusal is located with `node_locus` at the node being emitted:

- `dag_emit_production_unknown`: the identity stamp names no production in the root. This is the
  "missing emit row" red: a fixture grammar with one production removed must refuse here, at the
  stamped node.
- `dag_emit_nonterminal_stamp_mismatch`: a `Nonterminal { p }` position holds a wrap stamped with
  some other production.
- `dag_emit_shape_mismatch`: the captured shape is not what the arm builds, for example a non-Conj
  under a `Sequence`.
- `dag_emit_token_class_untransformable`: a value-carrying class with no transform row. This is the
  successor of `parse_tree_atom_token_class_not_recoverable`, which is deleted along with its only
  route.
- `dag_emit_choice_arm_ambiguous`: as described above.

There is no fallback arm anywhere. A refusal never widens to "emit the atom's spelling" (DESIGN §5).

## The cut

This is one change, per DESIGN §3 on replacement migrations: delete first, then fix forward.

1. **Add** the generic backward walk to `v2.std.grammar`. Add the dag `token_class_emit_transforms`
   rows to `v2.extdeps.languages.dag`, with the string encoder sharing `dag_string_escapes`.
2. **Route** the dag target's emit through it. `dag_translation_rules_node` stops carrying
   hand-built rows and names `dag_grammar_root`.
3. **Delete, in the same change:**
   - `dag_type_decl_structural_formal_productions`
   - `dag_type_decl_productions_only_translation_rules`
   - `dag_type_decl_structural_lex_rules`
   - `dag_type_decl_structural_target_model`
   - the flat `dag_formal_production_fn_*` table and its relation rows (the census roster)
   - `parse_tree_atom_token_class_not_recoverable` and its map-based recovery in `v2.compiler.ingest`

   The census closes against the roster in `gunbc.dag_grammar_fork_census`, so its consumers are
   enumerated before deletion. Four consumers sit outside the module for `fn_add`, among them
   `v2.compiler.body_producer_forward`.
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

There is also one red: a root with one production removed must refuse with
`dag_emit_production_unknown` at the right locus. Both the reds and the positives run through the
real `parse_module` route, not a supplied tree. This route is the inhabitance claim.

## Open questions, for parent and peers before code

1. **The subject.** This model emits from the parse-tree Node. If the brief meant the core Node
   (after body lowering), that would be inverting lowering, which is a separate authority. I
   recommend the parse tree, and treating the core-to-surface direction as its own work item.
2. **02_parse's prepared grammar.** I need its owner to confirm that the projection-edge shapes in
   the table above (`parse_tree_projection_edge`'s roster) are a stable contract. If they are
   prepared differently on the planned route (#11422's prepared choice plan), the backward walk
   should read the same prepared form.
3. **gentle-koi-724.** The quoted-key control here is what #12740 and #12758 wait on. The key's
   emit is the string-literal row, so there is no separate key transform.

# Precedence-climbing tree contract for v2 dag binary expressions — design plan

Status: DESIGN ONLY (work item adhoc-2680a784-274, grammar program, manager stern-bear-500;
reviewer sharp-raven-357). No code lands until this plan is approved. This file sits in a
docs-only PR that will not be merged.

## 0. The defect, re-derived (§6b)

The slice is `v2.extdeps.languages.dag dag_grammar_binary_expr_expr` → `v2.compiler.parse`
(`parse_expr_sequence`, `parse_expr_repeat`, `parse_expr_optional`) → the parse tree →
`v2.compiler.body_lowering_fold` (the only reader of the tower's internal shape).

- **What the grammar says.** `dag_grammar_binary_expr_expr` = `dag_grammar_pipe_expr_helper`.
  It is seven inlined helpers (pipe → or → and → equality → comparison → additive →
  multiplicative). Each is `Sequence(next_level, Repeat|Optional(Sequence(op_choice, next_level)))`
  and bottoms out at `^dag_production_unary_expr`.
- **What the parse establishes.** Every operand, even a bare `a`, passes through all seven levels.
  At each level the parse tries the continuation, fails on the operator choice, and records the
  failure in the tree. An empty Repeat goes through `parse_stamp_derived_repeat`, which mints an
  occurrence id. An empty Optional becomes `grammar_empty_node`. Every level is also a
  `parse_stamp_derived_conj` Conj with its own minted `DerivedBy` id.
- **The result.** The tree has one node per precedence level per operand. Precedence (which
  operator binds which operands) is encoded in how deep a node sits, not in any node of its own.
- **What the only consumer actually needs.** It needs the operator applications, folded
  left-associatively. `body_lowering_fold` throws the level structure away again:
  - `body_lower_strip_operator_free_levels` peels `sequence(level, empty)`.
  - `body_lower_fold_operator_tail` re-folds the Repeat spine.
  - `body_lower_is_pipe_tower_root` decides at the root that `|>` is the outermost level.

**The earliest unjustified boundary is the grammar's tree contract.** It states precedence as
tree nesting, which no admitted consumer demands. The parse then does that work for every operand
and stamps it as tree content, and lowering undoes it. This is §2's "work no admitted consumer
demands", so the fix is to delete it. Caching, memoizing or batching the levels would only make
it cheaper. #13244 already took the per-parse fixed cost (grammar preparation). What it left is
exactly this per-operand slope.

## 1. Proposal: one node per operator APPLICATION

Add a precedence-climbing operator-table combinator to the grammar algebra, the one authority
both directions read (DESIGN §4: one grammar, read in both directions).

```
// v2.std.grammar
type InfixAssociativity = InfixLeft | InfixNonAssociative
type InfixOperatorRow {
  token_class: Symbol          // exactly one terminal class per row
  binding_power: Int           // higher binds tighter
  associativity: InfixAssociativity
  guard: Optional<GrammarExpr> // a zero-width predicate run before the operator (§3 below)
}
GrammarExpr += InfixTable { operand: GrammarExpr, operators: List<InfixOperatorRow> }
```

**Forward (parse).** A new `v2.compiler.parse` `parse_expr_infix_table`, using precedence
climbing (Pratt / binding-power):

1. Parse one operand.
2. Peek the next token's class and look it up in the table, keyed by class. The lookup is
   prepared once in `prepare_grammar_expr` as `PreparedInfixTable` and indexed, like the
   prepared choice plan.
3. If no row matches, or the row's binding power is below the current minimum, return the operand
   **unwrapped**. No continuation node is attempted or stamped.
4. Otherwise run the row's guard, consume the operator, and recurse at
   `binding_power + 1` (left associative).
5. Stamp **one** application node per operator consumed:
   `Conj { grammar_infix_left, grammar_infix_operator, grammar_infix_right }`, minting one
   `DerivedBy` id from its three children.
6. A `InfixNonAssociative` row refuses a second operator of the same binding power with a typed
   `ParseRefusalReason` (proposed name `NonAssociativeOperatorChain`). The source stays
   describable, but no accepted tree contains the chain: rung 3.

**The production shell stays one per expression occurrence** (`parse_wrap_production_captured`
for `dag_production_binary_expr`). Inside it, the captured child is either the bare unary operand
or the root application node.

**Backward (emit).** A new `v2.std.grammar` `grammar_emit_infix_table`:

- An application node emits its left side, the operator terminal, then its right side.
- A non-application node emits as the operand.
- A tree that violates the table refuses: for example, an application whose left child is an
  application of lower binding power, or an unknown operator class. Such a tree would print text
  that reparses differently, so this refusal keeps the round trip honest rather than adding
  parentheses. Parse trees never need parentheses inserted, because grouping is a primary of
  their own.

**Fold.** Add `GrammarExprFold.infix_table`. This is what keeps
`body_lower_grammar_terminals` reading the operator set off the grammar rather than from a
hand-written list (review 70545).

**The dag rows.** `dag_grammar_binary_expr_expr` becomes
`InfixTable { operand: Nonterminal(dag_production_unary_expr), operators: dag_infix_operators }`.
The rows carry **v2's current precedence, unchanged**:

| binding power | operators | associativity |
|---|---|---|
| 1 | `\|>` | left |
| 2 | `\|\|` | left |
| 3 | `&&` | left |
| 4 | `==`, `!=` | non-associative (today an `Optional`) |
| 5 | `<`, `>`, `<=`, `>=` | non-associative (today an `Optional`) |
| 6 | `+`, `-` (guarded) | left |
| 7 | `*`, `/`, `%` | left |

**Why a grammar variant, and not a flat `unary (op unary)*` with precedence re-folded in
lowering.** That flat alternative ("single-level operator-table production") was considered and
is rejected:

- It moves precedence out of the grammar into a lowering-side table, so the grammar would no
  longer be the single authority on surface syntax.
- It would accept `a < b < c` grammatically, so that refusal would happen later, in lowering.
- The backward emitter could no longer tell a well-formed tree from a mis-associated one.

The price of the variant is one new arm in every `GrammarExpr` consumer. Section 4 enumerates
them.

## 2. Readers, by symbol (git grep of every production and emitted identity)

### The grammar (deleted, replaced by one table)

All in `v2.extdeps.languages.dag`:

- Deleted:
  - the seven `dag_grammar_{pipe,or,and,equality,comparison,additive,multiplicative}_expr_helper`
  - `dag_grammar_{multiplicative,additive,comparison,equality}_op_choice`
  - `dag_grammar_additive_newline_minus_guard`, which becomes the `-` row's guard
- Re-derived:
  - `dag_grammar_binary_expr_expr`
  - the explanatory block above `dag_grammar_arrow_lambda_name_head_expr`, whose
    "pipe → or → … → multiplicative" derivation argument is restated over the table
- Unchanged:
  - `dag_grammar_unary_expr_expr` (already a single level)
  - `dag_grammar_postfix_expr_expr`
  - the `dag_grammar_root` production rows for binary, unary and postfix
  - `dag_grammar_match_expr_expr`'s scrutinee nonterminal
  - `dag_surface_operator_canonicalization_lookup` / `_member`, which read token classes, not
    tree shape

### Grammar algebra and parse (std / compiler): one new arm each

- In `v2.std.grammar`:
  - `GrammarExpr`, `GrammarExprFold` and `fold_grammar_expr`
  - backward emit: `grammar_emit_expr` gains `grammar_emit_infix_table`
  - `formal_grammar_symbol_to_grammar_expr` does not change, because there is no formal-symbol
    form
- In `v2.compiler.parse` (`02_parse.dag`):
  - `prepare_grammar_expr`, which builds `PreparedInfixTable`
  - `parse_expr_with_first` dispatch
  - FIRST analysis: `compute_grammar_first_analysis_given`
    - FIRST(table) = FIRST(operand)
    - nullable(table) = nullable(operand)
  - `compute_nullable_set`
  - left recursion: `grammar_has_left_recursion_given`, where the table is left-recursive iff its
    operand is
  - `authored_choice_site_residue` / `flatten_choice_exprs`: no Choice site, so no roster row
  - lookahead: `lookahead_site_overlaps` / `lookahead_alternatives_overlap` treat the table as
    its operand followed by an optional infix suffix
  - every exhaustive `match` over `PreparedGrammarExpr`. The listed `PreparedRepeat` arm sites are
    the census, and the compiler refuses any that is missed.
- Grammar validation (`grammar_validate_and_analyze`) gains a structural refusal: duplicate
  `token_class` within one table (§3).

### Lowering: the only reader of the tower's internal shape

In `v2.compiler.body_lowering_fold`.

Deleted (they exist only to undo the level encoding):

- `body_lower_strip_operator_free_levels`
- `body_lower_tail_has_binary_operator`
- `body_lower_fold_operator_tail`
- `body_lower_tail_element_pipes_into_fold`
- `body_lower_tail_pipes_into_fold`
- `body_lower_tail_starts_with_pipe`
- `body_lower_node_is_pipe_token`
- `body_lower_is_pipe_tower_root`
- `body_lower_tower_pipes_into_fold`

Re-derived over the application node:

- `body_lower_read_operator_expression`: an application node lowers to
  `lower_binary_infix(left, op, right)`, and a prefix operator still goes to
  `lower_unary_prefix`.
- `body_lower_operator_operand`
- `body_lower_binary_apply` and `body_lower_piped_fold_optional`: "pipe into fold" becomes "an
  application node whose operator is `|>` and whose right operand is a fold-family call". This
  is decided at the node itself, so the root-detection apparatus and its census (#12550) go away.
- `body_lower_try_infix_transform`
- `body_lower_grammar_terminals`: one new algebra arm, collecting the table's token classes.
- `body_lower_binary_operator_tokens` and the `data` row `body_lower_binary_operators`.

Shell readers, unchanged because the production shells are unchanged:

- `body_lower_is_pass_through_emitted`
- `body_lower_find_core_substrate_first_hit_child`
- `body_lower_reduce_emitted`
- `body_lower_body_subtree_lower_from_binary` / `_from_postfix`
- `body_lower_arm_operand_resolved_read`
- `body_lower_postfix_expr`
- `body_lower_primary_expr`
- `body_lower_production_emitted`

Two more, which only receive the lowered shape:

- `v2.std.compilers.body_lowering` `lower_binary_infix` / `lower_unary_prefix` build the output.
  Their input is unchanged.
- `v2.lens.vacuity` `vacuity_bool_witness_fn_body_evidence` calls `body_lower_try_infix_transform`
  and receives the same lowered result.

### Normalize, resolve, infer, eval, emit: none read the parse shape

- Normalize and resolve: no reader found.
- `v2.compiler.occurrence_role` `dag_name_production_rows`: keyed on `dag_production_postfix_expr`.
  That production is untouched, so names under postfix keep `NamesAreReferences`.
- Infer:
  - `v2.compiler.infer` (`04_infer.dag`): `infer_operator_is_plus_token`,
    `infer_transform_binary_infix`, `infer_transform_is_binary_infix_algebra_shape`
  - `v2.extdeps.runtimes.v2_evaluator` `v2_eval_transform_is_int_add`
  - All read lowered operator atoms.
- Emit: the Rust, TypeScript and dag-target emitters read `v2.std.compilers.target_model`
  `InfixToken` over lowered nodes. The dag target's token emission (#13099 `TargetBindLetShape`,
  #13113 `bound_tokens_source_text_with_transforms`) produces token lists from lowered core, not
  parse trees.

### Backward grammar emit and round trip (#12878, #13099, #13113)

These re-run as controls; none needs re-derivation beyond the new arm:

- `v2.std.grammar`: `grammar_emit_parse_tree`, `grammar_emit_sequence`, `grammar_emit_repeat`,
  `grammar_emit_optional`, `grammar_emit_choice`
- `v2.extdeps.languages.dag` `dag_emit_parse_tree` / `_with_root`, which read `dag_grammar_root`
  and so pick up the table automatically
- `gunbc.instruments.dag_emit_real_grammar_round_trips`
- `v2.test.execution.dag_grammar_backward_round_trip`
- `v2.test.execution.dag_emit_interface`
- `v2.test.emit.dag_target_text_round_trip`

### Lenses

`v2.lens.text_string_importer_census` reads fixed-depth ancestor walks of identity strings:

- `primary_is_argument` expects postfix → unary → binary → `dag_surface_expr`.
- `value_prefix`, `value_is_argument`, `value_initializes_bare_string_annotation`,
  `value_is_statement_or_arm`, `value_is_field_init`, `value_initializes_annotation` and
  `value_is_right_plus_operand` / `site_kinds_at` (`dag_token_plus`) read positions w4 to w8.

**What changes:**

- Operands that stand alone keep the same walk, because the shells are unchanged and no level
  nodes are named.
- Operands *under* an application gain the application node in their walk.
- The `+` operand reading must be re-derived from the application node's operator edge, not from
  a sibling position.

**The control** is the census output compared before and after **at identity grain**: same sites
and same kinds, not the same count. The test `v2.test.text_string_importer_census` hard-codes
about 108 walk strings (`tsil_*`). Those are re-derived; the expected *census result* is
unchanged.

Other lenses:

- `v2.lens.complexity_accumulator_copy.analyze` `postfix_expr_capture` and the
  `collect_value_identifiers` comment: re-read; postfix is unchanged.
- `v2.workflow.floor_cost_debt_edit` (`removal_safe_follower`, `removal_operand_opener`,
  `removal_walk_*`): token stream only, unaffected.

### Tests that name the shape

- `v2.test.manual.body_lowering_normalize_add`: `body_lowering_binary_infix_holds`,
  `body_lowering_expr_infix_holds`.
- `v2.test.body_lowering.reify_operand_refusal`: `ror_first_binary`.
- `v2.test.parse.newline_dual_role_operator_parse`:
  - `n_is_unary`, the unary-count route control from #13034. This survives, because unary is
    unchanged.
  - The comments citing the additive helper are re-pointed at the `-` row guard.
- `v2.test.parse.block_expr_as_binary_operand_parse` and
  `v2.test.parse.block_headed_operand_class_parse`: the derivation prose is re-pointed. Their
  arms are part of the control set (§6).
- `v2.test.manual.grammar_choice_ambiguity_audit`: the comment about "binary_expr's precedence
  levels" yielding identical rows. Those rows are gone.
- `dag/test/claim/long/dag_arrow_lambda_witness_test` `pre_repair_expr_expr`: a copy of the expr
  grammar. It references the production, not the helpers, so it is unaffected; to be confirmed
  by compiling it.
- `layout_line_break_test`: a copy of the minus guard, re-read against the row guard.
- Fixtures that only use operator token atoms are unaffected:
  - `enclosing_expression_structure`, `fold_operand_structure`
  - `rust_binop_emit`, `add_body_*`, `body_lowering_infix`
  - `dag_bind_let_block_scoped`, `closure_parse_batch_two`, `variant_field_lowering`
  - `dag_line_comment_annotation_channel`

### Prose-only mentions (no consumer; re-pointed so they don't cite deleted symbols)

- `gunbc.recurring_failure_mode`
  `v2_lexer_carries_no_layout_so_a_newline_before_a_dual_role_operator_merges_silently`
- `gunbc.recurring_failure_mode`
  `data_initializer_match_reads_its_scrutinee_from_an_arm_at_v2_body_lowering`
- `gunbc.compiler_frontend_program_status`
- `v2.test.long.accumulator_copy_fold_analysis`

### Other languages

- `v2.extdeps.languages.kotlin` has its own named-production tower (`kotlin_grammar_binary_expr`
  → equality → comparison → additive). It shares no symbol with the dag tower, so it is **not** a
  consumer and is not changed in this transition. Once `InfixTable` exists it is the obvious next
  inhabitant. That is a separate, later change, named here so it does not read as forgotten.
- `swift.dag` uses one generic operator token and no levels. `rust.dag` and `java.dag` have no
  tower.
- No generated `*.rs` mirror names any of these identities.

## 3. Overlap roster stays 0, by construction

**The table contains no `Choice`.** The roster (`authored_choice_site_residue` over
`flatten_choice_exprs`) has no site to inspect inside it. Operator selection is an *index lookup
keyed by token class*, not an ordered alternative.

**Disjointness is structural.** `grammar_validate_and_analyze` refuses a table whose rows repeat
a `token_class`:

- A duplicate row is writable in source, but no validated grammar contains one: rung 3.
- Each row is exactly one terminal class, so "disjoint FIRST sets" is "distinct class symbols":
  decidable and total.
- There is no order among rows. The zero-ambiguity ruling (2026-10-02) holds because nothing is
  left to order.

**Prefix vs infix `-` are disjoint by position.** Infix `-` is consulted only *after* a complete
operand. Prefix `-` lives only in `dag_grammar_unary_expr_expr`, at operand start. This is the
same split #13034 relied on.

**The newline-minus guard survives as the `-` row's `guard`.** The guard is
`NotPredicate(RefuseOnMatch(AfterLineBreak(Terminal(minus))))`, exactly the expression #13034
authored, moved from the additive helper's sequence onto the row. `parse_expr_infix_table` runs
the guard before consuming the operator:

- A line-break `-` raises `NewlineBeforeDualRoleOperator`. The refusal survives the predicate, as
  in `parse_expr_not_predicate`.
- A same-line `-` is subtraction.

Its controls in `v2.test.parse.newline_dual_role_operator_parse`, both the refusal arm and the
same-line subtraction route that counts unary layers, must stay green *unchanged*.

**Relation to #13126 (still OPEN).** Whether or not it lands first, the table adds no row. If
#13126 lands first, its required zero-count claim
(`test.claim.parse_test_grammar_choice_overlap`) is part of this change's control set.

## 4. Replacement migration order (DESIGN §3: delete-first, one authority transition)

The parse tree is consumed by lowering inside the same compilation. That makes it a
**gap-intolerant boundary**: no intermediate state can have the grammar emitting one contract
while lowering reads the other. So the transition is **one PR**.

1. **Delete the root.** Remove the seven helpers, the four `_op_choice` functions and the
   standalone guard. `dag_grammar_binary_expr_expr` becomes the `InfixTable` row. This breaks
   every reader loudly, and what breaks is the census in §2.
2. **Add the algebra arm in `v2.std.grammar` and `v2.compiler.parse`.** The `GrammarExpr` /
   `PreparedGrammarExpr` matches are exhaustive, so the compiler enumerates every site, and each
   one gets its honest arm (§2). This includes the duplicate-class refusal and the
   `NonAssociativeOperatorChain` reason in `v2.std.parse_refusal_reason`.
3. **Re-derive lowering over the application node, and delete the tower readers outright.** No
   reader understands both shapes, and nothing falls back to the old tower. The old tree may be
   used only offline, from history, as the differential oracle in §6.
4. **Re-derive the census lens and its walk strings**, the test fixtures, and the prose citations.
5. **Re-measure the declared cost drops (§5).** Retire a drop only where its trigger is actually
   met, never by assertion.

**Root inside the required gate?** The parse/lowering route is exercised by required floor
claims, so the deletion census is loud there. The lens (`text_string_importer_census`) and
`kotlin.dag` are named explicitly above because a product-layer reader could stay quietly broken
(§3: enumerate before deleting).

## 5. Expected saving, retired drops, headroom

**What wise-ant-549 measured (#13244)** for one operand `a` entering at each level: binary layer
3,080 steps (about 440 per inlined level). Of that, an empty repeat stamp is 14 and memo
bookkeeping about 110. For comparison: primary 1,348, postfix 860, unary 493, expr guard 1,665,
arg guard 1,263.

**Structural claims.** These must hold, and each falsifies the design if it doesn't:

- **S1.** No Repeat or Optional continuation node is attempted or stamped for an operand with no
  operator. Per operand, the binary layer does one peek plus one indexed lookup.
- **S2.** The number of application nodes equals the number of operator tokens. Level nodes: 0.
- **S3.** The lowered output (after `body_lowering_fold`) is node-identical for every control
  expression, ignoring occurrence ids.

**Magnitude estimate.** This is labelled as an estimate: missing it falsifies the estimate, not
the diagnosis.

- The binary layer drops from about 3,080 to about 150 steps per operand (one prepared-table
  lookup at the cost of an indexed lookup, about 6.5, plus frame overhead). That saves **about
  2.9k steps per operand position**.
- An operator token adds one application stamp, about the cost of one sequence stamp, instead of
  re-entering seven levels for its right operand.
- Per *token*, the saving is about 2.9k × (operand positions / token). That fraction is around
  0.4–0.5 in typical bodies, so **about 1.2–1.5k of today's 2.6–3k per-token slope**.
- The rest of the slope is the expr and arg guards (1,665 + 1,263) and postfix/primary. Those are
  out of scope here and are the next target.

**Which drops retire.** Each is predicted, and each is retired only on measurement:

- **`gunbc.rung_drop.dag_text_round_trip_nested_bind_new_witness_eval_step_cost`**, with
  `floor_eval_step_cost_drop_dag_text_round_trip_rows` (Bind body 108,685; Bind value 106,590;
  budget 72,300 = 100 ms × 723 steps/ms, from `required_floor_new_witness_envelope_ms` and
  `floor_eval_step_calibration`).
  - Its trigger names exactly this capability.
  - **Honest prediction: this change alone probably does NOT bring them under budget.** The
    two-Bind text has about 4–6 operand positions, so the expected saving is about 12–17k
    against the 34–36k needed.
  - The drop therefore stands until the measurement says otherwise. Its trigger already admits
    "or parse claims evaluated on the native route" as the alternative capability.
  - The plan will **not** reword the trigger to declare it met.
- **`gunbc.rung_drop.dag_emit_round_trip_new_witness_eval_step_cost`**, with
  `floor_eval_step_cost_drop_dag_emit_round_trip_rows`. A candidate; re-measured against its
  "two parses of a one-line module" trigger.
- **Headroom (not drops).** The four #13233 claims sitting just under budget gain about 3–6k
  each: bare name 67,569, empty brace 67,713, minus-led 68,299, mixed conditional 46,473. So does
  every gated parse claim, roughly in proportion to its operand count.

The measured per-claim figures are re-derived by the floor run. Section 6 names the instrument
rather than transcribing future numbers.

## 6. Discriminating controls

1. **Tree controls (before/after).** A representative set, parsed on the real route
   (`dag_prepared_grammar`), with the tree printed both ways:
   - `a`, `a + b`, `a + b * c`, `a * b + c`, `a - b - c` (left associativity)
   - `!a && b`, `-a * b`
   - `a && b || c`, `a == b`, `a < b`
   - `xs |> fold(0, f)`, `a + b |> f`
   - `match x { … } && b`, `if c then 1 else 2 + 3` (block-headed operands, #12817)
   - `(a + b) * c`
   - `x` then a line `-1` (refusal)

   **Red arms:**
   - `a < b < c` and `a == b != c` refuse with `NonAssociativeOperatorChain`. Today they fail
     with the generic leftover-token reason. The change in reason is part of the diff and is
     stated, not hidden.
   - A grammar with a duplicated operator row refuses validation.
   - A hand-built tree whose application violates binding power refuses backward emit.
2. **Semantic equivalence after lowering.** For every expression above, and over the full corpus
   on the native route, the `body_lowering_fold` output equals the pre-change output (S3).
   - The oracle is the old grammar run from history as an offline differential: the pre-change
     binary run on the same inputs, with no fallback in production.
   - Corpus scope: `gunbc test //gunbc/instruments:self-host` plus the required floor.
3. **Round trip.** `gunbc.instruments.dag_emit_real_grammar_round_trips`,
   `v2.test.execution.dag_grammar_backward_round_trip` and `v2.test.emit.dag_target_text_round_trip`
   stay green. Text → tree → text → tree must be a fixed point over the control set.
4. **Overlap.** The roster count stays 0 (`v2.test.manual.grammar_choice_ambiguity_audit`, and
   #13126's zero-count claim once it lands). The newline-minus controls of #13034 stay green
   unchanged.
5. **Step saving, measured.** wise-ant-549's #13244 per-level probe is re-run on the same
   fragments (`x: 1`, `x: a + b + c`, the 15-token fragment), together with the floor run's
   per-claim eval steps for the rows in §5.
   - S1 and S2 must hold exactly. The magnitude estimate is reported against them with any miss
     named.
   - This needs the probe to be an entry point rather than a PR-body transcript. If it is not one
     today, landing it is part of the change (§6, name the instrument).

## 7. The v1 seed

**v1 does not need to change.** `src/v1/02_parse.dag` already precedence-climbs:

- `parse_expr_bp`, `infix_bp`, `find_operator_bp` / `find_operator_binop` over the
  `OperatorSpec` rows of `dag/extdeps/languages/dag/syntax.dag` `dag_operators`
- the newline refusal in `is_ambiguous_prefix_infix_newline_boundary`

Nothing in v2 reads that table, and nothing in this change serves the self-host program through
v1. So the v1-maintenance purpose test (`gunbc.v1_maintenance_standing v1_seed_standing`) admits
no edit, and none is proposed.

**This design does surface three seed/v2 divergences in the operator facts.** Per DESIGN §7, each
is either a v2 regression or a seed defect. This plan preserves v2's current behaviour and does
not decide them:

- **`|>` precedence.**
  - v1: binding power 17/18, the *tightest*, so `a + b |> f` = `a + (b |> f)`.
  - v2: the *loosest* level, so `(a + b) |> f`.

  This is a real semantic split on the same source text. **It needs a ruling** before the table's
  row order is treated as settled.
- **Chained comparison/equality.** v1 accepts `a == b == c` left-associatively; v2 refuses it.
  This plan keeps v2's refusal, now typed, as the stronger rung. It should be confirmed as a seed
  defect.
- **`??` (`NullCoalesce`).** It is in the v1 table and absent from v2. That is out of scope here,
  noted only so the table is not read as complete.

**Two operator tables for one language.** After this change, the seed's `dag_operators` and v2's
`dag_infix_operators` both describe dag infix precedence. That is not a new nickname. The seed
table is the frozen X of the seed carve-out, and v2's row is the authority going forward. No v2
code may read or derive from the seed table.

## 8. Open questions for the reviewer

1. **The `|>` precedence ruling (§7).** Recommendation: keep v2's loosest-pipe. It is what the
   lowering's pipe-into-fold reading and the corpus are written against. Record the seed's
   reading as the defect.
2. **`InfixTable` as a new `GrammarExpr` variant** versus the flat single-level production with
   precedence in lowering (§1). Recommendation: the variant.
3. **The application node's representation.** Proposed: a `Conj` with three named projections
   (`grammar_infix_left`, `grammar_infix_operator`, `grammar_infix_right`). The alternative is a
   dedicated `TypeNode` connective. The `Conj` form reuses the parse's existing stamping and
   emit's projection reading and adds no connective. Recommendation: `Conj`.

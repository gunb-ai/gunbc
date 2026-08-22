# Boundary 4's door-holder is the statement-form `let`, and it is a capability, not a defect (2026-08-21)

| | |
|---|---|
| what establishes this | one added instrument (a per-diagnostic ledger written by the transaction itself), one executed grammar dump, and one read of the grammar's own production |
| producer | `v2.workflow.product_receipt_transport` `run_seven_boundary_product_receipt`, executed through `gunbc run` on a BuildBuddy runner |
| subject | `src/v2/compiler/00_compile.dag`, the compiler's own 107-module closure |
| repository ref | **current `main` at dispatch time, plus this branch's diff** — see §4, this is not a sha this document may pin |
| what it supersedes | the ranking in [the seven-boundary product receipt](seven_boundary_product_receipt_2026-08-21.md) §3.1 and [the b4 wrapper-retained census](b4_wrapper_retained_census_2026-08-21.md), both already retired by [b4 causal versus carried](b4_causal_versus_carried_2026-08-21.md) |

---

## 1. The instrument, because the finding was not reachable without it

[b4 causal versus carried](b4_causal_versus_carried_2026-08-21.md) established by reading the producers' return constructors that 39 of boundary 4's 40 diagnostics are produced on `Accepted` paths and carried into the `Rejected` list by `rejected_with_pending`, so the door is held by exactly one. That reading names the door-holder's **reason**. It cannot name its **subject**: `body_lowering_reason_unsupported_form` has sixteen producer sites, and the receipt renders only a tally of reason symbols.

So the transaction now writes a second evidence artifact beside the retention census, `<txn>/b4-diagnostic-ledger.txt`: every diagnostic the refusal carries, in carriage order, attributed through the same locus channel and the same production-identity vocabulary the census already used, plus a bounded rendering of the atoms in the refused subtree. It is written from inside the transaction that produced the diagnostics, for the same reason the census is — a ledger rebuilt afterwards would be a second observation of a second run.

The door-holder came back as line 40 of 40:

```
  40  body_lowering_reason_unsupported_form
      dag_surface_let_expr  [Type/Conj]  arity 2  occurrence minted
      atoms: dag_surface_let_expr dag_token_kw_let lm dag_token_eq dag_surface_expr
             dag_surface_binary_expr dag_surface_unary_expr dag_surface_postfix_expr
             dag_surface_primary_expr dag_language_model dag_token_lparen dag_token_rparen
```

That is the source line `let lm = dag_language_model()`.

**The grep for it returns 104 sites, and that is the finding rather than a failure of the grep.** The door-holder is not an instance to go and repair; it is a class, and the class is the ordinary statement-form `let`.

## 2. What it refuses, read from the producers and confirmed by execution

`v2.compiler.body_lowering_fold` `body_lower_let_expr` calls `body_lower_try_let_in_from_captured`, whose first test is `body_lower_let_in_body_capture_optional` — a walk over the captured spine looking for the `dag_token_kw_in` atom. With no `in` it returns `Accepted { value: Absent }`, and `body_lower_let_expr` rendered that absence as `unsupported_form`.

Two independent confirmations that this is the statement form and not a failed spine walk:

- **The grammar says so by declaration.** `v2.extdeps.languages.dag` `dag_grammar_let_expr` is a *choice* of two alternatives: `let n = e in e`, which carries its own body, and `let n [: T] = e`, which does not.
- **The parse tree says so by execution.** `fn f() -> Int { let x = 1 \n x }` dumped through the real grammar gives

  ```
  dag_surface_stmt_seq captured = Conj
    grammar_sequence_left_node_projection  -> stmt -> let_expr(let x = 1)
    grammar_sequence_right_node_projection -> Conj
       grammar_sequence_left_node_projection  -> stmt -> expr(x)
       grammar_sequence_right_node_projection -> Conj  (empty; end of spine)
  ```

So the statement form's Bind body is **the remainder of the enclosing statement spine** — a sibling of the let, not a descendant of it. `body_lower_let_expr` was being asked a question its input cannot answer, and a refusal that fires on the ordinary case is a missing capability wearing a malformed-input costume.

## 3. What the repair is, and where it had to live

The lowering is at `dag_surface_stmt_seq` (`body_lower_stmt_spine`), because that is **the only node from which both halves of a `Bind` are reachable**. That is a structural argument, not a convenience one: no amount of work inside `body_lower_let_expr` can reach a body that is not in its subtree.

A spine is taken over only when it is headed by a statement-form let *and* has a following statement — the exact case with no lowering today — so no body that already lowers is re-routed. The `in` form is untouched and keeps its own path. A non-final statement that binds nothing refuses with its own cause (`body_lowering_reason_statement_precedes_without_binding`) rather than fabricating a continuation, and that refusal is the standing control: if it ever greens, the sequence lowering has begun inventing a body for a statement that produced none.

The cause split in `body_lower_let_expr` lands beside it and is independently worth landing: one reason stood over "malformed let" and "well-formed let whose body this node cannot see", which is #8801's conflation on the neighbouring form and is what gave every reader the wrong first hypothesis.

## 4. The subject line this document may not write, and why

The receipt's own §1 argues that identities must be computed in-transaction and never pinned. The same discipline applies to the *repository ref*, and it bites harder than expected on this transport: `ctrl-build --remote` fetches **current `main`** and applies the local worktree as a patch on top, so the base commit differs from the session's HEAD and differs between dispatches (three consecutive runs here based on `08be45b6`, `47ea349f`, `a9582f84`). A sha pinned in this document would name a tree no run used.

The correct statement of subject for any measurement taken this way is "current main at dispatch time, plus the branch diff". A **historical two-ref comparison is not available on this transport at all** — positioning the worktree beforehand does not help, because the patch is applied onto whatever `main` is current.

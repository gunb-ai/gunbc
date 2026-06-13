# Design: ParseTree → TargetModel Bridge — Row-Driven Forward Fold (Structural Inverse of `06_translate`)

> **Status: DESIGN — draft-until-sign** (INVARIANTS "Map vs territory"; E-10). No load-bearing
> `.dag` lands from this doc without the consumers in §8 and sign from the authorities in §10.
> Tracker: **#4801** / **#g-ingest-bridge** / `feature:parse-tree-to-target-model-bridge`.
> Parent scope: `#g-bidir` (`docs/planning/v4-g-bidir-emit-ingest-unification-scoping-worksheet-2026-06-10.md`);
> emit-matrix consumer: `cross_language_add_python_to_typescript.dag`.
>
> **Thesis hook:** cross-target translation is the **derived homomorphism** — N+M row models,
> not N×M adapters. This bridge is the **syntax-layer forward half** that lets ingest compose
> with the already-landed semantic coercion (`coerce_grounded_node`) and backward emit
> (`serialize ∘ translate`) without a per-language shim.

## 1. Problem

The cross-language emit matrix names a single unrealized link between two proven ends:

| Link | Shape | Today |
|---|---|---|
| Source text → `ParseTree` | `tokenize` + `parse_production_tree` | **Green** (e.g. `python_mvp1_grammar_parse_accepted`) |
| `ParseTree` → emit-side core `Node` | **this design** | **Red** — `parse_tree_to_target_model_bridge_not_realized` |
| Core `Node` → target source | `translate` + `05_emit` | **Green** on fixture / descriptor paths (e.g. `dag→ts`, `ts_effect_io_emit`) |

`02_parse` produces `Outcome<ParseTree>` (`ParseTree = Node`, `grammar.dag:264`). `06_translate`
and `05_emit` consume a **TargetModel** bundle plus an **already-emitted** core `Node` (the
`emitted` field of a `GrammarRelationRow`, or the producer-rooted bodied `Arrow` fixture).
Nothing in `std/` or the compiler stages performs the forward map:

```
parse captures  ──?──►  emitted Conj / Arrow  (the shape rows already declare)
```

The gap is surfaced fail-closed in `cross_language_add_python_to_typescript.dag` via
`ParseTreeToTargetModelBridgeCheck` — not hidden behind a claim-local shim. Until a real fn
lands, the python→typescript matrix cell correctly stays `ChainFailsClosedAtCheck`.

**Why this is not “just more parse”.** `02_parse` walks the **operational** grammar algebra
(`ParseGrammar` / `GrammarExpr` — CP-1b dissolution sources). `06_translate` serializes via
**grammar-relation rows** in `TargetModel.translation_rules`. The bridge must be the forward
interpreter of those same rows (`design-bidirectional-coercion.md` §4.1), not a third syntax
authority and not a per-language hand-converter in `extdeps/`.

## 2. What already exists (M9 DFS — extend, don't coin)

| Piece | Where | Role |
|---|---|---|
| `GrammarRelationRow { production, emitted }` + `grammar_relation_row_to_node` | `std/grammar.dag:255-506` | row carrier both directions share |
| `derive_grammar_relation_row` / `derive_grammar_relation_row_node` | `grammar.dag:483-1120` | **backward** half: emitted `Conj` → row + token spine |
| `grammar_relation_row_backward_selection` | `grammar.dag:1424-1447` | `find_witness` discipline: unique LHS-exact production match |
| `GrammarInterpretationDirection`, `BidirectionalGrammarObligation` | `grammar.dag:1298-1318` | G.2 modeling — names both interpreters |
| `grammar_relation_row_for_emitted`, `target_serialize_*` | `06_translate.dag` | **backward** consumer: emitted → row lookup → token serialize |
| `coerce_grounded_node` / `find_witness_derives` | `06_translate.dag` + coercion substrate | **semantic** ingest (landed — RTADD #4544) |
| `translate` (IR → target emitted) | `06_translate.dag` | **semantic** emit coercion |
| `ParseTree` alias | `grammar.dag:264` | parse-side tree = `Node` until G.3 converges |
| Cross-language matrix + fail-closed scaffold | `cross_language_add_python_to_typescript.dag` | consumer naming the debt |
| CI tracker `#4801` | `claim_witness_corpus_ci_runner.dag` | witness red until bridge lands |

**Substrate target named (P1):** no new substrate primitives. The bridge is a **forward
catamorphism over existing `GrammarRelationRow` data** already authored per language in
`extdeps/languages/*.dag` (`target_model_edge_translation_rules`). What this design adds is (i)
the **forward fold** and its selection predicate (inverse of backward selection + serialize),
(ii) the **composition law** with semantic coercion for cross-language cells, and (iii) the
**CoC=1 placement rule** — one generic fold, language cost = rows only.

## 3. The design in one paragraph

`parse_tree_to_target_model_bridge` is **one language-blind fold** over a target's declared
`translation_rules` rows: given a `ParseTree` and a source `TargetModel`, select productions
forward (RHS frontier / capture bijection), assemble the `emitted` `Conj` each row already
names, and return the emit-side core `Node`. That is the structural inverse of
`derive_grammar_relation_row_node` + `grammar_relation_row_backward_selection` +
`target_serialize_relation_row_from_model`. For cross-language matrix cells, compose **after**
the syntax fold with the **already-landed** semantic engine:
`coerce_grounded_node` (source tree → canonical IR) then `translate` (IR → target emitted) —
zero new per-language adapters. Adding a language edits **one** `extdeps` file (rows); the fold
never grows another arm.

## 4. Mechanism

### 4.1 Structural inverse map (backward ↔ forward)

`06_translate` grammar-inverse serialize (backward) decomposes into three decisions, each
already modeled as data or a std helper:

| Step | Backward (emit / serialize) | Forward (bridge / ingest syntax) |
|---|---|---|
| Row selection | `grammar_relation_row_backward_selection` — emitted `Conj` matches unique LHS shape | **`grammar_relation_row_forward_selection`** — parse frontier matches unique RHS prefix (G.2 obligation 2) |
| Slot wiring | `derive_grammar_relation_token_edges_recursive` — walk emitted `Conj` positional children → `BoundToken`/`FixedToken` spine | **`build_emitted_conj_from_parse_captures`** — walk RHS symbols → fill LHS named/positional edges from parse captures (obligation 1 bijection) |
| Row packaging | `grammar_relation_row_to_node(row, tokens)` | **`parse_tree_to_emitted_node`** — output is `row.emitted` shape, not the full row wire (tokens are evidence, not authority) |

The forward helpers are **inverses in the information-flow sense**, not bitwise inverses:
backward starts from a fully-built `emitted` and derives tokens; forward starts from parse
captures and builds `emitted`. Both read the **same** `FormalProduction` + `emitted` template
facts carried in each `GrammarRelationRow` authored in `extdeps/`.

Forbidden: a `python_parse_tree_to_ts_node` (or any `extdeps` bridge). Forbidden: a second row
lookup keyed on stringified production names outside `translation_rules`.

### 4.2 The generic fold (M11 — one stage shape)

Finished bridge shape (pseudocode — territory lands only post-sign):

```
parse_tree_to_emitted_node(parse_tree, target_model) =
  let rules = target_model.translation_rules
  fold_parse_tree(
    tree: parse_tree,
    algebra: GrammarRelationForwardAlgebra {
      productions: formal_productions_from_rules(rules),
      rows: grammar_relation_rows_from_rules(rules),
      select: grammar_relation_row_forward_selection,
      build: build_emitted_conj_from_parse_captures,
    }
  )
```

Properties:

- **Row-driven:** every construct decision is a row in `translation_rules`; the fold has no
  per-construct `match` arms (M11 litmus).
- **Fail-closed:** 0 matching productions → located diagnostic; ≥2 → ambiguity diagnostic
  (backstop; validated grammars make this unreachable per G.2).
- **Bodied-arrow path:** rows carrying `grammar_relation_field_bodied_scaffold` use the
  existing `bodied_arrow_scaffold_from_grammar_relation_row` seam symmetrically — forward
  builds the `Arrow` from parse captures + scaffold prefix/suffix tokens, backward already
  composes via `target_serialize_bodied_arrow_from_model`.

### 4.3 Cross-language composition (N+M, not N×M)

The matrix cell python→typescript does **not** get a python×typescript adapter. Compose three
**existing** morphisms:

```
parse_tree_to_target_model_bridge(parse_tree, source_model, target_model) =
  bind_outcome(
    o: parse_tree_to_emitted_node(parse_tree, source_model),
    f: fn(source_emitted) {
      bind_outcome(
        o: coerce_grounded_node(grounded: source_emitted, target: source_model),
        f: fn(canonical_ir) {
          translate(node: canonical_ir, target: target_model)
        }
      )
    }
  )
```

| Leg | Owner | New code? |
|---|---|---|
| Syntax: `ParseTree` → source emitted | **this bridge** | yes — generic forward fold |
| Semantic: source emitted → IR | `coerce_grounded_node` | **no** |
| Semantic: IR → target emitted | `translate` | **no** |
| Syntax: target emitted → source text | `05_emit` | **no** |

Same-language ingest (future `compile_ingest_staging`) drops the middle `translate` leg when
`source_model == target_model`. The `dag→ts` matrix cell stays `ChainProven` because its
source is already the language-neutral core — no `ParseTree` leg.

### 4.4 Cost of change = 1 per language (NOT per-language bridge code)

| Change | Files touched | Bridge fold |
|---|---|---|
| Add MVP-1 row for language L | `extdeps/languages/L.dag` only | unchanged |
| Add construct to existing L | same file — new `GrammarRelationRow` | unchanged |
| Fix selection ambiguity | row data + G.2 obligation witness | unchanged |

Regression tripwire (emit-breadth §1): if a second target requires **editing the forward fold**
instead of adding rows → **STOP** — §1 regressed; escalate per `v4-emit-breadth-deep-lane-design-2026-06-12.md`.

### 4.5 Impedance with `02_parse` (operational → formal)

Today `parse_production_tree` returns trees shaped by the **operational** `GrammarExpr` walk.
Rows are keyed on `FormalProduction`. Phase-1 bridge entry therefore accepts one of two
**signed** shapes (pick at implementation — do not land both):

1. **Projection entry (interim):** `operational_parse_tree_project_to_formal_captures` in
   `std/grammar.dag` — a bounded projection from operational parse nodes to the capture
   bundle the forward fold consumes. Dissolves when G.3 replaces `02_parse` for that construct.
2. **Formal entry (G.3-aligned):** `02_parse` itself emits trees already in `emitted`-compatible
   `Conj` shape for row-backed productions — bridge becomes thin validation + `coerce_grounded_node`.

**CONSOLIDATION gate** (`#g-bidir` ratification §2): G.3 forward interpreter wiring into
`02_parse` is stage-adoption territory (Mgr-CONSOLIDATION). This bridge **owns the fold** in
`std/grammar.dag`; CONSOLIDATION owns which parse entry feeds it. Sync before either touches
`02_parse.dag`.

## 5. Placement (P1 — name the substrate target)

| Artifact | Home | Rationale |
|---|---|---|
| `grammar_relation_row_forward_selection` | `std/grammar.dag` | symmetric with backward selection (already there) |
| `build_emitted_conj_from_parse_captures` | `std/grammar.dag` | inverse of `derive_grammar_relation_token_edges_recursive` |
| `parse_tree_to_emitted_node` | `std/grammar.dag` | generic fold — not a compiler stage arm |
| `parse_tree_to_target_model_bridge` | `std/grammar.dag` or thin `compiler/` wrapper | composition with `coerce_grounded_node` + `translate` |
| Per-language rows | `extdeps/languages/*.dag` | already landed (`translation_rules`) |

**Load-bearing caution:** `06_translate.dag` is a protected pipeline stage. This design
**does not** add forward logic inside `06_translate` — the inverse lives in `std/grammar.dag`
beside the row vocabulary `06_translate` already imports. Any new import edge from
`06_translate` → forward fold is a **design-sign** diff even if line count is small.

## 6. Relationship to adjacent designs

| Doc | Relationship |
|---|---|
| `design-bidirectional-coercion.md` | Parent bidir spec — §4.1 two interpreters / §4.3 obligations |
| `v4-g-bidir-…-scoping-worksheet` | G.3 forward interpreter **feeds** this bridge; G.4 text round-trip **consumes** it |
| `design-value-emit-schema.md` | Value-tier bodies still serialize backward through same rows after bridge builds `Arrow` |
| `mvp1_dag_add_round_trip.dag` | Proves semantic half; bridge extends chain to **foreign parse trees** |
| `cross_language_add_python_to_typescript.dag` | Primary matrix consumer — scaffold flips `Violates` → `Holds` |

## 7. Constraint on implementation (effective on sign)

1. **Rows, not arms.** Any construct that cannot forward-interpret from its
   `GrammarRelationRow` is a **model defect** — extend the row / obligations, never the fold.
2. **`05_emit` stays frozen** (`serialize ∘ translate`).
3. **No per-language bridge fns** in `extdeps/` — only row authoring.
4. **Emit S1 gate** (`#g-bidir` ratification): by-execution bridge claims wait until emit
   terminates on the keystone (same gate as G.2 execution witnesses).
5. **Quotient honesty:** bridge claims cite `target_model_edge_fidelity_quotient` — identity is
   up-to-declared-quotient, never bit-identical by default.

## 8. Consumers and minimal slice (E-10 / seesaw)

**Consumers (exist today):**

- `parse_tree_to_target_model_bridge_scaffold` → real fn (`cross_language_add_python_to_typescript.dag`)
- `#4801` CI tracker (`claim_witness_corpus_ci_runner.dag` — `ExpectFail` until bridge greens)
- Future: `compile_ingest_staging` dissolution (surface 1 in bidir scoping §2.2)

**Minimal slice** (exercises committed risk on add keystone):

1. Forward fold greens on **python** `translation_rules` for `python_mvp1` add production —
   `parse_tree_to_emitted_node` accepts `python_mvp1_grammar_parse_fixture()` output.
2. `coerce_grounded_node` on result matches RTADD-quality grounding witness.
3. `translate` to **typescript** + `emit` produces authority source; compose with existing
   `ts_effect_io_emit` receipt path optional for matrix link 5.
4. Discriminating **red:** perturbed parse tree → bridge `Rejected` with located reason (not
   silent coercion).
5. Matrix cell python→ts flips to `ChainProven`; `parse_tree_to_target_model_bridge_fails_closed`
   witness flips to **false** (bridge now holds).

Does **not** need to be green for minimal slice: full `02_parse` replacement (G.5), value-tier
body projection (V-track), bit-identical source round-trip (H.7.1).

## 9. Open questions — escalate, don't improvise

| ID | Question | Recommendation | Escalate if |
|---|---|---|---|
| Q-P1 | Phase-1 entry: operational projection vs wait for G.3 formal trees | **Projection entry** for python→ts matrix (unblocks #4801); mark 🟡 dissolve-on G.3 | CONSOLIDATION picks formal entry and both land |
| Q-P2 | Where `parse_tree_to_target_model_bridge` exports | `std/grammar.dag` composition fn; compiler imports it | circular import with `06_translate` appears |
| Q-P3 | Bodied-arrow forward: build full `Arrow` vs signature-only scaffold | Mirror backward: scaffold + body producer path for COMPREP bodies | emit-breadth tripwire fires (>1 kind arm) |
| Q-P4 | Cross-language canonical IR shape for add slice | Reuse RTADD canonical grounding map — no new IR type | python emitted node lacks `coerce_grounded_node` inhabitant |
| Q-P5 | Load-bearing file touch | `std/grammar.dag` forward fold is **medium** bar; `06_translate.dag` delta needs **high** bar + sign | implementation touches translate fold logic |

## 10. Sign authorities (draft-until-sign)

| Authority | Role |
|---|---|
| **snappy-crab-849** | Program sign coordinator — bind live design-ruling work-item before merge |
| **smart-bee-585** (parent) | Emit-surface / cross-language matrix lane — owns consumer claims |
| **Mgr-CONSOLIDATION** | `02_parse` adoption / G.3 — sync before parse-stage edits |
| **sharp-fox-370** | Emit-breadth tripwire owner — §1 regression checks |

**Merge gate:** load-bearing `.dag` merges only after ≥2 approving providers, no
`REQUEST_CHANGES`, and explicit sign on the structural diff (same discipline as fan-in (ii)
emit in `design-comprep-m0-branch-mapping.md` §11).

## 11. Non-goals

- No change to `find_witness` closedness / preservation vocabulary.
- No `IngestionDescriptor` or parallel syntax engine.
- No per-construct parse/render lambdas without rows.
- No wholesale `02_parse` deletion in the bridge PR.
- No bit-identical round-trip claims without quotient declaration.
- No implementation in this design session — **map only**.

## 12. Acceptance matrix

| Row | Required evidence | Forbidden evidence |
|---|---|---|
| Generic fold | One `parse_tree_to_emitted_node` — parameterised by `TargetModel` only | Per-language `*_parse_tree_to_*` fns |
| Structural inverse | Forward fold inverts `derive_grammar_relation_row_node` on MVP-1 fixtures (hash or structural witness) | Hand-wired token lists in compiler |
| Cross-language | python parse → bridge → translate → ts emit by execution | Direct python→ts string adapter |
| CoC=1 | Second language (e.g. go) greens by **row authoring only** after parse accepts | Bridge fold edit for second language |
| Fail-closed | Scaffold flips; discriminating red on bad parse tree | Silent skip; claim-local shim |
| Matrix | `cross_language_emit_matrix` python→ts = `ChainProven` | `ChainProven` without execution |

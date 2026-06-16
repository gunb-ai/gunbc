# Design: Bidirectional Emit/Ingest — One Declared Coercion Relation

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). No code lands from
> this doc without the consumers named in §7 (E-10). This is the design for the dep-graph BIDIR
> node ("emit/ingest = one coercion, the central unifying target") and for the in-tree
> convergence markers that already point at it: `feature:CP-1b-bidirectional-grammar-carrier`
> (`src/v2/std/grammar.dag:79,138,171`) — "dissolve-on: … single declarative **bidirectional**
> grammar model; forbidden: parallel production algebras."
>
> Housekeeping note: those markers bind `TASKS.md T-6/T-7`, and `06_translate.dag`'s anchor
> cites `docs/design-v2-compiler-homomorphism.md` — **neither file exists in this repo**. Until
> the references are repaired, this doc is the in-repo design home for the CP-1b convergence.

## 1. Problem

The thesis claims emit and ingest are *one* coercion run in opposite directions (THESIS "The
core flip"; ROADMAP "Coercion in both directions"). The tree today has two hand-built
directions that do not share a spec:

- **Forward (ingest side):** `01_tokenize` + `02_parse` interpret hand-authored operational
  grammar types (`ParseGrammar`, `GrammarExpr`, `GrammarProduction` — each carrying a CP-1b
  marker saying it should dissolve into the canonical substrate).
- **Backward (emit side):** `06_translate` coerces grounded IR onto target carriers and
  serializes via a grammar-inverse lex walk over `TargetModel` concrete-syntax rows;
  `05_emit` stays a frozen 42-line `serialize ∘ translate`.

If the emit ladder (T3–T6) keeps growing the backward direction as render logic, T7 ingest
arrives as a *bolt-on inverse*: a second authority for the same source↔IR relation, which is
exactly the P2 parallel-authority shape and the N×2 the derived-homomorphism thesis exists to
collapse. The design problem: **one declared relation per target, both directions derived,
with the round-trip as the inverse proof** — and the spec shape decided *now* so T3–T6 are
built inverse-aware rather than retrofitted.

## 2. What already exists (M9 DFS — the pieces are landed, the unification is not)

| Piece | Where | Role |
|---|---|---|
| `FormalGrammar` / `FormalProduction { lhs, rhs: List<FormalGrammarSymbol> }` with terminal **bindings** | `src/v2/std/grammar.dag:44-100` | the canonical declarative carrier — marked as the dissolution **destination** for the operational parse types |
| `ParseGrammar`, `GrammarExpr`, `GrammarProduction` (operational, forward-only) | `grammar.dag:139-182` | to dissolve into `FormalGrammar` (their own CP-1b markers) |
| `GrammarRelationRow`, `ConcreteSyntaxSchema`, grammar-inverse helpers (`formal_production_emitted_slot_count_step`, `grammar_relation_row_to_node`) | `grammar.dag:114,200,428,540` | the relation-row vocabulary and the start of slot accounting |
| `TargetModel` concrete syntax: `ConcreteSyntaxToken = FixedToken | BoundToken`, atom/type-expr realizations | `src/v2/std/target_model.dag` | per-target syntax facts, already data |
| Lexical layer: `LexPattern` / `LexRules` | `src/v2/std/lexing.dag` | the same relation shape one level down (chars ↔ tokens); CP-1b names its convergence with `GrammarExpr` |
| `find_witness` unique-candidate fold over a **closed declared candidate set** | `src/v2/std/find_witness.dag` | the selection discipline both directions reuse (§4.2) |
| IR-side coercion (R1/R2/R3, `find_witness_derives`) | landed, gunbc#4585 | the semantic half of ingest/emit (§5) |
| Round-trip claims: `dag_ingest_round_trip.dag`, `source_authority_contract.dag`; RTADD landed (#4544) | `src/v2/test/claim/round_trip/` | the inverse-proof harness; today it proves composition through the *hand-built* forward direction |

**Substrate target named (P1):** no new substrate. The relation is authored in the existing
`FormalGrammar` + `TargetModel` carriers; what this design adds is (i) the **obligations**
that make a declared grammar bidirectional (§4.3), (ii) the rule that both pipeline
directions are *interpreters of the same rows* (§4.1), and (iii) the staging constraint on
T3–T6 (§6).

## 3. The design in one paragraph

A target's concrete syntax is **one declared relation** `R ⊆ Node × TokenSeq`, authored once
as `FormalGrammar` production rows with named bindings. Emit interprets the rows backward
(node → tokens); ingest interprets the rows forward (tokens → node). Nobody writes a
direction; both are derived interpreters over the same data. The full pipelines compose the
syntactic relation with the already-landed semantic coercion:

```
        grammar relation (per target, declared once)        find_witness coercion (landed)
source ───────────────────────────────► target-model tree ───────────────────────────► IR
       ◄───────────────────────────────                    ◄───────────────────────────
        same rows, backward interpreter                     same fold, opposite direction
```

Emit = coerce-to (IR → target tree) then render (rows backward). Ingest = parse (rows
forward) then coerce-from (target tree → IR). The round-trip `dag → target → dag` landing at
identity **up to the declared normalization quotient** is the proof the two interpreters are
inverse — RTADD generalized, claim-by-claim, never assumed.

## 4. Mechanism

### 4.1 Two interpreters, one row set (P2: single authority)

A production row carries: LHS node shape (connective + named-edge slots), RHS symbol sequence
(fixed terminals, bound terminals, nonterminal occurrences), and the slot **bindings** that
connect RHS captures to LHS edges. The two interpreters are folds over the same rows:

- **Backward (render):** given a node, select the production whose LHS shape it inhabits;
  emit RHS in order — fixed terminals verbatim, bound terminals from the bound edge's
  realization, nonterminals by recursion on the bound child. (This is what `06_translate`'s
  grammar-inverse walk already approximates; the design makes the row the authority and the
  walk its interpreter.)
- **Forward (parse):** given a token window, select the production whose RHS frontier
  matches; consume fixed terminals, capture bound terminals into edges, recurse on
  nonterminals; build the LHS node. (This is what `02_parse`'s operational types do today
  against their own parallel algebra; they dissolve into row interpretation per their CP-1b
  markers.)

Forbidden by construction (and by the existing markers): a render lambda or parse function
for any construct that has a row. Free-form per-construct functions are the one-way
compression that makes the inverse underivable.

### 4.2 Production selection is `find_witness` (concept unification, not analogy)

Both interpreters face the same decision: *which production applies?* That decision is the
coercion fold applied at the syntax layer:

- candidates = the target's production set — **closed by declaration** (the same
  `coercion_property_closed_candidate_set` discipline; the grammar never generates
  candidates);
- preservation predicate = "node inhabits LHS shape" (backward) / "token frontier matches
  RHS prefix" (forward);
- multiplicity = unique passing candidate; 0 ⇒ located no-candidate diagnostic; ≥2 ⇒
  ambiguous.

With one sharpening: ambiguity in a *declared grammar* is a **model defect**, so it is
rejected fail-closed at model-validation time (a structural check over the rows, §4.3), not
discovered at use time. The per-use `find_witness` ambiguity arm remains as the fail-closed
backstop, but a validated grammar makes it unreachable.

### 4.3 Bidirectionality obligations (what gets checked, statically, per grammar)

A declared grammar earns the *bidirectional* verdict only if four structural obligations
hold — each a fold over the rows, each fail-closed, each a `TestClaim`-able lens verdict:

1. **Slot bijection (information preservation):** per production, RHS bound captures ↔ LHS
   named edges, one-to-one (extends `formal_production_emitted_slot_count_step`). A capture
   with no edge, or an edge with no capture, makes the row one-way; reject the model.
2. **Forward determinism:** for each nonterminal, alternative productions have disjoint
   token frontiers at declared lookahead k (§8 Q-B1). Overlap ⇒ reject the model with the
   colliding pair located.
3. **Backward determinism:** productions' LHS shapes are disjoint (connective + edge-label
   set selects at most one row). Overlap ⇒ reject.
4. **Quotient declaration (honest lossiness):** every token channel is declared either
   *modeled* (carried into the tree, round-trips bit-faithfully) or *quotient* (whitespace /
   formatting / comments — normalized away in **both** directions). The round-trip claim for
   the target states its quotient explicitly; "identity" always means identity-up-to-declared-
   quotient, and the existing round-trip claims' "not bit-identical unless claimed" label
   stays load-bearing (ROADMAP).

These four are the whole carve. A grammar that fails one is still a fine *parse-only* or
*render-only* model — it just never gets the bidirectional verdict, and T7-style derived
ingest refuses for it with the failing obligation as the located diagnostic. Derive what's
determined; refuse the rest.

### 4.4 The lexical layer is the same design one level down

`LexPattern`/`LexRules` ↔ token classes is a relation with the same two interpreters
(scan / render-lexeme) and the same obligations (capture bijection, frontier disjointness =
maximal-munch determinism, quotient channels live here). CP-1b's "GrammarExpr converges with
LexPattern" lands as: both are production-row algebras over different symbol alphabets, and
the obligation lenses are shared. No second framework.

## 5. What inversion does *not* give you (the semantic half stays coercion)

The grammar relation ends at the **target-model tree**. Ingest is not done there: the tree
must still coerce into canonical IR — project-to-core, widening, refusal — which is exactly
`find_witness_derives` (landed, #4585) run in the from-direction. Symmetrically, emit begins
with IR → target-tree coercion before any row renders. This split is load-bearing:

- syntax relation: per-target, declared in `extdeps/languages/*.dag` via
  `FormalGrammar`/`TargetModel` — bidirectional by §4.3;
- semantic relation: target-tree ↔ IR via the one coercion fold — bidirectional because
  widening/narrowing verdicts are direction-aware by construction (R1/R2).

Neither layer reaches around the other (tokenize/parse and print/render "must not become
separate adapter authorities" — ROADMAP). The "one coercion" headline cashes out as: **two
declared relations per target, each interpreted in both directions by shared machinery, zero
hand-written direction-specific adapters.**

## 6. Constraint on the emit ladder, effective immediately

This is the part that must be decided *now* (the operator has directed Mgr-SPINE to build
inverse-aware; this section is the spec for that directive):

> **Precedence note (2026-06-10, resolving a dual-authority seam):** item 1 below states the
> *direction* — rows-as-data, never render closures. The **binding decision** of the T3 fold
> carrier's shape — including whether one row/fold discipline-as-data carries both positional
> (Arrow) and labeled (Conj) edge disciplines — is **taken after the one bounded run**, per
> `design-optional-surface.md` §4 (the measured T3 root-cause sequence). That memo's sequence
> is the single authority for *when*; this section remains the authority for *what shape
> qualifies* once the datapoint is in. Do not commit a carrier shape before the run.

1. **T3's fold carrier (dep-graph Q1) is the production-row reference, not a render
   closure.** The `06_translate` `project_*` dissolution lands as "interpret row backward."
   Any new emit capability that cannot be expressed as a row + bindings is a substrate gap to
   surface (new `ConcreteSyntaxToken` kind / row field with its own obligation), never a
   bespoke lambda.
2. **T4–T6 additions author rows**, and each new row lands with its slot-bijection obligation
   green — so by T7, deriving ingest is *turning on the forward interpreter*, not a project.
3. **`05_emit` stays frozen** (42 lines, `serialize ∘ translate`) — unchanged by this design.
4. **The operational parse types dissolve per-construct, ratcheted by claims:** as a
   construct's row passes the obligations and its forward interpretation lands, the
   corresponding hand path in `02_parse` deletes (the CP-1b markers are the receipts).
   Wholesale parser replacement is explicitly not the plan — that's a bridge-sized bet; the
   per-construct ratchet keeps every step consumer-verified.

## 7. Consumers and minimal slice (E-10 / seesaw)

- **Consumers (exist):** the round-trip claim runner (`test/claim/round_trip/`, RTADD #4544);
  the translate-stage claims (mvp1 per-target translate claims); `02_parse` for the
  dissolution direction.
- **Minimal slice** — the home language first (`extdeps/languages/dag.dag`, per the RTADD
  decision's `dag_mvp1_target_model` direction), scoped to the `add` keystone subset:
  1. author the subset's `FormalProduction` rows with bindings (function decl, params, call,
     literal — just enough for `add`);
  2. land the four obligation checks as structural folds with `TestClaim`s — **green** on the
     subset grammar, plus discriminating **reds**: a row with a dropped capture (slot
     bijection); two rows with overlapping frontiers (forward determinism); two rows with the
     same LHS shape (backward determinism);
  3. derive **both** interpreters over those rows and run the round-trip claim: `add` source
     → tree → source lands at identity up to the declared quotient, **by execution**; one red
     variant proving the quotient is real (perturb whitespace ⇒ still green; perturb an
     identifier ⇒ red).
  The slice exercises the committed risk — same-rows-both-directions + the obligations — on
  the keystone, not a toy grammar.
- Follow-on, consumer-triggered: rust_mvp1 rows (first non-self target), per-construct parse
  dissolution, lex-layer convergence.

## 8. Open questions — escalate, don't improvise

- **Q-B1 — grammar class. RESOLVED (operator 2026-06-09): the commitment is the four
  obligations, not a formalism.** Grammar formalisms are themselves modeled data — a PEG (or
  any other formalism) may be modeled independently as its own carrier, and nothing here
  forbids it. What this design commits is narrower: the **bidirectional verdict** is earned
  only by grammars passing obligations 1–4 (§4.3), and "declared-lookahead LL(k)" is simply
  the name for what passes obligation 2 — a consequence, not an allegiance. PEG-style
  ordered choice cannot pass as authored, because priority-resolved ambiguity has no
  backward analogue (the render direction selects by node shape, blind to forward priority,
  so round-trip identity can silently break); a PEG whose priorities are vacuous is already
  the disjoint form. A PEG model therefore lives as a **parse-only** formalism (no
  bidirectional verdict, derived ingest refuses with the failing obligation located) unless
  someone proves a new obligation set for it — its own decidability argument, added with a
  consumer, per "fewer variants for now."
- **Q-B2 — where the quotient is declared.** Recommended: a `TargetModel` field (per-target
  fact, single authority) rather than per-claim labels; claims then *cite* it. Operator call
  on the field shape.
- **Q-B3 — non-context-free islands** (Python indentation, raw strings, heredocs). These live
  at the lex layer as declared modes with their own capture bijection — or the target's
  bidirectional verdict is refused for the affected constructs. No grammar-level hacks.
- **Q-B4 — marker repair.** CP-1b markers bind `TASKS.md T-6/T-7` and `06_translate` anchors
  `docs/design-v2-compiler-homomorphism.md`; neither exists in-repo. Re-point them (at this
  doc or the real tracker rows) so the dissolution triggers are checkable again.

## 9. Non-goals

- No change to `find_witness`, the preservation-rule vocabulary, or candidate-set closedness.
- No bit-identical round-trip claims (T8 territory; quotient-honest identity only).
- No multi-language ingestion breadth — one home-language slice plus the obligation
  machinery; breadth is row-authoring after that.
- No parser-generator tooling, no grammar inference: rows are authored facts, interpreters
  are derived, discovery stays out (P1 — grounded in declared sources only).

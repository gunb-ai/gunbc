# Design: Value-Expression Emit Schema — Behavior Bodies → Target Source via Declared Rows

> **Status: DESIGN — map, not territory** (INVARIANTS "Map vs territory"). No code lands from
> this doc without the consumers named in §6 (E-10). This is the contract for COMPREP wave 3
> ("bodies through translate/emit", `design-computation-representation.md` §3) and the emit
> ladder's value tier; it conforms to `design-bidirectional-coercion.md` §6 (rows + bindings,
> never render closures) and inherits that doc's precedence note: the positional-vs-labeled
> fold-carrier question stays open until the one bounded run
> (`design-optional-surface.md` §4) — this schema is deliberately independent of it (§7 Q-V3).
>
> Why now: COMPREP produces real `Arrow.body` Behavior sub-DAGs (#4608/#4616,
> `03_body_producer.dag:87`), but body *emission* is still either hand-enumerated fixture
> tokens (`ts_mvp1_concrete_tokens`, `typescript.dag:246`) or v2's hardcoded per-language
> emitters. 18 LanguageModels (`extdeps/languages/`) will grow value-tier spelling; without
> one declared schema they diverge into N hand-shapes. A skeleton lane (Transform-for-add,
> TypeScript) is dispatched as this design's vetting consumer; its plug-in seam is §4.2/§4.3.

## 1. Problem

A function body in v2 is a `ComputationNode` sub-DAG over the **closed** five-arm coproduct
`Behavior = Value | Transform | Branch | Loop | Bind` (`std/node.dag:21`), attached at the
`Arrow.body` edge (E-9 single authority, target-shape validated in `well_formed`, #4616).
The type tier already emits the modeled way: per-language
`TargetTypeExpressionProjection` rows (`std/target_model.dag:1620`; exemplar
`rust_type_expression_projection`, `rust.dag:3177`) drive `target_type_expr_*_emitted`
through the grammar-inverse serialize. The value tier has only **leaves**
(`TargetValueExpressionKind`, `target_model.dag:212` — symbol/bool/char/string literals plus
two reference wrappers) and **no projection** from a Behavior sub-DAG to those carriers.

The design questions this doc answers:

1. What composite target-value kinds exist, and who owns them (substrate vs per-language)?
2. What is the per-language authoring surface for *how this language spells each Behavior
   arm* — the analog of the type tier's projection row?
3. What is the **primitive-operation → target-spelling contract** — how a `Transform` names
   its operation and how each language maps that operation to `+` / a call / a method —
   such that the mapping is declared data, fail-closed, and bidirectionally derivable?

## 2. What already exists (M9 DFS — extend, don't coin)

| Piece | Where | Role |
|---|---|---|
| `Behavior` closed coproduct | `std/node.dag:21-26` | the source domain; exhaustiveness of the projection comes from its closedness |
| `Arrow.body` edge + target-shape `well_formed` | `std/node.dag:219-241`, #4616 | the single attachment authority (E-9/DB-14) |
| Real body producer: `Transform` + positional `[op-atom, param-x, param-y]` | `03_body_producer.dag:87-105` | the keystone LHS shape the first rows must match |
| `TargetValueExpression{Kind}` leaves + `TargetValueTemplate{Kind}` ingestion twins | `target_model.dag:212-237` | the value carriers — composites attach here |
| `TargetTypeExpressionProjection` per-language row | `target_model.dag:1620-1627`; `rust.dag:3177` | the authoring-surface pattern the value tier mirrors |
| `FormalProduction` rows + bindings; `GrammarRelationRow` | `grammar.dag:73,200` | the row substrate both directions interpret (bidir §4.1) |
| `find_witness` closed-candidate fold | `std/find_witness.dag` | selection discipline for production/spelling choice (bidir §4.2) |
| Grammar-inverse serialize on the translate path | `06_translate.dag` | unchanged consumer of derived `ConcreteSyntaxToken`s |
| v2 emitters' `ExprData` taxonomy (~20 arms) | `src/v1/05_emit_*.dag` | the dissolution target; its arm list is the long-tail breadth checklist, **not** a vocabulary to copy |

**Substrate target named (P1): no new substrate.** Everything lands in the existing
`TargetValueExpression` / projection-row / `FormalProduction` carriers.

## 3. The design in one paragraph

A body emits through the same two relations as everything else (bidir §5): **semantic** —
the Behavior sub-DAG projects onto target value carriers (`TargetValueExpression` nodes) by
one total fold, exhaustive over the closed five-arm coproduct, language-blind; **syntactic**
— those carriers serialize through the language's declared rows interpreted backward. The
schema is three declarations: (i) **composite value kinds** in `std/target_model.dag`, one
per Behavior arm-shape, substrate-owned and closed (§4.1); (ii) a per-language
**`TargetValueExpressionProjection` row** — pure spelling data, the value-tier analog of the
type tier's row (§4.2); (iii) a per-language **operator-realization catalog** keyed by one
canonical operation vocabulary (§4.3). Adding a language = authoring its row + catalog.
Adding a construct = one substrate kind + one row field per language. No per-language logic
anywhere; cost-of-change stays 1.

## 4. Mechanism

### 4.1 Composite value kinds (substrate-owned, closed, producer-gated)

`TargetValueExpressionKind` grows one kind per Behavior arm-shape — language-agnostic,
declared in `std/target_model.dag` next to the existing leaves:

| Kind | Source shape | Wave |
|---|---|---|
| `TargetValueExprBindingRef` | `Value`-leaf reference to a bound param/let (identity via the `binding_id` channel once #4581 lands; surface atom interim) | now (add keystone) |
| `TargetValueExprPrimitiveApply` | `Transform` whose operation resolves to a primitive operation | now (add keystone) |
| `TargetValueExprCallableApply` | `Transform` whose target is a declaration callee | with the COMPREP callee path |
| `TargetValueExprConditional` | `Branch` | with COMPREP wave 2 producer |
| `TargetValueExprBindLet` | `Bind` | with COMPREP wave 2 producer |
| `TargetValueExprLoop` | `Loop` | with COMPREP wave 2 producer |

Rules: each kind lands **with its producer and its `TargetValueTemplateKind` ingestion
twin** (rows land inverse-aware, bidir §6.2; E-10 forbids kinds ahead of producers — no row
authoring for `Branch/Loop/Bind` before COMPREP wave 2 exists). The kind set is derived from
`Behavior`'s closed arms: adding one is a substrate decision (C1-adjacent escalation), never
a per-language act. The existing literal leaves are unchanged and remain the `Value`-arm
recursion base.

### 4.2 The per-language projection row (the authoring surface; the skeleton's seam)

```
type TargetValueExpressionProjection {
  binding_ref_form: TargetBindingRefShape        // { ident_token }
  primitive_apply_form: TargetPrimitiveApplyShape // shape per fixity + catalog ref (§4.3)
  callable_apply_form: TargetCallApplyShape      // { callee_ident_token, open, separator, close }
  // conditional_form / let_form / loop_form: land with their kinds (E-6: no field
  // without a same-PR consumer — the row TYPE grows per wave, never speculatively)
}
```

Mirrors `TargetTypeExpressionProjection` exactly: a record of shape records whose fields are
token-class `Symbol`s (the `TargetGenericApply` / `TargetFunctionTypeShape` precedent), one
instance per LanguageModel in `extdeps/languages/<lang>.dag`, threaded on the TargetModel
bundle via a new edge key beside `target_model_edge_value_expression`. Naming/case
conventions stay in the rendering layer (SELF_HOSTING §2.3.2 layer 3) — `binding_ref_form`
names a token class, never a casing rule.

Per bidir §4.1/§6.1, these shapes are authoring sugar for **derived `FormalProduction`
rows**: each shape determines the production's RHS symbol sequence and bindings; the four
bidirectionality obligations (slot bijection, forward/backward determinism, quotient)
check on the **derived rows**. Whether the derived rows carry positional or labeled edge
disciplines is the open T3 carrier question — see §7 Q-V3; nothing in this section depends
on its answer.

### 4.3 The primitive-operation → spelling contract

**One canonical operation vocabulary, substrate-owned (M4: closed set, not strings).** A
`Transform`'s operation is a fact about the *source* program, named once in `std/`; the
spelling is a fact about the *target*, declared per language. The two never meet except
through the catalog:

```
// std: the operation identity — a CLOSED coproduct carrier (M4: enums, not strings;
// illegal operations are unrepresentable, not validated away at use time). Seeded
// producer-gated (E-10): wave-now carries only the arms a COMPREP producer emits.
// What grounds each arm (the algebra operation) is Q-V1; the coproduct is the
// carrier either way — arms gain algebra grounding, the carrier does not reopen.
type CanonicalOperation
  = OpAdd                            // wave-now (the add keystone)
  // arms land with their producers; adding one is a substrate decision

// extdeps/languages/<lang>.dag: how THIS language spells it
type TargetOperatorRealization {
  operation: CanonicalOperation
  shape: TargetOperatorShape
}
type TargetOperatorShape
  = InfixToken { token: Symbol }     // x + y       (C-family, dag)
  | PrefixToken { token: Symbol }    // not x
  | CallIdent { ident: Symbol }      // add(x, y)   (Lean, LLVM IR, wasm instructions)
  | MethodIdent { ident: Symbol }    // x.add(y)
```

- The catalog is a **closed declared candidate set**; selection is the `find_witness` fold
  (bidir §4.2): 0 entries ⇒ fail-closed located refusal — *this language cannot spell this
  operation* (C-8; no fallback, no fabricated spelling); ≥2 ⇒ model defect, rejected at
  model-validation time.
- The catalog entry × the `primitive_apply_form` shape **derive** the primitive-apply
  production row for that operation. Authoring stays one catalog row per op; obligations
  still check on the derived rows, so bidirectionality is proven, not assumed. The same
  catalog read forward gives T7 body ingest its operator recognition — one fact, two
  directions.
- `TargetOperatorShape` is substrate-owned and closed. A target whose operator surface
  doesn't fit (a real 19th case) is a substrate gap to surface with its own obligation
  (bidir §6.1), never a per-language lambda.

### 4.4 The projection (one total fold — not an "engine")

A pure function on the translate path beside the type-tier projection: exhaustive match
over `Behavior` (totality from closedness), recursion on children, every spelling read from
the language's projection row + catalog. Zero language mentions; zero string concatenation
for syntax (the output is `ConcreteSyntaxToken`s into the existing grammar-inverse
serialize; `05_emit` stays frozen). A missing row field or catalog entry is a typed,
located diagnostic. Function-decl productions compose the existing signature tier with a
body nonterminal — the signature tier is untouched.

### 4.5 Fixture dissolution (the certs become discriminating)

`ts_mvp1_concrete_tokens` (`typescript.dag:246`) and analogs get dissolve-on markers now:
when a language's value rows cover the add body, the cert's body-token segment is replaced
by projection output and the receipt becomes **body perturbation flips the source** (swap
operands; `+` → `-` via a catalog edit). A fixture token list surviving its own rows is a
dual representation (2FA) and gets deleted, per the white-box-test discipline.

## 5. Worked example — the add keystone, TypeScript

Body (real producer, `03_body_producer.dag:87`): `ComputationNode Transform` with
positional children `[atom +, param x, param y]`. Projection: `Transform` arm →
`PrimitiveApply`; catalog lookup (canonical add → `InfixToken { ^ts_token_plus }`); children
recurse to `BindingRef{x}`, `BindingRef{y}` via `binding_ref_form.ident_token =
^ts_token_ident`. Derived body tokens:

```
[ BoundToken { ts_token_ident, x }, FixedToken { ts_token_plus }, BoundToken { ts_token_ident, y } ]
```

— exactly the body segment of today's `ts_mvp1_concrete_tokens`, now **derived from the
body** instead of enumerated. The function-decl production wraps it with the
statement-context tokens (`return`, `;` — see Q-V2) and the existing signature tier.

## 6. Consumers and minimal slice (E-10 / seesaw)

- **Design-vetting consumer (dispatched):** the TS skeleton lane — Transform-for-add arm,
  one target, translating the real `produce_mvp1_add_arrow_with_body` body. Its seam is
  §4.2's row + §4.3's catalog; the skeleton lands with interim hand-instantiated row/catalog
  values for TS and the discriminating mutate-receipt of §4.5.
- **Minimal schema slice:** `BindingRef` + `PrimitiveApply` kinds with their template twins;
  `TargetValueExpressionProjection` + a one-entry operator catalog for TypeScript; greens —
  add body emits by execution; reds — operand swap changes emitted source, catalog
  `+` → `-` changes emitted source, missing catalog entry refuses with the located
  diagnostic. Then the home language `dag` + `rust` rows (round-trip per bidir §7).
- Follow-on, producer-gated: `CallableApply` with the COMPREP callee path; wave-2 behaviors
  with their producers; v2 emitter dissolution proceeds per-construct as row coverage lands.

## 7. Open questions — escalate, don't improvise

- **Q-V1 — what grounds the `CanonicalOperation` arms.** The carrier is settled (§4.3: a
  closed substrate coproduct, M4 — never a bare `Symbol`); the open question is each arm's
  grounding. Recommended: the algebra operation (`OpAdd` grounds in ordered-ring add), per
  THESIS grounding-completeness (dispatch consumes abstract algebra facts — the standing
  decision from the numeric-aliases gate) — not free-floating arm names, and never the
  surface token. Interim for the skeleton: the body producer's surface op atom
  (`^dag_token_plus`) maps to `OpAdd` at projection entry under a 🟡 mark whose dissolve-on
  is op-resolution (surface op token → canonical operation at resolve, the same move idents
  make to `binding_id`). Operator confirms the grounding before rows multiply across 18
  models.
- **Q-V2 — statement-bodied targets.** Python/Go/TS need `return` + statement context;
  expression-bodied targets (Rust) don't. Recommended: the **function-decl production row**
  owns statement wrapping (it is syntax of the declaration, not of the value), keeping the
  value projection context-free. Decide at the first statement-bodied landing — which is the
  TS skeleton, so this question resolves with its PR.
- **Q-V3 — positional-vs-labeled edge discipline in derived rows.** Inherited verbatim from
  `design-optional-surface.md` §4 / bidir §6 precedence note: decided **after** the one
  bounded run, not here. The kind vocabulary (§4.1) and catalog contract (§4.3) are
  independent of it by construction.
- **Q-V4 — ownership/sharing decoration** (Rust Rc/clone/borrow). Layer-1 semantic facts
  (the LS-4 lineage), out of scope for the value tier now; the existing `ReferenceRcNew` /
  `ReferenceBoxNew` leaves stay as-is and are not a precedent for spreading ownership into
  value rows.

## 8. Non-goals

- No rows or kinds for `Branch`/`Loop`/`Bind` ahead of COMPREP wave-2 producers (E-10).
- No render closures, no per-construct emit functions, no string-concat syntax — anywhere
  (bidir §4.1 "forbidden by construction").
- No new obligation kinds: the value tier reuses bidir §4.3's four, on derived rows.
- No bit-identical round-trip claims; quotient-honest identity only.
- No effects/services/transport spelling — separate tier, separate design.
- No pre-commitment on the T3 fold-carrier shape (Q-V3).

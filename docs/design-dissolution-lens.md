# Dissolution Lens — design spec

> **Status: design (rework-tracker PR #3240 task B1).** `modeling-discipline.md`
> Practice 10 carries the *rules*; this doc carries the *enforcement
> mechanism* — the checker / lens that makes the rules mechanical. Per
> Practice 10's own scoping: "the enforcement mechanism … is design work,
> specified in the planned `docs/design-dissolution-lens.md`."

A **lens** is a deterministic structural projection over the `.dag`
model — no LLM, no judgment. A finding either matches a lens's structural
signature or it does not. Run as a hard CI gate, a lens catches a finding
*every time*, instead of hoping a reviewer notices.

## 1. The core invariant

This doc operationalizes `modeling-discipline.md` Practice 10 — it does
not introduce a parallel rulebook. The invariants below are enforcement
scaffolding for Practice 10's rule, not a competing source of authority.

> **A0 — Every semantic fact must resolve to exactly one canonical
> structural witness path in the model.** A fact may be *derived* from
> a canonical carrier's shape (discriminant, catamorphism, traverse),
> *witnessed* by typed data (`data ... : Algebra<T>`, refinement clause,
> constructor restriction, alias-identity edge, canonical-carrier
> registry row), or *rejected as unknown* through diagnostic flow
> (`Outcome::Rejected`). Multiple structural artifacts pointing at one
> canonical authority (alias / re-export, retirement-ledger row, derived
> operation reading the same witness) are not multiplicities — they are
> the path. It may not be re-derived locally, asserted in prose / name /
> string form, duplicated as a competing authority, or guessed through
> a plausible default.

A0 is the umbrella. A1 is the operation-specific specialization that
originally seeded the lens suite:

> **A1 — Do not hand-roll a derived operation.** If a function's behavior
> is fixed entirely by the shape of a modeled type, it is re-deriving
> something the compiler already derives. *Inverse:* do not nominalize an
> operation as a type — an operation is a function, not a domain
> structure.

A0/A1 are proposed, pending operator ratification (rework-tracker task
A1). Once ratified into Practice 10, A0/A1 become citable hard rules;
this doc remains the enforcement mechanism.

## 2. Two tracks

The lens is not the cure — it is the net.

- **Track 1 — lenses (enforcement).** Catch hand-rolling at CI. This doc.
- **Track 2 — substrate-derivation (the cure).** Make hand-rolling
  *impossible*: the substrate derives an algebraic type's canonical
  operations (variant discriminants, catamorphism) from its declaration,
  so there is nothing left to hand-roll. The burn-down substrate nodes
  (the discriminant-predicate node; `fold_node` / catamorphism nodes)
  are Track 2.

The original seed findings exposed one substrate gap: the substrate
lets you *declare* an algebraic type but does not *derive its canonical
operations*, so workers hand-roll discriminants and catamorphisms.

The broader Layer-1 suite generalizes that lesson: when the substrate
lacks a canonical derived operation, witness table, authority map,
refinement edge, or diagnostic-flow carrier, workers encode the missing
fact locally — in prose, names, strings, duplicate homes, or plausible
defaults. Track 1 holds the line until Track 2 makes those witnesses
derivable or required.

## 3. Methodology — how a lens is derived

Every lens in this doc was derived, and every future lens is derived, the
same way:

> **slipped-by issue → core invariant violated → substrate root cause →
> mechanical lens.**

A lens is added from *real evidence* — a finding that reached merge-ready
despite being a defect (§8), or a checkable review / audit receipt that
names the same defect shape, its live substrate home, and its dissolve
trigger. The lens's signature is the smallest structural pattern that
catches that finding's class with zero false positives on the clean shape.

## 4. Layer 0 — standard compiler hygiene

The floor every mainstream compiler (rustc, clang, tsc, the ML family)
enforces. **On by default in every profile** (§7) — Layer 0 is table
stakes, not a strictness dial. Shipping below this is a regression from a
normal compiler. Build-first: Layer 1 composes Layer-0 primitives.

**Group A — Unused / dead**
- **L0.1 Unused variable** — a bound name never read.
- **L0.2 Unused parameter** — a function/lambda parameter never read.
  *(Layer-1 reuse: the constant-algebra check, L1.1, is this applied to a
  fold algebra.)*
- **L0.3 Unused import.**
- **L0.4 Unused declaration** — a `type` / `fn` / `data` constant
  declared but never referenced. *(Layer-1 reuse: the hollow-type lens,
  L1.3, is this + "no inhabitance edge.")*
- **L0.5 Unused field** — a struct field never read.
- **L0.6 Unreachable code** — statements/arms after a diverging point.
- **L0.7 Dead / constant branch** — an `if`/`match` arm a static check
  proves is never reached.

**Group B — Binding integrity**
- **L0.8 Unbound name** — use of an undefined identifier.
- **L0.9 Use-before-definition** — a binding read before assignment.
- **L0.10 Duplicate definition** — a name declared twice in one scope.
- **L0.11 Shadowing** — a binding silently shadowing an outer one.

**Group C — Match / exhaustiveness** *(gunbc has coproducts + `match` —
directly load-bearing)*
- **L0.12 Non-exhaustive match** — a `match` missing a coproduct variant.
  *(Relates to Practice 1, fail-closed: an unhandled variant is a silent
  gap.)*
- **L0.13 Redundant / unreachable arm** — an arm subsumed by an earlier
  one.

**Group D — Result / fact discipline**
- **L0.14 Ignored result** — a produced value, especially a
  diagnostic-carrying `Outcome`, silently dropped. *(This is Practice 3,
  "facts flow forward," made mechanical.)*

**Group E — Control flow**
- **L0.15 Missing return** — a value-typed function with a path that
  yields no value.

Layer 0 is not all new: L0.12 and L0.14 already have gunbc Practice
analogs (Practices 1, 3) — part of Layer 0 is making an existing Practice
a mechanical lens.

## 5. Layer 1 — dissolution lenses

The gunbc-specific modeling-honesty checks. Each: the finding it kills,
its `.dag` signature, decidability, and the escape valve (the legitimate
non-finding).

### 5.0 Three levels — invariant, theme, lens

The design has three explicit levels. Each plays a different role and
they are not interchangeable:

| level | example | role |
|---|---|---|
| **Invariant** | A0 / A1 (§1) | *the reason* a finding is wrong; citable hard rule |
| **Theme** | derive / witness / canonical-home / fail-closed | *the recurring pattern* a finding belongs to; explanatory tag |
| **Lens** | L1.x (this section) | *the mechanical detector* — structural signature, decidable, with test corpus and escape valve |

> Themes are explanatory tags only. They do not define CI gates,
> test-corpus boundaries, or implementation passes. The mechanically
> enforced unit remains the L1.x lens signature. Two lenses that share
> a theme do not share machinery — they are distinct detectors with
> distinct signatures, decidability arguments, and escape valves, and
> they remain distinct precisely because the §3 methodology says each
> lens's signature must be the smallest structural pattern that catches
> its finding's class with zero false positives.

**Lens → theme catalog:**

| lens | theme(s) |
|---|---|
| L1.1 Discriminant-predicate | derive |
| L1.2 Degenerate-type | witness |
| L1.3 Hollow-type | witness |
| L1.4 Carrier-clone *(lens family — see §5.0 exception)*: parent (whole-carrier clone), **IdenticalVariantPayload** (intra-carrier arms with identical canonical payload signatures), L1.4.b `VariantParameterClone` (intra-carrier variant-level type-only differences) | canonical-home / witness |
| L1.5 Catamorphism *(lens family — see §5.0 exception)*: parent (data-scope walker dissolution), L1.5.b `GeneratedForestCollapse` (emission-scope meta-walk that emits per-variant target artifacts collapsing to K skeletons) | derive |
| L1.6 *(merged into L1.10 — Textual-bypass)* | — |
| L1.7 Off-substrate-fact | witness |
| L1.8 Wrong-home | canonical-home |
| L1.9 Vacuous-arm | fail-closed / witness |
| L1.10 Textual-bypass *(lens family — see §5.0 exception)*: L1.10.a `TemplateHole`, L1.10.b `CanonicalCarrier`, L1.10.c `NameDiscriminantBypass` (spelled-name dispatch where resolved identity is available), L1.10.d `TargetSyntaxByConcat` (grammar-token concat that should be a `TargetSurfaceNode`) | witness / canonical-home |
| L1.11 Plausible-fallback *(lens family — see §5.0 exception)*: parent (constructor-fabricated fallthrough), L1.11.b `PlausibleScalarFallback` (scalar literal fabricates target-authority value) | fail-closed |
| L1.12 Parallel-authority *(lens family — see §5.0 exception)*: parent triggers (lexical / `CanonicalConcept`), L1.12.b `StructuralSimilarity` (different-lexical-name nickname case) | canonical-home |
| L1.13 Skeleton-collapse *(lens family — see §5.0 exception)*: parent (`MatchExpr`-scoped), L1.13.b `DecisionTreeCollapse` (generalized beyond match — any closed-vocab decision tree: match / if-else / string-equality), L1.13.c `TableDecisionTree` (function-encoded total tables over closed vocabularies that should be substrate `TotalMap` / `TotalPolicy` rows) | derive / canonical-home |

The only mechanical merge in this layout is **L1.6 → L1.10** — the
prior doc already stated that L1.10 generalizes L1.6, so the two were
sibling labels for one consolidated signature space.

### 5.1 L1.x acceptance-key names — substrate registry pointer

**Authority pointer (not authority itself).** The canonical
`coverage_defect_*` key set is **substrate data** — declared as rows
in `src/v4/lens/coverage.dag`. That file is the authoritative
registry; this design document **describes** the registry but does
NOT own it. If the table below and `src/v4/lens/coverage.dag`
disagree, **the substrate wins** and this section is stale (open an
issue / PR to reconcile the doc, not the substrate).

This framing follows the no-prose-ledger discipline: a maintained
doc table re-listing facts whose source of truth is substrate data
is a parallel-authority anti-pattern (Practice 9 / INVARIANTS P2;
operator-direct standing 2026-05-19 retiring the maintained-ledger-
doc class). The table is kept inline here as **reading scaffold**
for readers walking the lens family — it lets a reader see at a
glance which acceptance key each L1.x sub-signature contributes to
and which Trigger discriminator the diagnostic payload carries.
Treat each row as a **claim about substrate state at this revision**,
not a definitional rule.

**Downstream-consumer guidance.** Code that needs to register
acceptance / coverage rows (e.g. external coverage trackers, test
harnesses that join coverage receipts) reads the keys from
`src/v4/lens/coverage.dag` directly — NOT from this table.
Substrate-driven projection, not doc-rebase-driven.

| Lens / sub-signature | Canonical acceptance-key name |
|---|---|
| L1.1 Discriminant-predicate | `coverage_defect_discriminant_predicate` |
| L1.2 Degenerate-type | `coverage_defect_degenerate_type` |
| L1.3 Hollow-type | `coverage_defect_hollow_type` |
| L1.4 Carrier-clone | `coverage_defect_carrier_clone` |
| **IdenticalVariantPayload** (sub-signature of Carrier-clone family) | `coverage_defect_carrier_clone` *(shared with parent + L1.4.b — `CarrierCloneTrigger::IdenticalVariantPayload` distinguishes duplicate-payload arms from whole-carrier-clone and variant-parameter-clone)* |
| **L1.4.b** `VariantParameterClone` (sub-signature of Carrier-clone family) | `coverage_defect_carrier_clone` *(shared with parent — sub-signature contributes findings into the same acceptance key; diagnostic payload carries a `CarrierCloneTrigger` discriminator distinguishing whole-carrier-clone from variant-clone)* |
| **L1.5.b** `GeneratedForestCollapse` (sub-signature of Catamorphism family) | `coverage_defect_catamorphism` *(shared with parent — diagnostic payload carries a `CatamorphismScopeTrigger` discriminator distinguishing data-scope walker (parent) from emission-scope meta-walk (this sub-signature))* |
| **L1.10.c** `NameDiscriminantBypass` (sub-signature of Textual-bypass family) | `coverage_defect_template_hole` *(shared with the textual-bypass family — diagnostic payload carries a `TextualBypassKind` discriminator distinguishing the four sub-signatures a/b/c/d)* |
| **L1.10.d** `TargetSyntaxByConcat` (sub-signature of Textual-bypass family) | `coverage_defect_template_hole` *(shared with the textual-bypass family — same `TextualBypassKind` discriminator)* |
| **L1.11.b** `PlausibleScalarFallback` (sub-signature of Plausible-fallback family) | `coverage_defect_plausible_fallback` *(shared with parent — diagnostic payload carries a `PlausibleFallbackKind` discriminator distinguishing constructor-fabrication (parent) from scalar-fabrication (this sub-signature))* |
| **L1.13.b** `DecisionTreeCollapse` (sub-signature of Skeleton-collapse family) | `coverage_defect_skeleton_collapse` *(shared with parent — diagnostic payload carries a `SkeletonCollapseScopeTrigger` discriminator distinguishing `MatchExpr` (parent) from `DecisionTreeGeneralized` (this sub-signature) and `TableDecisionTree` (L1.13.c))* |
| **L1.13.c** `TableDecisionTree` (sub-signature of Skeleton-collapse family) | `coverage_defect_skeleton_collapse` *(shared with parent + L1.13.b — `SkeletonCollapseScopeTrigger` discriminator extended with `TableDecisionTree` distinguishing function-as-table from match-collapse and decision-tree-generalized)* |
| L1.5 Catamorphism | `coverage_defect_catamorphism` |
| ~~L1.6 Emit/template~~ | **retired — see L1.10.a below; no `coverage_defect_emit_template` key** |
| L1.7 Off-substrate-fact | `coverage_defect_off_substrate_fact` |
| L1.8 Wrong-home | `coverage_defect_wrong_home` |
| L1.9 Vacuous-arm | `coverage_defect_vacuous_arm` |
| **L1.10.a** `TemplateHole` (sub-signature of Textual-bypass family) | `coverage_defect_template_hole` |
| **L1.10.b** `CanonicalCarrier` (sub-signature of Textual-bypass family) | `coverage_defect_canonical_carrier` |
| L1.11 Plausible-fallback | `coverage_defect_plausible_fallback` |
| L1.12 Parallel-authority | `coverage_defect_parallel_authority` |
| **L1.12.b** `StructuralSimilarity` (sub-signature of Parallel-authority family) | `coverage_defect_parallel_authority` *(shared with parent — sub-signature contributes findings into the same acceptance key; no separate key needed since the resolution table and outcome shapes are the parent's)* |
| L1.13 Skeleton-collapse | `coverage_defect_skeleton_collapse` *(reserved-proposed — enforcement not active until skeleton extraction + classifier + clearing receipts land)* |

**Migration notes for existing downstream consumers:**
- Any consumer carrying `coverage_defect_emit_template` is **stale**;
  rename to `coverage_defect_template_hole` (the renamed
  sub-signature).
- Any consumer carrying `coverage_defect_string_escape_hatch` (the
  pre-merge name) is **stale**; replace with the two sub-signature
  rows `coverage_defect_template_hole` AND
  `coverage_defect_canonical_carrier`. L1.10 itself is no longer
  a single mechanical unit — it's a lens family per §5.0's exception.
- Further changes to the L1.x taxonomy update the substrate
  `src/v4/lens/coverage.dag` rows first; this table follows
  (substrate-first, doc-second). Treat any stale-table reports as
  doc-update opportunities, not substrate-rebase opportunities.

### L1.1 Discriminant-predicate lens — kills *predicate dissolution*

- *Signature:* a `Bool`-returning `fn` over a coproduct whose body is a
  single `match` with **literal `true`/`false` arms** per variant.
  **Extended (the #3249 lesson):** also a `fold`/`cata` call whose algebra
  is **constant in its recursive/accumulator argument** — a discriminant
  laundered through a fold. (The constant-algebra tell is L0.2,
  unused-parameter, applied to a fold algebra.)
- *Decidable:* fully — name + param type + body shape.
- *Verdict:* hard error on `std/` / substrate / reusable-helper files.
- *Escape:* a `match` whose arms do *computed* work (not literal
  `true`/`false`) is genuinely distinct → clean 🟢.
- *Kills:* `nat_is_zero` (#3255), `free_monoid_non_empty` (#3250),
  `free_monoid_is_empty`-via-fold (#3249), the `float_body_is_nan` /
  `edge_is_named` saturation class.

**Concrete match — the basic shape (`nat_is_zero` (#3255)):**
```dag
type Nat = Zero | Succ { prev: Nat }

fn nat_is_zero(n: Nat) -> Bool {
  match n {
    Zero               => true   // ← literal Bool per variant
    Succ { prev: _ }   => false
  }
}
```
The function's behavior is entirely determined by `n`'s top constructor.
The compiler already knows the variant from the parsed `match` — the
function is re-deriving the discriminant the substrate could expose for
free.

**Concrete match — the laundered shape (`free_monoid_is_empty` (#3249)):**
```dag
fn free_monoid_is_empty(m: FreeMonoid<T>) -> Bool {
  free_monoid_fold(
    m:    m,
    init: true,
    step: fn(_acc: Bool, _elem: T) -> Bool { false }   // ← acc never read
  )
}
```
A fold whose algebra ignores its recursive/accumulator argument is not
a catamorphism — it's a discriminant in disguise. The constant-algebra
tell is L0.2 (unused-parameter) applied to the fold's `step`. Without
this extension L1.1 would miss it.

**Clean shape:** delete the function; consume the substrate-derived
discriminant directly. The `match n { Zero => ..., Succ => ... }` form
already gives the caller what they need — `nat_is_zero` was an extra
layer they didn't need to write.

### L1.2 Degenerate-type lens — kills *nominalization*

- *Signature:* (a) a struct whose **every field is function-typed** with
  **no `data` instance**; (b) **N near-identical single-field structs**.
- *Decidable:* yes — field types, cross-ref for instances, similarity
  scan.
- *Verdict:* hard error on the wrapper shape.
- *Escape:* a genuine algebraic structure — non-function data fields,
  multiple `data` inhabitants (`Monoid`) — does not match.
- *Kills:* #3256's 26 `type ListMap { apply: fn }` wrappers; the
  `{ spelling: String }` ×N degenerate-wrapper class.

**Concrete match — (a) struct-of-functions (`ListMap` wrappers (#3256)):**
```dag
type ListMap<A, B> {
  apply: fn(List<A>) -> List<B>      // ← every field is function-typed
}
// no `data foo: ListMap<...> = ...`  ← no instance gives it algebraic content
```
`ListMap<A, B>` *is* `fn(List<A>) -> List<B>` wearing a name. The wrapper
adds no structure: there's no second field, no constructor laws, no
multiple inhabitants. It's an operation nominalized as a type.

**Concrete match — (b) N near-identical single-field structs:**
```dag
type Keyword    { spelling: String }
type Identifier { spelling: String }
type Symbol     { spelling: String }
// three "types" with structurally identical shape, distinguished only
// by the name — the closed set is doing coproduct's job
```

**Clean shape:**
```dag
// (a) — let the operation be a function, not a type
fn list_map<A, B>(xs: List<A>, f: fn(A) -> B) -> List<B> { ... }

// (b) — model the closed set as a coproduct, factor the shared field
type LexicalKind = Keyword | Identifier | Symbol
type LexicalToken { kind: LexicalKind, spelling: String }
```
Genuine algebraic structures (a `Monoid<T>` with `unit` + `combine` and
multiple `data` inhabitants like `additive_monoid_int`, `string_monoid`)
do not match — they have non-function content and real multiplicity.

### L1.3 Hollow-type lens — kills *hollow declarations* (Practice 8's hollow-alias finding, at the type level)

- *Signature:* a declared type **nothing inhabits** — no `data` instance,
  no `fn` returning it, no alias-identity to a substrate type. (This is
  L0.4, unused-declaration, + "no inhabitance edge.")
- *Decidable:* yes — cross-reference whether anything constructs it.
- *Verdict:* hard error.
- *Escape:* a type that *is* constructed, or aliases a substrate carrier,
  passes.

**Concrete match — declared, never inhabited:**
```dag
type ParseError {
  message: String
  span:    SourceSpan
}
// no `fn ... -> ParseError`, no `data ... : ParseError = ...`,
// no `type T = ParseError` alias, no record field of type ParseError.
// The type is a name with no edges into the rest of the model.
```
The author intended `ParseError` to mean something, but no code path
produces a value of it. From the substrate's perspective the type is
inert — it asserts an intention without committing structurally.

**Clean shape:** either delete the declaration (if it was speculative)
or make at least one inhabitance edge real:
```dag
fn parse(s: String) -> Outcome<Ast> {
  Rejected { diagnostic: ParseError { message: ..., span: ... } }   // ← inhabitance
}
```
Aliasing a substrate carrier also discharges the lens:
```dag
type ParseError = Diagnostic   // ← alias-identity is a structural edge
```

### L1.4 Carrier-clone lens — kills *carrier dissolution*

- *Signature:* a locally-declared coproduct **structurally isomorphic to
  a `std/` carrier** (`Foo { value: T } | FooRejected { diagnostic }` ≅
  `Outcome<T>`).
- *Decidable:* yes — type-shape match against the `std/` carrier set.
- *Verdict:* hard error.
- *Escape:* a coproduct carrying a payload the std carrier genuinely
  cannot express passes.

**Concrete match — `Outcome<T>` clone (the F2 / `NormalizeChildrenResult` shape):**
```dag
// in src/v4/compiler/03_normalize.dag
type NormalizeChildrenResult
  = NormalizedChildren        { children: List<Edge> }   // ← Produced
  | NormalizeChildrenRejected { diagnostic: Diagnostic } // ← Rejected
```
Structurally identical to `Outcome<List<Edge>>` — same two-variant
shape, same payload kinds, just renamed. The local coproduct adds no
information the canonical `Outcome<T>` can't express.

**Clean shape:**
```dag
// in src/v4/std/diagnostic.dag (canonical)
type Outcome<T> = Produced { value: T } | Rejected { diagnostic: Diagnostic }

// in src/v4/compiler/03_normalize.dag
fn normalize_children(...) -> Outcome<List<Edge>> { ... }
```
A coproduct that genuinely *can't* be expressed by the std carrier
passes — e.g. a three-variant `Cached | Produced | Rejected` where
`Cached` carries information `Outcome<T>` doesn't model.

#### L1.4.b Variant-parameter-clone — kills *variant-level type-only differences within one coproduct*

> **Status: reserved-proposed.** Sub-signature of L1.4 at a finer grain:
> the parent catches a *whole coproduct* that clones a `std/` carrier;
> L1.4.b catches *variants WITHIN one coproduct* that are structurally
> identical modulo field-name nickname, differing only in field type
> AND the field type unambiguously recovers the variant tag (per the
> tag-recoverability check below — operator-direct refinement
> 2026-05-21). The field type IS the discriminator AND must
> distinguish the variants from each other for parameterization to
> preserve information — which is precisely the case parametric
> carriers are designed for. Practice 11 (parameterize-don't-duplicate)
> applied at the variant level. **Enforcement gate**
> (parallel to L1.13 / L1.12.b): not active until (a)
> `v4.lens.structural_similarity` lands carrying per-variant
> field-shape facts, (b) the L1.4 diagnostic payload distinguishes
> whole-carrier-clone (parent) from variant-clone (this signature),
> (c) the substrate has at least one Practice-11-parameterized
> dissolution shape landed as the migration target (the Locus rework
> below, or equivalent).

- *Signature:* a coproduct contains two or more variants whose
  **field structures are isomorphic modulo field-type substitution**
  AND **the variant tag is recoverable from the payload type**. After
  α-renaming variant-bound names by canonical position (per field
  role, not by spelled name), the variant skeletons are structurally
  identical with only the field-type identity differing, AND the
  field-type identity itself unambiguously distinguishes which
  variant constructed the value. Both conjuncts are required — the
  variants must discriminate on type (so the parametric form
  `Locus<T>` preserves enough information to recover the variant
  tag from `T`), not just be isomorphic in shape.

- *Decidable:* yes — substrate-readable per-variant field-shape
  facts (from `v4.lens.structural_similarity`'s `TypeShape.variant_set`,
  see §10.2). The lens compares VariantShapes pairwise within each
  coproduct, applying field-name α-renaming. Same mechanical contract
  as L1.13 (skeleton extraction), at variant scope instead of
  match-arm scope.
  **Tag-recoverability check (load-bearing, per operator-direct
  refinement 2026-05-21; granularity sharpened per openai-pro
  REQUEST_CHANGES 2026-05-21; equivalence-class scope sharpened per
  codex BLOCKING #15978 2026-05-21):** after the variant-skeleton
  comparison identifies all variants sharing a skeleton (the
  **skeleton-equivalence class**), the lens checks that the
  **canonical payload signatures across the ENTIRE equivalence class
  are pairwise-distinct as a whole set** — NOT pairwise per candidate
  pair. The variant-tag projection from the payload-signature *product*
  (the full ordered tuple of field types, by canonical-position
  normalization) into the equivalence class's variant subset must be
  injective over the class. Why pairwise-over-the-set rather than
  pairwise-per-pair: with three variants `V1 { x: A }`, `V2 { x: A }`,
  `V3 { x: B }`, the pair (V1, V3) has distinct signatures and the
  pair (V2, V3) has distinct signatures, but the class as a whole
  contains the duplicate `(A,)` shared by V1+V2. Pairwise-per-pair
  would falsely PASS on (V1, V3) and (V2, V3) and propose `Locus<T>`
  that erases the V1-V2 distinction; equivalence-class injectivity
  correctly FAILS the whole class. Mechanically: the check is
  "let S = {payload-signature(v) | v in equivalence-class}; |S| ==
  |equivalence-class|."
  The check is at **payload-signature granularity**, NOT at per-field-
  type granularity:
  - **Single-field variants** (the original Locus motivating case):
    payload signature reduces to the single field type. `V1 { x: A }`
    + `V2 { x: A }` share signature `(A,)` → FAIL (same payload
    signature; variant tag not recoverable). `V1 { x: A }` +
    `V2 { x: B }` have signatures `(A,)` vs `(B,)` → PASS.
  - **Multi-field variants:** signature is the full ordered tuple.
    `V1 { shared: A, side: B }` + `V2 { shared: A, side: C }` have
    signatures `(A, B)` vs `(A, C)` — DISTINCT signatures even though
    field `shared` shares type `A`. PASS — parametric form preserves
    the discrimination via `side`'s type; per-field uniqueness would
    have falsely failed this case.
  - **Degenerate same-signature multi-field:** `V1 { a: A, b: B }` +
    `V2 { c: A, d: B }` (different field names, same ordered type
    tuple under canonical-position normalization) share signature
    `(A, B)` → FAIL (per-field-name renaming doesn't add discrimination
    if the type tuple is identical).

  Without equivalence-class-wide payload-signature-injective tag-
  recoverability, parameterizing would erase information the original
  coproduct represented; the lens cannot honestly recommend the
  dissolution.

  (Earlier wording history: original draft specified "field types
  pairwise-distinct" which over-constrained at per-field granularity
  (openai-pro caught — under-fired on multi-field cases). Subsequent
  revision said "pairwise-distinct between candidate variants" which
  under-constrained at the pairwise-per-pair scope (codex caught —
  a 3-variant class with two duplicates would let the lens fire on
  the non-duplicate pairs while leaving the duplicate-pair erasure
  in place). Current spec is **equivalence-class-wide payload-
  signature injectivity** — the honest endpoint.)

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **ConceptDisambiguation row.** If the variants represent
    semantically distinct concepts that happen to share a structural
    shape (e.g., legitimate cases where Node-id and Symbol-id are
    different domain notions despite both being single-field
    records), a `ConceptDisambiguation` row marks them as distinct.
    Same resolution shape (R3) as parent L1.12.
  - **Carrier already parametric.** The lens never fires on a
    parametric carrier's instantiations — those ARE the
    parameterization. Only fires on non-parametric coproducts with
    variant-level type-only redundancy.
  - **Variant carries non-type-only difference.** If two variants
    have the same field shape AND same field types but different
    auxiliary structure (e.g., one variant carries an additional
    refinement clause, attribute, or pragma), that auxiliary
    structure is genuine variant-axis information; the lens does
    not fire. The trigger requires the type-substitution to be the
    SOLE structural difference.
  - **Tag-not-recoverable from payload signature (equivalence-class-
    wide check).** If **any two variants in the skeleton-equivalence
    class** share the **same canonical payload signature** (the full
    ordered tuple of field types under canonical-position normalization),
    parameterizing would ERASE the variant tag for those two variants —
    the original coproduct carried information the parametric form
    cannot preserve. The lens does NOT fire on **the entire class**;
    every variant in the class stays as a distinct case (no partial
    dissolution — partial would leave the duplicate-pair erasure
    in place). Examples:
    - `V1 { x: A }` + `V2 { x: A }` — both signatures `(A,)` →
      FAIL tag-recoverability → STAY.
    - `V1 { a: A, b: B }` + `V2 { c: A, d: B }` — both signatures
      `(A, B)` (field-name α-renaming doesn't change the type
      tuple) → FAIL → STAY.
    - `V1 { shared: A, side: B }` + `V2 { shared: A, side: C }` —
      signatures `(A, B)` vs `(A, C)` are distinct → PASS
      tag-recoverability → lens fires (this is a multi-field
      dissolution candidate the parametric form preserves
      discrimination over).
    This is the operator-direct refinement (2026-05-21, granularity
    sharpened per openai-pro REQUEST_CHANGES same day; equivalence-
    class scope sharpened per codex BLOCKING #15978 same day)
    tightening the bar from "identical arity is enough" to
    "identical arity AND equivalence-class-wide payload-signature
    injectivity" (i.e., `|{signature(v) | v in class}| == |class|`).
    Without this guard, the lens would over-recommend parameterization
    that loses information.

- *Clearing receipt (single authoritative resolution table — same R1–R5 inherited from L1.12 family):*
  the substrate carries one of the inherited resolution shapes:
  (R1) alias-identity rewrite for the redundant variants (parametric
  carrier replaces them), (R3) `ConceptDisambiguation` row marking
  them as distinct, (R4) refinement edge declaring the relationship,
  or the dissolution rewrite which removes the trigger entirely.

- *Fix-confidence: templated auto-apply* — two clean-shape templates
  the lens emits as candidate `Diff`s, reviewer picks before commit:
  - **(Pattern A) Type-level parameterization.** Lift the whole
    carrier to a parametric form when the discriminator is purely
    type-level:
    ```dag
    // Before:
    type Locator = NodeLocator { node: Node } | PortLocator { port: Symbol }
    // After (Pattern A):
    type Locator<T> = Locator { at: T }
    // Callers thread the parameter: Locator<Node>, Locator<Symbol>.
    ```
    Use when callers don't need to enumerate the variant-axis at a
    single site (i.e., the discriminator can be carried at the type
    level by the calling context).
  - **(Pattern B) Parametric sub-carrier lift.** When the parent
    coproduct has asymmetric variants that DON'T fit one parametric
    shape, lift only the shared shape into a parametric sub-carrier
    and keep the coproduct discriminator for the asymmetric cases:
    ```dag
    // Before:
    type Locus
      = Textual { file: Symbol, extent: Extent }
      | NodeLocus { node: Node }
      | PortLocus { port: Symbol }
    // After (Pattern B):
    type Anchored<T> { at: T }
    type Locus
      = TextualLocus { file: Symbol, extent: Extent }
      | NodeAt(Anchored<Node>)
      | PortAt(Anchored<Symbol>)
    ```
    Use when callers DO enumerate variants at a site (e.g., a
    match-over-Locus that handles file ranges differently from
    anchored references); the parametric sub-carrier eliminates the
    intra-anchor variant redundancy while keeping the coproduct
    discriminator for the asymmetric variant.

  Reviewer picks pattern at the candidate-state stage; same flow as
  any typed-Diff. The auto-fix can emit both candidate forms and let
  the reviewer choose, since (A) and (B) commit to different call-site
  ergonomics.

- *Decidability boundary (explicit):* the lens catches variants whose
  field structures are bijection-isomorphic with type-only differences.
  It does **not** catch:
  - Variants whose field structures differ in arity or nesting (no
    skeleton bijection).
  - Variants whose field types differ but ALSO carry distinct
    structural information (e.g., a refinement carrier attached).
  - Cross-coproduct variant-skeleton matches — L1.4.b is intra-
    coproduct only. Variants of one coproduct whose shape matches
    variants of an unrelated coproduct are caught by L1.4 (whole-
    carrier clone) or L1.12.b (cross-decl structural similarity)
    instead.

  The lens output is restricted to within-one-decl findings to keep
  the trigger mechanically tight; broader matches escalate to the
  appropriate sibling lens.

- *Kills (real corpus — seed examples; full sweep deferred to a
  modeling-lane sweep PR):*
  - **`Locus.NodeLocus` / `Locus.PortLocus`** in `src/v4/std/diagnostic.dag:22`.
    `NodeLocus { node: Node }` and `PortLocus { port: Symbol }` are
    both single-field anchored records discriminating only on field
    type (Node vs Symbol). The field names (`node`, `port`) are
    nicknames for the anchor type, not independent semantic role.
    The asymmetric Textual variant (`{ file: Symbol, extent: Extent }`)
    doesn't fit a uniform parametric shape — Pattern B is the natural
    fix: lift `Anchored<T> { at: T }` and rewrite the two redundant
    variants to use it; keep TextualLocus distinct. Cited as seed for
    the modeling-lane sweep.
  - **(Additional kills TBD by the modeling-lane sweep.)** L1.4.b's
    enforcement gate is (c) "the substrate has at least one
    Practice-11-parameterized dissolution shape landed as the
    migration target" — that target is the Locus rework. Once that
    lands, the lens binding is honest (it has a worked precedent for
    the dissolution shape).

- *Producer stage (see §10):* L1.4.b consumes
  `v4.lens.structural_similarity`'s `TypeShape.variant_set` — the
  same per-variant `VariantShape` records used by L1.12.b's type-scope
  (C1). No new producer stage needed; the structural-shape index is
  the single source of variant-shape facts, consumed by both
  cross-decl (L1.12.b) and intra-decl (L1.4.b) lenses. Facts Flow
  Forward / P2: one mechanism, multiple downstream projections.

*(Moved: L1.4.c originally drafted here as Policy-table-as-functions.
On operator review the binding was relocated under L1.13's Skeleton-
collapse family as **L1.13.c TableDecisionTree** — see that section.
Rationale: "function body is a typed table" is algebraically closed-
vocab decision-table dissolution, not whole-carrier clone. The
mechanism is `decision_tree_shape` (same as L1.13.b), not the type-
declaration-shape mechanism that grounds L1.4 + L1.4.b.)*

### L1.5 Catamorphism lens — kills *walker / traverse dissolution*

- *Signature:* a `fn` recursing over a structural type by `match`ing its
  variants + self-calling on the sub-structure; or a `fold` body that is a
  `match acc { Rejected => propagate ; Ok => continue }` short-circuit
  ladder.
- *Decidable:* on the clean shape — recursion mirrors the data shape.
- *Verdict:* hard error on the clean shape; **reviewer-confirm** on
  genuinely-irregular recursion (call graph ≠ data graph) → clean 🟢.
- *Kills:* `ci.dag`'s hand-rolled `List` combinators (#3213); the
  resolve/normalize walkers (#3225).

**Concrete match — clean recursion mirrors the data shape (`ci.dag` `member` (#3213)):**
```dag
// in src/v4/workflow/ci.dag
fn ci_member(s: Symbol, xs: List<Symbol>) -> Bool {
  match xs {
    Nil                       => false
    Cons { head: h, tail: t } =>
      match symbol_eq(a: s, b: h) {
        True  => true
        False => ci_member(s: s, xs: t)   // ← recurse on the sub-structure
      }
  }
}
```
The function recurses by matching `List`'s variants (`Nil` / `Cons`)
and self-calling on `tail`. That recursion *is* the catamorphism over
`List` — the substrate's `fold` would discharge it. Same shape for
`any`, `all`, `count_if`, `find`: each is a fold-with-a-different-algebra.

**Concrete match — short-circuit fold ladder (resolve/normalize walkers (#3225)):**
```dag
fn resolve_children(...) -> Outcome<List<Node>> {
  match resolve(head) {
    Rejected { diagnostic: d } => Rejected { diagnostic: d }   // ← propagate
    Produced { value: h }      => match resolve_children(tail) {
      Rejected { diagnostic: d } => Rejected { diagnostic: d } // ← propagate
      Produced { value: t }      => Produced { value: cons(h, t) }
    }
  }
}
```
The `match acc { Rejected => propagate ; Ok => continue }` ladder is
`Outcome`'s monadic traverse over `List` — also a substrate-derivable
shape.

**Clean shape:** consume the substrate-derived combinator instead.
```dag
fn ci_member(s: Symbol, xs: List<Symbol>) -> Bool =
  list_any(xs, fn(h) { symbol_eq(a: s, b: h) })

fn resolve_children(xs: List<Node>) -> Outcome<List<Node>> =
  traverse_outcome(xs, resolve)
```
Genuinely-irregular recursion — the call graph does *not* mirror the
data graph, e.g. a graph walker that revisits visited nodes via a
side-table — falls under *reviewer-confirm* rather than hard-error.

#### L1.5.b Generated-forest collapse — kills *meta-walk that emits a per-variant target-language artifact when the forest collapses to K skeletons*

> **Status: reserved-proposed.** Sub-signature of L1.5 (Catamorphism)
> at the **emission scope** instead of the data scope. Parent L1.5
> catches a fn that recurses over a typed graph; L1.5.b catches a fn
> that walks a closed coproduct's variant-set AND emits one target-
> language artifact per variant, when the generated forest collapses
> to K distinct skeletons over N variants (K < N). The L1.13 skeleton-
> classification algorithm is reused at the emission scope — same
> mechanism, different scope (Practice 11 applied to lens machinery
> itself). **Enforcement gate**: not active until (a)
> `v4.lens.generated_forest_shape` lands, (b) the L1.5 diagnostic
> payload carries a `CatamorphismScopeTrigger` discriminator
> distinguishing data-scope (parent) from emission-scope (this).

- *Signature:* a fn whose body is a `map`/`fold` over a closed
  coproduct's variant-set that emits a per-variant target artifact
  (text, AST, or any derived projection), AND the per-variant
  emission templates collapse to K distinct skeletons (under L1.13's
  skeleton-extraction algorithm) with K < N. The forest IS
  parametric — should be a typed projection over the coproduct + a
  parametric emission template, not a hand-walked forest.

- *Decidable:* yes — `generated_forest_shape` produces the per-variant
  emission skeletons via the same `NormalizedArmBody` algebra
  `match_arm_skeleton` already produces. K/N classification reuses
  L1.13's `PureTemplate / Outlier / MultiOutlier / Categorical /
  Mixed` distribution-shape thresholds at the forest scope.

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **Mixed forest.** When the generated forest's per-variant
    skeletons are genuinely distinct (K = N or close to N), the forest
    is legitimate per-variant dispatch; passes naturally.
  - **Already a typed projection.** A `data ... : Projection<T, U>`
    substrate row exists with the same source-coproduct + target-artifact
    pairing; the fn is the canonical projection. Passes via (R1).
  - **Asymmetric coverage with substrate-derived subset witness.**
    When the forest deliberately covers a strict subset of variants
    (e.g., only the fielded variants of a sum-with-units), the
    asymmetry is genuine variant-axis information AND the substrate
    must carry a `VariantCoverage` row citing a substrate-derived
    subset predicate:
    ```dag
    data fielded_only_coverage: VariantCoverage = {
      carrier: T,
      subset:  FieldedVariants,             // substrate-recognized predicate over variant shapes
      witness: DerivedFromVariantShape,     // names the structural derivation (variant has ≥1 field, etc.)
    }
    ```
    Local unregistered filters (`children |> filter(child => child.children |> count > 0)`)
    do **NOT** pass — the filter IS the new hand-rolled discriminator
    the lens is supposed to catch. The escape requires structural
    authority; otherwise the case is L1.5.b / L1.10.c / L1.13.b
    territory depending on the filter's shape.

- *Clearing receipt:* substrate carries the parametric projection
  (a `data table: Projection<Source, Target> = ...` row); the fn
  (if retained) reads it.

- *Fix-confidence: templated auto-apply.* The (C2-style) auto-fix
  emits the parametric projection + a single emission template
  consuming it. Same algorithm as L1.13's auto-fix for the match-arm
  case, applied at the forest scope.

- *Decidability boundary:* L1.5.b fires on closed-coproduct iteration
  (variant-set known from `04_infer`). Open-set iteration (e.g., walk
  arbitrary fields not constrained to a closed coproduct) is out of
  scope.

- *Kills (real corpus — seed examples):*
  - **`emit_enum_shared_accessors`** (`src/v2/05_emit_rust.dag`,
    operator-identified). The function finds fields present in all
    fielded variants, filters by consistent type, then **generates an
    accessor method whose Rust body contains one match arm per
    variant** — the generated arms are mostly one skeleton (bind +
    clone the shared field) plus a panic arm for unit variants.
    Generated forest of K=2 (clone-arm + panic-arm) over N variants.
    Fires on L1.5.b. Clean shape: a `Projection<VariantSet,
    AccessorBody>` table consumed by a generic accessor-emit fn that
    reads it.

- *Producer stage (see §10):* L1.5.b consumes
  `v4.lens.generated_forest_shape` — variant-iteration evidence +
  per-variant emission skeletons. Facts Flow Forward / P2: forest-
  shape extraction is the same algebra as match-arm-skeleton extraction,
  reused at emission scope.

### L1.6 Deprecated alias — see L1.10.a `TemplateHole`

L1.6 is retained as a deprecated alias to keep prior test names,
slipped-by ledger references, and external citations traceable. The
original L1.6 "Emit/template lens" — catching string literals carrying
positional placeholders (`{0}`, `{1}`, …) used as emitters — is now
[L1.10.a `TemplateHole`](#l110-textual-bypass-lens--kills-typed-model-bypass-via-string-proposed-merged-l16),
a sub-signature of the L1.10 Textual-bypass lens family. The
`TemplateHole` sub-signature preserves L1.6's registry-free
decidability for placeholder literals.

### L1.7 Off-substrate-fact lens — kills *prose-asserted facts* (proposed)

> **Status: proposed.** Derived from the 2026-05-18 ingest of findings F3
> (hand-rolled lattice merges with prose-only inhabitance), F4 (fixed
> widths carried by identifier, not structure), F11 (`ResourceHandle`
> opacity claimed in a comment over a freely-constructible record).
> Generalizes the standing "machine-readable inhabitance is the bar"
> ruling.

- *Signature:* a `.dag` declaration whose header comment or identifier
  contains a fact-bearing token from a closed vocabulary
  (`inhabits <Algebra>`, `opaque`, `non-empty`, `Word<N>`, `Float<N>`,
  `bounded`, `non-forgeable`) **without a matching structural artifact**:
  - claimed algebra inhabitance → no `data ... : Algebra<T>` row in scope;
  - claimed cardinality / width → **no recursively-discharged refinement
    chain** on the carrier. A name-encoded width fact must be witnessed
    all the way down: `Word64` requires `bytes: List<Byte> where len(_) == 8`
    AND every `Byte` reachable from a `Word64` carries
    `bits: List<Bit> where len(_) == 8`. A single outer refinement that
    bottoms out at an unconstrained carrier (`List<Byte>` whose `Byte`
    has `List<Bit>` with no length clause) does not discharge `Word64`'s
    64-bit claim; an arbitrary-bit-count `Byte` still inhabits the
    "well-formed" `Word64`. Same recursion for `Float32`/`Float64`: the
    width fact must distinguish the two structurally (different
    exponent/significand width refinements), not by reusing one
    unconstrained `FloatBody`;
  - claimed opacity / non-forgeability → no constructor restriction
    (the type is a record whose fields are all freely constructible from
    user-reachable substrate values).
- *Decidable:* yes — the claim vocabulary is a closed set; the structural
  counterpart is locatable (data table, refinement clause, constructor
  visibility). Width-claim discharge is decidable by **recursive descent
  through the carrier**: walk every field whose type is a substrate
  collection, require a length refinement at each level until the
  recursion bottoms out at a fixed-cardinality leaf or a primitive bit.
- *Verdict:* hard error on `std/` and substrate files.
- *Escape:* prose without fact-bearing tokens (rationale, anchors,
  examples) passes. A claim *with* the structural counterpart present
  passes.
- *Kills:* the lattice-without-witness shape, the width-in-the-name
  shape, the opacity-in-the-comment shape.

**Concrete match — F3 (`dsl/std/fermi.dag`):**
```dag
// FermiDepth inhabits Lattice<FermiDepth>.   ← claim
fn fermi_meet(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth { ... }
fn fermi_join(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth { ... }
// no `data fermi_lattice: Lattice<FermiDepth> = { meet: fermi_meet, ... }`
```

**Concrete match — F4 (`src/v4/std/machine.dag`):**
```dag
type Word64 { bytes: List<Byte> }   // `64` only in the name
type Byte   { bits:  List<Bit>  }   // any length representable
```

**Concrete match — F11 (`dsl/std/resources.dag`):**
```dag
// Opaque proof of resource acquisition. Only the compiler's acquire
// nodes can mint these -- user code cannot construct handles directly.
type ResourceHandle {                 // ← claim says opaque
  type: String                        // ← record, all fields constructible
  resource_id: String
  key: String
  cap: Secret
}
```

**Clean shape (the cure):** width refinements discharge recursively;
`Float32` and `Float64` are distinguished by their refinement clauses,
not by sharing one carrier.
```dag
// machine.dag — recursive refinement chain
type Bit
type Byte   { bits:  List<Bit>  where len(_) == 8 }
type Word64 { bytes: List<Byte> where len(_) == 8 }   // 8 × 8 = 64 ✓

// float.dag — structurally distinct float widths
type Float32 {
  sign:                  Bit
  biased_exponent:       List<Bit> where len(_) == 8
  trailing_significand:  List<Bit> where len(_) == 23
}
type Float64 {
  sign:                  Bit
  biased_exponent:       List<Bit> where len(_) == 11
  trailing_significand:  List<Bit> where len(_) == 52
}

// fermi.dag — algebra membership as a typed witness, not prose
data fermi_lattice: Lattice<FermiDepth> = { meet: fermi_meet, join: fermi_join }
```

### L1.8 Wrong-home lens — kills *orphan operations* (proposed)

> **Status: proposed.** Derived from finding F5 (`nat_compare` defined in
> `src/v4/std/float.dag` rather than `src/v4/std/nat.dag` or
> `src/v4/std/algebra.dag`). Mechanizes MODELING M9 (DFS the concept
> DAG).

- *Signature:* the lens derives the function's **primary (domain)
  concept** — the unique receiver type the function operates over —
  and requires the function to live in that type's home file.
  Observation codomains (Bool, Ordering, comparison results) are
  *results of* the receiver concept, not peer home signals, and are
  classified structurally via a substrate registry:
  ```dag
  // canonical observation carriers — substrate data, not a hardcoded list
  data canonical_observations: Set<CanonicalObservation> = {
    CanonicalObservation { type: Bool,     role: predicate-result   },
    CanonicalObservation { type: Ordering, role: comparison-result  },
    CanonicalObservation { type: Unit,     role: side-effect-result },
    ...
  }
  ```
  The primary concept is selected by this priority cascade, all
  structurally decidable from the parsed model:
  1. **Declared witness target.** If `f` appears as a field of a
     `data ... : Algebra<T> = { ... f: ... }` witness, the primary
     concept is `T` (the algebra's type parameter). `f`'s home is `T`'s
     home file, regardless of what its arguments or return look like.
  2. **Domain-typed function.** Strip observation codomains and look at
     the remaining argument/return types. If the **non-observation
     types** are all the same type `T` (the function operates over `T`
     and returns either `T` or an observation of `T`), the primary
     concept is `T`. `fn(Nat, Nat) -> Ordering` selects Nat; `fn(Nat) -> Nat`
     selects Nat; `fn(Nat, Nat) -> Bool` selects Nat.
  3. **Upstream argument convergence.** Otherwise, if every
     non-observation type in the signature is declared in a single
     file `X`, and `f`'s current file `Y` imports `X`, the primary
     concept is the type in `X` and the home is `X`.
  4. **No primary concept (cross-cutting).** If none of (1)–(3)
     selects a single owning type, the function is genuinely
     cross-cutting and the lens does not fire.
  The lens fires when (1), (2), or (3) selects a primary-concept home
  `X` and `f` lives in `Y ≠ X`. Symmetric rule for `data` declarations.
- *Decidable:* yes — `canonical_observations` registry membership,
  witness-field membership, non-observation-type uniformity, and the
  import graph are all structural facts in the parsed model. The
  four-rule selector is a closed structural cascade.
- *Verdict:* hard error in `std/` and `extdeps/`.
- *Escape (structural — no comment anchors):* (4) cross-cutting
  functions pass via rule (4) without an exemption. For rule
  (1)/(2)/(3) cases where the lens fires but the function legitimately
  belongs in its current file, the exemption is itself a substrate
  data row read by the lens:
  ```dag
  data foo_wrong_home_exemption: WrongHomeExemption = {
    function: foo_fn,
    because:  bridge,                  // closed vocabulary
                                       // (bridge / coercion / display)
  }
  ```
  Comment anchors do not satisfy the escape, by construction. The
  vocabulary token set (`bridge`, `coercion`, `display`) is itself a
  closed coproduct declared structurally — extending it is a data
  edit, not a doc edit.

**Concrete match — F5 (`src/v4/std/float.dag:52-62`):**
```dag
// in float.dag, but every argument lives in nat.dag
fn nat_compare(a: Nat, b: Nat) -> Ordering {
  match a {
    Zero => match b { Zero => Equal, Succ { prev: _ } => Less }
    Succ { prev: ap } => match b {
      Zero => Greater
      Succ { prev: bp } => nat_compare(a: ap, b: bp)
    }
  }
}
```

**Clean shape:** move `nat_compare` into `src/v4/std/nat.dag` (or
`std/algebra.dag` as a `TotalOrder<Nat>` witness); `float.dag` imports it.

### L1.9 Vacuous-arm lens — kills *exhaustive-but-empty match* (proposed)

> **Status: proposed.** Derived from finding F1
> (`node_locally_well_formed` discharges every `ComputationNode { behavior: _ }`
> with `=> true`). Distinct from L0.12 (non-exhaustive match): the arm is
> present, but its body does no work. The closed set is *named* but not
> actually *checked*.

- *Signature (structural, no name-suffix vocabulary):* a single `match`
  on a coproduct whose arms are **asymmetric in body shape**:
  - **at least one arm's RHS is a trivial literal** of the function's
    return type (`true`, `false`, `Unit`, the matched input itself,
    `None` / `Empty`); and
  - **at least one sibling arm's RHS does non-trivial structural work**
    (calls another function, recurses, constructs a typed value with
    fields derived from the input).
  The finding is the asymmetry *within a single match* over a closed
  coproduct — not the function's name. A match where *every* arm is
  trivial is a different shape (likely L0.7 dead/constant branch or a
  genuinely-constant function) and is out of scope; a match where every
  arm does real work passes. The discipline-role of the function is
  inferred from the structural fact that *the author already wrote real
  work for some variants*, which is what makes the trivial siblings a
  vacuum rather than an honest constant.
- *Decidable:* yes — arm-body shape (literal-vs-call/recursion/ctor) is
  a structural property of the parsed match. No name inspection.
- *Verdict:* hard error in substrate files.
- *Escape (structural — no comment anchors):* the exemption is itself
  a substrate data row read by the lens:
  ```dag
  data unit_is_unit_vacuous_arm_exemption: VacuousArmExemption = {
    function:   unit_is_unit,
    at_variant: Unit,
    because:    variant-has-no-children,   // closed vocabulary
                                           // (variant-has-no-children /
                                           //  identity-on-Unit /
                                           //  proven-unreachable)
  }
  ```
  Comment anchors do not satisfy the escape, by construction. The
  `because` vocabulary is itself a closed coproduct declared
  structurally — extending it is a data edit.

**Concrete match — F1 (`src/v4/std/node.dag:115-120`):**
```dag
fn node_locally_well_formed(n: Node) -> Bool {
  match n.kind {
    TypeNode { connective: c } =>
      edges_conform(children: n.children, d: connective_edge_discipline(c: c))
    ComputationNode { behavior: _ } => true   // ← arm exists, body vacuous
  }
}
```

**Clean shape:** introduce a sibling `behavior_edge_discipline(Behavior)`
in `std/node.dag` and dispatch both arms uniformly.

### L1.10 Textual-bypass lens family — kills *typed-model bypass via String* (proposed, merged L1.6)

> **Status: proposed.** Derived from findings F6
> (`CiCommand::ShellCommand { command: String }` while
> `extdeps/posix.dag` already models a typed
> `Command { program, argv0, args, env }`) and F8 (string-template
> emitters such as `list_template: "Vec<{0}>"` in
> `dsl/std/languages.dag`). Absorbs the original L1.6 as a
> sub-signature. The two cases are different mechanical detectors but
> share one structural finding: **a string-valued artifact is carrying
> a typed fact that has, or should have, a model carrier.**

> **Exception to §5.0:** L1.10 is a *lens family*, not a single lens.
> Its mechanically enforced units are L1.10.a `TemplateHole` and L1.10.b
> `CanonicalCarrier`; they share a finding family and reporting label,
> but keep separate signatures, decidability arguments, escape valves,
> and test corpora — exactly the per-lens discipline §5.0 requires.

#### L1.10.a `TemplateHole` (registry-free)

- *Signature:* a field or value that is a **template string literal** —
  a string literal carrying positional placeholders (`{0}`, `{1}`, …)
  used as an emitter, where grammar-as-declarative-bidirectional-data
  belongs.
- *Decidable:* yes — a literal string with interpolation placeholders
  is a structural match (the keystone decidability table already
  classifies it "structural — a literal template-string field"). No
  registry required.
- *Verdict:* hard error on the literal-template shape.
- *Escape:* a plain string constant with no placeholders, or genuine
  string *data* that is not an emitter template, passes.

**Concrete match — F8 type-construction templates (`dsl/std/languages.dag`):**
```dag
type TypeMapping {
  string:            String
  int:               String
  list_template:     String   // ← positional placeholders inside
  optional_template: String
  map_template:      String
}

data rust_type_mapping: TypeMapping = {
  string:            "String",
  int:               "i64",
  list_template:     "Vec<{0}>",          // ← {0} is an emission hole
  optional_template: "Option<{0}>",
  map_template:      "HashMap<{0}, {1}>", // ← {0}, {1} are emission holes
}
```
Emission is happening — but it's a fill-in-the-hole string substitution
the lens can't structurally validate. The placeholder vocabulary
(`{0}`, `{1}`) is parallel to the model rather than part of it.

**Clean shape:** grammar-as-declarative-bidirectional-data — the target
type's construction is modeled, and emit is a fold over the model:
```dag
type RustTypeRealization
  = RustGeneric { ctor: RustIdent, args: List<RustTypeRealization> }
  | RustAtom    { ident: RustIdent }

data rust_list_realization: TypeRealization<List> = fn(elem) {
  RustGeneric { ctor: "Vec", args: [elem] }
}
```
Plain string constants without placeholders (e.g. a fixed `"i64"`
atom) pass — they're data, not a templated emitter.

#### L1.10.b `CanonicalCarrier` (substrate-declared registry)

- *Signature (substrate-declared registry, no opt-in tag):* the lens
  reads a **canonical-carrier registry** authored as data in the
  substrate. A typed carrier declares its coverage — the set of field
  names it claims authority over — as a structural witness:
  ```dag
  // in src/v4/extdeps/posix.dag
  data posix_command_canonical: CanonicalCarrier<posix.Command> = {
    supersedes_string_at_field_named: { command, shell_command, invocation }
  }
  ```
  The lens fires on **any** field of type `String` whose field name
  appears in any registered `CanonicalCarrier::supersedes_string_at_field_named`
  set, whenever that carrier's target type is in scope. The trigger is
  not author-controlled — the author cannot bypass by omitting a tag,
  because the registry-declared field-name set drives the gate
  unconditionally. The name set lives in substrate `data`, not in the
  lens body, so adding a new typed carrier is a registry edit, not a
  lens edit.
- *Decidable:* yes — `CanonicalCarrier` registry membership, the
  registry's `supersedes_string_at_field_named` set, the offending
  field's declared name, and the target carrier's in-scope status are
  all structural facts in the parsed model.
- *Verdict:* hard error.
- *Escape (structural, registered exemption):* a `String` field whose
  name *does not* appear in any in-scope registry entry passes
  naturally. For fields whose name happens to collide with a registry
  entry but whose value is legitimately raw (e.g. an opaque ID), the
  carrier author can add a structural exemption row to the same
  registry, e.g.
  `data raw_command_exempt: CanonicalCarrier.Exemption = { at_field: command, in_carrier: opaque_log_record, because: opaque-id }`,
  read as data, not as comment. Author-side opt-out via comment marker
  or omitted tag does not pass.

**Concrete match — F6 (`src/v4/workflow/ci.dag:23-28`):**
```dag
type CiCommand
  = LintCommand
  | TestCommand
  | IgnoredTestCommand { test_name: String }
  | BootstrapStageCompile { produces: Symbol }
  | ShellCommand { command: String }    // ← unannotated; field name = `command`
```
With `data posix_command_canonical: CanonicalCarrier<posix.Command> = { supersedes_string_at_field_named: { command, ... } }`
in `extdeps/posix.dag`, the lens reads the registry, sees that any
`String` field named `command` has a typed canonical home in scope
(`posix.Command`), and fires — independent of whether the author
opted in to any annotation.

**Clean shape:** consume the typed carrier directly.
```dag
import v4.extdeps.posix as posix
type CiCommand = ... | ShellCommand { command: posix.Command }
```

#### L1.10.c Name-discriminant-bypass — kills *string-spelling used as dispatch authority where resolved identity is available*

> **Status: reserved-proposed.** Sub-signature of L1.10 (Textual-bypass)
> family. Catches the case where a function branches on the
> SPELLED-NAME of a construct (`authored_name_at(...) == "StringVariant"`,
> `record_lit_type_name_at(...) == "Foo"`, `expr_var_name_at(...) == "..."`)
> instead of on the resolved constructor/field/variant identity. The
> typed identity exists in the substrate (`03_resolve` produces it);
> the function is bypassing the typed authority via name comparison.
> **Enforcement gate**: not active until `v4.lens.decision_tree_shape`
> lands (its `KeyVocabulary` carries the resolved-vs-spelled
> distinction the lens reads).

- *Signature:* a closed-vocab decision tree (per `DecisionTreeShape`)
  whose source-form is `StringEqChain` (or nested mix containing it),
  AND whose effective keys would resolve to a `KeyVocabulary` of
  constructor / field / variant identities. The fn is using
  name-spelling as the dispatch authority when resolved identity is
  available from `03_resolve`.

- *Decidable:* yes — `decision_tree_shape` records `source_form =
  StringEqChain` for name-equality dispatch AND records the
  `key_set: KeyVocabulary` (which resolves to constructor identities
  regardless of source-form). The mismatch — source uses
  name-equality but the resolved-identity equivalent exists — IS the
  fire condition.

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **Genuinely open vocabulary.** When the name being compared is
    NOT a member of any resolved closed coproduct (e.g., free user-
    supplied identifier where the runtime vocabulary is open), the
    name-comparison is genuine string-dispatch and does not fire.
  - **Cross-language identifier bridge.** A bridge consuming target-
    language identifiers from outside the substrate's resolved
    vocabulary passes via (R5) `GeneratedArtifactBinding`-style
    resolution naming the bridge boundary explicitly.

- *Clearing receipt:* substrate carries the resolved-identity dispatch
  (R1 alias-identity — the name-equality chain rewritten as a typed
  match over the resolved constructor / variant identity).

- *Fix-confidence: templated auto-apply.* The lens emits a `Diff`
  rewriting `name == "X"` to `Constructor::X` pattern-match (where
  Constructor is the resolved coproduct identity per `KeyVocabulary`).
  Reviewer overrides the chosen pattern-name before commit.

- *Decidability boundary:* L1.10.c fires when the spelled name IS
  resolvable to a closed-vocab member. Open-vocabulary cases (user
  input, runtime identifiers, cross-system bridges) are out of scope.

- *Kills (real corpus — seed examples):*
  - **`variant_encoding_is_string_variant`-shape predicates** and
    `resolve_wire_serde_policy_from_encoding_node`-shape resolvers in
    `src/v2/05_emit_rust.dag` (operator-identified). Use
    `authored_name_at(...) == "StringVariant"` style comparison where
    the encoding is a member of the resolved `VariantEncoding` closed
    coproduct. Fires on L1.10.c with the typed-pattern-match clean
    shape.

- *Producer stage (see §10):* L1.10.c consumes
  `v4.lens.decision_tree_shape` — same record L1.13.b consumes, with
  the `source_form` field driving the L1.10.c trigger specifically.

#### L1.10.d Target-syntax-by-concat — kills *target-grammar-token sequence built via `concat`/`format!` that should be a `TargetSurfaceNode`*

> **Status: reserved-proposed.** Sub-signature of L1.10 (Textual-bypass)
> family. Catches the deepest emit-side defect: a function builds a
> target-language grammar-token sequence via `concat()` / `format!()` /
> direct string assembly when a typed `TargetSurfaceNode` and a
> grammar-driven serializer would carry the same shape as substrate.
> Parent L1.10.a catches `{0}`-placeholder string templates; L1.10.d
> catches the same defect when the template is implicit in `concat()`
> sequencing rather than explicit in a template string with holes.
> **Enforcement gate**: not active until (a)
> `v4.lens.target_syntax_string_shape` lands, (b) at least one
> per-target `TargetGrammarTokenSet` substrate carrier exists (PR
> #3476's `rust.dag` LanguageModel is the canonical first instance —
> see substrate-prerequisite note in §10.2), (c) at least one worked
> grammar-driven serializer exists as the migration target (v4's
> `05_emit.dag` is the canonical first instance per PR #3465).

- *Signature:* a string-producing sub-expression whose
  `TargetSyntaxStringShape.classification = TargetGrammarTokenSequence`
  for some target language. The classification is the **closed
  four-case structural predicate** from `target_syntax_string_shape`'s
  §10.2 spec — no numeric token-density threshold. Specifically: the
  target has a `TargetGrammarTokenSet`, at least one emitted literal
  segment IS a grammar token or delimiter, the enclosing sink is
  target source / `TargetSurfaceNode` serialization / emitted-artifact
  text, AND the nonliteral holes are typed model values that would be
  children of a `TargetSurfaceNode`. The function is assembling target
  syntax by hand instead of consuming the grammar substrate.

- *Decidable:* yes — `target_syntax_string_shape` carries the
  classification + per-substring grammar-token evidence + the
  `missing_typed_path: Optional<TargetSurfaceNodeId>` that names the
  typed substrate path that would close the bypass. The lens reads
  classification = `TargetGrammarTokenSequence` AND
  `missing_typed_path = Some(...)` as the fire condition.

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **Unknown classification.** Targets without a
    `TargetGrammarTokenSet` substrate carrier yield classification =
    `Unknown` — the lens does not fire (fail-closed: no false
    positives on substrate-less targets). Resolution: land the
    target's LanguageModel + TargetGrammarTokenSet carrier.
  - **Data string.** Strings carrying data (file paths, log messages,
    user-facing copy) classify as `DataString` and pass naturally.
  - **Already a TargetSurfaceNode consumer.** A fn that calls a
    grammar-driven serializer (e.g., `serialize_target_source`) is
    consuming the typed substrate; passes via R1 alias-identity to
    the substrate carrier.

- *Clearing receipt:* substrate carries the `TargetSurfaceNode` typed
  representation; the fn (if retained) calls the grammar-driven
  serializer over it (R1).

- *Fix-confidence: structural sketch.* Unlike L1.10.c, the (C1-style)
  auto-fix is more disruptive — rewriting `concat("pub ", "enum ", name, " {", ...)`
  to a `TargetSurfaceNode` constructor requires knowing the target's
  grammar productions (which the substrate carries) and the typed-tree
  shape that would emit the same string. The lens emits a structural
  candidate Diff naming the missing `TargetSurfaceNode` path; reviewer
  authors the actual typed construction.

- *Decidability boundary:* L1.10.d fires when classification crosses
  the `TargetGrammarTokenSequence` threshold AND a `missing_typed_path`
  is identifiable. Targets without LanguageModel substrate (Unknown
  classification) are out of scope until the substrate lands.

- *Kills (real corpus — seed examples):*
  - **`emit_rust_block_stmts` / `emit_typed_record_lit` /
    `emit_rest_url_line` / `emit_rest_body_line`** in
    `src/v2/05_emit_rust.dag` (operator-identified). Each builds Rust
    target syntax via `concat()` / `format!()` from a typed Node tree;
    each could consume PR #3476's `rust.dag` LanguageModel grammar
    productions instead. The whole `src/v2/05_emit_rust.dag` file is a
    pile-up of L1.10.d findings (~240 match expressions, 300 fns over
    6876 lines) — the dissolution path is v4's `05_emit.dag`
    consuming `rust.dag` as substrate.

- *Producer stage (see §10):* L1.10.d consumes
  `v4.lens.target_syntax_string_shape` — string-construction-graph
  classification gated on per-target `TargetGrammarTokenSet` substrate.
  Without `TargetGrammarTokenSet`, the producer conservatively returns
  `Unknown` and the lens fails closed.

### L1.11 Plausible-fallback lens — kills *fabricated-sibling fallthrough* (proposed)

> **Status: proposed.** Derived from finding F10
> (`derive_effect_shape`'s `None => CreateEffect` arm for DELETE/PUT/PATCH
> when no path key exists). Sibling rule to P3 (fail-closed): the
> function returns a typed enum and the missing-info case returns a
> *different valid constructor* of that enum — a plausible guess —
> instead of escalating through `Outcome::Rejected`.

- *Signature (no return-type carve-out — RHS constructor structural check):*
  a `match` arm of shape `None => Ctor` / `Empty => Ctor` / `[] => Ctor`
  where `Ctor` is a constructor of the function's return type AND
  `Ctor` is **not** a structurally-registered fail-closed-diagnostic
  variant. Fail-closed-diagnostic variants are declared by the
  substrate:
  ```dag
  data outcome_rejected_variant: FailClosedDiagnostic = {
    type: Outcome,
    ctor: Rejected,
  }
  ```
  The lens fires on **both** the bare-return case and the
  Outcome-wrapped case:
  - `fn(...) -> EffectShape; None => CreateEffect` — return is not
    Outcome, `CreateEffect` is not registered → fires (F10).
  - `fn(...) -> Outcome<EffectShape>; None => Produced { value: ... }`
    — return is Outcome, `Produced` is not registered → fires
    (fabricated success — the prior carve-out's false negative).
  - `fn(...) -> Outcome<T>; None => Rejected { diagnostic: ... }` —
    `Rejected` IS registered as `FailClosedDiagnostic` → passes.
- *Decidable:* yes — return-type shape + arm-RHS constructor membership
  + matched-sub-pattern is a "nothing-here" variant + registry
  membership of the RHS constructor are all structural facts.
- *Verdict:* hard error. Fix depends on which case fired:
  - **Bare-return case** (`fn(...) -> T; None => Ctor`): lift the
    return type to `Outcome<T>` and return
    `Rejected { diagnostic: DerivationUnknown }` on the missing-info
    arm.
  - **Outcome-wrapped case** (`fn(...) -> Outcome<T>; None => Produced { value: ... }`):
    the return type is already `Outcome<_>` — replace the
    `Produced` constructor on the missing-info arm with
    `Rejected { diagnostic: DerivationUnknown }` (the registered
    fail-closed-diagnostic variant).
- *Escape (structural — no operator-confirm via prose):* for genuine
  total-by-design helpers (`or_default(opt, default)` etc.) where the
  `None` branch's RHS is the function's *definitional* result, the
  exemption is a substrate data row:
  ```dag
  data or_default_total: PlausibleFallbackExemption = {
    function:    or_default,
    arm_pattern: None,
    because:     total-by-design,    // closed vocabulary
                                     // (total-by-design /
                                     //  domain-restricted /
                                     //  proven-saturating)
  }
  ```
  Comment markers do not satisfy the escape, by construction. Same
  shape as L1.8 `WrongHomeExemption` and L1.9 `VacuousArmExemption` —
  the exemption is read as substrate data, not as prose.

**Concrete match — F10 (`dsl/std/effects.dag:268-282`):**
```dag
DELETE =>
  match last_path_param(template: path) {
    Some { value: p } => DeleteEffect { key_source: PathParam { param: p } }
    None              => CreateEffect          // ← fabricated sibling
  }
PUT =>
  match last_path_param(template: path) {
    Some { value: p } => UpsertEffect { key_source: PathParam { param: p } }
    None              => CreateEffect          // ← same pattern
  }
```

**Clean shape:**
```dag
fn derive_effect_shape(...) -> Outcome<EffectShape> {
  DELETE =>
    match last_path_param(template: path) {
      Some { value: p } => Produced { value: DeleteEffect { ... } }
      None              => Rejected { diagnostic: DerivationUnknown { ... } }
    }
}
```

#### L1.11.b Plausible-scalar-fallback — kills *missing/None arm returns a scalar literal that gets used as target syntax / coordinate / identity*

> **Status: reserved-proposed.** Sub-signature of L1.11 (Plausible-
> fallback) family. Parent catches `None => Constructor` /
> `None => Produced { value: Ctor }` patterns — a missing-info arm
> fabricating a typed answer via a coproduct constructor. L1.11.b
> catches the same defect mode when the fabricated answer is a
> **scalar literal** (`""`, `"Authorization"`, `"http://localhost"`,
> `0`, `false`) that downstream code uses as target syntax, resource
> coordinate, URL, header, identifier, or other typed authority.
> Algebraically identical to L1.11 — missing fact fabricates a
> plausible answer — but the RHS is a `String` / `Int` / `Bool`
> rather than a constructor. **Enforcement gate**: not active until
> (a) `v4.lens.decision_tree_shape` lands (for the `missing_behavior`
> arm detection that locates None-arm scalar RHSes), AND (b)
> `v4.lens.scalar_authority_use_shape` lands (for the per-scalar
> use-site authority-role classification — TargetSyntaxUse /
> IdentifierUse / UrlUse / HeaderNameUse / FilePathUse /
> ResourceCoordinateUse / DataStringUse / UnknownUse). The
> `scalar_authority_use_registry` substrate carrier must declare at
> least one row per ScalarUseRole variant before the corresponding
> sub-class of L1.11.b can fire. `v4.lens.target_syntax_string_shape`
> is NOT a prerequisite — it classifies what is being BUILT at the
> production site (L1.10.d's axis); L1.11.b consumes the orthogonal
> use-site-role axis from `scalar_authority_use_shape`. The
> `target_syntax_string_shape` producer is only referenced as a
> cross-reference for the `TargetSyntaxUse` role variant (it's where
> a use-site-classifier might cross-check what production-site
> classification the consumer's caller sees).

- *Signature:* a `None =>` / unhandled arm of a closed-vocab decision
  tree (per `DecisionTreeShape`) returns a non-diagnostic scalar
  literal whose downstream use-site classifies as **target-syntax-
  bearing** (target source string, identifier, header name, URL,
  resource path, file-system path). The scalar is filling a typed
  fact-shaped hole.

- *Decidable:* yes — `decision_tree_shape.missing_behavior` records
  the None-arm RHS shape; cross-referenced against
  `scalar_authority_use_shape` for per-scalar use-site role
  classification (TargetSyntaxUse / IdentifierUse / UrlUse /
  HeaderNameUse / FilePathUse / ResourceCoordinateUse / DataStringUse
  / UnknownUse). L1.11.b fires when the scalar's role is anything
  other than `DataStringUse` AND `UnknownUse`. `target_syntax_string_shape`
  classifies what is being BUILT at the production site (target-grammar
  tokens vs data strings); `scalar_authority_use_shape` classifies
  what role the scalar FILLS at the consumer call-site — different
  axes. Without the use-site classifier, L1.11.b would overclaim by
  smuggling scalar-authority-bearing through the target-syntax
  classifier (which only knows about target-grammar tokens, not
  header names / URLs / file paths / resource coordinates).

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **Genuinely-defaulted scalar.** A scalar whose absence is
    semantically equivalent to a specific value (e.g., empty string
    for "no prefix") AND the substrate carries a `DefaultValueWitness`
    row declaring the default. Passes via (R4) refinement-edge-style
    resolution.
  - **Locally-bounded scalar.** A scalar used only within the same
    function's local computation (not propagated as target syntax /
    identity / coordinate) passes — it's a local computation result,
    not a fabricated typed authority.
  - **DiagnosticAccumulation downstream.** When the None-arm returns
    a scalar AND the downstream consumer's typed flow rejects
    accordingly (e.g., the empty string is treated as
    `Rejected{value-absent}`), the scalar is participating in the
    rejection path and passes.

- *Clearing receipt:* substrate adopts one of:
  - **(R1) typed-rejection rewrite** — the `None =>` arm returns
    `Rejected { diagnostic: ... }` (parent L1.11's resolution shape
    extended to scalar RHSes).
  - **(R4) `DefaultValueWitness` row** — substrate declares the
    scalar's default as legitimate via a typed witness.

- *Fix-confidence: templated auto-apply.* The lens emits two
  candidate Diffs:
  - **Pattern A** (typed-rejection): rewrite the scalar to
    `Rejected { diagnostic: <derived> }` matching parent L1.11's auto-
    fix shape.
  - **Pattern B** (DefaultValueWitness): emit the substrate row
    declaring the scalar as a typed default; the None-arm becomes
    `total_lookup(default_witness)` style.

- *Decidability boundary:* L1.11.b fires when the scalar's use-site
  is target-authority-bearing AND a typed alternative exists. Pure
  computation-local scalars (intermediate accumulators, loop indices)
  are out of scope — the bypass surface is target-authority specifically.

- *Kills (real corpus — seed examples):*
  - **`emit_rest_auth_line`** in `src/v2/05_emit_rust.dag`
    (operator-identified). Defaults: `None => ""` (auth token),
    `None => "Authorization"` (transport header name), `None => "x-api-key"`
    (payload auth header). The substrate has the closed `AuthSource`
    vocabulary; the missing arm fabricates plausible HTTP-syntax
    defaults instead of rejecting.
  - **Service base URL fallback chain**: `from_fallback != "" ? from_fallback : "http://localhost"`
    style fallthroughs that produce a syntactic URL when the typed
    `transport_base_url` is absent. Fires on L1.11.b — substrate
    rejection is the clean shape.

- *Producer stage (see §10):* L1.11.b consumes
  `v4.lens.decision_tree_shape` (for the `missing_behavior` arm
  detection) + `v4.lens.scalar_authority_use_shape` (for per-scalar
  use-site role classification). Does NOT consume
  `target_syntax_string_shape` (that producer classifies target-
  grammar tokens at the production site, which is L1.10.d's axis —
  L1.11.b needs the orthogonal use-site-role axis).

### L1.12 Parallel-authority lens — kills *unmarked duplicate concept homes* (proposed)

> **Status: proposed.** Derived from finding F9 (`Bool`, `Char`, `Url`,
> machine words declared in both `dsl/std/` and `src/v4/std/` with no
> marker indicating which is canonical). The D2-resolver gap is
> related but **deliberately not collapsed into this lens** — see the
> "Cross-reference — D2-resolver" note below for why the dangling-import
> shape is a separate finding.

- *Signature (concept-level — two triggers, union):* the lens fires
  on **parallel authority for a concept**, which is detectable in
  two mechanically-distinct ways. The lens has **two triggers**;
  EITHER triggers the resolution check. P2 and Practice 5 demand
  single-authority *at the concept level*, not at the lexical-name
  level — both triggers exist so the lens detects the violation in
  the shapes the substrate can mechanically see.
  - **Trigger A (lexical):** a cross-file lexical-name collision —
    two `type T` declarations in different files sharing the same
    simple name.
  - **Trigger B (concept-graph):** two `type T1` and `type T2`
    declarations in different files that are **co-members of a
    `CanonicalConcept` row**, regardless of whether their lexical
    names match. This catches the "two parallel homes for one
    concept under different names" shape, which Trigger A misses by
    construction.
  Once either trigger fires, the lens checks for one of **five
  mechanically-distinct outcomes** — three passing and two firing —
  and the decision table is closed (no escape outside this
  enumeration):
  Each outcome is defined by which **substrate-data resolution
  shape(s) are present** — they are independent passing conditions,
  NOT compound predicates. `CanonicalConcept` co-membership is one
  resolution shape; an alias-identity edge is another; a
  `HistoricalDeclaration` retirement row is another. Any single
  passing shape's presence is enough.
  1. **Alias-identity edge.** The non-canonical declaration is a
     structural alias (`import` + `type T = <canonical-module>.T`)
     of another declaration. The alias IS the resolution — it
     re-exports rather than re-declares — regardless of whether a
     `CanonicalConcept` row is also present. → **passes.**
  2. **Retirement-record.** A `HistoricalDeclaration` row names one
     of the two declarations as retired with a `dissolves_when`
     trigger:
     ```dag
     data bool_dsl_std_retired: HistoricalDeclaration = {
       type:           dsl.std.types.Bool,
       dissolves_when: <trigger>,
     }
     ```
     The retirement ledger is substrate data read by the lens — its
     presence resolves the lens regardless of whether a
     `CanonicalConcept` row also exists. → **passes.**
  3. **Distinct-concepts disambiguation.** A `ConceptDisambiguation`
     row names the two declarations as legitimately distinct
     concepts in different namespaces:
     ```dag
     data network_vs_normalize_result: ConceptDisambiguation = {
       names:   { network.Result, compiler.normalize.Result },
       because: distinct-domain-concepts,
     }
     ```
     → **passes.** (Note: applies to Trigger A only. Under Trigger B,
     a `ConceptDisambiguation` row that *contradicts* a present
     `CanonicalConcept` co-membership row is a registry-inconsistency
     finding — caught by L0-class duplicate-data-row checks, not
     L1.12.)
  4. **CanonicalConcept co-membership without alias or retirement.**
     A `CanonicalConcept` row claims the two declarations as
     co-members, AND neither outcome (1) alias nor outcome (2)
     retirement is present → **fires** (the duplicate-authority
     case; reachable via either trigger). The CanonicalConcept row
     by itself asserts the concept identity but does not resolve
     the parallel authority — outcome (1) or (2) is required to
     resolve, otherwise the substrate is asserting "these two
     declarations are the same concept" while keeping both as
     authoritative declarations, which is the violation.
  5. **Silence.** Triggered lexically (Trigger A), AND none of
     outcomes (1) / (2) / (3) / (4) apply (no alias, no
     retirement-record, no disambiguation, no CanonicalConcept
     row) → **fires as unresolved-duplicate.** The substrate must
     take a position. (Not reachable via Trigger B — Trigger B's
     premise is that a CanonicalConcept row IS present, so this
     case becomes outcome (4) or one of the passing outcomes
     instead.)
  Note: **deletion / migration** (removing the redeclaration in the
  same change so only one `type T` remains across the corpus) is
  not a sixth resolution — it removes the trigger condition (the
  lexical collision and/or the registry co-membership) entirely, so
  the lens never engages.
  **Decidability boundary (explicit):** the lens catches concept-level
  parallel authority when EITHER (a) the duplicate uses the same
  lexical name (Trigger A) OR (b) the substrate has registered the
  concept identity via `CanonicalConcept` (Trigger B). It does NOT
  catch the case where two homes use *different* lexical names AND
  no `CanonicalConcept` row registers them as the same concept —
  that case is a P2 violation but is mechanically undetectable from
  parsed substrate alone; closing it requires either operator
  judgment or an extension primitive (e.g., a structural similarity
  fold) that the current substrate does not provide. This is the
  honest decidability boundary, and per the §3 methodology a lens's
  signature must catch its finding class with zero false positives —
  Trigger B's CanonicalConcept-driven gate is the structural surface
  the lens can decidably enforce today.
  Unresolved-silence fails closed. Same lexical name in different
  concepts passes via (3); the F9 motivating case (Bool in two
  files with no rows anywhere) fires via (5) until one of (1),
  (2), or (3) lands.
- *Decidable:* yes — lexical-name uniqueness, `CanonicalConcept`
  registry membership, structural-alias edges,
  `HistoricalDeclaration` registry membership, and
  `ConceptDisambiguation` registry membership are all structural
  facts in the parsed model. No comment/prose inspection.
- *Verdict:* hard error.
- *Escape (structural — same rule L1.7 enforces against itself):*
  the passing shapes are outcomes (1), (2), and (3) in the
  decision table above — alias edge, retirement-ledger row, or
  ConceptDisambiguation row — all substrate data read by the
  lens. Comment markers — including a `// Authority: canonical`
  header — do not satisfy the escape, by construction. (Scope
  clarification — the *planned-but-absent home* case is a
  different finding shape: unresolved reference / fail-closed P3,
  caught by L0.8 extended to dangling import paths, deliberately
  not L1.12.)

**Concrete match — F9 (`dsl/std/types.dag:173` and `src/v4/std/logic.dag:14`):**
```dag
// dsl/std/types.dag:163-173
type Bool = True | False

// src/v4/std/logic.dag:13-14
type Bool = True | False

// (no CanonicalConcept row, no ConceptDisambiguation row anywhere)
```

This is the **unresolved-duplicate** case — the trigger fires (lexical
collision across two files) and silence in both registries means the
substrate has not taken a position. Outcome (5): the lens fires.

A `CanonicalConcept` row alone is not enough — it asserts co-membership
but without a structural alias edge from `dsl.std.types.Bool` to
`v4.std.logic.Bool` and no `HistoricalDeclaration` retirement row,
outcome (4) "same-concept-without-alias-or-retirement" still fires:
```dag
data bool_concept: CanonicalConcept = {
  canonical_home: v4.std.logic.Bool,
  members:        { dsl.std.types.Bool },
}
// dsl.std.types still has `type Bool = True | False` (a redeclaration,
// not an alias, not retired) → outcome (4) → fires.
```

To pass, the historical declaration must become an alias-identity edge
(or get retired via `HistoricalDeclaration` ledger, or get deleted in
the same change). The hypothetical `network.Result` vs
`compiler.normalize.Result` case passes via resolution (3) — a
`ConceptDisambiguation` row marking them as legitimately distinct.

**Cross-reference — D2-resolver:** the D2-resolver gap is a mix of
shapes that **do not collapse into L1.12**:
- The *planned-but-absent* `extdeps/languages/resolver.dag` is an
  unresolved-reference / dangling-import finding (L0.8 extended), not a
  duplicate-declaration one. Reporting it as a duplicate-authority
  finding would name the wrong root cause.
- If, separately, `GroundingMap` were declared in two language files
  (e.g. both `rust.dag` and `python.dag` redeclared `type GroundingMap`),
  *that* would be an L1.12 match in its own right.

The fix sequence for D2-resolver is therefore: first resolve the
dangling-import finding by landing the canonical home; only then is
L1.12 the right lens for any residual duplicate declarations.

**Clean shape (structural, not prose):**
```dag
// src/v4/std/logic.dag  — canonical declaration
type Bool = True | False

// dsl/std/types.dag  — alias-identity edge, no redeclaration
import v4.std.logic as canonical_logic
type Bool = canonical_logic.Bool

// OR: a structural retirement record (read as data, not prose)
// some/retirement_ledger.dag
data bool_dsl_std_retired: HistoricalDeclaration = {
  type:            dsl.std.types.Bool,
  dissolves_when:  <substrate-trigger>,
}
```
For D2-resolver: land the planned `extdeps/languages/resolver.dag`
canonically and rewrite `rust.dag`'s `GroundingMap` declaration into an
import/alias of the resolver's authoritative shape.

#### L1.12.b Structural-similarity trigger — closes L1.12's decidability gap (the *nickname* case)

> **Status: reserved-proposed.** Closes the gap L1.12's decidability
> boundary already names: "two homes use *different* lexical names
> AND no `CanonicalConcept` row registers them as the same concept" —
> the *unregistered nickname*. The boundary text itself points at "an
> extension primitive (e.g., a structural similarity fold)" — this
> sub-signature IS that primitive, scoped narrowly so the lens stays
> mechanically decidable. **Enforcement gate** (parallel to L1.13):
> not active until ALL of (a) `v4.lens.structural_similarity` lands
> with the schema below, (b) the L1.12 diagnostic payload carries a
> `ParallelAuthorityTrigger` discriminator distinguishing parent
> triggers from this sub-signature, (c) the R1–R5 clearing carriers
> are mechanically consumable in substrate. Shares the parent's
> `coverage_defect_parallel_authority` acceptance key, but findings
> are distinguishable by trigger; downstream consumers must read the
> trigger to know whether a finding came from the parent or this
> sub-signature.

- *Diagnostic payload (trigger discriminator):* the L1.12 family
  diagnostic carries a `ParallelAuthorityTrigger` coproduct so a
  parent-triggered finding and a sub-signature-triggered finding are
  mechanically distinguishable:
  ```dag
  type ParallelAuthorityTrigger
    = LexicalNameCollision
    | CanonicalConceptCollision
    | StructuralSimilarity { layer: StructuralSimilarityLayer }
  type StructuralSimilarityLayer
    = StructuralIdentity            // (C1) full variant/signature+body bijection
    | CatamorphismEquivalence       // (C2) un-derived fold over registered fold_T
  ```
  Without this discriminator, downstream tooling cannot tell which
  trigger fired under the shared `coverage_defect_parallel_authority`
  key — silent enforcement expansion is the failure mode this guards
  against.

- *Trigger C (structural similarity):* fires when a declaration's
  structural shape matches a **registered canonical-home declaration**
  above a threshold defined by the layers below, AND no
  `CanonicalConcept`, alias-identity edge, or `HistoricalDeclaration`
  row connects them. The canonical-home set is **not** restricted to
  `std/` or `core/`; it is whatever the concept-home index (see
  `v4.lens.concept_home` in §10.2) declares as canonical. `std/` and
  `core/` are the default canonical homes for substrate primitives,
  but a non-`std/` declaration can be canonical when the
  concept-home index says so (e.g., `src/v3/lenses/named_function_count.dag`
  is the canonical home for the `count_named_bind` shape — outside
  `std/` but registered as canonical). Two scopes — both fire the
  same trigger:
  - **Type-scope.** A `type Foo = …` declaration in a non-canonical-
    home file whose **variant set + field shape** matches a registered
    canonical-home `type Bar = …`, modulo variant renames. Catches
    model nicknames where the variant sets are in full structural
    bijection. **Domain-sum hard-fire is intentional:** common
    shape-equivalent sums (`Result`-like, `Option`-like, three-way
    status enums, small enumerated domains) WILL fire L1.12.b until a
    `ConceptDisambiguation` row lands marking them as legitimately
    distinct concepts (resolution shape (R3)). That ergonomic cost
    is the design's chosen posture — making it explicit that two
    independently-declared `Win | Lose | Tie` carriers are the same
    concept until the substrate says otherwise. **Refinement/subset
    cases** (e.g. `RegisterStateSpace` enumerating a refined subset
    of `StateSpace`) are explicitly NOT in scope for this entry —
    (C1) requires a full variant-set bijection, which subset+
    extension shapes fail by construction. See the Kills section
    for `RegisterStateSpace` as the honest known gap (future (C1')
    refinement-subset sub-layer).
  - **Fn-scope.** A `fn helper(…) -> …` matches an existing
    **registered canonical-home** fn under the layer rules below
    (same canonical-home set defined for type-scope above — `std/`
    and `core/` are defaults, but the concept-home index can register
    non-`std/` declarations as canonical, which is what makes the
    `count_named_bind` kill in the corpus a (C1) fire against the
    non-`std/` canonical home `src/v3/lenses/named_function_count.dag`).
    The lens fires under EITHER (C1) — signature-shape match **AND**
    α-renamed body equality (parameter-bijection on the signature,
    plus body normalization that compares for structural identity
    after parameter + bound-name α-renaming) — OR (C2) —
    catamorphism-equivalence over a registered canonical-home type
    with a known `fold_T` (substrate-registered via
    `FoldRegistryEntry`, see (C2) below). (C1)-alone-by-signature is
    a deliberate non-fire: two fns with the same signature and
    different bodies are not nicknames.

- *Decidable:* yes — three mechanically-distinct layers, ordered by
  workhorse-first. The lens fires when **any** layer's structural
  fact crosses its threshold; the 5-outcome resolution table from
  L1.12 then resolves.
  - **(C1) Structural-identity match (workhorse).** For type-scope:
    the parsed model carries the carrier's **variant set + per-variant
    field shape**; two carriers match when there's a bijection between
    their variant sets preserving field arity + field-type coproduct
    identity (modulo coproduct-of-leaves identity per Practice 11).
    For fn-scope: the parsed model carries the fn's **signature shape
    AND α-normalized body shape**; two fn-decls match when (i) there's
    a parameter-bijection on the signature preserving type identity
    AND (ii) the bodies, after α-renaming parameter slots and pattern-
    bound names by canonical position, are structurally identical
    expression trees. Both conjuncts are required for (C1) to fire on
    a fn-pair — signature-only match is explicitly NOT a fire (two
    unrelated fns with the same `fn (T) -> U` signature is the
    expected steady-state, not a nickname). Substrate-readable, no
    heuristic.
  - **(C2) Catamorphism-equivalence (stricter sub-rule for
    fold-shape fns).** A fn that performs structural recursion over
    `T` (one arm per `T`-variant, recursive calls only at
    `T`-variant positions) is, by the homomorphism heuristic,
    equivalent to `fold_T` with its algebra. (C2) fires when **both**
    (i) the fn's `CatamorphismForm` resolves to
    `StructuralFoldOver { type_id: T, algebra: … }` (extracted by
    the producer stage; see §10.2), AND (ii) a substrate
    `FoldRegistryEntry` row exists for `T` whose `carrier: TypeId`
    points at a **registered canonical-home** type (same
    canonical-home discipline as type-scope and (C1) fn-scope —
    `std/` is the default but non-`std/` canonical homes qualify
    when the concept-home index says so):
    ```dag
    type FoldRegistryEntry {
      carrier:             TypeId            // the type being folded
      fold:                FnId              // the canonical fold_T fn
      recursive_positions: Set<VariantField> // must match the producer's extraction
      algebra_carrier:     TypeId            // the algebra record type the fn parameter takes
    }
    ```
    "Known `fold_T`" is **not** a naming convention — it is a
    `FoldRegistryEntry` row in the substrate. Without a registry
    row, (C2) does not fire even if the fn structurally is a fold
    over `T`; landing the row is what makes the substrate claim
    `fold_T` is canonical. This is Practice 10's homomorphism
    heuristic mechanized — same defect class, machine-readable
    trigger gated on substrate authority.
  - **(C3) Token-vocabulary overlap (triage, not a fire).** Names
    of a non-`std/` decl and a `std/` decl share above a vocabulary-
    overlap threshold (Jaccard ≥ 0.5 on the token-set extracted
    from identifier + variant/field names), as a heuristic surface
    for human cheap-glance. **Cheap-glance only — never fires the
    lens by itself; presented as advisory list alongside firings.**
    Same rule as L1.13's "advisory" tier being illegal: enforcement
    requires (C1) or (C2) substantiation.

  **Order is intentional.** (C1) fires on the structurally-identical
  case (the workhorse); (C2) fires on the un-derived-fold case (a
  separate defect class with its own clean resolution); (C3) is
  triage-only. The lens never fires on (C3) alone — operator-direct
  ruling on Practice 11 sub-rules + the no-fourth-tier rule.

- *Verdict:* hard error. Same 5-outcome resolution table as the
  parent L1.12 (alias-identity edge / retirement-record /
  ConceptDisambiguation / CanonicalConcept-without-resolution /
  silence-fails-closed). The substrate adds whichever resolution
  shape is honest for the case.

- *Escape (clean 🟢):*
  - **Refinement-not-parallel.** A `type RegisterStateSpace = StateSpace
    refined { variant ∈ {Reg, Const, Global, Local} } ∪ { ResParam { … } }`
    declaration (or its substrate equivalent) makes the relationship
    explicit; the parallel-coproduct nickname becomes a refinement
    edge. Equivalent to outcome (1) "alias-identity edge" extended
    to refinements.
  - **Practice-11-parameterized.** If the (C1) hit is "the same fn
    shape appears N times across files" (e.g., the `count_named_bind`
    quadruplet — identical body across 4 files), the parameterized
    move per Practice 11 is to define the fn once in a canonical home
    and import it elsewhere; the redeclarations become alias-identity
    edges (outcome (1)). Test-fixture redeclarations get the same
    treatment — copies are not exempted from L1.12 by being in a
    fixture file.
  - **Distinct domains.** A `ConceptDisambiguation` row marking the
    similarly-shaped types as legitimately distinct concepts in
    different namespaces (e.g., `network.Result` vs
    `compiler.normalize.Result`) — outcome (3) directly.
  - **Template → generated artifact (structural resolution required).**
    A generated `.dag` produced by `build.rs` (or any structural
    emission step) from a template authority is a derivative, not an
    independent reinvention — but **only when the generation edge is
    recorded structurally**. The resolution shape is a
    `GeneratedArtifactBinding` registry row in the substrate (named
    explicitly to avoid collision with the existing
    `DependencyKind::GeneratedFrom` variant in
    `src/v4/std/dependency.dag:25`):
    ```dag
    data r1_gates_generated_binding: GeneratedArtifactBinding = {
      generated:  src.v3.compiler.tests.fixtures.r1_gates,
      authority:  src.v3.compiler.tests.fixtures.r1_gates_template,
      emitter:    build_rs_emit_r1_gates_fixture,
    }
    ```
    The lens reads `GeneratedArtifactBinding` rows as the resolution shape —
    same discipline as outcome (1) alias-identity and outcome (2)
    `HistoricalDeclaration` retirement: substrate data, not comment
    prose. **Header comment markers and `build.rs` files alone do
    NOT satisfy the escape** — that would re-introduce a prose-shaped
    authority path and conflict with the parent L1.12's "No
    comment/prose inspection" rule and the no-prose discipline. The
    `GeneratedArtifactBinding` row must land. Worked example: until a
    `r1_gates_generated_binding: GeneratedArtifactBinding` row is committed, the
    `r1_gates.template.dag`/`r1_gates.dag` pair will fire L1.12.b
    today (as it should — the substrate has not taken a structural
    position on the relationship); landing the row resolves it. The
    pair's existing prose header (`// **Authority:** Hand-edit ...
    // Companion is generated ...`) is informational only and does
    not satisfy the escape.
(The carrier name `GeneratedArtifactBinding` is chosen here — not
    a placeholder — to make the design directly implementable; the
    short `GeneratedFrom` was earlier suggested but conflicts with
    the existing `DependencyKind::GeneratedFrom` variant in
    `src/v4/std/dependency.dag:25`.)

- *Decidability boundary (explicit):* (C1) and (C2) are mechanical and
  decidable over parsed substrate; (C3) is triage and never fires
  alone. The lens does **not** catch:
  - Two homes that share neither structural shape (C1 miss) nor
    fold-shape equivalence (C2 miss) but ARE conceptually the same.
    That residual case requires operator judgment and a
    `CanonicalConcept` row landing post-hoc, at which point Trigger
    B of the parent L1.12 takes over.
  - Behavioral equivalence in the general sense (undecidable). The
    lens does not attempt it; the homomorphism heuristic (C2) is the
    only behavioral-shape claim made, and it is structural, not
    semantic.

  This is the honest fold-bound: (C1) catches structurally-identical
  declarations regardless of name; (C2) catches un-derived
  catamorphisms regardless of how the body is spelled; everything
  else routes through operator judgment via the parent's resolution
  table.

- *Clearing receipt (single authoritative resolution table):* the
  parent L1.12 dispositions are inherited unchanged. The L1.12
  family **clearing table** is R1–R5, separately enumerated below.
  Parent outcomes (4) `CanonicalConcept`-without-resolution and (5)
  silence are **firing states**, not clearing shapes — when the
  lens engages and lands on those outcomes it is producing a
  diagnostic, not resolving one. The vocabulary distinction is
  load-bearing: "outcome" describes what the lens-engagement
  decision-table evaluates to (5 cases, 3 passing + 2 firing);
  "resolution shape" describes the substrate carriers a substrate
  author lands to clear a firing (5 shapes, all passing). The lens
  re-fires on the same head until one of the five resolution shapes
  lands or the declaration is deleted (deletion removes the trigger
  condition entirely, so the lens never engages — not a sixth
  resolution shape, just trigger absence).
  | # | Resolution shape | Source | Substrate carrier |
  |---|---|---|---|
  | (R1) | Alias-identity edge | parent L1.12, outcome (1) | `import` + `type T = canonical.T` redeclaration rewrite |
  | (R2) | Retirement record | parent L1.12, outcome (2) | `HistoricalDeclaration` row |
  | (R3) | ConceptDisambiguation row | parent L1.12, outcome (3) | `ConceptDisambiguation` row naming declarations as distinct concepts |
  | (R4) | Refinement edge | L1.12.b Escape (Refinement-not-parallel) | substrate `Refinement` carrier declaring the relationship (`type Sub = Super refined { … }` or equivalent) |
  | (R5) | Generated-artifact binding | L1.12.b Escape (Template → generated) | `GeneratedArtifactBinding` registry row |

  Parent outcomes (4) `CanonicalConcept`-without-resolution and (5)
  silence are FIRES, not resolutions — they're the cases the lens
  acts on, and resolution requires the substrate to add one of
  (R1)–(R5). All five resolution shapes are substrate data — no
  comment/prose path satisfies any of them (the no-prose discipline
  applies uniformly across the table).

- *Fix-confidence: templated auto-apply* for the firing-today cases
  only — for (C1) type-scope hits (full variant-set bijection), the
  (R1) alias-identity rewrite is mechanical (the lens emits a
  `Diff` rewriting the redeclaration into an `import` + `type T =
  canonical.T`); for (C1) fn-scope hits (signature + body match),
  the rewrite is the alias `fn helper(…) -> … = canonical.helper(…)`;
  for (C2) hits, the rewrite is the `fold_T` call with the algebra
  extracted from the original arms (the L1.5 catamorphism-derivation
  pattern, applied here as the fix shape). Reviewer overrides the
  canonical-home or algebra-binding names before commit. The
  refinement-subset case (RegisterStateSpace-shape) has **no
  auto-fix today** — it's part of the (C1') future sub-layer;
  resolution requires operator judgment landing one of (R3) (R4)
  via human-designed substrate carrier shape.

- *Kills (real corpus):*
  - **`count_named_bind` corpus.** Identical
    `fn count_named_bind(behavior: Behavior) -> Int = match behavior
    { Value(v) => 0; Transform(t) => 0; Branch(b) => 0; ... }`
    appears in four places, but only two are independent declarations:
    - `src/v3/lenses/named_function_count.dag:15` — canonical lens
      (the program-text authority).
    - `src/v3/compiler/tests/t_demo/t_demo_fixtures.dag:28` —
      hand-authored test fixture: **fires on (C1)** as an
      independent declaration with identical shape and body.
    - `src/v3/compiler/tests/fixtures/r1_gates.template.dag:139` and
      `src/v3/compiler/tests/fixtures/r1_gates.dag:139` — a
      template→generated pair (build.rs splices the lens source into
      the template). The pair **fires (C1) today** (identical body)
      AND will continue to fire until an (R5)
      `GeneratedArtifactBinding` registry row lands — (R5) IS the
      resolution path for this pair, not an alternative non-fire.
      Cited here to show the (R5) resolution shape working in
      practice (today: lens fires; after (R5) lands: lens resolves).
    Practice-11-parameterized clean shape: one canonical home
    (`lenses/named_function_count.dag`), one import-alias from
    `t_demo_fixtures.dag`. The template→generated pair stays as-is.
  - **`RegisterStateSpace` parallel to `StateSpace` — honest known
    gap (not a current fire; future (C1') candidate).** In
    `src/v4/extdeps/languages/ptx.dag:34` and `:102`. `StateSpace`
    enumerates 8 variants (`Reg | SReg | Const | Global | Local |
    ...`); `RegisterStateSpace` enumerates 6 with renamed
    constructors (`ResReg | ResConst | ResGlobal | ResLocal |
    ResParam{...}` + one more) — a refined subset with one
    extension. **(C1) does NOT fire** — it requires a full variant-
    set bijection (same cardinality, preserving field shape), which
    this pair fails by construction. (C2) doesn't apply either
    (these are type declarations, not fns). The lens as specified
    therefore does NOT catch refinement-subset parallel coproducts;
    that's an honest known gap. Cited here as the target shape for
    a future **(C1') refinement-subset sub-layer** — out of scope
    for this entry. Clean shape when (C1') lands or when operator
    judgment intervenes today: refinement-not-parallel per the
    Escape rule above (substrate carrier rewritten as a refinement
    edge or as a `CanonicalConcept` row binding the two carriers).
  - **`lookup_chain` (resolver) vs `fold_right` over scope chain.**
    `src/v4/compiler/03_resolve.dag:205` defines
    `fn lookup_chain(s: Scope, name: Symbol) -> Symbol?` as a
    hand-rolled structural recursion over `Scope = ScopeFrame {
    locals, outer } | ScopeRoot { module }`. Fires on (C2) — the fn
    is exactly `fold_T` with `T = Scope`, base case at `ScopeRoot`,
    inductive step at `ScopeFrame`, returning the first `map_get`
    hit. Should call `fold_right` (or the `Scope` carrier's algebra)
    instead. Note: this candidate is **(C2)-fires-only**, NOT (C1) —
    the signature is not identical to `fold_right`'s; it's the
    homomorphism shape that catches it.

- *Producer stage (see §10):* a new derived stage
  `v4.lens.structural_similarity` produces the shared index — the
  per-decl structural-shape facts (C1's variant-set + field-shape
  for types, C1's signature shape for fns, C2's catamorphism-form
  classifier, C3's token-set) that the lens consumes facts-flow-
  forward without re-walking declarations.

### L1.13 Skeleton-collapse lens — kills *parametric-arm duplication via constructor-name template* (proposed)

> **Status: proposed.** Derived from finding F14 (`feature_disposition` at
> `src/v4/extdeps/languages/llvm_ir.dag:570-585` — 12-arm match over
> `FidelityFeature` with 3 distinct RHS skeletons in a 1:5:6 distribution)
> and finding F15 (PR #3452 `complexity_bound_from_class` — 9 arms with
> 3 skeletons in a 1:1:7 distribution). Sibling-of L1.1 — where L1.1
> catches Bool-returning matches whose arms are literal `true`/`false`,
> L1.13 catches non-Bool matches whose arms are *templated by
> constructor name* (the RHSes share a skeleton, varying only by which
> constructor they apply). Practice 11's runtime-symptom catcher:
> Practice 11 stops parametric duplication at design time; L1.13 catches
> the symptom when Practice 11 was missed at design time.

> **(find, transform) discipline — auto-fix per dissolution.** Per §6
> convolution view: every L1.x lens is structurally a `(find, transform)`
> pair. L1.13's `find` half is the distribution-shape classifier; its
> `transform` half is the per-shape dissolution. Where the transform
> is **mechanically unambiguous from the find** (no human-design hooks
> needed), the lens emits a typed `Diff` (PR #3364 vocabulary —
> `Diff = List<Edit { at: Path, replacement: Node }>`) that goes into
> the candidate-state gate: `candidate_dag = apply_diff(dag, Diff)`,
> run lenses on candidate, commit if green. This is the auto-fix flow
> the operator framed: "every diagnostic can generate the correct code
> solution (and eventually apply it) — assuming we can safely infer
> user intent." Each distribution shape below carries an explicit
> **Fix-confidence** field stating whether the auto-fix is *direct*
> (no naming decisions, fully specified Diff), *templated* (Diff with
> name-holes the reviewer can override before commit), or — in future
> sub-signatures — *structural sketch* (lens identifies the kind of
> transform; concrete Diff requires human design). The fix-confidence
> axis is what makes L1.13 actionable rather than just diagnostic.

- *Signature:* a `match` expression over a closed coproduct with `N` arms
  whose RHSes collapse to `K` distinct skeletons under the substitution
  rule below, with a distribution shape meeting one of the four
  classifier definitions (PureTemplate / Outlier / MultiOutlier /
  Categorical). The diagnostic is the **(K, histogram)** pair where
  the histogram is the sorted list of group sizes; threshold
  definitions for each shape are exact and listed in the Distribution
  shapes section. Only `N ≥ 4` matches are considered for findings
  beyond PureTemplate / Outlier (smaller matches lack enough arms for
  Categorical / MultiOutlier reads to be load-bearing).
- *Decidable:* yes. The skeleton-extraction algorithm is:
  - **Identity source:** the lens consumes post-resolve / `InferredTree`
    canonical identities (qualified, alias-resolved). Source spelling
    variation, import renames, and module-qualification differences do
    NOT affect findings — two arms whose RHSes resolve to the same
    canonical identity have the same skeleton at that position.
  - **Substitution rule:** replace **every occurrence of the matched-arm
    constructor identity** (when used as a value or constructor in the
    RHS) with a per-arm hole; α-rename pattern-bound variables (their
    names don't affect skeleton equality); do NOT collapse unrelated
    free names, literals, or call arguments — those remain
    distinguishing.
  - **Skeleton equality:** two skeletons are equal iff they are
    structurally identical after the above substitution + α-renaming.
  - **Histogram:** sorted list of group sizes (largest first) where
    each group is a maximal set of arms whose RHSes share a skeleton.
    Distribution-shape classification below operates on the histogram.
- *Verdict:* hard error (🔴 dissolve-now). Per the universal three-disposition
  rule (§5.0 / Practices 4 + 10 — 🔴/🟡/🟢, "there is no fourth"), L1.13
  produces exactly one of those three. No "advisory" tier; legitimate
  exceptions pass via Escape (clean 🟢) below, not via a softer verdict.
- *Distribution shapes (closed enumeration with exact thresholds):*
  - **PureTemplate** — `K = 1`, `N > 1`. Histogram = `[N]`. All arms
    identical-modulo-constructor; dispatch doing zero work.
    **Dissolution:** delete the match; the operation doesn't depend on
    the variant. (Rare — usually a real bug.) **Clearing receipt:** the
    match is deleted OR replaced with the single shared RHS at the call
    site. **Fix-confidence: direct auto-apply.** Lens emits a typed `Diff`
    (PR #3364 vocabulary) replacing the `match` expression with its
    single arm's RHS. No naming decisions, no human-design hooks. Diff
    enters the candidate-state gate, runs lenses on candidate, commits
    if green.
  - **Outlier** — `K = 2`, histogram = `[N-1, 1]` (one base case + one
    uniform group). `N ≥ 3`. The match is performing a binary
    discriminator dressed as N-way dispatch. **Dissolution:** consume
    the discriminator *structurally* via match patterns + guards on the
    existing coproduct, OR substructure the coproduct so the one-vs-many
    distinction becomes a top-level variant. The function body becomes
    a 2-arm match of the form `match x { <Special_pattern> => special;
    _ => default }`, with the discriminator expressed as a sub-pattern
    or guard on the `Special_pattern`. **DO NOT extract a named `Bool`
    helper** — that is L1.1's predicate-dissolution anti-pattern.
    **Clearing receipt:** the function body is a 2-arm match (or
    pattern-with-guard) where the special/default split is *structural*
    in the pattern or sub-coproduct definition, NOT a `Bool` helper
    over the original coproduct. Recent precedent: PR #3359
    `connective_spec_fact` (6 arms → 2 skeletons, 5:1) — resolved via
    inline match-pattern with guard (`Atom { identity: id } if
    is_kernel_ambient(id) => ...; _ => default`), NOT via a named
    `Bool` helper. **Fix-confidence: split by RHS form** — skeleton
    equivalence is a comparison-time notion, not executable syntax, so
    the auto-fix has to materialize an actual replacement that compiles:
    - **(a) Constructor identity does NOT appear in the uniform RHS**
      (e.g. PR #3359 `connective_spec_fact`'s 5 uniform arms all
      executing `classify_density(facts)` — the matched constructor
      doesn't show up in the RHS at all). **Direct auto-apply:** the
      catch-all is literally `_ => <uniform_RHS>`; no constructor
      reconstruction needed.
    - **(b) Constructor identity DOES appear in the uniform RHS** (e.g.
      RHSes like `Disposition { feature: <matched_constructor> }` —
      this is more naturally a MultiOutlier shape, but Outlier
      examples exist). **Templated auto-apply:** bind the matched
      value in the catch-all (e.g. `c => <RHS_with_c_substituted_for_the_constructor>`)
      so the executable replacement is a real value-binding, NOT a
      skeleton hole. The lens emits the Diff with the binding name
      templated (default `c`/`x`/etc.; reviewer may override).
    - The naive "substitute the constructor with a wildcard `_`" form
      is a skeleton-equality device for COMPARING arms, not for
      generating executable replacement code; cases that require it
      should downgrade from direct-auto-apply to templated-auto-apply,
      not emit syntactically invalid `_ => Foo { field: _ }` RHSes.
  - **MultiOutlier** — `K ≥ 2`, histogram contains both singleton(s)
    AND non-singleton group(s). `N ≥ 4`. Multiple base-cases + one or
    more uniform groups. Covers F14 (1:5:6 — one singleton +
    two uniform groups) and F15 (1:1:7 — two singletons + one uniform
    group). **Dissolution:** substructure the coproduct so each
    singleton becomes its own top-level variant AND each uniform group
    becomes a single wrapped variant (`SizedKind { kind: SizedSubKind }`
    or similar). The function collapses to a number of arms equal to
    the number of distinct skeletons. **Clearing receipt:** the input
    coproduct's type definition carries the categorization
    structurally — singleton variants exist as top-level variants;
    uniform-group variants share a wrapping variant carrying the
    sub-discriminator. NOT acceptable: a hand-rolled `category_of(x):
    Category` function with N arms (just moves the L1.13 violation to
    a new name). Recent precedent: PR #3452 `complexity_bound_from_class`
    (9 arms → 3 skeletons, 1:1:7); F14 `feature_disposition`
    (`llvm_ir.dag:570-585`, 12 arms → 3 skeletons, 1:5:6).
    **Fix-confidence: templated auto-apply (name-templated, structure-direct).**
    Structural transform is unambiguous from the histogram (singleton →
    own top-level variant; uniform group → wrapped variant carrying a
    sub-coproduct). Lens emits a typed `Diff` with templated holes for
    NEW type names (wrapping variant name, sub-coproduct name, sub-variant
    names). Default templates derive from existing constructor names —
    e.g. F14's `DeclaredNormalized` uniform group (5 variants) → wrapping
    variant `NormalizedFeature` with sub-coproduct `NormalizedKind`.
    Singleton groups stay fieldless (no `{ kind: ... }` wrapper) per
    the dissolution rule below — only non-singleton groups get wrapped.
    Reviewer may override names before the candidate-state commit; same
    flow as any typo-fix Diff. Auto-apply when reviewer accepts default
    templates; one round of reviewer-edit when names need to differ.
  - **Categorical** — `K ≥ 2`, histogram contains ONLY non-singleton
    groups (every group size ≥ 2). `K ≤ ⌊N/2⌋`, `N ≥ 4`. Pure category
    projection with no singleton base case. Rarer than MultiOutlier in
    practice. **Dissolution:** push the categorization into the
    coproduct definition as N nested variants (each top-level variant
    wraps one of K sub-coproducts). **Clearing receipt:** identical
    discipline to MultiOutlier — the categorization lives in the input
    type's structure, not in a parallel hand-rolled projection function.
    **Fix-confidence: templated auto-apply (same as MultiOutlier).**
    Each uniform group → one wrapped variant carrying a sub-coproduct;
    templated names derive from group composition. Reviewer reviews the
    candidate `Diff`'s names before commit.
  - **Mixed** — `K` close to `N` (i.e., does NOT meet any of the four
    classifier thresholds above). Each arm does distinguishable work.
    **No finding** — legitimate per-variant dispatch.
- *Escape (clean 🟢):* `Mixed` distribution passes — distinct skeletons
  per arm IS legitimate per-variant dispatch. Two other clean-🟢 cases
  follow directly from the decidability rule:
  - **Per-arm distinct literal data.** A match whose arms each construct
    distinct literal data via per-arm references (e.g., test fixtures or
    compile-time data tables where arm `Variant_X` references
    `data variant_x_constant: T = ...`) is Mixed under skeleton
    extraction — each arm's literal IS a distinct skeleton because the
    reference is a free name, not bound by the arm constructor. Such
    matches are legitimate-per-variant enumeration and pass naturally;
    no separate verdict tier needed.
  - **Distinct call arguments per arm.** Matches whose arms call the
    same function with different concrete arguments (literals or distinct
    references) are Mixed for the same reason — the skeleton
    parameter-substitutes the leading constructor only, not call
    arguments (see decidability boundary below).
- *Kills:* `feature_disposition` (`llvm_ir.dag:570-585` — MultiOutlier
  1:5:6, arm-constructor name echoed in RHS) and `complexity_bound_from_class`
  (PR #3452 — MultiOutlier 1:1:7, inner `match size_var` block shared
  across 7 arms with only the outer `Bound` constructor differing).
  `manual_test_claim_for_manual_anchor` (`testgen.dag:253-270`) is a
  related but *distinct* pattern (see "Borderline case" below) — it
  passes the strict L1.13 decidability rule (Mixed) because the
  per-arm `claim_<name>` references are distinct literals; recognizing
  it as a finding requires the **L1.13.c Table decision-tree**
  sub-signature (formerly described as a future L1.13.b sub-signature
  OR a separate "match-as-typed-table" lens — both pointers unified
  under L1.13.c per the 2026-05-21 producer-shape factoring). Listed
  as a borderline case, NOT a current L1.13 kill.

**Decidability boundary (explicit):** RHS skeleton extraction collapses
the leading constructor name to a hole AND α-renames bound field names,
but does **NOT** collapse calls with distinct arguments. So:

```dag
A => f(x)
B => f(y)
```

is `K=1` (same skeleton: `f(<bound>)`); but

```dag
A => f(x, "literal-1")
B => f(x, "literal-2")
```

is `K=2` (literals are part of the skeleton). This matches the
real-world distinction — arms doing different literal work are doing
distinguishable work; arms doing the same call shape over different
bound names are duplicating.

**Connection to Practice 11.** L1.13 catches at implementation time
exactly what Practice 11 catches at design time: parametric duplication.
Practice 11 says "two declarations differing only by a typed parameter
are one parameterized declaration with the difference as a parameter."
L1.13 says "N match arms differing only by their constructor identity
are one parameterized operation with the constructor identity (or its
category) as a parameter." Same nerve, different layer.

**Concrete match — F14 (`src/v4/extdeps/languages/llvm_ir.dag:570-585`):**
```dag
fn feature_disposition(f: FidelityFeature) -> FidelityDisposition {
  match f {
    StructuralCore                  => Modeled
    SsaValueSpelling                => DeclaredNormalized { feature: SsaValueSpelling }
    BlockTextualOrder               => DeclaredNormalized { feature: BlockTextualOrder }
    LexicalTrivia                   => DeclaredNormalized { feature: LexicalTrivia }
    TargetConfigToken               => DeclaredNormalized { feature: TargetConfigToken }
    AdvisoryMetadata                => DeclaredNormalized { feature: AdvisoryMetadata }
    UnmodeledInlineAsm              => FailClosed { feature: UnmodeledInlineAsm }
    UnmodeledSemanticAttribute      => FailClosed { feature: UnmodeledSemanticAttribute }
    UnmodeledType                   => FailClosed { feature: UnmodeledType }
    UnmodeledInstruction            => FailClosed { feature: UnmodeledInstruction }
    UnmodeledConstExpr              => FailClosed { feature: UnmodeledConstExpr }
    MalformedStructure              => FailClosed { feature: MalformedStructure }
  }
}
```

12 arms, 3 distinct skeletons: `Modeled` (1), `DeclaredNormalized { feature: <constructor> }` (5),
`FailClosed { feature: <constructor> }` (6). **MultiOutlier, 1:5:6** (one singleton + two uniform groups; per the classifier definition above, Categorical requires non-singleton-only groups — the `Modeled` arm is a singleton, so this is MultiOutlier).

**Clean shape:** push the categorization into `FidelityFeature`.
**Singleton groups become fieldless top-level variants** (no `{ kind: _ }`
wrapping — a sub-coproduct of one element is empty overhead); **only
non-singleton groups get `{ kind: ... }` wrappers** carrying their
sub-coproduct:
```dag
type FidelityFeature
  = ModeledFeature                                 // singleton (StructuralCore) — fieldless
  | NormalizedFeature    { kind: NormalizedKind }  // 5 normalized variants
  | UnmodeledFeature     { kind: UnmodeledKind }   // 6 fail-closed variants

type NormalizedKind
  = SsaValueSpelling | BlockTextualOrder | LexicalTrivia
  | TargetConfigToken | AdvisoryMetadata

type UnmodeledKind
  = UnmodeledInlineAsm | UnmodeledSemanticAttribute | UnmodeledType
  | UnmodeledInstruction | UnmodeledConstExpr | MalformedStructure

fn feature_disposition(f: FidelityFeature) -> FidelityDisposition {
  match f {
    ModeledFeature                  => Modeled
    NormalizedFeature { kind: _ }   => DeclaredNormalized { feature: f }
    UnmodeledFeature  { kind: _ }   => FailClosed         { feature: f }
  }
}
```
3 arms instead of 12. The N-to-3 projection lives in `FidelityFeature`'s
type definition (its correct home — the category IS a property of the
feature), not as a hand-rolled function. **General rule for the
MultiOutlier / Categorical dissolution** — applies to every shape with
mixed singletons + non-singleton groups: singleton-group variants are
fieldless; only non-singleton groups carry `{ kind: <SubKind> }`
wrappers. Wrapping a singleton is pure structural overhead with no
discrimination work to do.

**Concrete match — F15 (PR #3452 `complexity_bound_from_class`):**
9 arms over `AsymptoticClass`, 3 distinct skeletons: `Constant` (1),
`unknown_complexity()` (1), `match size_var { Holds {v} => Bound { size_var: v, ...}; Violates _ => unknown_complexity() }` (7).
**MultiOutlier, 1:1:7** (two singletons + one uniform group; per the classifier definition above, Categorical requires non-singleton-only groups — both `Constant` and `unknown_complexity()` arms are singletons, so this is MultiOutlier).

**Clean shape:** collapse `ComplexityBound` from 9 parallel variants to
3 structural variants, wrapping `AsymptoticClass` for the 7 size-dependent
classes:
```dag
type ComplexityBound
  = UnknownComplexity { diagnostic: Diagnostic }
  | ConstantComplexity
  | SizedComplexity   { class: AsymptoticClass, size_var: SizeVariable }
```
Function collapses from 9 arms to 3 outer arms (`ClassConstant`,
`ClassUnknown`, `_`) — the `_` arm contains an inner 2-arm match over
`size_var`, so the total arm count is 4 if inner+outer arms are
flattened. The structurally meaningful reduction is 9 → 3 at the outer
dispatch. Full worked example in the PR review at #3452 comment.

**Borderline case — F16 (`src/v4/lens/testgen.dag:253-270`):**
13 arms over `T19ManualAnchorKey`. Strict skeleton extraction (per the
decidability boundary above) treats each arm's RHS as a distinct skeleton
because the per-arm `claim_<name>` references are distinct free literals
NOT bound by the arm constructor — under the strict rule, K = N → Mixed
→ **not an L1.13 finding as the lens is currently signed**.

Recognized as a borderline because the human pattern recognition does
catch it: the references follow a `claim_<arm_constructor_lower_case>`
naming convention that PARAMETERIZES BY the arm constructor. Catching
this structurally is the job of **L1.13.c Table decision-tree** (see
below): when the per-arm RHSes are references to N distinct typed
`data` declarations with one-to-one correspondence to the matched
variants, the match IS a typed table indexed by the closed variant
set; the substrate carrier is `TotalMap<K, V>` (or `TotalPolicy` for
payload-bearing variants per the operator-direct refinement
2026-05-21).

(Historical note for traceability: the borderline case was originally
described as resolving to a "future L1.13.b sub-signature" OR a
"separate match-as-typed-table lens." Both pointers have been unified
under **L1.13.c**. L1.13.b now binds a different mechanism — the
decision-tree-collapse generalization beyond match expressions. See
the L1.13.b and L1.13.c sub-sections after the L1.13 base.)

The clean shape for the F16 pattern (sketched below) is what either
of those future lenses would target. F16 is documented here as a
related-pattern example so the boundary stays explicit, NOT as a
current L1.13 kill.


**Clean shape:** the 12 identical-skeleton arms are a typed registry
mapping anchor → claim. Refactor to a **totality-checked** table consumed
via direct lookup — `Option` + reader-convention "unreachable" comments
would be a fail-open smell here (Practice 2 — illegal states
unrepresentable). The closed enum already guarantees the lookup hits;
the substrate's job is to make that guarantee structural, not
commentary-based:

```dag
// Substructure the enum: split the outlier (Absent) from the lookup-
// carrying variants. T19ManualNonAbsent's k field is the closed sum of
// the 12 claim-bearing constructors.
type T19ManualAnchorKey
  = T19ManualAnchorAbsent
  | T19ManualNonAbsent { k: T19ManualNonAbsentKind }

type T19ManualNonAbsentKind
  = T19ManualTcConjEmpty
  | T19ManualTcDisjEmpty
  | ... // 12 variants, exactly the claim-keys

// TotalMap<K, V> for closed-coproduct K is a typed primitive whose
// construction is well-formed iff every variant of K appears as a key.
data manual_anchor_claims: TotalMap<T19ManualNonAbsentKind, ManualTestClaim> = {
  T19ManualTcConjEmpty            -> claim_conj_empty_compiles,
  T19ManualTcDisjEmpty            -> claim_disj_empty_compiles,
  ...
}

fn manual_test_claim_for_manual_anchor(key: T19ManualAnchorKey) -> Outcome<TestClaim> {
  match key {
    T19ManualAnchorAbsent           => Rejected { diagnostic: ... }
    T19ManualNonAbsent { k: kind }  => Produced { value: total_lookup(manual_anchor_claims, kind) }
  }
}
```

The 13 arms reduce to a 2-arm match over the substructured enum + one
totality-checked data table. `total_lookup` returns `ManualTestClaim`
directly (not `Option<ManualTestClaim>`) because `TotalMap`'s
well-formedness check makes the absence case unrepresentable. No
"unreachable" arm guarded by comment.

**Substrate dependency (scoped to L1.13.c Table decision-tree, NOT base L1.13).**
`TotalMap<K, V>` for finite payload-free closed-coproduct K (and
`TotalPolicy<K, Context, RowTemplate>` for payload-bearing K) are typed
primitives whose well-formedness check is "every K-variant appears as
a key." These primitives are **load-bearing for L1.13.c** (Table
decision-tree — the lens that catches the F16 match-as-typed-table
pattern, bound under the Skeleton-collapse family per the producer-
shape factoring + payload-aware refinement) — they are NOT a dependency
of base L1.13. Base L1.13 (PureTemplate / Outlier / MultiOutlier /
Categorical on F14 + F15) has enough substrate to run today via
`fold_node` + skeleton extraction; it does NOT need `TotalMap` /
`TotalPolicy` to fire or clear. Scoping the table primitives to
L1.13.c prevents table-cleanup substrate work from blocking enforcement
of the simpler base lens.

#### L1.13.b Decision-tree collapse — kills *closed-vocab dispatch collapse regardless of source spelling (match / if-else / string-equality)*

> **Status: reserved-proposed.** Sub-signature of L1.13 (Skeleton-
> collapse) that **generalizes the parent beyond `match` expressions**.
> Base L1.13's algorithm is `match`-scoped: it walks match-arm RHSes,
> extracts skeletons, classifies the distribution. L1.13.b applies the
> SAME algorithm to **any closed-vocab decision tree** — `match`,
> `if/else if` chains, or `name == "X" ? ... : ...` style — as long as
> the keys resolve to a closed coproduct's constructor identity set
> (per `KeyVocabulary`). The same K/N skeleton-collapse classification
> fires regardless of source spelling.
> **Enforcement gate** (parallel to L1.12.b / L1.4.b / L1.13.c): not
> active until `v4.lens.decision_tree_shape` lands, plus the L1.13
> diagnostic payload carries a `SkeletonCollapseScopeTrigger`
> discriminator distinguishing `MatchExpr` (parent) from
> `DecisionTreeGeneralized` (this sub-signature).

- *Signature:* a closed-vocab decision tree (per `DecisionTreeShape`)
  whose per-branch normalized bodies (`BodyShape`) collapse to K
  distinct skeletons over N branches under L1.13's skeleton-extraction
  algorithm, with K thresholds matching parent L1.13's distribution
  classes (PureTemplate / Outlier / MultiOutlier / Categorical /
  Mixed) — REGARDLESS of source-form (`MatchExpr` /
  `IfElseChain` / `StringEqChain` / `NestedMix`).

- *Decidable:* yes — `decision_tree_shape` exposes the normalized
  decision-tree IR per syntactic locus; parent L1.13's skeleton-
  extraction + classification reuse without re-walking source. The
  `source_form` field is a fact-of-the-finding (helps reviewers
  navigate to the right span) but does NOT vary the lens semantics.

- *Verdict:* hard error.

- *Escape (clean 🟢):* same Escape table as parent L1.13 — Mixed
  distribution passes, distinct call arguments per branch pass, etc.
  All inherited; no new Escapes specific to L1.13.b. (The
  `source_form = StringEqChain` case has a different concern —
  name-as-discriminant bypass — but that's L1.10.c's territory, not
  an L1.13.b escape.)

- *Clearing receipt:* same as parent L1.13 — substrate carries the
  parameterized form (one-arm-with-catch-all for PureTemplate,
  Outlier-rewrite for Outlier, etc.). The (R1) alias-identity
  rewrite generalizes: if the source-form was `IfElseChain`, the
  clean shape is the parameterized `match` expression; if
  `StringEqChain`, the clean shape is the same parameterized match
  over resolved identities (jointly with L1.10.c if name-as-
  discriminant was the dispatch authority).

- *Fix-confidence: templated auto-apply.* Same auto-fix algorithm as
  parent L1.13, applied at the decision-tree level. The lens emits a
  candidate Diff rewriting the decision-tree's source form to a
  parameterized match against the resolved `KeyVocabulary` carrier,
  with the appropriate distribution-class collapse.

- *Decidability boundary:* same as parent L1.13. L1.13.b extends the
  *source-form scope* (now catches if-else + string-eq chains, not
  just match) but does NOT loosen the skeleton-comparison rule.
  Branch bodies with distinct literal data, distinct call arguments,
  or distinct constructor identities still don't collapse — same
  decidability bar as parent.

- *Kills (real corpus — seed examples):*
  - **Parallel decision-tree shapes** in `src/v2/05_emit_rust.dag`:
    `rust_string_policy_for_naming` vs `rust_internal_policy_for_naming`
    (operator-identified). When the per-branch bodies are themselves
    parameterizable across the two context-fns (which is L1.13.c's
    territory), the L1.13.b finding co-fires with L1.13.c on the same
    shape — see §11 Subsumption below: L1.13.c is the root fix
    subsuming L1.13.b's individual decision-tree findings.

- *Producer stage (see §10):* L1.13.b consumes
  `v4.lens.decision_tree_shape` — the same record L1.13.c + L1.10.c
  consume. Facts Flow Forward / P2: one decision-tree-shape index,
  multiple downstream lens projections.

#### L1.13.c Table decision-tree — kills *function-encoded total tables over closed vocabularies that should be substrate data rows*

> **Status: reserved-proposed.** Sub-signature of L1.13 (Skeleton-
> collapse) at the **table-row** scope. Parent L1.13 catches K
> distinct skeletons over N match-arms (parametric duplication);
> L1.13.b generalizes to any closed-vocab decision tree; L1.13.c is
> the limiting case where **every branch produces a typed record of
> the same shape, differing ONLY in field-value-templates** — the
> function IS a typed table indexed by the closed key set.
>
> Originally drafted as L1.4.c under the Carrier-clone family; moved
> here on operator review (2026-05-21). Rationale: the mechanism is
> closed-vocab decision-table dissolution (same producer-stage
> `decision_tree_shape` as L1.13.b), not type-declaration carrier-
> clone. Practice 11 applied to function-as-table: the table is the
> substrate authority; the function is the un-derived projection.
>
> **Enforcement gate** (parallel to L1.13.b): not active until (a)
> `v4.lens.decision_tree_shape` lands carrying per-branch resolved
> identity + body-shape facts, (b) the substrate has at least one of
> `TotalMap<K, V>` (for finite payload-free tables) or `TotalPolicy<K,
> Context, RowTemplate>` (for payload-bearing key vocabularies — see
> Clean shapes below), (c) at least one Practice-11-parameterized
> dissolution shape lands as the migration target.

- *Signature:* a fn whose body is a closed-vocab decision tree (per
  `DecisionTreeShape`) returning a typed record per key, where the
  per-key bodies differ only in **field-value-templates parameterized
  by the key's payload** (not in structural shape). The fn IS a typed
  table; the substrate should carry it as data.

- *Decidable:* yes — substrate-readable from `decision_tree_shape`:
  - `key_set` is closed (substrate-resolved coproduct, possibly with
    payload-bearing constructors).
  - All branches reach a record literal of the same type (the table's
    row type).
  - Per-branch bodies, after α-renaming parameter slots and key-
    payload-bound names by canonical position, differ ONLY in
    field-value-templates whose parameters are exactly the key's
    payload fields — no structural divergence beyond what the
    payload-template parameterization expresses.
  - `missing_keys = ∅` (the table is total over the key set; partial
    tables are L1.9 vacuous-arm territory).

- *Verdict:* hard error.

- *Escape (clean 🟢):*
  - **Already a substrate row.** A `data t: TotalMap<K, V>` or
    `data t: TotalPolicy<...>` row exists with the same key set and
    row type; the fn is the canonical accessor. Passes via (R1).
  - **Branch bodies differ structurally.** If two branches' bodies
    differ in shape beyond payload-parameterized field values (e.g.,
    one branch wraps in an Option, another returns a different
    record-type union), the fn is NOT a pure table; L1.13.c does
    not fire.
  - **Context-parameterized table.** Two fns over the same closed
    vocabulary differing only by a context parameter are a context-
    table dissolution — see Fix-confidence below.

- *Clearing receipt:* substrate carries a `TotalMap<K, V>` or
  `TotalPolicy<K, Context, RowTemplate>` data row; the fn (if
  retained) is a one-line lookup. R1 applies.

- *Fix-confidence: templated auto-apply.* The lens emits two candidate
  clean shapes per the operator's payload-aware refinement
  (2026-05-21):

  - **(Pattern A) `TotalMap<K, V>` — finite payload-free keys, fully
    materialized values.** When the key vocabulary has no payload
    fields (every constructor is a 0-field tag) and the row type
    contains no key-derived field values, a literal `TotalMap` over
    fully materialized values is honest:
    ```dag
    data finite_naming_policy_table: TotalMap<FinitePolicyKey, FinitePolicyRow> =
      TotalMap { ... }
    ```

  - **(Pattern B) `TotalPolicy<K, Context, RowTemplate>` — payload-
    bearing key vocabularies.** When the key vocabulary has
    payload-bearing constructors (e.g., `StripPrefixAndSnakeCase {
    prefix: String }`, `StripSuffixAndSnakeCase { suffix: String }`,
    or contexts like `InternallyTaggedContext { tag_field: String }`),
    a literal `TotalMap<K, V>` would imply an infinite table over
    arbitrary payload values — that's NOT honest. The substrate
    carrier for this case is `TotalPolicy`:
    ```dag
    type RowTemplate { ... }                              // record shape with parameterizable fields
    data rust_serde_policy_table:
      TotalPolicy<VariantNamingConstructor, SerdeContext, RowTemplate> = TotalPolicy {
        // one row template per CLOSED CONSTRUCTOR (not per concrete payload value);
        // the row template references the constructor's payload fields by name and
        // is materialized per-call by substituting the runtime payload into the
        // template's holes.
      }
    fn rust_serde_policy(
      constructor: VariantNamingConstructor,
      payload:     VariantNamingPayload,
      context:     SerdeContext,
    ) -> RustEnumWireSerde =
      total_policy_apply(rust_serde_policy_table, constructor, payload, context)
    ```
    Pattern B is the right shape when ANY key has payload fields the
    row's value-templates project. Pattern A is the right shape only
    when EVERY key is payload-free AND every row value is independent
    of any payload (the literal finite-table case).

  Reviewer overrides which pattern at the candidate-Diff stage. The
  auto-fix infers the pattern from `decision_tree_shape`'s
  `KeyVocabulary` payload-bearing-ness.

- *Decidability boundary (explicit):* L1.13.c fires on **total tables**
  over closed vocabularies. It does **not** catch:
  - Partial tables — L1.9 vacuous-arm territory.
  - Tables with structurally-divergent branch bodies — legitimate
    per-variant dispatch (L1.13 Mixed escape).
  - Non-total dispatch where the missing-key behavior matters
    semantically — L1.11 plausible-fallback territory (or, for
    scalar-RHS, L1.11.b).
  - Open-vocabulary "tables" — when the key vocabulary is open or
    ambiguous (per `decision_tree_shape`'s typed-subject rule), the
    lens does not fire; this is the L1.10.c open-vocabulary escape
    extended to L1.13.c.

- *Kills (real corpus — seed examples):*
  - **`rust_string_policy_for_naming` / `rust_internal_policy_for_naming`**
    pair (`src/v2/05_emit_rust.dag`, operator-identified). Two near-
    parallel decision trees over `VariantNaming = AsAuthored |
    SnakeCase | StripPrefixAndSnakeCase { prefix } | StripSuffixAndSnakeCase { suffix } | ...`
    differing only in the `enum_attr` field (empty vs `#[serde(tag =
    "...")]`). The key vocabulary has **payload-bearing constructors**
    (`prefix`, `suffix`, `tag_field`) — Pattern A `TotalMap<K,V>` would
    imply an infinite table; Pattern B `TotalPolicy` is the honest
    clean shape. Cited as seed for the broader emit_rust.dag
    dissolution work.
  - **`build_call_kind_dispatch_for_func_*` shapes** (call-kind
    dispatch over a closed-vocab built-in identifier set) are likely
    L1.13.c candidates pending the substrate-fill sweep that ratifies
    the built-in-fn vocabulary as closed.

- *Producer stage (see §10):* L1.13.c consumes
  `v4.lens.decision_tree_shape` — the same record L1.13.b + L1.10.c
  consume. Facts Flow Forward / P2: one decision-tree-shape index,
  multiple downstream lens projections.

## 6. The discriminant / catamorphism distinction

L1.1 and L1.5 enforce one algebraic fact worth stating directly: a
coproduct has **two distinct derived operations** that must never be
conflated.

- The **discriminant** — "which variant?" — inspects the top constructor.
  Non-recursive, O(1).
- The **catamorphism** — the structural fold. Consumes the whole
  structure, O(n).

`free_monoid_is_empty` is a discriminant; expressing it through
`free_monoid_fold` (a catamorphism) is a category error — O(n) for an
O(1) fact — and *camouflages* the predicate-dissolution finding by moving
the variant-discriminant out of a `match` into fold-algebra arguments,
where the un-extended L1.1 would not see it. Hence L1.1's constant-algebra
extension: **a fold whose algebra ignores its recursive argument is not a
catamorphism — it is a discriminant, and is the same finding.**

## 7. The selective-profile model

A lens is a derived projection — composable and scoped. "Strictness" is
not a global dial; it is a **profile** — a set of lenses assigned to a
scope.

| profile | scope | lenses |
|---|---|---|
| **substrate** (strictest) | `src/v4/std/`, `src/v4/compiler/` | Layer 0 + all of Layer 1, all hard gates |
| **target** | `src/v4/extdeps/`, target models | Layer 0 + Layer 1, but L1.5 reviewer-confirms (target modeling has more legitimately-irregular recursion) |
| **scaffold** | early-milestone code marked `// scaffold:` | Layer 0 only |

Rules:
- **Layer 0 is on in every profile**, scaffold included. "Lens-shaped"
  does not mean "optional"; selectivity governs Layer-1 strictness only.
- **A profile downgrade is never silent and never self-service.** A file
  cannot quietly dial its own lens set down — that turns "selectively
  applied" into "selectively ignored." The `// scaffold:` marker is
  permitted, but it is the *in-file record of a reviewer-approved,
  scope-level decision*, not a worker's self-applied dial: it is
  **ratchet-only** — it may not be added to a file to dodge a lens that
  currently fires. Strict-by-default; loosen only deliberately and
  visibly.
- **Turning Layer 1 off does not turn the *discipline* off.** A
  dissolution finding in scaffold code still carries its 🟡 disposition
  tag with a bound plan — the keystone's universal "every 🟡 binds a
  dissolution plan" mandate is profile-independent. The scaffold profile
  suppresses the *CI hard-error*, never the *modeling obligation*.
  Otherwise `// scaffold:` becomes lens-off + no-tag = invisible debt —
  exactly the escape hatch this section forbids.
- The compiler runs the **substrate** profile on its own
  `src/v4/**/*.dag`. The compiler does not exempt itself from the
  discipline it enforces — that is the example others follow.

## 8. The slipped-by ledger

A living record. Every finding that reaches merge-ready despite a defect
is logged here, root-caused by the §3 methodology — so the lens set grows
from real evidence, not speculation.

| date | receipt | finding | core invariant violated | lens |
|---|---|---|---|---|
| 2026-05-18 | #3250 | `free_monoid_non_empty` hand-rolled discriminant | A coproduct's variant-discriminant is a derived operation | L1.1 |
| 2026-05-18 | #3255 | `nat_is_zero` hand-rolled discriminant | same | L1.1 |
| 2026-05-18 | #3256 | 26 combinators nominalized into single-field wrapper types | an operation is a function, not a type | L1.2 |
| 2026-05-18 | #3249 | `free_monoid_is_empty` laundered through a fold | discriminant ≠ catamorphism; do not conflate | L1.1 (extended) |
| 2026-05-18 | ingest | `node_locally_well_formed` discharges every `ComputationNode { behavior: _ } => true` (`src/v4/std/node.dag:115-120`) | exhaustive-in-shape, vacuous-in-content; closed-set named but not checked | L1.9 (proposed) |
| 2026-05-18 | ingest | `merge_evidence` / `encoding_meet` / `fermi_meet` hand-rolled while inhabitance claimed in a `// ` comment, no `data ... : Lattice<T>` row (`dsl/std/{termination,encoding,fermi}.dag`) | algebra inhabitance is a typed witness, not prose | L1.7 (proposed) |
| 2026-05-18 | ingest | `Word64 { bytes: List<Byte> }`, `Float32`/`Float64` share unconstrained `FloatBody` (`src/v4/std/{machine,float}.dag`) | cardinality / width is a refinement, not a name | L1.7 (proposed) |
| 2026-05-18 | ingest | `ResourceHandle` is a freely-constructible record under prose claiming opacity (`dsl/std/resources.dag:17-25`) | non-forgeability is a constructor restriction, not a comment | L1.7 (proposed) |
| 2026-05-18 | ingest | `nat_compare(Nat, Nat)` defined in `src/v4/std/float.dag:52-62` while `nat.dag` and `algebra.dag` exist | an operation lives in its argument-type's home (M9 / DFS the concept DAG) | L1.8 (proposed) |
| 2026-05-18 | ingest | `CiCommand::ShellCommand { command: String }` while `extdeps/posix.dag` models a typed `Command` (`src/v4/workflow/ci.dag:23-28`) | String escape hatch for a domain that has a typed model in scope | L1.10.b `CanonicalCarrier` (proposed) |
| 2026-05-18 | ingest | `list_template: "Vec<{0}>"`, `optional_template: "Option<{0}>"`, `map_template: "HashMap<{0}, {1}>"` etc. in `dsl/std/languages.dag` + per-language emit tables | string literal with positional placeholders used as emitter; grammar-as-bidirectional-data belongs | L1.10.a `TemplateHole` (proposed; absorbs original L1.6) |
| 2026-05-18 | ingest | `derive_effect_shape` `DELETE/PUT/PATCH None => CreateEffect` (`dsl/std/effects.dag:268-282`) | missing info must escalate through `Outcome::Rejected`, not return a different valid sibling | L1.11 (proposed) |
| 2026-05-18 | ingest | `Bool`, `Char`, `Url`, machine words declared in both `dsl/std/` and `src/v4/std/` (`type T = ...` redeclared, no structural alias/retirement/migration) | one concept must have one home (structural alias, retirement-ledger row, or migration) | L1.12 (proposed) |
| 2026-05-18 | ingest | `extdeps/languages/resolver.dag` referenced via imports but file does not exist; provisional `GroundingMap` lives in `extdeps/languages/rust.dag` | unresolved reference / dangling import — fail-closed P3, distinct root cause from duplicate authority | L0.8-extended (planned-but-absent home; deliberately not L1.12) |

Pattern from the seed PR rows: the original four are burn-down
*substrate* PRs — the lane built to remove dissolution debt produced
it. Each was *mostly* correct with one dissolution defect; #3249's was
invisible to reviewers because the fold-laundering hid it. This is why
the lens suite (mechanical, every time) and the burn-down pre-gate
(catch at the source) both exist. The later `ingest` rows extend the
ledger to A0's broader territory (prose / name / string / canonical-home /
plausible-fallback findings); they are not all derived-operation
defects but share the same root-cause pattern — a missing structural
witness that workers backfill locally.

## 9. Build path — model-derived only

The lens is **compiler-integral**: a modeled `src/v4/lens/` projection
(joining `complexity.dag` et al.) that the compiler itself runs and
rejects on. There is **no interim hand-written-script form.** A
`scripts/check-*` script that text-scans `.dag` is itself a hand-rolled
`.dag`-walker — the exact anti-pattern this lens exists to remove. The
dissolution lens cannot be hand-rolled either; it is a `fold` over the
*parsed* model, a consumer of the substrate-first sequence.

Consequence: the lens is **gated on the v4 front-end** (CP-1) being able
to parse `.dag` into a model, plus the v4 lens stage that runs
projections over it. Until then there is no *mechanical* dissolution
enforcement — the interim net is the reviewer prompts and the burn-down
pre-gate (human/agent, reactive), not a script. The lens's timeline is
CP-1's timeline.

Build order once the machinery exists: Layer 0 first (Layer 1 composes
its primitives — L0.2 → L1.1, L0.4 → L1.3), then Layer 1.

## 10. Dependency model — lenses are pipeline stages

The lens framework is **not separate infrastructure**. There is no
"lens engine" with its own dependency model. Lenses are `.dag` stages
in the existing compiler pipeline, and they participate in the same
`consumes:` / `produces:` machinery every other stage uses. A second
dependency system would itself be the L1.12-class parallel-authority
hazard the lens suite exists to kill — self-application: the lens
framework cannot violate the discipline it enforces.

### 10.1 Shared indices come from existing pipeline stages

Each compiler stage already produces structural facts as outputs. The
shared indices the lenses need are not a new layer — they're outputs
of stages that run anyway for typecheck / normalize / resolve / infer.
A lens consumes them via the same edges any other downstream stage
uses.

| Shared index | Produced by (existing stage) | Lenses that consume |
|---|---|---|
| AST + spans | `v4.compiler.02_parse` | universal |
| Symbol resolution (name → declaration site) | `v4.compiler.03_resolve` | L0.8, L1.8, L1.12, plus most L1.x |
| Coproduct variant list (type → variants) | `v4.compiler.02_parse` / `03_resolve` | L1.1, L1.5, L1.9, L0.12, L0.13 |
| Inhabitance edge set (type → {ctors, returns, fields, aliases}) | `v4.compiler.03_resolve` / `04_infer` | L1.2, L1.3, L1.4 |
| Witness / registry index (`data ... : Algebra<T>`, `CanonicalCarrier`, `HistoricalDeclaration`, `CanonicalConcept`, `canonical_observations`) | `v4.compiler.03_resolve` | L1.7, L1.8, L1.10.b, L1.12 |
| Refinement-clause index (type → length / domain refinements) | `v4.compiler.04_infer` | L1.7 |
| Import graph + target existence | `v4.compiler.02_parse` | L0.8, L1.8 |
| Return-type → fail-closed-carrier? | `v4.compiler.03_resolve` | L0.14, L1.11 |
| Match-arm RHS skeleton + per-arm group membership + per-skeleton constructor-hole presence (normalized RHS after α-renaming + matched-arm constructor substitution; groups expose arm-ids so the auto-fix consumes them as facts, not by re-walking) | `v4.lens.match_arm_skeleton` (new derived stage — see §10.2) | L1.13 |
| Per-decl structural-shape facts (type-decl: variant set + per-variant field shape; fn-decl: signature shape + body catamorphism-form classifier + identifier token-set) | `v4.lens.structural_similarity` (new derived stage — see §10.2) | L1.12.b, L1.4.b |
| Normalized decision-tree shape (closed-vocab branch dispatch over resolved constructor identities — exposes per-branch skeleton + missing-key behavior, REGARDLESS of source spelling as `match` / `if-else` / string-equality chain; ALSO carries typed-subject requirement: `StringEqChain` produces `KeyVocabulary` only when subject has unique resolved closed carrier per `04_infer`) | `v4.lens.decision_tree_shape` (new derived stage — see §10.2) | L1.13.b, L1.13.c, L1.10.c |
| Generated-forest shape (`map`/`fold` over a closed coproduct's variant-set that emits per-variant target artifacts — exposes per-generated-arm skeleton + variant-iteration evidence) | `v4.lens.generated_forest_shape` (new derived stage — see §10.2) | L1.5.b |
| Target-syntax string-construction shape (string-build graph classified as data-strings vs target-grammar-token sequences vs Unknown, gated by a `TargetGrammarTokenSet` substrate carrier per target — closed structural predicate, no numeric threshold) | `v4.lens.target_syntax_string_shape` (new derived stage — see §10.2) | L1.10.d |
| Scalar use-site authority-role classification (per-scalar set of consumer roles: TargetSyntaxUse / IdentifierUse / UrlUse / HeaderNameUse / FilePathUse / ResourceCoordinateUse / DataStringUse / UnknownUse, gated by a `scalar_authority_use_registry` substrate carrier) | `v4.lens.scalar_authority_use_shape` (new derived stage — see §10.2) | L1.11.b |

### 10.2 Nine small derived stages cover what the pipeline doesn't already expose

These are themselves `.dag` stages — small folds with declared
`consumes:` edges — and they're reusable across multiple lenses:

```dag
module v4.lens.match_arm_shape
consumes: v4.compiler.02_parse
produces: Map<MatchArmId, ArmShape>     // {trivial-literal | structural-work | identity-passthrough}
// reusable by: L1.1, L1.9, L1.11, L0.7, L0.13

module v4.lens.closed_vocab_scan
consumes: v4.compiler.02_parse
produces: Map<DeclId, Set<ClaimToken>>  // fact-bearing tokens in comments/identifiers
// reusable by: L1.7

module v4.lens.concept_home
consumes: v4.compiler.03_resolve, v4.lens.canonical_observations_index
produces: Map<DeclId, File>             // each declaration's primary-concept home file
                                        // (DeclId = FnId | TypeId — covers both fn-scope and type-scope
                                        // canonical-home authority for L1.12.b. The historical narrower
                                        // shape Map<FnId, File> was insufficient: L1.12.b's type-scope
                                        // (C1) requires resolving a non-std/ type back to its canonical
                                        // home in the same way the fn-scope (C1) does. Broadening to
                                        // DeclId keeps one producer-stage authority for both — Facts
                                        // Flow Forward / P2.)
// reusable by: L1.8 (fn-home class), L1.12.b (fn-scope + type-scope canonical-home authority)

module v4.lens.structural_similarity
consumes: v4.compiler.02_parse, v4.compiler.03_resolve
produces: Map<DeclId, StructuralShape>
// StructuralShape = TypeShape | FnShape
// TypeShape = {
//   variant_set: [VariantShape],            // sorted; bijection-comparable
//   total_variants: Nat,
// }
// VariantShape = {
//   field_arity: Nat,
//   field_type_idents: [TypeId],            // by coproduct identity, not by name
// }
// FnShape = {
//   signature: SignatureShape,              // input coproduct identity, output coproduct identity, parameter arity
//   body_normalized: BodyShape,             // α-renamed body expression tree (parameter slots + bound
//                                           //   names normalized by canonical position); compared structurally.
//                                           //   REQUIRED for (C1) fn-scope; signature-only-match is NOT a fire.
//   catamorphism_form: CatamorphismForm,    // None | StructuralFoldOver { type_id, algebra }
//   token_set: Set<Token>,                  // identifier + variant/field names (C3 triage only)
// }
// CatamorphismForm = None | StructuralFoldOver { type_id: TypeId, algebra: AlgebraShape }
// AlgebraShape = {
//   per_variant:         Map<VariantId, NormalizedArmBody>,
//                                           // one entry per variant of type_id; NormalizedArmBody is the arm's
//                                           // RHS expression tree after α-renaming pattern-bound names by canonical
//                                           // position. Consumed by the (C2) auto-fix to emit the fold_T call
//                                           // with the extracted algebra structurally — no re-walking of the fn body.
//   recursive_positions: Set<VariantField>, // which variant fields recurse (drives fold_T's recursive-position
//                                           // contract); empty when the fold is fully consuming. MUST match the
//                                           // FoldRegistryEntry's recursive_positions for (C2) to fire.
//   free_params:         List<ParamSlot>,   // fn parameters NOT bound by the fold-walk (e.g., lookup_chain's `name`
//                                           // parameter is a free param threaded into every arm). The (C2) auto-fix
//                                           // must include these in the fold_T call's algebra-binding.
//   fold_order:          FoldOrder,         // LeftFold | RightFold | UnorderedFold; extracted from the order in
//                                           // which the body composes recursive results. Required for foldl-vs-foldr
//                                           // distinction in the auto-fix.
//   result_carrier:      TypeId,            // the output type identity. Drives the algebra-record type the fold_T
//                                           // call instantiates.
//   short_circuit:       Optional<ShortCircuitCarrier>,
//                                           // None for total folds; Some(carrier) when the body has early-return
//                                           // semantics (e.g., lookup_chain returns Some on first hit, never
//                                           // recursing on Some). The short-circuit carrier names the discriminator
//                                           // type (typically Option / Outcome / a 2-variant decision carrier);
//                                           // the (C2) auto-fix wraps the fold accordingly.
// }
// Together these six fields are enough for the (C2) auto-fix to emit
// a structurally-correct fold_T call with no consumer re-inference.
// They are extracted in the same fold-walk that classifies
// CatamorphismForm — single pass, single authority.
// SignatureShape comparison: bijection on parameter slots preserving
// type identity (NOT names); output type by coproduct identity.
// BodyShape comparison: structural identity of the α-renamed
// expression tree. Two fns match under (C1) iff signature AND body
// shape BOTH match — signature-only is a deliberate non-fire (two
// fns sharing a `(T) -> U` shape is expected steady-state).
// CatamorphismForm extraction: scan fn body for one-arm-per-T-variant
// match with recursive calls only at T-variant positions; classify
// `StructuralFoldOver { type_id: T, algebra: extracted-algebra }`
// (carrying the per-variant arm bodies + recursive-position set as
// substrate facts) if so; `None` otherwise. The algebra extraction
// is the SAME fold-walk that classifies the form — extracted in
// the same pass, emitted in the same record. Facts Flow Forward
// (Practice 3 / P2): the (C2) auto-fix consumes the algebra
// directly from this record; no consumer re-walks the fn body or
// re-extracts the algebra.
// Facts Flow Forward (Practice 3): the lens emits everything L1.12.b
// needs for (C1) signature-match, (C2) catamorphism-equivalence, and
// (C3) token-vocabulary triage. No consumer re-walks declarations or
// re-derives shape equivalence — single authority (P2), one
// mechanism, multiple downstream projections.
// reusable by: L1.12.b, future cross-substrate-version drift lens,
// future module-graph homomorphism lens
// Algorithm: walk each parsed declaration; for `type` decls emit
//   TypeShape with the sorted normalized variant set; for `fn` decls
//   emit FnShape with signature normalized (parameter-bijection
//   canonical form), body classified for catamorphism form, and the
//   token-set extracted from the AST identifiers. Substrate-only,
//   no comment/prose inspection.

module v4.lens.match_arm_skeleton
consumes: v4.compiler.02_parse, v4.compiler.03_resolve
produces: Map<MatchExprId, SkeletonReport>
// SkeletonReport = {
//   arm_count: Nat,
//   distinct_skeletons: Nat,
//   histogram: [Nat],                // sorted group-sizes, largest first
//   classifier_shape: ClassifierShape, // PureTemplate | Outlier | MultiOutlier | Categorical | Mixed
//   groups: [SkeletonGroup],         // one entry per distinct skeleton — per-arm membership
// }
// SkeletonGroup = {
//   skeleton: RhsSkeleton,                       // the normalized RHS expression tree
//   arm_ids: [MatchArmId],                       // arms in this skeleton-equivalence class
//   constructor_hole_present: Bool,              // does the skeleton contain the constructor-hole at any
//                                                // position? (distinguishes Outlier sub-cases a/b
//                                                // without re-walking — drives Direct vs Templated
//                                                // auto-apply per the L1.13 entry above)
//   matched_constructors: [ConstructorId],       // the constructors whose arms collapsed to this skeleton
//                                                // (lens reads to derive templated names for the
//                                                // MultiOutlier/Categorical wrapping-variant default)
// }
// Facts Flow Forward (Practice 3): the lens emits everything the L1.13
// auto-fix needs (and the L1.13.c table-decision-tree sub-signature
// reuses it for per-arm-RHS-referencing-typed-data cases). No consumer
// re-walks the arms or re-derives skeleton equivalence — single
// authority (P2), one mechanism, multiple downstream projections.
// reusable by: L1.13 (base, MatchExpr-scoped), L1.13.c (table
// decision-tree — per-arm references to typed `data` declarations
// with one-to-one correspondence to the matched variants)
// Algorithm: tree-walk each arm's RHS, α-rename pattern-bound names,
//   substitute every occurrence of the matched-arm constructor identity
//   with a per-arm hole, structurally compare; group arms by skeleton
//   (each group records arm_ids + matched_constructors + whether the
//   skeleton contains the constructor-hole anywhere); sort group sizes
//   (largest first) to form histogram; classify distribution-shape per
//   L1.13's thresholds (PureTemplate / Outlier / MultiOutlier /
//   Categorical / Mixed).
```

```dag
module v4.lens.decision_tree_shape
consumes: v4.compiler.02_parse, v4.compiler.03_resolve, v4.compiler.04_infer
produces: Map<DecisionTreeId, DecisionTreeShape>
// DecisionTreeShape normalizes branch dispatch over a closed vocabulary,
// REGARDLESS of source-level spelling (match / if-else chain /
// string-equality chain). This is the algebraic generalization of
// `match_arm_skeleton`: that stage handles match-arms only; this stage
// handles any closed-vocab decision tree. Lenses that fired only on
// `match` shapes are extended via this producer to fire on the same
// algebraic shape regardless of syntax.
// DecisionTreeShape = {
//   key_set:          KeyVocabulary,                  // the closed-vocab keys: resolved ConstructorId | FieldId
//                                                     //   | VariantId. NOT spelled names — see KeyVocabulary below.
//                                                     //   Authority for membership is 03_resolve; an if-else over
//                                                     //   name-string-equality on a closed coproduct's variant set
//                                                     //   resolves to the SAME KeyVocabulary as the corresponding
//                                                     //   match would. That's how the producer catches
//                                                     //   name-as-discriminant cases (L1.10.c) at the same algebraic
//                                                     //   shape as match-over-coproduct cases (L1.13).
//                                                     // **Typed-subject requirement (load-bearing):** a StringEqChain
//                                                     //   produces a KeyVocabulary ONLY when the compared subject
//                                                     //   has a UNIQUE resolved closed carrier via 04_infer. A
//                                                     //   string comparison like `name == "StringVariant"` resolves
//                                                     //   to a KeyVocabulary iff 04_infer says the subject expression's
//                                                     //   type is exactly one closed coproduct whose variant set
//                                                     //   contains the matched spelling. If the subject's resolved
//                                                     //   type is ambiguous across namespaces, open, or unresolved,
//                                                     //   source_form is `OpenOrAmbiguousNameDispatch` (not
//                                                     //   StringEqChain) and downstream lenses (L1.10.c) do NOT
//                                                     //   auto-fire — open-vocabulary escape preserved.
//   branches:         [BranchShape],                  // one entry per detected branch; sorted by key for
//                                                     //   stable comparison.
//   missing_keys:     Set<KeyId>,                     // closed-vocab keys absent from the tree (drives
//                                                     //   vacuous-arm / missing-case detection in L0/L1.9
//                                                     //   consumers).
//   missing_behavior: MissingKeyBehavior,             // FailClosed | DefaultArm(BranchShape) | Unhandled
//                                                     //   (= silently fall through). Drives plausible-fallback
//                                                     //   sub-signatures' detection of None=>literal patterns.
//   source_form:      DecisionTreeSourceForm,         // MatchExpr | IfElseChain | StringEqChain | NestedMix |
//                                                     //   OpenOrAmbiguousNameDispatch (see KeyVocabulary above —
//                                                     //   this last variant fires when name-equality dispatch
//                                                     //   does NOT resolve to a unique closed carrier; carries
//                                                     //   no KeyVocabulary; lens consumers SHOULD NOT auto-fire
//                                                     //   on it). Recorded as fact (for the diagnostic surface
//                                                     //   only) for MatchExpr / IfElseChain / StringEqChain /
//                                                     //   NestedMix — downstream lens semantics are UNIFORM
//                                                     //   regardless of source-form for those four; the field
//                                                     //   exists to point reviewers at the actual source span
//                                                     //   when a finding fires.
// }
// KeyVocabulary = {
//   carrier:   TypeId,                                // the closed coproduct being dispatched on (must resolve
//                                                     //   to a closed-sum decl from 03_resolve).
//   members:   Set<ConstructorId>,                    // the resolved constructor identities; canonical, not by
//                                                     //   spelling.
// }
// BranchShape = {
//   key:                   KeyId,                     // ConstructorId | VariantId — resolved identity.
//   guarded:               Bool,                      // whether the branch has a guard clause.
//   body_normalized:       BodyShape,                 // α-renamed body expression tree (parameter slots +
//                                                     //   bound names normalized by canonical position).
//                                                     //   COMPARED structurally — same algebra as the FnShape
//                                                     //   body_normalized used by L1.12.b's (C1) fn-scope.
//   produces_typed_value:  Bool,                      // distinguishes "this branch returns a typed
//                                                     //   constructor / value" from "this branch returns a
//                                                     //   scalar literal or String". Drives the L1.11.b
//                                                     //   plausible-scalar-fallback detection without a
//                                                     //   separate walk.
// }
// Facts Flow Forward (Practice 3 / P2): one producer captures the
// canonical form of every closed-vocab decision tree; consumers
// (L1.13 generalized, L1.13.b new, L1.10.c name-discriminant-bypass,
// L1.13.c table decision-tree) read the same record. No
// consumer re-walks source expressions or re-classifies if-else as
// decision-tree.
// reusable by: L1.13 (base, MatchExpr-scoped), L1.13.b (decision-tree
// collapse — generalized beyond match), L1.13.c (table decision-tree
// — function-as-table dissolution), L1.10.c (name-discriminant-bypass
// — string-equality dispatch where resolved identity is available)
// Algorithm: walk each fn body's expression tree; identify
//   subgraphs that branch over a closed coproduct's resolved
//   constructor-identity set (whether spelled as `match`,
//   `if/else if`, or `name == "X" ? ... : ...`); normalize into
//   the DecisionTreeShape record above; collect into the produced
//   map keyed by syntactic locus.

module v4.lens.generated_forest_shape
consumes: v4.compiler.02_parse, v4.compiler.03_resolve, v4.compiler.04_infer
produces: Map<GeneratedForestId, GeneratedForestShape>
// GeneratedForestShape captures the *meta* case of "a fn iterates
// over a closed coproduct's variant-set AND emits a per-variant
// target-language artifact (text or AST)." This is the case
// `emit_enum_shared_accessors`-shape: the function body is a
// `children |> map(child => emit_per_child_code)` that generates
// what amounts to an N-arm match in the TARGET language as a
// derived projection. Current L1.13 catches source-level match
// arms; this catches the META-emission analog.
// GeneratedForestShape = {
//   iterated_carrier:    TypeId,                      // the closed coproduct whose variants are iterated
//                                                     //   (e.g., a TypeNode's variant_set).
//   variant_coverage:    VariantCoverage,             // Total (all variants iterated) | Filtered { predicate }
//                                                     //   | Partial (covers strict subset by hand-roll —
//                                                     //   probably itself a defect).
//   emitted_artifact:    EmittedArtifactKind,         // GeneratedString | GeneratedMatchArm | GeneratedNode.
//   per_variant_skeleton: Map<ConstructorId, NormalizedArmBody>,
//                                                     // the emission template's per-variant body
//                                                     //   (same NormalizedArmBody form as L1.13's
//                                                     //   SkeletonReport groups carry — algebra reused).
//   skeleton_groups:     [SkeletonGroup],             // same SkeletonGroup as match_arm_skeleton's;
//                                                     //   classifies the generated forest's distribution-shape
//                                                     //   (PureTemplate / Outlier / MultiOutlier / Categorical
//                                                     //   / Mixed). When the generated forest collapses to K
//                                                     //   distinct skeletons over N variants with K < N, the
//                                                     //   forest is parametric and should be a typed projection
//                                                     //   over the coproduct, not a generated forest.
// }
// reusable by: L1.5.b, future emit-pattern-detector lenses
// Algorithm: identify fns whose body is a fold/map over a closed
//   coproduct's variant-set (i.e., consumes the variant-set of a
//   resolved type from 04_infer); normalize the per-variant emission
//   template into the same NormalizedArmBody shape used by
//   match_arm_skeleton; reuse the skeleton-classification algorithm
//   to determine if the generated forest is collapsing-to-K-skeletons.
//   Cross-substrate: the algorithm shares the skeleton-extraction
//   and grouping code path with match_arm_skeleton, applied at the
//   emission scope instead of the match-arm scope (Practice 11 —
//   same mechanism, two scopes).

module v4.lens.target_syntax_string_shape
consumes: v4.compiler.02_parse, v4.compiler.03_resolve,
          v4.compiler.04_infer,
          target_grammar_token_set_registry  // see substrate-carrier note below
produces: Map<StringConstructionId, TargetSyntaxStringShape>
// TargetSyntaxStringShape classifies string-construction graphs as
// data-strings vs target-grammar-token sequences. Distinguishing
// "concat-builds-a-target-grammar-statement" (target-syntax bypass —
// fires) from "concat-builds-a-data-string-payload" (legitimate —
// passes) requires substrate authority on what counts as a target-
// grammar token for a given target language.
// SUBSTRATE PREREQUISITE: a `TargetGrammarTokenSet` registry carrier
// per target language MUST exist before this producer fires. PR
// #3476's `rust.dag` LanguageModel — specifically the `rust_wave1_*`
// grammar productions + lex rules — is the substrate authority for
// Rust grammar tokens, and is the canonical first-instance. Other
// target languages land their TargetGrammarTokenSet when their
// LanguageModel does. Until a TargetGrammarTokenSet row exists for
// a target, the producer conservatively classifies all string
// constructions involving that target as `Unknown`; downstream
// lenses (L1.10.d) treat `Unknown` as non-fire (fail-closed: no
// false positives on targets without LanguageModel substrate).
// TargetSyntaxStringShape = {
//   construction_graph: StringConstructionGraph,      // the AST sub-tree of String-producing operations
//                                                     //   (concat / format! / explicit literals).
//   classification:     StringClassification,         // DataString | TargetGrammarTokenSequence { target: LangId }
//                                                     //   | Unknown.
//                                                     //   Classification semantics — closed predicate, no
//                                                     //   numeric threshold:
//                                                     //
//                                                     //   (1) No TargetGrammarTokenSet exists for any target the
//                                                     //       string might reach → Unknown. (NOT DataString —
//                                                     //       absence of substrate is not evidence of data; it's
//                                                     //       absence of evidence.)
//                                                     //   (2) Known TargetGrammarTokenSet AND no emitted literal
//                                                     //       segment matches a grammar token/delimiter AND the
//                                                     //       enclosing sink is a data-string sink (log, message,
//                                                     //       file-path-as-data, diagnostic, etc.) → DataString.
//                                                     //   (3) Known TargetGrammarTokenSet AND at least one
//                                                     //       emitted literal segment IS a grammar token or
//                                                     //       delimiter AND the enclosing sink is target source /
//                                                     //       TargetSurfaceNode serialization / emitted artifact
//                                                     //       text AND the nonliteral holes are typed model values
//                                                     //       that would be children of a TargetSurfaceNode →
//                                                     //       TargetGrammarTokenSequence.
//                                                     //   (4) Any other case (ambiguous, partial, sink unknown) →
//                                                     //       Unknown.
//                                                     //
//                                                     //   Each of (1)-(4) is a closed structural test; no numeric
//                                                     //   density / token-count threshold. Lenses (L1.10.d) treat
//                                                     //   Unknown as non-fire (fail-closed: no false positives).
//   grammar_evidence:   [GrammarTokenMatch],          // per-substring evidence from TargetGrammarTokenSet
//                                                     //   matches (keyword / delimiter / attribute / type-name
//                                                     //   prefix / etc.) when classification =
//                                                     //   TargetGrammarTokenSequence.
//   sink_classification: SinkClassification,          // DataStringSink | TargetSourceSink | EmittedArtifactSink |
//                                                     //   UnknownSink. Drives the sink-test in classification
//                                                     //   rules (2) and (3) above.
//   missing_typed_path: Optional<TargetSurfaceNodeId>,
//                                                     // when classification = TargetGrammarTokenSequence AND
//                                                     //   a substrate TargetSurfaceNode would naturally model
//                                                     //   this construction, names the missing typed path that
//                                                     //   would close the bypass. Drives L1.10.d's clean-shape
//                                                     //   recommendation.
// }
// reusable by: L1.10.d, future grammar-bidirectional lenses
// Algorithm: walk each fn body's expression tree; identify String-
//   producing sub-expressions; for each, query the per-target
//   TargetGrammarTokenSet registry for token-match evidence AND the
//   enclosing sink classification; apply the four-case closed predicate
//   above; emit the produced record. Absence of TargetGrammarTokenSet
//   → Unknown (NOT DataString — corrects a pre-rev1 inconsistency).

module v4.lens.scalar_authority_use_shape
consumes: v4.compiler.02_parse, v4.compiler.03_resolve, v4.compiler.04_infer,
          scalar_authority_use_registry  // see substrate-carrier note below
produces: Map<ScalarRhsId, ScalarAuthorityUseShape>
// ScalarAuthorityUseShape classifies the USE-SITE of a scalar RHS
// (String / Int / Bool literal returned from a closed-vocab decision
// tree's missing/None arm). The same scalar `""`, `"Authorization"`,
// `"http://localhost"`, `0`, `false` can be benign (a data field) or
// fabricating-target-authority (HTTP header coordinate, URL,
// identifier, file path, resource coordinate). The lens L1.11.b needs
// the USE-SITE classification — what role does the consumer treat
// this scalar as? — NOT just the production-site shape.
// ScalarAuthorityUseShape = {
//   rhs_locus: ScalarRhsId,                            // the scalar literal's site (in a None=>/default arm).
//   uses:      Set<UseSite>,                           // each downstream consumer of this scalar's value.
// }
// UseSite = {
//   use_locus:   UseSiteId,                            // where the scalar is consumed.
//   role:        ScalarUseRole,                        // closed coproduct — see below.
// }
// ScalarUseRole
//   = TargetSyntaxUse              { target: LangId }   // consumed as target-grammar source (cross-references
//                                                       //   target_syntax_string_shape's classification at the
//                                                       //   USE site).
//   | IdentifierUse                { id_kind: IdentifierKind }
//                                                       // consumed as a symbolic identifier (variable name,
//                                                       //   function name, module name) where the substrate would
//                                                       //   carry a typed Symbol/Name.
//   | UrlUse                       { url_kind: UrlKind } // consumed as a URL / network coordinate.
//   | HeaderNameUse                { protocol: ProtocolId }
//                                                       // consumed as a wire-protocol header name (HTTP header,
//                                                       //   etc.) — typically a substrate-typed enum.
//   | FilePathUse                  { path_kind: FilePathKind }
//                                                       // consumed as a file-system path or path fragment.
//   | ResourceCoordinateUse        { resource: ResourceKind }
//                                                       // consumed as a typed resource coordinate (queue name,
//                                                       //   service endpoint, etc.).
//   | DataStringUse                                     // consumed as data (log message, diagnostic body,
//                                                       //   user-facing text) — the BENIGN case.
//   | UnknownUse                                        // no substrate carrier in scalar_authority_use_registry
//                                                       //   declares the consumer's role; conservatively
//                                                       //   classify as Unknown (lens fails closed).
// SUBSTRATE PREREQUISITE: a `scalar_authority_use_registry` substrate
// carrier MUST exist before this producer fires. The registry declares
// per-consumer the ScalarUseRole the consumer's parameter / field
// expects. Without the registry, the producer returns Unknown for all
// uses; L1.11.b treats Unknown as non-fire (fail-closed). The registry
// can be populated incrementally as call-sites are classified —
// HTTP-transport consumers, URL parsers, file-system primitives, etc.
// each declare their authoritative parameter roles. Until at least one
// registry row exists per ScalarUseRole variant, the corresponding
// L1.11.b sub-class doesn't fire.
// Facts Flow Forward (Practice 3 / P2): one producer captures every
// scalar's use-site role classifications via the registry; L1.11.b
// consumes this directly. L1.10.d consumes target_syntax_string_shape
// for TARGET-SYNTAX classification at the PRODUCTION site (different
// classification axis: what is being built, not what role the scalar
// fills).
// reusable by: L1.11.b, future authority-sink-classification lenses
// Algorithm: walk each fn body's expression tree; identify scalar
//   literal sub-expressions (especially those reached from a closed-
//   vocab decision tree's missing/None arm per decision_tree_shape);
//   for each scalar, walk its data-flow to consumer call sites; query
//   the scalar_authority_use_registry for each consumer's declared
//   ScalarUseRole; emit the produced record with the union of uses.

```

Each is a single deterministic fold. Once landed, multiple lenses
share the result — landing `match_arm_shape` unblocks five lenses,
`match_arm_skeleton` unblocks L1.13 (base) and L1.13.b's match-arm
sub-cases, `structural_similarity` unblocks L1.12.b + L1.4.b,
`decision_tree_shape` unblocks the generalized L1.13.b + L1.13.c +
L1.10.c family (replaces match-only detection with shape-detection-
regardless-of-syntax, gated on typed-subject identity from
`04_infer`), `generated_forest_shape` unblocks L1.5.b,
`target_syntax_string_shape` unblocks L1.10.d (gated on per-target
TargetGrammarTokenSet substrate landing — `rust.dag` LanguageModel
from PR #3476 is the canonical first instance), and
`scalar_authority_use_shape` unblocks L1.11.b (gated on per-consumer
`scalar_authority_use_registry` rows declaring authoritative
parameter roles).

### 10.3 A lens is a stage with declared dependencies

```dag
module v4.lens.dissolution
consumes:
  v4.compiler.02_parse
  v4.compiler.03_resolve
  v4.compiler.04_infer
  v4.lens.match_arm_shape
  v4.lens.match_arm_skeleton
  v4.lens.structural_similarity
  v4.lens.decision_tree_shape
  v4.lens.generated_forest_shape
  v4.lens.target_syntax_string_shape
  v4.lens.scalar_authority_use_shape
  v4.lens.closed_vocab_scan
  v4.lens.concept_home
produces:
  Set<Diagnostic>
```

The compiler's existing stage-ordering — same machinery that ensures
`04_infer` runs after `03_resolve` — automatically schedules the lens
stage after its dependencies. **Adding a new lens = land a `.dag`
stage with its own `consumes:` declaration; pipeline ordering and
parallelism are derived, not configured.**

### 10.4 Parallelism falls out of the dependency graph

By §5.0's design, every lens is a deterministic structural projection
with no cross-lens mutable state. That gives two natural parallelism
axes derived directly from the dependency graph:

- **Per file.** Each `.dag` file's parse/resolve/infer state is
  independent of every other file's; the lens-stage fan-out is
  embarrassingly parallel — one task per file in scope.
- **Per lens within a file.** Two lenses reading the same shared
  indices have no contention (indices are read-only by the time
  lenses run); they fan out at the predicate-evaluation level.

Combined with `v4.lens.affected_set` (already in the pipeline; only
re-lens files whose downstream closure has changed), CI cost scales
with PR size, not with corpus size.

### 10.5 What this gives you

- **One dependency model** for the whole compiler — pipeline stages
  and lenses live in the same graph.
- **Adding a lens** is a `.dag` stage land, not a framework
  extension.
- **Adding an index** (when a new lens needs facts current stages
  don't expose) is a small derivation stage between an existing
  producer and the lens — and reusable by any future lens that
  shares the same dependency.
- **No "lens framework"** — there's just the pipeline, and lenses
  are stages in it.
- **Self-application is clean**: the pipeline is `.dag`-modeled,
  the lens stage participates as a peer, and the compiler enforces
  the discipline it follows.

## 11. Subsumption model — minimal-edit dissolution ordering

The lens framework so far is **per-lens-per-defect**: each lens fires
independently, the diagnostic surface enumerates every finding
separately. A file like `src/v2/05_emit_rust.dag` (6876 lines, 240
match expressions, 300 fn declarations) would surface dozens of
findings simultaneously — L1.5 × many, L1.10.d × many, L1.13.b ×
several, L1.13.c × handful, L1.5.b × handful — when **the minimal
fix is one Diff: "consume the substrate `LanguageModel` via a
grammar-driven serializer"** that mechanically closes ALL of them.

The per-lens-per-defect surface forces the substrate author to fix
one finding at a time, with each fix revealing the next finding
behind it. That is the wrong UX. The framework needs **subsumption
ordering**: a property of fix Diffs that the diagnostic surface
respects so the highest-leverage fix surfaces first.

### 11.1 The subsumption relation

A fix Diff `D_root` **subsumes** Diffs `D_a, D_b, D_c, …` when:
- Applying `D_root` mechanically resolves the substrate facts the
  lenses producing `D_a/D_b/D_c` consume, such that those lenses no
  longer fire on the post-`D_root` substrate.
- The relationship is decidable: given a candidate root Diff and a
  set of leaf Diffs, the framework MUST be able to apply `D_root`
  to a substrate snapshot and re-run the lenses to confirm the
  leaves clear. Subsumption is **mechanically verifiable**, not
  declared.

Subsumption forms a DAG over the candidate fix set (most lenses
already emit candidate Diffs in `Fix-confidence: templated auto-apply`
form). The DAG's **roots** are the highest-leverage fixes — applying
them closes downstream leaves automatically. The DAG's **leaves** are
local fixes that don't subsume anything else.

### 11.2 The substrate carrier

```dag
type DissolutionSubsumption {
  root_fix:        DiffId          // the dominating fix candidate
  subsumed_fixes:  Set<DiffId>     // fixes closed by applying root_fix
  verification:    SubsumptionVerification
                                   // how the subsumption was verified
                                   // (test-run + lens-re-run, or
                                   // structurally-derived from
                                   // producer-stage facts)
}
type SubsumptionVerification
  = MechanicalReverification {
      // applied root_fix to a substrate snapshot, re-ran the lens
      // suite, confirmed subsumed_fixes' lenses no longer fire.
      // The verification is a re-runnable test-claim row.
      test_claim: TestClaimId
    }
  | ProducerStageDerivation {
      // root_fix changes a substrate fact that one or more
      // producer stages downstream-derive from; the subsumption is
      // mechanically derivable from the producer-stage's
      // `consumes:` graph + the fact-update.
      derivation_path: List<ProducerStageId>
    }
```

Each lens that emits a candidate Diff in its `Fix-confidence:` clause
optionally emits a `DissolutionSubsumption` row when its candidate is
known to subsume others. The framework integration test verifies the
row mechanically — apply the root, re-run lenses, confirm closure.
Comment-only subsumption claims are not honored (same no-prose
discipline as the rest of the framework).

### 11.3 Diagnostic surface ordering

When N findings are present, the diagnostic surface presents them
ordered by subsumption DAG topology:

1. **Roots first.** Findings whose Diffs are subsumption-DAG roots
   surface as the primary recommendation. Each root's surface
   includes the count of leaf-findings it subsumes — so the
   substrate author sees `Apply X (closes 47 other findings)`
   rather than 47 individual findings.
2. **Roots are grouped.** When multiple roots are independent (no
   subsumption relation between them), they're presented as parallel
   options the author chooses between.
3. **Leaves on root-rejection.** If the author rejects a root fix
   (e.g., declines the substrate rewrite as out-of-scope for the
   current change), the subsumed leaves surface individually as
   fallback. The author can fix locally if they reject the root.
4. **Subsumed-leaf preview.** Each root surface lists the subsumed
   leaves' line numbers as a preview so the author can verify the
   subsumption claim against the actual finding sites without
   running the lens suite themselves.

### 11.4 Canonical worked example — `src/v2/05_emit_rust.dag`

The file surfaces (will surface, once L1.5.b / L1.10.c / L1.10.d /
L1.13.b / L1.13.c / L1.11.b enforcement gates are met) on the order
of 100+ individual findings across six families. The subsumption
roots:
- **R-root-A**: "consume `rust.dag` LanguageModel via a grammar-driven
  serializer" — applying this single transformation subsumes the
  bulk of L1.10.d, L1.5.b, L1.10.c findings.
- **R-root-B**: "lift the serde-policy + auth-defaults + transport-
  fallback decision-trees into substrate table rows — `TotalMap<K, V>`
  for finite payload-free tables, `TotalPolicy<K, Context, RowTemplate>`
  for payload-bearing policies (per the L1.13.c payload-aware
  refinement)" — subsumes L1.13.c + L1.11.b findings. The
  serde-policy case is specifically a `TotalPolicy` shape (because
  `VariantNaming` includes payload-bearing constructors like
  `StripPrefixAndSnakeCase { prefix }`); the auth-defaults case may
  be `TotalMap` or `TotalPolicy` depending on whether the auth-source
  vocabulary has payload fields.
- **R-root-C**: "rewrite name-as-discriminant dispatch to use
  resolved `KeyVocabulary`" — subsumes L1.10.c findings not closed
  by R-root-A.

The diagnostic surface presents R-root-A first (highest leverage,
subsumes the most findings), with R-root-B and R-root-C as parallel
options. The substrate author chooses R-root-A; the 100+ leaf
findings clear automatically when the LanguageModel-driven serializer
lands.

### 11.5 What this gives you

- **The substrate author sees the actual minimal-edit-set**, not a
  pile of symptoms. The framework's value compounds: each new lens
  that fires alongside others increases the subsumption DAG's
  density, which makes the per-author surface SIMPLER, not more
  cluttered.
- **The framework is honest about leverage.** A lens that fires
  often but is always subsumed by a higher root never blocks an
  author — it's the root that matters. The lens itself stays valid
  (it catches the defect class) but the diagnostic surface respects
  the hierarchy.
- **Producer-stage authority is preserved.** Subsumption rows are
  substrate data verified mechanically. The framework doesn't
  guess; it reads.

### 11.6 Bootstrap

The first concrete `DissolutionSubsumption` row to land is the
`R-root-A` example above (rust.dag LanguageModel subsumes emit_rust
findings) — verified via `SubsumptionVerification::MechanicalReverification`
once v4's `05_emit.dag` consuming `rust.dag` is the canonical
substrate and the lens suite re-runs against that snapshot. Earlier
subsumption rows (during the v3→v4 transition) MAY use
`SubsumptionVerification::ProducerStageDerivation` when the
subsumption is structurally derivable from producer-stage `consumes:`
graphs without a full re-run.

## 12. Open — audit of current coverage

To be filled: an audit of which Layer-0 checks the v4 compiler enforces
today vs. the gap. The v4 compiler is early-stage (the pipeline is still
being modeled), so Layer 0 is expected to be a substantial current gap —
B1 should state that plainly once audited.

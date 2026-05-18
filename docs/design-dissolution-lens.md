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

Every dissolution finding violates one invariant, in one of two
directions:

> **A1 — Do not hand-roll a derived operation.** If a function's behavior
> is fixed entirely by the shape of a modeled type, it is re-deriving
> something the compiler already derives. *Inverse:* do not nominalize an
> operation as a type — an operation is a function, not a domain
> structure.

A1 is proposed, pending operator ratification (rework-tracker task A1).
Ratifying it makes "this violates A1" a citable hard rule. The lens suite
is how A1 is enforced *mechanically*.

## 2. Two tracks

The lens is not the cure — it is the net.

- **Track 1 — lenses (enforcement).** Catch hand-rolling at CI. This doc.
- **Track 2 — substrate-derivation (the cure).** Make hand-rolling
  *impossible*: the substrate derives an algebraic type's canonical
  operations (variant discriminants, catamorphism) from its declaration,
  so there is nothing left to hand-roll. The burn-down substrate nodes
  (the discriminant-predicate node; `fold_node` / catamorphism nodes)
  are Track 2.

Every dissolution finding is a symptom of one substrate gap: **the
substrate lets you *declare* an algebraic type but does not *derive its
canonical operations*** — so workers hand-roll them. Track 1 holds the
line until Track 2 closes that gap.

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

### L1.3 Hollow-type lens — kills *hollow declarations* (Practice 8's hollow-alias finding, at the type level)

- *Signature:* a declared type **nothing inhabits** — no `data` instance,
  no `fn` returning it, no alias-identity to a substrate type. (This is
  L0.4, unused-declaration, + "no inhabitance edge.")
- *Decidable:* yes — cross-reference whether anything constructs it.
- *Verdict:* hard error.
- *Escape:* a type that *is* constructed, or aliases a substrate carrier,
  passes.

### L1.4 Carrier-clone lens — kills *carrier dissolution*

- *Signature:* a locally-declared coproduct **structurally isomorphic to
  a `std/` carrier** (`Foo { value: T } | FooRejected { diagnostic }` ≅
  `Outcome<T>`).
- *Decidable:* yes — type-shape match against the `std/` carrier set.
- *Verdict:* hard error.
- *Escape:* a coproduct carrying a payload the std carrier genuinely
  cannot express passes.

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

### L1.6 Emit/template lens — kills *emit/template dissolution*

- *Signature:* a field or value that is a **template string literal** — a
  string literal carrying positional placeholders (`{0}`, `{1}`, …) used
  as an emitter, where grammar-as-declarative-bidirectional-data belongs.
- *Decidable:* yes — a literal string with interpolation placeholders is a
  structural match (the keystone decidability table already classifies it
  "structural — a literal template-string field").
- *Verdict:* hard error on the literal-template shape.
- *Escape:* a plain string constant with no placeholders, or genuine
  string *data* that is not an emitter template, passes.
- *Kills:* string-templated emitters (`template: "Vec<{0}>"`).

### L1.7 Off-substrate-fact lens — kills *prose-asserted facts*

- *Signature:* a declaration whose name or comment claims a closed-set
  structural fact (`inhabits <Algebra>`, `opaque`, `non-empty`, `Word<N>`,
  `Float<N>`, `bounded`, `non-forgeable`) without the corresponding
  machine-readable artifact: algebra witness data, recursive cardinality
  refinement, or constructor restriction.
- *Decidable:* yes — claim vocabulary is closed, and each required
  counterpart is a parsed-model fact. Width claims require recursive
  discharge through the carrier chain (`Word64` needs `Byte` and `Bit`
  cardinality, not only an outer field name).
- *Verdict:* hard error on `std/` / substrate files.
- *Escape:* rationale comments without fact-bearing tokens pass; a claim
  with the structural counterpart present passes.
- *Kills:* prose-only algebra inhabitance, width-in-the-name, and
  opacity-in-the-comment shapes.

### L1.8 Wrong-home lens — kills *orphan operations*

- *Signature:* a function or data declaration whose primary concept is
  structurally owned by another file. Primary concept is selected by:
  witness target first; same-type closure second; upstream argument and
  return convergence third. If no single primary concept is selected, the
  declaration is cross-cutting and the lens does not fire.
- *Decidable:* yes — witness membership, argument and return types, and
  import graph are parsed-model facts.
- *Verdict:* hard error in `std/` and `extdeps/`.
- *Escape:* genuinely cross-cutting bridge / coercion / display functions
  pass only through a closed-token operator-confirmed marker.
- *Kills:* operations such as `nat_compare(Nat, Nat)` living outside the
  `Nat` or algebra home.

### L1.9 Vacuous-arm lens — kills *exhaustive-but-empty match*

- *Signature:* a `match` on a coproduct where at least one arm returns a
  trivial literal of the function's return type while a sibling arm does
  structural work. The finding is asymmetric work over a closed set: the
  author named every variant but left one branch content-free.
- *Decidable:* yes — arm RHS shape is structural.
- *Verdict:* hard error in substrate files.
- *Escape:* a trivial arm may pass only with a closed-token structural
  justification (`variant-has-no-children`, `identity-on-Unit`, etc.).
- *Kills:* `node_locally_well_formed`-style arms that discharge an entire
  sibling variant as `true` while other siblings are actually checked.

### L1.10 String-escape-hatch lens — kills *typed-model bypass via String*

- *Signature:* a `String` field carrying a structural role tag whose role
  is registered by a `CanonicalCarrier<T>` row as superseded by a typed
  carrier in scope. This generalizes L1.6 from emitter templates to domain
  strings. `CanonicalCarrier<T>` is not introduced as a new standalone
  authority here; it names the future derived view over the existing substrate
  homes for role-bearing carriers, including `extdeps/process.dag` `Command`
  for command text.
- *Decidable:* yes — registry membership and role-tag refinements are
  parsed-model facts once that derived view exists. Until then, L1.10 is gated
  on the role-carrier registry task and must not be implemented by a hardcoded
  `command` / `path` / `url` name table.
- *Verdict:* hard error.
- *Escape:* untagged strings, or role tags with no canonical carrier row
  in scope, pass.
- *Kills:* `ShellCommand { command: String : command_role }` when
  `process.Command` is declared as the canonical command carrier.

### L1.11 Plausible-fallback lens — kills *fabricated sibling fallthrough*

- *Signature:* a missing-info arm (`None => Ctor`, `Empty => Ctor`,
  `[] => Ctor`) where `Ctor` is a constructor of the function's return
  type and the return type is not `Outcome<_>`.
- *Decidable:* yes — return type, constructor membership, and
  missing-info pattern are structural.
- *Verdict:* hard error. The fix is to return `Outcome<T>` and route the
  missing fact to `Rejected { diagnostic: ... }`.
- *Escape:* total defaulting helpers whose definition is exactly
  `None => default` require operator confirmation.
- *Kills:* "safe-looking" default constructors for unknown DELETE/PUT/PATCH
  effect shape derivations.

### L1.12 Parallel-authority lens — kills *duplicate concept homes*

- *Signature:* the same type name `T` introduced by `type T = ...` in two
  different `.dag` files without a structural alias / re-export,
  retirement-ledger row, or same-change migration deleting one home.
- *Decidable:* yes — duplicate declarations and structural alias/ledger
  rows are parsed-model facts. Comment markers do not count as authority.
- *Verdict:* hard error.
- *Escape:* structural alias / re-export, structural retirement record,
  or deletion plus consumer migration.
- *Kills:* duplicated `Bool`, `Char`, `Url`, and machine-word concept
  homes with no machine-readable canonical authority.

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
| 2026-05-18 | `src/v4/DECISIONS.md` Part 6 `SL-3229-LLVM-WIDTH` / `SL-3229-FLOAT-NOMINAL`; `docs/audit/dissolution-inventory.md` P1 | raw width / nominal-width facts carried by names or comments instead of bounded cardinality structure | facts live in substrate structure, not comments or names | L1.7 |
| 2026-05-18 | `src/v4/DECISIONS.md` `SL-P7-NAT-COMPARE-VPRED`; `docs/audit/dissolution-inventory.md` P6 | `nat_compare` originally homed under `std/float.dag` while the primary concept is `Nat` | operations live with their primary concept | L1.8 |
| 2026-05-18 | codex review on design commit `cfbc247c0`, resolved by `3fb3e4dfc`; live home: compiler-side local well-formedness predicates until migrated into `.dag`; dissolve trigger: reject or require closed-token justification for asymmetric trivial match arms in substrate files | exhaustive shape is not evidence of actual validation | L1.9 |
| 2026-05-18 | `src/v4/DECISIONS.md` `LB-P4-3213`; process `Command` target from #3209 | `CiCommand::ShellCommand { command: String }` while `extdeps/process.dag` models a typed `Command` | typed domain facts must not tunnel through strings | L1.10 |
| 2026-05-18 | `docs/design-effect-enumeration-resource-threading.md` §2.4 / §8.1; `docs/design-transport-taxonomy.md` `derive_effect_shape` migration | `None => CreateEffect`-style fabricated sibling fallback in effect-shape derivation | missing facts reject; they do not guess a plausible variant | L1.11 |
| 2026-05-18 | `src/v4/DECISIONS.md` D2 / D2-REV; `docs/briefs/t-ground-languagespec.md` parallel-authority dissolution | duplicated type homes without alias / retirement / migration | one concept has one structural authority | L1.12 |

Pattern across the ledger: the first four receipts are burn-down
*substrate* PRs — the lane built to remove dissolution debt produced it.
Later rows are checkable review / audit receipts for the same failure
classes before the full registry-backed lens runner exists. #3249's was
invisible to reviewers because the fold-laundering hid it. This is why the
lens suite (mechanical, every time) and the burn-down pre-gate (catch at
the source) both exist.

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

## 10. Open — audit of current coverage

To be filled: an audit of which Layer-0 checks the v4 compiler enforces
today vs. the gap. The v4 compiler is early-stage (the pipeline is still
being modeled), so Layer 0 is expected to be a substantial current gap —
B1 should state that plainly once audited.

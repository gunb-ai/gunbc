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
despite being a defect (§8). The lens's signature is the smallest
structural pattern that catches that finding's class with zero false
positives on the clean shape.

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
  - claimed cardinality / width → no refinement clause on the carrier
    (the `List<T> where len(_) == N` shape);
  - claimed opacity / non-forgeability → no constructor restriction
    (the type is a record whose fields are all freely constructible from
    user-reachable substrate values).
- *Decidable:* yes — the claim vocabulary is a closed set; the structural
  counterpart is locatable (data table, refinement clause, constructor
  visibility).
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

**Clean shape (the cure):**
```dag
type Word64 { bytes: List<Byte> where len(_) == 8 }
data fermi_lattice: Lattice<FermiDepth> = { meet: fermi_meet, join: fermi_join }
```

### L1.8 Wrong-home lens — kills *orphan operations* (proposed)

> **Status: proposed.** Derived from finding F5 (`nat_compare` defined in
> `src/v4/std/float.dag` rather than `src/v4/std/nat.dag` or
> `src/v4/std/algebra.dag`). Mechanizes MODELING M9 (DFS the concept
> DAG).

- *Signature:* a `fn f(x: T, ...) -> ...` where every argument's type is
  declared in file `X`, but `f` lives in file `Y`, and `Y` imports `X`
  (i.e. `X` is upstream of `Y`). Also: a `data` declaration whose primary
  key-type lives upstream of its file.
- *Decidable:* yes — the import graph and argument-type ownership are
  both queryable from the parsed model.
- *Verdict:* hard error in `std/` and `extdeps/`.
- *Escape:* operations whose argument types span multiple files with no
  single upstream owner (genuinely cross-cutting) — operator-confirm with
  an `// Anchor: cross-cutting` marker.

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

- *Signature:* a `match` on a coproduct where one arm's RHS is a trivial
  literal (`true`, `false`, `Unit`, the input itself, `None`) AND the
  function's name implies a predicate / validation / discipline
  (`*_well_formed`, `*_valid`, `*_locally_*`, `validates_*`, `is_legal_*`).
- *Decidable:* yes — arm body shape + function-name suffix vocabulary.
- *Verdict:* hard error in substrate files (where validation discipline
  must be uniform across a closed set).
- *Escape:* the trivial arm is genuinely the correct answer (e.g.
  `Unit => true` for a unit-typed thing), accompanied by an
  `// Anchor: trivially-true` marker pinning the justification.

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

### L1.10 String-escape-hatch lens — kills *typed-model bypass via String* (proposed, generalizes L1.6)

> **Status: proposed.** Derived from finding F6
> (`CiCommand::ShellCommand { command: String }` while
> `extdeps/process.dag` already models a typed
> `Command { program, argv0, args, env }`). L1.6 catches string templates
> as emitters; L1.10 catches strings as *carriers* for domains that have
> a typed model in scope.

- *Signature:* a record or variant field of type `String` whose field
  name (or surrounding variant name) matches a type declared in `std/` or
  `extdeps/` (closed name table: `command` → `process.Command`, `path` →
  `Path` / `AbsolutePath`, `url` → `Url`, `method` → `HttpMethod`, etc.).
- *Decidable:* yes — field-name to canonical-type-name lookup.
- *Verdict:* hard error.
- *Escape:* the field is named generically (`name`, `id`, `key`) without
  colliding with the canonical-name set, OR the typed model legitimately
  does not fit (operator-confirm).

**Concrete match — F6 (`src/v4/workflow/ci.dag:23-28`):**
```dag
type CiCommand
  = LintCommand
  | TestCommand
  | IgnoredTestCommand { test_name: String }
  | BootstrapStageCompile { produces: Symbol }
  | ShellCommand { command: String }    // ← `command` matches process.Command
```

**Clean shape:**
```dag
import v4.extdeps.process as process
type CiCommand = ... | ShellCommand { command: process.Command }
```

### L1.11 Plausible-fallback lens — kills *fabricated-sibling fallthrough* (proposed)

> **Status: proposed.** Derived from finding F10
> (`derive_effect_shape`'s `None => CreateEffect` arm for DELETE/PUT/PATCH
> when no path key exists). Sibling rule to P3 (fail-closed): the
> function returns a typed enum and the missing-info case returns a
> *different valid constructor* of that enum — a plausible guess —
> instead of escalating through `Outcome::Rejected`.

- *Signature:* a `match` arm of shape `None => Ctor` / `Empty => Ctor` /
  `[] => Ctor` where `Ctor` is a constructor of the function's return
  type AND the function's return type is **not** `Outcome<_>` (does not
  carry a diagnostic variant).
- *Decidable:* yes — return-type shape + arm-RHS constructor membership +
  matched-sub-pattern is a "nothing-here" variant.
- *Verdict:* hard error. The fix is to lift the return type to
  `Outcome<T>` and return `Rejected { diagnostic: DerivationUnknown }`.
- *Escape:* the missing-info case has a *uniquely correct* answer (e.g.
  `or_default(opt: Option<Nat>, default: Nat)` style helpers where
  `None => default` is the function's definition) — operator-confirm.

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

### L1.12 Parallel-authority lens — kills *unmarked duplicate concept homes* (proposed)

> **Status: proposed.** Derived from finding F9 (`Bool`, `Char`, `Url`,
> machine words declared in both `dsl/std/` and `src/v4/std/` with no
> marker indicating which is canonical) and the D2-resolver gap
> (`extdeps/languages/resolver.dag` is referenced as the planned home
> while a provisional `GroundingMap` sits in `extdeps/languages/rust.dag`).

- *Signature:* a type name `T` declared in two `.dag` files where neither
  carries an `// Authority: canonical` / `// Authority: historical`
  header marker. Or: a type name referenced via an import path under a
  file that does not exist (planned-but-absent home — the degenerate
  case of duplication).
- *Decidable:* yes — name uniqueness across the corpus + header marker
  check + import-target existence check.
- *Verdict:* hard error.
- *Escape:* one file carries `// Authority: canonical`, the other
  carries `// Authority: historical { dissolves_when: <trigger> }`.

**Concrete match — F9 (`dsl/std/types.dag:173` and `src/v4/std/logic.dag:14`):**
```dag
// dsl/std/types.dag:163-173   (legacy-scanner anchor, no authority designator)
// v3 Path A (Lane 1e-2b): `Bool` still parses here for the legacy scanner ...
type Bool = True | False

// src/v4/std/logic.dag:13-14   (dissolution classification, no authority designator)
// 🟢 coproduct dissolution — DECISIONS.md classification ledger: Bool.
type Bool = True | False
```

Both declarations *are* annotated, but neither annotation answers the
question this lens asks. The existing tags classify the *finding shape*
(dissolution status, scanner anchor) — neither names this file as the
canonical home or the other as historical. L1.12's required form is a
designator (`// Authority: canonical` / `// Authority: historical
{ dissolves_when: <trigger> }`) that picks a winner between the two
parallel declarations.

**Concrete match — D2-resolver (provisional + planned-absent):**
```dag
// src/v4/extdeps/languages/rust.dag
data GroundingMap = { ... }               // ← provisional home

// (src/v4/extdeps/languages/resolver.dag does not exist)
// other language files defer to a file that isn't there
```

**Clean shape:** mark one file canonical, the other historical with a
dissolution trigger; or land the planned resolver and migrate the
provisional copy.

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

| date | PR | finding | core invariant violated | lens |
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
| 2026-05-18 | ingest | `CiCommand::ShellCommand { command: String }` while `extdeps/process.dag` models a typed `Command` (`src/v4/workflow/ci.dag:23-28`) | String escape hatch for a domain that has a typed model in scope | L1.10 (proposed) |
| 2026-05-18 | ingest | `derive_effect_shape` `DELETE/PUT/PATCH None => CreateEffect` (`dsl/std/effects.dag:268-282`) | missing info must escalate through `Outcome::Rejected`, not return a different valid sibling | L1.11 (proposed) |
| 2026-05-18 | ingest | `Bool`, `Char`, `Url`, machine words declared in both `dsl/std/` and `src/v4/std/` with no authority marker; `extdeps/languages/resolver.dag` referenced but absent | parallel concept homes must be marked canonical / historical, or one must be retired | L1.12 (proposed) |

Pattern across the ledger: all four are burn-down *substrate* PRs — the
lane built to remove dissolution debt produced it. Each was *mostly*
correct with one dissolution defect; #3249's was invisible to reviewers
because the fold-laundering hid it. This is why the lens suite (mechanical,
every time) and the burn-down pre-gate (catch at the source) both exist.

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

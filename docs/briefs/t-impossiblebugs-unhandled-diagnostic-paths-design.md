# T-ImpossibleBugs — unhandled diagnostic paths (DESIGN/SCOPING doc)

> Output of [`t-impossiblebugs-unhandled-diagnostic-paths-worker.md`](t-impossiblebugs-unhandled-diagnostic-paths-worker.md)
> per the 2026-04-25 reframe (post-`sunny-deer-629` STOP-AND-ESCALATE).
> Doc-only artifact. No v3 substrate change in this PR.
>
> **Recommendation: (a) bypass-feasible — totality-by-omission, the
> pattern gunbc already uses for `force_unwrap`. Reasoning + sequencing
> below.**

## 1. DB-11 interaction analysis

The original brief framed proof-or-totality as *"attach a `where b != 0`
refinement on `b` and the type-checker honors it as a proof for `a / b`."*
That framing directly contradicts the design DB-11 already shipped.

### Evidence at HEAD

`src/v3/compiler/src/infer.rs:3935-` (function start; ~80 lines) — `resolve_operator_arrow` is the
operator dispatch site. Lines 3940-3950 carry an explicit comment block:

```rust
// DB-11 (3a.3) operand normalization. Primitive operators (`+`,
// `>`, `!=`, etc.) are structurally over BASE types — refinements
// are surface-level facts about values, not part of the operator's
// arrow contract. Mirroring a refined lhs like `Int where d != 0`
// onto both operand positions (as the old fallback did) made the
// call-site refinement-discharge pass treat the refinement as a
// real requirement on every operand — so a literal `10` in
// `d > 10` failed discharge because literals carry no refinement.
// Strip refinements once up front; algebra-Conj walks and the
// primitive fallback both operate on the base.
let source_id = strip_refinement_to_base(dag, lhs_type.declaration);
```

The strip is *deliberate*. It is the designed-in fix for a prior failure
mode where mirrored refinements broke symmetric operators (`d > 10`
rejected the literal `10`). DB-11's discharge pass operates downstream of
the strip, which is why the comment lands here.

DB-11's discharge semantics are **structural identity** of refined types,
not **logical entailment**. The locked test surface
(`src/v3/compiler/tests/integration/m2_feature_parity_test.rs:331-700`,
sixteen `test_3a3_*` cases) validates:

- refined parameter ↔ refined argument structural-identity match (line 603);
- distinct refinements do *not* auto-unify (line 619);
- if-arm narrowing produces arm-local refined declarations matched
  structurally (line 655);
- literal `0` in `div(10, 0)` is rejected because literals carry no
  refinement (line 578).

None of that is logical entailment. Asking *"does `b != 0` entail
`denominator != 0`?"* is a different operation against a different
substrate, not present today.

### Why the brief's original framing fights DB-11

To make `a / b` require `b: Int where b != 0`, three things must
simultaneously be true that are not true today:

1. **Operator dispatch must read refinements on the precondition operand**,
   contradicting the strip at line 3950.
2. **The contradiction must be asymmetric** — `>` continues to strip
   refinements (or `d > 10` regresses), while `/` does not strip on the
   denominator slot. Per-operator per-operand refinement-honoring
   policy.
3. **The check must compare predicates by entailment** — the user's
   refinement predicate must logically imply the operator-declared
   precondition predicate, not just match it structurally. DB-11 only
   does structural match.

That's the load on the original framing. The "ownership_lens" cited as
precedent does not bear it: `src/v3/lenses/named_function_count.dag:10-25`
+ `src/v3/std/verification.dag:137-141` + `src/v3/compiler/src/test_runner.rs:1425,1555`
show ownership_lens is a post-hoc observability lens that consumes a
lowered DAG and asserts a count via `LensOutputEquals`. It does not
gate type-checking, and it carries no proof obligation. The shape (decl
ref + input ref + expected ref) does not generalize to a type-time
proof carrier.

## 2. Substrate proposal — what the in-place-proof path would actually cost

If Director chooses to enforce proof-or-totality *in place* (i.e., keep
bare `a / b` as written by the user, but reject it when `b` is an
unrefined `Int`), the substrate work is:

1. **Per-operator partiality fact** — for each partial operator, name
   which operand carries the precondition and the precondition predicate
   itself. Attaches to operator declarations in `dsl/std/algebra.dag`
   (`OrderedRing.div` / `FreeMonoid.index` / etc.) or a sibling registry. New
   carrier; no current home.

2. **Predicate-entailment check** — given the user's refinement
   predicate on the precondition operand and the operator's declared
   precondition predicate, decide whether the former entails the latter.
   For `b != 0` ⊨ `denominator != 0` that's a syntactic alpha-rename;
   for arbitrary user predicates it's general first-order entailment.
   No existing infrastructure. DB-11's structural-identity check is
   strictly weaker.

3. **Asymmetric per-operand refinement-honoring at dispatch** — modify
   `resolve_operator_arrow` so the strip-to-base happens per-operand,
   driven by the partiality fact. The denominator slot of `/` retains
   its refinement; the operand slots of `>` continue to strip. This
   directly reopens the failure mode the strip was added to close, so
   the per-operand policy must encode *why* `/` is asymmetric and `>`
   is not. That encoding is the partiality fact from item 1.

Items 1 and 3 are mutually entangled — the partiality fact is what
drives the asymmetric strip — but they are still distinct work. Item 2
is the largest and is independent: any proof-or-totality system that
tries to honor user-attached refinements as proofs needs an entailment
checker, and gunbc has none.

This is the work behind THESIS:391's *"Gated on Tier 2 substrate
(post-R1)"* phrasing in the [R2+] enumerable-bug-classes list. The
gate is real. (Note on anchors: THESIS:391-393 names the narrower
R2 gate covering division-by-zero, OOB, and force-unwrap; the
broader Tier 2 commitment at THESIS:175 also includes integer
overflow and partial functions. The brief cites THESIS:391 for the
R2 gate and THESIS:175 for the broader "made total" branch
referenced in §3.)

## 3. Bypass investigation — is there a totality-only path?

THESIS:175 (Tier 2 — Runtime safety) reads: *"Division by zero, integer overflow, out-of-bounds,
force-unwrap, partial functions — either proven safe at compile time
**or made total**. No partial functions in the runtime."*

The "or made total" branch closes the bug class without any of the
section-2 substrate. And gunbc already uses this branch — by omission,
not by parallel surface.

### Evidence: force_unwrap is closed today by not existing

`grep -rn "force_unwrap" src/v3/std/ dsl/std/` returns nothing —
the partial form is simply absent from the gunbc surface. Closure
is by absence, not by a typed std Option API. (`dsl/std/languages.dag:322,325,1026`
declares `NullCoalesceStrategy` emit-time templates for how each
target language renders null-coalescing — Rust uses
`{lhs}.unwrap_or_else(|| {rhs})`, Python uses a ternary, etc. —
but those are per-target rendering carriers, not a gunbc-level
total Option API; any typed std Option API for the total form is
itself follow-on work.) The bug class *force-unwrap on None* is
closed because the partial form is unexpressible at the gunbc
surface, not because a paired total replacement ships. No proof
obligation, no entailment, no asymmetric dispatch.

This is the load-bearing pattern. It's already the gunbc convention;
the unhandled-diagnostic-paths lane should follow it rather than invent
a parallel proof system.

### Applied to the THESIS:175 / THESIS:391 enumeration

| Bug class | Totality-by-omission shape |
|---|---|
| force-unwrap on None | Already done — partial form not in std/. |
| Out-of-bounds indexing | **Declared partial; reachability to audit** — `dsl/std/algebra.dag:305` declares `index: fn(Int) -> T` on FreeMonoid (returns bare `T`), but the user surface does not appear to expose square-bracket indexing or a callable-access path that resolves to this field today (no `Index` variant in `src/v3/compiler/operators.dag`; no `[i]` syntax found in surface tests). Audit step (slice §4.1) must demonstrate concrete reachability before scoping a removal sub-lane. If reachable, closure shape: retype to `index: fn(Int) -> T?` (or `Result<T, IndexOutOfBounds>`); Map's `get: fn(K) -> V?` at `:340` is already total and is the model shape. If unreachable today, the partial declaration may still warrant retyping prophylactically before any future surface adds. |
| Division by zero | **Closure target updated 2026-04-25 per codex review at sha `e41297cd`**: algebra.dag now declares `OrderedRing.div: fn(T, T) -> T` at `dsl/std/algebra.dag:182` (added in a sibling lane since the original brief was authored). `Int` resolves to `OrderedRing<Word64>` via the alias chain `dsl/std/integer.dag:43` (`type Int = Int64`) → `:34` (`type Int64 = OrderedRing<Word64>`); operator dispatch consumes this chain via `TypeConnective::Atom(ResolvedBy*)` traversal at `src/v3/compiler/src/infer.rs:3975-3977`, so `Int / Int` dispatch resolves through the algebra-Conj walk to `OrderedRing.div`. (Note: `kernel_algebra_profile` at `dsl/std/algebra.dag:459` is consumed by `cardinality_lens`/`complexity_lens` per `dsl/std/computation.dag:437`, not by operator dispatch.) `src/v3/compiler/operators.dag:53` (`Div => "div"`) matches this field by name. The primitive scaffold at `src/v3/compiler/src/infer.rs:4003-4015` is still present as a fallback for types whose walk doesn't terminate at an algebra Conj, but for `Int` it is no longer the dispatch path. **Updated closure shapes**: **(i) Algebra retype + per-target realization migration** — change `OrderedRing.div`'s return type at `algebra.dag:182` from `fn(T, T) -> T` to `fn(T, T) -> Result<T, DivideByZero>` (or `Option<T>`), AND migrate each per-target `OperatorRealization` carrier keyed on `OrderedRing.div`. At HEAD: `src/v3/spec/rust.dag:816` (`rust_int_div`) and `src/v3/spec/go.dag:742` (`go_int_div`) render `carrier: "({lhs} / {rhs})"` (bare division). `src/v3/spec/python.dag:486` (`python_int_div`) renders `carrier: "(__v3_idiv({lhs}, {rhs}))"` — Python uses a helper because `//` semantics differ from Rust/Go integer division for negative operands; the helper is emitted by `src/v3/compiler/src/emit/python_target.rs:680` and pinned by `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs:108-109`. After the algebra retype, each carrier must emit a Result/Option construction matching the new return shape (e.g., Rust: `if rhs == 0 { Err(DivideByZero) } else { Ok(lhs / rhs) }`; idiomatic per-target). This is **not** a single-line change — algebra retype is one line, realization migration is one row per supported target plus any tests pinning the old emission. The follow-on implementation brief must include the realization migration explicitly; emit otherwise produces type-mismatched code or silently-partial division under a total type. **(ii) Audit the primitive fallback** at `infer.rs:4004` for any types that still resolve through it for `Arithmetic(Div)` (e.g., types whose `inhabits` chain doesn't reach OrderedRing/Field). If any partial-Div paths remain, retype the fallback for `Arithmetic(Div)` accordingly or remove the fallback for that case. **(iii) Non-operator total function** `fn divide_safe(a: Int, b: Int) -> Result<Int, DivideByZero>` — expressible today, but if (i) is not also done, bare `a / b` still type-checks via the algebra-Conj path; pairing without (i) is paired-not-closed (the acceptance-theatre trap). Closure of the bug class requires (i); (ii) is the audit completing it; (iii) alone is insufficient. The NonZeroInt-typed-denominator shape remains a separate concern: `(Int, NonZeroInt) -> Int` is asymmetric per-operand and is not expressible as the `/` operator today (every algebra-operator-decl uses symmetric per-operand types via `T,T -> T` arrow shapes); requires either extending the algebra-operator carrier for per-operand type variance or accepting a non-operator function shape `fn divide_nz(a: Int, b: NonZeroInt) -> Int`. |
| Integer overflow | Two valid totalities: (i) wrap-by-design with explicit `WrappingInt` carrier (totality via documented modular arithmetic, no failure case), or (ii) checked ops returning `Result<Int, IntOverflow>`. Either, not both, on the same operator. |

Each entry above closes its bug class by *making the partial form
unwriteable*, not by adding a proof obligation alongside it. That's the
"or made total" branch literally.

### Acceptance-theatre risk explicitly flagged

The risk in the brief's req 3 is real: writing a `divide_safe(a, b) ->
Result<Int, DivideByZero>` wrapper *alongside* an unchanged `/`
operator does **not** close the bug class. Both forms remain
expressible; the partial one continues to type-check. This is theatre.

The bug class is closed only when the partial form becomes
unexpressible — i.e., the surface-language change is *removal* of `/`
in its `(Int, Int) -> Int` shape, not *addition* of a Result-returning
sibling. That's a language-design decision Director owns, not a
substrate decision.

### What about *proof-mode* uses?

For programmers who can statically prove `b != 0` (e.g., `b = denom *
denom + 1`), the totality-shifted form is ergonomic noise — a Result
they always discard. Two existing-language patterns address this
without entailment substrate:

- `NonZeroInt` smart constructor — programmer constructs once at the
  proof site, uses the proof token thereafter. Same shape as Rust's
  `NonZeroU32`. The "proof" is the *constructor call site*, dischargeable
  by `match` / `unwrap_or_else`. No entailment check needed; the type
  carries the discharge.
- Pattern-match destructuring against a checked constructor — e.g.
  `match NonZeroInt::new(b) { Some(nz) => divide_nz(a, nz), None =>
  ... }` where `NonZeroInt::new: fn(Int) -> Option<NonZeroInt>` is
  the smart constructor that *checks* `b != 0` and returns `None`
  otherwise, and `divide_nz: fn(Int, NonZeroInt) -> Int` is a
  non-operator total function. The `nz` binding is structurally
  `NonZeroInt`, not a wrapped raw `Int`, so the type carries the
  proof. (A generic `Option::from(b)` wrapper would NOT close the
  bug class — it can produce `Some(0)` and the `Some` arm still
  admits division by zero. The constructor must be checked.)
  **Caveat**: writing this as `a / nz` instead of `divide_nz(a, nz)`
  would require asymmetric per-operand operator signatures (see §3
  Division-by-zero row); not expressible today without substrate
  extension to `AlgebraOperatorDecl`.

Neither requires DB-11 entailment. Both are pure
algebra/sum-totality work in `dsl/std/`. The non-operator function
form (`divide_nz`) sidesteps the asymmetric-operator-signature
question entirely.

## 4. Director-actionable recommendation

**(a) Bypass-feasible — totality-by-omission, sequenced per partial-op
class.**

Reasoning:

- The "made total" branch of THESIS:175 closes the bug class without
  the section-2 substrate. The substrate work in section 2 is genuine
  M+ (predicate-entailment is a major addition; the asymmetric strip
  reopens an explicitly-closed DB-11 design).
- gunbc already uses totality-by-omission for `force_unwrap`. Following
  the existing convention is cheaper than inventing a parallel
  enforcement system.
- For `/` specifically, the closure cost has **dropped since the
  original brief authored**: `dsl/std/algebra.dag:182` now
  declares `OrderedRing.div: fn(T, T) -> T`, and `Int` resolves
  to `OrderedRing<Word64>` via the alias chain
  `integer.dag:43 → :34` (consumed by the dispatch walk at
  `infer.rs:3975-3977`), so `Int / Int` dispatches through the
  algebra-Conj walk to `OrderedRing.div` (matching
  `operators.dag:53`'s `Div => "div"` mapping). The closure is now
  an **algebra retype plus per-target realization migration**:
  (1) change `OrderedRing.div`'s return at `algebra.dag:182` to
  `Result<T, DivideByZero>` (or `Option<T>`); (2) migrate each
  per-target `OperatorRealization` keyed on `OrderedRing.div`
  (`src/v3/spec/rust.dag:816` `rust_int_div`, `go.dag:742`
  `go_int_div` — both render bare `({lhs} / {rhs})`;
  `python.dag:486` `python_int_div` — renders
  `(__v3_idiv({lhs}, {rhs}))` via a helper at
  `python_target.rs:680` pinned by
  `m1_4_emit_python_test.rs:108-109`) to construct the new return
  shape idiomatically per target. Algebra retype is one line;
  realization migration is one row per supported target plus any
  tests pinning the prior emission. Without the realization
  migration, emit produces type-mismatched or silently-partial
  code under a total type. The Rust-side primitive scaffold at
  `infer.rs:4003-4015` remains as a general fallback for types
  whose `inhabits` chain doesn't reach an algebra Conj that
  declares the requested field; the follow-on audit (slice step
  1) should verify whether any types still resolve `Arithmetic(Div)`
  through that fallback and close those paths separately. Adding
  `divide_safe` as a non-operator function alongside an unchanged
  `OrderedRing.div: fn(T,T) -> T` is paired-not-closed (the
  acceptance-theatre trap); closure requires the algebra retype.
  The asymmetric NonZeroInt-typed-denominator shape remains a
  separate substrate question (per-operand type variance in the
  algebra-operator carrier) deferred to its own brief.
- The proof-mode ergonomic concern is handled by smart-constructor /
  match patterns, not by predicate-entailment.

This recommendation is **not** the same as the narrow-demo theatre. The
distinguishing factor is whether the partial form is *removed* from the
surface (real closure) or merely *paired with a total alternative*
(theatre). Removal closes; coexistence does not.

### Follow-on brief shape

**Implementation brief: T-ImpossibleBugs — totality-by-omission for
THESIS:175 / THESIS:391 partial-ops (per-class sub-lanes).**

Slice:

1. **Audit**: enumerate every partial operator/function currently
   reachable from user code (does `[i]` exist? does `force_unwrap`
   exist? does `Int -> Int` `/` exist? **also: are
   `OrderedRing.quotient` / `OrderedRing.remainder` at
   `dsl/std/algebra.dag:477-478` user-reachable as partial forms,
   distinct from the `/` operator? If so they are separate sub-lane
   targets**). Output: a table with one row per partial form on
   each std collection / numeric type + current totality status.
2. **Per-row decision**: for each partial form still reachable, pick
   either Result-shape, Option-shape, or NonZero-typed-input shape.
   Director-callable on shape choice; worker proposes default. **For
   any row that picks NonZero-typed-input as the operator-syntax
   shape (e.g., `a / nz` rather than `divide_nz(a, nz)`)**: the
   row STOPs and escalates — the asymmetric per-operand
   operator-signature question is deferred to a separate substrate
   brief (per-operand type variance in the algebra-operator
   carrier; see §3 Division-by-zero row caveat). Non-operator
   function shape (`fn divide_nz(a: Int, b: NonZeroInt) -> Int`)
   is always available without escalation.
3. **Removal sub-lane(s)**: one PR per partial-op class with
   - the partial-form removal (or retype),
   - the total replacement (if a new shape is needed),
   - regression tests demonstrating the partial form is no longer
     accepted at the surface (the specific diagnostic class depends
     on the closure shape — see acceptance criterion below),
   - migration of any in-tree callers to the total form.
4. **Acceptance**: for each closed class, a test asserting the
   partial form produces a structured diagnostic at the surface
   matched to the closure shape, AND the total form compiles. The
   appropriate diagnostic varies:
   - **Removal of the partial form** (e.g., remove the `[i]`
     indexing surface) → `ResolveError` or parse error.
   - **Return-retype** (e.g., `OrderedRing.div: fn(T,T) -> Result<T,
     DivideByZero>`) → `TypeMismatch` at any stale bare site
     (Result<Int> ≠ Int) — the operator still parses and
     dispatches; the partiality manifests as a type-shape mismatch
     against any caller assuming the old return.
   - **Total-only retype** (e.g., `index: fn(Int) -> T?`) →
     `TypeMismatch` at any stale site that consumed the bare `T`
     directly.
   The acceptance test must name the expected diagnostic class for
   the chosen shape; "ResolveError or parse error" alone is too
   narrow for return-retype closures.

This sequencing avoids:

- net-new substrate (no partiality fact carrier, no entailment check,
  no asymmetric strip);
- any DB-11 conflict (operator dispatch unchanged; refinements
  continue to strip);
- acceptance theatre (the partial form is removed, not paired).

It is **not** a single-PR demo. Each partial-op class is its own
removal sub-lane, sized by the migration cost of in-tree callers. `/`
is likely the largest because integer division is widespread; force-
unwrap is already done.

### When to revisit section-2 substrate

If a future feature demands *in-place proof acceptance* — e.g., a
verified-arithmetic mode where `b * b + 1` should let `a / (b*b+1)`
compile because the denominator is provably nonzero — that's the
right time to author the predicate-entailment + per-operator partiality
+ asymmetric-strip substrate. It is a Tier 2 R2+ extension, not a
prerequisite for the bug-class closure THESIS:175 / THESIS:391 promises.

## Receipts

- DB-11 strip site: `src/v3/compiler/src/infer.rs:3940-3950`
  (and `strip_refinement_to_base` at `:4032`).
- DB-11 test surface: `src/v3/compiler/tests/integration/m2_feature_parity_test.rs:331-700`.
- ownership_lens precedent (cited in original brief, ruled shape-only
  here): `src/v3/lenses/named_function_count.dag:10-25`,
  `src/v3/std/verification.dag:137-141`,
  `src/v3/compiler/tests/t_demo/t_demo_fixtures.dag:87-97`,
  `src/v3/compiler/src/test_runner.rs:1425,1555`.
- Totality-by-omission precedent: `force_unwrap` is absent from
  the gunbc surface (`grep -rn "force_unwrap" src/v3/std/ dsl/std/`
  returns nothing). `dsl/std/languages.dag:322,325,1026` shows
  per-target `NullCoalesceStrategy` emit templates (Rust:
  `unwrap_or_else`, Python: ternary, etc.) — those are
  rendering-time carriers, not a typed gunbc Option API; any
  total Option surface is itself follow-on work.
- DiagnosticKind taxonomy:
  `src/v3/std/verification.dag:29-35`.
- Operator declaration surface: `dsl/std/algebra.dag:196,305,379`.
- Partial-form audit at HEAD: `dsl/std/algebra.dag:305` (FreeMonoid.index — partial), `:340` (Map.get — total via `V?`).
- `/` dispatch path at HEAD: `src/v3/compiler/operators.dag:53` maps `Div => "div"`; `dsl/std/algebra.dag:182` declares `OrderedRing.div: fn(T, T) -> T` (added in a sibling lane since the original brief was authored); the alias chain `dsl/std/integer.dag:43` (`type Int = Int64`) → `:34` (`type Int64 = OrderedRing<Word64>`) is consumed by the dispatch walk at `src/v3/compiler/src/infer.rs:3975-3977` (`TypeConnective::Atom(ResolvedBy*)` traversal), so `Int / Int` dispatches through the algebra-Conj walk to `OrderedRing.div`. The Rust-side primitive scaffold at `src/v3/compiler/src/infer.rs:4003-4015` remains as a fallback for types whose walk doesn't terminate at an algebra Conj declaring the requested field. Closure of bare `/` for `Int` requires both the algebra retype at `algebra.dag:182` AND migration of the per-target `OperatorRealization` carriers keyed on `OrderedRing.div` (`src/v3/spec/rust.dag:816`, `go.dag:742` render bare `({lhs} / {rhs})`; `src/v3/spec/python.dag:486` renders `(__v3_idiv({lhs}, {rhs}))` via the helper at `python_target.rs:680` pinned by `m1_4_emit_python_test.rs:108-109`) — under a total return shape, every carrier (including the Python helper) needs migration to construct the new return shape.
- THESIS gates: `THESIS.md:175` (broad Tier 2 commitment — div / overflow / OOB / force-unwrap / partial functions, "proven safe or made total"); `THESIS.md:391-393` (narrower [R2+] enumerable-bug-classes gate — div / OOB / force-unwrap, "Gated on Tier 2 substrate (post-R1)").
- DB-11 history: `docs/db-history/db-11.md`.

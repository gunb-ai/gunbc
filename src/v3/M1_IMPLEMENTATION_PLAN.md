# M1(2.6+) Implementation Plan — Upfront Enumeration

**Baseline:** `free-cod-972` as of round-6 + dissolution work (current HEAD
at time of writing).
**Status:** plan-of-record for M1(2.6) tail + M1(3) + M1(4).
**Enforced by:** `INVARIANTS.md` §"No short-term solutions," §"No bridges,"
§"No deprecations," and the enumeration discipline codified in
`feedback_enumerate_before_substrate.md`.

---

## §1. Why this document exists

Every reviewer round on `free-cod-972` has found one overloaded field where
a downstream stage reconstructs a fact the upstream stage did not represent
structurally. The pattern is consistent:

1. Upstream writes a field `X`
2. Downstream needs to distinguish two cases from `X`
3. Downstream reconstructs via name inspection / string match / lifecycle-
   state inference
4. Reviewer flags it as a bridge
5. Round fixes one field
6. Next round finds the next overloaded field
7. Go to step 1

This document breaks the loop. Its purpose is to **enumerate every
structural distinction downstream consumers need before writing any more
downstream code**. Gaps become pre-identified substrate work, not post-
implementation bridges. No consumer writes code until the substrate
answers its questions structurally.

**Per `INVARIANTS.md` §"No short-term solutions":** PRs under this plan
are deliberately larger than industry default. See §6 below for rationale.
The short version: bridges appear at layer boundaries, and a PR that can
only see one layer at a time will invent a provisional shape that the
next layer has to reconstruct. The antidote is atomic PRs that touch
substrate + every affected consumer + their shared reading patterns in
one push.

---

## §2. Baseline: `free-cod-972` current state

**Substrate shape** (after round-6 + latest dissolutions):

- `TypeConnective`: 6 variants (Atom, Conj, Disj, Arrow, Cardinality, Instantiation)
- `AtomPayload`: `UnresolvedIdentifier(String) | ResolvedIdentifier(DeclarationId) | TypeParam(String) | Literal(LiteralBits)` — phase coproduct dissolved in the last round
- `ArrowBody`: `UserDefined(NodeId) | ExternalRealization(DeclarationId) | Pending` — 3 variants, Pending restricted (per thesis §Q7) to primitive realization lag
- `CardinalityBound`: `Exact(u32) | AtMostOne | Unbounded`
- `Declaration`: `meta_tag` + `inhabits` split (value-construction vs algebra-inhabitance)
- `DeclarationId` + `NodeId` distinct newtypes
- Five L1 behaviors unchanged since M0 (Value, Transform, Branch, Loop, Bind)

**Test status:** 51 green (40 M0 acceptance + 4 M1 substrate + 7 real_stdlib
parse smoke), clippy clean, all real `dsl/std/*.dag` bootstrap files
parsing.

**Known open structural concerns** from the latest review round:

1. Operator dispatch threads through `UnresolvedIdentifier("+")` — reuses
   the unresolved-identifier shape as an operator-token sentinel
2. `lower.rs` descent check still does `target == "-"` string match
3. Operator knowledge scattered across `operators.rs`, `infer::is_comparison_operator`,
   `lower::is_strictly_smaller`
4. `TransformNode.target: DeclarationId` has no type-level distinction
   between callable Arrow, operator token, and placeholder

---

## §3. Downstream consumer enumeration

For each consumer that reads the substrate, list every structural question
it asks. Mark each:

- **S** = answered structurally by current substrate
- **R** = currently reconstructed (bridge risk; must be fixed before next
  downstream feature depends on it)
- **N** = not yet asked (future work; enumerate here so the substrate answer
  is pre-committed before the consumer is written)

### §3.1 `lens_provenance.rs` (76 lines, working)

| # | Question | Status |
|---|---|---|
| 1 | What is a Port's `produced_by` NodeId? | S |
| 2 | What Behavior variant is the producer? | S |
| 3 | Is there a producer at all (leaf port)? | S |

All S. Zero reconstruction. v3 success-bar proof point #1.

### §3.2 `lens_depth.rs` (74 lines, working)

| # | Question | Status |
|---|---|---|
| 1 | What is a Port's `produced_by` NodeId? | S |
| 2 | What Behavior variant is a Node? | S |
| 3 | For Transform: input Ports? | S |
| 4 | For Branch: condition Port + path output Ports? | S |
| 5 | For Loop: source + init Ports? | S |
| 6 | For Bind: value Port? | S |

All S. Zero reconstruction. v3 success-bar proof point #2.

### §3.3 `infer.rs` (653 lines, mostly working — 3 open R-rows)

| # | Question | Status | Notes |
|---|---|---|---|
| 1 | What's a Port's state (Uninferred/Resolved/Unresolved)? | S | `Port::state()` |
| 2 | What's a Node's Behavior variant? | S | `dag.node()` pattern match |
| 3 | What's a Declaration's TypeConnective? | S | `decl.connective` pattern match |
| 4 | For Arrow: inputs/output? | S | `TypeConnective::Arrow { .. }` |
| 5 | For Arrow: body kind? | S | `ArrowBody` 3 variants |
| 6 | Is a Bind's value Port resolved? | S | Port state check |
| 7 | **Is a Transform target callable, operator, or unresolved?** | **R** | Reconstructed via `unresolved_operator_name(decl)` — string match on `UnresolvedIdentifier.name` against `OPERATOR_FIELD_MAP` |
| 8 | **What kind of operator is this (arithmetic vs comparison)?** | **R** | Reconstructed via `is_comparison_operator(name)` — separate string match |
| 9 | **What's the operator's return type semantics (returns T or Bool)?** | **R** | Hard-coded dispatch: if comparison → Bool, else → T from inhabitance walk |
| 10 | What DeclarationId does an Identifier resolve to? | S | `ResolvedIdentifier(DeclarationId)` carries it |
| 11 | What does a DeclarationId map to at port-level? | S | `walk_to_type_shape` returns `TypeShape::new(current)` for named decls |
| 12 | For Instantiation: template + arguments? | S | `Instantiation { template, arguments }` |
| 13 | Walk substitution context? | S | `SubstStack` |

**Three R-rows** — all related to operator dispatch.

### §3.4 Descent evidence (currently in `lower.rs`, `is_strictly_smaller`)

| # | Question | Status | Notes |
|---|---|---|---|
| 1 | **Is this call operating on a structurally smaller sub-value?** | **R** | Walks SurfaceExpr, matches `target == "-"` + constant pattern |
| 2 | **What's the "smaller-by-1" pattern for each operator?** | **R** | Hard-coded for subtraction only; lexicographic / structural descent not supported |

**Two R-rows** — descent evidence reconstruction. Facts known at parse
time (the typed operator kind + the constant literal), dropped at lowering,
reconstructed by re-reading SurfaceExpr.

### §3.5 `lens_cost.rs` (future, M1(3))

| # | Question | Status | Notes |
|---|---|---|---|
| 1 | What's a Node's per-op cost? | N | Needs cost-per-primitive declared somewhere |
| 2 | What's a Transform's target cost? | N | Depends on callable-vs-operator typing (R-row #3.3.7) |
| 3 | How do costs compose across Behavior kinds? | N | Sequence / Loop / Branch composition rules |
| 4 | **Where does the lens STORE its results?** | N | Deferred from M0 — first writer lens forces the decision |
| 5 | For ExternalRealization: target-world cost? | N | Blocked on language spec declaring realization costs per primitive |

**Five N-rows.** Rows 1–3 are blocked on §3.3 R-rows being closed. Row 4
is a substrate decision that applies to every future writer lens.

### §3.6 Rust emitter skeleton (future, M1(4))

| # | Question | Status | Notes |
|---|---|---|---|
| 1 | How does each TypeConnective project to Rust syntax? | N | Reads language spec `dsl/extdeps/languages/rust.dag` |
| 2 | For Arrow UserDefined: emit sub-DAG as Rust fn body | N | Walks computation substrate |
| 3 | For Arrow ExternalRealization: emit target-language binding | N | Reads realization declaration |
| 4 | For Arrow Pending: fail-closed | N | Enforced by §8.11 ratchet; must not reach emission |
| 5 | For Instantiation: substitute template args, emit specialization | N | Lazy substitution via SubstStack |
| 6 | Ownership: which fields get `Rc`, which get moves? | N | Ownership lens — deferred to M1(5)+ |

**Six N-rows.** Rows 1, 3 depend on §3.3 R-rows and on parser support for
`realization { ... }` record literals. Row 6 is orthogonal (different
lens, different milestone).

### §3.7 Interpreter (future, later)

| # | Question | Status | Notes |
|---|---|---|---|
| 1 | For each Node: evaluate its value given input Port bindings | N | Tree walker over L1 behaviors |
| 2 | For Transform over primitive: call host primitive | N | Reads ExternalRealization for runtime binding |
| 3 | For Transform over user function: recursively evaluate sub-DAG | N | Same substrate as emission |
| 4 | Termination: is the current walk bounded? | N | Reads descent evidence (blocked on §3.4) |

**Four N-rows.** All depend on §3.3 + §3.4 R-rows being closed.

---

## §4. Gap analysis

**Open R-rows (reconstructive facts currently in the codebase):** 5

1. Transform target kind (callable/operator/unresolved) — §3.3
2. Operator kind (arithmetic/comparison) — §3.3
3. Operator return type semantics — §3.3
4. "Is this a smaller sub-value?" (descent evidence) — §3.4
5. "What's the smaller-by-1 pattern?" (descent shrink factor) — §3.4

**Open N-rows (future consumer questions):** 15 (5 cost, 6 emit, 4 interp)

**Dependency analysis of N-rows on R-rows:**

- All 5 cost lens questions depend on R-rows 1–3 (cost lookup needs typed
  operator kinds; target cost needs typed call kind)
- Emit rows 1, 3 depend on R-rows 1–3 (typed dispatch at emission)
- All 4 interpreter questions depend on R-rows 1–5 (interpreting operators
  + descent)

**Conclusion:** closing the 5 R-rows in one PR (PR-A below) unblocks 11 of
15 future consumer questions. The remaining 4 (cost lens storage mechanism,
emission ownership, emission language spec reading, interpreter) are
independent substrate additions addressed in PR-B.

---

## §5. Atomic work units

Two large PRs. Each updates substrate + every affected consumer
simultaneously. No intermediate states, no bridges.

### §5.1 PR-A: Structural operator handling (est. 8–12 hours)

**Purpose:** close all 5 R-rows by lifting operator knowledge out of
string-matching and descent-evidence reconstruction into structural types.
Consolidate operator knowledge into a single authority.

**Substrate changes (`dag.rs`):**

```rust
pub struct TransformNode {
    pub id: NodeId,
    pub target: TransformTarget,           // was DeclarationId
    pub descent: DescentEvidence,          // new structural field
    pub inputs: Vec<PortId>,
    pub output: PortId,
    pub span: SourceSpan,
}

pub enum TransformTarget {
    Callable(DeclarationId),
    Operator(OperatorKind),
    Unresolved(String, SourceSpan),        // must have diagnostic attached
}

pub enum OperatorKind {
    Arith(ArithOp),
    Cmp(CmpOp),
}
pub enum ArithOp { Add, Sub, Mul, Div }
pub enum CmpOp { Eq, Ne, Lt, Le, Gt, Ge }

pub enum DescentEvidence {
    StrictSubValue { parameter: String, shrink: ShrinkFactor },
    PreservedValue { parameter: String },
    Unrelated,
}
pub enum ShrinkFactor { Constant(u32), Structural }
```

**`operators.rs` becomes the single authority:**

```rust
pub fn from_symbol(s: &str) -> Option<OperatorKind> { ... }
pub fn signature(kind: OperatorKind) -> OperatorSignature { ... }
pub fn descent_shrink(kind: OperatorKind) -> Option<ShrinkFactor> { ... }
pub fn primitive_cost(kind: OperatorKind) -> CostUnit { ... }
```

(If implementation reveals all four functions are field lookups on a
single `OperatorSpec` record, collapse to `pub const OPERATOR_SPECS:
&[OperatorSpec]`. See collapse opportunity #3 below.)

**Files touched:**

- `parse.rs` — parser emits typed operator kinds at parse time via
  `operators::from_symbol`
- `lower.rs` — `TransformTarget` dispatch; descent evidence classification
  at lowering time; deletes `is_strictly_smaller`
- `infer.rs` — dispatches on `TransformTarget` variants structurally;
  deletes `unresolved_operator_name`, `is_comparison_operator`
- `operators.rs` — becomes single authority for symbol/kind/signature/
  cost/descent
- `dag.rs` — adds `TransformTarget`, `OperatorKind`, `DescentEvidence`;
  refactors `TransformNode`
- `m0_acceptance.rs` — updates test helpers; recursive descent tests read
  `DescentEvidence` structurally
- `m1_substrate_test.rs` — new test proves a new operator can be added by
  editing ONLY `operators.rs`

**Deletions:**

- `infer::unresolved_operator_name`
- `infer::is_comparison_operator`
- `operators::is_operator_name` (operators are structural post-parse, not
  name-matched)
- The operator-skip branch in `lower::resolve_pending_identifiers`
- `lower::is_strictly_smaller`

**Horizontal collapse opportunities (5):**

1. **Three dispatch paths become one.** Currently callable goes through
   `decide_transform` + Arrow walk, operator goes through
   `unresolved_operator_name` + `OPERATOR_FIELD_MAP` + `is_comparison_operator`,
   unresolved goes through sweep-error. After: one match on `TransformTarget`
   in `decide_transform`. Every consumer gets the same structural shape.

2. **Descent evidence is no longer operator-special.** `DescentEvidence`
   is a general structural fact on every Transform. Adding lexicographic
   descent or structural descent over children lists means adding a variant
   — no new string matches, no new reconstruction.

3. **`operators.rs` may collapse to declarative data.** If `signature`,
   `descent_shrink`, and `primitive_cost` are all field lookups on a
   single `OperatorSpec` record, the module becomes
   `pub const OPERATOR_SPECS: &[OperatorSpec]` — pure data, not scattered
   functions. This collapse isn't visible in the enumeration but emerges
   when PR-A writes the three functions side by side.

4. **`TransformTarget::Unresolved` may collapse with `alloc_identifier_stub`.**
   Currently `alloc_identifier_stub` creates a Declaration with
   `UnresolvedIdentifier` connective; `TransformTarget::Unresolved` carries
   the same information on a Transform. During PR-A we may discover
   Transforms never need the stub-Declaration indirection — the target
   field carries the unresolved name directly, and `alloc_identifier_stub`
   only exists for type references (in Arrow input/output positions). That
   cross-cutting simplification is only visible at PR-A's scope.

5. **`resolve_pending_identifiers` operator-skip branch disappears.**
   Currently the sweep iterates every `UnresolvedIdentifier(name)` and
   skips names matching `is_operator_name()`. After PR-A: operators never
   become `UnresolvedIdentifier` in the first place. The sweep is simpler
   and the condition doesn't exist.

**Acceptance gates (in addition to §7 universal gates):**

- `grep -E "target == \"|name == \"|is_operator_name|unresolved_operator_name|is_comparison_operator|is_strictly_smaller" src/v3/compiler/src/` → zero matches
- `grep 'OPERATOR_FIELD_MAP' src/` → matches only inside `operators.rs`
- `TransformTarget` has exactly 3 variants (compile-time assertion)
- `OperatorKind` has exactly 2 variants; `ArithOp` has 4; `CmpOp` has 6
  (compile-time assertions)
- New test: adding a new operator symbol to the enum updates ONE file
  (`operators.rs`); parser/lowerer/infer/descent all read the new variant
  structurally without edits

**Closes:** R-rows 1, 2, 3, 4, 5 (all open reconstructive facts).

### §5.2 PR-B: M1(3) + M1(4) — cost lens + Rust emitter (est. 12–18 hours)

**Purpose:** first writer lens, first real target-language emission,
parser support for `realization` literals, `ArrowBody::Pending` dissolution
— all as one atomic unit because they share reading patterns and
interact.

**Substrate additions:**

- Lens storage mechanism (deferred from M0 retrospective — first writer
  lens forces the decision). Options: per-lens side tables vs `Dag::lens_results:
  HashMap<LensId, Box<dyn LensResult>>` vs annotations map on Port. PR-B
  picks one and all subsequent writer lenses follow.
- Parser support in `parse.rs` for `realization { for: X.add; target:
  rust; body: "i64::wrapping_add" }` item syntax (the one remaining
  parser gap identified in M1_FOLLOWUPS.md)

**New files:**

- `src/v3/compiler/src/lens_cost.rs` — first writer lens; reads substrate
  + OperatorKind + ArrowBody; produces per-Node cost
- `src/v3/compiler/src/emit.rs` — walks Dag via language spec; produces
  Rust source
- `dsl/extdeps/languages/rust.dag` — first real language spec; declares
  Rust primitive realizations; replaces `inject_realization_stub`

**Files touched:**

- `parse.rs` — adds `realization` item parsing + record-literal body
- `bootstrap.rs` — deletes `inject_realization_stub`; parses
  `dsl/extdeps/languages/rust.dag` as the 8th bootstrap file
- Tests: new `smoke_compile_and_run` takes a user program through
  parse → lower → infer → lens_cost → emit → `cargo check` → run

**Horizontal collapse opportunities (5):**

1. **Cost lens and emitter share a `StructuralVisitor` trait.** Both
   walk `TypeConnective` variants, both dispatch on `ArrowBody`, both
   consume `OperatorKind` from PR-A. Writing them together reveals the
   shared "walk a Dag, produce a per-node output" pattern. Writing them
   separately would produce two walkers and notice the duplication
   later.

2. **The language spec IS the unified cost + realization source.** The
   `rust.dag` declaration for `Int.add` contains both "realizes as
   `i64::wrapping_add`" (emission) and "costs 1 machine instruction"
   (cost analysis). Both consumers read the SAME declaration. Building
   cost lens first with a temporary cost table and then realizing the
   emitter should read the same table is exactly the bridge pattern
   the invariants forbid.

3. **`ArrowBody::Pending` eliminates in the same PR that introduces the
   mechanism to populate `ExternalRealization`.** Before: bootstrap
   initializes with Pending, fixed up later. After: bootstrap parses
   `rust.dag`, primitive Arrows land with `ExternalRealization(decl_id)`
   from the start. Pending is never written to production declarations.
   §8.11 ratchet reaches zero in the same commit that introduces the
   dissolution mechanism. This is the cleanest possible scaffolded-state
   dissolution.

4. **`inject_realization_stub` deletion is the natural falling-out.** The
   stub exists because parse couldn't handle `realization` literal syntax.
   PR-B adds the parser support AND deletes the stub. Before → after:
   compiler bootstrap has zero Rust-manufactured declarations in
   production paths.

5. **Lens storage decision applies to every future writer lens.** Cost
   is the first; ownership, effects, purity, space-bounds will all
   follow the same pattern. Getting storage right in PR-B means PR-C
   through PR-N have zero substrate impact — the v3 thesis's "new lens
   = new file, zero substrate edits" success bar becomes provable by
   construction.

**Acceptance gates (in addition to §7 universal gates):**

- End-to-end: `compile("let x: Int = 1 + 2").emit_rust()` produces Rust
  source that compiles under `cargo check` and runs returning `3`
- `grep "inject_realization_stub\|Pending" src/v3/compiler/src/` → only
  historical / doc references, zero active code paths
- `lens_cost.rs` is < 250 lines (success-bar budget)
- `grep "\"i64::\\|quote!\\|\"Rust " src/v3/compiler/src/emit.rs` → zero
  matches (the emitter has no hardcoded target knowledge; it reads
  language spec declarations)
- Cost lens + emit both go through a shared `StructuralVisitor`-like
  pattern (proven by refactoring, not mandated in advance — the PR may
  discover the right abstraction shape)

**Closes:** 11 of 15 N-rows (all 5 cost; 5 of 6 emit; last emit row is
ownership, deferred).

---

## §6. Why larger PRs here — rationale

This document defaults to two large PRs rather than the ~6 smaller ones a
conventional code review workflow would prefer. Three reasons specific to
gunbc:

1. **Bridges appear at layer boundaries, not inside single files.** Every
   reviewer round on `free-cod-972` found a bridge where one piece of
   code assumed an upstream stage represented a fact structurally while
   the upstream stage represented it by name/lifecycle. A smaller PR
   that lands ONE layer at a time ensures the author can't see the next
   layer's needs, so they invent a provisional shape and the next layer
   has to reconstruct. A larger PR that touches substrate + parser +
   lowerer + inference + consumers at once forces reconciling every
   layer's needs against a single design — exactly what prevents
   reconstructive patterns.

2. **Horizontal collapses require seeing related code simultaneously.**
   The `StructuralVisitor` trait collapse in PR-B, the `OperatorSpec`
   record collapse in PR-A, the `resolve_pending_identifiers`
   operator-skip removal, and the `TransformTarget::Unresolved`/
   `alloc_identifier_stub` simplification are all things a smaller PR
   would miss because the related code would be in separate branches
   or already merged. Each PR in this plan explicitly lists 5 collapse
   opportunities it enables. Missing horizontal collapses is worse
   than PR-review cost.

3. **Non-production repo tolerates the reviewability cost.** gunbc has
   no external users blocked by the review window, no CI/CD pressure
   to merge fast, no team coordination overhead that scales with PR
   count. The typical "small PR for reviewability" argument assumes
   review is the bottleneck; here structural debt propagation is the
   bottleneck, and larger atomic PRs are the response. Review cost
   scales linearly with change size; debt cost scales quadratically
   with time-to-remediation. Paying the linear cost is obviously
   correct.

**The rule for atomicity:** for a PR to be atomic under this plan,
every consumer that reads or writes the changed substrate must be
updated in the same PR. If that scope exceeds ~20 hours of work, the
representation change is the wrong size — split the representation
change, not the consumer updates.

---

## §7. Universal acceptance gates (every PR)

1. **All existing tests stay green** — 51 minimum, growing as PRs add
   their own tests.
2. **Clippy clean** — `cargo clippy -p v3-compiler --all-targets -- -D warnings`
3. **No-bridges audit** — grep for adapter-function name patterns:
   `grep -E "fn .*_to_.*|fn convert_.*|fn adapt_.*|fn bridge_.*" src/v3/compiler/src/`
   returns zero new matches (INVARIANTS.md §"No bridges")
4. **No-name-dispatch audit** — `grep -E 'target ==|name ==|\.name\(\) ==' src/v3/compiler/src/infer.rs src/v3/compiler/src/lower.rs`
   returns zero matches (parser is exempt — raw input). Enforces that
   downstream consumers read structural facts, not names.
5. **No-deprecation audit** — `grep -E "TODO.*M[0-9]|scope-bound|dissolves in|_legacy|_v1|_v2" src/v3/compiler/src/`
   returns zero new matches (INVARIANTS.md §"No deprecations")
6. **Variant-count closure** — compile-time `const _ASSERT_*` match-
   exhaustiveness checks ensure no new enum variants were added
   silently. Any new variant requires explicit sign-off against the
   C1-class stop signal (INVARIANTS.md §"No short-term solutions"
   and THESIS.md §"The substrate").

These gates enforce invariants structurally rather than by convention.
Any PR failing one is not mergeable.

---

## §8. Non-goals (explicit scope exclusions)

Out of scope for M1(2.6) → M1(4):

- Ownership analysis / lattice-on-bindings (M1(5)+)
- Interpreter (deferred; revisit when omni-emission starts)
- Omni-emission Shape B user-program artifacts (YAML, Terraform,
  SPICE, docs) — M2+
- Unification (five behaviors as patterns over Node) — recorded as
  future candidate in THESIS.md, not committed
- Three-primitive reduction — recorded as future candidate, not
  committed
- `dsl/std/meta.dag` as a first-class mechanism — defer until second
  consumer appears
- Law verification (associativity, commutativity, etc.) — M1(5)+
  algebraic simplification lens
- Substrate-extension full CI audit — simple grep-based version lands
  with PR-A; full mechanism deferred to after PR-B
- v3-compiler in the CI workflow — tracked separately; lands as a small
  PR ASAP, not folded into this plan

---

## §9. How to use this document

**When starting a new M1 iteration:**

1. Read §2 to confirm the baseline is still accurate. If `free-cod-972`
   has moved beyond what §2 describes, update §2 first.
2. Read §3 and identify which consumer you're about to touch. Every
   question that consumer asks must already be in §3 with status
   S or R. If you discover a new question not in §3, STOP — add the
   row (with status R if the code currently reconstructs it, or N
   if you're pre-enumerating for future work) and resolve it
   structurally before writing the consumer code.
3. Map your work to one of the PRs in §5. If your work doesn't fit
   an existing PR, either it belongs in a future M1(5+) that this
   document doesn't cover, or the document needs a new §5 entry
   before implementation begins.
4. Before merge, run every §7 universal gate plus the PR-specific
   gates listed in §5.

**When you find a reconstructed fact in the codebase:**

Per `INVARIANTS.md` §"No short-term solutions" escalation procedure:
stop, back up, assess the damage, raise it as alarming. **Do not**
silently work around it in your own code. Add it as an R-row in §3
and figure out whether it needs to land in the current in-flight PR
or a new one.

**When enumerating a new downstream consumer:**

1. Read its expected code path mentally or sketch it.
2. List every question it asks about Nodes, Declarations, Ports, or
   Behaviors.
3. Mark each S/R/N.
4. For each R: that's a bridge to eliminate now.
5. For each N: confirm the substrate will answer structurally when
   the consumer is written, or flag the gap as future substrate work.

The enumeration is the design phase. Writing code is the
implementation phase. Implementation must not create new R-rows; any
R-row discovered mid-implementation means the design phase missed
something and the enumeration needs an update before proceeding.

---

## §10. Open questions (to resolve before starting PR-A)

Small design decisions that would benefit from being pinned before
implementation begins:

1. **Should `TransformTarget::Unresolved` carry the failed name and span,
   or just a generic "unresolved" marker with the name living in the
   attached Diagnostic?** Trade-off: carrying the name makes
   diagnostics self-contained but couples `TransformTarget` to
   diagnostic text; not carrying it requires consumers to cross-
   reference the diagnostic table. Likely answer: carry it — the
   name is a structural fact, not just diagnostic metadata.

2. **Is `OperatorKind` the right split (Arith vs Cmp), or should it be
   flatter (10 variants directly)?** Trade-off: the hierarchical form
   lets some dispatch code match just on the category (`if matches!(k,
   OperatorKind::Cmp(_))`) without enumerating operators; the flat form
   is simpler but loses that. Likely answer: hierarchical — the Cmp
   family shares a return type `Bool`, the Arith family shares `T →
   T`, and that distinction is structurally real.

3. **Where does `DescentEvidence` live — on `TransformNode` directly, or
   on a separate analysis table?** On-node is more honest (facts flow
   forward on the node); side table is more flexible for late analysis.
   Likely answer: on-node, per §"facts flow forward" invariant. Cost
   is a small struct per Transform; storage overhead is negligible.

4. **What's the `CostUnit` type for primitive cost in PR-A's operator
   table?** A single `u64` cycle-count? A `{cycles, allocations,
   io}` record? Defer: `u64` is enough for PR-A's purposes, the real
   cost-algebra decision happens in PR-B's `lens_cost.rs`. PR-A just
   reserves a field.

These are intentionally left open — they get pinned in the PR itself,
not in this plan.

---

## §11. References

- `THESIS.md` §"The substrate: two coordinated shapes"
- `THESIS.md` §"Two groundings: static validation vs efficient realization"
- `THESIS.md` §"Epistemic stacking: every concept grounds in primitives"
- `INVARIANTS.md` §"No short-term solutions (this is not a production codebase)"
- `INVARIANTS.md` §"No bridges"
- `INVARIANTS.md` §"No deprecations"
- `src/v3/M1_DESIGN.md` — the substrate shape spec that M1(2.5) executed against
- `src/v3/M0_RETROSPECTIVE.md` — M0 closing notes; history
- `feedback_enumerate_before_substrate.md` — the memory that this document
  operationalizes

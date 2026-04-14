# M1(2.5) Implementer Task List

**Oracle:** `src/v3/M1_DESIGN.md`. Read it in full before starting.
Every section reference below points at that file.

**Scope:** substrate rework only. No cost lens, no emitter, no
unification, no three-primitive reduction, no full shell.dag.
All deferred per the convergence note in `M1_DESIGN.md` §1 and
§"Explicit non-goals."

**Total estimate:** 14–16 hours. Single focused PR. Do not split
— the phases below depend on each other and leave the tree in an
incoherent state between steps.

---

## Phase 1 — Substrate data model (~1.5h)

Drop the new types into `src/v3/compiler/src/dag.rs`. Spec in §3.

- [ ] Add `Declaration`, `TypeConnective`, `Field`, `AtomPayload`,
      `LiteralBits`, `CardinalityBound`, `TemplateArgument`,
      `ArrowBody` per §3.
- [ ] Keep `DeclarationId` and `NodeId` as **separate newtypes**
      (§Q6). No shared ID space. Use distinct Vec fields on `Dag`.
- [ ] Delete `DeclKind::{Type, Function}`, `FunctionRef { name:
      String }`, `LiteralValue`, `Signature`, `primitive_signature`
      function, `Dag.signatures` HashMap, `register_signature`,
      `lookup_function`.
- [ ] `CardinalityBound` commits to three variants: `Exact(u32)`,
      `AtMostOne`, `Unbounded`. §Q5.
- [ ] `ArrowBody` commits to three variants: `UserDefined(NodeId)`,
      `ExternalRealization(DeclarationId)`, `Pending`. §Q7.

**Done when:** `cargo check -p v3-compiler` compiles the new
substrate types. (Other modules will be broken at this point —
expected.)

---

## Phase 2 — Tokenizer + parser (~4.5h)

### Tokenizer (~30m)

File: `src/v3/compiler/src/tokenize.rs`.

- [ ] Verify `<` and `>` tokenize as type-parameter delimiters.
- [ ] Verify `|` tokenizes as a top-level separator (for sum types).
- [ ] Verify `fn(` tokenizes as the start of an Arrow type expression.
- [ ] Verify `?` tokenizes after a type expression (for `T?`).

### Parser (~4h)

File: `src/v3/compiler/src/parse.rs`. §8.7, §8.8, §8.9.

- [ ] Two-pass lowering. Pass 1 collects all top-level declaration
      names into a symbol table. Pass 2 resolves Identifier atoms
      against the symbol table. §8.1.
- [ ] `type Foo<T> { ... }` — parse into a Conj declaration with
      a TypeParam Atom child for `T` plus labeled content children.
      §8.3 (shared parameter Atom identity).
- [ ] `type Foo = Bar<X>` — parse into an Instantiation declaration
      with `template: Bar` and `arguments: [T_of_Bar := X]`.
- [ ] `type Name = A | B(payload)` — parse into a Disj declaration
      with unit or payload variants. §8.7.
- [ ] `fn(A, B) -> C` as a type-position expression — parse into
      an Arrow connective usable inside field types and parameter
      lists. §8.8.
- [ ] `T?` — parse into `Cardinality { element: T, bound: AtMostOne }`.
- [ ] `List<T>` — parse into `Cardinality { element: T, bound:
      Unbounded }`.
- [ ] Infix operators (`+`, `-`, `*`, `/`, `==`, etc.) — parse into
      a Conj with `{function: Ref, args: Sequence<Expr>}` shape.
      The lowerer / inferer resolves the function by walking
      inhabitance on the argument types. §8.9.
- [ ] Drop the old `fn std::int::add(...)` primitive-signature
      shortcut. Primitives come from `std/algebra.dag` inhabitance
      instead.

**Done when:** parser produces Node trees for the four bootstrap
std/ files and the synthetic nested-domain test input without
panicking. (Inference may still fail — that's Phase 4.)

---

## Phase 3 — Lower (~2h)

File: `src/v3/compiler/src/lower.rs`.

- [ ] Lower surface syntax into the `TypeConnective` shape from
      Phase 1.
- [ ] Identifier resolution: at lowering time, every `Atom(Identifier
      { resolved: None, .. })` gets resolved to
      `Some(DeclarationId)` against the symbol table from Phase 2.
      §8.1.
- [ ] Build substitution scopes for parameterized declarations.
      Type parameters declared at the top of a `type Foo<T> { ... }`
      are in scope throughout Foo's body.
- [ ] Preserve spans structurally on every Declaration (same rule
      as M0 — no span side tables).

**Done when:** lowering produces fully resolved Declaration trees
for the bootstrap files and the synthetic oracle, with no
unresolved Identifier atoms anywhere.

---

## Phase 4 — Inference (~3h)

File: `src/v3/compiler/src/infer.rs`. §Q1, §Q4, §Q7.

- [ ] Replace old `DeclKind` dispatch with `TypeConnective` match
      over the six variants.
- [ ] Implement `SubstStack` as `Vec<Vec<TemplateArgument>>` with
      linear lookup. §8.4.
- [ ] Lazy substitution: walkers push on entering Instantiation,
      pop on exit, look up TypeParam references in the stack.
      §Q4.
- [ ] Walking `Int.add`: `Int → Instantiation(OrderedRing, [T :=
      Word64]) → OrderedRing (Conj) → add child (Arrow) → substitute
      T → Arrow(Word64, Word64, Word64)`. §5 has the full step-by-
      step.
- [ ] Handle `ArrowBody::Pending` as realization-pending: signature
      type-checks via inhabitance; body-checking is skipped.
      Inference must NOT panic on Pending. §Q7.
- [ ] Handle `ArrowBody::ExternalRealization(decl_id)` by walking
      the realization declaration and verifying the declared
      signature matches the Arrow's signature.
- [ ] Handle `ArrowBody::UserDefined(node_id)` by walking the
      computation sub-DAG and checking the body against the
      declared input/output types (same as M0 semantics).
- [ ] Infix operator resolution: given a Conj-shape function call
      `{function: "+", args: [lhs, rhs]}`, walk lhs/rhs inferred
      types → inhabits chain → find `add` field → use its Arrow
      signature. §8.9.
- [ ] Exhaustiveness check on Branch nodes: a Conj-shape match
      `{scrutinee, cases: Disj<Case>}` with unbound Disj variants
      is an unbound-child error. §Q0.

**Done when:** the two substrate tests from Phase 6 pass against
the new inference.

---

## Phase 5 — Bootstrap std/ (~1h)

Files: `dsl/std/logic.dag`, `dsl/std/bit.dag`,
`dsl/std/algebra.dag`, `dsl/std/types.dag`, `src/v3/compiler/src/bootstrap.rs`.
§8.6.

- [ ] **Verify or create** `dsl/std/logic.dag`: Classical with
      `Bit = Disj { On: Atom; Off: Atom }` and `True`/`False` as
      Atoms. If v2 doesn't have this file, create a minimal stub.
- [ ] **Verify** `dsl/std/bit.dag` parses: `Word8..Word64` as finite
      Bit sequences.
- [ ] **Verify** `dsl/std/algebra.dag` parses into the new
      substrate: every algebra (Magma through OrderedRing, Field,
      Lattice, BooleanAlgebra, FreeMonoid, PartialFunction) lands
      as a Conj with TypeParam children and Arrow/Atom fields.
      Arrow bodies land as `Pending`.
- [ ] **Verify** `dsl/std/types.dag` parses: `Int = OrderedRing<
      Word64>`, `Bool = Classical`, `String = FreeMonoid<Char>`,
      `List<T> = FreeMonoid<T>`. All as Instantiation declarations.
- [ ] **New or updated** `src/v3/compiler/src/bootstrap.rs`: parses
      the four std/ files in order at `Dag::new()` time. Replaces
      the old `bootstrap_primitives` function.
- [ ] **Delete** per-primitive `fn std::int::add(...)` etc.
      declarations in `dsl/std/core.dag`. They become inhabitance-
      derived via OrderedRing.

**Done when:** `Dag::new()` loads all four std/ files without
error, the declaration table contains Magma through OrderedRing
and Int/Bool/String/List as expected, and an ad-hoc assertion
confirms the Int declaration has connective `Instantiation` with
template OrderedRing.

---

## Phase 6 — Tests (~2h)

File: `src/v3/compiler/tests/m0_acceptance.rs` (update helpers),
`src/v3/compiler/tests/m1_substrate_test.rs` (NEW).

- [ ] Update M0 test helpers (`fn_ref`, `type_shape`,
      `atom_literal`, etc.) to build the new `TypeConnective` tree.
      Semantics of the 40 M0 tests is unchanged; only helper
      construction changes.
- [ ] **NEW `parse_std_algebra_and_walk_int_add`**: parses the
      four bootstrap std/ files, declares `let x: Int = 1 + 2`,
      walks `Int.add` via inhabitance, asserts the result type is
      `Arrow { inputs: [Word64, Word64], output: Word64 }` with
      `body: Pending`. §5 has the step-by-step walk.
- [ ] **NEW `parse_synthetic_service_all_layers`**: parses the
      synthetic nested-domain model from §6 (SyntheticService /
      SyntheticOperation meta-types and a CmdExec instance), asserts
      the Declaration tree structure matches the five-level
      nesting shown in §6.

**Done when:** `cargo test -p v3-compiler` shows 42 green (40 M0
+ 2 substrate).

---

## Phase 7 — Close out (~30m)

- [ ] `cargo clippy -p v3-compiler --all-targets -- -D warnings`
      clean.
- [ ] **Variant audit**: `grep -n 'TypeConnective::' src/v3/compiler/
      src/*.rs | cut -d: -f3 | sort -u` lists exactly the six
      committed variants (Atom, Conj, Disj, Arrow, Cardinality,
      Instantiation). No new variants added during implementation.
- [ ] **Behavior audit**: `grep -n 'Behavior::' src/v3/compiler/src/*.rs`
      lists exactly the five committed behaviors (Value, Transform,
      Branch, Loop, Bind). M0's behavior enum unchanged.
- [ ] **`DeclKind` audit**: `grep -n 'DeclKind' src/v3/compiler/src/`
      returns zero matches. The old enum is fully gone.
- [ ] **Name-based dispatch audit**: `grep -rn '"std::int::add"\|
      "std::int::' src/v3/compiler/src/` returns zero matches.
      Primitives resolve via inhabitance, not name lookup.
- [ ] `src/v3/M0_RETROSPECTIVE.md` gets a short addendum at the
      bottom noting M1(2.5) supersedes the scaffolded primitive
      substrate from M1(1).
- [ ] PR description cites this task list and `src/v3/M1_DESIGN.md`
      as the spec; reviewers can check each phase's acceptance
      gate against the box above.

**Done when:** all six audits pass, clippy is clean, all tests
green.

---

## If you hit a wall

- **Ambiguous spec** → reread `src/v3/M1_DESIGN.md` §4 (the six
  Q answers) and §8 (sub-decisions committed). If still ambiguous,
  flag it on the PR and ask before guessing.
- **Substrate extension pressure** (want to add a 7th connective
  or 6th behavior) → STOP. Follow the C1-class stop signal in
  `THESIS.md` §"Substrate extension." All four dissolution patterns
  must be attempted. Tag this task list with the attempt before
  proceeding.
- **Infer fails on Pending**: check §Q7. Pending arrows must type-
  check the signature via inhabitance without walking the body.
  If inference is trying to walk the body, the Pending handling
  is wrong.
- **SubstStack lookups miss**: verify that `type Monoid<T> { ... }`
  is producing a single TypeParam Atom (§8.3) and that all four
  references in the body resolve to the same DeclarationId. If
  they're different atoms per reference site, lowering is broken.
- **M0 test fails**: the test helper changes for Phase 6 are
  structural, not semantic. If semantics diverge, the new
  substrate has a bug. Do NOT "fix" it by changing test
  assertions. Back out the semantic change.

## Non-goals (DO NOT do these in M1(2.5))

- Cost lens (M1(3))
- Single Rust emitter (M1(4))
- Unification of five behaviors into patterns (deferred candidate)
- Three-primitive reduction (deferred)
- `std/meta.dag` as a first-class mechanism (deferred)
- Full `dsl/extdeps/shell.dag` fidelity including exit / mock_response
  / transport_shell_template (M1(2.6) or M2)
- `extdeps/languages/rust.dag` realization declarations (M3-ish)
- Law verification (associativity, commutativity) (M1+ algebraic-
  simplification lens)
- Omni-emission projection rules (post-emitter)
- Interpreter (deferred)
- SUBSTRATE_EXTENSION_AUDIT.md + CI ratchet (M1(3), §8.10)
- Bootstrap B/C count refresh (separate docs-only commit)

If any of these feel tempting during M1(2.5), write a followup
task but do not land the work in the M1(2.5) PR.

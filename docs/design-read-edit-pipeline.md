# Read/Edit Pipeline — Node-Centric Agent Surface (design spec)

> **Status: design.** Frames the substrate's read/edit surface — how
> arbitrary code is read and modified at the Node level. The mechanical
> primitives (`Path` / `Edit` / `Diff`, `apply_lens`, `apply_diff`)
> already exist in `src/v4/std/node.dag` + `src/v4/lens/application.dag`;
> T-23 realizes them. This doc captures the design intent + worked
> examples + open interface questions, especially for LLM/agent
> consumers where the interface — not the mechanics — is the hard part.

## 0. The framing

**Code is Nodes, not files.** Files are a delivery / persistence
mechanism — they map onto the substrate via
`src/v4/extdeps/file_system.dag`, but the language doesn't couple to
them. Reads target Nodes (and scopes within Nodes); writes are
structural Edits to Nodes. Node-to-File binding is its own concern,
modeled *alongside* Node, not *inside* it.

This is the operator-ratified position per QRY-1 (2026-05-15, in
`src/v4/lens/application.dag`) + the Path/Edit/Diff vocabulary
ratified in PR #3162.

## 1. Why node-centric

Three reasons:

1. **Files are decoupled from concepts.** One concept can span
   multiple files (e.g. an alias re-exports a `std/` carrier); one
   file can carry multiple concepts. Reading at file granularity
   loses concept structure.
2. **Edits should be structural, not textual.** A textual edit at
   "line 42" doesn't compose; a structural Edit at a `Path` composes
   via `apply_diff`. Structural edits are also unaffected by
   reformatting / whitespace / comment churn.
3. **Agent reasoning is at concept level.** An LLM reasoning about
   "rename concept X" or "extract this fold pattern" wants a `Path`
   that addresses the concept's structure — not a "line range in
   file Y." The right abstraction is the Node.

The substrate today is incomplete on file-tying. `src/v4/extdeps/file_system.dag`
models POSIX file operations (open / read / write / close) but **does
NOT yet carry a Node→File rendering binding** as a first-class
substrate fact. The Node-to-File relationship is currently **emergent
from the emit stage** — emit reads the Dag, produces text output per
target language — not stored as a queryable `data ... : NodeToFileBinding`
row. This is an honest gap: the design intent (file-tying as
substrate data so it's queryable and auditable, not implicit in emit
behavior) is right, but the substrate primitive doesn't exist yet.
See §6.8 / §7 — landing a Node→File binding registry (e.g.
`data <node>_rendered_into: NodeToFileBinding = { node: ..., file: ..., region: ... }`)
is a tracked gap for the file-as-effect framing to become fully
structural.

## 2. The read interface

Per QRY-1, the read surface is `apply_lens(lens, scope, mode)`. There
is **no separate query language and no `query.dag`** — those would be
decorative parallel authority.

**Lens** = a deterministic structural projection over the parsed
model. Type:
- `lens(scope) → Witness<finding>` in `Introspect` mode
- `lens(scope) → Outcome<()>` in `Enforce` mode

**Scope** = where the lens runs. Per the ratified
`SectionRef = DeclarationScope | NodeScope` contract in
`src/v4/lens/application.dag`, there are **two** scope forms:
- `DeclarationScope(decl_ref)` — at a declaration unit (module-level
  / decl-level scope)
- `NodeScope(node_ref)` — at a sub-Node ("ask <lens> about <this
  section>")

There is **no separate `RootScope` / corpus-wide variant** —
corpus-wide application is achieved by **composition over the
declaration set**: iterate over `declarations_in(dag)`, applying
`apply_lens(lens, DeclarationScope(d), config)` per declaration. The
substrate intentionally does not have a "scan the whole corpus"
primitive; that's a fold over the section set the caller assembles
(or that `affected_set(dag, diff)` derives).

**Mode** = what the lens does:
- `Introspect` — read-only; produces a structural fact carrier
- `Enforce` — fail-closed gate; produces `Outcome<()>` (passes, or
  `Rejected { diagnostic }`)

**The lens catalog** (`src/v4/lens/*.dag`, 12 files):
- `affected_set` — incremental re-exec frontier
- `application` — the `apply_lens` surface itself (the meta-lens)
- `complexity` / `cost` — complexity / cost projection
- `coverage` — meta-lens for coverage discipline
- `effect` / `parallelism` / `ownership` / `idempotency` —
  behavioral lenses
- `synthesis` — cross-algorithm complexity
- `testgen` — TestClaim corpus generator
- `registry` — single-authority registry projection

**Composition**: lenses compose by feeding into each other.
`affected_set(dag, diff)` produces a `Witness<ReExecFrontier>` — a
*set* of declaration / node references that need re-validation given
the Diff. The frontier is **not** a third `SectionRef` variant; it's
a set that the caller folds over by re-applying the lens at each
member's `DeclarationScope` or `NodeScope`. The two-branch
`SectionRef = DeclarationScope | NodeScope` discipline holds — gating
over the affected frontier is composition, not a new scope shape.

## 3. The write interface

Per Path/Edit/Diff (`src/v4/std/node.dag`, ratified PR #3162):

- **`Path { steps: List<Symbol> }`** — a structural address to a
  sub-Node. Not a filesystem path; the Node graph's own coordinates.
- **`Edit { at: Path, replacement: Node }`** — a structural rewrite
  at a Path. **Only** a replacement; the ratified `std/node.dag`
  definition has no separate insert / delete variants. Insertions
  and deletions are expressed by replacing the parent node with a
  new parent whose children include / exclude the targeted child.
- **`Diff { edits: List<Edit> }`** — ordered sequential rewrite
  program. The list ordering is significant — Edits compose by
  sequential application, NOT parallel.

Per the ratified contract in `src/v4/lens/application.dag`:

```
apply_diff(root: Node, d: Diff) -> Result<Node, Diagnostic>
subterm_at(root: Node, p: Path) -> Result<Node, Diagnostic>
```

(The carrier is `Result<_, Diagnostic>` and the input is `Node` —
note that `affected_set.dag` does define `type Dag = Node`, so the
`Dag` shorthand is acceptable in prose, but the published signatures
use `Node` directly.)

`apply_diff` semantics:
- Folds Edits in order, each as `t[s]p` (substitute at path)
- **All-or-nothing**: if any Edit's Path no longer resolves (because
  a prior Edit invalidated it), the whole Diff fails with one
  Diagnostic; no partial application
- **No independence/well-formedness predicate** on Diff — any
  `List<Edit>` is a valid rewrite program (PR #3162, P2 finding 2)

## 4. The read → edit pipeline

Reads precede writes (the existing pattern; preserve it). The agent
loop is seven steps, closed, with **gates running against the
candidate post-edit state**, not the pre-edit graph. The candidate
root is **explicit in the gate surface**, not a comment-level
convention:

```
1. Read       →  apply_lens(lens, scope_in(dag, ref), Introspect)
2. Diagnose   →  reasoning over the facts (LLM or human)
3. Propose    →  produce Diff
4. Candidate  →  candidate_dag = apply_diff(dag, Diff)      # uncommitted candidate
                                                            # (fail-closed Diagnostic if any Edit's
                                                            #  Path doesn't resolve)
5. Gate       →  affected_set(dag, Diff).frontier.for_each(ref =>
                   apply_lens(_, scope_in(candidate_dag, ref), Enforce)
                 )                                          # candidate_dag is the gate root
6. Commit     →  dag := candidate_dag                       # only if every gate passed
7. Re-emit    →  emit per target language                   # files are a downstream effect
```

`scope_in(root: Node, ref: NodeRef) -> SectionRef` is the helper
that **explicitly binds a frontier ref to a dag root**, producing a
`DeclarationScope` / `NodeScope` resolved in that root. It makes the
candidate-vs-pre-edit context **structurally visible in every gate
call** — a worker can't accidentally lose the candidate authority by
calling `apply_lens(_, ref, Enforce)` with an ambiguous root.

**Why gate the candidate, not the pre-edit graph.** Gating the
pre-edit graph + applying the Diff afterwards would let a Diff
introduce a post-edit invariant violation that never gets enforced —
exactly the semantic gap the project's "structure gates emission"
thesis exists to close. The candidate-state pattern keeps the
fail-closed promise honest: validation happens against the state
that will be emitted, not against a state already known to be valid.

Step 7 is the **only** place files re-enter the picture — they are a
*consequence* of the committed substrate state, not a cause.

The pipeline is closed: any Diff that doesn't pass the gates against
the candidate state doesn't commit, period. There is no "force apply."
The agent earns no special trust; structure gates it.

## 5. Worked examples

Three concrete .dag-level scenarios showing read-then-edit shape.

> **Pseudo-code disclaimer.** The examples in §5 and §6 use
> **higher-level edit verbs** (`replace_with`, `replace`, `insert`,
> `insert_field`) as a **future combinator layer** shorthand —
> *not* as literal `.dag` against the ratified single-Edit shape.
> The ratified carrier is exactly `Edit { at: Path, replacement:
> Node }` (§3). Insertions and field-additions decompose into
> parent-replacement: build a new parent node whose children list
> includes the desired child, and emit a single `Edit { at:
> parent_path, replacement: new_parent }`. The shorthand verbs are
> what §6.8 item 1 (machine-readable Clean shape) and item 3
> (DAG-of-edits composition) are about: a thin combinator layer
> over the ratified primitive that compresses common intent shapes.
> Treat the examples as intent illustrations, not as authoritative
> Edit constructors.

### Example A: bare-alias refactor (`type RustBool = Bool` → canonical-B)

Same shape as the canonical-B work in PR #3338 (operator-ratified
2026-05-19, "ALL external types grounded in our substrate").

**Read** — find all bare-alias sites by composition over the
declaration set:
```dag
matches = declarations_in(dag).flat_map(d =>
  apply_lens(L1.7.bare_alias_signature, DeclarationScope(d), Introspect)
    .matches()
)
// → List<Match { path: Path, type_name: Symbol, aliased_to: Symbol }>
```

**Diagnose** — for each match, decide: trivial decl-ref (Bool) or
refinement (numeric width)? Per the operator's no-quirked-types
framing, both are legitimate patterns; choose based on what the spec
demands.

**Propose** — Diff for the Bool decl-ref case:
```dag
edits = matches.filter(m => m.type_name == RustBool).map(m =>
  Edit {
    path: m.path,
    replace_with: decl_ref(canonical_authority: bool_boolean_algebra)
  }
)
diff = Diff(edits)
```

**Candidate + Gate + Commit** — candidate-state pattern: apply Diff
into an uncommitted candidate Node, gate against the candidate's
affected frontier, commit only if all gates pass:
```dag
candidate_dag = apply_diff(dag, diff)
// → fail-closed Diagnostic here if any Edit's Path doesn't resolve

affected = affected_set(dag, diff)
// gate against the CANDIDATE state — scope_in(candidate_dag, ref) makes
// the candidate root structurally explicit in every call:
affected.frontier.for_each(ref =>
  apply_lens(L1.7, scope_in(candidate_dag, ref), Enforce)
  // → fails if any new prose-asserted facts introduced
)
affected.frontier.for_each(ref =>
  apply_lens(L1.10.b, scope_in(candidate_dag, ref), Enforce)
  // → checks no String-escape-hatch introduced
)

// commit only if every gate passed:
dag := candidate_dag
```

**Re-emit** — `rust.dag` re-renders to Rust source per target emit.
The file change is an *effect*, not the cause.

### Example B: rename a concept across the corpus

Goal: rename `Bool` → `BooleanValue` everywhere it's referenced. The
`CanonicalConcept` registry is the source of truth; the Diff cascades.

> **Dependency caveat (same as §6.4).** `CanonicalConcept` is a
> design-pending carrier today — it exists in
> `docs/design-dissolution-lens.md` (PR #3334) but **NOT** as
> ratified `.dag` substrate. See §6.8 item 8 — this example
> assumes the carrier has landed; until it does, this case is
> design-pending, not runnable.

**Read** — both lenses fold over the declaration set:
```dag
concept_row = declarations_in(dag).find_map(d =>
  apply_lens(canonical_concept_for(symbol: Bool), DeclarationScope(d), Introspect)
    .single_match()
)
// → CanonicalConcept { canonical_home: v4.std.logic.Bool, members: { ... } }

references = declarations_in(dag).flat_map(d =>
  apply_lens(references_to(symbol: Bool), DeclarationScope(d), Introspect)
    .matches()
)
// → List<Path>
```

**Diagnose** — rename the `canonical_home`, every `members` entry,
and every reference. The substrate's structural-alias edges propagate
transitively, so the rename is closed.

**Propose**:
```dag
edits = [
  Edit { path: concept_row.canonical_home, replace: BooleanValue },
  ...concept_row.members.map(m => Edit { path: m, replace: BooleanValue }),
  ...references.map(r => Edit { path: r, replace: BooleanValue }),
]
diff = Diff(edits)
```

**Gate + Apply + Re-emit** — same shape as Example A. **L1.12
Parallel-authority** lens stays green because the `CanonicalConcept`
registry tracks the renamed concept's identity (this is exactly why
L1.12 keys on `CanonicalConcept` membership and not lexical spelling).

### Example C: TestClaim breakage diagnosis + fix

A `TestClaim` row asserts a property; after a recent Diff the claim
fails.

**Read**:
```dag
claim = declarations_in(dag).find_map(d =>
  apply_lens(test_claim_lookup(id: claim_id), DeclarationScope(d), Introspect)
    .single_match()
)
// → TestClaim { subject: Node, property: lens, expected: Witness }

actual = apply_lens(claim.property, NodeScope(claim.subject), Introspect)
// → Witness (the current value)

recent_diff = declarations_in(dag).flat_map(d =>
  apply_lens(diffs_since(commit: last_known_passing), DeclarationScope(d), Introspect)
    .matches()
)
// → List<Diff>
```

**Diagnose** — compare `actual` vs `claim.expected`. Walk `recent_diff`
to identify which Edit invalidated the claim.

**Propose** — either revert the offending Edit (apply_diff of its
inverse) or produce a new Diff that restores `claim.property` while
preserving the Edit's intent.

**Gate** — `apply_lens(claim.property, NodeScope(claim.subject), Enforce)`
is the literal validation; the TestClaim's property *is* the lens.

## 6. Lens as `(find, transform)` — the convolution view + auto-fix demonstrations

The framing operators have used to make this concrete:
*"edit-via-code is a convolution — find this pattern, transform to
this pattern."* Every L1.x dissolution lens is already structurally
a `(find, transform)` pair: the Signature is the **find** half, the
Clean shape is the **transform** half. Today both halves are
authored as prose + example; the convolution-driven view turns them
into executable rewrite operators.

This is the most concrete answer to §7.1 below ("higher-order Edit
combinators") — the L1.x catalog is *already* the seed library of
combinators, just not yet machine-readable as transforms.

### 6.1 Convolution vs synthesis — two distinct shapes

Two distinct code-via-code shapes, both useful, both produce Diffs
against the same substrate:

| Shape | Kernel | Input | Output | When to use |
|---|---|---|---|---|
| **Convolution** | `(find_lens, transform_fn)` pair | Node graph | Diff | refactor / dissolve / interface cascade |
| **Synthesis** | spec + cost constraints | `TestClaim` + complexity bound | Diff that lands a *new* function | "write merge sort" / pure synthesis |

Convolution rewrites existing structure; synthesis generates new
structure to satisfy a spec. They compose: synthesis lands a function,
convolution-style refactoring keeps it clean as the substrate evolves.

### 6.2 Auto-fix per L1.x — the seed combinator library

Every L1.x lens has a hero auto-fix case implicit in its existing
prose Clean shape. The table makes them explicit as
`(find, transform)` pairs ready to be authored as machine-readable
substrate data:

| Lens | Find (signature) | Transform (auto-fix shape) | Hero case |
|---|---|---|---|
| **L1.1** Discriminant-predicate | `fn(coproduct) -> Bool { match { V => true/false } }` | Delete fn; rewrite consumers to inline `match` (or use derived discriminant once Track 2 lands) | `nat_is_zero` removed; consumers rewritten to `match n { Zero => ..., Succ => ... }` |
| **L1.2** Degenerate-type | (a) struct-of-fns no `data`; (b) N near-identical wrappers | (a) inline as plain fn; (b) coalesce to coproduct + factor shared field | 26 `ListMap<A,B> { apply: fn }` wrappers collapse to one `list_map<A,B>(xs, f)` |
| **L1.3** Hollow-type | declared but no inhabitance edge | Delete the type; rewrite the (very rare) incoming refs | Inert `ParseError` removed; consumers use `Diagnostic` directly |
| **L1.4** Carrier-clone | local coproduct ≅ `std/` carrier | Replace local with std/ carrier; rewrite all consumers | `NormalizeChildrenResult` → `Outcome<List<Edge>>`; consumers updated |
| **L1.5** Catamorphism | recursive `match`-over-data-shape | Replace with `fold` / `traverse` primitive | `ci_member` → `list_any(xs, fn(h) { symbol_eq(a: s, b: h) })` |
| **L1.7** Off-substrate-fact | prose claim, no structural witness | Synthesize witness from claim type | `merge_evidence` + "inhabits BoundedLattice" comment → emit `data ... : BoundedLattice<DescentEvidence> = { ... }` |
| **L1.8** Wrong-home | fn whose primary concept lives upstream | Move fn + rewire imports | `nat_compare` from `float.dag` → `nat.dag`; `float.dag` imports it |
| **L1.9** Vacuous-arm | `match` arm with trivial RHS, asymmetric within fn | (mostly) operator-confirm exemption; rare auto-fix | Operator-judgment dominated; structured 🟡 instead of auto-apply |
| **L1.10.a** TemplateHole | string literal with `{N}` placeholders | Replace template with grammar-as-data structural emitter | `list_template: "Vec<{0}>"` → `RustTypeRealization` data row |
| **L1.10.b** CanonicalCarrier | `String` field whose name maps to registered typed carrier | Replace field type with typed carrier; rewrite construction sites | `ShellCommand { command: String }` → `ShellCommand { command: posix.Command }` |
| **L1.11** Plausible-fallback | `None => non-Rejected Ctor` | Lift return to `Outcome<T>`; replace with `Rejected { diagnostic: DerivationUnknown }` | `derive_effect_shape` `DELETE None => CreateEffect` → `Rejected { diagnostic: ... }` |
| **L1.12** Parallel-authority | duplicate concept home | Make non-canonical an alias edge OR add `HistoricalDeclaration` row | `dsl.std.types.Bool` → alias-identity edge to `v4.std.logic.Bool` |

**Most cells have a clean unambiguous transform.** The exceptions
(L1.9, parts of L1.7) need operator judgment — captured structurally
as `🟡 needs operator decision { because: <closed-vocab> }` rather
than freeform prose. Same pattern as the "explicit throwaway
acknowledgment vs no test" stance the operator ratified 2026-05-19.

### 6.3 Hero case (a): L1.5 catamorphism auto-fix

The cleanest illustrative case — unambiguous transform, small cascade,
no operator judgment.

**Find** — L1.5's existing signature matches `fn` recursing over a
structural type by `match`ing its variants + self-calling on the
sub-structure:
```dag
fn ci_member(s: Symbol, xs: List<Symbol>) -> Bool {
  match xs {
    Nil                       => false
    Cons { head: h, tail: t } => match symbol_eq(a: s, b: h) {
      True  => true
      False => ci_member(s: s, xs: t)
    }
  }
}
```

**Transform** — replace with the substrate-derived combinator.
The lens's Clean shape (currently authored as prose) becomes an
executable mapping:
```dag
// pseudo machine-readable Clean shape
L1_5_transform(matched_node) -> Diff {
  let inner_pred = extract_arm_body(matched_node, variant: Cons)
  Diff([
    Edit {
      path: matched_node.path,
      replace_with: build_fold_application(
        primitive: list_any,
        list_arg: matched_node.list_param,
        predicate: synthesize_lambda(inner_pred)
      )
    }
  ])
}
```

**Pipeline** (candidate-state ordering — gates against post-edit):
```
matches = declarations_in(dag).flat_map(d =>
  apply_lens(L1.5, DeclarationScope(d), Introspect).matches()
)
diff = Diff(matches.map(m => L1_5_transform(m)))

candidate_dag = apply_diff(dag, diff)             // build uncommitted candidate
affected = affected_set(dag, diff)
affected.frontier.for_each(ref =>
  apply_lens(L1.5, scope_in(candidate_dag, ref), Enforce)   // candidate root explicit
)

dag := candidate_dag                              // commit if all gates passed
// per-target emit re-renders the affected files
```

Result: `ci_member`, `bs_member`, `ci_id_occurrences`, etc. all
become single-line fold applications. The substrate enforces the
clean shape going forward via the same L1.5 in `Enforce` mode.

### 6.4 Hero case (b): L1.12 canonical-B aliasing

Same shape as canonical-B in #3338, but expressed as the lens's own
auto-transform — with **honest treatment of the silence case**:
auto-fix only when canonical authority is structurally declared;
otherwise `NeedsDecision`.

> **Dependency caveat.** This case references `CanonicalConcept`,
> `ConceptDisambiguation`, and `HistoricalDeclaration` as substrate
> carriers. Today they exist as *design* in
> `docs/design-dissolution-lens.md` (PR #3334) but **NOT** as
> ratified `.dag` substrate. See §6.8 item 8 — this hero case is
> design-pending until those carriers are authored as substrate
> data rows.

**Find** — L1.12 outcomes (4) and (5):
- Outcome (4) **same-concept-without-alias-or-retirement**: a
  `CanonicalConcept` row EXISTS but the non-canonical declaration is
  neither aliased nor retired. The registry tells us which is
  canonical. Auto-transform is grounded.
- Outcome (5) **silence**: no `CanonicalConcept` row, no
  `ConceptDisambiguation` row, no `HistoricalDeclaration` row. The
  substrate has not declared which side is canonical. Auto-picking
  would be ungrounded inference, exactly what INVARIANTS forbids.

**Transform** — branching on which outcome fired:
```dag
L1_12_transform(matched_pair) -> ConditionalDiff {
  match outcome_of(matched_pair) {
    Outcome_4 {
      // CanonicalConcept row exists; READ canonical_home from it
      concept: CanonicalConcept,
      historical: NodeRef,
    } => Auto(Diff([
      // (4) → (1): add the alias-identity edge to the structurally-named canonical home
      Edit { path: historical.path, replace_with:
        import <concept.canonical_home_module> as canonical_alias
        type T = canonical_alias.T
      },
    ])),

    Outcome_5 {
      // silence — NO canonical authority declared. The substrate has
      // not taken a position; we MUST NOT pick one.
      pair: (NodeRef, NodeRef),
    } => NeedsDecision {
      because: no_canonical_authority,    // closed vocabulary
      hint: <None>,                       // both candidates plausible;
                                          // operator authors a CanonicalConcept row,
                                          // re-running the lens then re-fires outcome (4)
      needs: operator_authors_CanonicalConcept_row {
        candidates: pair,
      },
    },
  }
}
```

**Hero**: the operator-ratified canonical-B work in #3338 is the
worked example for outcome (4) — once `CanonicalConcept` rows for
Bool/Char/Url/etc. land structurally, the substrate could have
generated the alias edges automatically. The silence case
(outcome 5) is what blocked auto-fix during #3338 — that's the
correct shape: the operator authors the CanonicalConcept row first;
the alias is then automatic.

This is the general pattern for auto-fix: **transforms ground in
substrate-declared authority; absence of authority becomes
`NeedsDecision`, never an inferred guess.**

### 6.5 Hero case (c): interface cascade with conditional updates

The "find downstream affected lines, make conditional updates"
case from the operator's framing.

**Scenario**: `BooleanAlgebra<T>` gains a required `xor` operation
in `std/algebra.dag`. All existing `data ... : BooleanAlgebra<X>`
rows are now incomplete.

**Find** — consumers of the changed interface:
```dag
consumers = declarations_in(dag).flat_map(d =>
  apply_lens(consumers_of(symbol: BooleanAlgebra), DeclarationScope(d), Introspect)
    .matches()
)
// → List<Match { path, instance_type: X, current_fields: {...} }>
```

**Conditional transform per consumer** — this is where the "DAG of
edits" framing earns its keep:
```dag
diff_per_consumer(m) -> ConditionalDiff {
  let derived_xor = try_synthesize_xor_from(m.current_fields)
  match derived_xor {
    Some { fn } =>
      Auto(Diff([Edit { path: m.path, insert_field: xor = fn }]))
    None =>
      NeedsDecision {
        because: cannot_derive_xor_structurally,
        hint: Diff([Edit { path: m.path, insert_field: xor = pending_operator_xor }]),
        marker: 🟡_needs_operator_decision { interface_extension: xor }
      }
  }
}
```

Three buckets of consumers:
- **Auto-update**: `meet`/`join`/`complement` are present → derive
  `xor = (a ∨ b) ∧ ¬(a ∧ b)` structurally → land the Diff
- **Needs decision**: no derivable form → add a structured 🟡 marker
  with the closed-vocab reason; the lens fires next CI run until the
  operator authors a decision
- **Alias / unchanged**: re-exports of canonical homes don't need
  their own xor; propagation handled by the alias edge

The conditional transform is the substrate-acknowledged version of
"some sites I can fix; some need you." The 🟡 markers themselves are
substrate data, not comments — they fire in CI with structured
diagnostic until resolved.

### 6.6 Hero case (d): CLI-driven concept declaration

The agent-shape from the open question §7.5 below (agent-loop
composition at the user-program level — this CLI workflow is itself
a user program composing the substrate primitives, not gunbc
self-application).

**Scenario**: worker types
```bash
gunbc declare-concept Bool \
  --canonical-home v4.std.logic.Bool \
  --members dsl.std.types.Bool
```

**Substrate execution**:
1. CLI parses the invocation into a `ConceptDeclarationIntent`
   carrier (substrate data, not prose)
2. `intent_to_diff(intent)` synthesizes:
   - `data bool_concept: CanonicalConcept = { canonical_home: ..., members: ... }`
   - Alias-identity Edit for each `members` entry
3. `candidate_dag = apply_diff(dag, diff)` — uncommitted candidate
4. `affected_set(dag, diff).frontier.for_each(ref => apply_lens(L1.12, scope_in(candidate_dag, ref), Enforce))`
   validates **against the candidate** (candidate root structurally
   explicit) → should pass via outcome (1) alias for each affected
   declaration
5. `dag := candidate_dag` — commit if all gates pass
6. Re-emit affected files

The CLI invocation is the spec; the Diff is the convolution output;
the agent never touches a file directly. **This is the substrate-pivot
fully realized for one specific intent shape** — the same pattern
generalizes to every other "declare X, derive Y" intent.

### 6.7 Hero case (e): merge sort synthesis (the synthesis-shape case)

Distinguished from (a)–(d) because it's *synthesis*, not convolution.

**Scenario**: a `TestClaim` row declares:
```dag
data merge_sort_claim: TestClaim = {
  subject: <function-to-be-synthesized>,
  property: sorts_correctly_and_stable,
  cost_bound: complexity ≤ n_log_n,
}
```

**Substrate execution**:
1. Search the implementation space (LLM-driven; the substrate's job
   is to shape the search, not to solve it)
2. Each candidate Diff is validated via:
   - T-22 eval over the TestClaim's property → does it sort?
   - L1.5 catamorphism lens → is the recursion honest fold structure?
   - Cost lens → does complexity match `n_log_n`?
3. Iterate until all three gates pass
4. Land the winning Diff

The agent's job is the *search*; the substrate's job is the *gates*.
Once T-22 eval is live, this becomes operational; until then, the
shape is named but not runnable.

### 6.7b Hero case (f): mechanical refactor — declarative model transition

The **cleanest convolution shape** — judgment applied at
*command-selection* time, not at per-site application. The agent
declares "transition from model A to model B across the corpus"; the
substrate finds all matches and applies the same transform uniformly;
no per-site decisions needed.

**Distinguishing trait — zero per-site judgment**. Compare to the
other cases:
- (a) L1.5 catamorphism auto-fix — *one lens, one transform*
- (b) L1.12 canonical-B — *one lens, branches on outcome*
  (`Auto` vs `NeedsDecision`)
- (c) Interface cascade — *per-site conditional* based on consumer
  structure (`ConditionalDiff`)
- **(f) Mechanical refactor** — *declarative target, uniform per-site*;
  the agent picks the named refactor and the substrate guarantees
  uniform application

**Scenario**: "transition all `type LangBool = Bool` bare aliases to
canonical-B decl-ref grounding across all 6 languages." The worked
example is exactly PR #3338's canonical-B work, executed as a single
mechanical refactor rather than a hand-edited PR.

**Read** — find all matches via a pattern-match lens (compose
existing L1.x's `Signature` half):
```dag
matches = declarations_in(dag).flat_map(d =>
  apply_lens(bare_alias_pattern_lens, DeclarationScope(d), Introspect)
    .matches()
)
// → List<Match { path, type_name, aliased_to }>
```

**Affected-structural-paths enumeration** — pre-execution, the agent
can inspect the exact scope structurally (not by grep):
```dag
preview = {
  site_count:    matches.length,
  exact_paths:   matches.map(m => m.path),    // structural Paths in the Node graph
  re_exec_scope: affected_set(dag, refactor_diff).frontier,
}
```
This answers *"what are all the affected structural sites"* — a
substrate-native enumeration of `Path`s + re-validation scope,
available **before** any Edit applies.

> **Translation to file/line ("affected LOC") depends on the
> Node→File binding registry (§6.8 item 6).** Until that registry
> lands, the substrate-native answer is in terms of `Path`s, not
> file:line pairs. Per §1, file/line is an *emergent* property of
> emit behavior today — to project from structural Path to
> file:line you'd need to walk the emit stage's mapping (implicit)
> or wait for the Node→File binding registry to make it queryable
> substrate data. The substrate-native answer is still complete —
> "affected sites" is a structural fact; "affected LOC" is the
> file/line projection of that fact, downstream of emit.

**Transform** — uniform per-site, declaratively expressed:
```dag
refactor_bare_alias_to_decl_ref(matches) -> Diff {
  Diff(matches.map(m =>
    Edit { at: m.path,
           replacement: build_decl_ref_node(canonical_authority: m.aliased_to) }
  ))
}
```
No conditionals; every match gets the same shape of replacement.

**Migration guarantee — the candidate-state pattern from §4 IS the
guarantee:**
1. `candidate_dag = apply_diff(dag, refactor_diff)` — fail-closed
   Diagnostic if any Edit's Path doesn't resolve; whole refactor
   bails atomically, no partial state.
2. `affected_set(dag, refactor_diff).frontier.for_each(ref => apply_lens(L1.7, scope_in(candidate_dag, ref), Enforce))`
   per relevant lens (candidate root structurally explicit) —
   fail-closed if any new violations introduced by the candidate.
3. `dag := candidate_dag` only if every gate passed.

**The guarantee is structural**: either the refactor lands completely
or no-op. Never a half-migrated state. Site count is irrelevant —
the substrate handles 10 sites or 10,000 the same way. Atomicity is
a property of `apply_diff` + candidate-state, not of the refactor
authoring.

**Hero**: PR #3338 (canonical-B across 6 languages + 7 v3 ratchet
dissolutions) is the worked example for this shape. Judgment applied:
- "use decl-ref for Bool" (one decision)
- "dissolve the 7 v3 ratchets" (one decision)

The 13 affected sites (6 langs + 7 ratchets) plus the cascading
INVARIANTS / sg0_census / integration.rs changes were **uniform
per-class** — no per-site judgment. The substrate (had it been
operational) could have applied the entire refactor mechanically from
those two decisions.

**Composition with existing cases**: a mechanical refactor often
*decomposes into* per-lens auto-fixes from §6.2's catalog. The
canonical-B refactor decomposes into L1.7 transforms (introduce
witness rows) + L1.12 outcome (4) transforms (alias the historical
declarations). The agent's job is **picking the named refactor**;
the substrate's job is **composing the per-lens transforms** that
implement it.

**Where this differs from §6.6 CLI declaration**: §6.6 is *single
intent → single concept*; mechanical refactor is *single intent →
named class-wide transition*. Both are agent-shape, but mechanical
refactor scales by enumeration over the matched set; CLI declaration
scales by composing within a single intent shape.

### 6.8 What's missing substrate-side to make this operational

The convolution view is implicit today. To make it executable:

1. **Machine-readable Clean shape per lens** — currently prose +
   example. Author each L1.x's Clean shape as `transform_fn(Matched)
   → Diff` substrate data. The §6.2 table is the spec for this.
2. **Conditional/branching transforms** — for cases like §6.5 where
   the right transform depends on the consumer's structure. ADT shape:
   `ConditionalDiff = Auto(Diff) | NeedsDecision { because: <closed-vocab>, hint: Diff? } | NotApplicable`.
3. **DAG-of-edits composition** — sequential `List<Edit>` works for
   small cases; non-local cascades need a partial-order graph (some
   Edits must precede others). Natural extension; doesn't replace
   `apply_diff`'s fail-closed semantics.
4. **Intent-shaped declaration carriers** (for §6.6) — `ConceptDeclarationIntent`,
   `AlgebraExtensionIntent`, etc. as substrate data + `intent_to_diff` as
   a fold. This is Track 2 from #3313 generalized.
5. **Search loop primitive** (for §6.7) — once T-22 eval is live,
   compose into a synthesize-validate-iterate loop. Heaviest dep.
6. **Node→File binding registry** — `data ... : NodeToFileBinding =
   { node: ..., file: ..., region: ... }` rows so file-tying is
   queryable substrate data, not emergent emit behavior. Today
   `file_system.dag` carries only POSIX file ops; the rendering
   binding is implicit in the emit stage. Making it explicit closes
   the "files are a downstream effect of substrate state" framing —
   that effect becomes a structurally-recorded fact, not just a
   compile-time side effect.
7. **Library wrappers for the substrate primitives** — the
   agent-surface layer. See §6.10 for the commitment + §6.11 for the
   layered dependency order.
8. **L1.12 concept-identity registry carriers** — the §6.4 hero case
   (b) and the L1.12-transform examples reference `CanonicalConcept`,
   `ConceptDisambiguation`, and `HistoricalDeclaration` as substrate
   data rows the lens consumes. Today these carriers exist as
   *design* in `docs/design-dissolution-lens.md` (PR #3334, operator
   manual-merge queue) but **NOT** as ratified `.dag` substrate.
   Until the design lands AND the carriers are authored as
   `data ... : <CarrierType>` rows in some substrate-owned `.dag`
   file (likely `std/` or a new `lens/concept_identity.dag`), the
   §6.4 examples are *design-pending*, not runnable. Concretely:
   - `data <X>_concept: CanonicalConcept = { canonical_home, members }`
   - `data <X>_retired: HistoricalDeclaration = { type, dissolves_when }`
   - `data <X>_distinct: ConceptDisambiguation = { names, because }`

### 6.9 Recommended ordering for hero demonstrations

Easiest-and-most-illustrative to hardest, if you want to build them:

1. **(a) L1.5 ci_member auto-fix** — clean unambiguous transform,
   small cascade, no operator judgment. Best first demo.
2. **(b) L1.12 canonical-B aliasing** — aligned with #3338 already
   merged; transform branches on outcome (Auto vs NeedsDecision).
3. **(f) Mechanical refactor** — declarative model-A → model-B
   transition; zero per-site judgment; the substrate guarantees
   atomic application or no-op. Composes per-lens transforms from
   §6.2 catalog. PR #3338 is the worked example (canonical-B + 7
   ratchet dissolutions). Most directly useful hero shape for
   day-to-day refactoring work.
4. **(c) Interface cascade (BooleanAlgebra + xor)** — conditional
   updates per consumer; introduces the `ConditionalDiff` carrier.
5. **(d) CLI-driven concept declaration** — most agent-shaped; uses
   the most new substrate (intent carriers).
6. **(e) Merge sort synthesis** — heaviest; needs T-22 eval and
   cost-lens-as-constraint live.

(a)–(d) and (f) are convolutions; (e) is synthesis. Distinct enough
that (e) probably warrants its own design doc when picked up.

### 6.10 Agent-surface — library-first commitment

The agent-side surface for the read/edit pipeline is **library
calls**. Three implications:

1. **All substrate primitives surface as library functions** —
   `apply_lens`, `apply_diff`, `subterm_at`, `affected_set`,
   `declarations_in` are library calls. Not RPC services. Not
   CLI-first. The Rust binding is the in-substrate today; once v4
   self-hosts, these become `.dag` intrinsics callable from
   user-program `.dag`.
2. **CLI wraps libraries** — `gunbc apply-lens ...`,
   `gunbc auto-fix ...`, `gunbc refactor ...` etc. are thin shells
   that compose library calls. Adding a new CLI subcommand is
   composition, not new substrate.
3. **The library boundary IS the agent-substrate surface.** No
   premature transport layer (JSON-RPC, gRPC) — the agent loop is
   function calls. Network transport, if ever introduced, is its
   own concern that doesn't pre-date library landings.

**First concrete I/O workflow — auto-fix-for-lens-error:**

The most directly useful agent-shape orchestration; concretizes the
§4 seven-step pipeline for one specific input (a lens-fired
diagnostic):

```
auto_fix_for_lens(lens, scope) -> AutoFixOutcome
  1. Lens error: apply_lens(lens, scope, Enforce) returns Diagnostic
                 (the trigger — the agent observes a fail-closed lens)
  2. Search:     agent picks transform — machine-readable Clean shape
                 per lens (§6.8 item 1) or hand-authored for the
                 specific case
  3. Verify:     candidate_dag = apply_diff(dag, transform_diff)
                 affected_set(dag, transform_diff).frontier.for_each(ref =>
                   apply_lens(lens, scope_in(candidate_dag, ref), Enforce)
                 )                                          # candidate root explicit per §4
  4. Fix/Edit:   dag := candidate_dag (atomic commit)
  5. Re-emit:    per-target emit re-renders files
```

§6.3 hero case (a) L1.5 catamorphism auto-fix is the first concrete
instance — `auto_fix_for_lens` folded over `declarations_in(dag)`
(i.e. `declarations_in(dag).for_each(d => auto_fix_for_lens(L1.5, DeclarationScope(d)))`),
swapping recursive `match` shapes for `list_any` / `traverse` fold
primitives. Per §2, corpus-wide application is the fold over the
declaration set; `auto_fix_for_lens` itself takes a `SectionRef`
(`DeclarationScope` or `NodeScope`) — never an invented `RootScope`.

`AutoFixOutcome` is a structural carrier reporting either
`AppliedClean { sites: N, diff: Diff }` (every match auto-fixed
cleanly) or `PartiallyApplied { sites: N, needs_decision: List<NeedsDecision> }`
(some matches required operator judgment, structurally recorded).

### 6.11 Dependent build order — layered implementation sequence

The library-first commitment implies a layered dependency order.
**Each layer builds on the prior.** The build sequence below
is the dependency-correct one for getting from today's
scaffold-only state to operational auto-fix-for-lens-error:

**Layer 0 — Substrate primitives (T-23 fills today's scaffold).**
- `apply_lens(lens, section, config)` and `apply_diff(root, d)` in
  `src/v4/lens/application.dag`
- `subterm_at(root, p)` (same file)
- `Path`, `Edit`, `Diff` types ratified in `src/v4/std/node.dag`
  (PR #3162, already on main)
- `affected_set(dag, diff)` in `src/v4/lens/affected_set.dag`
- All scaffold today; T-23 is the named lane that fills them.

**Layer 1 — Library wrappers (depend on Layer 0).**
- Thin language bindings to invoke the Layer-0 primitives. Rust
  crate today (consumed by v3 + tooling); `.dag` intrinsics once
  v4 self-hosts.
- Helpers: `declarations_in(dag)` corpus fold, plus the
  `affected.frontier.for_each(...)` idiom captured as a fold
  combinator.
- This is what §6.8 item 7 names.

**Layer 2 — Per-lens transform Clean shapes (depend on Layer 1 +
§6.8 item 1).**
- Machine-readable `transform_fn(Matched) → Diff` (or
  `→ ConditionalDiff` for branching cases) for each L1.x.
- §6.2 table is the spec for what these need to be.
- Authored in parallel — each per-lens transform is independent
  modeling work.

**Layer 3 — Orchestration wrappers (depend on Layers 0–2).**
- `auto_fix_for_lens(lens, scope) → AutoFixOutcome` — the §6.10
  workflow as a single library call.
- `mechanical_refactor(refactor_name, scope) → RefactorOutcome` —
  the §6.7b hero case (f) as a library call.
- Built on top of `apply_lens` + `apply_diff` + the per-lens
  Layer-2 transforms.

**Layer 4 — Composition wrappers (depend on Layers 0–3).**
- `interface_cascade(interface_change, conditional_fn) → CascadeOutcome`
  — §6.5 hero case (c) as a library call. Introduces
  `ConditionalDiff` carrier (§6.8 item 2).
- `intent_to_diff(intent) → Diff` — §6.6 hero case (d) as a
  library call. Consumes intent-shaped declaration carriers
  (§6.8 item 4).

**Layer 5 — CLI wrappers (depend on Layers 1–4).**
- `gunbc apply-lens <lens> --scope <scope> [--enforce]` — Layer 1
- `gunbc auto-fix <lens> [--scope <scope>]` — Layer 3
- `gunbc refactor <named-refactor>` — Layer 3
- `gunbc cascade <interface-change>` — Layer 4
- `gunbc declare-concept <name> ...` — Layer 4
- All are thin shells; the substantive logic is in the libraries.

**Layer 6 — Synthesis loop (depends on T-22 eval + Layers 0–3).**
- `synthesize_from_claim(test_claim) → Diff` — §6.7 hero case (e).
- Heaviest dep: requires T-22 eval live + cost-lens-as-constraint.
- Distinct enough that it warrants its own design doc when picked up.

**Critical-path for first-functional system (auto-fix L1.5 demo):**

```
Layer 0 (apply_lens + apply_diff + affected_set scaffolds filled — T-23)
     ↓
Layer 1 (Rust crate wrapping the primitives)
     ↓
Layer 2 (just L1.5's transform — one row from §6.2)
     ↓
Layer 3 (just auto_fix_for_lens orchestration)
```

That's the minimum dependency closure for the first runnable demo —
auto-fixing `ci_member` and siblings to `list_any` fold applications.
Layer 5 CLI is convenience; Layer 4 composition and Layer 6
synthesis are out-of-scope for first demo.

**Parallelizable within each layer:**
- Layer 2 transforms are mutually independent — multiple workers can
  author per-lens Clean shapes in parallel
- Layer 4 composition wrappers are independent of each other (modulo
  shared Layer 2/3 deps)
- Layer 5 CLI subcommands are independent (modulo shared library
  surface)

**Serial within the layer stack:** Layer N depends on Layer N-1; no
shortcut from Layer 3 to Layer 0. Each new layer is a layer of
agent-surface authority, not a parallel pseudo-authority — same
single-authority discipline the substrate enforces.

## 7. The hard part — remaining open interface questions

The mechanical primitives are ratified. The **interface** is the part
the operator named as hard. Six open design questions, each its own
follow-up:

1. **Higher-order Edit combinators.** Today `Diff = List<Edit>` is
   sequential primitives. Refactors like Example A naturally express
   as "apply *this transform* to every site matching *this lens*."
   Combinator candidate: `transform_via_projection(lens, transform_fn): Diff`.
   Implementation TBD.

2. **Edit composition rules under overlap.** When two Edits affect
   overlapping Paths, the order matters and Path resolution can fail.
   Is there a higher-level way to express "apply this set of Edits,
   retrying / re-pathing as needed"? Or stay sequential-fail-closed
   and require the caller to order correctly?

3. **Intent-shaped declarations** (generalize Track 2 of #3313).
   Declaring an algebra gives operations for free; generalize to:
   the agent declares concept-level intent (`CanonicalConcept`
   membership, refinement clause, inhabitance edge), substrate
   derives the structural artifacts. **Write the concept; the code
   follows.**

4. **LLM-targeted diagnostic shape.** Diagnostics today are typed
   for human/structural review. An LLM-shaped variant produces
   *next-action hints* in the same structural carrier — e.g., "L1.5
   fired; the clean shape is `traverse_outcome(xs, ...)`; here's the
   Diff that would land it." Self-suggesting rewrites.

5. **Agent-loop composition at the user-program level.** The read →
   diagnose → propose → gate → apply → re-emit loop is currently
   orchestrated by shell + LLM. The substrate already exposes the
   primitives (`apply_lens`, `apply_diff`, `affected_set`, emit) for
   a *user program* to compose its own agent loop as data. **This is
   a user-program concern, not gunbc self-application.** THESIS
   (2026-05-15 retraction of meta-process / work-direction modeling)
   narrowed gunbc's own `workflow/` surface to `{ bootstrap, ci }`;
   the agent loop is *not* an extension of that, it's a downstream
   consumer that uses the same primitives any user program uses.
   Open question: what user-program-side carriers (intent shapes,
   decision points, audit traces) ship with gunbc as helpful
   conveniences vs are left for user programs to author themselves?
   Either way, **`workflow/agent_loop.dag` is not the right place**;
   the THESIS narrowing stands.

6. **Read/write provenance traces.** When an agent modifies code, an
   honest *structural* trail of "lens X said Y, so Diff Z was
   applied" — substrate-side, not commit-message-side. Makes audit +
   rollback structural rather than textual.

## 8. What this doc isn't

- **Not a new framework.** The substrate-pivot principle holds: no
  LLM-specific nouns. Same Nodes, same lenses, same primitives —
  agents and humans are *both* consumers of the same structural
  language.
- **Not a replacement for T-23.** `lens/application.dag` is the
  operations / types file; this doc is the design rationale + worked
  examples.
- **Not an implementation plan.** The open questions in §7 are
  *interface design questions* — implementation follows once the
  interface is clear. The §6 hero demonstrations are illustrative,
  not specifications — they show the convolution shape but the
  authoring of machine-readable transforms is its own work.

## 9. Status / open

- **Vocabulary** (`Path` / `Edit` / `Diff` in `std/node.dag`):
  **ratified** per PR #3162.
- **Operations** (`apply_lens` / `apply_diff` in
  `lens/application.dag`): **scaffold** today; T-23 fills them.
- **Composition** (T-23 + T-21 affected_set + B2-OMNI emit + C5/C4
  faithful re-emit): **named contract**, scaffold today.
- **The 6 open interface questions**: design-side, awaiting operator
  direction. Each can become its own follow-up doc / PR when ready.

When the operator picks up any of the six, the next artifact is
either (a) a focused design doc for that question, or (b) a worker
brief dispatching the implementation against the question's chosen
shape. This doc is the framing, not the resolution.

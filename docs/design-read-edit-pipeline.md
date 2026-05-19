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

The substrate models File explicitly via
`src/v4/extdeps/file_system.dag` so file-tying is a *structural fact*
("Node N is rendered into File F by emit, at region R") — not implicit
text coupling.

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
`affected_set(dag, diff)` produces a `Witness<ReExecFrontier>`; that
frontier is itself a scope, so other lenses can run AT that frontier
("which complexity lenses need to re-run given this Diff?").

## 3. The write interface

Per Path/Edit/Diff (`src/v4/std/node.dag`, ratified PR #3162):

- **`Path`** — a structural address to a sub-Node. Not a filesystem
  path; the Node graph's own coordinates.
- **`Edit`** — a structural rewrite at a Path. Replace / insert /
  delete at a position determined by the substrate's connective
  family.
- **`Diff = List<Edit>`** — ordered sequential rewrite program. The
  list ordering is significant — Edits compose by sequential
  application, NOT parallel.

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
loop is six steps, closed:

```
1. Read     →  apply_lens(lens, scope, Introspect)        # gather structural facts
2. Diagnose →  reasoning over the facts (LLM or human)    # decide what should change
3. Propose  →  produce Diff                               # express change as structural edits
4. Gate     →  apply_lens(_, Enforce) on affected_set     # validate fail-closed
5. Apply    →  apply_diff(dag, Diff)                      # land the edits
6. Re-emit  →  emit per target language                   # files are a downstream effect
```

Step 6 is the **only** place files re-enter the picture — they are a
*consequence* of substrate state, not a cause.

The pipeline is closed: any Diff that doesn't pass the gates doesn't
land, period. There is no "force apply." The agent earns no special
trust; structure gates it.

## 5. Worked examples

Three concrete .dag-level scenarios showing read-then-edit shape.

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

**Gate** — run dissolution lenses on the affected frontier:
```dag
affected = affected_set(dag, diff)
apply_lens(L1.7, NodeScope(affected), Enforce)
  // → Outcome<()>; fails if any new prose-asserted facts introduced
apply_lens(L1.10.b, NodeScope(affected), Enforce)
  // → Outcome<()>; checks no String-escape-hatch introduced
```

**Apply**:
```dag
new_dag = apply_diff(dag, diff)
```

**Re-emit** — `rust.dag` re-renders to Rust source per target emit.
The file change is an *effect*, not the cause.

### Example B: rename a concept across the corpus

Goal: rename `Bool` → `BooleanValue` everywhere it's referenced. The
`CanonicalConcept` registry is the source of truth; the Diff cascades.

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

## 6. The hard part — open interface questions

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

5. **Workflow-as-data for the agent loop.** The read → diagnose →
   propose → gate → apply → re-emit loop is currently orchestrated
   by shell + LLM. Self-application would model the loop in
   `workflow/agent_loop.dag`. THESIS / #3322 narrows self-application
   to `workflow/{bootstrap, ci}`; extending to `agent_loop` would
   close this gap.

6. **Read/write provenance traces.** When an agent modifies code, an
   honest *structural* trail of "lens X said Y, so Diff Z was
   applied" — substrate-side, not commit-message-side. Makes audit +
   rollback structural rather than textual.

## 7. What this doc isn't

- **Not a new framework.** The substrate-pivot principle holds: no
  LLM-specific nouns. Same Nodes, same lenses, same primitives —
  agents and humans are *both* consumers of the same structural
  language.
- **Not a replacement for T-23.** `lens/application.dag` is the
  operations / types file; this doc is the design rationale + worked
  examples.
- **Not an implementation plan.** The open questions in §6 are
  *interface design questions* — implementation follows once the
  interface is clear.

## 8. Status / open

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

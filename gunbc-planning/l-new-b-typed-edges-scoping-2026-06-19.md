# L-NEW-b — Typed graph edges in the core Node model (scoping proposal)

> **Status:** SCOPING / PROPOSAL for the **gunbc substrate lane** — not yet greenlit for
> implementation. This document hands a load-bearing core-substrate decision to
> `node.dag` / `dependency.dag` / T-9 resolve owners; it is **not** an implementation PR.
> Parent context: ctrl Track-A access model §2.4 propagation semantics —
> [gunb-ai/ctrl#1732 §2.4](https://github.com/gunb-ai/ctrl/pull/1732) (containment vs reference
> edges; fail-closed grant scope).

---

## 1. Problem — the leak

Today `v2.std.node.Edge` is `{ label, target }` only. Structural shape is governed by
`EdgeDiscipline` (labeled vs positional per Connective/Behavior), but **containment vs reference
is not a substrate fact** — it is inferred post-hoc in `v2.std.dependency` via
`classify_named_edge_usage`, which defaults **unrecognized named edges and all positional edges to
`Contains`**.

`fold_node` and `dependency_lens` traverse **all** `children` uniformly. Any subtree read
therefore implicitly treats every edge as structural containment unless a magic symbol
(`^dependency_binds_to_edge`, etc.) says otherwise. That is the fail-open leak: **seeing B can
auto-grant A's enclosing context** — exactly the propagation hazard ctrl §2.4 names.

---

## 2. Current authorities (parallel, not unified)

| Layer | What it models | Authority status |
|-------|----------------|------------------|
| `v2.std.node.Edge` | `label` + `target` | substrate primitive — **no role** |
| `EdgeDiscipline` | arity/labeling rules per connective/behavior | substrate — shape only |
| `v2.std.dependency.DependencyKind` (14 variants) | `Contains`, `BindsTo`, `TypeDependsOn`, … | **derived** by symbol-heuristic classifier |
| v1 `Node.children` vs `Node.uses` | structural vs reference split | v1 Rust only; v2 collapsed to `children` |
| Lenses (`unused_parameters`, `ownership`, `affected_set`) | map `DependencyKind` → domain classifiers | consumers of derived view |

Key consumer receipts already distinguish `Contains` vs `BindsTo` semantically:

- **`unused_parameters`:** `Contains` ⇒ structural composition; `BindsTo` ⇒ use-of-declaration
- **`affected_set/irt1`:** `Contains` propagates despite child read-receipt; non-`Contains` needs
  receipt to exclude
- **`ownership`:** `Contains` ⇒ `OwnedContainment`
- **`application.dag`:** `SectionRef`/`NodeId` needs node-containment evidence (gated
  `feature:section-ref-identity-evidence`)

Dissolve trigger already named in-tree:
`feature:dependency-usage-classifier-consumes-resolve-ground-facts` — T-9 resolve should stamp
`^dependency_binds_to_edge` / module / bootstrap labels inline; the classifier is interim.

---

## 3. Proposed substrate direction (model-before-implement)

**Single authority on the carrier; projection everywhere else.**

### 3.1 Minimal closed `EdgeRole` coproduct on `Edge`

Two top-level arms (ctrl §2.4 alignment):

```dag
type EdgeRole
  = Containment { inheritable: Bool }
  | Reference { kind: ReferenceKind }
```

- **`Containment { inheritable: Bool }`** — A contains B; grants **may** flow down only when
  `inheritable = true` (opt-in on the edge/grant; **default `false` = fail-closed**).
- **`Reference { kind: ReferenceKind }`** — B depends on A; traversal/read of B must **not**
  expand grant scope to A.

`ReferenceKind` should **refine**, not re-invent, the existing non-`Contains` `DependencyKind`
variants (`BindsTo`, `TypeDependsOn`, `DataDependsOn`, …). Horizontal move: one `Reference` arm +
existing kind enum — not 14 parallel edge types.

### 3.2 `dependency_lens` becomes projection

Replace `classify_named_edge_usage` (heuristic default-to-`Contains`) with
`dependency_view_from_edge` projecting from `edge.role`. Unknown labels on `Reference` edges fail
closed to a typed diagnostic, not silent `Contains`.

### 3.3 Parameterized traversal

- `fold_node` stays total-tree (dependency analysis, emission, etc.).
- Add **`fold_containment`** (role-filtered catamorphism) for grant-scope / `section_subject` /
  readiness walks.
- Reference edges remain reachable via explicit `fold_references` or full `fold_node` when
  dependency analysis intends it.

### 3.4 Grant inheritance

Grant inheritance is a **policy layer on `Containment`**, not a third edge type: `inheritable:
Bool` on the containment arm ties to lens-application opt-in depth (`SectionRef`). Fail-closed
default: `inheritable = false`.

---

## 4. Touch surface (implementation phase — escalate before edit)

**Load-bearing — substrate owners must sign off before edits:**

| File / area | Why load-bearing |
|-------------|------------------|
| `src/v2/std/node.dag` | `Edge`/`EdgeShape` type, `well_formed`, `content_hash` canonicalization, `EdgeDiscipline` interaction |
| `src/v2/std/dependency.dag` | classifier → projection |
| `src/v2/std/node_shape.dag` | `EdgeShape` stamp bridge |
| T-9 resolve/parse | stamp roles at construction (replaces magic symbols) |
| `fold_node` call sites assuming all-children = containment | `dependency_lens`, `affected_set` ancestor frames, `ready_set`, scheduler |

**Downstream (mechanical once projection lands):**

- `unused_parameters`, `ownership`, `affected_set`, structural_resolution fixtures, scheduler,
  `ci_floor_plan`

**v1 alignment:** `dsl/std/node.dag` declares separate `children` + `uses` inductive fields — v2
re-split vs role-on-`Edge` is a de-fork decision; role-on-`Edge` is fewer concepts than
reintroducing `uses` as a second list.

**Out of scope for substrate lane (separate follow-on):**

- Grant/policy semantics beyond the `inheritable` flag (lens-application + import-visibility
  coupling)
- `feature:section-ref-identity-evidence` — Path-backed `DeclarationId`/`NodeId` should consume
  typed containment, not Path heuristics
- `content_hash` must include role in merkle fold (canonicalization receipt needed when implemented)

---

## 5. Suggested implementation sequence (substrate lane — not greenlit here)

| Item | Scope |
|------|-------|
| **L-NEW-b.1** | Substrate spec: `EdgeRole` + `ReferenceKind` refinement in `v2.std.node` (types + `well_formed` gates only; no parser) |
| **L-NEW-b.2** | `fold_containment` / role-filtered catamorphism + leak witness (RED: reference child read must not appear in containment fold) |
| **L-NEW-b.3** | Dissolve `classify_named_edge_usage` → `dependency_view_from_edge` projection; migrate structural_resolution fixtures off magic symbols |
| **L-NEW-b.4** | Grant inheritance on `Containment` + `SectionRef` containment evidence adapter |

Each item lands with a named dissolution trigger and a discriminating witness (DESIGN §5).

---

## 6. Consistency checks (DESIGN.md)

- **§2 horizontal:** one `EdgeRole` concept; `DependencyKind` projects from it — no parallel
  nicknames.
- **§3 single authority:** role lives on `Edge`; lenses consume projection, not re-infer.
- **§5 fail-closed:** default `inheritable = false`; reference traversal must not expand grant
  scope; witness required before calling done.

---

## 7. Open questions for substrate owners

1. **`ReferenceKind` placement:** co-locate with `DependencyKind` in `dependency.dag` vs duplicate
   closed enum in `node.dag` to avoid import cycle — pick one authority.
2. **Positional reference edges:** do any exist in production graphs, or are positional children
   always containment? Affects T-9 stamping rules.
3. **`content_hash` migration:** role in merkle fold changes digests — batch golden updates vs
   staged rollout.
4. **Bind-body special case:** today `^dependency_binds_to_edge` under `Bind` behavior maps to
   `Contains` heuristically — with explicit roles, parse should stamp containment at construction
   instead.

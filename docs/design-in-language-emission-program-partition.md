# Design: In-Language Emission + Program Partition

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). No load-bearing
> stage edits land from this doc without the consumers named in §6 (E-10). This is the contract
> for making **emit a first-class `.dag` operation over arbitrary program segments**, with
> **target as an explicit value** — the operator sketch `emit_rust(dag_tree())` means "call
> emit from a `.dag` program on whatever subtree you chose," not "drive emit through a CLI
> flag."
>
> **Deliverable (this session):** this doc + modeled carriers in
> `src/v4/std/program_segment.dag` and `src/v4/compiler/program_partition.dag` (fail-closed
> reason symbols + scaffold resolution for `WholeProgram` / `SubtreeAtPath` / `SubtreeRoot`).

## 1. Problem

Today emit exists in `.dag` but only at **whole-program** granularity:

```dag
// src/v4/compiler/05_emit.dag — frozen composition form (M11 receipt)
fn emit(tree: InferredTree, target: TargetModel) -> Outcome<TargetSource>
```

Callers must construct a full `InferredTree { root, facts }` and pass a `TargetModel` value.
That is already "target as value" (no hidden Rust `LOADED_SPEC` — CODING.md). What is **missing**:

1. **Segment selection** — no substrate way to say "emit *this* `Node` subtree inside a larger
   program." COMPREP wave 3 needs body-only emit; test claims need partial witnesses; self-host
   needs to emit a stage slice without re-inferring the universe.
2. **In-language surface** — emit is importable (`mvp1_*_add_translate.dag` already calls
   `emit(...)`), but there is no ergonomic **partition-then-emit** API beside hand-building a
   fresh `InferredTree` with a truncated `facts` map.
3. **Operator sketch decoding** — `emit_rust(dag_tree())` is two facts, not a CLI:
   - `dag_tree()` → the ambient program as data (`InferredTree` or a value that lowers to one);
   - `emit_rust(...)` → emit with the **Rust `TargetModel` row as an explicit argument**, not a
     side channel. The `_rust` suffix is **target-row selection sugar**, not a separate emitter.

The v2 anti-pattern (`compile.dag` `match artifact.target { Rust => emit_rust ... }`) hard-wires
target dispatch to an enum roster. v4's derived-homomorphism form keeps **one** `emit` and passes
`TargetModel` data (`rust_mvp1_target_model`, `mvp1_python_target_model`, …).

## 2. What already exists (M9 DFS — extend, don't coin)

| Piece | Where | Role |
|---|---|---|
| `emit = serialize_target ∘ translate` | `src/v4/compiler/05_emit.dag` | stage composition; **consumer** of `InferredTree` |
| `InferredTree { root, facts }` | `src/v4/compiler/04_infer.dag:96` | infer output; facts map keyed by `Node` |
| `TargetModel` bundle | `src/v4/std/compilers/target_model.dag` | target as explicit value (grammar rows, inhabitants, …) |
| `TargetSource` | `src/v4/compiler/07_target_carriers.dag` | emitted text carrier |
| `Path` + `subterm_at` | `std/node.dag`, `v4/lens/application.dag` | structural subtree selection (lens section projection) |
| `SectionRef` | `v4/lens/application.dag:67` | declaration / node scope handles (Path-backed, 🟡 identity evidence) |
| `ChangedSubgraphFrontier` | `v4/lens/affected_set.dag` | **diff-driven** subgraph (incremental re-exec), orthogonal axis |
| Eval subgraph MVP | `eval_runtime_mvp.dag` | arbitrary `Node` as eval root — proves runtime can target a subgraph |
| `ProjectionKind::EmitProjection` | `src/v4/std/projection.dag:15` | projection-as-data names emit; no partition carrier yet |

**Substrate target named (P1):** segment selection lands in new `ProgramSegment` /
`ProgramPartition` carriers (this PR). **No** new emit stage, **no** per-target `emit_<lang>`
substrate functions, **no** CLI surface.

## 3. The design in one paragraph

**Program partition** resolves a `ProgramSegment` against an ambient `InferredTree` to a
`ProgramPartition { ambient, segment, root, boundary }`, then **projects** a emit-ready
`InferredTree` by re-rooting at `root` and carrying the **facts closure** for nodes the
translate/serialize fold will visit. **Emit** stays one function; partition is the new
first-class input shaping:

```
emit_for_target(program, segment, target)
  = emit(partition_inferred_tree(resolve(program, segment, boundary)), target)
```

`emit_rust(x)` is documented sugar for `emit_for_target(x, WholeProgram, rust_*_target_model)` —
three arguments, all values, callable from any `.dag` module. Slicing is **data** (`ProgramSegment`
variants), not argv parsing.

## 4. Mechanism

### 4.1 `ProgramSegment` — how to name the slice (substrate-owned)

Declared in `src/v4/std/program_segment.dag`:

```
type ProgramSegment
  = WholeProgram                          // today's emit(tree.root) behavior
  | SubtreeAtPath { path: Path }          // select from ambient.root via Path.steps
  | SubtreeRoot { root: Node }            // emit this Node as translation root
```

Rules:

- **Closed coproduct (M4).** No stringly segment specifiers; illegal shapes are unrepresentable.
- **No `SectionRef` in `std/`.** Lens-owned `SectionRef` maps to `ProgramSegment` in the
  compiler adapter (`program_segment_from_section`) to avoid `std → lens` dependency inversion.
- **`SubtreeRoot` does not prove containment.** Resolution against `ambient.root` is a separate
  fail-closed check (§4.3) so arbitrary nodes cannot silently hijack a foreign program's facts.

### 4.2 `ProgramPartition` — resolved slice + boundary policy (compiler-owned)

Declared in `src/v4/compiler/program_partition.dag`:

```
type ProgramPartition {
  ambient: InferredTree
  segment: ProgramSegment              // provenance: how root was chosen
  root: Node                           // resolved emit root
  boundary: PartitionBoundaryPolicy
}

type PartitionBoundaryPolicy
  = FailClosedOnFreeRefs               // default (C-8)
  | AmbientContextAvailable            // module-in-context: ambient declarations visible
```

**`partition_inferred_tree(partition) -> InferredTree`** is the emit adapter:

- `root` ← `partition.root`
- `facts` ← **closure projection** of `partition.ambient.facts` over nodes reachable from
  `root` under the translate/serialize catamorphism (M11: the closure walk is a fold, not
  hand-rolled stage logic)

**Scaffold honesty (landed this session):** closure projection is **not** implemented — the
scaffold copies the full ambient `facts` map and gates only segment **root resolution**. The
`partition_facts_not_closed` fail-closed reason is reserved for the closure fold consumer
(§6.1). This matches INVARIANTS "map vs territory": types + fail-cl symbols are territory;
closure filtering waits for a running consumer.

### 4.3 Resolution — fail-closed (C-8 / P3)

`resolve_program_partition(program, segment, boundary) -> Outcome<ProgramPartition>`:

| Check | Failure reason | When |
|---|---|---|
| Path does not resolve from `program.root` | `partition_segment_unresolvable` | `SubtreeAtPath` |
| Path resolves ambiguously (duplicate named edges) | `partition_segment_ambiguous` | `subterm_at` rejection |
| `SubtreeRoot` not contained in `program.root` | `partition_root_not_in_ambient` | containment fold (scaffold: syntactic identity / path witness — 🟡) |
| Resolved root is error/empty node | `partition_empty_segment` | well-formed gate |
| Free reference escapes segment under `FailClosedOnFreeRefs` | `partition_boundary_free_ref` | closure/boundary fold (§6.1) |
| Translate fold needs facts for node N, N ∉ closure | `partition_facts_miss` | closure projection (§6.1) |

No silent truncation, no "best effort" emit of a dangling subtree.

### 4.4 In-language API (canonical + sugar)

**Canonical** (compiler module — one authority, P2):

```
fn emit_for_target(
  program: InferredTree,
  segment: ProgramSegment,
  target: TargetModel,
  boundary: PartitionBoundaryPolicy
) -> Outcome<TargetSource>

fn emit_partition(
  partition: ProgramPartition,
  target: TargetModel
) -> Outcome<TargetSource>
```

Bodies (designed, not load-bearing until §6 consumers land):

```
fn emit_partition(partition, target) =
  emit(
    tree: partition_inferred_tree(partition: partition),
    target: target
  )

fn emit_for_target(program, segment, target, boundary) =
  bind_outcome(
    o: resolve_program_partition(program: program, segment: segment, boundary: boundary),
    f: fn(p) { emit_partition(partition: p, target: target) }
  )
```

**Sugar** (per-target **data** accessors, not substrate functions):

```
// In a .dag test / workflow module:
import v4.extdeps.languages.rust { rust_mvp1_target_model }
import v4.compiler.program_partition {
  emit_for_target,
  program_segment_whole_program
}

fn dag_tree() -> InferredTree { ... }   // caller's program value

fn main_emission_receipt() -> Outcome<TargetSource> {
  emit_for_target(
    program: dag_tree(),
    segment: program_segment_whole_program(),
    target: rust_mvp1_target_model,
    boundary: partition_boundary_fail_closed()
  )
}
```

`emit_rust` as a **function name** is discouraged in substrate — it revives the v2 per-target
roster. If operators want the spelling, a **fixture-local** alias in `extdeps/languages/rust.dag`
is acceptable (one file per target, cost-of-change = 1).

### 4.5 Relationship to other "subgraph" axes

| Mechanism | Question it answers | Orthogonal to partition? |
|---|---|---|
| **Program partition** (this doc) | "What slice do I **emit**?" | — |
| `ChangedSubgraphFrontier` | "What changed slice do I **re-run**?" | Yes — diff-driven, not author-selected |
| `SectionRef` + `apply_lens` | "What slice do I **analyze**?" | Adapter only — maps into `ProgramSegment` |
| `eval_mvp2_add_subgraph` | "What slice do I **execute**?" | Shares containment/facts lessons; eval does not reuse partition carrier directly (future unify is optional, not blocking) |

## 5. Stage-fold alignment (M11)

Partition resolution should be **one fold** over the ambient program (containment + closure
evidence), not nested `if segment.kind` stage logic beyond the initial segment dispatch.
`partition_inferred_tree` is a **zero-residue projection** once closure is modeled:

```
partition_inferred_tree = facts_closure_fold ∘ re_root
emit_for_target         = emit ∘ partition_inferred_tree ∘ resolve
```

The existing `emit` composition (`serialize_target ∘ translate`) is unchanged — partition is
**upstream input shaping**, not a fifth pipeline stage.

## 6. Consumers (E-10 — nothing is "done" until these run)

### 6.1 Wave A — partition scaffold consumer (next PR)

- Implement `facts_closure_fold` + `partition_boundary_free_ref` checks.
- Wire `emit_partition` / `emit_for_target` in `src/v4/compiler/program_partition.dag` (or
  thin `05_emit.dag` re-export if import cycles demand).
- **Keystone claim:** emit only `Arrow.body` Behavior subtree for COMPREP add-fn (uses
  `SubtreeAtPath` or `SubtreeRoot`), whole module stays inferred once.

### 6.2 Wave B — ergonomic targets (parallel, data-only)

- Documented `dag_tree()` helpers in fixture modules (already implicit in `mvp1_*_inferred_tree`).
- Optional `emit_with_target_model(program, segment, target_model_node)` adapter where the
  target is carried as a `Node` witness (bootstrap pattern) — still explicit, not hidden state.

### 6.3 Wave C — self-host slice emit

- Emit one compiler stage DAG (`05_emit.dag` root only) inside the fixed-point loop
  (`design-self-host-fixed-point.md`) using `SubtreeAtPath`.

**Explicit non-consumer:** CLI / artifact-plan drivers. They may *call* the same API eventually,
but CLI is not the design center — in-language callability is.

## 7. Open questions (escalate before implementing)

| ID | Question | Default if unanswered |
|---|---|---|
| Q-EP1 | Containment evidence for `SubtreeRoot`: syntactic `content_hash` path witness vs nominal declaration identity (#4581)? | Path witness first (reuses `subterm_at` machinery) |
| Q-EP2 | Should `AmbientContextAvailable` auto-include sibling declarations or only imports? | Only declarations reachable via `DependencyView` from segment root |
| Q-EP3 | Unify `SectionRef` and `ProgramSegment` into one substrate coproduct? | **No** — lens section carries enforcement config; emit segment is translation-shaped. Adapter stays in compiler. |

## 8. Non-goals

- No new per-target emitter functions in substrate (`emit_rust`, `emit_python`, …).
- No CLI flags for segment selection.
- No change to `translate` / `serialize_target` internals in this design wave.
- No host-transport work (`design-omni-emission-transport.md` is orthogonal).
- No closure implementation in the scaffold PR — types + root resolution + fail-cl symbols only.

## 9. Receipt checklist (this session)

- [x] Design doc (this file)
- [x] `ProgramSegment` + `PartitionBoundaryPolicy` + fail-cl `Symbol` data in `std/`
- [x] `ProgramPartition` + root-resolution scaffold in `compiler/`
- [ ] Closure fold + `emit_for_target` wiring (Wave A consumer — explicitly deferred)
- [ ] Executed keystone claim over partial emit (Wave A — E-10)

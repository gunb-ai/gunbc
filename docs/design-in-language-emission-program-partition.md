# Design: In-Language Emission + Program Partition

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). No load-bearing
> stage edits land from this doc without the consumers named in §6 (E-10). This is the contract
> for making **emit a first-class `.dag` operation over arbitrary program segments**, with
> **target as an explicit value** — the operator sketch `emit_rust(dag_tree())` means "call
> emit from a `.dag` program on whatever subtree you chose," not "drive emit through a CLI
> flag."
>
> **Deliverable (this session):** this doc + modeled carriers in
> `src/v2/std/program_segment.dag` and `src/v2/compiler/program_partition.dag` (fail-closed
> reason symbols + scaffold resolution for `WholeProgram` / `SubtreeAtPath` / `SubtreeRoot`).

## 1. Problem

Today emit exists in `.dag` but only at **whole-program** granularity:

```dag
// src/v2/compiler/05_emit.dag — frozen composition form (M11 receipt)
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
target dispatch to an enum roster. v2's derived-homomorphism form keeps **one** `emit` and passes
`TargetModel` data (`rust_mvp1_target_model`, `mvp1_python_target_model`, …).

## 2. What already exists (M9 DFS — extend, don't coin)

| Piece | Where | Role |
|---|---|---|
| `emit = serialize_target ∘ translate` | `src/v2/compiler/05_emit.dag` | stage composition; **consumer** of `InferredTree` |
| `InferredTree { root, facts }` | `src/v2/compiler/04_infer.dag:96` | infer output; facts map keyed by `Node` |
| `TargetModel` bundle | `src/v2/std/compilers/target_model.dag` | target as explicit value (grammar rows, inhabitants, …) |
| `TargetSource` | `src/v2/compiler/07_target_carriers.dag` | emitted text carrier |
| `Path` + `subterm_at` | `std/node.dag`, `v2/lens/application.dag` | structural subtree selection (lens section projection) |
| `SectionRef` | `v2/lens/application.dag:67` | declaration / node scope handles (Path-backed, 🟡 identity evidence) |
| `ChangedSubgraphFrontier` | `v2/lens/affected_set.dag` | **diff-driven** subgraph (incremental re-exec), orthogonal axis |
| Eval subgraph MVP | `eval_runtime_mvp.dag` | arbitrary `Node` as eval root — proves runtime can target a subgraph |
| `ProjectionKind::EmitProjection` | `src/v2/std/projection.dag:15` | projection-as-data names emit; no partition carrier yet |
| `TargetTypeExpressionProjection` rows | `src/v2/std/compilers/target_model.dag` | per-language type-tier spelling; pattern for static projectability checks |
| Cross-target coercion fold | `mvp_int_cross_target_coercion.dag` | `find_witness` over declared inhabitants across targets — Wave D seam (§6.4) |
| Source-as-data pipeline bridge | `comprep_eval_by_execution.dag` | sanctioned compile-time body/subtree acquisition — no runtime reflection (§4.6) |

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

Declared in `src/v2/std/program_segment.dag`:

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

Declared in `src/v2/compiler/program_partition.dag`:

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
import v2.extdeps.languages.rust { rust_mvp1_target_model }
import v2.compiler.program_partition {
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

### 4.5 Typed-projection sugar — `emit<rust>(segment)` (optional static layer)

The value-level primitive (`emit_for_target(program, segment, target, boundary)`) is the
**substrate authority** (P2). Operator steer (2026-06-12): the ergonomic surface is
**probably like `emit<rust>` or something like a projection** — a type-parameterized sugar
layer *over* the value primitive, not a replacement for it.

**Lowering (designed shape):**

```
emit<TargetTag>(program, segment, boundary)
  = emit_for_target(
      program: program,
      segment: segment,
      target: target_model_for_tag<TargetTag>(),   // static: resolves to extdeps row
      boundary: boundary
    )
```

`TargetTag` is a **closed type-level index** into the landed `TargetModel` rows
(`Rust`, `TypeScript`, `Python`, …) — the same closed-set discipline as
`TargetTypeExpressionProjection` per language (M4). It is **not** a new per-target emitter;
it is compile-time selection of which `TargetModel` **value** the value primitive receives.

**STATIC PROJECTABILITY — the payoff to design through.** With the target fixed at the type
parameter, the compiler can check **before emission** whether every connective in the
segment's subtree is covered by that target's declared rows (grammar-inverse productions,
value-expression projection arms, operator-realization catalog entries). Failures surface as
**typed compile-time diagnostics** — "this `Branch` arm has no `TargetValueExpressionProjection`
row under `Rust`" — rather than only the runtime `partition_facts_miss` / translate rejections
the value primitive discovers after partition resolution. This mirrors the existing
projection family:

| Layer | Carrier | Check timing | Example |
|---|---|---|---|
| Type tier | `TargetTypeExpressionProjection` | compile / infer boundary | `translate_type_expression_project` row miss |
| Value tier | `TargetValueExpressionProjection` | compile when segment + target tag known | §4.5 static projectability |
| Emit partition | `ProgramSegment` + `TargetModel` value | runtime (value primitive) | `partition_facts_miss`, translate `Rejected` |

`ProjectionKind::EmitProjection` (`projection.dag:15`) names the artifact class; the static
sugar attaches an **`EmitProjection` witness** when `emit<TargetTag>(segment)` is
well-projected — the segment's connectives ⊆ target rows, analogous to how type-expr
projection rows gate type-tier emission today.

**Both layers, one engine (Q-EP4 default):** targets-as-**values** (this doc's primitive —
dynamic `TargetModel` argument, can compute partition at runtime) and targets-as-**types**
(static sugar — cannot compute partition, but *can* reject ill-projected segments before
`emit` runs). The static layer does not reopen the v2 per-target function roster; it is
generic sugar parameterized by a closed `TargetTag` coproduct, lowering to the same
`emit_for_target`. Escalate to operator if a consumer needs *only* the static layer with
no value fallback.

### 4.6 Segment denotation for self-referential programs (reflection ban)

§4.4's `dag_tree() -> InferredTree` — "the caller's program value" — is correct for
**programs over explicit trees** (fixtures, source-as-data bridges, hand-authored
`InferredTree` data items). It is **insufficient** for the operator's actual sketch: a
`.dag` module that emits **parts of itself** (self-host slice emit, Wave C). That scenario
must not smuggle in **runtime reflection**.

**Ban (ratified 2026-06-07; `design-lens-subject-supply.md` R1, THESIS read-axis /
`programmatic-access-single-roof-2026-06-07`):** dynamic name-keyed lookup over the loaded
module table, `body_of("fn_name")` at runtime, or any primitive that discovers structure
by executing over live program state. The read axis is **compile-time sugar only** —
static expansion the compiler derives from declarations it owns.

**Constraint (design now, mechanism may defer):** a `.dag` module denotes its own subtree
as a `ProgramSegment` only through **compile-time staging** — never runtime introspection:

| Sanctioned denotation | Shape | Example |
|---|---|---|
| **Static path literal** | `SubtreeAtPath { path: Path { steps: [^stage, ^body, ...] } }` authored at compile time; compiler resolves against the module's own lowered `Node` | `program_segment_subtree_at_path(path: self_stage_body_path)` where `self_stage_body_path` is a `data` item fixed when the module is compiled |
| **Source-as-data recompilation** | Hold module (or slice) source as `String` data; run `tokenize → parse → … → infer` inside `.dag` (COMPREP / `comprep_source_bridged_*` pattern) | Self-host fixed-point compares *staged* source text, not runtime module table |
| **Quote / staging cell (deferred)** | A compile-time `StagingCell<Node>` (or equivalent) the compiler fills with the module's own lowered subtree before user code runs — **not** a runtime `get_current_module()` | Wave C consumer; substrate extension gated on operator-STOP |

**Forbidden denotation (explicit):** `dag_tree()` implemented as "return the module I'm
currently executing" via host injection, runtime `Node` enumeration, or reflection over
eval context. Those shapes invite a reflection-shaped consumer and violate the 2026-06-07
ruling even if they typecheck.

**Receipt for Wave C:** self-host slice emit must cite *which* sanctioned denotation it uses
(static path `data` item, or source-as-data recompile of the stage file) in the same PR that
wires `SubtreeAtPath` against the compiler's own DAG — silence is not an option.

### 4.7 Relationship to other "subgraph" axes

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
- Wire `emit_partition` / `emit_for_target` in `src/v2/compiler/program_partition.dag` (or
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
- **Denotation obligation (§4.6):** cite static path `data` or source-as-data recompile —
  no runtime `dag_tree()` reflection.

### 6.4 Wave D — multi-target partition (named, deferred)

**Operator demo scenario:** one system with **parts in Rust and parts in TypeScript** —
not two independent emits, but a single partitioned program where each slice carries its
own `TargetModel` and the slices **link** at the boundary. Single-target partition (this
doc's `ProgramPartition`) remains the substrate; Wave D designs the **seams**, not the
linkage rows themselves.

**Seam 1 — boundary meets cross-target coercion.** When `PartitionBoundaryPolicy` encounters
a free reference at the segment edge, the resolver must consult the existing cross-target
coercion machinery (`mvp_int_cross_target_coercion.dag` — `find_witness` over declared
inhabitants, fail-closed narrowing, faithful widening) rather than inventing a parallel
boundary adapter. Question (deferred): does `partition_boundary_free_ref` refine into a
**coercion witness required** diagnostic when the escaping ref targets another target's
inhabitant class, vs a hard reject?

**Seam 2 — linkage realization rows (named, not designed here).** How the TypeScript slice
**calls** the Rust slice is target-model data: FFI symbol, subprocess spawn, wasm import,
etc. — authored in `extdeps/languages/` as `LinkageRealization` rows (name only; schema
deferred). Wave D consumer lands when a multi-target fixture claim needs it; until then,
single-target partition + per-slice `emit_for_target` is the honest capability statement.

**Existing half:** cross-target coercion claims (`mvp_int_cross_target_coercion.dag`) prove
the coercion fold works across `rust` / `python` inhabitant rows. **Missing half:** partition
selects the slice, linkage rows declare how slices compose, coercion fold bridges types at
the seam.

**Explicit non-consumer:** CLI / artifact-plan drivers. They may *call* the same API eventually,
but CLI is not the design center — in-language callability is.

## 7. Open questions (escalate before implementing)

| ID | Question | Default if unanswered |
|---|---|---|
| Q-EP1 | Containment evidence for `SubtreeRoot`: syntactic `content_hash` path witness vs nominal declaration identity (#4581)? | Path witness first (reuses `subterm_at` machinery) |
| Q-EP2 | Should `AmbientContextAvailable` auto-include sibling declarations or only imports? | Only declarations reachable via `DependencyView` from segment root |
| Q-EP3 | Unify `SectionRef` and `ProgramSegment` into one substrate coproduct? | **No** — lens section carries enforcement config; emit segment is translation-shaped. Adapter stays in compiler. |
| Q-EP4 | Targets-as-types (`emit<Rust>(segment)` static sugar) vs targets-as-values (`emit_for_target(..., target_model, ...)`) — one layer or both? | **Both** — value primitive is substrate authority; typed sugar lowers to it and adds static projectability (§4.5). Escalate if a consumer needs static-only with no value fallback. |
| Q-EP5 | Self-referential denotation: static path `data` vs source-as-data recompile vs `StagingCell` quote — which lands first for Wave C? | Static path `data` first (no new substrate); `StagingCell` is operator-STOP until consumer names it. |
| Q-EP6 | Wave D: does boundary free-ref become "coercion witness required" or hard reject when cross-target? | Defer to Wave D consumer; default hard reject until coercion seam is modeled. |

## 8. Non-goals

- No new per-target emitter functions in substrate (`emit_rust`, `emit_python`, …).
- No CLI flags for segment selection.
- No change to `translate` / `serialize_target` internals in this design wave.
- No host-transport work (`design-omni-emission-transport.md` is orthogonal).
- No closure implementation in the scaffold PR — types + root resolution + fail-cl symbols only.
- No `emit<TargetTag>` static sugar implementation in this wave — §4.5 is design only.
- No `LinkageRealization` rows or multi-target fixture in this wave — Wave D named only (§6.4).
- No runtime self-referential `dag_tree()` — §4.6 constraint is normative; mechanism deferred.

## 9. Receipt checklist (this session)

- [x] Design doc (this file)
- [x] `ProgramSegment` + `PartitionBoundaryPolicy` + fail-cl `Symbol` data in `std/`
- [x] `ProgramPartition` + root-resolution scaffold in `compiler/`
- [ ] Closure fold + `emit_for_target` wiring (Wave A consumer — explicitly deferred)
- [ ] Executed keystone claim over partial emit (Wave A — E-10)

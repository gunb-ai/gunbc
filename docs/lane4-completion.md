> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 4 master)

# Lane 4 — Completion layer

**Lane:** 4 (of 4)
**Size:** L (four stages)
**Status:** Plan. No code changes yet.

---

## Mandate

Lane 4 exists to ensure the plan leaves **zero backlog**. It covers items that don't fit Lanes 1–3's themes but are real thesis obligations:

| Item | Why not in Lanes 1–3 | Source |
|---|---|---|
| Transport declarations | Surface feature, but specifically tied to runtime execution (not compilation) | M2 remaining |
| `dag run` interpreter | New execution modality — neither emission nor property proof | M2 remaining |
| Side effects as compile-time dimension | Extends Lane 2's property-proof framework; distinct algebra from idempotency | THESIS.md dimensions matrix |
| Space bounds as compile-time dimension | Same — extends Lane 2 framework; separate from symbolic cost | THESIS.md dimensions matrix |
| Async/concurrent emission modeling | Execution concurrency axis — not thesis-core but a real aspiration | ROADMAP.md "what NOT to build yet" (moved in) |

Four stages, each closing one of these:

---

## Stages

### Stage 4a — Transport declarations + `dag run` interpreter (L)

**Scope.** Two coupled items; transport shape is **locked in [DB-6](./design-transport-taxonomy.md)** — this stage implements it, does not restate it.

**Transport declarations** — transports are spec files in `extdeps/transports/`, not a closed compiler-side taxonomy. Per DB-6, substrate adds exactly one minimal carrier:

```
type TransportDeclaration {
  spec_ref: DeclarationId   // points at e.g. `rest` in extdeps/transports/rest.dag
  fields: List<FieldEntry>  // the literal fields the user declared
}
```

Parser lowers a user declaration like `transport rest { method: PUT, path: "/v1/secrets/{id}" }` into `TransportDeclaration { spec_ref: <rest.dag>, fields: [...] }`. Per-spec field-shape validation happens at lowering: each transport spec declares its own field-shape type (e.g., `rest.dag` declares the expected method/path/body fields for `transport rest { ... }`); parser checks the user's fields match. **No closed compiler-side transport coproduct, no enum of transport kinds** — adding a new transport means adding one spec file, not editing the compiler. See DB-6 "Rejected alternatives" for the specific shapes ruled out.

**`dag run` interpreter** — execute a `.dag` program directly without emission. The interpreter walks the DAG and, for every `TransportDeclaration` site, looks up the spec via `spec_ref` and runs the spec-declared invocation steps. The interpreter contains zero per-transport handlers; the spec IS the implementation (per THESIS.md:117-139).

Use case: `dag run my_cloud_bringup.dag` actually launches the infrastructure by walking declared transports through their spec-declared invocation steps.

**Acceptance:**
- `transport <name> { ... }` parses into `TransportDeclaration { spec_ref, fields }` with spec-ref resolution (per DB-6 acceptance gate)
- Parser rejects a user declaration whose fields don't match the spec's declared field shape
- `dag run fixture.dag` executes a multi-step cloud workflow by dispatching on `spec_ref`, not on a compiler-side enum
- `dag run` code path has zero `match transport_kind { Rest => ..., Shell => ... }` — all dispatch is through spec lookup
- Adding a new transport (e.g., `extdeps/transports/graphql.dag`) requires zero interpreter or emitter code changes
- Real-transport mode (non-mock) gated behind explicit flag, not default

**Escalation:**
- If the interpreter ever needs a per-transport branch, that's a spec-shape inadequacy — extend the spec vocabulary (invocation steps, effect declarations), not the interpreter.
- If `dag run` needs effect execution ordering that the current substrate can't express (e.g., parallel operations with dependencies), surface — parallelism sequencing might need Lane 2 Stage 2e's output.

### Stage 4b — Side effects dimension (M)

**Scope.** Add side effects as a compile-time-enforced dimension, following the Lane 2 Stage 2f `Dimension` pattern.

Today `dsl/std/behavioral.dag` declares `OperationBehavior` with effect metadata, but there's no compiler lens that:
- Composes declared effects across a workflow
- Rejects composition that violates declared safety properties (e.g., hermetic operations cannot emit side effects)
- Generates test obligations for effect-free paths

After 4b: `side_effects: Dimension` instance, declared in std/, consumed by a workflow lens analogous to idempotency (Lane 2 Stage 2b).

**Acceptance:**
- A workflow that mixes a hermetic operation with a non-hermetic one fails compile with diagnostic naming the violation
- Test obligations emit assertions that hermetic ops produce no side effects on state
- Declared side-effect modifiers on extdeps cross-check against derivation (analogous to `check_modifier_vs_derivation`)

**Escalation:**
- If side-effect algebra doesn't compose cleanly (e.g., a mix of reads and writes in a branch), surface — might need `EffectShape` extension beyond what idempotency required.

### Stage 4c — Space bounds dimension (M)

**Scope.** Add space (memory/allocation) bounds as a compile-time dimension.

Uses the same `Dimension` abstraction. Adds structural space cost for:
- List construction: O(n) bytes in list element count
- Map/fold chains: derive from input + body space
- Recursion: space = depth × per-frame size

Emits diagnostic when a program's space bound exceeds a declared limit (e.g., `fn handler(req) where memory_bounded(1_MB)`).

**Acceptance:**
- Thesis doc doesn't list a specific test, but the natural fixture is: a recursive function with unbounded accumulator growth declared `memory_bounded(1KB)` fails compile
- Structural space cost composes through std/list operations the same way structural time cost does

**Escalation:**
- If space cost requires knowing allocator behavior (which differs per target), scope tighter — report structural space (node count × size), not allocator-specific bytes.

### Stage 4d — Async emission modeling (S)

**Scope.** Model async/concurrent emission as a target-spec option. Not "make everything async," just: declare that a target supports async patterns, emit the async version when the target spec requests it.

```
// in spec/rust_async.dag (or as a mode flag on spec/rust.dag)
data rust_async_emission: AsyncEmissionStrategy = {
  runtime: Tokio
  async_fn_syntax: "async fn {name}({params}) -> {ret}"
  await_syntax: "{expr}.await"
}
```

The generic walker (Lane 1 Stage 1e) reads the async strategy from spec and emits `async fn` / `.await` where appropriate.

**Acceptance:**
- A `.dag` cloud workflow can be emitted to either sync Rust or async Rust via spec choice
- The choice is ONE spec field, zero new walker code (proves Lane 1's consolidation contract)

**Escalation:**
- If async emission requires rewriting function signatures throughout the walker (e.g., everything becomes `async` downstream), surface — that's a consolidation-design flaw, escalate back to Lane 1d.

---

## Dependencies

- **Requires Lane 3 Stage 3a complete** — transport declarations need full surface generics and `data` semantics; `dag run` needs mutual recursion support
- **Requires Lane 2 Stage 2f complete** — side effects and space bounds extend the `Dimension` abstraction from 2f
- **Requires Lane 1 Stage 1e complete** — async emission modeling uses the consolidated walker

Lane 4 therefore starts **after both Lane 2 and Lane 3 have landed their framework stages** — specifically, after 1e, 2f, and 3a converge.

---

## Cross-cutting acceptance (Lane 4 done when)

- [ ] `dag run fixture.dag` executes a cloud workflow through declared transports
- [ ] Transports lower to `TransportDeclaration { spec_ref, fields }` carriers (per DB-6); no closed compiler-side coproduct; adding a new transport is one spec file
- [ ] Side effects compose as a `Dimension` instance; a mixed hermetic/non-hermetic workflow fails compile with diagnostic
- [ ] Space bounds compose as a `Dimension` instance; unbounded recursion with declared bound fails compile
- [ ] A workflow emits to both sync and async Rust via spec choice, no walker code change

---

## Size

L aggregate, four stages:
- 4a: L
- 4b: M
- 4c: M
- 4d: S

---

## What's TRUE when Lane 4 completes

With Lane 4 done, every named thesis obligation is either implemented or explicitly deferred with rationale. No M-milestone is "deferred" without a lane owning its components. No dimension in the THESIS.md correctness matrix is marked "not wired."

The plan is truly backlog-free.

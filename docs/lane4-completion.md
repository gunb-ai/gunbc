> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 4 master)

# Lane 4 — Completion layer

**Lane:** 4 (of 4)
**Time budget:** ~4 weeks
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

### Stage 4a — Transport declarations + `dag run` interpreter (1.5 weeks)

**Scope.** Two coupled items:

**Transport declarations** — formalize how a service operation maps to a concrete runtime transport:

```
service gcp.SecretManager {
  transport rest { method: PUT, path: "/v1/secrets/{id}" }    // HTTP transport
}

service local.filesystem {
  transport shell { command: "cp", args: ["{src}", "{dst}"] }  // shell transport
}
```

Already partially declared in `dsl/extdeps/*/`; this stage makes transport a first-class substrate type with typed variants (`RestTransport`, `ShellTransport`, `GrpcTransport`, etc.) rather than string-typed `transport rest { ... }` annotations.

**`dag run` interpreter** — execute a `.dag` program directly without emission. The interpreter walks the DAG, calls transports through their declared bindings, and returns results. This is distinct from emission: emission produces target source code; interpretation produces runtime results.

Use case: `dag run my_cloud_bringup.dag` actually launches the infrastructure (through declared transports) rather than emitting code to do so.

**Acceptance:**
- `transport rest {...}` / `transport shell {...}` / etc. parse into typed substrate carriers (not strings)
- `dag run fixture.dag` executes a multi-step cloud workflow through mock transports, returns final state
- Real-transport mode (non-mock) gated behind explicit flag, not default

**Escalation:**
- If transport types multiply (REST + shell + gRPC + GraphQL + SSH + …), don't try to enumerate everything at once. Ship REST + shell, document the pattern for adding new transports.
- If `dag run` needs effect execution ordering that the current substrate can't express (e.g., parallel operations with dependencies), surface — parallelism sequencing might need Lane 2 Stage 2e's output.

### Stage 4b — Side effects dimension (1 week)

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

### Stage 4c — Space bounds dimension (1 week)

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

### Stage 4d — Async emission modeling (0.5 week)

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

Lane 4 therefore starts **after both Lane 2 and Lane 3 have landed their framework stages**. Can't start before week 9 of the plan.

---

## Cross-cutting acceptance (Lane 4 done when)

- [ ] `dag run fixture.dag` executes a cloud workflow through declared transports
- [ ] Transport types are typed carriers (`RestTransport`, `ShellTransport`, etc.), not string-annotation soup
- [ ] Side effects compose as a `Dimension` instance; a mixed hermetic/non-hermetic workflow fails compile with diagnostic
- [ ] Space bounds compose as a `Dimension` instance; unbounded recursion with declared bound fails compile
- [ ] A workflow emits to both sync and async Rust via spec choice, no walker code change

---

## Estimate

4 weeks total, weeks 10–13 of the plan:
- 4a: 1.5 weeks (weeks 10–11.5)
- 4b: 1 week (week 11.5–12.5)
- 4c: 1 week (week 12.5–13.5)
- 4d: 0.5 week (week 13.5–14)

---

## What's TRUE when Lane 4 completes

With Lane 4 done, every named thesis obligation is either implemented or explicitly deferred with rationale. No M-milestone is "deferred" without a lane owning its components. No dimension in the THESIS.md correctness matrix is marked "not wired."

The plan is truly backlog-free.

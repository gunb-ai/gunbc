# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (40 completed items from earlier lanes)

---

## Four Lanes

| Lane | Doc | Goal | Open Items |
|------|-----|------|------------|
| **1. Type System** | [`TODO/type-system.md`](TODO/type-system.md) | Compositional type coverage — decisions obligate, obligations propagate. WS-1 through WS-7. | 29 open + 11 done across 7 workstreams |
| **2. Compiler Debt & App Layer** | [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md) | Fix compiler gaps that force runtime bridges. 10 accidental bridges → delete. Each has specific files/LOC to remove. | 10 bridges + app layer cleanup |
| **3. SDLC Pipeline** | [`TODO/sdlc.md`](TODO/sdlc.md) | Run the SDLC pipeline end-to-end. Phase 0 (prove compilation) is a **hard gate**. | 10 done + 9 in progress across 5 phases |
| **4. Compiler Pipeline** | [`TODO/compiler-pipeline.md`](TODO/compiler-pipeline.md) | End-to-end pipeline hardening + interpreted/compiled parity. Three invariants: binary logic, minimalism, resolve early. | 42 items across 9 workstreams |

### Cross-Cutting Reliability Lane

Source of truth: [`TODO/rolling-postmortem.md`](TODO/rolling-postmortem.md)

1. **RR-1 (P0)**: Replace heuristic test-time confidence with measured runtime budget checks for `test-xs/s/m/l/xl` (maps to RC-P0-004).
2. **RR-2 (P1)**: Split monolithic exhaustive tests into bounded shards or explicit integration-only flows; default loops should stay interactive (maps to RC-P1-005/006).

### Cross-Cutting `.dag` Migration

Source of truth: [`TODO/gunbc-dag-simplification.md`](TODO/gunbc-dag-simplification.md)

1. **DM-1 (P0) — DONE (2026-03-05)**: Deleted the remaining dead handwritten cloud/provider crates that now have `.dag` replacements: `lib/gcp-ops` and `lib/aws-ops`, following the earlier removal of `lib/gcp-ops/src/ops.rs`, `lib/gcp-ops/src/services/local_auth.rs`, and `lib/cloud-ops/src/infra_*`. Workspace/config/guardrail references were updated in `Cargo.toml`, `dsl/config/workspace.dag`, `dsl/config/arch_rules.dag`, `dsl/extdeps/gunbc.dag`, `gunbc-dag/tests/boundary_gate.rs`, and `lib/transport/src/pragma_lint.rs`. Update later on 2026-03-05: the last scheduled handwritten survivor in this lane, `gunbc-dag/src/testgen_dag/graph.rs`, was deleted and replaced by [`dsl/tools/testgen.dag`](dsl/tools/testgen.dag), with Rust reduced to narrow discovery/render extern bridges. Audit result: the acceptance grep now returns compiler/framework internals and thin `.dag` entrypoint shims (`gunbc-dag/src/dsl_builder.rs`, `gunbc-dag/src/tool_graphs.rs`, `gunbc-dag/src/pragma/mod.rs`), not handwritten provider/workflow graphs. Rule remains: no new provider/runtime logic lands in Rust unless the compiler cannot yet express it.

### Design docs

| Doc | Scope |
|-----|-------|
| [`docs/design/compilation-pipeline.md`](docs/design/compilation-pipeline.md) | Full pipeline map (.dag → execution), data shapes at each stage, gap analysis |
| [`docs/design/v4/compiler-densification-roadmap.md`](docs/design/v4/compiler-densification-roadmap.md) | Prioritized roadmap: kill interpreter, hermeticity, dual-encoding, service codegen |
| [`docs/design/v4/compositional-type-coverage.md`](docs/design/v4/compositional-type-coverage.md) | Type system vision, audit, gaps, workstreams, worked examples |
| [`docs/design/sdlc/domain-modeling-comprehensive.md`](docs/design/sdlc/domain-modeling-comprehensive.md) | SDLC entity/relationship/state machine model |
| [`docs/design/sdlc/production-gap-analysis.md`](docs/design/sdlc/production-gap-analysis.md) | SDLC activation blockers |

### Dependency between lanes

```
Lane 1 (type system)    ──→  Lane 3 (SDLC) uses the type system
Lane 2 (compiler debt)  ──→  Lane 3 (SDLC) needs working compilation pipeline
Lane 4 (pipeline)       ──→  Lane 2 (bridges) benefits from pipeline hardening
Lane 4 (pipeline)       ──→  Lane 3 (SDLC) needs reliable compilation + emit
```

Lane 1 and Lane 2 can proceed in parallel. Lane 3 Phase 0 (prove it compiles) can start now — it doesn't need type system improvements, just basic compiler correctness. Lane 4 is independent groundwork — hardening the pipeline benefits all other lanes.

**Recommended start order**:
1. Lane 3 Phase 0 (S-1 through S-4) — fix known bugs, prove SDLC compilation. **Hard gate.**
2. Lane 2 "Delete immediately" (bridges 4, 5, 10) — trivial cleanup, ~300 LOC deleted
3. Lane 2 "Compiler fixes" (bridges 1-3, 8-9) — each has specific deletion targets + grep verification
4. Lane 1 WS-1 + WS-3 (no blockers) — in parallel with Lane 2

**Operating principles** (from retrospective):
- Prove before building on top. No Phase N+1 work until Phase N is green.
- Each task names what gets **deleted** and a `grep` command to verify deletion.
- No intermediate abstractions. Go a→f directly.
- `@annotation` is never the final state — go straight to structural blocks.
- Check `Cargo.toml` dependency graphs before moving code between crates.

---

## Completed (archived)

40/40 items complete across earlier lanes:

- **Lane 1: Compiler Pipeline** — 26/26 (C1-C30)
- **Lane 1: Binary Elimination** — 10/10 (A2-A11)
- **Phase 3: Purist Engine** — 4/4 (C28-CT8)

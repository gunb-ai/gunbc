# R3 Wave-1 S1 — Parallelism walker port (#81)

**Owner**: Wave-1 Substrate worker (to be assigned by spawn)
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH-READY (no prerequisites)

Substrate-ready per `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` F-α phase. Walker logic exists in hand-Rust at `src/v3/compiler/src/workflow_parallelism.rs`; port to `.dag` form to close gate #81 `parallelism_lens_behaviorally_complete`.

Parallel-dispatchable with the F-β.1 canvas (S3) — no predicate-block between them per Cluster-F plan.

## §1. Scope

Port the parallelism walker from `src/v3/compiler/src/workflow_parallelism.rs` to DSL form. This is a v3 P5 dissolution candidate — hand-Rust → `.dag` substrate migration.

### Phase A — Read the hand-Rust authority

Inventory `src/v3/compiler/src/workflow_parallelism.rs`:
- Public surface (what callers consume)
- Internal walker structure (recursion shape, accumulator, branching)
- Carrier shapes the walker pattern-matches against

The hand-Rust authority is the **source of truth at port time** per `feedback_grep_verify_brief_authoring`. Cite specific function names + line numbers in the PR body.

### Phase B — Express the walker in DSL

Author the equivalent walker as a DSL fold/recursion in the appropriate module (likely `dsl/std/` or a new module; grep authority-namespace before placement per `feedback_self_hosting_md_authority_audit_before_naming`).

The DSL form should:
- Match the hand-Rust walker behavior **byte-identically** at the test surface (use a shadow-mode test if the consumer can be exercised both ways)
- Honor the v3 lens pattern (this is a lens-behavior gate)
- Preserve any optimization-opportunity comments from the hand-Rust as DSL-level annotations

### Phase C — Wire the consumer

Replace consumer calls to the hand-Rust function with calls to the DSL fold. Preserve type signature exactly; the v3 boundary discipline doesn't need to change.

### Phase D — Retire the hand-Rust

When the DSL port is consumer-wired AND tests are green, **delete** `src/v3/compiler/src/workflow_parallelism.rs` (or whatever subset of it the port retires). Closes gate #81.

## §2. STOP conditions

1. **Walker recursion shape mismatch** — if the hand-Rust walker depends on a Rust-specific feature (HashMap iteration order, trait specialization, etc.) that the DSL doesn't have, **STOP** and surface — this is a substrate-shape question.
2. **Consumer surface drift** — if porting requires changing the consumer's call signature, **STOP** — call-site changes are a separate PR scope.
3. **P5 hand-Rust accounting** — confirm with grep that retiring `workflow_parallelism.rs` does NOT leave dangling `pub fn` referenced from other crates' integration tests. If it does, surface — call-site cleanup needs to land first or bundle in.

## §3. Verification

- `cargo test --workspace` green (including any parallelism-lens tests)
- `cargo clippy --all-targets -- -D warnings` clean
- Grep verification: `grep -rn "workflow_parallelism" src/v3/` returns zero hits at landing
- Manual: pick 3 representative workflow DAGs, run the lens through both old (pre-retirement-commit) and new (DSL) paths, confirm output equivalence

## §4. PR body framing

- Cite gate #81 closure
- Cite v3 P5 Pure Bootstrap accounting: count hand-Rust lines retired
- Cross-link `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` F-α phase
- Cite the byte-identical-behavior verification path (test cases used)

## §5. Out of scope

- F-β (effect_enum) work — separate Substrate worker S3 (canvas) + Wave-2 implementation
- #83 register status — Wave-2; gates on all 4 lenses COMPLETE
- #95 opt-in iteration demo — Wave-2; gates on #81 + T-LAS #91

## §6. Reference

- `src/v3/compiler/src/workflow_parallelism.rs` — port source
- `docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md` — F-α phase authority
- `docs/r3-remaining-work-dependency-graph.md:56,118` — gate-row metadata
- INVARIANTS.md P5 — Pure Bootstrap progress accounting

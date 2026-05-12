# R3 Wave-1 S6 — Slice 7 affected-set implementation (#103)

**Owner**: Wave-1 Substrate worker
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH GATES ON cool-crab-565 PR #2766 canvas-status

**Do not start authoring until prerequisite clears**:

1. **PR #2766 (Slice 7 pre-impl prequeue: harness contract + Layer 2 path-regex inventory ratchet) merged** — the post-merge canvas establishes the implementation contract. Merge = post-Director-ratification.

## §1. Scope

Implement the **BinaryShim affected-set selection** per the ratified Slice 7 canvas (`docs/design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` in main, post-PR-#2766). Closes gate #103 `ci_uses_affected_set_selection`.

**Authority chain** (corrected per codex BLOCKING review on PR #2782 sha b28cf884 — earlier draft of this brief mis-cited high-level T-WAD substrate-shape framing):
- **Upstream lens authority**: **PR #2713** — affected-set lens substrate (already merged per scope docs); `docs/design-affected-set-lens.md` §2 defines `affected_set(Dag_before, Dag_after, dim)` per-dimension + aggregate
- **Slice 7 canvas authority**: `docs/design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` (in main) — §1 BinaryShim consumption contract, §3 fail-closed policy, §4 selection-derivation algorithm, §5 path-regex removal invariant
- **Implementation harness**: PR #2766's harness contract + Layer 2 path-regex inventory ratchet (in main)

### Phase A — Read the canvas + lens authorities

Inventory canvas §1: BinaryShim runner consumes PR #2713's affected-set lens output (NOT path globs). At PR time, runner builds `Dag_before` (merge-base) + `Dag_after` (PR head) using same compiler revision + feature flags per `CIWorkflowDag` + pinned toolchain facts. Lens output: `Set<NodeRef>` (or NodeRef-keyed record with dimension + provenance — treat **authoritative serialized form from PR #2713** as single source of truth).

### Phase B — Implement the selection function per canvas §4

Algorithm verbatim from canvas §4:
1. Compute `A = ⋃_dim affected_set(Dag_before, Dag_after, dim)` (aggregate affected nodes)
2. For each `TestClaim` or test-shaped obligation `t ∈ B`: derive selected vs skip-safe status from `TestClaim` declarations + gate records + `CIWorkflowDag` metadata
3. Map `NodeRef` sets to **executable CI actions** (cargo filters, job labels, gate commands) using metadata anchored in `CIWorkflowDag`, `TestClaim` declarations, and gate records — **NOT** by parsing GitHub `paths`/`paths-ignore` in YAML

Implementation MUST:
- Honor single-authority derivation from `Dag_before` + `Dag_after` + PR #2713 lens output + `TestClaim` / gate metadata
- Be testable structurally (per TESTING.md band-A)
- Handle canvas §3 fail-closed cases: `Dag_before`/`Dag_after` unbuildable → superset + diagnostic; `TestClaim` lacks dimension declarations → superset (cannot prove skip-safe)

### Phase C — Path-regex removal invariant (canvas §5)

After this PR lands, **no** workflow `if:` condition may encode run-vs-skip on `git diff` path patterns, `paths-filter` equivalents, or hand-maintained regex allowlists. That logic lives **only** in the BinaryShim runner using PR #2713 output. Layer 2 path-regex inventory ratchet (in main per PR #2766) tracks removal.

## §2. STOP conditions

1. **PR #2713 lens output shape uncertainty** — if the authoritative serialized form (`Set<NodeRef>` vs richer NodeRef-keyed record per canvas §1.2) isn't unambiguous at landing time, **STOP** and surface — the canvas explicitly names PR #2713 as the single source of truth; do not re-derive.
2. **Single-authority violation in selection logic** — if any selection rule requires reading state outside `Dag_before` / `Dag_after` / PR #2713 lens output / `TestClaim` / gate records / `CIWorkflowDag` (e.g., env vars, external service calls, hand-written config, path globs), **STOP** — P2/P3 violation; the canvas §5 invariant explicitly bars path-regex shortcuts.
3. **Layer 2 path-regex inventory drift** — if canvas §5 invariant cannot be enforced without leaving residual `if:` path-regex conditions in main, **STOP** and surface — the inventory ratchet (in main per PR #2766) is the structural enforcement; can't dissolve incrementally.

## §3. Verification

- `cargo test --workspace`
- Hermetic test cases (per TESTING.md): N input `CIWorkflowDag`s × M diff shapes; assert selected gate-id sets match canvas-specified expectations
- Verifier ratchet shows monotone reduction (or holds at zero) on expected scenarios

## §4. PR body framing

- Cite gate #103 closure
- Cite PR #2766 ratified canvas as substrate authority
- Cite verifier ratchet contract (specific paragraph in canvas)
- Inline 2-3 worked examples of selection behavior

## §5. Out of scope

- Compiled-binary main() integration (separate scope; S7 BinaryShim brief covers the projection-arm)
- ci.yml hand-authority dissolution (#98 / Slice 5)
- Affected-set heuristics not specified by canvas (no extensions per `feedback_assertion_vector_does_not_change_scope`)

## §6. Reference

- PR #2766 (cool-crab-565 Slice 7 canvas) — substrate authority
- `docs/r3-remaining-work-dependency-graph.md:125` — gate-row metadata
- TESTING.md band-A — structural test discipline

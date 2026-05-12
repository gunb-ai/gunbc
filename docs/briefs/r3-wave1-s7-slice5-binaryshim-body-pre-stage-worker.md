# R3 Wave-1 S7 — Slice 5 BinaryShim arm body (pre-stage; #98 prep)

**Owner**: Wave-1 Substrate worker (spawn on PR #2774 merge)
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH BLOCKED on PR #2774 merge

**Do not start authoring until prerequisite clears**:

1. **PR #2774 (proud-dove-838 / Slice 4 YamlStatic body + WI-2 substrate) merged.** This brief consumes the `project_github_actions: (CIWorkflowDag, WorkflowRuntime) → Workflow` signature + the `WorkflowRuntime = YamlStatic | BinaryShim | PythonShim` ratified shape (per `docs/design-ci-workflow-emitter-dispatch.md:126`).

Pre-stage brief lands now (Wave-1 brief queue); worker spawns on Slice 4 merge.

## §1. Scope

Implement the **BinaryShim arm body** of `project_github_actions` in `dsl/gunbc/ci_emission.dag` (post-Slice-4-merge).

The BinaryShim arm emits a thin YAML `Workflow` value whose Steps invoke a compiled Rust binary entry-point (not a static workflow). The binary at runtime reads the `CIWorkflowDag` value, computes the affected-set (Slice 7 territory), and dispatches the gate matrix.

### Phase A — Determine the compiled-binary contract

Coordinate with cool-crab-565's PR #2766 (Slice 7 canvas) for the binary entry-point shape: it must consume the same `CIWorkflowDag` value Slice 4's YamlStatic body emits as YAML literal, but at runtime not emit-time.

### Phase B — Emit the thin shim Workflow

The BinaryShim `Workflow` value is structurally minimal:
- One `Job` (or N jobs per matrix axis, depending on canvas ratification)
- Each Step is a `UsesStep` invoking the compiled-binary action OR a `RunStep` invoking the binary directly
- All expression-substrate consumption per PR #2751 §5.5 (same 17 string-container + 1 enum-extension scope as YamlStatic; same exact-enumeration discipline per PR #2773 fix-forward)

### Phase C — Single-authority derivation

**Same strict single-authority discipline as Slice 4 brief** — every `Workflow` field comes from `CIWorkflowDag` input or already-modeled `gunbc.ci.*` carrier. No fabricated values. No second source of truth. STOP-on-fabrication holds.

## §2. STOP conditions

1. **PR #2766 (Slice 7 canvas) not ratified at start time** — BinaryShim arm body needs the binary entry-point contract. If PR #2766 is still in canvas-status (not Director-ratified), **STOP** and surface; brief may need revision per ratified contract.
2. **Carrier-gap** — same protocol as Slice 4 brief §1 Phase B. Surface to warm-wolf-698 + wait for resolution (substrate-prereq PR / out-of-scope narrowing / brief revision).
3. **Compiled-binary scope creep** — if implementing the BinaryShim arm body requires authoring the binary's main() in this PR, **STOP** — binary implementation is separate scope (likely cool-crab-565's Slice 7 PR or a follow-on); BinaryShim arm body emits a `Workflow` that *references* the binary, not the binary itself.

## §3. Verification

- `cargo test --workspace` — including the byte-compare integration test if Slice 4 added one for the YamlStatic mirror
- BinaryShim arm body produces a deterministic, audit-traceable `Workflow` value
- A/B comparison with YamlStatic arm: same `CIWorkflowDag` input → BinaryShim emits structurally minimal shim; YamlStatic emits the full static workflow. Both correct projections of the same source.

## §4. PR body framing

- Cite gate prep for #98 `ci_yml_hand_authority_dissolved` (Slice 5 actual swap is a follow-on; this PR lands the BinaryShim arm body itself)
- Cite `docs/design-ci-workflow-emitter-dispatch.md` §5.2 BinaryShim semantics
- Cross-link Slice 4 PR (#2774) as the substrate it extends
- Cite the regen-vs-runtime boundary per proud-dove-838's framing (Slice 4 PR body)

## §5. Out of scope

- Compiled binary main() implementation (separate scope)
- Slice 7 affected-set logic — cool-crab-565 lane / PR #2766 + future implementation
- The actual `.github/workflows/ci.yml` swap — Slice 5 dissolution (#98) is a follow-on PR

## §6. Reference

- `docs/design-ci-workflow-emitter-dispatch.md` §5.2 — BinaryShim semantics
- PR #2774 (Slice 4 substrate / pending merge) — direct prerequisite
- PR #2766 (cool-crab-565 Slice 7 canvas) — binary entry-point contract source
- `docs/briefs/t-wad-wi2-substrate-and-slice4-yamlstatic-body-worker.md` (in main) — sibling brief; same single-authority + carrier-gap STOP disciplines apply

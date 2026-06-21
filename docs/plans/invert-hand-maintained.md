# Inverting hand-maintained artifacts — generate from `.dag`, drift-gate the projection

> The §7 self-hosting recursion applied **beyond the compiler**: every hand-maintained artifact
> (`ROADMAP.md`, doc indexes, eventually more) is a *second representation* of an authority that already
> lives in the substrate — so it drifts. Invert it: the `.dag` model is the authority, the artifact is an
> **emitted projection**, drift-gated (`generated == committed`) exactly as `ci.yml` already is. Flagship:
> the ROADMAP. DESIGN refs: §2 (no dual representation), §3 (single authority), §5 (construction over
> validation — the projection makes the defect class *unwritable*), §6 (emission = ingestion⁻¹; the doc
> medium), §7 (the recursion — the repo emits its own scaffolding).

## 1. Why — a hand-maintained artifact is a dual representation that drifts

The ROADMAP *says it itself*: "checkboxes are authoritative for progress … a task's real state is its
branch/PR + the carrier marks." So the real authority is the work-DAG + the substrate; the markdown is a
**hand-kept copy** of it — a §2/§3 dual representation. And it drifts, *provably*: in the last day alone
this repo hit a **stale checkbox** (the realization-vocab guard sat `[ ]` while done on main), a
**dangling link** (a cause-table doc referenced but never written), and **orphan docs** (added with no
inbound link). Each is a hand-maintenance defect that **cannot exist in a projection**.

## 2. The flagship — the ROADMAP, emitted

Authority = the **work DAG** (the dashboard work-items / substrate work carriers) + the doc graph. Each
roadmap line is `emit(work_node)`:

- **title** ← the work node;
- **checkbox status** ← *derived* (PR merged → `[x]`, branch open → in-progress) — never hand-toggled, so
  the stale-checkbox class is **gone**;
- **plan-doc pointer** ← the carrier's link, emitted *only if the doc exists* — so a **dangling link is
  unwritable**;
- **indentation** ← the dependency edges (the roadmap's "indentation = depends on the item above" becomes
  literal, not a hand convention);
- the **doc index** emits one entry per doc reachable in the graph — so an **orphan is unwritable** (a doc
  with no work-node becomes the *error*, surfaced at emit, not a silent omission).

This is the **construction upgrade of the reachability lens** ([inert-layer-lens](inert-layer-lens.md)):
that lens *validates* reachability after the fact (a ② residue); a generated ROADMAP makes
unreachability / dangling / stale **unwritable** (a ① wall). Same defect class, moved from lens to wall by
emission — the §5 "construction over validation" move applied to the doc layer.

## 3. The pattern — reuse `ci.yml`, don't reinvent

`ci.yml` is **already** this. `expected_ci_yml()` (`dsl/gunbc/ci_yaml_emit.dag`) emits it from
`CiFloorSpec`, and `ci_yaml_gate` (`dsl/tools/ci_yaml_gate.dag`) fails the build if `ci.yml !=
expected_ci_yml()`, byte-for-byte. The doc project is the **same three pieces** over the markdown medium:

1. **authority model** — the work DAG in `.dag` (what `CiFloorSpec` is for ci.yml);
2. **`emit(model, Markdown)`** — the row-driven inverse (§6) over a new `Markdown` target, the **same
   `06_translate` machinery** that already emits Rust / TypeScript / ci.yml, not a bespoke printer;
3. **drift gate** — `ROADMAP.md == emit(work_model)`, the `ci_yaml_gate` shape.

No new mechanism — a new **target medium** (`Markdown`) + the **work-DAG authority model**. The §6
emission lane and the idea-machine's medium axis are exactly this; the doc is the next medium after ci.yml.

## 4. The census — what's hand-maintained, and how invertible

| artifact | authority | status |
| --- | --- | --- |
| `ci.yml` | `CiFloorSpec` | **inverted** — emitted + drift-gated (the template / proof) |
| stage0 seed | the `.dag` compiler | **inverting** — self-hosting (§5/§7), Route-A last mile |
| **`ROADMAP.md`** | the work DAG | **flagship to invert next** — nearly pure structure + status + pointers |
| doc indexes / cross-links | the doc graph | inverts *with* ROADMAP (the reachability index emits) |
| plan docs (prose) | authored | **partial** — frame / index / status / pointers generate; the *reasoning prose* stays authored |
| `DESIGN.md` | authored axioms | **mostly authored** — but its `e.g.` receipts can be *generated + verified* (§7: the doc's own claims become witnesses) |

"Invert everything hand-maintained" is therefore a *spectrum*, not a flip: structured artifacts fully
invert; prose artifacts invert their **frame** and fence their **body**.

## 5. The fidelity boundary (where it can't fully invert — and that's honest)

A plan doc is `generated frame + authored body`. The frame (title, status, pointers, the index, the
dependency structure) is **derivable** → emit it. The body (the design reasoning, the §1 intersubjective
content) is **not derivable** → author it. The seam is the idea-machine's **`DecodeFidelity`** (§4/§7):
emit what is lossless, fence what is authored, and be *honest at the boundary* rather than fake-generating
prose. The ROADMAP is almost all frame (that is why it is the flagship); DESIGN is almost all body.

## 6. Sequencing

1. **The work-DAG authority model** — what a roadmap line *is* in `.dag` (a work node: title, deps,
   status-source = its PR/branch, plan-doc carrier). DFS the existing work-item carriers + the dashboard
   model before minting (§2/§3 — do not re-coin the work graph).
2. **`Markdown` as an emit target** — rows in `06_translate` / `extdeps/languages/` (the §6 medium axis);
   `emit(work_model, Markdown)` produces the ROADMAP bytes.
3. **The drift gate** — `roadmap_gate`: `ROADMAP.md == emit(work_model)`, the `ci_yaml_gate` clone.
4. **Status derivation** — wire `[x]` / in-progress from PR-merged / branch-open (closes the
   stale-checkbox class; this is the host-fed status bridge, see §7).
   - **Landed (pure core, first slice):** `gunbc.roadmap_status` derives a line's `RoadmapItemStatus`
     (`ItemDone | ItemInProgress | ItemTodo`) — and its GFM checkbox `[x]`/`[ ]` — from the *existing*
     authorities, no re-coining: PR state (`extdeps.github.pulls.PullRequest`) classified by the
     lifecycle owner (`ctrl.code_change_workflow.classify_github_pr_terminal_anchor` →
     `CodeChangeStage`). The only new fact is roadmap-specific — a line may have no work node yet
     (`Absent → Todo`). The 3→2 medium collapse (in-progress → `[ ]`) is the §5 fidelity boundary,
     faithful to the current legend. Discriminating witnesses
     (`test/claim/roadmap_status_witness_test.dag`) cover every branch and go RED on a merged-line →
     `[ ]` regression — the exact stale-checkbox bug this project kills.
   - **Still ahead:** the *host-fed* wiring (live PR/branch state → status), and consuming this status
     in the markdown line emit (#2) under the drift gate (#3).
5. Then the **doc index** (orphan / dangling become emit errors — the reachability wall by construction),
   then partial plan-doc frames.

## 7. Open

- **Authority of the work DAG.** Is it the dashboard work-items (the live source of PR/branch state), a
  `.dag` work model, or both bridged? The dashboard knows PR/branch status; the `.dag` knows structure.
  The *status* edge likely stays a host-fed bridge (like `concept_index`) until self-host; the *structure*
  is `.dag` now.
- **How much prose stays authored** — the frame/body split per doc kind (ROADMAP ≈ all frame; DESIGN ≈ all
  body; plan docs in between). The fidelity boundary (§5) is where this is decided per artifact.
- **Order vs self-hosting.** Does the ROADMAP emit need v2 self-host (the markdown medium on the
  self-hosted emitter), or can it land on the v1 seed emitter now? `ci.yml` lands on the seed today, so
  the ROADMAP can likely invert **before** full self-host — a near-term win, not gated on Route-A.
- **Relationship to [inert-layer-lens](inert-layer-lens.md).** The reachability lens is the *validation*
  form; this project is its *construction* form. Ship the lens first (cheap, catches drift today), then
  let emission dissolve the lens into a wall (the lens flips advisory→unnecessary once the artifact is
  generated). Same ratchet→wall arc.

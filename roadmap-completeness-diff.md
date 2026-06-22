# ROADMAP completeness diff — current main (245 ln) vs emit(authority) (152 ln)

Classification: `[intended-correction]` = one of the 6 status corrections · `[benign-rewrap]` =
pointer label/placement reframed, **path + content preserved** · `[GAP]` = content lost, must fix.

## HEADLINE FINDING — staleness, scope is bigger than the 4 named gaps

The authority was transcribed from a **stale ROADMAP base**, not current main. The 6 corrections were
applied correctly, but on an older/thinner skeleton. So beyond the 4 named gaps, **§1 is entirely stale**
(wrong title + wrong prose + wrong milestones + wrong groups) and **§2/§3/§4/§5/§6 each dropped current
nested items**. Fix = re-transcribe from current main faithfully, re-apply only the 6 corrections + benign
rewrap. The gate is unaffected — pure authority-completeness.

## Preamble
- `[GAP]` para2: dropped `— don't restate it here (no dual representations)` (the §2 no-dual-rep tag).
- `[GAP]` Legend: dropped the entire `Each section opens with a **◆ Milestones** spine — … Read L→R = the path.` sentence.

## ✦ Ergonomics LANE (main L60–88)
- `[GAP]` **ENTIRELY MISSING.** GAP 1. Whole lane (why-a-tier prose, ◆ Milestones, Audit half ×3,
  Fix half ×3, Own-runway ban-source-comments, Pairs-with prose). Restore as lead-lane (model shape below).

## §1  (main "CI as the substrate integration dogfood (the correctness floor)")
- `[GAP]` GAP 2 — **title** emitted as "CI under control (the correctness floor)"; revert to original.
- `[GAP]` **prose** fully rewritten/lost (the "flexes every substrate layer … forcing function … Deliverable = shared abstractions proven by CI consuming them …" para → replaced by a wrong 1-liner).
- `[GAP]` dropped section pointer `→ [charter: causal-chain gap analysis](docs/plans/ci-process-end-to-end.md) — …`.
- `[GAP]` **milestones** replaced (execution-as-DAG ✓ · width ✓ … → wrong opt-3 milestones).
- `[GAP]` dropped groups: **What's on `.dag` today** (4 items), **Host-operation band G1–G3 + CI-on-fabric**, **Adjacent gaps G4–G5 + floor-selection (+nested) + tree-scoped-registry + sccache-false-greens**, **Shared abstractions ×3**, **Downstream/parked ×4**. (My emit's §1 items are an older, different set.)
- NOTE: C1 (measure #5470/#5478) belongs as a §1 item per the correction; it currently rides on the stale skeleton — re-home it into the re-transcribed §1.

## §0
- `[GAP]` dropped section pointer `→ [audit + checklist](docs/plans/fail-closed-lockdown.md)`.
- `[GAP]` rust-gate item: dropped `; \`.dag\`→rust coverage wall = edge-(b), SCOPED / pending operator greenlight`, the 2nd pointer `[edge-(b) brief](…edge-b-rust-dag-provenance-brief.md)`, and `*(quick-ant-298)*`. (C5 = "no status change" — but the line's content was truncated; restore in full.)
- `[GAP]` promote-lenses: dropped `(also unlocks coverage/testgen)`.
- `[GAP]` realization-vocab: dropped `; dissolve-on: bash-sidecar arc empties the roster → pure wall → \`program.dag\` deletable`.
- `[GAP]` GAP 4 — **cardinality refinement** item (last in Fenced-OUT) MISSING. Restore (#5512 MVP-1/P4 landed; P1/P2/P5 pending; 2 pointers).
- `[GAP]` construction-justification: dropped 2nd pointer `[DESIGN §6](DESIGN.md)`.
- `[GAP]` axiom+syllogism: dropped pointer `[scope](docs/plans/axiom-syllogism-lens.md)`.
- `[benign-rewrap]` fork / remaining / numeric-tower / disposition / confront-skipped: pointer label or paren-placement reframed to ` [plan](path)`; path preserved.
- `[benign-rewrap]` cross-tree (§5)/(#5473)/LANDED token reorder; content preserved.

## §2
- `[GAP]` dropped the whole nested cache-realization subtree under F2/F3: **P1 honest-keys (#5429) → P2 one-door realize() (#5446) [+ M4.1 #5236 / M5 hermetic fixtures] → P3 resolve-cache enable (core ask)**.

## §3
- `[GAP]` dropped nested items: **a subject-producer for every fn (#5437 helper)** → **complexity budget gates the whole codebase**.

## §4
- `[GAP]` dropped nested **anemia lens? (parked, DESIGN §2 leaf-side)** under affected-set.
- `[benign-rewrap]` oracle-method pointer `[map](…) §2` → `[plan](…)`; path preserved.

## §5
- `[intended-correction]` C4 — milestones add `· class-3 corpus-coherence + cargo-green seed ✓ (#5481)` and the `*(§7 regen-fixpoint deferred, #5514; src/v1 NOT yet deletable)*` no-overclaim tail. ✓
- `[GAP]` dropped **adjacent-lane prose** (algorithmic-cost rewrite engine, → plan).
- `[GAP]` dropped nested under de-fork: **collapse clear duplicates (algebra/logic/nat/reducible/measure)** + **resolve same-name/different-job pairs**.
- `[GAP]` collapsed the emitted-crate subtree into 1 line; dropped **real fixed point**, **regen --verify keystone (+ dissolve seed hand-patches)**, **TypeScript to first-class**, **seed-honesty (DDC)**, **collapse src/v1 (terminal)** as distinct nested items.

## §6
- `[intended-correction]` C2 — **round-trip law (#5525/#5527)** row added as own line, distinct from medium-axis, no overclaim. ✓
- `[intended-correction]` C3 — **PR→checkbox** expanded with slice-1/2/3a (#5491/#5508/#5520). ✓
- `[GAP]` dropped nested **`Medium<A> ↔ Medium<B>` homomorphisms** under cross-media-targets.

## §7, §8
- Clean. No deltas.

## Corrections accounted for
- C1 measure #5470/#5478 — present but on stale §1; re-home into re-transcribed §1.
- C2 medium/round-trip #5525/#5527 — ✓ §6.
- C3 inversion slices 1/2/3a — ✓ §6.
- C4 self-host PATH-A #5481 / §7-deferred #5514 / src/v1-not-deletable — ✓ §5.
- C5 edge-b rust-gate scoped/no-status-change — line restored in full per §0 GAP above (no `[x]`/`[ ]` flip).
- C6 (the 6th) — verify against warm-lark's list during re-transcription.

**Target after re-transcription: ZERO [GAP].**

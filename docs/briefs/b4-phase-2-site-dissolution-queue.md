# B4 Phase 2 — Site dissolution queue (skeletons, dispatch-as-Phase-1-lands)

> **Tracking doc, not a worker brief.** Reports through Substrate Manager
> (post-R2 spin-up). Names the Phase 2 mechanical site-dissolution work
> deferred from the
> [B4 Identity-Carrier Substrate Pass program](b4-identity-carrier-substrate-pass.md)
> (merged via #814).
>
> Phase 2 dispatches **as each Phase 1 carrier lands**; site dissolutions
> are mechanical follow-up PRs once substrate is in place. Worker briefs
> are skeleton-only here; full content authored by Substrate Manager
> (or Director pre-spin-up) at dispatch time per the carrier readiness
> signal.

## Phase 2 dispatch matrix

The B4 program identified **8 surface dissolution sites** (§0.1 through §0.8).
Each Phase 1 carrier subsumes its primary consumer site in the same-PR
migration; Phase 2 covers **secondary consumers** of the same carrier
that didn't land in the carrier PR.

| # | §0 site | File:line | Phase 1 carrier | Phase 2 dispatch trigger |
|---|---------|-----------|-----------------|--------------------------|
| B4.5 | §0.5 | `lower.rs:836` — `span.file == "dsl/std/types.dag"` type-alias bridge | B4.1 (`DeclarationRef` migration) | After B4.1 first-consumer (#823) merged + B4.1b residual (#826 + Slice 4-5) closes; this site is **not in B4.1's known scope**, dispatch as separate Phase 2 consumer-migration PR. |
| B4.6 | §0.7 | `dag.rs:2735-2764` + `lower.rs:1451-1452, 1546-1547` — `declaration_name_preference_rank(&span.file)` | B4.1 (`DeclarationRef` migration) | After B4.1 carrier landing fully covers DeclarationRef paths; this site cross-cuts file-preference logic and may share substrate territory with PB-Tier1-Sweep. **Cross-program coordination with Pure Bootstrap Manager.** |
| B4.7 | §0.4 secondary | `lens_apply.rs` siblings (if any) of `:38, :372-383` | B4.2 (fold-shape carrier) | After B4.2 lands; audit for any sibling fold-skip consultation of `span.file` not covered in B4.2's same-PR migration. |
| B4.8 | §0.6 secondary | `emit.rs` siblings (if any) of `:3181, :3206` | B4.3 (emit-helper carrier) | After B4.3 lands; production-consumer audit on `primitive_type_id_for_port_shared` / `walk_to_disj` / siblings. |
| B4.9 | §0.8 secondary | `bootstrap_regen_fresh.rs` regen-host filter | B4.4 (extdeps-fixture-set carrier) | If B4.4 lands shape (b) (authority + tracked debt), full dissolution of regen-host filter is a Phase 2 follow-up tracked via ROADMAP debt row. **Cross-program coordination with Pure Bootstrap Manager.** |
| B4.10 | (Phase 3) | n/a | n/a | **Phase 3 reviewer-discipline ratchet** (per program brief): no new `span.file ==` / `span.file.ends_with` / sentinel-string sites in `src/v3/compiler/src/`. Lands as PR-template-line addition + reviewer-discipline note. Authored by R2 Release Manager (per #827 dispatch-discipline framework enforcement). |
| B4.11 | reserved | n/a | n/a | Reserved for surfaces that emerge during Phase 1 audits (e.g., a fifth carrier surfaces during B4.2 / B4.3 / B4.4 work). |
| B4.12 | reserved | n/a | n/a | Reserved for downstream emit/lens consumers that surface during integration testing. |

## Skeleton brief shape (each Phase 2 worker brief)

When Substrate Manager dispatches a Phase 2 worker brief, the shape is:

```markdown
# B4.<N> — <site name> consumer migration `(S; B4 Phase 2)`

> **Worker brief.** Reports through Substrate Manager. Phase 2 site
> dissolution following B4.<phase-1-#> merger.

## Read first
- B4 program brief (`docs/briefs/b4-identity-carrier-substrate-pass.md`)
- Phase 1 carrier brief (`docs/briefs/b4-<phase-1-#>-...-worker.md`) — merged
- Cited file:line site

## Frame
The B4.<phase-1-#> carrier landed in <PR>; this brief migrates the
secondary consumer at <site> from `span.file ...` to structural query.

## Slice
1. Replace `span.file == "..."` (or equivalent) with structural query against the carrier.
2. Regression test: site dispatches structurally without the file-name marker.
3. DB-8 fixed-point converges.

## Acceptance
- Site cited in §0 dissolves via structural query.
- No replacement sentinel string introduced.
- Regression test added.
- DB-8 converges.
```

## Phase 2 cross-program coordination

- **Pure Bootstrap Manager:** B4.6 (file-preference rank) and B4.9 (regen-host filter) cross-cut PB territory. Coordinate at dispatch time per #827's cross-program handoff pattern.
- **R2 Release Manager:** B4.10 Phase 3 reviewer-discipline ratchet is owned by Release Manager per the #810 dispatch-discipline framework.
- **Modeling Manager / Impossible-Bugs Manager:** no Phase 2 cross-program dependencies known at authoring time.

## Status (at brief authoring, 2026-04-26)

- B4.1 first-consumer landed (#823); residual via #826 (open, regen drift on `r1_gates.dag`); Slice 4-5 (full LensOutputEquals + DifferentialEquals migration) implied as B4.1b — not yet authored as separate worker brief.
- B4.2 brief authored (`b4-2-structural-fold-shape-carrier-worker.md`); no PR yet.
- B4.3 PR open (#824); CI failing; worker iterating.
- B4.4 PR open (#825); CI failing; worker iterating; flagged parallel-rep concern.

## Reporting

This is a tracking doc, not a single PR. Each Phase 2 worker brief lands
as its dispatch trigger fires (per the matrix above). Substrate Manager
maintains the matrix; Director (pre-spin-up) maintains via this file.

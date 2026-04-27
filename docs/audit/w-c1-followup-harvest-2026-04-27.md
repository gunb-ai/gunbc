# W-C1 Follow-up Harvest Table (2026-04-27)

**Lane:** Cleanup (W-C1)
**Owner:** session calm-ant-861 (R2-cleanup)
**Dispatch:** tidy-dove-734 inbox #941 → calm-ant-861 inbox #942
**Control surface:** No `ctrl#263` issue exists in repo (PR #263 is a closed/merged bootstrap PR, unrelated). Per dispatch fallback, this single docs/audit artifact is the agreed control surface for the W-C1 cleanup harvest. New rows append here; no per-row issues are opened unless a row's tracking authority demands it.

## Disposition legend

- **closed** — finding addressed pre-merge or dissolved by a later landed change.
- **tracked** — finding stands; tracked by an existing ROADMAP row or peer worker; no new authority needed.
- **needs worker** — finding stands; no existing tracker; row below is the authority.

Coordination: bright-wolf-465 owns deeper archaeology for #897/#824/#825. Rows below are kept narrow so they can be dropped/merged when bright-wolf-465 reports.

## Rows

| # | Source PR | Gap | File / Invariant | Owner lane | Dissolution trigger | Acceptance check | Tracking authority | Disposition |
|---|---|---|---|---|---|---|---|---|
| 1 | #900 | NominalOpacity carrier landed as opt-in field, not yet enforced | `src/v3/compiler/src/dag.rs:222` (`pub nominal_opacity: Option<NominalOpacity>`); walker has no fail-closed read | Modeling (#858) | Walker enforces NominalOpacity at every consumer; promote `Option<NominalOpacity>` to non-optional once Secret<T> graduates | `nominal_opacity: None` initialiser at `dag.rs:3689` is gone; walker raises Diagnostic on opacity violation; Secret<T> is the canonical user surface | control-table-only; cross-post to Modeling #858 | needs worker |
| 2 | #900 | Diagnostic for opacity violations is hand-Rust, not authored in `.dag` | (no `.dag` source yet) | Modeling (#858) | Author opacity Diagnostic in `dsl/std/` once walker enforcement lands | Diagnostic emitted from compiled `.dag`, not `src/v3/compiler/src/*.rs` | control-table-only | needs worker |
| 3 | #900 | Test fixture seeded `Some(NominalOpacity{..})` then re-set `nominal_opacity: None`, so the carrier path is not exercised — original BLOCKING (sha 42d4ef41) | `src/v3/compiler/src/dag.rs:3689` trailing `nominal_opacity: None` initialiser still present at HEAD (3a18fa80b) | R2-cleanup | Drop the trailing `None` so the Some-branch is compiled & tested | `grep -n nominal_opacity src/v3/compiler/src/dag.rs` shows no shadowing initialiser; test exercises the Some path | control-table-only | needs worker |
| 4 | #901 | `rest_request_wire_serde_alignment` not closed; opaque-Json removal landed but provider-specific wire shapes still missing — original BLOCKING (sha 6f494af6, 6 findings) | `dsl/extdeps/github/auth.dag` (no scope/expiry evidence on Secret-Manager token); `dsl/extdeps/llm/anthropic.dag` (role/content-block variants collapsed to product); `dsl/extdeps/llm/openai.dag` (role-keyed union flattened) | R2-cleanup or B4-substrate | Model role-discriminated wire carriers (anthropic/openai); model token scope+expiry as typed evidence (auth.dag) before any caller relies on typed-body 200s | Round-trip tests for each provider's wire shape pass; no caller pattern-matches on `Json` for these endpoints | ROADMAP row required (gates typed-body 200 callers) | needs worker |
| 5 | #920 | Bot review absent on merge commit; UCD citation missing from PR body | n/a (process gap) | R1-process / Cleanup | Re-run codex review on merge SHA `40740a6b`; add UCD citation pointer (or back-fill in next docs sweep) | PR #920 has a non-environmental codex review; UCD line present in either PR description or a follow-up docs commit | control-table-only | needs worker |
| 6 | #897 | ~~Annotated-let literal narrowing skips diagnostic-producing range/refinement check~~ | `src/v3/compiler/src/infer.rs` annotated-let literal seed path | R2-cleanup | — | bright-wolf-465 audit (inbox #945, 2026-04-27): annotated-let retry gated with diagnostic path + Diagnostic emission landed pre-merge; later codex re-review approved | — | closed (bright-wolf-465) |
| 7 | #824 | ~~Latest scheduled review env-failed; merge-SHA re-review needed~~ | n/a (review-coverage gap) | R1-process | — | bright-wolf-465 audit (#945): bootstrap snapshots refreshed pre-merge; serializer/tests fixed to Python literals; later codex approved/no findings | — | closed (bright-wolf-465) |
| 8 | #825 | ~~B4.4 carrier wired into fresh-regen but committed bootstrap snapshot + parity loop missing~~ | `src/v3/compiler/src/bootstrap.rs`; `bootstrap_generated.rs` / `bootstrap_generated_without_parse_surface.rs` | R2-cleanup | — | bright-wolf-465 audit (#945): committed snapshots regenerated pre-merge; parity test added; accessor/assert semantics tightened; later codex approved | — | closed (bright-wolf-465) |

## Notes

- Rows 1–3 (#900) are kept distinct because the dissolution triggers are independent (walker, `.dag` Diagnostic authoring, and the fixture compile-shadow bug). Folding them risks losing the fixture row, which is a concrete pre-merge BLOCKING that the env-failed re-review never re-checked.
- Row 5 (#920) is process-only; if R1-process surfaces a recurring bot-review-on-merge gap, promote to ROADMAP rather than expanding rows here.
- Rows 6–8 are flagged `tracked (bright-wolf-465)` per dispatch coordination note. If bright-wolf-465 reports the findings landed pre-merge, flip disposition to `closed` and strike the row body (keep the row id for audit).
- This file is the W-C1 control surface. Future cleanup-lane harvest entries append below the existing rows; do not split into per-PR files.

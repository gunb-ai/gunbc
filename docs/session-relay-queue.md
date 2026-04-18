# Session relay queue (dashboard / GitHub ingests)

Operational pointers relayed into the worktree — **not** product authority (contrast [`phase-plan-2026-04-18.md`](./phase-plan-2026-04-18.md), ROADMAP). Delete or trim a row when the review completes or the item is obsolete.

## still-deer-308 — PR [#530](https://github.com/gunb-ai/gunbc/pull/530)

**Head:** `ca4540dcc03b89357ee087e2e7eda6269b49e09d`

| Ingested (UTC) | Source | Summary |
|---|---|---|
| 2026-04-18 ~15:26 | ChatGPT auto-review (placeholder) | Superseded by later comments — was “check back ~30m.” |
| 2026-04-18 15:26:46Z | Claude review (`[claude-review]` sha `ca4540d`) | **Approve.** Confirms C+D bundle: Track 9 / 3a.1 / M2 / §4.1 memos / cross-ROADMAP v3 notes / DB-numbering fix / migration-candidates section. **Explicitly not in PR (OK):** `ParamRef`/`TransformRef` substrate.dag comment (defer tiny PR or next substrate touch); #519 planner vs `is_first` code (separate XS PR); `feedback_substrate_principle_audit` → INVARIANTS graduation (trigger-gated). |
| 2026-04-18 15:27:05Z | Codex connector (bot) | Boilerplate only — no code suggestions in excerpt. |
| 2026-04-18 15:27:06Z | Codex inline `src/v3/ROADMAP.md:30` | **P2:** Reconcile M2 **status table** vs **`## M2`** detailed bullets (contradiction). **Addressed in repo:** `## M2` rewritten into Landed vs Still open aligned with §Status at a glance + Lane 3 Stage 3a table. |
| 2026-04-18 15:43:01Z | ChatGPT auto-review `sha:ca4540dcc…` **status:complete** | **APPROVE_WITH_COMMENTS.** Principle audit: doc hygiene OK; **NON-BLOCKING** — §4.1 must not become a second authority for gates. **Follow-up applied in repo:** `docs/phase-plan-2026-04-18.md` §4.1 **trimmed to pointer index**; **Lane 1e + determinism (DB-8)** mirrored in `docs/post-l15-phase-plan.md` Lane 1 summary. [View conversation](https://chatgpt.com/g/g-p-69d1a39d61e88191835a38f9eba3ec9b-auto-review/c/69e3a260-873c-83ea-9f43-a2557027ae71) |
| 2026-04-18 15:45:10Z | Codex review (`[codex-review]` `ca4540dc`) | **Approve** (blocking 0). **NON-BLOCKING:** M2 summary vs `## M2` could still fork authority — **addressed:** `## M2` now points shipped scope **only** at § Lane 3 Stage 3a table; duplicate “Landed” prose list removed. Otherwise verified Track 9/3a.1 / DB memos grounded. |
| 2026-04-18 15:45:11Z | Codex review (duplicate relay, same `sha:ca4540dcc…`) | **No delta** from 15:45:10Z row — same body (extra HTML comment line in GitHub only). M2 fork already resolved: `src/v3/ROADMAP.md` has `## M2 — Feature parity (absorbed into Lane 3 Stage 3a)` + § Lane 3 Stage 3a table as shipped authority. |

### Follow-ups (from Claude review — not blockers for #530)

1. ~~**`substrate.dag` cross-ref**~~ — **landed:** `src/v3/std/substrate.dag` comments on `ParamRef` / `TransformRef` point at `ROADMAP.md` Track 9 tracked-debt.
2. ~~**Planner vs `is_first`**~~ — **landed** in `lower.rs`: `compute_mutually_recursive` takes `is_first` and skips duplicate `fn` items; see `mutual_recursion_planner_respects_is_first_on_duplicate_fn`.
3. **`feedback_substrate_principle_audit` → INVARIANTS.md** — defer until graduation trigger (e.g. citation threshold) per phase plan.

### Codex / #530 (non-blocking)

4. ~~**M2 summary vs `## M2` authority fork**~~ — **landed (Codex 15:45):** `src/v3/ROADMAP.md` `## M2` is tail-gaps only; **§ Lane 3 Stage 3a** sub-stage table owns what shipped for 3a.1–3a.5.

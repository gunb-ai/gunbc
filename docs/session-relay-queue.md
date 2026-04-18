# Session relay (pointer)

Transient PR review and CI state lives on **GitHub** — not as a hand-maintained mutable ledger in git (ChatGPT #530 review, 2026-04-18: coordination docs should **point**, not copy live bot/review fields).

- **PR #530 —** https://github.com/gunb-ai/gunbc/pull/530 — use the PR **Conversation** and **Checks** tabs for current threads and status.

- **PR #540 —** https://github.com/gunb-ai/gunbc/pull/540 — Stage 1d emitter consolidation design docs (`emit-functions-inventory.md`, `spec-field-gaps.md`, `emit-bridges.md`). Same tabs for review threads and CI.

**Latest ingest — PR #540 `chatgpt-codex-connector` inline @ 2026-04-18T20:04:47Z (`docs/emit-bridges.md:54`, P2):** **ADDRESSED.** Bridge “site counts” table drifted from methodology §B: **`named_variant_id(dag,` is 29/8/3 (40 total)**, not the broader `named_variant_id(` line count. Table now has **per-file `rg -c` columns**, separate **B** vs **B′** rows, **A+B+C+D = 84**, and **E** (`_0` payload) called out — reproducible from repo root.

**Also pending — PR #540 ChatGPT auto-review `sha:4a0c5dedfb48b046164e72415ada26b4becb04a0` @ 2026-04-18T20:03:45Z:** **PENDING.** Placeholder comment only — full verdict not yet posted; use the [conversation](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e3e39f-d4b8-83ea-a330-3a17941f70e2) for the completed review. _(That SHA predates follow-up doc commits on the branch; re-run or re-read against current `session/stern-badger-526` HEAD if the bot does not auto-update.)_

**Previous ingest — PR #530 ChatGPT auto-review `sha:4014aa346e67b88090b47055ebf7061c9d38b7ee` complete @ 2026-04-18T16:31:14Z:** **APPROVE_WITH_COMMENTS.** Confirms `lower.rs` `is_first` → mutual-recursion planner + regression test; doc pointer/index cleanup (phase-plan §4.1, M2 vs Lane 3 Stage 3a, `post-l15` → DB-8 determinism); substrate `ParamRef` / `TransformRef` comment-only tracked debt. **Non-blocking:** phase-plan header should not mirror bot verdicts — **addressed** in [`phase-plan-2026-04-18.md`](./phase-plan-2026-04-18.md) **Since last refresh** (this ingest); optional follow-up if `compute_mutually_recursive` gains callers: a first-authority view vs parallel `(items, is_first)` slices. [View conversation](https://chatgpt.com/g/g-p-69d1a39d61e88191835a38f9eba3ec9b-auto-review/c/69e3ada0-b13c-83ea-9da3-00184b788a28)

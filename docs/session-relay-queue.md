# Session relay (pointer)

**Single authority:** PR review threads, bot comments, inline findings, and CI status are **authoritative on GitHub** — not duplicated as a mutable ledger in this repo (see [PR #530 discussion](https://github.com/gunb-ai/gunbc/pull/530), ChatGPT review 2026-04-18: docs should **point**, not copy live review fields).

| PR | URL |
|:---|:---|
| #530 | https://github.com/gunb-ai/gunbc/pull/530 |
| #540 | https://github.com/gunb-ai/gunbc/pull/540 |
| #556 (SG-1, session `neat-pike-779`) | https://github.com/gunb-ai/gunbc/pull/556 |

#### #556 relay index (dashboard ingest — pointers only)

Full text stays on **GitHub** / linked ChatGPT threads; this table is a stable handle for agents.

| Source | Link |
|:---|:---|
| Human — blocking SG-1 brief (cutover, semantics, corpus parity) | [issue comment](https://github.com/gunb-ai/gunbc/pull/556#issuecomment-4275393872) |

ChatGPT rounds (parent commit SHA in marker):

| SHA (prefix) | Thread status | Verdict / one-line theme | Conversation |
|:---|:---|:---|:---|
| `fd6ef9e0486cf1fa01081f1cf41b581405057108` | complete | **REQUEST_CHANGES** — second syntax authority vs `extdeps.languages.dag.syntax`; choose one authority, or name SG-1 as explicit scaffold; derive/check `PunctSpec.width` from `pattern`. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e47f3d-9908-83ea-b617-779ef6a88804) |
| `06b14483d9bb589ffc9df161a11921f42ef96c71` | complete | **APPROVE_WITH_COMMENTS** — `.dag` + `regen_tokenize` + drift test; follow-ups: derive `width`, structural `kind_name` later, clarify lexer-vs-tables scope. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be) |
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | pending | `status:pending` stub @ 2026-04-19T08:01:39Z — full review not posted yet; refresh [PR #556](https://github.com/gunb-ai/gunbc/pull/556) for a later `status:complete` with the same SHA. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb) |

**`06b14483…` follow-ups** (same round as complete @ 2026-04-19T07:55:14Z — **APPROVE_WITH_COMMENTS**, non-blocking): derive `width` from `pattern` and drop redundant `width`; later replace `kind_name: String` with a structural ref to the `TokenKind` variant when value bodies allow; if SG grows past closed tables, decide whether comment / string-escape / identifier rules stay in this `.dag` or remain codegen by design. Full prose: [PR #556 conversation](https://github.com/gunb-ai/gunbc/pull/556) → same SHA marker / [ChatGPT thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be).

**Remaining bot queue:** one round — `37c3d25…` (stub @ 2026-04-19T08:01:39Z) still `pending` on GitHub until a `status:complete` comment with the same parent SHA lands.

**Superseded bot stubs:** For a given parent SHA, ChatGPT often posts `status:pending` (“review in progress…”) first, then a later comment with `status:complete` for the **same** SHA (e.g. `06b14483…`: pending stub @ 2026-04-19T07:31:40Z → complete @ 07:55:14Z on [PR #556](https://github.com/gunb-ai/gunbc/pull/556)). The table rows are the **final** outcome per SHA, not the queue at a single timestamp.

For each PR: use **Conversation** (reviews, bots, threads) and **Checks** (CI).

Stage 1d design artifacts carried on #540: [`emit-functions-inventory.md`](./emit-functions-inventory.md), [`spec-field-gaps.md`](./spec-field-gaps.md), [`emit-bridges.md`](./emit-bridges.md).

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
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | pending | Awaiting full review body on thread. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb) |

For each PR: use **Conversation** (reviews, bots, threads) and **Checks** (CI).

Stage 1d design artifacts carried on #540: [`emit-functions-inventory.md`](./emit-functions-inventory.md), [`spec-field-gaps.md`](./spec-field-gaps.md), [`emit-bridges.md`](./emit-bridges.md).

# Session relay (pointer)

**Single authority:** PR review threads, bot comments, inline findings, and CI status are **authoritative on GitHub** — not duplicated as a mutable ledger in this repo (see [PR #530 discussion](https://github.com/gunb-ai/gunbc/pull/530), ChatGPT review 2026-04-18: docs should **point**, not copy live review fields).

| PR | URL |
|:---|:---|
| #530 | https://github.com/gunb-ai/gunbc/pull/530 |
| #540 | https://github.com/gunb-ai/gunbc/pull/540 |
| #556 (SG-1, session `neat-pike-779`) | https://github.com/gunb-ai/gunbc/pull/556 |

#### #556 relay index (dashboard ingest — pointers only)

Full text stays on **GitHub** (and linked external threads where applicable); this index is a stable handle for agents.

| Source | Link |
|:---|:---|
| Human — blocking SG-1 brief (cutover, semantics, corpus parity) | [issue comment](https://github.com/gunb-ai/gunbc/pull/556#issuecomment-4275393872) |

ChatGPT rounds (parent commit SHA in marker):

| SHA (prefix) | Thread status | Verdict / one-line theme | Conversation |
|:---|:---|:---|:---|
| `fd6ef9e0486cf1fa01081f1cf41b581405057108` | complete | **REQUEST_CHANGES** — second syntax authority vs `extdeps.languages.dag.syntax`; choose one authority, or name SG-1 as explicit scaffold; derive/check `PunctSpec.width` from `pattern`. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e47f3d-9908-83ea-b617-779ef6a88804) |
| `06b14483d9bb589ffc9df161a11921f42ef96c71` | complete | **APPROVE_WITH_COMMENTS** — `.dag` + `regen_tokenize` + drift test; follow-ups: derive `width`, structural `kind_name` later, clarify lexer-vs-tables scope. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be) |
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | complete | **APPROVE_WITH_COMMENTS** — net authority win; keep `kind_name: String` + `keyword_*` / `punct_*` / `string_escape_*` prefix scans as **explicit scaffolds** until typed refs / structural tables exist. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb) |

**Duplicate `chatgpt-review` comments:** [PR #556](https://github.com/gunb-ai/gunbc/pull/556) may show **more than one** `status:complete` issue comment with the **same** parent SHA and the same body — e.g. `fd6ef9e…` @ 2026-04-19T07:25:09Z and again @ **08:26:30Z** — still **one** ChatGPT verdict (**REQUEST_CHANGES** for that SHA); do not double-count.

**ChatGPT meta-review** (`<!-- [chatgpt-meta-review] sha:… -->` — **loop-health** across the PR’s review cycle; distinct from per-diff `chatgpt-review`):

| Parent SHA | Posted (UTC) | Thread status | One-line | Conversation |
|:---|:---|:---|:---|:---|
| `3fc44f8456a992d001ccbc6b21cb2bd53afb35a9` | 2026-04-19T08:26:49 | pending | Stub: “meta-review in progress…” — await `status:complete` on [PR #556](https://github.com/gunb-ai/gunbc/pull/556). | [ChatGPT (meta)](https://chatgpt.com/g/g-p-69e42928d3ec819192415828194826d4-gunbc-review/c/69e491ab-4a94-8325-b0e6-d4502ea885cb) |

**Codex** (`<!-- [codex-review] sha:… -->` markers — full text on [PR #556](https://github.com/gunb-ai/gunbc/pull/556)):

| Parent SHA | Posted (UTC) | Blocking | One-line | Model / tool |
|:---|:---|:---|:---|:---|
| `7c53b24f910a8212434d8aa666fbd0ded08a0ea9` | 2026-04-19T08:25:10 / 08:25:11 | 0 | Clean SG-1 cutover; `tokenize.rs` exposes only generated tokenizer; `sg1_tokenize_authority_test` = consumer + freshness proof. | `codex` · `gpt-5.4` |
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | 2026-04-19T08:27:11 | 0 | **ROADMAP SG-1 slice verified**; non-blocking: DAG `Token { kind, span }` vs hardcoded Rust `Token` in `regen_tokenize` — project from Dag or drop unused declaration in a follow-up lane. | `codex` · `gpt-5.4` |

**Same SHA, different tools:** A `codex-review` parent SHA may **equal** a `chatgpt-review` parent SHA (here `37c3d25…`) because both key off the same PR head — **independent** verdicts; index **Codex** and **ChatGPT** rows separately.

**Duplicate `codex-review` comments:** [PR #556](https://github.com/gunb-ai/gunbc/pull/556) may show **two** issue comments a second apart with the **same** parent SHA and duplicated `<!-- [codex-review] -->` lines — still **one** Codex verdict; do not double-count.

**`06b14483…` follow-ups** (complete @ 2026-04-19T07:55:14Z — **APPROVE_WITH_COMMENTS**, non-blocking): derive `width` from `pattern` and drop redundant `width`; later replace `kind_name: String` with a structural ref to the `TokenKind` variant when value bodies allow; if SG grows past closed tables, decide whether comment / string-escape / identifier rules stay in this `.dag` or remain codegen by design. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be).

**`37c3d25…` follow-ups** (complete @ 2026-04-19T08:24:54Z — **APPROVE_WITH_COMMENTS**, non-blocking): dissolve `kind_name: String` when value bodies carry typed declaration references; dissolve prefix-based row collection when the surface can author an explicit structural table; optional tiny scaffold note on `KeywordSpec` / `PunctSpec` or **ROADMAP** row naming triggers. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb).

**Indexed automated rounds:** **ChatGPT** (per-diff) — three **distinct** parent SHAs **complete** (table above). **ChatGPT meta-review** — `3fc44f8456a992d001ccbc6b21cb2bd53afb35a9` still **pending** (stub @ 2026-04-19T08:26:49Z). **Codex** — `7c53b24f…` posted, **0** blocking. Other **open** items on #556 sit outside these markers — e.g. the **human** blocking brief, **Checks**, inline threads — see [PR #556](https://github.com/gunb-ai/gunbc/pull/556). Dashboard “+N queued” counts often include **duplicate** bot posts and human/CI work — **dedupe by parent SHA** and marker kind (`chatgpt-review` vs `chatgpt-meta-review` vs `codex-review`).

**Superseded bot stubs:** For a given parent SHA, ChatGPT often posts `status:pending` first, then `status:complete` for the **same** SHA (e.g. `06b14483…`: stub @ 2026-04-19T07:31:40Z → complete @ 07:55:14Z; `37c3d25…`: stub @ 08:01:39Z → complete @ 08:24:54Z on [PR #556](https://github.com/gunb-ai/gunbc/pull/556)). The same **pending → complete** pattern applies to **`chatgpt-meta-review`**. The table rows are the **final** outcome per SHA, not the queue at a single timestamp.

For each PR: use **Conversation** (reviews, bots, threads) and **Checks** (CI).

Stage 1d design artifacts carried on #540: [`emit-functions-inventory.md`](./emit-functions-inventory.md), [`spec-field-gaps.md`](./spec-field-gaps.md), [`emit-bridges.md`](./emit-bridges.md).

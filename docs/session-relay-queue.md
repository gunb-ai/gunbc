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
| Human — **inline** on `src/v3/compiler/tokenize.dag` (~line 6), **2026-04-19T09:30:30Z** — **BLOCKING**: SG-1 tokenizer authority overlaps **`.dag` frontend** keyword/operator facts in `dsl/extdeps/languages/dag/syntax.dag` → **second syntax source** without the **named scaffold trigger** single-authority discipline expects | [PR #556](https://github.com/gunb-ai/gunbc/pull/556) (see **Files** / inline threads — authoritative text on GitHub) |

ChatGPT rounds (parent commit SHA in marker):

| SHA (prefix) | Thread status | Verdict / one-line theme | Conversation |
|:---|:---|:---|:---|
| `fd6ef9e0486cf1fa01081f1cf41b581405057108` | complete | **REQUEST_CHANGES** — second syntax authority vs `extdeps.languages.dag.syntax`; choose one authority, or name SG-1 as explicit scaffold; derive/check `PunctSpec.width` from `pattern`. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e47f3d-9908-83ea-b617-779ef6a88804) |
| `06b14483d9bb589ffc9df161a11921f42ef96c71` | complete | **APPROVE_WITH_COMMENTS** — `.dag` + `regen_tokenize` + drift test; follow-ups: derive `width`, structural `kind_name` later, clarify lexer-vs-tables scope. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be) |
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | complete | **APPROVE_WITH_COMMENTS** — net authority win; keep `kind_name: String` + `keyword_*` / `punct_*` / `string_escape_*` prefix scans as **explicit scaffolds** until typed refs / structural tables exist. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb) |
| `e529d136eb40f7ca6411e2a21b66e4165abd9506` | complete | **REQUEST_CHANGES** — second live syntax authority vs `extdeps.languages.dag.syntax` (drift example: `then` in `tokenize.dag` vs `dag_keyword_set`); before merge: derive overlapping lexer facts from existing syntax authority, **or** explicit scaffold + ROADMAP receipt + alignment test for shared keyword/operator subset. | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e492ca-c498-83ea-853c-b181f6e0a3cf) |
| `527d7fd3724c4609923de67ba04ccb30b3b39322` | complete | **APPROVE_WITH_COMMENTS** — practical single tokenizer authority + projection + freshness test; non-blocking hardening: validate `StringEscapeSpec.output_codepoint` at **regen** (avoid user-path panic from bad table); optional: stricter `kind_name` → zero-payload `TokenKind` checks. Deeper `tokenize.dag` vs `extdeps.languages.dag.syntax` story still a design follow-up (see also `fd6ef9…` / `e529…`). | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e499d2-c71c-83ea-a3d6-6e8b00a52627) |
| `1cbda4f9502e34eda4113138454900521d91ca9f` | pending | Stub @ 2026-04-19T09:31:36Z — `status:pending` — await `status:complete` on [PR #556](https://github.com/gunb-ai/gunbc/pull/556). | [ChatGPT](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e4a0da-3f90-83ea-956c-caf2c15e9c52) |

**Duplicate `chatgpt-review` comments:** [PR #556](https://github.com/gunb-ai/gunbc/pull/556) may show **more than one** `status:complete` issue comment with the **same** parent SHA and the same body — e.g. `fd6ef9e…` @ 2026-04-19T07:25:09Z / **08:26:30Z**; **`06b14483…`** @ **07:55:14Z** / **09:25:06Z**; **`37c3d25…`** @ **08:24:54Z** / **09:25:07Z** — still **one** ChatGPT verdict per SHA; do not double-count.

**ChatGPT meta-review** (`<!-- [chatgpt-meta-review] sha:… -->` — **loop-health** across the PR’s review cycle; distinct from per-diff `chatgpt-review`):

| Parent SHA | Posted (UTC) | Thread status | One-line | Conversation |
|:---|:---|:---|:---|:---|
| `3fc44f8456a992d001ccbc6b21cb2bd53afb35a9` | 2026-04-19T08:26:49 / 08:31:06 | complete | **SHIP_WITH_DEBT** — loop converged: canonical `tokenize.dag` + `regen_tokenize` + snapshot test; remaining debt localized to generator scaffolds (`kind_name`, `width`); post-merge follow-ups acceptable. Full meta on [PR #556](https://github.com/gunb-ai/gunbc/pull/556). | [ChatGPT (meta)](https://chatgpt.com/g/g-p-69e42928d3ec819192415828194826d4-gunbc-review/c/69e491ab-4a94-8325-b0e6-d4502ea885cb) |

**Codex** (`<!-- [codex-review] sha:… -->` markers — full text on [PR #556](https://github.com/gunb-ai/gunbc/pull/556)):

| Parent SHA | Posted (UTC) | Blocking | One-line | Model / tool |
|:---|:---|:---|:---|:---|
| `7c53b24f910a8212434d8aa666fbd0ded08a0ea9` | 2026-04-19T08:25:10 / 08:25:11 / 09:25:23 | 0 | Clean SG-1 cutover; `tokenize.rs` exposes only generated tokenizer; `sg1_tokenize_authority_test` = consumer + freshness proof. | `codex` · `gpt-5.4` |
| `37c3d25f1283cba04d55b3d9ae9a24c724f96e67` | 2026-04-19T08:27:11 / 08:27:12 | 0 | **ROADMAP SG-1 slice verified**; non-blocking: DAG `Token { kind, span }` vs hardcoded Rust `Token` in `regen_tokenize` — project from Dag or drop unused declaration in a follow-up lane. | `codex` · `gpt-5.4` |
| `da52ee9ac9ca7b94dc22c3b32465627dafd7297f` | 2026-04-19T09:30:30 / 09:30:31 | **1** | **BLOCKING**: overlapping keyword/operator facts — `dsl/extdeps/languages/dag/syntax.dag` (`SyntaxSpec` / shared set for v2 oracle) vs `tokenize.dag`; **either** derive shared lexer subset from `dag_syntax_spec` **or** explicit bounded scaffold + alignment test + named dissolution trigger. Non-blocking: validate `StringEscapeSpec.output_codepoint` in `regen_tokenize`. ROADMAP: SG-1 path **verified**; single `.dag` syntax authority **incomplete** per review. | `codex` · `gpt-5.4` |

**Same SHA, different tools:** A `codex-review` parent SHA may **equal** a `chatgpt-review` parent SHA (here `37c3d25…`) because both key off the same PR head — **independent** verdicts; index **Codex** and **ChatGPT** rows separately.

**Duplicate `codex-review` comments:** [PR #556](https://github.com/gunb-ai/gunbc/pull/556) may show **multiple** issue comments with the **same** parent SHA (seconds apart or later reposts) and duplicated `<!-- [codex-review] -->` lines — e.g. `7c53b24f…` @ 08:25:10Z / 08:25:11Z / **09:25:23Z**; `37c3d25f…` @ 08:27:11Z / **08:27:12Z**; **`da52ee9a…`** @ **09:30:30Z** / **09:30:31Z** — still **one** Codex verdict per SHA; do not double-count.

**`06b14483…` follow-ups** (first `status:complete` @ 2026-04-19T07:55:14Z; body-duplicate @ **09:25:06Z** — **APPROVE_WITH_COMMENTS**, non-blocking): derive `width` from `pattern` and drop redundant `width`; later replace `kind_name: String` with a structural ref to the `TokenKind` variant when value bodies allow; if SG grows past closed tables, decide whether comment / string-escape / identifier rules stay in this `.dag` or remain codegen by design. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e484c0-00b8-83ea-9dd5-1a3da56a67be).

**`37c3d25…` follow-ups** (first `status:complete` @ 2026-04-19T08:24:54Z; body-duplicate @ **09:25:07Z** — **APPROVE_WITH_COMMENTS**, non-blocking): dissolve `kind_name: String` when value bodies carry typed declaration references; dissolve prefix-based row collection when the surface can author an explicit structural table; optional tiny scaffold note on `KeywordSpec` / `PunctSpec` or **ROADMAP** row naming triggers. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e48bc5-4418-83ea-88e3-d3c57b27b7cb).

**`e529d136…` follow-ups** (complete @ 2026-04-19T09:06:59Z — **REQUEST_CHANGES**): resolve **single vs dual** syntax authority (`tokenize.dag` vs `extdeps.languages.dag.syntax`) per review body; `kind_name` / `width` / prefix scans remain tracked generator scaffolds once the split is honest. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e492ca-c498-83ea-853c-b181f6e0a3cf).

**`527d7fd3…` follow-ups** (complete @ 2026-04-19T09:25:01Z — **APPROVE_WITH_COMMENTS**, non-blocking): generator-time validation for escape codepoints and stricter `kind_name` resolution; typed `kind_name`, derive/drop `width`, explicit lexical authority vs `extdeps` syntax — tracked debt, not merge blockers per review. [PR #556](https://github.com/gunb-ai/gunbc/pull/556) / [thread](https://chatgpt.com/g/g-p-69e3c70def688191bf8fa7c2cb3292ba-gunbc-review/c/69e499d2-c71c-83ea-a3d6-6e8b00a52627).

**`da52ee9a…` Codex** (first @ 2026-04-19T09:30:30Z; duplicate marker @ **09:30:31Z** — **1 blocking**, same wall clock as **human inline** on `tokenize.dag`): resolve dual syntax authority (`syntax.dag` vs `tokenize.dag`) as in table row; escape validation in regen as hardening. Full text on [PR #556](https://github.com/gunb-ai/gunbc/pull/556).

**Indexed automated rounds:** **ChatGPT** (per-diff) — **five** parent SHAs **complete** + **`1cbda4f9…` pending** (stub @ 2026-04-19T09:31:36Z — refresh [PR #556](https://github.com/gunb-ai/gunbc/pull/556) for `status:complete`). Verdicts **differ by parent SHA** (**`fd6ef9…`** / **`e529…`**: **REQUEST_CHANGES** on global syntax authority; **`527d7fd3…`** / several earlier rounds: **APPROVE_WITH_COMMENTS**; **meta** `3fc44f…`: **SHIP_WITH_DEBT**) — reconcile on [PR #556](https://github.com/gunb-ai/gunbc/pull/556). **Human inline** @ `tokenize.dag:~L6` (**BLOCKING** on dual authority vs `syntax.dag`) aligns with **`fd6ef9…` / `e529…` REQUEST_CHANGES** and Codex **`da52ee9a…` (1 blocking)** more than with **meta SHIP_WITH_DEBT** — resolve on GitHub. **Codex** — **three** parent SHAs: **`7c53b24f…`** / **`37c3d25f…`** (**0** blocking each), **`da52ee9a…`** (**1** blocking). A “+1 queued” count may be this **pending ChatGPT** round, **Checks**, or **remaining inline**; “+2” adds duplicate bot posts — **dedupe** (**ChatGPT** reposts @ ~09:25Z; **Codex** e.g. **`da52ee9a…`** @ 09:30:30Z / **09:30:31Z**; **`7c53b24f…`** @ **09:25:23Z**) first. **Dedupe by parent SHA and marker kind** (`chatgpt-review` vs `chatgpt-meta-review` vs `codex-review`; same SHA may appear under **both** ChatGPT and Codex).

**Superseded bot stubs:** For a given parent SHA, ChatGPT often posts `status:pending` first, then `status:complete` for the **same** SHA (e.g. `06b14483…`: stub @ 2026-04-19T07:31:40Z → complete @ 07:55:14Z; `37c3d25…` (per-diff): stub @ 08:01:39Z → complete @ 08:24:54Z; **`e529d136…`**: stub @ 08:31:37Z → complete @ **09:06:59Z**; **`527d7fd3…`**: stub @ 09:01:37Z → complete @ **09:25:01Z**; **`chatgpt-meta-review` `3fc44f…`**: stub @ 08:26:49Z → complete @ **08:31:06Z** on [PR #556](https://github.com/gunb-ai/gunbc/pull/556)). The table rows are the **final** outcome per SHA, not the queue at a single timestamp.

For each PR: use **Conversation** (reviews, bots, threads) and **Checks** (CI).

Stage 1d design artifacts carried on #540: [`emit-functions-inventory.md`](./emit-functions-inventory.md), [`spec-field-gaps.md`](./spec-field-gaps.md), [`emit-bridges.md`](./emit-bridges.md).

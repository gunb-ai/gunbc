### dsl/extdeps/

**git.dag** — 8.5/10
- M4: `GitRemote.fetch_refspec` as String — could encode grammar
- Good: faithful object model from git-scm documentation

**cargo.dag** — 7/10
- M4: `CargoFeature.dependencies` as `List<String>` — should reference features
- Missing: structured error types for build/test failures

**github/github.dag** — 7.5/10
- M4: `Scopes` as `List<String>` — should reference `GitHubScope` enum
- Should import Git types where GitHub concepts reference Git (e.g., branches, commits)

**github/gists.dag** — 8/10
- M1: `files` is `List<GistFile>` but GitHub API returns `Map<filename, GistFile>`
- Good: comprehensive mock responses

**github/auth.dag** — 4/10
- Very minimal, magic string `"github-token"`, no composition

**cloud/gcp/gcp.dag** — 8/10
- Hardcoded regions data will go stale (GCP adds regions)
- Good: dual identity, precise service account model, real scope URIs

**llm/anthropic.dag** — 9/10
- M4: ~~`ThinkingConfig.type` as String~~ — DONE: `ThinkingMode = Enabled | Disabled`
- M4: `CacheControl.type` as String — should be enum (same pattern)
- Good: ContentBlock tagged union, cache_control, precise token budgets

**llm/openai.dag** — 8/10
- Nested destructuring via string paths (`"content/0/text"`) is fragile
- Good: ResponseFormat tagged union, ToolChoice tagged union

**llm/llm.dag** — 9/10
- `Role`, `StopReason`, `TokenUsage` are shared concepts documented by both providers — valid
- M1: ~~`LlmMessage.content` as String~~ — DONE: `List<ContentBlock>` with `TextContent | ImageContent`; provider-specific blocks in anthropic.dag/openai.dag


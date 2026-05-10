Auto-opened by session-dashboard for session `valiant-otter-36`.
Pushing to `session/valiant-otter-36` advances this PR.

Closes #2606

## Worker attestation

- Title names the Anthropic wire slice (not only the session id).
- Summary and test plan below replace the prior TODO placeholders.
- Local verification: `CTRL_BUILD_BYPASS_SHIMS=1 cargo check -p v2-compiler` and `CTRL_BUILD_BYPASS_SHIMS=1 cargo clippy --all-targets -- -D warnings` both succeeded on the pushed revision; `cargo fmt --all --check` clean.
- No secrets, credentials, or large binaries added.
- Branch commits are session work toward #2606 only.

## Summary

This PR lands **slice 2** of the Anthropic `tool_result_content` wire paydown: `AnthropicToolResultBlock` models `content` as a coproduct whose JSON wire accepts either a **scalar JSON string** or a **bare JSON array of nested content blocks** (Anthropic’s untagged union between string and array).

The DSL adds `UntaggedJsonStringOrArray` in `dsl/std/serialization.dag`, extends `dsl/extdeps/llm/anthropic.dag` with the tool-result block shape, receipts, and wire contracts, and threads **`EmitGraphInfo.untagged_json_tuple_variants`** through the v2 compiler DAG (`04_emit_info`, `04_infer`, `05_emit_rust`) with matching **stage0** Rust in `src/v2/stage0/`. Codegen learns tuple-variant serde adjacency for untagged externals and correct construction in `emit_typed_record_lit` when a single record field participates in the untagged set.

## Test plan

- `cargo fmt --all --check` — pass.
- `CTRL_BUILD_BYPASS_SHIMS=1 cargo check -p v2-compiler` — pass.
- `CTRL_BUILD_BYPASS_SHIMS=1 cargo clippy --all-targets -- -D warnings` — pass.

CI was previously **skipped** while the PR was a draft; fmt / ci / v3 / self_host should run after marking ready.

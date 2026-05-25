# V4 Active Bridge Dissolution Items

**Status:** Documentation-only bridge record from the 2026-05-25 bridge
audit. No code changes are implied by this brief.

**Scope:** These are active v4 bridges where an ahead task has already
introduced a structural direction but a behind task or consumer path still
preserves an older authority. Each row names the bridge, the owner lane(s),
and the concrete dissolution condition.

## 1. Layout in Lex Literals

**Owner:** T-6 tokenizer carrier lane + T-11 per-target emit tables.

**Ahead:** T-11 has five targets whose `LiteralPattern.text` values still
carry layout as embedded source text. The observed target set is Rust, Java,
TypeScript, Swift, and WASM, with examples such as `fn `, ` + `, and ` { `.

**Behind:** T-6 does not yet provide a `TriviaPolicy` or `TokenLayout`
carrier that lets layout travel as data separate from token spelling.

**Dissolution condition:** T-6 adds the layout/trivia policy carrier; T-11
strips embedded whitespace from all five target lex literals; and
`token_sequence_to_source` interleaves token spellings with layout from the
carrier. Each existing `// layout whitespace is still baked...` comment is
the per-file dissolution trigger.

## 2. CI/YAML Parallel Authority

**Owner:** T-20 bootstrap workflow lane + T-24 CI workflow lane.

**Ahead:** `src/v4/workflow/ci.dag` and `src/v4/lens/affected_set.dag`
model CI selection and workflow behavior as substrate facts.

**Behind:** There is no generator from `ci.dag` to
`.github/workflows/ci.yml`, so the committed YAML remains a live authority.
The v3 string ratchets also remain as parallel verification authority.

**Dissolution condition:** A generator produces the checked YAML from
`ci.dag` facts; the hand-authored YAML is deleted; and the v3 string
ratchets are replaced by `TestClaim`s against the generated output.

## 3. `authority_source_text` Reachable in Emit/Translate

**Owner:** T-10 emit/translate lane, with T-11 as the per-target consumer
lane.

**Ahead:** T-11 has grammar-inverse token-sequence serialization work in
place as the intended emit path.

**Behind:** No structural gate prevents fallback reads from
`authority_source_text` or related `*_source_literal` fields on
`TargetModel`.

**Dissolution condition:** Delete `authority_source_text` from `TargetModel`
once no emit path reads it, or add a P3 gate in `05_emit.dag` /
`06_translate.dag` that blocks reads outside fixed-point contexts.

## 4. Resolver Lacks DeclaredBinding Substrate

**Owner:** T-8 normalize/resolve lane + std binding substrate lane.

**Ahead:** `03_resolve.dag` is functionally working, including the immediate
F1 fix from PR #3644.

**Behind:** `std/` has no `DeclaredBinding` carrier. The resolver still
infers declaration-ness from structural position instead of consuming a typed
fact.

**Dissolution condition:** A `DeclaredBinding` carrier lands in
`std/node.dag` or `std/binding.dag`; `03_normalize.dag` stamps it on
declaration nodes; and `03_resolve.dag` is rewritten to consume
`DeclaredBinding` exclusively.

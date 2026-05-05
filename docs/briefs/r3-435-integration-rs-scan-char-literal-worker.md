---
status: draft (wait-window; awaits R3 host restoration before dispatch)
authority parent: R3 Substrate Manager (#1739)
roadmap row: ROADMAP.md "`IntegrationRsScan` / `integration_rs_active_line_contains` … byte-constant workaround attractor" (P2 Boundary Discipline / T-Receipts)
---

# R3 :435 — `IntegrationRsScan` char-literal state + comment correction

## Context

`src/v3/compiler/tests/integration/common/mod.rs` carries an
`IntegrationRsScan` state machine with four states: `Code`,
`LineComment`, `BlockComment(depth)`, `String`. The scanner is
"deliberately narrow" and does not model Rust char literals. The doc
comment at `:320-326` warns that any test author touching the scan
path must avoid `b'\\'` / `b'"'` and substitute numeric byte
constants — and that hitting a raw / byte-string opener inside a
char literal will **panic** rather than silently false-green.

Historical evidence:
- #686's text-slicing helper introduced `DAG_ESCAPE_BYTE` /
  `DAG_QUOTE_BYTE` constants for exactly this reason.
- #705 deleted that helper; constants went with it.
- No surviving in-tree workaround instances today; the scanner
  constraint itself remains and is a workaround attractor for the
  next test author landing in this scan path.

Dissolution alternatives named by the row: widen the scanner to
model Rust char literals, **or** replace the scan with a structural
reader that does not need to exclude char-literal syntax.

## Slice (extension path)

This brief picks the **scanner-extension** path because it is the
smaller scope; if the worker's pre-flight reveals scanner widening
is harder than a small structural reader (e.g., `syn`-based parse
of `tests/integration.rs`), STOP and surface for re-scoping.

1. Add a fifth state `Char` to `IntegrationRsScan` in
   `src/v3/compiler/tests/integration/common/mod.rs:138-200`.
2. Wire the `Code` → `Char` transition on a `'` byte that is not
   immediately preceded by an identifier-continuation byte (lifetime
   syntax `'a` is not a char literal). Concretely: a `'` is char-
   literal opener iff the previous scanned byte is not in
   `[A-Za-z0-9_]`. (This same check is what `rustc`'s lexer uses to
   disambiguate; the byte-oriented scan can apply it directly.)
3. While in `Char`, consume bytes to the closing `'`, honoring `\`
   as an escape (skip the next byte). Numeric / hex / unicode escapes
   inside the `\…` form do not need byte-level decoding — only
   escape-skip is required for closing-quote correctness.
4. Update the `Code`-state raw / byte-string opener probe so it does
   not fire when the apparent opener is inside `Char`. (With state
   #5 in place, this is automatic — the probe lives in `Code` only.)
5. Replace the doc-comment block at `:320-326` ("`Not handled:` …
   char literals — none appear in today's `tests/integration.rs`
   module list. … extend `IntegrationRsScan` **or** the scan will
   **panic**") with: "Char literals are recognized via the `Char`
   state; raw / byte-string openers cannot fire inside char literals."
6. Add a unit test in the same file (or its sibling test module)
   that scans an `tests/integration.rs`-shaped fragment containing
   `b'\\'` and `b'"'` inside a `#[test]` body and asserts:
   (a) the scan does not panic;
   (b) `integration_rs_active_line_contains` returns `false` for
   needles that appear only inside char literals (and in
   `String` / comment) — code-only positivity is preserved.

## Acceptance

- Scanner extends to five states; char-literal recognition matches
  rustc's lifetime-vs-char disambiguation rule.
- Doc comment at `:320-326` reflects the post-extension behavior.
- Unit test for `b'\\'` / `b'"'` inputs lands and is green.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- ROADMAP row flips Open → Retired with PR sha; ledger row 71 in
  `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` flips Open →
  Retired with the dissolution shape ("scanner widened to char
  literals; workaround attractor removed").

## STOP-AND-ESCALATE

- If pre-flight reveals the lifetime-vs-char disambiguation needs
  more context than a single previous-byte lookback (e.g., the
  scanner sees a `'` after whitespace following an identifier),
  STOP and surface — the structural-reader alternative may be
  cheaper than a correct byte-level disambiguator.
- If extending the scanner triggers cascading test failures
  elsewhere that consume `integration_rs_active_line_contains` with
  expectations baked against the four-state behavior, STOP — that's
  a different scope (consumer migration) than the row names.

## Authority audit receipt

1. **Substrate exists?** N/A — Rust test-harness code, not `.dag`
   substrate.
2. **Existing brief?** None; row is owner-unassigned per ROADMAP.
3. **Design-doc match?** N/A.
4. **Citations live?** `mod.rs:138, 141, 320-326` verified via
   wait-window grep at HEAD (2026-05-05). State enum at `:138`,
   doc comment block at `:320-326`.
5. **Carrier dissolves the bridge?** Yes — adding `Char` state
   removes the "byte-constant workaround attractor" the row names;
   dissolution sentence in ROADMAP is "widen the scanner to model
   Rust char literals."

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction. Ratification pending host restoration and
parent dispatch slot allocation.

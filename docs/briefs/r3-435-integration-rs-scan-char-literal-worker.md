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

## Slice — two co-equal paths; worker picks at pre-flight

The worker picks **one** of the two paths below at pre-flight,
based on which is more honest given the live state of
`tests/integration.rs` and workspace dependency surface. Both paths
are spec-backed and explicitly named in the row's dissolution
sentence; neither is a STOP-fallback to the other.

### Path A — structural Rust reader (preferred when feasible)

Replace the byte-oriented scanner with a structural Rust source
reader that already encodes Rust's lexical grammar:

- **`rustc_lexer`** (rustc's own lexer crate; thin, no syntax-tree
  construction): tokenize `tests/integration.rs` and walk the
  token stream skipping `LineComment` / `BlockComment` /
  `Literal { kind: Str | RawStr | Byte | ByteStr | RawByteStr |
  Char | ByteChar }` tokens. `integration_rs_active_line_contains`
  becomes a token-stream filter rather than a hand-coded state
  machine. Lifetime-vs-char disambiguation is rustc-faithful by
  construction (the crate IS rustc's authority).
- **`syn::parse_file`** (heavier; full AST): parse the file and
  walk only `Item` / `ImplItem` text spans for the needle. AST
  walk is structurally exact; cost is the AST allocation per scan.

Worker pre-flight verifies which dependency is easier to add to
the workspace; if `rustc_lexer` is not already transitively
available and adding it requires a vendored fork, prefer `syn`
(more likely already in the workspace via proc-macro deps).

Path A acceptance: `IntegrationRsScan` deleted entirely;
`integration_rs_active_line_contains` reimplemented as a
token-stream / AST-span filter. Doc comment block at `:320-326`
deleted (no longer applicable). Workaround attractor (the
constraint itself) is gone, not just narrowed.

### Path B — scanner extension with rustc-lexer-faithful rule

If Path A is blocked (workspace dependency policy, rustc_lexer
not available, syn too heavyweight for a test harness), extend
the existing byte-oriented scanner. The disambiguation rule MUST
mirror `rustc_lexer::Cursor::single_quoted_string` semantics, NOT
a naive byte heuristic.

**Worker pre-flight obligation (Path B):** read
`rustc_lexer/src/lib.rs` `Cursor::lifetime_or_char` (the actual
disambiguator entry point — NOT `single_quoted_string`, which is
the body-consumer that runs after disambiguation) and reproduce
its disambiguation logic faithfully. Cite the upstream source
SHA in the PR body. Naive lookback heuristics (e.g., "previous
byte is not identifier-continuation") are explicitly rejected
per the BLOCKING finding on the prior brief revision — they
misclassify `&'a T` and `'label: loop` as char openers.

`lifetime_or_char` decides between lifetime and char-literal by
peeking the next two characters (after the opening `'`):
- if char1 is a non-identifier-start byte, OR
- if char1 is an identifier byte AND char2 is `'` (closing quote)
  with char1 being a single-character literal,
- OR if the body matches `\<escape>'` (escape pattern + closing
  quote within bounded window),
then it's a char literal; otherwise lifetime.

Path B's byte-oriented scan reproduces these branches exactly,
not a quote-search approximation.

Sketch of the lexer-faithful rule:

1. Add a fifth state `Char` to `IntegrationRsScan` in
   `src/v3/compiler/tests/integration/common/mod.rs:138-200`.
2. Wire the `Code` → `Char` transition via **branch-based
   disambiguation mirroring `Cursor::lifetime_or_char`**, NOT
   bounded-quote-search. A quote-search rule (peek N bytes for
   a closing `'`) silently misclassifies adjacent lifetimes:
   in `<'a, 'b>` the second `'b` quote sits inside any
   reasonable lookahead window from the first `'`, so a
   quote-search rule classifies `a, '` as a `Char` body and
   silently violates P3 fail-closed. The same trap fires for
   `for<'a, 'b>`, `&'a 'b ()`, and label-then-char shapes when
   the second token's `'` falls inside the window. The rule
   below avoids the trap by branching on the FIRST byte after
   `'`, never on a far-away closing quote.
   - **Sees `'` at position p in `Code`.** Peek `char1` = byte
     at p+1.
   - **Branch 1: `char1` is `\` (escape).** Consume the escape
     sequence per rustc rules (`\<single>` or `\x..` /
     `\u{...}`), then assert the following byte is `'`. If yes
     → enter `Char` and emit the closed range `[p, close]`;
     if no → malformed input (in `tests/integration.rs` this
     is a syntax error and the scan can panic per existing
     `IntegrationRsScan` discipline).
   - **Branch 2: `char1` is an identifier-start byte
     (`[A-Za-z_]`).** Peek `char2` = byte at p+2.
       - If `char2 == '` → char literal (`'a'` shape). Enter
         `Char` for the closed range `[p, p+2]`.
       - **Otherwise → lifetime / label.** Stay in `Code`,
         advance past `'` only. The identifier-continue bytes
         after `'` resume normal `Code` scanning. Critically,
         this branch fires regardless of what later bytes look
         like — `'a, 'b>` correctly stays in `Code` because
         `char2 = ','`, not `'`. The second `'b` is processed
         independently when the scanner reaches it.
   - **Branch 3: `char1` is any other byte (non-`\`,
     non-identifier-start) — e.g., space, digit, punctuation,
     non-ASCII.** Treat as a single-byte char-literal body:
     assert byte at p+2 is `'`; if yes → `Char` for `[p, p+2]`;
     if no → malformed (panic / surface).
   The three branches together reproduce `lifetime_or_char`'s
   decision exactly. **No window scan, no closing-quote search.**
   Lifetimes followed by other lifetimes within any distance
   stay in `Code` because Branch 2's `char2 != '` test fires
   immediately, before the second `'` is ever inspected.
3. Within the closed `Char` range determined by step 2, no
   further escape-skipping is needed — the range was already
   chosen with escapes consumed. The body is structurally
   bounded: at most 1 byte (Branch 2/3 simple-char) or the
   `\<escape>` length (Branch 1). Numeric / hex / unicode
   escapes inside the `\<escape>` form do not require byte-
   level decoding for the scanner — only the escape-skip in
   Branch 1 is required for closing-quote correctness.
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

Path A or Path B, whichever the worker selects at pre-flight,
must satisfy:

- **Path A:** `IntegrationRsScan` deleted;
  `integration_rs_active_line_contains` reimplemented as a
  token-stream / AST-span filter. Doc comment at `:320-326`
  removed (no longer applicable). PR body cites the upstream
  crate (`rustc_lexer` SHA or `syn` version) used.
- **Path B:** Scanner extends to five states; disambiguation rule
  cites `rustc_lexer::Cursor::single_quoted_string` (or live
  equivalent) at the worker's pinned upstream SHA in the PR
  body. No false-`Char` classification on lifetime / label
  inputs. Doc comment at `:320-326` reflects the post-extension
  behavior.
- **Both paths** — unit tests land covering: (a) `b'\\'` / `b'"'`
  char-literal bodies (the original workaround-attractor pattern);
  (b) `&'a T` lifetime in reference position stays code (Path A:
  not classified as literal-skip; Path B: stays in `Code`);
  (c) `'label: loop { ... }` label form stays code (same);
  (d) **adjacent lifetimes** `for<'a, 'b>` and `Fn() -> &'a 'b ()`
  stay code — multiple consecutive `'<ident>` tokens never enter
  `Char`; (e) **adjacent label-then-char** `'l: loop { let c = 'x';
  break 'l; }` correctly disambiguates the inner `'x'` as `Char`
  while the outer `'l` lifetimes stay `Code`. Tests (a)-(e) are
  the BLOCKING-finding regression cases — not optional. Path A
  inherits rustc's disambiguation by construction; Path B's
  byte-oriented scan must pass tests (a)-(e) explicitly.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- ROADMAP row flips Open → Retired with PR sha; ledger row 71 in
  `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` flips Open →
  Retired with the dissolution shape ("scanner widened to char
  literals; workaround attractor removed").

## STOP-AND-ESCALATE

- If a real-world `tests/integration.rs`-shaped fragment in the
  worker's pre-flight contains a char-literal body legitimately
  longer than the 16-byte lookahead window, STOP and surface —
  either widen the window with explicit upper-bound justification
  or escalate to the structural-reader alternative the row's
  dissolution sentence names.
- If a closing `'` lookahead can be ambiguous against an unrelated
  later `'` in pathological code (e.g., a lifetime followed soon
  after by a char literal on the same line), STOP and verify
  against an actual fragment from the integration scan path. The
  bounded-lookahead rule is fail-closed for the lifetime case (no
  closing `'` ⇒ stay `Code`), but a worker who finds an
  adversarial counter-example must surface rather than band-aid.
- Add a unit test that exercises both `&'a T` (lifetime in
  reference type) and `'label: loop { break 'label; }` (label
  scope) and asserts both stay in `Code` (no false-`Char`); these
  are the BLOCKING-finding regression tests, not optional.
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

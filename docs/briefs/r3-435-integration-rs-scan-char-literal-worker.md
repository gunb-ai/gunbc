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

**Structural binding fact preserved.** The current
`integration_rs_active_line_contains` is a generic substring
helper, but its load-bearing consumer is
`integration_rs_cementing_path_attr_binds_mod_stem` at
`src/v3/compiler/tests/integration/common/mod.rs:301` — the
structural fact being verified is "`#[path = "integration/cementing/<stem>.rs"]`
attribute is immediately followed by `mod <stem>;` declaration."
Path A's typed extractor MUST preserve this binding fact, not
merely report code-token visibility. Concretely: the
replacement must produce typed `(path_attr_literal, mod_ident)`
pairs (or equivalent typed binding-fact representation) so
`integration_rs_cementing_path_attr_binds_mod_stem` can be
re-implemented as a typed predicate over the binding pairs,
not a substring search through filtered tokens. Replace the
byte-oriented scanner with a structural Rust source reader
that operates at the **token or typed-AST level** — never at
the source-text-span level (which is what `IntegrationRsScan`
does today and what the row's dissolution sentence names as
the attractor to remove). Two sub-options, both structural
and both producing the typed binding pairs:

- **`rustc_lexer` (token-level structural extractor):**
  tokenize `tests/integration.rs` once; the resulting token
  stream has per-token `TokenKind` discriminators. Walk the
  stream with a small state machine that recognizes the
  `#[path = "<lit>"]` ... `mod <ident> ;` pattern in
  Code-set tokens (`Pound` / `OpenBracket` / `Ident("path")` /
  `Eq` / `Literal { kind: Str }` / `CloseBracket` / `Ident("mod")` /
  `Ident(<stem>)` / `Semi`), skipping `LineComment` /
  `BlockComment` / non-string `Literal` tokens. The state
  machine emits the same typed binding pairs `(path_attr_literal,
  mod_ident)` as the `syn` path. `integration_rs_cementing_path_attr_binds_mod_stem`
  is reimplemented as a typed predicate over the binding-pair
  list. Lifetime-vs-char disambiguation is rustc-faithful by
  construction (the crate IS rustc's authority).

- **`syn` typed-AST visitor:** parse the file once via
  `syn::parse_file`; walk the AST with a `syn::visit::Visit`
  implementation. For each `syn::ItemMod`, the visitor
  inspects the preceding `#[path = "..."]` attribute (if any)
  and emits a typed binding pair `(path_attr_literal: LitStr,
  mod_ident: Ident)` capturing the structural fact
  "`#[path = LIT]` binds to `mod IDENT`". `integration_rs_cementing_path_attr_binds_mod_stem`
  reduces to a typed predicate "is there a binding pair whose
  `path_attr_literal` matches the expected
  `integration/cementing/<stem>.rs` shape AND whose `mod_ident`
  matches `<stem>`?" The visitor MUST NOT call
  `.span().source_text()` or any equivalent source-span-fetch
  API; if it does, the implementation has slipped back into
  the very pattern this slice retires. **Acceptance for the
  syn path includes (a) a grep that no source-span-fetch is
  called in the visitor, AND (b) a regression test asserting
  `integration_rs_cementing_path_attr_binds_mod_stem` returns
  `true` for the live binding pairs in `tests/integration.rs`
  and `false` when either the attribute or the mod decl is
  commented out / mismatched.**

Worker pre-flight verifies which dependency is easier to add to
the workspace; if `rustc_lexer` is not already transitively
available and adding it requires a vendored fork, prefer `syn`
(more likely already in the workspace via proc-macro deps).

Path A acceptance: `IntegrationRsScan` deleted entirely;
`integration_rs_active_line_contains` reimplemented as a
typed-token or typed-AST filter (NOT a source-span scan). Doc
comment block at `:320-326` deleted (no longer applicable).
Workaround attractor (source-span scanning over Rust syntax) is
gone, not just narrowed.

### Path B — scanner extension with rustc-lexer-faithful rule

If Path A is blocked (workspace dependency policy, rustc_lexer
not available, syn too heavyweight for a test harness), extend
the existing byte-oriented scanner. The disambiguation rule MUST
mirror `rustc_lexer::Cursor::single_quoted_string` semantics, NOT
a naive byte heuristic.

**Worker pre-flight obligations (Path B):**

1. **Read `rustc_lexer/src/lib.rs` `Cursor::lifetime_or_char`**
   (the actual disambiguator entry point — NOT
   `single_quoted_string`, which is the body-consumer that
   runs after disambiguation) and reproduce its disambiguation
   logic faithfully. Cite the upstream source SHA in the PR
   body. Naive lookback heuristics (e.g., "previous byte is
   not identifier-continuation") are explicitly rejected per
   the BLOCKING finding on the prior brief revision — they
   misclassify `&'a T` and `'label: loop` as char openers.

2. **Non-ASCII pre-flight scan.** Before Path B can be
   selected, scan `tests/integration.rs` and every file in
   the integration scan path for non-ASCII bytes (high bit
   set). If any non-ASCII byte appears in a position that
   could be inside an identifier, lifetime, or char-literal
   body — i.e., outside `String` / comment regions — Path B's
   byte-oriented branch logic is **out of domain** and Path A
   is mandatory. Path B's byte-keyed disambiguation is not
   UTF-8-aware; rustc allows Unicode-identifier lifetimes
   (`'αβ`) and Unicode-codepoint char literals (`'α'`), and
   silent byte-level handling reintroduces the P3 fail-closed
   hazard. Pre-flight must record the result in the PR body
   ("non-ASCII pre-flight scan: <count> non-ASCII bytes
   found in scan path; Path B selected / Path A required").

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
   - **Branch 3: `char1` is any other ASCII byte (non-`\`,
     non-identifier-start) — e.g., space, digit, punctuation.**
     Treat as a single-byte char-literal body: assert byte at
     p+2 is `'`; if yes → `Char` for `[p, p+2]`; if no →
     malformed (panic / surface).
   - **Branch 4: `char1` is non-ASCII (high bit set, i.e.,
     start of a UTF-8 multi-byte codepoint).** Path B's byte-
     oriented scan is **NOT lexer-faithful for Unicode**:
     rustc allows Unicode-identifier lifetimes (`'αβ`) AND
     Unicode-codepoint char literals (`'α'`), and Branch 3's
     byte-counting consumed-bytes-then-look-for-`'` rule is
     wrong for both — it treats the first UTF-8 byte as a
     single-byte body, then checks the second UTF-8 byte
     (which is part of the same codepoint) for a closing `'`
     and fails. Path B must NOT silently take this branch;
     instead, it MUST surface the non-ASCII byte as an
     **out-of-domain signal** and either (a) escalate to
     Path A for the file, or (b) the worker decodes the UTF-8
     codepoint and dispatches into Branch 1/2/3 keyed on
     codepoint properties (Unicode-identifier-start, etc.).
     Option (b) is a substantial extension to byte-oriented
     scanning and turns Path B into a partial Unicode lexer;
     prefer (a) when the integration scan path actually
     contains non-ASCII identifiers (which is uncommon in
     `tests/integration.rs` today, but **Path B must verify
     this at pre-flight rather than assume it**).
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
  cites `rustc_lexer::Cursor::lifetime_or_char` (the actual
  disambiguator entry point) at the worker's pinned upstream SHA
  in the PR body, and reproduces its three-branch logic exactly
  (escape branch / identifier-start `char2 == '` branch /
  non-identifier-start single-byte branch). No false-`Char`
  classification on lifetime / label inputs. No window-scan / no
  closing-quote-search heuristics. Doc comment at `:320-326`
  reflects the post-extension behavior.
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
  the BLOCKING-finding regression cases — not optional.
- **Unicode regression cases** — (f) Unicode char literal `'α'`
  (4 bytes: `'` + 2-byte UTF-8 `α` + `'`) recognized as `Char`;
  Path A inherits this from rustc by construction; Path B's
  pre-flight obligation (2) MUST escalate to Path A on detection
  of the non-ASCII byte. (g) Unicode lifetime `'αβ` (mid-fragment,
  e.g., `&'αβ T` where `αβ` is a Unicode XID-continue identifier)
  stays in `Code`; same — Path A handles it, Path B must escalate.
  (h) **Path B pre-flight escalation receipt** — the pre-flight
  scan correctly reports the non-ASCII count and selects Path A
  for any fragment containing tests (f)/(g) shapes. Test (h) is
  the lockdown for the BLOCKING finding on Branch 4 / non-ASCII
  out-of-domain.

  Path A inherits rustc's disambiguation by construction
  (including Unicode); Path B's byte-oriented scan must pass
  tests (a)-(e) explicitly **on ASCII-only fragments**, and must
  pass (f)/(g)/(h) by escalating to Path A — Path B never directly
  handles Unicode in this slice.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- ROADMAP row flips Open → Retired with PR sha; ledger row 71 in
  `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` flips Open →
  Retired with the dissolution shape ("scanner widened to char
  literals; workaround attractor removed").

## STOP-AND-ESCALATE

- **Path B's `lifetime_or_char` reproduction diverges from rustc.**
  If the worker's branch-by-branch reproduction does not match
  `rustc_lexer::Cursor::lifetime_or_char`'s observable behavior on
  the test corpus (a)-(e), STOP — Path B is not lexer-faithful and
  must be either corrected or escalated to Path A. No
  window-scan / closing-quote-search fallback is authorized.
- **Non-ASCII byte appears in scan path.** If pre-flight obligation
  (2) finds any non-ASCII byte outside `String`/comment regions
  in the integration scan path, Path B is **out of domain** —
  byte-oriented scan cannot honestly dispatch on Unicode codepoints
  (multi-byte UTF-8 sequences would be miscounted as single-byte
  char-literal bodies, reintroducing the P3 fail-closed hazard).
  Path A is mandatory in that case; do not silently extend Branch 3
  to non-ASCII bytes.
- **A real `tests/integration.rs` fragment exhibits a char-literal
  shape outside `lifetime_or_char`'s decision domain** (e.g., a
  Rust feature-gate addition that creates a new lexical category
  rustc itself doesn't yet ratify in stable). STOP and surface —
  Path A's structural-reader path inherits any rustc lexer update
  by construction, while Path B requires manual chase. A divergent
  fragment is a Path-A-or-escalate signal, not a band-aid.
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

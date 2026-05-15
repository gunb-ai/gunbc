---
status: PM-authored worker brief (deep-wolf-155 direct dispatch)
authority_parent: Operator briansrls 2026-05-15 directive — "do path B for tokenize/parse NO workarounds, spawn workers under you directly"
authoring_date: 2026-05-15
brief_set: docs/r3-path-b-tokenize-parse-brief-set.md (§Brief 3)
worker_session: bright-swift-668
reporting: directly to deep-wolf-155 via dashboard-message (no Director/Mgr intermediary)
---

# Path B Brief 3 — Substrate-Language: Char-Class Structural Completion

## Context

Part of the Path B tokenize/parse retirement program — see `docs/r3-path-b-tokenize-parse-brief-set.md` for the full 9-brief set. Operator chose NO WORKAROUNDS.

**Workers under deep-wolf-155 directly**: report findings + blockers via `dashboard-message send --to deep-wolf-155 --body "..."`. Do NOT route through Director/Mgr layer.

## The gap

`src/v3/compiler/tokenize.dag` lines 23-58 carry an explicit named scaffold:

```
// **Tracked scaffold (character-level under-consumption — consumption gap,
// not substrate gap).** The scan phases below slice the ASCII/Unicode
// codepoint space in two parallel forms, neither of which consumes the
// character-level authorities that already exist in `dsl/std/`:
//
//   1. Reserved individual codepoints as opaque strings. ...
//   2. Reserved codepoint classes as hidden Rust predicates.
//      `regen_tokenize.rs` push_str's `is_ascii_whitespace` / `is_ascii_digit` /
//      `is_ascii_alphabetic` / `is_ascii_alphanumeric` plus a bare
//      `byte == b'_'` into the generated tokenizer. Not mentioned in `.dag` at all.
//
// **Existing authorities to consume** (already imported cross-tree today):
//   - `std.types::Char = Int` (Unicode scalar, U+0000–U+10FFFF).
//   - `std.string_type::String = FreeMonoid<Char>`.
//   - `std.unicode` (`DisplayWidth`, `UnicodeBlock`, block/width classification).
//   - `std.encoding` ...
//   - `std.bit::Byte`.
//
// **CharClass — tokenizer half (bounded interim, not full lane closure).**
// `std.unicode` declares `CharClass` + `char_in_class` (canonical `.dag`
// authority). `ascii_scan_order` makes tokenizer scanner precedence a
// structural `List<CharClass>` consumed directly by `regen_tokenize`; ASCII
// predicate bodies remain a bounded generator bridge until `char_in_class`
// semantics are structurally consumed.
```

Translation: `std.unicode::CharClass` + `char_in_class` exist as `.dag` substrate authority, BUT `char_in_class(c, IdentStart)` is NOT structurally executable yet — `regen_tokenize.rs` hardcodes `byte.is_ascii_whitespace()` / `byte.is_ascii_digit()` etc. as Rust predicate bridges.

## Why this matters for Path B

Briefs 7-9 (driver authoring) need `.dag`-native character-class testing. The tokenizer's behavior is "scan precedence is `[Whitespace, Digit, IdentStart, IdentContinue]`; for each character, test which class it's in." Without structural `char_in_class`, the `.dag` driver would have to emit the same hardcoded predicate bridges to Rust that regen_tokenize.rs does today — which means substrate isn't actually driving emission for this piece.

## Scope of this brief

### Phase A — Investigate current state

Grep + read:

```bash
grep -rn "type CharClass\|fn char_in_class\|fn is_ascii_" dsl/std/unicode.dag src/v3/std/ 2>/dev/null
grep -rn "is_ascii_whitespace\|is_ascii_digit\|is_ascii_alphabetic" src/v3/compiler/src/regen_tokenize.rs
```

Specifically:
1. Read `dsl/std/unicode.dag` (if exists) or equivalent to find the `CharClass` enum declaration + `char_in_class` function.
2. Understand WHY `char_in_class` isn't structurally executable today — is it missing a body? Lowering issue? Inference issue? Missing runtime support for Char comparison?
3. Audit which Char-class checks `regen_tokenize.rs` actually needs (the scaffold names: `is_ascii_whitespace`, `is_ascii_digit`, `is_ascii_alphabetic`, `is_ascii_alphanumeric`, `byte == b'_'`).

Surface findings to deep-wolf-155 via dashboard-message.

### Phase B — Land structural `char_in_class`

Once investigation identifies what's missing:
1. Land the missing piece(s) so `.dag` code can call `char_in_class(c, IdentStart)` at runtime and get a Bool back.
2. Verify each ASCII class (Whitespace, Digit, IdentStart, IdentContinue) is correctly implemented per the existing predicate semantics.
3. Add tests that exercise each class.

### Phase C — Refactor `tokenize.dag` to consume `char_in_class` structurally

Once `char_in_class` is structurally executable:
1. Verify `data ascii_scan_order: List<CharClass> = [Whitespace, Digit, IdentStart, IdentContinue]` at `tokenize.dag:103` is correctly consumed by `regen_tokenize`.
2. Update `regen_tokenize.rs` (or the future `.dag` codegen driver) to emit `char_in_class(c, X)` calls instead of `byte.is_ascii_X()`.
3. Verify `tokenize_generated.rs` is byte-identical (if the ASCII predicate bodies were previously inlined, they may now be calls to `char_in_class`).
4. Delete the tracked-scaffold comment at `tokenize.dag:23-58` — it's no longer accurate.

## Deliverables (concrete)

1. **Investigation report** to deep-wolf-155: current state of `std.unicode::CharClass` + `char_in_class` + the specific gap blocking structural execution.
2. **Substrate completion PR**: lands the missing piece(s) for `char_in_class`.
3. **Tokenize refactor PR**: removes the hand-Rust predicate bridge; `tokenize.dag` consumes `char_in_class` structurally.
4. **Scaffold comment removal**: `tokenize.dag:23-58` deleted in the same PR as the refactor.

## Acceptance criteria (substrate-fact-at-HEAD)

- `cargo test -p v3-compiler --test integration char_class_structural_test` passes (test name TBD).
- `grep -n "is_ascii_whitespace\|is_ascii_digit\|is_ascii_alphabetic\|is_ascii_alphanumeric" src/v3/compiler/src/tokenize_generated.rs` returns 0 hardcoded predicate calls (or, only what `char_in_class` emits as its implementation).
- `grep -n "Tracked scaffold (character-level under-consumption" src/v3/compiler/tokenize.dag` returns 0 matches.

## Anti-paper-shrink check

Naive workarounds that DO NOT count:
- Adding `char_in_class` as an FFI bridge to Rust `is_ascii_X` functions (no substrate-language growth — the predicate bodies are still hand-Rust)
- Moving the ASCII predicate names into a `data` row in `tokenize.dag` but having `regen_tokenize.rs` emit them as Rust predicate calls anyway (still hand-Rust bridge)

The discriminator: `char_in_class(c, X)` must be a STRUCTURAL `.dag` function whose body is `.dag` substrate (e.g., a match over `CharClass` variants with `.dag`-expressible predicates for each variant). The Rust generated code should either call `char_in_class` directly or be derivable mechanically from it; the predicate semantics live in `.dag`.

## Risks + open questions to surface back

- **What is `IdentStart` semantically?** Is it ASCII-only or full Unicode (UAX #31 identifier start)? If full Unicode, the implementation requires consuming `std.unicode::UnicodeBlock` substrate which may itself be NYI.
- **Char comparison semantics**: `.dag` has `Char = Int` (per `dsl/std/types.dag`). Does `.dag` support `Int < Int` / `Int <= Int` comparisons at runtime? If not, that's a sub-blocker.
- **Whitespace ambiguity**: `is_ascii_whitespace` = `' '` / `'\t'` / `'\n'` / `'\r'` / `'\x0C'`. Make sure the `.dag` implementation matches exactly (regen produces a tokenizer; bug-compatibility with the existing hand-Rust matters for byte-identical regen).
- **Performance**: hand-Rust uses byte-level predicates that the compiler optimizes. `.dag`-driven `char_in_class` may be slower if the runtime walks variants per call. For tokenize this probably doesn't matter (one-shot at compile time), but flag if significant.

## Coordination

- **Report findings** to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` after Phase A.
- **Tag PRs** with title prefix `r3-path-b-brief-3: ...`.
- **Coordinate with sibling Briefs 1, 2** (workers `witty-moth-725` and `sunny-tern-495`) — sibling work may interact (e.g., if Brief 1's non-endomorphic map lands, a `List<Char>.map<Bool>(c -> char_in_class(c, X))` walk becomes expressible).

## Estimated effort

1-2 months. The scaffold is named + scoped; completion should be tractable.

## Read first

- `src/v3/compiler/tokenize.dag` lines 23-103 (the tracked scaffold + `ascii_scan_order` declaration)
- `dsl/std/unicode.dag` (CharClass + char_in_class authority — confirm path; may be at a different location)
- `dsl/std/types.dag` (Char = Int with unicode_scalar predicate)
- `src/v3/compiler/src/regen_tokenize.rs` (the hand-Rust predicate bridges — what we're retiring)
- `docs/r3-path-b-tokenize-parse-brief-set.md` (full brief set context)
- `feedback_corrections_must_grep_verify_source` — verify substrate state before claiming gaps

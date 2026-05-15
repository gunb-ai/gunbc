---
status: PM-authored worker brief (deep-wolf-155 direct dispatch)
authority_parent: Operator briansrls 2026-05-15 directive — "do path B for tokenize/parse NO workarounds, spawn workers under you directly"
authoring_date: 2026-05-15
brief_set: docs/r3-path-b-tokenize-parse-brief-set.md (§Brief 2)
worker_session: sunny-tern-495
reporting: directly to deep-wolf-155 via dashboard-message (no Director/Mgr intermediary)
---

# Path B Brief 2 — Substrate-Language: `format(template, args)` + Conversion Primitives

## Context

Part of the Path B tokenize/parse retirement program — see `docs/r3-path-b-tokenize-parse-brief-set.md` for the full 9-brief set. Operator chose the NO WORKAROUNDS path.

**Workers under deep-wolf-155 directly**: report findings + blockers via `dashboard-message send --to deep-wolf-155 --body "..."`. Do NOT route through Director/Mgr layer.

## The gap

`src/v3/compiler/src/regen_tokenize.rs` has **220 string-building call sites** (`push_str`, `format!`, `writeln!`). Many use templated strings:

```rust
format!("TokenKind::{label}")
format!("|| byte == b'{}',", c)
format!("    pub kind: {kind},\n", kind = ...)
```

`.dag` has `String = FreeMonoid<Char>` (`dsl/std/string_type.dag:17`) with `concat`, `append`, `length`, `map`, `fold`, etc. — string-building IS expressible. What's missing:

1. **`format(template, args)`-style templating**: no equivalent of Rust's `format!("Hello, {name}!")` in `.dag`. Workaround = explicit concat, but for 220 sites that's painful and obscures intent.
2. **Conversion primitives**: `int_to_string(i: Int) -> String`, `char_to_string(c: Char) -> String`, `bool_to_string(b: Bool) -> String` — need to verify which exist + author missing ones.

## Why this matters for Path B

Briefs 7-9 (driver authoring) cannot land cleanly without string-building primitives. Workers authoring `.dag` codegen drivers would otherwise spend most of their effort on `concat([str("foo"), int_to_string(n), str("bar")])`-style verbosity for every string. Brief 2 unblocks Briefs 7-9.

## Scope of this brief

### Phase A — Inventory existing primitives

Grep current state:

```
grep -rn "fn int_to_string\|Int.to_string\|fn char_to_string\|Char.to_string\|fn bool_to_string" dsl/std/ src/v3/std/ 2>/dev/null
grep -rn "fn format\b" dsl/std/ src/v3/std/ 2>/dev/null
grep -rn "String.from\|str_from" dsl/std/ src/v3/std/ 2>/dev/null
```

Surface the existing-primitives inventory to deep-wolf-155 via dashboard-message. This determines the scope of new work.

### Phase B — Land missing primitives

For each missing primitive (likely `int_to_string`, `char_to_string`, `bool_to_string`, possibly `string_to_int` etc.):
1. Author `fn` declaration in appropriate `dsl/std/` or `src/v3/std/` module.
2. Land lower / infer / emit support if needed (test compilation).
3. Add tests.

### Phase C — Land `format(template, args)`

Two possible shapes (worker investigates which is better):

**Shape α — Runtime function**:
```dag
fn format(template: String, args: List<String>) -> String {
  // Walks template looking for `{N}` placeholders, substitutes args[N]
}
```
- Pros: simpler implementation
- Cons: no compile-time check that args list length matches placeholder count

**Shape β — Substrate-language string-interpolation**:
```dag
let msg = "Hello, ${name}! count=${int_to_string(n)}"
```
- Pros: compile-time type-checked + integrated into the language
- Cons: substantial parser / lower / infer work for a new syntax form

Surface architectural choice to deep-wolf-155 BEFORE landing.

### Phase D — Test fixture

```dag
data sample_name: String = "world"
data sample_count: Int = 42
data sample_msg: String = format("Hello, {0}! count={1}", [sample_name, int_to_string(sample_count)])
// or, if Shape β: data sample_msg: String = "Hello, ${sample_name}! count=${int_to_string(sample_count)}"
```

Test asserts `sample_msg == "Hello, world! count=42"`.

## Deliverables (concrete)

1. **Inventory report** to deep-wolf-155 via dashboard-message: list of existing primitives + list of missing ones.
2. **Architectural decision** ratified by deep-wolf-155: Shape α (runtime fn) vs Shape β (substrate-language interpolation).
3. **Conversion primitives PR**: land `int_to_string` / `char_to_string` / `bool_to_string` / etc. as needed.
4. **Format PR**: land the chosen `format` shape with tests.

## Acceptance criteria (substrate-fact-at-HEAD)

- `cargo test -p v3-compiler --test integration string_templating_test` passes (test name TBD).
- `.dag` fixture: `let msg = format("hello, {0}! count={1}", [name, int_to_string(n)])` produces expected string (or equivalent Shape β syntax).
- Cross-check: an existing `.dag` file (e.g., `tokenize.dag`) that currently relies on hand-Rust string-emission could be rewritten to use the new substrate primitives (don't necessarily REWRITE it in this brief — just verify the substrate supports it).

## Anti-paper-shrink check

Naive workarounds that DO NOT count:
- Adding `fn format` that's a thin wrapper around existing Rust `format!` via FFI (no substrate-language growth)
- Authoring conversion primitives as Rust-only with `.dag` reference but no substrate-language realization (parallel-authority)
- Re-exposing existing Rust primitives via `.dag` declarations without runtime backing

The discriminator: the new primitives must work end-to-end at lower-and-infer time in a `.dag` program with no `use crate::...` Rust escape hatches.

## Risks + open questions to surface back

- **Argument-index syntax**: `{0}` / `{1}` vs `{name}` vs `{}`? (Different ergonomics + different parser complexity.) Surface to deep-wolf-155.
- **Type-safety in format**: should `format("{0}", [non_string])` error at lower/infer time or at runtime? Stronger check is preferred per `feedback_state_space_vs_behavioral_invariants` but requires more parser/infer work.
- **Composition with FreeMonoid**: should `format` interact with FreeMonoid<Char>'s concat-based string operations? Or be a separate utility?
- **Locale/Unicode**: out of scope for this brief, but flag any places where conversion primitives might need locale parameters (recommendation: punt to a follow-up).

If `int_to_string` already exists in some form (e.g., via `dsl/std/int.dag` or similar), surface that — may simplify scope substantially.

## Coordination

- **Report inventory** to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` after Phase A.
- **Pause for ratification** of Shape α vs Shape β before Phase C.
- **Tag PRs** with title prefix `r3-path-b-brief-2: ...`.
- **Coordinate with sibling Briefs 1, 3** (workers `witty-moth-725` and `bright-swift-668`) — sibling work may interact (e.g., Brief 1's non-endomorphic map enables `List<Int>.map<String>(int_to_string)` cleanly).

## Estimated effort

1-3 months. Likely much less if Phase A reveals most primitives already exist + format is a small addition.

## Read first

- `dsl/std/string_type.dag` (String = FreeMonoid<Char>)
- `dsl/std/algebra.dag` lines 380-420 (FreeMonoid<T> declaration)
- `dsl/std/int.dag` (Int substrate — check for to_string)
- `dsl/std/types.dag` (Char = Int with unicode_scalar predicate)
- `docs/r3-path-b-tokenize-parse-brief-set.md` (full brief set context)
- `feedback_corrections_must_grep_verify_source` — verify substrate state before claiming primitives missing

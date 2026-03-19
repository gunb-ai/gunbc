# Performance Audit: End-to-End Pipeline Trace

**Date:** 2026-03-18
**Trigger:** Generated v2 crate OOMs on 1,515 lines of gist sources even in
release mode (SIGKILL after 16 minutes, 1061s CPU).

**Purpose:** Trace the complete pipeline from `.dag` source through v1 codegen
to generated Rust, identify every performance cliff, and determine whether
prior optimizations addressed the right layer.

---

## Status (2026-03-18)

| Smoking Gun | Description | Status | Fix |
|-------------|-------------|--------|-----|
| SG-1 | `char_at` O(pos) per call -> O(n^2) tokenization | **FIXED** | R6: O(1) byte-offset indexing for ASCII |
| SG-2 | `string_length` O(n) per call | **FIXED** | R6: O(1) byte-length for ASCII |
| SG-3 | `substring` O(start + len) per call | **FIXED** | R6: O(len) byte-slice for ASCII |
| SG-4 | Codegen `.clone()` on every non-final variable use | **IN PROGRESS** | R3: borrow-based codegen for read-only args |
| SG-5 | Non-TCO recursive functions clone through return path | **OPEN** | Bounded by expression depth; not dominant cost |
| SG-9 | `list_push(struct.field, item)` always O(N) — Rc field clone prevents try_unwrap | **PARTIAL** | Tokenizer: extract tokens from state struct; systemic: needs `force_clone` gating or record-update optimization |
| SG-6 | R6 byte-index `char_at` corrupts output on non-ASCII source | **FIXED** | Strip Unicode from .dag comments; long-term: Vec\<char\> |
| SG-7 | Nested Rc match arms: duplicate outer patterns → unreachable!() | **FIXED** | Merge same-variant arms into if-let chains |
| SG-8 | Operator precedence: `(a-b)/c` renders as `a-b/c` → infinite loop | **FIXED** | Parenthesize BinOp sub-expressions in render_operator_operand |

---

## Postmortem: SG-6 — Byte/Character Position Mismatch (2026-03-18)

**Symptom:** Gist resolve takes >20 minutes in release mode for 1,515 lines
(42K characters) even after R3 (no String cloning), R6 (O(1) intrinsics),
and R8 (Rc-wrapped types, O(1) clone). Memory improved from 10GB to 620MB
but time stayed at >20 minutes.

**Discovery path:**
1. Per-stage profiling test showed the FIRST file (`types.dag`, 17K chars)
   hadn't finished tokenizing after 60+ seconds
2. Scaling test with synthetic ASCII input (`"type Foo { x: Int }\n" × N`)
   showed correct O(n^1.17) behavior: 10K chars in 43ms
3. The same tokenizer hanging on real 17K-char files pointed to a
   content-dependent issue, not algorithmic complexity
4. `types.dag` contains Unicode mathematical symbols in comments: ⟦, ⟧,
   ⊆, ⊥, ⊤, ℤ, 𝔽, Σ, ∪, ∩, etc. (multi-byte UTF-8)

**Root cause:**

R6 optimized `char_at(source, pos)` with a byte-indexed ASCII fast path:
```rust
if pos < bytes.len() && bytes[pos] < 128 {
    return String::from(bytes[pos] as char);  // O(1)
}
s.as_ref().chars().nth(pos).map(...)  // O(pos) fallback
```

The tokenizer maintains `pos` as a **character index** (increments by 1 per
character regardless of byte width). After the first multi-byte character
(⟦ at ~byte 100 in types.dag):

- `pos` (character index) = 100
- Actual byte position of character 100 = ~106 (6 extra bytes from 3-byte ⟦⟧)
- `bytes[100]` is a valid ASCII byte from an **earlier** position in the file
- `bytes[100] < 128` is TRUE → fast path returns the **wrong character**

The tokenizer silently processes corrupted input. Instead of seeing the
expected character at position 100, it sees a character from position ~94.
This causes it to misinterpret tokens — likely scanning endlessly for a
delimiter that never appears at the expected position.

**Why this wasn't caught:**
- All v2 compiler `.dag` files (`src/v2/*.dag`) are ASCII-only
- The gist dependency chain includes `dsl/std/types.dag` which has
  Unicode in set-theory notation comments
- The `v2_crate_cargo_check` test (which passes) only compiles the
  generated crate — it doesn't run the gist resolve test
- The tokenizer scaling test used synthetic ASCII input
- The failure mode is a hang (wrong characters → infinite scan), not a
  crash or error message

**Fix:**
- Immediate: strip Unicode from `.dag` file comments (ASCII equivalents)
- Long-term: convert source to `Vec<char>` once at tokenize entry; use
  index-based access (`chars[pos]`) for O(1) correct access regardless
  of encoding

**Structural lesson:**

Any optimization that assumes byte position = character position is a
latent corruption bug. The hybrid approach ("check if this specific byte
is ASCII") is only correct when ALL preceding bytes are single-byte
characters — a property that cannot be verified per-byte. Either commit
fully to byte indexing (track byte offsets everywhere) or fully to
character indexing (accept O(pos) or pre-convert to Vec<char>).

This is the same class of bug as SG-1 (O(n²) char_at) — the R6 "fix"
traded one failure mode (slow but correct) for another (fast but silently
wrong on non-ASCII input). The correct fix addresses both: `Vec<char>`
conversion gives O(1) access that works for all encodings.

### Why tests didn't catch it

**What exists:**

- Generated crate tokenizer tests: `tokenize("fn foo()")`, `tokenize("type Foo { x: Int }")`,
  self-parse of `src/v2/*.dag` files — **all pure ASCII inputs**
- `v2_crate_cargo_check`: compiles the generated crate but **doesn't run it**
- `v2_crate_gist_resolve`: processes `dsl/std/types.dag` (which had Unicode) —
  **but is `#[ignore]` because it was "too slow"** (circular: slow because of this bug)
- 92 non-ignored host tests, 363 codegen unit tests — **none process non-ASCII input**

**What's missing:**

| Missing test layer | What it would catch |
|--------------------|---------------------|
| `v2_rt` intrinsic unit tests for non-ASCII | `char_at("AB⟦CD", 3)` returns wrong char |
| Tokenizer round-trip property test | `tokens.map(text).join() != source` detects corruption |
| Non-ignored gist tokenize smoke test | Any gist processing bug surfaces immediately |
| Token position/content assertions | Position N produces unexpected token kind/text |

**The structural gap:** `v2_rt.rs` is a 250-line string constant embedded in the
v1 emitter. It has **zero test coverage**. Every other layer (parser, codegen,
type system) has tests. But the runtime that every generated program depends on
is invisible to tests, linters, and profilers. Both SG-1 (O(n²) char_at) and
SG-6 (corrupt char_at) lived in this untested layer.

**TODO:** Add `v2_rt` intrinsic unit tests (especially non-ASCII char_at/
string_length), a non-ignored gist tokenize smoke test, and token content
assertions. These are the minimum coverage to prevent this class of bug.

---

## Postmortem: SG-7 — Nested Rc Pattern Match Regression (2026-03-18)

**Symptom:** Parser panics with `unreachable!()` in `status_expr_to_str`
when parsing `gcp.dag` response blocks containing integer status codes.

**Root cause:** R8's codegen for nested Rc-wrapped patterns produced
duplicate match arms. When multiple arms match the same outer variant
(`Expr::Literal`) but differ in the inner Rc-wrapped sub-pattern
(`LitInt` vs `LitStr`), the codegen emitted:

```rust
match expr.as_ref() {
    Expr::Literal { ref value, .. } => {
        let LiteralValue::LitInt { .. } = value.as_ref() else { unreachable!() };
        // ...
    }
    Expr::Literal { ref value, .. } => {  // DEAD CODE
        let LiteralValue::LitStr { .. } = value.as_ref() else { unreachable!() };
    }
}
```

Rust always matches the first arm. When the literal is `LitStr`, the
`let-else` hits `unreachable!()`.

**Fix:** Added arm-grouping logic in `compile_match_typed` that detects
consecutive arms with the same Rc-nested variant key and merges them
into a single arm with an `if let` / `else if let` chain.

---

## Postmortem: SG-8 — Operator Precedence in Generated Code (2026-03-18)

**Symptom:** After fixing SG-7, the parser hangs (infinite loop) in
`int_to_string_acc` when converting status code `200` to a string.
`samply` profiling confirmed 100% CPU in `int_to_string_acc`.

**Root cause:** `render_operator_operand` in `render_rust.rs` did not
parenthesize `BinOp` sub-expressions. The DAG source:

```dag
let rest = (value - digit) / 10
```

Generated as:

```rust
let rest = value.clone() - digit.clone() / 10;
```

Due to Rust's operator precedence, `/` binds tighter than `-`, so this
evaluates as `value - (digit / 10)` instead of `(value - digit) / 10`.
For `value = 200, digit = 0`: `rest = 200 - 0 = 200` — value never
decreases, infinite loop.

**Fix:** One-line change — add `Expr::BinOp { .. }` to
`needs_grouping_in_operator` in `render_rust.rs`. All nested binary
sub-expressions are now parenthesized in the generated code.

**Why this wasn't caught:** The v2 compiler's own `.dag` source
(`src/v2/`) doesn't use `(a - b) / c` patterns. The gist dependency
chain exercises `int_to_string` via `status_expr_to_str` in service
operation response blocks — a code path unique to extdep `.dag` files.
No test exercised this path until the gist pipeline test.

---

## Final Performance Results (2026-03-18)

```text
Gist pipeline: 11 sources, 42K chars, ~6K tokens

  Tokenize: 20ms    (was >20 minutes — SG-6 fix)
  Parse:     3ms    (was infinite — SG-7 + SG-8 fixes)
  Resolve: 342us    (was never reached)
  Total:   24ms     (50,000x improvement)
```

The three layered causes, each masked by the one above it:

1. **SG-6 (Unicode corruption):** `char_at` byte-indexed fast path
   returned wrong characters after multi-byte UTF-8 in comments.
   Tokenizer hung processing corrupted input.

2. **SG-7 (Duplicate match arms):** R8's nested Rc pattern codegen
   produced dead match arms. Parser panicked on `status_expr_to_str`
   when encountering a string literal where it expected an integer.

3. **SG-8 (Operator precedence):** Missing parentheses in generated
   binary expressions. `int_to_string_acc` looped forever because
   `(value - digit) / 10` rendered as `value - digit / 10`.

Each fix revealed the next bug. Only after all three were resolved did
the pipeline complete.

---

## Table of Contents

1. [Pipeline Overview](#pipeline-overview)
2. [Layer 1: Test Harness](#layer-1-test-harness)
3. [Layer 2: v1 Codegen](#layer-2-v1-codegen)
4. [Layer 3: Generated Rust Runtime](#layer-3-generated-rust-runtime)
5. [Smoking Guns](#smoking-guns)
6. [Prior Optimizations: What They Fixed vs What They Missed](#prior-optimizations)
7. [Compounding Effects](#compounding-effects)
8. [Recommended Fix Order](#recommended-fix-order)
9. [Structural Prevention](#structural-prevention)
10. [Connection to Self-Hosting](#connection-to-self-hosting)

---

## Pipeline Overview

```text
.dag source files (13,871 lines across 9 v2 modules)
        │
        ▼
v1 PARSE: daglang-syntax parses .dag → SourceFile ASTs
        │
        ▼
v1 CODEGEN: daglang-emit compiles ASTs → generated Rust
  ├── type_codegen.rs: type definitions → Rust structs/enums
  ├── fn_codegen.rs: function bodies → code_ir → Rust expressions
  └── render_rust.rs: code_ir → Rust source text with stacker wrapping
        │
        ▼
GENERATED CRATE: 13 Rust source files (~10K lines)
  ├── v2_core.rs (types), tokenize.rs, parse.rs, resolve.rs
  ├── infer.rs, emit.rs, emit_rust.rs, emit_python.rs
  ├── pipeline.rs, v2_rt.rs (runtime intrinsics)
  └── generated_tests.rs (gist/self-compile tests)
        │
        ▼
cargo build → compiled v2 compiler binary
        │
        ▼
RUNTIME: gist_resolve_all_modules test
  tokenize(12 files) → parse(12 files) → resolve(12 modules)
  Input: 1,515 lines / ~50K characters / 12 modules
  Expected: seconds. Actual: OOM (release mode, 16 min, SIGKILL)
```

---

## Layer 1: Test Harness

**File:** `src/v2/tests/src/lib.rs:3513`

```text
v2_crate_gist_resolve()
  → assemble_v2_crate_to_dir("v2-compiler-gist-resolve")   [~2s]
    → read 9 .dag files
    → v1 parse each → SourceFile AST
    → v1 codegen → generated Rust files
    → write to /tmp/v2-compiler-gist-resolve/
  → cargo test (debug mode) in generated crate               [~5s build]
    → gist_resolve_all_modules test runs                      [OOM]
```

**Observation:** Assembly and compilation are fast (<10s total). All time is
in the generated code's runtime.

---

## Layer 2: v1 Codegen

### Type codegen (`src/v1/07_emit/daglang-emit/src/type_codegen.rs`)

Generates Rust structs/enums from `.dag` type definitions. Key decisions:

- **Boxing:** `compute_recursive_fields()` (line 1509) detects cycles and
  boxes recursive fields. Currently boxes `return_type`, `body`,
  `type_annotation` on Node. Does NOT box `transport` or `config`.
- **Rc wrapping:** All `List<T>` fields become `Rc<Vec<T>>` in generated Rust.
  All `Map<K,V>` fields become `Rc<HashMap<K,V>>`.

**Generated Node struct** (from `src/v2/00_core.dag` lines 338-357):
```rust
// src/v2_core.rs (generated)
pub struct Node {
    pub name: String,                          // 24b
    pub span: SourceSpan,                      // 16b
    pub children: Rc<Vec<Node>>,               // 8b (pointer)
    pub connective: Option<Connective>,        // 2b + padding
    pub params: Rc<Vec<Param>>,                // 8b
    pub return_type: Box<Option<Node>>,        // 8b (boxed ✓)
    pub uses: Rc<Vec<ResourceUse>>,            // 8b
    pub body: Box<Option<Expr>>,               // 8b (boxed ✓)
    pub transport: Option<TransportBinding>,   // ~56b INLINE ✗
    pub properties: Rc<Vec<FieldInit>>,        // 8b
    pub type_annotation: Box<Option<Node>>,    // 8b (boxed ✓)
    pub config: Option<ServiceConfig>,         // ~32b INLINE ✗
}  // ~200+ bytes total
```

**DAG source:** `src/v2/00_core.dag:338-357`

### Function codegen (`src/v1/07_emit/daglang-emit/src/fn_codegen.rs`)

Compiles `.dag` function bodies to `code_ir` then renders to Rust.

**TCO transformation** (line 4837): Tail-recursive functions become
`loop { ... continue; }` patterns. The R5 fix (line 4857) changed
clone → move at loop top.

**Clone insertion** (`clone_if_needed`, line 3997): Every variable mention
that isn't the final use gets `.clone()`. This is correct for value
semantics but generates enormous clone traffic.

**DAG source (tokenizer):** `src/v2/01_tokenize.dag`
**DAG source (parser):** `src/v2/02_parse.dag`
**DAG source (resolver):** `src/v2/03_resolve.dag`

### Render (`src/v1/07_emit/daglang-emit/src/render_rust.rs`)

Wraps all function bodies in `stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, ...)`.

---

## Layer 3: Generated Rust Runtime

### Runtime intrinsics (`v2_rt.rs`, hardcoded ~226 lines)

**File:** `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs` (V2_RUNTIME_SOURCE constant)

Key functions:

```rust
// char_at: takes String by value, clones on every call site
pub fn char_at(s: String, pos: i64) -> String {
    s.chars().nth(pos as usize).map(|c| c.to_string()).unwrap_or_default()
}

// string_length: takes String by value
pub fn string_length(s: String) -> i64 {
    s.chars().count() as i64
}

// substring: takes String by value
pub fn substring(s: String, start: i64, end: i64) -> String {
    s.chars().skip(start as usize).take((end - start) as usize).collect()
}

// list_push: Rc-based COW
pub fn list_push<T: Clone>(list: Rc<Vec<T>>, item: T) -> Rc<Vec<T>> {
    let mut v = Rc::try_unwrap(list).unwrap_or_else(|rc| (*rc).clone());
    v.push(item);
    Rc::new(v)
}
```

### Tokenizer (`tokenize.rs`, generated from `src/v2/01_tokenize.dag`)

**Entry:** `tokenize(source: String) → Rc<Vec<Token>>`

**Loop body (generated):**
```rust
// Line ~47 in generated tokenize.rs
let ch = v2_rt::char_at(source.clone(), s.clone().pos.clone());
//                       ^^^^^^^^^^^^^^
//                       CLONES ENTIRE SOURCE STRING PER CHARACTER
```

**Per-character cost:**
- `source.clone()`: O(n) where n = source length
- `char_at()`: `.chars().nth(pos)` scans from start = O(pos)
- Combined: O(n + pos) per character → O(n²) total for tokenization

**DAG source:** `src/v2/01_tokenize.dag:128-130`
```dag
let ch = char_at(s: source, pos: s.pos)
```
This innocent-looking line becomes a full string clone + linear scan in
generated Rust because:
1. `source` is passed by value in the DAG (value semantics)
2. The codegen inserts `.clone()` because `source` is used again later
3. `char_at` takes `String` by value (not `&str`)
4. `.chars().nth(pos)` is O(pos) not O(1)

### Parser (`parse.rs`, generated from `src/v2/02_parse.dag`)

**Accumulator pattern (TCO, correct):**
```rust
// parse_items_acc: TCO loop with Rc::try_unwrap for list_push
let __rc_1 = acc;  // move, refcount stays 1
let mut __appended_0 = Rc::try_unwrap(__rc_1)
    .unwrap_or_else(|rc| (*rc).clone());  // succeeds: refcount == 1
__appended_0.push(r.clone().item);
Rc::new(__appended_0)
```

**Token access pattern:**
```rust
// Every token lookup clones the token list
let kind = tokens.clone()[state.clone().pos as usize].clone().kind;
//         ^^^^^^^^^^^^^^^                           ^^^^^^^^
//         clones Rc (cheap)                         clones Token
```

**DAG source (token access):** `src/v2/02_parse.dag:212-220`

### Resolver (`resolve.rs`, generated from `src/v2/03_resolve.dag`)

**Kahn's algorithm (correct, O(M+E)):**
```rust
// Edge partitioning per round — O(E) total across all rounds
let processed_edges = state.edges.iter().cloned()
    .filter(|e| zero_set.contains_key(&e.from_module))
    .collect();
let remaining_edges = state.edges.iter().cloned()
    .filter(|e| !zero_set.contains_key(&e.from_module))
    .collect();
```

**DAG source:** `src/v2/03_resolve.dag:464-485`

---

## Smoking Guns

### SG-1: `char_at(source.clone(), pos)` — O(n²) tokenization

**Severity:** CRITICAL — dominates all other costs
**Location (DAG):** `src/v2/01_tokenize.dag:128`
**Location (generated):** `tokenize.rs:~47`
**Location (runtime):** `v2_rt.rs` in `V2_RUNTIME_SOURCE` constant,
  `src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs:~line 70+`

The tokenizer calls `char_at(s: source, pos: s.pos)` for every character.
In the generated Rust:
- `source.clone()` copies the entire string: O(n)
- `s.chars().nth(pos)` scans from the beginning: O(pos)
- Called ~50,000 times for 1,515 lines
- Total: Σ(pos) for pos=0..50000 ≈ **1.25 billion character operations**

**Fix:** Change `char_at` to take `&str` instead of `String`. Requires
codegen to pass string references instead of cloning. Alternatively,
convert source to `Vec<char>` once and index in O(1).

### SG-2: `string_length(source.clone())` — O(n) per call, called per token

**Severity:** HIGH
**Location (DAG):** `src/v2/01_tokenize.dag:124`
**Location (runtime):** `v2_rt.rs`

`string_length` takes `String` by value and calls `.chars().count()`.
The codegen clones the source string to pass it. Called at the top of
every tokenizer loop iteration.

**Fix:** Same as SG-1 — take `&str`, avoid clone.

### SG-3: `substring(source.clone(), start, end)` — O(n) per call

**Severity:** HIGH
**Location (DAG):** `src/v2/01_tokenize.dag:150+` (in scan_ident, scan_number, etc.)
**Location (runtime):** `v2_rt.rs`

Every identifier/keyword extraction clones the source string, then
iterates to the start position, then collects characters.

**Fix:** Take `&str`, use byte-offset slicing if ASCII, or precomputed
char indices.

### SG-4: Codegen inserts `.clone()` on every non-final variable use

**Severity:** MEDIUM (compounds with SG-1/2/3)
**Location:** `src/v1/07_emit/daglang-emit/src/fn_codegen.rs:3997`

The `clone_if_needed` strategy clones every variable mention except the
last. For `source` in the tokenizer, this means cloning a potentially
large string dozens of times per token.

**Fix:** Use `&str` borrows for read-only access. Only clone when
ownership transfer is required (rare in practice).

### SG-5: Non-TCO recursive functions still clone through return path

**Severity:** MEDIUM
**Location (DAG):** `src/v2/02_parse.dag` — `parse_expr_bp` calls itself
  recursively for right-associative operators and prefix expressions.

Each recursive call receives `tokens.clone()` (cheap Rc clone) but also
passes result structs containing `Expr` (~208 bytes) through the stack.

**Impact:** Bounded by expression depth (usually <20 for gist sources).
Not the dominant cost but contributes to stack pressure.

---

## Prior Optimizations: What They Fixed vs What They Missed

### What the .dag-level perf audit (Track S) fixed

| Fix | Layer | Impact | Still relevant? |
|-----|-------|--------|-----------------|
| concat→list_push (28→38 sites) | .dag source | O(n²)→O(n) list building | Yes, but only matters if list_push is O(1) |
| Kahn O(N×E)→O(E) | .dag source | Edge partitioning | Yes, correct |
| typed_expr_to_expr deletion | .dag source | Removed dead O(n) traversal | Yes |
| Emitter classification dedup | .dag source | 3x less code to maintain | Yes, maintenance win |

### What the .dag-level audit missed

| Issue | Layer | Why missed |
|-------|-------|-----------|
| **SG-1: char_at clones source** | v2_rt.rs (hardcoded runtime) | Audit only examined .dag files |
| **SG-2/3: string intrinsics take String by value** | v2_rt.rs | Runtime was treated as a black box |
| **SG-4: pervasive .clone() in codegen** | fn_codegen.rs (v1) | Audit didn't examine generated Rust patterns |
| **R5: TCO clone leak** | fn_codegen.rs (v1) | Codegen was treated as correct-by-construction |
| **Node size 200b+ with inline transport/config** | type_codegen.rs (v1) | Type representation not audited |

### The gap

The .dag-level audit asked: "are the DAG algorithms efficient?"
It did not ask:
- "What Rust code does the codegen actually produce?"
- "Are the runtime intrinsics efficient?"
- "Does the codegen's clone strategy interact poorly with value semantics?"

**The performance cliff lives at the boundary between .dag semantics and
Rust codegen** — specifically in how value-semantic string operations are
lowered to ownership-based Rust.

---

## Compounding Effects

The individual issues compound multiplicatively:

```text
Per character in tokenizer:
  source.clone()           O(n)     ← SG-1: string clone
  × char_at nth(pos)       O(pos)   ← SG-1: linear scan
  × string_length clone    O(n)     ← SG-2: another string clone
  × check/advance calls    O(k)     ← SG-4: clone per variable use

Per token:
  × scan_ident/number      O(len)   ← SG-3: substring clones
  × list_push              O(1)     ← FIXED by R5

Per file (n chars, t tokens):
  tokenize: O(n² + t×n)             ← dominated by char_at × n
  parse:    O(t) with O(1) push     ← FIXED by concat→list_push + R5
  resolve:  O(M + E)                ← FIXED by Kahn optimization

For 12 files totaling 50K chars:
  tokenize alone: ~50K × 50K/2 = 1.25 billion string-char ops
  At ~10ns per char operation: ~12.5 seconds theoretical minimum
  With clone overhead: 10-100x → minutes
```

The parse and resolve layers are likely fine after our fixes. **Tokenization
is the bottleneck**, and it's not a DAG algorithm issue — it's a runtime
intrinsic implementation issue.

---

## Recommended Fix Order

### P0: Fix string intrinsics in v2_rt.rs

**Impact:** Eliminates O(n²) tokenization, likely makes gist resolve <10s.
**Scope:** ~50 lines in `V2_RUNTIME_SOURCE` constant.
**Risk:** Low — runtime intrinsics are self-contained.

Change `char_at`, `string_length`, `substring`, `string_contains` to take
`&str` instead of `String`. This requires the codegen to pass borrows
for string arguments.

Alternatively (smaller change): convert source to `Vec<char>` at
tokenizer entry and use index-based access throughout. This avoids
changing the codegen's ownership model.

### P1: Index-based string access in tokenizer

**Impact:** O(1) character access instead of O(pos).
**Scope:** Changes to `src/v2/01_tokenize.dag` + corresponding codegen.

If `char_at` takes a `Vec<char>` + index instead of `String` + position,
character access becomes O(1) instead of O(pos).

### P2: Box transport/config on Node (Track R2)

**Impact:** Reduces Node size, reduces clone traffic, reduces stack frames.
**Scope:** Changes to boxing algorithm in `type_codegen.rs`.

### P3: Reference-based codegen for read-only access (Track R3)

**Impact:** Eliminates unnecessary clones for immutable access patterns.
**Scope:** Large — changes `clone_if_needed` strategy in `fn_codegen.rs`.

---

## v2_rt.rs: Full Intrinsic Audit

**File:** `src/v1/07_emit/daglang-emit/src/v2_runtime_shim.rs`
**Injected into:** every generated v2 crate as `src/v2_rt.rs`

v2_rt.rs is a 226-line Rust string constant embedded in the v1 emitter.
It provides the primitive operations that `.dag` intrinsics compile to.
It was written once, never profiled, never tested at scale, and never
included in any performance audit.

### Function-by-function complexity

| Function | Signature | Complexity | Problem | Status |
|----------|-----------|-----------|---------|--------|
| `char_at` | `impl AsRef<str>, i64 → String` | **O(1)** | Was O(pos): `.chars().nth(pos)` scanned from start | **FIXED** (R6) |
| `string_length` | `impl AsRef<str> → i64` | **O(1)** | Was O(n): `.chars().count()` iterated full string | **FIXED** (R6) |
| `substring` | `impl AsRef<str>, i64, i64 → String` | **O(len)** | Was O(start + len): skip + collect + allocate | **FIXED** (R6) |
| `string_contains` | `impl AsRef<str>² → bool` | O(n×m) | Correct for small patterns | OK |
| `scan_while` | `impl AsRef<str>, i64, Fn → i64` | **O(scanned)** | Was O(n): rebuilt `Vec<char>` every call | **FIXED** (R6) |
| `skip_horizontal_ws` | `impl AsRef<str>, i64 → i64` | **O(scanned)** | Was O(n): rebuilt `Vec<char>` every call | **FIXED** (R6) |
| `scan_to_eol` | `impl AsRef<str>, i64 → i64` | **O(scanned)** | Was O(n): rebuilt `Vec<char>` every call | **FIXED** (R6) |
| `scan_string_end` | `impl AsRef<str>, i64 → i64` | **O(scanned)** | Was O(n): rebuilt `Vec<char>` every call | **FIXED** (R6) |
| `concat` (String) | `String, String → String` | O(a+b) | Correct | OK |
| `concat` (Rc<Vec>) | `Rc<Vec>, Rc<Vec> → Rc<Vec>` | O(a+b) or O(b) | COW when refcount=1 | OK |
| `list_push` | via codegen Rc pattern | O(1) amortized | Was O(n): cloned entire Vec | **FIXED** (R5) |
| `map_insert` | `Rc<HashMap>, K, V → Rc<HashMap>` | O(1) amortized | COW when refcount=1 | OK |
| `map_merge` | `Rc<HashMap>² → Rc<HashMap>` | O(overlay) | Correct | OK |
| `lookup` | `&HashMap, &str → Option<V>` | O(1) | Correct | OK |
| `index_by` | `Rc<Vec>, Fn → Rc<HashMap>` | O(n) | Correct | OK |
| `code_point` | `impl AsRef<str> → i64` | O(1) | Correct | OK |
| `from_code_point` | `i64 → String` | O(1) | Correct | OK |
| `filesystem_read` | `String → Result` | O(file_size) | Correct (I/O) | OK |

**Note:** The string functions accept `impl AsRef<str>` — they CAN take
`&str` without cloning. The clone problem is in the codegen, which
generates `source.clone()` at every call site regardless.

### Compounding with codegen clone strategy

The codegen (`fn_codegen.rs:3997`, `clone_if_needed`) inserts `.clone()`
on every non-final variable use. So even though `char_at` accepts
`impl AsRef<str>`, the generated code passes `source.clone()`.

Three costs compound per character in the tokenizer:
1. `source.clone()` — O(n) string copy (codegen)
2. `.chars().nth(pos)` — O(pos) scan (runtime)
3. `string_length(source.clone())` — O(n) per iteration (both)

For 50K characters: Σ(n + pos) ≈ **1.25 billion character operations**.

---

## Process Observation: Why This Happened

### The audit cycle

1. **Track S (DAG-level):** Found and fixed O(n²) list builders, linear
   scans, dead code. All valid fixes at the algorithm level.

2. **P0 investigation:** Found stack overflow in generated code, traced to
   Node size. Fixed stacker threshold. Missed that runtime intrinsics
   were the dominant cost.

3. **R5 (TCO clone leak):** Found and fixed Rc refcount issue in TCO loops.
   Valid fix — list_push is now O(1) in TCO paths.

4. **This audit:** Found that the dominant cost is `char_at(source.clone())`
   in the hardcoded runtime, which was never examined by any prior audit.

### Three structural failures enabled this

**1. v2_rt.rs is a string constant, not a real source file.**

It lives inside `v2_runtime_shim.rs` as a `const &str`. It's invisible
to tests, linters, profilers, and code review. It's 226 lines that
every generated program depends on, hidden inside a Rust string literal.
No other code in this project operates this way — everything else is
either `.dag` source (auditable) or real Rust source (compilable,
testable). v2_rt.rs is neither.

**2. No end-to-end performance gate.**

We have correctness tests (does the output match?) but zero performance
tests (does it finish in <N seconds?). An O(n²) regression passes every
test we have. A function that takes 16 minutes instead of 10ms is
indistinguishable from a correct one in our test suite.

**3. The codegen has no complexity model.**

`clone_if_needed` doesn't know whether it's inside a loop that runs 50K
times. It treats every clone as equally cheap. There's no mechanism to
say "this argument is read-only, pass by reference" or "this clone is
inside a hot loop and costs O(n)."

---

## Structural Prevention: Making This Impossible

The goal is not to find and fix this class of bug faster — it's to make
it **structurally unrepresentable**. Three changes would do this:

---

## Structural Prevention

Wall-clock performance tests are the wrong answer. They're
machine-dependent, symptom-testing, and tell you nothing about cause.
The question isn't "is it fast?" — it's "can we prove it's fast?"

### Why this happened

Three v1 infrastructure layers sat below the `.dag` source and escaped
every invariant the project enforces:

| Layer | File | Lines | Subject to invariants? |
|-------|------|-------|----------------------|
| Runtime intrinsics | `v2_runtime_shim.rs` (string constant) | 226 | **No** — invisible to tests, linters, review |
| Expression codegen | `fn_codegen.rs` | 5,000+ | **No** — hand-written Rust, never profiled |
| Type codegen | `type_codegen.rs` | 2,000+ | **No** — boxing/clone decisions unaudited |
| Rendering | `render_rust.rs` | 200+ | **No** — stacker wrapping only |

Every layer that caused this problem is **v1 scaffolding that self-hosting
eliminates.** The `.dag` source has invariants. The v1 codegen does not.

### What prevents this structurally

Three mechanisms, each addressing a different level:

**A) Kernel primitives have declared complexity contracts.**

Each primitive operation (`char_at`, `string_length`, `list_push`,
`map_insert`) must declare its complexity. The declaration is the spec;
implementations must satisfy it.

```dag
// In dsl/std/primitives.dag or equivalent
extern func char_at(s: String, pos: Int) -> String
  // complexity: O(1)

extern func string_length(s: String) -> Int
  // complexity: O(1)

extern func list_push(list: List<T>, item: T) -> List<T>
  // complexity: O(1) amortized
```

An implementation that's O(pos) for `char_at` violates its declared
contract. This is verifiable without timing — you inspect the
implementation and check that it uses indexed access, not linear scan.

**B) The cost algebra (Track D) runs on the compiler itself.**

The tokenizer's `tokenize_loop` gets a `ComplexitySummary`:

```text
W_tokenize(n) = Sum(i = 0..n,
  PrimCost("char_at", [Const(1)], model)
  + PrimCost("string_length", [Const(1)], model)
  + PrimCost("list_push", [Const(1)], model)
)
```

If `char_at`'s model says O(1), this reduces to O(n). If the model says
O(i), it reduces to O(n²) and the analyzer flags it. **The compiler
proves its own performance** — no wall-clock heuristics needed.

**C) Self-hosting makes the proof load-bearing.**

A compiler that can't tokenize 1,515 lines can't compile itself. The
fixed-point test (Track A, stage A6: `stage1 output == stage2 output`)
requires the compiler to be efficient enough to process its own source
(~14,000 lines). An O(n²) tokenizer makes self-compilation infeasible.

Self-hosting isn't a performance test — it's a structural constraint.
You don't need to assert "tokenize < 2s" if the compiler must tokenize
itself to exist.

---

## Connection to Self-Hosting

In the self-hosted world, every layer that caused this problem disappears:

| v1 layer (disappears) | v2 replacement | Why it's auditable |
|------------------------|---------------|-------------------|
| `v2_runtime_shim.rs` (string constant) | `.dag`-defined `extern func` with complexity contracts | Subject to parse/infer/emit pipeline, Track D analysis |
| `fn_codegen.rs` (clone strategy) | `05_emit_rust.dag` (v2 emitter) | The emitter IS `.dag` code — same invariants apply |
| `type_codegen.rs` (boxing decisions) | `05_emit_rust.dag` type emission | Boxing rules expressed in `.dag`, not hidden in Rust |
| `render_rust.rs` | Subsumed by `05_emit_rust.dag` | One layer, not two |

The fundamental shift: in v1, there are **two compilers** — the `.dag`
compiler and the Rust codegen. The `.dag` compiler has invariants; the
Rust codegen doesn't. In v2, there's **one compiler** that compiles
itself. Every layer is subject to the same invariants.

---

## Postmortem: SG-9 — Rc Struct Field list_push is Always O(N) (2026-03-18)

**Symptom:** `self_compile_all_modules` hangs for 60+ seconds (expected: milliseconds).
Gist compile (11 small files) completes in 30ms. Self-compile (14 large files,
including 3000-line reconciler) never completes.

**Root Cause:** Every `list_push(state.field, item)` in the compiled Rust is O(N)
instead of O(1). The v1 compiler emits:

```rust
let __rc = state.field.clone();   // Rc clone → refcount 2
let mut v = Rc::try_unwrap(__rc)  // FAILS — refcount is 2
    .unwrap_or_else(|rc| (*rc).clone());  // falls back to Vec clone: O(N)
v.push(item);
Rc::new(v)
```

The `state.field.clone()` creates a second Rc reference to the same Vec.
`state` itself still holds the original reference. `Rc::try_unwrap` requires
refcount == 1, so it always fails, triggering a full Vec clone.

**Why refcount is always 2:** Two independent mechanisms prevent refcount 1:

1. **Struct field access clones the Rc.** `state.field` on an `Rc<Struct>`
   returns a reference to the inner field. The only way to get an owned
   `Rc<Vec<T>>` is `.clone()`, which creates refcount 2.

2. **`force_clone` flag.** In `fn_codegen.rs:1724`, the v1 compiler forces
   `.clone()` on ALL variable references when any Rc-wrapped types exist:
   `let force_clone = !ctx.rc_wrapped_types.is_empty()`. This prevents
   single-use variables from being moved, even when move would be safe.

**Impact:** O(N²) for any function that accumulates a list inside a struct
via recursion or TCO. Affects: tokenizer (N = tokens per file), parser
(N = statements per block), reconciler (various accumulators).

For the tokenizer processing a 3000-line file with ~15,000 tokens:
O(15,000²) ≈ 225M token copies × ~50 bytes ≈ 11 GB of data movement.

**Partial fix (tokenizer):** Restructured `01_tokenize.dag` to pass `tokens`
as a standalone parameter instead of inside `TokenizerState`. The TCO loop
variable has refcount 1, so `try_unwrap` succeeds. Added `TokPos` type
(pos + interp_depth, no tokens) for the position-only state.

**Systemic fix needed:** The `force_clone` flag must be scoped to only
force clone on match-bound variables (references from enum destructuring),
not standalone let bindings or function parameters. Alternatively, implement
a "functional record update" optimization: when a record is constructed from
an existing record's fields with one field modified via `list_push`, the
compiler should destructure the old record first (getting owned fields),
modify in-place, and re-wrap.

**Partial fix validation:** After the tokenizer restructure, the compiled
tokenizer uses `let __rc_1 = tokens;` (move, no clone) for the standalone
parameter — try_unwrap succeeds, O(1) per push. The parser's standalone `acc`
parameters also produce moves (22/26 list_push calls in parse.rs use
`let __rc_1 = acc;`). The remaining bottleneck is in the reconciler's
`typecheck` fold, where `acc.module_index.clone()` and `acc.diagnostics.clone()`
are struct field accesses that always clone.

**Remaining hot paths (reconciler fold):**
- `map_insert(acc.module_index, name, typed_module)` — clones the full module index per module
- `concat(acc.diagnostics, new_diags)` — clones all accumulated diagnostics per module
- Various `resolve_env_bindings` calls that merge parent type environments

**Severity:** Critical for self-hosting. The reconciler's fold accumulator
pattern accounts for the remaining hang. Until the systemic fix is applied,
any fold that accumulates into struct fields has O(N²) complexity.

**Investigation trace (2026-03-18):**

1. Tokenizer restructure: extracted `tokens` from `TokenizerState` into
   standalone TCO parameter → `TokPos` type. Compiled output confirmed
   `let __rc_1 = tokens;` (move, no clone). **Result:** tokenizer TCO loop
   is O(1), but sub-function calls still clone (`.clone()` on function args).

2. Typecheck fold → explicit `typecheck_modules` recursion with standalone
   `modules`, `module_index`, `diagnostics` parameters. Same TCO pattern.
   **Result:** fold overhead eliminated, but same sub-function clone issue.

3. `force_clone` exemption for fold accumulators: allowed `fold_accum_name`
   to skip force_clone when count <= 1. **Result:** too narrow — only helps
   fold callbacks, not TCO or regular function calls.

4. Branch-aware use counting: changed `count_ident_uses_expr` for Match and
   If to compute max across exclusive branches, not sum. **Result:** correct
   counting, but `force_clone` still blocks the optimization for non-fold vars.

5. Attempted `owned_param_names` tracking to distinguish owned function
   params from match-bound references. **Incomplete — reverted.**

**Why the incremental fixes fail:** The `force_clone` flag at `fn_codegen.rs:1752`
forces `.clone()` on ALL variable references when Rc types exist. It was added
because removing it causes compile errors for match-bound variables (which are
references, not owned values, in the compiled Rust). But the flag also blocks
clone elision for owned variables (function params, let bindings, TCO loop vars)
where moving is safe.

The branch-aware counting correctly gives count=1 for variables used once per
exclusive branch. But `force_clone` overrides: `if count <= 1 && !force_clone`.
The fold_accum exemption punches a hole for one specific case, but the pattern
repeats everywhere (TCO params, function call args, let bindings).

**Root cause is SG-4, not SG-9.** SG-9 is a symptom of the broader SG-4 issue:
the v1 compiler doesn't distinguish owned values from borrowed references.
`force_clone` is the band-aid that prevents borrow-checker errors by cloning
everything. The real fix is ownership-aware codegen (SG-4), which would make
SG-9's workarounds (tokenizer restructure, typecheck recursion) unnecessary.

**Invariant violations in the current workarounds:**

1. **No case enumeration for open sets:** `is_builtin_collection_func` and
   `is_emitter_builtin_func` hardcode 14 builtin names as string comparisons.
   Builtins are an open set. The method-call handler already knows these names
   structurally — the free-function path should derive from the same source.

2. **No duplicate representations:** The two builtin lists (reconciler + emitter)
   encode the same knowledge. `TokenizerState.tokens` is now dead (replaced by
   standalone parameter). `TypecheckAccum` is now dead (replaced by explicit
   recursion).

3. **No parallel implementations:** The tokenizer and typecheck restructures
   are structural workarounds for a codegen deficiency. If SG-4 is fixed, these
   workarounds should be reverted to the simpler fold/struct patterns.

---

Self-hosting doesn't automatically make `char_at` O(1). But it makes the
`char_at` implementation **visible, auditable, and subject to the cost
algebra.** The combination of declared complexity (A), proven bounds (B),
and self-hosting pressure (C) makes O(n²) intrinsics structurally
unrepresentable — they'd violate their declared contract, fail the cost
analysis, and prevent the compiler from reaching fixed point.

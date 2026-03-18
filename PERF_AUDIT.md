# Performance Audit: End-to-End Pipeline Trace

**Date:** 2026-03-18
**Trigger:** Generated v2 crate OOMs on 1,515 lines of gist sources even in
release mode (SIGKILL after 16 minutes, 1061s CPU).

**Purpose:** Trace the complete pipeline from `.dag` source through v1 codegen
to generated Rust, identify every performance cliff, and determine whether
prior optimizations addressed the right layer.

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

## Process Observation

The performance audit cycle looked like this:

1. **Track S (DAG-level):** Found and fixed O(n²) list builders, linear
   scans, dead code. All valid fixes at the algorithm level.

2. **P0 investigation:** Found stack overflow in generated code, traced to
   Node size. Fixed stacker threshold. Missed that runtime intrinsics
   were the dominant cost.

3. **R5 (TCO clone leak):** Found and fixed Rc refcount issue in TCO loops.
   Valid fix — list_push is now O(1) in TCO paths.

4. **This audit:** Found that the dominant cost is `char_at(source.clone())`
   in the hardcoded runtime, which was never examined by any prior audit.

**The lesson:** Each optimization pass focused on its own layer
(.dag algorithms, stack frames, Rc refcounting) without examining the
layer below. The actual bottleneck was in the simplest, most "obviously
correct" code — the 5-line `char_at` function that nobody questioned.

**What should change:** Performance audits must trace end-to-end through
generated code, not stop at the .dag source level. The generated Rust
is the actual program that runs — its runtime characteristics are the
only ones that matter.

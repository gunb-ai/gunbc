# Unicode Everywhere Contract

Status: accepted design target

This document defines the Unicode contract for the compiler, runtime
intrinsics, tokenizer, diagnostics, and emitted backends.

The decision is simple:

- Source files are UTF-8.
- Strings are Unicode.
- Identifiers are Unicode.
- All language-visible string indexing and lengths use Unicode scalar
  values, not bytes.
- Source spans remain UTF-8 byte offsets for diagnostics and tooling.

That split is intentional. Runtime string semantics and source-position
semantics solve different problems and should not be forced into one
index space.

## Why this exists

Today the repo already has a mixed contract:

- the v1 evaluator implements `char_at`, `string_length`,
  `substring`, `scan_while`, `scan_to_eol`, `skip_horizontal_ws`, and
  `scan_string_end` using Unicode scalar iteration;
- evaluator tests already lock Unicode behavior such as
  `string_length("café") == 5`;
- the emitted Rust runtime shim still panics on non-ASCII for some
  intrinsics;
- the v2 Rust and Go emitters still generate byte-based indexing and
  slicing in places;
- source diagnostics in v1 are byte-based.

That is not a "no fallback" issue. It is a contract mismatch. This
document chooses one contract and pushes it through every layer.

## Non-goals

- We are not adopting grapheme-cluster semantics.
- We are not normalizing strings or identifiers.
- We are not making arbitrary Unicode whitespace significant syntax.
- We are not requiring generated code to preserve source identifier
  spelling literally.

## Core terms

### Unicode scalar value

The language-visible unit for string indexing is the Unicode scalar
value. This matches the existing evaluator behavior and is the most
practical cross-backend contract.

Consequences:

- `string_length("café") == 5`
- `char_at("café", 4)` returns the combining accent
- `substring` slices by scalar indices, not bytes and not grapheme
  clusters

### Byte offset

The language-visible unit for `SourceSpan` is the UTF-8 byte offset.
This keeps diagnostics, file I/O, host-parser interop, and editor/LSP
integration aligned with the rest of the toolchain.

Consequences:

- `SourceSpan.start` and `SourceSpan.end` remain byte offsets
- line/column rendering derives from byte offsets by decoding UTF-8
- tokenization may scan in scalar space, but emitted spans are bytes

## Contract

### 1. Source encoding

- All `.dag` source is UTF-8 text.
- Invalid UTF-8 is rejected at the file-read boundary with a diagnostic.
- There is no ASCII fallback path and no late runtime panic for valid
  UTF-8 source.

### 2. String semantics

All language-visible string operations are defined over Unicode scalar
values.

The authoritative behavior is:

- `string_length(s)` returns the number of Unicode scalar values in `s`
- `char_at(s, pos)` returns the scalar at index `pos` as a one-scalar
  string, or `Unit` if out of bounds
- `substring(s, start, end)` returns the half-open scalar slice
  `[max(start, 0), max(end, 0))`, clamped to the string length
- `scan_while(s, start, pred)` advances in scalar space
- `scan_to_eol(s, start)` returns the scalar index of `"\n"` or the
  scalar length
- `skip_horizontal_ws(s, start)` advances over ASCII space and tab only
- `scan_string_end(s, start)` advances in scalar space while honoring
  escapes

Two strings compare equal iff their scalar sequences are identical.
There is no normalization step. `"é"` and `"é"` are distinct strings.

This is deliberate:

- normalization is a semantic rewrite, not a representation detail;
- silently normalizing would violate the "no fallbacks that fabricate"
  invariant;
- later diagnostics may warn on confusables or mixed normalization, but
  warnings are not semantics.

### 3. Source positions and diagnostics

`SourceSpan` remains:

```dag
type SourceSpan {
  start: Int
  end: Int
}
```

with `start` and `end` interpreted as UTF-8 byte offsets.

Rationale:

- file APIs and host parsers already operate in bytes;
- byte offsets map directly to slices of the original file contents;
- editor integrations and downstream tools already expect byte-oriented
  spans.

Line/column display rules:

- line numbers are 1-based;
- columns are 1-based scalar-value columns, not bytes;
- conversion walks `source.char_indices()` up to the byte offset.

This means the repo has two index spaces:

- strings and tokenizer cursors: scalar indices
- spans and file offsets: byte indices

That is not duplication. It is an explicit boundary contract.

### 4. Grammar whitespace

The source file may contain arbitrary Unicode text, but the grammar only
recognizes these separator characters outside strings/comments:

- space `" "`
- tab `"\t"`
- line feed `"\n"`
- carriage return may be normalized or rejected explicitly at the file
  boundary, but it must not silently change span accounting

Other Unicode whitespace code points are not syntax separators.
If they appear outside strings/comments, the tokenizer should reject
them with a diagnostic rather than guessing.

This keeps the grammar explicit and avoids invisible-token bugs.

### 5. Identifiers

Identifiers become Unicode identifiers.

Source-level rules:

- first character: `_` or Unicode XID_Start
- subsequent characters: Unicode XID_Continue or `_`
- keywords remain the ASCII reserved words already defined by the
  language
- identifier equality is raw scalar-sequence equality with no
  normalization or case folding

That means:

- `résumé` is valid
- `变量` is valid
- `é` and `é` are distinct identifiers

### 6. Emitted names

The language contract does not require emitted Rust, Go, or Python
identifiers to preserve source spelling literally.

Instead, all emitters must share one structural rule:

- every source identifier lowers through one canonical mangle function;
- the mangle is deterministic, injective, and backend-safe;
- backend differences are limited to reserved-word escape tables and
  file/module naming rules.

The default emitted identifier policy is:

- preserve safe ASCII identifiers when possible;
- otherwise emit ASCII using scalar-value escapes such as `_u03C0_`;
- prefix backend keywords and invalid starts with a stable escape such
  as `dag_`.

This avoids parallel per-backend heuristics and lets the language accept
full Unicode identifiers without depending on each target language's
Unicode identifier rules.

## Implementation design

### A. Separate general string semantics from tokenizer hot-path semantics

The kernel string contract is scalar-based, but the tokenizer is a hot
path and must not implement that contract by repeatedly rescanning UTF-8
from the start.

The wrong migration would be:

- keep the current tokenizer shape;
- replace ASCII indexing with `chars().nth()` everywhere;
- accidentally turn scanning into hidden quadratic work.

The required design is to decode once per source file.

### SourceText

Introduce a source-index structure for compiler hot paths:

```dag
type SourceText {
  text: String
  scalar_to_byte: List<Int>
  scalars: List<String>
}
```

Contract:

- `scalars[i]` is the `i`th Unicode scalar rendered as a one-scalar
  string
- `scalar_to_byte[i]` is the UTF-8 byte offset of scalar `i`
- `scalar_to_byte[count(scalars)] == string_byte_length(text)`

This gives:

- O(1) scalar lookup for tokenizer logic
- O(1) scalar-index to byte-offset conversion for spans
- O(k) slicing by joining a scalar slice

If `List<String>` proves too expensive in practice, the implementation
may switch to `List<Int>` code points or a backend-specific indexed
representation. The contract is the same.

### SourceCursor

Tokenizer state should track scalar position explicitly:

```dag
type SourceCursor {
  scalar_pos: Int
  interp_depth: List<Int>
}
```

All tokenizer movement happens in scalar space.
All emitted token spans convert scalar positions to bytes at the point
of token construction.

### B. Make source helpers explicit

The tokenizer should stop using generic string intrinsics directly on
raw `String` values for source traversal. Instead it should use
source-specific helpers over `SourceText`:

- `source_char_at`
- `source_substring`
- `source_scan_while`
- `source_scan_to_eol`
- `source_skip_horizontal_ws`
- `source_scan_string_end`
- `source_span(start_scalar, end_scalar)`

This avoids conflating two contracts:

- general string semantics for user programs
- indexed source traversal for the compiler itself

### C. Keep evaluator semantics as the language oracle

The v1 evaluator already has the closest thing to a coherent language
contract for these intrinsics. The emitted backends must match it.

That means:

- the Rust runtime shim must stop panicking on valid non-ASCII input;
- the v2 Rust emitter's generated `v2_rt` helpers must stop using byte
  length and byte slicing for Unicode-visible operations;
- the Go emitter must stop lowering string `Index` and `Slice`
  directly to native byte indexing/slicing;
- Python should still route through helpers when needed so parity is
  explicit rather than accidental.

### D. Unify backend string lowering

Backends must not each decide independently when native string syntax is
"close enough." The lowering rule should be structural:

- if an operation is defined in the kernel contract, emit through the
  kernel helper for that backend;
- do not use direct native indexing/slicing for string operations unless
  the backend helper is the thing being called.

This applies to:

- `char_at`
- `string_length`
- `substring`
- expression `Index` when the base type is `String`
- expression `Slice` when the base type is `String`
- tokenizer/scanner helpers

### E. Identifier classification and mangling must each have one owner

There are two separate concerns:

- tokenizer acceptance of source identifiers
- backend-safe emitted naming

They must not be mixed.

Tokenizer ownership:

- one identifier classifier, ideally exposed as a kernel builtin or
  runtime helper based on Unicode XID rules

Emitter ownership:

- one shared mangle algorithm
- no per-emitter ad hoc identifier rewriting

## Migration plan

### Wave 1: Lock the contract in tests

Add cross-layer tests for:

- `string_length`, `char_at`, `substring`, `scan_while`,
  `scan_to_eol`, `skip_horizontal_ws`, `scan_string_end`
- combining marks
- multi-byte BMP characters
- astral scalars
- out-of-bounds `char_at`
- no-normalization equality: `"é" != "é"`

The evaluator remains the initial oracle until the emitted backends are
fully aligned.

### Wave 2: Define indexed source traversal

- add `SourceText` and `SourceCursor`
- rewrite the v2 tokenizer to scan in scalar space and emit byte spans
- keep `SourceSpan` byte-based
- add line/column tests with multi-byte and combining characters

### Wave 3: Unicode identifiers

- replace ASCII `is_ident_start` / `is_ident_char`
- add identifier acceptance tests for XID cases
- add rejection tests for Unicode whitespace and invalid code points
- add a single shared mangle contract for emitters

### Wave 4: Backend parity

- Rust runtime shim: remove ASCII panic path and implement scalar
  semantics
- v2 Rust `v2_rt`: same semantics
- Go runtime helpers: add explicit rune-based helpers and route emitted
  string indexing/slicing through them
- Python: use helpers where direct syntax does not make the contract
  explicit enough

### Wave 5: Self-host and downstream validation

- compile the v2 compiler with Unicode identifiers and Unicode string
  literals in fixtures
- verify Rust, Go, and Python emitted programs agree on observable
  output
- verify diagnostics point to correct spans and columns in UTF-8 source

## Acceptance criteria

This design is complete when all of the following are true:

- valid UTF-8 `.dag` source never crashes an emitted runtime because of
  non-ASCII content
- the evaluator and emitted backends agree on all kernel string
  operations
- token spans are byte-accurate for UTF-8 source
- rendered line/column diagnostics are scalar-accurate for UTF-8 source
- Unicode identifiers parse, resolve, and emit correctly
- emitted backend naming is deterministic and collision-free
- no backend uses direct native byte indexing for language-visible
  string operations

## Tradeoff summary

The cost of this decision is real:

- more explicit semantics
- one new source-index abstraction
- backend runtime work
- identifier-mangling work
- more parity tests

But the upside is better than "Unicode support":

- the language contract becomes explicit instead of accidental
- diagnostics and runtime semantics stop disagreeing
- backend behavior stops depending on host-language string internals
- "no fallback" becomes enforceable because the contract is actually
  written down

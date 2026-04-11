> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1: Structural** (CX gate)
> See also: [cx-design.md](cx-design.md)

# Parser Restructuring: Integer Indexing → List Consumption

## Thesis motivation

The parser uses `ParserState { pos: Int }` to index into a token list.
The CX analyzer cannot prove termination because integers are opaque —
it reports `SameArgumentCall` for every function in the parser SCC.
This accounts for 132 of 421 CX violations (31%).

The tokenizer has the same pattern (`pos: Int` on source characters),
contributing 22 more violations.

**Thesis principle:** all iteration is bounded by construction. The parser
should use the same bounded primitives as all other code. Integer position
advancement is an optimization that trades structural clarity for runtime
efficiency. In a decidable language, structural clarity wins — complexity
proofs are a free consequence.

**The fix:** restructure the parser to consume tokens from a
`List<Token>`. Each recursive call gets a shorter list. This is standard
collection descent — `CollectionShrinkCall` — already provable by
existing CX infrastructure. Zero new concepts. One fewer special case.

| Thesis principle | Current parser | Target parser |
|-----------------|----------------|---------------|
| All iteration bounded by construction | No — `pos: Int` is opaque | Yes — `List<Token>` shrinks structurally |
| Complexity is a conserved quantity | Violated — CX can't see pos bound | Preserved — collection descent is structural |
| No bolt-on analyzers | Would need PositionAdvancement analysis | Standard ProgressRelation proves it |
| Cost of change = 1 | Adding new parser fn requires 0 CX changes | Same — collection descent is automatic |

---

## Design decisions

### D1: Token list as separate parameter, not bundled in state

`tokens: List<Token>` stays a top-level parameter to every parse function.

**Rationale:** The CX analyzer tracks provenance per-binding. If tokens
are bundled in a struct (`ParseState { tokens: ... }`), the analyzer sees
a struct binding, not a list binding — it can't see the collection
shrinking. As a separate parameter, `tokens` has standard
`IteratedSubValue` provenance via `skip(1)`.

### D2: ParseContext for metadata, separate from tokens

```dag
type ParseContext {
  source_index: NewlineIndex?
  intern_table: InternTable
}
```

Replaces `ParserState { pos: Int, source_index: NewlineIndex?, intern_table: InternTable }`.
The `pos: Int` field is deleted. `source_index` is read-only.
`intern_table` is mutated once (module name interning in `parse_module`).

### D3: Helper functions take only tokens

`peek`, `advance`, `skip_newlines`, `eat`, `expect`, `current_span`, and
all `peek_is_*` functions take `tokens: List<Token>` only. They don't need
ParseContext.

```dag
fn peek(tokens: List<Token>) -> Token? { tokens |> first }

fn advance(tokens: List<Token>) -> AdvanceResult {
  match tokens |> first {
    Some { value: t } => AdvanceResult { token: t, tokens: tokens |> skip(1) }
    None => AdvanceResult { token: eof_token, tokens: tokens }
  }
}

fn skip_newlines(tokens: List<Token>) -> List<Token> {
  match tokens |> first {
    Some { value: t } =>
      if t.shape == ShNewline { skip_newlines(tokens: tokens |> skip(1)) }
      else { tokens }
    None => tokens
  }
}

fn expect(tokens: List<Token>, expected: ExpectedToken) -> ExpectResult {
  match tokens |> first {
    Some { value: t } =>
      if token_matches_expected(token: t, expected: expected) {
        ExpectResult { token: t, tokens: tokens |> skip(1), err: none }
      } else {
        ExpectResult { token: t, tokens: tokens, err: Some { ... } }
      }
    None => ExpectResult { ... }
  }
}

fn current_span(tokens: List<Token>) -> SourceSpan {
  match tokens |> first {
    Some { value: t } => t.span
    None => make_span(start: 0, end: 0)
  }
}
```

### D4: Result types carry remaining tokens + ctx

All ~55 result types change from:
```dag
type TypeResult { type_expr: Node, state: ParserState, err: ErrorNode? }
```
to:
```dag
type TypeResult { type_expr: Node, tokens: List<Token>, ctx: ParseContext, err: ErrorNode? }
```

Helper results carry only `tokens`:
```dag
type AdvanceResult { token: Token, tokens: List<Token> }
type ExpectResult { token: Token, tokens: List<Token>, err: ErrorNode? }
```

### D5: Callers pass remaining tokens

The fundamental call pattern changes:

```dag
// Before: every call receives the FULL token list; state.pos tracks position
let r = parse_type_expr(tokens: tokens, state: state)
let s2 = skip_newlines(tokens: tokens, state: r.state)
parse_callable(tokens: tokens, state: s2)

// After: every call receives REMAINING tokens; list shrinks at each step
let r = parse_type_expr(tokens: tokens, ctx: ctx)
let s2 = skip_newlines(tokens: r.tokens)
parse_callable(tokens: s2, ctx: r.ctx)
```

### D6: Lookahead uses offset into remaining list

Most lookahead is 1-token (`tokens |> first`). Limited cases use offset:

```dag
// 2-token lookahead:
tokens |> skip(1) |> first

// scan_for_fat_arrow_after_braces (unbounded read-only scan):
fn scan_braces_depth(tokens: List<Token>, offset: Int, depth: Int) -> Bool {
  match tokens |> skip(offset) |> first {
    Some { value: t } =>
      if is_lbrace(t) { scan_braces_depth(tokens: tokens, offset: offset + 1, depth: depth + 1) }
      else if is_rbrace(t) { scan_braces_depth(tokens: tokens, offset: offset + 1, depth: depth - 1) }
      else { scan_braces_depth(tokens: tokens, offset: offset + 1, depth: depth) }
    None => false
  }
}
```

`scan_braces_depth` doesn't consume tokens — it's a read-only scan with
offset. Bounded by `|tokens|` (arithmetic descent on offset toward list
length).

### D7: Tokenizer restructured in same PR

The tokenizer (`01_tokenize.dag`) uses the same pattern: `pos: Int` on
`source_chars: List<Int>`. Restructure to pass remaining chars:

```dag
// Before:
fn source_skip_ws(source: SourceRef, start: Int) -> Int {
  if start >= source_len(source) { start }
  else { let ch = source.source_chars[start]; ... source_skip_ws(source, start + 1) }
}

// After:
fn source_skip_ws(chars: List<Int>) -> List<Int> {
  match chars |> first {
    Some { value: ch } =>
      if ch == code_point(" ") || ch == code_point("\t") {
        source_skip_ws(chars: chars |> skip(1))
      } else { chars }
    None => chars
  }
}
```

### D8: SubValueRelation renamed to ProgressRelation

The type models all forms of progress toward termination, not just
structural sub-values. Rename in the same PR:

```dag
type ProgressRelation
  = StrictSubValue { field: InductiveField, factor: ShrinkFactor }
  | IteratedSubValue { field: InductiveField }
  | ArithmeticDescent { param: String, factor: ShrinkFactor }
  | PreservedValue
  | ProgressUnknown  // was SubValueUnknown
```

Rename functions: `meet_sub_value` -> `meet_progress`,
`compose_sub_value` -> `compose_progress`,
`sub_value_to_evidence` -> `progress_to_evidence`,
`sub_value_to_lowering_target` -> `progress_to_lowering_target`.

---

## Target structure

### Parser call flow

```
tokenize(source, file)
  -> List<Token>
       |
       v
parse(tokens, source_index)
  -> ParseResult { module, err }
       |
       +-- creates ParseContext { source_index, intern_table: empty() }
       +-- parse_module(tokens, ctx)
       |     +-- advance(tokens) -> { token, remaining }
       |     +-- skip_newlines(remaining) -> remaining'
       |     +-- parse_imports(remaining', ctx)
       |     |     \-- returns { imports, tokens: remaining'', ctx }
       |     +-- parse_items(remaining'', ctx)
       |     |     \-- ... recursive descent, tokens shrinks at each step ...
       |     \-- returns { module, tokens: remaining_final, ctx }
       \-- extracts module from result
```

### CX proof structure

```
parse_type_expr(tokens: List<Token>, ctx: ParseContext) -> TypeResult
  |
  +-- let s = skip_newlines(tokens: tokens)       // s: shorter List<Token>
  +-- let adv = advance(tokens: s)                // adv.tokens: skip(1) of s
  +-- parse_callable(tokens: adv.tokens, ctx: ctx)  // RECURSIVE: shorter list
  |     binding provenance: adv.tokens has IteratedSubValue of tokens
  |     -> CollectionShrinkCall -> CX proves descent
  |
  \-- SCC analysis: every cycle through parse_* consumes >=1 token
      -> bounded by |tokens| -> terminates
```

---

## Scope

| Component | Files | Lines affected | Nature |
|-----------|-------|---------------|--------|
| Parser restructure | 02_parse.dag | ~4500 | Mechanical: sig+body changes |
| Tokenizer restructure | 01_tokenize.dag | ~600 | Same pattern, smaller |
| Compile boundary | compile.dag | ~10 | Wire new parse interface |
| ProgressRelation rename | induction.dag, computation.dag, 04_env.dag, 04_infer.dag, complexity.dag | ~30 sites | Rename |
| Result types | 02_parse.dag (top) | ~55 types | Add tokens field, rename state->ctx |
| Stage0 regen | stage0/ | generated | Regen from .dag changes |

### Key numbers

- ~198 function signatures to update
- ~55 result types to update
- ~500+ call sites to update (pass remaining tokens)
- ~15 helper functions rewritten (peek, advance, expect, etc.)
- ~30 rename sites (ProgressRelation)

### Not in this PR

- Body-inferred return contracts — separate follow-up PR
- Arithmetic classification refinement (`(n-d)/10`) — separate PR
- Graph DFS worklist pattern (10 violations) — needs language primitive

---

## Expected CX impact

| Before | After | Delta | Source |
|--------|-------|-------|--------|
| 421 | ~267 | -154 | Parser SCC (132) + tokenizer (22) dissolved by construction |

---

## Verification

1. Hand-written tests pass: `cargo test --workspace --exclude v2-compiler-tests`
2. Lint clean: `cargo clippy --all-targets -- -D warnings`
3. Compiler tests pass: `cargo test -p v2-compiler-tests`
4. Full DSL compiles: `full_dsl_compiles -- --ignored`
5. Diagnostic ratchet: `strict_compile_diagnostic_count -- --ignored` -> expect ~267
6. L1 gate: `scripts/l1-ratchet.sh --check` -> GREEN
7. Stage0 freshness: `scripts/check-stage0-freshness.sh` -> GREEN
8. Self-compile fixed-point: regenerated binary produces identical output

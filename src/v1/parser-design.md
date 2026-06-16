> Part of: [THESIS.md](../../THESIS.md) > [ROADMAP.md](../../ROADMAP.md) > **Tier 1: Structural** (CX gate)
> See also: [cx-design.md](cx-design.md)

# Parser Restructuring: Integer Indexing → List Consumption

## Thesis motivation

The parser uses `ParserState { pos: Int }` to index into a token list.
The CX analyzer cannot prove termination because integers are opaque —
it reports `SameArgumentCall` for every function in the parser SCC.
This accounts for 132 of the violations measured locally (421; ratchet
in bootstrap.rs is 424).

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

**This is an interim constraint shaped by the analyzer's current
limitation**, not the thesis end state. The thesis says dimensions
should be generic, computed at binding sites, and carried through the
IR — which means the analyzer should eventually see descent through
struct fields. Under the Root-Cause Depth invariant, the deeper fix is
field-level provenance tracking in the analyzer, which would make D1
unnecessary. D1 is pragmatic: it gets provable descent now without
waiting for analyzer improvements.

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

### D3: Minimum information — inspection vs consumption

**Principle:** each helper receives the absolute minimum information it
needs. Inspection helpers need a `Token?`. Consumption helpers need a
`List<Token>`. No helper receives more than it uses.

**Inspection helpers** take `Token?` (or `Token`), not `List<Token>`.
They check properties of a single token. The caller extracts the token
from the correct remaining list — no ambiguity possible.

```dag
// All ~30 peek_is_* functions become token inspectors:
fn is_newline_token(tok: Token?) -> Bool {
  match tok { Some { value: t } => match t.shape { ShNewline => true  _ => false }  None => false }
}
fn is_keyword_token(tok: Token?, kw: String) -> Bool {
  match tok { Some { value: t } => match t.shape { ShKeyword => t.text == kw  _ => false }  None => false }
}
fn is_ident_token(tok: Token?) -> Bool {
  match tok { Some { value: t } => match t.shape { ShIdent => true  _ => false }  None => false }
}
fn is_eof_token(tok: Token?) -> Bool {
  match tok { Some { value: t } => match t.shape { ShEof => true  _ => false }  None => true }
}
fn token_span(tok: Token?) -> SourceSpan {
  match tok { Some { value: t } => t.span  None => make_span(start: 0, end: 0) }
}
fn token_shape(tok: Token?) -> TokenShape? {
  match tok { Some { value: t } => Some { value: t.shape }  None => none }
}
```

`peek` is eliminated — `tokens |> first` IS peek. No wrapper needed.

**Consumption helpers** take `List<Token>` and return a new `List<Token>`:

```dag
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

fn expect(tokens: List<Token>, expected: ExpectedToken) -> TokenResult {
  match tokens |> first {
    Some { value: t } =>
      if token_matches_expected(token: t, expected: expected) {
        TokenResult { token: t, tokens: tokens |> skip(1), err: none }
      } else {
        TokenResult { token: t, tokens: tokens, err: Some { ... } }
      }
    None => TokenResult { ... }
  }
}

fn eat(tokens: List<Token>, expected: ExpectedToken) -> EatResult {
  match tokens |> first {
    Some { value: t } =>
      if token_matches_expected(token: t, expected: expected) {
        EatResult { consumed: true, tokens: tokens |> skip(1), token: Some { value: t } }
      } else {
        EatResult { consumed: false, tokens: tokens, token: none }
      }
    None => EatResult { consumed: false, tokens: tokens, token: none }
  }
}
```

**Why this dissolves the token threading problem:** In the old design,
helpers took `List<Token>` and the question "which list?" caused ~100+
bugs in the bulk transform. With inspection helpers taking `Token?`,
the caller does `let tok = tokens |> first` once and passes `tok` to
all inspectors. No list reference passes through inspection helpers,
so no ambiguity is possible.

**`current_span` becomes `token_span`** — takes `Token?`, not a list.
`peek` is eliminated — `tokens |> first` IS peek.

### D4: Result types carry remaining tokens + ctx

All ~55 result types change from:
```dag
type TypeResult { type_expr: Node, state: ParserState, err: ErrorNode? }
```
to:
```dag
type TypeResult { type_expr: Node, tokens: List<Token>, ctx: ParseContext, err: ErrorNode? }
```

**Why ParseContext is minimal:** `source_index` is read by most parse
functions that construct AST nodes (for span attribution via
`source_index`). `intern_table` is read/written once in `parse_module`.
Both are genuinely shared across parser functions — this is not a
convenience bundle. A function that only inspects tokens (helpers) does
NOT take ctx; a function that builds AST nodes (parse functions) does.

**Bespoke result types are a deliberate deferral, not the end state.**
The ~55 result types (TypeResult, ExprResult, etc.) are a known modeling
debt flagged by MODELING.md (M6: one generic result pattern, not N
bespoke types). The structural parser rewrite is the natural moment to
unify them, but doing both in one PR conflates the position fix with
the result-type fix. This design deliberately scopes to position only;
result-type unification is a first-class follow-up requirement.

Helper results (no ctx — callers use `ctx` from their own scope):
```dag
type AdvanceResult { token: Token, tokens: List<Token> }
type EatResult { consumed: Bool, tokens: List<Token>, token: Token? }
type TokenResult { token: Token, tokens: List<Token>, err: ErrorNode? }
```

### D5: Shadow `tokens` at each step — one variable, always current

The fundamental call pattern changes. Instead of a separate `s` variable
for advanced state, **shadow `tokens` at each consumption step**:

```dag
// Before: tokens is FULL list, state.pos tracks position, s rebinds state
let s = skip_newlines(tokens: tokens, state: state)
let r = expect(tokens: tokens, state: s, expected: ExpectKeyword { text: "module" })
if has_err(err: r.err) { return ModuleResult { ..., state: r.state, err: r.err } }
let s = r.state
let r = parse_dotted_ident(tokens: tokens, state: s)

// After: tokens IS the remaining list, shadowed at each step
let tokens = skip_newlines(tokens: tokens)
let tok = tokens |> first                           // inspect current token
let r = expect(tokens: tokens, expected: ExpectKeyword { text: "module" })
if has_err(err: r.err) { return ModuleResult { ..., tokens: r.tokens, ctx: ctx, err: r.err } }
let tokens = r.tokens                               // shadow: tokens is now after expect
let r = parse_dotted_ident(tokens: tokens, ctx: ctx)
let tokens = r.tokens                               // shadow: tokens is now after dotted_ident
```

**Key invariant:** `tokens` always means "current remaining." No `s`,
`s2`, or `s3` variables. Inspection uses `tokens |> first` (unambiguous).
Error returns use `ctx` from scope (not `r.ctx`), since helper results
don't carry ctx.

**Why this works:** In the old code, `tokens: tokens` in every call was
correct because `tokens` was the unchanging full list. In the new code,
`tokens: tokens` is STILL correct at every call — because `tokens` is
shadowed to always be the current remaining list.

### D6: Lookahead is suffix-based, not offset-based

Most lookahead is 1-token (`tokens |> first`). For 2-token lookahead,
`tokens |> skip(1) |> first`.

For unbounded lookahead (`scan_for_fat_arrow_after_braces`), the scanner
recurses on the remaining suffix — not an integer offset. This keeps
the "no integer opacity" invariant consistent throughout:

```dag
// scan_for_fat_arrow_after_braces: recurse on suffix, not offset
fn scan_for_fat_arrow_after_braces(tokens: List<Token>, start_skip: Int) -> Bool {
  scan_braces_depth(remaining: tokens |> skip(start_skip), depth: 1)
}

fn scan_braces_depth(remaining: List<Token>, depth: Int) -> Bool {
  if depth <= 0 {
    // matched all braces — check for fat arrow
    match remaining |> first {
      Some { value: t } => is_fat_arrow_shape(shape: t.shape)
      None => false
    }
  } else {
    match remaining |> first {
      Some { value: t } =>
        if is_lbrace_shape(shape: t.shape) {
          scan_braces_depth(remaining: remaining |> skip(1), depth: depth + 1)
        } else if is_rbrace_shape(shape: t.shape) {
          scan_braces_depth(remaining: remaining |> skip(1), depth: depth - 1)
        } else {
          scan_braces_depth(remaining: remaining |> skip(1), depth: depth)
        }
      None => false
    }
  }
}
```

`remaining |> skip(1)` is standard collection consumption — bounded by
`|remaining|`, provable by construction. No integer offset reasoning.

### D7: Tokenizer deferred (separate PR)

The tokenizer (`01_tokenize.dag`) has the same integer-opacity pattern
(`pos: Int` on `source_chars: List<Int>`), accounting for 22 violations.
However, the tokenizer **constructs spans from byte positions**
(`make_span(start: pos, end: pos + len)`). Restructuring to list
consumption requires designing how to track byte offsets for span
construction without reintroducing integer position tracking. This is
a separate design problem deferred to a follow-up PR.

### D8: SubValueRelation → ProgressRelation deferred (separate PR)

The rename is conceptually clean but touches files in Streams A and C
(induction.dag, computation.dag, 04_env.dag, 04_infer.dag,
complexity.dag). To maintain zero file overlap between streams, the
rename is a separate PR sequenced after any in-flight Stream A work.

The type models all forms of progress toward termination, not just
structural sub-values. Target:

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
  +-- let tokens = skip_newlines(tokens: tokens)   // shadow: shorter List<Token>
  +-- let tok = tokens |> first                    // inspect: Token?
  +-- let adv = advance(tokens: tokens)            // consume: adv.tokens = skip(1)
  +-- parse_callable(tokens: adv.tokens, ctx: ctx) // RECURSIVE: shorter list
  |     binding provenance: adv.tokens has IteratedSubValue of tokens
  |     -> CollectionShrinkCall -> CX proves descent
  |
  \-- SCC analysis: every cycle through parse_* consumes >=1 token
      -> bounded by |tokens| -> terminates
```

### Typical parse function (after)

```dag
fn parse_module(tokens: List<Token>, ctx: ParseContext) -> ModuleResult {
  let tokens = skip_newlines(tokens: tokens)
  let tok = tokens |> first
  let start_span = token_span(tok: tok)

  let r = expect(tokens: tokens, expected: ExpectKeyword { text: "module" })
  if has_err(err: r.err) { return ModuleResult { ..., tokens: r.tokens, ctx: ctx, err: r.err } }
  let tokens = r.tokens

  let r = parse_dotted_ident(tokens: tokens, ctx: ctx)
  if has_err(err: r.err) { return ModuleResult { ..., tokens: r.tokens, ctx: r.ctx, err: r.err } }
  let mod_name = r.name
  let tokens = skip_newlines(tokens: r.tokens)

  let r = parse_imports(tokens: tokens, ctx: ctx)
  if has_err(err: r.err) { return ModuleResult { ..., tokens: r.tokens, ctx: r.ctx, err: r.err } }
  let imports = r.imports
  let tokens = r.tokens

  let r = parse_items(tokens: tokens, ctx: ctx)
  // ... same pattern ...
  ModuleResult { module: mod, tokens: r.tokens, ctx: ctx, err: none }
}
```

Note: `tokens: tokens` in every call is ALWAYS correct because `tokens`
is shadowed at each step. Error returns use `ctx` from scope (not `r.ctx`)
for helper results. Parse results DO have ctx.

---

## Scope

| Component | Files | Lines affected | Nature |
|-----------|-------|---------------|--------|
| Parser restructure | 02_parse.dag | ~4500 | Function-by-function rewrite |
| Compile boundary | compile.dag | ~10 | Wire new parse interface |
| Stage0 regen | stage0/ | generated | Regen from .dag changes |

### Not in this PR (separate PRs)

- Tokenizer restructuring — deferred (span construction needs design; only 22 violations)
- ProgressRelation rename — separate PR (overlaps Stream A files)
- Body-inferred return contracts — separate follow-up PR
- Arithmetic classification refinement (`(n-d)/10`) — separate PR
- Graph DFS worklist pattern (10 violations) — needs language primitive

### Implementation order

**Phase 0: Refactor helpers to minimum information (DONE)**
Rewrite ~25 `peek_is_*` functions to `tok_is_*(tok: Token?)`.
This is interface narrowing only — `ParserState { pos }` still
exists, `peek(tokens, state)` still exists, and the root cause
(integer-indexed position) is not yet fixed. Phase 0 reduces
the surface area for the structural fix (Phases 1-2) by ensuring
inspection helpers can't reference the wrong token list.

**Known deferrals in Phase 0:**
- `tok_keyword_to_name` / `tok_is_keyword_name` preserve the same
  keyword-name authority that was in `keyword_to_name`. This is the
  same logic in a narrower wrapper, not a structural dissolution.
  The structural fix: derive keyword-name logic from the tokenizer's
  keyword table (SyntaxSpec) rather than maintaining a parser-side copy.
  Deferred — the authority question is orthogonal to the position fix.
- 5 lookahead helpers (`peek_is_*_after_ident`, `peek_is_*_at`) remain
  with `(tokens, state)` signatures — they genuinely need multi-token
  access. These dissolve in Phase 2 when `tokens` becomes the remaining
  list and lookahead uses `tokens |> skip(N) |> first`.

**Phase 1: Types + consumption helpers**
Change ParserState → ParseContext (delete `pos`). Update ~55 result
types. Rewrite advance/skip_newlines/expect/eat. Compile — expect
many errors from the ~198 functions that still reference old types.

**Phase 2: Parse functions, function by function**
Transform each parse function top-to-bottom. For each:
1. Change signature: `state: ParserState` → `ctx: ParseContext`
2. Shadow `tokens` at each consumption step
3. Use `tokens |> first` for inspection, pass `Token?` to inspectors
4. Error returns: `ctx` from scope for helper results, `r.ctx` for
   parse results
Compile every ~10 functions to catch errors early.

**Phase 3: compile.dag boundary**
Update the tokenize→parse call site.

**Phase 4: Regen + test**
Regen stage0, build, run full test suite.

### Lessons from first attempt

1. **Never bulk-sed call-site threading.** Token threading requires
   understanding local dataflow. Transform function by function.
2. **Helper results don't have ctx.** Error returns must use `ctx`
   from scope, not `r.ctx`, when `r` is a TokenResult/EatResult.
3. **Phase 0 (helper refactoring) is the key insight.** By making
   inspection helpers take `Token?` FIRST, the token threading
   problem disappears by construction — inspectors can't reference
   the wrong list because they don't take a list.

---

## Expected CX impact

| Before | After | Delta | Source |
|--------|-------|-------|--------|
| 421 | ~289 | -132 | Parser SCC dissolved by construction |

Note: tokenizer (22 violations) deferred to separate PR.

---

## Verification

1. Hand-written tests pass: `cargo test --workspace --exclude v1-compiler-tests`
2. Lint clean: `cargo clippy --all-targets -- -D warnings`
3. Compiler tests pass: `cargo test -p v1-compiler-tests`
4. Full DSL compiles: `full_dsl_compiles -- --ignored`
5. Diagnostic ratchet: `strict_compile_diagnostic_count -- --ignored` -> expect ~267
6. L1 gate: `scripts/l1-ratchet.sh --check` -> GREEN
7. Stage0 freshness: `scripts/check-stage0-freshness.sh` -> GREEN
8. Self-compile fixed-point: regenerated binary produces identical output

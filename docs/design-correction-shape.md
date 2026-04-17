> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 (all stages), Lane 3 Stage 3b, Lane 4 Stages 4b/4c

# Design DB-1 — `Diagnostic.fix: List<Correction>` shape

**Design blocker:** DB-1
**Consumers:** Lane 2 (workflow idempotency lens, symbolic cost lens, parallelism lens); Lane 3 Stage 3b (diagnostics-as-corrections); Lane 4 Stages 4b/4c (side effects, space bounds — same Dimension diagnostic shape)
**Status:** Design ready for implementer review.

---

## Problem

Today `Diagnostic` carries `kind: DiagnosticKind` + `span: SourceSpan` + `message: String`. Errors name the problem but don't tell the user what to type to fix it.

THESIS.md §"Diagnostics as corrections" and `docs/error-examples.md` show the target shape:

```
ERROR at line 2: field `c` does not exist on Point

  fn read(point: Point) -> Int = point.c
                                       ^

Available fields: a, b
FIX: did you mean `point.a` or `point.b`?
```

`FIX` lines are structured data (a *Correction* type), not free-text appended to messages. This lets IDE integrations apply the fix automatically, and it lets the compiler emit multiple alternative corrections.

Lane 2's property lenses will emit diagnostics when workflows break. If Lane 2 ships without knowing the Correction shape, every diagnostic gets retrofit in Lane 3b. That's avoidable.

---

## Design

### Core types

Add to `src/v3/std/diagnostics.dag` (new, or extend existing):

```dag
type Correction {
  description: String       // human-facing label for the fix option
  span: SourceSpan          // what the fix replaces
  new_source: String        // literal replacement code
  target_language: TargetLanguageId?  // None = language-agnostic fix
}
```

Extend `Diagnostic`:

```dag
type Diagnostic {
  kind: DiagnosticKind
  span: SourceSpan
  message: String
  fixes: List<Correction>   // NEW: may be empty; never null
}
```

**Field discipline:**
- `fixes: List<Correction>` not `fix: Correction?` — multiple alternative fixes are common (the `point.c` → `point.a OR point.b` case)
- Empty list is legal — not every diagnostic has a mechanical fix. Type mismatch on a user's domain value may need human judgment.
- `description` is short (≤60 chars), user-facing. e.g., "did you mean `point.a`?" or "wrap in Some"
- `new_source` is the literal text that replaces `span`'s range. No markup, no placeholders.
- `target_language` is `None` for source-level fixes (most common). `Some(TargetLanguageId)` for target-specific fixes (e.g., "add `async` keyword" in async Rust mode).

### Correction style per target

Different targets format corrections differently. Rust fixes need certain indentation rules; Python differs.

Add to each target spec (extends Lane 1 Stage 1c's `CleanEmissionContract`):

```dag
// in spec/rust.dag
data rust_correction_style: CorrectionStyle = {
  indent_unit: "    "              // 4 spaces
  line_ending: "\n"
  string_quote: "\""
  trailing_semicolon: true
}

// in spec/python.dag
data python_correction_style: CorrectionStyle = {
  indent_unit: "    "              // 4 spaces
  line_ending: "\n"
  string_quote: "\""
  trailing_semicolon: false
}
```

`CorrectionStyle` type lives in `std/clean_emission.dag` (see DB-4):

```dag
type CorrectionStyle {
  indent_unit: String
  line_ending: String
  string_quote: String
  trailing_semicolon: Bool
}
```

Diagnostic generators producing target-specific fixes consult this style. Source-level fixes (most diagnostics) don't need it.

### Fix generators per diagnostic kind

Each `DiagnosticKind` variant has an associated fix generator. Not every kind needs fixes; those that don't emit an empty list.

Starter catalog (more added as lanes progress):

| DiagnosticKind | Fix generator | Number of fixes emitted |
|---|---|---|
| `FieldNotFound { type_name, bad_field, available }` | "did you mean `{available[i]}`?" for each field | len(available), bounded to 5 |
| `NonExhaustiveMatch { scrutinee, missing_variants }` | Insert arm `{variant} => todo()` for each missing | len(missing) (or 1 bundled) |
| `TypeMismatch { expected, got, expr_span }` | Option A: change annotation to `got`; Option B: change value to match `expected` | 2 |
| `UnresolvedIdentifier { name, suggestions }` | "did you mean `{suggestions[i]}`?" | len(suggestions), bounded |
| `UnusedParameter { param_name }` | Prefix `_` to param (rename) OR remove param | 2 |
| `IdempotencyBreak { op_name, reason }` | Wrap op in idempotent adapter (if possible) OR remove from retry context | 1–2 |
| `NonTerminatingRecursion { fn_name }` | Add `where` clause with descent evidence; add bound parameter | 2 |
| `WorkflowBreaksIdempotency { breaking_op }` | Remove op from workflow; replace with idempotent equivalent | 0–2 |

A fix generator is a pure function:

```dag
fn generate_fixes(d: Dag, diag: Diagnostic) -> List<Correction>
```

Dispatched by `diag.kind` variant. Implementation per variant lives in `src/v3/lenses/corrections.dag` (new).

### Example end-to-end

User writes:
```v3
fn read(p: Point) -> Int = p.c
```

Compiler emits:
```
Diagnostic {
  kind: FieldNotFound {
    type_name: "Point",
    bad_field: "c",
    available: ["a", "b"]
  }
  span: <p.c span>
  message: "field `c` does not exist on Point"
  fixes: [
    Correction {
      description: "did you mean `p.a`?"
      span: <p.c span>
      new_source: "p.a"
      target_language: None
    }
    Correction {
      description: "did you mean `p.b`?"
      span: <p.c span>
      new_source: "p.b"
      target_language: None
    }
  ]
}
```

Display output:
```
ERROR at line 1, col 27: field `c` does not exist on Point

  fn read(p: Point) -> Int = p.c
                              ^

Available fields: a, b
FIX (option 1): did you mean `p.a`?
FIX (option 2): did you mean `p.b`?
```

---

## Rationale

**Why `List<Correction>` not `Option<Correction>`?** Multiple valid fixes are common. `point.c` could be `point.a` or `point.b`. Type mismatch has two symmetric fixes (change the annotation vs change the value). Forcing a single fix means picking arbitrarily or synthesizing a "fix = null, see message" case.

**Why `new_source: String` not `Edit { insertions, deletions }`?** The replacement text IS the fix. Span + new text = minimal information. Edits are reconstructable from span+new: the editor sees "replace span with new_source" and does a simple substitution. Modeling granular edits is premature (the IDE will chunk them anyway).

**Why include `target_language`?** Most fixes are source-level — user edits their `.dag`. But some diagnostics only make sense against a target (e.g., a rustfmt-layer issue that surfaces only in Rust emission). Those need to identify the target. Default `None` keeps the common case simple.

**Why separate `CorrectionStyle` per target?** Rust's 4-space indent, Python's PEP 8, Go's tabs — corrections emitted INTO target-language source must respect the target's style. But the style is target-declared, not fix-per-fix. Reuses the `CleanEmissionContract` dispatch pattern from Lane 1 Stage 1c.

**Why `span` on Correction (redundant with Diagnostic.span)?** The diagnostic's span points at the error site; the correction's span may be broader or narrower. For non-exhaustive match, diag span is the `match` keyword; correction span is the closing brace position where the new arm inserts. They're genuinely different.

---

## Rejected alternatives

**Inline fix text in `message`** — free-text strings don't compose. IDE integration impossible. Rejected.

**`Fix` as an enum (`Rename`, `Insert`, `Replace`, `Remove`)** — too taxonomy-heavy. All fixes reduce to "at span X, new text is Y." Rejected; would need to be reassembled on the way out anyway.

**One `Correction` per diagnostic, add separate `alternatives: List<Correction>`** — implies hierarchy the compiler doesn't actually have. `point.a` is not more correct than `point.b`. Rejected.

**Per-target Correction type (`RustCorrection`, `PythonCorrection`, etc.)** — violates single-authority. The Correction type IS language-agnostic; the style IS per-target. Clean separation. Rejected (the hyphenated design above).

---

## Implementation notes

- **Empty `fixes: []`** is the default for any diagnostic lacking a generator. Implementers adding new `DiagnosticKind` variants MUST either add a fix generator or document why no fix is possible.
- **Fix generation is pure** — no I/O, no mutation. Takes `(Dag, Diagnostic)`, returns `List<Correction>`.
- **Generated fixes MUST parse** — Lane 3 Stage 3b adds a test gate: every emitted fix's `new_source` is parsed back through `compile_to_dag`; if any fix doesn't parse, the fix generator is broken.
- **Generated fixes MUST NOT break the program** — stronger test gate (optional, Lane 3b extension): applying a fix should at minimum reduce the diagnostic count, not increase it.
- **Ordering matters** — fixes are shown to users in list order. Most-likely first. `point.a OR point.b` ordering can be alphabetic; `change annotation OR change value` should order by which the user probably wants (heuristic: keep-annotation-change-value first for explicit annotations, flip otherwise).

---

## Associations

- **Lane 3 Stage 3b** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — this design is the type Lane 3b implements
- **Lane 2 Stages 2b, 2d, 2e** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — idempotency / symbolic cost / parallelism lenses emit diagnostics with fixes matching this shape
- **Lane 4 Stages 4b, 4c** ([lane4-completion.md](./lane4-completion.md)) — side effects / space bounds lenses likewise
- **Lane 1 Stage 1c** ([phase1-lane2-clean-emission-invariant.md](./phase1-lane2-clean-emission-invariant.md)) — `CorrectionStyle` field in `CleanEmissionContract`
- **Update `src/v3/std/diagnostics.dag`** — add `Correction` and extend `Diagnostic`
- **Thesis anchor** — THESIS.md §"Diagnostics as corrections"; `docs/error-examples.md` is the aspiration source

---

## Acceptance (Lane 3 Stage 3b owns)

- [ ] `Correction` type declared in `std/diagnostics.dag` with receipt 🟢
- [ ] `Diagnostic.fixes: List<Correction>` field added (empty list = "no mechanical fix")
- [ ] `CorrectionStyle` type declared in `std/clean_emission.dag`; `rust_correction_style`, `python_correction_style`, `go_correction_style` data items in each target spec
- [ ] Fix generators for the 8 starter diagnostic kinds (table above) live in `lenses/corrections.dag` and pass unit tests
- [ ] Every new `DiagnosticKind` variant added after this design either has a fix generator OR documents why not (code comment at the variant declaration)
- [ ] Gate test: every emitted fix's `new_source` parses via `compile_to_dag`

---

## Open questions

1. **Should Corrections carry confidence scores?** For ambiguous cases (multiple fields within edit distance 1), ranking matters. For now: skip scores, rely on list ordering. If IDE integration needs scores, add in a follow-up.
2. **Can fixes reference other parts of the program?** E.g., "create a missing type `Point` at top of file to match this usage." Cross-span fixes are a stretch goal. For now: single-span fixes only.
3. **Should fixes be target-conditional?** E.g., different fix in Rust vs Python emission context. Target language is captured in `target_language` field but dispatch logic TBD. For now: None-default is fine; target-specific fixes added only when a concrete case requires them.

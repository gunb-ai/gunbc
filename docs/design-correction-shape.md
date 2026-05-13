> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 2 (all stages), Lane 3 Stage 3b, Lane 4 Stages 4b/4c

# Design DB-1 — mandatory `Diagnostic.correction: Correction` shape

**Design blocker:** DB-1
**Consumers:** Lane 2 (workflow idempotency lens, symbolic cost lens, parallelism lens); Lane 3 Stage 3b (diagnostics-as-corrections); Lane 4 Stages 4b/4c (side effects, space bounds — same Dimension diagnostic shape)
**Status:** Updated by R3 Gap 9 row #106. The original DB-1
`fixes: List<Correction>` sketch is superseded for the authoritative
diagnostic carrier by a mandatory sum:
`Correction = LiveCorrection(CorrectionWitness) | DeferredCorrection`.
Live multi-edit repairs are bundled into one source-level witness when
applying a partial fix would leave the same diagnostic class failing;
true alternative user choices remain future correction-workflow
surface, not the mandatory carrier.

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

`FIX` lines are structured data (a *Correction* type), not free-text appended to messages. R3 row #106 tightened this from a list that could mean "no fix" into a single mandatory carrier: apply-able source rewrites are `LiveCorrection`, and residuals are explicit `DeferredCorrection` values with a retirement plan.

Lane 2's property lenses will emit diagnostics when workflows break. If Lane 2 ships without knowing the Correction shape, every diagnostic gets retrofit in Lane 3b. That's avoidable.

---

## Design

### Core types

Add to `src/v3/std/diagnostics.dag` (new, or extend existing):

```dag
type CorrectionWitness {
  description: String       // human-facing label for the fix option
  span: SourceSpan          // what the fix replaces (always in .dag source)
  new_source: String        // literal replacement code (always .dag syntax)
}

type RetirementPlan {
  owner: String
  exit_condition: String
}

type Correction
  = LiveCorrection { witness: CorrectionWitness }
  | DeferredCorrection { reason: String, retirement_plan: RetirementPlan }
```

**Source-only by construction.** A `Correction` edits `.dag` source. There is no `target_language` field and no target-conditioned variant — if a fix were target-specific (e.g., "add `async` keyword to emitted Rust"), it would edit a derived target file, not `.dag` source, and therefore belongs in a different carrier (see "Future extension: TargetCorrection" below). Mixing the two domains inside one `Correction` admits a state the type cannot locate or own cleanly; the illegal-states-unrepresentable discipline forbids that shape.

Extend `Diagnostic`:

```dag
type Diagnostic {
  kind: DiagnosticKind
  span: SourceSpan
  message: String
  correction: Correction    // mandatory: live witness or explicit deferral
}
```

**Field discipline:**
- `correction: Correction` not `fix: Correction?` or `fixes: List<Correction>` — absence is represented by a named `DeferredCorrection`, not by `None` or an empty list.
- Multi-edit repairs that are needed together are bundled into one `LiveCorrection` witness. For example, a non-exhaustive match missing two constructors inserts both arms in one edit so the roundtrip can reach zero diagnostics.
- Ambiguous human-choice alternatives are not smuggled in as an empty or partial fix. Until a richer choice workflow is modeled, diagnostics with no canonical source rewrite use `DeferredCorrection` and name their retirement plan.
- `description` is short (≤60 chars), user-facing. e.g., "did you mean `point.a`?" or "wrap in Some"
- `new_source` is the literal text that replaces `span`'s range. No markup, no placeholders.
- `span` always locates within the user's `.dag` source. Target-file spans do not appear in this carrier.

### Future extension: TargetCorrection (out of scope for DB-1)

If a future consumer needs target-language fixes (e.g., "the emitted Rust has a lint; here's the Rust fix"), introduce a separate carrier:

```dag
// Hypothetical — NOT in scope for DB-1
type TargetCorrection {
  description: String
  target: TargetLanguageId        // Rust / Go / Python / ...
  target_span: TargetFileSpan     // DIFFERENT from SourceSpan;
                                   // identifies a location in the
                                   // emitted file, not the .dag source
  new_target_source: String       // target-language syntax, not .dag
}
```

Location model is genuinely different (emitted file path, line/col in that file, regenerated-on-every-compile identity). Replacement text is in the target language. Deferred until concrete use case.

### Correction style per target

The per-target style covers how the IDE surfaces the source-level fix (e.g., rustfmt re-run after applying the fix for smart reformat) — NOT target-language syntax for the fix. Fixes themselves are always `.dag`-source edits.

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

Each `DiagnosticKind` variant has an associated correction generator.
Generators return one mandatory `Correction`: a `LiveCorrection` when
they can produce an apply-able source rewrite, otherwise a
`DeferredCorrection` naming the retirement plan.

Starter catalog (more added as lanes progress):

| DiagnosticKind | Correction generator | Carrier shape |
|---|---|---|
| `FieldNotFound { type_name, bad_field, available }` | "did you mean `{available[i]}`?" when one structural suggestion is canonical | `LiveCorrection` or `DeferredCorrection` for ambiguous choices |
| `NonExhaustiveMatch { scrutinee, missing_variants }` | Insert all missing arms in one edit | `LiveCorrection` |
| `TypeMismatch { expected, got, expr_span }` | Canonical rewrite when structurally known | `LiveCorrection` or `DeferredCorrection` for semantic choices |
| `UnresolvedIdentifier { name, suggestions }` | Rename to the sole structural suggestion | `LiveCorrection` or `DeferredCorrection` for ambiguous suggestions |
| `UnusedParameter { param_name }` | Prefix `_` when that is the canonical repair | `LiveCorrection` |
| `IdempotencyBreak { op_name, reason }` | Wrap op in idempotent adapter when available | `LiveCorrection` or `DeferredCorrection` |
| `NonTerminatingRecursion { fn_name }` | Add descent evidence when derivable | `LiveCorrection` or `DeferredCorrection` |
| `WorkflowBreaksIdempotency { breaking_op }` | Remove/replace op when canonical | `LiveCorrection` or `DeferredCorrection` |

A fix generator is a pure function:

```dag
fn generate_correction(d: Dag, diag: Diagnostic) -> Correction
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
  correction: DeferredCorrection {
    reason: "FieldNotFound has multiple valid field repairs: a, b"
    retirement_plan: { owner: "R3 Gap 9 row #106", exit_condition: "field-choice correction workflow lands" }
  }
}
```

Display output:
```
ERROR at line 1, col 27: field `c` does not exist on Point

  fn read(p: Point) -> Int = p.c
                              ^

Available fields: a, b
No `FIX` line renders until the diagnostic carries a `LiveCorrection`;
the explicit deferral remains machine-visible for the row #106
retirement tally.
```

---

## Rationale

**Why mandatory `Correction` not `Option<Correction>` or `List<Correction>`?** Multiple valid user choices are common, but a list also made absence look legitimate (`[]`) and allowed partial repairs to masquerade as complete fixes. Row #106 makes the state explicit: one live witness when the compiler knows a roundtrip repair, or one named deferral when it does not. A future choice workflow can add a separate carrier without weakening the mandatory diagnostic contract.

**Why `new_source: String` not `Edit { insertions, deletions }`?** The replacement text IS the fix. Span + new text = minimal information. Edits are reconstructable from span+new: the editor sees "replace span with new_source" and does a simple substitution. Modeling granular edits is premature (the IDE will chunk them anyway).

**Why source-only, no `target_language` field?** Because `Correction` names a single edit domain — the user's `.dag` source. A fix that edits emitted Rust or Python code lives in a different carrier (see "Future extension: TargetCorrection") that owns a target-file span and target-native syntax. Folding both domains into one type with an optional `target_language` admits a state the type cannot locate cleanly: `span` means one thing when `target_language` is None and a different thing when it's Some. Illegal-states-unrepresentable rejects that shape.

**Why `CorrectionStyle` is a `TargetCorrection` concern, not here?** Style differences (Rust's 4-space indent, Python's PEP 8, Go's tabs) only matter when writing into target source. `Correction` writes `.dag` source; `.dag`'s single style rules it. When `TargetCorrection` arrives, it will reuse the `CleanEmissionContract` dispatch pattern from Lane 1 Stage 1c — but that layering is a future concern, not a field on `Correction`.

**Why `span` on Correction (redundant with Diagnostic.span)?** The diagnostic's span points at the error site; the correction's span may be broader or narrower. For non-exhaustive match, diag span is the `match` keyword; correction span is the closing brace position where the new arm inserts. They're genuinely different.

---

## Rejected alternatives

**Inline fix text in `message`** — free-text strings don't compose. IDE integration impossible. Rejected.

**`Fix` as an enum (`Rename`, `Insert`, `Replace`, `Remove`)** — too taxonomy-heavy. All fixes reduce to "at span X, new text is Y." Rejected; would need to be reassembled on the way out anyway.

**One `LiveCorrection` plus hidden alternatives** — implies hierarchy the compiler doesn't actually have. `point.a` is not more correct than `point.b`. Until alternatives are first-class, use `DeferredCorrection` instead of picking arbitrarily.

**Per-target Correction type (`RustCorrection`, `PythonCorrection`, etc.)** — violates single-authority. The Correction type IS language-agnostic; the style IS per-target. Clean separation. Rejected (the hyphenated design above).

---

## Implementation notes

- **DeferredCorrection is the default for any diagnostic lacking a live generator.** Implementers adding new `DiagnosticKind` variants MUST either add a live generator or name the deferral retirement plan.
- **Correction generation is pure** — no I/O, no mutation. Takes `(Dag, Diagnostic)`, returns `Correction`.
- **Generated live corrections MUST parse in context** — Lane 3 Stage 3b adds a test gate: apply each emitted `LiveCorrection` to the source at `CorrectionWitness.span`, then re-run tokenize + parse (and for the shipped fixtures, full `compile_to_dag`). Fragment fixes like `ok` or `A => 1` are not standalone programs; the gate is on the repaired source artifact, not the raw replacement string in isolation.
- **Generated live corrections MUST NOT leave the same diagnostic class failing** — when a repair needs multiple edits in the same span, bundle them into one witness.

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
- [ ] `Diagnostic.correction: Correction` field added (live witness or explicit deferral)
- [ ] `CorrectionStyle` type declared in `std/clean_emission.dag`; `rust_correction_style`, `python_correction_style`, `go_correction_style` data items in each target spec
- [ ] Correction generators for the 8 starter diagnostic kinds (table above) live in `lenses/corrections.dag` and pass unit tests
- [ ] Every new `DiagnosticKind` variant added after this design either has a live generator OR emits a `DeferredCorrection` with a named retirement plan
- [ ] Gate test: every emitted live correction applies to the source at `CorrectionWitness.span` and the repaired source reparses (plus cleanly recompiles for the shipped Stage 3b fixtures)

---

## Open questions

1. **Should Corrections carry confidence scores?** For ambiguous cases (multiple fields within edit distance 1), ranking matters. For now: skip scores, rely on list ordering. If IDE integration needs scores, add in a follow-up.
2. **Can fixes reference other parts of the program?** E.g., "create a missing type `Point` at top of file to match this usage." Cross-span fixes are a stretch goal. For now: single-span fixes only.
3. **What about target-conditional fixes?** Not in this carrier. A diagnostic that specifically lives at the emitted-target level (e.g., "the emitted Rust triggers a rustfmt warning; here's how to fix the Rust") edits target source, not `.dag` source, and therefore belongs in the future `TargetCorrection` type (sketched in §"Future extension"). The split is by authoring domain, not by flag on a shared carrier.

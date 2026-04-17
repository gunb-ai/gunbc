> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1c, Lane 1 Stage 1e (walker reads contract), Lane 3 Stage 3b (CorrectionStyle extends it)

# Design DB-4 — `CleanEmissionContract` concrete fields

**Design blocker:** DB-4
**Consumers:** Lane 1 Stage 1c (invariant E-5 pilot); Lane 1 Stage 1e (generic walker dispatches on contract); Lane 3 Stage 3b (CorrectionStyle field)
**Status:** Design ready for implementer review.
**Depends on:** DB-1 ([design-correction-shape.md](./design-correction-shape.md)) — CorrectionStyle embeds here

---

## Problem

Lane 1 Stage 1c ([phase1-lane2-clean-emission-invariant.md](./phase1-lane2-clean-emission-invariant.md)) establishes the E-5 invariant: emitted code must satisfy the target's clean-code contract by construction. The design doc sketches:

```
data rust_clean_emission: CleanEmissionContract = {
  expression_wrapping: WrapOnlyInOperandPosition
  pattern_bindings: EmitUnderscoreWhenUnused
  imports: IncludeOnlyReferenced
  block_return: NoRedundantWrapping
  post_emit_verifier: RustfmtPlusRustcDashDWarnings
}
```

But these rule-type names (`WrapOnlyInOperandPosition`, `EmitUnderscoreWhenUnused`) are placeholders. The concrete types they resolve to need pinning down.

Lane 1 Stage 1e (generic walker) dispatches on these fields. Lane 3 Stage 3b adds `correction_style: CorrectionStyle` as another field. Both consumers need the contract's shape locked.

---

## Design

### Top-level contract type

```dag
// src/v3/std/clean_emission.dag (new file)
module std.clean_emission

import std.list { List }

// 🟢 TERMINAL. Per-target emission invariant. Emitted code must satisfy
// every rule in this contract by construction (Invariant E-5). Violations
// are emission bugs; no post-emission suppression is valid.
type CleanEmissionContract {
  expression_wrapping: ExpressionWrappingRule
  pattern_bindings: PatternBindingRule
  imports: ImportRule
  block_return: BlockReturnRule
  variable_bindings: VariableBindingRule
  match_arm_body: MatchArmBodyRule
  correction_style: CorrectionStyle  // from DB-1
  post_emit_verifier: PostEmitVerifier
}
```

**Eight fields.** Each is a typed rule enum covering a constructive rendering concern — how to shape emitted code so it doesn't trigger a warning by construction. Dead-code / struct-visibility is explicitly NOT in the contract (removed per codex feedback on PR #491); it's a separate publicity concern.

**Framing the E-5 universal claim**: E-5 means ***"no escape hatches"*** — NOT "we've already covered every warning category." The 8 rules cover the warning classes currently observed. As new targets surface new warnings (Verilog's unused-regs, Python's import-star, etc.), the contract GROWS with a new typed rule — it does NOT grow with `#[allow(...)]` / `# noqa` / pragma suppression. Growth is structural (new rule type + new dispatch point in the walker); suppression is forbidden.

### Rule type definitions

Each rule is a closed coproduct. The variant chosen per target determines emission behavior for that rule.

#### ExpressionWrappingRule (handles `unused_parens`)

```dag
type ExpressionWrappingRule
  = WrapEverything                // pre-L2 behavior (rejected — always wraps)
  | WrapOnlyInOperandPosition     // wrap when precedence matters, not at statement-level
  | NoWrapping                    // never wrap (target handles precedence intrinsically, e.g. SPICE)
```

Emitter dispatch:
- `WrapEverything` — current behavior; emits `(a + b)` always
- `WrapOnlyInOperandPosition` — wraps only when the expression is a subexpression of a precedence-sensitive context (inside a binop, after a keyword, as a field initializer for non-constant value, etc.); omits wrapping at statement terminus
- `NoWrapping` — emits bare `a + b`; requires the target to be precedence-tolerant

Rust: `WrapOnlyInOperandPosition`. Python: `WrapOnlyInOperandPosition`. Go: `WrapOnlyInOperandPosition`. SPICE: `NoWrapping`. Verilog: `WrapOnlyInOperandPosition`. English: `NoWrapping`.

#### PatternBindingRule (handles `unused_variables` in match arms)

```dag
type PatternBindingRule
  = EmitBindingAlways             // current — emits `Value(v)` even if `v` unused
  | EmitUnderscoreWhenUnused      // emits `Value(_)` when arm body doesn't reference `v`
  | EmitPrefixedUnderscoreWhenUnused  // emits `Value(_v)` — Python convention
```

Lane 1c pilot uses `EmitUnderscoreWhenUnused` (with `EmitPrefixedUnderscoreWhenUnused` for Python where `_v` is idiomatic).

Emitter dispatch: walks the arm body, collects referenced PortIds. For each payload binding in the pattern, emits the binding name if referenced, else dispatches to the rule variant.

Rust: `EmitUnderscoreWhenUnused`. Python: `EmitPrefixedUnderscoreWhenUnused`. Go: `EmitUnderscoreWhenUnused` (Go accepts `_`). Additional Shape A targets declare their own rule.

#### ImportRule (handles `unused_imports`)

```dag
type ImportRule
  = ImportEverythingDeclared      // current — emit all declared imports regardless of use
  | IncludeOnlyReferenced         // emit only imports a symbol of which appears in the module
  | ImportPerUsageSite            // (future) emit import at the site where it's used
```

Rust: `IncludeOnlyReferenced`. Python: `IncludeOnlyReferenced`. Go: `IncludeOnlyReferenced`. Additional Shape A targets declare their own rule (most programming languages want `IncludeOnlyReferenced`).

#### BlockReturnRule (handles `unused_parens around block return value`)

```dag
type BlockReturnRule
  = WrapReturnValue               // current — emits `{ (expr) }` at block end
  | NoWrappingOnTerminalExpression // emits `{ expr }`
  | ExplicitReturnKeyword         // emits `{ return expr; }` (Go / explicit-return languages)
```

Rust: `NoWrappingOnTerminalExpression`. Python: `ExplicitReturnKeyword`. Go: `ExplicitReturnKeyword`. Additional Shape A targets declare their own rule.

#### VariableBindingRule (handles `unused_variables` at let bindings)

```dag
type VariableBindingRule
  = EmitBindingAlways
  | EmitUnderscoreWhenUnused      // Rust accepts `let _x = value`
  | OmitBinding                    // if the target allows bare expressions, skip the let
```

Rust: `EmitUnderscoreWhenUnused`. Python: `EmitUnderscoreWhenUnused` (convention: prefix underscore). Go: compile error on unused — emitter MUST reference or use `_ =` assignment.

#### (Removed) StructFieldRule

**Dead-code on emitted structs is NOT an E-5 concern.** Reviewer (codex on PR #491) correctly identified that `StructFieldRule::AllowAttributeOnStructDecl` treats a target-context symptom as part of the emission contract, when it's actually a visibility/publicity concern.

Removed from E-5. The clean-emission invariant covers **constructive rendering rules** — how to SHAPE emitted code so it doesn't trigger warnings by construction. `#[allow(dead_code)]` is suppression, which is what E-5 forbids.

The real fix for dead-code on emitted structs lives in a separate concern: a **publicity declaration** on the struct (is this struct's field set part of a public API boundary? is it used only internally?). That's separate design work deferred until a concrete use case demands it. Meanwhile, emitted test wrappers may carry targeted `#[allow(dead_code)]` in the wrapper code itself — explicitly, in one place, not baked into every struct via `CleanEmissionContract`.

#### MatchArmBodyRule (handles `unused_parens around match arm expression`)

```dag
type MatchArmBodyRule
  = WrapBody                     // current — emits `=> (expr),`
  | NoWrappingOnNonComplexBody   // emits `=> expr,` for atoms; wraps only for multi-statement blocks
```

Rust: `NoWrappingOnNonComplexBody`. Python: `NoWrappingOnNonComplexBody`. Go: N/A (uses switch, different shape).

#### CorrectionStyle (from DB-1)

See [design-correction-shape.md](./design-correction-shape.md).

```dag
type CorrectionStyle {
  indent_unit: String
  line_ending: String
  string_quote: String
  trailing_semicolon: Bool
}
```

#### PostEmitVerifier

```dag
type PostEmitVerifier {
  command: String                 // e.g., "rustc", "gofmt", "black", "verilator"
  args: List<String>              // e.g., ["--edition=2021", "-D", "warnings"]
  syntax_only: Bool               // if true, only syntax-check (don't compile)
  expected_exit_code: Int         // 0 for pass; some verifiers use 1 for "diffs found"
}
```

Rust: `{ command: "rustc", args: ["--edition=2021", "-D", "warnings"], syntax_only: false, expected_exit_code: 0 }`.
Go: `{ command: "gofmt", args: ["-l"], syntax_only: true, expected_exit_code: 0 }` (exit 0 + empty stdout = clean).
Python: `{ command: "python3", args: ["-m", "py_compile"], syntax_only: true, expected_exit_code: 0 }`.
Additional Shape A targets (Swift, Kotlin, etc.) declare their own `post_emit_verifier`. **Shape B formats** (SPICE, Verilog, English, YAML) are NOT compiler targets per THESIS.md §"Two shapes of omni-emission" — they're produced by `.dag` programs, not emitted by the compiler.

### Per-target declarations

Each target spec includes exactly one `CleanEmissionContract` instance:

```dag
// spec/rust.dag
data rust_clean_emission: CleanEmissionContract = {
  expression_wrapping: WrapOnlyInOperandPosition
  pattern_bindings: EmitUnderscoreWhenUnused
  imports: IncludeOnlyReferenced
  block_return: NoWrappingOnTerminalExpression
  variable_bindings: EmitUnderscoreWhenUnused
  match_arm_body: NoWrappingOnNonComplexBody
  correction_style: rust_correction_style
  post_emit_verifier: {
    command: "rustc"
    args: ["--edition=2021", "-D", "warnings"]
    syntax_only: false
    expected_exit_code: 0
  }
}
```

Note: no `struct_fields` field — per the "(Removed) StructFieldRule" section above, dead-code on emitted structs is a visibility/publicity concern handled outside E-5, not part of the clean-emission contract.

Similar for `go_clean_emission`, `python_clean_emission`, and future Shape A targets (Swift, Kotlin, etc.).

---

## Rationale

**Why 8 fields, not more?** Each field corresponds to a distinct warning category observed (parens, unused binds, unused imports, block return, unused vars, match arm body) or needed for extension (correction style, post-emit verifier). A 9th field per new category is cheap to add; starting wider invites fields with no concrete use. Struct-field dead-code was explicitly removed (see "Removed StructFieldRule" above) — that's a publicity concern, not an emission-cleanliness concern.

**Why closed enum per rule, not `fn (Input) -> Output` callbacks?** Walker implementation is simpler: dispatch on variant via match, no first-class functions needed. Targets' rule choice is visible in the spec file. Future "user-defined rules" could add a variant like `Custom(RuleFn)` but not needed today.

**Why `NotApplicable` per-target for some rules?** Explicit "this target doesn't have this concept" beats implicit "field is null." If a future Shape A target lacks pattern matching (rare), it declares `PatternBindingRule::NotApplicable`. Every field is present in every `CleanEmissionContract` instance, with `NotApplicable` variants where the rule doesn't apply.

Actually, simpler: introduce `N/A` variants per rule where relevant:

```dag
type PatternBindingRule
  = EmitBindingAlways
  | EmitUnderscoreWhenUnused
  | EmitPrefixedUnderscoreWhenUnused
  | NotApplicable                 // target has no pattern matching
```

Or use `Option<PatternBindingRule>` in the contract:

```dag
type CleanEmissionContract {
  pattern_bindings: PatternBindingRule?  // None = target has no patterns
  ...
}
```

**Choice: use `NotApplicable` variant per rule.** Field is always `Some`, keeps contract shape uniform. Walker dispatches on the variant; `NotApplicable` is a no-op.

**Why post_emit_verifier in the contract, not a CI config?** Because it IS part of the invariant. The contract's claim is "emitted code passes this verifier." Separating the verifier declaration from the contract would let CI and contract drift; coupling them keeps a single authority.

---

## Rejected alternatives

**Boolean flags (`wrap_parens: Bool`, `emit_unused_binds: Bool`, ...)** — can't express the three-way dispatch each rule actually has. Rejected.

**Single "style" string per target (`"rust-default"`, `"python-strict"`)** — opaque; requires a lookup table. Rejected (inversion of fact-flow — spec carries named policies, not the policies themselves).

**Rule functions as fn pointers / closures** — premature abstraction. Every current rule has ≤3 variants; dispatch-by-match is clearer and cheaper. Rejected for now; revisit if user-defined rules become a need.

**Omit CorrectionStyle (move to separate type)** — CorrectionStyle IS emission-surface data; belongs in the contract. Rejected.

**Omit post_emit_verifier (move to CI)** — invariant requires verifier be tied to contract. Rejected.

---

## Implementation notes

### Walker dispatch pattern

```
fn emit_function_declaration(d: Dag, decl: Declaration, contract: CleanEmissionContract) -> String {
  let body_rendered = render_body(d, decl.body, contract)
  let wrapped_body = match contract.block_return {
    NoWrappingOnTerminalExpression => body_rendered
    WrapReturnValue => "(" + body_rendered + ")"
    ExplicitReturnKeyword => "return " + body_rendered + ";"
  }
  ...
}
```

Each rule dispatch is local and mechanical.

### Post-emit verifier invocation

Lane 1 Stage 1e generic walker finishes emission, then:

```rust
fn run_post_emit_verifier(emitted_source: &str, contract: &CleanEmissionContract) -> Result<(), EmitError> {
    let output = Command::new(&contract.post_emit_verifier.command)
        .args(&contract.post_emit_verifier.args)
        .stdin(/* pipe emitted_source */)
        .output()?;
    if output.status.code() != Some(contract.post_emit_verifier.expected_exit_code) {
        return Err(EmitError::VerifierFailed {
            target: /* target name */,
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        });
    }
    Ok(())
}
```

CI gates on this. No `#[allow(warnings)]` attributes anywhere emit_*.

### Migration impact on existing tests

Current tests have `#[allow(warnings, clippy::all)]` on emitted wrapper modules. Lane 1 Stage 1c removes these as each warning category's rule lands structurally.

Rollout per rule:
1. Implement the rule in the walker
2. Regenerate snapshots (warnings eliminated at source)
3. Remove the corresponding lint from the test wrapper's `#[allow(...)]`
4. Run verifier — must pass without suppression

By end of Lane 1 Stage 1c pilot (unused pattern bindings): `unused_variables` removed from the `#[allow]` list. Other categories follow in Lane 1 Stage 1e's consolidation.

---

## Associations

- **Lane 1 Stage 1c** ([phase1-lane2-clean-emission-invariant.md](./phase1-lane2-clean-emission-invariant.md)) — implements E-5 invariant + pilot with `PatternBindingRule`
- **Lane 1 Stage 1e** — generic walker dispatches on `CleanEmissionContract` during emission
- **Lane 3 Stage 3b** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — `correction_style` field populated; fix generators use it
- **DB-1 `Correction` shape** ([design-correction-shape.md](./design-correction-shape.md)) — `CorrectionStyle` embeds here; prior design of that type is a hard dep
- **Create `src/v3/std/clean_emission.dag`** — new file
- **Update `src/v3/spec/rust.dag`, `go.dag`, `python.dag`** — add `*_clean_emission` data items with concrete rule choices
- **Add `src/v3/spec/verilog.dag`, `spice.dag`, `english.dag`** — each with its own `*_clean_emission`
- **Update `INVARIANTS.md`** — E-5 entry pointing at this design

---

## Acceptance

- [ ] `std/clean_emission.dag` created with `CleanEmissionContract` + 8 sub-rule types + `NotApplicable` variants where relevant
- [ ] Each existing target spec (`rust.dag`, `go.dag`, `python.dag`) has a `*_clean_emission` data item populated with concrete rules
- [ ] Lane 1 Stage 1c pilot (pattern bindings) dispatches on `contract.pattern_bindings` and produces expected output for each rule variant
- [ ] Walker invokes `post_emit_verifier` after each emission and fails emission on verifier errors
- [ ] `INVARIANTS.md` E-5 entry references this design

---

## Open questions

1. **Is the rule set closed?** 8 rules cover current warnings for Rust/Go/Python. Additional Shape A targets may introduce new warning categories (e.g., Kotlin's null-safety lints); add a typed rule when a real case arises. Per E-5's "no escape hatches" framing, suppression is not an alternative to adding the rule.

2. **Should `post_emit_verifier.args` templates support variable substitution** (e.g., `{file}`, `{target_dir}`)? Probably yes, but deferred — first concrete use case determines the variable set.

3. **How does `CleanEmissionContract` interact with async emission (Lane 4 Stage 4d)?** Possibly a second contract per "emission mode" (sync vs async Rust). Defer — design once a second Rust mode is concrete.

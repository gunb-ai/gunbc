### E-5: Clean-emission contract is satisfied by construction (2026-04-17)

For every target T and program P, the emission pipeline produces
source code that passes T's declared clean-code verifier without
modification. The verifier is named in the target's
`CleanEmissionContract.post_emit_verifier` (e.g., `rustc --edition=2021
-D warnings` for Rust). A diagnostic from the verifier is an emission
bug, not a formatting difference — the fix is structural, not
suppression.

**What "no escape hatches" means.** The contract GROWS by adding
typed rules to `CleanEmissionContract` when new warning categories
surface. It does NOT grow by adding `#[allow(...)]` / `# noqa` /
pragma suppression. E-5's commitment is to CONSTRUCTIVE emission —
"we've covered every category by SHAPE, not by silencing" — so new
targets that surface new warnings (Kotlin's null-safety lints,
Verilog's unused-regs, etc.) land as new typed rules, not as new
suppressed categories. Every rule's dispatch in the emitter removes
(or narrows) a corresponding entry from the test-wrapper
`#[allow(...)]` umbrella; umbrellas only shrink.

**The contract.** Eight rules cover the warning classes currently
observed. See `docs/design-clean-emission-contract.md` (DB-4) for
the full type table and
`src/v3/std/clean_emission.dag` for the declarations.

| Rule | Concern (rustc lint) |
|---|---|
| `expression_wrapping` | `unused_parens` on wrapped expressions |
| `pattern_bindings` | `unused_variables` in match arm payloads |
| `imports` | `unused_imports` |
| `block_return` | `unused_parens` around block terminal value |
| `variable_bindings` | `unused_variables` at let bindings |
| `match_arm_body` | `unused_parens` around match arm body |
| `correction_style` | surface-level fix rendering (DB-1) |
| `post_emit_verifier` | declared verifier command + args |

**Why in INVARIANTS.md.** Once E-5 lands, every emitter change must
justify compliance. A future emitter PR adding a new warning
category without either (a) structurally fixing the cause or (b)
adding a typed rule to the contract is an E-5 violation. The
invariant is the mechanism that prevents "ship now, allow-suppress
later, forget forever."

**Staged rollout (Lane 1 Stage 1c).** E-5 is declared structurally
(rule type system + per-target contract + invariant entry) at the
same time as the first pilot rule (Rust's `PatternBindingRule::
EmitUnderscoreWhenUnused`). Remaining rule dispatches land in
Stage 1c PR 2 (Go + Python pilots) and Stage 1d/1e (consolidation
sweep for parens, imports, etc.). Each PR that wires a new rule
removes the matching lint from the test-wrapper allow umbrella.

**Structural prevention:** the rule dispatch lives inside the
emitter. Emitted text is derived from `contract.<rule>` by match,
not by string rewrites over already-emitted source.

**Test:** each rule's dispatch ships with at least one pilot test
that compiles the emitted code under `#[deny(<rustc-lint>)]` and
asserts compilation succeeds.

**Origin:** DB-4 + Lane 1 Stage 1c. Replaces the band-aid
`#[allow(warnings)]` approach that accumulated around emitted
wrapper modules through M1.


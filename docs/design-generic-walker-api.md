> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1e, Lane 1 Stage 1f, Lane 3 Stage 3c, Lane 4 Stage 4d

# Design DB-2 — Generic walker API

**Design blocker:** DB-2
**Consumers:** Lane 1 Stage 1e (consolidation implementation); Lane 1 Stage 1f (consolidation proof, optionally adding one additional Shape A language); Lane 3 Stage 3c (self-hosting cycle runs through walker); Lane 4 Stage 4d (async emission via walker's spec dispatch)
**Status:** Design ready for implementer review.
**Depends on:** DB-4 ([design-clean-emission-contract.md](./design-clean-emission-contract.md)) — walker reads `CleanEmissionContract`; DB-5 ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — walker consumes keyed accessors

---

## Problem

Today `src/v3/compiler/src/emit_rust.rs` is ~3600 lines of hand-written per-target emission logic. `emit_go.rs` and `emit_python.rs` duplicate the pattern. Lane 1 Stage 1e consolidates these into **one generic walker** + three target specs.

The walker's API determines:
- How much logic actually disappears from per-language files
- Whether adding one additional Shape A language (Swift, Kotlin, etc.) is realistic (Lane 1f). **Shape B artifacts (SPICE, Verilog, English, YAML, etc.) are NOT compiler targets** per THESIS.md §"Two shapes" — they're produced by `.dag` programs.
- Whether self-hosting cycles cleanly (Lane 3c)
- Whether async variants can emit from the same walker (Lane 4d)

Getting this API wrong means rework at every downstream stage. Getting it right means ~90% of current per-language code dissolves.

---

## Design

### Top-level entry point

```rust
// src/v3/compiler/src/emit.rs (new file; dissolves the per-language emit_*.rs files)
pub fn emit(
    dag: &Dag,
    target: TargetLanguageId,
) -> Result<EmittedSource, EmitError>;

pub struct EmittedSource {
    pub text: String,
    pub target: TargetLanguageId,
}
```

That's the entire public surface. Everything else is internal.

### Target resolution

```rust
fn resolve_target(dag: &Dag, target: TargetLanguageId) -> Result<TargetContext, EmitError> {
    let spec = dag.target_spec(target)?;                  // looks up e.g. rust_spec data item
    let clean_emission = spec.clean_emission_contract()?; // DB-4 contract
    let realizations = spec.realization_indexes()?;       // existing RealizationIndex pattern
    let post_emit = clean_emission.post_emit_verifier.clone();
    Ok(TargetContext {
        id: target,
        clean_emission,
        realizations,
        post_emit,
    })
}

pub struct TargetContext {
    pub id: TargetLanguageId,
    pub clean_emission: CleanEmissionContract,   // from DB-4
    pub realizations: RealizationIndexes,        // existing substrate shape
    pub post_emit: PostEmitVerifier,             // from DB-4
}
```

**Target is looked up by substrate fact, not by name-prefix.** Each `TypeRealization` / `CallableRealization` / `PatternRealization` / `BehaviorRealization` has a typed `target: TargetLanguageId` field (fixes inherited bug B11). The walker filters by this field, not by `name.starts_with("rust_")`.

### Core walker

```rust
fn emit_dag(dag: &Dag, ctx: &TargetContext) -> Result<String, EmitError> {
    let mut out = String::new();
    emit_module_header(dag, ctx, &mut out)?;
    for decl in dag.declarations.iter() {
        emit_declaration(dag, ctx, decl, &mut out)?;
    }
    for bind in top_level_binds(dag) {
        emit_top_level_bind(dag, ctx, bind, &mut out)?;
    }
    emit_module_footer(dag, ctx, &mut out)?;
    Ok(out)
}
```

No per-target logic here. The walker is target-agnostic; all target-specific behavior comes from `ctx`.

### Node dispatch

```rust
fn emit_behavior(
    dag: &Dag,
    ctx: &TargetContext,
    behavior: &Behavior,
    locals: &LocalScope,
) -> Result<String, EmitError> {
    match behavior {
        Behavior::Value(v) => emit_value(dag, ctx, v),
        Behavior::Transform(t) => emit_transform(dag, ctx, t, locals),
        Behavior::Branch(b) => emit_branch(dag, ctx, b, locals),
        Behavior::Loop(l) => emit_loop(dag, ctx, l, locals),
        Behavior::Bind(b) => emit_bind(dag, ctx, b, locals),
    }
}
```

Each sub-function reads relevant substrate facts + the contract:

```rust
fn emit_transform(
    dag: &Dag,
    ctx: &TargetContext,
    t: &TransformNode,
    locals: &LocalScope,
) -> Result<String, EmitError> {
    let realization = ctx.realizations
        .callable_for(t.target)
        .ok_or(EmitError::MissingCallableRealization { target: t.target })?;
    let inputs = t.inputs.iter().map(|&port| {
        emit_port(dag, ctx, port, locals)
    }).collect::<Result<Vec<_>, _>>()?;
    let rendered = substitute_template(&realization.template, &[
        ("callable_name", &realization.carrier),
        ("args", &join_with_separator(&inputs, &realization.arg_separator)),
    ]);
    apply_expression_wrapping(&rendered, ctx.clean_emission.expression_wrapping, locals.position())
}
```

The `apply_expression_wrapping` step is where `CleanEmissionContract.expression_wrapping` rule dispatches. See DB-4 for the rule variants.

### Template substitution

```rust
fn substitute_template(template: &str, bindings: &[(&str, &str)]) -> String {
    // Reads `{name}` placeholders in template, replaces with binding values.
    // Existing render_named_template from emit_rust.rs lifts here verbatim.
}
```

No change to how templates work — they're declared in target specs today. Moving this function into the generic walker is a 1:1 lift.

### LocalScope (replaces per-file `RenderLocals` + `bound_names`)

```rust
pub struct LocalScope {
    names: HashMap<PortId, LocalBinding>,       // pattern-bound, let-bound names in scope
    bound_names: HashMap<PortId, LocalBinding>, // top-level Bind names
    parent: Option<&'a LocalScope<'a>>,         // for lambda captures
    position: EmissionPosition,                 // Statement | Operand | BlockReturn | MatchArmBody
}
```

`EmissionPosition` drives `apply_expression_wrapping`'s choice for `WrapOnlyInOperandPosition` — wrap if Operand, not if Statement/BlockReturn/MatchArmBody.

### Pattern emission (DB-4 dispatch)

```rust
fn emit_pattern(
    dag: &Dag,
    ctx: &TargetContext,
    path: &Path,
    arm_body_uses: &HashSet<PortId>,    // collected from emitting the arm body
) -> Result<String, EmitError> {
    match &path.pattern {
        BranchPattern::ResolvedVariant(decl_id) => {
            let realization = ctx.realizations
                .pattern_for(*decl_id)
                .ok_or(EmitError::MissingPatternRealization { target: *decl_id })?;
            // Dispatch on pattern_bindings rule
            let binding_rendered = match ctx.clean_emission.pattern_bindings {
                PatternBindingRule::EmitBindingAlways => {
                    realization.binding_template_full.clone()
                }
                PatternBindingRule::EmitUnderscoreWhenUnused => {
                    render_pattern_with_unused_underscores(
                        &realization.binding_template,
                        path.binding.as_ref(),
                        arm_body_uses,
                    )?
                }
                PatternBindingRule::EmitPrefixedUnderscoreWhenUnused => {
                    render_pattern_with_unused_prefix(
                        &realization.binding_template,
                        path.binding.as_ref(),
                        arm_body_uses,
                        "_",
                    )?
                }
                PatternBindingRule::NotApplicable => {
                    return Err(EmitError::UnsupportedBehavior("target has no pattern matching".into()));
                }
            };
            substitute_template(&realization.template, &[
                ("constructor", &realization.constructor_name),
                ("binding", &binding_rendered),
            ])
        }
        BranchPattern::UnresolvedVariant { .. } => {
            Err(EmitError::UnsupportedBehavior("unresolved variant in emit".into()))
        }
    }
}
```

### Post-emission verification

```rust
fn emit(dag: &Dag, target: TargetLanguageId) -> Result<EmittedSource, EmitError> {
    let ctx = resolve_target(dag, target)?;
    let text = emit_dag(dag, &ctx)?;
    run_post_emit_verifier(&text, &ctx.post_emit)?;
    Ok(EmittedSource { text, target })
}
```

Every `emit` call runs the verifier. CI gate on this.

### Per-target Rust shims (deletable layer)

Existing consumers call `emit_rust(dag)`, `emit_go(dag)`, etc. These become trivial wrappers that go away in Lane 1 Stage 1e completion:

```rust
#[deprecated]
pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    emit(dag, TargetLanguageId::Rust).map(|e| e.text)
}

// Same for emit_go, emit_python. All marked #[deprecated] during consolidation,
// deleted at end of Lane 1 Stage 1e.
```

After Lane 1e: callers pass `TargetLanguageId` explicitly; no per-target function exists.

---

## API contract promises

What the walker MUST do (contracts the implementation holds):

1. **Target agnostic core.** `emit_dag`, `emit_behavior`, `emit_pattern` contain zero Rust/Go/Python specific branches. Any such branch = bug, fixable by moving the fact into the target spec.
2. **Spec-driven templates.** Every rendered string comes from a template declared in the target spec OR from direct composition of spec fields (arg separators, etc.). No hardcoded syntax strings in Rust.
3. **Contract-driven warning shape.** Every emission rule that could trigger a warning dispatches on `ctx.clean_emission`. Adding a new warning category = add a rule type (DB-4) + add a dispatch point in the walker.
4. **Fail-closed realizations.** If a required `Realization` is missing for a target, `EmitError` with clear diagnostic. No silent defaults.
5. **Post-emit verification is not optional.** `emit()` always calls `run_post_emit_verifier`. No flag to skip.

What callers can rely on:

- Pure function: `emit(dag, target)` depends only on `dag` + target spec + clean-emission contract. No hidden state.
- Deterministic: same inputs produce bit-identical output. Critical for Lane 3c fixed-point.
- Per-target error messages: `EmitError` variants carry `target: TargetLanguageId` so multi-target failures are debuggable.

---

## Rationale

**Why one `emit(dag, target)` entry point not three?** Because the target is data, not a function identity. Calling `emit(dag, TargetLanguageId::Swift)` adds a target without adding a function. Matches the thesis claim "adding a new target = one spec file, zero new Rust." One Rust function that reads the new spec.

**Why `TargetContext` bundles contract + realizations + post-emit?** Because they're read together at every emission site. Bundling avoids threading three arguments through every helper; mirrors the existing `Ctx` struct in `emit_rust.rs`.

**Why not trait-dispatch the walker?** Because a trait `Emit` with per-target `impl` puts Rust-specific logic back in Rust files per-target — the exact thing we're dissolving. The data-driven dispatch (spec → rule → walker action) keeps everything in the spec and the walker, not in per-target impls.

**Why not emit to an AST first, then serialize per-target?** Because the spec's templates ARE the serialization. Going through an intermediate AST adds a layer without changing what's expressible. If a new target needs AST-level transformation (not just template substitution), add it as a substrate-level fact, not an emitter layer. Rejected as premature.

**Why `LocalScope.position: EmissionPosition` instead of recomputing context from call stack?** Explicit > implicit. The walker knows which position it's in (it CHOSE to descend into an operand); that choice is data passed to the child. Cheaper than reconstructing from recursion depth.

**Why keep per-target Rust shims during transition?** Because existing consumers (tests, CLI) use `emit_rust(dag)`. Breaking them all simultaneously = 30+ file changes in one PR. Shims let consumers migrate incrementally while the walker is the authority.

---

## Rejected alternatives

**Visitor pattern with trait** — every Rust file implements a Visitor for each target. Puts logic back in per-target files. Rejected.

**One emit function per Behavior variant, one per target** — combinatorial explosion. Rejected.

**Emit to s-expression intermediate, transform, print** — useful for some tools but not for structural emission. Overkill. Rejected.

**`emit_rust(dag)` as the top level, call through to `emit_target(dag, TargetLanguageId::Rust)` internally** — preserves the existing function name but hides the target-is-data reality. Keeps a name-keyed public API that consumers learn. Prefer explicit `emit(dag, target)`.

**Bundle post-emit verification into CI script, not walker** — lets emitter and verifier drift. E-5 invariant requires coupling. Rejected.

---

## Implementation plan

### Lane 1 Stage 1e execution order

Six sub-steps, ordered by dependency. Sub-step sizes sum to the lane's L total; individual sub-steps are S (roughly a commit each, with 1e.2 larger because it covers the first end-to-end lift):

1. **1e.1** (S): Create `src/v3/compiler/src/emit.rs` with `emit`, `emit_dag`, top-level structure. All helpers stubbed or deferring to `emit_rust.rs` functions.
2. **1e.2** (M): Lift `emit_value`, `emit_transform`, `emit_loop` from `emit_rust.rs` into `emit.rs`. Add `TargetContext`. Templates read from the chosen target spec.
3. **1e.3** (S): Lift `emit_branch`, `emit_pattern`. Implement `PatternBindingRule` dispatch (DB-4). Tests for pattern binding underscore behavior.
4. **1e.4** (S): Lift `emit_bind`, `emit_function_declaration`. Implement `ExpressionWrappingRule` dispatch. All Rust tests pass via the generic walker.
5. **1e.5** (S): Same lift for Go and Python. `emit_rust.rs`, `emit_go.rs`, `emit_python.rs` become shims (or deleted if tests migrate cleanly).
6. **1e.6** (S): Post-emit verifier wired. `-D warnings` is live for Rust; gofmt lint is live for Go; etc. `#[allow(warnings)]` attributes removed everywhere.

### Test strategy

Each existing emit test (m1_3_emit_rust_test, m2_lens_* migration tests, etc.) continues to pass. They invoke `emit_rust(dag)` which lowers to `emit(dag, TargetLanguageId::Rust)`. Emission output should be bit-identical to pre-consolidation OR better (no warnings). Any difference is reviewed and approved as a snapshot update.

### New test: target swap

A meta-test proving the "one spec = one target" claim:

```rust
#[test]
fn adding_new_target_changes_zero_walker_code() {
    // Set up a tiny Swift spec (just enough to emit `let x = 1`)
    // Call emit(dag, TargetLanguageId::Swift)
    // Assert: output matches expected Swift snippet
    // No src/v3/compiler/src/emit.rs changes committed between this test and the Swift spec addition
}
```

This test is the regression gate for Lane 1 Stage 1f (new targets).

---

## Associations

- **Lane 1 Stage 1e** ([phase1-lane3-consolidation-build-plan.md](./phase1-lane3-consolidation-build-plan.md) → execution doc TBD) — this is its core design
- **Lane 1 Stage 1f** — adding one additional Shape A language via target spec only, no walker changes
- **Lane 3 Stage 3c** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — self-hosting cycle runs compiler.dag through this walker
- **Lane 4 Stage 4d** ([lane4-completion.md](./lane4-completion.md)) — async emission = target spec field, walker dispatches
- **DB-4 `CleanEmissionContract`** ([design-clean-emission-contract.md](./design-clean-emission-contract.md)) — walker reads this for every rule dispatch
- **DB-5 Substrate keyed-lookup** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — walker uses `port(d, id)`, `node(d, id)`, `resolve_producer(d, id)`
- **Create `src/v3/compiler/src/emit.rs`** — the walker itself
- **Eventually delete `src/v3/compiler/src/emit_rust.rs`, `emit_go.rs`, `emit_python.rs`** — end of Lane 1e

---

## Acceptance (Lane 1 Stage 1e owns)

- [ ] `src/v3/compiler/src/emit.rs` exists; `pub fn emit(dag, target)` is the only public entry
- [ ] `emit_rust.rs`, `emit_go.rs`, `emit_python.rs` are `#[deprecated]` shims or deleted
- [ ] All existing emit tests pass
- [ ] `grep -rn "fn render_" src/v3/compiler/src/emit.rs` — target-agnostic helpers only; no `fn render_rust_*` etc.
- [ ] Every `Realization` entry in spec/*.dag carries a typed `target: TargetLanguageId` field (no name-prefix dispatch)
- [ ] Post-emit verifier invocation is part of `emit`; `emit()` fails on verifier errors
- [ ] Lane 1f test (if Option B chosen): adding `spec/swift.dag` with `swift_spec: TargetSpec` enables `emit(dag, TargetLanguageId::Swift)` without any change to `emit.rs`

---

## Open questions

1. **How does the walker handle `rust_rendering` / language-specific rendering fields** (e.g., `rust_read_strategy: ReadStrategy`)? Current proposal: they're fields on the target spec that the contract dispatches on, or they roll into `CleanEmissionContract` if warning-affecting. Specific fields TBD at Lane 1d (build plan).

2. **Does `emit` return partial output on error?** Current: all-or-nothing (`Result<EmittedSource, EmitError>`). Alternative: return partial output + error list for IDE use cases. Deferred.

3. **Can a single `emit` call produce multiple files** (e.g., `main.rs` + `lib.rs`)? Today: one output per call. Multi-file output would be a follow-up if a target needs it. No current target does.

4. **How are cross-file imports handled in multi-target mode?** If a user program is one .dag but emission is to multiple targets, imports between emissions are out of scope. Each `emit(dag, target)` call is independent. Confirmed scope.

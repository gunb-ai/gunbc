> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 3 master)

# Lane 3 — Self-hosting cycle

**Lane:** 3 (of 3)
**Time budget:** ~4 weeks
**Status:** Plan. No code changes yet.

---

## Thesis mandate

From project memory: *"gunbc is a causal engine; the compiler describes itself in .dag and is its own first consumer."*

From SELF_HOSTING.md: the compiler's source is `compiler.dag`; emission produces Rust; that Rust, compiled, IS the compiler. Fixed-point: running the compiler on its own source produces bit-identical Rust.

Today:
- `compiler.dag` exists (PR #418) as the single authority for the self-hosting cycle
- `src/v3/compiler/src/` is ~97% hand-written Rust "sketch, not oracle" (ROADMAP.md §Sketch vs Oracle framing)
- The emit → compile → emit cycle is NOT wired
- M2 feature parity gaps (mutual recursion, `data` semantics, `where` refinement, full surface generics) block `compiler.dag` from being fully expressible
- Diagnostics use compiler-internal vocabulary; `docs/error-examples.md` aspiration doc shows the target shape but the implementations don't emit corrections

Lane 3 closes all three.

---

## Stages

### Stage 3a — M2 feature parity for compiler.dag (1.5 weeks)

**Scope:** the surface features `compiler.dag` needs that v3 doesn't yet have.

From ROADMAP.md §M2 + Half B B13 deferral:
- **Mutual recursion → Loop** (SELF_HOSTING §2.4, design locked in [DB-9](./design-mutual-recursion-lowering.md)). Compiler pipelines (parse → lower → infer → emit) are structurally mutually recursive across modules.
- **`data` value semantics** — `data foo: Type = value` as structural value declarations. Today parsed but semantics incomplete for generic `data`.
- **`where` refinement predicates** — `fn f(x: Int where x > 0)`. Needs refinement carrier on Port types + Branch-boundary verification.
- **Full surface generics** — implicit generic inference (Prereq 0.5) covers inference; explicit surface generics `fn f<T>(x: T) -> T` for compiler code.
- **Transport declarations** — compiler-internal services (parser, emitter) need transport-compatible calling conventions.
- **Dotted-path access to `Disj` variants** (Half B B13 deferral). Currently the `.dag` parser supports dotted-path access only for `Conj` fields (`record.field`). Half B needed `enum.variant.field` access for typed Python pattern emission but had to defer because the parser rejects it. Stage 3a wires this: parser extension + lowering to field-access on variant payload.

Each feature has a tightly-scoped acceptance:

| Feature | Acceptance test |
|---|---|
| Mutual recursion | `fn a(n: Int) -> Int = if n == 0 then 0 else b(n - 1)` + `fn b(n: Int) -> Int = if n == 0 then 0 else a(n - 1)` compiles, both marked as `MutualLoop` per DB-9 |
| `data` value semantics | `data answer: Int = 42` + `data config: Config = { ... }` compile, values accessible at emission |
| `where` refinement | `fn div(n: Int, d: Int where d != 0) -> Int` — compile-time rejection of `div(1, 0)` |
| Surface generics | `fn id<T>(x: T) -> T` — compiles with explicit type param |
| Disj dotted-path | `match opt { Some(s) => s.field, None => ... }` parses and lowers — unblocks Half B's B13 |

**Escalation:** each sub-feature has its own failure mode. If mutual recursion needs a substrate extension to represent SCC (strongly-connected-component) binds, surface — that's a legitimate substrate gap, not a Loop pattern hack.

### Stage 3b — Diagnostics as corrections (1 week)

**Scope:** every diagnostic carries `fix: List<Correction>` where each `Correction` is literal code the user can paste.

Per `docs/error-examples.md`:

```
ERROR at line 2: field `c` does not exist on Point

  fn read(point: Point) -> Int = point.c
                                       ^

Available fields: a, b
FIX (option 1): did you mean `point.a`?
FIX (option 2): did you mean `point.b`?
```

New substrate additions:

```
type Diagnostic {
  // existing fields...
  fix: List<Correction>
}

type Correction {
  description: String
  new_source: String  // literal replacement code
  span: SourceSpan    // what to replace
}
```

Per-target fix syntax declared in the same spec surface as Lane 1c's `CleanEmissionContract`:

```
data rust_correction_style: CorrectionStyle = {
  braces_required: true
  ...
}

data python_correction_style: CorrectionStyle = { ... }
```

**Acceptance:**
- Every T-series test in `thesis_validation_test.rs` (T1.1–T1.5, T2.4) emits at least one Correction
- Diagnostic rendering in test output shows the FIX lines from error-examples.md
- Renaming variants in source → non-exhaustive match diagnostic emits the missing-variant Correction

**Escalation:** if corrections need semantic information the diagnostic site doesn't have (e.g., "suggest the right field" requires schema lookup from wherever the type is defined), surface — that's a legitimate API extension. Don't fabricate corrections without the needed context.

### Stage 3c — Self-hosting cycle (1.5 weeks)

**Scope:** wire the emit → compile → emit fixed-point.

Pipeline:

```
compiler.dag
  + src/v3/spec/rust.dag  (Lane 1e unified spec)
  + Lane 1e generic walker
  → Rust source code
  → rustc
  → v3_compiler binary (next generation)
```

Fixed-point ratchet: run the NEW v3_compiler on `compiler.dag` again. Output Rust must be bit-identical to the first emission. If not, non-determinism or unstable emission is a bug.

New CI gate: `cargo run --bin self-host-fixed-point`:
1. Emit Rust from `compiler.dag` using current binary → `stage1.rs`
2. Compile `stage1.rs` → `stage1_bin`
3. Run `stage1_bin` on `compiler.dag` → `stage2.rs`
4. `diff stage1.rs stage2.rs` — must be empty

**Acceptance:**
- `self-host-fixed-point` passes on CI
- A change to `compiler.dag` that regenerates the binary produces a binary that re-emits the SAME Rust (not just compiles)
- The Rust-sketch dependencies on hand-written `emit_rust.rs` (now-deleted per Lane 1e) have been replaced by `compiler.dag`-emitted code

**Escalation:**
1. **Non-determinism in emission** — if stage1.rs ≠ stage2.rs, trace the difference. Usually HashMap iteration order. Fix with deterministic iteration (BTreeMap) in emission; do NOT accept "almost identical" output.

2. **Bootstrapping paradox** — if `compiler.dag` requires features that the current compiler doesn't support, the cycle can't close. Stage 3a is supposed to close this gap; if a gap remains, defer 3c and expand 3a's scope.

3. **Fixed-point runtime explosion** — if the cycle takes longer than, say, 5 minutes, something is wrong. v2 had a 20-minute self-compile incident that led to the merge_envs fix. Watch for the same class of bug.

---

## Cross-cutting acceptance (Lane 3 done when)

- [ ] `cargo run --bin self-host-fixed-point` passes — bit-identical Rust output on re-emit
- [ ] Every T-series diagnostic in `thesis_validation_test.rs` emits a literal Correction
- [ ] Mutual recursion, `data` semantics, `where` refinement, surface generics all compile and test
- [ ] `compiler.dag` is the single authority for the self-hosting cycle (no hand-written Rust in the cycle path)
- [ ] Hand-written `src/v3/compiler/src/emit_rust.rs` and the other per-target emit files DO NOT EXIST (they dissolved in Lane 1e; 3c verifies the dissolution holds through self-hosting)

---

## Dependencies

- **Requires Lane 1 Stage 1e complete** — self-hosting through fragmented per-target emitters is worthless. The dissolved single-emitter is what gets re-emitted in 3c.
- **Can overlap Lane 2** — 3a (M2 feature parity) is independent of Lane 2 work; 3b (corrections) references CleanEmissionContract from Lane 1c but doesn't depend on Lane 2 property proofs; 3c is purely about the emitter cycle.
- **Blocks nothing** — Lane 3 is the thesis terminus.

---

## Estimate

4 weeks, starts roughly week 7 (after Lane 1 Stage 1e lands around week 6):
- 3a: 1.5 weeks (weeks 7–8.5)
- 3b: 1 week (weeks 8.5–9.5)
- 3c: 1.5 weeks (weeks 9.5–11)

Buffer at week 12 for integration issues.

---

## What's TRUE when Lane 3 completes

A user writes `compiler.dag`, runs the current compiler on it, gets a Rust source file. They compile that Rust with `rustc`. They run the resulting binary on `compiler.dag` again. The Rust output is bit-identical.

The compiler is its own first consumer. The thesis's "causal engine" claim is no longer a slogan — it's a CI gate.

Every error message the compiler emits tells the user not just what's wrong but what to type to fix it. The compiler teaches.

The plan is done.

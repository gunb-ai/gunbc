> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 3 master)

# Lane 3 — Self-hosting cycle

**Lane:** 3 (of 3)
**Size:** XL (three stages, one with five sub-stages)
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

### Stage 3a — M2 feature parity for compiler.dag (XL, 5 sub-stages)

**Scope:** five surface-language / substrate extensions `compiler.dag` needs.

**Why XL, not L:** director review (PR #491) correctly flagged earlier L estimate as unrealistic. Several of these are substantial extensions historically. Split into sub-stages with honest per-feature sizes:

| Sub-stage | Size | Scope | Design doc |
|---|---|---|---|
| 3a.1 | L | Mutual recursion: lowering produces a `Behavior::Loop` whose `bound = LoopBound::Descent { cluster }` references a `Cluster` in the new `Dag.clusters` sidecar. Per-member descent positions; real call topology preserved. **Five `Behavior` variants unchanged**; localized substrate extension on `LoopBound` + sidecar. See [DB-9 R2](./design-mutual-recursion-lowering.md). | [DB-9 R2](./design-mutual-recursion-lowering.md) |
| 3a.2 | S | `data` value semantics: `data foo: Type = value` as structural value declarations accessible at emission | [DB-10 (consolidated)](./design-m2-feature-parity.md) |
| 3a.3 | M | `where` refinement predicates: `fn f(x: Int where x > 0)`. Needs refinement carrier on type declarations + Branch-arm narrowing integration. | [DB-11 (consolidated)](./design-m2-feature-parity.md) |
| 3a.4 | S | Full surface generics: explicit `fn f<T>(x: T) -> T` syntax for compiler code (Prereq 0.5 covers inference; 3a.4 is surface/lowering) | [DB-12 (consolidated)](./design-m2-feature-parity.md) |
| 3a.5 | S | Disj dotted-path parser extension: `match opt { Some(s) => s.field }` — unblocks Half B B13 | [DB-13 (consolidated)](./design-m2-feature-parity.md) |

**Acceptance per sub-stage:**

| Sub-stage | Acceptance test |
|---|---|
| 3a.1 Mutual recursion | `fn a(n: Int) -> Int = if n == 0 then 0 else b(n - 1)` + `fn b(n: Int) -> Int = if n == 0 then 0 else a(n - 1)` compiles (members stay as ordinary peer Binds in `Dag.nodes`; lowering wraps the cluster in a `Behavior::Loop` whose `bound = LoopBound::Descent { cluster }` points at `Dag.clusters[cluster]`; five `Behavior` variants unchanged per [DB-9 R2](./design-mutual-recursion-lowering.md)); termination lens reads cluster membership from the sidecar and verifies per-member descent positions; `fn a(n) = b(n); fn b(n) = a(n)` fails with cluster termination diagnostic naming the failing `CallEdge` + `MemberDescent.position` |
| 3a.2 `data` value semantics | `data answer: Int = 42` + `data config: Config = { ... }` compile, values accessible at emission |
| 3a.3 `where` refinement | `fn div(n: Int, d: Int where d != 0) -> Int` — compile-time rejection of `div(1, 0)` |
| 3a.4 Surface generics | `fn id<T>(x: T) -> T` — compiles with explicit type param |
| 3a.5 Disj dotted-path | `match opt { Some(s) => s.field, None => ... }` parses and lowers — unblocks Half B's B13 |

**Escalation:** if any single sub-stage materially exceeds its size classification (e.g., an S stretches to M, an M stretches to L), stop and escalate. Don't silently absorb; each sub-stage is individually sized to give real overrun signal.

### Stage 3b — Diagnostics as corrections (M)

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

Type shapes are **locked in [DB-1](./design-correction-shape.md)**. Lane 3 does not restate them here — see DB-1 for the `Correction` record, the `Diagnostic.fixes` field (plural `fixes`, a `List`), `CorrectionStyle` per-target style, rejected alternatives, and the source-only-by-construction rationale. Lane 3 Stage 3b consumes that locked shape without re-deriving it.

Per-target correction style is declared in each target spec alongside its `CleanEmissionContract` — DB-1 specifies the exact field set. Don't restate here; reference the locked shape.

**Acceptance:**
- Every T-series test in `thesis_validation_test.rs` (T1.1–T1.5, T2.4) emits at least one Correction
- Diagnostic rendering in test output shows the FIX lines from error-examples.md
- Renaming variants in source → non-exhaustive match diagnostic emits the missing-variant Correction

**Escalation:** if corrections need semantic information the diagnostic site doesn't have (e.g., "suggest the right field" requires schema lookup from wherever the type is defined), surface — that's a legitimate API extension. Don't fabricate corrections without the needed context.

### Stage 3c — Self-hosting cycle (L)

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

## Size

XL aggregate, starts once Lane 1 Stage 1e lands:
- 3a: XL (5 sub-stages: L, S, M, S, S)
- 3b: M
- 3c: L

Build integration buffer into 3c's acceptance, not into calendar.

---

## What's TRUE when Lane 3 completes

A user writes `compiler.dag`, runs the current compiler on it, gets a Rust source file. They compile that Rust with `rustc`. They run the resulting binary on `compiler.dag` again. The Rust output is bit-identical.

The compiler is its own first consumer. The thesis's "causal engine" claim is no longer a slogan — it's a CI gate.

Every error message the compiler emits tells the user not just what's wrong but what to type to fix it. The compiler teaches.

The plan is done.

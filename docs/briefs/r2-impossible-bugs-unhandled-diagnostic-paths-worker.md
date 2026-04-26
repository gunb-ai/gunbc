# T-ImpossibleBugs — Unhandled diagnostic paths via totality-by-omission `(M; per design doc Director-recommendation)`

> **Worker brief.** Reports through Impossible-Bugs Manager (post-R2
> spin-up) / Director (pre-spin-up). T-ImpossibleBugs Goal 4 class 2
> of 3.
>
> **Path: totality-by-omission per partial-op class** — the
> Director-actionable recommendation in
> [`docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md`](t-impossiblebugs-unhandled-diagnostic-paths-design.md)
> §4. NOT predicate-entailment substrate (that's M+, reopens DB-11
> explicitly-closed design). The work is per-class algebra retype +
> per-target realization migration, not new substrate.

## Read first

- **[`docs/briefs/t-impossiblebugs-unhandled-diagnostic-paths-design.md`](t-impossiblebugs-unhandled-diagnostic-paths-design.md)** — design doc; **read §4 Director-actionable recommendation in full** before slicing. Key facts:
  - gunbc already uses totality-by-omission for `force_unwrap` — convention precedent.
  - Predicate-entailment (the "section 2" substrate path) is M+ scope and reopens DB-11's explicitly-closed asymmetric-strip design. **Not the recommended path.**
  - For `/` specifically: closure cost has dropped since original framing — `OrderedRing.div: fn(T, T) -> T` at `dsl/std/algebra.dag:182` is consumed via algebra-Conj dispatch (`infer.rs:3975-3977`); closure is **algebra retype + per-target realization migration**.
- **[`THESIS.md` lines 175, 348-350, 391](../../THESIS.md)** — class definition + the "made total" branch the design doc closes against.
- **[`dsl/std/algebra.dag:182`](../../dsl/std/algebra.dag)** — `OrderedRing.div: fn(T, T) -> T` (the line to retype to total form).
- **[`src/v3/compiler/src/infer.rs:3975-3977`](../../src/v3/compiler/src/infer.rs)** — algebra-Conj dispatch site for `Int / Int`.
- **[`src/v3/spec/rust.dag:832`](../../src/v3/spec/rust.dag)** — `rust_int_div` realization (renders bare `{lhs} / {rhs}`). (`:816` is `rust_int_sub` — verify by grep `rust_int_div` at HEAD.)
- **[`src/v3/spec/go.dag:758`](../../src/v3/spec/go.dag)** — `go_int_div` realization (same shape). (`:742` is `go_int_sub`.)
- **[`src/v3/spec/python.dag:500`](../../src/v3/spec/python.dag)** — `python_int_div` realization (renders via `__v3_idiv` helper at `src/v3/compiler/src/emit/python_target.rs:680`; pinned by `m1_4_emit_python_test.rs:108-109`). (`:486` is inside `python_int_sub`.)
- **[`dsl/std/algebra.dag:477-478`](../../dsl/std/algebra.dag)** — `OrderedRing.quotient` / `OrderedRing.remainder` (separate per-class sub-lane candidates per design doc audit).
- **`feedback_totality_by_omission`** — discipline anchor: partial-op classes close by removing the partial form; coexistence-with-paired-total is the trap.

## Frame

Per design doc §4: the bug class closes by **removing the partial form**. For `/`, this means changing `OrderedRing.div`'s return type from `T` to a typed-split `Result<T, DivError>` carrier where `DivError = DivideByZero | Overflow` preserves the two distinct failure modes (per Slice §2 below). All per-target realizations migrate to construct the typed-split return shape idiomatically.

**Why not pair total + partial?** Coexistence is the **theatre trap** the design doc warns against. Adding `divide_safe` as a separate function alongside an unchanged `div: fn(T,T) -> T` leaves the partial form reachable; closure requires removal.

**Why not predicate-entailment substrate?** Per design doc: M+ scope, reopens DB-11's asymmetric-strip design. Discarded.

**Why not park?** The closure is now small (algebra retype + 3 realizations) and satisfies `feedback_totality_by_omission` directly.

## Slice (per design doc §4 follow-on brief shape)

1. **Audit** every partial operator/function currently reachable from user code:
   - `Int / Int` — primary target (this PR).
   - `[i]` indexing — separate sub-lane row.
   - `force_unwrap` — already total; verify and confirm no regression.
   - `OrderedRing.quotient` / `OrderedRing.remainder` at `dsl/std/algebra.dag:477-478` — separate sub-lane rows. **Verify user-reachability** distinct from `/`; if so, queue separately.
   - Any other partial form per audit. Output: one-row-per-partial-form table in PR body.
2. **Per-row decision** — for `Int / Int`: pick Result-shape `Result<T, DivError>` where `DivError = DivideByZero | Overflow` (or worker-equivalent typed split that preserves distinct failure modes). **NOT a single-error `Result<T, DivideByZero>`** — signed integer division has two structurally distinct failure modes (zero divisor + signed overflow on `MIN / -1`), and collapsing them violates fail-closed C-8 (per `feedback_fail_closed_discipline`: each detectable problem is its own typed Diagnostic). **Option-shape (`Option<T>`) also rejected** for the same reason — `None` carries no failure-mode information. **STOP-AND-ESCALATE for NonZero-typed-input shape** (e.g., `a / nz` rather than `divide_nz(a, nz)`) — that's a per-operand type-variance question deferred to a separate substrate brief.
3. **Algebra retype**: one-line change at `algebra.dag:182` to the total return type. **The total return type carries a typed-split error carrier** (e.g., `DivError`) per Slice §2 — surface the carrier declaration's location (likely `dsl/std/errors.dag` or a sibling under `dsl/std/algebra.dag`).
4. **Per-target realization migration** — each target reshapes from bare division to construct the typed-split Result idiomatically; **must distinguish divide-by-zero from overflow** (do NOT collapse to a single error variant):
   - `rust.dag:832` (`rust_int_div`) — reshapes to explicit branching: `if rhs == 0 { Err(DivError::DivideByZero) } else if /* overflow check */ { Err(DivError::Overflow) } else { Ok(lhs / rhs) }`. Note: `i64::checked_div` collapses both failures to `None`, so it's NOT a one-line `ok_or` — the realization must split the cases. Worker authors the exact Rust idiom; surface in PR.
   - `go.dag:758` (`go_int_div`) — reshapes to Go-idiomatic typed-split Result; same case-split discipline.
   - `python.dag:500` (`python_int_div`) + `src/v3/compiler/src/emit/python_target.rs:680` helper — reshape `__v3_idiv` to return typed-split Result; update test pin at `m1_4_emit_python_test.rs:108-109`. (Python's overflow semantics differ from Rust's — surface the per-target equivalence in PR body.)
5. **Audit fallback path** — `infer.rs:4003-4015` Rust-side primitive scaffold (general fallback for types whose `inhabits` chain doesn't reach an algebra Conj). Slice §1 audit confirms whether any types still resolve `Arithmetic(Div)` through that fallback; close those paths separately if so.
6. **Regression tests:**
   - `Int / Int` returns `Result<Int, DivError>` with both `DivideByZero` and `Overflow` variants reachable.
   - **Distinct typed-split coverage:** `1 / 0` produces `Err(DivideByZero)`; `i64::MIN / -1` produces `Err(Overflow)`. Each failure mode tests independently — collapsing them is a discipline violation per `feedback_fail_closed_discipline` C-8.
   - `let result = a / b; match result { Ok(v) => ..., Err(DivError::DivideByZero) => ..., Err(DivError::Overflow) => ... }` compiles (full match coverage).
   - Existing safe `/` code migrated.
   - Spoofing test: any other partial form (per Slice §1 audit) does NOT accidentally inherit the new total shape.
7. **DB-8 fixed-point bit-identical** for emit output post-realization-migration.

## Acceptance

- [ ] Audit table in PR body enumerating all partial forms reachable from user code.
- [ ] For `Int / Int`: algebra retype to total return; 3 per-target realizations migrated; test pin updated.
- [ ] Typed-split error carrier shape (e.g., `DivError = DivideByZero | Overflow`) recorded with placement (declaration location) and per-target realization rationale. NonZero-typed-input was a STOP, not a worker autonomous path.
- [ ] Fallback path (`infer.rs:4003-4015`) audited; any open paths closed or queued.
- [ ] Regression tests cover positive (`/` returns total) + spoofing + existing-program migration.
- [ ] DB-8 fixed-point bit-identical.
- [ ] Cross-program signal: lane close → Impossible-Bugs Manager → R2 Release Manager. **Note:** this PR closes only `Int / Int`; `[i]` indexing, `quotient`/`remainder`, etc. are sibling per-class sub-lanes. Surface remaining sub-lanes in PR body for follow-up dispatch.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **NonZero-typed-input shape chosen** (`a / nz` operator-syntax rather than `divide_nz(a, nz)` function syntax) — STOP. Per-operand type variance in algebra-operator carrier is a separate substrate brief.
- **Audit reveals additional partial forms not enumerated in design doc** — surface; queue as sibling sub-lanes; do not subsume in this PR.
- **Realization migration breaks emission for an existing target idiom** — surface; this is a target-realization design call, not a worker call.
- **`Result<T, DivError>` requires authoring `DivError` (with `DivideByZero | Overflow` variants) declaration** — verify the typed-split carrier doesn't exist via audit; if not, surface placement decision (`dsl/std/errors.dag`?). Single-error `Result<T, DivideByZero>` shape is explicitly rejected per Slice §2; STOP if any reading drifts back to a single-variant carrier.
- **Asymmetric-operator interaction** — if the totality migration affects symmetric operators (`>`, `<`, etc.) that DB-11 explicitly strips refinements from, surface — the design doc treats those as separate; this PR shouldn't broaden.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- **Not predicate-entailment substrate.** Per design doc, M+ scope, reopens DB-11. Worker DOES NOT author predicate-entailment paths.
- Not authoring all sibling sub-lanes (indexing, quotient, remainder) in one PR — per-class sub-lanes per design doc §4.
- Not extending refinement-checking — orthogonal to this brief.
- Not addressing other T-ImpossibleBugs classes.

## Cross-program note

- **No producer prerequisite** — totality-by-omission needs no new substrate.
- **Producer:** this brief.
- **Consumer:** existing v3 type-checker / per-target emitters.
- **Downstream signal:** R2 Release Manager (Goal 4 partial close — `Int / Int` only; sibling sub-lanes queued separately).
- **Adjacent:** Modeling Manager — `Result<T, E>` shape if not yet first-class in std/. Audit-time check.

## Reporting

Single PR, narrow scope (`Int / Int` totalization). Title: `feat(v3): T-ImpossibleBugs unhandled diagnostic paths — Int/Int totality via OrderedRing.div algebra retype + per-target realization migration`. Body cites this brief + design doc §4 + audit table + algebra retype + 3 realization migrations + DB-8 disposition.

On merge: signal R2 Release Manager (partial Goal 4 close); surface remaining per-class sub-lanes for follow-up dispatch.

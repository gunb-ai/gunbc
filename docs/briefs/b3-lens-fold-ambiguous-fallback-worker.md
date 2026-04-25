# B3 — Lens fold ambiguous unique-candidate fallback → require structural template-formal edge `(S, Tier 0)`

> **Worker brief.** Reports through Director (`zesty-bear-812`).
> Tier 0 fail-closed P3 fix per
> [`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md) §3.
> Independent of B1/B2; dispatch in parallel. **Note**: the file-suffix
> special case elsewhere in `lens_apply.rs` is part of the §0
> identity-carrier class (B4); do not touch it here.

## Read first

- **[`src/v3/compiler/src/lens_apply.rs:132-148`](../../src/v3/compiler/src/lens_apply.rs)** — the site. After failing to find a `BindNode` whose argument's `parameter` is in `formals`, the code falls back to *"if exactly one unique callable bind candidate exists across all arguments, return it."* This is a heuristic: the **structural** fact (this argument's parameter is one of the fold-template's formals) is replaced by an **enumerative** fact (only one candidate exists, so it must be right).
- **[`src/v3/compiler/src/lens_apply.rs`](../../src/v3/compiler/src/lens_apply.rs)** `:118-131` — context: the fold-template + its formals are looked up, and each argument is walked. The `formals.contains(&arg.parameter)` check IS the structural invariant; the fallback admits arguments that don't match it.
- **`fold_template_callable_formals`** + **`monomorph_callable_bind_root`** — supporting helpers; verify they produce the right structural-edge data when called.
- **`feedback_lenses_not_passes`** — *"analyses are lenses over physics; zero heuristics; heuristic = missing physics."* The unique-candidate fallback is the heuristic shape; the structural template-formal edge is the missing physics.
- **`feedback_groundedness_gates_lenses`** — language has primitives + namespacing only; lenses apply by construction. If the fold lens needs a heuristic fallback to run on some programs, the design has a leak.
- **`feedback_fail_closed_discipline`** (C-8) + **`feedback_construction_over_ratchets`**.

## Frame

The fold lens today reasons about which `BindNode` represents the fold's callable step by walking template arguments and checking whether each argument's `parameter` is a fold-template formal. When that check fires, the answer is structurally certain. When the check **doesn't** fire on any argument, the current code shrugs and returns the unique callable bind candidate if there's only one.

The unique-candidate fallback is wrong for two reasons:
1. **It's a heuristic.** Two distinct callables (fold step + an unrelated lambda passed in the same call) collapse to "one unique" in many cases; the heuristic gives a confident-but-wrong answer.
2. **It hides a missing structural edge.** If the formals walk doesn't identify the step, that's the lens detecting that the input program lacks the structural template-formal edge it needs. The right response is **fail closed with a diagnostic**, not "guess the unique candidate."

Per `feedback_lenses_not_passes`: the fix is to require the structural fact (template-formal edge identification) and remove the heuristic. If the fact isn't present in the input DAG, the lens reports a structural-coverage diagnostic and the program is treated as out-of-scope for this lens — **not** silently fabricated through.

## Slice

1. **Confirm structural-fact availability.** Audit: when does `formals.contains(&arg.parameter)` legitimately fail to fire on any argument of a fold call? If it can fail in well-formed v3 programs, the structural edge is missing from the DAG — that's a substrate gap and STOP-AND-ESCALATE; this brief becomes contingent on substrate work.
   - **Expected outcome of audit**: every well-formed fold call has at least one argument whose parameter is in the template's formals. If the audit confirms this, the fallback is dead code masking a bug.
2. **Replace the unique-candidate fallback** at `lens_apply.rs:132-148` with `None` (or a typed lens-coverage-gap diagnostic via `LensApplyError`, if the surrounding code supports it). The function already returns `Option<&BindNode>`; making the fallback `None` should be sufficient.
3. **Audit downstream consumers** of the `Option<&BindNode>` return value. They already handle `None`; the change makes the previously-`Some(unique)` paths now `None`, which means downstream produces a structural-coverage diagnostic (or whatever its existing `None` path does). **If a downstream consumer treats `None` as success, that's a separate fail-open bug — surface for follow-up.**
4. **Regression test**: construct a fold call where the formals-walk fails on every argument but a unique callable bind exists. Assert the lens now returns `None` (or the structural-coverage diagnostic), not the heuristic match.

## Acceptance

- [ ] Fallback removed; function returns `None` (or typed diagnostic) when no argument's parameter is in the template's formals.
- [ ] Audit recorded in PR description: every well-formed fold call structurally identifies the step via the formals edge (or substrate gap surfaced as STOP).
- [ ] Downstream-consumer audit recorded: every consumer of the function's `None` return path is fail-closed.
- [ ] Regression test asserts the new behavior.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.

## STOP-AND-ESCALATE

- **Audit finds well-formed fold calls where the formals walk legitimately fails** — STOP. Substrate gap; this brief reframes as "structural template-formal edge work." Director call.
- **A downstream consumer of the function's return treats `None` as success** — STOP. Fail-open class; surface for follow-up dispatch.
- **The fallback turns out to be load-bearing for some lens-evaluation case (existing tests fail when removed)** — STOP. Surface the case; revise framing.
- **Touching this file pulls in the file-suffix special case** elsewhere in `lens_apply.rs` — STOP. That's B4 territory; do not bundle.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not addressing the `lens_apply.rs` file-suffix special case (§0 identity-carrier class — B4).
- Not refactoring `fold_template_callable_formals` or `monomorph_callable_bind_root`.
- Not introducing a new `LensApplyError` variant unless the existing `Option` shape proves inadequate.

## Reporting

Single PR. Title: `fix(v3): B3 lens fold ambiguous fallback → structural template-formal edge required (Tier 0)`. Body cites this brief + records the formals-walk audit outcome.

On merge: signal Director.

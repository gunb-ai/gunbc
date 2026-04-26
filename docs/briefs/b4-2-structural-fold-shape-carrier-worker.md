# B4.2 — Structural fold-eligibility predicate `(M; B4 Phase 1 #2 of 4 — REVISED 2026-04-26 per royal-badger-32 audit)`

> **Worker brief.** Reports through Substrate Manager (post-R2 spin-up) /
> Director (pre-spin-up). Sub-brief of the
> [B4 Identity-Carrier Substrate Pass program](b4-identity-carrier-substrate-pass.md)
> (merged via #814).
>
> **🔄 REVISED 2026-04-26 by Director.** Original framing
> (`fold_step_formal: Option<DeclarationId>` carrier on `Instantiation`)
> was misdiagnosed: that carrier memoizes a fact already
> structural at `lens_apply.rs:114`
> (`find_fold_step_bind_via_instantiation`) — borderline
> parallel-representation. The §0.4 line-38 bridge actually skips
> on **accumulator/element type eligibility for R1's bounded
> interpreter**, NOT on step-formal binding. The two carriers
> address different questions; landing the original carrier would
> NOT close §0.4. PR #834 closed as misframed; this brief is
> re-authored against the correct dissolution shape.

## Read first

- **[`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md)** — parent program brief.
- **[`src/v3/compiler/src/lens_apply.rs:14`](../../src/v3/compiler/src/lens_apply.rs)** — `is_fold_instantiation` already structurally identifies fold instantiations via `dag.std_list_fold_decl() == Some(*template)`.
- **[`src/v3/compiler/src/lens_apply.rs:22-39`](../../src/v3/compiler/src/lens_apply.rs)** — `fold_site_skips_d1_monomorph_list_fold_path` doc comment. **The line-38 bridge exists because algebra.dag folds use `List<SymbolicCost>` accumulators while R1's bounded interpreter only certifies `Int` accumulator + `Behavior` elements.** This is the actual §0.4 dissolution target.
- **[`src/v3/compiler/src/lens_apply.rs:114`](../../src/v3/compiler/src/lens_apply.rs)** — `find_fold_step_bind_via_instantiation`: already-structural step-Bind recovery via `fold_template_callable_formals(dag, *template)` walked against `arguments`. **Do not duplicate this with a memoization carrier** (per `feedback_parallel_representation_debt`).
- **[`src/v3/compiler/src/lens_apply.rs:38, :372-383`](../../src/v3/compiler/src/lens_apply.rs)** — the §0.4 sites. Line 38 is the load-bearing bridge (accumulator-type skip); :372-383 is the same skip applied to a sibling fold path.
- **[Helper's own dissolution-trigger doc](../../src/v3/compiler/src/lens_apply.rs)** — at the line-38 site, the inline comment names: *"R1-certified step shape, or third Transform operand for the step so arity matches eval_std_fold."* Either is the dissolution target — neither is `fold_step_formal`.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — substrate authority.
- **`feedback_parallel_representation_debt`** — do not memoize already-structural facts as new substrate fields.
- **`feedback_audit_adjacent_authority_first`** — grep + read call sites before designing carriers.
- **`feedback_design_before_implement`** — pre-flight audit prevents 1382-site propagation of a contested shape.

## Frame (revised)

Per the audit at `lens_apply.rs:22-39`: the `span.file.ends_with("std/algebra.dag")` check at line 38 exists because the bounded R1 interpreter at `eval_std_fold` only certifies certain accumulator/element type shapes. `algebra.dag` folds use `List<SymbolicCost>` accumulators that the interpreter cannot evaluate. The skip prevents the bounded interpreter from running on incompatible shapes; it falls back to a different path.

The **actual §0.4 dissolution** is a structural eligibility predicate: given a fold `Instantiation`, can the bounded R1 interpreter certify its accumulator + element types? If yes, run; if no, skip via the fallback. The predicate is a **structural query against the resolved type arguments**, NOT a new substrate field.

**The right shape is likely no new substrate.** Possible paths:

1. **Pure structural query (preferred).** A helper `interpreter_eligible_fold(dag, instantiation) -> bool` that resolves the accumulator + element type arguments and matches against the bounded interpreter's supported set. No substrate change. The line-38 bridge becomes a structural call.
2. **Structural witness carrier (fallback if pure-query has surprising structural holes).** Optional eligibility tag on the fold's `Instantiation` populated at lowering when the type args resolve to interpreter-supported shapes. Only justified if the pure-query path doesn't compose cleanly with existing dispatch.

Per `feedback_audit_adjacent_authority_first`: **start with the audit. The audit may show option 1 closes §0.4 with zero new substrate.**

## Pre-author authority audit (mandatory)

Before designing any new substrate, grep for and read carefully:

1. **`is_fold_instantiation` (`lens_apply.rs:14`)** — already-structural; reuse don't duplicate.
2. **`fold_template_callable_formals` + `find_fold_step_bind_via_instantiation` (`lens_apply.rs:114`)** — already-structural step-Bind recovery; **do NOT add a memoization carrier**.
3. **`eval_std_fold` and the bounded interpreter** — what types does it actually certify? What's the supported set? The audit must produce a precise list of the eligible accumulator+element type shapes.
4. **Line-38 + line-372-383 skip paths** — what's the fallback? Is the fallback the right path for ineligible folds, or does the bridge mask a different bug?
5. **Existing structural query helpers** in `lens_apply.rs` and `infer.rs` that resolve type arguments — reuse rather than re-implement.

**If audit shows the eligibility predicate is fully derivable from existing structure**, the implementation lands as a helper function migration with NO substrate change. Surface that finding.

**If audit shows an irreducible structural hole**, a minimal carrier may be justified — but the carrier must be JUSTIFIED INDEPENDENTLY (per `feedback_design_before_implement`), not added speculatively.

## Slice (audit-conditional)

Per audit findings:

1. **Pure-query path (most likely):**
   - Author `interpreter_eligible_fold(dag, instantiation)` (or worker-equivalent name) as a helper that resolves the accumulator + element type args structurally and matches against the bounded interpreter's supported set. Per `feedback_lenses_not_passes`: structural fact, zero heuristics.
   - Replace `span.file.ends_with("std/algebra.dag")` at `lens_apply.rs:38` and `:372-383` with calls to the new structural helper.
   - Regression test: a user-authored fold over an interpreter-eligible accumulator type (e.g., `Int`) outside `std/algebra.dag` runs through the bounded path; a user-authored fold over a `List<SymbolicCost>`-style accumulator is correctly skipped via the structural predicate.

2. **Carrier path (only if audit justifies):**
   - Surface the audit finding that explains why a pure structural query doesn't work.
   - Land minimal carrier; populate at lowering; same-PR consumer migration.

Either way: **single PR; same-PR consumer migration**, per `feedback_parallel_representation_debt`.

## Acceptance

- [ ] Audit receipt recorded in PR body: `eval_std_fold`'s supported type set; existing structural helpers consulted; pure-query feasibility verdict.
- [ ] §0.4 sites at `lens_apply.rs:38, :372-383` removed; replaced with structural eligibility query.
- [ ] Regression test: structural-only fold dispatch (eligible fold runs through bounded path; ineligible fold correctly skips).
- [ ] **NO new substrate carrier** unless audit produces explicit justification + PR body documents the structural hole.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] No replacement sentinel string introduced (per `feedback_no_textual_enforcement_bridges`).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **Audit reveals the bounded interpreter's supported set is not structurally enumerable** (e.g., it dispatches via runtime hooks not visible at type-check time) — STOP. Surface to Substrate Manager; the §0.4 dissolution may need substrate-deeper work.
- **Pure-query path requires more than reading existing structural helpers** (e.g., needs lowering-time computation that doesn't compose) — STOP and surface the hole; carrier path may then be justified.
- **Audit reveals additional `span.file`-based dispatch in the lens evaluation chain** that wasn't named in #810's §0.4 — surface as separate Phase 2 site (route to B4 Phase 2 queue).
- **DB-8 drifts on a fold path that was previously routing through the file-suffix bridge** — surface; may indicate the bridge masked a real eligibility bug. Do NOT silently change semantics.
- **Worker concludes the bridge is structurally undissolvable** — STOP. The brief reframes as substrate-deeper work; cross-program escalation to Substrate Manager.

## Non-goals

- **NOT adding `fold_step_formal` carrier on `Instantiation`** (that was the misframing of the original brief — borderline parallel-rep with `find_fold_step_bind_via_instantiation`).
- **NOT extending `eval_std_fold`'s supported type set** in this lane (that's substrate-deeper work; out of scope).
- Not extending `std.list.fold` semantics or template parameters.
- Not touching B3's lens fold ambiguous fallback (already merged via #821).
- Not addressing other §0 sites (those are B4.1/B4.3/B4.4).

## Cross-program note

- **Producer:** Substrate Manager (T-Substrate / B4 Phase 1 #2) — but likely no new substrate; this lane may close as helper-migration-only.
- **Consumer:** lens-apply runtime — same-PR migration.
- **No cross-program consumer signal** — lens-apply only.

## Pre-flight audit credit

Original brief framing was caught by `royal-badger-32` at PR #834 pre-flight audit; #834 closed as misframed. This brief is re-authored against the correct dissolution target. The pre-flight audit discipline royal-badger-32 demonstrated (audit carrier shape against helper's own doc + actual call sites BEFORE propagating) is the right `feedback_design_before_implement` shape — saved as a feedback memory.

## Reporting

Single PR. Title: `feat(v3): B4.2 structural fold-eligibility — replace span.file.ends_with("std/algebra.dag") with structural type-arg query`. Body cites this brief + B4 program brief + audit receipt + decision (pure-query vs carrier path) + DB-8 disposition.

On merge: signal Substrate Manager / Director; B4 Phase 1 carrier #2 of 4 closes (with note that #2 may have landed without new substrate, per audit).

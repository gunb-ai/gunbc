# B1 — Go `UnknownVariant` fabrication → `EmitError::VariantParentNotFound` `(S, Tier 0)`

> **Worker brief.** Reports through Director (`zesty-bear-812`).
> Tier 0 fail-closed P3 fix per
> [`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md) §3.
> Independent of B2/B3; dispatch in parallel.

## Read first

- **[`src/v3/compiler/src/emit.rs:1456-1464`](../../src/v3/compiler/src/emit.rs)** — the site. `variant_parent_info(self.dag, variant_id)` returns `Option<(String, String)>`; on `None` the code falls back to `declaration(variant_id).name`, then to the literal string `"UnknownVariant"`. Both fallbacks fabricate plausible Go output instead of failing closed when the variant has no resolvable parent enum.
- **[`src/v3/compiler/src/emit.rs:2989`](../../src/v3/compiler/src/emit.rs)** — `variant_parent_info` definition; returns `None` when `Disjunction` parent isn't found. Confirms the failure mode that today silently emits a bad identifier.
- **[`src/v3/compiler/src/emit.rs:1791`](../../src/v3/compiler/src/emit.rs)** + **[`src/v3/compiler/src/emit/rust_target.rs:4325`](../../src/v3/compiler/src/emit/rust_target.rs)** — sibling `variant_parent_info` callers; both already pattern-match `let Some(..) = .. else { … }` and fail explicitly. The Go path is the outlier.
- **[`src/v3/compiler/src/emit/rust_target.rs`](../../src/v3/compiler/src/emit/rust_target.rs)** `:54` — `EmitError` definition. Use the **existing fail-closed shape** (`MissingRealizationMeta`, `MissingTargetSyntax`, `MalformedRealization`); add a new variant `VariantParentNotFound { variant_id: DeclarationId }` (or equivalent — worker picks; surface choice in PR description). No parallel error taxonomy.
- **`feedback_fail_closed_discipline`** + **`feedback_compile_time_errors`**.

## Frame

This is **not a scaffold** — it's a fabrication leak. The Go emitter today, when a variant declaration has no `Disjunction` parent, silently emits the literal token `UnknownVariant` (or the variant's own name, which still doesn't compile against any real Go type). Downstream Go compilation fails opaquely; the bug surfaces as "Go output references undefined identifier" rather than "compiler couldn't find variant parent."

The fix is one-site, structural, and parallels existing `variant_parent_info` callsites. Replace the `unwrap_or_else` chain with an explicit `Result` early return.

## Slice

1. Add `EmitError::VariantParentNotFound { variant_id: DeclarationId }` (or worker-equivalent shape) to the existing `EmitError` enum. Coproduct dissolution receipt only if the variant introduces a new top-level error class — likely **purely additive**, no receipt required.
2. At `emit.rs:1456-1464`, replace:
   ```rust
   let variant_name = variant_parent_info(self.dag, variant_id)
       .map(|(_, variant_name)| variant_name)
       .unwrap_or_else(|| {
           self.dag.declaration(variant_id).name.clone()
               .unwrap_or_else(|| "UnknownVariant".to_string())
       });
   ```
   with the `let Some(..) = .. else { return Err(..) }` shape used at `emit.rs:1791` and `rust_target.rs:4325`. The outer function's signature already returns `Result<_, EmitError>`.
3. Verify call-graph: every caller of the Go branch-emission path already propagates `Result`. No signature change needed.
4. **Regression test — optional, deferrable; skip surfaces a substrate signal.** A unit test that constructs a `Dag` with a variant declaration lacking a `Disjunction` parent would assert the new error; however, `emit.rs` has no existing `#[test]` precedent and emit testing today happens via integration fixtures (`tests/integration/*`). If a hermetic unit test is straightforward, add it. **If test setup requires building a novel Dag-construction harness, skip and (a) add a ROADMAP debt row for emit-side hermetic-unit-test infrastructure (or cite an existing one) with named dissolution trigger, and (b) reference that row in the PR body.** The fact that this test would require novel harness work is itself a `feedback_emitter_workaround_is_gap_symptom` signal — emit-side hermetic-unit-test infrastructure is the missing substrate. The skip becomes productive (surfaces a substrate gap for dispatch) rather than corrosive (does not set a "tests are optional" precedent). The structural fail-closed at step 2 is the load-bearing change.

## Acceptance

- [ ] `EmitError::VariantParentNotFound` (or equivalent) added; existing `EmitError` shape preserved.
- [ ] `emit.rs:1456-1464` no longer falls back to a literal string; fails closed via `Result`.
- [ ] Regression test asserts the new error variant is returned for the missing-parent case **OR** PR adds a ROADMAP debt row (or cites an existing one) for emit-side hermetic-unit-test harness infrastructure with named dissolution trigger, and references it in the PR body.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically (no Go-emit changes for well-formed inputs).

## STOP-AND-ESCALATE

- **Existing `EmitError` shape doesn't fit the new variant cleanly** (e.g., the variant requires carrying parent context beyond `DeclarationId`) — STOP. Surface design call.
- **Audit reveals additional fail-open string fallbacks in the Go emitter** — STOP. This brief covers only the `:1456-1464` site; sibling fallbacks are out of scope but should be reported for follow-up dispatch.
- **DB-8 drifts** — STOP immediately.
- **`variant_parent_info` returning `None` turns out to be a legitimate runtime case in any well-formed program** — STOP. The diagnosis says it's always a compiler bug; if execution refutes that, the brief's framing needs revision.

## Non-goals

- Not refactoring `variant_parent_info`'s shape.
- Not addressing the file-suffix special case in `lens_apply.rs` (that's the §0 identity-carrier class — B4).

## Reporting

Single PR. Title: `fix(v3): B1 Go UnknownVariant fabrication → EmitError::VariantParentNotFound (Tier 0 fail-closed)`. Body cites this brief + names the dissolution: *"Go emitter no longer fabricates plausible identifiers when variant parent resolution fails."*

On merge: signal Director.

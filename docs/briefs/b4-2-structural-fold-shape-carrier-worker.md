# B4.2 — Structural fold-shape carrier `(M; B4 Phase 1 #2 of 4)`

> **Worker brief.** Reports through Substrate Manager (post-R2 spin-up) /
> Director (pre-spin-up). Sub-brief of the
> [B4 Identity-Carrier Substrate Pass program](b4-identity-carrier-substrate-pass.md)
> (merged via #814). Replaces §0.4 — `lens_apply.rs` `span.file.ends_with("std/algebra.dag")`
> fold-skip — with a structural template-formal carrier.

## Read first

- **[`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md)** — parent program brief; Phase 1 / Phase 2 framing; the 8 surface dissolution sites table.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority where the carrier lands.
- **[`src/v3/spec/v3_l1.dag:69`](../../src/v3/spec/v3_l1.dag)** — `DeclarationRef` sentinel meta-type (already-landed authority; B4.1 consumed it). Likely shape for the fold-step formal field.
- **[`src/v3/compiler/src/lens_apply.rs:38, :372-383`](../../src/v3/compiler/src/lens_apply.rs)** — the §0.4 sites: `span.file.ends_with("std/algebra.dag")` checks. The dispatch goes from "is this in algebra.dag?" → "does this Instantiation carry a fold-step formal?".
- **[`src/v3/compiler/src/lens_apply.rs::find_fold_step_bind_via_instantiation`](../../src/v3/compiler/src/lens_apply.rs)** — the production consumer that recovers the fold step bind. B3 (#821) already removed the unique-candidate fallback; this brief extends with the structural carrier.
- **`feedback_audit_adjacent_authority_first`** — grep before designing; B4.1 found `DeclarationRef` already existed.
- **`feedback_no_textual_enforcement_bridges`** — no replacement sentinel strings.
- **`feedback_construction_over_ratchets`** — no parity assertions as primary enforcement.
- **`feedback_parallel_representation_debt`** — if audit finds existing authority, consume it; don't add a sibling.

## Frame

Today `lens_apply.rs:38, :372-383` skips fold-template walks via `span.file.ends_with("std/algebra.dag")` — file-suffix dispatch, not structural. The substrate gap: fold-template instantiations don't structurally record which template-formal binds the step closure. Lens consumers reach for the file suffix as a stand-in for "this is the standard fold path, the step formal is well-defined."

The fix is to add a typed carrier on fold-template instantiations that names the step-formal `DeclarationRef`. The dispatch becomes a structural query (does this Instantiation carry a `fold_step_formal`?), and the file-suffix check disappears.

## Pre-author authority audit (mandatory)

Before designing the new carrier, **grep `src/v3/std/` + `src/v3/spec/`** for any existing fold-template-formal authority. Search terms:

- `fold_step_formal`, `template_formal`, `step_formal`
- existing fields on `Instantiation` declaration in `src/v3/std/substrate.dag` that carry `DeclarationRef` for template-formal positions
- existing `Arrow.body` shape on `std.list.fold` that names the step formal structurally

**If audit reveals existing authority, reframe as consumer migration, not carrier landing.** B4.1's consumer-migration shape is the precedent: cite the existing authority, name what's missing as "consumer not yet routing through it," and skip the substrate-port step.

Surface the audit receipt explicitly in the PR body — what was searched, what was found.

## Slice (assume audit shows substrate gap)

1. **Land the typed carrier in `src/v3/std/substrate.dag`.** Likely shape: an optional field on `Instantiation` that records `fold_step_formal: DeclarationRef?` for fold-template instantiations. Worker picks shape; surface choice + reasoning in PR body. Coproduct dissolution receipt for any new variant per `feedback_coproduct_dissolution`.
2. **Mirror in Rust** (`src/v3/compiler/src/dag.rs`) — substrate.dag is canonical authority, Rust mirrors. Both updated, builder defaults to `None`.
3. **Lower the carrier** — when fold-template instantiations are constructed (presumably in `src/v3/compiler/src/lower.rs` at the `std.list.fold` callsite lowering), populate the field structurally with the template-formal that binds the step closure.
4. **Replace the `span.file.ends_with("std/algebra.dag")` checks** at `lens_apply.rs:38` and `lens_apply.rs:372-383` with structural queries against the carrier. The dispatch goes from path-suffix → "does this Instantiation carry a fold-step formal?".
5. **Regression test:** a fold-template instantiation whose source file would NOT match `std/algebra.dag` (e.g., user-authored fold over a custom monoid declaration in a `tests/fixtures/*.dag` file) — assert lens evaluation succeeds via the structural path, not the file-suffix shortcut. Hermetic per `TESTING.md`. Precedent for emit-side test harness: `emit.rs:3124` `#[cfg(test)] mod tests`. Lens-apply has its own test module — reuse precedent.

## Acceptance

- [ ] §0.4 `span.file.ends_with("std/algebra.dag")` sites in `lens_apply.rs:38, :372-383` removed.
- [ ] Lens fold path runs end-to-end via structural carrier; no replacement sentinel string introduced (per `feedback_no_textual_enforcement_bridges`).
- [ ] Substrate carrier lives in `src/v3/std/substrate.dag` with Rust mirror parity.
- [ ] Authority audit receipt recorded in PR body (what was searched, what was found, or which existing authority is being consumed).
- [ ] Coproduct dissolution receipt for any new variant.
- [ ] Regression test asserts structural-only fold dispatch (custom-monoid fold over user-authored declaration outside `std/algebra.dag`).
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas (if any new generated module).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note in PR body.

## STOP-AND-ESCALATE

Surface to Substrate Manager (post-spin-up) / Director (pre-spin-up):

- **Audit reveals more than one fold-template authority** (e.g., `std.list.fold` and `std.tree.fold` both need this carrier) and they don't share a substrate parent — that's a substrate-design call, not a B4.2 implementation call.
- **The fold-step formal isn't structurally identifiable at lowering time** for some legitimate fold path (e.g., curried/partial-applied folds where the formal-edge isn't yet bound). The brief reframes as substrate-deeper work.
- **DB-8 drifts** — STOP immediately.
- **A consumer requires more than the carrier provides** (e.g., needs to know not just *which* formal but *which* step shape) — surface; carrier might need extension.
- **Audit reveals `Instantiation` field shape conflicts with B4.1's `DeclarationRef` consumer pattern** — consult B4.1 author for cross-sub-brief substrate alignment.

## Non-goals

- Not extending `std.list.fold` semantics or template parameters.
- Not touching B3's lens fold ambiguous fallback (already merged via #821) — B3's removal of unique-candidate heuristic is the prerequisite that exposes this substrate gap.
- Not addressing other §0 sites (those are B4.1/B4.3/B4.4).
- Not replacing the file-suffix check with another string-keyed dispatch (per `feedback_no_textual_enforcement_bridges`).

## Cross-program note

- **Producer:** Substrate Manager (T-Substrate program / B4 Phase 1 #2).
- **Consumers:** lens-apply runtime; eventually any third-party fold-template (out of scope for B4.2).
- **No cross-program consumer signal** — the carrier is consumed by lens-apply same-PR; no other program waits on this lane.

## Reporting

Single PR. Title: `feat(v3): B4.2 structural fold-shape carrier — replace span.file.ends_with("std/algebra.dag") with structural template-formal edge`. Body cites this brief + B4 program brief + records the authority audit + coproduct receipt + lowering call sites updated.

On merge: signal Substrate Manager (post-spin-up) / Director (pre-spin-up); B4 Phase 1 carrier #2 of 4 lands. B4.5–B4.12 Phase 2 site dissolution for §0.4 becomes mechanical follow-up.

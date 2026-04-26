# B4.4 — Structural extdeps-fixture-set carrier `(S/M; B4 Phase 1 #4 of 4)`

> **Worker brief.** Reports through Substrate Manager (post-R2 spin-up) /
> Director (pre-spin-up). Sub-brief of the
> [B4 Identity-Carrier Substrate Pass program](b4-identity-carrier-substrate-pass.md)
> (merged via #814). Replaces §0.8 — `EXTDEPS_BOOTSTRAP_FIXTURES` hardcoded Rust list —
> with a typed declaration in `src/v3/std/`.

## Read first

- **[`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md)** — parent program brief.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority.
- **[`src/v3/spec/v3_l1.dag:69`](../../src/v3/spec/v3_l1.dag)** — `DeclarationRef` (consumed by B4.1; likely shape for fixture entries).
- **[`src/v3/compiler/src/bootstrap.rs::std_fixtures`](../../src/v3/compiler/src/bootstrap.rs)** — production consumer that today reads `EXTDEPS_BOOTSTRAP_FIXTURES` constant.
- **[`src/v3/compiler/src/bootstrap_regen_fresh.rs`](../../src/v3/compiler/src/bootstrap_regen_fresh.rs)** — regen host; runs **before** the bootstrap Dag is loadable. Material constraint: the regen host can't read a substrate authority that requires a loaded Dag. This is the **pre-promotion constraint** worth surfacing explicitly.
- **`feedback_audit_adjacent_authority_first`** — grep before designing.
- **`feedback_parallel_representation_debt`** — DO NOT keep parallel Rust-const + substrate-authority unless the regen-host pre-promotion constraint genuinely forces it; surface as tracked debt with named ROADMAP dissolution trigger if so.
- **`feedback_construction_over_ratchets`** — parity-by-runtime-assertion is a ratchet, not dissolution; only acceptable if the dissolution trigger is named in ROADMAP.

## Frame

`bootstrap.rs::std_fixtures()` consumes a hardcoded Rust list `EXTDEPS_BOOTSTRAP_FIXTURES`. The §0.8 site. Substrate gap: the extdeps fixture set should be a typed declaration in `src/v3/std/` (or `src/v3/spec/`), not a Rust constant.

## Pre-author authority audit (mandatory)

Before designing the new carrier, **grep `src/v3/std/` + `src/v3/spec/`** for any existing fixture-set / bootstrap-set carrier. Search terms:

- `BootstrapFixture`, `FixtureSet`, `bootstrap_fixture_authority`, `extdeps_bootstrap`
- existing typed fixture declarations consumed by bootstrap

The fixtures have `DeclarationId`s once loaded — the question is whether a typed `List<DeclarationRef>` (or equivalent) declaration already exists or needs to be authored.

**If audit reveals existing authority, reframe as consumer migration.**

## Slice (assume audit shows substrate gap)

1. **Land typed `extdeps_bootstrap_fixture_authority` (or worker-equivalent) declaration in `src/v3/std/extdeps_bootstrap_fixtures.dag`** (or appended to existing `src/v3/std/substrate.dag` — worker picks; surface choice). Element shape: typed entries naming each fixture's virtual path (e.g., `virtual_path: String` per row), with the path resolving to a `DeclarationRef` post-load. Coproduct dissolution receipt if any new variant lands.
2. **Migrate `bootstrap.rs::std_fixtures()` to read from the typed declaration** instead of the hardcoded Rust list — at the latest possible point in the bootstrap sequence, so the substrate authority is loaded before its consumer fires.
3. **Pre-promotion constraint disposition.** Shape (a) is the only autonomous worker path; shape (b) is a cross-manager design escalation, NOT a worker call:
   - **(a) Single authority (autonomous worker path)**: regen host reads the substrate authority directly via a minimal sub-loader that loads only the fixture-set declaration before the full bootstrap. **Worker authors this without further escalation if feasible.**
   - **(b) Authority + tracked debt** — **NOT autonomous**: if the audit shows shape (a) is genuinely infeasible (e.g., circular dependency, build-time computation requirement), **STOP-AND-ESCALATE to Substrate Manager** for a regen-host-loader sub-lane decision. Per `feedback_construction_over_ratchets` + `feedback_parallel_representation_debt`, the parallel-Rust-const + substrate-authority shape is **only acceptable** with explicit Substrate Manager approval AND a named ROADMAP dissolution trigger. **Do not author shape (b) without that approval citation in the PR body.**
4. **Replace `EXTDEPS_BOOTSTRAP_FIXTURES` Rust constant** — delete (shape a) or narrow to regen-host-only (shape b, only if Substrate Manager approval citation present). Surface the disposition in PR body.
5. **Regression test:** bootstrap output is bit-identical (DB-8 must converge); SG-0 census deltas covered if any.

## Acceptance

- [ ] `EXTDEPS_BOOTSTRAP_FIXTURES` Rust constant deleted (shape a) **OR** narrowed to regen-host-only with **explicit Substrate Manager approval citation + named ROADMAP debt row** in PR body (shape b).
- [ ] Typed `extdeps_bootstrap_fixture_authority` (or equivalent) declaration lives in `src/v3/std/`.
- [ ] `bootstrap.rs::std_fixtures()` reads the typed authority.
- [ ] Authority audit receipt recorded in PR body.
- [ ] Pre-promotion constraint disposition recorded explicitly. Shape (a) is the autonomous default; shape (b) requires the Substrate Manager approval citation per Slice §3.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] SG-0 census deltas covered.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **The fixtures are non-trivially ordered, conditional, or require build-time computation** that doesn't lower cleanly — surface for design call.
- **Shape (a) is genuinely infeasible** (circular dependency, build-time computation required, etc.) — STOP and escalate to Substrate Manager for a regen-host-loader sub-lane decision. **Do not author shape (b) without explicit Substrate Manager approval.** Per PM tightening on #836: shape (b) cannot be a worker autonomous-path call.
- **Shape (b) approved by Substrate Manager but ROADMAP debt row would need to track an indefinite dissolution** — re-escalate; permanent parallel-representation is not acceptable even with manager approval.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not redesigning the bootstrap sequence beyond what's needed for B4.4 disposition.
- Not migrating other Rust constants in `bootstrap.rs` (those are separate program work, possibly Pure Bootstrap Manager territory).
- Not addressing other §0 sites.

## Cross-program note

- **Producer:** Substrate Manager (T-Substrate / B4 Phase 1 #4).
- **Consumer:** bootstrap loader (same-PR migration).
- **Cross-program coordination:** if shape (b) lands a ROADMAP debt row for full dissolution, the long-term consumer is **Pure Bootstrap Manager** (post-promotion full-substrate-load lane). Heads-up to PB Manager at landing.

## Reporting

Single PR. Title: `feat(v3): B4.4 structural extdeps-fixture-set carrier — typed authority replaces EXTDEPS_BOOTSTRAP_FIXTURES Rust constant`. Body cites this brief + B4 program brief + records authority audit + pre-promotion-constraint disposition (a or b) + ROADMAP debt row reference if shape (b).

On merge: signal Substrate Manager / Director; B4 Phase 1 carrier #4 of 4 lands. **All four Phase 1 carriers complete; Phase 2 mechanical site dissolutions become dispatchable.**

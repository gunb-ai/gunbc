# extdeps Loader-Close — Worker Brief `(S, substrate-capability unblock)`

> **Director ad-hoc dispatch.** Cross-program substrate-capability work
> unblocking R2 Grounding's Engine-Phase-1 sharpened-(b) re-dispatch
> (per [Engine-Phase-1 audit escalation, PR #768](https://github.com/gunb-ai/gunbc/pull/768)).
> Reports back to Director (`zesty-bear-812`); not under a standing
> manager. Cross-program coordination heads-up sent to Grounding Manager
> (`crisp-seal-366`) and Zero-Floor Manager (`stern-swift-335`) at
> dispatch.

## Read first

- **[`docs/briefs/t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md)** — the audit that surfaced this gap. Cites `bootstrap.rs:14-19` as the framing decision being revisited.
- **[`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs)** — current bootstrap; `Dag::new()` loads four authority sets (`std_fixtures`, `STAGED_FILES`, `V3_SPECS`, `COMPILER_FILES`). Header comment at `:14-19`: *"Production bootstrap does not inject target-language realizations."* That comment is being revisited in this PR.
- **[`dsl/extdeps/languages/rust/primitives.dag`](../../dsl/extdeps/languages/rust/primitives.dag)** — the file that must become loadable. Contains `rust_pilot_primitives: List<RustPrimitive>` declarations (`RustPrimitive` is sum type partitioned into `IntegerPrimitive | NonIntegerPrimitive` per pilot brief / codex P2 adjudication on PR #765).
- **[`src/v3/compiler/build.rs`](../../src/v3/compiler/build.rs)** — generated-snapshot pattern (`extdeps_generated`, `gunbc_generated`, etc.). Reference for adding a new fixture-set entry.
- **[`docs/briefs/pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md)** — sibling program; non-goals revised under 0-floor (PR #770). Worth knowing PB-1 will eventually absorb extdeps as a fifth fixture-set extension after the loader-close lands.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)** — same governing rules.

## Frame

The Engine-Phase-1 audit found that `dsl/extdeps/languages/rust/primitives.dag` is not in any bootstrap fixture set. Engine cannot consume `rust_pilot_primitives` symbolically without a public accessor. Without this loader-close, Engine collapses to mirroring (forbidden per its brief), and the Engine-Phase-1 implementation stays parked.

This work is the **smallest unblocking step** (Route 1 per Director routing decision). Sharpened-(b) Engine — sibling crate consuming `.dag` declarations symbolically — becomes possible after this lands. Route 2 (full bundling of loader + list-body emission + heterogeneous-variant match) is not in scope here; this lane is loader-close only.

## Five consumer-side requirements (bake in upfront)

Per Grounding Manager's escalation doc §"Manager-side input for the loader-close brief", these must all be satisfied:

1. **Public accessor returns parsed `Declaration` walkable structurally.** Engine consumes `rust_pilot_primitives` as a `Declaration`-shaped value with structural walk semantics (matching how other bootstrap-loaded declarations are consumed). Not a string; not a private internal handle.
2. **Stable shape Engine doesn't reach into private `Dag` internals.** The accessor must be a public API surface; consuming Engine code must not need `pub(crate)` or unsafe reach-throughs to walk the loaded data.
3. **Stale `bootstrap.rs:14-19` comment revisited in same PR.** That comment asserts production bootstrap doesn't load target-language realizations. This PR contradicts that assertion; the comment must be updated to reflect the new state (which realizations load, why, and what the boundary is now). Don't leave contradictory documentation.
4. **SG-0 stance picked and documented.** This work touches `bootstrap.rs` (SG-0-ratcheted hand-Rust). Two acceptable framings, document the choice in PR body:
   - **Ratchet bump:** small directly-justified increment to the SG-0 census (load-bearing thesis claim + Director-sanctioned).
   - **Transitional shape:** the loader logic lands in a form PB-1 absorbs cleanly when PB-1's data-bootstrap of std_fixtures / STAGED_FILES extends to include extdeps as a fifth set. Pick this if the loader can be authored to fit PB-1's emerging pattern.
5. **Coverage scope explicitly bounded.** Just `dsl/extdeps/languages/rust/primitives.dag` for this PR (smallest unblock for Engine-Phase-1)? Or all `dsl/extdeps/languages/*/*.dag` (covers Python/Go targets too, which Grounding's full-reference lanes will eventually need)? Pick + document. Smaller (rust only) is fine if it cleanly extends; larger (all extdeps languages) is fine if the loader pattern naturally generalizes.

## Slice — extdeps loader close

**Goal:** make `dsl/extdeps/languages/rust/primitives.dag` (at minimum; per req 5, optionally other extdeps language files) loadable into the bootstrapped Dag with a public accessor that downstream consumers can walk structurally.

**Round-trip:**

1. Add the file(s) per req 5 to a bootstrap fixture set. Could be a new fifth set (`EXTDEPS_LANGUAGE_FILES` or similar), or an extension of an existing set if cleaner. Worker's call.
2. Implement public accessor (req 1, req 2). Naming convention should match existing accessors in `Dag` API surface; consult `bootstrap.rs` and `lib.rs` for prior art.
3. Update `bootstrap.rs:14-19` comment per req 3.
4. Write integration test asserting:
   - `Dag::new()` loads `rust_pilot_primitives` without error
   - Public accessor returns a `Declaration`-shaped value with the correct identity
   - Walking the value structurally (e.g., enumerating list items) returns the expected count of `RustPrimitive` entries

## Acceptance

- [ ] All 5 consumer-side requirements satisfied + documented in PR body.
- [ ] `dsl/extdeps/languages/rust/primitives.dag` loadable into bootstrap (per req 5 scope).
- [ ] Public accessor for `rust_pilot_primitives` exists; Engine sharpened-(b) consumer-shape assertion holds (Engine team verifies post-merge).
- [ ] `bootstrap.rs:14-19` comment updated to reflect new boundary (req 3).
- [ ] SG-0 stance documented in PR body (req 4).
- [ ] Integration test passes: `Dag::new()` loads + accessor returns walkable structure.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean (pre-push hook will enforce).
- [ ] DB-8 `self_host_fixed_point` still converges bit-identically (the no-compromise gate).

## STOP-AND-ESCALATE

Surface to Director (`zesty-bear-812`); do not absorb scope.

- **If the SG-0 stance choice (req 4) reveals the loader logic genuinely cannot fit PB-1's emerging pattern** (i.e., transitional-shape isn't viable, ratchet-bump is the only path) — STOP. Director needs to confirm ratchet bump explicitly before the PR commits to it.
- **If req 5's coverage scope (rust-only vs all extdeps languages) reveals divergent loader patterns per language** (e.g., rust loads cleanly but python/go need extension extension) — STOP. Surface the divergence; pick rust-only and re-dispatch other languages per their needs.
- **If the public accessor shape (req 1, req 2) requires extending `Dag` API surface beyond what existing accessors do** — STOP. Surface the extension proposal; Director coordinates with Zero-Floor Manager on whether the extension belongs in PB-Substrate scope.
- **If `bootstrap.rs:14-19` comment update (req 3) reveals the framing change has cross-doc implications** (e.g., other authority docs assert "no target-language realizations in bootstrap") — STOP. Surface the cross-doc consistency check; Director routes to docs-cascade if needed.
- **If DB-8 fixed-point drifts** — STOP immediately. Same no-compromise gate as PB-1.

## Non-goals

- **Not implementing Engine-Phase-1.** Engine re-dispatches against sharpened-(b) after this loader closes; that's separate work owned by R2 Grounding Manager.
- **Not extending PB-1's pattern to extdeps as a fifth set.** PB-1 absorbs extdeps post-this-loader (per non-goal-inversion in PR #770); not this PR's job.
- **Not authoring the bootstrap.dag workflow.** That's PB-Bootstrap-Process scope under Zero-Floor Manager.
- **Not migrating other extdeps consumers** (build.rs, gunbc_generated, etc.). Loader-close is the smallest unblock; broader migration is downstream.
- **Not generating Rust source from the loaded `.dag` declarations.** That's a separate emit-pipeline concern.

## Reporting

- Single PR. Title pattern: `feat(v3): extdeps loader close — make rust/primitives.dag loadable into bootstrap (unblocks Engine-Phase-1 sharpened-(b))`.
- PR description cites this brief + addresses each of the 5 consumer requirements explicitly + documents SG-0 stance choice.
- On merge: signal Director (`zesty-bear-812`); Director signals Grounding Manager to re-dispatch Engine-Phase-1 against sharpened-(b).
- On STOP-AND-ESCALATE: surface to Director; Director resolves before resuming.

## Cross-manager note

- **Grounding Manager (`crisp-seal-366`)**: heads-up'd at dispatch. Engine-Phase-1 re-dispatch waits on this loader-close PR landing. Re-dispatch shape: sharpened-(b) sibling crate consuming `rust_pilot_primitives` symbolically via the public accessor this PR introduces.
- **Zero-Floor Manager (`stern-swift-335`)**: heads-up'd at dispatch. PB-1's eventual extension to cover extdeps as a fifth fixture set is downstream of this loader; no current conflict on `dag.rs` shape (this PR doesn't touch substrate types). If req 4's SG-0 stance picks ratchet-bump, Zero-Floor knows the bump is justified by Engine-Phase-1's load-bearing dependency.

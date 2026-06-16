# Pure Bootstrap to Zero — Design

**Status:** `LIVE` (promoted 2026-04-25 via cascade promotion [PR #782](https://github.com/gunb-ai/gunbc/pull/782)). Supersedes the ≤5-floor framing in [`docs/design-pure-bootstrap.md`](design-pure-bootstrap.md) (now marked SUPERSEDED with explicit retraction) and the 2-3 principled-floor framing introduced via [PR #756](https://github.com/gunb-ai/gunbc/pull/756) (also retracted). This doc is the live authority on the Pure Bootstrap to Zero program.

> **v2 supersession (2026-05-15).** The 0-floor target articulated in this doc now applies to **v2** ([`src/v2/`](../src/v2/)) as the operational instantiation. v2 binary serves as v2's stage minus one (per [`src/v2/workflow/bootstrap.dag`](../src/v2/workflow/bootstrap.dag) bootstrap chain); v2's compiler emits its own Rust trampoline (`bin/main.dag`) to satisfy 0-floor without needing the runtime-resolution choices (shipped binary / runtime crate / rustc-macro) described below. v3 references throughout this doc describe the v2→v3 transition that v2 supersedes; v3 is frozen pending v2 ship.

**v2 workflow authorities (registry).** Load-bearing workflow models: [`src/v2/workflow/bootstrap.dag`](../src/v2/workflow/bootstrap.dag) (bootstrap orchestration as data) and [`src/v2/workflow/release.dag`](../src/v2/workflow/release.dag) (RELEASE_TODO §5 Phase 1 — GH Releases binary distribution as data; module `v2.workflow.release_dist`). CI is **not** modeled as `.dag` data: the prior `src/v2/workflow/ci.dag` mirror was a descriptive-only model with no runtime consumer and was deleted (operator GO 2026-06-08); `.github/workflows/ci.yml` is the direct hand-edited CI authority, and `src/v2/workflow/lens_ci_gate.dag` retains only the lens-CI claim-run roster the must-pass lens gate consumes. Bootstrap remains the Pure Bootstrap / lens-self-application core; release is a bounded product-ship lane with a hand-synced YAML projection posture until YamlStatic emission (T-24). This doc governs their Pure Bootstrap / N=0 discipline (A3, PROOF-1, and the STOP rail: only bootstrap-modeled authority in those surfaces). **C4** (committed `release.yml` as a checked projection from `.dag`) is ratified via operator-signed PR review (no maintained manifest).

**Promotion evidence chain (cited in cascade promotion PR body):**
- D1 audit: PRs #769 + #771 + #775 + #777 + #779 (audit doc with substrate-generation already proven; 23 generated files + 24 REGEN_OUTPUTS entries; 38-type substrate.dag coverage survey)
- D2 PB-1 brief amendment: PR #770 (non-goals revised under 0-floor)
- D3 TESTING.md "Post-R2 shape" rewrite: bundled into cascade promotion PR (Director-call per Promotion mechanism below)
- D4(c) characterization: in #775 audit reframe (substrate generation already proven and shipping)
- D4(a) prototyped lane closure: PR #780 (PB-Substrate v2 — ArithmeticOp/ComparisonOp/LogicalOp/OperatorKind via existing regen pattern)
- PM acknowledgement: explicit user sign-off on direction + multiple manager-brief reviews from PM session

### Promotion mechanism

**(Historical context — promotion executed 2026-04-25 via cascade promotion PR; all retraction steps below were performed atomically per the no-piecewise-promotion clause. Section preserved for audit-trail readability of how the promotion happened.)**

This doc was PROPOSAL until promoted. **Promotion was a single Director-authored cascade PR** that did **all** of the following atomically (no piecewise promotion — the framing change is load-bearing across multiple authorities and partial promotion would have created contradiction):

- **Who promotes:** Director, after Grounding Manager (R2's standing manager) and at least one R2 substrate-prereq sub-lane have produced stable enough evidence that the (γ) shape is achievable in v3's substrate (i.e., not a paper proposal).
- **What gate:** the cascade PR includes (a) at least one prototyped lane closure proving an existing hand-Rust file can be retired via `.dag` migration without regression, and (b) explicit acknowledgement from PM that the program structure is dispatchable.
- **What retraction:**
  - `docs/design-pure-bootstrap.md` updated with a banner pointing at this doc as the new authority; the ≤5-floor framing struck through with explicit retraction language; cross-refs migrated.
  - PR #756's "2-3 principled floor" framing similarly retracted in `docs/design-pure-bootstrap.md` with same-PR replacement to "0 floor."
  - `ROADMAP.md` T-PB-A row (`:53`) amended to point at the 0-floor target via this doc; the `≤5 irreducible-shim` language struck.
  - `docs/r2-structure.md` "Lanes deliberately absent" entry for `T-ShimFloor` updated to reflect the program's new home (R2 absorption or separately-named program — Director call at promotion time).
  - `TESTING.md` "Post-R2 shape" residual section either rewritten in this same cascade PR (bundled) or sibling PR landed first (separated). See "TESTING.md rewrite" section below.
  - This doc's status banner updated from `PROPOSAL` to `LIVE` with promotion date and citing the cascade PR number.

~~If any retraction step can't land in the same PR (e.g., cascading conflicts), promotion blocks; this doc stays `PROPOSAL` until the cascade can be authored atomically.~~ **(Resolved.)** Cascade landed atomically on 2026-04-25; all five retractions present in the cascade PR; doc is now LIVE.

### Pre-promotion deliverables

These are assertions in this PROPOSAL whose empirical backing must surface in the promotion cascade PR (or in sibling PRs landed before promotion). Naming them explicitly so the cascade PR's reviewer can verify each before the framing change is locked:

- **35-file audit table** — the §"Frame" claim that the "irreducible tier" (build.rs, bootstrap.rs, lib.rs, dag.rs, dag/ports.rs, dag/effects.rs) isn't structurally irreducible needs an inline table mapping each of the 35 `EXPECTED_HAND_AUTHORED_NON_TEST` entries to (a) why it's currently hand-authored, (b) which migration path retires it (PB-Substrate / PB-1 / PB-4 / PB-5 / PB-6 / PB-Lib+Build / PB-Runtime / PB-Bootstrap-Process / PB-Tier1-Sweep). Goes in the promotion cascade PR.
- **PB-1 brief amendment** — `docs/briefs/pb-1-data-driven-bootstrap.md`'s non-goals (don't delete tokenize/parse/lower/infer/emit; don't go binary-blob; don't change Dag runtime format) invert under 0-floor. The brief must be amended in the promotion cascade PR (not silently after) so PB-1's downstream consumers see the revised scope.
- **TESTING.md "Post-R2 shape" rewrite** — Director-call at promotion time on bundle-vs-sibling per §"TESTING.md rewrite" below; either way, must be authored before this PROPOSAL flips to LIVE.

## Frame

**Goal: zero hand-authored files in v3's source tree.** Better than v2's 1-residual.

The ≤5-floor target was a sizing convenience masquerading as an architectural lower bound. Auditing the 35 files currently in `EXPECTED_HAND_AUTHORED_NON_TEST` shows the "irreducible" tier (build.rs, bootstrap.rs, lib.rs, dag.rs, dag/ports.rs, dag/effects.rs) isn't structurally irreducible — it's just not-yet-migrated substrate authoring. Each of those files can be generated from a `.dag` authority, with the same pattern v3's other generated files already use.

Zero is achievable. v2 proved the cycle (1 residual was a TESTING.md choice, not a structural limit). v3's substrate is more structural than v2's (lens-as-substrate-declaration, types-as-data, programs-as-data) — so v3 should hit a tighter floor than v1.

## Design choice — shape (γ)

Three escalating shapes were considered:

- **(α)** PB-1's current scope: data-bootstrap of DAG, hand-Rust everything else. Floor ~6-8.
- **(β)** Bootstrap process modeled in `.dag` + a small hand-Rust universal evaluator. Floor ~1-2.
- **(γ)** Substrate, evaluator, and bootstrap process all data. The compiled binary is fully generated; the host runtime (or the build trampoline) is the only thing not in v3's source tree. Floor: 0.

**This proposal commits to (γ).** Rationale:

- (β) is a bridge that gets dissolved into (γ); writing the bridge first is wasted work unless mandatory. It's not mandatory — see "First-time bootstrap" below.
- (γ) matches the thesis claim that the substrate is its own subject ([`docs/thesis/self-inspection.md`](thesis/self-inspection.md)). Hand-authored files in v3's source tree contradict that claim at the implementation layer.
- The runtime requirements for (γ) (a small generated evaluator + Cargo trampoline) are themselves generatable. There's no architectural reason to author them by hand.

## Bootstrap as data — the conceptual core

The current model: bootstrap is a *process* implemented in hand-Rust (`bootstrap.rs::Dag::new()`) that loads `.dag` source, runs the compiler pipeline, populates a `Dag`. The chicken-egg is "Dag::new() needs the compiler pipeline, which needs Dag::new()."

The (γ) model: **bootstrap is data.** A `bootstrap.dag` (or equivalent authority) declares:
- Which `.dag` files load in which order
- What state the loaded `Dag` should reach
- What invariants must hold post-load
- What the entry point evaluates to

The "evaluator" is a small generic interpreter that:
- Reads the bootstrap data
- Constructs the `Dag`
- Hands it to the rest of the compiler (also data)
- Runs `main` (also data)

The evaluator is itself generated from a `.dag` authority that describes its operations. Per `feedback_compiler_is_dag_processor`: the compiler knows only Node / Conj / Disj / Cardinality / Bit. The evaluator therefore needs only those primitive operations — small.

**The chicken-egg breaks at the build step**, not at runtime: the previous-generation binary reads the new bootstrap data and emits the new evaluator binary. Same cycle v2 used, applied recursively to the evaluator itself.

## First-time bootstrap (the genuine N=0 problem)

Even (γ) has a first-time bootstrap problem: the very first compiled binary has to come from somewhere.

Three resolutions, in increasing aggressiveness:

1. **Shipped pre-built binary.** The "first" binary is a release artifact (downloaded via install script, embedded in a published crate, or shipped as a release asset on GitHub). Subsequent bootstraps regenerate from source. Hand-Rust in v3's source tree: 0. Hand-Rust elsewhere (in the install script or release tooling): negligible / not part of v3.

2. **Universal-runtime crate.** A tiny `gunbc-runtime` crate (published separately, not in v3's source tree) provides the generic evaluator. v3's source tree is `.dag` files + a generated Cargo.toml that depends on `gunbc-runtime`. The runtime crate is hand-authored, but **outside v3's source tree** (cleanly factored as runtime support, not compiler).

3. **Procedural macro on existing rustc.** A `#[gunbc::data]` attribute (in a tiny crate) tells rustc to interpret the `.dag` content at compile time. Same shape as 2 but different boundary — the macro is the runtime; the rest is data.

All three give **0 hand-Rust in v3's source tree.** Resolution choice is downstream of taste + ecosystem strategy; doesn't affect the in-tree floor.

For this proposal: the in-tree floor target is 0 regardless of which N=0 resolution lands. Pick the resolution as a separate decision; it doesn't gate the migration program.

## Subsumed lanes

Existing lanes that fold into this program:

- **PB-1** (brief `pb-1-data-driven-bootstrap.md`, removed in #4162; XXL) — data-driven bootstrap loader. **Non-goals revised:** PB-1's current non-goals (don't delete tokenize/parse/lower/infer/emit; don't go binary-blob; don't change Dag runtime format) were appropriate for the ≤5-floor framing but contradict 0-floor. Under this proposal they invert: PB-1 is the first phase of a chain that ends with all of those files generated.
- **PB-4** (lower) — generate `lower.rs` from `lower.dag`. SG-3 series + lower.dag authority required.
- **PB-5** (infer) — generate `infer.rs` from `infer.dag`. SG-4 dispatch.
- **PB-6** (emit) — generate `emit.rs` + emit targets from spec authorities (`rust.dag`, `python.dag`, `go.dag`). Lane 1e dependency.

Existing scope from `design-pure-bootstrap.md` continues to apply for the file-by-file work, just with a tighter target.

## New lanes

**PB-Substrate** (new, sized M-L) — generate `dag.rs`, `dag/ports.rs`, `dag/effects.rs` from `src/v3/std/substrate.dag`. The substrate type definitions become data; the Rust types are emitted from them. Cementing test: generated Rust matches the structural facts the substrate model declares.

**PB-Lib + PB-Build** (subsumes PB-8 aspiration, sized M) — generate `lib.rs` (module declarations + crate exports) and `build.rs` (Cargo build script content) from emit authority. Both files become trampolines that `include!()` generated content from `OUT_DIR`, or vanish entirely if Cargo can be told to look elsewhere.

**PB-Runtime** (new, sized M-L) — generate `test_runner.rs`, `lens_apply.rs`, `lens_testgen.rs`, `post_emit_verifier.rs` from `.dag` authorities. The "test runner" and "lens evaluator" become data + a tiny generic interpreter.

**PB-Bootstrap-Process** (new, sized M, the conceptual core) — author `bootstrap.dag` describing the bootstrap workflow as data. Replace `bootstrap.rs` with a generated trampoline. Section "Bootstrap as data" above is its design.

  **N=0 runtime boundary verification (per codex `7db5c913` review).** Whichever N=0 resolution lands (shipped binary / `gunbc-runtime` crate / rustc proc-macro), the runtime must be a *generic fixed-point substrate runner* — not hand-authored compiler logic relocated out of v3's source tree. Verification gates that PB-Bootstrap-Process must satisfy:
  - **Operation set bounded.** The runtime invokes only the primitive substrate operations (Node / Conj / Disj / Cardinality / Bit per `feedback_compiler_is_dag_processor`) — no compiler-pass logic (no tokenize, no parse, no infer, no lower, no emit) lives in the runtime. If the runtime needs to do any of those, it does so by invoking generated `.dag` programs, not by carrying their logic.
  - **Size budget.** A specific LOC ceiling on the runtime (target: <500 LOC of hand-Rust outside v3's tree, modulo language-stdlib-equivalent primitives). If the runtime grows past the budget, it's a signal that compiler authority leaked out.
  - **Substrate-runner equivalence test.** A generic-substrate-runner test: feed the runtime a `.dag` program that's structurally equivalent to but textually different from v3's compiler.dag, verify the runtime evaluates it identically. This proves the runtime is generic over `.dag` programs, not specialized to v3's compiler.

  These verification gates land in PB-Bootstrap-Process's acceptance criteria; the promotion cascade PR cites the test results.

**PB-Workflow** (existing scope continued) — `workflow_idempotency.rs` and `workflow_parallelism.rs` migrate as Lane 2 dissolution lands.

**PB-Tier1-Sweep** (per-file fast-retire, sized S each) — depends on above. The 13 Tier-1 files (regen binaries + bin helpers) retire as their backing migrations land. Not blocked on PB-Substrate / PB-Lib / etc., but blocked on PB-1 + PB-4/5/6.

## TESTING.md rewrite (sibling PR or bundled into promotion cascade — Director call)

Current `TESTING.md` carves out two permanent Rust-authored test categories:
- Compiler-internal unit tests for Rust-only helpers
- External-toolchain boundary tests (rustc/go/python invocations)

Both dissolve under 0-floor:

- **Rust-only-helper unit tests** vanish naturally — there are no Rust-only helpers; everything generates from `.dag`.
- **External-toolchain boundary tests** migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations as the cascade-named successor pattern. **Capability state (2026-04-25, updated):** the `ExecuteCommand` schema is at `src/v3/std/verification.dag:115-119` (PR #678). The PB-Runtime runner extension is **landed** — the Rust `TestRunner` and M1.5 testgen share `evaluate_execute_command_exit_code` / `parse_execute_command_fields` in `src/v3/compiler/src/test_runner.rs` for arbitrary `command` + `args` with exit-code capture, **discarded** child I/O, and a fixed wall-clock cap (`EXECUTE_COMMAND_WALL_TIMEOUT`, fail-closed on exceed). The remaining migration work is **bulk** porting of pre-existing class-5 boundary tests, not a runner gap. See `TESTING.md` (cascade callout) and `src/v3/compiler/tests/dag/t_pb_b_1_execute_command_boundary.dag` for a receipt.

**TESTING.md rewrite scope:**
- Replace "Post-R2 shape" residual section with "0-residual" framing
- Reference `ExecuteCommand` predicate as the migration path
- Update authority list

The rewrite is structurally separable from this design doc. **Director-call at promotion time** whether to bundle into the promotion cascade or land as a sibling PR first.

## Acceptance gates

R2 / future release acceptance for this program:

- `EXPECTED_HAND_AUTHORED_NON_TEST` count = 0
- `EXPECTED_HAND_AUTHORED_FRAGMENTS` count = 0
- `EXPECTED_HAND_AUTHORED_TEST` count = 0
- TESTING.md "Post-R2 shape" residual rewritten to 0-residual
- DB-8 `self_host_fixed_point` converges bit-identically (the no-compromise gate; same as PB-1's)
- Compile cycle still produces a working binary (regen → rustc → run → diff fixed-point)
- All `[ext]` test predicates evaluate against generated authorities
- `bootstrap.dag` declares the bootstrap workflow + first-time bootstrap resolution explicitly

## Dependency DAG

```
PB-Substrate ────────────────┐
                              ├──→ enables PB-Lib + PB-Build
PB-1 (data-driven loader) ───┤        (emit Rust files from .dag = generic capability)
                              │
PB-4 (lower)  ┐               ├──→ enables PB-Bootstrap-Process
PB-5 (infer)  ├──→ generic    │        (compiler in .dag → bootstrap can be in .dag too)
PB-6 (emit)   ┘    .dag       │
                   compiler   │
                              │
PB-Runtime ──────────────────┤
   (test_runner, lens_apply  │
    in .dag)                  │
                              │
PB-Workflow (Lane 2 dissolution)
                              │
PB-Tier1-Sweep (per-file ──┘  
   retirement; depends on
   above-listed migrations)
```

**Parallelization:** PB-Substrate, PB-1 (sub-lanes), PB-4, PB-5, PB-6, PB-Runtime, PB-Workflow are all parallel-capable Day-1. PB-Lib + PB-Build is parallel after PB-Substrate's pattern is proven. PB-Bootstrap-Process is parallel after PB-1 + PB-4/5/6 (needs the full compiler in .dag to bootstrap from). PB-Tier1-Sweep is per-file fast-retire after each backing migration lands.

**Worker capacity:** 6-8 parallel workers can productively dispatch against this DAG without coordination collisions, given the substrate.dag / SG-0 census is the only shared-edit hotspot (each PR removes one entry). Higher worker count = more rebase churn on `sg0_census_test.rs`; lower count = longer wall clock.

## STOP-AND-ESCALATE

Inherits from `design-pure-bootstrap.md` + PB-1 brief, plus:

- **If first-time bootstrap (N=0) resolution requires hand-Rust in v3's source tree** — STOP. The resolution is supposed to live outside v3's source tree (install script, gunbc-runtime crate, or rustc macro). If it can't, the 0-floor target is unreachable and the framing needs revision back toward (β).
- **If the universal evaluator turns out to need substrate operations beyond Node / Conj / Disj / Cardinality / Bit** — STOP. The bounded-kernel invariant is load-bearing; expanding it to make (γ) work is wrong.
- **If TESTING.md rewrite reveals test categories that genuinely cannot express in `ExecuteCommand`-based `.dag`** — STOP. May indicate a `TestClaim` predicate gap that's its own substrate work. Surface before forcing the migration.

## Non-goals

- **Not picking the N=0 resolution shape in this doc** — three options (shipped binary / runtime crate / rustc macro) are scoped above; pick separately. Doesn't gate migration.
- **Not authoring the program structure as ROADMAP lanes** — this is the design doc; the ROADMAP lane structure is a separate sibling that PM authors when this doc lands.
- **Not committing to a release boundary** — whether 0-floor lands in R2, "before R2 finishes," or a separately-named program is a director call. Per user direction: "needs to finish before R2 finishes, that's all I care."

## Open calls

### 1. Cargo conventions vs zero-source-tree

Cargo expects `Cargo.toml` + `src/lib.rs` (or `src/main.rs`) at fixed paths. Even if these are 1-line trampolines that `include!()` generated content, they're physical files in the source tree. Two readings:

- **(a) Trampolines count as 0** because their content is generated; the file is just a path-binding for Cargo. Pragmatic; matches "what's in version control" rather than "what's on disk."
- **(b) Trampolines count as N**, where N is the number of trampoline files Cargo conventionally requires. Stricter; floor is N (probably 1-3).

For this proposal: **trampolines are 0 if their content is generated.** The user's stated goal is "0 hand-authored files" — a 1-line `include!()` trampoline that's itself emitted from a `.dag` authority is generated, not hand-authored. Cargo's path constraint is an environment convention, not v3's source-tree commitment.

### 2. v3's first hand-Rust file: who owns it during transition?

During the migration (PB-1 → PB-Substrate → ... → 0), v3 will have a shrinking-but-nonzero hand-Rust count. SG-0 census tracks this. Two questions:

- Who declares "v3 has reached 0"? Director, on the closure declaration PR.
- What if a migration regresses (introduces a new hand-Rust file)? STOP-AND-ESCALATE; the ratchet only goes down.

### 3. Bundling vs separating TESTING.md rewrite

TESTING.md rewrite is structurally separable from the design doc. Bundling: faster, atomic. Separating: cleaner authority layering, easier review.

Director (you / me) calls. PM may have a preference based on review bandwidth.

## Cross-refs

- Existing parent: [`docs/design-pure-bootstrap.md`](design-pure-bootstrap.md) — supersedes the ≤5-floor framing on this doc's promotion (see Promotion mechanism above for the cascade requirement).
- Subsumed brief: `docs/briefs/pb-1-data-driven-bootstrap.md` (removed in #4162) — non-goals revised under the 0-floor target.
- Thesis claim: [`docs/thesis/self-inspection.md`](thesis/self-inspection.md) — "the substrate is its own subject"; this proposal makes that claim load-bearing at the implementation layer (no hand-Rust contradicts the claim).
- Related fixed-point ratchet: `DB-8` — the no-compromise convergence gate.
- TESTING residual authority: `TESTING.md §"Post-R2 shape"` — sibling PR or bundled cascade rewrites this section to 0-residual.
- ROADMAP T-PB-A row at `:53` — amended in promotion cascade (acceptance target shifts from ≤5 to 0).
- R2 structure at `docs/r2-structure.md` — "Lanes deliberately absent" T-ShimFloor entry amended in promotion cascade (program either absorbed into R2 or separately-named).
- Floor-target precedent: PR #756 introduced "2-3 principled floor" framing — gets superseded by 0 under this proposal; retraction happens in promotion cascade.

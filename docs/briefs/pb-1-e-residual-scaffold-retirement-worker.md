# PB-1-e — Residual scaffold retirement + DB-8 cross-check re-grounding `(M)`

> **Worker brief.** Reports through Zero-Floor Program Manager
> (`stern-swift-335`). Replaces the withdrawn PB-1-b brief (rolled
> back same-PR alongside this brief's authoring) per the
> `warm-raven-373` STOP-AND-ESCALATE finding that PB-1-a as shipped
> already covered all four authorities; PB-1-b/c/d were folded into
> PB-1-a, leaving PB-1-e (reframed) as the residual production work.

## Read first

- **[`docs/briefs/pb-1-data-driven-bootstrap.md`](pb-1-data-driven-bootstrap.md) §"Working state (verified 2026-04-25)"** — the program brief's working-state amendment recording PB-1-a-actual-coverage. Read in full before starting.
- [`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs) `:91-99` — **the named dissolution trigger**: *"delete this helper once the PB-1 drift harness no longer needs to re-run the pre-snapshot std bootstrap path."* PB-1-e's job is to satisfy that condition.
- [`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs) `:131-150` — `load_runtime_bootstrap_authorities`: the regen-scaffold runtime-parse path PB-1-e retires.
- [`src/v3/compiler/src/bootstrap.rs`](../../src/v3/compiler/src/bootstrap.rs) `:115-130` — `bootstrap_all_runtime` + `compile_full_bootstrap_dag_from_std_seed` callers.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) `:2214-2218` — `Dag::new()` body. Already loads `bootstrap_generated::bootstrapped_fixture_dag()` directly. PB-1-e doesn't change this path; it retires the legacy parallel path.
- [`src/v3/compiler/src/bin/regen_bootstrap.rs`](../../src/v3/compiler/src/bin/regen_bootstrap.rs) — the regen tool whose runtime-parse step PB-1-e re-grounds.

## Frame

PB-1's program brief originally framed PB-1-e as *"runtime-path retirement + measurement after b/c/d land"*. PB-1-a's actual landing covered all four authorities, collapsing PB-1-b/c/d. PB-1-e's residual work is therefore **two coupled tasks**:

1. **Retire `load_runtime_bootstrap_authorities` + `compile_full_bootstrap_dag_from_std_seed`** — the regen-scaffold runtime-parse path. Today these helpers exist only to support `regen_bootstrap`'s "fresh parse from std seed → diff against committed snapshot" DB-8 cross-check.
2. **Re-ground DB-8's cross-check on a different mechanism.** The current DB-8 acid test is *"compile_full_bootstrap_dag_from_std_seed produces a Dag bit-identical to the committed bootstrap_generated.rs snapshot."* Retiring the fresh-parse path collapses this mechanism. The replacement must preserve DB-8's no-compromise property — that the committed snapshot matches a from-scratch compile.

**The two tasks are coupled** — retiring the helper without replacing the cross-check is unsafe; replacing the cross-check without retiring the helper is incomplete. Land both in the same PR.

## Slice — two coupled deliverables

### Deliverable A — retire `load_runtime_bootstrap_authorities`

- Delete `load_runtime_bootstrap_authorities` (`bootstrap.rs:131-150`) and its callers `bootstrap_all_runtime` (`:107-121`) + `compile_full_bootstrap_dag_from_std_seed` (`:115-130`) once Deliverable B is in place.
- Delete `bootstrap_std_fixtures_only` (`bootstrap.rs:91-105`) per the named dissolution trigger comment at `:91-99`.
- Audit any remaining call sites in `regen_bootstrap.rs` and PB-1 drift tests; rewire to the new mechanism from Deliverable B.

### Deliverable B — re-ground DB-8 cross-check

Three candidate mechanisms (worker picks; surface choice + reasoning in PR description):

- **(i) Per-authority bit-identical composition.** Each authority generates its own snapshot module (`bootstrap_std_generated.rs` already exists; add `bootstrap_staged_generated.rs`, `bootstrap_specs_generated.rs`, `bootstrap_compiler_generated.rs`); DB-8 asserts that loading them in sequence produces a Dag bit-identical to `bootstrap_generated.rs`. Cross-check is "the four parts compose to the whole." Pro: structural; con: requires per-authority snapshot split (substantial regen work).
- **(ii) Separate fresh-compile-once gate at regen time.** Move the fresh-parse-vs-snapshot check into `regen_bootstrap` itself: the regen binary, when run, performs the fresh compile, diffs against the snapshot, and refuses to update if drift is detected. DB-8 in-tree becomes "the committed snapshot is internally consistent" (cheaper structural checks); the fresh-compile gate runs only at regen time. Pro: cleanest scoping; con: DB-8 weakens slightly — no longer asserts "snapshot matches fresh compile" on every test run.
- **(iii) Hybrid.** Per-authority composition gate (i) AS the in-tree DB-8, plus a `regen_bootstrap`-side fresh-compile-once gate (ii) that runs on regen. Two layers of protection.

**Manager lean: (ii)** — cleanest scoping, doesn't require per-authority snapshot split, cheapest in-tree test cost. (i) is good if PB-Substrate proper later forces per-authority splits anyway. (iii) is over-engineered for current scope. Worker is welcome to pick differently if execution surfaces a reason — surface the reasoning in PR description.

If worker concludes none of these preserves DB-8's no-compromise property well enough, STOP-AND-ESCALATE.

## Acceptance

- [ ] Deliverable A: `load_runtime_bootstrap_authorities`, `bootstrap_all_runtime`, `compile_full_bootstrap_dag_from_std_seed`, `compile_full_bootstrap_without_parse_surface_dag`, `bootstrap_std_fixtures_only` all retired (or any kept entries justified explicitly in PR description).
- [ ] Deliverable B: DB-8 cross-check re-grounded per worker's mechanism choice (i/ii/iii); no-compromise property preserved.
- [ ] `regen_bootstrap.rs` rewired to the new mechanism.
- [ ] PB-1 drift tests rewired or retired.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] **DB-8 `self_host_fixed_point` converges bit-identically** under the new mechanism.
- [ ] SG-0 census deltas: any retired helpers off the list; new generated files (if any) in REGEN_OUTPUTS partition.

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager (`stern-swift-335`).

- **If none of (i)/(ii)/(iii) preserves DB-8's no-compromise property** — STOP. The cross-check mechanism is load-bearing; the manager + Director need to weigh in before any PR commits to a weaker DB-8.
- **If retiring `load_runtime_bootstrap_authorities` reveals consumers beyond the named scaffold paths** (e.g., a non-regen, non-test consumer) — STOP. Indicates the brief's "scaffold-only" framing was wrong and the path serves production work this brief didn't anticipate.
- **If the new DB-8 mechanism requires per-authority snapshot splits** (mechanism (i)) **and that split surfaces unexpected substrate-shape questions** (e.g., the snapshot serializer can't cleanly composition-decompose) — STOP. Per-authority work may belong in PB-Substrate proper or in its own follow-up brief.
- **If DB-8 fixed-point drifts** — STOP immediately.
- **If pilot scope balloons beyond Deliverables A+B** — STOP.

## Non-goals

- **Not extending substrate generation** — orthogonal lane (PB-Substrate proper).
- **Not migrating any of `tokenize.rs` / `parse.rs` / `lower.rs` / `infer.rs` / `emit.rs`** — those are PB-4/5/6 (per amended PB-1 brief).
- **Not changing `Dag::new()` itself** — already loads the full snapshot directly; this brief retires only the parallel scaffold path.
- **Not amending substrate.dag declarations.**

## Reporting

- Single PR. Title pattern: `feat(v3): PB-1-e — retire load_runtime_bootstrap_authorities scaffold; re-ground DB-8 cross-check on mechanism (X)`.
- PR description: cite this brief; cite chosen mechanism (i/ii/iii) + reasoning; cite DB-8 cross-check shape post-retirement.
- On merge: Zero-Floor Manager confirms PB-1 program closure to Director (PB-1-a + PB-1-e cover the full program; b/c/d folded).

## Cross-manager note

No cross-manager signal needed at brief-authoring time. If Deliverable B mechanism choice surfaces substrate-shape implications affecting Grounding's territory, surface to manager → Director per established cross-program coordination.

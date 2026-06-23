# Plan — v2 self-hosting

**Status:** planning tracker · **DESIGN.md + the carriers remain the authority** (DESIGN §6). A task's real state is its branch/PR, not this file. Linked from `ROADMAP.md` §5 *Self-host v2 → delete `src/v1`*. Related to but distinct from the de-fork audit ([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md)): de-fork collapses duplication; self-hosting makes v2 compile itself. De-fork is a *prerequisite* (the compiler closure needs one std authority), not the goal.

**Carrier facts below were cross-verified against `main @ 6164261490` (#5419) and branch `emitter/seed-green-integration @ 239ff284d5` by session merry-deer-374 (5 evidence-backed verifiers, 2026-06-21).** Re-check receipts before acting.

> **END GOAL (decided 2026-06-21 — anchored in ROADMAP §5 *Self-host*, do not re-litigate).** Three languages: **`.dag` is the authority/truth; v2 emits BOTH Rust AND TypeScript as first-class realizations** (not one-or-the-other). The fixed point is proven per realization (Rust bytes reproduce; TS bytes reproduce). **Purely self-hosting:** v2 emits its own seed — no stage0 hand-edits (enforced by the regen-lockstep gate, see Track A "Purity"). **Terminal goal: shrink the seed to zero** — the ~154k lines of hand-written Rust *compiler logic* go to zero; a minimal pinned, reproducible, v2-emitted bootstrap binary survives (it must still run the first `.dag`). Not a big-bang `rm src/v1` (see Track Z). Rust is furthest along (the seed language); TypeScript is proven only on the `add` slice today and needs target-completeness to join the fixed point.

## 0. State of play (what's done, what's open)

- **Front-end — DONE, proven at scale.** `src/v2/` is 100% `.dag` (698 modules, 0 `.rs`; all Rust is the `src/v1` seed, ~154k lines). Full pipeline present: `01_tokenize → 02_parse → 03_resolve/name_resolve/ingest → 04_infer → 05_emit/eval → 06_translate → 07_target_carriers`. The historical blockers are fixed and merged: parse-perf (#5093 ParseTable memo, #3661), resolve/infer cost (#5258 the O(2^depth) double-resolve fix, #5266, #5146), cross-file names (#5271 Route C Phase B, #5154 filepath→QualifiedName). gap-4 (let-stmt in match arm) merged #5369; whole-tree parse-regression scanner added #5406. *Nuance:* per-file parse is fast, but a **combined `dsl`+`src/v2` whole-tree resolve is still heavy (590s+)** — "no perf wall" means the fixed blockers, not an instant full-tree resolve.
- **Emit / Route A — runs whole-tree, but the emitted crate doesn't build yet.** `gunbc compile --source-root … --target rust` exists and the CI `dsl_compile_clean` gate runs it over `[src/v2, dsl]`, gating all of batch-2. **But the gate only proves the tree is well-typed** (emit runs to completion, writes `$OUT`, then `rm -rf $OUT`). It does **not** `cargo build` the emitted whole-tree crate. Only the emit-host MVP smokes compile tiny fixtures. **The green build of the emitted crate is the open Route-A last mile.**
- **Fixed-point proof — fail-closed (not achieved).** `self_host.dag`'s `self_host_fixed_point_validate` **unconditionally returns `Rejected`** (`self_host_runner_not_realized`) — the honest contract until Stage C. `self_host_fixed_point_digests_match` (`==` over `content_hash`) is realized but de-risked only on fixture stages. The whole-compiler fixed point over real digests does not yet run.
- **Emitter — genuinely multi-target.** One `fold_node` catamorphism in `06_translate.dag`; emit selects a target's `target_model_edge_translation_rules` and walks the same rows **backward** (`grammar_relation_row_reverse_parse_selection`). 14 targets in `src/v2/extdeps/languages/` (rust, python, go, bash, dag, cpp, typescript, kotlin, swift, java, lean, ptx, verilog, …), 5 in `dsl`. A new target is **data rows, never a new emitter**.

## 1. The tracks

### Track A — Rust self-host bootstrap (the active path)

Get the v2-emitted Rust compiler to build and reproduce itself.

1. Emit whole tree `--target rust` — **done** (well-typed under CI gate).
2. `cargo build` the emitted crate green — **open** (the last mile).
3. `regen_stage0` from the emitted crate; replace the hand-Rust seed.
4. Flip the lockstep / fixed-point gate from fail-closed to asserting the real `content_hash` digest match (Stage C).

Unmerged work: branch `emitter/seed-green-integration @ 239ff284d5` (**not in main**) carries 6 commits toward a green regen — import-completeness (E0425), List→host-`Vec`, Int-as-generic-arg→`i64`, `regen_stage0` patch, + 2 integration commits. PR **#5325 is CLOSED, not merged.** Recent emitter slices on main: #5413 (length→count, phantom-marker derives, Measure-alias E0560 peel, E0308 Box-deref cluster); MachineWidth is **#5397** (E0107), a separate PR.

### Purity: no stage0 hand-edits (the requirement, and the enforcement gap)

The requirement — **v2 emits its own seed; no human stage0 patches** — is *modeled, currently unmet, and un-enforced.* `src/v2/workflow/bootstrap.dag` is the authority and states it exactly ("seed→stage0→stage1→stage2, fixpt stage1==stage2 … seed used once; v2 is never in the loop again", DESIGN §7), including the trust machinery to retire the seed (`SeedHonestyDischarge`, `DiverseCompilationAgreement`/`IndependentCompilerPair` = Diverse Double-Compiling, the Thompson trusting-trust defense). But:

- **The seed is hand-maintained today.** `regen_stage0.rs` carries `HAND_MAINTAINED_STAGE0_FILES` + `patch_*` (e.g. `patch_bootstrap_dag_collect`) that compensate for emitter gaps — each a "stage0 hand-edit standing in for a thing v2 should emit itself," honestly marked with a dissolve-on pointing at the emitter fix. (Gotcha: regen has pre-existing codegen drift, so focused PRs hand-edit the `.rs` seed mirror rather than commit a full regen — itself a symptom of the gap.)
- **The no-drift gate is real but not wired.** `regen_stage0 --verify` (`verify_stage0_matches` — "committed stage0 matches fresh self-compile") exists, but a grep of the CI floor (`src/v2/workflow`, `dsl/tools`, `dsl/gunbc`, `dsl/test`) finds it nowhere. The `Stage0LockstepGate` that would wire it in is the content of **closed/unmerged #5325**. So hand-drift goes silently uncaught — erosion by one more honest-looking `patch_*` at a time.
- **bootstrap.dag is 🟡 scaffold** — structural wiring only, placeholder hashes (dissolve-on T-15/T-20 `content_hash` supplying real per-stage merkle digests), so it does not yet *prove* convergence.

**This gate is the keystone for both the purity requirement and a trustworthy cutover.** It is what makes "no stage0 hand-edits" enforceable; without it the requirement can only erode.

### Track B — the fixed-point proof (Stage C)

`candidate_generation.dag` (`generate_stage_candidate_from_ingest`) drives `assemble_program_from_ingest → infer → translate`, capturing the emitted **Node** before `serialize_target` — the input Stage C needs. Comparison substrate (emitted Node vs emitted bytes) is operator-pending (merry-crab-687). When it lands, the fail-closed runner flips to `Accepted` + real-digest match. Proven **per realization** (Rust first; TypeScript once its target rows are complete).

### Track T — TypeScript as a first-class realization

TS is 1 of 14 emit targets, proven only on the `add` slice (`cross_language_add_python_to_typescript_test.dag`, Python→core→TS). To join the fixed point it needs target-completeness over what the compiler actually uses (records, coproducts, folds, generics) — a gap census from `add` → full `src/v2`, then a TS regen + `node`-build green analogous to Track A's cargo path.

### Track Z — shrink the seed to zero (terminal)

The clean cutover. **"Shrink to zero" = the ~154k lines of hand-written Rust *compiler logic* go to zero — not literally zero bytes.** Something must still execute the first `.dag` (the substrate is data; v1/Rust is the runner today). The honest end-state (rustc/GCC model; what `SeedHonestyDischarge`/DDC is for) is a **pinned, content-addressed, reproducible-from-`.dag`, v2-emitted bootstrap binary** — itself re-derivable, since v2 even models its own `V4EvaluatorRuntime`.

So "delete stage0" decomposes — `src/v1` also provides the **CLI bins** (`claim_executor`, `regen_stage0`, `yaml_check`, …), the **CI floor runner**, the **host-effect transports**, and the **evaluator that runs `.dag`**. Each must be v2-emitted or pinned *before* it can go; a big-bang `rm -rf src/v1` after a green fixed point would take out the execution substrate, not just the redundant compiler.

**Forced precondition order (each gates the next):**

1. Whole-tree emit → `cargo build` green (Track A last mile).
2. Real fixed point: `self_host_fixed_point_digests_match` over real `content_hash` (Track B / Stage C; dissolve the placeholder hashes, T-15/T-20).
3. `regen_stage0 --verify` green **and wired into CI** (the `Stage0LockstepGate`, closed #5325) + all `patch_*` / `HAND_MAINTAINED_STAGE0_FILES` dissolved so the emitter emits the whole seed. **This is the step that actually retires "stage0 hand-edits"** and makes the cutover trustworthy.
4. Seed-honesty discharge (ideally via Diverse Double-Compiling).
5. Then collapse `src/v1` to the pinned reproducible seed and delete its compiler logic **incrementally** — verify by execution, not assumption.

## 2. Prerequisites / dependencies

1. **De-fork / cross-tree import** ([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md) §1) — the compiler closure is only well-defined once v2 imports the single `dsl/std` authority, not its mirror copies. Cross-tree import is wired but fail-closed (`03_name_resolve.dag:644`).
2. **Whole-tree resolve cost** — the 590s+ combined-tree resolve is the practical wall for a fresh whole-tree green receipt; relevant to both the CI gate and Stage C.
3. **TS-target completeness** — only needed if the end goal is Track-(b) TypeScript runtime; the `add` slice is proven, the compiler uses far more of the language.

## 3. Open questions (for the operator)

End goal is settled (Rust + TypeScript, `.dag` authority, delete v1 — see END GOAL). Remaining:

1. **Reopen direction** (raised by merry-deer-374, `node://session-merry-deer-374`): should the reopened "v2 self host (p2)" item drive **(a)** finishing the Route-A green bootstrap (merge the `emitter/seed-green-integration` branch → green regen → flip the lockstep gate), or **(b)** folding into a pure-`.dag` coherence restart? (Now decidable: (a) is the direct path to Track Z.)
2. **Scope of the first fixed point** — whole `src/v2`, or a defined compiler-core subset first, before widening to the full bin set Track Z must replace?
3. **TS sequencing** — pursue Track T in parallel with A, or after the Rust fixed point lands?

## Dissolution trigger (DESIGN §6)

Delete this doc when Track Z lands: `self_host.dag` asserts the real-digest fixed point (Stage C) and `src/v1` is gone. At that point the self-host witness + the absent `src/v1` *are* the authority and this tracker is redundant.

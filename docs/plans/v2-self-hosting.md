# Plan — v2 self-hosting

**Status:** planning tracker · **DESIGN.md + the carriers remain the authority** (DESIGN §6). A task's
real state is its branch/PR, not this file. Linked from `ROADMAP.md §3`. Related to but distinct from
the de-fork audit ([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md)): de-fork collapses duplication;
self-hosting makes v2 compile itself. De-fork is a *prerequisite* (the compiler closure needs one std
authority), not the goal.

**Carrier facts below were cross-verified against `main @ 6164261490` (#5419) and branch
`emitter/seed-green-integration @ 239ff284d5` by session merry-deer-374 (5 evidence-backed verifiers,
2026-06-21).** Re-check receipts before acting.

> **END GOAL — CONFIRM (Rust vs TypeScript).** The ROADMAP titles this "self-hosting v2 (TypeScript)",
> but **the active self-host path is Rust**: Rust is the seed language, so the fixed point is exercised
> through the Rust emit path (`--target rust`, `cargo`-green, `regen_stage0`). TypeScript is *one of 14*
> emit targets and is proven only on the `add` slice (Python→core→TS). So decide: is the north star
> (a) finish the **Rust** self-host bootstrap (Rust seed shrinks to zero against a v2-emitted Rust
> compiler), with TS as just another target; or (b) make **TypeScript** the self-host runtime (v2 emits
> TS that re-emits bit-identical TS)? (a) is what the carrier is actually driving toward today; (b)
> would be a new track. This decides whether the fixed point is measured over Rust bytes or TS bytes.

---

## 0. State of play (what's done, what's open)

- **Front-end — DONE, proven at scale.** `src/v2/` is 100% `.dag` (698 modules, 0 `.rs`; all Rust is
  the `src/v1` seed, ~154k lines). Full pipeline present: `01_tokenize → 02_parse →
  03_resolve/name_resolve/ingest → 04_infer → 05_emit/eval → 06_translate → 07_target_carriers`. The
  historical blockers are fixed and merged: parse-perf (#5093 ParseTable memo, #3661), resolve/infer
  cost (#5258 the O(2^depth) double-resolve fix, #5266, #5146), cross-file names (#5271 Route C Phase B,
  #5154 filepath→QualifiedName). gap-4 (let-stmt in match arm) merged #5369; whole-tree parse-regression
  scanner added #5406. *Nuance:* per-file parse is fast, but a **combined `dsl`+`src/v2` whole-tree
  resolve is still heavy (590s+)** — "no perf wall" means the fixed blockers, not an instant full-tree
  resolve.
- **Emit / Route A — runs whole-tree, but the emitted crate doesn't build yet.** `gunbc compile
  --source-root … --target rust` exists and the CI `dsl_compile_clean` gate runs it over
  `[src/v2, dsl]`, gating all of batch-2. **But the gate only proves the tree is well-typed** (emit runs
  to completion, writes `$OUT`, then `rm -rf $OUT`). It does **not** `cargo build` the emitted whole-tree
  crate. Only the emit-host MVP smokes compile tiny fixtures. **The green build of the emitted crate is
  the open Route-A last mile.**
- **Fixed-point proof — fail-closed (not achieved).** `self_host.dag`'s `self_host_fixed_point_validate`
  **unconditionally returns `Rejected`** (`self_host_runner_not_realized`) — the honest contract until
  Stage C. `self_host_fixed_point_digests_match` (`==` over `content_hash`) is realized but de-risked
  only on fixture stages. The whole-compiler fixed point over real digests does not yet run.
- **Emitter — genuinely multi-target.** One `fold_node` catamorphism in `06_translate.dag`; emit selects
  a target's `target_model_edge_translation_rules` and walks the same rows **backward**
  (`grammar_relation_row_reverse_parse_selection`). 14 targets in `src/v2/extdeps/languages/` (rust,
  python, go, bash, dag, cpp, typescript, kotlin, swift, java, lean, ptx, verilog, …), 5 in `dsl`. A new
  target is **data rows, never a new emitter**.

## 1. The two tracks

### Track A — Rust self-host bootstrap (the active path)
Get the v2-emitted Rust compiler to build and reproduce itself.
1. Emit whole tree `--target rust` — **done** (well-typed under CI gate).
2. `cargo build` the emitted crate green — **open** (the last mile).
3. `regen_stage0` from the emitted crate; replace the hand-Rust seed.
4. Flip the lockstep / fixed-point gate from fail-closed to asserting the real `content_hash`
   digest match (Stage C).

Unmerged work: branch `emitter/seed-green-integration @ 239ff284d5` (**not in main**) carries 6 commits
toward a green regen — import-completeness (E0425), List→host-`Vec`, Int-as-generic-arg→`i64`,
`regen_stage0` patch, + 2 integration commits. PR **#5325 is CLOSED, not merged.** Recent emitter
slices on main: #5413 (length→count, phantom-marker derives, Measure-alias E0560 peel, E0308 Box-deref
cluster); MachineWidth is **#5397** (E0107), a separate PR.

### Track B — the fixed-point proof (Stage C)
`candidate_generation.dag` (`generate_stage_candidate_from_ingest`) drives `assemble_program_from_ingest
→ infer → translate`, capturing the emitted **Node** before `serialize_target` — the input Stage C
needs. Comparison substrate (emitted Node vs emitted bytes) is operator-pending (merry-crab-687).
When it lands, the fail-closed runner flips to `Accepted` + real-digest match.

## 2. Prerequisites / dependencies

1. **De-fork / cross-tree import** ([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md) §1) — the compiler
   closure is only well-defined once v2 imports the single `dsl/std` authority, not its mirror copies.
   Cross-tree import is wired but fail-closed (`03_name_resolve.dag:644`).
2. **Whole-tree resolve cost** — the 590s+ combined-tree resolve is the practical wall for a fresh
   whole-tree green receipt; relevant to both the CI gate and Stage C.
3. **TS-target completeness** — only needed if the end goal is Track-(b) TypeScript runtime; the `add`
   slice is proven, the compiler uses far more of the language.

## 3. Open questions (for the operator)

1. **Rust or TypeScript self-host?** (see END GOAL block). The carrier drives Rust today.
2. **Reopen direction** (raised by merry-deer-374, `node://session-merry-deer-374`): should the reopened
   "v2 self host (p2)" item drive **(a)** finishing the Route-A green bootstrap (merge seed-green branch
   → green regen → flip the lockstep gate), or **(b)** folding into a pure-`.dag` coherence restart?
3. **Rust seed end-state** — retire the `src/v1` seed once self-host lands, or keep it as a second
   realization?
4. **Scope** — whole `src/v2` to a fixed point, or a defined compiler-core subset first?

## 4. Dissolution trigger (DESIGN §6)

Delete this doc when Stage C lands: the whole v2 compiler emits a bit-identical copy of itself and
`self_host.dag` flips from fail-closed to asserting the real-digest match. The self-host witness then
*is* the authority and this tracker is redundant.

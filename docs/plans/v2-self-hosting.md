# Plan — v2 self-hosting (toward a TypeScript fixed point)

**Status:** planning skeleton · **DESIGN.md + the carriers remain the authority** — this doc is a
tracker, not a fact ledger (DESIGN §6). A task's real state is its branch/PR, not this file. Linked
from `ROADMAP.md §3`. Related to but distinct from the de-fork audit
([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md)): de-fork collapses duplication; self-hosting makes
v2 compile itself. De-fork is a *prerequisite* (the compiler closure must have one std authority), not
the goal.

**Verified against the live tree on 2026-06-21.** Line numbers are receipts; re-check before acting.

> **END GOAL — CONFIRM.** Best current reading of "self-hosting v2 (TypeScript)": v2 compiles its own
> `src/v2` source and **emits TypeScript** that runs the compiler, reaching a fixed point where the
> emitted compiler re-emits a **bit-identical** copy of itself. TypeScript becomes the realization/host
> the Rust v1 seed shrinks toward zero against (DESIGN §7: "Rust is one realization, a seed that shrinks
> to zero"). **Open:** is TS the *target* of the self-host fixed point (replacing Rust as the runtime),
> or is "emit valid TS" a separate milestone from "v2 compiles v2"? See §4.

---

## 0. Where self-hosting actually stands

Honest current contract, from `src/v2/compiler/self_host/fixed_point_test.dag` (the **only**
live-executed self-host coverage, discovered by `.github/workflows/ci.yml`):

> ⚠️ NOT SELF-HOSTING ACHIEVED. These greens prove the comparison machinery on hand-built FIXTURE
> nodes and the fail-closed runner contract — nothing more.

So the machinery to *compare* two emitted stages exists and is green on fixtures; the whole-compiler
fixed point over real digests does not yet run.

## 1. The stages (A → B → C)

- **Stage A — comparison machinery** (DONE on fixtures). `self_host_fixed_point_digests_match` /
  `stage_emission_equality` proven over hand-built fixture nodes; whole-compiler runner asserted
  **fail-closed** (`self_host_fixed_point_fails_closed_until_substrate`) — the honest contract until C.
- **Stage B — candidate generation** (LIVE, partial). `src/v2/compiler/self_host/candidate_generation.dag`:
  `generate_stage_candidate_from_ingest` drives `assemble_program_from_ingest → infer → translate`
  (`06_translate`) and captures the emitted **Node** *before* `serialize_target` — the missing input
  Stage C needs. The comparison substrate (emitted Node vs emitted bytes) is operator-pending
  (merry-crab-687 brief). Closure ingest witnesses: `compiler_closure_emit_from_ingest_test.dag`.
- **Stage C — whole-compiler fixed point** (NOT STARTED / gated). v2 compiler emits itself, stage1 ==
  stage2 over real merkle digests. When it lands, the fail-closed witness flips to assert `Accepted`
  + `self_host_fixed_point_digests_match` over the real emitted stages.

## 2. The TypeScript emit path

TypeScript is already a modeled emit target, more mature than most:

- Language model: `src/v2/extdeps/languages/typescript.dag` + `language_model/typescript_r2a|r2b|r3_external.dag`;
  formatter `extdeps/formatters/prettier.dag`.
- Cross-language emit **proven by execution** on the `add` slice:
  `src/v2/compiler/manual/cross_language_add_python_to_typescript_test.dag` walks Python source →
  canonical Node core → **TypeScript** source (`neutralize_core_for_target` maps python `int` → TS
  `number`); hermetic per-merge witness + a heavy ~627s real-tree receipt.

This is the "N + M, not N × M" thesis (one core, N source models, M target models) — TS is one M.

## 3. Prerequisites / dependencies

1. **De-fork / cross-tree import** ([dsl-v2-defork-audit.md](dsl-v2-defork-audit.md) §1). The compiler
   closure (what v2 must compile to host itself) is only well-defined once v2 imports the single
   `dsl/std` authority instead of its bootstrap mirror copies. Cross-tree import is wired but
   fail-closed (`03_name_resolve.dag:644`).
2. **Resolver / generic-instantiation cost** — historically the self-host blocker (resolve blowup on
   the real tree). Confirm current state before Stage C.
3. **TS language-model completeness** — the `add` slice is proven; the compiler uses far more of the
   language (records, coproducts, folds, generics). Gap census from `add` → full `src/v2` needed.

## 4. Open questions (for the operator)

1. **TS as the fixed-point target?** Is the north star "v2 emits TS that re-emits bit-identical TS"
   (TS replaces Rust as the runtime), or are "v2 compiles v2" (any target) and "emit production TS"
   two separate end goals? This decides whether §1 Stage C is measured in TS bytes or Node digests.
2. **Rust seed end-state** — does the Rust v1 seed (`claim_executor`, the interpreter) get *retired*
   once TS self-host lands, or kept as a second realization?
3. **Scope of "self-host"** — whole `src/v2` to a fixed point, or a defined compiler-core subset first?

## 5. Dissolution trigger (DESIGN §6)

Delete this doc when Stage C lands: the whole v2 compiler emits a bit-identical copy of itself and
`fixed_point_test.dag` flips from fail-closed to asserting the real-digest match. At that point the
self-host witness *is* the authority and this tracker is redundant.

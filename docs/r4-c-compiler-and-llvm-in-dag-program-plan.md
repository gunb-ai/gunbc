# C Compiler + LLVM in .dag — execution promotion plan

**Status**: DRAFT (2026-05-12). Authored per operator directive 2026-05-12T~19:30Z: parallel program to ctrl-migration; promote existing research-viability artifacts to executing production program.

**Authority**: existing research plan at `~/ctrl/research/market/viability/demos/c-compiler-in-dag/PLAN.md` (Brian-approved 2026-05-04 via `gunb-ai/ctrl#339` with locks A1-A10). This doc proposes the **execution shape** that consumes the research plan; the workstream content stays authoritative there.

**Companion docs**:
- `docs/design-decomposition-algebra.md` — decomposition algebra (PR #2775)
- `docs/r4-ctrl-dag-migration-project-plan.md` — sibling parallel program (PR #2775)

---

## §1. Mission + scope decision

**Mission**: model C compiler + LLVM IR + (optionally) XLS IR in `.dag` substrate so the .dag model is **single authority** for compiler behavior. Bidirectional extdeps for C/C++ (ingestion + emission). Demonstrates the gunbc thesis (omni-emission; substrate-as-integration-surface-coherence) on a production compiler infrastructure surface.

**Scope decision required** — two interpretations of "C compiler + LLVM in .dag":

**Interpretation (a) — "LLVM IR + C frontend modeled in .dag"** (existing research scope):
- C subset frontend (Kaleidoscope-Ch8-equivalent per lock A4: numeric expressions + control flow + functions + locals + integer/double types) ingested → typed `.dag`
- LLVM IR (v20.1.7 per lock A6) modeled as `.dag` substrate
- LLVM IR emitted (`emit_module: Module -> String`) — W4 spike done
- LLVM IR ingested (`parse_module: String -> Module`) — W2d, ~6-10 worker-weeks
- Bidirectional symmetry: same `.dag` substrate compiles to native binaries (LLVM) OR to Verilog (XLS, per lock A2)
- This is the existing PLAN.md scope. Multi-month effort but bounded.

**Interpretation (b) — "LLVM entirely in .dag"** (much larger scope):
- All of (a), plus:
- LLVM optimizer passes (Mem2Reg, InstCombine, GVN, LICM, ...) re-implemented as `.dag` lenses + transforms
- Target-specific lowering (x86, ARM, etc.) modeled in `.dag`
- Register allocation, instruction selection, scheduling as `.dag`
- Replaces LLVM-the-C++-library entirely
- Multi-year effort; ~10M LOC equivalent

**Proposed**: start with (a). It's already scoped + has W4 spike landed. (b) is a Phase 6+ extension *after* (a) ships. Operator confirm.

---

## §2. Existing research scaffolding audit

Inventoried from `~/ctrl/research/market/viability/demos/c-compiler-in-dag/`:

### Authoritative planning doc
- `PLAN.md` — Brian-approved Round-2 plan, 13 workstreams (W0-W11), locks A1-A10. **This stays the authoritative workstream content**; the doc you're reading is the execution shape that consumes it.

### W0 — shared abstract IR primitives (DONE on paper)
`research/.../extdeps/common/ir/`:
- `bitvector.dag` — fixed-width bitvectors
- `integer.dag` — integer types (load-bearing pair with bitvector)
- `array.dag` — array primitive
- `product.dag` — product type
- `ssa_value.dag` — SSA value primitive

**Promotion gate** (added per codex/operator BLOCKING 2026-05-12T20:19Z): wholesale move into `dsl/extdeps/common/ir/` risks parallel substrate for existing std primitives. Each of the 5 W0 files goes through a per-concept **M9 + extdeps-spec grounding gate** before promotion:

1. **M9 DFS from `dsl/std/`** — for each primitive (bitvector / integer / array / product / ssa_value), DFS the existing std concept DAG. Where the concept already exists (`std.bit` / `std.integer` / `std.constructors.Product`), the W0 file MUST reuse-or-extend the std primitive rather than introduce a parallel carrier. Pure parallel-substrate promotion is forbidden.
2. **Extdeps-spec citation** — for each primitive that is genuinely IR-domain (not reusable from std/), cite the external authority (e.g., LLVM IR language reference §"Type System" for bitvector / array; SSA spec for ssa_value) so the extdeps placement is grounded in an external-spec fact, not in research-folder convenience.
3. **Decision matrix per file** (must be in the promotion PR description):
   - `bitvector.dag` → likely **extend** `std.bit` (if std.bit covers single-bit only) or **reuse** if std.bit handles fixed-width. Spec: LLVM IR `iN` types.
   - `integer.dag` → likely **reuse** `std.integer` (integer is a math primitive, not IR-specific). Spec: LLVM IR integer-types reference for any IR-specific narrowing.
   - `array.dag` → **MUST preserve element cardinality** (LLVM array `[N x T]` is a fixed-size container; cardinality N is a load-bearing fact per INVARIANTS P2 facts-flow-forward). Concrete options: (1) reuse `FixedArray<T, N>` or `Vector<T, N>` if std has a cardinality-carrying container; (2) extend `List<T>` to carry an explicit cardinality field; (3) introduce a new cardinality-aware `LlvmArray<T, N>` carrier in std/ BEFORE the extdeps promotion lands. Plain `List<T>` reuse is **forbidden** — it drops the cardinality fact. M9 DFS must find or land the cardinality-carrying carrier first; promotion blocks until that substrate exists. Spec: LLVM IR array-type reference (`[N x T]` syntax).
   - `product.dag` → **reuse** `std.constructors.Product` (existing M2 type-authority). Promotion is forbidden if it creates parallel Product carrier.
   - `ssa_value.dag` → likely **new extdeps carrier** (SSA is genuinely IR-domain, no std equivalent). Spec: SSA-form references.

Promotion destination is `dsl/extdeps/common/ir/` per PLAN.md §"Bidirectional-extdeps reframe" **only after** each file passes the gate above. Files failing the gate either dissolve into std reuse/extension PRs or get rescoped to extdeps-only IR-domain concepts.

### W4 — LLVM IR text emitter spike (DONE)
`research/.../spike-llvm-ir/`:
- `ir_substrate.dag` + `ir_types.dag` — Module / Function / BasicBlock / Instruction (Ret) / Value (IntegerConstant | LocalRef) / IrType (I32 | Void) declared
- `ir_emit.dag` — `emit_module: Module -> String` pure emitter
- `trivial_program.dag` — constructs `define i32 @main() { ret i32 42 }`
- `expected_output.ll` — hand-written reference
- `VALIDATION.md` — pass/fail criteria

**Tier 1 surface deliberately omitted** (per spike scope): named basic-block emission + CFG successors / arithmetic / memory / branch / call / phi / conversion / comparison / pointers / aggregates / metadata / debug / attributes / datalayout / target triple / linkage / calling conventions. These are W2 production work.

### W7 — XLS/Verilog IR spike (DONE)
`research/.../spike-verilog/`:
- Parallel structure to W4 spike: `ir_substrate.dag` / `ir_emit.dag` / `ir_types.dag` / `trivial_program.dag` / `expected_verilog.v` / `expected_xls_ir.txt`

### Phase-2 expression evaluator (DONE)
`research/.../lens-counterfactuals/phase-2-expression-evaluator/`:
- `evaluator_types.dag` — typed expression AST
- `evaluator_eval.dag` — pure evaluator
- `evaluator_effects.dag` — effect modeling
- `evaluator_lens_slot.dag` — lens slot for invariant application
- `evaluator_fixture.dag` — test fixtures

### Lens counterfactuals (3 real-world bug case studies)
`research/.../lens-counterfactuals/path-1/`:
- `llvm-tbaa-metadata-instcombine-101164` — LLVM TBAA metadata bug; gunbc-decl.dag + lens-application.dag
- `pytorch-scatter-add-determinism-50469` — PyTorch determinism bug
- `xls-useless-io-recv-tuple-4008` — XLS bug

Each shows: "this bug would be impossible by construction if the IR were modeled in .dag."

### Existing gunbc compiler lens substrate (CONSUMED)
`src/v3/lenses/` — 16 working lens instances ready to consume:
- complexity / cost / cost_target_realization / dag_shape
- effect_enumeration / emission_provenance / idempotency
- infer_helpers / lens_composition_associative_witness / lower_helpers
- named_function_count / parallelism / provenance
- structural_resolution / unused_parameters / variant_payload

These are EXACTLY the compiler-tier lens instances a production optimizer needs.

---

## §3. Promotion plan: research → production substrate

### What promotes immediately (Phase A — small, NOW)

| Research artifact | Promote to | Effort | Why now |
|---|---|---|---|
| W0 `extdeps/common/ir/*.dag` (5 files) | `dsl/extdeps/common/ir/` in gunbc | 1-2 PRs, ~3 days | Already authored; review + merge |
| W4 spike (`ir_*.dag`) | `dsl/extdeps/llvm/v20/ir/` in gunbc | 1-2 PRs, ~3-5 days | Spike validated; promote to versioned extdeps |
| W7 spike (XLS/Verilog) | `dsl/extdeps/hardware/xls_ir/ir/` in gunbc | 1-2 PRs, ~3-5 days | Spike validated; same promotion shape |
| Phase-2 expression evaluator | `dsl/std/expression_evaluator/` OR `src/v3/expression_evaluator/` | 1-2 PRs, ~2-3 days | Already typed; minor refactor |

**~6-9 PRs over 1-2 weeks** to land all current research into production substrate.

### What requires real engineering (Phase B — multi-week)

| Workstream | Effort | Description |
|---|---|---|
| W2a — LLVM IR Tier 1 surface | 4-8 worker-weeks | Add named basic-block emission, CFG, arithmetic, memory, branch, call, phi, conversion, comparison, pointers — the rest of LLVM IR beyond the Ret/i32 spike |
| W2d — LLVM IR ingestion (`parse.dag`) | 6-10 worker-weeks | Parser for LLVM IR text → typed `.dag` (bidirectional symmetry) |
| W3a — C ingestion (Ch8-equivalent) | 6-10 worker-weeks | C subset lexer/parser/AST in `.dag` |
| W3b — C emission | 4-6 worker-weeks | `.dag` → C source for backwards compatibility |
| W11 — C++ ingestion (xlscc-subset) | 8-12 worker-weeks | C++ ingestion pinned to xlscc commit `bab31d0817` (per lock A5) |
| W4-extended — XLS IR Tier 1 surface | 4-6 worker-weeks | Same as W2a for XLS IR |
| W8 — Mem2Reg as lens (pressure test) | 3-5 worker-weeks | First real optimization pass as `apply_lens` invariant |
| W10 — `apply_lens` substrate (v0 demo per A8) | 2-4 worker-weeks | Lens-binding pattern; v0 = lens calls inside Transform body |
| W1 / W5 / W6 / W9 — substrate-prereq work | varies | per PLAN.md |

**~6-12 months** for full Phase B with 4-6 parallel workers.

### What requires Phase 3 emission targets (Phase C — gated)

Mostly the C compiler EXECUTION path:
- LLVM linking / object-file generation — needs `dsl/extdeps/llvm/v20/codegen.dag` or shell-out to llc
- Native binary execution — needs runtime linkage emission
- C++ template instantiation — much harder substrate

---

## §4. Program tree (new Director under PM/CEO)

```
operator (Brian)
└── deep-wolf-155 (CEO/PM, root)
    ├── zesty-bear-812 (gunbc R3-close Director, idle)
    ├── clever-ant-97 (Ctrl-Migration Director, idle — just spawned)
    └── C-Compiler+LLVM Director  ← NEW (proposed; awaiting spawn confirmation)
        ├── Substrate Mgr           (Phase A — promote research to production substrate)
        │   └── ~6-9 PRs landing W0/W4/W7/phase-2-evaluator into gunbc
        ├── Frontend Mgr            (Phase B — W3a/W11 — C/C++ ingestion)
        │   └── 2-4 parallel workers on C + C++ ingestion
        ├── IR Mgr                  (Phase B — W2/W4-extended/W2d — LLVM IR + XLS IR Tier 1 + ingestion)
        │   └── 2-3 parallel workers on LLVM IR surface
        ├── Lens-Application Mgr    (Phase B — W8/W10 — Mem2Reg + apply_lens substrate)
        │   └── 1-2 workers consuming src/v3/lenses/ for production optimizer
        └── Pressure-Test Mgr       (Phase B — lens-counterfactual validation)
            └── 1-2 workers extending the 3 case studies + integration tests
```

**Why 5 Mgrs (vs ctrl-migration's 4)**: the workstream parallelism is higher because Frontend / IR / Lens-Application / Pressure-Test are genuinely independent lanes once Phase A substrate lands. Substrate Mgr is the critical-path bottleneck Phase A; rest spawn after.

**Coordination with ctrl-migration + gunbc R3-close**: all three programs are PM children. PM owns inter-program coordination.

---

## §5. Phase sequencing

```
Phase 0 (DONE — research already in ~/ctrl/research/)
  ├── PLAN.md authored, Brian-approved with A1-A10 locks
  ├── W0/W4/W7/phase-2-evaluator/lens-counterfactuals authored
  └── 13 workstreams scoped
       ↓
Phase A (~1-2 weeks; promote research → production substrate)
  ├── Substrate Mgr lands W0 + W4 + W7 + phase-2-evaluator into gunbc
  └── ~6-9 PRs, doc-shape + minor refactor
       ↓
Phase B (~6-12 months parallel; the real engineering)
  ├── Frontend Mgr: W3a (C) + W11 (C++) ingestion
  ├── IR Mgr: W2a + W4-extended Tier 1 surfaces + W2d (LLVM IR ingestion)
  ├── Lens-Application Mgr: W8 (Mem2Reg-as-lens) + W10 (apply_lens v0)
  └── Pressure-Test Mgr: extend lens-counterfactuals; validation harnesses
       ↓
Phase C (multi-month; emission gates)
  ├── `dsl/extdeps/llvm/v20/codegen.dag` — native binary emission
  ├── `dsl/extdeps/hardware/xls_ir/codegen.dag` — Verilog emission via XLS
  └── Runtime / linker / object-file emission targets
       ↓
Phase D (open-ended; "LLVM entirely" expansion if pursued)
  └── Optimizer passes, target lowering, regalloc, instruction selection as .dag
```

---

## §6. First-week concrete actions

**Operator (Brian) — Day 0**:
1. Review + ratify this execution plan + scope decision (a) "C frontend + LLVM IR" vs (b) "LLVM entirely"
2. Confirm spawn of C-Compiler+LLVM Director under PM (parallel to clever-ant-97)
3. Optionally: ratify research → production promotion authority (existing PLAN.md A1-A10 locks carry forward to production-tier work)

**PM (deep-wolf-155) — Day 1**:
1. Spawn C-Compiler+LLVM Director via `dashboard-ops work-items create`
2. Dispatch charter (this doc + PLAN.md + research artifacts)

**C-Compiler+LLVM Director — Day 1**:
1. Ratify scope per operator directive
2. Spawn Substrate Mgr first (Phase A critical-path)
3. Defer Frontend / IR / Lens-Application / Pressure-Test Mgrs to Day-N when Phase A nears landing (~Day 7-10)

**Substrate Mgr — Day 2-7**:
1. Author 5 promotion briefs:
   - Brief A1: W0 promotion (`dsl/extdeps/common/ir/`) — 5 files
   - Brief A2: W4 promotion (`dsl/extdeps/llvm/v20/ir/`)
   - Brief A3: W7 promotion (`dsl/extdeps/hardware/xls_ir/ir/`)
   - Brief A4: phase-2-evaluator promotion
   - Brief A5: PLAN.md → gunbc-side doc move (or symlink) per `feedback_dissolution_authority_not_file_presence.md`
2. Spawn 3-5 workers in parallel (each brief is small; ~3-5 day worker time)

**Phase A wraps ~Day 10** — Phase B Mgrs spawn:
- Frontend Mgr (W3a + W11)
- IR Mgr (W2a + W4-extended + W2d)
- Lens-Application Mgr (W8 + W10)
- Pressure-Test Mgr (lens-counterfactual extension)

---

## §7. Cross-program coordination

Three sibling programs under PM:

| Program | Director | Critical interface |
|---|---|---|
| gunbc R3-close | zesty-bear-812 | Substrate primitives in `dsl/std/`; lenses in `src/v3/lenses/` |
| Ctrl-migration | clever-ant-97 | `dsl/ctrl/*.dag` for ctrl/ subsystems |
| C-Compiler+LLVM | TBD (proposed spawn) | `dsl/extdeps/{common,llvm,hardware}/` |

**Shared concerns**:
- **`dsl/std/` substrate** — all three programs may need substrate-shape changes. Conflicts route through PM.
- **`src/v3/lenses/`** — C-Compiler+LLVM Lens-Application Mgr consumes lens instances; coordination with zesty-bear-812 R3 Substrate Mgr on lens-instance shape.
- **Practice 4 dissolution discipline** — applies uniformly across all three programs.
- **`Lens<C>` substrate** at `src/v3/std/lens.dag` — Director-locked 6-field shape; any extension request from C-Compiler+LLVM program routes to Substrate Mgr via PM.

**Director-tier autonomy**: each Director owns their program scope independently; PM bridges substrate-shape conflicts.

---

## §8. What this plan does NOT do

- Implement any compiler code (Phase A is doc-shape promotion only)
- Override the existing PLAN.md A1-A10 locks (those stay authoritative for workstream content)
- Commit to Interpretation (b) "LLVM entirely" — that's Phase D, operator-decision
- Set hard timelines on Phase B (worker-weeks per workstream are estimates from PLAN.md; actual cadence depends on staffing)

---

## §9. Open questions for operator

**Q-A: Scope decision** — Interpretation (a) "C frontend + LLVM IR substrate" or (b) "LLVM entirely"?
- Proposed: (a) now, (b) as Phase D extension if pursued.

**Q-B: Director spawn** — should PM spawn the new Director now (as 3rd child under PM, parallel to clever-ant-97), or hold pending ratification?
- Proposed: spawn now per CEO authority; Director can ratify charter when alive.

**Q-C: Research-to-production authority** — do A1-A10 locks in `~/ctrl/research/.../PLAN.md` carry forward to the production-tier program automatically, or require re-ratification?
- Proposed: carry forward (locks were Brian-approved); document any deltas as new locks.

**Q-D: PLAN.md home** — keep authoritative content at `~/ctrl/research/.../PLAN.md` with gunbc-side doc pointing to it, OR migrate PLAN.md to `~/gunbc/docs/r4-c-compiler-llvm-research-plan.md`?
- Proposed: migrate the authoritative content to gunbc once Phase A lands (production substrate lives in gunbc; the plan should live with the substrate).

**Q-E: XLS / hardware target priority** — keep Tier 4 (XLS) ship-at-launch per A2, or drop given the urgency of "replace ctrl/ ASAP" framing for the sibling program?
- Proposed: keep XLS per A2; the bidirectional-thesis claim depends on it; XLS work doesn't compete with ctrl-migration for the same Mgr time.

**Q-F: Lens consumption from `src/v3/lenses/`** — should the C-Compiler+LLVM program treat the 16 existing lenses as authoritative-and-reused, or audit each for compiler-domain fit and propose extensions?
- Proposed: audit each (Lens-Application Mgr's first task); propose extensions via Substrate Mgr if needed.

---

## §10. Cross-references

- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/PLAN.md` — Round-2 authoritative plan
- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/README.md` — research README
- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/spike-llvm-ir/` — W4 spike
- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/spike-verilog/` — W7 spike
- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/extdeps/common/ir/` — W0 primitives
- `~/ctrl/research/market/viability/demos/c-compiler-in-dag/lens-counterfactuals/` — 3 real-world bug case studies
- `dsl/std/algebra.dag`, `dsl/std/logic.dag`, `dsl/std/bit.dag` — gunbc primitives W0 builds on
- `src/v3/std/lens.dag` — Director-locked Lens<C> substrate (production lens carrier)
- `src/v3/lenses/` — 16 production lens instances ready to consume
- `~/gunbc/THESIS.md` — omni-emission / bidirectional-extdeps thesis
- `~/gunbc/docs/thesis/what-else-falls-out.md` §"Frontend/backend agnosticism" — bidirectional-extdeps thesis claim
- `feedback_lenses_not_passes.md` — lenses are analyses (read-only); passes are Transforms
- gunb-ai/ctrl#339 — Brian's approval of A1-A10 locks

---

— Authored by deep-wolf-155 (PM/CEO) 2026-05-12 per operator directive for parallel C compiler + LLVM program planning. Promotion of existing research-viability artifacts to executing production program; existing PLAN.md A1-A10 locks carry forward.

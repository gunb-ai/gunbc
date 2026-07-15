# Language-target self-host frontier — beginning-to-end dependency map

**Status:** staging map, captured pre-un-shelve. The multi-language self-host lane is §4-shelved (ROADMAP: "TypeScript first-class emit + TS self-host … resumes after the Rust fixed point"). This map exists so the lane can be **staffed continuously** the moment it un-shelves: every dependency is understood from beginning to end, Rust is the model, and the exotic languages are placed as deliberate stresses on it. `solve` is resolved as **not a substrate change** — see [solve-higher-order-design.md](solve-higher-order-design.md).

**Governing invariant (the anti-hack guarantee):** every target ends in exactly one of two *declared* states — **green-by-execution at a named bar**, or a **typed frontier row** (`bar reached · reason · dissolution trigger`). No silent `body_lexeme: String` anemic leaves, no orphan un-enrolled smokes, no anchor-stub masquerading as a finished target. "Done" and "refused-with-reason" are the only terminal states. This is the §7 typed self-host frontier applied to every target at once.

Bars: **(a)** grammar-inverse translation rows on the committed TargetModel · **(b)** emit witness produces the expected source · **(c)** a real toolchain runs the output green-by-execution through the *typed* `run_emit_host` transport · **self-host** the target emits the compiler's own sources, behaviorally equivalent (§7).

---

## Two orthogonal stress axes

The whole point of exotic-first is that the curly-brace family *confirms* the design while the exotics *falsify* it. There are two independent things to falsify:

- **Emit-generality axis** — does "one grammar read backwards" reach targets whose computational model is *not* imperative-forward? Stressed hardest by: **Verilog** (concurrent / clock-stepped), **SPICE** (continuous-time / ODE), **LLVM-IR** (SSA / no runtime). If emit generalizes here, the "medium-agnostic, one grammar N directions" claim (§4) is real.
- **Self-host axis** — does the seed-shrink story (§7) hold in a *second* language, or is it secretly Rust-shaped? Stressed by generalizing the self-host frontier runner past Rust to a second runnable language.

**The emit-only family can only stress the first axis** (you cannot run a compiler as a circuit or as bare SSA). The runnable family stresses both. Keeping these axes separate is what keeps the lane honest about what each language *proves*.

---

## Shared foundation (mostly built by Rust — the front-loaded cost, already paid)

| id | foundation | state |
| --- | --- | --- |
| **F0** | `emit` / `emit_module` pipeline (walks TargetModel edges backward; new target = rows, no pipeline edit) | ✓ done |
| **F1** | `TargetModel` 4-edge + grammar-inverse translation rows | ✓ done |
| **F2** | VEP (`TargetValueExpressionProjection`) — the general body producer | full (rust) · partial (TS: match/loop/bind-in unwired) · **absent** (python, ecmascript) |
| **F3** | `BlockEvaluationMode` statement spine (`ValueProducing` vs `StatementSequenced`) | ✓ type exists; TS's `StatementSequenced` arm produces the correct fail-closed refusals |
| **F4** | `ProcessProgram` transport + `run_emit_host` + `host_tool_program_name` registry | ✓ machinery; **registry knows only `cargo`, `cc`** (C Phase 0 added `cc`) |
| **F5** | self-host frontier runner (`run_test_claim_module_emit_vs_eval`: emit → build → run → compare-to-eval) | ✓ wired end-to-end **for Rust only** |
| **F6** | `solve` (`Residual`/`Constraint` std type + `solve` fold + host-solver handler) | **off critical path** — not needed for emit-to-simulator; see solve doc |

### The two highest-leverage shared dependencies (do these once, everyone inherits)

1. **F4 `host_tool_program_name` registry extension** — this ~8-line function gates **bar-c for every language**. Registering `node`, `python3`, `go`, and the simulator tools (`ngspice`, `verilator`/`iverilog`, `llc`/`clang`, `wasmtime`) makes each target's *already-declared* runtime row runnable through the typed self-host path in one edit each. Highest-leverage de-fork on the board.
2. **F2 VEP completion** — wire the unwired forms (match / loop / bind-in) **once** in the `StatementSequenced` arm; the entire interpreted family inherits body breadth. Python additionally needs its *first* VEP edge.

---

## Per-target frontier table (grounded from the 2026-07-15 survey)

| target | family | current bar | honest end-state | key deps | stress axis |
| --- | --- | --- | --- | --- | --- |
| **rust** | compiled | self-host ✓ | reference model | — | (confirms) |
| **cpp (C)** | compiled | bar-c ✓ (Phase 0) | self-host | F4(`cc`)✓, F5-generalize | (confirms) |
| **go** | compiled | bar-a | **self-host** (first non-rust self-host — the F5 generalization proof) | F4(`go`), surface spellings, F5 | **self-host axis** |
| **java / kotlin / swift** | compiled | skeleton | bar-c → self-host | F4(tool), spellings, inherit F2/F3/F5 | (confirms — parallel) |
| **typescript** | interpreted | bar-b (9 VEP families); bar-c RED | self-host | F4(`node`/`npx`), F2 completion (match/loop/bind-in), operator catalog | self-host axis |
| **python** | interpreted | add-only, no VEP | bar-c → self-host | F2 *first* edge, operator catalog, F4(`python3`) | self-host axis |
| **ecmascript** | interpreted | orphan (0 consumers) | **decide first**: adopt as TS's JS base, or delete | (adoption decision) | — |
| **lean** | expression/ML | bar-a | self-host on an *expression* spine (not statement) | F3 generalization (expression-mode), F2, F5 | self-host axis (spine falsifier) |
| **verilog** | HDL | bar-a anchor; 34 `body_lexeme:String` scars | **bar-c green + frontier row `self-host: N/A (hardware)`** | decompose `body_lexeme` → structured; F4(`verilator`/`iverilog`) | **emit-generality (concurrent)** |
| **spice (+modelica)** | analog format | bar-b live; ngspice oracle in history | **bar-c green + frontier row `self-host: N/A (analog)`**; dual-emit ngspice+Modelica | F4(`ngspice`); already the §2 "one model → N backends" proof | **emit-generality (continuous)** |
| **llvm_ir** | IR/backend | bar-a core anchor | **bar-c green + frontier row `self-host: N/A (no runtime)`** — the Rust/cargo-free lowering path | F4(`llc`/`clang`) | **emit-generality (SSA)** |
| **wasm** | IR/backend | most-developed exotic; has transport descriptor | bar-c green + frontier row `self-host: N/A` | F4(`wasmtime`) | emit-generality (stack machine) |
| **machine_code / ptx** | ISA/GPU | bar-a anchor | bar-c green + frontier row `self-host: N/A` | F4(assembler/`ptxas`) | emit-generality (ISA) |

---

## Sequencing — Rust-as-model, then stress (dependency-ordered)

- **Phase A — now-ish (pre-un-shelve):** this map + the `solve` rationale. No emit code. ← *we are here*
- **Phase B — foundation de-fork (the barrier; everything downstream inherits):** F4 registry extension + F2 VEP completion. Once B lands, C/D/E run largely in parallel.
- **Phase C — self-host axis proof:** generalize F5 past Rust to **Go** (first non-Rust self-host). This is the real "does the seed-shrink story generalize" milestone. If it holds, java/kotlin/swift are near-mechanical parallel fan-out.
- **Phase D — emit-generality axis (parallel):** Verilog · SPICE · LLVM-IR to bar-c + frontier rows. Each decomposes its `body_lexeme:String` scars and registers its simulator/toolchain transport. This is where the design gets *stressed*; a construct that won't lower produces a typed, located refusal that *names the design gap* — the refusal is the product.
- **Phase E — interpreted self-host:** TS → Python (Python inherits TS's VEP completion).
- **Phase F — expression spine:** Lean (falsifies "is F3 secretly imperative-shaped?").

Each language is **priced by the design risk it displaces**, not by completeness (§6): Verilog=concurrent, SPICE=continuous, LLVM=SSA/self-host-boundary, Lean=expression-spine. A confirm-only language (java/kotlin/swift) earns its slot only as cheap parallel fan-out after its family's proof lands.

---

## Staffing note (for continuous dispatch)

Off this map, the natural fan-out is **one child session per language axis**, gated on Phase B. The dependency structure that makes this safe: **B is the only hard barrier**; after it, the self-host axis (C, then the compiled fan-out) and the emit-generality axis (D) have no cross-dependency, so they staff independently. Lean (F) and the interpreted family (E) each ride one Phase-B deliverable (F3-expression-mode and F2-completion respectively). ecmascript's adopt-or-delete decision is a prerequisite gate on the interpreted family, not parallel work.

## Dissolution trigger (DESIGN §6)

**Un-shelve is an operator decision (ROADMAP §4 → §1/§3 by PR, priced in displaced cost).** When it fires, this doc + the solve doc are the lane's charter; the frontier table rows migrate onto per-target carrier rows, and this file dissolves.

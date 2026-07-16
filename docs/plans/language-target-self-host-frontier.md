# Language-target self-host frontier — beginning-to-end dependency map

**Status:** staging map, captured pre-un-shelve. The multi-language self-host lane is §4-shelved (ROADMAP: "TypeScript first-class emit + TS self-host … resumes after the Rust fixed point"). This map exists so the lane can be **staffed continuously** the moment it un-shelves: every dependency is understood from beginning to end, Rust is the model, and the exotic languages are placed as deliberate stresses on it. `solve` is resolved as **not a substrate change** — see [solve-higher-order-design.md](solve-higher-order-design.md).

**Governing invariant (the anti-hack guarantee):** every target ends in exactly one of two *declared* states — **green-by-execution at a named bar**, or a **typed frontier row** (`bar reached · reason · dissolution trigger`). No silent `body_lexeme: String` anemic leaves, no orphan un-enrolled smokes, no anchor-stub masquerading as a finished target. "Done" and "refused-with-reason" are the only terminal states. This is the §7 typed self-host frontier applied to every target at once.

**Not yet built (declared gap, un-shelve-scale — DO NOT pretend present):** the typed `LanguageTargetFrontierRow` carrier that would make each row below *machine-checked* — plus its completeness lens and per-row RED witnesses — **does not exist today**. This table is the interim prose ledger. **Enrollment trigger** = lane un-shelve → author the carrier + witnesses. It is recorded here as a gap, not claimed as present machinery.

**Offline / on-demand smokes (honest exception to "no orphan un-enrolled smokes"):** the C and Rust F5 emit-vs-eval witnesses live under `test/claim/execution/`, which is in `witness_exclusion_substrings` (`ci_layer_roots.dag`), so they are **excluded from discovery and un-enrolled** in per-PR CI. They are labeled **OFFLINE** — run offline / on-demand — with a **named enrollment trigger = lane un-shelve** (un-shelve → move them out of the exclusion set / enroll). They are NOT claimed as enrolled.

Bars: **(a)** grammar-inverse translation rows on the committed TargetModel · **(b)** emit witness produces the expected source · **(c)** a real toolchain runs the output green-by-execution through the *typed* `run_emit_host` transport · **self-host** the target emits the compiler's own sources, behaviorally equivalent (§7).

---

## Two orthogonal stress axes

The whole point of exotic-first is that the curly-brace family *confirms* the design while the exotics *falsify* it. There are two independent things to falsify:

- **Emit-generality axis** — does "one grammar read backwards" reach targets whose computational model is *not* imperative-forward? Stressed hardest by: **Verilog** (concurrent / clock-stepped), **SPICE** (continuous-time / ODE), **LLVM-IR** (SSA / no runtime). If emit generalizes here, the "medium-agnostic, one grammar N directions" claim (§4) is real.
- **Self-host axis** — does the seed-shrink story (§7) hold in a *second* language, or is it secretly Rust-shaped? Stressed by generalizing the self-host frontier runner past Rust to a second runnable language. (Reality check: Rust itself is **not yet self-hosted** — see the F5 / rust rows below.)

**The emit-only family can only stress the first axis** (you cannot run a compiler as a circuit or as bare SSA). The runnable family stresses both. Keeping these axes separate is what keeps the lane honest about what each language *proves*.

---

## Shared foundation (Rust-built; the front-loaded cost — partly still owed)

| id | foundation | state |
| --- | --- | --- |
| **F0** | `emit` / `emit_module` pipeline (walks TargetModel edges backward; new target = rows, no pipeline edit) | ✓ done |
| **F1** | `TargetModel` 4-edge + grammar-inverse translation rows | ✓ done |
| **F2** | VEP (`TargetValueExpressionProjection`) — the general body producer | **partial** (rust: `^rust_token_unwired_{else,loop,match,fat_arrow}`, `rust.dag:1148-1176`) · partial (TS: match/loop/bind-in unwired) · **absent** (python, ecmascript) |
| **F3** | `BlockEvaluationMode` statement spine (`ValueProducing` vs `StatementSequenced`) | ✓ type exists; TS's `StatementSequenced` arm produces the correct fail-closed refusals |
| **F4** | `ProcessProgram` transport + `run_emit_host`; executable identity **bound per extdeps transport row** (`HostTransportDescriptor` / `ProcessProgram`, `host_transport.dag`), compiler dispatch **generic** | ✓ machinery; **DEFECT: dispatch is a central switch** — `host_tool_program_name` (`emit_host.dag:164-172`) is `if cargo … else if cc … else reject`, so every new tool edits one function (contradicts F0). Fix = row-bound exe identity + generic dispatch, **not** another switch branch |
| **F5** | self-host frontier runner (`run_test_claim_module_emit_vs_eval`: emit → build → run → compare-to-eval) | ✓ machinery **for Rust only**, run **OFFLINE** (excluded from discovery). Rust is **not** self-hosted: `compiler_frontier_self_emitted_baseline = 0` of `compiler_frontier_module_count_expected = 27` (`src/v2/compiler/self_host/frontier.dag`) |
| **F6** | `solve` — **structural** = existing `solve_constraints` / `ConstraintGraph` authority (extend, no fork); **numerical** = *absent* typed gap (finite measure + `TerminationProof`, typed residual-acceptance contract, extdeps solver handler) | **off critical path** — not needed for emit-to-simulator; see solve doc |
| **F7** | declaration emission generic over target (`emit_semantic_decl.dag`) | **FORKED — `emit_semantic_decl.dag` is rust-hardwired**; de-fork to row-driven per-target decl spellings is a front-loaded barrier, sibling to the F4 de-fork |

### The two highest-leverage shared dependencies (do these once, everyone inherits)

1. **F4 exe-identity de-fork** — today executable identity is chosen by a central `host_tool_program_name` switch (`emit_host.dag:164-172`: `if cargo … else if cc … else reject`), which gates **bar-c for every language** and forces a branch edit per tool (contradicts F0's "new target = rows, no pipeline edit"). Bind exe identity in each extdeps transport row (`HostTransportDescriptor` / `ProcessProgram`, `host_transport.dag`) and make compiler dispatch **generic**; then each target's *already-declared* runtime row runs through the typed self-host path with no pipeline edit. Highest-leverage de-fork on the board. **F7 (decl-emission de-fork of the rust-hardwired `emit_semantic_decl.dag`) is its sibling barrier.**
2. **F2 VEP completion** — wire the unwired forms **once** (rust's `^rust_token_unwired_{else,loop,match,fat_arrow}`; TS's match / loop / bind-in) in the `StatementSequenced` arm; the family inherits body breadth. Python additionally needs its *first* VEP edge.

---

## Per-target frontier table (grounded from the typed census, 2026-07-15)

| target | family | current bar (verified) | honest end-state | key deps | stress axis |
| --- | --- | --- | --- | --- | --- |
| **rust** | compiled | compiled; bar-c; F2 **partial** (`^rust_token_unwired_{else,loop,match,fat_arrow}`, `rust.dag:1148-1176`); F5 emit-vs-eval fixture **OFFLINE** | **`SeedRetained` — 0/27 self-emitted** (`compiler_frontier_self_emitted_baseline = 0`, `frontier.dag`); reference model, **NOT self-hosted** | F2 completion; F5 self-host generalization | (reference model) |
| **cpp — Phase-0-C** | compiled | bar-c via `cc`, proven by an **OFFLINE** witness in `test/claim/execution/` (excluded from discovery) | bar-c green (offline); self-host N/A at this phase | F4(`cc`) row-bound, F5-generalize | (confirms) |
| **cpp — full C target** | compiled | **below bar-a** for full C | blocked — needs **monomorphization** (grep-**zero** in tree), **closure conversion**, a **discriminant-tag row kind**, **multi-file (header/impl) projection**, and **ABI/linkage** — none present | monomorphization, closure conversion, tag-row kind, multi-file projection, ABI/linkage | (confirms — hard) |
| **go** | compiled | **skeleton** — needs TargetModel buildout | **self-host** (first non-rust self-host — the F5 generalization proof), once built | TargetModel buildout, F4(`go`) row-bound, surface spellings, F5 | **self-host axis** |
| **java / kotlin / swift** | compiled | **skeleton** — need TargetModel buildout | bar-c → self-host | TargetModel buildout, F4(tool) row-bound, spellings, inherit F2/F3/F5 | (confirms — parallel) |
| **typescript** | interpreted | F2 **partial** (match/loop/bind-in unwired); **bar-c RED** — runtime row present but `node`/`npx` unregistered | self-host | F4(`node`/`npx`) row-bound, F2 completion (match/loop/bind-in), operator catalog | self-host axis |
| **python** | interpreted | **no VEP edge — add-only** | bar-c → self-host | F2 *first* edge, operator catalog, F4(`python3`) row-bound | self-host axis |
| **ecmascript** | interpreted | **orphan (0 consumers)** | **decide first**: adopt as TS's JS base, or delete | (adoption decision) | — |
| **lean** | expression/ML | **below bar-a** — type model + anchor test only, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | self-host on an *expression* spine (not statement), once a TargetModel is built | committed TargetModel, F3 generalization (expression-mode), F2, F5 | self-host axis (spine falsifier) |
| **verilog** | HDL | **below bar-a** — type model + anchor, **no committed TargetModel**; **11** `body_lexeme:String` fields to decompose | **bar-c green + frontier row `self-host: N/A (hardware)`** | committed TargetModel, decompose `body_lexeme` → structured, F4(`verilator`/`iverilog`) row-bound | **emit-generality (concurrent)** |
| **spice** | analog format | **bar-b only** — the witness compares emitted text to a **golden** (`spice_rc_ngspice_oracle_test.dag:18-25`); it **never runs ngspice** | bar-c green + frontier row `self-host: N/A (analog)`. **No Modelica carrier in-tree** — dual-emit is future, not present | F4(`ngspice`) row-bound (bar-b → bar-c); Modelica carrier does not yet exist | **emit-generality (continuous)** |
| **llvm_ir** | IR/backend | **below bar-a** — type model + anchor, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | bar-c green + frontier row `self-host: N/A (no runtime)` — the Rust/cargo-free lowering path, once a TargetModel is built | committed TargetModel, F4(`llc`/`clang`) row-bound | **emit-generality (SSA)** |
| **wasm** | IR/backend | has a **TargetModel** (bundle/lex/binding_spellings) BUT **bar-c unreachable** — `runtime_row: target_emit_host_runtime_row_unconfigured` (HostRuntimeRowAbsent, `wasm.dag:624-632`) | bar-c green + frontier row `self-host: N/A` — **blocked on configuring the runtime row** | configure runtime row, F4(`wasmtime`) row-bound | emit-generality (stack machine) |
| **machine_code / ptx** | ISA/GPU | **below bar-a** — type model + anchor, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | bar-c green + frontier row `self-host: N/A`, once a TargetModel is built | committed TargetModel, F4(assembler/`ptxas`) row-bound | emit-generality (ISA) |

---

## Sequencing — Rust-as-model, then stress (dependency-ordered)

- **Phase A — now-ish (pre-un-shelve):** this map + the `solve` rationale. No emit code. ← *we are here*
- **Phase B — foundation de-fork (the barrier; everything downstream inherits):** F4 exe-identity de-fork (row-bound identity + generic dispatch) + F7 decl-emission de-fork (`emit_semantic_decl.dag`) + F2 VEP completion (rust + TS). Once B lands, C/D/E run largely in parallel.
- **Phase C — self-host axis proof:** generalize F5 past Rust to **Go** (first non-Rust self-host) — and land Rust's own self-host (0/27 today). This is the real "does the seed-shrink story generalize" milestone. If it holds, java/kotlin/swift are near-mechanical parallel fan-out.
- **Phase D — emit-generality axis (parallel):** Verilog · SPICE · LLVM-IR to bar-c + frontier rows. Verilog and LLVM-IR first need a **committed TargetModel** (both below bar-a today); SPICE needs its witness to actually run ngspice (bar-b → bar-c). Each decomposes its `body_lexeme:String` scars and binds its simulator/toolchain transport in a row. This is where the design gets *stressed*; a construct that won't lower produces a typed, located refusal that *names the design gap* — the refusal is the product.
- **Phase E — interpreted self-host:** TS → Python (Python inherits TS's VEP completion).
- **Phase F — expression spine:** Lean (falsifies "is F3 secretly imperative-shaped?") — first needs its committed TargetModel.

Each language is **priced by the design risk it displaces**, not by completeness (§6): Verilog=concurrent, SPICE=continuous, LLVM=SSA/self-host-boundary, Lean=expression-spine. A confirm-only language (java/kotlin/swift) earns its slot only as cheap parallel fan-out after its family's proof lands.

---

## Staffing note (for continuous dispatch)

Off this map, the natural fan-out is **one child session per language axis**, gated on Phase B. The dependency structure that makes this safe: **B is the only hard barrier** (F4 + F7 de-forks + F2 completion); after it, the self-host axis (C, then the compiled fan-out) and the emit-generality axis (D) have no cross-dependency, so they staff independently. Lean (F) and the interpreted family (E) each ride one Phase-B deliverable (F3-expression-mode and F2-completion respectively) plus their own missing TargetModel where noted. ecmascript's adopt-or-delete decision is a prerequisite gate on the interpreted family, not parallel work.

## Dissolution trigger (DESIGN §6)

**Un-shelve is an operator decision (ROADMAP §4 → §1/§3 by PR, priced in displaced cost).** When it fires, this doc + the solve doc are the lane's charter; the typed `LanguageTargetFrontierRow` carrier + completeness/RED witnesses are authored (they do not exist today), the frontier table rows migrate onto per-target carrier rows, the OFFLINE C/Rust F5 smokes are enrolled, and this file dissolves.

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
| **F2** | VEP (`TargetValueExpressionProjection`) — the general body producer | **partial** (rust: `^rust_token_unwired_{else,loop,match,fat_arrow}`, `v2.extdeps.languages.rust` `rust_value_expression_projection`) · partial (TS: match/loop/bind-in unwired) · **absent** (python, ecmascript) |
| **F3** | `BlockEvaluationMode` statement spine (`ValueProducing` vs `StatementSequenced`) | ✓ type exists; TS's `StatementSequenced` arm produces the correct fail-closed refusals |
| **F4** | `ProcessProgram` transport + `run_emit_host`; executable identity **bound per extdeps transport row** (`v2.std.host_transport` `HostTransportDescriptor` / `ProcessProgram`), compiler dispatch **generic** (`v2.compiler.emit_host` `process_program_name`) | ✓ done — `HostToolProgram.executable` is carried by the row and generic dispatch returns that value; target-specific work is only to configure a runtime row where one is absent |
| **F5** | self-host frontier runner (`v2.compiler.emit_host` `run_test_claim_module_emit_vs_eval`: emit → build → run → compare-to-eval) | ✓ machinery **for Rust only**, run **OFFLINE** (excluded from discovery). Rust is **not** self-hosted: `v2.compiler.self_host.emitter_producer_provenance` `emitter_produced_baseline` (the roster's `compiler_frontier_emitter_produced_count` is deleted; this is the surviving count of the same fact) is pinned to `v2.compiler.self_host.emitter_producer_provenance` `emitter_produced_baseline = 0`; the roster is DELETED, so no roster size derives — `v2.compiler.self_host.emitter_producer_provenance` carries the producer facts that outlived it. |
| **F6** | `solve` — **structural** = existing `solve_constraints` / `ConstraintGraph` authority (extend, no fork); **numerical** = *absent* typed gap (finite measure + `TerminationProof`, typed residual-acceptance contract, extdeps solver handler) | **off critical path** — not needed for emit-to-simulator; see solve doc |
| **F7** | declaration emission generic over target (`v2.compiler.emit_semantic_decl` `emit_semantic_type_decl`) | ✓ done — the declaration emitter accepts a `TargetModel` and reads its emission bundle through `v2.std.compilers.semantic_decl_emission` `semantic_decl_emission_from_target` |

### The highest-leverage shared dependencies (do these once, everyone inherits)

1. **F2 VEP completion** — wire the unwired forms **once** (rust's `^rust_token_unwired_{else,loop,match,fat_arrow}`; TS's match / loop / bind-in) in the `StatementSequenced` arm; the family inherits body breadth. Python additionally needs its *first* VEP edge.

---

## Per-target frontier table (grounded from the typed census, 2026-07-15)

| target | family | current bar (verified) | honest end-state | key deps | stress axis |
| --- | --- | --- | --- | --- | --- |
| **rust** | compiled | compiled; bar-c; F2 **partial** (`^rust_token_unwired_{else,loop,match,fat_arrow}`, `v2.extdeps.languages.rust` `rust_value_expression_projection`); F5 emit-vs-eval fixture **OFFLINE** | **`SeedRetained` roster; zero producer-qualified emissions** — `v2.compiler.self_host.emitter_producer_provenance` `emitter_produced_baseline` (the roster's `compiler_frontier_emitter_produced_count` is deleted; this is the surviving count of the same fact) equals `v2.compiler.self_host.emitter_producer_provenance` `emitter_produced_baseline = 0`, and `seed_emitter_behavioral_green_count` equals `seed_emitter_behavioral_green_baseline = 0`; reference model, **NOT self-hosted** | F2 completion; F5 self-host generalization | (reference model) |
| **cpp — Phase-0-C** | compiled | bar-c via `cc`, proven by an **OFFLINE** witness in `test/claim/execution/` (excluded from discovery) | bar-c green (offline); self-host N/A at this phase | F5-generalize | (confirms) |
| **cpp — full C target** | compiled | **below bar-a** for full C | blocked — needs **monomorphization** (grep-**zero** in tree), **closure conversion**, a **discriminant-tag row kind**, **multi-file (header/impl) projection**, and **ABI/linkage** — none present | monomorphization, closure conversion, tag-row kind, multi-file projection, ABI/linkage | (confirms — hard) |
| **go** | compiled | **skeleton** — needs TargetModel buildout | **self-host** (first non-rust self-host — the F5 generalization proof), once built | TargetModel buildout, surface spellings, F5 | **self-host axis** |
| **java / kotlin / swift** | compiled | **skeleton** — need TargetModel buildout | bar-c → self-host | TargetModel buildout, runtime-row configuration, spellings, inherit F2/F3/F5 | (confirms — parallel) |
| **typescript** | interpreted | F2 **partial** (match/loop/bind-in unwired); runtime row is configured (`v2.extdeps.languages.typescript` `ts_runtime_row`), but this pre-un-shelve map does not establish current bar-c execution | self-host | F2 completion (match/loop/bind-in), operator catalog | self-host axis |
| **python** | interpreted | **no VEP edge — add-only** | bar-c → self-host | F2 *first* edge, operator catalog | self-host axis |
| **ecmascript** | interpreted | **orphan (0 consumers)** | **decide first**: adopt as TS's JS base, or delete | (adoption decision) | — |
| **lean** | expression/ML | **below bar-a** — type model + anchor test only, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | self-host on an *expression* spine (not statement), once a TargetModel is built | committed TargetModel, F3 generalization (expression-mode), F2, F5 | self-host axis (spine falsifier) |
| **verilog** | HDL | **committed TargetModel landed** (`v2.extdeps.languages.verilog` `verilog_target_model_for`) and a first module emitted through the generic fold from language-neutral intent; **Icarus Verilog compiled and simulated the emitted bytes** — the four-row interlock truth table is a recorded run (`product.compute_board.simulation_receipt` `interlock_verilog_run`), joined to today's emission by an enrolled witness. Residue: the emitted profile is combinational module/port/assign only, and nothing re-executes the simulator on a cadence | **bar-c green + frontier row `self-host: N/A (hardware)`** | widen the emitted profile past combinational assign; a cadence that re-runs the recorded simulations rather than joining to a one-time observation | **emit-generality (concurrent)** |
| **spice** | analog format | **ngspice has now run the emitted deck** — the operating point of the emitted conditioning network is a recorded run (`product.compute_board.simulation_receipt` `conditioning_network_run`) whose 1.2 mA source current discriminates the `k` scale suffix a bare magnitude would have lost; the golden comparison remains beside it. The product-local SPICE printer is deleted and `v2.extdeps.formats.spice` owns every byte | bar-c green + frontier row `self-host: N/A (analog)`. **No Modelica carrier in-tree** — dual-emit is future, not present | a cadence that re-runs the recorded simulation rather than joining to a one-time observation; Modelica carrier does not yet exist | **emit-generality (continuous)** |
| **llvm_ir** | IR/backend | **below bar-a** — type model + anchor, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | bar-c green + frontier row `self-host: N/A (no runtime)` — the Rust/cargo-free lowering path, once a TargetModel is built | committed TargetModel, configure the `llc`/`clang` runtime row | **emit-generality (SSA)** |
| **wasm** | IR/backend | has a **TargetModel** (bundle/lex/binding_spellings) BUT **bar-c unreachable** — `runtime_row: target_emit_host_runtime_row_unconfigured` (`v2.extdeps.languages.wasm` `wasm_target_model`) | bar-c green + frontier row `self-host: N/A` — **blocked on configuring the runtime row** | configure the `wasmtime` runtime row | emit-generality (stack machine) |
| **machine_code / ptx** | ISA/GPU | **below bar-a** — type model + anchor, **no committed TargetModel** (0 `target_model` / `translation_rules_node` refs) | bar-c green + frontier row `self-host: N/A`, once a TargetModel is built | committed TargetModel, configure the assembler/`ptxas` runtime row | emit-generality (ISA) |

---

## Sequencing — Rust-as-model, then stress (dependency-ordered)

- **Phase A — now-ish (pre-un-shelve):** this map + the `solve` rationale. No emit code. ← *we are here*
- **Phase B — foundation completion (the barrier; everything downstream inherits):** F2 VEP completion (rust + TS). Once B lands, C/D/E run largely in parallel.
- **Phase C — self-host axis proof:** generalize F5 past Rust to **Go** (first non-Rust self-host) — and land Rust's own self-host (0/27 today). This is the real "does the seed-shrink story generalize" milestone. If it holds, java/kotlin/swift are near-mechanical parallel fan-out.
- **Phase D — emit-generality axis (parallel):** Verilog · SPICE · LLVM-IR to bar-c + frontier rows. LLVM-IR still needs a **committed TargetModel**; Verilog's has landed, and SPICE and Verilog have both now been executed by a real tool once, recorded as observations joined to the live emission — what remains for those two is re-execution on a cadence. Each decomposes its `body_lexeme:String` scars and binds its simulator/toolchain transport in a row. This is where the design gets *stressed*; a construct that won't lower produces a typed, located refusal that *names the design gap* — the refusal is the product.
- **Phase E — interpreted self-host:** TS → Python (Python inherits TS's VEP completion).
- **Phase F — expression spine:** Lean (falsifies "is F3 secretly imperative-shaped?") — first needs its committed TargetModel.

Each language is **priced by the design risk it displaces**, not by completeness (§6): Verilog=concurrent, SPICE=continuous, LLVM=SSA/self-host-boundary, Lean=expression-spine. A confirm-only language (java/kotlin/swift) earns its slot only as cheap parallel fan-out after its family's proof lands.

---

## Staffing note (for continuous dispatch)

Off this map, the natural fan-out is **one child session per language axis**, gated on Phase B. The dependency structure that makes this safe: **B is the only hard barrier** (F2 completion); after it, the self-host axis (C, then the compiled fan-out) and the emit-generality axis (D) have no cross-dependency, so they staff independently. Lean (F) and the interpreted family (E) each ride one Phase-B deliverable (F3-expression-mode and F2-completion respectively) plus their own missing TargetModel where noted. ecmascript's adopt-or-delete decision is a prerequisite gate on the interpreted family, not parallel work.

## Dissolution trigger (DESIGN §6)

**Un-shelve is an operator decision (ROADMAP §4 → §1/§3 by PR, priced in displaced cost).** When it fires, this doc + the solve doc are the lane's charter; the typed `LanguageTargetFrontierRow` carrier + completeness/RED witnesses are authored (they do not exist today), the frontier table rows migrate onto per-target carrier rows, the OFFLINE C/Rust F5 smokes are enrolled, and this file dissolves.

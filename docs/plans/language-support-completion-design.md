# Language and medium support completion — profiles, registry, and the exact frontier

**Status: Wave 1 (LANG-0 identity/registry) LANDED via gunbc.language_target_registry (#7803, 2026-08-06). Profiles, capabilities, evidence references, and full support closure remain open — this note is the program authority for those lanes, not a design-only artifact.** Roadmap registration and dispatch for the remaining waves are operator sign-off steps. The operator's framing that seeded this program: language support in this repo repeatedly reaches "80% and good enough" and stops — hand-authored `ts_*` target AST at the product layer being the live specimen — and that is tech debt, not support.

## 0. The grounding receipt (executed 2026-08-06 on current main, this is not a feeling)

- `realization_vocab_live_corpus_receipt_holds` (`src/v2/test/claim/long/realization_vocabulary_containment_witness_test.dag`) — the live-corpus containment receipt for the target-AST wall — **executed PASS** against current main after TS-0 froze the live leak edges. Enrolled on falsifier substrate long lane batch 6 (`falsifier_substrate_long_lane_rows`); per-PR mechanism coverage via `no_new_debt_gate_test.dag` and the existing synthetic/scanner/roster-soundness receipts.
- The live leak population at edge identity grain (importer_path, vocab_module), census re-run 2026-08-06 — **five exact edges**, classified:
  - **Product-layer target-AST debt:** `(dag/gunbc/roadmap_component.dag, extdeps.languages.typescript.program)` · `(dag/gunbc/design/theme_transition.dag, extdeps.languages.typescript.program)`
  - **Misplaced realization edge:** `(dag/gunbc/node_http_server_emit.dag, extdeps.languages.typescript.program)` (interim dashboard static-server emit) · `(src/v2/workflow/effect_plan_bash_materialize.dag, v2.extdeps.languages.bash_build)` (shell-to-intent wave-A interim) · `(dag/tools/build_step.dag, v2.extdeps.languages.bash_build)` (pre phase-2 tooling)
  - **Sanctioned realization boundary:** none in the current population — every live edge is debt awaiting migration.
- Prior receipt (2026-08-02) counted three *paths* and is **superseded** by this edge-grain census; main moved and path-grain obscured which import is the debt.
- `gunbc.plans.typescript_gap_census` (approved 2026-07-01) already issued the warning — bar (c) was UNMET even for `fn add` at authoring: "**Do not stack breadth on a red tsc foundation.**" The ROADMAP lane it cites (`5-ts-first-class`) no longer exists; `typescript` has zero references in today's roadmap authority. The census's own history is the 80%-and-stop class, enacted.

## 1. The contract

Every language or medium this repo claims to support is **complete for an explicit support profile**, or visibly partial, parked, or retired. Completion is **derived, never authored**: a target is complete for a profile exactly when the profile's required capabilities equal the target's executed-proven capabilities AND unclassified residue is zero. Target-language vocabulary is confined to realization boundaries with the live receipt executing in CI. An executable target's support claim is an executed toolchain-and-behavior receipt with a discriminating planted defect — never emitted text alone (bar (b) is not support; bar (c) is).

"Fully support every language" is deliberately NOT the contract — it is unbounded and recreates the same ambiguity at larger scale.

## 2. Support profiles (not one universal "full")

Rust, HTML, SPICE, and Verilog cannot honestly share one completion checklist. A target declares one or more profiles; profile selection derives which capability axes are required.

| Profile | Meaning (all bars by execution) |
|---|---|
| Compiler bootstrap | emits the compiler's own required source, builds, executes, repeats the generation |
| Product browser application | realizes the browser-facing behavior, state, effects, rendering, transport the product uses |
| General executable source | ingests/emits the declared subset, toolchain-accepts, executes, preserves behavior |
| System/ABI target | emits and links bounded system components with declared ABI/memory/multi-file behavior |
| Accelerator kernel | lowers a declared numerical subset with explicit layout/numerics, verified against an oracle |
| Hardware description/simulation | structurally represents and emits the declared HDL subset, validated through a simulator |
| Circuit simulation | emits a netlist, executes a simulator, recovers typed observations from its output |
| Structured medium | ingests/emits the structured representation with declared fidelity and security boundaries |
| Proof/checker target | emits an artifact the external checker accepts, with a discriminating failed proof |
| Human-language medium | bounded controlled language, refusing outside it |

Capability axes (shared vocabulary, not per-target booleans): ingest · emit · round-trip · types · declarations · modules/imports · functions/closures · control flow · collections · effects · errors · async/concurrency · generics · runtime execution · multi-file/linkage · external-tool verification · real consumer · self-generation.

## 3. The shared denominator (Wave 1 — before any language-specific lane)

- **LANG-0 registry.** One typed registry: target identity, upstream authority, declared profiles, priority, host runtime/checker, required capability set, evidence refs, explicitly parked capabilities. This **extends `gunbc.plans.language_target_self_host_frontier`** — the existing per-target carrier whose three-bar rows (bar (a) grammar-inverse enrolled / bar (b) source-string witness / bar (c) toolchain-accepted consumer) already state several findings this note relies on — and supersedes its prose tables with typed rows (§2/§3: extend the authority, never mint a second registry beside it). `typescript_gap_census` folds in as the TS rows.
- **LANG-1 residue census.** Exact identities, never counts: target-AST imports outside realization edges (the containment lens's leak model, kept), target source embedded in strings, structure-bearing `*_lexeme` leaves (verilog carries 22, 11 of them `body_lexeme`), per-language compiler-stage branches, unsupported target-model arms, host runtimes without execution receipts, fixtures with no production consumer.
- **LANG-2 no-new-debt gate.** **LANDED (TS-0, 2026-08-06):** enroll the live-corpus containment receipt on falsifier substrate long lane; freeze the five current exact edges on `realization_vocab_grandfathered_edge_roster` with typed dissolution triggers and debt-class labels; a NEW exact edge refuses. Per-PR mechanism coverage via `no_new_debt_gate_test.dag`. Precedes any migration breadth.
- **LANG-3 common execution receipt.** One receipt shape for every executable target: source produced → toolchain invoked → toolchain accepted → executed where applicable → behavior compared against a language-neutral oracle → planted defect discriminated. Generalizes `emit_host_gate` (today: literal rows for rust, python, go, c only); bar (c) becomes the only admissible support claim.
- **LANG-4 derived status projection.** The support view derives from registry × census × receipts — "TypeScript · Product browser · N/M required capabilities · K residues" — with no hand-authored percentage or status prose anywhere.

## 4. Priorities (operator-routed 2026-08-02)

- **P0 — product path.** TypeScript product-browser profile (the site subsumption and gunbc-served dashboard lanes are the consumers); HTML/CSS web medium (consolidation onto the existing Fragment/markup authority, which is materially ahead of TS — typed model, ingestion, serialization, XSS/void-element witnesses); **Rust by reference** — compiler bootstrap stays under the self-host frontier/cutover lanes and is deliberately not re-scoped here, so generic language completion can never inflate the self-host counter.
- **P1 — experiments and likely extensions.** SPICE (cheapest full completion demonstration: the frontier row itself says bar-b only, "never runs ngspice"); Verilog (the behavior-not-just-declarations stress test); Python (broad existing surface, needs its exact profile gap derived); C + accelerator seam (bounded C system/kernel first, then CPU fused-kernel oracle → PTX validation → CUDA execution, per the existing accelerator plan's seam — never "full CUDA support" first).
- **P2.** Go (second compiled general target), LLVM IR (backend/validator profile), WASM (backend/runtime), Lean (proof/checker), broader C++ (only after the profile decision below).
- **P3.** Java, Kotlin, Swift, SQL, machine code, English, and the ECMAScript question; JSON/YAML/Markdown stay consumer-driven maintenance. P2/P3 targets enter the registry with explicit profile-or-parked dispositions — a row each, not a lane each.

## 5. TypeScript and web program (the P0 detail)

- **TS-0 freeze.** **LANDED (2026-08-06):** the five live leak edges are grandfathered at `(importer_path, vocab_module)` identity grain; new exact edges refuse; live-corpus receipt executes on falsifier substrate long lane. No migration in this slice.
- **TS-1 browser interface authority.** Model only the interfaces the product uses — document query, element text/removal, attributes/dataset, class-list, event subscription, media-query observation, timers/cancellation, network request, storage — each homed with its upstream subject (WHATWG/browser extdeps decomposition), never a TS-specific business carrier.
- **TS-2 event/state semantics — the real design slice.** Observation, event, handler registration, state transition, effect request/result, timer lifetime, cancellation, async continuation modeled language-independently BEFORE wholesale migration; otherwise the migration just renames AST constructors. (This is why "all of it is derivable" is too strong today: syntax is derivable, behavioral semantics are not yet modeled.)
- **TS-3 target rows.** Fill the TypeScript realization for constructs the product demands (member call, assignment, projections, closures, conditions, iteration, handlers, template strings, async sequencing, modules, typed boundaries, refusal projection) — extending existing target-model families, not TS-only compiler branches.
- **TS-4/TS-5 specimens.** First `gunbcObserveImpactSync` (query/count/set-text/remove — no events, no fetch): authored as ordinary language-independent behavior, emitted through the production TS target, tsc accepts, a DOM fixture reproduces the observable result, a planted defect discriminates, and the hand-authored AST for it deletes in the same change. Then `theme_transition` — the natural test of TS-2 (media query, state, listeners, timer cancellation, class effects).
- **TS-6 migration by behavior family** (DOM sync, click handlers, status repaint, filtering, network, retry/timeout, streaming, preferences, focus) — never one giant PR; each family may expand the browser model only at a genuinely new concept.
- **TS-7 whole-product receipt.** Structured HTML/CSS emitted + client TS emitted from intent + tsc accepts + browser executes required interactions + zero product-layer target AST outside declared realization edges.
- **TS-8 (decision, deliberately deferred).** TsProgram's terminal role — confined realization AST vs dissolved into the shared target-model fold — is decided only after real consumers exist. TsProgram is NOT the raw-bash-string class: it is structured target AST authored at the wrong layer; the migration moves the authoring boundary, it does not necessarily delete the AST.
- **WEB.** One typed product view authority projecting server HTML, client update wire, optional JSX, and accessibility representation from the same facts; structured markup all the way down (raw text only at typed script/style boundaries); React as a projection, never a second component tree; browser-executed receipts (rendered structure, hydration, XSS refusal, theme, focus).

## 6. Experimental programs (P1 detail, compressed)

- **SPICE:** pinned ngspice runtime row → execute the emitted RC deck → parse typed measurements (operating point, transient samples, refusal) → differential witness against an analytical expectation (a changed component changes the curve) → device/analysis families as rows, unsupported constructs refusing.
- **Verilog:** exact lexeme-residue census (field/declaration identity, not grep counts) → typed expression carriers → typed procedural behavior (assignments, blocks, case, event control, always/initial) → bounded round-trip → iverilog/Verilator execution receipt → a generated hardware consumer (small FSM/pipeline), not a hand-authored fixture.
- **Python:** derive the profile gap from the capability registry (no prose census) → close required families with emit→compile→execute→compare→perturb receipts → one real product specimen → the cross-language path (Python-ingested program emitting to TS or Rust through the shared core, no language-pair logic).
- **C/accelerator:** bounded C (scalars/arrays, functions, structs, control flow, multi-file, compile-link-run receipt, ABI declarations) → accelerator seam in order: numerical subgraph recognizer, CPU fused-kernel oracle, PTX validation, CUDA execution, optional WGSL last.

## 7. Decisions this program surfaces (operator-owned, all open)

1. **ECMAScript authority** — is ECMAScript the runtime-semantic base the TypeScript target projects onto, or a deliberately separate model? The two must not both own JavaScript semantics indefinitely.
2. **C++ profile** — bounded system/kernel C++ vs a general-purpose C++ target (templates, exceptions, ownership, linkage — a much larger, separately-scoped program). Whichever is chosen is named honestly; the bounded profile is never labeled "full C++ support".
3. **TsProgram terminal role** — decided only after TS-4..TS-6 produce real consumers.

## 8. What does not land from this note

LANG-0 identity/registry rows and TS-0 no-new-debt gate land from their own PRs against this program authority. Remaining waves (LANG-1 residue census, LANG-3 execution receipt, LANG-4 status projection, TS-1..TS-8) still require operator sign-off before implementation. **Dissolution:** this note dissolves into the typed registry rows and a registered `gunbc.plan.Plan` when Wave 1 fully closes, per house pattern.

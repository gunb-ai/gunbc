# gunbc Roadmap

Where the project stands and where it is headed. For the intellectual goal, read [THESIS.md](THESIS.md). For rules that protect that goal, read [INVARIANTS.md](INVARIANTS.md). For how to extend the language safely, read [MODELING.md](MODELING.md).

> **v4 is the active development phase.** New substrate modeling and compiler pipeline work live in [`src/v4/`](src/v4/). v3 is frozen. v2 remains the production self-hosted compiler today.

## Active wave — the stage-fold program

> *In-repo gunbc-scale view, tightly coupled to the portfolio dep graph above it (the fractal's top scale). Portfolio direction authority: [ctrl/ROADMAP.md](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md) (operator-consolidated 2026-06-14, ctrl #1609); this section threads **up** to it and is the stale copy if they drift (portfolio wins). The dated `ctrl/gunbc-planning/dependency-graph-*`, `ctrl/ctrl-planning/dependency-graph-*`, and `ctrl/planning/dependency-graph.md` trees are archive. The graph is **fractal**: portfolio → project (this) → lane, same node shape (goal / edges / owner / falsifier) at each scale; cross-cutting requirements are defined once at the scale they span and referenced — not duplicated — where each scale inhabits them.*

**The one principle (§0).** Repoint the consumer onto the fold/model, then the old code deletes itself in the **same PR**. If the file is *larger* afterward, it was a graft, not a migration — the same-PR rule should have blocked it.

**§1 — stage-fold (active, frontloaded ahead of all feature work).** Every compiler stage collapses to one fold over its model: `stage(x) = fold_carrier(x, stage_algebra(model))` — traversal owned by a reusable fold, every former hand-arm becomes one **data row**. Existence proof already on main: `05_emit` is 43 lines (`serialize ∘ translate`, frozen); this program makes the rest match it.

| § | Stage | Carrier | State |
|---|-------|---------|-------|
| 1.1 | `06_translate` (keystone, load-bearing) | `fold_node` | ✅ merged (#4699) |
| 1.2 | `03_normalize` | `fold_node` | ✅ merged (#4691, #4694) |
| 1.3 | `03_resolve` / `03_name_resolve` | `fold_node` + env | ✅ core merged (#4700) |
| 1.4 | `03_body_producer` | `fold_node` | ✅ PIN1 merged (#4695) |
| 1.5 | `06_value_expression` | merge into `translate_algebra` | ✅ merged (#4809 — dissolved into target_model value-tier rows) |
| 1.6 | `04_infer` (gather; solver stays a named kernel) | `fold_node` | ✅ merged (#4692) |
| 1.7 | `02_parse` | `fold_grammar_expr` | ✅ merged (#4693) |
| 1.8 | `01_tokenize` | **`fold_source`** (one new combinator) | not started — gated on the combinator |

Keystone §1.1 landed (#4699): **4,912 → 3,973 lines** on main, `_go` accumulators **35 → 0** (traversal owned by the fold). Serialize-side `_bounded` fuel remains (🟡 W2 dissolve-on: TargetModel acyclicity witness). The file shrank, so it was a migration not a graft. `fold_source` (§1.8) is the *only* genuinely-new machinery left in §1; everything else is repoint-then-delete onto folds that already exist.

**§2 — control-flow bodies** (Branch → Bind → Loop, via a COMPREP function-body producer). **Active** — the real next wave after §1, and the highest-leverage one — emit breadth (§3) and a runnable IO program (§4) both sit on it. #4699 keystone merged (2026-06-12); COMPREP source-bridge gate verified closed (#4646/#4655).

**§3 — emit breadth** (N data rows, not N×M hand-arms); includes the bidirectional emit/ingest round-trip (the fold's inverse proof) and dissolution of host-transport bridges. Depends on §1 + §2.

**§4 — first runnable v4 program with I/O** (effect handlers, run-loop/scheduler). Depends on §2.

> **Priority fork (operator's call, after §2 lands):** §3 + self-host (compiler/breadth lane) vs §4 (a demo-able running program). Both unblock once §2 lands; the pick sets the deepest staffing.

**§5 — self-hosting** (get-off-v3 caller-census ratchet → zero hand-maintained Rust). Depends on §3.

### L1 v4-compiler-spine — self-hosting lane

Drive the v4 compiler spine to **self-hosting**: §1 done (§1.8 `fold_source` excepted), §2 done, §3 WIP, §5 scaffold. Lane node (goal / edges / owner / falsifier), threading **up** to the LANGUAGE/v4 pillar under `zesty-swift-79`. Owner: L1 v4-compiler-spine lane (durable).

**Target-independence (non-negotiable):** the §4 canonical program is authored **once** in substrate; TypeScript is only a backend. A target forcing a program-shape change is a thesis violation — escalate to the pillar, do not bend the program.

**Self-hosting milestones** (the falsifier is the CI-enrolled execution witness, not prose):

| # | Milestone | Goal | Falsifier (witness) |
|---|-----------|------|---------------------|
| **M1** | §3 emits a §4 program to TS | One canonical §4 program fixture (Branch + Bind + Loop + at least one effect arm) emits to TypeScript by execution — program written once, TS backend only. | `gunbc run --claim-run` green on the canonical §4-program TS emit witness; operand/catalog perturbation witnesses discriminate (target-independence tripwire). |
| **M2** | §3 emits the compiler to TS | A **named, bounded** compiler subset (stages under `src/v4/compiler/`, ratcheted upward) emits TypeScript that compiles — not full L4 self-compile yet. | Executable emit receipt: emitted TS artifact passes `tsc`/host compile gate with structured PASS/FAIL headline; bounded slice named in the witness label. |
| **M3** | §5 self-host fixed-point green | Iteration *N* emitted Rust digest equals iteration *N+1* — real merkle digests, not scaffold placeholders. | `claim_t15_self_host_fixed_point` + host harness `t_15_self_host_fixed_point` green **by execution** (EqualsClaim verdict, not parse/grep). |

Edges: M2 depends on M1; M3 depends on M2 + B1 merkle digest carriers landing in `v4.bin.main`. **M1 completion unblocks L2-M2** (browser demo) — report to the LANGUAGE pillar the moment it lands.

**T-22 eval harness scope (pillar question, answered):** `claim_t15` execution is blocked on (a) **B1 merkle digest operands** in `main.dag` — **in-scope for L1 / §5**; L1 lands the digest carriers as part of the M3 path. (b) **T-22 eval / host runner substrate** that executes `TestClaim` verdicts — **out-of-scope for L1 to rebuild**; it is a **root portfolio dependency** (`bold-crane-680` Runtime/TestClaim lane). L1 **consumes** the runner (wires claims, enrolls corpus rows, activates `claim_t15` green once the runner + digests exist) and does not duplicate parse/eval-host infrastructure. Absorb: §3 substance from the closed §3 ICs; `bold-dove-299` parse slice (#4957) folds into this lane when signed — do not re-task or duplicate parse work.

### Cross-cutting requirements (portfolio scale — threaded here)

Some requirements span every layer of the stack; they are defined once at the portfolio scale and referenced — not duplicated — here where gunbc inhabits them. Full requirement, ARC, edges, and falsifier: [ctrl/ROADMAP.md — Realization pattern](https://github.com/gunb-ai/ctrl/blob/main/ROADMAP.md#cross-cutting-requirement--the-realization-pattern). Design root: [`docs/planning/recompute-pure-of-content-transparent-memoization-design-2026-06-14.md`](docs/planning/recompute-pure-of-content-transparent-memoization-design-2026-06-14.md).

- **The Realization pattern** — *critical, operator-elevated (2026-06-14).* Content-addressed reconciliation of a pure spec to its realized effect across an impurity boundary. The recurring root behind language-level caching, build caching, the §10 OS work, and (ahead) provisioning / DB migration / scheduling: every consumer of an already-computed result must cross the model→host execution boundary, and there is **no first-class reified execution receipt** asserting "this host computation is the faithful, content-keyed realization of that pure node" — so each re-derives both the need (redundancy) and the risk (un-enforced purity) with a local `HashMap`. The recurrence is a theorem (*Build Systems à la Carte*), and v4 still recurs (`ParseTable`, `TestClaimCacheKey`) **because the carrier is staged, not inhabited** (`compute_fabric` / `cache_interface` are mostly `🟡 forward = Node` stubs → M9-DFS finds nothing consumable → the author hand-rolls → P1 violation is structurally guaranteed). The carrier: `Realization<Spec, Effect>` = pure Node spec + content-hash identity + receipt + locality, parameterized by an algebraic effect handler — one kernel, N handlers (cost-of-change 1). gunbc wins where Nix/Bazel half-won because pure-by-construction gives impurity **one door** (the handler), the only place enforcement must live.
  - **ARC inhabitant #1 = resolve-cost** (gunbc, deepest place): PR1 #4867 (illustration — cache *exposed* host intern-table impurity), PR2 #4878 (first real inhabitant, grounding staged `cache_interface` dims).
  - **ARC inhabitant #2 = sccache / build cache** (de-risking rung): closed-world + reversible — proves the identity kernel carries across realize-steps; does **not** alone prove open-world layer-agnosticism.
  - **ARC inhabitant #3 = §10 OS work** (cross-layer stress test): open-world + irreversible provisioning / migration / syscall realization — the requirement is **met** only when this handler carries on the same kernel (not another compiler cache).
  - **Endpoint = a Realization Lens** that detects the hand-rolled-realization shape and *errors*, forcing the carrier — self-enforcing, the same *substrate-forbids-the-hand-roll* family as R-reflect (coproduct reflection) and INVARIANTS P1.

## What works today (v2)

The v2 compiler in [`src/v2/`](src/v2/) is self-hosted from `.dag` source: tokenize, parse, infer, emit, complexity, and ownership pipelines are authored as `.dag` programs with a small Rust bootstrap (`stage0`) that shrinks over time.

You can:

- Write `.dag` programs using `dsl/std/` and `dsl/extdeps/`
- Compile and validate causal structure (types, termination, effects, ownership)
- Emit to Rust, Python, and Go from the same declarations
- Run the compiler and test suite via Cargo (`cargo test -p v2-compiler-tests`)

The `gunbc` CLI (from the `v2-compiler` crate) is the primary entry point for compiling v4 trees during bootstrap work.

## What v4 is building toward

v4 combines substrate depth (typed Node + Behavior kernel, algebra-grounded std library, rich `extdeps/`) with a full compiler pipeline rewritten in that substrate:

| Area | Status (approximate) |
|------|----------------------|
| `src/v4/std/` — shared vocabulary (node, algebra, effects, grammar, …) | Substantial; core carriers landed |
| `src/v4/extdeps/` — language and transport models | Broad coverage; ongoing grounding work |
| Compiler stages (`01_tokenize` … `06_translate`, `00_compile`) | Parse and type-check `.dag`; emission in progress |
| Lenses (complexity, cost, coverage, testgen, …) | Many structural lenses; runner integration ongoing |
| Pure bootstrap / self-host | Trajectory to zero hand-maintained Rust; `self_host.dag` ratchet |
| Tests as `.dag` `TestClaim` data | Growing corpus under `src/v4/test/claim/` |

### Bounded bridge receipts

| Lane | Interim bridge | Dissolve-on |
|------|----------------|-------------|
| T-22 eval host transport dispatch | `run_emit_host_go` eval calls use the existing `emit_host_runner` host boundary while projecting the modeled `v4.std.host_run.EmitHostRunReceipt` / `Outcome` carriers. | Generated `.dag` eval dispatches `v4.compiler.emit_host.run_emit_host_go` host transports directly; delete the evaluator shim with the surrounding `src/v3/compiler/src/emit_host_bridge.rs` host-transport bridge. |

### Public Operational Lanes

| Row | Public tracking intent |
|-----|------------------------|
| T-PB-B / `pb_rust_tests_outside_residual_zero` | Move remaining hand-authored Rust boundary and smoke tests into `.dag` `TestClaim` / generated-runner coverage, keeping same-path SG-0 expansions at +0 new paths until the matching claim runner executes those facts directly. |

**Honest v4 status:** the v4 pipeline compiles and type-checks `.dag` over `src/v4` in CI. Lowering, full multi-target emission, and execute-verified test claims are still landing. v2 remains the reference for end-to-end emit until v4 closes the loop.

Design direction: **model local, derive global** — every target modeled once in shared vocabulary; translations are derived homomorphisms, not hand-written adapters ([docs/thesis/the-derived-homomorphism.md](docs/thesis/the-derived-homomorphism.md)).

### Coercion in both directions — ingestion and emission

Coercion is one mechanism, run both ways. **Ingestion is coercion and emission is coercion**: a structure-preserving search over declared inhabitants, performed by the compiler rather than any hand-written adapter ([`src/v4/std/coercion.dag`](src/v4/std/coercion.dag)). The whole language is **`.dag → IR → .dag`**, where a `.dag` can be anything modeled in the substrate — *including a language itself*. Ingesting a program coerces it from its language model into the canonical IR; emitting coerces the IR back out into a target's declared inhabitants. The same **semantic realization search** underlies ingestion and emission once both sides are modeled as declared inhabitants; tokenize/parse and print/render remain boundary projections and must not become separate adapter authorities. This is what makes omni-ingestion and omni-emission cheap — model N targets once and *derive* the N×M translations (**model local, derive global**), instead of authoring an adapter per pair. Here `.dag → IR → .dag` means canonical `.dag` *source* regeneration, not a JSON IR receipt; JSON remains a boundary/debug artifact unless explicitly promoted by Branch H's canonical source AST and serializer.

Coercion is a **total decision procedure** in both directions — every attempted realization either produces a structure-preserving `HomomorphismWitness` or fails closed with a located `CoercionMismatchKind`, never a guarantee that translation always succeeds:

- **Realizable → translate, with a witness.** When a type structurally grounds to an inhabitant — including a *faithful refinement* such as widening a fixed-width `i32` into an arbitrary-precision `int` — the compiler carries a `HomomorphismWitness` proving structure was preserved.
- **Not realizable → fail closed.** When no faithful realization exists, the compiler refuses and reports a located diagnostic; the closed `CoercionMismatchKind` taxonomy classifies every refusal — a *missing inhabitant* (`NoTargetCandidate`), a *lossy coercion* such as narrowing an unbounded `int` into `i32` (`WouldLoseInformation`), and — load-bearing — an **opaque atom with no per-target realization** declared in [`extdeps/`](src/v4/extdeps/). An opaque atom (a resource handle, a hash) is honest only when each target declares how to realize it; absent that, the homomorphism is undefined and translation fails closed rather than synthesizing silent glue — no partial or implicit realization, in either direction.

These claims are marked *proven* only by `TestClaimRun` verdicts, not prose — positive (`i32 → int` faithful widening emits a `HomomorphismWitness`), negative (unbounded `int → i32` `WouldLoseInformation`; a source atom with no target inhabitant `NoTargetCandidate`; an opaque atom lacking per-target realization), and the `emit → ingest` round-trip (normalized equality, explicitly *not* bit-identical unless claimed). Today emission-as-coercion is wired through the translate stage, exercised as such `TestClaim` data under [`src/v4/test/claim/`](src/v4/test/claim/) with fail-closed refusals landing first as the safety floor. The symmetric ingestion side is taking shape via `.dag → IR → .dag` round-trip claims ([`src/v4/test/claim/round_trip/`](src/v4/test/claim/round_trip/)): a wave-1 readiness / shape contract today, with the executable `emit → ingest` normalized-equality compare staged as a follow-up — the claim labels are explicit that they do not yet imply bit-identical fidelity. Full multi-language ingestion breadth and the refinement-aware rule that distinguishes faithful widening from lossy narrowing are likewise later-wave work.

## Milestone shape

Work is organized around closing the bootstrap loop, not a calendar:

1. **Substrate complete** — std + extdeps fact-bundles ground external primitives without hollow aliases.
2. **Compiler pipeline** — tokenize → parse → resolve → infer → emit → translate with fail-closed diagnostics.
3. **Lenses and tests** — structural `TestClaim` predicates evaluated by generated or substrate runners; lenses over the same Node tree users write.
4. **Self-host fixed point** — `compiler.dag` emits bit-identical stage0; hand-maintained file count → 0 per [docs/design-pure-bootstrap-zero.md](docs/design-pure-bootstrap-zero.md).
5. **Public release** — v2 + v4 story documented; binaries via GitHub Releases; public repo snapshot with these root docs.

### Nine lanes

| Lane | Gate / row | Tracked obligation |
|------|------------|--------------------|
| **T-PB-B** | `pb_rust_tests_outside_residual_zero` | Pure Bootstrap test floor: hand-maintained Rust tests under `src/v3/compiler/tests/` shrink toward zero as receipts migrate to `.dag` `TestClaim` declarations or generated harness coverage. Each temporary Rust test scaffold must be listed in the SG-0 hand-authored-test census with a dissolution trigger, and the same PR removes that census entry when the modeled or generated replacement lands. |

Earlier release-program lanes (complexity parity, testgen, multi-target emit, pure-bootstrap floors) informed v4 scope; that era's detailed operational tracking is not carried in this repo.

## Active Deferrals

| ID | Target | Dissolution trigger |
|----|--------|---------------------|
| `PD-3-DOGFOOD` | `module_skips_direct_call_arg_check` TRANSITION scaffold (A3 / PD-3): prefix skip for `v4.*` and `v2.compiler.*` while bounded direct-call brand-twin rejection is dogfooded on user modules only. | Delete the skip when `scripts/v4-lens-ci-gate.sh` and `.github/ci-floor/v4-rust-full-tree-emit-probe.sh` pass with the skip removed — v4/compiler substrate compiles through `direct_call_arg_mismatch_diags` with zero false-positive diags. |
| `PB-Runtime-External-Toolchain-TestClaims` | Hand-authored Rust boundary tests that spawn external target toolchains while v4 leaf-model verification still uses host runners. | Delete the Rust boundary test when its corresponding `src/v4/test/claim/**` row is exercised by substrate `run_target_verification` / `ExecuteCommand`-style `.dag` `TestClaim` execution with typed verdicts. |

## How to read the tree

The doc map and the single-authority rule live in one place: see [docs/thesis/doc-authority.md](docs/thesis/doc-authority.md).

## Contributing orientation

- Prefer extending `std/` or `extdeps/` before adding compiler-local types.
- Every scaffold needs a dissolution trigger; progress reduces duplicate authority.
- Run `cargo test --workspace`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --all --check` before pushing.

For deep design context: [docs/architecture.md](docs/architecture.md) and essays under [docs/thesis/](docs/thesis/).

Current-state audits live under [docs/audit/](docs/audit/). On information-hiding specifically — how close v4's foundational concepts are to the below-boundary opacity THESIS names (the "touch-once contract") — see [docs/audit/v4-encapsulation-touch-once-contract-2026-06-05.md](docs/audit/v4-encapsulation-touch-once-contract-2026-06-05.md): the headline is that opacity holds at substrate atoms and leaks at transparent aliases, and that v4 already has a `.dag` information-hiding primitive (`nominal_opaque`) pointed at zero foundational concepts.

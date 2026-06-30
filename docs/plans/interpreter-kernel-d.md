# HAND kernel D — v1_interpreter pure-eval authority + pinned host-physics

**Status:** design-first (2026-06-29). Parent `sunny-crab-671` seed burn-down, **HAND kernel D**. **This slice:** lands `gunbc.interpreter_kernel_model` + pure `.dag` floor witness only — no `v1_interpreter.rs` emit flip, no `HAND_MAINTAINED` registry edit, no `regen --verify` flip. Mirrors #5894 / `compile_source_model` sequencing.

## 0. Verdict — pure-eval authority vs pinned host-physics

`v1_interpreter.rs` (~6.3k LOC, `HAND_MAINTAINED_STAGE0_FILES`) is a **hybrid**: pure expression evaluation + host-effect dispatch (shell / REST / file / fixture replay / caches). Total dissolution is not one file delete — the seam splits on **decidability**: pure-eval grounds into `v2.compiler.eval` + `emit_host`; host-physics (transports / fixtures / cache backends) is the **terminal irreducible kernel** (seed-shrink-census §6).

| class | authority / carrier | this slice | follow-up |
| --- | --- | --- | --- |
| pure-eval | `05_eval.dag` · `emit_host.dag` · `v2_evaluator.dag` · `host_transport.dag` · `host_run.dag` | `gunbc.interpreter_kernel_model` + `interpreter_kernel_model_witnesses` | wire `05_emit_rust` pure-eval emit → shrink `v1_interpreter.rs` eval core |
| pinned host-physics | `recorded_fixture.rs` · `resolved_graph_cache.rs` · `rest_transport_facts.rs` · `wire_value_serialize.rs` · `cache_purity_oracle.rs` | roster + dissolution triggers in model | collapse onto Materialization kernel (`realize(subject)`); stays HAND until then |
| hybrid remainder | `v1_interpreter.rs` host-effect + CLI render dispatch | explicitly NOT pinned as a whole file | split: eval → GENERATED; host dispatch → pinned submodule or `.dag` transport handlers |

## 1. Pure-eval emit seam (`emit_host`)

**Symptom:** `v2.compiler.emit_host.run_host_process` is fail-closed (`emit_host_transport_not_wired`) — the modeled `HostTransportDescriptor` rows in `extdeps/languages/*` are not yet consumed by the v1 interpreter tap. **Authority:** `run_test_claim_emit_vs_eval` already compares `emit(tree)` against `eval(tree)` for `EqualsClaim` / `CompilesClaim`; the host run is the residue.

**Construction direction (this slice):** name the authority modules and phase gate (`ModelAndWitnessOnly` → `PureEvalEmitWired`). **Follow-on:** wire transport realization in the seed interpreter (or thin host handler) from `HostTransportDescriptor` rows — same class as #5075 `TargetModel.runtime_row`.

## 2. Pinned host-physics (transports / fixtures / cache)

**Verdict:** these five `HAND_MAINTAINED` stage0 files are honestly **physics-bound** — OS/shell/network/fixture-store/cache I/O. They are pinned in the model until the §4 Materialization kernel (`realization_measurement_loop` Phase 2/3) provides one `CacheLookupResult` fold consumed by both v1 handlers and v2 `05_eval`.

- `recorded_fixture.rs` — hermetic service replay store (M4/M5 consolidation target).
- `resolved_graph_cache.rs` — cross-run resolve memo (content-hash keyed).
- `rest_transport_facts.rs` + `wire_value_serialize.rs` — REST + wire JSON host seam.
- `cache_purity_oracle.rs` — warm==cold perturbation oracle (wiring-liveness dual reading).

## 3. Explicit non-goals (this slice)

- No `v1_interpreter.rs` refactor or LOC deletion.
- No `regen_stage0.rs` `HAND_MAINTAINED_STAGE0_FILES` edit.
- No `05_emit_rust.dag` emitter edit.
- No `emit_host.dag` `run_host_process` wiring (transport realization PR).

## 4. Discriminating witness (follow-on PRs)

- **Pure-eval emit:** `run_test_claim_emit_vs_eval` GREEN on a rust `EqualsClaim` row without `Deferred` transport diagnostic.
- **Pin receipt:** pinned roster matches `regen_stage0.rs` HAND list for the five files — drift-gated when invert-hand-maintained derives the registry.

## Dissolution trigger (DESIGN §6)

Pure-eval eval core is v2-emitted from `05_eval` ( `v1_interpreter.rs` eval functions GENERATED ); host-physics backends are either collapsed onto the Materialization kernel or explicitly pinned as the terminal bootstrap kernel with a content-addressed receipt — at which point this tracker is redundant.

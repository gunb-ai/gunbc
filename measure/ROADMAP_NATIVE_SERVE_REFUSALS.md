# Roadmap serve entry — native-serve refusal map (Route B first slice)

Goal (manager scope 1): run the roadmap serve entry natively OFF srv1 and name every refusal
between it and serving, fixing each at its earliest unjustified boundary. Remote build is down and
local whole-tree build is forbidden, so the EMPIRICAL confirmation is a fleet run (requested). This
is the static map that targets that run; each refusal is confirmed/extended by the actual emit+build+run.

Working native-serve entry to copy: `gunbc.auth.approval_device_crypto_serve` — a `NativeIngressDriver`
whose `NativeIngressProgram {binding, handler, startup?}` descriptor is resolved as a closed typed
boundary by `v1.compiler.emit_rust` `native_ingress_program_resolution`; instrument
`gunbc test //gunbc/instruments:native-serve-device-crypto` emits, builds, starts it and drives real TCP.

Roadmap serve entry today (interpreted): `gunbc run --entry dag/gunbc/roadmap/roadmap_ingress.dag
 --function roadmap_ingress --arg handler=... --arg listener=tcp:... --arg release_revision=<40hex>
 --arg cpu_limit_ms=.. --arg wall_limit_ms=..`; entry returns `HttpIngressAdmission{binding, handler}`
via binding function `roadmap_ingress_function`; served handler `roadmap_serve_handle_srv1_lab`
(signature) / `roadmap_serve_handle`.

## R0 — no native ingress driver exists for roadmap  (confirmed statically)
`grep -rn NativeIngressDriver dag/gunbc/roadmap` = none. The native route requires, in the module
that declares the binding constructor, a
`data compiler_pipeline_entry: CompilerEntryDriver = NativeIngressDriver { program: DeclarationRef }`
naming a `NativeIngressProgram {binding, handler, startup?}` descriptor (std/compiler_entry.dag:154).
The emitter refuses emission unless the descriptor names exactly one NativeIngressProgram whose three
refs name whole declarations of the right kind/signature.
EARLIEST BOUNDARY: create the roadmap native ingress descriptor + driver row (model on
approval_device_crypto_serve). Authorized by scope (1).

## R1 — handler signature must be plannable (http_ingress_fact_plan), judged before any bind
Handler params must be request facts over the listener + at most one startup value.
roadmap_serve_handle(method, path, body, tailscale_identity, request_headers, release_revision) —
these are request facts (method/path/body/header-derived/tailnet-attested peer/release), so it should
plan. REFUSAL IF a parameter names no fact (a_serve_contract_widened_for_one_handler_refuses_every_other).
EARLIEST BOUNDARY: if a param names no fact, supply it as a request fact or as the startup value; do
not widen the shared contract. Confirm empirically.

## R2 — binding constructor: release_revision must be exactly 40 lowercase hex
http_ingress binding constructor refuses a non-40-hex release (my earlier measurement proxy used
`measure-rev` and would refuse here). A real serve must pass a real git object name.
EARLIEST BOUNDARY: the deployment supplies a real revision; not a compiler concern.

## R3 (PREDOMINANT / caveat 2) — handler reads repo state at request time = a non-admitted native effect
roadmap_serve_handle calls `roadmap_plan_projection()` (reads the roadmap authority .dag) and
`srv1_live_dashboard_instance()`. The native main "performs only what no fold can" and admits only
route-declared effects (`std/compiler_entry` NativeEffectRealizationAdmission, per native ROUTE CLASS).
A per-request file read / host effect is very likely NOT admitted on the ingress route -> emission refuses.
EARLIEST BOUNDARY: model the authority projection as the NativeIngressProgram **startup fold**
(`NativeIngressStartup {fold, bound}`), run ONCE per worker generation before the bind, handing the
handler a bound startup value — exactly what the descriptor provides (device_crypto uses a startup
fold for its receipt). Reading the repo at request time is an ambient fact the design already forbids.
If a genuine per-request effect is required, name the admission at its owning route, not the handler.

## R4 — emit/build of the emitted closure (overlaps smart-badger-726's compile-time node)
1629 modules incl. the whole v2 compiler + giant data modules; every op must have an emit fold, and
the crate must cargo-build. EARLIEST BOUNDARY: each missing-fold emission refusal fixed where the fold
belongs (v1.compiler.emit_rust); the build-time cost is the sibling node, not this one.

## R5 — startup fold bound (Second)
If the plan projection is heavy it must finish inside NativeIngressStartup.bound or it refuses
(startup_slow / startup_overrun fixtures exist). EARLIEST BOUNDARY: bound sized to the real startup
work; do not raise a budget to pass.

## R6 — red_control 1 parity harness
Need a FIXED request set + the rule that the natively served entry answers the SAME responses as the
interpreted serve at the same revision. Model on native_serve_device_crypto_probe (seed host writes each
request to real TCP, returns wire bytes, verdict made in the .dag). EARLIEST BOUNDARY: define the fixed
request set + the native-serve instrument for roadmap (instruments: native_serve_* rows).

## Stored product (scope 2,3)
Reuse `native_product_cache.rs` (keys the emitted executable+build by the six causal axes; verified
read REFUSES a disagreement and rebuilds -> reds 2&3). Do NOT build a second cache. Folding into
provider request_key is in scope ONLY if the provider serves the native artifact as its only producer
(its doc's stated trigger); otherwise leave it and say so in the handback. (Check: provider does not
yet serve the native artifact per native_product_cache.rs:17 — likely leaves it in place.)

## What the fleet run must capture (ordered)
1. `native_ingress_program_resolution` on a roadmap driver row -> the exact refusal (R0 shape, then R3/R4).
2. emit of the emitted ingress main -> first non-foldable op / effect (R3/R4).
3. build + start -> listener bind, /livez, then the fixed request set (R6).

## DESIGN DOUBT (raised to manager; do not guess past it)
`NativeIngressProgram.handler` fixes the served handler by COMPILE-TIME DeclarationRef, but roadmap's
interpreted serve selects the handler as a DEPLOY-TIME argv fact (`roadmap_ingress --arg handler=...`,
gunbc.roadmap.roadmap_ingress:11), and the srv1 serve uses handler VARIANTS
(roadmap_serve_handle_srv1_lab, _door, _srv2_*) that bright-ibex-807 owns. So "which handler becomes
the native program" is a srv1 DEPLOY decision, not a compiler one, and silently fixing one would both
guess a srv1-touching fact and collapse the deploy-time handler selection the design intends
("serving is a deployment fact about an ordinary entry, carried by the binding").

DEFAULT PROPOSAL (pending manager): add a SEPARATE module dag/gunbc/roadmap/roadmap_native_ingress.dag
that mirrors instruments/native_serve_fixture.dag — its own binding ctor calling http_ingress_binding,
descriptor NativeIngressProgram {binding=that ctor, handler=roadmap_serve_handle_srv1_lab (the LAB
handler, parity target vs the interpreted lab serve), startup Absent}, plus
data compiler_pipeline_entry: NativeIngressDriver {program}. Leaves interpreted roadmap_ingress
untouched; srv1 native cutover stays bright-ibex-807's deploy step (scope 5). R0 = this row; then the
fleet run confirms R3 (per-request repo read -> startup fold) and R4 (emit/build).

## Refusals OBSERVED (self-serve fleet loop)
- run 43684606577: `gunbc test: no such target: //gunbc/instruments:native-serve-roadmap`. Root: the
  running compiler resolves test targets from the GENERATED src/v1/stage0/src/instrument_registry_generated.rs
  (instrument_registry_emit output) read at runtime, not the .dag roster. Fix: add the
  NativeServeProgram{entry} row the generator emits for the new instrument_bindings() binding.
- run 43685432632: `V2-NATIVE REFUSAL cause=EmissionRefused stage=emit ... native ingress program
  refused ... the handler names gunbc.roadmap.roadmap_serve.roadmap_serve_handle, whose module ... is not
  in the emitted closure (native_ingress_program_resolution: binding, handler and startup must each resolve
  to exactly one whole declaration ... before any main is rendered)`. Fix: import gunbc.roadmap.roadmap_serve
  { roadmap_serve_handle } in roadmap_native_ingress.dag so the handler's module is in the closure (mirrors
  native_serve_fixture importing its handler). compile.emit ran 13s then hard-refused. Expected next: R3
  (roadmap_serve_handle's per-request repo read: roadmap_plan_projection / srv1_live_dashboard_instance)
  not being a route-admitted native effect, and/or R4 missing emit folds across the now-included v2 compiler.

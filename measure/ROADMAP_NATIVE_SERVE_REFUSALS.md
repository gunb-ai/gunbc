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
- run 43686495756 (head 5c78cb4): native ingress PROGRAM now resolves (descriptor/handler/startup OK;
  closure emitted through compile.frontend 71s / normalize 3s / reconcile 20min / analyses 9s), then
  `V2-NATIVE REFUSAL cause=EmissionRefused stage=emit ... 2 hard diagnostics` = R4 missing emit folds in
  OTHER OWNERS' production modules the dashboard handler pulls in transitively:
    (a) roadmap_auth_routes.dag:90 `map(octets, o => o as Int)` -- no declared conversion UInt8<UInt,
        MachineWidth<>> -> i64; diagnostic says "name a declared conversion ... instead of `as`."
    (b) runner_slot_retirement.dag:510 `f as String` where the emitter emitted `compile_error!(
        UNRESOLVED_CompilerError)` for the operand, then `... operator_reason as String` (NonEmptyStr ->
        String). An unresolved inner + a NonEmptyStr->String cast with no fold.
  The handler's closure includes the whole v2 compiler (1629 modules) + auth/runner/roadmap data modules;
  no dashboard-scale program has ever been natively emitted, so more folds are expected behind these.
  COLD COMPILE of this closure ~24min (reconcile 20min) -- overlaps smart-badger-726's serve-entry
  COMPILE-TIME node (coordinate; do not duplicate).

SCOPE DOUBT raised to manager: red_controls 1-3 are about the native PRODUCT CACHE / serve-image load path
(entry-agnostic; already embodied by native_product_cache.rs: verified-read refuses disagreement & rebuilds
= reds 2&3, miss = rebuild). Proving them does not require the ENTIRE dashboard closure to emit. Options:
 (a) keep fixing R4 folds in other owners' production modules within this node (cross-owner, blind, many); or
 (b) prove reds 1-3 on a representative SMALLER served entry / handler, and treat full-dashboard native
     emission-readiness as its own track (owners' modules / overlaps the compile-time node).

## Scope (3) RESOLVED — do NOT fold native_product_cache into request_key (trigger not met)
native_product_cache.rs states the fold's trigger: dissolve the seed-local six-axis key into provider
`request_key` "when the native ancestry lane makes `provider_serve` the only producer
(`native_product_cache_seed_growth`)"; today "the provider does not yet serve the native artifact so
there is no `request_key` to inhabit." Verified: no `native_product_cache_seed_growth.dag` exists and no
provider_serve producer of `ArtifactRequest::NativeCompilerArtifactRequest` (the native emitted binary) is
present. => Trigger does NOT hold. Per manager scope (3): LEAVE the cache as the stored program product
(seed-local six-axis key incl producer compiler/source closure/toolchain/build config/lens contract;
verified-read re-hashes and REFUSES a disagreement then rebuilds; a miss costs one rebuild). Reuse that as
red_controls 2 (mismatch -> recompute) and 3 (corruption -> refuse+recompute, never serves). HANDBACK must
state: request_key fold not done because provider_serve is not yet the sole producer of the native artifact.

## R4 round progress (manager-authorized; calling-.dag fixes; never the emitter)
### (a) roadmap_auth_routes.dag:90 — FIXED (credential/PKCE path)
Retyped the octet chain `List<UInt8>` -> `List<Int>` (the values were already Int: base64_decode returns
List<Int>, auth_hex_bytes conses h*16+l) and dropped the refused `map(o, o as Int)`. Codified repair for
this class: gunbc.recurring_failure_mode::bounded_natural_arithmetic_evaluated_as_unbounded_int (UInt8 is a
phantom brand over Int that checks nothing; retype the producer to what it holds; the emitter correctly
refuses undeclared `as`). The range admission is carried UNCHANGED by uint8_octets_of_ints ->
first_uint8_out_of_range, which still refuses any member outside 0..255 before base64. WITNESS (manager asked
for one on PKCE): for every input the OLD route ACCEPTED, every octet was already an Int in 0..255 (that is
exactly what uint8_octets_of_ints admits), and on such values `o as Int` is the identity (the brand is
transparent to Int at a formal), so the List<Int> passed to uint8_octets_of_ints -- and hence the emitted
base64url wire -- is identical on every accepted input; refused inputs are still refused by the same fold.
No emitter change, no admission weakened, no value crosses the change. Sites touched: auth_base64url_wire
(param + body), AuthHexFold octets_reversed (both variants), auth_hex_bytes (return type).
### (b) runner_slot_retirement.dag:510 — DOUBT, not blindly edited
Diagnostic: `operator_reason as String` (NonEmptyStr -> String) + `f as String` emitted
compile_error!(UNRESOLVED_CompilerError). BUT approval_device_crypto_realization.dag:170 uses the IDENTICAL
shape join([X as String, " | ", Y as String], "") as NonEmptyStr on NonEmptyStr operands AND emits natively
on the fleet (native_serve_device_crypto_probe passes). So NonEmptyStr->String `as String` is NOT itself
refused; the genuine failure is the "unresolved inner" (`f as String` -> UNRESOLVED), and the NonEmptyStr->
String line is most likely a cascading secondary from the failed type of f. f is bound by
`Present { value: f } =>` over fabric_part: Optional<NonEmptyStr> (host_fabric_interruption_reason). This is
another owner's runner/production module with an emit-time resolution quirk I cannot root-cause from the log
alone; a blind edit risks changing behaviour. Next productive fleet round (with the auth fix) will report the
WHOLE list (emitter stops at 2) and re-emit 510 verbatim -> then decide. If it stays opaque, it goes to the
manager as its own question, not a guess-fix.

### Round 2 (job 132736722835, head 5c82314) result: list shrank 2 -> 1
Auth fix CONFIRMED: roadmap_auth_routes.dag:90 no longer refused. Remaining hard diagnostic (the emitter
aggregates ALL hard diagnostics per round, so this list is COMPLETE):
  cast cannot be emitted for target 'rust' -- no declared conversion route from
  compile_error!("UNRESOLVED_CompilerError") to String ... name a declared conversion instead of `as`
  (dag/gunbc/runner/runner_slot_retirement.dag:510:72)
ROOT CAUSE: the join combine closure names its second fold parameter `f` (the checked fold idiom
`f: (acc, f) => ...`, cf. test/claim/live_deploy/deployed_tree_observation_witness.dag:176). The inner match
arm bound its value to a bare `f` inside the join -> collided with the fold parameter -> `f` typed as an
error -> emitter lowered it to compile_error!("UNRESOLVED_CompilerError"), which has no declared crossing to
String. NOT a missing NonEmptyStr->String route: the identical join([X as String, " | ", Y as String], "") as
NonEmptyStr emits on the fleet from approval_device_crypto_realization.dag:170.
FIX (calling .dag, behaviour-preserving alpha-rename): rename the inner binding `f` -> `fabric_reason`. Pure
local rename; no behaviour, admission, or value crosses. Emitter not touched. Round 3 will report the complete
remaining list (expected: emit proceeds to native build/serve, or the true residual set).

# R2 PB — Canonical lens-name dispatch bridge: disposition (partial retirement + structural blocker)

**Status:** disposition artifact. Authored 2026-04-29 by PB Manager continuation per dispatch on inbox #1149 ("PB-owned bridge retirement slice — canonical lens-name dispatch / fixture-name routing").

**Gate:** `bridge_canonical_lens_name_dispatch_retired` (R3 T-Bridge-Retirement, distributed bridge #3, PB-owned per [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owns (R3 continuation)" + [`docs/r3-structure.md`](../r3-structure.md) T-Bridge-Retirement distribution map).

**This PR retires no further bridge surface.** It is an honest disposition artifact: the dispatch acceptance shape and ratchet test pin current state and document the precise substrate gap blocking full retirement, per the dispatch directive: "If a substrate gap blocks full retirement, make the PR an honest blocker/disposition artifact with a failing/ratcheting test for the remaining bridge shape and a precise dependency, rather than claiming the bridge is retired."

## What was already retired (state on main at authoring)

These were retired in earlier PB / B4 work and survive structurally on main:

1. **`PROGRAM_INPUT_SENTINEL` constant** — gone from `src/v3/compiler/src/test_runner.rs`. Replaced by the structural `ProgramInputRole` enum (`test_runner.rs:1418`) over the typed substrate carriers `ProgramInput {}` and `ProgramOutputBind { output_ref: DeclarationRef }` declared at `src/v3/std/verification.dag:239` and `:243`. The string `"r1_lens_output_input_from_program"` no longer dispatches; only fixture *declaration names* using that string survive in `tests/integration/test_runner_test.rs:761,771` (regression coverage scaffold), without any runner-side semantic load.
2. **`cost_bind_for_claim_file` filename routing** — gone. Replaced by the structural `ProgramOutputBind { output_ref }` carrier; `LensOutputEquals(cost_of)` reads the bind name via `ProgramInputRole::output_bind_name(...)` at `test_runner.rs:1798`. Fixture filename no longer selects an output bind.

## What remains (the canonical lens-name dispatch bridge surface)

One category of structural debt remains in `src/v3/compiler/src/test_runner.rs`.
R3 gate #33 retired the runner-side lens-name dispatch by routing
`LensOutputEquals` through typed marker declarations
(`CanonicalNamedFunctionCountLens`, `CanonicalCostLens`) instead of
`lens_decl.name`.

### A. `include_str!` of canonical lens bytes (2 sites)

- `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS` (`test_runner.rs:28-31`) — pulls `src/v3/lenses/named_function_count.dag` bytes at compile time.
- `R1_CANONICAL_COMPLEXITY_LENS` (`test_runner.rs:38-41`) — pulls `src/v3/lenses/complexity.dag` bytes.

The runner re-compiles these bytes via `compile_to_dag(...)` to obtain a "canonical" `Dag` for `apply_lens_declaration`, distinct from the fixture-compiled stub.

### B. String-name dispatch arms in `LensOutputEquals` (retired)

Retired by R3 gate #33. The `cost_of` and `named_function_count` special paths
now dispatch on `lens_ref` declarations that inhabit closed marker types in
`std.verification`, not on function-name strings.

### C. Generic name-keyed lens lookup (retired)

Retired by R3 gate #33. Ordinary non-canonical `LensOutputEquals` lenses execute
from the fixture graph via their `DeclarationRef`; they no longer probe
`program_dag` by the lens declaration's name.

## Why these survive — the substrate gap

The dispatch exists because [`INVARIANTS.md#p2-boundary-discipline`](../../INVARIANTS.md#p2-boundary-discipline) (Boundary Discipline) requires `Dag`-coherent reflection: when the runner reflects program nodes via `reflect_program_dag_nodes_in_file(...)` to feed a lens, the `Behavior`/`List` variant `DeclarationId`s in the reflected `FieldValue`s **must come from the same `Dag` instance** that the lens is applied against. Otherwise the lens's pattern-matches on `Behavior::Bind { ... }` etc. compare `DeclarationId`s across `Dag` instances and fail spuriously.

Concretely (see comment block at `test_runner.rs:1762-1772` and `:1832-1833`):

- The fixture (`r1_gates.dag`) declares `fn named_function_count(d: Dag) -> Int = fold(...)` at `:138` with a body textually identical to the canonical lens.
- The fixture also embeds the canonical lens text as a string literal in `source: "..."` at `:156-157` for `user_authored_lens_compiles_gate`.
- These are **two different `Dag` compilations** — the fixture's compiled output (`self.dag`) and the canonically-compiled lens (`compile_to_dag(R1_CANONICAL_..., ...)`) — with **different `DeclarationId` spaces** for the same `Behavior` variant constructors.
- For `LensOutputEquals` to evaluate, reflection and lens-application must agree on which `Dag` carries the variant ids. The runner picks one (canonical) and reflects against that same `Dag`.

**There is no `import` / cross-`Dag` `DeclarationRef` resolution mechanism today.** A `DeclarationRef` resolves only within its own compilation unit's `DeclarationId` space. Without one of:

(a) a structural mechanism for fixtures to import the canonical lens body into the same compilation unit so the fixture's `Dag` IS the canonical lens's `Dag`, OR
(b) a typed cross-`Dag` `DeclarationRef` carrier with reconciliation rules (substrate introduction), OR
(c) PB-Runtime interpreter-as-data that loads the canonical lens once and routes `lens_ref` resolution through typed identity rather than name-keyed lookup,

…the runner cannot drop the canonical byte bridge (A) without sacrificing the
P2 cross-`Dag` invariant. The name-keyed dispatch parts (B/C) are now replaced
by typed marker declarations.

## Precise dependency

Full retirement of `bridge_canonical_lens_name_dispatch_retired` is gated on **either**:

1. **PB-Runtime interpreter-as-data** landing (load-bearing for T-LensProducer-Retirement's `lens_apply.rs` retirement; see [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owns (R3 continuation)"). That work makes lens application a `.dag`-driven walk over typed `DeclarationRef` identity and can dissolve the remaining canonical byte bridge (A). OR
2. **A typed lens-registry carrier substrate-introduction** authored by Substrate Manager per [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) substrate-fact-introduction procedure — would dissolve the `include_str!` text bridges (A) by replacing canonical-lens-text with a typed `LensRegistryEntry` reference. The B4.1a worker brief explicitly STOPs and splits this as a sub-brief if it surfaces (per [`b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md) §STOP-AND-ESCALATE: "Canonical lens identity requires loading a second DAG by path. Do not replace one `include_str!` bridge with another string registry; split a structural lens-registry carrier brief.").

Either path is a substrate-level change outside PB authoring authority. PB territory cannot self-serve here without violating the dispatch guardrail "do not replace one string/path side channel with another."

## Acceptance shape (this PR)

- [x] No `PROGRAM_INPUT_SENTINEL` remains (already retired pre-PR; verified by grep).
- [x] Canonical lens lookup status documented; **not** retired in this PR (substrate gap above).
- [x] Ratchet test added that fails if the bridge surface grows: pins counts of (1) `include_str!` of `lenses/*.dag` files in `test_runner.rs`, (2) `lens_decl.name.as_deref() == Some("...")` arms, and (3) generic `lens_decl.name.as_deref()` name-keyed lookups. R3 gate #33 lowers (2) and (3) to zero. Rule per `feedback_ratchet_only_down`: never increase. See `tests/integration/canonical_lens_bridge_ratchet_test.rs`.
- [x] Manager brief annotated with partial-retirement / blocker state for the slice (this disposition file). **No per-bridge retirement signal to Verification** — `bridge_retirement_ledger_zero` (the actual unified ledger authority) is not advanced by this PR; the bridge is not retired. This is a blocker/disposition receipt, not a ledger-advance.
- [x] No replacement string / path side channel introduced.

**Not asserted:** that the bridge has been retired. The PR does not extend the partial retirement; it documents and pins it.

## Cross-refs

- Dispatch: PB Manager inbox #1149 (cool-stag-230 → neat-boar-747, 2026-04-29).
- Parent manager brief: [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owns (R3 continuation)" `bridge_canonical_lens_name_dispatch_retired`.
- T-Bridge-Retirement distribution map: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-Bridge-Retirement row.
- Adjacent worker brief (already-retired §0.1/§0.2 carriers): [`b4-1-declarationref-consumer-migration-worker.md`](b4-1-declarationref-consumer-migration-worker.md).
- Identity-carrier substrate pass: [`b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md).
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
- P2 cross-`Dag` reflection invariant (the constraint that holds the bridge in place): [`INVARIANTS.md#p2-boundary-discipline`](../../INVARIANTS.md#p2-boundary-discipline); runtime comment block at `src/v3/compiler/src/test_runner.rs:1762-1772` + `:1832-1833`.
- Gating R3 lane: T-LensProducer-Retirement (PB-Runtime interpreter-as-data sub-gate); see [`r3-structure.md`](../r3-structure.md) §"Lane structure".

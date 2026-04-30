---
status: PROPOSAL
owning_manager: Substrate Manager (R2 → R3 continuation)
lane: T-Anthropic-Wire (scope expansion per PR #1319)
authored: 2026-04-30 (PM deep-wolf-155 per PR #1319 Director ratification)
---

# R3 ProviderTypedWire<P> Carrier Extraction Worker Brief

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-30 by PM (deep-wolf-155) per PR #1319 ratification ask 4 ([gunbc#828 escalation](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356314413)).

**Owning manager:** Substrate Manager (R2 → R3 continuation per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 1; T-Anthropic-Wire scope expansion per Director ratification 2026-04-30).

**Lane size:** M (~1-2 weeks).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated on T-Anthropic-Wire base scope landing (or running in parallel per Substrate Mgr judgment) + R2-Evaluator readiness; see §"Dispatch preconditions" + §"STOP conditions". Substrate Manager re-reads at gate-clear.

## Scope

Author `ProviderTypedWire<P>` substrate primitives per Director ratification 2026-04-30 (PR #1319 amendment to [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire"). The primitives replace parallel-authority provider mirrors (OpenAI #1028 + Anthropic #1276 cadence) with **operation-indexed** parametric substrate consumed by per-provider parameter rows. Implementation splits provider-level shared facts (`ProviderConfig<P>`) from per-operation typing (`ProviderOperationWire<P>`) — see §"Carrier shape" for the design rationale (codex BLOCKING review on PR #1331 sha `1870104a` flagged single provider-level envelopes as losing per-operation type information).

Resolves R3 design challenge #8 ("R3 Anthropic vs R2 OpenAI") per [`docs/r3-structure.md`](../r3-structure.md) §"Design challenges" by committing to **path (a) — extract shared carrier** instead of the prior post-R3 dissolution-trigger framing. Drops the prior "6-month elapsed-time check" entirely per user directive 2026-04-30 *"nothing can be deferred past R3."*

The lane delivers:
1. `ProviderOperationWire<P>` + `ProviderConfig<P>` parametric substrate carrier declarations in `src/v3/std/provider_typed_wire.dag` (the conceptual `ProviderTypedWire<P>` per PR #1319 = these two carriers acting together; operation-indexed key per codex BLOCKING).
2. Per-provider parameter rows in `dsl/extdeps/providers/*/` (NEW directory; absorbs `dsl/extdeps/llm/openai.dag` + `anthropic.dag` shape into uniform structure).
3. OpenAI refactor to consume the carrier (currently at `dsl/extdeps/llm/openai.dag`).
4. Anthropic consumes the carrier from day one (matches the existing `anthropic_wire_typed_serde_alignment` gate per [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire").
5. Retirement of the parallel `src/v3/std/anthropic_schema.dag` mirror (the lockstep test's existence is structural evidence the bridge wants to dissolve — per `feedback_isomorphism_or_generation_for_mirrors.md`).

## Out of scope

- **Provider-domain divergence beyond shared schema** (e.g., Anthropic content blocks, OpenAI function-calling). Those remain provider-specific data rows at the leaf of the parametric carrier; the carrier does NOT force unification of genuinely-divergent wire formats.
- **Additional provider integrations beyond OpenAI + Anthropic** in this lane. The carrier shape is what unblocks future providers; actual integrations are post-R3 ecosystem (per user-affirmed §"Items confirmed staying post-R3" in PR #1319 escalation).
- **Anthropic content-block model** (`AnthropicUserContentBlock` etc.). Those stay as Anthropic-domain types per the `anthropic_schema_lockstep_test`; ProviderTypedWire<P> covers the shared shape (model list / message envelope / error envelope / wire-format), not provider-specific content shapes.

## Carrier shape (design sketch)

**Operation-indexed, not provider-level** (codex BLOCKING review on PR #1331 sha `1870104a` flagged that a single provider-level envelope loses per-operation typing — `dsl/extdeps/llm/anthropic.dag:168-200` declares `service llm.Anthropic { operation Messages { transport rest { ... } } }`; providers carry MULTIPLE operations each with own request/response/error envelopes). Carrier key is `(provider, operation)`, not `(provider)`. Two carriers split provider-level shared facts from per-operation typing:

```dag
// Per-operation wire envelope — one row per (provider, operation) pair.
type ProviderOperationWire<P> {
  provider_identity: P                  // type-tag (OpenAi | Anthropic | ...)
  operation_name: String                // e.g., "messages", "chat_completions", "embeddings"
  request_envelope: TypeRef             // request body type for this operation
  response_envelope: TypeRef            // success body type
  error_envelope: TypeRef               // error body type
  wire_contract: VariantEncoding        // typically shared per-provider; per-op override possible
  transport: RestTransport              // method, path, status mapping for this operation
}

// Provider-level shared facts — one row per provider.
type ProviderConfig<P> {
  provider_identity: P
  base_url: String
  auth: AuthScheme
  models: List<ModelSpec>
}

data anthropic_messages_wire: ProviderOperationWire<Anthropic> = { ... }
data openai_chat_completion_wire: ProviderOperationWire<OpenAi> = { ... }
data openai_responses_wire: ProviderOperationWire<OpenAi> = { ... }

data anthropic_config: ProviderConfig<Anthropic> = { ... }
data openai_config: ProviderConfig<OpenAi> = { ... }
```

The `P` parameter is a phantom-style type-tag that namespaces declarations without forcing nominal unification at the wire-shape level. Aligns with `feedback_naming_is_aliasing` (named types are namespaces) + `feedback_compositional_not_templating` (compositional substrate, not template duplication).

**Multi-operation coverage required.** R3 acceptance scope covers all THREE currently-declared REST operations across the two providers (verified against `dsl/extdeps/llm/{anthropic,openai}.dag`):

| Provider | Operations declared | Source |
|---|---|---|
| Anthropic | `Messages` | `dsl/extdeps/llm/anthropic.dag:168-200` (`service llm.Anthropic { operation Messages { transport rest { ... } } }`) |
| OpenAI | `ChatCompletion`, `Responses` | `dsl/extdeps/llm/openai.dag:152-220` (`service llm.OpenAI { operation ChatCompletion { ... } operation Responses { ... } }`) |

Each operation lands as its own `ProviderOperationWire<P>` row; future operations (Anthropic Tool Use, OpenAI Embeddings, etc.) land as additional rows under the same carrier without schema change.

**Design call deferred to dispatch time:** whether `ModelSpec` should itself be parametric (`ModelSpec<P>` for per-provider model-id format) or uniform. Substrate Mgr decides at brief-finalization based on what the OpenAI + Anthropic existing types share structurally vs diverge on. The brief assumes uniform `ModelSpec`; if dispatch reveals divergence, escalate back to Director.

## Acceptance gates (`.dag`)

Five gates compose the lane closure:

| Gate | Acceptance |
|---|---|
| `provider_operation_wire_carrier_landed` | Both `ProviderOperationWire<P>` and `ProviderConfig<P>` declared in `src/v3/std/provider_typed_wire.dag`; `BOOTSTRAP_FIXTURE_PATH_KEYS` includes the file; structural test confirms carriers are consumable from `.dag` programs |
| `openai_consumes_provider_operation_wire` | OpenAI module declares one `ProviderOperationWire<OpenAi>` row per OpenAI REST operation in `dsl/extdeps/llm/openai.dag` (currently `ChatCompletion` + `Responses` — both required) plus `data openai_config: ProviderConfig<OpenAi> = { ... }`; covers the migration via OpenAI ratchet test (new lockstep or extension of existing) |
| `anthropic_consumes_provider_operation_wire` | Anthropic module declares `data anthropic_messages_wire: ProviderOperationWire<Anthropic> = { ... }` + `data anthropic_config: ProviderConfig<Anthropic> = { ... }`; subsumes/replaces `src/v3/std/anthropic_schema.dag` as canonical authority; satisfies the existing `anthropic_wire_typed_serde_alignment` gate per [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire" |
| `provider_wire_old_authority_dissolved` | **Old per-provider files (`dsl/extdeps/llm/openai.dag`, `dsl/extdeps/llm/anthropic.dag`, `src/v3/std/anthropic_schema.dag`) are EITHER deleted OR exist as one-way generated projections from the new carriers** — NEVER as parallel hand-maintained authorities. See §"Migration discipline" below. SG-0 census reflects the dissolution. |
| `provider_wire_no_per_provider_duplication` | No parallel mirror authority for fields that the carriers cover; future provider integrations land as new `data ...: ProviderOperationWire<P>` rows, not new mirror files |

**Composition.** The lane closes when `Conj` over all five gates fires. Per `feedback_compiler_is_dag_processor`: no new substrate variant; structural composition over existing `BehavioralObservation`-shaped TestPredicates.

## Migration discipline — no parallel-authority window

Codex BLOCKING review on PR #1331 sha `1870104a` flagged the prior framing ("either retired or kept as v2-parsed legacy until v2 retirement") as exactly the parallel-authority anti-pattern (`feedback_parallel_representation_debt`). **The migration commits to ONE of two paths in the same PR — not deferred to T-V2-Retirement, not "kept as legacy":**

**Path (a) — preferred: deletion.** Old per-provider files (`dsl/extdeps/llm/openai.dag`, `dsl/extdeps/llm/anthropic.dag`, `src/v3/std/anthropic_schema.dag`) are deleted in the migration PR. Any v2 parser tests that referenced the old shape either get removed (if v2 retirement is concurrent) or get migrated to read from the new carriers via a thin adapter. **No file with hand-authored content survives the migration if a new-carrier equivalent exists.**

**Path (b) — fallback: one-way generated projection.** If v2 parser cannot read `ProviderOperationWire<P>` and v2 retirement (T-V2-Retirement, sibling R3 lane) is not yet ready, old files are regenerated from the new carriers via `regen_lens` or equivalent. The new carriers are the ONLY hand-authored authority; old files become build artifacts (per `feedback_no_generated_code_on_disk` discipline — they live as `OUT_DIR` outputs OR carry a generation header making them un-editable). **Bidirectional / hand-maintained-on-both-sides is rejected.**

**Decision criterion at dispatch time:** Substrate Mgr verifies whether v2 parser can be retired in the same wave (path a) or needs the old file shape during the dissolution window (path b). If path b: the projection script lands in the same PR; the projected file's first line is a generation marker; CI rejects manual edits to projected files.

**No third option.** "Kept as legacy until v2 retirement" is rejected — that's the deferral pattern user directive 2026-04-30 explicitly targets.

## Deliverables

1. **Carrier declarations** in `src/v3/std/provider_typed_wire.dag` (NEW file). Imports from `std.serialization`, `std.types`, `std.errors`. Exports `ProviderOperationWire<P>` + `ProviderConfig<P>` + supporting types (`ModelSpec`, `RestTransport`, etc.).
2. **OpenAI rows** at `dsl/extdeps/providers/openai/operations.dag` + `dsl/extdeps/providers/openai/config.dag` (NEW files under NEW directory). One `ProviderOperationWire<OpenAi>` row per OpenAI REST operation currently declared in `dsl/extdeps/llm/openai.dag` — verified live: `ChatCompletion` (`:163`) + `Responses` (`:200`). Both are required for migration; dropping either loses per-operation wire facts (codex BLOCKING inline review on line 46).
3. **Anthropic rows** at `dsl/extdeps/providers/anthropic/operations.dag` + `dsl/extdeps/providers/anthropic/config.dag` (NEW). One `ProviderOperationWire<Anthropic>` row for Messages (current scope of `src/v3/std/anthropic_schema.dag`); future rows for Tool Use etc. land under same shape.
4. **Old-authority dissolution** per §"Migration discipline" — path (a) deletion preferred; path (b) one-way generated projection acceptable IF v2 parser blocker forces it. Decision lands in same PR, not deferred. Files in scope:
   - `dsl/extdeps/llm/openai.dag` (delete or project)
   - `dsl/extdeps/llm/anthropic.dag` (delete or project)
   - `src/v3/std/anthropic_schema.dag` (delete; v3 has no v2-parser blocker since it's already a v3-side file)
5. **Lockstep tests retired or migrated** — `anthropic_schema_lockstep_test` (`src/v3/compiler/tests/integration/`) either dies (single-authority carrier; nothing to lockstep) or transforms into "carrier-row matches expected operation structure" assertion. Substrate Mgr decides at brief-finalization.
6. **`.dag` `TestClaim` suite** authored at `src/v3/std/verification.dag` (or sibling) — 5 claims per gate table above; composed into `provider_typed_wire_lane_closed` (lane-level structural acceptance; not a 6th gate, just the conjunction).
7. **Migration receipt PR** — single PR landing carriers + per-operation rows + old-authority dissolution together (per `feedback_bundle_workstreams_per_pr` — bundling closes the parallel-authority window in one merge, not over multiple PRs).

## Dependencies

Per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" + §"Dependency DAG":

1. **R2-Evaluator landed.** Carrier consumption requires `.dag` body evaluation. T-Evaluator close is the upstream gate (per [`docs/r3-structure.md`](../r3-structure.md) §"R3 worker dispatch precondition" — this lane is one of the 7 Evaluator-gated R3 lanes).
2. **R2 OpenAI typed wire (#1028) landed.** OpenAI must be in stable shape before refactor; otherwise refactor target moves under us.
3. **Existing T-Anthropic-Wire base scope.** This lane EXTENDS T-Anthropic-Wire per Director ratification 2026-04-30; must not contradict the base `anthropic_wire_typed_serde_alignment` + `anthropic_unit_enum_role_serialization_correct` gates.
4. **`feedback_isomorphism_or_generation_for_mirrors`** — anthropic_schema.dag mirror is exactly the pattern that memory entry warns against. Carrier extraction IS the dissolution.

## Dispatch preconditions

Worker dispatches when:
- R2-Evaluator readiness signal received from Evaluator Manager (or PR-B/C/D/E cadence has converged sufficiently).
- OpenAI #1028 has landed and stabilized on `main`.
- Substrate Mgr has finalized the design call on `ModelSpec` parametricity (uniform vs per-provider).

## STOP conditions

Worker STOPs and PINGs (canonical output: docs-only audit PR, per `feedback_worker_stall_diagnosis` substrate-gap-stall pattern) if:

1. **OpenAI + Anthropic wire shapes diverge structurally** beyond what `ProviderTypedWire<P>` can express. Surfacing this gap IS the right output — don't fabricate a unification that hides divergence. Escalate to Director: divergence may indicate path (b) (terminal-divergence ROADMAP row) was correct after all, OR the carrier shape needs additional axes.
2. **Parser doesn't yet handle `service { operation { transport rest } }` blocks.** Per `src/v3/std/anthropic_schema.dag` header note: "v3's parser does not handle the service block." If carrier extraction surfaces this gap, route to T-Ground-Services (parser-grammar slice) — cross-program coordination, not in-lane work.
3. **OpenAI refactor breaks the existing `canonical_lens_bridge_ratchet_test`** (or sibling). Migration must keep ratchet green; if it can't, surface to Substrate Mgr — likely carrier shape needs adjustment.

## Discipline

**Per `feedback_audit_adjacent_authority_first`:** before authoring `ProviderTypedWire<P>`, audit existing `services.dag` carriers, `extdeps_bootstrap_fixtures.dag`, `anthropic_messages.dag` for adjacent authority. Cite existing parents in PR description; don't restate.

**Per `feedback_isomorphism_or_generation_for_mirrors` (newly canonicalized 2026-04-30):** the existence of `anthropic_schema.dag` + the lockstep-test ratchet is canonical evidence the mirror wants to dissolve. Carrier extraction IS the dissolution; the lockstep test should not survive the migration unless it transforms into "carrier is the single authority" assertion.

**Per `feedback_substrate_principle_audit`:** apply the 6-question audit before declaring the carrier:
1. What problem are we solving? (Parallel-authority risk between OpenAI + Anthropic + future providers)
2. Is this an enumerable terminal? (Yes — provider identity is a phantom-tag namespace)
3. What structural recovery pattern? (Track 9 — parametric substrate primitive consumed by per-instance rows)
4. What dissolves? (`anthropic_schema.dag` mirror; future provider mirrors prevented from existing)
5. What's the C-checkpoint signal? (Carrier landed; parallel mirrors deleted)
6. What's the ratchet? (`provider_wire_no_per_provider_duplication` gate)

**Per `feedback_construction_over_ratchets`:** the gates are structural-acceptance, not heuristic perf-warnings. If migration can't satisfy a gate, the design is wrong — not the gate threshold.

**Per `feedback_no_textual_enforcement_bridges`:** gates fire on `.dag` `TestClaim` evaluation, NOT on grep over import lists.

## Cross-refs

- Parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire" (scope-expanded 2026-04-30 per PR #1319)
- Sibling brief: [`r3-pb-tier3-perf-budget-worker.md`](r3-pb-tier3-perf-budget-worker.md) (C1 sub-gate of T-Tier3-Dissolution; co-sibling authored in same wave per PR #1319 ratification ask 4)
- Substrate Manager scope: [`r2-substrate-manager.md`](r2-substrate-manager.md) §"R3 continuation: T-CostLens-Composition" + T-Anthropic-Wire scope expansion
- Existing OpenAI: `dsl/extdeps/llm/openai.dag`
- Existing Anthropic: `dsl/extdeps/llm/anthropic.dag` + `src/v3/std/anthropic_schema.dag`
- Director ratification: [PR #1319](https://github.com/gunb-ai/gunbc/pull/1319) (R3 amendment)
- Closure ledger: [`docs/r2-closure-ledger.md`](../r2-closure-ledger.md) §"Substrate Manager"
- INVARIANTS: §P5 (dispatch-discipline) + §P2 (no parallel authority)
- Memory cross-refs: `feedback_isomorphism_or_generation_for_mirrors`, `feedback_independent_cross_validation`, `feedback_audit_adjacent_authority_first`, `feedback_substrate_principle_audit`

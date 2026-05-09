---
status: PROPOSAL
owning_manager: Substrate Manager (R2 → R3 continuation)
lane: T-Anthropic-Wire (scope expansion per PR #1319)
authored: 2026-04-30 (PM deep-wolf-155 per PR #1319 Director ratification)
---

# R3 Provider Wire — Mirror Dissolution + T-Ground-Services Consumption Worker Brief

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-30 by PM (deep-wolf-155) per PR #1319 ratification ask 4 ([gunbc#828 escalation](https://github.com/gunb-ai/gunbc/issues/828#issuecomment-4356314413)).

**Owning manager:** Substrate Manager (R2 → R3 continuation per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 1; T-Anthropic-Wire scope expansion per Director ratification 2026-04-30).

**Lane size:** M (~1-2 weeks).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated on T-Anthropic-Wire base scope landing (or running in parallel per Substrate Mgr judgment) + R2-Evaluator readiness; see §"Dispatch preconditions" + §"STOP conditions". Substrate Manager re-reads at gate-clear.

## Scope

Address the provider parallel-authority risk per Director ratification 2026-04-30 (PR #1319 amendment to [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire"). Per codex BLOCKING reviews on PR #1331 (shas `1870104a` + `19dc267a`), the structural answer is **NOT** to extract a substrate carrier replacing the canonical service-block authority — that loses per-operation facts (`feedback_projections_must_compose_facts`). The structural answer is to make the canonical authority *natively parseable* via T-Ground-Services (R3 Grounding lane) and dissolve the v3-side parallel mirror.

Resolves R3 design challenge #8 ("R3 Anthropic vs R2 OpenAI") per [`docs/r3-structure.md`](../r3-structure.md) §"Design challenges" by committing to **canonical-authority preservation + parallel-mirror dissolution**, not carrier extraction. Drops the prior "6-month elapsed-time check" entirely per user directive 2026-04-30 *"nothing can be deferred past R3."*

The lane delivers:
1. **Provider directory relocation** — `dsl/extdeps/llm/{openai,anthropic}.dag` → `dsl/extdeps/providers/{openai,anthropic}/wire.dag` (path-only move; content unchanged). Establishes the uniform per-provider directory layout PR #1319 named.
2. **v3-side parallel mirror deletion** — `src/v3/std/anthropic_schema.dag` deleted; `BOOTSTRAP_FIXTURE_PATH_KEYS` reads the canonical extdeps directly via T-Ground-Services parser; `anthropic_schema_lockstep_test` retires.
3. **Optional thin parametric handle** — `ProviderTypedWire<P>` declared as a thin alias wrapping `Service<P>` (T-Ground-Services owned), IF cross-provider lens-instance authoring requires a parametric handle. Director-decision at brief-finalization; defaults to "no alias unless consumer demand" per `feedback_design_before_implement`.
4. **`anthropic_wire_typed_serde_alignment` gate satisfied** via the canonical service block as the single authority (per [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire"), not via a v3-side mirror.

## Out of scope

- **Provider-domain divergence beyond shared schema** (e.g., Anthropic content blocks, OpenAI function-calling). Those remain provider-specific data rows at the leaf of the parametric carrier; the carrier does NOT force unification of genuinely-divergent wire formats.
- **Additional provider integrations beyond OpenAI + Anthropic** in this lane. The carrier shape is what unblocks future providers; actual integrations are post-R3 ecosystem (per user-affirmed §"Items confirmed staying post-R3" in PR #1319 escalation).
- **Anthropic content-block model** (`AnthropicUserContentBlock` etc.). Those stay as Anthropic-domain types per the `anthropic_schema_lockstep_test`; ProviderTypedWire<P> covers the shared shape (model list / message envelope / error envelope / wire-format), not provider-specific content shapes.

## Lane reframe — depends on T-Ground-Services landing first

Codex BLOCKING review on PR #1331 sha `19dc267a` correctly flagged that my prior carrier sketch (`ProviderOperationWire<P>` with `request_envelope: TypeRef` / `response_envelope: TypeRef` / etc.) captures only a fraction of what `service { operation { ... } }` blocks at `dsl/extdeps/llm/{anthropic,openai}.dag` carry. **Deleting those extdeps with my carrier as replacement would lose facts** (per `feedback_projections_must_compose_facts`).

The facts at risk in `service { operation { ... } }` blocks (verified at `dsl/extdeps/llm/anthropic.dag:168-225` + `openai.dag:152-225`):

| Fact-class | Example (Anthropic Messages) |
|---|---|
| Output projection paths | `content: String from "content/0/text"`; `input_tokens: Int from "usage/input_tokens"` |
| Input field declarations (with types) | `api_key: Secret`, `model: String`, `messages: List<AnthropicChatMessage>`, `max_tokens: Int = 4096`, `temperature: Float?` |
| Output field declarations + types | per-field with projection path |
| Transport body composition | `body: { model: model, messages: messages, max_tokens: max_tokens, temperature: temperature, system: system }` |
| Transport headers | `headers: { "anthropic-version": current_api_version.version }` |
| Response status mapping | `200 => AnthropicMessages200Body, 4xx => AnthropicErrorShape, 5xx => AnthropicErrorShape` |
| Mock responses | per-status-code JSON examples with descriptions |
| Service-level config | `endpoint`, `auth`, `auth_input`, `auth_source: EnvVar`, `rate_limit`, `retry` policy |

A flat `ProviderOperationWire<P>` with 6 fields can't carry these without re-encoding every shape from scratch — duplicating the service-block grammar v2 already understands.

**The structural answer is to stop trying to bypass the parser.** Per `src/v3/std/anthropic_schema.dag` header note + ROADMAP `### Post-merge debt (2026-04-30 analyses)` "Provider/API mirror multiplication risk": the canonical fix is **T-Ground-Services parser-grammar slice landing** (R3 Grounding work; service/operation/transport-rest v3 parser-grammar + lowering). Once v3 parses `service { operation { transport rest { ... } } }` natively, the existing extdeps files at `dsl/extdeps/llm/{anthropic,openai}.dag` ARE the canonical authority — no carrier extraction needed for fact-preservation.

This lane therefore reframes as **dependent on T-Ground-Services** + **scoped to deletion of the v3-side parallel MIRROR** (`src/v3/std/anthropic_schema.dag`), NOT deletion of the canonical extdeps.

## What this lane actually does (post-reframe)

Three structural moves, all gated on T-Ground-Services landing:

1. **Provider directory relocation.** `dsl/extdeps/llm/openai.dag` → `dsl/extdeps/providers/openai/wire.dag`; `dsl/extdeps/llm/anthropic.dag` → `dsl/extdeps/providers/anthropic/wire.dag`. **Pure rename + reorganize**: file content unchanged structurally; service blocks land under a uniform per-provider directory layout. Old paths either delete-after-relocate (single PR) or carry a `// moved to ...` redirect during a documented transition window (see §"Migration discipline").
2. **v3-side mirror deletion.** `src/v3/std/anthropic_schema.dag` deleted; `BOOTSTRAP_FIXTURE_PATH_KEYS` updated to read the canonical extdeps file (which T-Ground-Services now parses). This is what the `feedback_isomorphism_or_generation_for_mirrors` discipline calls for: dissolve the parallel mirror once a generation-or-parser path makes it redundant.
3. **Optional thin parametric handle** (`ProviderTypedWire<P>` per PR #1319 ratification). IF cross-provider lens-instance authoring (e.g., `Lens<ProviderRateLimit>` over all providers) requires a parametric handle, author it as a **thin alias** over the parsed service block — NOT a re-reification of facts. Shape sketch:

```dag
// Thin parametric handle wrapping a parsed Service<P>.
// Does NOT re-encode operation facts; references the canonical service block.
type ProviderTypedWire<P> {
  provider_identity: P                  // type-tag (OpenAi | Anthropic | ...)
  service: Service<P>                   // canonical authority — parsed from extdeps
}

data anthropic_provider_wire: ProviderTypedWire<Anthropic> = { provider_identity: Anthropic, service: anthropic_service }
data openai_provider_wire: ProviderTypedWire<OpenAi> = { provider_identity: OpenAi, service: openai_service }
```

**`Service<P>` itself is owned by T-Ground-Services**, not this lane — the parametric type that lowers from a parsed `service { ... }` block. This lane only AUTHORS the optional `ProviderTypedWire<P>` thin alias if cross-provider parametric handles are needed for downstream consumers; otherwise the lane closes with just relocation + mirror deletion. Director-decision at brief-finalization: alias needed or not?

**Multi-operation coverage** is preserved by the existing service-block expressivity (verified at `dsl/extdeps/llm/anthropic.dag:168-225` for `Messages`; `openai.dag:152-225` for `ChatCompletion` + `Responses`). No fact is dropped because the canonical authority IS the service block; we don't replace it.

## Acceptance gates (`.dag`)

Four gates compose the lane closure. Each gate names its concrete observable subject + the substrate variant that observes it (or surfaces a substrate gap if no existing variant fits — codex BLOCKING on PR #1331 sha `aca422d2` line 65 caught the prior brief draft hand-waving "compose via existing `BehavioralObservation`" without naming subjects).

| Gate | Acceptance shape | Observable via |
|---|---|---|
| `provider_extdeps_relocated_under_providers_dir` | `dsl/extdeps/providers/openai/wire.dag` + `dsl/extdeps/providers/anthropic/wire.dag` exist; `dsl/extdeps/llm/{openai,anthropic}.dag` deleted (or carry redirect-with-deadline) | **Substrate gap** — no existing `TestPredicate` variant structurally observes filesystem path existence/absence. **Path (a)**: new variant `BootstrapFixturePathPresent { path: FilePath, must_exist: Bool }` or similar (Substrate Mgr territory). **Path (b)**: `ExecuteCommand { command: "test", args: ["-f"/"-e", path], expect_exit_code: 0 }` fallback (loses structural precision; subprocess-shaped) |
| `anthropic_schema_v3_mirror_dissolved` | `src/v3/std/anthropic_schema.dag` deleted; `BOOTSTRAP_FIXTURE_PATH_KEYS` reads canonical extdeps via T-Ground-Services parser; `anthropic_schema_lockstep_test` retired or transformed; SG-0 census reflects the deletion. Satisfies the existing `anthropic_wire_typed_serde_alignment` gate per [`docs/r3-structure.md`](../r3-structure.md) §"T-Anthropic-Wire" via the canonical service block as authority. | **Substrate gap (same as gate 1)** for the file-deletion check; for the BOOTSTRAP_FIXTURE_PATH_KEYS membership, a `Compiles` predicate on a fixture importing the canonical extdeps suffices (existing `TestPredicate::Compiles` covers it). For the lockstep-test retirement, SG-0 census counts already track Rust-test deletion structurally. |
| `provider_typed_wire_alias_optional` | EITHER (a) `ProviderTypedWire<P>` thin alias declared in `src/v3/std/provider_typed_wire.dag` wrapping `Service<P>` — IF cross-provider lens-instance authoring needs a parametric handle, OR (b) explicit Director-decided "no alias needed at this time" with consumer-demand trigger noted for future revisit. **Not both** — alias OR documented-no-alias, both rejected if facts get re-encoded. | If alias declared: `TestPredicate::Compiles` on a fixture that imports + applies the alias as type witness (covered by existing variant). If alias deferred: meta-fact recorded in lane closure ledger as PR-level checklist (not a `.dag` `TestClaim` — it's a recorded design decision, not a structural property). |
| `provider_wire_no_fact_re_encoding` | No carrier in `src/v3/std/` re-encodes per-operation facts (output projection paths, transport body bindings, headers, mock responses, response status mapping, rate-limit/retry config) — these live in the parsed `Service<P>` from the canonical extdeps. The only carriers post-this-lane are: `Service<P>` (T-Ground-Services), `ProviderTypedWire<P>` (optional thin alias). | **Substrate gap** — no existing `TestPredicate` variant structurally observes "no declaration of kind X exists in directory Y." **Path (a)**: new variant `NoDeclarationMatching { kind: DeclarationKind, in_directory: Path, except: List<DeclarationName> }` (Substrate Mgr territory; structurally walks declarations in scope). **Path (b)**: `ExecuteCommand { command: "tier3_no_re_encoding_check", ..., expect_exit_code: 0 }` fallback. **Path (c)**: this gate is PR-review checklist, not a structural lens — reviewer-enforced rather than gate-fired. Director-decision at brief-finalization. |

**Composition.** The lane closes when the gates fire — composition shape depends on which substrate paths land. Per `feedback_projections_must_compose_facts`: no fact authored at extdeps is dropped; all preservation is via T-Ground-Services parsing the canonical authority.

**Substrate-gap summary (Substrate Mgr decision at brief-finalization):**
- Path-existence/absence variant for gates 1 + 2 (subset of `feedback_no_textual_enforcement_bridges` — file-presence is a typed substrate fact, not a grep operation)
- Structural-absence variant for gate 4 (or accept gate 4 as PR-review checklist with explicit decision)
- Gate 3 already covered by existing `TestPredicate::Compiles` if alias declared; meta-fact recorded otherwise

This brief assumes path (a) for gates 1, 2, 4 (substrate-Mgr-authored variants) at finalization — same shape as C1's `PerfWithinBaseline` substrate gap. STOP+PING per §"STOP conditions" if Substrate Mgr declines.

## Migration discipline — no parallel-authority window

Codex BLOCKING reviews on PR #1331 (sha `1870104a` re kept-as-legacy + sha `19dc267a` re facts-flow-forward) flagged two parallel-authority anti-patterns. The migration discipline addresses both:

**Anti-pattern 1: kept-as-legacy until later.** Rejected entirely. The migration commits to relocation/dissolution in the same PR; "deferred to T-V2-Retirement" is not an option for this lane.

**Anti-pattern 2: replace canonical authority with subset-carrier.** Rejected — the canonical authority (the parsed service block in `dsl/extdeps/...`) MUST be preserved as the single-source-of-truth for facts. The deletion target is the v3-side parallel mirror at `src/v3/std/anthropic_schema.dag`, NOT the canonical extdeps.

**The migration moves are structural relocation + parallel-mirror dissolution** (no fact loss):

1. **Relocate** `dsl/extdeps/llm/{openai,anthropic}.dag` → `dsl/extdeps/providers/{openai,anthropic}/wire.dag`. File content unchanged structurally; only the path changes. Imports across the codebase update in the same PR.
2. **Delete** `src/v3/std/anthropic_schema.dag` (the v3-side parallel mirror). The canonical extdeps file is now the single authority; T-Ground-Services (R3 Grounding) gives v3 the parser to read it directly.
3. **Optionally author** `ProviderTypedWire<P>` thin alias if cross-provider lens-instance authoring needs a parametric handle. The alias references `Service<P>` (from T-Ground-Services); no fact reification.

**Old-authority deletion** in this lane refers ONLY to the v3-side parallel mirror, not the canonical extdeps. Path (a) deletion vs path (b) generated projection (from the prior brief draft) does not apply — there is no carrier-vs-extdeps fork. The extdeps IS the canonical authority.

## Deliverables

This lane delivers **3 structural moves**, all gated on T-Ground-Services landing:

1. **Provider directory relocation** — single PR moves `dsl/extdeps/llm/openai.dag` → `dsl/extdeps/providers/openai/wire.dag` and `dsl/extdeps/llm/anthropic.dag` → `dsl/extdeps/providers/anthropic/wire.dag`. File content unchanged structurally. Update all imports + bootstrap-fixture path keys + manifest references in the same PR. **Old paths deleted in same PR** (no transition window unless Substrate Mgr identifies a v2-side blocker, in which case a single-PR redirect with `// moved to ...` and a 1-week deletion deadline applies).
2. **v3-side parallel mirror deletion** — `src/v3/std/anthropic_schema.dag` deleted; `BOOTSTRAP_FIXTURE_PATH_KEYS` updated to read the canonical extdeps file (which T-Ground-Services now parses natively); `anthropic_schema_lockstep_test` either deletes (single authority; nothing to lockstep) or transforms into "service block parses + lowers consistent with expected operation reach" assertion (Substrate Mgr decides at brief-finalization).
3. **Optional `ProviderTypedWire<P>` thin alias** — Director-decision at brief-finalization. Author IF cross-provider lens-instance authoring requires a parametric handle (e.g., `Lens<ProviderRateLimit>` over all providers). Alias is a thin wrapper around `Service<P>` (T-Ground-Services owned); does NOT re-encode facts. If unnecessary for current lens consumers, defer until consumer demand surfaces — the canonical service-block authority is sufficient on its own.
4. **`.dag` `TestClaim` suite** authored at `src/v3/std/verification.dag` (or sibling) — claims per gate table above; composed into `provider_typed_wire_lane_closed` (lane-level structural acceptance).
5. **Migration receipt PR** — single PR landing relocation + mirror deletion + (optional) alias together (per `feedback_bundle_workstreams_per_pr`).

## Dependencies

Per [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" + §"Dependency DAG":

1. **T-Ground-Services parser-grammar slice landed (R3 Grounding lane).** **Hard prerequisite** — this lane CANNOT close without v3 parsing `service { operation { transport rest { ... } } }` blocks natively. Per ROADMAP `### Post-merge debt (2026-04-30 analyses)` "Provider/API mirror multiplication risk": "Corrective action: prioritize **shared T-Ground-Services ingestion path** over per-provider mirrors. Owner: R3 Grounding (post-Anthropic-chain); aligns with services.dag PR-γ..ω." Without T-Ground-Services, v3 can't read the canonical extdeps and the mirror dissolution would re-introduce the fact-loss problem.
2. **R2-Evaluator landed.** `Service<P>` lowering + downstream lens-instance authoring requires `.dag` body evaluation. T-Evaluator close is the upstream gate.
3. **R2 OpenAI typed wire (#1028) stable.** OpenAI must be in stable shape before relocation; otherwise the move target moves under us.
4. **Existing T-Anthropic-Wire base scope.** This lane EXTENDS T-Anthropic-Wire per Director ratification 2026-04-30; must not contradict the base `anthropic_wire_typed_serde_alignment` + `anthropic_unit_enum_role_serialization_correct` gates.
5. **`feedback_isomorphism_or_generation_for_mirrors`** — `anthropic_schema.dag` parallel mirror is exactly the pattern that memory entry warns against. The dissolution mechanism here is "canonical-parser-eliminates-mirror" (option 1: generation/parsing from the canonical extdeps), NOT carrier-extraction.

## Dispatch preconditions

Worker dispatches when:
- T-Ground-Services parser-grammar slice has merged on `main` and v3 parses `service { operation { transport rest { ... } } }` blocks natively (verified by a sample `.dag` test importing `dsl/extdeps/llm/anthropic.dag` and reading the parsed `Service<Anthropic>` reach).
- R2-Evaluator readiness signal received from Evaluator Manager (or PR-B/C/D/E cadence has converged sufficiently).
- OpenAI #1028 has landed and stabilized on `main`.
- Substrate Mgr has finalized two design calls: (i) optional `ProviderTypedWire<P>` alias needed or not? (defaults to "no alias unless lens-consumer demand exists"); (ii) substrate-gap resolution per gate table — author new path-existence + structural-absence `TestPredicate` variants (path a) OR fall back to `ExecuteCommand` (path b) OR accept gate 4 as PR-review checklist (path c). Per-gate decision recorded in lane closure ledger.

## STOP conditions

Worker STOPs and PINGs (canonical output: docs-only audit PR, per `feedback_worker_stall_diagnosis` substrate-gap-stall pattern) if:

1. **T-Ground-Services has not landed.** This lane has no work it can do without the parser-grammar slice. STOP and route signal to R3 Grounding Mgr — coordination, not in-lane work.
2. **Substrate-gap variants for gates 1, 2, 4 not authored** by Substrate Mgr (path a) AND no Director sign-off on path-(b) `ExecuteCommand` fallback or path-(c) PR-checklist downshift. Phase 2 has nothing structural to fire on. STOP and escalate.
3. **Service-block grammar diverges between OpenAI + Anthropic** beyond what `Service<P>` can express. Surfacing this IS the right output. Escalate to T-Ground-Services + Director: divergence may indicate the service-block grammar needs additional axes, OR the providers are genuinely terminal in their divergence.
4. **Optional alias scope creeps into fact reification.** If during dispatch the worker realizes the alias is being asked to encode facts the parsed `Service<P>` already carries, STOP and re-read this brief's §"Migration discipline" — that's the prior-brief mistake the codex BLOCKING review flagged. The alias is a thin wrapper, not a parallel encoding.
5. **Relocation breaks v2 parser tests in the same wave.** If v2 still reads from `dsl/extdeps/llm/...` paths during the dissolution window, the relocation can't simply delete old paths. Either (a) v2 retirement is concurrent (T-V2-Retirement sibling lane), OR (b) old paths get a `// moved to ...` redirect with a documented 1-week deletion deadline (single-PR transition; not "kept as legacy").
6. **OpenAI refactor breaks the existing `canonical_lens_bridge_ratchet_test`** (or sibling). Migration must keep ratchet green; if it can't, surface to Substrate Mgr — likely carrier shape needs adjustment.

## Discipline

**Per `feedback_audit_adjacent_authority_first`:** before authoring `ProviderTypedWire<P>`, audit existing `services.dag` carriers, `extdeps_bootstrap_fixtures.dag`, `anthropic_messages.dag` for adjacent authority. Cite existing parents in PR description; don't restate.

**Per `feedback_isomorphism_or_generation_for_mirrors` (newly canonicalized 2026-04-30):** the existence of `anthropic_schema.dag` + the lockstep-test ratchet is canonical evidence the mirror wants to dissolve. The dissolution mechanism is option (a) from that memory — generation/parsing from the canonical authority — implemented here via T-Ground-Services parsing the canonical extdeps. The lockstep test should not survive the migration unless it transforms into "service block parses + lowers consistent with expected operation reach" assertion.

**Per `feedback_substrate_principle_audit`:** apply the 6-question audit before this lane dispatches:
1. What problem are we solving? (Parallel-authority risk between OpenAI + Anthropic + future providers — the v3-side `anthropic_schema.dag` mirror is the concrete instance)
2. Is this an enumerable terminal? (Yes — provider identity is a phantom-tag namespace; service blocks are a finite grammar parsed by T-Ground-Services)
3. What structural recovery pattern? (Track 9 + `feedback_isomorphism_or_generation_for_mirrors` option (a) — canonical authority preserved; parser eliminates the parallel mirror)
4. What dissolves? (`src/v3/std/anthropic_schema.dag` v3-side parallel mirror; future provider mirrors prevented from existing because the canonical extdeps IS the single authority)
5. What's the C-checkpoint signal? (T-Ground-Services parses service blocks; v3-side mirror deleted; no `.dag` declaration in `src/v3/std/` re-encodes service-block facts)
6. What's the ratchet? (`provider_wire_no_fact_re_encoding` gate — name matches §"Acceptance gates" table verbatim)

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

# Provisioning window / executor-capability model — design for sign-off

> **Status: DESIGN-ONLY, sign-ready draft.** No implementation. Parent arc: **ShellProgram → DAG de-fork** (witty-ibex-317). Formalizes the operator-signed **bash-minimization rule** ([shell-emission-model.md](shell-emission-model.md)) into typed authorities that gate when shell/bash emission is legitimate vs when typed argv / typed transport / typed plan interpretation is required. Complements [orchestration-as-intent-design.md](orchestration-as-intent-design.md) (gap B — *what* to emit) and [host-effect-orchestration.md](host-effect-orchestration.md) (Phase D `EmitArtifactThenThinRun`); the residual census and arc-completion scoping that builds on this authority is tracked in [shell-to-dag-residual-census-and-arc-completion.md](shell-to-dag-residual-census-and-arc-completion.md). DESIGN refs: §2 (decompose, don't re-coin), §3 (intent / transport / policy are three facts; dispatch is realization), §5 (fail-closed; construction over validation), §6 (scaffold with named dissolution trigger).
>
> Every place I am unsure is tagged **⚠ FLAG**. This lane records direction for operator + parent sign-off; no load-bearing `.dag` edit until signed.

---

## 0. The one-sentence claim

> **Shell emission is authorized exactly when the execution site's executor capability requires an opaque shell payload *or* the host is still inside a bootstrap provisioning window where no gunbc runtime and no typed transport handler exist on the path** — otherwise the site must realize as typed argv, typed REST/Redfish, or binary-interpreted effect-plan (`EmitArtifactThenThinRun`), and any new bash outside those two justifications is a typed refusal, not a roster add by default.

The bash-minimization rule today lives in prose and partial lens residue (`medium_structure_exception_roster`, `realization_vocabulary_containment`). It is **correct but not constructible**: a contributor can still add concat-built shell or a new `ShellProgram` importer and only discover the violation via a shrinking roster ratchet. This design names the **single authority** that makes the rule a construction wall.

---

## 1. The gap (why prose is not enough)

Three live failure modes motivate the model:

| failure mode | live tell | why rosters alone miss it |
| --- | --- | --- |
| **Runtime-present shell** | `live_deploy`, srv3 install tails, fleet-converge srv1/srv2 arms, former `dag/tools` `ShellProgram` witnesses | A roster entry permits the leak but does not say *what should replace it*; migration lanes re-litigate the same axis each time |
| **Foreign-executor confusion** | GHA `RunStep { run: String }`, cron entry lines, githooks | These **are** legitimate shell sites — but only because the executor capability is `ShellPayloadRequired`, not because "it's CI" |
| **Bootstrap vs steady-state drift** | `.github/fleet-converge.sh` steady-state arms vs fresh-standup bootstrap fragment | Without a typed window + lane E ruling, slice 2 cannot distinguish *interim authorized* steady-state shell (Doc + golden, dissolution-trigger-bound) from fresh-standup cutover (gated on slice 1) |

The 2026-07-03 **pre-runtime census** ([shell-emission-model.md](shell-emission-model.md)) already classified sites. This design **lifts that census into typed rows** so slice sequencing, host-effect migration, and lens enforcement share one authority.

---

## 2. Grounding (DFS before coining)

The brief names `ProvisioningWindow` and `ExecutorCapability`. DFS against the concept DAG:

| proposed name | DFS verdict | grounds into (existing carrier) |
| --- | --- | --- |
| **`ExecutorCapability`** | what payload shapes an execution site **accepts** | §3 **transport** fact — orthogonal to effect shape and policy. Decomposes to a closed coproduct of payload classes; **not** a nickname for `HostEffectTransport` (that is *how gunbc reaches a host*, not what the outer executor understands). |
| **`ShellPayloadRequired`** | foreign executors that only consume opaque shell text | `extdeps.github.actions.RunStep.run`, `extdeps.cron` entry line, git hook file body, (future) autoinstall late-command — each cited in extdeps with version |
| **`TypedArgv`** | program + args → exit/stdout; no bash-the-language | `extdeps.*` service ops + `gunbc.WitnessBin.Run` pattern (`dag/tools/dag_compile_clean_transport.dag` — Phase 3a landed) |
| **`TypedRest`** | structured HTTP/Redfish verbs | `RedfishAction` arm + BMC extdeps |
| **`TypedPlanInterpretation`** | binary interprets a modeled effect-plan; emits receipts | `EmitArtifactThenThinRun` transport (prose-only today; host-effect Phase D **models** the arm — this capability is the consumer-side justification for *not* emitting steady-state converge bash) |
| **`ProvisioningWindow`** | lifecycle phase where bootstrap shell is the **lowest-dependency** correct realization | Host lifecycle before runtime availability on the execution path — **not** a third policy axis. Grounds into fleet lifecycle facts (`FactoryDefault → OsInstalled → …`) **at the product use-site**; std keeps the abstract window shape only |
| **`RuntimeAvailability`** | evidence that typed realization is reachable | `GunbcBinaryPresent` · `TypedTransportBound{transport}` · `EffectPlanInterpreterPresent` — a **conjunctive** gate; bootstrap window closes when **any** typed arm is live on the path |

**Net-new concepts:** `ExecutorCapability` (payload-class coproduct), `ProvisioningWindow` (abstract window + product lifecycle binding), `ShellEmissionJustification` (the authorization witness tying a site to `(capability, window)`). Everything else is injection from extdeps specs or product fleet rows.

**Layer-DAG invariant (§3):**

| part | layer | why |
| --- | --- | --- |
| `ExecutorCapability`, `RuntimeAvailability`, `ShellEmissionJustification` shape | **std** (`v2.std.execution_surface` — parent-endorsed home; ⚠ FLAG 2a: operator sign at merge) | universal framework; no fleet knowledge. **Not** `std.effects` (effect shape ≠ executor payload demand — fusing them is the shape/transport fusion §3 warns about). **Not** product-only — M2 lens (`src/v2/lens`) and census rows (`dag/gunbc`) both consume the shape; dag-tree may import `v2.std` per the slice-0 `ci_spec` cutover precedent, so one `v2.std` home serves both trees. Distinct from `v2.std.host_transport` (how gunbc *reaches* a host, not what the outer executor accepts). |
| `ForeignExecutorKind` (GHA `run:`, CronLine, GitHook, AutoinstallLateCommand) | **extdeps** | cite real upstream specs (`actions.dag`, cron model, …) |
| `HostLifecyclePhase`, per-host window rows, census site table | **product** (`gunbc.fleet_intent` / shell-emission census module) | references `ComputeHost`, install milestones |
| dispatch selecting realization from `(justification, effect)` | **peripheral** (realization edge + `host_effect_realize`) | §3: dispatch is realization, not central shape |

---

## 3. The model (the shape to sign)

### 3.1 ExecutorCapability — demand axis

```text
type ExecutorCapability
  = ShellPayloadRequired { foreign: ForeignExecutorKind }   # only opaque shell text is consumable
  | TypedArgv                                               # witness/service-op path; bash syntax is spelling only
  | TypedRest                                               # Redfish/REST; structured body
  | TypedPlanInterpretation                                 # EmitArtifactThenThinRun / effect-plan interpreter

type ForeignExecutorKind
  = GitHubActionsRunStep                                    # extdeps.github.actions — run: block
  | CronEntryLine                                           # extdeps.cron.schedule_model
  | GitHookScript                                           # .githooks/* foreign media
  | AutoinstallLateCommand                                  # cloud-init/autoinstall late-commands (future; zero live shell today)
```

**Decidability rule (§5 wall, not vibe):** classify by **who executes the emitted artifact**, not by module path prefix:

- `claim_executor` invoking `gunbc compile` → `TypedArgv` (even if a shell could spell it).
- GHA runner evaluating `run: |` → `ShellPayloadRequired { GitHubActionsRunStep }`.
- srv1 `LocalShell` with gunbc as invoker → `TypedArgv` / `TypedPlanInterpretation`, **never** `ShellPayloadRequired`.

**⚠ FLAG 3a — `UsesStep` actions.** Third-party actions (`actions/checkout`) are typed argv **at the action boundary** but GHA still orchestrates them via YAML — not shell emission from gunbc. No gunbc shell is authorized for `UsesStep`; flag retained in case an action wrapper emits inner shell (today: none in corpus).

### 3.2 ProvisioningWindow — lifecycle axis

```text
type ProvisioningWindow
  = BootstrapBeforeRuntime { until: RuntimeAvailabilityGate }
  | SteadyState                                             # runtime or typed transport available — bootstrap shell forbidden

type RuntimeAvailabilityGate {
  gunbc_binary: AvailabilityFact
  typed_transport: AvailabilityFact
  plan_interpreter: AvailabilityFact
}

type AvailabilityFact = Absent | Present | Unknown { cause: String }   # Unknown = fail-closed bottom
```

**Window semantics:**

- **`BootstrapBeforeRuntime`** — all three availability facts are `Absent` on the execution path. This is the "(b) bootstrap windows" half of the bash-minimization rule: before gunbc runtime **or** a typed transport exists on the host, emitted bash is the honest lowest-dependency bootstrap.
- **`SteadyState`** — at least one availability fact is `Present`. Shell emission requires `ShellPayloadRequired`; otherwise refuse.

**Product binding (use-site injection, not std import):**

| host phase (product) | typical window | notes |
| --- | --- | --- |
| `FactoryDefault … OsInstalling` | `BootstrapBeforeRuntime` | BMC/out-of-band only; in-band shell not reachable yet |
| `OsInstalled`, gunbc on PATH, witness transports live | `SteadyState` | srv3 install tail, live_deploy, dag/tools — **runtime-present** census class |
| GHA runner (no gunbc on runner until bootstrap build step) | per-step: bootstrap fragment → `BootstrapBeforeRuntime`; post-build steps with `claim_executor` | CI three-tier boundary ([emission-ingestion-inverse.md](emission-ingestion-inverse.md) gap C) |

**Window granularity (parent-endorsed; ⚠ FLAG 3b: operator sign at merge):** **per emission site**, not per file or per job. A single GHA job mixes bootstrap shell steps and post-build runtime-present steps; coarser grain would be state-space conflation — silently authorizing steady-state shell where `TypedArgv` is required. Each census row carries `site id` + step boundary.

### 3.3 ShellEmissionJustification — authorization witness

```text
type ShellEmissionJustification
  = ForeignExecutorMandated { capability: ShellPayloadRequired }
  | BootstrapProvisioned { window: BootstrapBeforeRuntime, site: ShellEmissionSiteId }
  | RosterException { roster_key: String, dissolve_trigger: String }   # shrinking residue only

type ShellEmissionAuthorization
  = Authorized { justification: ShellEmissionJustification }
  | Refused { cause: ShellEmissionRefusal }   # typed, located, counted

type ShellEmissionRefusal
  = RuntimePresentRequiresTypedRealization { capability: ExecutorCapability }
  | WindowClosed { window: SteadyState, attempted: ForeignExecutorKind? }
  | UnknownAvailability { gate: RuntimeAvailabilityGate }
  | UndeclaredSite                                          # no justification row and not on roster
```

**Authorization predicate (the sign-off bar):**

```text
authorize_shell_emission(cap: ExecutorCapability, window: ProvisioningWindow, site: ShellEmissionSiteId)
  → if cap = ShellPayloadRequired(_)           → Authorized ForeignExecutorMandated
  → else if window = BootstrapBeforeRuntime{…} → Authorized BootstrapProvisioned   (only if availability Unknown → Refused)
  → else if site ∈ roster with dissolve_trigger → Authorized RosterException
  → else                                         → Refused RuntimePresentRequiresTypedRealization
```

This is **construction-first** target state: an emit API takes `ShellEmissionJustification` and makes `UndeclaredSite` unwritable. Until then, the lens enforces the same predicate read-only (§5: validation residue with named dissolve-on).

---

## 4. Census table (2026-07-03 receipt → typed rows)

Authoritative until re-census. Each row is one `ShellEmissionSiteId` with `(capability, window)` — the sign-off artifact operators can diff.

| site | capability | window | migration target |
| --- | --- | --- | --- |
| `ci_spec.ci_cargo_eagain_retry_core` | `ShellPayloadRequired GHA` | `BootstrapBeforeRuntime` (pre-`claim_executor` build) | **LANDED** #6467 — slice 0 `emit(Retry, Bash)` production cutover |
| `ci_workflow` inline `RunStep`s (8) | `ShellPayloadRequired GHA` | mixed per step | slice 4 — `TargetArchitecture` dispatch |
| `fleet_converge` fresh-standup arm | `ShellPayloadRequired GHA` | `BootstrapBeforeRuntime` | **Lane E** (witty-lark-895): cutover gated on tier-1 `If` emit band (slice 1); stays emitted bash until then |
| `fleet_converge` srv1/srv2 steady-state arms | `ShellPayloadRequired GHA` *(interim)* / target `TypedPlanInterpretation` | `SteadyState` | **Lane E ruling (parent, 2026-07-11):** **untouched** — Doc projection + committed golden (`.github/fleet-converge.sh`); ONE dissolution-trigger row binding → `EmitArtifactThenThinRun` (slice 2). Do not rewrite steady-state bash until thin-run lands |
| `githooks` pre-push shim | `ShellPayloadRequired GitHook` | `BootstrapBeforeRuntime` | slice 4 thin shim + typed stdin parse in binary |
| `bmc_token_federation` smoke | `ShellPayloadRequired GHA` | `BootstrapBeforeRuntime` | slice 4 (slice 0 machinery) |
| cron entry lines | `ShellPayloadRequired Cron` | `SteadyState` | foreign executor — permanent shell **framing** (§7) |
| srv3 install / reconcile tails | `TypedArgv` | `SteadyState` | host_effect typed observe rows |
| `live_deploy` | `TypedArgv` / plan | `SteadyState` | slice 3 typed effects — not freeze-and-emit |
| `dag/tools` witness transports | `TypedArgv` | `SteadyState` | **LANDED** Phase 3a — proof the axis works |

Construct scope for **pre-runtime justified bash** (unchanged from census): Run seq · If/else · pipes · cmdsubst · `$?` · AndOr · redirects · env · Retry · `TargetArchitecture` dispatch. **Not justified** at any pre-runtime site: For, While, trap, background, functions, arrays, arithmetic, process substitution.

**Lane E cross-link (witty-lark-895, parent ruling 2026-07-11):** slice 2 has one story across this census and the fleet-converge emit lane — (i) steady-state srv1/srv2 arms **stay** as Doc projection + committed golden with a single `EmitArtifactThenThinRun` dissolution trigger (no premature rewrite); (ii) fresh-standup bootstrap cutover waits on the tier-1 `If` emit band (slice 1). Authorization model: steady-state arms are **interim** `ShellPayloadRequired` via the foreign GHA executor path until `TypedPlanInterpretation` is modeled; the lens must not RED them today — only refuse *new* steady-state shell after slice 2 closes the window.

---

## 5. Enforcement integration (construction → lens)

Phased, same pattern as §5 containment guard ([emission-ingestion-inverse.md](emission-ingestion-inverse.md)):

1. **Phase M0 — LANDED (this PR).** `ExecutorCapability` / `ForeignExecutorKind` / `ProvisioningWindow` / `RuntimeAvailabilityGate` / `AvailabilityFact` / `ShellEmissionJustification` / `ShellEmissionAuthorization` / `ShellEmissionRefusal` minted in `src/v2/std/execution_surface.dag` (the FLAG-2a-endorsed home), with the `authorize_shell_emission` predicate implementing §3.3's decision table exactly (foreign-executor demand always authorized; bootstrap window authorized unless the availability gate carries an `Unknown` fact, which fails closed rather than defaulting to `Absent`; steady state authorized only through a named `ShellEmissionRosterEntry` with its `dissolve_trigger`, else refused). Green by execution: `src/v2/test/claim/shell_emission_authorization_test.dag` exercises all four `ShellEmissionAuthorization`/`ShellEmissionRefusal` outcomes, including the fail-closed `Unknown`-availability refusal and the roster-miss-for-a-different-site refusal (a same-shaped-window false positive would go RED). Zero emit-path change — no producer calls the predicate yet; that is Phase M1.
2. **Phase M1** — each shell-emitting module declares `shell_emission_site_id` + `ShellEmissionJustification` datum (scaffold — lying datum goes RED via witness).
3. **Phase M2** — `v2.lens.shell_emission_authorization` (pure reader): undeclared site, closed-window shell, or capability mismatch → `ShellEmissionLeak` (sibling to `MediumStructureLeak`, **not** merged — different predicate).
4. **Phase M3** — emit APIs require `Authorized` justification; roster exceptions shrink; undeclared → **unwritable**.

**Relationship to existing lenses:**

| existing mechanism | relationship |
| --- | --- |
| `realization_vocabulary_containment` | orthogonal — catches `program.dag` AST imports; this model catches **whether shell should exist at all** |
| `medium_structure_containment` | complementary — catches stringly medium leaks; authorized shell still must route through `emit(intent, Bash)` / grammar rows at the edge |
| `medium_structure_exception_roster` | **shrinking residue** for M1–M2 only; each entry must carry `dissolve_trigger` pointing at a census migration row. Roster growth ratchet stays fail-closed — **frozen at current 58 baseline** (parent-endorsed; ⚠ FLAG 4: operator sign at merge); new shell admits only via `ForeignExecutorMandated` / `BootstrapProvisioned` or explicit operator sign-off row |

**Discriminating RED witnesses (§5):**

- Plant a runtime-present module emitting shell without justification → RED.
- Plant a GHA bootstrap site with `TypedArgv` misclassification → RED (would authorize wrong realization).
- Closed-window fleet-converge steady-state shell after slice 2 → RED.

---

## 6. Thin-run guardrail (load-bearing)

`TypedPlanInterpretation` is **not** a license for an opaque Rust driver. Authorization requires:

- the emitted artifact is a **thin invocation** (argv) plus optional foreign-media bootstrap fragment, and
- the binary interprets a **typed** `EffectPlan` / orchestration intent authority, emitting typed receipts.

Replacing a 400-line bash script with a 400-line imperative loop in the seed is **refused** — same §5 fabricated-plausible-output trap as absorbing fallback, one level up. This guardrail is already prose in [shell-emission-model.md](shell-emission-model.md); this model makes it a refusal variant on `ShellEmissionAuthorization`.

---

## 7. Permanent vs migratable residue (roster honesty)

Split roster entries per shell-emission-model §7(c):

| class | fate | example |
| --- | --- | --- |
| **Migratable control-flow shell** | dissolves via orchestration intent + thin-run (after lane E interim period) | fleet-converge steady-state (interim: Doc + golden until `EmitArtifactThenThinRun`), live_deploy verbs |
| **Permanent foreign-media framing** | stays `ShellPayloadRequired` forever | GHA `run:` block wrapper, cron line, git hook file envelope |

Permanent rows **never** claim `BootstrapBeforeRuntime` in steady state — they use `ForeignExecutorMandated` only. The lens checks this distinction by model walk, not grep.

---

## 8. Sequencing (post sign-off — NOT this task)

1. Mint std/product types (M0 code) — **blocked on this sign-off**.
2. Wire census rows as data in `gunbc.shell_emission_census` (product).
3. Land M1 scaffolds alongside orchestration slice 0/1 (same PR acceptable if justification datums are honest).
4. M2 lens + RED witnesses before slice 2 (converge thin-run) — slice 2 **requires** the lens to prove steady-state arms are not authorized shell.
5. M3 construction wall when emit path accepts justification parameter.

**Dependency:** gap B orchestration vocabulary ([orchestration-as-intent-design.md](orchestration-as-intent-design.md)) is **parallel**, not upstream — authorization governs *whether* shell may be emitted; orchestration governs *what* intent emits when authorized. Slice 0 **landed** (#6467): authorized bootstrap shell + `Retry` intent.

---

## 9. Open questions (sign-blocking vs follow-on)

**Sign-blocking (parent-endorsed 2026-07-11; final sign = operator at merge per project convention — strike ⚠ FLAG markers when signed):**

1. **⚠ FLAG 2a — module home.** **Parent-endorsed:** new module **`v2.std.execution_surface`**. Do **not** extend `std.effects.dag` (effect shape and executor payload demand are different axes). Do **not** go product-only (M2 lens + census rows in `dag/gunbc` both consume the shape). Genuinely net-new — distinct from `v2.std.host_transport`.
2. **⚠ FLAG 3b — window granularity.** **Parent-endorsed:** **per emission site** (not per job or per file — coarser grain is state-space conflation in mixed bootstrap/runtime GHA jobs).
3. **⚠ FLAG 4 — roster exception admissibility.** **Parent-endorsed:** **freeze `RosterException` at current 58 baseline** (`medium_structure_exception_roster_size_baseline`). New shell admits only via `ForeignExecutorMandated` / `BootstrapProvisioned` or explicit operator sign-off row — never self-service roster add. Ratchet is downstream of migration, not a path to it.

**Follow-on (does not block M0 mint):**

4. Autoinstall late-command shell (zero live sites today) — pre-authorize enum arm only.
5. Cross-link `EmitArtifactThenThinRun` modeling in host-effect Phase D with `TypedPlanInterpretation` — same concept, two readers (host-effect dispatch + shell authorization).
6. Whether `Run.command: String` in orchestration dissolves to `Do{effect}` leaves before or after M3 (host-effect Phase B sequencing — [host-effect-orchestration.md](host-effect-orchestration.md)).

---

## Dissolution trigger (DESIGN §6)

Delete this doc when `ShellEmissionJustification` is minted in std/product, every live shell-emit site carries an `Authorized` witness or a shrinking roster exception with dissolve trigger, the M2 lens is green on the clean tree + RED on planted leaks, slice 2+ migrations are gated on the authorization predicate, and roster exceptions for migratable control-flow shell have emptied — at which point the bash-minimization rule lives in executable construction + lens, and this prose is redundant (fold into [shell-emission-model.md](shell-emission-model.md) as a short § pointer or dissolve both into the model rows).

# Shell → intent: the workflow layer is language-blind

**Status:** design, operator-aligned 2026-07-17; **finish-line sharpened 2026-07-19** (subatomic-modeling standard). This doc owns the §3/§4 *intent-names-no-language* facet; the residual census, the routing decision, and the target composition band live in sibling authorities and are **referenced, not restated** here (see "This doc's scope" below). Anchors the rescope of the shell→dag arc onto §3/§4. The sidecar delete (bash `ShellStmt`/`program.dag` removal) is *Phase 0* of this, not the whole story.

> **The finish line (operator, 2026-07-19).** The arc is *not* done at "no `bash_build` in the intent." It is done when **every raw-concat shell string is dissolved** — routed to its correct realization (typed effect-plan for runtime-present sites; bounded bash emit for foreign-executor/bootstrap sites) — and the anemic `String` carriers (`host_effect.ShellCommand{script}`, `std.orchestration.Run.command`, `host_effect.BootstrapFragment.script`) can no longer be *constructed* from the intent layer. Rationale is §5-economic, not aesthetic: **while a raw-shell builder exists in the tree, LLMs copy it** — one anemic `concat("sudo -n ", …)` becomes ten. So the standard is set high up front and no site is exempted "because the repo is early" — *assume anything left behind is duplicated/triplicated* (operator). The per-site remainder inventory is the census authority's job (below); this doc does not maintain a parallel copy.

## The thesis (one line)

The `.dag` intent may not name a target language. A workflow expresses *what it wants* as an ordinary `.dag` dependency graph over modeled operations; rendering that to bash — or PowerShell, or direct execution — is a separate, target-parametrized concern downstream. Bash appears exactly where Rust appears: its grammar spec and the emit rows, and nowhere in the intent.

This is not a new principle. It is §3 (single authority; a transport is one Realization handler of N, not a fact fused into intent) and §4 (one grammar read both directions; a new target is rows, N models not N×M) restated for the shell case.

## The correction that sharpened it (operator, 2026-07-17)

- **The intent IS the `.dag` graph — not a `Pipeline`.** §4 already says a program is a dependency graph over `Node` + `Edge`. "Get the git diff and feed it to X" is an operation node (typed I/O) with a dependency edge to its consumer. That graph *is* the intent. `src/v2/std/orchestration.dag` `Pipeline` is one *convenience framework* for writing linear-orchestration-shaped graphs — fine to keep as a writing aid, never the authority, never required. Do not elevate it to THE representation.
- **Emit and realization are two distinct downstream layers** (do not conflate — "realization" is the precise §2 concept):
  - **Emit** (§4): render the intent graph to a target's surface syntax, `emit(intent, Bash)` — one grammar read backward, a pure transform with the target as a *parameter*.
  - **Realization** (§2): pure-spec → host-effect, content-addressed, N transport handlers (`LocalShell`, `SshShell`, …). It realizes a typed effect on a host, and it is not emit. Per the routing decision below, *most* (runtime-present) sites realize **directly, with no emit in the loop**; only the foreign-executor/bootstrap path has realization *consume `emit`'s output* (the rendered bash). Emit is one input to realization on that path, not a universal precursor.

## The layering

| Layer | Owns | Names a language? | Carrier (today) |
|---|---|---|---|
| **Intent** | the `.dag` graph: modeled operations + data-dependency edges | **No** | the workflow's own `.dag` |
| **Interface** | each operation = its dependency's *semantics* (git's diff semantics), transport-agnostic | No | `extdeps/**` operation shapes |
| **Emit** | intent-graph → target surface syntax, target as parameter | Yes (the grammar) | `src/v2/extdeps/languages/bash.dag` + emit rows |
| **Realization** | pure-spec → host-effect, N transports | the transport, not the intent | `dag/gunbc/host_effect_realize.dag` |

`git diff` argv is a nickname for git's diff semantics; hardwiring the bash-CLI transport into the workflow *is* the §3/§4 N×M-adapter trap (libgit2, the GitHub compare API are the other handlers). The business policy ("which base ref") stays a parameter on the operation; the regression tell is an argv carrying `origin/main...HEAD` as a literal.

## The invariant (what makes it enforceable, not aspirational)

**The `.dag` intent may not import any language-construction vocabulary.** That vocab — `bash_build`, `bash_command_fold_serialize`, the `ShellStmt`/`ShellWord`/`ShellProgram` coproduct, `serialize_bash`, and the same for every future target — is emit-internal, so it may be imported *only* from realization-edge paths. `src/v2/lens/realization_vocabulary_containment.dag` enforces exactly that: any path **not** on the realization-edge whitelist — the intent layer included — reds if it imports the bash-AST vocab. So the invariant is **half-enforced today**: the *bash-AST-vocabulary* half is a live wall (**LANDED #6854**), while the *meta-exec* half (`shell.Exec.Run/.Check`) is still open. The single authoritative landed/open split is "The carriers and the two enforcement milestones" below — this paragraph states the invariant, not a second copy of that split. This wall is the natural home for the `StandingIntent` "ask once, compile forever" enforcement (see enforcement-intent thread).

## The same invariant, generalized — config is a language too (2026-07-24 CI audit)

The invariant above is stated for shell *strings*: the intent may not construct bash. A three-facet CI-workflow audit (2026-07-24) found the identical §3/§4 violation in a **second sink that isn't a string at all** — hand-spelled *config*. The intent names no language; the corollary the audit forces is sharper: **the emitter names no concept.**

**The second sink.** Beyond the `run:` bash strings (owned by the census, §4.I), the workflow emit hand-picks *typed config values* whose authority already exists elsewhere:

- **toolchain isolation** — `extdeps/toolchain/types.dag` models `ToolchainEnvIsolation = SharedHomeAcrossJobs | PerJobCargoHome{…} | HermeticContainer`, and `sccache`/`fleet_intent` *declare* their posture with it. But the workflow emit never reads `env_isolation`; it hand-writes "isolate the build job / don't isolate the ci job." The concurrent-job race (`rustup: Text file busy` / ETXTBSY) then lives in a **prose note** (`ci_floor_gate_toolchain_note`) instead of a modeled, gated state.
- same shape: the GHA **cache key** (modeled `gha_actions_cache_facts` key-derivation vs the hand-spelled `ci_cache_key_template`), the setup-rust **action version/components** (vs the modeled `ToolchainCapability`), and each job's **permissions** dict (vs `std.effect_grant`).

These are not shell strings, so they are **not** the census's sink and **not** dissolved by the containment wall above — but they are the *same disease*: a fact that a model owns, hand-copied into the realization. §3, one sink over.

**The litmus — "opaque to emission."** The test for whether a concept is modeled right is: *does the emitter have to know what it is?* Emit is a mechanical projection of the intent graph (§4, one grammar read backward) — it must carry **zero special knowledge** of "toolchain isolation" or "cache key." Those are projectable *facts* (nodes) the general emitter renders via grammar rows; the moment the emitter carries a *special case* for one of them — and it does, an "isolate this job" branch — the concept isn't a fact, it's forked emitter logic, and that special case **is** the violation. So the reviewer's question is the one this doc asks of shell, turned on config: *should this be opaque to emission?* If the emitter special-cases it, no.

**The bespoke emitter is itself the fork.** The sharpest form: there should be **one** `emit(intent, target)`, not a bespoke `ci_workflow.dag` / `ci_yaml_emit` that hand-projects the workflow. A second emitter that hand-writes the yaml structure + the config **is** the N×M-adapter trap at workflow scale — the same fork this doc names for a hardwired transport, one layer up. The end-state roadmap `2-emit-partition` already targets (YAML-the-language rows in `extdeps/languages/`, GHA-workflow as rows on top) *is* "the workflow is a modeled intent the one emitter projects." Config-opacity and the multi-target Python litmus (the corrections section below) are the same property twice: if the intent and its config are language-blind and concept-opaque, the target — bash, a `uses:` action, Python, direct execution — is a **parameter**, and the workflow emitter is not a place code lives.

**Enforcement (the wall, generalized).** The shell half is walled by `realization_vocabulary_containment` (bash-AST vocab) + `meta_exec_confinement` (`shell.Exec`). The config half needs its own construction wall: a workflow-emit that hand-writes a value for which a modeled authority exists — an `env_isolation` chosen without reading `ToolchainCapability`, a cache key not derived from `*_cache_facts` — should red. A lens over the workflow emit that requires config to be *derived, not literal* is the missing wall the audit names; it is what makes "opaque to emission" enforceable rather than aspirational, the same posture as this doc's invariant applied to the config sink.

*Sites are tracked in the census §4.I (the shell strings) with the config forks cross-referenced there; this section states only the principle, per this doc's no-second-census discipline.*

## This doc's scope within the arc — and where the rest lives

This doc owns exactly one facet of the shell→dag arc: the §3/§4 **invariant that the intent names no language**, and the containment wall that enforces it. The arc's census, routing decision, and target composition band are **owned by sibling authorities and must not be restated here** — restating them is the §3 parallel-ledger drift:

| Concern | Authority | Owner |
|---|---|---|
| Residual-shell census + arc-completion scoping (the site categories, per-site homing, the runtime-present-vs-legitimate-shell split) | [`shell-to-dag-residual-census-and-arc-completion.md`](shell-to-dag-residual-census-and-arc-completion.md) | witty-ibex-317 |
| When bash emission is authorized vs typed realization required (`ExecutorCapability` / `ProvisioningWindow` / `authorize_shell_emission`) | [`provisioning-window-executor-capability-design.md`](provisioning-window-executor-capability-design.md) | witty-ibex-317 |
| The target composition band — `std.effect_plan`, leaves = `Do{effect}` never a command string | [`host-effect-orchestration.md`](host-effect-orchestration.md) | host-effect lane |
| The bash-minimization rule (bash only for foreign executors + bootstrap) | [`shell-emission-model.md`](shell-emission-model.md) | signed 2026-07-03 |
| Grant / authorization model (deploy-preflight's real home) | [`effect-namespace-grants.md`](effect-namespace-grants.md), `std.effect_grant` | silent-ibex-417 |

## The routing decision — most sites do NOT emit bash

The load-bearing correction (operator review, 2026-07-19): **"dissolve the raw concat" is not "wrap it in `emit(intent, Bash)`."** Two realization paths exist, selected per-site by executor capability / provisioning window (the provisioning-window authority):

- **Runtime-present** (a gunbc runtime / typed transport is on the path): the effect realizes as a **typed service operation, typed transport, or binary-interpreted effect-plan via `host_effect_apply`** — *no bash is emitted*. This is the majority. Precedent already in tree: `floor_diff_observe.dag` calls `git.Core.DiffUnified0(...)` directly; `host_identity_converge.dag` funnels through `host_effect_apply`.
- **Legitimate shell** (foreign executors — GHA `run:`, cron entry lines, git-hook files — plus bootstrap windows before any runtime exists on the path): renders through the **v2 bash grammar rows**. Bounded by a roster, not a growth surface.

So the arc is not "N files onto the bash emitter." It is: route each site to the correct path, and generalize `host_effect_apply` from `ShellCommand{script:String}` toward typed effects — the census authority's central finding.

## The carriers and the two enforcement milestones (corrected)

The anemic `String` leaves are **more than one**: `host_effect.ShellCommand{script}` (`host_effect.dag:25`), `std.orchestration.Run.command` (the `Do{run}` leaf), and `host_effect.BootstrapFragment.script` (`host_effect.dag:59`) — plus the `ThinInvocation`/`EmitArtifactThenThinRun` medium-as-string scaffolds. The target for all of them is **`Do{effect}` — never a command string** (host-effect-orchestration.md's `std.effect_plan` band). Sequencing is **add-replacement → migrate all consumers → delete at arc close** (§5 construction, but you cannot delete `ShellCommand` while other constructors and consumers still populate it). `host_effect.dag` is a DESIGN-named load-bearing carrier — the delete is escalated/co-designed, not improvised.

Two enforcement milestones, kept distinct — my earlier draft conflated them:

- **LANDED — bash-AST-vocab containment (#6854).** `realization_vocabulary_containment` reds any intent-layer import of `bash_build` / `ShellStmt` / `serialize_bash`. This confines the *language-AST vocabulary* only; it does **not** recognize `extdeps.shell` or forbid `shell.Exec`.
- **PARTIAL — meta-exec confinement (Wave A1 import wall LANDED #6900; construction wall OPEN).** `shell.Exec` is isolated in `extdeps/shell/exec.dag`, and `v2.lens.meta_exec_confinement` enforces the module-granular import wall. Its exception roster has since shrunk **3 → 0**, so a new intent-layer import outside the declared realization-edge prefixes is a typed leak with no exception to grant. That is validation over the import graph, not the terminal construction wall: `TransportScript` remains a transparent brand over `String`, and direct `shell.Exec.Run` callers on an allowed realization edge can still construct computed/cast inputs. `host_language_transport_script` is live but deliberately green on `ComputedApplication`; it backstops raw literals rather than making the cast path unwritable. Closing that surviving path is the DESIGN-named meta-exec-confinement milestone, coupled to reference/import visibility.

### Import-visibility for meta-exec confinement (operator, 2026-07-19; Wave A1 import wall landed, construction wall open)

`shell.Exec.Run` is the honest bottom bash transport — kept, not deleted — but it must be **heavily sequestered** so its only caller is realization and its input is always `emit`'s output. Wave A1's import-graph analysis (`LayerImportFact`) confines the isolated module and now has an empty exception roster. It does **not** constrain how an allowed realization-edge caller obtains a `TransportScript`; because the brand is transparent, a computed `String as TransportScript` remains writable there. Two ways were proposed to sequester the operation; the first landed the import half only:

- **(a) Isolate the bottom transport into its own module** (`extdeps/shell/exec.dag`) → the existing module-granular wall confines imports. **Landed (Wave A1), necessary but not sufficient for construction confinement.**
- **(b) Add symbol/reference-level visibility** so the bottom transport can be callable only from the emit-owned realization seam — the general feature, converging with the namespace-only-resolution lane (references become `container.member`). This is the still-open construction-confinement direction; do not infer its completion from the empty import-exception roster.

## The access/auth exemplar (deploy-preflight) — a grant-model consumer, contributed to its lane

deploy-preflight (`ci_deploy_access.dag`) is the natural first exemplar, reframed at the operator's direction (2026-07-19): express it as an **authorization query**, not a shell script. The intent is *"the deploy frame requires the current principal to hold grants {execute-no-passwd hostnamectl-set-hostname, execute-no-passwd apt-get-install, …} and to be the expected principal."* It grounds on vocabulary that already exists — `GroundedPosixPrincipal.sudo_grants` + `std.effect_grant.Grant` (silent-ibex-417's P-A model) — and **earns the `Execute` verb** that `std.effect_grant.Verb` currently defers ("Execute/Create arrive as later rows only when a displaced cost names one" — deploy-preflight is that cost). `PosixSudoGrant` becomes the POSIX *materialization* of a general `Grant`, resolving a §3 fork rather than minting a parallel grant vocab.

Because deploy-preflight is **runtime-present** (it runs on srv1 via `shell.Exec.Run` from inside `gunbc run`), it realizes as typed effect-plan through `host_effect_apply` — **not** bash emit. That dissolves the machinery objection: it needs no `For`/capture *emission* (both currently `outcome_rejected` in `05_emit_orchestration`), because the fold-over-commands becomes a typed map and the `whoami` comparison a typed predicate over captured values. Any inequality predicate is `Not{StrEq}` (reuse), never a minted `StrNeq`.

This work belongs to the grant lane (silent-ibex-417) and the census/arc lane (witty-ibex-317) — it is contributed there and coordinated, not landed as a parallel plan here.

## Corrections carried out of the 2026-07-19 draft (for the record)

The first version of this section (PR #6897) over-reached; operator review corrected it. These are the standing corrections, so the errors are not re-copied:

- **No everything-emits-bash.** The routing decision (above) supersedes any implication that operations traverse `emit(…, Bash)`. Runtime-present is the majority path, and it is typed realization, not emission.
- **No disguised coreutils catalogue.** `cat` (read-file) is `Filesystem.Read`; `wc`/`tr`/`cut`/`printf` are pure `std` text/collection over values, **not** shell dependencies — DFS onto those authorities before minting anything (DESIGN §2). Only genuinely-external OS/tool reads (systemctl, apt, git, curl, and the identity/clock reads) are extdeps operations, and most already exist. There is **no** `extdeps/shell/text.dag`.
- **Python is a modeling litmus, not an executed column.** Operations hold *one* optional transport and dispatch selects one operation transport (or one service fallback); a real second-transport column would require multi-handler dispatch as an explicit prerequisite. "Emit to Python without touching the intent" stays a *thought-test* on shape agnosticism. (The sloppy examples were also wrong: `getpass.getuser()` is not `whoami`'s effective-UID semantics, and default `datetime.isoformat()` does not produce fixed-second `Z`.)
- **One quoting/join authority (the one §5 catch worth keeping).** `join(argv, " ")` (`live_deploy/operations.dag:15`) is a naive space-join with **no shell quoting** — a latent §5 correctness hole. One cited POSIX shell-word-quoting/join authority should absorb it and the scattered `git_shell_join_argv` / `shell_single_quote`. Contributed to the extdeps/census lane; emit-internal, never in the intent.

## Non-goals

- Not a rewrite of the substrate — the intent representation already exists (the `.dag` graph); this removes shell *from* it.
- Not deleting `Pipeline` — it stays as optional sugar; it just stops being load-bearing.
- Not deleting `shell.Exec.Run` — it is the legitimate bottom bash transport; it is *confined* to realization, not removed.
- **Not a second census** — the residual inventory is owned by the census authority above; this doc references it and does not maintain a parallel copy.

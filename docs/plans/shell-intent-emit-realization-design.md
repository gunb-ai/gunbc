# Shell → intent: the workflow layer is language-blind

**Status:** design, operator-aligned 2026-07-17. Anchors the rescope of the shell→dag arc onto §3/§4. The sidecar delete (bash `ShellStmt`/`program.dag` removal) is *Phase 0* of this, not the whole story.

## The thesis (one line)

The `.dag` intent may not name a target language. A workflow expresses *what it wants* as an ordinary `.dag` dependency graph over modeled operations; rendering that to bash — or PowerShell, or direct execution — is a separate, target-parametrized concern downstream. Bash appears exactly where Rust appears: its grammar spec and the emit rows, and nowhere in the intent.

This is not a new principle. It is §3 (single authority; a transport is one Realization handler of N, not a fact fused into intent) and §4 (one grammar read both directions; a new target is rows, N models not N×M) restated for the shell case.

## The correction that sharpened it (operator, 2026-07-17)

- **The intent IS the `.dag` graph — not a `Pipeline`.** §4 already says a program is a dependency graph over `Node` + `Edge`. "Get the git diff and feed it to X" is an operation node (typed I/O) with a dependency edge to its consumer. That graph *is* the intent. `src/v2/std/orchestration.dag` `Pipeline` is one *convenience framework* for writing linear-orchestration-shaped graphs — fine to keep as a writing aid, never the authority, never required. Do not elevate it to THE representation.
- **Emit and realization are two distinct downstream layers** (do not conflate — "realization" is the precise §2 concept):
  - **Emit** (§4): render the intent graph to a target's surface syntax, `emit(intent, Bash)` — one grammar read backward, a pure transform with the target as a *parameter*.
  - **Realization** (§2): pure-spec → host-effect, content-addressed, N transport handlers (`LocalShell`, `SshShell`, …). It *consumes* emit's output to produce an effect on a host. It is not emit.

## The layering

| Layer | Owns | Names a language? | Carrier (today) |
|---|---|---|---|
| **Intent** | the `.dag` graph: modeled operations + data-dependency edges | **No** | the workflow's own `.dag` |
| **Interface** | each operation = its dependency's *semantics* (git's diff semantics), transport-agnostic | No | `extdeps/**` operation shapes |
| **Emit** | intent-graph → target surface syntax, target as parameter | Yes (the grammar) | `src/v2/extdeps/languages/bash.dag` + emit rows |
| **Realization** | pure-spec → host-effect, N transports | the transport, not the intent | `dag/gunbc/host_effect_realize.dag` |

`git diff` argv is a nickname for git's diff semantics; hardwiring the bash-CLI transport into the workflow *is* the §3/§4 N×M-adapter trap (libgit2, the GitHub compare API are the other handlers). The business policy ("which base ref") stays a parameter on the operation; the regression tell is an argv carrying `origin/main...HEAD` as a literal.

## The invariant (what makes it enforceable, not aspirational)

**The `.dag` intent may not import any language-construction vocabulary.** That vocab — `bash_build`, `bash_command_fold_serialize`, the `ShellStmt`/`ShellWord`/`ShellProgram` coproduct, `serialize_bash`, and the same for every future target — is emit-internal. This is `src/v2/lens/realization_vocabulary_containment.dag` **generalized**: today it confines the language-AST vocab to realization-edge paths; the target invariant extends it so the *workflow/intent layer* is in scope and importing any language vocab reds. "Shell conforms to §3/§4" is a precise, executable statement: *the containment wall is green with the intent layer in scope.* This is the natural home for the `StandingIntent` "ask once, compile forever" enforcement (see enforcement-intent thread).

## The residual (what's actually still there)

Two classes of shell-in-intent survive, and the arc so far addressed only part of the first-cousin of them:

1. **Raw string concat** — functions returning `String` built by nested `concat(...)` of shell fragments. Canonical: `dag/gunbc/ci_spec.dag` `gunbc_ci_deploy_invoke` builds `"$ROOT/target/release/gunbc" run <flags> --entry <e> --function <f>\n` by hand. Never touched a structured AST. **This is the class the operator flagged 2026-07-17; the `bash_build` migration did not touch it.**
2. **Structured-but-still-bash** — code constructing bash via `bash_build` Node constructors or the `ShellStmt` coproduct and serializing (`bash_command_fold_serialize`, `bash_fold_serialize_program`). Better than (1) — analyzable, cannot smuggle arbitrary text — but still bash-in-the-workflow. Canonical: `src/v2/workflow/floor_diff_observe.dag` `floor_git_diff_unified_stmts` / `floor_serialize_program`.

### Census (2026-07-17)

Swept both `.dag` roots (2169 files) by six angles (`concat(`, `bash_build`, serialize, `ShellStmt`, literal shell tokens, `*_shell`/`*_flags`/`*_script`/`*_argv` names), body-filtered to genuine shell-execution (excluding `.dag`'s own `&&`/`||` operators). Counts defensible to ~±10% on the long tail.

- **Class 1 + 3 (raw `String` shell builders — hand-assembled scripts / argv / flags): ~110 functions across ~50 files.** This is the class the operator flagged; the `bash_build` migration never touched it.
- **Class 2 (structured-but-still-bash — `bash_build`/`ShellStmt` + serialize): ~9 construction sites.**
- Excluded as legitimate (grammar spec + emit rows + the `shell_bash_runner` execution edge + `05_emit_orchestration`): ~14 files. Not residual.

By layer — the residual concentrates in the intent/workflow layer, exactly as the diagnosis predicts:

| Root | Files | Fns | Contents |
|---|---|---|---|
| `dag/gunbc/**` | ~35 | ~87 | the bulk: CI spec, merge-admission, deploy, fleet, `live_deploy`, host-effect, roadmap-site |
| `dag/tools/**` | ~12 | ~23 | behavioral-transport harnesses, build-step/host prelude, compile-clean transports |
| `src/v2/workflow/**` | ~7 | ~10 | `ci_workflow_run_emit` (partially migrated), floor-diff-observe, ingest/survey transports |
| `dag/extdeps/{os,tools,bmc}/**` | — | ~7 | extern-tool wrappers (curl, apt, systemd, ISO) — see policy note below |
| `dag/std`, `src/v2/lens`, `src/v2/compiler` | — | 0 | zero true offenders |

The ~110 functions **collapse to ~10 operation families** — model each once (§3 shape + bash transport) and its callers become intent graphs over it. This is the key scope fact: it is *not* 110 independent problems.

1. **gunbc-run / claim-executor invocations** (`gunbc_ci_deploy_invoke`, `claim_executor_bin_shell`, `scheduler_invoke*`)
2. **witness / source-root flag assembly** (`witness_layer_source_flags_rooted` + siblings; `dcc_*_args`, `*_manifest_emit_args`)
3. **git ops** (`git_fetch_script`, `ci_merge_base_diff_range`, `merge_target_tree_hash`)
4. **CI floor / retry / cargo-build / rustup / tar-pack** (`ci_cargo_eagain_retry_script`, `ci_release_bins_pack_script`, `ci_pin_rustup_default_command`)
5. **deploy / access preflight** (sudo + `apt-get install`)
6. **live_deploy** (systemd unit write + apt ensure)
7. **install / provisioning** (ISO fetch/remaster, hostname CAS, nbd-proxy transient units)
8. **systemd effective read-back** (`systemctl show …`)
9. **githooks emission** (pre-commit / pre-push bodies)
10. **HTTP / curl probes** (roadmap-site readback, language smokes)

Per-file hotspots: `ci_spec.dag` (14), `live_deploy/emit.dag` (7), `fleet_show_effective_read.dag` (7), `merge_admission_produce.dag` (6), `host_effect_nbd_proxy_serve.dag` (6, class-2), `ci_workflow_run_emit.dag` (4, partially migrated), `fleet_converge_emit.dag` (4), `ci_deploy_access.dag` (4).

**Policy note — the `dag/extdeps/**` wrappers (~7 fns):** these build shell for external tools (curl, apt, systemd, ISO tooling). Per §3 they are *legitimate as transport handlers* — the bash-CLI realization of that tool's operation — **provided** they are structured as one-of-N handlers bound to an agnostic operation shape, not the single fused transport. Where a wrapper is the only hardwired transport with policy literals baked into its argv, it is residual (the N×M trap); where it is a declared handler under a modeled operation, it is fine. This is a per-wrapper §3 call during Phase 2, not a blanket verdict.

Legitimate and **excluded** from the residual: `src/v2/extdeps/languages/bash.dag` (the grammar spec — the Rust analog) and the emit rows. Their existence is correct.

## Rescoped phases

The headline is **"no language in the intent,"** with the sidecar delete demoted to Phase 0.

0. **Sidecar delete** (in flight — PR-A/PR-B merged, PR-C migrate-last-consumers + PR-D trivial-delete queued). Removes the `serialize_bash`/`RawLine` string-smuggling vector and consolidates bash on the one grammar. Necessary, not sufficient.
1. **Complete the agnostic emit.** `src/v2/std/orchestration.dag` emit has holes (bounded-poll/`While` rejected, `Retry` hardcoded to two levels) — the gap that pushed workflows to reach for `bash_build`. Close it so the intent graph can express what workflows need.
2. **Model the operations with transports.** `git.diff`, the `gunbc run` invocation, unit upsert, package fetch, readiness-poll — each a §3 operation shape in `extdeps/**` with the bash-CLI as one handler.
3. **Migrate the intent off shell.** Rewrite `gunbc_ci_deploy_invoke`, `floor_diff_observe`, `build_step_emit`, `live_deploy`, CI to build modeled-operation graphs; `bash_build` and raw `concat`-shell vanish from the intent layer.
4. **Wall green with the intent layer in scope** — the generalized `realization_vocabulary_containment`, as the standing acceptance criterion throughout.

**Flagship:** `git.diff` via `floor_diff_observe` — small, load-bearing, already in hand — proves intent → emit → realization end-to-end before generalizing.

## Coupled lane

`dag/gunbc/host_effect.dag` `HostEffect` still has the bash-shaped hole `ShellCommand{script}`: in the target, that variant becomes "run this operation," with bash-rendering downstream — the same move one layer down. Align the converge/membership work (nimble-eagle lane) to this shape rather than deepening `ShellCommand{script}`.

## Non-goals

- Not a rewrite of the substrate — the intent representation already exists (the `.dag` graph); this removes shell *from* it.
- Not deleting `Pipeline` — it stays as optional sugar; it just stops being load-bearing.
- Not one big PR — staged, each phase green-by-execution, priced by the shell-in-intent it removes.

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

<!-- CENSUS: filled from the 2026-07-17 residual census — counts per class, per layer, per operation. -->

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

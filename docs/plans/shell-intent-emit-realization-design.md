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

## The subatomic standard: operations are semantics, not commands

The operator's litmus (2026-07-19): **"could I emit this same intent to Python tomorrow without touching the intent graph?"** That single test fixes the modeling depth, and it rejects the shallow reading of "model the shell library" (catalogue bash *commands*). The atom is **not the binary** — it is the *semantics the binary realizes*:

- `whoami` is not the atom. **"read the current user identity"** is; `whoami` is its *POSIX-shell transport*, `getpass.getuser()` its Python transport, a syscall its direct-execution transport.
- `date -u +%Y-%m-%dT%H:%M:%SZ` → **"read current time (UTC, ISO-8601)"** (already homed: `extdeps/clock/clock.dag`); bash `date`, Python `datetime.now(UTC).isoformat()`.
- `hostname -s` → **"read short host identity"**; bash `hostname -s`, Python `socket.gethostname()`.
- `[ "$x" != "$y" ]` → a **`Predicate` node** (`StrNeq`) whose operands are captured values — it renders to *neither* bash `[ ]` nor Python `!=` until emit.

So the catalogue in `extdeps/**` is a set of **operation shapes, each with 1..N transport handlers**, and bash is one column. The python test = *a second transport column can be added without editing the intent or the shape.* An operation whose shape **can only be read as bash** — a baked `$(…)`, a `2>/dev/null`, an `sh -c` string — fails the test by construction: it cannot grow a Python column. Those are exactly the raw-concat residue this arc dissolves.

### The three-layer decomposition (what "subatomic" means per script)

Every raw shell script fuses **three separable concerns**; the migration un-fuses each onto the layer that owns it (this is DESIGN §3's interface/transport/policy triple, and the intent/emit/realization layering above, applied *inside* a script):

1. **Operations (the command leaves)** → one modeled operation shape each in `extdeps/**`, by semantics, bash as one bound transport. `systemctl show`, `apt-get install`, `curl`, `whoami`, `printf`.
2. **Data flow / command substitution** (`X=$(…)`, `$(…)` inline, pipes feeding values) → **dependency edges** in the intent graph: an operation's captured output bound to a value, consumed downstream. Never a `$(…)` string; the *edge* is the substitution, and emit renders the substitution from the bash grammar.
3. **Control flow** (`if`/`while`/`case`/`&&`/`||`/`exit`/`trap`) → **orchestration intent nodes** (`If`/`While`/`Do`/`Predicate`/`Retry`), already modeled in `std.orchestration` and already emittable (Phase 1). Predicate operands are captured reads + typed comparisons, not `[ … ]` strings.

The subatomic depth is thus *(operations × data-edges × control-nodes)*, each operation itself decomposed to its cited semantic atom. When all three are modeled, the carrier no longer needs a `String` — which is the root fix.

## The root fix — make raw shell unrepresentable (construction, not validation)

The whole arc converges on **two `String` carriers**; dissolving them is what makes the residue *unwritable* (§5 construction over validation) rather than merely swept:

| Carrier | Layer | Fix | Why differs |
|---|---|---|---|
| `host_effect.HostEffect.ShellCommand { script: String }` (`host_effect.dag:25`) | **intent** | **Delete the variant.** Replace with an effect that carries a typed intent — the fleet-reconcile/`ConvergePlan { policy }` precedent in the same coproduct proves a typed effect works here. Bash rendering moves *inside* realization (`emit(intent, Bash)` → the string handed to the transport). | It sits in the intent layer; a `String` here is the duplication vector. Once the variant is gone, no caller can construct a hand-concat program — the bad state is unwritable. |
| `shell.Exec.Run(script: String)` / `.Check(command: String)` (`extdeps/shell/shell.dag`) | **realization (bottom transport)** | **Keep, but confine.** This is the legitimate bash transport primitive — *someone* must eventually hand bash a string to run. Its `script` argument must only ever be the output of `emit`, never a hand-concat, and the intent layer may not import it. | It is already in `extdeps` (correct layer). The wall (below) confines it to realization; deleting it would mean bash could never run at all. |

The one path from intent to a bash string is therefore **through emit**. `shell.Exec.Run` staying `String`-typed is not a hole once (a) the intent can't reach it and (b) `HostEffect` has no `String`-carrying variant — the two conditions the wall + the variant-delete establish together.

**Note the single-transport-per-operation constraint** (`extdeps/shell/shell.dag`, cited to `execve.2.html`): an operation binds exactly one `transport shell { argv }`, which is *why* `shell.Exec` is already forked into `Run` (`bash -s` + stdin, avoids `MAX_ARG_STRLEN`) vs `Check` (`sh -c {command}`, one argv token, hits `E2BIG`). Modeled operations inherit this — the `Run`/`Check` split is a realization detail selected downstream of emit, not an intent choice.

## The extdeps shell-library catalogue (the subatomic layer)

The census found **no cited coreutils/POSIX *operation* catalogue** — `extdeps/shell/gnu_coreutils.dag` is a 26-line stub (flag constants only), and `whoami`/`hostname`/`wc`/`tr`/`printf`/`id`/`uname`/`cat`/`head`/`base64` are modeled nowhere. So the subatomic layer is real greenfield modeling, organized **by semantic domain** (not by binary — the python test):

| Semantic domain | Operations (semantics) | bash transport | Python column (proof of agnosticism) | Home |
|---|---|---|---|---|
| **identity** | current-user, host-identity, os/arch | `whoami`/`id -u`, `hostname -s`, `uname` | `getpass.getuser()`, `socket.gethostname()`, `platform.*` | **new** `extdeps/os/identity.dag` |
| **clock** | current-time (UTC ISO-8601), unix-secs | `date -u +…` | `datetime.now(UTC)` | reuse `extdeps/clock/clock.dag` |
| **text** | line-count, translate/strip, field-cut, format-line, read-file | `wc -l`, `tr`, `cut`, `printf`, `cat` | `len(...)`, `str.translate`, split, f-string, `open().read()` | **new** `extdeps/shell/text.dag` |
| **predicate/test** | file-exists, is-executable, str-eq/neq, numeric-cmp | `test -f`/`[ … ]` | `os.path.exists`, `==` | reuse `shell.Test`, extend |
| **process** | which/available, background+wait, trap-cleanup, exec | `command -v`, `&`+`trap`, `exec` | `shutil.which`, `subprocess.Popen`, `atexit` | **new** `extdeps/os/process.dag` |
| **systemd** | show-property, set-property, list-units, enable/restart/… | `systemctl …` | dbus | reuse `extdeps/os/systemctl.dag` (extend `ShowProperty`/`SetProperty`/`ListUnits`) |
| **package** | install, status | `apt-get`, `dpkg -s` | apt python bindings | reuse `extdeps/package_managers/{apt,dpkg}.dag` |
| **git** | fetch, show, diff-range, toplevel | `git …` | pygit2 / compare API | reuse `extdeps/git/git.dag` (extend) |
| **http** | bounded GET (timeout/connect), http-code probe | `curl -sf …` | `httpx`/`requests` | **promote** `extdeps/tools/curl.dag` from raw-flag-concat to an operation |
| **archive** | pack/unpack tarball | `tar -czf`/`-xzf` | `tarfile` | **new** `extdeps/tools/tar.dag` |
| **filter** | fixed-string match, in-place-substitute, json-extract, sha256-check | `grep -qF`, `sed -i`, `jq -r`, `sha256sum` | `in`, `re.sub`, `json`, `hashlib` | reuse `extdeps/tools/{grep,sed,jq,sha256sum}.dag` (extend) |

**One quoting/join authority (§3 + §5).** Today the argv→string boundary is scattered and unsafe: `live_deploy/operations.dag:15` does `join(argv, " ")` (**naive, no shell quoting** — a latent §5 correctness hole), git has its own `git_shell_join_argv`, and the only quoting fn is `shell_single_quote` in `extdeps/github/log_annotations.dag`. The plan lifts **one cited POSIX shell-word-quoting + argv-join authority** into `extdeps/shell/` (authority: POSIX `V3_chap02.html`, already cited by `shell.dag`); every argv→string projection routes through it, and the naive space-joins are dissolved onto it. This is emit-internal — it renders inside the bash grammar, never in the intent.

**Command substitution** (`$(whoami)`, `X=$(…)`, pipes-into-values) becomes a **captured-read edge** in the intent (operation output bound to a value, consumed downstream) and is rendered as `$(…)` by the bash grammar at emit. Modeling gap to name: confirm `std.orchestration` supports *bind operation output to a value and reference it in a later predicate/operation* (a value-capture node); if not, that node is the first modeling task of Tier 0.

## The remainder catalogue (exhaustive census, 2026-07-19)

Three parallel sweeps covered every `.dag` root (`dag/gunbc/**` ~230 files, `dag/tools/**` + `src/v2/**`, `dag/extdeps/**`). Every executor and every raw-concat builder is homed. **No site is exempted.**

**Totals:** ~**28 executor sites** in `dag/gunbc` (13 `shell.Exec.Run`, 1 `.Check`, 9 `ssh_session_exec`, 10 `ShellCommand{script}` constructions feeding `host_effect_apply`) + **5 non-fixture executors** in `tools`/`src/v2`; ~**70 raw-concat builder fns** across 19 `dag/gunbc` files + ~**22** in `tools`/`src/v2` + ~**6** in `extdeps`. Nearly every raw-concat cluster **already carries a `Scaffold` + 🟡 dissolve-on trigger** whose declared terminus is exactly this arc (`shell_bash_runner_dissolution_trigger`, `live_deploy_emit_shell_dissolution_trigger`, `srv3_host_effect_script_dissolution_trigger` (Wave-4 §1C), `cli_invoke_shell_spelling_dissolve_trigger`, the two `deploy_*_preflight_shell_disposition` rows, `host_build_cache_provision_script_dissolution_trigger`, `roadmap_site_readback_script_scaffold`, …). The migration *fires* those triggers.

Per operation family (each collapses many sites to one modeled shape):

| # | Family | Representative sites | Decomposes into | Homes at |
|---|---|---|---|---|
| 1 | **gunbc-run / claim-executor invocation** | `cli_invoke.dag` (9), `ci_spec.dag` (~16), `merge_admission_produce.dag` (~9), `falsifier_workflow.dag` (2), `fleet_converge_emit.dag` (2) | already-typed `*_transport_argv` (good) + the `$ROOT`-path + flag *string* wrapper (raw). The argv exists; only the string projection is raw. | reuse `cli_services.dag` `gunbc.Cli`/`claim_executor.Executor` service ops; delete the `_shell` string wrappers |
| 2 | **git ops** | `ci_spec` (`git_fetch_script`, `ci_merge_base_diff_range`, `ci_repo_root_shell`), `merge_admission_produce` (`git_fetch_script_for_gate`), extdeps `git_*_shell` fragments | fetch / show-tree / toplevel / diff-range operations; the `2>/dev/null || true` / `$(…)` decoration → control-flow + capture | extend `extdeps/git/git.dag` ops; decoration → orchestration nodes |
| 3 | **cargo / rustup / tar / sccache (CI standup)** | `ci_release_build_emit.dag`, `ci_workflow_run_emit.dag` (~10 leaves), `ci_spec` (`ci_release_bins_pack_script`), `ci_materialization.dag` (`ci_sccache_provider_shell_injection`) | cargo-build/fmt (reuse `cargo.Build.*`), rustup-default, tar pack/unpack, `echo … >> $GITHUB_ENV` (env-append op) — leaves inside an already-typed `Run`/`Pipeline` skeleton | reuse `extdeps/rust/cargo_build.dag`; new `extdeps/tools/tar.dag`, `extdeps/rust/rustup.dag`; env-append is a modeled GHA op |
| 4 | **deploy / access preflight** ⭐ | `ci_deploy_access.dag` (`deploy_access_preflight_script`, `_check_block`, `_sudo_check_cmd`), `ci_deploy_target_host.dag` (`deploy_target_host_preflight_block`, `_observed_hostname`) | whoami/hostname/printenv reads (capture) + sudo-wrapper + `if [ … ]`/`grep -Eq` predicates + `echo`→typed receipts + fold-over-commands | identity ops (new) + `sudo` privilege-wrapper + `std.orchestration` If/Predicate + receipt effects |
| 5 | **live_deploy apply/retract** | `live_deploy/emit.dag` (~9 Run-leaf fragments: heredoc `cat >` unit/server.js, `tailscale serve`, `install -m`, `rm -f`), `operations.dag` (`privileged_command` sudo prefix), `intent.dag`/`readiness.dag` (curl) | file-write (reuse `Filesystem.Write`), systemd unit upsert, tailscale-serve op, install/rm ops, curl health-probe; heredoc payload → typed file content | reuse `Filesystem`; new `extdeps/tools/tailscale.dag`, `extdeps/os/install.dag`; curl → http op |
| 6 | **systemd effective read-back** | `fleet_show_effective_read.dag` (4: `systemctl show/list-units \| wc \| tr`), `host_converge_slice1.dag` (memory-max read/set) | `ShowProperty`/`ListUnits` ops (extend systemctl) + `wc -l`/`tr -d` text ops + capture | reuse+extend `extdeps/os/systemctl.dag`; text ops (new) |
| 7 | **srv3 / BMC install diagnostics** (hardest probes) | `srv3_install_diagnostic_observe_script.dag` (6: curl/redfish/`jq`/`pgrep`/`ss`/`ps`/`sha256sum`), `srv3_host_effect_script.dag`, `srv3_os_install_reconcile_receipt.dag` (approval record + `date`) | http-code probe, redfish GET+jq, process-presence (`pgrep`/`ss`/`ps`), sha256-check, `date` capture, printf receipts — a 5-surface observation | new `extdeps/os/process.dag` (pgrep/ss/ps), extend curl/jq/sha256sum; receipts → typed |
| 8 | **host-effect receipts / identity** | `host_identity_adopt.dag`/`_assimilation.dag` (`echo` receipts), `host_identity_observation.dag` (hostname), `srv3_install_diagnostic_checklist.dag` (`echo_body`) | `echo "…"` → typed receipt-emission effect; hostname → identity read | receipt effect (typed, not `echo`); identity op |
| 9 | **roadmap-site read-back** | `roadmap_static_site.dag` (6: curl \| `node -e` digest, `[ = ]`, `date` receipt) | http GET (capture) + fnv1a digest (the `node -e` one-liner → a modeled digest op, not embedded JS) + str-eq predicate + receipt | http op + `std.content_hash` (the fnv1a authority already exists) + orchestration |
| 10 | **githooks emission** | `githooks_pre_commit_emit.dag` (staged-scan `while`/procsub, `cargo fmt`), `githooks_pre_push_emit.dag` (`case`/`ensure_bins`/`exec`) | These *emit a bash file* as a committed artifact — legitimately bash-targeted, but built by raw concat. Model as orchestration intent → `emit(intent, Bash)` (the hook body is the emitter's output, not hand-concat) | `std.orchestration` + bash emit; the artifact stays bash, the *construction* becomes typed |
| 11 | **toolchain provisioning** (hardest, own tier) | `emit_host_transport.dag` (go/ts/node smokes: `curl \| tar` tarball fetch, heredoc program bodies, `PORT=$((…$$…))`, backgrounded `node &` + `trap … EXIT`), `host_build_cache_provision_script.dag` (sccache ensure + heredoc unit) | tarball-fetch op, process background/wait/trap (process domain), heredoc payload → typed file content, arithmetic port-alloc → modeled | new `extdeps/os/process.dag` + `extdeps/tools/tar.dag`; the deepest control-flow surface |

Two special cases to note: `tools/review.dag` (dev tool, inline `shell.Exec.Run` string literals — cd/git-show) and `assimilate/bmc_token_federation.dag` (`gcloud`/`echo` leaves inside a typed `Pipeline`). Neither is exempt.

## Sequencing — exemplar-first, tiered by difficulty, each tier priced

Phases 0/1/4 from the original rescope **have landed** (#6831 sidecar delete, #6832 agnostic `While`/`BoundedPoll`/`Retry` emit, #6854 the generalized containment wall). What follows expands the old "Phase 2/3" into concrete tiers, each priced by the raw-concat count it zeroes and the scaffolds it fires. **Nothing here starts before the operator co-designs the Tier-0 exemplar pattern.**

- **Tier 0 — the root fix + the exemplar (co-design first).** (a) Confirm/add the value-capture node in `std.orchestration`; (b) nail **deploy-preflight `ci_deploy_access.dag`** end-to-end as the canonical reference — it exercises *every* hard case at once (coreutils read `whoami`, privilege `sudo`, control-flow `if [ … ]`, command-sub `$(whoami)`, `echo`→typed receipt, `fold`-over-commands iteration), and is already `Scaffold`-marked. Prove intent → `emit(…, Bash)` → realization green-by-execution with a discriminating RED. This establishes the pattern before any propagation. *(Also delete the `HostEffect.ShellCommand{script:String}` variant once one typed effect replaces it — the construction wall.)*
- **Tier 1 — the coreutils / semantic-op catalogue.** Stand up `extdeps/os/identity.dag`, `extdeps/shell/text.dag`, `extdeps/os/process.dag`, the one quoting/join authority; each op cited, each with the bash column and a *sketched* Python column (the python-test acceptance, executed as a golden). Prices: unblocks Tiers 4–7.
- **Tier 2 — CI standup leaves (family 3).** Orchestration skeleton already typed; only leaf command *strings* remain. Cheapest propagation. Zeroes ~10 raw leaves in `ci_workflow_run_emit`/`ci_floor_peak_emit`/`ci_release_build_emit`.
- **Tier 3 — systemd read-back (family 6).** `fleet_show_effective_read`, `host_converge_slice1`; `systemctl show`/`ShowProperty` mostly modeled. Fires `fleet_show_effective_read_transport_script_scaffold`, `host_converge_slice1_transport_script_scaffold`.
- **Tier 4 — gunbc-run / git invocations (families 1–2).** Highest *count*, single shape — the `_transport_argv` already exists; delete the `_shell` string wrappers, route callers through `cli_services` service ops. Fires `cli_invoke_shell_spelling_dissolve_trigger` and the `ci_spec`/`merge_admission` scaffolds.
- **Tier 5 — live_deploy + receipts (families 5, 8).** systemd unit upsert, tailscale, install/rm, health-probe curl, `echo`→typed receipts. Fires `live_deploy_emit_shell_dissolution_trigger` (the shared bind target).
- **Tier 6 — probes + read-back + hooks (families 7, 9, 10).** srv3 5-surface diagnostics, roadmap-site digest read-back, githooks emission. Needs the process domain from Tier 1.
- **Tier 7 — toolchain provisioning (family 11).** The deepest control-flow (background/trap/heredoc-payload/arithmetic). Last, because it stresses the process domain hardest.

**Acceptance per tier:** the containment wall stays green with the intent layer in scope; each migrated site is green-by-execution with a discriminating RED; the tier's scaffold rows are *deleted* (not re-marked); and the raw-concat count for that family reaches zero. **The arc closes** when the `HostEffect.ShellCommand{script:String}` variant is gone, `shell.Exec.Run/.Check` is import-unreachable from intent, and the raw-concat census is empty.

## Coupled lane

`dag/gunbc/host_effect.dag` `HostEffect` still has the bash-shaped hole `ShellCommand{script}`: in the target, that variant becomes "run this operation," with bash-rendering downstream — the same move one layer down. Align the converge/membership work (nimble-eagle lane) to this shape rather than deepening `ShellCommand{script}`. `host_effect.dag` is a DESIGN-named load-bearing carrier — the variant-delete is escalated/co-designed, not improvised.

## Non-goals

- Not a rewrite of the substrate — the intent representation already exists (the `.dag` graph); this removes shell *from* it.
- Not deleting `Pipeline` — it stays as optional sugar; it just stops being load-bearing.
- Not deleting `shell.Exec.Run` — it is the legitimate bottom bash transport; it is *confined* to realization, not removed.
- Not one big PR — staged per tier, each green-by-execution, priced by the shell-in-intent it removes.

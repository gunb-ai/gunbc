# W2 handoff — dissolve `live_deploy` ensure-dependency retained-shell site (cool-stag-394)

Brief: W2 of the shell→dag residual census. Site = `realize_live_deploy_ensure_dependency_on_host`
→ `gunbc.live_deploy.emit deployment_ensure_dependency_script`, in
`dag/gunbc/host/host_effect_realize.dag`. Dissolve to TYPED effects, delete the script builder AND
its `retained_srvn` call in the SAME PR, do NOT merge; report PR + head to `snappy-bat-554`
(landing goes through them to `stern-bat-401`, not the operator).

Everything below was read off `9f3da28c34c` (current origin/main) in this session — file:line was
re-checked, so trust the citations over your own memory, but re-verify if main has moved.

---

## 1. What has ALREADY landed in this branch (WIP, compiles-unknown)

4 files, additive only, nothing deleted yet, `host_effect_realize.dag` **untouched**:

| file | what |
|---|---|
| `dag/extdeps/package_managers/dpkg.dag` | `data dpkg_status_program: NonEmptyStr = "dpkg"` + `fn dpkg_status_command(package: NonEmptyStr) -> ArgvCommand` spelling `["dpkg","-s",{package}]`, cited to the `Package.Status` op row |
| `dag/extdeps/package_managers/apt.dag` | `fn apt_get_install_command(package: NonEmptyStr) -> ArgvCommand` spelling `["apt-get","install","--yes",{package}]`, cited to the `PackageManager.Install` op row; added `import extdeps.exec.command { ArgvCommand, argv_command }` + `import extdeps.exec.program { uncataloged_program }` |
| `dag/extdeps/exec/command.dag` | admitted both new builders in `argv_command`'s admit list (after `dpkg_query_status_command`) |
| `dag/extdeps/exec/program.dag` | admitted both in `uncataloged_program`'s admit list |

Both builders use `uncataloged_program(invocation: ...)` + inline word literals, mirroring the two
precedents in the same layer: `dpkg_query_status_command` (dpkg.dag ~143) and
`systemctl_daemon_reload_command` (systemctl.dag ~942).

## 2. What REMAINS (in order)

1. `dag/gunbc/host/host_effect_realize.dag`: imports + pure staging builders + pure folds + rewrite
   the realizer (all exact shapes in §4–§6). Delete the `deployment_ensure_dependency_script` import
   (line 196) and the `retained_srvn` call (line ~4606).
2. Delete `deployment_ensure_dependency_intent` (emit.dag ~4747) +
   `deployment_ensure_dependency_script` (emit.dag ~4773). Consumer census already done (§7).
3. Witness `dag/test/claim/live_deploy/ensure_dependency_witness.dag` + migrate the orphaned
   `witness_dependency_ensure_is_idempotent` in `dag/test/claim/live_deploy/emit.dag` (~393, import
   at ~159) off the deleted builder onto `argv_words` asserts.
4. Gates (§8), then PR + report to parent.

## 3. The crux, already settled (do not re-litigate; evidence inline)

- **`run_typed_argv_transport(argv: List<String>, transport)` is NOT viable.** It refuses
  `LocalShell` outright ("LocalShell has no remote target — use SshShell or FleetSsh"). LocalShell is
  live on this seam, so a typed-list route would regress it. → use the sealed-`ArgvCommand` seams.
- **Sealed `ArgvCommand` builders beside an op row are the landed, sanctioned form.** Precedent:
  `systemctl_daemon_reload_command` hand-spells `sudo -n systemctl daemon-reload` over the
  `DaemonReload` row's `["systemctl","daemon-reload"]` and is gate-admitted
  (command.dag:219). The repo's own framing (systemctl.dag ~938): *"one operation, two carrier
  grammars, each module owning its own."* The anti-fork tombstone in dpkg.dag lines 18–22 forbade a
  loose **`List<String>`** speller (`dpkg_status_argv`) — a different shape. No op-row→`ArgvCommand`
  bridge exists anywhere (`argv_command(program, arguments)` at command.dag is the only constructor;
  `shell_materialize_operation_argv` yields bare `List<String>` inside `ArgvMaterialized`, and no
  materialized→sealed bridge exists).
- **DRIFT IS CHECKED, NOT ASSERTED.** Add two match-authority witnesses that materialize the op row
  with `shell_materialize_operation_argv` and compare token-by-token with `argv_exact_token_equal`
  against `argv_words(command: ...)`. Copy `systemd_run_transient_operation_argv_matches_authority`
  (`dag/gunbc/systemd_run_transient.dag` ~228) verbatim in structure; `OperationRef { service, name }`
  + bindings `{ package: pkg }`. Rows: `("dpkg","Package.Status")`, `("apt","PackageManager.Install")`.
- **`deployment_serve_route_command` (W3, commit `6b028ce9cf`, landed) is the same shape** — minted a
  sealed builder because no op row exists there. Its realizer is the one-seam template at
  host_effect_realize.dag ~4630.

## 4. Faithful original behaviour (must be preserved exactly; refusal-for-refusal)

`deployment_ensure_dependency_script(dep)` = `emit_pipeline_or_poison(deployment_ensure_dependency_intent(dep), live_deploy_ensure_dep_emit_refused_poison)`;
intent = `Pipeline { steps: ensure_dependency_steps(dep, privileged: false), on_failure: FailFast }`.

- **Package arm** (`ensure_apt_package_step`, emit.dag ~471, with `privileged: false` from ~4749):
  `If { cond: Not { inner: ExitZero { run: dpkg_status_effect(pkg) } }, then_: [apt_install_unelevated_effect(pkg)] }`.
  → probe **unelevated** `dpkg -s {pkg}`; if exit 0 → **no install, no-op**; if non-zero → **unelevated**
  `apt-get install --yes {pkg}`. Op rows: `["dpkg","-s","{package}"]` (0=installed, 1=not-installed,
  other=error) and `["apt-get","install","--yes","{package}"]` (0 / 100 / nonzero), both in
  `extdeps/package_managers/{dpkg,apt}.dag`. NOTE the probe is the **`dpkg`** binary, not
  `dpkg-query` — different executable, different row (`dpkg_query_status_command` is the *other* one).
- **Marker arm** (`ensure_dependency_steps`, emit.dag ~1858, `EnsuredOwnershipMarker`): leg1
  `deploy_stage_write_command("store-ownership-marker", content)` (operations.dag:185) =
  `printf %s {content} > $_gunbc_stage/store-ownership-marker` **unelevated**; leg2
  `deploy_stage_install_owned_command(...)` (operations.dag:217) =
  `sudo -n install -m {mode} -o {owner} -g {group} $_gunbc_stage/store-ownership-marker {dest}`
  **elevated**. `$_gunbc_stage` is set by `mktemp -d` **only** in the bulk-apply frame
  (host_effect_realize.dag ~3211) — it is UNBOUND on this standalone route, i.e. the script body was
  already broken here (latent bug), which is part of why this site is worth dissolving.
- `FabricStorageOwnershipMarkerSpec { path, content, mode: FileMode, owner, group }` — owner/group are
  **principal NAMES** (`NonEmptyStr`), not uids (fabric_storage_placement.dag ~100). Production values:
  path `<fabric_storage_mount_root() as String>/fabric-storage-control/ownership`, content
  `role-singleton:` ++ instance_id, mode `FileMode { owner: ReadWrite, group: Read, other: None }`,
  owner/group = the service principal (spec.dag ~1220-1245). `DeploymentDependencyStep { kind, subject }`
  (spec.dag ~160); `EnsuredSubject = EnsuredPackage { package: NonEmptyStr } |
  EnsuredOwnershipMarker { marker: FabricStorageOwnershipMarkerSpec }`. Production packages:
  `package_tailscale`, `package_tmux`.

## 5. STAGING DECISION — process-owned tmpdir, NOT a `.gunbc-stage` sibling

My earlier pin assumed the compile-pool idiom (sibling `<dest>.gunbc-stage` + elevated `dd`).
**Do not copy that here.** Two independent reasons, both verified:

1. **`sudo dd` is not grantable.** The ci-runner's sudo vocabulary
   (`ci_runner_sudo_binaries`, `dag/gunbc/fleet/fleet_posix_accounts.dag` ~360-420) is a CLOSED
   coproduct with no `dd` arm (`SudoInstall`/`SudoRm` exist; there is no `SudoDd`), and no `/usr/bin/dd`
   appears in any sudoers roster in the tree. The compile-pool gets away with elevated `dd` because its
   target dir is root-owned and it is a different locus. Here, an elevated `dd` would surface as
   "command not allowed" — a route that always refuses.
2. **The original never wrote next to the marker.** It wrote into a `mktemp -d` directory (mode 0700,
   process-owned), which is why the write leg was legitimately unelevated.

So the faithful+capable shape is: **mktemp scratch dir → unelevated `dd` write into it → elevated
`install -m -o -g` into the control dir → discard**. Reuse, do not mint:

- `mktemp_directory_command(template: NonEmptyStr) -> ArgvCommand` = `["mktemp","-d",template]`
  (gnu_coreutils.dag ~571).
- `admitted_scratch_path(prefix: String, stdout: String) -> String?`
  (`dag/gunbc/host_command_binding.dag` ~62) — the house validator for mktemp stdout: trims, requires
  `starts_with(prefix)`, rejects empty/equal-to-prefix, rejects `\n`, space, `/..`; returns
  `Present { value }` / `none`. **host_effect_realize.dag ALREADY imports `gunbc.host_command_binding`
  (line 3), so reusing it adds no new module edge and no cycle.**
- `rm_recursive_force_one_file_system_command(path: String) -> ArgvCommand` (gnu_coreutils.dag ~169)
  for the discard — unelevated, because the scratch dir is process-owned.

Prefix convention: follow `install_media_remaster_work_prefix` (a `data` prefix + `X` template tail).

## 6. Realizer design (exact shapes; all signatures re-verified)

Seams, both at host_effect_realize.dag ~3685 / ~3724:
`run_argv_over_host_effect_transport(command: ArgvCommand, transport) -> RunArgvTransportOutcome`
`run_argv_stdin_over_host_effect_transport(command: ArgvCommand, stdin_payload: String, transport) -> RunArgvTransportOutcome`
Arms: `LocalShell` **executes** (`shell.Exec.RunArgv` / `RunArgvStdin`); `SshShell { ssh_host }` →
`typed_argv_exec_over_ssh[_with_stdin]`; `FleetSsh` and `EmitArtifactThenThinRun` →
`RunArgvTransportRefused { reason }`. That is exactly where `run_shell_transport` refused them too, so
the refusal posture is preserved, not widened.
`RunArgvTransportOutcome = RunArgvTransportObserved { observed: ShellExecObserved { exit_code, success, stdout, stderr } } | RunArgvTransportRefused { reason: String }`.

Terminal fold (host_effect_realize.dag ~3933):
`reconcile_shell_exec_observed(observed, intent)` → exit 0 ⇒
`Converged { evidence: ShellEffectApplied { exit_code, stdout, stderr }, applied: intent }`; non-zero ⇒
`NotConverged { reason: shell_exec_failure_reason(...), applied: intent }`.
`Reconciliation = Converged { evidence, applied } | NotConverged { reason, applied }`
(dag/std/realization_reconcile.dag ~37).

**WITNESS MUST BE DRY.** LocalShell really executes, so never call the seam from a test. Split into
PURE builders + PURE folds that take `List<RunArgvTransportOutcome>` (the compile-pool split at
5592-5597 says this explicitly), and have the realizer run legs then hand outcomes to the fold.

Mirror these three helpers in-file (compile-pool, ~5818-5872) — the wording matters for review:
`..._install_stage_failure` (step + unit + observed), `..._install_step_failure_line` (splits
Refused → "did not run … -- {why}" vs Observed → "operation exited {code}"; a refusal never started a
process so it must not claim an exit status), `..._install_discard_line` (observed names its exit
code, refused says the discard did not run either). `run_argv_transport_succeeded(outcome) -> Bool`
(~5874) is already in-file: only `RunArgvTransportObserved` with exit 0 counts.

Builders to add (pure):
- package: probe = `dpkg_status_command(package)`; install = `apt_get_install_command(package)`.
- marker: `staged = concat(scratch_dir, "/store-ownership-marker")`;
  write = `dd_overwrite_path_command(path: staged)` run via `run_argv_stdin_over_host_effect_transport`
  with `stdin_payload: marker.content as String` (**unelevated**, unlike compile-pool);
  install = `sudo_elevate_command(command: coreutils_install_file_owned_argv(mode: file_mode_octal(marker.mode), owner: marker.owner, group: marker.group, source: staged, dest: marker.path as String))`;
  discard = `rm_recursive_force_one_file_system_command(path: scratch_dir)` (**unelevated**).
  (`coreutils_install_file_owned_argv` takes `mode/owner/group: NonEmptyStr`; `file_mode_octal(mode:
  FileMode) -> String` from `extdeps.access.posix` ~126 — needs the same `non_empty_str(...)`
  conversion operations.dag ~375-400 uses. `sudo_elevate_command(command: List<String>)` prepends
  `sudo -n`. **Not** `coreutils_install_file_argv` — that one has no `-o/-g`.)

Fold requirements (every original failure → a typed NotConverged, never a silent default):
- package: probe **Refused** → NotConverged (name the probe); probe exit 0 → Converged, **no install
  run** (idempotence evidence = the probe's own observation); probe non-zero → install; install
  **Refused** → NotConverged; install 0 → Converged; install non-zero (100 or other) → NotConverged.
- marker: scratch acquisition refused/unparsable (`admitted_scratch_path` → `none`) → NotConverged;
  write failed or refused → run discard, NotConverged with the write's line + discard's line; install
  failed or refused → run discard, NotConverged likewise; both ok → discard (best-effort, its outcome
  reported either way) → Converged.

## 7. Consumer census (delete-first, already enumerated by name)

- `deployment_ensure_dependency_intent` / `deployment_ensure_dependency_script`: only
  host_effect_realize.dag (import 196 + call ~4606), the emit.dag definitions, and
  `dag/test/claim/live_deploy/emit.dag` (~159 import, ~393 `witness_dependency_ensure_is_idempotent`,
  ~519 comment). No product/app consumer → outside the required gate.
- **KEEP** `ensure_apt_package_step`, `ensure_dependency_steps`, `deploy_stage_write_command`,
  `deploy_stage_install_owned_command`, `live_deploy_ensure_dep_emit_refused_poison` — the bulk-apply
  route (emit.dag ~4418, `privileged: true`) still consumes them. Check `..._poison` consumers again
  after deletion; delete it only if orphaned.
- Both dispatcher arms call the realizer (ungated ~4718, gated ~4978) and pass `transport` through
  ungated — keep the realizer TOTAL over `dep.subject` (no silent default arm). Preserve the ungated
  dispatcher's separate refusals of Apply/Retract (`ungated_live_deploy_emission_refusal`, ~4660).

## 8. Gates / PR-body evidence (parent's lane rule from #13988 — non-negotiable)

1. `gunbc test` for the touched claims is NOT sufficient — it does not catch an unresolved name used
   only inside the claim's own module. For every witness added/changed run the strict closure compile:
   `gunbc compile --entry <claim-file.dag> --source-root dag --source-root src/v2 --output-dir <dir> --target dag`
   **BOTH source roots** (with only `dag` you get 1000+ bogus `v2.*` unresolved-name errors). "Pass" =
   0 blocking and 0 unresolved type/name/import errors.
2. PR body must contain, from the EXACT head: strict-compile output, the positive control, and the RED
   output. Copy W3's (#13988) format.
3. RED must be *discriminating*: e.g. assert the install leg's words and then mutate
   `apt_get_install_command` — a `retained_srvn`/`realize_shell_on_host` regression must fail it.
4. Cargo goes through `ctrl-build` (remote linux/amd64 runner; never a whole-tree local build).
5. Expect conflicts in `host_effect_realize.dag` with sibling workers → integrate main with a merge commit.

## 9. Loose ends / judgment calls left for the successor

- Whether the standalone `live_deploy_ensure_dependency` route has any live production caller (grep
  found none direct) — the realizer must stay total regardless, per §7.
- The elevated `install` leg inherits whatever sudoers posture the bulk-apply route has; it is the
  original's own leg verbatim, so no widening — but a wet run would be the first true coverage.
- Consider whether to also delete `deploy_stage_path_word`'s `$_gunbc_stage` coupling once nothing on
  this route uses it (the bulk-apply frame still does; probably leave it).

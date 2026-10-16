# W6 HANDOFF — approval-broker dark install → typed effects (zesty-eagle-794)

Author: zesty-eagle-794. Pushed 2026-10-16 at babysitter request (msg_48d35cf8) when a
capable-model worker was assigned to take over the wave. **No `.dag` code was ever committed** —
this branch carries the complete verified design, the exact bash reference, the consumer
enumeration, and empirically-proven algorithms, so the successor can author directly.

Base at push: `10b73b6105b` (origin/main). Worktree clean at every other point.

## 1. SCOPE (authoritative = msg_960911e3 + msg_bd4245af + msg_7633e70a + msg_d06d66a3)

ONE PR:

- **(A)** New typed tree-install vertical `approval_broker_tree_install` (typed replacement of the
  `release_locus_install_steps` concept). Lives in `gunbc.host.host_effect_realize` (NEXT TO
  `run_argv_over_host_effect_transport`) — babysitter ruling msg_bd4245af explicitly rejected the
  emit.dag home and rejected W5's proposed `gunbc.host.host_effect_argv_transport` extraction
  (no new module, no 42-line import churn).
- **(B)** Full typed cutover of the dark-install site
  `realize_approval_broker_dark_install_on_host` (dag/gunbc/host/host_effect_realize.dag, ~:4562
  at older heads — grep, do not trust line numbers): delete `approval_broker_dark_install_script`,
  `approval_broker_dark_install_intent`, `approval_broker_dark_install_body_fragments` and the
  `retained_srvn(` call at that site (the `retained_srvn` import stays — 7 other sites still use it).

DO NOT touch `realize_live_deploy_apply_on_host` (PB2/valiant-owl-428) or
`realize_microvm_controller_install_on_host` (W5/silent-newt-387) sites. Their tree copy lives
inside their still-shell scripts — swapping one leg of a still-shell script is the partial
migration the census forbids (msg_960911e3).

KEEP (do not delete) every shared bash emitter: `release_locus_install_steps`,
`release_locus_tree_copy_steps`, `release_locus_binary_install_step`,
`release_locus_tree_receipt_step`, `approval_broker_tree_copy_steps`,
`approval_broker_dark_install_release_steps` is dark-only (delete),
`apply_approval_broker_unit_write_steps`, `approval_broker_unit_write_steps_on`,
`approval_device_store_fragments`, `approval_broker_helper_grant_fragments`,
`approval_broker_helper_grant_steps` (CENSUS KEY — must stay intact for
`gunbc.auth.privileged_effect_census`), `approval_broker_lifecycle_steps`,
`apply_slice_control_converge_fragments`. Declare the frontier in the PR body: consumers remaining
= `realize_live_deploy_apply_on_host` (PB2) and `realize_microvm_controller_install_on_host` (W5);
whoever cuts the LAST consumer deletes the emitters.

## 2. CONSUMER ENUMERATION (delete-first, verified at this head)

Dark-only builders (delete with the cutover):
- `approval_broker_dark_install_script` — consumers: host_effect_realize.dag:4570 (the site),
  `dag/test/claim/live_deploy/emit.dag:146` (import), `:266`
  (`witness_approval_broker_dark_script_text`), `:2154` (a claim `let script = …`).
  Both test consumers must be FLIPPED to the typed route (witness over the intent/legs, not the
  script text).
- `approval_broker_dark_install_intent` — consumed by the site only (grep re-confirm).
- `approval_broker_dark_install_body_fragments` — site + `dag/test/claim/live_deploy/
  approval_broker_helper_grant.dag:29/381` (flips too).
- `approval_broker_dark_install_release_steps` — dark-only wrapper of
  `release_locus_install_steps` for this site; delete with the site.

Shared (KEEP): `release_locus_install_steps` ← `approval_broker_tree_copy_steps` (apply site),
`approval_broker_unit_write_steps_on` (:2461), `approval_broker_lifecycle_steps` (apply :3428),
`approval_device_store_fragments` (apply :3912), `approval_broker_helper_grant_fragments`
(:3131/:3913). Re-run the enumeration by name before deleting anything.

## 3. THE SHARED VERTICAL — API (confirmed with W5, msg_c44bd592 + msg_a0d6a703 replies)

```dag
data ApprovalBrokerTreeInstallConverged {
  installed_digest: String,   // the receipt value (64 lowercase hex)
  exit_code: Int, stdout: String, stderr: String,  // last install leg's evidence
}
data ApprovalBrokerTreeInstallRefused { step: String, reason: String }
type ApprovalBrokerTreeInstallVerdict =
    TreeInstallConverged { c: ApprovalBrokerTreeInstallConverged }
  | TreeInstallRefused { r: ApprovalBrokerTreeInstallRefused }

fn approval_broker_tree_install(
  owner: NonEmptyStr,        // dark: spec.service.service_user ; W5: microvm_controller_owner
  base_root: String,         // dark: spec.approval_broker.repo_root ; W5: microvm_controller_root
  source_root: String,       // EXPLICIT param (msg_7633e70a — NEVER ambient cwd).
                             // Both current callers pass git.Inspect.Toplevel().path as String
                             // (extdeps/git/inspect.dag:137 — readonly, returns absolute
                             // RepoPath{path: FilePath}). absolute source + absolute dest =>
                             // every leg absolute => LocalShell seam needs no cwd.
  revision: ReleaseRevisionBinding,
  transport: HostEffectTransport,
) -> ApprovalBrokerTreeInstallVerdict
```

Self-contained: it takes the transport and runs its whole fail-fast fold internally (its own
`run_argv_over_host_effect_transport` / `_stdin_` legs). W5 consumes it as ONE leg of his fold:
`TreeInstallRefused{step,reason}` → his `NotConverged{reason: "microvm-controller-install:
tree-copy <step>: <reason>", applied: intent}`; Converged → proceed (his later legs'
ShellEffectApplied wins — my evidence never becomes his final).

`RevisionNotInstalled` → `TreeInstallRefused{step: "release-dir", reason: …}` BEFORE any mutation
(the bash dies on the `__GUNBC_DEPLOY_UNREACHABLE__` poison the same way; keep that refusal).

## 4. LEGS (exact bash reference, verified from emit.dag at this head — all in one ArgvCommand
per transport call, elevated with `sudo_elevate_command(argv_words(...))` exactly as the bash's
`sudo` prefix does):

0. (site-level, NOT the vertical) owner poison guard: the bash refuses via
   `approval_broker_dark_install_body_fragments` being wrapped by the ownership predicate;
   replicate as a pure observation BEFORE any mutation → site-level NotConverged naming the step.
1. dir-ensure (all mode 0755):
   - `/opt/gunbc` — `install_directory_command(mode: install_d_shared_mode, "/opt/gunbc",
     privileged: true)` → typed: `coreutils_install_directory_argv` (gnu_coreutils.dag:314,
     mode = `install_d_shared_mode` FilePermissionString("0755")) + sudo_elevate.
   - `base_root`, `<base_root>/releases`, `<root>`, `<root>/<sub>` for each
     `approval_broker_tree_subdirs` (["dag", "src/v2"]) — typed:
     `coreutils_install_directory_owned_argv(mode: install_d_shared_mode, owner, group: owner,
     path)` (gnu_coreutils.dag:394) + sudo_elevate.
   - Note `install -d` creates the whole chain (empirically verified; the per-subdir rows below
     would work without these, but keep them: they match the bash and give the refusal strings).
2. tree copy per subdir `sub` in ["dag","src/v2"] (bash, from `release_locus_tree_copy_steps`):
   - dirs:  `(cd $PWD && find <sub> -type d -print0) | xargs -0 -I{} sudo install -d -m 0755 -o owner -g owner <root>/{}`
   - files: `(cd $PWD && find <sub> -type f -print0 | sort -z) | xargs -0 -I{} sudo install -m 0644 -o owner -g owner "$PWD"/{} <root>/{}`
   - typed: ONE `shell.Find.FilesAndSymlinksWithMode(root: <source_root>/<sub>)` per sub
     (extdeps/shell.dag; readonly, transport shell; output `{success, stdout, stderr}`; rows =
     `\t`-separated `%y\t%m\t%P\t%l\t%s` = kind/mode/REL-PATH/symlink-target/size, pre-sorted by
     the HOST sort — do NOT rely on that order, re-sort in-substrate). Filter rows to kind `f`
     (the bash is `-type f` only; `l` rows are NOT installed by this script — refuse on a `l`
     row appearing rather than silently dropping? NO: the bash silently drops symlinks too; keep
     behavior: files only. Record this as preserved-bash-behavior in a comment).
     Parsing precedent: `observe_package_materialized_tree` (package_delivery.dag:1182 — split
     `\n`, parse `\t`, refuse on a malformed row).
     Each file row → `coreutils_install_file_owned_argv(mode: install_file_shared_mode "0644",
     owner, group: owner, source: <source_root>/<rel>, dest: <root>/<rel>)` + sudo_elevate via
     the seam. Fail-fast: first failed leg → `TreeInstallRefused{step: "tree-copy <rel>", reason:
     stderr/exit}` — the bash's `set -e` + `|| exit` semantics (NEVER a silent default).
   - The Find itself failing (`success == false`) → Refused{step: "find <sub>"}.
3. binary install: `sudo install -m 0755 <source_root>/target/release/gunbc <root>/gunbc` →
   `coreutils_install_file_argv(mode: install_executable_shared_mode "0755", …)` + sudo_elevate.
4. receipt — THE INDEPENDENT READBACK (babysitter msg_d06d66a3; closes the "open half" named at
   emit.dag's `release_locus_install_steps` note: "the readback runs IN the deploying checkout,
   so it computes the source digest itself and compares against what it reads here. Shell reports
   one fact; the fold decides."). Do BOTH sides in-substrate (the gunbc process runs ON the host —
   runtime-present — so source checkout AND installed tree are locally readable; this is DESIGN
   §3d independent observation, not self-reference):
   - `source_digest` = substrate hash of the file set at `<source_root>` under the same two
     subdirs; `installed_digest` = same procedure at `<root>`.
   - Procedure (BYTE-IDENTITY PROVEN — see §6): rows = `find <sub> -type f` → per file
     `line = "<sha256hex>  <relpath>\n"` (TWO spaces, the sha256sum format) → sort lines
     **BY RELPATH, BYTEWISE** → digest = sha256 of the concatenation.
   - If `source_digest != installed_digest` → `TreeInstallRefused{step: "receipt-verify",
     reason: naming both digests}`. This is the RED target (§7).
   - Then write `candidate_revision=<sha>\ninstalled_digest=<installed_digest>\n` (release_revision_execstart_text
     gives `<sha>`) at `<root>/gunbc-approval-broker-tree-receipt` (data
     `release_locus_tree_receipt_name`), mode 0644, owner+group. Bash: `printf … | sudo install
     -m 0644 -o owner -g owner /dev/stdin <path>` → typed:
     `run_argv_stdin_over_host_effect_transport(sudo_elevate_command(argv_words(["/usr/bin/install",
     "-m","0644","-o",owner,"-g",owner,"/dev/stdin",<path>])), content, transport)`.
   - sha256 primitives: per-file and of-string via extdeps/tools/sha256sum.dag —
     `sha256sum_file_digest_via_shell(path)` → `FileDigestLineObserved{line}` then
     `sha256sum_digest_from_line(line)` extracts the hex; `sha256sum_stdin_digest_via_shell(content)`
     is the of-string final digest (no temp file needed).
     ALTERNATIVE (faster, precedent `observe_package_materialized_tree`): the whole-file hex can
     also come from per-file DigestFile over transport if you want the digest legs over the seam —
     either is admissible; in-substrate matches the package_delivery precedent.

## 5. SITE CUTOVER (B) — the fold shape

Follow the LANDED precedent: `compile_pool_slice_install_staging` + `compile_pool_slice_install_step`
(host_effect_realize.dag — the #12989 pattern; extracted copy at
`docs/plans/w6-evidence/12989_STEP_FULL.txt`): pure staging builder returns the leg list, the step
fn folds each leg over the seam, fail-fast — first refusal is a typed `NotConverged` NAMING the
step, all-success is `Converged { evidence, applied }` where evidence carries the LAST leg's
`ShellEffectApplied` (host_effect_realize.dag:421). Keep the existing `Reconciliation` shape the
current site returns (the sibling 7 `retained_srvn` sites all return it — grep for how
`realize_shell_on_host` maps to it).

Leg order = the bash's (from `approval_broker_dark_install_body_fragments` + the intent):
1. strict frame (`deploy_shell_strict_options_command` = `set -euo pipefail`) → DISSOLVES: fail-fast
   fold replaces it (each leg's nonzero exit → Refused/NotConverged). The `$STAGE` mktemp `let`
   dissolves too: stage paths become static derived paths (`<dest>.gunbc-stage`), no mktemp shell var.
2. `approval_device_store_fragments()` — same fns the apply site uses; type as legs (install -d +
   manifest write via stdin seam).
3. `approval_broker_helper_grant_fragments(revision, principal)` — sudoers: write snippet to stage
   (stdin seam) → `extdeps.sudo.visudo.visudo_check_command(stage)` (W5 op, ALREADY ON MAIN —
   PR #13890; do NOT mint) → `mv` → `visudo_policy_check_command(<final>)` → readback. KEEP the
   census key `approval_broker_helper_grant_steps` INTACT and re-check
   `gunbc.auth.privileged_effect_census` classification after the change (brief: privileged site).
4. `approval_broker_dark_install_release_steps` → REPLACED by `approval_broker_tree_install(...)`
   (§3). Refused → NotConverged{reason: "dark-install: tree-install <step>: <reason>"}.
5. unit write (from `apply_approval_broker_unit_write_steps` minus the tree legs: the unit file
   bytes → `<root>/gunbc-approval-broker.service` via stdin seam / gunbc.typed_remote_file_write).
   NOTE: `apply_approval_broker_unit_write_steps` also calls
   `approval_broker_tree_copy_steps(spec.approval_broker.service_user, base_root, revision)` —
   that is why the dark install DOES install the tree today.
6. `deploy_effect(inv: systemctl_daemon_reload_effect())` — exists (extdeps.systemd;
   `systemctl_daemon_reload_command`), typed leg.
7. `approval_broker_lifecycle_steps(spec)` — systemctl enable/restart/start legs
   (`systemctl_enable_now_unit_command` etc. — already typed argv in extdeps.systemd; run over seam).
8. `apply_slice_control_converge_fragments(spec)` — per-cap `systemctl_revert_memory_caps_argv` +
   `systemctl_set_property_argv` (already typed argv builders; seam legs).

Seam choice for the PR body (msg_960911e3): `run_argv_over_host_effect_transport` is the LocalShell
seam — `run_typed_argv_transport` REFUSES LocalShell ("LocalShell has no remote target"); this site
is runtime-present on the caller (the broker host == deploy host), which is why a typed fold over
it is the admissible realization. Say that.

Flip the 2 test consumers (§2) in the same PR: witness claims move from script text to the typed
intent/leg list.

## 6. VERIFIED FACTS (do not re-derive; re-run the script to confirm)

- **Receipt byte-identity PROVEN** twice:
  (a) real repo tree, 8591 files: bash `cd <root> && find dag src/v2 -type f -print0 | sort -z |
  xargs -0 sha256sum | sha256sum | cut -d' ' -f1` == in-substrate per-file-hex lines sorted BY
  RELPATH bytewise → sha256(concat) → `146966554d9f6e3c6cc4d963a34fa0eafce7e2c16a4b10934233a4d94ac9cc1d`
  both sides (`docs/plans/w6-evidence/byte_identity_output.txt`; rerun
  `sh docs/plans/w6-evidence/receipt_digest_byte_identity.sh` from repo root).
  (b) DISCRIMINATING fixture (`dag/a`, `dag/a/x`, `dag/p/bb`, `src/v2/c`): sorting the hash LINES
  gives a DIFFERENT digest than sorting by relpath; sorting by relpath is what matches bash
  (`sort -z` sorts the FILE LIST, not the lines — the sort key is the relpath).
- Locale: the bash `sort` is locale-dependent; the deployment host runs LANG=C (proven by the
  ntfy readback witness showing LANG=C.UTF-8), and every other in-repo tree-manifest digest binds
  LC_ALL=C bytewise (runner_browser_toolchain.dag:477, self_host_compile_phase_frontier.dag:581,
  gnu_coreutils.dag:548). Substrate `sort_by` (dag/std/primitive_identity.dag:22) is bytewise
  UTF-8 — witness `canonical_order_enrolled_witness.dag`
  `sort_by_orders_string_keys_byte_lexicographically` proves ["é","z","a","Z"] → ["Z","a","z","é"].
  So the substrate order == C-locale bash order. (`std.algebra.list_sort_by_key` is also fine;
  std.algebra is NOT retired on main per msg_7633e70a.)
- The receipt's `installed_digest` is otherwise SELF-REFERENTIAL in the readback chain (written →
  read back via `observe_broker_tree_receipt` (auth/approval_broker_cutover.dag:64) → parsed →
  joined into `plan_hash` → compared by workspace_commissioning_apply_readback/credentialing — but
  nothing RE-COMPUTES it independently). Today's bash therefore proves nothing about installed
  bytes (the babysitter's concern); the §4 dual-digest comparison is the fix and is exactly what
  emit.dag's own note calls "the open half".
- `install -m 0644 src dest` does NOT create parent dirs (ENOENT) → dir-ensures load-bearing.
- Source tree today: dag 6709 files / 414 dirs, src/v2 1679 / 217, 0 symlinks.
- `shell.Find.FilesAndSymlinksWithMode` already exists and runs over the host transport
  (extdeps/shell.dag; used by package_materialized_tree_observation_note precedent).
- visudo op on main: `extdeps/sudo/visudo.dag` — `visudo_check_command(file)`,
  `visudo_policy_check_command(file)`, `data visudo_binary_path`.
- install ops on main: `extdeps/tools/gnu_coreutils.dag` — `coreutils_install_directory_argv:314`,
  `coreutils_install_file_argv:340`, `coreutils_install_file_owned_argv:364` (owner+group),
  `coreutils_install_directory_owned_arguments:375` / `_argv:394`. Modes:
  `install_d_shared_mode`="0755", `install_file_shared_mode`="0644",
  `install_executable_shared_mode`="0755". Elevation: `sudo_elevate_command`
  (extdeps/sudo/elevation.dag:96) — pattern: `sudo_elevate_command(argv_words([...]))` (see
  live_deploy/operations.dag:396-412).
- `git.Inspect.Toplevel()` (extdeps/git/inspect.dag:137, readonly, RepoPath{path: FilePath}).

## 7. RED + WITNESS (required; babysitter lane rules msg_f2d435ab + msg_79d4fbfc)

- Witness SUPPLIES the boundary values (owner, base_root, source_root, revision, transport) +
  one inhabitance claim on the real route (DESIGN §3 witness rule).
- **RED must exercise the receipt-verify refusal**: a fixture where ONE installed file differs
  (source digest ≠ installed digest) → `TreeInstallRefused{step: "receipt-verify"}`; and it must
  be discriminating (the same witness passes when trees match). Positive control = the matched case.
- **STRICT CLOSURE COMPILE** of each witness/claim file you add/change, BOTH source roots
  (`gunbc test` alone misses unresolved names inside the claim's own module — #13988 lesson):
  `gunbc compile --entry <claim-file.dag> --source-root dag --source-root src/v2 --output-dir <dir> --target dag`
  → 0 blocking + 0 unresolved type/name/import errors. Put that output + the positive control +
  RED output FROM THE EXACT HEAD in the PR body (copy #13988's format).
- Full-corpus validation loop: `gunbc run` ~8.5 min (the interpreter is at
  /cargo-target/release/gunbc; scoped `compile --entry` on host_effect_realize's closure is too
  huge — use the full run).

## 8. COORDINATION

- W5 (silent-newt-387) CONFIRMED (§3 API verbatim, msg_c44bd592): consumes `approval_broker_tree_install`
  as one leg; owner=microvm_controller_owner, base_root=microvm_controller_root,
  source_root=Toplevel, revision bound-at-emission; his fold maps Refused→NotConverged as above.
  He will NOT build any tree copy; integrate with a merge commit (he does the same).
- valiant-owl-428 (PB2, apply site) consumes the typed install later; send both the PR URL +
  head when open (babysitter msg_960911e3 instruction).
- Do NOT merge; report PR + head when reviews are green. Expect host_effect_realize.dag conflicts
  with sibling workers; integrate main with a merge commit.

## 9. STATE AT HANDOFF

- No `.dag` edits exist. Nothing was pushed before this. `/tmp/w6_build/` (persists across
  restarts) has ~1.6 MB of extracted references (bash bodies, seam, fold precedent, ops, digests).
- Suggested sequence: (1) author `ApprovalBrokerTreeInstallVerdict` + `approval_broker_tree_install`
  + `approval_broker_tree_digest_in_substrate` helper in host_effect_realize.dag above the
  `run_argv_over_host_effect_transport` block; (2) strict-compile a small claim exercising it
  (positive + receipt-mismatch RED); (3) cut the site over (legs §5), flip the 2 test consumers;
  (4) full `gunbc run`; (5) delete dark-only builders + their poisons + imports; (6) PR.

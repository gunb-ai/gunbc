# Method

Pin: 1dd3eda5696a10e8cb49700f37d61b40c304d326. Read-only: the source is `git archive <pin> dag src/v1 src/v2`, extracted to a scratch directory.

## Classes (scripts/phase1.py `klass`, applied in this order)
- test: a `/test/` or `tests/` path, `dag/test/`, `*_test.rs`, `tests.rs`
- seed: `src/v1/**/*.rs`
- compiler: `src/v2/**/*.dag`. Flagged, not scored.
- instrument: a path containing `/instruments/`
- ledger: `recurring_failure_mode/`, `rung_drop/`, `/plans/`, `design_ruling*`
- roster (ruling 2026-10-24 (b)): `dag/gunbc/guarantee_stall/**`, or a basename ending `_scaffold|_seed_growth|census|_frontier|_freeze` or starting `census_`. Exempt: `privileged_effect_census` (a named authority). Counted, not hand-read. The full member list is roster_members.txt.
  `_receipt` was tried and dropped: it caught runtime receipt types with real findings (gunbc.auth.approval_store_receipt holds a MAC-key path literal, gunbc.fleet.fleet_converge_receipt writes fixed /tmp files). Those stay production.
- extdeps: `dag/extdeps/**`
- production: everything else under `dag/`
Population at the pin: production 2,226; extdeps 1,129; seed 315; compiler 530; roster 253; ledger 1,008; instrument 145; test 4,015.
The brief's 4,138 was production + extdeps + compiler (ruling (c)).

## Phase 1 heuristics (one module at a time, comment lines stripped; leads only)
| lead | rule | FP risk | FN risk |
|---|---|---|---|
| L2 per dimension | imports a listed authority module | an import for a type alone, not a value | a module receiving the fact as a bare parameter, with no import |
| storage L4 | a string literal (no spaces, ≤160 chars) starting /var /etc /home /tmp /srv /opt /run /usr /root /nix /mnt /Users /private /proc /sys /dev | literals that are upstream facts (/proc/self/cgroup, /dev/null) or L1 declarations | relative roots built by join() from a dirname row |
| storage L4/L5 raw write | Filesystem.Write / fs::write / File::create; L5 when a storage authority is also imported (ruling (a): none-raw) | the store realization itself (fabric_storage_file_store) | writes through a wrapper function defined elsewhere |
| storage L5 mechanism | flock, O_EXCL, create_new, WriteCreateNew, Mkdir.*, mkdir_*command, create_dir_all, fs::rename, rm_recursive, Remove.RecursiveForce | the library that owns the mechanism | a mechanism under another name |
| caching L5 (seed) | `static X: OnceLock / LazyLock / Mutex<HashMap>`, thread_local!, lazy_static! | a constant initialised once (no memo) | HashMap memo fields inside structs |
| caching L3 | identifier containing memo (not memory) or cache, when no caching authority is imported | vocabulary (CacheHit as an observed arm), prose in multi-line strings | memo tables named otherwise (seen, visited, table) |
| access L4 | shell.Env.Get, env::var; absolute literal containing token/secret/credential/.pem/.key/mac-key/oauth | env vars that are upstream contract (CI variables) | credential reach through a helper |
| authz L4 | sudo_elevate, "sudo", /usr/bin/sudo, visudo, sudoers | the sudo-grant roster authority (L1) | privileged effects without sudo vocabulary (root unit, setcap) |
| placement L3 | `*_live_dashboard_instance()` / `srvN_*dashboard_instance()` call | the instance declaration and desired-deployment modules (L1) | re-selection through another accessor |
| placement L4 | literal containing srvN / fciN / .gunb.ai / localhost / 127.0.0.1 / 0.0.0.0 / r2 / amazonaws / tailscale / .ts.net; unit file names (.service .socket .timer .path .slice); /etc/systemd/ literal; upstream binary path /usr/(local/)?(s)bin/x, /(s)bin/x, /opt/homebrew/bin/x (ruling (b)) | module-path strings mentioning srv3 (decl_ref), L1 declarers | a host passed through an untyped field |
| ambient | count of Filesystem.* shell.* Http.* Git.* R2.* Systemd.* Process.* Ssh.* (dag); std::fs::*, fs::read/write/..., Command::new, env::var, TcpStream, File::open/create, OpenOptions (rs) | calls inside the realization of the seam itself | calls through helper wrappers |
| signal | ambient > 0 or any dimension ≥ L3 | | modules whose only problem is invisible to the patterns: Phase 3 measures this |

Two heuristic bugs were fixed before slicing. `Memo` matched `Memory`, which put most std modules on the cache lead. `rename` matched as a bare word. A `ttl` probe was dropped (it matched settle and title).

## Calibration (Phase 0)
25 modules: 15 named, plus 10 random production modules (python `random.seed(20261024)`, `random.sample(production, 10)`, drawn under the pre-roster class filter).
Mechanical vs hand: 104 of 125 cells agree exactly (83%); 115 of 125 agree on problem (L3+) vs not (92%).
The dominant disagreement: mechanics cannot see L1 (a declaring module reads as L3/L4 from its own literals), plus cache-vocabulary noise.
Rulings applied after calibration (parent, 2026-10-24): (a) raw state files beside a store library are L5, with note mechanism=none-raw. (b) upstream binary paths score under placement: L4 in business code, L1 only in the environment/toolchain declaration. (c) population as above.

## Seeds
The parent's precomputed seeds were unreachable: different hosts, no shared /tmp, worktree or /cargo-target mount. All leads here are recomputed at the pin. A cross-check against the seeds is pending their arrival.

## Phase 3 sample
`random.seed(20261025)`, 60 of the no-signal production modules (phase3_sample.txt, drawn before the roster class existed: roster members in it are reported separately).

## Phase 3 result (false negatives)
60 no-signal production modules hand-read: 6 had a problem the mechanics missed (10%), listed in modules.tsv with phase=P3.
gunbc.host.host_identity_adopt, gunbc.host.host_identity_assimilation, gunbc.host.host_kernel_dependency and gunbc.machine_intake.mtcollins1_boot_admission
show per-host SYMBOLS (operator_host_srvN, srvN_host, fn *_srvN_*), used to branch or to enumerate hosts, which no literal pattern sees.
gunbc.workspace_image_delivery and gunbc.ledger_row_coherence hold REPO-RELATIVE storage literals ("target/..."); the abs-path rule sees only absolute ones.
No other false negatives in the sample. Several L1 declarers looked suspicious but are the authority: gunbc.runner.runner_slot_desired (memory max),
gunbc.auth.gcp_iam_bootstrap_federation (service account), gunbc.namecheap.account (registrar user), gunbc.bmc.bmc_fan_alignment_standing (host roster).
Follow-up: both shapes were run as leads over all 2,639 remaining no-signal production+extdeps modules. They match 82 modules (48 per-host symbol, 34 relative literal).
The 73 not already read are slice 09-phase3-shapes, sent to their own reader. So the 10% miss rate is measured BEFORE that slice, and the residual
miss rate after it is unmeasured (the remaining shapes in the sample were zero of 54).
A roster-shaped directory the basename rule misses: gunbc.git_use_census.* (directory name, not basename). Noted, and left in production.

## Merge decisions (applied by scripts/merge.py when slices are combined)
- M1 (slice 03): a guest/image rootfs built as a build product is not kept data. Storage L0, a dimension `other` row, note "scratch build tree, not kept state". Guest-internal paths are not host placement. Slice 03 was sent back for instance-grain findings.
- M2 (slice 04): gunbc.fleet.fleet_bmc_observation is class production (it sits under dag/gunbc). gunbc.systemd_directive_value_grain is production under the roster rule and stays scored.
- M3 (slice 04): gunbc.site.site_pxe_edge_converge placement is L4 (no authority exists: site edge). An actuator that declares its own facts inline is not an L1 authority. L1 is reserved for a declaration that others consume.
- M1 test (parent): a build tree is scratch only while it is consumed within the same run and then discarded. Anything read by a later invocation, process or host is storage (and caching, if recomputable).
- M3 confirmed (parent): L1 only when the declaration is the home other modules consume, or the designated declaration module. A module that declares inline and acts on it is L4 (no authority exists).
- M4 = ruling (d) (parent). The test is whether the fact varies by host or deployment.
  Kernel ABI and upstream-constant well-known paths (/proc/*, /sys/*, /sys/fs/cgroup/*, /etc/os-release) are ACCESS: L4 in business code, L1 in the extdeps module modelling the interface.
  Device nodes (/dev/vdb, /dev/nvme0n1) are PLACEMENT: L4 in business code, L1 in the host or instance declaration naming the device.
  Deployment-chosen locations among upstream-legal options (/etc/systemd/system vs /run/systemd/system vs user units) are PLACEMENT: the deployment layer chooses, and the upstream module only lists the legal locations.
- M5: seed files with the 'Generated by v1 compiler' header are class seed-generated. They are counted and get one finding against the emitter template (the thread_local CACHED wrapper), not 86 findings.
- M6: #[cfg(test)] instances in seed files are test code. They are dropped, not scored L2.
- M7 (slice 01): the seed reader's rework predates ruling (d). scripts/seed_supplement.py re-adds, over the 62 hand-written .rs files (non-test lines only), kernel-ABI literals as access L4, device nodes as placement L4, and CI-environment env reads (GITHUB_*, RUNNER_*, CI) as placement L4. That is 31 rows: slices/01-seed/findings_supplement.tsv. Among them are the reader's two suspected false L0s, which proved real: memory_governor.rs and cli_run/user_work_scope.rs read /proc/self/cgroup and /sys/fs/cgroup outside their test regions. 3 rows in interp_recorded_fixture_witness.rs whose /tmp/x, /tmp/y specimens are .dag source inside string literals are dropped (slices/01-seed/findings_dropped.tsv).
- Seed class membership: 315 files, i.e. 306 src/v1/**/*.rs plus 9 src/v1/*.dag (the v1 compiler's own .dag sources, e.g. src/v1/04_env.dag). Of the 161 seed modules with a signal, 90 are seed-generated (generated_members.txt), 62 hand-written .rs and 9 .dag.
- M8 (slice 04 second pass): the reader set gunbc.systemd_directive_value_grain to class=roster. M2 keeps it production (the roster rule does not match it), so merge.py restores the class. Depth: the reader states that ~17 large modules (host_effect_realize, fleet_converge_*, package_delivery, altra_autoinstall_seed, secret_provision_actuator, ...) were read by grep plus context around every hit and a declaration scan, not line by line. Their L0-L2 cells carry that caveat.

## Slice commits the merge uses (one reader per slice; later pushes by duplicate sessions are not merged)
- 01-seed e97029f (stern-crane-458: rework 3daab4e, then 'drop prose rows'), plus findings_supplement.tsv (M7)
- 03-runner-ci d188d2c (warm-pike-594, rework). A duplicate session, lively-cat-254, never landed a push.
- 04-fleet-host 07fcc30 (fierce-boar-597, second pass)
- 05-machine-intake-sites fc3e189 (bold-tern-682). A second session on the same slice, still-lynx-483, later fast-forwarded the branch to f37754f with findings auto-generated from module cells (specimen = evidence_symbols). That push is below instance grain and is NOT merged; the branch head is not the merged sheet.

## Phase 3 reading, corrected after slice 09
The 6/60 (10%) figure is the measured miss rate of the SAMPLE. It should not be extrapolated, because the sample over-represents the two miss shapes.
The shape leads match 82 of the 1,633 no-signal production modules (5.0%), but 9 of the 60 sampled (15%). For a random draw that is about a 1-in-200 outcome. Under a different seed the sample would have shown fewer misses.
Every one of the 82 shape members has now been hand-read (9 in the sample, 73 in slice 09). 12 have a problem (6 in the sample, 6 in slice 09: lab_host_plan, managed_directory, pre_os_capture_replay, srv3_host_effect_apply, srv3_install_media_fetch, srv3_os_install_actuator_toolchain_ensure). As leads, the shapes have 15% precision.
The other 1,551 no-signal production modules were sampled 51 times (60 minus the 9 shape members) with 0 misses. By the rule of three, the miss rate there is below ~6% at 95% confidence: up to ~90 unread modules with a problem, unbounded below by this sample.
So the census's known gap is "up to ~6% of the no-signal remainder, shape unknown", not "10% of everything".
Slice 09 read depth: grep leads, plus full text for ~15 modules; L0/L1 rows there are lower confidence (reader's caveat).
- 09-phase3-shapes 7d1ae7b (witty-ant-451)
- 07-fabric-compute-workspace a0f040b (merry-raven-458, rework). Its branch history held a full-repo-tree commit (1a13ec7) from a plumbing slip, so the branch was deleted right after this fetch; the sheets are kept locally under slices/07-fabric-compute-workspace/.
- 06-roadmap-auth-dispatch 1f59ea7 (eager-swift-535). Its 8 calibration-overlap modules were copied verbatim from the parent's sheets; merge rule 1 keeps the parent's module rows, and the exact-duplicate instance filter counts the copied rows once.
  Caveat: 120 of slice 06's 173 specimens describe the mechanism in words naming the calls (e.g. 'shell.Mkdir.Parents + Chmod.OwnerOnlyDirectory + WriteOwnerOnly ...'), not a quoted line. Every row names a real fn/data symbol, so it is identity-backed; it is weaker evidence than a quoted line. The reader also reports ~15 more srv1_live_dashboard_instance() wrapper pins beyond the brief's 26 (serve, apply, tick_cli, dogfood_route, ntfy_publish, deployment_gate, dispatch_preflight ...).
- M9: any src/v1/** module is class seed (or seed-generated), whatever a sheet wrote. Slice 01 had marked the 9 src/v1/*.dag files production.
- M10: exact-duplicate instances (module, dimension, level, symbol, specimen) count once. Slice 06 had copied the 8 calibration modules' rows verbatim (40 rows removed).

## classes.tsv (schema addendum 2)
scripts/classes.py assigns every hand-read-class instance at Rederived/Literal/Forked to exactly ONE error class, using the first matching rule
(dimension + site + specimen/note/symbol keywords, in the order written in the script). Unmatched rows fall to <dim>_<site>_unclassified; that count is
reported and is currently 0. The class membership is findings.tsv filtered by class_id, so every class count is identity-backed.
current_rung is DESIGN.md §4b. This census looked for decision SITES, not for gates that already refuse them. A class is therefore written rung 0 unless a
wall is named from the corpus (authz_ambient_sudo is 1: gunbc.auth.privileged_effect_census classifies the sites, which contains them, but refuses nothing).
A 0 means "no wall observed by this census", not "proved unguarded".
attainable_ceiling, wall and red_control are written in the parent's vocabulary (admit_callers roster / type / declared requires / derived from one
declaration). They state what would hold the class; they are not a design proposal, and no new vocabulary is minted.
- 02-extdeps 001901d (nimble-bear-464). 123 modules, 13 instance rows. The reader reports ~90% of mechanical leads in extdeps were false: upstream layout and env vars are Home, and kernel ABI in extdeps.linux.* is Home.
- M11 (slice 02 misfit 1): an upstream binary location in an extdeps tool-catalog module is Home when it is where the UPSTREAM puts it (distro package path, Homebrew's documented prefix, the FHS /usr/local/bin, systemd's listed unit directories). It is placement Literal when it is OUR install choice carried in the upstream model. One instance: extdeps.cache.sccache::sccache_installed_binary_path "/usr/local/bin/sccache" (added as a parent row; the reader's L1 for the other 13 modules stands). Misfit 2: extdeps.google.issue_tracker ~/issue-tracker-shots is a provenance locator, not a runtime path: L0, as the reader scored it.
- M12: a module's class is a function of its PATH (the brief's definition, taken from mechanical.tsv). Readers' own labels (observation 29, fleet_declaration 7, host_model 7, witness 2, test 1, all under dag/gunbc/) are relabelled to their path class. The only reader refinement kept is seed -> seed-generated (M5). Relabels are listed in merge_report.txt.

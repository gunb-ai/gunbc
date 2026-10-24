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

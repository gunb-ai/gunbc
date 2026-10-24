# Child brief: hand-read one slice of the realization-decision census

You are one reader in a composite lane (parent session quiet-seal-78). CLASSIFY; do not redesign, do not fix.
Rules: read-only on the repository. NO PR, NOTHING committed, no builds. Message only your parent (quiet-seal-78).
Pin: 1dd3eda5696a10e8cb49700f37d61b40c304d326. Read with `git show <pin>:<path>`.

## Channel (containers do not share files: everything moves through data-only git branches)
Fetch the lane's data: `git fetch origin census-data/quiet-seal-78`, then
`git show FETCH_HEAD:.realization-census/slices/<SLICE>.tsv` (your slice list) and
`git show FETCH_HEAD:.realization-census/modules.tsv` / `findings.tsv` (the calibrated examples, already scored by the parent).
Write your two sheets untracked at <your worktree>/.realization-census/slices/<SLICE>/modules.tsv and findings.tsv, then push them to
the branch census-data/quiet-seal-78-<SLICE>. Use plumbing, so that your own branch and index are untouched:
```
export GIT_INDEX_FILE=$(mktemp -u); git add -f .realization-census/slices/<SLICE>/
T=$(git write-tree); C=$(git commit-tree $T -m "census data only; not for merge; pin 1dd3eda5696a; slice <SLICE>")
git push origin $C:refs/heads/census-data/quiet-seal-78-<SLICE>; unset GIT_INDEX_FILE
```
Branch conditions (set by the grand-parent; they bind you):
1. The branch holds .realization-census/** only: no code, no .dag edits.
2. Never open a PR from it; never merge or rebase it. If the dashboard opens one on its own, do not touch it: message me.
3. Every commit message starts with "census data only; not for merge" and names the pin 1dd3eda5696a.
4. Do not delete the branch; I delete it after I merge your sheets.
Otherwise: NO PR, nothing committed to any other branch, no builds, read-only on the repository.

## Your slice
The slice file has one row per module, with mechanical default levels and the leads that put the module there. Hand-read EVERY module in it.
The mechanical levels are leads, not verdicts. Measured on 25 calibration modules: 83% of cells agree exactly. The main bias is that the mechanics
cannot recognise L1: a module that IS the declared authority (a roster of sudo grants, an instance table, an upstream extdeps model) gets L3/L4
from its literals. Also noisy: the "cache vocab" L3 lead (the word appears, nothing is cached), prose inside multi-line strings, and "raw writes"
when the write goes through a seam the module is the realization of. Match the calibrated rows' judgement.

## Output
modules.tsv columns: module, class, storage_level, caching_level, access_level, authz_level, placement_level, ambient_calls, worst_finding, evidence_symbols, phase(=P2)
findings.tsv columns: module, dimension, level, symbol, specimen (literal or mechanism, quoted), note, existing_authority, fix_shape
One modules row per slice module, even when every level is L0 (then worst_finding = "none: <why the lead was false>").
One findings row per instance at L3/L4/L5 and per `other`. L1/L2 rows are optional.
Cite symbols (fn/data/type names), never line numbers alone. Text inside string literals is prose, not code.
When pushed, send ONE message to your parent (quiet-seal-78): the branch, the row counts, modules per worst level, and any taxonomy misfits.

## The taxonomy (verbatim from the lane brief)
The five dimensions (the facts business code should receive, not decide):
1. STORAGE: where kept data lives (root, path, dirname, bucket, served endpoint), key-to-name layout, codec, write semantics (overwrite / create-only / compare-and-set / append), retention, setup (who creates the directory), permissions.
2. CACHING / MEMOIZATION: whether a recomputable value is cached at all, where (memory / disk / remote), its key, size or TTL, invalidation; hand memo tables and cache directories.
3. ACCESS (reach): how code reaches a resource: credential and secret locations, tokens, env vars, endpoints, sockets, ssh targets, the identity it calls as.
4. AUTHORIZATION (privilege): the authority a privileged effect needs: sudoers rows, grants, approval capabilities, token scopes, `requires` on operations; ambient root vs a received capability.
5. PLACEMENT / SETUP: which host or instance the work and its state belong to (literal `srv1`, `srv1_live_dashboard_instance()` re-picks, hermetic vs live), unit names, ports, listen addresses, environment-specific branches, hand-maintained deploy steps.

The levels (one ladder, applied per dimension per module):
- L0 none: the module makes no decision in this dimension.
- L1 declared authority: the module IS the one place the fact is declared. Not a problem. Example: `gunbc.roadmap.roadmap_dashboard_instance` declares each instance's roots, owner and environment.
- L2 received: the module takes the fact as a parameter and decides nothing. Not a problem (note "path-typed" when the parameter is a raw path rather than a capability). Example: `gunbc.roadmap.roadmap_attempt_worktree_claim` functions take `root: NonEmptyStr`.
- L3 locally re-derived: the module recomputes a fact from an authority it could have been handed, or re-selects it. Mild: duplicated derivation, and the class behind the wrong-instance bug. Examples: `roadmap_belt_actuate` `belt_worktree_claim_ensure` receives `stores` AND re-derives `claims_root` from `instance`; the 26 entry wrappers that pin `srv1_live_dashboard_instance()` around `*_for_instance` cores.
- L4 hard-coded: a literal path, host, user, bucket, size or TTL in business code with no authority. Example: `gunbc.auth.approval_assertion_counter` `approval_assertion_counter_root = "/var/lib/gunbc/approval-assertion-counters"`; `/etc/systemd/system/` hard-coded in 13 modules.
- L5 hand-rolled mechanism: the module re-implements a shared mechanism beside the library (its own compare-and-set, lock, codec, retention sweep, cache, mkdir, credential lookup, approval check).
- AMBIENT flag (orthogonal, per module): count of direct world calls from business logic: `Filesystem.*`, `shell.*`, http, git, R2, systemd operations, not through an injected seam.
A module's level in a dimension is the WORST level it exhibits; the findings sheet keeps every instance.

Authorities (the homes a conforming module consumes; DESIGN.md section 3b is the rostered list):
storage: std.artifact_store, std.cache_interface, std.durable_compare_and_set, gunbc.durable_cas_file_store, gunbc.filesystem_interface, gunbc.owned_directory, gunbc.managed_directory, gunbc.roadmap.dashboard_instance_directories, std.realization (Placement).
caching: std.materialization_ladder, std.materialization_provider, std.cache_interface, extdeps.realization.cache_purity.
access: gunbc.auth.access_token_source, gunbc.auth.gcp_secret_access, gunbc.auth.github_gcp_federation, std.resources.
authorization: std.scoped_authorization, std.effect_grant, gunbc.auth.authorization_pattern_selection, gunbc.auth.privileged_effect_census (it already classifies privileged-effect sites: cite its classification, do not re-derive it), gunbc.auth.approval_*.
placement: gunbc.roadmap.roadmap_dashboard_instance (HostDashboardInstance), product.fabric.selection, gunbc.compute.work_provider_local (ComputeLayout), gunbc.live_deploy.*, gunbc.fleet.*.
A module that imports an authority and passes its values through is L2; one that imports it and rebuilds the fact is L3; one that names the fact without the authority is L4.
A module whose home authority does not exist yet is "L4 (no authority exists)", with the missing home named. It is not an invitation to invent one.
fix_shape: L3 "receive the value instead of re-deriving"; L4 "declare once and consume"; L5 "adopt the library and delete".

## Conventions settled at calibration (follow them)
- Raw state-file writes (Filesystem.Write of a kept record) beside an existing store library: L5 storage.
- extdeps modules that faithfully model an upstream's own layout, env vars or paths: L1, not L4. They become L4 only when an extdeps
  module carries OUR policy (our host, our path, our bucket).
- Raw state files where a store library exists = L5 storage, note "mechanism=none-raw" (ruling a).
- Upstream binary paths (/usr/bin/systemctl) score under PLACEMENT: L4 when literal in business code; L1 only in the environment/toolchain
  declaration (gunbc.roadmap.roadmap_dashboard_instance execution_environment, dispatch_capability_program) (ruling b).
- Roster modules (guarantee_stall/*, *_scaffold, *_seed_growth, census/frontier/freeze rosters about the repo) are their own class and are not in your
  slice. If you meet one anyway, give it class=roster and do not score it.
- In the seed (Rust), OnceLock/thread_local/static memo tables are L5 caching. std::fs on CLI paths is ambient and, where a path is literal, L4 storage.
- Does not fit? Use dimension `other` with a note, never force it.

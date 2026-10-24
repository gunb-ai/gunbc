# Classify durable-store sites for the sealed-store migration (read-only)

Repo: /home/briansrls/.worktrees/gunbc/bright-gull-439 (detached at origin/main). Do NOT edit, commit, build or run
anything in the repo. Read only. Write your output file only into the scratchpad path you are given.

## Context (what the migration is)
Every durable store is being moved onto the "sealed-store" pattern (reference PR gunb-ai/gunbc#14944, read
`git show pr14944:dag/gunbc/roadmap/roadmap_attempt_worktree_claim_root.dag` for the canonical example):
1. one declaration per store in its declaring module: `data <store>_slots = { member, mode }`;
2. one sealed record `type <Store> sole_constructor { ... }`;
3. one mint taking the BUSINESS KEY (instance, owner, host...) and never a path; it derives the root from the
   declaration, lists/ensures the directory, returns Ready{store} | Unready{detail};
4. business functions take the sealed record, never a root/path/filesystem;
5. the owned-directory/provisioning row is derived from the same declaration;
6. no caller allowlists.
The store inside the sealed record today is the CAS `SlotStore` (gunbc.durable_cas_file_store: generation-checked,
create-new slot files). A "durable store" = state written by one invocation and READ BY A LATER invocation,
process or host.

## Your input
A TSV slice (columns: module, path, subsystem, in_census, in_14944, in_14972, signals, census_classes, abs_paths).
`signals` came from a mechanical symbol reader: cas:<CAS library symbols bound>, mkdir:<n> (shell.Mkdir.* or
install/mkdir argv), write:<ambient Filesystem.Write* counts / fs field writes>, shell:<Copy/Move/Chmod/Remove>,
mktemp:<n>. Census rows for a module: `awk -F'\t' '$3=="<module>"' <SCRATCH>/storage.tsv` (columns:
finding_id, class_id, module, class, dimension, site, level, symbol, specimen, note, existing_authority, fix_shape).
Read each module's file (the `path` column) — enough to identify every durable store it writes or owns.

## Rulings to apply (from the census method)
- M1: a build tree / temp dir is SCRATCH only while consumed within the same run and then discarded. Anything read
  by a later invocation, process or host is storage.
- (d): kernel ABI / upstream-constant paths (/proc, /sys, /sys/fs/cgroup, /etc/os-release) are ACCESS, not stores.
  Device nodes are placement. An upstream tool binary path (/usr/local/bin/x, distro paths) is not a store.
- Writing a host config file that an upstream daemon reads (/etc/systemd/system/*.service, /etc/ssh/*, nginx conf)
  is host CONFIGURATION realization, not a business store: NotAStore(host-config) unless gunbc itself reads it back
  as its own state.
- An instrument/witness that writes a receipt/artifact into the repo tree (artifacts/, target/, relative paths)
  that a later run reads: classify as `ReceiptArtifact` kind; still a store if read back later.

## Output: one row per (module, store) — a module may own 0..n stores. TSV with header:
module	store	kind	business_key	root_spelling	readers_writers	disposition	reason	notes

- store: a short name for the store (e.g. `approval_decision_records`), or `-` when the module has none.
- kind: one of Slot (already on SlotStore/CAS), RawRecords (keyed records written raw with Filesystem.Write* to a
  kept root), SingleFile (one kept state file), AppendLog (append-only log/journal), ReceiptArtifact, Provisioning
  (module only creates a directory another store owns), Scratch, HostConfig, KernelABI, ToolBinary, Secret
  (materialized credential files), Other.
- business_key: instance | host | owner | principal | subject:<what> | none.
- root_spelling: the symbol(s) that spell the root today and HOW (literal "/var/lib/gunbc/x", derived from
  dashboard_instance_*_root(instance), passed in as a parameter from caller X, relative "artifacts/...").
- readers_writers: modules/functions that read or write this store (cross-module consumers matter: search with grep
  for the root symbol and the record path function across dag/ and src/v2/).
- disposition: Migrate | NotAStore | Blocked:<blocker> | Covered:<PR> (already sealed by #14944/#14972; check
  `git show pr14944:<path>` / `git show pr14972:<path>`) | Gone (the census symbol no longer exists on main; say
  where it went if you can tell).
  Blocked reasons to use when they apply: `Blocked:format` (raw records/file not on SlotStore; sealing requires
  moving the durable format onto the slot store — say whether live data exists on hosts), `Blocked:no-key`
  (no business key), `Blocked:shared` (one store shared across instances/hosts), `Blocked:append-log`,
  `Blocked:ambient` (cannot be sealed without removing ambient filesystem authority), `Blocked:seed` (Rust),
  `Blocked:held` (file also touched by open PR #14799: dag/gunbc/plan_governance.dag,
  dag/gunbc/plan_registry_batch_g.dag, dag/gunbc/plans/logical_work_realization.dag).
  Use Migrate only when the pattern fits as-is: it is already a SlotStore (or trivially one) keyed by a business key.
- reason: one line citing symbols (module + symbol, never file:line).
- notes: anything a cohort planner needs (e.g. "root also spelled in dashboard_instance_directories", "consumer in
  another subsystem: gunbc.x.y").

Be honest: if you could not determine something, write `unknown` and why. Do not invent stores. A module with only
scratch writes gets one row with store `-`, kind Scratch, disposition NotAStore.

When done, reply with a <=15-line summary: counts by disposition and kind, and any store that does not fit the
pattern for a reason not in the list above (these are the important ones).

## RULING UPDATE (parent, supersedes Blocked:format)
The pattern is NOT SlotStore-specific. The sealed record may hold the store's OWN operations (e.g.
`ApprovalDecisionStore sole_constructor { write_record: fn(...), read_record: fn(...), list: fn() }`), each closed over
the derived root and the fs, with on-disk bytes unchanged. So a raw-file store (RawRecords, SingleFile, ReceiptArtifact
read back later) is `Migrate`, with notes `FormatConsolidation` (moving onto CAS is a later lane). Do NOT use
Blocked:format. Use `Blocked:root-per-call` when the store's operations cannot be closed over ONE derived root: the root
is chosen per call by a caller, or paths are computed from record content / span several roots. Name these precisely
(module + symbol) — they are the interesting cases. Other Blocked reasons (no-key, shared, append-log, seed, held,
transport) still apply.

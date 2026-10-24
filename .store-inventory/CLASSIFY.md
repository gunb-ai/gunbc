# Durable-store disposition: classification brief

Repo checkout (current main, read-only for you): /home/briansrls/.worktrees/vivid-ibex-111
Module `a.b.c` lives at `dag/a/b/c.dag` (`v2.x.y` at `src/v2/x/y.dag`).

## Context
A lane is moving every durable store in the gunbc .dag corpus onto the "sealed-store pattern"
(reference: dag/gunbc/roadmap/... worktree claim store in PR #14944, not on main yet):
one `{member, mode}` declaration per store in its declaring module; a `sole_constructor` record
for the store; one mint that takes a BUSINESS KEY (instance, owner, host...) never a path, derives
the root from the declaration, observes the directory, returns Ready{store}|Unready{detail};
business functions take the sealed record, never a root/path/filesystem; the owned-directory
provisioning row derives from the same declaration. Today the only store mechanism the pattern
wraps is the generation-checked CAS slot store `gunbc.durable_cas_file_store` (observe a key,
create-only commit of generation N+1, list keys).

Your job is READ-ONLY CLASSIFICATION. Do not edit any file in the repo.

## Input
Your digest file (listed in your prompt) has one `## <module>` block per module, with:
- `census <class> <site> <symbol>: <specimen>` rows from a prior hand census (may be stale), and
- reader rows `effect|cas <symbol> @<enclosing decl> L<line>: <args>` = every storage-effect
  call (Filesystem.Write*, Filesystem.Delete, shell.Mkdir/Remove/Move/Copy/Chmod/Mktemp/Link) and
  every call into gunbc.durable_cas_file_store, found on current main.

For each module, open the source and read the cited functions far enough to know WHERE each write
lands and WHO reads it later. Do not stop at the digest.

## Output: one TSV row per STORE (or per non-store site cluster) in the module
Columns (tab-separated, no header):
1. module
2. store — short name of the durable thing (e.g. `approval decisions`, `belt-last-tick.json`), or `-`
3. shape — exactly one of:
   - SLOT        already kept through gunbc.durable_cas_file_store (observe/commit/keys, windowed/retained variants)
   - HAND_CAS    own create-once / O_EXCL lease / ordinal-claim / compare-and-set beside the CAS library (WriteCreateNew + readback adjudication, marker files, Move.NoReplace elections)
   - RECORD_DIR  kept records in a directory, overwritten in place (Filesystem.Write/WriteOwnerOnly of JSON etc.), read back by a later run/process
   - APPEND_LOG  kept append-only log/events/spool (written once per event, consumed/drained or replayed later)
   - SINGLE_FILE one kept state file at a fixed host path (a counter, a last-tick record, a cursor)
   - SECRET_PATH a credential/key/config file at a host path that this module reads or stages (it is provisioned elsewhere)
   - DEPLOY_TREE files written into a host's installed tree as actuation (unit files, /etc config, an install root, an image rootfs), not a store the corpus reads back as records
   - RECEIPT     a run receipt/report written to a repo-relative or artifacts path (target/..., artifacts/...) for a human or a later CI step
   - SCRATCH     within-one-run scratch: mktemp dirs, files written then consumed and removed in the same run, build trees discarded after the run (census ruling M1: scratch only while consumed within the same run and discarded; anything read by a later invocation/process/host is storage)
   - KERNEL_ABI  /proc, /sys, /dev, configfs, cgroup paths
   - TOOL_BINARY an upstream tool's binary/install path
   - LIBRARY     the module IS a store library / realization (defines the mechanism, e.g. durable_cas_file_store itself)
   - WITNESS_FIXTURE a production module whose writes exist only to set up a probe/instrument fixture in a temp dir
   - NONE        the cited site is not a write to anything durable (false positive; say why)
4. root — how the root/path is obtained: `literal:<path>` | `instance:<fn>` (derived from a HostDashboardInstance or similar) | `param` (caller passes a path) | `mktemp` | `env:<VAR>` | `repo-relative:<path>`
5. business_key — what naturally keys this store if sealed (instance / host / owner / principal / none / shared-across-instances)
6. readers — the modules/functions that read it back later (best effort, name symbols), or `same-run` / `none-found`
7. evidence — the enclosing symbols you read (fn names), comma-separated
8. note — one line: anything that makes the pattern NOT fit (no business key, shared across instances, append-only, format change needed, a live format other consumers parse, held by another lane), or why it is NONE.

Rules:
- Cite symbols, never line numbers, in evidence/note.
- If uncertain between two shapes, pick the one the READERS imply (who reads it later and how) and say so in note.
- Every module in your digest gets at least one row. A module may get several (one per distinct store).
- Write your rows to the output path given in your prompt (create the file; overwrite if it exists).
- Your final message: the row count, a count per shape, and any module you could not classify, with why. Keep it short.

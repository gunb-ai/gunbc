# Sealed-store lane inventory (jolly-hawk-60). Data only; never open a PR from this branch.

inventory.tsv: one row per (module, symbol). The population is the union of
- the realization census storage rows (census-data/quiet-seal-78 f0e73dc1d045, dimension=storage, site Forked/Literal/Rederived), rebased onto main a3c3ce131c14;
- jolly-hawk-60's symbol-level reader (reader.py over main and #14972 6e87465f5717): import-resolved call sites by enclosing declaration, comments and strings stripped; root-taking CAS ops (signatures read from gunbc.durable_cas_file_store), the second hop through root-taking library fns (hold, fabric storage, materialization store, caller backlog, build cell), Filesystem.Write*/Delete/LinkCreateNew, shell.Mkdir/Chmod/Copy/Move/Remove outside Mktemp-scoped fns;
- bright-gull-439's independent reader + per-store hand reads (memory/pending/bright-gull-439/out).
Columns: disposition = merged under the rulings; jolly / bright = each source's own call; conflict=Y = the sources disagree and the cohort child settles it first.

Rulings (stern-ram-763, 2026-10-24):
- Run receipts (single-file target/*.txt, CLI + instrument): ReHomedObservability; kept here at module x symbol grain for an observability lane; not migrated here.
- Plain-file records and sweeps: seal in place, no format change. Each store has its own sole_constructor record whose fields are its own operations closed over the derived root and an fs; on-disk bytes stay identical (prove by readback of an existing record). Moving onto SlotStore is a FormatConsolidation follow-up row per store.
- Hand WriteCreateNew/Mkdir.NewOwnerOnly locks are a fork of std.durable_exclusive_hold: move onto the hold library in the cohort PR; state why no lock can be live across the deploy boundary.
- Host singletons: nullary mint; the root derives from ONE declared host-state tree root plus the store's member declaration, never a literal. No host key (nothing consumes one); a store that turns out to need one is Blocked: host-key.
- Blocked (named): append-only logs; ops that cannot close over one derived root (root-per-call); remote-host realization; GitHub Actions artifact/cache transport; shared stores (approval store: gunbc.rung_drop approval_store_single_writer_by_agreement).

## Final merge (v2)
Spine: vivid-ibex-111's per-store hand read (census-data/vivid-ibex-111-store-inventory b3a8cb4cf8e, shape column), plus rows only jolly-hawk-60's or bright-gull-439's readers found (census-data/bright-gull-439 0cc49a7b3cb9): hold-library consumers, instruments, extdeps, src/v2.
Further rulings (stern-ram-763, 2026-10-24): APPEND_LOG -> Blocked: append-only (named); SECRET_PATH -> ReHomedAccess; DEPLOY_TREE -> ReHomedPlacement; shared parent roots split into one member declaration per store under the parent (Blocked only if two stores share one directory's entries); BMC /tmp credential files OutOfLane.
cohort = the child PR that owns the row. 'note' starting 'child checks:' is a disposition the child confirms before migrating.

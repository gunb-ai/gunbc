# bright-gull-439 sealed-store lane state (2026-10-24)
- Inventory in progress. reader.py = symbol-level storage-surface reader (run from worktree root, outputs JSON).
- population.tsv = union of census storage rows (census78 = census-data/quiet-seal-78) + discovery, 368 modules.
- slices/*.tsv classified by 7 subagents per classify_brief.md -> scratchpad/out/*.tsv (lost on restart: re-run).
- #14944 and #14972 both OPEN as of 2026-10-24; #14972 deletes file_slot_store_over (contradicts brief residual) -- raise.
- gunbc.fleet.printer_batch folded into gunbc.fleet.printer_actuation (#14945).
- Seed (22 .rs) -> Blocked:seed.
- RULING (stern-ram-763, 2026-10-24, msg_e98b9660): option C. The pattern is NOT SlotStore-specific. The sealed record holds the
  store's own operations (write_record/read_record/list fns closed over the derived root + fs); bytes on disk stay
  byte-identical (prove with a readback of an existing fixture record per PR). Raw stores are in wave 1 = Migrate.
  Format->CAS is a FormatConsolidation follow-up row per store, NOT Blocked. Blocked only when ops cannot close over one
  derived root (root chosen per call, paths from record content) -- name those. Copy jolly-hawk-60 if not the lead.
- #14944/#14972 heads both keep file_slot_store_over (14972 body stale). 14972 not on current 14944 head.

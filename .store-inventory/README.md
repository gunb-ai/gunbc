# Durable-store inventory (vivid-ibex-111, handed to jolly-hawk-60)

Data only. Not for merge; never open a PR from this branch. Built on census-data/quiet-seal-78 (f0e73dc1d045).
Subject: gunbc main at a3c3ce131c1 (2026-10-24).

## Population (333 production modules; dag/test and *.test.* excluded)
Union of:
1. `.realization-census/findings.tsv` rows with dimension=storage, site in Forked/Literal/Rederived, seed (Rust) rows excluded: 212 modules (one, `gunbc.fleet.printer_batch`, deleted since by #14945).
2. `reader.py` over main: every Filesystem.Write*/Delete/LinkCreateNew and shell.Mkdir/Remove/Move/Copy/Chmod/Mktemp/Link call (effect primitives cannot be import-aliased, so a comment- and string-stripped token read is their symbol grain), and every call into `gunbc.durable_cas_file_store` (import-aware: selective import list or qualified name). Raw rows: `sites.tsv`. 275 modules.
   Cross-check: the resolve binding receipt of instrument-dispatch run 40076688259 (base 7a03b68be4c4) found zero CAS bindings the reader missed; it misses body calls the reader found, so the reader is the population source.
Seed (src/v1 Rust, 22 census modules) not dispositioned here: v1 is semantics-frozen and the sealed record is a .dag construct.

## Columns of inventory.tsv
module, store, shape, root, business_key, readers, evidence (symbols), note, proposed.
`shape` is from a symbol-by-symbol hand read (brief: CLASSIFY.md; per-module inputs: digest/). `proposed` is a mechanical mapping of shape to the lead's rulings as relayed by stern-ram-763; it is a hint, not a ruling.

## Shape counts (439 rows)
RECEIPT 139, SCRATCH 80, SLOT 34, RECORD_DIR 34, HAND_CAS 29, NONE 23, DEPLOY_TREE 22, APPEND_LOG 17, SECRET_PATH 16, WITNESS_FIXTURE 11, TOOL_BINARY 11, SINGLE_FILE 10, LIBRARY 9, KERNEL_ABI 4.

## Findings the cohort plan must carry
- `dashboard_instance_dispatch_attempt_state_root` is one directory holding several stores (dogfood start receipt slots, session placement records, dispatch identity ordinals, belt projects). Each needs its own member before it can be sealed.
- `/var/lib/gunbc/printer-starts` holds three stores (print_converge records, printer_actuation start claims, assembly_inventory_store CAS slots): a shared-root problem.
- Workspace allocation/acquisition/gateway SLOT modules commit through `gunbc.fabric.durable_cas_fabric_storage` over a FabricStorageBinding, not a directory root.
- The four approval SLOT stores have module-literal roots for one broker deployment: no instance key (singleton rule applies).
- `authorization_claim_store_root` is declared in the library `gunbc.durable_cas_file_store`, consumed by `gunbc.authorization_claim_slot`.
- `runner_microvm_cell_readiness_store` is RECORD_DIR with ~9 readers parsing its JSON; a format change touches all of them.
- `scm.repository_save` is a second write path for the store `repository_slot` publishes through CAS.
- Credential hygiene seen in passing (not this lane): fixed /tmp paths holding a BMC password (`srv3_bmcweb_login_body_path`, `bmc_onboard` pwbody/netrc), never removed.
- Open PRs touching CAS-store files on 2026-10-24: #14944/#14972 (attempt family), #15033 and #14087 (materialization_store_local, assembly_inventory_store), #14721/#14388 (approval_device_redemption), #14064 (host_boot_attempt_admission), #14396 (roadmap_dogfood_route), #14421 (compute_seat_ensure, worktree claim).

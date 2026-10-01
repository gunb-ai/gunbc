# Workflow and commissioning qualification repair

The workflow now exposes `workspace_commissioning_plan`, selecting the existing commissioning-only fleet entry. It binds the dispatched host and revision, consumes the fleet SSH authority, and uses the shared plan-artifact upload. This adds no alternative VM launch or privileged execution path.

## Local controls

- Roadmap: 40/40, recorded in the adjacent roadmap-assertions receipt.
- Workflow input and projection: 36/36, exit 0; peak RSS 12,231,316 KiB (11.7 GiB).
- Fleet scope: 20/20, exit 0; peak RSS 6,367,328 KiB.
- Commissioning dispatch/installer integration: 4/4, exit 0; peak RSS 8,439,768 KiB.
- Release readiness: 2/2, exit 0 under 6 GiB. The control bodies stay beside the sealed constructor; their test declarations now live in a discoverable `_test.dag` entry.

Integrated runs used MemoryMax=12G, MemorySwapMax=0 and MALLOC_ARENA_MAX=2. No deployment limits changed. Compiler SHA-256: `71f79c1f82268376fde1a8784b8620ce6a057af0ac72fcc4f20fb505793c4a31` (the previously rebuilt compiler; Rust sources unchanged by this cut).

The workflow formerly loaded 2,012 modules and failed at the memory limit after eight controls. Removing an unused ledger-coherence import reduced it to 1,411, but that still failed. Moving the scope vocabulary and artifact directory out of the effectful planner, and importing the dashboard receipt constant from its receipt authority, reduced the test closure to 1,323 and allowed the full suite to finish. The scope variants and wire behavior are unchanged. The remaining boot-prelude assertion was stale after the existing typed Bash emitter began quoting arguments; it now checks the actual curl argument sequence with both shared timeout values.

Each snapshot was produced by `tools/tests/dag_validation_closure.py` from unchanged module copies. `populations.json` records compact source-identity digests rather than another large duplicate source manifest. Tests ran with `gunbc run --source-root target/<snapshot> --entry target/<snapshot>/<module>.dag --claim-run`. The logs retain each named result. These are local controls; integrated exact-head CI remains a separate gate.

## Generated artifacts and limits

The design ledger was regenerated using its registered producer, `gunbc.design_ledgers.expected_design_rung_drops_md`; its one stale row now agrees with the authority. The historical dispatch fleet manifest was compacted losslessly against its sibling manifest, with byte-for-byte reconstruction verified before replacement. No source identities or test outcomes were discarded.

Ordinary eager workflow regeneration exceeded 12 GiB while evaluating unrelated top-level data. The supported `--claim-run --function main` route evaluates demanded data while retaining Wet execution and real filesystem writes (see `run_one_function` and `run_in_context_with_args` in the compiler). Regeneration uses the same registered `expected_fleet_converge_yml` producer and retains all its admissions. Its result is recorded separately; this does not qualify the ordinary eager entry within 12 GiB.

No production install, commissioning, initial readiness, allocation, VM boot, SSH, release or reuse is claimed by these controls. The next gate is complete source qualification, followed by the reviewed installation/commissioning and live acceptance sequence.

Workflow generation and its provenance-header producer both completed with exit 0. Their outputs were concatenated exactly as `generated_artifact_emit` does. The parsed final YAML has one commissioning plan step, both admission environment bindings, the shared artifact upload, and the canonical 15-minute planning bound. `workflow-projection.json` records its content hash. No YAML body was hand-authored.

# Reproducing the shadow census

**The instrument deliberately does not live in the tree** (ruling, `smart-ram-730`,
2026-08-22): a shadow judgment merged into the seed is a permanent second authority reading
the same facts, a parallel-representation debt whose dissolution nobody owns. It is carried
here as a patch instead, so the measurement is reproducible without the machinery becoming
live.

1. `cp shadow_arg_conformance.rs.instrument src/v1/stage0/src/shadow_arg_conformance.rs`
2. `git apply docs/probes/shadow_direct_call_arg_conformance_2026-08-22/instrument_seed_wiring.patch`
3. build `gunbc` **locally** (the runner is amd64 and the session arm64; this run executes
   what it builds, so it stays in one place)
4. arm and run, naming subject, ref and ROOT SET together — the root set is the field that
   silently makes one series look like another:

```
GUNBC_SHADOW_ARG_CONFORMANCE=/path/rows.tsv \
  ./target/release/gunbc compile \
    --source-root dag --source-root src/v2 --source-root src/v1 \
    --dependency-pool-index primary-precedence \
    --entry src/v2/compiler/03_ingest.dag --output-dir "$(mktemp -d)" --target rust
```

5. `python3 summarize.py rows.tsv` — population by outcome, fail-closed on an unknown tag
6. `python3 adjudicate.py rows.tsv` — reduces every `WouldDiagnose` against the corpus's own
   `type A = B` declarations; prints the residue in full and absorbs nothing
7. `python3 join_board.py rows.tsv <board sites_classified.tsv>` — file-grain join only

**Control to run first, every time:** compare the armed run's `compiled: N files emitted, M
diagnostics` line against an unarmed run at the same ref. If M moves, the instrument is
perturbing the board it is explaining and the measurement is void.

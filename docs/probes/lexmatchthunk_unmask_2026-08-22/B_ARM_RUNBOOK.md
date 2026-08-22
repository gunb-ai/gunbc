# B-arm runbook — what to run when the censor is removed (2026-08-22)

Written **before** the repair exists, so it is a procedure rather than a description of whatever was
done. It executes what [`PRE_REGISTRATION.md`](PRE_REGISTRATION.md) commits; it does not amend it.
If a committed rule turns out to be wrong, that is reported as a **falsified pre-registration**, not
quietly improved.

Owner: whoever lands the repair, so exposure and measurement sit together. The two raw `cargo.log`s
are published either way, so the adjudication can be re-run independently by anyone.

## Arms

| arm | ref |
|---|---|
| **A′** | the repair commit's **parent** |
| **B** | the repair commit |

**Not** `967b5bc1b92`. That ref is the *mechanism* baseline in
[`A_ARM_MASK_MECHANISM.md`](A_ARM_MASK_MECHANISM.md) and main has since moved; comparing across
several refs puts every unrelated landing inside the delta, and the join would attribute other
lanes' work to the repair.

## Steps

1. **A′ and B boards**, at their own refs:
   ```
   CSSL_STD_SEED_LINK=1 PROBE_KEEP_LOG_DIR=<dir> PROBE_EXPECT_BASE_SHA=<that arm's sha> \
     bash docs/probes/curated_cargo_probe_one.sh src/v2/compiler/03_ingest.dag ""
   ```
   This is **one entry, M=1** — the `03_ingest` closure (177 emitted files), not a whole-corpus
   compile. Measured cost on this hardware: ~13 min cold, of which ~4m15 is the probe building
   `gunbc` + `cssl_assemble` from the tree it is measuring.
2. **Twice per arm**, comparing `rustfmt`-**normalized** output. The emitter is nondeterministic —
   pure `pub use` line reordering — and the churn *set itself varies* run to run, so one run per arm
   cannot even establish the churn population. Normalization makes that class unrepresentable rather
   than measured-and-subtracted; the second run is then a **detector for any class that survives
   normalization**.
3. **Classify each arm:** `python3 docs/probes/e0308_classify_sites.py <cargo.log> <out.tsv>`.
4. **Join B** against [`registered_masked_population.tsv`](registered_masked_population.tsv) on the
   committed key: **file + normalized expected/found relation + mechanism**. Never generated line
   number — both arms renumber freely, and the stronger key (enclosing declaration) is not
   recoverable for the historical roster.
5. **Report** the three movements separately — visible board, repair movement
   (`LexMatchThunk.apply` `E0599` N→0), exposure movement (Y newly observable, J joined, K newly
   classified successors, `unexplained`) — and never lead with a single before→after total.

## Remote dispatch, with the three things that bite

```
ctrl-build --remote -- bash -lc '
set -uo pipefail
export CSSL_STD_SEED_LINK=1
export PROBE_KEEP_LOG_DIR=/tmp/keep
export PROBE_EXPECT_BASE_SHA=<sha>
bash docs/probes/curated_cargo_probe_one.sh src/v2/compiler/03_ingest.dag ""
echo "LOG_B64_BEGIN"; gzip -9 -c /tmp/keep/03_ingest.cargo.log | base64 -w200; echo "LOG_B64_END"
' > out.txt 2>&1
```

- **Env vars go inside the remote script.** `ctrl-build` forwards `RUSTFLAGS` and friends and *not*
  these; it prints `forwarding env: (none)` and the same-base refusal you armed silently never
  exists on the runner.
- **Return the log in the same dispatch.** The runner's filesystem is gone afterwards and the
  `cargo.log` *is* the measurement. Redirect to a file; piping the dispatch through `tail`/`head`
  eats the payload.
- **Do not background it.** A backgrounded `ctrl-build` dies with its shell and exits 0 with a
  truncated log, which reads as success.

## Preflight, before interpreting anything

Source SHA, compiler identity beside it (rebuilt from the tree, not the baked image), and a healthy
-pool positive control. For the mechanism controls that control is
[`controls/algebra_genericity_pair.dag`](controls/algebra_genericity_pair.dag): after the repair
`arm_b` must flip to clean **and `arm_a` must stay clean**. If `arm_a` ever refuses, the harness
moved, not the variable.

# Evidence packet v16 — `sleek-seal-226-evidence-v16` (PR gunb-ai/gunbc#15999) — REVIEW-READY, freeze-2 pending #15992

Re-frozen from v15 per sleek-owl-397 (msg_748081d7) and tidy-ant-251 (msg_549b0721 + keen-otter binding
msg_4a58e797 / msg_825f8123 relay msg_4a58e797). The probe bytes are UNCHANGED from v14 (blob 861944d8,
reader eef548c5). Seeds unchanged (SEED_A d92fd54d… on PARENT_A a4a21da…; SEED_B a2b334f8… on base
3c14fdab…); **final freeze (freeze-2) re-cuts SEED_A on the post-#15992 merge head of #15999** — until then
these bytes are REVIEW-READY, not grant-READY (parent msg, main is known-red on v2-bootstrap E0425 since
#15972; fix-forward #15992 OPEN; no further reruns spent).

## What changed v15 -> v16 (frame only; no prediction changed, no expectation edited)
- **B1 (BLOCKER)**: `--functions` is built as ONE comma-joined token. claim_batch parses CSV
  (`src/v1/stage0/src/bin/claim_batch.rs:316-328`, split on `,`); spaces in the value would make ONE field
  and kill the join (spent attempt). claim_entry normalises the roster (commas/whitespace are separators),
  validates every field as a bare identifier BEFORE invocation (malformed -> exit 2, binary never invoked;
  stub ran-marker proves it), logs `claim_entry_argv[leg tag]=…` verbatim to drv.log, and the runtime
  fixtures assert the EXACT comma argv through the REAL claim_entry for BOTH rosters (wc 4, rtp 17).
- **M1**: rows are classified with the v14 classify/verdict authority: only bare `PASS <fn>`/`FAIL <fn>` are
  verdicts. Reasoned forms (`(runtime error …)`, `(returned … not Bool)`, `(host tool unresolved …)`,
  `(returned ProcessExit …)`), UNOBSERVED and PANICKED are NONE -> exit 2 NO OBSERVATION, lines preserved.
  A reasoned FAIL can NEVER satisfy an expected FAIL.
- **M2**: probe_run gates `ctl_ok=r=0` and `ctl_out=r=X` IN PACKET (off => exit 2 VOID, never exit 0) and
  runs BOTH readers (frozen shell path + embedded python, sha-gated) on the REAL PROBE_REASON with
  normalized byte-equality; breach preserves reader-bash-<LEG>.txt / reader-python-<LEG>.txt and exits 2.
- **M3**: `grep -a "" | grep -aq .` replaced by `[ -s "$L" ]`; completion is the producer's record via
  `completed "$L"` (single `[process-partition] total_ms=<digits>` line).
- **M4**: probe cap 900 s (v14 measured total_ms 566,373).
- **M5 / keen-otter item 4 — per-entry ROOTS**: entries run `--source-root dag --source-root src/v2` only.
  Per-entry transitive closure traced AT BOTH SEEDS resolving declared `module` names
  (module_path_index/index.rs indexes by DECLARED name): wc 245 modules = 104 `dag` + 141 `src/v2`;
  rtp 245 = same; probe 246 (+1 `src/v2`); **src/v1 modules in any closure: ZERO**; no name resolved under
  src/v1. `std.*`/`extdeps.*` live under `dag/` (dag/std/algebra.dag; dag/extdeps/communication/medium.dag),
  `v2.*` under `src/v2/` (src/v2/std/diagnostic.dag declares `v2.std.diagnostic`) — both remaining roots are
  load-bearing, per entry. BUILTIN-REACH: the probe makes no host-tool/native-lane call (pure .dag folds;
  `native_census_module_resolution` is itself a .dag fn at v2/compiler/source_root_eval.dag:3087), so
  builtins add no roots. src/v1 remains in the BUILD diff (claim_batch compiles from src/v1/stage0).
- **Isolation (keen-otter item 5)**: separate exclusive `git init` worktrees wt-A/wt-B; fixed A-then-B order
  in one process; one shared $R/target under the one-build gate; DETERMINISM GATE unsets
  GUNBC_NATIVE_CACHE_ROOT / GUNBC_NATIVE_PRODUCT_REUSE / RUNNER_TEMP / GITHUB_* before any entry
  (native_product_cache.rs:207-208 store_root -> None; native_lane_runner.rs:63,744,794,834;
  emitted_closure_compile_host.rs:790-794 private probe root per process). SHARED, NOT GATED (stated limit,
  m7): $HOME caches (cargo registry, git config) — performance-only; all claim inputs are pinned blobs,
  gates fail closed. A clean porcelain is NOT offered as isolation.
- **Caps with kill-grace (m1)**: (900 wc + 900 rtp + 900 probe) x 2 = 5400; fetch 120+120 (measured seconds);
  one-build pre-fetch 60; build 900 (measured ~224 s) = 6600; + 9x30 s timeout -k 30 grace = 6870;
  + margin 330 = 7200 = RUNTIME_S. Launcher RuntimeMaxSec >= 7200 (keeper reads back).
- **m2/m3/m4/m5/m6/m7**: claim entries complete via `completed "$L"`; rc/set law complete (rc 1 with zero
  FAIL -> exit 2; rc 2 must carry FAIL or non-bare; rc 0 with FAIL rows = FINDING recorded, not refused);
  dead kit sweep removed from read_v16.sh; PRE-REG rev2 (R2 exclusivity explicit, R2d separate axis,
  R0.9 entry roots, R5 cites the ONE-table line 214 + in-packet gate, '109 fns' clause dropped,
  contradictory claim cell = join-law refusal); $HOME caches stated as unverified limit.
- SEED_PARENT constant and the v14 fragment removed; the one-build gate label corrected to its post-fetch
  execution.

## False-evidence fixtures through the REAL claim_entry (keen-otter item 1)
Exact-argv (wc/rtp, byte-equal expectations); typed-exit FAIL; non-Bool FAIL; UNOBSERVED; duplicate lines;
foreign name; missing roster line; missing completion; rc-contradictions (rc 1 without FAIL; rc 2 all-bare
PASS; out-of-contract exit 4); setup refusals (space/malformed pre-invocation, ran-marker asserted absent);
genuine bare mixed rc=1 -> exit 0. Plus the runtime-path probe fixtures (valid3&rc1, exit3/exit0 refusals,
trailing noise, junk-only).

## Pre-run evidence (local stubs only; NO compiler run, no staging, no retry)
- Happy E2E rc=0: seeds gates held, ONE_BUILD_GATE=held pre+post fetch, both legs 21/21 rows,
  CONTROLS HELD + READERS AGREE on both legs, TOTAL_CLAIM_FAILS=0, fixtures all held.
- Inversion suite (12 mutations): each trips a NAMED law with the contracted exit — see INVERSIONS-v16
  FAILS=0 log (posted with the bytes).

## Frozen artifacts (sha256)
- packet-evidence-v16.sh: 680fa020072d1b4e6fffcc0d6842d5f188ddf5b6f6ed1fd14068fb79657e7347 (118339 B)
- read_v16.sh:           8fc15b3b2c7ce3a30e2074ed31b765de1a483e6de978d57972495bb33ff01074 (5160 B)
- PRE_REGISTRATION_v16.md: d64bae960d2432cea512846d39b951a0f92c10c9b6c8587762a5242ecdcde91f (10031 B)
- root-trace.json (closure trace, this comment): bceee1141ced26e068e5e7d6135414572e19a9aeabe29e91736e510c4959e171
- probe blob 861944d8… (UNCHANGED from v14); reader blob eef548c5… (UNCHANGED)

Exact argv (keeper): `calmcrab-udrv.sh sleek-seal-226-evidence-v16 7200 32 28 4096 3200% <staged-script-path> 680fa020072d1b4e6fffcc0d6842d5f188ddf5b6f6ed1fd14068fb79657e7347 <OUT>`

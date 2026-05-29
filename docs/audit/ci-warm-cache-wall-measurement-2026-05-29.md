# CI warm-cache wall measurement — manager-brief steps + v3 job (2026-05-29)

Audit input for the CI efficiency manager (parent session clever-cat-115).
Captures per-step wall time on a warm Cargo cache so the optimization
backlog is anchored to measured numbers, not estimates.

## Method

- Sample run: `gh run view 26614863944` (push to `main`, commit
  `bfcad2c185a4a7ef3916966bfd93a622fdb2d83c`, 2026-05-29T02:45Z).
- "Warm cache" = adjacent push runs share the `cargo-ci-*` and
  `cargo-v3-*` keys (Cargo.lock + v3 sources unchanged). The prior run
  `26613903900` exercised identical keys; this run's `ci`/`v3` jobs
  hit the cache.
- Step start/complete timestamps pulled from the Actions REST API
  (`/runs/<id>/jobs`), converted to seconds.

## Job-level wall (warm cache)

| Job                | Wall    | Notes                                |
| ------------------ | ------- | ------------------------------------ |
| `fmt`              | 24 s    |                                      |
| `affected`         | 7 s     |                                      |
| `ci`               | 2 m 56 s| Discipline gates + bootstrap freshness |
| `v3`               | 12 m 31 s| **Dominant bottleneck on warm cache**|
| `self_host_ratchet`| 43 m 54 s| (Out of scope for this work item.)   |

Cross-check (run 26613903900, prior warm push): `v3 = 12 m 31 s` — same
within seconds, so the v3 wall below reproduces.

## `ci` job — manager-brief contribution

Two steps in `ci` are the manager-brief authority discipline:

| Step                                                          | Wall |
| ------------------------------------------------------------- | ---- |
| Manager-brief authority check (P2 single-authority discipline)| 9 s  |
| Manager-brief authority self-test (consumer not ceremonial)   | 18 s |
| **Subtotal**                                                  | **27 s** |

Subtotal share of `ci` job wall: 27 / 176 ≈ **15 %**. Not the largest
slice of `ci`, but it is the largest single discipline pair in that job
— and unlike the bootstrap/cache steps, both run *unconditionally* on
every push and every PR (no `affected` gating).

Other notable `ci` steps for context (warm cache):

| Step                                                    | Wall |
| ------------------------------------------------------- | ---- |
| Gate #103 — Layer 1 selection + workflow YAML policy    | 50 s |
| v3 bootstrap snapshot freshness gate (--verify)         | 43 s |
| Setup Rust + Cache Cargo restore                        | 14 s |
| Post Cache Cargo                                        | 25 s |
| Manager-brief (check + self-test, see above)            | 27 s |
| Release-doc / R4-carve / SG-0 / T-19 / fabrication / toolchain checks | <10 s combined |

## `v3` job — step breakdown (warm cache)

`v3` job: 12 m 31 s wall (751 s). The top four steps below sum to
605 s ≈ 81 %; the full row set (through "L-7 / L-8 / consolidation /
banked-dissolutions ratchets") sums to ~99 % — the residual is
runner overhead between steps.

| Step                                                                | Wall   | Share |
| ------------------------------------------------------------------- | ------ | ----- |
| v3 tests (full suite lib+bins, timings part 1/4)                    | 6 m 36 s | 53 % |
| v3 test targets compile with bootstrap-regen-fresh                  | 1 m 13 s | 10 % |
| Prebuild v3 integration test binary (lane2d wall-clock denominator) | 1 m 17 s | 10 % |
| v3 Compiler Clippy (default features)                               | 59 s   | 8 %  |
| v3 gunbc-ci binary (wall-clock warn manifest + BinaryShim stub)     | 38 s   | 5 %  |
| v3 Compiler Clippy (bootstrap-regen-fresh)                          | 34 s   | 5 %  |
| Setup Rust + caches + post-cache                                    | ~45 s  | 6 %  |
| v3 tests (full suite integration harness, HOT-FIX zero-test-filter) | 14 s   | 2 %  |
| Build ExecuteCommand unshare helper                                 | 3 s    | <1 % |
| Determinism part 2/4 + doc tests part 3/4 + per-test 2 s ratchet    | ~5 s   | <1 % |
| L-7 / L-8 / consolidation / banked-dissolutions ratchets            | ~3 s   | <1 % |

## Observations for the optimization backlog

1. **v3 lib+bins test pass dominates** (`6 m 36 s`, > 50 % of v3 wall,
   ~30 % of the whole pipeline excluding `self_host_ratchet`). Any
   seconds-scale CI target has to attack this first. Candidates worth
   the parent's investigation:
   - Is the prebuild step (`--no-run`, 77 s) already producing the
     test binary the part-1/4 step then re-links? If the lib+bins step
     is recompiling from scratch despite the prebuild, the prebuild is
     pure waste on the v3 wall.
   - The two Clippy invocations (default + `bootstrap-regen-fresh`)
     and the `test targets compile with bootstrap-regen-fresh` step
     together compile the v3 crate three more times on top of the test
     compile — 2 m 46 s combined. The feature-flag matrix is the
     proximate cause.

2. **Manager-brief 27 s runs unconditionally**. The `affected` job
   currently exposes only `v2`, `v3`, `v4`, and `workflow_policy`
   (`.github/workflows/ci.yml:69-73`), so there is no existing output
   to gate this pair on. Two grounded paths for the parent: (a) extend
   `scripts/detect-affected-components.sh` + the `affected` job
   `outputs:` map with a new component covering `docs/briefs/`,
   `scripts/check-manager-brief-*`, and the consumer paths, then gate
   both manager-brief steps on it; or (b) leave them ungated and
   accept the 27 s. Do not copy `affected.outputs.docs` into CI — it
   does not exist.

3. **`v3 bootstrap snapshot freshness gate --verify` (43 s)** lives in
   `ci`, not in `v3`. On warm cache it is the largest single discipline
   in `ci` after manager-brief. Already gated on `affected.outputs.v3`
   — no action needed, just noting it is the next largest target if
   manager-brief gating lands.

4. **`self_host_ratchet` (43 m 54 s)** is out of scope here but
   dwarfs everything else combined. Flagged for the parent so the
   prioritization of v3-job work is not mis-scoped.

## Raw timestamps (audit trail)

`ci` job (run 26614863944):
- start `2026-05-29T02:45:36Z` → complete `2026-05-29T02:48:32Z`
- Manager-brief authority check: `02:45:49Z → 02:45:58Z`
- Manager-brief authority self-test: `02:45:58Z → 02:46:16Z`

`v3` job (run 26614863944):
- start `2026-05-29T02:45:37Z` → complete `2026-05-29T02:58:08Z`
- Prebuild v3 integration test binary: `02:46:02Z → 02:47:19Z`
- v3 tests lib+bins part 1/4: `02:47:19Z → 02:53:55Z`
- v3 Compiler Clippy (default): `02:54:51Z → 02:55:50Z`
- v3 Compiler Clippy (bootstrap-regen-fresh): `02:55:50Z → 02:56:24Z`
- v3 test targets compile with bootstrap-regen-fresh: `02:56:24Z → 02:57:37Z`

# v2 compiler-root basis: does the 03_ingest canary represent the whole compiler?

2026-08-22. Measurement only — no repairs. All measurement in this document is pinned to
`ba63edc09b75f52998c1770815260ce340364549` unless a number is explicitly marked otherwise; every
probe below reproduced `MARKER_REF=ba63edc09b75f52998c1770815260ce340364549` with an empty
`git status --porcelain` dirty list.

## Question

The fleet's long-running v2-compiler canary board (`03_ingest`, 339 sites at `629252b6df`, 399/385
raw/distinct at this pin as a same-tree control) is derived from emitting and cargo-probing
`src/v2/compiler/03_ingest.dag`'s emitted closure. That closure covers 25 of the 41 top-level
`src/v2/compiler/*.dag` modules — 61.0%. Does the canary represent the whole v2 compiler, or only
what 03_ingest happens to pull in?

## Headline: the canary generalizes further than its module coverage

```
03_ingest module coverage:              25/41 = 61.0%
03_ingest share of the full failure surface: 385/522 = 73.8%
```

These are two different numbers about two different things, and the gap between them is the
finding. The 16 modules outside the 03_ingest closure contribute only 137 net-new distinct sites to
the union — because failures concentrate in shared downstream code (parse, tokenize, the v2 std
carriers) that 03_ingest already reaches even though it does not statically pull in these 16
modules' own source. **61% module coverage understates what the board actually observes.** Read 522
as a denominator expansion over 339/399, not a regression — it is what "the whole compiler,
measured" looks like once you stop measuring only what one closure happens to pull in.

## Method

- Site identity: `(error_code, location)`, where `error_code` is the coded `E[0-9]+` rustc
  diagnostic code and `location` is the first `--> file:line:col` line within 3 lines of the
  `error[E...]:` header. Raw diagnostic counts (`grep -c '^error\['`) are NOT distinct-site counts —
  the same site is frequently re-diagnosed by rustc more than once.
- **Grain bound, stated explicitly per request:** two supplemental roots emitting the same shared
  module produce identical `(code, file:line:col)` for the same underlying defect, which is exactly
  why the union below is meaningful rather than an artifact of 13 independent probes. But a site
  that is genuinely the same underlying defect, re-emitted at a *different* line under a different
  root (e.g. a different call-site instantiation), counts twice. **522 is therefore an upper bound
  on distinct defects; no lower bound is established by this measurement.**
- Each of the 41 top-level `src/v2/compiler/*.dag` modules is either inside the 03_ingest emitted
  closure (25) or was probed independently as one of 13 supplemental roots (16 uncovered modules,
  reduced to 13 independent probes after folding overlap — `emit_module` covered via emit_host /
  emit_produced overlap, plus one further reduction).
- Every probe: `curated_cargo_probe_one.sh <module.dag>` at the pin, full cargo log captured (not
  just a count — a first pass of this session's group-2/3 dispatch used `grep -c` instead of `cat`
  and was discarded before any site-level number was computed from it, precisely because agreement
  on a raw *count* across a broken and a working capture says nothing about the *locations*, which
  is the half that was missing).
- No result here is EMIT_REFUSE (an emitter-level refusal before reaching cargo). **Every one of the
  41 top-level compiler roots emits successfully today; all 522 sites are downstream Rust
  type-checker failures on the emitted output, not emission refusals.** This is stated explicitly
  because it is the kind of fact that gets assumed rather than established — one apparent
  EMIT_REFUSE surfaced earlier this session at a stale, unpinned ref and did not reproduce at the
  pin.

## Per-root: distinct site-identity count, and how many are absent from the 03_ingest 385

`absent_from_03_ingest` = sites in this root's board that are NOT in the 385-site 03_ingest board at
this same pin.

| root | distinct | absent_from_03_ingest |
|---|---:|---:|
| self_host | 44 | 22 |
| emit_host | 250 | 54 |
| emit_produced | 166 | 33 |
| emit_semantic_decl | 154 | 21 |
| emit_orchestration | 166 | 33 |
| program_partition | 159 | 26 |
| body_producer (`03_body_producer.dag`) | 1 | 1 |
| use_site_verdict | 5 | 0 |
| trait_derive_completeness | 108 | 38 |
| host_run_boundary_admission | 36 | 36 |
| native_selected_bundle_process | 29 | 15 |
| ingested_fixture_arrows | 219 | 0 |
| discovery_enumeration | 0 (green — `CARGO_GREEN`, 0 errors) | 0 |

Two rows are the interesting extremes, not the largest ones:

- **host_run_boundary_admission: 36/36, 100% net-new.** Every failure site in this module is outside
  every board this fleet has published to date. This is the single most under-observed module found
  by this probe.
- **ingested_fixture_arrows: 219/0, entirely already-covered** despite being outside the 03_ingest
  emitted closure. A 219-distinct/0-absent row is exactly the shape a join bug produces, so this was
  spot-checked by hand (5 sampled sites, each independently confirmed present in the 03_ingest set)
  before being reported.

### `absent` does not sum either — the same trap one level down

```
sum of per-root absent_from_03_ingest   = 279
actual net-new sites (union − 385)      = 137
implied cross-root overlap              = 142
```

142 sites are absent-from-03_ingest for **more than one** supplemental root simultaneously — the
same shared-downstream-code effect that produces the 61%-vs-73.8% headline also means several
supplemental roots rediscover the same not-yet-canaried defect. Summing the `absent` column gives
279 and is wrong by more than a factor of two; the number that answers "how much does the canary's
denominator actually grow" is 137, read off the union directly, never off a column sum.

## The union, on its own, never folded into either canary series

```
03_ingest canary series:      339 at 629252b6df   (and 399 at ba63edc09b, same-tree control)
compiler-root basis v1:       522 at ba63edc09b75f52998c1770815260ce340364549, covering 41/41
                               top-level src/v2/compiler/*.dag roots
```

522 = |03_ingest(385) ∪ self_host ∪ emit_host ∪ emit_produced ∪ emit_semantic_decl ∪
emit_orchestration ∪ program_partition ∪ body_producer ∪ use_site_verdict ∪
trait_derive_completeness ∪ discovery_enumeration(∅) ∪ host_run_boundary_admission ∪
native_selected_bundle_process ∪ ingested_fixture_arrows|, computed at the single pin, never
summed from overlapping per-root diagnostics.

This is a **denominator expansion**, not a regression: 339/399 measures one closure; 522 measures
the union of all 41 top-level roots at the same pin. The two series are not comparable by
subtraction or by an arrow between them — they answer different questions ("what does the long-run
canary see" vs "what does the whole compiler surface look like today").

## Raw data

Per-root raw `(code, location)` rows (not deduplicated — dedup by loading as a set) are committed
alongside this document in
`docs/plans/measurements/compiler-root-basis-2026-08-22/*.sites.tsv`, one file per root plus
`03_ingest.sites.tsv` for the control, so a future join against a newer 03_ingest board does not
require re-running these probes.

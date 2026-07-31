# Whole-corpus namespace census receipt — 2026-07-31

This directory makes the namespace measurement reproducible without committing its
1.9 MB raw log or the synthetic compile root. Every reported number is labelled by
its actual authority; the regex catalogue is never presented as compiler resolution.

## Pinned inputs

- Corpus: `0337fb27c039a800a1aff4b80140d6dbf027e595` (main at measurement time).
- Compiler: release build from PR 7427 commit
  `b222f079a0425a4015718175dd7f41a55b76b759`, canonical binding enabled.
- Compiler binary identity: 21,563,472 bytes, SHA-256
  `87538a75c2b56111f9ab1440336b858ee8618cd7892758f9c21c5af0655a68b6`.
- Raw stderr identity: SHA-256
  `06289db522ff4cbf1613d07219e6241fe1d92994710e2fc871ad82c3de19823f`.

The corpus and compiler are deliberately different commits. The measurement asks
how that exact compiler reads that exact corpus; neither identity may float.

## Reproduce

A normal whole-tree compile follows transitive imports and reached only 1,634 of the
2,746 declared modules, leaving 1,112 uncompiled. Generate an uncommitted third
source root whose one module brace-lessly imports every declared module:

```sh
python3 docs/probes/namespace_census_2026-07-31/generate_root.py \
  . /tmp/namespace-census/complete_population_root.dag
```

Compile that entry with the pinned binary, the `dag`, `src/v2`, and generated-root
source roots, and capture stderr as `pop_stderr.log`. The synthetic root forces the
compile closure to contain all 2,746 corpus modules (the compiler reports 2,747
including the synthetic root); it is a measurement scaffold and is never committed.

Then verify the raw identity and classify every diagnostic, failing closed on any
new line shape:

```sh
python3 docs/probes/namespace_census_2026-07-31/parse_diagnostics.py \
  pop_stderr.log \
  --expected-sha256 06289db522ff4cbf1613d07219e6241fe1d92994710e2fc871ad82c3de19823f \
  --population-json /tmp/namespace-census/population.json
python3 docs/probes/namespace_census_2026-07-31/provider_bounds.py \
  . /tmp/namespace-census/population.json
```

The normalized parser output must equal `summary.json`'s compiler-authoritative
classification. The provider script must reproduce its regex-bound brackets.

## Classification-total witness

`COMPILER-AUTHORITATIVE`: the compiler fold reported 18,048 hard diagnostics. The
fail-closed parser partitions those same diagnostics as:

```text
17,112 unresolved-name
   324 ambiguous-variant
   594 no-field
    12 type-mismatch
     6 singleton diagnostic shapes
------
18,048 = compiler-reported hard diagnostic count
```

The log therefore contains 18,049 lines in the diagnostic section: one compiler
header plus 18,048 diagnostics. The header says how many diagnostics follow; it is
not itself a diagnostic.

`REGEX BOUND`: `provider_bounds.py` approximates declarations in two deliberately
opposite ways. Category-strict lookup over-counts one-provider references because it
mis-keys providers declared under another category. Category-agnostic lookup
over-counts two-plus references because it merges namespaces the compiler keeps
separate. Together they bracket, rather than determine, mechanical share at
60.7–81.6% and unique consumer-to-provider edges at 2,197–2,717. In particular, the
strict catalogue's 2,867 zero-provider rows are not a semantic finding: 2,663 have a
different-category declaration and only the agnostic bound's 204 remain absent.

`INFERRED GROUPING`: grouping the 324 ambiguity occurrences by variant plus candidate
population produces 315 decisions. That is a reproducible analysis grouping, not a
compiler output. The class labels recorded alongside it are likewise analysis, never
compiler-authoritative resolution.

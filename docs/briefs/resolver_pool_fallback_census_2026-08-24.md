# Self-host bootstrap contamination receipt (2026-08-24)

PR #9075 initially produced 361 floor diagnostics and 71 regeneration
refusals. Those measurements are withdrawn: they came from a compiler seeded
from an earlier state of the same PR, not from a known-good compiler.

The discriminating experiment used one dispatch and identical merged sources:

- Seed: `claim_executor` built from merge-base `bd84f6696`.
- Subject: merged PR sources at `8913ff5`.
- Result: v2 compilation completed with zero hard diagnostics; regeneration
  planned/executed 136/136, with only generated-surface drift
  (`first_generation_equal=false`, `declared_divergent=1`).

The earlier 71 refusals were artifacts of a contaminated self-hosting seed. A
source-only diff cannot distinguish “the source is wrong” from “the compiler
that produced its mirror is wrong”; a known-good seed on the same subject is
the required control. Do not use the withdrawn 361-line taxonomy as evidence
about resolver behavior.

This receipt does not invalidate independent observations about pool fallback,
the `std.types` `List = FreeMonoid` alias, or #9083's stripped-header finding;
those require their own provenance. It also does not authorize installing the
drifted candidate mirror; drift remains a separate fixed-point question.

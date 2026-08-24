# Resolver pool-fallback census (2026-08-24)

The required-floor run on PR #9075 exposed 361 diagnostic lines after the
emission mask was removed. These are not 361 independent defects. Symptom
bucketization yields two root families:

- No definer in the scoped closure: `split` (26), `first` (38), `skip` (10),
  `length` (10), `trim` (8), receiver `split` (8), and indexing (8).
- Wrong carrier selected through pool fallback: `Present`/`Absent` variants
  on `FreeMonoid` (18 each), `Empty`/`Cons` exhaustiveness (46), incompatible
  `FreeMonoid`/`List`/`String` branches, and generic-field fallout.

The second family matches the diagnosed alias: `std.types` maps bare `List`
and `list` to `FreeMonoid` in `container_template_alias_rows`. A bare name
missing scoped visibility can therefore bind a plausible but wrong carrier
from the whole-tree pool. The first family is the same mechanism rendered as
an unresolved name rather than a wrong binding. Counts are symptom buckets,
not per-site root proofs; resolver provenance is required for exact attribution.

This census is evidence produced by the floor after the emission leaf-name
mask was removed. It does not authorize consumer repairs in PR #9075. The
upstream fix belongs to the namespace/admission work (including #9113).

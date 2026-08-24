# Resolver pool-fallback census (2026-08-24)

The required-floor run on PR #9075 exposed 361 diagnostic lines after the
emission mask was removed. These are not 361 independent defects. Symptom
bucketization yields two root families:

- No definer in the scoped closure: `split` (26), `first` (38), `skip` (10),
  `length` (10), `trim` (8), receiver `split` (8), and indexing (8).
- Wrong carrier selected through pool fallback: `Present`/`Absent` variants
  on `FreeMonoid` (18 each), `Empty`/`Cons` exhaustiveness (46), incompatible
  `FreeMonoid`/`List`/`String` branches, and generic-field fallout.

The second family matches the diagnosed alias: `std.types` declares
`type List<element> = FreeMonoid<element>` (and analogous `Map`/`Set` aliases)
and records the same mapping in `container_template_alias_rows`. A bare name
missing scoped visibility can therefore bind a plausible but wrong carrier
from the pool. A prior `Map` realization defect produced `PartialFunction` in
kernel `Map` type positions through this same alias family. Counts are symptom
buckets, not per-site root proofs; resolver provenance is required for exact
attribution.

There are two lookup layers and their bounds must not be conflated. The
file-closure `pool_bare_census` is root-scoped and has a measured population
of 733 cross-root candidates (510 `src/v1 -> dag`, 223 `dag -> src/v1`). The
function-signature fallback (`func_sig_from_global_bare` and
`global_bare_callable_node`) has no root parameter and reads `symbol_index.global_bare`;
whether its `TypeEnv` is itself root-scoped remains an open question. Thus the
733 bound is reported only for the file-closure path, not asserted for Root A.
The algebra-template guard also excludes names such as `split`, `first`,
`skip`, `length`, `trim`, and `join` from this signature fallback, so those
symptoms require separate provenance before being assigned to either path.

This census is evidence produced by the floor after the emission leaf-name
mask was removed. It does not authorize consumer repairs in PR #9075. The
upstream fix belongs to the namespace/admission work (including #9113).

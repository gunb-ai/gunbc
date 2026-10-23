# Seat 17 scoping: record_field_value_does_not_inhabit (NEVER MERGE; durable note only)

Base d945cc9074aa. Measured by grep over the base tree; nothing below was built or run.

## Root class
Fatal infer class `record_field_value_does_not_inhabit`: a string literal assigned to a `NonEmptyStr` field
(`NonEmptyStr = String where string_non_empty`, dag/std/types.dag) refuses. Roots: extdeps.currency.currency,
units.iso8601/dimensionless/iec_80000_13/iso_80000_3 (fan-out 66 each), std.interval, communication.medium,
fidelity_carriers. Confirmed natively (srv1 probe at 0c843ac2202a): a literal into any local where-refinement
field refuses; control with `String` accepts. Cause: no literal-into-where-refinement producer in
v2.compiler.infer. The consumer exists (v2.compiler.refinement_discharge); `ObligatedInferredTree.obligations`
is hard-coded Empty.

## Why the obligation arm (M2) was stopped
infer.dag (comment above infer_where_predicate_set_edge) records operator ruling 2026-10-02: PARKED; NO INFER
JUDGMENT IS COMING; refinements are being deleted for checked constructors returning Optional. M2 would invest
in a structure scheduled to die. M1 (obligations accumulator, 2fee3fe3dbe0) has no consumer without M2 and
is not to land.

## The A4(a) route (consistent with the ruling)
NonEmptyStr := FreeSemigroup<Char>; literal admitted by `literal_homomorphism_rows` row
`nonempty_unicode_scalar_literal_homomorphism` plus `literal_unfolding_refusal_for_scalar_count`; host String
only via a checked constructor. Measurement branch faf9930078c7 proved only the literal row (v1/std).

## Counts (base)
- NonEmptyStr in 4523 .dag files.
- `as NonEmptyStr`: 24316 uses in 3095 files; 18470 are string-literal casts; ~5850 non-literal
  (join/concat/variable: ethtool, git, api_rate_limit, rfc_8118, ubuntu_seeded_install_media, ...).
- 276 `type X = NonEmptyStr where brand("X")` (pijul, version, mercurial, uuid, pci/identity, dgx_spark_setup,
  std.types PathSegment/GlobSegment, ...); 13 bare aliases (GitRemoteName, PythonProgramText, ...).
- `NonEmptyStr as String` widening casts are pervasive and need a declared route from FreeSemigroup<Char>.
- v1: 48 stage0 .rs files mention NonEmptyStr (v1_std_core kernel_type_set, std_literal_elaboration, ...).

## Not covered by A4(a)
Other where-refinements over String sharing `string_non_empty`: LanguageId, SecretName, FilePath, SecretValue.
They keep the same fatal refusal.

## Not one PR; staging
1. Checked constructor (Optional/Result) for host String -> NonEmptyStr; migrate the ~5850 non-literal casts in
   waves by top dir.
2. Re-carry the 276 brands + 13 aliases.
3. Flip std.types NonEmptyStr to structural; v1 sites in the same change.
Step 3 alone clears the 66-fan-out roots only if step 1's population is already migrated.

## Measurement still owed (needs srv1)
What flipping the NonEmptyStr definition (A4(a) branch faf9930078c7 rebased on current main) does to the ~5850
non-literal casts, the 276 brands/13 aliases, and the NonEmptyStr->String widenings: new whole-corpus errors
by class and first root, via the N7 label //v2/test/claim/parse/expression_bodied_fn_decl_parse:all.

# Testgen Seed Policy Post-Mortem (Auth Regression)

**Status**: Partially complete (core fix landed; follow-up items open)
Status date: 2026-02-12
Owner: codegen/testgen + auth modeling
**DSL Alignment**: Testgen correctness hardening for DSL-generated workflows
**Track**: D — Runtime/Test Hardening

## Incident

`make gist-recent` failed in real mode with:

- `missing accessToken in impersonation response`

Generated optional-input tests were seeding required semantic auth inputs with
shape-valid placeholders (for example shell `<MOCK>` response forms), which do
not satisfy parser semantics.

## Root Cause

The generator treated two different properties as equivalent:

- structural/type validity ("this value has the right outer type")
- semantic validity ("this value is meaningful for this operation")

For auth and transport carrier types, those are not equivalent.

## Missing Pattern

The missing abstraction is a **seed policy matrix** keyed by:

- type class (what kind of value this is)
- test mode/context (how the node is being executed)

Without that matrix, testgen defaults drift toward local heuristics and spot
fixes.

## Rule We Added (Current Slice)

For `Real` single-node optional-input tests:

- required semantic-carrier inputs must be explicitly seeded from authored
  data, not synthesized placeholders.
- accepted explicit seed sources:
  - `MockSpec::input_mock`
  - `MockSpec::node_example`
  - `Node::with_example`
- if missing, generation hard-fails with a clear panic.

Current semantic-carrier class includes:

- `TransportRequest`
- `TransportResponse`
- `Credential`
- `Secret`
- `FilesystemHandle`
- `NetworkHandle`
- `ToolHandle`

## General Pattern (Target, All Types/Modes)

Policy must be centralized and deterministic:

1. Classify type into seed class:
   - `StructuralGeneratable`
   - `SemanticCarrier`
2. Classify test context:
   - `RealSingleNodeRequiredInput`
   - `DryRunBoundaryMock`
   - `LiveFlowInput`
3. Apply matrix:
   - if class/context requires explicit seed: hard-fail on missing explicit seed
   - otherwise allow witness/synthetic generation

Seed provenance must be tracked and validated in priority order:

- explicit (authored mock/example)
- witness (contract/type-derived)
- synthetic fallback (last resort)

The key invariant:

- semantic-carrier inputs are never silently satisfied by synthetic fallback in
  contexts where behavior correctness is being asserted.

## Why This Avoids Spot Fixes

Failures become policy-driven, not node-driven:

- new nodes automatically inherit rules by type class + mode
- missing authored seeds are caught at generation time
- no parser-local hacks (for example special-casing placeholder strings)

## Follow-Up Work

1. ✅ Move seed-class classification to a shared IR-level module so codegen/testgen
   and future generators consume one source of truth. _(2026-02-18: `SemanticCarrierClass`
   + `semantic_carrier_class_for_type_id` moved to `core/ir::types`, and testgen now reads
   classification from shared IR helpers.)_
2. Extend matrix enforcement beyond current slice:
   - scenario generation contexts
   - live-flow generation contexts
3. Add tests that assert unknown semantic carrier types fail closed unless
   explicitly classified.
4. Keep parser behavior strict; no placeholder-specific parsing branches.

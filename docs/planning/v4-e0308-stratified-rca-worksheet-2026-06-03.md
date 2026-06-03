# v4 E0308 Stratified RCA Worksheet — 2026-06-03

> **Status:** DRAFT RCA WORKSHEET — routes subfamilies; does not authorize one broad E0308 implementation.
> **Authority:** PR #4140 (`E0308` = 2,953); pair histogram from Jun1 M1 probe (regenerated `vivid-lynx-81` session matched #4140 exactly).
> **Mechanical rule:** each implementation lane needs its own single-authority fact and falsification probe. Do not dispatch "fix E0308".

---

## Pair Histogram

Top expected/found pairs from the regenerated M1 rustc log:

| Pair | Count | Route |
| --- | ---:| --- |
| `String => Symbol` | 1,344 | SG-1b `TargetFunctionSignatureRealization` |
| `Rc<Diagnostics> => Diagnostics` | 300 | SG-RC-LAYERING |
| `Outcome<_> => Rc<Outcome<_>>` | 184 | SG-RC-LAYERING + SG-2 residual |
| `Box<_> => Rc<Node>` | 134 | SG-RC-LAYERING |
| `Node => Rc<Node>` | 113 | SG-RC-LAYERING |
| unparsed E0308 blocks | 96 | classify after P0 routes; do not open new class yet |
| `TestClaim => Rc<TestClaim>` | 69 | SG-RC-LAYERING / TestClaim boundary slice |
| `Vec<Rc<Edge>> => FreeMonoid<_>` | 47 | SG-COLLECTION-PROJECTION (#4151) |
| `FreeMonoid<_> => Rc<FreeMonoid<_>>` | 38 | SG-RC-LAYERING around collection carrier |
| `Vec<Rc<PrimitiveFactBundle>> => FreeMonoid<_>` | 32 | SG-COLLECTION-PROJECTION |
| `Rc<Diagnostics> => Option<_>` | 32 | diagnostic result-shape boundary |
| `Vec<Rc<AlgebraInhabitanceDecl>> => FreeMonoid<_>` | 22 | SG-COLLECTION-PROJECTION |
| `Vec<Rc<Node>> => FreeMonoid<_>` | 16 | SG-COLLECTION-PROJECTION |
| `DecimalDigit => NonZeroDecimalDigit` | 15 | refinement / subset constructor proof |
| `Vec<Rc<FormalGrammarSymbol>> => FreeMonoid<_>` | 14 | SG-COLLECTION-PROJECTION |
| `Outcome<Rc<Vec<Rc<Edge>>>> => Rc<Outcome<_>>` | 14 | SG-RC-LAYERING + SG-2 residual |
| `Diagnostics => Option<_>` | 13 | diagnostic result-shape boundary |
| `Coverage<CoverageDefectAcceptanceKey> => CoverageDefectAcceptanceKey` | 13 | lens carrier/value boundary |

---

## P0 Subfamilies

### E0308-A: Function-boundary atom signature mismatch

```text
Representative:
  pub fn foo_symbol() -> String { Symbol("foo".to_string()) }

Single-authority fact:
  TargetFunctionSignatureRealization in v4.std.target_model, keyed by source_carrier and FunctionBoundarySite.

Existing worksheet:
  v4-sg-1b-function-signature-realization-worksheet-2026-05-30.md (#4099)

Worker:
  TargetAtom / Target Realization lane, not broad E0308.

Acceptance:
  SG-1b falsification probes: signature and SG-1 value form stay coupled; fn-boundary atom spelling does not bypass the realization lookup.
```

### E0308-B: Per-use-site ownership layering

```text
Representative:
  expected Rc<Diagnostics>, found Diagnostics
  expected Node, found Rc<Node>
  expected Box<_>, found Rc<Node>
  expected TestClaim, found Rc<TestClaim>

Single-authority fact:
  TargetUseSiteOwnershipRealization in v4.std.target_model, with TargetOwnershipUseSite and TargetReferenceLayer.

Existing worksheet:
  v4-sg-rc-layering-worksheet-2026-05-31.md (#4116); manual receipt sg_rc_layering.dag

Worker:
  Target Realization + Compiler Spine, with Runtime/TestClaim falsification.

Acceptance:
  Type and value positions change together when a reference_layer row changes; no name-keyed Rc/Box wrapping.
```

### E0308-C: Collection boundary projection

```text
Representative:
  expected Vec<Rc<Edge>>, found FreeMonoid<_>
  expected Vec<Rc<PrimitiveFactBundle>>, found FreeMonoid<_>
  expected Vec<Rc<FormalGrammarSymbol>>, found FreeMonoid<_>

Single-authority fact:
  TargetCollectionRealization extension for FreeMonoid<T> → Vec<Rc<T>> at consumer boundary (SG-COLLECTION-PROJECTION).

Existing worksheet:
  v4-sg-collection-projection-worksheet-2026-06-01.md (#4151); substrate gated on main; manual receipt sg_collection_projection.dag

Worker:
  Target Realization — consume collection realization row at translate boundary; no per-site collect() patches.

Acceptance:
  ProbeSeq<T> or FreeMonoid<Probe> projects to Rust boundary storage by row change only; no hardcoded FreeMonoid or Edge branch.
```

### E0308-D: Diagnostic/result shape boundaries

```text
Representative:
  expected Rc<Diagnostics>, found Option<_>
  expected Diagnostics, found Option<_>
  expected Outcome<_>, found Rc<Outcome<_>>

Single-authority fact:
  Split before dispatch:
    - Rc/raw wrapping routes to SG-RC-LAYERING.
    - Option vs Diagnostics routes to diagnostic result-shape modeling; likely Outcome/Diagnostic carrier construction, not ownership.

Dispatch:
  Do not open until SG-RC and SG-2 land, then remeasure. The remaining Option/Diagnostics band may become a small diagnostic-constructor worksheet.
```

---

## Non-Goals

- Broad E0308 implementation.
- Per-function return-type patches.
- Hardcoded `Rc::new` / `Box::new` / `.into()` insertion tables.
- Folding collection projection into SG-RC without Modeling DFS approval.
- Using E0308 count reduction as acceptance.

---

## Manager Decision

P0 implementation fanout uses existing approved SG-1b, SG-RC, and SG-COLLECTION-PROJECTION
worksheets first. Diagnostic/result-shape work waits for a post-SG-RC/SG-2 remeasure unless
a worker proves an independent single-authority fact before then.

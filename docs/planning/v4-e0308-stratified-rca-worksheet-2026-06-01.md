# v4 E0308 Stratified RCA Worksheet — 2026-06-01

> **Status:** DRAFT RCA WORKSHEET — routes subfamilies; does not authorize one broad E0308 implementation.
> **Authority:** PR #4140 (`E0308` = 2,953); local regenerated M1 probe for expected/found pair histogram.
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
| `Vec<Rc<Edge>> => FreeMonoid<_>` | 47 | SG-COLLECTION-PROJECTION |
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
  docs/planning/v4-sg-1b-function-signature-realization-worksheet-2026-05-30.md

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
  docs/planning/v4-sg-rc-layering-worksheet-2026-05-31.md

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
  TargetCollectionBoundaryProjection (name provisional) or an approved extension of TargetCollectionRealization that names boundary role and storage carrier.

Existing worksheet:
  none complete for this exact FreeMonoid -> Vec<Rc<T>> boundary. SG-5 covers Set/Map target realization and explicitly excludes SG-COLLECTION-PROJECTION.

Draft worker brief:
  Author a separate SG-COLLECTION-PROJECTION worksheet before implementation. The fact must distinguish algebraic source carrier (`FreeMonoid<T>`) from Rust storage/ABI boundary (`Vec<Rc<T>>`) and must compose with SG-RC for inner element ownership.

Acceptance:
  New carrier `ProbeSeq<T>` or `FreeMonoid<Probe>` projects to Rust boundary storage by row change only; no hardcoded `FreeMonoid` or `Edge` branch.
```

#### Arbiter adjudication (E0308-C / SG-COLLECTION-PROJECTION) — CLOSED 2026-06-01 (`proud-fox-405`)

- [x] **Single authority:** extend `TargetCollectionRealization` in `v4.std.target_model` — add boundary/storage axis for `FreeMonoid<T>` → Rust `Vec<Rc<T>>` (e.g. `TargetCollectionReprVecRc` + `source_algebraic_carrier` on row). **Reject** parallel `TargetCollectionBoundaryProjection` carrier unless extension proof fails in worksheet.
- [x] **Compose with SG-RC:** inner `Rc<T>` element ownership via `TargetBundleEdge` / SG-RC rows — do not fold ownership into collection emit branches.
- [x] **Forbidden:** `Vec<Rc<T>>` emit shims; hardcoded `FreeMonoid` / `Edge` / `PrimitiveFactBundle` name branches in `06_translate`.
- [x] **Worksheet still required:** `vivid-lynx-81` (Collection/Algebra Manager) authors full §10.0 `SG-COLLECTION-PROJECTION` worksheet before impl dispatch; this adjudication unblocks worksheet authoring only.
- [x] **READY-FOR-WORKSHEET-AUTHOR** (not impl until worksheet §8)

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

P0 implementation fanout should use existing approved SG-1b and SG-RC worksheets first. **SG-COLLECTION-PROJECTION:** Arbiter adjudication CLOSED 2026-06-01 — extend `TargetCollectionRealization` (see E0308-C adjudication); `vivid-lynx-81` authors §10.0 worksheet then §8 before impl. Diagnostic/result-shape work waits for a post-SG-RC/SG-2 remeasure unless a worker can prove an independent single-authority fact before then.


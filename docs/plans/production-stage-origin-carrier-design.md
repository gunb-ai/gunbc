# Production stage-origin carrier — design note (PR B0)

**Status: DESIGN ONLY. No implementation follows from this note.** Operator ruling
2026-08-04: write the carrier law now; wait for a closed constructor/consumer denominator
before claiming or implementing class-wide closure.

## The class this exists to close

`FixtureOriginCanMintProductionQualification` — a fixture-origin stage output satisfying a
production qualification. Measured specimen (gunbc#7683, executed 2026-08-04):

| Input to `mint_provenanced_rust_artifact` | Result |
| --- | --- |
| `direct_rust_door_inferred_tree` (hand-authored) | Accepted |
| `infer(assemble(direct_rust_door_ingest()))` (real pipeline) | Rejected |

Assemble, infer, and mint each pass in isolation; only the real composition fails. The
fixture did not bypass a check — it used a supported API whose signature had already
discarded the epistemic distinction. `mint_provenanced_rust_artifact` accepts a bare
`InferredTree`, and nothing in that type says whether it came from `infer`, a test helper,
or a literal record.

Cargo may genuinely compile the bytes and a behavioral corpus may genuinely run them,
while the bytes came from a substitute subject. That is why adding downstream execution
does not close this: the evidence is real about the wrong subject.

## What this note is NOT waiting on

The census result is **not required to design the carrier**. It is required to prove that
implementing the carrier closes the whole class. Two separable obligations:

```
this note                    the census (#7786 successor)
  what must a production-      where are all current construction
    origin carrier mean?         and qualification paths?
  who may construct it?        are there bypasses outside the boundary?
  what must production         has the repository-wide denominator
    qualification consume?       actually closed?
  what refusal states exist?
```

## 1. Production qualification requires origin-bearing input

```
CompilerInferredArtifact {
  subject: CompilationSubject
  inferred: InferredTree
  origin: CompilerInferenceReceipt
}

ProductionSourceArtifact {
  subject: CompilationSubject
  source: Medium<String>
  inference: CompilerInferenceReceipt
  emission: CompilerEmissionReceipt
}
```

Production qualification constructors consume those carriers:

```
mint_source_produced(artifact: ProductionSourceArtifact) -> SourceProducedArtifact
```

They do **not** accept a raw `InferredTree`, raw emitted source, a module-path string, or
a fixture receipt.

The fixture carrier merged in gunbc#7761 stays parallel and non-convertible:

```
SyntheticEmittedArtifact  -/->  ProductionSourceArtifact
                          -/->  ProducerEmissionReceipt
```

That containment is live today and is this note's prerequisite, not part of its scope.

## 2. Subject continuity is part of the type

```
SourceRootIngest<S>
  -> AssembledProgram<S>
  -> CompilerInferredArtifact<S>
  -> ProductionSourceArtifact<S>
  -> CargoGreenArtifact<S>
  -> BehavioralEquivalentArtifact<S>
```

A production qualification must not be constructible from a collection of individually
plausible receipts carrying **different** subjects. Each rung constructor consumes the
previous rung, so a fixture cannot skip into the chain: it cannot construct the first
production carrier.

## 3. Origin is derived, never caller-authored

Prohibited shape — a field a caller can simply set:

```
origin: CompilerDerived
```

Required shape — the constructor is owned by the stage that actually performed the
operation:

```
infer_compiler_subject(assembled: AssembledProgram) -> Outcome<CompilerInferredArtifact>
```

A fixture module may carry an `InferredTree`. It cannot construct a
`CompilerInferredArtifact`.

## 4. Unknown origin refuses; it never disappears

The prohibited shape, which #7786 is currently escaping from:

```
recognized fixture marker  -> fixture row
recognized compiler name   -> compiler row
otherwise                  -> disappear          <-- FAIL-OPEN
```

Required:

```
every production-significant construction occurrence
  -> CompilerDerived | FixtureDerived | UnknownOrigin

UnknownOrigin  -> production admission REFUSED
FixtureDerived -> production admission REFUSED
```

**Origin and measurement are orthogonal axes and must not share a coproduct** (operator
correction 2026-08-04; an earlier revision of this note listed `ExecutionMeasured` as a
fourth origin variant, and gunbc#7786 inherited that error from here). A fixture-derived
artifact can also be execution-measured — that is exactly the dangerous direct-door state,
where the bytes genuinely compiled and could genuinely run while originating from a
hand-authored substitute tree. Making measurement a sibling origin forces the model to
discard one of the two facts: `FixtureDerived -> ExecutionMeasured` would erase fixture
origin merely because downstream execution occurred, reopening qualification laundering
under better-typed vocabulary.

```
ProductionStageOrigin
  = CompilerDerived { stage_chain: CompilerStageChain }
  | FixtureDerived  { fixture: FixtureIdentity }
  | UnknownOrigin   { cause: OriginVerificationFrontier }

ProductionEvidenceGrounding
  = StructurallyGrounded
  | ExecutionMeasured { receipt: ExecutionMeasurementReceipt }
  | Unmeasured
```

Measurement always **carries** an origin; it never replaces one. Where execution is used
to bind an otherwise inferred provenance claim, the binding form is
`ExecutionBound { origin, receipt }` — still origin-bearing.

Admission states, so that lookup failure and unclassified construction forms cannot
collapse into acceptance:

```
ProductionStageOriginAdmission
  = OriginConstructionAdmitted { subject, stage_chain }
  | OriginConstructionUnverified { cause: OriginVerificationFrontier }

OriginVerificationFrontier
  = CastConstructionUnclassified
  | ModuleIdentityBindingUnavailable
  | ConstructorLookupUnavailable
  | ConstructionFormUnclassified
```

## 5. Local construction prevention is not global closure

```
ProductionOriginClosureCertificate {
  construction_admission:   CoveredConstructionAdmission
  constructor_population:   ClosedConstructorPopulation
  consumer_population:      ClosedProductionConsumerPopulation
  fixture_reachability_zero: FixtureReachabilityZero
  unknown_origin_zero:      UnknownOriginZero
}
```

The first field can begin with today's limited mechanism. The rest **cannot be claimed**
until the call-occurrence frontier closes, or a stronger proof shows every production
qualification necessarily funnels through one sealed constructor.

## 6. Parameterized by the future census, not by today's three sites

The design law is written over an abstract derived population:

```
production_significant_construction_occurrences(tree: RepositoryTree)
  -> List<ProductionConstructionOccurrence>

LAW:  for every occurrence in that population,
      the occurrence has exactly one resolved origin
```

#7786's successor supplies that population. Until it does the state is
`ConstructorPopulationOpen` — **not** "three known rows therefore class closed." This
keeps the carrier design stable whether the census finds three, thirty, or three hundred
sites.

## The `sole_constructor` audit (executed 2026-08-04)

`type_has_sole_constructor` (`v1_compiler_infer.rs`) has **exactly one call site**, on the
named-record-expression typing path.

- **Record literals only.** Any other route to a value of a sealed type is unguarded. The
  repo already records this: `std_realization_schedule.rs` states the unverified
  population "explicitly includes generic refinements and casts."
- **File identity, not module identity.** The scope test is
  `decl.span.file == span.file`. That contradicts DESIGN section 3 ("a fact's home is its
  layer, not its file") and breaks both ways under the module-identity-vs-storage lane: a
  module split across files falsely refuses; two modules in one file falsely permit.
- **Fail-open on lookup miss.** `None => Rc::new(vec![])` — a construction site whose type
  fails to resolve is silently admitted.

## Ceiling — the claimed rung is the MINIMUM of these rows

| Property | Current mechanism | Honest status |
| --- | --- | --- |
| External record literal detected | `sole_constructor` recognized call site | Mechanically preventable **on covered form** |
| Cast construction detected | none | **Open** |
| Module moves preserve constructor ownership | file-identity scoped | **Open** |
| Missing declaration lookup refuses | fail-open today | **Open** |
| All production qualification constructors enumerated | #7786 call-occurrence frontier | **Unknown** |
| Fixture origin cannot enter the known direct-door transport | parallel fixture carrier (#7761) | Closed for that path |
| Fixture origin cannot reach any production qualification anywhere | not yet proven | **Open** |

**Claimed rung: mechanically preventable for resolved record-literal construction sites
inside the currently recognized file-identity scope.** That phrasing is deliberately ugly;
it is the measured subject grain. The mechanism is not a wall and this note does not call
it one.

## Mutation contract — required before class closure may be claimed

```
 1. Raw fixture InferredTree passed to production mint
      -> type mismatch or typed origin refusal
 2. Fixture artifact cast to production carrier
      -> refused; a cast cannot bypass construction admission
 3. Origin carrier literal outside owning stage
      -> detected for EVERY supported literal form
 4. Owning module moved to another file
      -> authority stays bound to declaration/module identity,
         or migration refuses explicitly
 5. Constructor lookup fails
      -> LookupUnavailable refusal, never admission
 6. New production qualification call in an UNRELATED module
      -> census grows with no import and no taught qualified name
 7. New qualifying call with unfamiliar provenance
      -> UnknownOrigin row, merge refused
 8. Fixture and compiler artifacts carry byte-identical source
      -> remain different types; byte equality cannot erase origin
 9. Stage receipts carry mismatched subject identities
      -> chain construction refused
10. All known fixture-origin sites removed, one indirect consumer remains
      -> closure certificate remains unwritable
```

Mutation 6's planted case must contain an **actual call** to the production constructor.
It must not be detectable because its synthetic name was hard-coded — that is the defect
#7786's first two drafts carried.

## Implementation gates

```
B0  design only                        <- this note
B1  covered-form construction admission     after B0 sign
B2  call-occurrence population closure      consumes #7786 successor
B3  production API migration                raw stage outputs removed from
                                              qualification boundaries
B4  bypass closure                          casts, module identity, fail-open lookup
```

B1-B4 may be recut later; the point of naming them now is that they are **distinct proof
obligations**, and B1 alone never establishes the class.

## Out of scope

CI evidence lifecycle (route, expectation, semantic-evidence currency) is a separate
program, identified by the slug `witness-evidence-lifecycle-design` and governed by
gunbc#7778. This design does not depend on whether that bind is present in the current
merge candidate; the slug is named rather than path-linked so the statement holds under
either merge order.

That program prevents missing evidence; this one prevents convincing evidence about the
wrong thing. They join at one invariant:

```
CI requires current evidence for every witness
AND
production evidence can only be constructed from the real subject path
```

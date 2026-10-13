## Post-merge audit: `main@9474584`

I found **seven additional defects or control gaps** beyond the already-scheduled `MatchArmScopeExitEdge` repair. Two are compiler-semantic defects, one is an external integration break, and four weaken the cutover’s enforcement or preservation evidence.

I did **not** find evidence of an internal missed `tools.*`, `gunbc.product.*`, or `gunbc.namespace.*` rename. The concern that deleting an entire namespace directory could silently pass is also closed: enumeration treats an absent directory as empty, but the required layout check separately refuses a declared namespace with no occupying module.

---

## Already scheduled: match-arm scope-exit acquisition

**Severity: High**

The landed `HeadBindingUndecided` model correctly prevents a match-bound `v2.member` from becoming a false cross-tree request, but it keeps those undecided names alive through a lowered `MatchArmScopeExitEdge`. A qualified module reference in the post-match tail can therefore be suppressed and later refuse as unbound.

The follow-up should implement the boundary already specified:

- the marked bind’s value remains under the arm candidates;
- its binder and body return to the outer undecided scope;
- enclosing match layers remain;
- imported undecided names remain module-level.

---

# Additional defects

## 1. Demand acquisition performs full normalization before it has acquired the closure

**Severity: High — valid programs can fail**

`acquisition_front_end` tokenizes and parses one module and immediately invokes full `normalize`. Any normalization refusal becomes `AcquisitionMemberRefused`, stopping acquisition before the module’s dependency requests can be completed. fileciteturn411file0L2-L2

The code already acknowledges one concrete population affected by this ordering: a service declaration whose realization is provided elsewhere can fail ordinary normalization in isolation. That is why `acquisition_test_markers` has a separate parse-tree-only path—it needs to observe test markers in modules that cannot yet be normalized alone. fileciteturn411file0L2-L2

That means a valid two-module program can have this shape:

```text
module A:
  declares a service whose realization is supplied by B
  imports or otherwise references B
```

Acquisition reads A, normalizes A before B has entered the closure, and refuses A. It never reaches the complete-closure stage where the service declaration could be judged with its realization present.

### Concrete fix

Split the front end used for dependency discovery from closure-complete normalization:

```text
parse one module
→ extract imports and qualified-reference demand
→ acquire closure
→ normalize/resolve over closure
```

The request-reading pass must either:

- operate on the parse tree; or
- use a dedicated request-only lowering pass that tolerates unresolved cross-module realization facts and produces no semantic verdict beyond the request population.

Add a discriminating pair:

1. A service declaration in A whose realization is correctly supplied by B: acquisition and final compilation admit.
2. The same A with B absent: acquisition obtains the declared request and final closure validation refuses with the missing-realization cause.

Do not merely special-case services inside `normalize`; the architectural defect is using the final semantic pass to discover the inputs required by that pass.

---

## 2. Test discovery still hard-codes `v2.test`, contradicting marker-only test identity

**Severity: High — marked witnesses outside `v2.test` can disappear from the native universe**

The cutover’s operator ruling says:

> A test is a declaration carrying the authored `test` marker, and nothing else.

But native-universe construction still begins from one namespace, `v2.test`, rather than from all governed modules carrying marked declarations. The native-route authority likewise retains a `v2.test` universe prefix. fileciteturn441file0L2-L2 fileciteturn462file0L2-L2

There are marked `test fn` declarations outside that namespace—for example in workflow modules. The old filename convention had separate `*_test.dag` sidecars for at least some of this population; after those tests were folded or renamed, a namespace-restricted universe can silently omit them even though the file and marker survive.

This is exactly the failure the cutover was supposed to eliminate: test identity moved away from filenames, but the lane still applies a second, unrelated location predicate.

### Concrete fix

Build the native test universe as:

```text
all governed modules
→ parse-tree marker capture
→ every declaration carrying TestMarker
```

Then enforce:

- every captured marker yields exactly one universe entry;
- marker-read refusal is a lane refusal, never zero tests;
- duplicate `(module, declaration)` identities refuse;
- the selected universe is nonempty where the lane contract requires claims;
- no namespace predicate participates in test identity.

Add controls for:

- a marked declaration under `v2.test`;
- a marked declaration outside `v2.test`;
- an unmarked declaration inside `v2.test`;
- a malformed module containing a marker-shaped spelling;
- two duplicate marker identities.

Retire `native_lane_universe_namespace` and `native_route_universe_prefix` rather than broadening them to another hand-maintained list.

---

## 3. `acquisition_exactness` proves global set membership, not an entry-rooted closure

**Severity: Medium-High — the “exact read set” wall can pass a disconnected or over-read graph**

The verifier derives one global set from:

- entries;
- every request stored on every acquired member;
- every dotted prefix of qualified requests.

It then checks that acquired members and reads appear in that global population. fileciteturn411file0L2-L2

That does not prove the modules are reachable from an entry. For example:

```text
entry E requests nothing
A requests B
B requests A
```

If E, A, and B are all supplied as acquired members and reads, A and B mutually justify one another in the global set. The verifier can call the receipt exact even though neither is reachable from E.

Qualified-reference prefix expansion creates a second false-green shape. For:

```text
a.b.c.member
```

the lookup may probe `a.b.c`, `a.b`, and `a`. Once the verifier flattens every candidate prefix into “requested names,” it can admit both the actual winning module and a shorter losing prefix as read modules, even though one reference should settle on only one module.

### Concrete fix

Make acquisition produce an edge receipt per request:

```text
RequestReceipt {
  requester,
  spelling,
  candidates_probed,
  absent_candidates,
  selected_module?,
  selected_file?,
  verdict
}
```

Then verify by graph traversal from entries:

1. seed reached modules with entries;
2. only process requests belonging to reached modules;
3. each successful request selects at most one module;
4. selected modules become reached;
5. every read belongs to exactly one selected module or entry;
6. every acquired module is reached;
7. losing prefixes may be absent probes but never extra acquired members.

Required RED controls:

- disconnected A↔B cycle;
- disconnected self-loop;
- both the longest matching prefix and its shorter prefix read;
- an acquired module requested only by an unreachable module.

---

## 4. Module-header agreement uses a raw line scanner rather than the language parser

**Severity: Medium-High — valid formatting can be refused before parsing**

Storage agreement locates the module declaration by scanning text lines for a trimmed line beginning with the literal prefix `"module "`, then takes the remainder as the declared name. fileciteturn409file0L2-L2

That creates a second recognizer for module syntax. It can disagree with the lexer/parser over whitespace and line-ending forms—for example tabs or multiple non-space whitespace characters between `module` and the name—causing `ReferenceModuleLineMissing` or `ReferenceModuleLineDisagrees` before the real parser judges the source.

It can also evolve independently when the language grammar changes.

### Concrete fix

Use one of these, in order of preference:

1. a prepared, bounded module-header parse using the real Dag lexer and grammar;
2. a lexer-level header reader consuming the same token classes as the parser;
3. a shared header projection from the parse artifact already produced for acquisition.

Do not repair the raw scanner with more string cases.

Controls should compare storage agreement with full parsing for:

- ordinary spacing;
- tabs and mixed legal whitespace;
- CRLF;
- leading comments and blank lines;
- comments adjacent to the header;
- malformed names;
- a later string/comment containing `module ...`.

The law should be:

```text
header agreement accepts exactly when the parsed module identity equals the requested identity
```

---

## 5. The required layout check omits the generated-artifact registry join

**Severity: Medium — missing or misclassified generated modules can pass required CI**

The PR explicitly records that `RegistryArtifactMissing` was removed from the required layout check because importing the registry would widen the native CLI closure. The remaining registry-side check is a non-required claim, and several registered stage0 `.dag` artifacts are not reached by a required compile. fileciteturn414file0L2-L2

Consequently required CI can pass while:

- a registered generated `.dag` artifact is absent;
- a generated `.dag` file exists but is unregistered;
- one file is classified as both authored and generated.

The layout check proves location, but not the authored/generated ownership partition promised by the plan.

### Concrete fix

Generate an import-free compact registry projection for the CLI, containing only:

```text
relative path
module identity
generated-artifact identity
```

The required layout check should join both directions:

- every generated namespace `.dag` has exactly one registry row;
- every registered `.dag` exists at its rule-derived path;
- no governed module is both authored and generated;
- duplicate registry identities or paths refuse.

This avoids importing the full registry domain into the CLI while preserving the required invariant.

---

## 6. Witness identity observation can collapse duplicate actual rows before the join

**Severity: Medium — duplicate witnesses can be hidden**

`layout_witness_join` indexes observed witness rows by identity using map insertion. I did not find an explicit duplicate-observation refusal before that insertion. fileciteturn422file0L2-L2

If two declarations produce the same witness identity, a last-write map can collapse them into one row. The old/new preservation join may then report one preserved identity rather than:

```text
duplicate witness identity
```

That weakens both migration evidence and the permanent identity wall.

### Concrete fix

Replace direct map construction with a typed unique-index fold:

```text
WitnessIndexBuild
  = WitnessIndexReady
  | DuplicateWitnessIdentity {
      identity,
      first_locus,
      second_locus
    }
```

Apply it independently to:

- old observations;
- new observations;
- declared removals;
- declared changes;
- unread declarations.

Add controls for:

- duplicate identity with identical content;
- duplicate identity with different content;
- duplicate declarations on only one side;
- duplicate unread/file rows.

No duplicate should be treated as preservation.

---

## 7. Witness body preservation does not preserve the semantics the witness depends on

**Severity: Medium — witness can be reported unchanged while its meaning changes**

The marker observer captures the marked declaration’s own textual content. fileciteturn436file0L2-L2 That is useful for detecting direct body changes, but it excludes semantic dependencies such as:

- import target changes;
- changes to imported names;
- changes to local helper declarations called by the test;
- changes to constants or fixtures read by the test;
- a provider module being renamed or replaced.

Therefore the identity join can report a witness as preserved while the same textual declaration now executes against different helpers or providers—or no longer resolves once claims are actually run.

### Concrete fix

Keep the raw declaration digest, but add a second semantic digest derived after resolution:

```text
WitnessSemanticIdentity {
  declaration_identity,
  normalized_declaration_digest,
  resolved_reference_edges,
  reachable_local_helper_digests,
  imported_provider_identities
}
```

The migration join should distinguish:

- text preserved and semantic closure preserved;
- text preserved but dependency closure changed;
- body changed;
- lost/gained.

This need not recursively hash the entire compiler closure. It should follow the declaration’s resolved reference graph to the bounded helper/provider population that establishes the claim.

---

# Integration and evidence defects

## 8. The public cutover landed while known private consumers still referenced renamed modules

**Severity: High operational break**

The merged PR states that `gunb-ai/gunbc-private` imports fourteen renamed modules plus one renamed test module, and that a repoint PR must land immediately after the cutover. fileciteturn414file0L2-L2

I did not find a visible open private repoint PR during this audit. The connector’s private-repository search may be incomplete, but the public PR’s own disclosure is sufficient to establish that the public merge knowingly broke those consumers unless the private change was coordinated out of band.

### Concrete fix

Land an exact private repoint immediately and add a permanent external-consumer contract:

```text
PublicModuleRename {
  old,
  new,
  external_consumers
}
```

For future authority migrations, either:

- merge external consumers before or atomically with the public cutover; or
- pin those consumers to the old public revision until repointed.

A public PR should not be marked complete while its known external-consumer action remains only prose.

---

## 9. The full witness-preservation run was not reproducibly executed from the committed observer

**Severity: Medium evidence gap**

The PR discloses that the committed full-population observer was killed after roughly 45 minutes. Marker facts and declaration digests were instead supplied out of band and only sample-cross-validated against the committed reader. fileciteturn414file0L2-L2

This is not proof that a particular witness was lost, but it means the strongest “zero unexplained witness loss” conclusion is not reproducible from one committed command and one exact source revision.

### Concrete fix

Make observation natively sharded:

```text
ObservationPartition {
  population_digest,
  partition_index,
  partition_count,
  first_key,
  last_key,
  rows
}
```

The aggregator must prove:

- all partitions share the same population digest;
- every partition number appears exactly once;
- key ranges do not overlap;
- union of partition keys equals the enumerated module population;
- every governed file has FILE, UNREAD, or observed declaration rows.

Run that aggregate in required CI or as a required, attached landing receipt.

---

## 10. Generated plan documentation was stale at the landed semantic head

**Severity: Medium**

The final blocker fix changed the source plan projection to the three-state `HeadBindingUndecided` design. fileciteturn365file0L2-L2 The checked-in Markdown plan still described the prior behavior in the version inspected during this audit. fileciteturn445file0L2-L2

That happened because no required CI step compares all generated documents with their authorities; the PR itself discloses that fixed-point generation is manual evidence. fileciteturn414file0L2-L2

### Concrete fix

Regenerate `docs/plans/source-layout-authority.md` from `gunbc.plans.source_layout_authority` immediately.

Then restore a cheap required fixed-point check for generated governance artifacts:

```text
generate into temporary tree
→ compare hashes against committed outputs
→ refuse any difference
```

It need not rebuild expensive native products. The current absence of any required projection check allows an authority and its human-facing plan to disagree on the exact behavior under review.

---

## 11. The cutover was merged before exact-head required checks completed

**Severity: High process/control defect**

The final source head’s `witnesses` and generated-artifact checks were still running when the merge occurred. The preceding head had green results, but the final head changed load-bearing compiler and resolver code. A successful earlier-head run is not evidence for that delta.

This is especially material because one post-merge claim run had already found a real non-exhaustive match that the gate had missed, and the final blocker repair changed `resolve.dag`, `lexical_scope`, acquisition, and test claims.

### Concrete fix

Configure branch protection or the merge queue so that:

- approval is bound to the exact SHA;
- required checks must be green on that exact SHA;
- a merge commit cannot be created while checks are pending;
- emergency override requires an explicit incident/fix-forward record and prevents downstream release until the exact merged tree is validated.

“Main is frozen” is not a substitute for exact-head evidence.

---

# Priority order

The fix-forward order I recommend is:

1. **MatchArmScopeExitEdge**, already assigned.
2. **Closure-before-normalization acquisition**, because it can reject valid multi-module programs structurally.
3. **Global marker-derived test universe**, because otherwise witnesses can silently disappear from the native lane.
4. **Private consumer repoint**, if it has not already landed.
5. **Entry-rooted per-request acquisition receipts**, replacing the current global-set exactness proof.
6. **Grammar-grounded module-header agreement**.
7. **Required generated-artifact registry join**.
8. **Duplicate-safe and semantically stronger witness observation**.
9. **Reproducible sharded identity join and required generated-document fixed point**.
10. **Exact-SHA merge protection**.

The two highest-risk semantic themes are now clear: acquisition is still doing semantic work before it owns the complete dependency population, and test discovery still carries a location predicate after the cutover declared marker identity authoritative. Those should be treated as direct continuation of the cutover, not unrelated cleanup.

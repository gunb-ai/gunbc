# Scoped keying relations

**Operator-agreed design, 2026-08-15; no implementation authorized by this note.** Name the relation and scope that make a key meaningful, then migrate a real consumer. Existing typed owners absorb this design as it lands; do not maintain a parallel ledger. [Original design, specimens and rulings](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/keying-relation-design.md).

## Goal and existing homes

`std.keyed_row.KeyedRow`, `keyed_roster_build`, `std.change.keyed_two_way_diff` and `membership_reconcile` accept anonymous key/equality functions. They do not declare subject, scope, grain, relation, multiplicity or upstream grounding. Stable identity versus content identity therefore remains a prose convention.

Keep `SubjectKey`, `StateRevision`, `ResourceLocator`, `ContentIdentity` and `DisplayLabel` distinct. A Git ref is keyed by repository × ref name; its OID is a value. The same OID may identify content in another relation; a URL is not repository identity. Cross-family content fingerprints yield typed incomparability, not ordinary inequality, and occurrence identity belongs to its own scoped relation.

Own external operation discriminants in `extdeps`, generic scope/equality/composition in `std`, product subjects above them, and reconciliation above those subjects. Examples: Tailscale listener × mount, tmux server × session, systemd manager × unit, Git repository × ref, GitHub App × installation, HTTP namespace × protocol × address × port. Credential binding is a relation, not a host pathname.

The first concrete specimen is Tailscale serve: listener × mount identifies the subject, backend is payload. `live_deploy` currently lets rendered argv `NonEmptyStr` masquerade as that path. Dashboard isolation similarly needs owner/borrower/credential/concurrency/mutation relations, not identity derived from a host-live credential path.

The earlier 23-constructor census is **retracted**. `flagship_belt_observe_workdir` deliberately uses host-global `/` for tmux observation/teardown, distinct from instance-local spawn. A raw singleton read does not prove a defect; neither a corrected count nor a broad cleanup is authorized by that census.

## Relation and cardinality

The proposed minimal vocabulary is:

```
ScopedKey<Scope, Key> { scope, key }
KeyMultiplicity = AtMostOne | ExactlyOne | ZeroToMany
KeyRelation<Subject, Scope, Key> {
  scope_of, key_of, scope_eq, key_eq, multiplicity
}
```

A subject may have multiple named scoped relations. Introduce only what the first consumer needs; replace anonymous function pairs with declared relations, not a universal identity function. Multiplicity is declared by the subject's contract, not discovered by whichever collection happens to hold it. `std.map` uniqueness and keyed-roster duplicate refusal are particular policies; a multimap is a separate missing home.

JSON (RFC 8259) permits duplicate authored members. Preserve the authored list and distinguish `JsonMemberLookup` Found / Absent / Duplicated / NotObject rather than choosing a winner or calling a scalar “missing.” Cardinality should remain structural through containers: a conversion must not silently forget the nonempty guarantee.

A construction wall can check declared grain once the carrier supplies the needed instance/scope. It cannot decide whether the relation is the world's correct one; that remains a ratcheted modeling obligation. A single-variant coproduct can be a brand, sealed type or pinned union. Regex/counting such shapes is not a defect metric.

## Infer-facts site relation: agreed correction, still unlanded

The 2026-10-01/04 ruling gives `v2.compiler.infer` facts a separate, concrete relation. The `eager-newt-412/facts-site-key` branch is orphaned, not main; `InferSite` / `InferFactsTrie` are not landed. Main's `inferred_facts_map_enter` still chooses first structural `Node` key. The consumed rung-drop `infer_facts_key_conflict_reverted_to_first_wins` records the regression; no owner is assigned. This does not gate the v1 derived-identity step below.

Childless `Conj` qnames or integer digits at different sites can have equal structural keys. #12582's conflict refusal could not separate that case from a true same-site conflict and was reverted by #12852. The repair's contract is:

- A fact is about a site **in one `InferredTree`**. Scope by that root's content hash; lookup under another root refuses. No persistence or identity across trees, runs or stages is claimed.
- A site key is the root-to-node edge path of `(label, ordinal)`, where ordinal is position among **all parent children**, including equal subtrees and repeated labels. Paths are injective within that root; equality compares paths within the same scope.
- Same-site different facts refuse with located `infer_facts_key_conflict`; same-site same facts merge; equal facts at different sites remain distinct entries.
- This is not authored occurrence identity. Normalization alone allocates occurrence identity; resolve/later nodes are `OccurrenceSynthetic`, and inference sees the resolved tree. Do not invent authored provenance to repair the facts relation.
- Every reader takes a site: eval threads it through calls and callee-body child edges in the same tree; lenses and `program_partition` produce it while traversing. Leave no `Node`-only facts lookup. A shared fixture constructor resolves a supplied path in the named root and preserves the supplied input.
- Builders must not reuse one node object for two meanings. The `E1Arrow` empty-`Conj` children control requires distinct objects/sites.

A future A′ identity for every infer-input node requires builders to name each cause. Its trigger is a consumer needing identity beyond these facts (for example synthetic provenance across stages); it is **not** an extension of #12790 for convenience.

## Sequence and validation

1. Ground Tailscale listener/mount keys on the existing operation model; introduce only the required scoped-relation machinery.
2. Make `membership_reconcile` consume the named relation, with explicit exactly-one/many outcomes and no invented winner; retain the independent v1 derived-identity step.
3. Derive dashboard logical-replica identity from its product relations after the first consumer proves the seam.

Validate same-key payload changes, distinct scopes, duplicate/many subjects, cross-family incomparable fingerprints and each consumer's refusal. Infer-facts adoption additionally needs same/different-site controls, root mismatch, builder-site distinction and all production readers consuming paths; a fixture alone does not close the migration.

Keep neighboring concerns separate: frame/envelope authority differs from `DashboardObserveOnly` (belt dispatch, not provisioning); fleet identity is declaration/allocator-owned, not unit-path content; storage paths can remain load-bearing realization edges without becoming subject identity.

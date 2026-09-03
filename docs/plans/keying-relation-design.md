# Keying as a modelled relation

**Status:** design note, no code lands from it yet. Operator-agreed 2026-08-15 that this
becomes a note before any implementation.

**Thesis:** `std` models keyed *collections*, not *keying*. Every defect below follows from
that gap, and several were already fixed locally, one lane at a time, without naming the
shared cause.

---

## 1. What already exists, and precisely where it stops

The container and diff machinery is real and works:

- `std.keyed_row` `KeyedRow<K, V>` — a key beside a value.
- `std.keyed_roster` `keyed_roster_build` — refuses duplicate keys.
- `std.change` `keyed_two_way_diff` — derives Added / Removed / Modified.
- `gunbc.membership_reconcile` — converts a domain population into keyed rows and
  invokes the generic diff.

All of them receive

```dag
key_of: fn(M) -> K
key_eq: fn(K, K) -> Bool
```

**from the consumer.** So none of them models:

- where the key came from;
- the scope within which it is unique;
- which identity relation it denotes;
- whether it is too coarse (two subjects collapse) or too fine (it embeds mutable
  state, so an ordinary update reads as Remove + Add);
- whether it is a subject identity, a state revision, a locator, a content hash, or a
  display label;
- whether two observation surfaces are using the same key;
- whether the key is grounded in an upstream operation at all.

**The load-bearing law is prose-only.** `membership_reconcile` states that `key_of` must
return stable identity rather than content — otherwise content drift reads as Remove+Add
instead of Modified. A generic receiving `key_of` from its caller cannot enforce that; the
reconcile spine's most important rule is a comment.

Local precedent exists: `std.occurrence_identity` says an `OccurrenceId` is unique only
inside one allocator scope and forbids filename, span, spelling, structure and content hash
as identity inputs — exactly the scoped-key law that should be general, not restated per lane.

## 2. Keying is a relation, not an `id` field

A key answers:

> Under a **named relation**, inside a **named scope**, when do two observations denote
> the same subject?

One subject legitimately participates in several non-interchangeable relations at once:

```
entity identity · resource-update identity · membership identity
state revision · content identity · transport locator · display name
```

Git makes the distinctions unavoidable:

```
ref key      = repository x ref name
ref value    = object id
content id   = object id            (the same bytes, a different relation)
remote URL   = locator, and NOT repository identity
```

So "the key of X" is not well formed. "The key of X under relation R in scope S" is.

## 3. Verified specimens

Each was read on `main` while writing this note; none is recalled.

### 3a. A structured key discovered, then discarded — `extdeps.tailscale.serve`

The model correctly derives route identity from what the upstream `off` operation
requires — listener port x mount — and deliberately excludes the backend, since changing
a backend repoints the *same* route. That hard-won reasoning is then thrown away:

```dag
fn tailscale_serve_endpoint_key(endpoint: TailscaleServeEndpoint) -> NonEmptyStr {
  join(tailscale_serve_endpoint_flags(endpoint: endpoint), " ") as NonEmptyStr
}
```

Route identity becomes **a rendered CLI argv string**, filed by its consumer in
`gunbc.live_deploy.spec` as a `path:` field on a `DeploymentArtifactStep`. Three category
errors on a correct insight: a subject key rendered to a locator, spelled as argv, which
DESIGN section 3 separately rules a nickname for the dependency's semantics.

### 3b. An impostor key as the whole defect — dashboard provider state

`gunbc.roadmap_dashboard_instance` `isolated_dashboard_instance` isolates source,
binary, origin, dispatch roots, tmux server and port, then binds provider state to
`layout.roadmap_codex_home` — the host's live credential store.

The usual reading is "the coproduct has only one arm." The keying reading is stronger:
**a path (`ResourceLocator`) is standing in for provider-state identity (`SubjectKey`).**
That is why adding an isolated arm beside the existing one does not fix it — the impostor
stays nameable, so the wrong construction stays reachable. The correction is a relation
carrying owner, borrower, credential identity, concurrency ceiling and mutation posture,
not a better path.

### 3c. Two local fixes for one missing primitive

The repository has hit key-soundness twice and solved it in place both times:

- `std.content_hash` grounds `ContentHash` as a family coproduct so cross-family
  comparison returns `ContentHashCrossFamilyIncomparable` rather than a Bool.
- `std.occurrence_identity` and `gunbc.host_resource` are kept as separate carriers
  specifically so `occurrence_id_eq` cannot answer `true` on a numeric coincidence
  across two subject spaces.

Same law — keys from different spaces must not silently compare — fixed twice, per
family, because there is nowhere to state it once.

### 3d. Grain failures are keying failures

**Retraction (2026-08-15).** An earlier revision of this section asserted a census of
"23 constructors that take an isolation parameter and then reach past it to a module-level
singleton", with six `*_for_instance` functions in `gunbc.roadmap_belt_actuate` reaching
`belt_observe_workdir` as its flagship. **That flagship is a false positive and the count is
withdrawn.** `belt_observe_workdir` is deliberately `/` — a host-global execution workdir for
tmux observation and teardown — and `gunbc.roadmap_belt_actuate` `belt_actuate_workdir_note`
explicitly distinguishes it from the instance-specific spawn workdir, calling their collapse a
§5 fail-open. The detector answered *function accepts an instance parameter and also reads a
module singleton*, which is **not** the question: it cannot tell a correctly host-global
singleton from one keyed too coarsely, so it establishes reach, not defect, and no repo-wide
cleanup follows. The count is deleted rather than corrected because a sound detector needs the
fact this document argues does not yet exist: a declared key scope to compare the reach against.

The claim itself survives, carried by the two specimens verified by reading their subjects
(§3a, §3b) rather than by a count: a fact reached at a coarser scope than its consuming subject
is keyed by **nothing** (arity-zero — a module singleton) where the subject is keyed by
instance. "Wrong grain" and "wrong key arity" are one statement, and the refuting specimen is
why the distinction must be *declared* before it can be *checked* — today the correct and
defective cases are byte-identical in source.

## 4. Where each layer's authority sits

Causality runs upward from reality, not downward from a product taxonomy:

```
upstream reality
  -> extdeps exposes the resource's real keying relation
     -> std models key scope, equality, composition, keyed operations
        -> product/workflow composes those keys into its subjects
           -> reconciliation consumes them
```

This note originally got it backwards: the dashboard's logical-instance / replica /
posture split is a **conclusion** of asking what the external systems treat as the same
addressable resource — derived from keying, not introduced first as a product taxonomy.

### The `extdeps` rule

> If an upstream interface can address two resources independently, the model must
> expose the typed discriminants that make that possible.

The external API states its key through the operation that addresses, updates or removes
a resource. Current flattenings, each a candidate first consumer:

```
Tailscale Serve mapping   listener x mount
tmux session              server scope x session name
systemd unit              manager scope x unit name
Git ref                   repository x ref name
GitHub installation       App x installation id
HTTP listener             netns x protocol x address x port
Codex account state       credential/account binding, not a path spelling
```

### The `std` surface

Grounded by real consumers, not authored as a speculative framework:

```dag
type ScopedKey<Scope, Key> {
  scope: Scope
  key: Key
}

type KeyMultiplicity = KeyAtMostOne | KeyExactlyOne | KeyZeroToMany

type KeyRelation<Subject, Scope, Key> {
  scope_of:     fn(Subject) -> Scope
  key_of:       fn(Subject) -> Key
  scope_eq:     fn(Scope, Scope) -> Bool
  key_eq:       fn(Key, Key) -> Bool
  multiplicity: KeyMultiplicity
}
```

The load-bearing change: the relation becomes **a named value**, not an anonymous pair of
caller-supplied functions. A consumer names *the declared relation under which this
population is reconciled* instead of passing `key_of` — which also lets one subject carry
several relations without pretending to a global primary key.

### Multiplicity belongs to the relation

A relation is **(subject, scope, key, multiplicity)** — how many values one key may
denote: zero, one, or n. Today that axis is implicit and inconsistent, so 1:1 is an
assumption rather than a declaration:

- `Map<K, V>` is 1:1 by construction.
- `std.keyed_roster` `keyed_roster_build` treats a duplicate key as an ERROR, carrying
  `KeyedRosterDuplicateEvidence<K, V>` and `keyed_roster_locate_duplicate`.
- There is no multimap, `group_by`, or 0..n keyed structure anywhere in `std`.

So a domain whose upstream permits many values per key has no shape to inhabit; its author
must fabricate a winner or route around the roster.

**The one place it is modelled correctly is JSON** — copy it rather than re-derive. RFC 8259
permits duplicate member names without saying which wins, so `extdeps.languages.json.parse`
keeps members as an authored list and reports multiplicity as its own outcome:

```dag
type JsonMemberLookup
  = JsonMemberFound { value: JsonValue }
  | JsonMemberAbsent
  | JsonMemberDuplicated { count: Int }
  | JsonMemberNotAnObject

fn json_object_members_named(members: List<JsonKeyValue>, key: String) -> List<JsonValue>
```

Its note's reason generalises exactly: a last-wins fold silently picks a value the document
never committed to, and for an identity field like a verdict, last-wins is how a document
carrying two verdicts reads as whichever an attacker put second. `Absent` and `NotAnObject`
are separate arms — a scalar has not *omitted* a field, it has no fields — the same
state-space discipline one axis over.

Two consequences for the key model:

1. **Multiplicity is declared, not discovered.** A relation states 0..1, exactly 1, or
   0..n, and lookup is total over what it declared. `keyed_roster_build`'s duplicate
   refusal is then a legitimate *policy* for relations declared unique, not a law imposed
   on relations that were never unique.
2. **Cardinality is already meant to be structural.** DESIGN lists "a possibly-empty
   collection flowing into a nonempty consumer" among the classes the substrate carries in
   the type. Keying is where that fact is dropped: the relation knows the cardinality and
   the container throws it away.

### Separating the impostors

```dag
SubjectKey<K> · StateRevision<R> · ResourceLocator<L> · ContentIdentity<H> · DisplayLabel
```

Not all need become wrappers at once, but the distinction belongs in the standard key
model. Specimen 3a is a `SubjectKey` rendered as a `ResourceLocator`; specimen 3b is a
`ResourceLocator` standing in for a `SubjectKey`.

## 5. Decidability — what can become a wall, and what cannot

Per DESIGN section 5, "never" is the trap, so each class is classified before anything
is promised:

- **Wall now.** A body referencing an instance-scoped fact without an instance. Decidable
  once the fact lives on its grain-carrier; the error is an ordinary resolution or
  type-argument mismatch, not a new checker.
- **Wall after grounding.** A concept with no declared relation. Decidable once the
  relation is declared once, as a single authority — the `Vendor<Domain>` move applied
  to identity.
- **Ratchet forever.** Whether the *declared* relation is correct. Whether a workdir is
  truly per-instance or genuinely host-global is a claim about the world; no compiler
  derives it, so it stays modelling judgment, made once per concept.

So: a compiler error for key *violations*, permanently. Not for key *mistakes*.

**Explicit non-goal.** A one-variant coproduct is not itself a defect — it may be a
branded wrapper, a sealed constructor, or a pinned upstream union with one current member.
An early draft treated a regex count of such types as a defect map; that was overreach and
the count is not a correctness metric. The enforcement target is resource ownership and
lifecycle grain.

## 6. Sequencing

Nothing here is built until the first real consumer needs it.

1. Pick one `extdeps` specimen and expose its typed key — Tailscale Serve is cleanest:
   the correct discriminants are already derived and only the carrier is wrong.
2. Introduce `ScopedKey` / `KeyRelation` in `std` only as far as that consumer requires.
3. Convert `membership_reconcile` to take a declared relation instead of an anonymous
   `key_of` / `key_eq` pair, turning its prose law into a checked one. Its 1:1 assumption
   becomes an explicit `KeyExactlyOne` declaration, and a `KeyZeroToMany` population stops
   needing a fabricated winner.
4. Only then revisit the dashboard split, deriving logical-instance vs replica from the
   relations rather than asserting it.

Dissolution: this note retires into the carriers it names as each step lands, per DESIGN
section 6 — the mark on the carrier is the authority; design notes are not a parallel ledger.

## 7. Related threads

- Effect grants over namespaces — the `Frame` is where ambient axes belong, and the
  execution envelope is where a missing *plan* value belongs (`Hermetic | Wet | Record`
  has no live-read/no-write arm today; `DashboardObserveOnly` is a served-dashboard
  actuation posture that gates belt dispatch only, and does not constrain provisioning).
- Fleet-reconcile spine — already rules identity declaration-owned and allocator-minted,
  never derived from unit, path or content, because inferring identity from content
  similarity is the heuristic DESIGN section 4 rules out. Keying doctrine, stated once,
  for one lane.
- Module identity vs storage — a load-bearing path literal is an edge living in prose;
  the same defect in the storage direction.

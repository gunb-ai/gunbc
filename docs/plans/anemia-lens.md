# Anemia lens — can a lens mechanically diagnose the leaf-side of decomposition?

Node: `node://adhoc-21adf1c1-4ae` · **DESIGN DOC for joint review — not an implementation.**
Anchor: DESIGN.md §2 (deep decomposition: `decompress → map → reduce`; "a `String` leaf hiding named
parts is anemic modeling"; "net concepts must not grow by re-invention"), §6 (enforce with lenses,
not grep — a lens is a pure reader over the `Node` tree, storing nothing), and the **structural law**:
a lens reads **shape** (which constructor / `Node` kind), **never contents** (which characters) — no
lens may substring-scan a value for a verdict.

This resolves the DESIGN.md open thread *"can a lens mechanically diagnose the leaf-side of
decomposition (§2)?"* (operator-parked). The recommendation: **yes, in part** — three structural
signals are mechanically decidable today; one residual class is irreducibly a judgment call (and is
the natural home of a future cited-grammar oracle).

---

## 1. The problem

§2's deep direction says nothing is opaque that isn't *genuinely* atomic. The **sum-side** of that
already has a lens — `v2.lens.fact_density` rejects a `NoFact` carrier (a type with zero named fields
= hollow alias). Its **leaf-side dual** has had no lens: a field typed `String` / `Symbol` / `Int`
that actually stands for named parts the type fails to name. The canonical example is in DESIGN.md
itself — `socket: "LGA4926"` that is really `CpuSocket { package, contact_count }`.

Left unenforced, anemia is the *root* of a recurring downstream disease: because the structure is
absent from the type, every consumer that needs a part must recover it by **cracking the string open**
(split / slice / `contains` / parse). That is precisely how the #5132 substring-lens trap was born —
`EmbeddedShellExecFact { script_text: String }` forced its lens to scan characters, the §2 failure
turned on the lens itself. So an anemia lens is upstream of the structural law, not a peer of it.

## 2. The hard constraint, and the reframe

The naïve detector — *"a `String` whose **values** parse as a known structured type (URL, semver,
target-triple)"* — reads the leaf's **characters**. The structural law forbids exactly this. So the
lens may never inspect the value.

The reframe that escapes the trap: **anemia is not a property of the value; it is a property of the
program around the value.** A genuinely atomic leaf is treated *whole* everywhere — passed through,
stored, equality-compared. An anemic leaf betrays itself because *the structure it hides has to live
somewhere*, and the only contents-free places it can live are all **declared program shape**:

- in **another type that already exists** (the concept is modeled — the field just points at `String`
  instead of at it);
- in a **finite set of literal constants the program compares the leaf against** (a closed enum wearing
  a `String` coat);
- in a **string-decomposition op some consumer applies** to recover parts.

Each is read off the `Node` tree, never off the value. These are the three signals below.

## 3. The three mechanically-decidable signals

### Signal A — existing-authority coincidence (the `map`/nickname signal)

A kernel-ambient leaf field whose concept **already has a grounded authority** in the tree. This is the
strongest signal because the §2 "net concepts must not grow by re-invention" test passes for free — we
are not minting, we are *pointing at what exists*.

Concrete, live corpus:

| anemic site | existing authority | layer fact |
|---|---|---|
| `Endpoint.base_url: String` (`extdeps/cloud/cloud.dag:74`) | `type Url = String` branded, with coercion chain `[Url] < [NonEmptyStr] < [String]` (`std/types.dag:270,361`) | `Url` is a real type; the field is a nickname for it |
| a `digest: String` / image-ref hash | `type ContentHash = NonEmptyStr where brand("ContentHash")` (`std/types.dag:346`) | branded authority exists |

Decidable from shape alone: the field's declared **type** is a kernel-ambient atom (`String` / its
brand-erased ancestors) **and** a type whose brand/name names the same concept exists. No value is read.
The field *name* is consulted, but a name is declared program shape, not runtime contents — so this
stays inside the law. (Name-coincidence is a heuristic both ways; see §5 guards and §6 tiers.)

### Signal B — closed-set-by-comparison (the enum-as-`String` signal)

A leaf that the program only ever **equality-compares against a finite set of literal constants**. The
set of comparison literals is a *static program fact* (the shape of the consumer's `==` / `match`
arms), not the runtime value. A `String` compared against N fixed literals and used no other way **is**
a closed enum with N variants — "a default reveals a closed set."

The corpus shows this pattern already *decomposed correctly*, which is the cleanest proof it is
real and mechanical: `extdeps/container/oci/types.dag` lexes OCI architecture/os at the wire boundary —
`parse_image_config_goarch(raw) { if raw == goarch_amd64_wire {Amd64} else if raw == goarch_arm64_wire
{Arm64} ... }` over a fixed table of `data goarch_*_wire: String` constants, producing an `Os` / arch
**enum** internally with an `…Unknown { wire: String }` escape. The signal is: the universe of literals
a leaf is compared against is finite and enumerable from the tree → the leaf is that enum. Where this
has *not* been done (a raw arch/os `String` stored as a fact and branched on inline) the same finite
comparison set is the violation witness.

### Signal C — consumer-cracks-the-leaf (the `decompress` signal)

A consumer applies a **string-decomposition op** — split / substring-slice / `contains` / index-of /
char-fold / parse — to the leaf to recover parts. The *existence* of that op (its `Node` kind) is the
contents-free proof that the leaf has internal structure the type omits. This is the general form of
the part-number / target-triple / path cases. The motivating live instance is the #5132
`EmbeddedShellExecFact { script_text: String }` trap — a stored `String` whose consumer was forced to
character-scan it (`mktemp` / heredoc detection), the §2 failure made concrete; the cracking op is the
witness. The terminal fix is always the same shape — flip the leaf's **type** so the cracking op
becomes unrepresentable ("the terminal form is usually a TYPE, not a lens").

**Not to be confused with `host_language_transport_script` (read with §7).** That lens is *not* a
Signal-C detector. It does not scan for a consumer cracking a leaf; it classifies the **producer-side**
parse-tree shape of the `script:` argument at `shell.Exec.Run` call sites — `BareStringLiteral` /
`LetBoundStringLiteral` / `StringInterpLiteralsOnly` / `ComputedApplication` — and REDs on a literal
blob (`host_language_transport_script.dag:26-31, 47-54`). That is the **interim migration guard** for a
leaf *already known* to be anemic (the bash `script:` case), not a way to discover one. It belongs to
the enforcement side (§7), not the detection taxonomy.

## 4. What is NOT mechanically decidable (the residual)

A leaf that is (a) never compared to a closed set, (b) never cracked in-tree, and (c) whose *specific*
concept has no in-tree authority — yet *does* have structure in the world.

First, the boundary cut precisely, because the corpus makes it subtle. An **exact** version *is* already
covered: `type SemVer = String` exists (`std/types.dag:272`, itself a brand-deferred alias awaiting
`where pattern("^\d+\.\d+\.\d+")` post-Q-Regex) and is consumed (`OciRuntimeVersion = SemVer`,
`oci/types.dag:162`). So a field holding an exact version is **Signal A** — a nickname for `SemVer` —
*not* a residual. The doc earlier overstated this as "no authority exists"; corrected.

The genuine residual is a **version constraint**, which `SemVer` (a single point) does **not** cover:
`CargoDepSource.RegistryDep { version: String }` (`extdeps/rust/cargo.dag:54`) is a Cargo *version
requirement* (`^1.0` / `>=1.2, <1.5`, caret-by-default per the cited Cargo Reference) — a grammar of
*operator + SemVer bound(s)*, a relation over `SemVer` with no in-tree authority. It is a residual on
**both** counts: (c) no `VersionConstraint` / `VersionReq` type exists, and (b) within the repo the
field is merely stored and handed to `cargo`, so the program **never depends on its parts** and no
signal fires. (`ServiceEndpoint.version` / `GcpApiEndpoint.version` at `cloud.dag:75` / `gcp.dag:114`
are a *third* concept — API version labels like `v1`, neither `SemVer` nor a constraint — a separate
missing authority; omitted here to keep the example clean.)

Here the lens is *correctly silent*: it cannot know the constraint grammar without either reading the
characters (banned) or being told the grammar. The two honest resolutions:

- **Human judgment** — the modeler knows a Cargo requirement has parts and mints `VersionConstraint`
  *over* the existing `SemVer` (a *map*-then-mint: reuse `SemVer` for the bounds, mint the constraint
  relation — net concepts grow by **one**, legitimately, per §2's "reuse on proven coincidence, else
  mint"; this is **not** re-invention, since no constraint authority exists).
- **Oracle (future)** — a *cited grammar* in `extdeps/` (the Cargo version-requirement grammar modeled
  as a type) turns this residual into Signal A: once `VersionConstraint` exists, every `version:
  String` requirement becomes a nickname for it. The oracle does not read the leaf; it adds the
  authority the leaf should point at.

This is the boundary the doc most wants reviewed: **the lens flags "structure the program already
depends on"; it cannot flag "structure that exists in the world but the program ignores."** The latter
is a modeling decision, not a mechanical defect.

## 5. False-positive guards (each structural)

A `String` is **not** anemic — and the lens must stay green — in these cases, all detectable from shape:

1. **The `Other` / `Unknown` escape payload.** The `String` payload of a *fallback variant* of a
   coproduct whose siblings are modeled is the deliberate open-world escape, not anemia:
   `OciMediaTypeOther { raw: NonEmptyStr }` (`oci/types.dag:171`), `…Unknown { wire: String }`,
   `PrerequisiteKind … | Other` (`std/behavioral.dag:21`). Guard: leaf is the payload of a sum's
   catch-all arm. (This is the by-design extensible-registry escape the brief names.)
2. **Wire-boundary parse INPUT vs stored FIELD.** The `raw: String` *argument* to a parse function is
   correct — it is the un-decomposed wire form being decomposed *right there*. The violation is storing
   the wire form as the durable fact and re-cracking it downstream. Guard: distinguish a transient
   parse-input at the realization boundary from a persisted field. (The OCI parser is green; a `Fact {
   arch: String }` that other code re-branches is red.)
3. **Grounded cited external identifier (the SKU rule).** A `String` that is a cited upstream key with
   no further modeled structure is genuinely atomic *at our layer* (the rename test / below-boundary
   opacity, §3). Guard: citation present + only atomic consumers + no existing authority. A vendor SKU
   row is not anemic.
4. **`String`-as-subject (pipeline subject).** Raw source text consumed by the tokenizer *is* the
   subject, not a fact field. Guard: the leaf is the pipeline's input, not an attribute of a modeled
   entity. (This is the chief over-fire risk for Signal C and must be calibrated before any ratchet.)

## 6. Proposed lens shape

A member of the **fact-cardinality family** (one invariant: *every fact has exactly one grounded
authority*; an atomic leaf's authority is itself, an anemic leaf's authority lives elsewhere). It is a
pure reader storing nothing, the leaf-side dual of `fact_density`.

- **Extractor** — projects an `AnemicLeafFact` per kernel-ambient leaf field off the `Node` tree,
  carrying: the field's declared kind; whether an existing authority's brand/name coincides (Signal A);
  the finite comparison-literal set, if any (Signal B); the consuming op kinds (Signal C); and the
  guard facts of §5 (is-fallback-payload, is-parse-input, is-cited, is-subject).
- **Verdict tiers** (confidence, not binary — this is what makes it safe to run before it gates):
  - **HardViolation** — a signal fires and every §5 guard is clear (e.g. `base_url: String` while `Url`
    exists and it is not an `Other` payload). Mechanically certain; correction names the target type.
  - **SoftCandidate** — weak/ambiguous (name-only Signal A, or Signal C on a possible subject). Surfaced
    for human review, never auto-gated.
  - **Atomic** — a §5 guard matched. Green.
- **Output** — `std.lens_verdict.LensVerdict` (`Holds` / `Violation { diagnostic }`), as
  `host_language_transport_script` already returns (`:100-108`). Note `fact_density` instead returns
  `Outcome<Witness<Node>>` (`fact_density.dag:96`) — so the lens family is *not* yet unified on one
  verdict carrier; converging them onto `LensVerdict` is a follow-up that `std.lens_verdict`'s own
  `dissolve-on` already anticipates. This lens should be authored on `LensVerdict` from the start.
- **Diagnostic** — the §2 `map` strengthens the correction: not just "decompose," but "decompose onto
  `<existing type>`" when Signal A fires (the reduce step is then free).

Illustrative shape only (NOT to be enrolled by this doc):

```
type LeafSignal = AuthorityExists { target: Symbol }   // A
               | ClosedComparisonSet { arity: Nat }    // B
               | CrackedByConsumer                     // C
type LeafGuard  = FallbackPayload | ParseInput | CitedAtom | PipelineSubject
// HardViolation iff (some LeafSignal) && (no LeafGuard); reads node kinds, never characters.
```

## 7. Relationship to existing lenses (no new authority)

- **`fact_density`** — sum-side (`NoFact` hollow alias). This lens is its **leaf-side dual**; together
  they make §2 decomposition structurally covered in both directions.
- **`host_language_transport_script`** — **not** a detector to generalize (it is not Signal C). It is
  the *interim enforcement guard* for one leaf already known to be anemic (the bash `script: String`):
  it reads the producer-side arg shape (literal-blob = unmigrated) and dissolves on a type flip
  (`emit(intent, Bash)`). Its relevance to this lens is as the **template for HardViolation
  enforcement** — interim arg-shape guard → terminal type flip — not as an extractor to absorb. The
  two are complementary (discover-anemia vs guard-a-known-anemic-leaf during migration) and need not
  merge.
- **`fact_cardinality`** — same single-authority invariant, cross-tree census flavor. Shared invariant,
  different extractor — the unification this family is built on.

## 8. Open questions for joint review

1. **The residual boundary (§4)** — do we accept "the lens flags only program-depended structure, the
   oracle/human handles world-structure"? Or do we want the cited-grammar oracle scoped now (it makes
   §4 collapse into Signal A)?
2. **Signal A name-coincidence strength** — is field-name + existing-type enough for HardViolation, or
   only with a second corroborating signal? The §4 `version` case is the cautionary one: the same field
   name can mean an exact `SemVer` (Signal A correct), a version *constraint* (residual — `SemVer` is
   the *wrong* target), or an API version label (a third authority entirely). So name alone cannot pick
   the target type, which argues for name-only coincidence being **SoftCandidate**, not HardViolation.
3. **Pressure-test before any ratchet** — the operator's standing ask: run the extractor over live
   `dsl/**` read-only and inspect HardViolation/SoftCandidate counts for over/under-fire *before*
   enrolling in the CI floor. This doc proposes the signals; the calibration run is the next step and
   the gate is downstream of it.
4. **Terminal vs lens** — for each HardViolation class, is the end state a **type flip** (Signal A/B —
   `base_url: Url`, `arch: Arch`, which delete the lens's teeth by typecheck) with the lens only as the
   interim guard, exactly as `host_language_transport_script` is interim to `emit(intent, Bash)`?

## 9. Recommendation

Adopt the three-signal structural reframe as the answer to the open thread: **anemia is decidable
wherever the hidden structure is already load-bearing in the program** (Signals A/B/C), and only there;
the rest is modeling judgment, optionally automated later by cited-grammar oracles. Build **one**
fact-cardinality-family lens (leaf-side dual of `fact_density`), confidence-tiered, authored on
`std.lens_verdict.LensVerdict`, reusing `host_language_transport_script`'s interim-guard → type-flip
pattern as the enforcement template (not absorbing it). Do a read-only calibration pass over the live
corpus before gating. Each HardViolation's terminal fix is a type flip; the lens is the interim guard.

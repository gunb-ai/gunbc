# A binding RISK CENSUS for the namespace cut

> **This is not the terminal oracle, and must not be read as licensing merge.**
> It classifies GLOBAL DECLARER UNIQUENESS. The terminal oracle is
> `pre-cut resolved binding identity == post-cut resolved binding identity`
> per retained occurrence, which requires running both resolvers. See
> "Honest limits" below.

**Status:** built and run, 2026-08-17.
**Tool:** `tools/namespace_cut/binding_identity_oracle.py` (materializes both trees from git refs itself).
**Receipt:** `tools/namespace_cut/binding_risk_census_receipt.json` — every row, plus pinned provenance.
**Reproduce:** `python3 tools/namespace_cut/binding_identity_oracle.py e095c27543a HEAD out.json`

Numbers below are from an earlier tree and are superseded by the receipt; the
receipt is the authority, this prose is a reading of it.

## Why a count of diagnostics is the wrong instrument

The obvious way to judge the cut is to compile the corpus and count diagnostics.
That was tried and it does not work, for a reason that is structural rather than
incidental: **a bare cross-module reference that resolves emits nothing**, and
DESIGN's Class B says how such a reference resolves — by *pool-membership
coincidence*, because some unrelated module elsewhere in the closure happened to
drag its declarer in. Those bindings score as successes under a diagnostic count
while being exactly the accidental coverage the cut exists to remove.

The measured consequence: qualifying the corpus took one entry closure from 165
diagnostics to 281 on an identical 790-source denominator, which reads as a
regression and is not one — qualification converts a silent coincidental binding
into a located refusal, which is what §5 asks for. A metric that punishes that
is measuring the wrong thing.

## What this oracle asks instead

For every name a pre-cut `import` bound, that is still referenced BARE in the
branch: **is the binding uniquely determined by global declaration identity?**

That question is decidable without running the resolver, and it does not care
how many diagnostics either arm emits.

## Result, over 62,933 bare cross-module references

```
55,995  (89.0%)  SAFE          exactly one declarer corpus-wide
 3,357   (5.3%)  SEED FORK     two declarers, v1/v2, disambiguated by the referencing file's own subtree
   780   (1.2%)  KERNEL FAMILY Present/Absent/Optional/Empty — resolve to the kernel constructor, correctly
 1,668   (2.7%)  RESIDUAL      two or more declarers in the same world; NOT determined by declaration identity
 1,133   (1.8%)  UNKNOWN       no declarer found by the index
```

So the cut is **~95% safe by binding identity**, and the residue is ~4.5% —
1,668 undetermined plus 1,133 unknown. Crucially the residue is *concentrated*:
1,668 pairs across only **221 distinct names**, not scattered arbitrarily.

## The residue is per-name, and several entries are deliberate

```
177  Scaffold     std.disposition   vs test.fixture.scaffold_disposition_census.pool.decoy_nullary / decoy_record
159  Named        v2.std.node       vs std.algebra, v2.extdeps.formats.spice
139  Holds        v2.std.witness    vs v2.std.lens_verdict
 73  Plan         gunbc.plan        vs gunbc.apply
 64  emit         v2.compiler.emit  vs test.claim.roadmap_document, test.claim.roadmap_emit
 61  Terminal     std.disposition   vs v2.std.grammar
 53  ExitSuccess  std.process       vs extdeps.transports.shell
```

`Scaffold`'s competing declarers are modules literally named `decoy_nullary` and
`decoy_record` — fixtures authored to be caught by an ambiguity check. Their
presence in this list is the oracle working, not a defect found.

## Honest limits

This is a STATIC over-approximation of risk, and states so rather than implying
precision it does not have. It asks whether global declaration identity
determines the binding; it does NOT run the resolver, so containment and lexical
scope may disambiguate cases counted here as undetermined, and two declarers
that never co-occur in one pool are not a live hazard. Read the residue as an
UPPER BOUND on names needing disposition, not a defect list.

It also inherits the declaration index's coverage: `UNKNOWN` means the index
found no declarer, which can mean the name is a builtin, a locally bound name
the caller's filter missed, or a declaration form the index does not parse. Those
1,133 need triage before they are called a risk.

## What it is for

Per-name disposition of 221 names is tractable work with a decidable question
attached, which "3,096 diagnostics" never was. It also gives the qualification
work an acceptance test that cannot move the wrong way for the right reason: a
qualification is correct when it names the declarer the pre-cut import
identified, and that is checkable per site.

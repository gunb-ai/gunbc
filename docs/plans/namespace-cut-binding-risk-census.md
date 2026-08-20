# A binding RISK CENSUS for the namespace cut

> **This is not the terminal oracle, and must not be read as licensing merge.**
> It classifies GLOBAL DECLARER UNIQUENESS. The terminal oracle is
> `pre-cut resolved binding identity == post-cut resolved binding identity`
> per retained occurrence, which requires running both resolvers. See
> "Honest limits" below.

**Status:** built and run, 2026-08-17.
**Tool: DELETED 2026-08-20** with its receipts. `binding_identity_oracle.py` and
`occurrence_expander.py` were regex instruments whose only output was the census below;
with the executed floor as the oracle they had nothing left to produce.
**Receipt: DELETED 2026-08-20, and this document is now a reading of nothing it can cite.**
The two census receipts (`binding_risk_census_receipt.json`, `occurrence_index_receipt.json`,
11.5 MB and 545,775 lines between them — 81% of this branch's entire diff) are removed.
They had no executing consumer: nothing but this prose ever read them, and
`is_edit_manifest: false` on the larger one says it drove no edit either.

THE TERMINAL ORACLE THIS DOCUMENT SAID IT WAS NOT NOW EXISTS. The header below is
explicit that a global-declarer-uniqueness census does not license merge, and that the
real oracle needs both resolvers run. That is what the required-floor now does on a
pinned subject: `planned == executed == terminal` witness outcomes at a named SHA and
subject digest. An executed measurement supersedes a static census, so keeping the
census beside it would be two answers to one question with the weaker one 11.5 MB long.

AND THE WEAKER ONE IS WEAKER THAN IT LOOKS: both receipts are REGEX-DERIVED
(`"grain": "one row per bare occurrence (regex-derived)"`). Regex instruments over this
corpus were measured wrong repeatedly on 2026-08-20 — one census read 74,864 rewrites
where the real defect population was a few hundred, and a qualified-reference checker
needed four rounds of false-positive removal (2,450 -> 978 -> 451 -> 398 -> 17) before
its number described the tree rather than its own regex. Committed as a `receipt`, that
class of number acquires the standing of evidence without the execution to earn it.
**Reproduce:** not reproducible — the instrument is deleted. The question it asked is
answered instead by `claim_executor --required-floor`, which runs both resolvers rather
than modelling one.

Numbers below are from an earlier tree, were superseded by the receipt, and the receipt
is now deleted — so TREAT EVERY FIGURE IN THIS DOCUMENT AS UNVERIFIABLE. The reasoning
is kept because it is independently useful (see the next section on why a diagnostic
count is the wrong instrument); the counts are not evidence and must not be quoted.

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

## A blind spot this census cannot see at all: alias substitution

Added 2026-08-17, from a specimen found by session quiet-deer-375 on the regen
lane. It is recorded here rather than in a separate note because it is a limit
on THIS instrument, and leaving it out would make the number above read as more
complete than it is.

`dag/extdeps/rust/version.dag` declares `type CargoPackageVersion =
SemVerIdentity`. The SOURCE IS UNCHANGED by the cut. Yet emit now produces
`extdeps.version.VersionIdentity` where main produced `SemVerIdentity` -- the
alias's TARGET rather than the alias. The hop happens in emit's
`resolved_type(item)`: the inferred Resolved node is the target, and its name is
a containment path.

So **which declaration a reference names can change without anyone editing the
reference.** Deleting imports moved the resolved identity.

This census is structurally incapable of detecting that. It compares GLOBAL
DECLARER UNIQUENESS per spelling; alias substitution keeps the spelling
identical and changes the referent, so every such site is scored SAFE. A name
with exactly one declarer -- the 89% majority, the part of this report that
reads as reassuring -- is no more protected than any other.

Per DESIGN §5 that is the one category outside the guarantee ladder entirely:
not a wrong answer that refuses loudly, but a silently different program. The
terminal oracle (per-occurrence pre/post binding parity, with both resolvers
run) WOULD catch it, because it compares resolved declarations rather than
declarer counts. That is a further argument for building it, and a reason not
to treat this census as a substitute in the meantime.

Open question at time of writing: whether the same mechanism explains the
integration branch's largest diagnostic class, 51 rows of
`expected 'Product(CommutativeSemiring)', got 'Primitive(Int)'` at fully
qualified `std.nat.Nat` annotations, `Nat` itself being an alias of
`CommutativeSemiring<Magnitude>`. Consistent but unproven -- the discriminating
control is whether a Nat-annotated field still accepts an integer literal in a
tree that compiles green.

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

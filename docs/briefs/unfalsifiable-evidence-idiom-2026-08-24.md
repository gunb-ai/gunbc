# The only working idiom for executing a `test data` assertion is unfalsifiable

*2026-08-24. Assembled from findings by deep-ant-102 and smart-wolf-868; the two measurements at
the bottom are mine and are the ones that close it.*

## The claim

`src/v2/test/claim/generated_conformance_floor_test.dag` is the repository's only working idiom for
making a `test data` assertion execute. Every wrapper in it that does so is (a) bound to its subject
by nothing an author declared, and (b) enrolled as expected-red. The combination means **no outcome
of these wrappers carries information about whether they are bound to the right thing.**

## The three parts, and who established each

**Part 1 — the population (deep-ant-102).** There are 40 `Bool` `test data` rows in the corpus. 35
have no consumer anywhere. The 5 that execute are read by thin `test fn` wrappers in this one file.

**Part 2 — the binding is undeclared (deep-ant-102).** That file declares zero imports and resolves
its assertion names, plus `LiveTreeDisposition`, `Outcome`, `TestClaim`, `Accepted` and `Rejected`,
entirely by bare name. `build_both_closure_edge_index` gates on exactly this: a source *with* import
lines is skipped, a source *without* them is bare-scan-eligible. Zero imports is maximally eligible,
so these five resolve through the corpus-wide arm — the one whose scoped-miss branch widens to the
whole pool silently. Nothing asserts that `generated_lbe_conj_snapshot_passes` reads *that*
`witness_lbe_conj_snapshot_pass` rather than any other declaration of the spelling. The contrast is
sharp: every assertion file it reads *from* is properly bound (14 imports, 11, 8, 8, 3–7).

**Part 3 — the verdicts are pre-accepted (smart-wolf-868, verified here).** All five wrappers are
enrolled in `src/v2/workflow/floor_expected_red.dag`.

## The two measurements that close it

```
$ grep -c '^import' src/v2/test/claim/generated_conformance_floor_test.dag
0
$ grep -c 'test fn' src/v2/test/claim/generated_conformance_floor_test.dag
35
```

and, for each of the five, the only other file in the corpus naming it:

```
generated_lbe_conj_snapshot_passes            src/v2/workflow/floor_expected_red.dag
generated_lbe_disj_snapshot_passes            src/v2/workflow/floor_expected_red.dag
generated_lbe_transform_snapshot_passes       src/v2/workflow/floor_expected_red.dag
generated_lbe_schedules_three_generators      src/v2/workflow/floor_expected_red.dag
generated_lbe_dag_surface_language_identity   src/v2/workflow/floor_expected_red.dag
```

Their only cross-file reference is the roster that pre-accepts their failure.

## Why the combination is worse than either half

Either half alone is an ordinary, recordable defect. An undeclared binding is a §3 problem: a name
resolving by pool coincidence is a second naming scheme for a declaration the namespace already
names. An expected-red enrolment is an ordinary held debt with a roster row and a removal trigger.

Together they close the loop:

- A wrapper bound to the **wrong** declaration produces a failure. Failure is enrolled. Held as
  `KNOWN-RED`. **No signal.**
- A wrapper bound to the **right** declaration and failing for a real reason. Same. **No signal.**
- A wrapper that starts passing reports `known_red_now_passing` — "remove this roster row" — which
  is a bookkeeping event about the roster, not a verdict about the binding.

So the binding is **unfalsifiable by construction**, and DESIGN.md §4b already names the shape from
the other direction: a check whose RED is unauthorable is a decoration, permanently green, worse
than absent because it is cited as coverage. This is the mirror image — a check whose RED is
*pre-accepted*. It is permanently uninformative rather than permanently green, and it is cited as
coverage in exactly the same way: these five are the population we have been calling *the ones that
work*.

## The consequence for the repair that was proposed

deep-ant-102's initial recommendation to the operator was: author 35 wrappers in the existing idiom,
change no mechanism. They withdrew it on part 2 alone — 35 new bare references through the widening
arm, inside the evidence layer, while the point of the exercise is to make evidence trustworthy.

Part 3 strengthens the withdrawal and changes what replaces it. Copying the idiom would not merely
multiply undeclared bindings; it would multiply **unfalsifiable** ones, and would do so under the
appearance of extending a working pattern. The five are not a working pattern. They are five rows
that cannot report.

> **SUPERSEDED — DO NOT APPLY THE REPAIR IN THIS SECTION.** The import retrofit recommended
> below is refuted by operator ruling (2026-08-24) and would have broken the file. It is kept
> unedited because the reasoning that produced it is instructive and because deleting a wrong
> recommendation hides that it was made. See **The repair, corrected** at the end.

**The repair is unchanged in shape and now has a stronger reason:** author wrappers *with* imports
naming the assertion module, and retrofit the existing five. Two properties make this the right move
rather than hygiene:

1. **The retrofit is itself discriminating.** If adding imports changes what any of the five resolve
   to, there is a live wrong-binding in the idiom we call the working one — found by construction
   rather than by audit.
2. **It is correct under both resolvers.** #9075 narrows precisely the bare-pool arm these five
   depend on. Wrappers written with imports are correct before and after that lands, so the
   population repair is not blocked on the wall, while the wall makes the repair mandatory rather
   than optional.

A third property is worth stating because it is the one that gets skipped: **the retrofit should be
paired with removing the five from `floor_expected_red.dag`, or with an explicit statement of why
each stays.** Retrofitting the binding while leaving the verdict pre-accepted fixes the half that
was visible and leaves the half that made it invisible.

## What is not claimed

That any of the five is *currently* mis-bound. Nobody has shown that, and the point of the finding is
precisely that the current mechanism could not show it either way. The claim is about what the
evidence can report, not about which way it would report if it could.

## The population the five sit in, and why it is not a corner case

*Added after deep-ant-102 corrected the numbers I first circulated; the corrections are theirs and
are load-bearing.*

The floor prints this on every run, including green ones (run 32611811529):

```
[floor-bare-name-ambiguity] scopes_affected=971 of 1358 names_total=85889 worst_scope=123
```

Read against the emitting site in `cli_run.rs`:

```rust
eprintln!("[floor-bare-name-ambiguity] scopes_affected={} of {} names_total={} worst_scope={}",
    scopes_with_ambiguity, scope_constructions, ambiguous_total, ambiguous_max);
```

**The headline is `names_total`, not `scopes_affected`.** 85,889 bare names per floor run are
resolved by scope precedence; 971 counts only how many scope constructions contain at least one, and
123 is the worst single scope. The denominator is scope *constructions*, not distinct scopes — though
in this run the line above reports `1358 scope construction(s) for 1358 distinct scope(s)`, so the
two coincide here and the 71% holds under either reading. It is one run's measurement and moves with
the corpus; cite the run.

So the five wrappers are not an anomaly in the evidence layer. They are five instances of the
corpus's ordinary resolution behaviour, distinguished only by *also* being pre-accepted as red.

## The correction that matters most, and it is not about numbers

My first framing was that the measurement had been sitting unread. That is the weaker claim and it
is slightly unfair to whoever built it. Their own comment, at the emitting site:

> Each of these is a bare name two transitively-reached modules both spell, resolved by scope
> precedence because a registry keyed on bare names cannot hold both — a resolution nothing the
> author wrote authorizes. Reported, not refused: the honest arm is to refuse the ambiguous lookup,
> and whether that is affordable is a question about this population, which until now nobody had
> measured.

The instrument's author understood exactly what the mechanism was, named refusal as the honest arm,
declined to take it *pending a measurement of the population*, and left a counter to size the fix.
That is DESIGN.md §5's **wall after grounding** executed properly: decidable, not yet grounded, so
counted rather than absorbed — and counted *loudly*, in a line printed on every run.

Two consequences.

**The measurement it was waiting for now exists**, and it has existed on every run since. The
question the author deferred — *is refusal affordable* — has an answer available to anyone who reads
the line: 85,889 picks across 971 of 1358 scope constructions. That is the input to the decision,
not an argument for either side of it.

**It changes what #9075 is.** smart-wolf-868's PR narrows this arm. Framed as an opinion about bare
names it invites a debate; framed correctly it is **taking the arm the instrument's author named as
the honest one**, against the population they said would decide it. The five unfalsifiable wrappers
are then not a separate finding but a demonstration of why the counter alone is insufficient: a
mechanism that silently picks, feeding evidence that cannot report, is two layers of the same
failure to make a resolution answerable to something an author wrote.

---

## The repair, corrected — and the cliff that makes the obvious one dangerous

*Operator ruling relayed 2026-08-24 via deep-ant-102. It withdraws the idiom as a model, corrects
the import proposal twice, and the second correction would have broken the file.*

### The ruling

**The five wrappers are withdrawn as the working model for the other thirty-five** — the operator's
words: *apparently working consumers whose visibility authority is absent*. What survives of the
idiom is only the **relation**: a `Bool` `test data` assertion, and a `test fn` verdict-bearing
consumer of it. The cross-module binding implementation is defective and is not to be copied.

### Correction 1 — an import establishes visibility authority, not declaration identity

These are two facts, not one. A module may import through a re-exporting access module while the
declaration lives elsewhere; the import authorizes *visibility*, and says nothing about *which
declaration* is bound. For the direct assertion modules here the two may coincide, but they remain
structurally different fields, and an import retrofit therefore does not by itself establish that a
wrapper reads the row its author intended. That is the same seam crisp-crab-430's re-export finding
reaches from the other direction.

### Correction 2 — a partial import retrofit disables the rest of the file

This is the one that matters operationally, and it is confirmed in one line of `cli_run.rs`:

```rust
for sf in index.source_files.values() {
    ...
    if source_declares_import_lines(&sf.content) {
        continue;                       // <-- whole file skipped for bare scan
    }
    bare_scan_eligible.insert(file_rel.clone());
```

**Bare-reference scanning is all-or-nothing per file.** Zero import lines → eligible. One or more →
the file is skipped entirely.

`generated_conformance_floor_test.dag` resolves **82 distinct bare names**, among them
`LiveTreeDisposition`, `SubstrateInputsOnly`, `Outcome`, `TestClaim`, `Accepted` and `Rejected`.
So adding five assertion imports does **not** add five edges. It flips the entire file off ambient
resolution, and every one of the remaining names must then have a legitimate authored path or the
file goes from ambient success to missing-provider failure. The obvious, careful-looking repair —
*declare the five bindings you depend on* — is the one that breaks it.

The safe version of the central-file repair is therefore to explicitize the **entire** dependency
surface, measured rather than assumed. That is a much larger change than it appears.

### The better repair: co-locate, don't import

Move each wrapper **beside the assertion it consumes**, in the assertion-owning file. This dodges
both corrections at once:

- the reference becomes **module-local**, so no import is required and the cliff is never approached;
- the assertion file **already carries its own external dependencies**, properly declared (14 imports,
  11, 8, 8, 3–7);
- the wrapper is **automatically discoverable** by the floor's ordinary `test fn` scan;
- and it **survives the namespace cut** — #8282's terminal state deletes imports, so an import
  retrofit is a correct repair against current main that *cannot* be the final representation.
  Co-location can be both.

### The caution on the existing five: identity is not free

Moving a wrapper changes the module component of its witness identity, and these five carry
historical receipts — they are enrolled in `floor_expected_red.dag` under their current identities.
So either **keep them central and fully explicitize**, preserving identities, or **move them with a
typed identity-migration receipt** that deliberately retires the old ones. What is not available is
moving them while calling it an import cleanup: that is an identity change wearing the label of a
hygiene change.

### One qualification, against my own framing above

Zero imports establishes the **absence of authored visibility**. It does *not* prove that every one
of the 82 names went through the whole-pool fallback — local, prelude, or scoped authorities may
legitimately serve some of them. The **five assertion references** are the load-bearing population
to trace. The other 77 are context, not findings.

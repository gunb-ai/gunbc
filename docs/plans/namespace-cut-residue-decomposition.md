# The 185-diagnostic residue, decomposed

Snapshot 2026-08-17 at `e229bb584af` (before the type-parameter repair lands).
Written so the remaining work is a list of decisions rather than a number.

| count | class | state |
|-------|-------|-------|
| 50 | `Nat` <- integer literal in field/named-arg | BLOCKED on the two-`Nat`-fork decision |
| 22 | type parameter shadowed by global `C` | repair pushed, awaiting measurement |
| 18 | cross-source-root reference (see below) | needs a root/ownership decision |
| ~11 | lost dependency edges | REPAIRED (196 -> 185) |
| 7 names | ambiguous owner | one decision each |
| rest | uncharacterized | |

## Cross-source-root references — a class the cut creates

`src/v2/test/manual/ownership_movable_test.dag` carries 18 of the remaining
diagnostics, all of them names reported as undeclared:
`EdgeKind`, `EdgeClassification`, `OwnershipProof`, `BindingUsage`, `Consumed`,
`Projected`, `Threaded`, `build_movable_set`.

They are NOT undeclared. They live in `src/v1/ownership.dag`. The compile's
source roots are `dag` and `src/v2`; `src/v1` is not among them. So a v2 test
depends on v1-world declarations across a root boundary that the reference
closure cannot cross, because the declaring file is in no scanned root at all.

An `import` used to bridge that gap by naming the module directly. A reference
closure cannot, because it can only pull files it can see. This is the same
"protection by absence became a live problem" shape as the shadowed type
parameter, inverted: there, absence stopped protecting; here, absence stops
providing.

Three candidate dispositions, and the choice is a modeling one:
- add `src/v1` to that compile's roots (widest, and re-imports v1-world names
  into a v2 compile that deliberately excludes them);
- move the ownership types to a root both worlds see;
- retire the test, if v1's ownership model is not v2's subject.

## A correction to this document's own instrument

The first pass reported eight of those names as NO DECLARER FOUND, because the
declaration index scanned `dag/**` and `src/v2/**` and not `src/v1/**`. The
names were declared the whole time. That is a denominator error of exactly the
class this branch keeps producing -- an instrument's silence read as the world's
emptiness -- and it is recorded rather than quietly fixed because the same index
backs the binding risk census, whose `UNKNOWN (no declarer found)` bucket
(1,135 rows) is subject to the identical bound and should be re-run over all
three roots before anyone treats that number as a defect population.

## The 7 ambiguous-owner names

Each needs one decision, not a script -- and per the parse_diagnostic lesson,
the repair must name the declaration that OWNS the name, which main's import
list is not a reliable guide to.

```
nat_max     std.nat | v2.std.nat                         (subtree decides)
Accepted    gunbc.workflow.types | v2.std.diagnostic
Rejected    std.markup | v2.std.diagnostic
Present     v2.std.optional | v2.std.execution_surface | gunbc.srv3_install_media_fetch
Absent      std.upsert_decision | v2.std.execution_surface | gunbc.host_converge | ...
Missing     extdeps.coverage | std.filesystem.types | v2.workflow.ci_placement
```

`Present`/`Absent` are the kernel-optional family and must NOT be qualified --
qualifying them degrades the scrutinee to a generic `T`. They need the
kernel binding, which is a resolver question rather than an authoring one.

## DISPOSITION 2026-08-17: the 18 cross-source-root rows — retired with a receipt

`src/v2/test/manual/ownership_movable_test.dag` is deleted. It carried 18 of the
residue's diagnostics, all names reported undeclared (`EdgeKind`,
`EdgeClassification`, `OwnershipProof`, `BindingUsage`, `Consumed`, `Projected`,
`Threaded`, `build_movable_set`) which are in fact declared in `src/v1/ownership.dag`
— a root this compile does not scan.

WHY RETIREMENT RATHER THAN A ROOT WIDENING OR A MOVE. The three candidate
dispositions this document listed are not equally available post-cut. Adding
`src/v1` to the roots re-admits v1-world names into a compile that deliberately
excludes them, and its closure pulls `src/v1/04_resolve.dag`, which the file's own
note records as carrying a pre-existing dark type error. Moving the declarations
to a shared root is a real modeling change to v1's ownership model, made to satisfy
a test rather than a consumer.

WHAT IS ACTUALLY LOST, taken from the file's own dissolution note rather than from
my assessment of it: it was OFF THE AUTO-PATH (not reached by
`claim_batch --roster-from-discovery`, since `src/v1` is not a discovery scan dir),
NOT CI-ENROLLABLE (the floor witness run is dag+src/v2-scoped and cannot resolve
the v1 reference), and NOT RESOLVABLE ON CLEAN MAIN. Its own words: "this is NOT a
live .dag regression guard today. The live v1 ownership logic is carried by the
compiled seed (v1_compiler_ownership.rs) and exercised via rust_tests — that is
where the real coverage sits, not here."

So the executed-coverage delta is ZERO, and this is the #8146 shape: a witness
that had already stopped executing, kept alive by its file existing. What is lost
is the AUTHORED INTENT that a v2-side successor should eventually exist — which
was carried in the dissolution trigger, and is preserved here rather than in a
file that cannot run.

WHAT THE CUT ACTUALLY CHANGED, stated because it is the general fact and not a
property of this file: an `import` could NAME a module outside the scanned roots;
a reference closure can only pull files it can SEE. So cross-root dependencies are
expressible under imports and structurally inexpressible after the cut. Any other
file in this position has the same three dispositions and the same reasoning —
this is a class, not an incident, even though its current population is one.

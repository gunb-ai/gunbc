# Emitter ownership de-fork: a status correction, a population bound, and three instruments that could not answer

**This is not a defect report.** An earlier draft was, and
[`docs/plans/emitter-ownership-defork.md`](../plans/emitter-ownership-defork.md) answers its central
claim — **against it**. What survives is a status correction, one number, and a method note.

**Subject.** `src/v1/ownership.dag`, `src/v1/05_emit_rust.dag`, and the de-fork plan.
**Digests.** Subject `staging.dag` `812d95660a6dd909` in every arm; probe mirrors
`0548ebb56a2243ac` (subtractive), `2460edde31b1e574` (additive), baseline `432de5408dffa412`.
`ctrl-build` replays the worktree as patches, so a runner's `HEAD` names the *base* — digests are the
provenance, not SHAs.

---

## 1. What was NOT found — retracted in full

The draft claimed a defect: `make_decision` requires exactly one `Consumed` edge; `walk_expr` records
`Consumed` **only in tail position**; therefore any value in **argument position** is `Read`,
`sc == 0`, `Unclassified`, never in `movable`, and always cloned.

**The mechanism description is accurate. Calling it a defect is wrong.** The plan's increment-1
*Soundness* paragraph states the rule and its justification:

> `movable` gates **every** whole-value var-ref emission … not just tail returns, so a per-name move
> license is sound only if the binding has exactly **one** whole-value use site. `Consumed` is
> recorded only at tail position; `Read`/`Threaded` edges are whole-value positions emitted *before*
> it, so any such edge plus a licensed move = use-after-move. Hence the rule: SoleOwner **and** zero
> Read/Threaded edges.

So tail-only `Consumed` is **the wall that makes per-name licensing sound at all**, and the plan
records that an earlier draft admitting Read-then-Consume was a latent use-after-move, since fixed.

**The probe run contained the proof of this and it was misread.** Forcing `build_movable_set` to
admit everything emitted `resolve_probe(lookup(x), stage, x)` — `x` moved **twice**. That is not an
incidental artifact of a maximal arm; it is precisely the failure mode the wall exists to prevent,
observed and filed as a curiosity.

**Also already documented, in `src/v1/trait_bound_witness.dag`'s own note** — that the Clone-bound
chain is blind to function bodies because `emit_fn_def` calls
`v1_generic_params_needing_clone_bound` with params/ret/bounds but **never with the body** — and
already censused: `dag/tools/e0599_emitter_decision_census.dag`, cause `CloneSharedRequirement`,
lowering `DerefCloneWholeValue`, **139 counted occurrences**, with the carrier question open among
three named candidates including that plan's `UseSiteVerdict.CloneShared`.

## 2. STATUS CORRECTION — the designed recovery is absent from the tree

The plan's recovery for exactly these sites is per-name → **per-site** licensing (Perceus-style
last-use), increment 2, whose status line reads *"proof-side machinery AND whole-value emitter
consumption are implemented"*.

**Measured in this tree — zero, corpus-wide across all `.dag` and `.rs`:**

| symbol | hits | | positive control (same method, same paths) | hits |
|---|---|---|---|---|
| `build_move_site_licenses` | **0** | | `build_movable_set` | 2 |
| `move_sites_index` | **0** | | `make_decision` | 1 |
| `move_licensed_at_site` | **0** | | `build_read_only_params` | 2 |
| `take_owned_counted` | **0** | | `owned_bindings` | 3 |
| | | | `FoldAccUnwrapProof` | 1 |

And **PR #6248 — the PR the plan says increment 1 is "in this PR" — is CLOSED, not merged.** #6249
and #6250 merged.

Increment 1's *core* did land by some other route: `build_movable_set` today filters on
`make_decision(usage) == SoleOwner` and admits params via `param_names`, exactly as described.
**Increment 2's per-site licensing did not.** The status line is **false for this tree**.

This is the finding with the shortest path to a decision, and it is not a compiler finding — it is a
plan whose recorded status does not match the tree, so any work planned against that status is
planned against a premise that is not true.

## 3. POPULATION BOUND — at most 446 of 775 clone sites

Same subject, 14 emitted files in both arms; baseline guarded UNPROBED (probe markers `0`,
`SoleOwner` gate `1`, `lambda-capture` `1`).

```
BASELINE total clone() across the emit:  775
FORCED-MOVABLE comparator:               329
DELTA:                                   446        BASELINE total Clone bounds: 93
```

> **AT MOST 446 of 775 clone sites in this emit exist because a value could not be proven movable.**

**The qualifier is part of the sentence**, for two compounding reasons: 329 is a floor **no correct
repair can reach** (that arm emits unsound Rust — §1), and this counts clone **sites**, not `movable`
membership, so it bounds the *consequence* rather than observing the *cause*. **Do not restate it as
"58% of clones are spurious."**

**It is a different quantity from the census's 139**, which counts E0599-causing lowering sites. This
bounds total clone sites attributable to non-movability — the first number attached to the population
from that end, and it complements the plan's own cost denomination (whole-tree `compile --target dag`
at ~72 min, ~85–90% emit, the default-clone paths keeping `Rc` refcounts ≥ 2 so every
`rc_map_insert`/`rc_list_push` copy-on-writes the whole container — O(n²) surfacing as the CI
timeout).

**What it does not settle:** whether `movable` is **inert** (near-empty — coverage by illusion) or
merely **partial**. The delta measures the consequence and cannot separate them; that needs set
sizes and its own evidence.

## 4. Three instruments that could not answer

Nothing in the plan contains this, and unlike the mechanism it is not re-derivable from the code.
**Two of the three looked like results.**

**(a) The probe never compiled.** `src/v1/ownership.dag` has a **stage0 mirror**;
`cargo build -p v1-compiler` compiles `src/v1/stage0/src/v1_compiler_ownership.rs`, **not the
`.dag`**. Detected with one grep, no build: probe markers `0` in the mirror, `lambda-capture` `1` as
the positive control. Every provenance column was *true* and none answered the question — the `.dag`
digest changed (the edit arrived *on disk*), `Checking patch src/v1/ownership.dag` appeared in the
runner log (applied to a file nothing reads), and the rebuild was genuine (*of the unchanged
mirror*). **The guard asserted the probe was present in the SOURCE TREE; for a self-hosted compiler,
*present in the tree* and *present in the executed path* are different questions.**

**(b) A subtractive probe cannot discriminate against an inclusion set.** `build_movable_set` folds
qualifying names **in**, so removing the capture collapse removes entries *entirely* — and absence
yields the same emitted clone as exclusion. Even with the collapse verifiably gone from the executed
path (`lambda-capture` = 0 in the mirror, guard passed), the null was unreadable. **A control is not
made decisive by being maximal.** The decisive direction was additive.

**(c) Reasoning about signatures while treating bodies as illustration.** A draft made `cached_stage`
a zero-diagnostic/two-defect specimen. Counting the uses of `x` — `lookup(x.clone())` *and*
`x.clone()`, **twice** — shows its `A: Clone` is **required**. The discriminator was in a two-line
body that had been quoted repeatedly and never read.

**The guards that replace (a) and (b):** assert on the **compiled** artifact in *both* directions —
the probe marker present **and the construct being removed absent** — plus a positive control so a
zero is readable, and a **sensitivity control** (total `clone()` across the whole emit) so "nothing
changed" is distinguishable from "the probe did nothing".

**And the meta-failure this document is itself an instance of:** four separate findings here were
rediscoveries of things already written down — in the plan, and in the note at the top of a file
under the author's own sole-write ownership, whose functions were quoted repeatedly without its note
being read. **Read the note on the file you are about to claim a finding about.**

## 5. Genuinely open, and deliberately small

- **`resolve_probe`'s `x`**: an ordinary function parameter, used **once**, in argument position,
  cloned. Under §1's wall this is expected — one `Read`, no `Consumed`. Whether per-site licensing
  (§2, unlanded) would recover it is exactly the class increment 2 exists for. No lambda is
  involved, which makes it the cleanest specimen for that increment.
- **`movable` / `read_only_params` do not partition the space.** `sc == 0` bars the single-use
  argument from the first; `binding_fan_out > 1` bars it from the second (plus `is_owned_local`,
  whose `effective_kind` is first-non-`none`-writer-wins over a seed of `none` — an order
  dependence). A param used **twice by reference** qualifies as read-only; used **once by value** it
  is cloned.
- **The `ExprLambda`/`ExprForEach` capture collapse** re-records every binding used in the body —
  including the lambda's **own binders** — as a capture `Read` at the *enclosing* scope, with nothing
  subtracting binder names, though `lambda_param_names_at` is imported in that very file. Under §1
  this is **not** the cause of the clones investigated here, and the plan's increment-2 note says
  lambda bodies "license nothing and poison their captures live" by design. Recorded as an
  observation, **not** as a defect claim.

---

**Ownership.** `src/v1/ownership.dag` has no sole-write declaration in `dag/gunbc`, and git
attribution names no current owner (last human commit 2026-07-06). The live authority for this area
is the de-fork plan and its lane (`node://adhoc-0717d295-672`), **not this document** — which exists
to correct that plan's status line, contribute one bounded number, and record the instruments.

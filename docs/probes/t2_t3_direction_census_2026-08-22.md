# T2/T3 are not one cluster: the direction census

**OBSERVED ON:** `docs/probes/e0308_partition_2026-08-21/sites_classified.tsv`, board sha
`2a2bd0ad59cdc4d37f0ef35a72232bac57c9bbe7`, entry `src/v2/compiler/03_ingest.dag`, `M=1`.
**CLAIM ABOUT:** that artifact only. A refresh against current main is in flight and this
document is not re-stated against it; if the refresh moves the population, the shape below is
re-measured before it is quoted, not carried forward.

## The question

The work item premised T2 (34 sites, text carrier vs host `String`) and T3 (25 sites, collection
carrier fork) as "the ONE plausibly-shared realization cluster". Sharing a cluster means sharing
an arbiter. An arbiter that disagrees with itself is *position-dependent*; a mapping that is
simply absent is *one-directional*. Those are different defects with different repairs, and the
direction census separates them without needing the emitted Rust.

## The discriminator

For each root, count emitted positions (`file:line:col`) at which the same carrier pair is
reported in **both** directions.

| root | sites | positions with both directions | modules |
|---|---:|---:|---:|
| T2 | 34 | **4** | 3 (32/34 in `v2_compiler_tokenize.rs`) |
| T3 | 25 | **0** | 10 |

The four T2 positions are `v2_compiler_tokenize.rs` lines 228, 251, 272, 340 — each at column 13,
each carrying `Rc<Vector<_>> <- String` *and* `String <- Rc<Vector<_>>` at that one column, with a
third `Rc<Vector<_>> <- String` at column 25 of the same line.

T3's directions are `expected host, found modeled` at 24 of 25 sites — `OrdSet<String> <-
Rc<PointwisePower<_>>` (8), `HashMap<..> <- Rc<PartialFunction<_,_>>` (10), the rest scalar-vs-
`Vector` arity in `std_state_durability.rs`. The single reverse site is
`v2_std_compilers_target_model.rs:6056:5`, `Rc<Rc<PartialFunction<..>>> <- Rc<HashMap<_,_>>` — a
different file, a different position, not a within-construct disagreement.

## What this establishes

**T2 and T3 do not share an arbiter.** T2 exhibits within-position disagreement: one dag text
carrier rendered two ways at a single emitted column. T3 exhibits a uniform modeled→host
direction: the collection surface (`PointwisePower`, `PartialFunction`) is not lowered to its host
realization, everywhere, the same way. A repair to a position-dependent arbiter does not address
an absent lowering, and vice versa.

The brief's premise is therefore refuted by measurement, which was the brief's own first
deliverable.

## What this does NOT establish

The **mechanism** behind the T2 reversal. The emitted Rust for those lines is a probe artifact
and is not in the tree, so what construct sits at `tokenize.rs:228:13` is unread. Both directions
at one column is consistent with more than one construct (a comparison whose operands were
rendered from one dag type, a scrutinee/arm pair, a call whose parameter and return were arbitrated
separately). The shape of the divergence is measured; its cause is not. Naming a cause here would
be the fabricated-plausible-output failure this lane has already corrected itself for twice.

Nor does it establish anything about the *other* 174 E0308 sites, or about T2/T3 under the
refreshed board.

---

# Addendum: the T2 mechanism, read from in-tree source

The section above declined to name the mechanism behind the four within-position reversals,
because the emitted Rust is a probe artifact. It turns out the emitted Rust is not needed: the
chain is readable from committed source, and it is named here with each link's evidence.

## The links

1. `src/v2/std/text.dag` declares **`type String = FreeMonoid<Char>`**. In v2, `String` *is* a
   sequence of `Char` — an alias, not a distinct carrier.
2. `src/v2/compiler/01_tokenize.dag` declares `lexeme: String` (on `LexMatchAccepted`,
   `LexRuleToken`, `LexRuleTrivia`, `LexRuleAnnotation`, `RepeatState`, `DelimitedState`) and
   constructs those fields with free-monoid operations — `Cons`, `std.algebra.Empty`,
   `list_append` — at eight sites. **This is correct**, not a source defect: those are exactly
   `FreeMonoid`'s operations. An earlier reading of mine treated it as a wrong-overload call
   against `std.source_annotation`'s `advance_line_prefix_indent_only(code_points: List<Int>)` /
   `..._text(lexeme: String)` pair; link 1 refutes that, and the pair is not implicated.
3. `src/v1/coercion.dag` `structural_declaration_modules_for` maps
   `"String" => ["src/v2/std/text.dag", "dag/std/string_type.dag"]`, and
   `decl_file_declares_structurally` matches by `contains(decl_file, m)`.
4. `src/v1/coercion.dag` `type_reference_decl_file` answers a reference's declaring file from
   `n.inferred` when it is `Present { Resolved }`, and **otherwise falls back to the reference
   node's own `ident_span` file**.

## The consequence

Link 4's two arms give one `String` reference two different declaring files depending on whether
inference resolved *at that reference*:

| arm taken | decl_file answered | roster match (link 3) | rendered as |
|---|---|---|---|
| `Present { Resolved }` | `src/v2/std/text.dag` | yes | structural — `Rc<Vector<_>>` |
| fallback | the referencing module, e.g. `src/v2/compiler/01_tokenize.dag` | no | host `String` |

So the arbitration is not position-in-the-file dependent; it is **resolution dependent**, and two
sub-expressions of one construct can take different arms. That is precisely the observed
signature: `tokenize.rs` 228/251/272/340 each carry both directions **at one column**, which a
file-position rule cannot produce and a per-reference resolution rule produces naturally. The
`Rc<Vector<i64>>` variants are the same split with `Char` resolved to its code-point width rather
than left inferred.

`coercion.dag`'s own `type_reference_identity_note` states the governing rule in the repository's
words — *"the empty string means identity is unknown, which yields NO realization, so a reference
site that omits identity silently renders structurally"* — and records that this exact class
already caused one regression (`dag/std/integer.dag` aliases rendering as `crate::std_nat::Nat`
at eight reference sites). T2 is that class recurring at a different alias.

## Why the designed repair does not reach it

This lane's earlier repair design routed `TypeRealizationDecision`. That does not touch link 4:
`type_realization_decision` takes `decl_file` as a **parameter**, so it renders faithfully whatever
identity it is handed. The wrong answer is produced upstream, by the fallback arm, before the
arbiter is consulted. Routing changes the path and not the answer — which is the horn the parent
lane raised, now confirmed from the emission side.

## The discriminating RED this predicts

If link 4's fallback arm is the cause, then making it answer *absent* rather than the reference's
own file must collapse T2's host-`String` side specifically, and must not move T3 at all (T3 has
zero within-position reversals and a uniform modeled→host direction). Both halves are falsifiable
against the same board producer.

## Evidentiary boundary

Links 1–4 are read from committed source at branch head; the site population is the 2026-08-21
board artifact at sha `2a2bd0ad`. The chain from link 4 to the emitted bytes is **not executed
end-to-end here** — no run in this document ties a specific `tokenize.rs` column to a specific arm
of `type_reference_decl_file`. It is a mechanism consistent with every measurement taken, and it
makes the falsifiable prediction above; it is not yet a receipt. The board count it is stated
against is 339 coded / 135 E0308 at `629252b6df` (measured by the parent lane with provenance
attached, cited rather than re-measured).

---

# Registered predictions for the intervention (written before the run)

The mechanism above is consistent with every observation, which is not the same as being its
cause: an observation cannot separate "this mechanism produces the signature" from "something else
does and this mechanism is also true". Changing the input can. One dispatch, two arms, same ref,
same tree, one variable — the fallback arm's answer.

- **ARM A** — fallback returns the reference's own `ident_span` file (today's behaviour).
- **ARM B** — fallback answers absent (`String::new()`).

Numbers registered before the run, so a partial result cannot be read as confirmation:

| # | quantity | prediction |
|---|---|---|
| P1 | `T2_POSITIONS_BOTH_DIRECTIONS` | arm A > 0, arm B **exactly 0** |
| P2 | `T3_LIKE_SITES` | arm B − arm A = **exactly 0** |
| P3 | T2 sites whose `expected` is host `String` | arm B **exactly 0** |
| P4 | *control* — `sha256(target/release/gunbc)` | arms **must differ** |
| P5 | *control* — the fallback source line as read on the runner | arms **must differ** |

P4 and P5 are not decoration. The probe rebuilds `gunbc` keyed on `git rev-parse HEAD` via a
`.tree` stamp, so an *uncommitted* patch leaves HEAD unchanged, skips the rebuild, and measures the
baseline twice — which would render as "no effect" and read as a refutation. Arm B therefore
commits (on the runner, never pushed) to move HEAD, and both controls must show a difference before
any P1–P3 number is reportable. A control that agrees with the thing it controls for is
indistinguishable from a control that never differed.

T3 is a genuine control rather than a hoped-for null because it was established independently and
before it was needed: zero within-position reversals, uniform modeled→host at 24 of 25, ten
modules.

Both arms will be reported with both numbers whether or not they match.

---

# Intervention result: controls PASS, predictions NOT INTERPRETABLE

One dispatch, both arms, `MARKER_REF=90986d19469397098ddaa799dfc9e9087541cbf4`, empty dirty list.

## Instrument controls — both pass

| # | control | arm A | arm B | verdict |
|---|---|---|---|---|
| P4 | `sha256(target/release/gunbc)` (16) | `ea59c72fbac8f384` | `6d40b1d2a0417935` | **differ — PASS** |
| P5 | fallback line as read on the runner | `_ => decl_identity_file(n.clone()),` | `_ => { let _ = &n; String::new() }` | **differ — PASS** |

The patched compiler really was the one measured. Both arms ran to `PROBE_EXIT=0` with one cargo
log each, and the extractor announced `TOTAL_MISMATCH_SITES` in both, so this is not the
failed-instrument case.

## The measurement

| quantity | arm A | arm B |
|---|---:|---:|
| `TOTAL_MISMATCH_SITES` | 134 | **160** |
| `T2_LIKE_SITES` | 21 | 21 |
| `T2_POSITIONS_BOTH_DIRECTIONS` | **0** | **0** |
| `T3_LIKE_SITES` | 14 | 14 |

## Why no prediction may be read off this

**P1 registered `arm A > 0`, and arm A is 0.** That is a pre-registered precondition failing, and
it invalidates P1, P2 and P3 together: a predicted zero that is *also* zero in the control arm
measures nothing. Had the predictions not been registered as exact values before the run, `arm B =
0` would have read as clean confirmation of the mechanism. It is not.

**Arm A does not reproduce its own baseline.** The 2026-08-21 artifact recorded 34 T2 sites, four
positions carrying both directions, and a top signature of `expected Rc<Vector<_>>, found String`
×19. Arm A here reports 21 T2-like sites, zero both-direction positions, and a top signature of
`expected String, found Rc<Vector<String>>` ×8 — a different population with different signatures.
By this lane's own standing rule, a walk that does not reproduce the known population as its
diagnostic-producing subset is wrong and must not be published as a divergence count. So it is not
published as one.

**The cause of the baseline miss is not established.** At least two candidates, not separated:
the artifact was taken at `2a2bd0ad` and this run at `90986d19`, and the board moved substantially
between them (199 E0308 blocks then, 135 now), so the T2 population may genuinely have changed; and
this extractor is a fresh regex over cargo logs, not the board's classifier, so it may simply bucket
differently. `TOTAL_MISMATCH_SITES=134` against the parent lane's independently measured `E0308=135`
suggests the *total* is very close to right, which makes the *bucketing* the more likely suspect —
but that is a ranking, not a finding.

## What does survive

Arm B has **26 more** mismatch sites than arm A (134 → 160) while the T2-like and T3-like buckets
are **byte-identical** between arms. So the fallback change demonstrably moved the emission — it is
not inert — and it moved nothing this extractor counts as T2 or T3. That is consistent with the
`integer.dag` regression `type_reference_identity_note` warns about (empty identity renders
structurally, at sites that are not text carriers), and it is *not* evidence for or against the T2
mechanism.

**The mechanism stated in the addendum above is therefore still unconverted.** It remains consistent
with every measurement and still makes its prediction; this run failed to test it, because the
instrument could not see the signature it was built to move.

## Next step, and what it must fix first

Re-establish the baseline before re-running the intervention: either re-derive the board's own T2
classification on `90986d19` with the board's classifier rather than this regex, or take both arms
at `2a2bd0ad` where the four reversals are known to exist. Until arm A shows a nonzero
`T2_POSITIONS_BOTH_DIRECTIONS`, the intervention has no signal to move and re-running it costs 25
minutes to learn nothing.

---

# Baseline discriminator: the bucketing is sound and the population changed

The two candidates above are separated at zero cost, without a runner, by applying this
extractor's own bucketing rules to the `2a2bd0ad` artifact's own `expected`/`found` rows.

| quantity | this extractor's rules on the artifact | board's own labels |
|---|---:|---:|
| both-direction positions | **4** | **4** |
| T2-like sites | 42 | 34 (`root == T2`) |
| T3-like sites | 21 | 25 (`root == T3`) |

Top signatures produced by these rules on that data: `Rc<Vector<_>> <- String` ×19,
`String <- Rc<Vector<_>>` ×7, `Rc<Vector<i64>> <- String` ×4, `String <- Rc<Vector<i64>>` ×4 — the
board's own top signatures at the board's own counts.

**The discriminator reproduces exactly.** The site totals differ because this rule is deliberately
broader than the board's `T2` label (it also admits pairs the board classified under other roots),
but the quantity the intervention turns on — positions carrying both directions — agrees at 4.

So the extractor buckets correctly, and **the population changed between the trees.** The earlier
ranking in this document, which called bucketing the likelier suspect on the strength of
`TOTAL_MISMATCH_SITES=134` against an independently measured `E0308=135`, was wrong. That agreement
established that the extractor *reads* the whole population; it said nothing about how it *divides*
it, and those are different properties.

## The consequence, which is larger than this lane

**The text-carrier reversal signature has no occurrences on current main.** At `2a2bd0ad`: 34 sites
across those four signatures, 4 both-direction positions. At `90986d19`: zero `Rc<Vector<_>> <-
String`, zero both-direction positions, and the surviving T2-like sites carry a different shape —
`String <- Rc<Vector<String>>` ×8, with a *concrete* element type where the board had an inferred
one.

Not claimed: that anything repaired it. The board fell from 199 E0308 blocks to 135 across the same
interval, and 34 of the missing sites are T2's, but whether they were fixed or moved into another
error class is **not established here**.

What this does establish is that the direction census at the top of this document is scoped to
`2a2bd0ad` **load-bearingly**, not cautiously: its population does not exist on current main, and
any plan that treats T2 as a live 34-site root on today's tree is planning against a stale board.

## The signature was REMOVED, not relocated

The section above left open whether the 34 missing T2 sites were repaired or had migrated into a
different error class. The parent lane (`smart-ram-730`) answered it by joining two retained
per-site boards that bracket the window, at site identity rather than by count:

- `v2_compiler_tokenize.rs`, between `ba63edc09b` (399) and `629252b6df` (339): **24 E0308 sites
  removed, 1 E0282 removed, zero added.** E0308 sites remaining in that file on current main: **0**.
- Whole-board join for the same window: 328 same / 57 removed / **3 new**.

A migration into another class would have surfaced as new sites somewhere in that join, and three
new sites cannot absorb twenty-four. The removed line list includes `225:13 248:13 269:13 337:13` —
the same construct family, at the same column, as the four both-direction positions recorded at the
top of this document, offset by the line drift that file is known to carry.

So the class was genuinely repaired. **Which of the six PRs merged in that window did it is not
established** (#8825, #8829, #8831, #8818, #8828, #8832), and the −39 in E0308 is unattributed.

## Standing of this lane

The mechanism documented here explains a signature that **no longer occurs**. That does not refute
it: the fallback arm still stands, the state-space conflation in it is still real, and the
`integer.dag` precedent shows the class recurs at different aliases. But the lane is no longer
repairing live sites — it is characterising a defect that has stopped firing at one alias for
reasons nobody has identified.

The intervention at `2a2bd0ad`, where the signal is known to exist, is therefore the last work this
lane does. A mechanism that survives an intervention is a durable asset even at a current
population of zero; one that fails it saves the next reader from believing it. Beyond that result
there is no further investment here — no re-derivation on main, no repair design, and no
counterfactual work against a population of zero.

## Calibration at `2a2bd0ad` — PASSES

The live probe re-run at the board's own sha reproduces the board, which the `90986d19` run did not:

| quantity | live probe @ `2a2bd0ad` | board artifact @ `2a2bd0ad` |
|---|---:|---:|
| E0308 blocks | **199** | 199 |
| both-direction positions | **4** | 4 |
| `Rc<Vector<_>> <- String` | 19 | 19 |
| `String <- Rc<Vector<_>>` | 7 | 7 |
| `Rc<Vector<i64>> <- String` / reverse | 4 / 4 | 4 / 4 |

`PROBE_EXIT=0`, empty dirty list, `TOTAL_MISMATCH_SITES=215` against the board's 235 distinct sites
(this regex admits slightly fewer than the board's classifier; the quantity the intervention turns
on agrees exactly).

**P1's precondition is satisfied at this subject.** The earlier run's failure is now fully
explained: the instrument was sound and the population at `90986d19` was empty, exactly as the local
row-level check predicted. The intervention run at this sha is therefore interpretable.

### The 24 removed sites are five or six construct shapes, not one

Grouping the removed `v2_compiler_tokenize.rs` E0308 sites by column (parent lane's artifact):

| column | count | lines |
|---|---:|---|
| 13 | 8 | 196 225 248 269 294 337 359 367 |
| 36 | 5 | 559 585 589 617 620 |
| 99 | 3 | 558 588 619 |
| 16 | 2 | 154 195 |
| 57 / 58 / 62 | 3 | 462 463 464 |
| 15 / 11 / 43 | 1 each | 142 143 682 |

The column-13 group of eight is the construct family carrying the four both-direction positions
recorded at the top of this document. The 36/99 interleaving across 558–620 is a distinct repeating
pattern — three lines each contributing a 99 and a 36, reading as one construct reported twice per
site. The 462/463/464 run at ascending columns 57/58/62 is a third: one multi-line expression.

So whatever removed them either was one change reaching several construct families, or was several
changes inside the six-PR window. A single narrow repair is the least likely of the three. The
attribution remains open and currently has **no dependent**, so it is recorded rather than pursued.

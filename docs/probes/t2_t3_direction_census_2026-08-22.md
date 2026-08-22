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

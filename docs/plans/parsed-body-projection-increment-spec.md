# Parsed-body projection — increment spec (admission decision input)

Status: **specification for an admission decision, not an implementation plan.** No work has
started and none may start on this until the operator rules, because part of the increment lands
in the frozen v1 seed (§5).

Audience: whoever decides admission. This document states what is broken, what would fix it, how
strong the evidence for each claim is, and — separately, because they are different admission
questions — which parts are v1 seed changes and which are v2 `.dag` changes.

Subject measured: `a6ca6882d18114b52532c0804dc89f97b441f493`.
Evidence: → [projection blocker and derived shell seed](shell-dag-census-0a-projection-blocker.md).

---

## 1. What breaks without it

SHELL-DAG-CENSUS-0A must derive every authored path that can attempt POSIX/Bash program
interpretation, from parsed bodies, and its merge bar includes **zero production parse refusals**
and **zero unexplained unknown routes**. That bar is unreachable on today's fact surface — not
difficult, unreachable — and the failure is silent rather than loud.

The sharpest way to say it is not the coverage percentage. It is this:

> **A file that was never loaded reads identically to a file that carries no shell route.**

No per-file `ParseRefused` row exists to count the difference, because non-import is not an event
the surface reports. So a census on this substrate does not produce an incomplete answer that
announces its incompleteness — it produces a clean, reviewable, confident population that is
silently wrong. That is DESIGN's **empty-observation narrow**, and it is the exact artifact this
cut was dispatched to prevent.

Everything below is downstream of that sentence.

## 2. The six axes, with evidence grade

Evidence grade is stated per axis because they are not equally established, and an admission
decision should not treat a code reading and an executed discriminating pair as the same thing.

| # | Axis | Remedy | Evidence grade |
|---|---|---|---|
| 1 | Service operations and transports are absent entirely — `ItemKind::ServiceItem` has no accessor | a `ServiceItem` accessor carrying operations, transport argv, and stdin channels | **Structural, verified independently.** The enum has six variants and exactly three have accessors; confirmed by the dispatching lane |
| 2 | `FnArrowDecl.output` is a wiring-liveness skeleton — a bound-but-unused `let` RHS is dropped | a body projection total over statements: effect positions retained independently of return reachability | **Execution-proven** (discriminating fixture pair, §3) |
| 3 | No named-argument edges — labels erased, literals hoisted to the call node | argument edges labelled with the authored argument name, positional index preserved | **Execution-proven** (same run, §3) |
| 4 | No arm or occurrence identity | stable occurrence identity on body nodes; arm identity on match arms | Read from the marshal (`marshal_generic` emits undifferentiated positional children) |
| 5 | Callee is an authored lexeme, not a resolved identity — a string literal and a constructor are the same atom | resolved declaration identity on callees; a node-kind discriminator separating literal / callee / constructor / variant | Read from the marshal (`atom_identity_node` is the single atom kind) |
| 6 | The registry is the entry's import closure, not the corpus | a corpus-grain denominator that is an enumerated file set, not an import closure | **Execution-measured** (§4) |

### 3. The execution proof for axes 2 and 3

Two functions of identical shape, folded through `fn_arrow_decl_facts_live`, atom identities
collected from each `output`:

```text
fn fixture_dead_let_shell() -> String {
  let unused_result = fixture_sink(program: "echo DEADLETMARKER")
  "returned-without-using-unused_result"
}
fn fixture_live_named_args() -> String {
  fixture_sink2(program: "echo LIVEMARKER", args: "echo ARGSMARKER")
}

DECL probe.fixture.fixture_dead_let_shell   ATOMS: (empty)
DECL probe.fixture.fixture_live_named_args  ATOMS: fixture_sink2 | echo LIVEMARKER | echo ARGSMARKER
```

The dead-let arm yields **nothing** — callee identity and program literal both absent. The live
arm yields both. Same construct, opposite verdict, so the loss belongs to the projection and not
to the probe. The live arm simultaneously demonstrates axis 3: three atoms in authored order with
**no labels**, so nothing records which literal was `program:` and which was `args:`.

### 4. The measurement for axis 6, with its counting method stated

A fold over `fn_arrow_decl_facts_live()` under `--source-root dag --source-root src/v2` reported
**1,698** declarations.

The denominator depends on how declarations are counted, so the method is stated rather than the
bare number:

| method | count | note |
|---|---|---|
| line-start `fn` / `func` / `test fn` | **41,965** | the accessor's filter is `ItemKind::FnItem \|\| FuncItem`, and `ItemKind` has no separate test variant, so a `test fn` **is** an `FnItem` — this is the matching denominator |
| line-start `fn` / `func`, excluding `test fn` | 30,851 | the same count with the 11,114 test declarations removed |

The conclusion is invariant under the choice: **1,698 of 41,965 is 4.0%; of 30,851 it is 5.5%.**
Either way the surface is a small single-digit fraction of the tree, and its content is whatever
the entry happened to import.

This is a declared frontier rather than a discovery, and it is already guarded:
`v2.lens.affected_set.corpus_dependency_view` calls `fn_arrow_decl_substrate_is_whole_tree` and,
when false, routes to a host refusal reading `corpus_dependency_view per-PR execution refused …
(blocked-on-#6239)`. Fail-closed, correctly. **A census inherits that refusal**, which is why
axis 6 is plausibly not new work at all — see §5.

## 5. Which parts are v1 seed changes and which are v2 `.dag` changes

Stated separately because they are different admission questions and bundling them makes the call
harder than it needs to be.

| axis | v1 seed (`src/v1/stage0/src/coproduct_reflection.rs`) | v2 `.dag` | notes |
|---|---|---|---|
| 1 | **yes** — a new `eval_service_decl_facts_live` accessor | yes — the carrier type and its `.dag` surface | additive |
| 2–5 | **yes** — a new non-lossy body marshal | yes — the carrier types | **additive, not a modification** (below) |
| 6 | **probably none** | none | likely already-sanctioned pending work (below) |

Two properties of this decomposition matter for admission:

**Axes 2–5 are an additive second accessor, not an edit to the existing one.** The existing
marshal's lossiness is not a defect — dropping a dead `let` RHS is precisely what
`v2.lens.wiring_liveness` needs, and its comments say so. Changing it would break that lens. So the
increment adds a second projection beside it and leaves the existing behaviour untouched. That
keeps the blast radius off every current consumer, and it means the freeze question is "may the
seed grow a new accessor", not "may the seed's existing semantics change".

**Axis 6 is probably not this increment's to fix.** The whole-tree registry path already exists and
is gated on #6239, with a typed refusal already in place. If that work lands, axis 6 closes without
anything here. It is listed for completeness, not as a request.

## 6. The cost argument for doing it at ingestion rather than corpus-wide

The alternative — a corpus-wide fold that reads source and parses it in `.dag` via
`v2.compiler.source_authority` `parse_dag_source_ast` — was measured rather than assumed, and it
fails on **both** axes:

- **Correctness.** On a random 10-file corpus sample: 6 accepted, 4 refused, and all four refusals
  carry the same reason as the earlier `dag/extdeps/shell/exec.dag` refusal —
  `parse_grammar_choice_overlap_residue`. One grammar deficiency with many victims. The refusal set
  contains the census's own seed file, and the merge bar is zero production parse refusals. Filed
  separately: → [parse grammar choice overlap residue](parse-grammar-choice-overlap-residue-finding.md).
- **Cost.** 1 file / 63.1 s; 10 files / 67.5 s; 40 files / **`EXIT=137`, OOM-killed after 34 files**.
  Marginal cost is ≈0.49 s per file against ≈62.6 s of fixed world-acquisition overhead, so time is
  not the wall. Memory is — and not because of large files: the kill came at file ~34 of 40 with
  only **94 KB** of source consumed, on ~7.8 KB files, the 212 KB outlier sorted last and never
  reached. The mechanism (parse-tree retention vs interpreter heap growth) is **not** established
  by this probe and is not claimed. What is established is that the process cannot hold the result
  of parsing 34 small files, against a subject of 3,733 files and 31.3 MiB.

DESIGN §6 already rejected this shape once, and the passage is the reason to prefer ingestion:
the corpus-wide censuses were deleted in #8140 because *"the unit of computation was the world, the
unit of fact was one module's authorship, and the price was paid by every consumer that wanted a
witness roster and nothing else."* Its declared next-rung trigger is exactly this increment's
shape: a fact *"belongs on the module's own declaration, checked at ingestion where the module is
parsed anyway — one module's facts from one module's source — rather than reconstructed corpus-wide
by a consumer that wanted something else."*

## 7. The admission question, stated without advocacy

The v1 seed is frozen with an admission test of **purpose**: a change is admitted when it serves
the v2 self-host program. This increment does not obviously pass that test, and the decision is
not the requesting lane's to make. Both readings are recorded here rather than one:

**For admission.** A non-lossy body projection is the substrate capability *any* consumer needs to
read real bodies rather than a wiring skeleton. The existing surface is the reason
`v2.lens.effect_reach` classifies host-effect sinks by name equality against a remembered callee
list — not because its author chose a weak principle, but because the surface it reads cannot
express a stronger one. Every future lens wanting real bodies inherits that ceiling.

**Against admission.** The requesting program is a shell-migration census, not v2 self-host. Seed
growth justified by a neighbouring program is how a freeze with an active-maintenance arm becomes
an absorbing one, which `gunbc.v1_maintenance_standing` names as a standing danger.

If admission is refused, the census stays blocked and honestly so. That is a worse outcome for
this lane and a correct one for the repository, and it is preferable to a census whose population
is confident and silently wrong.

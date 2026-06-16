# Design: Lens-Subject Supply — Real Compiled Bodies as Lens-Witness Subjects (No Reflection)

> **Status: DESIGN — handoff seam only, 2026-06-11 (swift-bat-315).** Pairs the landed
> COMPREP producer (`design-computation-representation.md`, #4608/#4624/#4660) with the
> landed cost/complexity lens stack (`src/v4/lens/cost.dag`, `src/v4/lens/complexity.dag`).
> Both ends exist; this designs only the connection: how a `-> Bool` lens witness obtains a
> REAL compiled function body as its `Node` subject. First consumer: the complexity-gate
> exemplar row (work item adhoc-4e07f6f5-ebc / S1a). No code in this doc.

## 1. The problem

Every `lens_cost` / `lens_complexity` claim today folds a hand-built `SyntheticOccurrence`
fixture `Node` (all marked "scaffold — compile-only"). The E2 mutation matrix (2026-06-07)
showed why that is not evidence: mutate the real declaration, leave the mirror → still
green. A complexity gate is only a gate if the lens folds the body the compiler actually
produced.

The obvious mechanism — a `.dag`-callable `body_of("name")` — is **banned** (operator,
2026-06-07; gunbc#4506 CLOSED): dynamic, name-keyed lookup over the loaded module table is
metaprogramming that leaves the typed stack. The ruling's line: *static expansion the
compiler derives from a declaration it owns = fine; dynamic name-keyed runtime lookup =
banned.*

## 2. The mechanism: source-as-data through the compiler's own pipeline (COMPREP pattern)

The sanctioned shape already exists and is post-ban-landed:
`comprep_source_bridged_add_arrow_with_body()`
(`src/v4/test/claim/manual/comprep_eval_by_execution.dag:293`) runs

```
tokenize(dag_mvp1_source_text) → parse → normalize → resolve
  → produce_mvp1_add_arrow_with_body_from_resolved
```

entirely inside `.dag`, over a **source text held as a data value**, and returns
`Outcome<Node>`: an `Arrow` carrying a real `arrow_body_edge` body. There is no lookup
over the loaded module table — the witness statically imports the pipeline functions and
compiles source it holds as data. This is in-stack by the ruling's own line: the compiler
deriving structure from input it owns.

It is also non-tautological where fixtures are not: the subject is *produced by the
pipeline*, so perturbing the source (or the producer) perturbs the subject — the COMPREP
keystone's swapped-operand red already demonstrates the discrimination.

**The handoff is therefore a pure `.dag` composition, no host change:**

1. **Subject acquisition** — call the source-bridge producer to get the compiled
   `Arrow`-with-body (`Outcome<Node>`).
2. **Body extraction** — the canonical accessor `arrow_body_target_lookup`
   (`src/v4/std/node.dag:302`). Do NOT write a local edge-walker next to it
   (INVARIANTS "second path" anti-pattern).
3. **Fold** — `cost_lens(n: body)` → `Witness<SymbolicCost>`
   (`src/v4/lens/cost.dag:429`).
4. **Project** — `asymptotic_class_of_cost` / `complexity_lens` (complexity.dag is the
   asymptotic PROJECTION of cost.dag's carrier — consume, never re-derive).
5. **Gate** — `complexity_bound_dominates(declared, computed)` against the declared
   budget; `ClassUnknown` is the fail-closed top.
6. **Fail closed** — every `Rejected` on the way to a subject returns `false` (no
   subject → red, never vacuous green).

The witness is an ordinary `-> Bool` entry run by the existing host boundary
(`gunbc run --claim-run --entry --function`), wired as one new row in
`lens_ci_claim_run_rows` (`src/v4/workflow/lens_ci_gate.dag`), perturb-checked by
the v4 lens CI rows-fn invocation in `scripts/v4-affected-tests-gate.sh` (gate-3).

### Placement

The acquisition+extraction composition (steps 1–2) is a small additive helper. Home it
with the consumer (under `src/v4/test/claim/` next to the gate row, or `src/v4/lens/` as
an additive module) — NOT in `cost.dag` / `application.dag` / `05_eval.dag`, which carry
operator-STOP headers. It imports `v4.compiler.body_producer` and `v4.std.node` only.

## 3. Honest labeling: the subject set rides the COMPREP wave ladder

What can be a subject = what the v4 pipeline can ingest = COMPREP grammar coverage.
Today that is **wave 1: the MVP `add` production**. So the first real subject is
**source-ingested `add`** — a real compiled body, but not yet a compiler-stage function.
`02_parse` functions are full v4 grammar (match, let, generics); the self-pipeline cannot
parse them until COMPREP waves 2+ land, and full stage coverage arrives with self-host
breadth (wave 4).

This is the same honest-labeling discipline as the emit ladder's signature-tier relabel:
the exemplar row must say "source-bridged `add`", not claim stage coverage it doesn't
have. The gate's coverage then grows **automatically with COMPREP** — each grammar wave
makes more real functions ingestible, with no new mechanism. Complexity gating needs no
node of its own in the dep graph beyond this seam; its subject reach is an OUT-edge of
COMPREP.

## 4. Rejected alternatives

- **R1 — runtime reflection primitive.** Banned (2026-06-07). Not revisited.
- **R2 — host-boundary supply** (v2 stage0 grows a `--subject <file:fn>` flag, compiles
  the file, encodes the function body from its internal representation into a v4 `Node`
  runtime value passed to the witness). Rejected: grows the bootstrap seed (Rust shrinks
  toward zero); requires a v2-internal-repr→v4-`Node` encoder, which is a *second*
  body-access path next to `body_producer` (INVARIANTS "second path" anti-pattern) and a
  new parallel representation seam to keep faithful; and its coverage advantage over §2 is
  temporary — self-host closes it, leaving the encoder as debt.
- **R3 — compile-time `body_of(<static fn reference>)` sugar.** Legal under the ban's
  line (the compiler expands from a declaration it owns), but it is grammar/substrate
  surface — operator-STOP class — and unnecessary while §2 suffices. Revisit only if a
  consumer needs body access where source-as-data is impossible; that revisit is an
  operator decision, not a worker improvisation.
- **R4 — hand-built fixture Nodes.** Status quo; tautological per E2. Existing scaffolds
  stay as algebra unit-claims but must never be presented as gate evidence.

## 5. The budget side (S1b interface)

Declared bounds must not reintroduce name-keyed lookup. A budget row therefore binds to
the **subject producer**, not to a string name: each roster row pairs a subject-producer
function reference (the §2 composition for one function) with its declared
`ComplexityBound`, and the gate row consumes the pair. Missing budget for a discovered
subject = fail-closed red (S2 scope, after S1 lands).

## 6. Perturb-check obligations for the exemplar row

The standard `--perturb-check` (witness body rewritten to `false` must go red) plus one
semantic red the row should carry in-witness: a deliberately tightened budget (declared
class strictly below the computed class) must yield `false` via
`complexity_bound_dominates` — proving the dominance check, not just the plumbing,
discriminates.

# `parse_grammar_choice_overlap_residue` — a named grammar deficiency nothing executes

Status: finding, filed independently of the census that surfaced it. It outlives that census.

**The unexecuted law and the unmeasured deficiency are the same fact seen twice.**

Subject: `a6ca6882d18114b52532c0804dc89f97b441f493`.

---

## The finding

`v2.compiler.source_authority` `parse_dag_source_ast` refuses a large fraction of authored `.dag`
source, always with one reason: `parse_grammar_choice_overlap_residue`.

Measured with a positive control in the same run, so a harness fault could not masquerade as a
finding:

```text
CONTROL_3LINE  the tree's own "module m / fn add" fixture      => ACCEPTED
REAL_SMALL     dag/gunbc/bash_materialized_transport.dag       => ACCEPTED
REAL           dag/extdeps/shell/exec.dag                      => REJECTED
                                        reason = parse_grammar_choice_overlap_residue
```

Then a random 10-file sample of real corpus files:

```text
A  src/v2/std/constraint_satisfaction_predicate.dag
R  dag/test/claim/filesystem_read_hermetic_witness.dag                parse_grammar_choice_overlap_residue
R  dag/test/claim/wet_hermetic_equivalence_witness_test.dag           parse_grammar_choice_overlap_residue
R  src/v2/test/claim/.../cargo_fmt_dead_param_test.dag                parse_grammar_choice_overlap_residue
A  dag/test/fixture/sole_constructor_sealed/admitted_caller.dag
A  dag/extdeps/transports/file.dag
A  src/v2/workflow/host_discovered_owned_data_manifest.dag
A  src/v2/lens/structural_similarity.dag
R  src/v2/test/claim/round_trip/source_authority_contract_test.dag    parse_grammar_choice_overlap_residue
A  dag/extdeps/cache/catalog_placement.dag

6 accepted, 4 refused — all four the same reason
```

**The grouping is the finding, not the rate.** Ten files do not support a corpus refusal rate and
none is claimed here. What ten files do support is that five separate refusals across two source
roots — four sampled at random, plus `extdeps/shell/exec.dag` found independently — share a single
named cause. This is one grammar deficiency with many victims, not a scattered set of
file-specific problems. Group by *why*, not by where the fix would be typed.

## Why it is invisible

`parse_dag_source_ast` has **no consumer**. Its only two call sites are inside
`canonical_dag_source_parse_print_law`, and that function's name occurs exactly once in the entire
tree — its own definition. It has no callers.

So the deficiency is not hiding. It is simply never asked: the only code path that would surface
it is a law that nothing executes. That is DESIGN §5's **specification-without-execution** class,
sitting in the compiler's own source authority — a parse/print round-trip law, written, typed,
and never run.

The two halves are one fact. A law nobody runs cannot report the deficiency it would catch, and a
deficiency nobody measures leaves the law looking healthy. Executing the law is what turned an
unknown into a named reason with a file list.

## Why this matters beyond the census that found it

1. **The v2 self-host program depends on this path.** `parse_dag_source_ast_with_model` sits under
   `semantic_ir_from_source_with_model` and the normalize/resolve chain — the same
   tokenize→parse→normalize→resolve pipeline the `wave1_gate1` body-producer witnesses exercise on
   fixtures. Those witnesses pass on hand-authored three-line modules. The refusal population lives
   in *authored corpus source*, which is where self-host has to work.
2. **Fixture-grain evidence is not corpus-grain evidence.** The gap between "parses `module m / fn
   add`" and "parses `extdeps/shell/exec.dag`" is exactly the gap between a green witness and a
   working compiler, and today only the first is measured.
3. **It is a wall on any future consumer of the parse route**, not just this census — including the
   corpus-parse alternative considered in the
   [projection increment spec](parsed-body-projection-increment-spec.md), which this finding is
   part of the reason to reject.

## What is not claimed

- No corpus refusal rate. Ten files is a sample that establishes a shared cause, not a proportion.
- No root cause in the grammar. The reason symbol names a choice-overlap residue; which
  productions overlap, and whether the fix is one rule or many, is not established here.
- No claim that repairing it makes the parse route viable for corpus-grain work — the separate
  memory measurement in the increment spec says it would not.

## Next step

Measure the refusal population properly — every file under both production roots, verdict and
reason per file, emitted as a structured artifact — and reduce the refusals to the grammar
production(s) at fault. That is a bounded measurement on an existing route and needs no seed
change. Repairing the grammar is downstream of knowing which productions overlap.

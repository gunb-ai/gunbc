# Wave-1 occurrence-join, census-row evidence

Evidence for the admission gate at `dag/gunbc/instruments/namespace_wave_occurrence_join.dag`
(wired into `gunbc.instruments.namespace_wave_binding_diff`), and for the test label
`gunbc test //gunbc/instruments:namespace-wave-occurrence-join-controls` registered beside it.

The join pairs a module's BASE census rows with its HEAD census rows on the key
`spelling @ <line>:<column>`, where the head position is the base byte offset carried through
the wave's own edit rounds (`map_through_rounds`) and re-rendered by `locus_at`. This file is
the claim that the REAL census emits rows of exactly that shape — the controls themselves
derive every locus from a text's own `chars(` sites, so a hand-typed locus can never be the
thing that makes a control pass.

## Provenance

    instrument   //gunbc/instruments:namespace-wave-binding-diff  (exit 0)
    dispatch run 40268668972, gh run download -n instrument-dispatch-receipt-base
    base tree    e54fbfba9d18ab3bb9e80a1e1377b81c6c3bfe7a
    shard file   base-e54fbfba9d18ab3bb9e80a1e1377b81c6c3bfe7a-61ba94f9196d655f-3ef27d84750332df.txt
    sha256       5e26e4b7a5f9967a22cf3c2c40f1094a3ce90052297dd8e96b13d81b8d689c42

## What the census prints (452,429 base_row lines)

    row kind                          count     the join consumes it as
    binding                           31,782    located row: join on (spelling, line:column)
    binding_partial                  390,980    located row: same
    binding_unbound                   16,611    located row: same; joins to a declared head
    binding_unanswered                 4,475    not joined (unanswered is the gate's input, not a site)
    binding_module / ..._module rows  1,221+1,967+1,967  module-level tallies incl. refused modules
    binding_unreached                  2,013    not joined
    ambient_summary                         33    header rows
    kit_fixture                          158    not joined
    module_subjects                    1,221    not joined

Every `binding`, `binding_partial` and `binding_unbound` row carries either `<line>:<column>`
or exactly `-`:

    binding          located=29,124   unlocated=2,658
    binding_partial  located=364,590  unlocated=26,390
    binding_unbound  located=16,482   unlocated=129

Located rows enter the position join. Unlocated rows (`-`, generated sites the census cannot
place) never enter it; they are admitted only as an exact multiset of
(module, spelling, source, target) — `join_generated_multiset`. So a generated row that moves,
appears or disappears is refused (`GeneratedChanged`) without the gate ever pretending to know
where it is.

Refused modules still print their unbound rows plus a `binding_unbound_module` tally (and the
companion `binding_file` shape in `occurrence_binding_census.dag`), e.g.

    gunbc.ci.ci_compile_jobs                 unbound_rows=0   flagged=1
    gunbc.roadmap.dispatch_worker_traversal  unbound_rows=3   flagged=1
    v2.lens.reference_deps                   unbound_rows=20  flagged=1

## Spot checks (locus points at the spelling it claims)

    extdeps.bmc.http.dag                      chassis_id              227:127   `assis/{chassis_id}/Th`
    a_comparator_read_the_inspect_field...    MechanicallyPreventable  10:50    `rung: MechanicallyPreventab`

Column is 1-based, line is 1-based from the FIRST line of the file (never from the end);
`offset_at` turns (line, column) into a byte offset and `locus_at` turns it back, so the two
are inverses on any offset inside the text and `offset_at` refuses (returns -1, giving
JoinUnobserved "the census locus lies outside the base text") on a line past the end.

## What is NOT claimed here

No ordinal, rank or "same-line rank" is part of any join key: two sites in one line are
distinguished only by their columns, and two rows with the same key on one side are refused
as `Collided`, never disambiguated by position in the print order. The print order itself is
not trusted anywhere: the census prints a module's rows in an order that differs between runs
(1070 of 1331 modules, measured), so "same rows" is multiset equality (`rows_multiset_equal`).

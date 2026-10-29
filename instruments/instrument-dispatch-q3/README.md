# Q3 provenance probe — fixture + truth table (staged for the #14526 fix cycle)

ROOT CAUSE (fleet defect 2026-10-26): the Q3 repair's run_produced_by_this_workflow probed
(w.get('workflow') or {}).get('path'), a shape I invented in a hand-built fixture. The GitHub
single-run endpoint /repos/<repo>/actions/runs/<id> returns workflow: null and carries the file
at the TOP-LEVEL 'path' key. Every real candidate failed identity -> zero base files staged ->
shards refused 'no base receipt' (wm707 kit-e2e 40290883213, 40290874699, 40290877223, 40290880333).

FIXTURE PROVENANCE (per tidy-ant-251): recorded_run_40219194854.json is the RAW response of
`gh api repos/gunb-ai/gunbc/actions/runs/40219194854` (this workflow's base leg, head
9e4d16a5bc3; fetched 2026-10-26T10:35Z; fields stripped ONLY by the probe harness's consumers).
recorded_run_40307182339_foreign.json is the RAW response for a heal.yml run (must reject).

NEGATIVES kept: foreign workflow run (40307182339, path=.github/workflows/heal.yml);
nonexistent run (HTTPError -> False); workflow:null shape with the WRONG top-level path.
`workflow` is null in EVERY real runs/<id> response (both files above), so any predicate reading
w['workflow']['path'] rejects everything: fail-closed, never wrong-green.

THE PROBE HARNESS (out of tree by design; the repo ignores *.py and a committed script would be a
second execution authority beside the instrument labels, DESIGN 6): content sha256
0d500ea4f02384863cd4045e8f559866d77fdeb6c0f8648ca18affcb53ed2719, invoked as
`python3 instruments/instrument-dispatch-q3/probe_truth_table.py` from a working copy kept
outside the tree. It asserts the truth table against the LIVE API: old predicate False on all
legit candidates, fixed predicate True on 40219194854 / 40268668972 / 40205818229, False on the
foreign run and a nonexistent run. Its executed output is pasted in the PR body as evidence.
THE DESIGN 5 RED's executing consumer is THE FLEET RUN, not the script: the RED twin shards
(t02-red14526, shard 3 withheld) exit 2, and the twin's merge/4 refuses naming shard 3.

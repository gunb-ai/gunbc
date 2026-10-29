A16 PRINTED-PART GOLDEN MANIFESTS: the baseline the geometry refactor is diffed against

What: printable_manifest (product.printed_chassis.printable_assembly) for every PrintedAssemblyRoot
whose wire starts with "a16", via select_printable_assembly, one JSON object per line, in root order:
a16-plug-duct, a16-sleeve-duct, a16-plug-fit-ring, a16-sleeve-fit-ring, a16-blower-duct,
a16-blower-flat-on-end-duct. All six were admitted (PrintableAssemblyReady).

Tree: main (with #15369, 48285a7) merged with PR #15243 head bf338f7d4324, on branch
session/loyal-lark-497-geometry, before any geometry refactor. Taken 2026-10-29.

Producer (re-derive, do not trust the bytes): copy golden_entry.dag.txt to
dag/test/claim/zz_duct_golden_scratch.dag and run
  gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/zz_duct_golden_scratch.dag
then keep the lines starting with '{'. Delete the scratch entry afterwards; it is not a witness.

Why: the brief's hard constraint. The A16 plug fit ring is printed and FITS
(product.printed_chassis.fit_observations plug_fit_ring_fit). After the refactor derives every
dimension from extdeps card, fan and CEM rows, the A16 manifests must equal these byte for byte, or
differ only where a cited component dimension says the old value was wrong, with each difference and
its source stated in the PR. Expected candidate: the blower ducts' outlet aim, once it is derived from
extdeps.fans.delta delta_bfb1012m_outlet.

Dissolves: when the refactor PR lands with its A16 diff stated, this directory is deleted in that PR;
the preserved dimensions then stand on witnesses, not on this snapshot.

REFRESH 2026-10-29: #15243 merged as c6959e2 with the blower roots' notes/markers rewritten to cite
delta_bfb1012m_outlet (text only; the dimensions are unchanged). a16_manifests.jsonl was re-derived at main
+ c6959e2 with the producer above: roots 1-4 (plug-duct, sleeve-duct, both fit rings) are byte-identical to
the bf338f7 capture; roots 5-6 differ from it only in those cited-outlet note strings. This is the baseline
the refactor is diffed against. The directory name keeps its original label.

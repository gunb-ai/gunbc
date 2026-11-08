## wc-probe-v2 classifier + grammar fixture output (local run of the frozen fixtures block)

Keeper form: the fixtures block from the frozen bytes verbatim; live cgroup reads shimmed to the booked values; the single live scope-gate line omitted; nothing else changed. No gunbc/claim execution.

```
$ bash fx-harness-v2.sh
fixtures: all held
FXRUN_OK
$ echo $?
0
```

Covered by the block, in order: frame fixtures (1)-(6) as v4 (events parser, REAL completion line + sha, rc contract, scope glob suffix, tree_clean, out_reusable, hash_of); (7) probe classifier: valid EXIT3 with verbatim reason capture, duplicate->NONE, PASS, bare FAIL, code 1->OTHER, code 2->UNOBSERVED, unclosed paren->OTHER, code 33 boundary->OTHER, NotBool->OTHER, code 4->EXIT4; (8) bound arithmetic 41+4*38=193<=193; (9) grammar through the shipped code: valid OK, valid RF, garbage, extra field, unknown tag, r=24, leading zeros, bare RF, nested paren, valid maximum 193, direct 194-char length refusals, 20-digit al.

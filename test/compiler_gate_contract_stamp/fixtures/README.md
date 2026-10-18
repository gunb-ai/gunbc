# The silent-green fixture

`silent-green-head-witnesses.yml` is the `.github/workflows/witnesses.yml` **as committed at
`5003782716`**, one of the four heads that ran the OLD parse-population verb under the NEW
workflow, printed the old counts shape (no `ancestor=`), reported `reused=0`, and exited **green**:

| run id | head | result |
| --- | --- | --- |
| 39066871919 | `5003782716` | success, `ancestor=` absent, `reused=0`, `miss_identity_absent=8692` |
| 39067193189 | `9c7c9def31` | success, same shape (`miss_identity_absent=8700`) |
| 39067690307 | `3612d1c513` | success, same shape (`miss_identity_absent=8696`) |
| 39069382384 | `61f2777b72` | success, same shape (`miss_identity_absent=8693`) |

All four share this **one** committed artifact (`sha256 be4c68f495dada62…`, blob `beaf4ebc4a4b…`);
they differ in source, not in workflow text. Recorded in
`dag/gunbc/recurring_failure_mode/instruction_and_subject_resolved_from_different_revisions.dag`.

It is committed rather than fetched by sha because those four heads live on **deleted lanes**: the
blobs are reachable from the object store, but a fresh clone has no ref that reaches them, so a
reviewer running `./run_evidence.sh` must not depend on a fetch that can fail.

`../run_evidence.sh` runs the CURRENTLY COMMITTED guard (extracted from
`.github/workflows/witnesses.yml` by `../extract_guard.py`, never retyped) against this file. The
guard must **refuse** it: the tree declares no `GUNBC_COMPILER_GATE_CONTRACT_STAMP`, so its
workflow predates the instruction that is being executed. Green here means the guard is not looking.

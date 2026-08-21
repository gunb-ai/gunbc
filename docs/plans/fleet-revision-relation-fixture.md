# The fleet-revision-relation fixture repository

Rebuild recipe for the controlled git repository behind
`dag/test/claim/fleet_revision_relation_wet_matrix_test.dag`.

## Why a fixture repository at all

The pure merge-base fold is already witnessed by authored exit codes, and
`git.Inspect.MergeBase` itself is witnessed by record/replay. Neither covers the
*composition* — `observe_fleet_revision_relation_in` → `MergeBase` →
`fleet_revision_relation_from_merge_base` — which is where a swapped operand, a dropped
`trim`, or a mis-decoded stdout would live. A fold fed authored inputs cannot catch any of
those, because authoring the inputs is the step under test.

## The graph

```
        B  main
       /
  A ──┤   base-a
       \
        C  sibling

  D  orphan      an independent root, no common ancestor with anything above
```

Every commit carries a branch **on purpose**. `C` was originally created on a detached
HEAD, which left it unreachable; `merge-base` still answered because the object had not
been collected yet. A fixture whose correctness depends on gc not having run is not a
fixture.

## Recipe

Identities and timestamps are fixed because the object ids participate in the operation
input hash: a machine-dependent id makes the recorded fixtures unportable.

```sh
FX=target/test-fixtures/fleet-revision-relation
rm -rf "$FX"; mkdir -p "$FX"
export GIT_AUTHOR_NAME=fixture GIT_AUTHOR_EMAIL=fixture@invalid
export GIT_COMMITTER_NAME=fixture GIT_COMMITTER_EMAIL=fixture@invalid
export GIT_AUTHOR_DATE="2020-01-01T00:00:00+0000"
export GIT_COMMITTER_DATE="2020-01-01T00:00:00+0000"
git -C "$FX" init -q -b main
echo a > "$FX/f"; git -C "$FX" add -A; git -C "$FX" commit -q -m A
git -C "$FX" branch base-a
echo b > "$FX/f"; git -C "$FX" add -A; git -C "$FX" commit -q -m B
git -C "$FX" checkout -q -b sibling base-a
echo c > "$FX/f"; git -C "$FX" add -A; git -C "$FX" commit -q -m C
git -C "$FX" checkout -q --orphan orphan
echo d > "$FX/f"; git -C "$FX" add -A; git -C "$FX" commit -q -m D
git -C "$FX" checkout -q main

mkdir -p target/test-fixtures/not-a-repository   # case 7 needs a real non-repository path
```

Verified reproducible: two independent builds produced identical `show-ref` output.

## Object roster

| ref | commit | oid |
|---|---|---|
| `base-a` | A | `3728e5635c811256194971bcaed367914a6040e6` |
| `main` | B | `3ad80d79bd1f64a7f75e121d7323aed053a81f0b` |
| `sibling` | C | `d0d3a7f6e323428bd771a32b1a19554481e1ae7c` |
| `orphan` | D | `8286843fce4511d0157573ba33b48ba46e48958a` |
| — | absent, decodable | `1111111111111111111111111111111111111111` |

The absent id must be **decodable**, or the probe is never dispatched and the case proves
nothing about exit 128.

## The seven cases, and what each one is for

| # | current, accepted | exit | relation | why it is in the matrix |
|---|---|---|---|---|
| 1 | B, B | — | `SameRevision` | short-circuits before any dispatch |
| 1b | B, B (non-repository path) | — | `SameRevision` | the short-circuit does not depend on the repository existing |
| 2 | A, B | 0, base = A | `DeployedIsAncestor` | ordinary forward advance; base **is** current |
| 3 | B, A | 0, base = A | `CandidateIsAncestor` | superseded; base **is** accepted |
| 4 | B, C | 0, base = A | `UnrelatedHistories` | diverged **with a shared ancestor** — the case whose receipt used to claim a disjoint history |
| 5 | B, D | 1 | `UnrelatedHistories` | no common ancestor; exit 1 is an **answer** |
| 6 | B, absent | 128 | `RelationUnverifiable` | could not look — must not collapse into 5 |
| 7 | B, C (non-repository path) | 128 | `RelationUnverifiable` | a second route to 128; if it ever answered a relation, the observation is reading some other repository |

Cases 5 and 6 are additionally asserted as one row
(`no_merge_base_and_could_not_look_stay_distinguishable`) so the distinction cannot be
half-satisfied. A consumer reading the operation's `success: Bool` instead of its exit code
cannot tell them apart at all — the live residual recorded on `MergeBaseOutcome`.

## Record, replay, and the controls

Commands are in the exclusion row's dissolution description
(`gunbc.ci_layer_roots`, pattern `fleet_revision_relation_wet_matrix_test.dag`) so that the
recipe and the reason a reader is looking it up sit together.

Measured results:

- **record (wet)** — 9/9 PASS, 6 distinct fixtures written for `git.Inspect.MergeBase`
  (six, not seven, because case 5 and case 6 each appear twice across the nine rows).
- **replay (hermetic, exact store)** — 9/9 PASS.
- **control: empty store** — the two short-circuit rows still PASS; the other seven FAIL
  with `missing recorded fixture`, each naming a **distinct** input hash. This is what
  proves the passing replay consumed recorded observations rather than reaching live git,
  and separately proves the short-circuit is real.
- **control: wrong input hash** — one fixture renamed to a bogus hash, the rest left in
  place. Its case refuses with its own exact hash while a sibling case still passes from
  the same store. The store is an exact observation map, not a nearest-match catalog.

The recorded `inputs` array includes `repository_path`, which is the direct evidence that
the repository participates in the key rather than being ambient.

## What this does not cover

The file does not run in the required floor. `git.Inspect.MergeBase` publishes no
`mock_response`, and there is no way to hand recorded shell observations to a floor run:
`WitnessEvaluationFrame` carries only `rest_fixtures`, and `--fixture-store` is a
`claim_batch` flag with no `claim_executor` equivalent. The exclusion row carries the
substantiated reason and the dissolution trigger — a shell/service fixture arm on
`WitnessEvaluationFrame`, after which these rows enroll unchanged.

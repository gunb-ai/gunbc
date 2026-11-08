# OPTION-A proposal (review material only — NOT executed, NOT authorized)

**What it is.** The freeze-2 seed `4a9efe08…` is not reachable on GitHub, so a frame that fetched it by pin would
spend the one granted attempt at the fetch. This document describes what publishing it would involve. It is review
material only: **no push and no auto-PR closure is authorized** (keen-otter `msg_3646a6d4`, `msg_81c240d4`). The
packet's frame does NOT use this path — its `MODE=B` is a frozen literal that rebuilds the seed from published
bytes. If Option A is ever granted, the frame changes and must be re-reviewed.

## 1. The commits, in full, with a rebuildable parent chain

| role | commit | tree | reachable on GitHub today |
|---|---|---|---|
| seed | `4a9efe081d8046969eb4e9ef06d219e27ce31113` | `a0de248f1918aff315145392e2057db6ecc66c1a` | **no** |
| base (seed's only parent) | `57ce2e787aa5227fc1840961c607981f4abdde86` | `5a14ac7ca50f43312760f092c86ebee7c679ce37` | **no** |
| base parent 1 | `a4a21da4…` | `c7c77919…` | yes |
| base parent 2 | `7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9` | `ab6a5011c7d45107b56f4b95e989fc9919fa5b6a` | yes, as a reachable object |

So only **two** objects are missing, and both sides are fully determined: `4a9efe08` = base + the seed patch, and
the base = merge of the two published commits + the base-side patch. The raw commit objects are attached
(`build/raw_seed.commit`, `build/raw_base.commit`) together with both patches, so either sha can be rebuilt
bit-for-bit offline, author and committer identities and dates included, without any fetch:

* `git diff 7d062bda 57ce2e78` → `BASE_PATCH_7d062bda_to_57ce2e78.diff` (18 files, +69/−69)
* `git diff 57ce2e78 4a9efe08` → `SEED_FREEZE2_4a9efe08.patch` (1 file, `src/v2/test/claim/namespace_xl0/return_tail_position.dag`)

## 2. The ref, and the remote state before and after

* Proposed ref: **`refs/freeze2/cc1-15695-a0de-v18`** → `4a9efe081d8046969eb4e9ef06d219e27ce31113`. It is neither a
  branch (`refs/heads/…`) nor a tag (`refs/tags/…`), which is what makes the effects in §4 analysable rather than
  "whatever branches do".
* BEFORE, measured in this session and re-measured before posting (read-only, no write):
  `git ls-remote origin refs/freeze2/cc1-15695-a0de-v18` → **no output** (rc 0); `git ls-remote origin
  4a9efe08… 57ce2e78…` → **no output**; a `--depth 1` fetch of either pin returns 422 "not our ref";
  `refs/heads/main` measured `8653d24f4998…`.
  "Reachable" for `7d062bda` means the different thing that matters here: it is not a ref tip (so it does not
  appear in `ls-remote`), but `git fetch --depth 1 origin 7d062bda…` SUCCEEDS — which is exactly what MODE B
  depends on and what `recon_fixture_v4.sh` exercises on every run.
* AFTER: `git ls-remote origin refs/freeze2/cc1-15695-a0de-v18` → exactly
  `4a9efe081d8046969eb4e9ef06d219e27ce31113 refs/freeze2/cc1-15695-a0de-v18`, and nothing else changes.

## 3. Verification after the push (each a refusal condition, before any build)

1. `git fetch --depth 1 <repo> refs/freeze2/cc1-15695-a0de-v18` succeeds and `FETCH_HEAD` == `4a9efe08…`.
2. `git rev-parse FETCH_HEAD^{tree}` == `a0de248f1918aff315145392e2057db6ecc66c1a`.
3. `git rev-parse FETCH_HEAD:src/v2/test/claim/namespace_xl0/return_tail_position.dag` == the registered blob
   `f2009028bba066c829d555492b7a7d644083e86f`, and the ce-side blob `ee1c679e7ef1e2923edadfe0a6f0c63cf881b019`
   is present as registered.
4. The 21 `fn ` test names in that file, in roster order, equal the 21 registered rtp claim names.
5. Only then build. Anything else: refuse, exit 2, attempt not spent on a wrong tree.

## 4. Automatic effects of the push — derived from the triggers at main, not assumed

Measured at `main` = `8653d24f4998…`, all eight workflows under `.github/workflows/`:

| workflow | `on:` | reached by pushing `refs/freeze2/…`? |
|---|---|---|
| `branch-run-grant-broker.yml` | `workflow_run` on `fleet-converge`, `types: [requested]` | no — needs a fleet-converge run |
| `drift-watch.yml` | `schedule` (cron 17 5 * * *) + `workflow_dispatch` | no |
| `fleet-converge.yml` | `workflow_dispatch` | no |
| `heal-publish.yml` | `workflow_run` on `heal`, `types: [completed]` | no — needs a heal run first |
| `heal.yml` | `pull_request` → `branches: [main]`, `types: [opened, synchronize, reopened]`, with a `paths:` filter that includes `src/v2/**` | **not by the push** — but YES if a PR is opened, because both patches touch only `src/v2/**` |
| `instrument-dispatch.yml` | `workflow_dispatch` | no |
| `microvm-shakedown.yml` | `workflow_dispatch` (self-hosted, input-named runner) | no |
| `witnesses.yml` | `workflow_dispatch` | no |

**No workflow in the repository has an `on: push` trigger.** So a push of that ref matches no workflow filter
directly; the entire risk is the pull request, not the push:

* **A PR is likely to be opened.** The dashboard is reported (by the manager, `tidy-ant-251`) to open a PR for any
  pushed branch, and to do it late. That behaviour is NOT derivable from these files — it is outside the repository
  — so it is stated here as an unverified dependency, and it must be confirmed before a push is granted. If a PR is
  opened, `heal.yml` runs (its `paths:` filter matches `src/v2/**`), and `heal-publish.yml` follows on its
  completion. Whether a `refs/freeze2/…` (not a branch) satisfies the dashboard's "pushed branch" condition is
  exactly the question to put to the operator, and it is why the ref is proposed outside `refs/heads/`.
* **Closing a PR is not authorized either**, and neither is any other write-back: the proposal asks for one ref and
  nothing else.
* A tag ref (`refs/tags/…`) would likewise match no workflow filter (there is no `on: push` at all), so from the
  repository's own triggers it is equivalent; its only difference is the dashboard's branch test, which again cannot
  be measured here.

## 5. What is NOT claimed

Tree and blob equality verify **content** only. They do not establish that the pushed commit is the same *commit
object* that was frozen (author, committer, dates, message), nor anything about execution — only the run itself
establishes that. The raw commit objects in §1 exist precisely so a reviewer can check the identity offline.

## Review scope

I reviewed the pushed branch head **`713212e`** on `session/still-dove-186`. The two uncommitted witness-join edits are not visible from GitHub, so the findings below are against the pushed state.

At the time of review, the branch was **21 commits behind current `main`**, and #13731 remained open.

## Verdict

**Equivalent verdict: REQUEST CHANGES on the current pushed head.**

The overall cutover remains coherent, and **it should stay one PR**. I would not split it, restart it, or turn the remaining work into follow-up PRs. The branch has successfully avoided the original failure mode: C0–C4 are being assembled as one replacement rather than landed as intermediate repository states.

However, the work is not yet merely “finish the join, regenerate, and run lanes.” I found:

1. one serious demand-acquisition correctness defect;
2. one semantic hole in the shrink-only parse-refusal model;
3. one smaller but central non-injective module-name/storage mapping;
4. two closeout claims that need exact evidence before C2/C3 can be called complete.

All of these belong in this PR because they are defects in the new authority, not unrelated cleanup.

---

# Blocking findings

## 1. Qualified-reference acquisition can read an unrelated module for a lexical field projection

This is the largest defect.

`reference_site_collector` deliberately emits a dotted body expression such as `x.y` as one qualified reference site. That is correct for the mention census. But `module_acquisition` then converts every multi-segment reference site into module candidates without first deciding whether the head is lexically bound. fileciteturn62file0L2-L2 fileciteturn61file0L2-L2

For example:

```dag
fn read_name(x: User) -> String:
  x.name
```

If a module named `x` happens to exist, acquisition can:

1. see `x.name`;
2. probe `x.name`;
3. fall back to module prefix `x`;
4. read and parse `x.dag`;
5. recursively acquire its dependencies.

Only later can resolution establish that `x` is a parameter and `x.name` is a projection.

That has two bad consequences:

- the read receipt contains a module that the program did not depend on;
- a malformed, unreadable, or dependency-refused `x.dag` can break compilation of the unrelated lexical projection.

Adding or removing a same-named module can therefore change the compile verdict of a program whose meaning did not change. The existing acquisition exactness check cannot detect this, because acquisition itself minted `x` as a requested candidate.

### Root cause

A **reference-site census** is being used as a **module-dependency census**. Those are not the same concept.

The collector should remain a producer of mentions. Acquisition needs an explicit decision between:

```text
LexicalProjection
LocalDeclarationProjection
ImportedBinding
CandidateModuleReference
Unclassifiable / Refused
```

Only `CandidateModuleReference` may probe storage.

This does not require full external resolution before acquisition. It requires the local fact that can be established from the current module alone:

- whether the head is bound in an enclosing frame;
- whether it is a local/root declaration;
- whether an import binding already explains it;
- otherwise, whether it is eligible for module probing.

Do not fix this with another spelling heuristic. Share the resolver’s local-head/scope classification, or introduce one producer consumed by both acquisition and resolution.

### Required controls

At minimum:

```dag
fn local_projection_does_not_read_same_named_module() -> Bool
fn malformed_same_named_module_does_not_change_projection_verdict() -> Bool
fn qualified_module_reference_does_read_named_module() -> Bool
fn adding_same_named_module_does_not_change_lexical_projection_receipt() -> Bool
```

The strongest fixture is:

- entry module contains parameter/local `x` and evaluates `x.y`;
- a valid control has no `x` module;
- a second tree adds malformed or unreadable `x.dag`;
- compile verdict and read receipt must be identical.

Until that holds, C2’s “reads only the modules it requests” is true only relative to an over-broad request producer.

---

## 2. `parse-population` can return `held: true` while declared refusal rows were never encountered

The population parser correctly handles two directions for files it encounters:

- an exact declared cause increments `declared`;
- a declared file that now parses becomes a refusal;
- an undeclared or differently caused refusal becomes a refusal.

But `population_parse_report` never compares `tally.declared` with `declared_total`. When there are no other refusal lines and at least one module parsed, it returns `held: true` even if some declared refusal files were deleted, renamed, excluded from enumeration, or otherwise never visited. The missing rows are merely printed as `D of K` in the success text. fileciteturn65file0L2-L2

The stage0 native lane compensates by parsing that human-readable suffix and rejecting `met != declared`. fileciteturn66file0L2-L2

That means the actual shrink-only invariant currently lives in:

```text
Rust stdout parser over a nominally successful Dag verdict
```

rather than in the Dag verdict that owns the concept.

This is especially important here because the cutover is supposed to replace accidental side effects with explicit checks. A `PopulationReport { held: true }` must not require a host consumer to reinterpret its prose as false.

### Required fix

The best shape is not just another count comparison. Initialize the fold with the declared refusal keys as a remaining set:

```text
PopulationParse {
    parsed,
    remaining_declared,
    refusals
}
```

Then:

- exact refusal removes its exact `(tree, path, cause)` declaration;
- listed file parsing adds a contradiction;
- wrong cause adds a contradiction;
- at completion, every remaining declaration becomes a located “declared refusal not encountered” line;
- duplicate declaration identities refuse while building the roster.

Then `held` follows entirely from the typed result.

Keep the Rust assertion as redundant integration evidence, but it must not be the implementation of the policy.

Required pure controls:

- exact declared refusal is consumed;
- declared file now parses;
- declared file is absent from enumeration;
- declared file fails for the wrong cause;
- duplicate declaration identity;
- empty population remains red.

---

## 3. The module-name-to-file function is not injective over the names it accepts

The storage authority accepts a raw `String`, rejects only the empty name and empty dot-separated segments, and then joins the segments using `/`. fileciteturn68file0 fileciteturn68file1

Consequently, under an elided `v2` root:

```text
v2.a/b
v2.a.b
```

both derive:

```text
a/b.dag
```

The first spelling has no empty dot segment, so it reaches the file derivation.

Normal source syntax probably cannot author the slash-bearing name, but the CLI entry remains a raw string when it is handed to `acquire_verified`; conversion to `QualifiedName` happens later, after acquisition. fileciteturn70file0L2-L2

Header agreement should prevent the alias from silently compiling as the other module, but that is downstream damage control. The storage authority still probes the same file for two accepted names, contradicting the intended one-to-one rule.

### Required fix

Carry a validated module name before storage projection:

```text
CliEmitClosure {
    entry: QualifiedName
    ...
}
```

and make the storage function consume `QualifiedName`, or a stricter `ModuleName`, rather than `String`.

The validation should refuse path syntax before any filesystem effect:

- `/`
- `\`
- empty segment
- `.` or `..`
- absolute-path forms
- any non-identifier segment admitted by the string splitter but not by Dag module syntax.

Add two structural controls:

```text
distinct accepted module names derive distinct relative paths
every derived relative path is confined beneath its bound source root
```

This is small, but it belongs in the storage-authority cutover rather than a later hardening PR.

---

# Closeout findings

## 4. C3 is operationally cut over, but its structural closure still needs a consumer census

The production CLI path is directionally correct:

```text
acquire_verified
  -> AcquisitionVerified
  -> acquisition_ingest
  -> v2_cli_run
```

and the population verbs explicitly bypass closure ingestion. fileciteturn70file0L2-L2

However, the old-shaped lower interface remains:

```dag
fn v2_cli_run(plan: CliPlan, ingest: SourceRootIngest) -> CliRunOutcome
```

and `source_authority.dag` still exposes several `SourceRootIngest`-accepting functions and old commentary describing it as the cross-tree ingest authority. fileciteturn68file0 fileciteturn68file2 fileciteturn68file4

This is not automatically wrong. A verified acquired closure may reasonably be lowered into a pure in-memory source carrier before compilation. But C3 should not be declared complete merely because the intended CLI caller uses the verified door.

Before closeout, prove:

1. no production entrypoint constructs `SourceRootIngest` by walking a source root;
2. no production caller reaches `v2_cli_run` or an equivalent emitter with an independently assembled ingest;
3. every production ingest comes from a completed verified acquisition;
4. any remaining whole-ingest APIs are pure compiler/law fixtures, not alternate acquisition doors;
5. stale “the host walks the roots” and pre-cutover manifest prose is removed or recut.

Do not expand this into a redesign of every source-authority law. The bounded goal is a **no-live-bypass census** and deletion of stale claims. If a real production bypass is found, change that boundary to accept the verified/acquired carrier rather than relying on call-site convention.

---

## 5. The witness join is well-shaped, but its explanation tables must be exact in both directions

The pushed join handles the important unread case correctly:

- a governed file is still in the file population;
- marker-channel failure becomes an `UNREAD` observation;
- its old rows are not treated as absent or preserved;
- undeclared unread/lost/altered rows remain red. fileciteturn74file0L2-L2

That is substantially better than treating a failed parse as an empty marker set.

The pushed function accepts supplied `declared_removals` and `declared_unread` sets and uses them to classify findings. In the part I inspected, I did not see the reverse exactness join that rejects an explanation row nobody used. Your two local edits may be adding this, but they are not reviewable yet.

The final join should require:

- every old row is preserved, altered red, or attached to one exact declared removal mechanism;
- every declared removal names an old file that was actually removed;
- every declared unread row names an actual unread file;
- no removal/unread explanation is unused;
- no catch-all mechanism can turn an arbitrary `RowLost` into INFO;
- the mechanism/commit table is derived from or exactly joined to the findings, not merely printed beside them.

The 48/48 claims are useful unit coverage, but they are not the full-tree result. The full-tree join is the migration evidence.

### Do not overclaim independence of the native universe

The acquisition front end captures `test_markers`, and native universe discovery is then handed those captured marker names as supplied values. fileciteturn61file0L2-L2 fileciteturn73file0

Thus the universe and downstream execution can still agree after a shared marker-capture omission. Their agreement proves preservation after that common boundary; it does not independently prove that every authored marker reached the boundary.

For this cutover, the old-tree/new-tree witness identity join supplies the missing independent baseline. Keep that join red on unexplained loss or alteration. Permanent comments should not describe two fully independent source readings if both consume the same captured marker channel.

---

# What looks solid

Several important parts of the implementation are better than the original plan-level design:

- **The work remains atomic.** None of the report, sweep, acquisition, or exception-roster states has leaked independently to `main`.
- **The storage lookup distinguishes absence from refusal.** An unreadable candidate does not silently fall back to a shorter prefix.
- **Cross-tree admission is decided before the known/absent memo shortcuts.** A module acquired for an allowed caller is not automatically admitted for a disallowed caller. fileciteturn61file0L2-L2
- **The production CLI uses the verified acquisition door.**
- **Population parsing is explicit and streaming.** It retains verdicts rather than the whole source population.
- **Per-cause refusal rows are much stronger than a blanket exception count.**
- **The witness join keeps unread files observable instead of manufacturing an empty observation.**
- **Retiring `*_test.dag` in favor of the actual marker is coherent.** Test identity now follows the language fact rather than a storage nickname.
- **The raw diff size is not itself evidence of uncontrolled scope.** Most of it is the unavoidable one-sweep layout migration.

I see no reason to abandon or split the branch.

---

# Change the remaining execution order

The current ordering says:

1. regenerate;
2. integrate `main`;
3. run lanes.

That should be reversed around regeneration.

## Recommended final sequence

### 1. Finish semantic changes first

On the current branch:

- fix lexical-projection over-acquisition;
- close the parse-refusal roster inside the Dag verdict;
- validate/type module names before storage projection;
- finish and push the witness-join explanation exactness;
- complete the orphan/no-bypass census;
- run the focused controls for those changes.

Do not regenerate stage0 or generated docs yet except for disposable local debugging.

### 2. Integrate current `main`

This is now a substantive migration operation. The branch is 21 commits behind, and `main` overlaps with:

- reference-site collection;
- compiler stages;
- witness files;
- CI rosters;
- generated workflows and documents;
- stage0 mirrors.

Because the sweep contains approximately 3,333 renames—already beyond ordinary rename detection—do not rely on a conflict-free Git merge as proof that main-side changes survived.

Use the generated old-path → new-path map to audit integration:

1. enumerate every file changed on `main` since the merge base;
2. for each old path in the move map, verify the main-side patch exists at the mapped destination;
3. classify every new `.dag` file from `main` under the layout rule and move/rename it in the merge result if necessary;
4. reject any deleted-old/added-new pair that the migration map does not explain;
5. rerun the bare-reference and binding repairs against the merged tree.

Raising `merge.renameLimit` and `diff.renameLimit` will improve Git’s assistance, but the generated map—not rename detection—should be the authority.

### 3. Rerun the semantic evidence after the merge

The pre-merge results no longer establish the final tree.

Rerun:

- old-versus-new semantic closure differential;
- layout check;
- parse-population exact refusal join;
- witness identity join;
- acquisition exactness and projection noninterference;
- orphan/dead-authority analysis;
- native-universe selection controls;
- no-old-path/no-stale-name census.

The “ten C4 checks” should have one final-head aggregate receipt or command. They should not remain a list someone must remember manually.

### 4. Regenerate only from the integrated final tip

Then regenerate:

- stage0 mirrors;
- `witnesses.yml`;
- `fleet-converge.yml`;
- rung-drop output;
- `ROADMAP.md`;
- generated DESIGN sections.

Run every generator a second time and require **zero diff**. That second pass is the idempotence receipt.

### 5. Run both required remote lanes

Only the post-integration, post-regeneration results count.

A red lane may require a fix, but it should not create another phase or PR. Fix it on this branch, regenerate affected artifacts, and rerun.

### 6. Open the one PR and review by strata

Keep the commit structure reviewable. GitHub’s aggregate diff will not reliably present 3,333 renames, so the PR description should direct reviewers to:

1. authority and demand-loader commits;
2. generated move-map/sweep commit range;
3. binding repairs;
4. deletion of old acquisition producers;
5. C4 evidence and generated outputs.

Do not squash the whole branch into one opaque 5,838-file commit before review.

---

# Treatment of #13731

Do not wait indefinitely for #13731.

- If it lands before the final `main` integration, absorb it and delete the entire now-obsolete `requires-R` drop class.
- If it has not landed, retain the exact class the current tree requires.
- Do not pre-delete the class based on an expected merge.
- Any subsequent main integration invalidates the prior final-lane result and requires the affected closeout evidence to rerun.

---

# Final readiness bar

I would consider the PR ready to open for final review when all of these are true:

| Area | Required state |
|---|---|
| Acquisition | Lexically bound `x.y` never probes module `x`; same-named module presence cannot change its receipt or verdict |
| Storage | Only validated module names reach path derivation; accepted names map injectively and remain beneath the bound root |
| Parse population | Unencountered declared refusal rows make the Dag report itself red |
| C3 deletion | No production whole-tree producer or raw-ingest bypass survives |
| Witness identity | No unexplained lost/altered/unread rows; explanation rows are also exact and non-stale |
| Main integration | Every main-side edit to a renamed file is observed at its mapped destination |
| Generated state | All regenerations are from the integrated tip; second generation produces zero diff |
| Native evidence | Both required remote lanes green on that exact tip |
| Repository state | No uncommitted closeout edits and no stale pre-cutover authority prose |

The cutover is still on a finishable path. The main correction is to stop treating the remaining work as mostly mechanical: the lexical-projection acquisition bug and the false-green parse roster are real semantic defects. Fix those on this branch, integrate `main` through the migration map, regenerate afterward, and land the whole replacement as the single PR originally intended.

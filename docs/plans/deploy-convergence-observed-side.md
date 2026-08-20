# Deploy convergence: the observed side, and why the obvious repair is a regression

**Status:** design record, 2026-08-20. Reached by two independent derivations plus an adversarial exchange across two sessions. Most of its value is in the DEAD ENDS: the observed-provider design below was fully specified, then refuted, and the next person to look at live_deploy's empty observed side will re-derive it unless this record stops them.

## The defect that starts it

gunbc.live_deploy.emit calls the grain-agnostic membership_reconcile in both directions, but only at its DEGENERATE POLES -- apply passes an empty observed list, retract passes an empty desired list. So live_deploy has no observed provider and cannot converge; it can only install-everything or remove-everything. That is why a redeploy has always been the only remedy for any drift, and why a missing toolchain precondition went unnoticed for the workflow's entire life: an apply-all pole never asks what is already true, so it has nowhere to notice a precondition either.

## REFUTED: supplying an observed provider does not close the drift loop

The obvious repair -- observe the host, feed the apply pole, stop being degenerate -- was designed in full and is WRONG. The member value is a function of its own key, so the diff cannot see staleness.

- **The vacuity:** DeploymentArtifactStep carries only kind and path. key_of returns the path. value_eq is whole-value structural equality. So for any matching KEY the only remaining field is kind, which the spec fixes per path -- value_eq cannot distinguish anything and MemberChanged is UNREACHABLE. The diff is a set-membership check, not a state comparison.
- **The consequence, and it is a regression:** every artifact path on srv1 already exists; only their CONTENT is stale. A non-degenerate apply therefore produces an EMPTY PLAN and deploys nothing, reporting converged over a stale host -- where today's add-all unconditionally rsyncs and actually deploys. The repair would have been the silently-converged failure, reached through the fix rather than through the thing being fixed.
- **Why the member cannot be taught:** DeploymentSpec carries WHERE (topology, stable addresses); CandidateRelease carries WHAT (revision, expected surface identity). That separation is correct. Making members candidate-sensitive is exactly what would make MemberChanged reachable -- which activates EffectReplace, which the emitter deliberately models as impossible and answers with a loud poison rather than guessing. Replacement behaviour would have to be defined for every member kind first. That is a substantial modelling project, not a step.
- **Receipt for the blindness:** tree_sync_was_a_host_singleton_note already records that the sync transport files are not paths in DeploymentArtifactStep, so a claim over declared members could not see them.

## THE CLOSER: bind tree-sync to revision standing

The drift question is answered by REVISIONS, not by member presence, and the authority already exists and already refuses correctly. fleet_desired_observe fleet_revision_standing decides the cell three ways; fleet_converge_cli converge_cli_receipt_join_revision MEETS it into the receipt verdict so a knob run that never observed the revision cannot render converged. What is missing is only that nothing ACTS on the drifted arm.

- The catch-up realization exists: tree-sync in gunbc.live_deploy.emit and gunbc.live_deploy.spec, preserved through the deploy-workflow root cut.
- Its safe form is a compare-and-swap, and the inputs are already in the verdict: RevisionDrifted carries BOTH desired and local, and the converge_cli carrier note states that is precisely why -- a catch-up needs the prior.
- It needs no member work, no observed provider, and no new decision model. srv1 is already an enrolled fleet-converge host at mode=apply, so it rides an existing step.

## STANDING: retract must never take a host observation

Ownership is a claim about HISTORY -- who wrote this. An observation returns PRESENT STATE. Two files of identical shape and different origin are indistinguishable to any present-state read, so a host-observed retract asks the observation for a fact it structurally does not contain, and every answer is a plausible fabrication with delete on the other end. The poles need ownership asymmetrically: apply never needs provenance because desired bounds what it writes; retract needs it for every member because Removed is irreversible. So the degenerate retract pole is not a limitation awaiting a fix -- it is correct, and spec-driven is its right permanent source. Release identity does not rescue it: that is provenance for the running PROCESS, not for bytes on disk, and an install manifest is ownership STORED, which the ownership authority deliberately refused.

## SHELVED, and NOT blocked by the refutation above

The census answers a question nothing in the tree answers today -- what unowned state is on srv1 -- and it never touches value_eq, MemberChanged, EffectReplace, or the member model. It was split from the diff observation precisely so neither could contaminate the other, and only the diff half died.

- **Two observations, never one.** A diff observation shaped as a per-desired-member query cannot construct an observed-only key, so Removed stays unreachable on apply STRUCTURALLY rather than by a filter the provider remembers to apply. A census observation enumerates host state, feeds nothing into the diff, and is the only thing that can see beyond the roster. The property that makes each safe is the other's disqualification.
- **Class grain, not instance grain.** Worktree registrations are created per dispatch, so an instance-keyed contract refuses every time an agent session starts -- a gate that fires on normal operation gets silenced within a day, manufacturing the escape hatch out of a contract.
- **n equals one is PROVISIONAL.** One observation cannot distinguish an instance from a singleton class. A single observed path licenses no refusal and is not baselined; it becomes a class when a second instance appears or when a typed disposition names it. Three transitions are recorded: provisional to class, class to wider class, class to absent.
- **The control gates the baseline, not merely accompanies it.** The baseline is a measurement of the current host, which DESIGN 5 names as not-an-oracle; the monotone-debt carve-out admits it only if the universe is independently discovered. So the census must first be shown to have seen the worktree-registration grain, and if it has not, NOTHING may be baselined.

## THE TREE-PUBLICATION CLOSER: what exists, what is missing, and the two traps

The deploy threads the admitted revision to the artifact that ANNOUNCES which release it is and not to the artifact that DETERMINES it. gunbc.live_deploy.emit emit_artifact_upsert takes a ReleaseRevisionBinding; its ServeBinary arm consumes it, and its GunbcSourceTree arm sources the tree from the ambient working directory and never reads it. Readiness cannot catch this: the process-reported revision and the expected revision both derive from the same admitted value, so they agree by construction whatever the tree contains -- the evidence layer verifies the announcement, and the announcement is not derived from the thing announced.

The consequence is a NON-CONVERGING LOOP rather than a false converged, and the distinction is operational. gunbc.fleet_desired_observe fleet_revision_standing reads the deployed tree back, and the .git leg whitelists HEAD, so a tree published from a wrong-revision checkout reports RevisionDrifted correctly and forever -- republishing an arbitrary tree each cycle while the gate visibly never converges, which is the state that invites someone to disable the gate.

**This is a ceiling finding, not a discovery.** gunbc.live_deploy.deployed_tree_scope states in its own header that the deployed source is the working tree and not the commit, and the candidate-admission machinery downstream exists precisely because of it. What that reasoning never considers is the checkout being at the WRONG COMMIT: it classifies paths, and every path in a wrong-revision checkout is an ordinary tracked path. The mitigation is complete for the class it was designed against and structurally blind to this one.

## The closer's dependency list, after both halves were measured

- **EXISTS, do not rebuild.** deployed_tree_excluded_prefixes is already the single row, already deriving BOTH deployed_tree_rsync_exclude_flags and path_is_deployed, already witnessed segment-wise. path_is_deployed IS the projection membership function an identity computation needs.
- **IDENTITY IS OVER THE PROJECTION, never the tree.** The deployed bytes are an rsync projection, so a digest of the source tree and a digest of the deployed directory are unequal BY CONSTRUCTION and every run reports drift forever -- which fails safe but is indistinguishable from real drift, so it is silenced. The exclude set therefore stops being a copy filter and becomes part of the definition of sameness, and a change to it is a typed mass re-baseline. The identity function must DERIVE the set from the same authority the rsync argv derives from, never take it as a caller-supplied parameter, or the caller-property class is rebuilt inside the fix meant to dissolve it.
- **The .git leg stays OUT of the byte-identity subject.** It is answering a different question: it exists so the deployed tree is self-describing. Its claim is that HEAD names the admitted revision and that revision is REACHABLE in the deployed .git -- one-directional, decidable, and immune to the additivity that makes equality impossible. Reachability is the claim one would want even with --delete, so this is not a concession.
- **MISSING: an on-disk tree digest.** The existing readback (live_deploy.apply) is keyed on the listen PORT and reads the served surface over HTTP -- the right instrument for readiness and the wrong subject for tree identity, because a served-surface digest cannot distinguish a correct tree from a process rendering a tree that is no longer there.
- **MISSING: a remote revision observation.** git.Core.RevParseInRepo declares transport shell with argv git -C REPO rev-parse TARGET and takes no host, so fleet_revision_standing can answer about exactly one host: the one the process runs on. That is a ceiling on the observation, not a caller property that happens to hold.

## Why the remote observation is an ARGV lift and not a second operation

Multi-transport binding is real and in use in extdeps, so the §3 prescription -- bind another handler to the existing shape rather than fork the operation -- is the right instinct. But there is no ssh transport KIND anywhere in the corpus (240 shell, 72 rest, 9 handler, 6 file). Remoteness here is an OPERATION, ssh.Session.ExecPortableWords, reached through a FleetSshExecutionContext carrying the credential, the attempt-owned known-hosts anchor and the portable-word allowlist. That is a context threaded, not a handler bound, and teaching the transport layer to carry host-key verification would rebuild exactly the failure ExecPortableWords was built to correct.

So the forked thing is one level below the operation: it is the ARGV, hardwired inside a transport literal and therefore unreachable to anything that is not that transport. Lift the words to their own authority -- the same move deployed_tree_excluded_prefixes already made -- and the shell transport renders them locally while shape_fleet_ssh_exec carries the same words remotely. One authority, two consumers, no RevParseInRepoRemote nickname, no new transport kind, no caller choosing between two spellings of one concept. Minting the -Remote variant is the forbidden move and the obvious one.

## THE PAIR MUST SHARE ONE EXECUTION CONTEXT, decided before either is built

The two remote observations -- tree digest and revision -- are built separately and joined into one verdict. If each takes its own host parameter, the join can read the revision off host A and the digest off host B and report a convergence about neither, and it would be perfectly well typed: two host-parameterized observations, one join, no refusal available. That is the same fusion this record is about, relocated from inside an operation to the plan layer.

The construction form is that neither observation takes a host. Both are taken THROUGH one FleetSshExecutionContext, and the join consumes observations carrying that context's identity, so a cross-host join has no constructor rather than being refused by a check. This is cheap now, while both are being written and can share the carrier, and expensive later, because retrofitting means changing both signatures and every call site. The caller sentence for the version one would otherwise write -- this verdict is about one host provided both observations were given the same host argument -- is non-empty and unenforced, which is the whole test.

This row is the first instance of the class recorded as a PREDICTION rather than as a post-mortem: the check ran ahead of the defect instead of behind it. That is the transferable result of the thread, more than any single site it found.

## The executable form of the whole class

Every reconcile binding must have a control that holds the key fixed, varies meaningful state, and expects MemberChanged. If that control cannot be written, the comparison is vacuous and the binding is a set-membership check wearing a reconcile's clothes. The projection trap is the same test approached from the other end: if identity is computed over the raw tree, the CHANGED arm is the only reachable arm, and a comparison that can never return equal is exactly as uninformative as one that can never return different. Both fail the one test.

## The class this record is really about

A value that was true BY CONSTRUCTION under a degenerate input stops being true when the input becomes real, while the type still asserts it. Seven instances surfaced in one afternoon on this one subject: ownership always Present; Owned meaning ours; the belt revision gate agreeing with itself because both sides pointed at the same hand-set ref; Removed unreachable because the provider filters; residue empty because a per-member query has no walk; a class boundary derived from a common prefix collapsing to instance grain at n equals one; and value_eq meaningful. Every one is a predicate trivially satisfied because one side of a comparison was empty, identical, or determined by the other.

**The check, which is cheap and which caught the last three:** state in ONE SENTENCE what must remain true about your CALLERS for your fix to hold. If that sentence is non-empty and unwritten, it is the next instance. This is not review-your-own-work: each instance was invisible to its author and obvious to the reader, in both directions, because the thing that makes a fix feel finished is precisely the assumption just made about how it will be called. Faces four, five and six each arrived INSIDE the fix for the previous one.

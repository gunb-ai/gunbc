# gunbc — from a bare clone to one landed change

Generated from `gunbc.contributor_onboarding_path`. Do not hand-edit — the generated-artifact phase adjudicates this file against that authority. Every command below is derived from the authority that owns it, so this page moves when the build, the lane partition or the source roots move.

This page is an index of a path, not the proof that it works. What establishes the path is walking it: clone, build, run the lanes, make one change, get it reviewed, land it. If a step here does not work, that is a defect in the path and worth reporting as one — the obstacle roster below exists because each of its rows was hit rather than imagined.

`DESIGN.md` is the authority for every judgement about a change and is not a first-day document. The reading order at the end of this page says what you need before your first build, what you need before your first commit, and what can wait.

**TWO RULES THAT APPLY TO EVERY STEP BELOW, because two strangers walking this page lost their build to them.** FIRST, build into the repository's own `target/` and keep everything there: later steps resolve binaries at repo-root/`target/release` by construction, so a build redirected by `CARGO_TARGET_DIR` succeeds and then leaves those steps with nothing to run. SECOND, address every binary by path from the repository root — `./target/release/<name>` — and never by bare name, because a bare name resolves against the shell path, which in some environments carries a wrapper or an older build rather than the one you just made. Where a command below is reproduced verbatim from the authority that also prints it in a refusal, it names its program bare; supply the path yourself.

---

## The path

Ten steps in order. Run each verification before moving on: a step that half-worked is the expensive kind, because the failure surfaces several steps later attached to the wrong cause.

### 1. clone

Obtain the repository.

`git clone https://github.com/gunb-ai/gunbc.git`

Who is needed: self-serve

Done when: DESIGN.md is present at the root, and README.md and CLAUDE.md are symlinks to it.

### 2. toolchain

Get the pinned Rust toolchain, with clippy and rustfmt.

`rustup show`

Who is needed: self-serve

Done when: The active toolchain is the channel rust-toolchain.toml pins, reported as the directory override rather than the default.

### 3. build

Build the two binaries every later step needs.

`cargo build --release -p v1-compiler --bin claim_executor --bin gunbc`

Who is needed: self-serve

Done when: BUILD INTO THE REPOSITORY'S OWN target/ DIRECTORY, NOT WHEREVER CARGO_TARGET_DIR POINTS. Later steps resolve binaries at repo-root/target/release by construction -- gunbc.instruments.host_prelude witness_bin_release_path joins the repo root to that exact path -- so a build redirected elsewhere succeeds and then step 5 cannot find what it needs. If this environment sets CARGO_TARGET_DIR, override it back to the in-tree target for this build. THEN CHECK THE ARTIFACT, NOT THE EXIT CODE: list both binaries at target/release and run each one. A cold build of this pair takes minutes, so a return in seconds means no compiling happened here. If the build printed anything about dispatching or uploading artifacts, it compiled on another machine and you have nothing local -- read the build-obstacle rows below, which name the environment variables that force it local.

### 4. seed-hooks

Point git at the generated hooks, which is the one documented manual seed per clone.

`git config core.hooksPath .githooks`

Who is needed: self-serve

Done when: git config core.hooksPath reads back .githooks. THAT IS THE WHOLE CHECK AT THIS POINT, and the reason to say so is that the thing the hooks converge -- merge.generated-artifact.driver -- stays UNSET until the generated pre-commit hook first runs, so a reader who checks for the driver here finds nothing and cannot tell a broken seed from an unused one. Check the driver after your first commit, at step 7, not now.

### 5. run-witnesses

Run the corpus lane the merge gate runs: the parse sweep and the witness floor.

`target/release/claim_executor --required-ci --source-root dag --source-root src/v2 --required-lane witnesses`

Who is needed: needs a machine: a host whose cgroup exposes an enforceable memory bound large enough for the floor fold

Done when: The run announces one phase line per phase it owns and one routed line per phase it does not, and exits nonzero on any failed phase. QUALIFY THE HOST BEFORE SPENDING THE TIME: this is the longest step by a wide margin and the most memory-hungry, so confirm the process has an enforceable cgroup memory bound at or above the declared per-slot allowance in gunbc.runner_slot_allocation first -- a host under it will run for a long time and then refuse or be killed. When it does refuse, classify the refusal with the three arms in the obstacle rows below before touching the corpus.

### 6. run-build-lane

Run the emission lane: the regen comparison and the emission compile.

`target/release/claim_executor --required-ci --source-root dag --source-root src/v2 --required-lane build`

Who is needed: self-serve

Done when: The generated-artifact phase reports each population it owns, and names the path plus the regeneration recipe when a projection has drifted.

### 7. change

Make one small change in the .dag authority that owns the fact, then regenerate what projects from it. A NAMED FIRST CHANGE, because a step that says only which KIND of change to make is a step that sends you exploring: edit one why string in gunbc.contributor_onboarding_path onboarding_design_reading -- a row of the reading order on this very page -- and regenerate. It is the smallest projection in the repository, so the loop closes in one run, and the file you are reading is the diff.

`gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/instruments/docs_projection_gate.dag --function regen`

Who is needed: self-serve

Done when: RUN IT AS ./target/release/gunbc, NOT AS THE BARE gunbc PRINTED ABOVE -- that substitution is the second of the two rules at the top of this page and it is the one readers miss, because the recipe looks copy-pasteable. Then the build lane above goes from refusing the drift to exiting zero, with the regenerated bytes in the diff. THE LEADING gunbc IN THIS RECIPE MUST BE THE BINARY STEP 3 BUILT, ADDRESSED BY PATH. The command is reproduced verbatim from the authority that the drift refusal itself prints, so it names the program bare -- and a bare name resolves against the shell path, which is the stale-binary trap two rows below. Steps 5 and 6 are path-addressed because their program has a location authority; this one has none, so you supply the path.

### 8. local-checks

Run the checks that block a merge and the ones that are local diligence only.

`cargo clippy --all-targets -- -D warnings`

Who is needed: self-serve

Done when: Clippy over every target is clean; a warning is an error.

### 9. propose

Push a branch and open a pull request.

Who is needed: needs a credential: push access to the repository, or a fork plus the ability to open a pull request from it

Done when: The pull request exists and the required contexts are queued against its head.

### 10. land

Get the change reviewed and merged.

`git merge-tree --write-tree origin/main HEAD`

Who is needed: needs an operator decision: who reviews and who merges; the merge itself is performed by the operator under the standing squash-merge policy

Done when: A review has been recorded, the required contexts are green on the current head, and a local merge-tree of the head against the base reports no conflict.

## What bites, and where

Each row is attached to the step it breaks. The last sentence of each row says whether the error text explains itself, because that is the difference between a newcomer losing minutes and a newcomer needing someone who already knows.

- **clone — The repository root offers DESIGN.md, and README.md and CLAUDE.md are symlinks to the same file. There is no CONTRIBUTING file and no first-day document.** DESIGN.md is the single authority and is a serially-reasoned argument from axioms, loaded in full every session. Measured on the tree: it names no build command, no toolchain installer and no binary path, so the document a newcomer is pointed at cannot get them to a build. REMEDIED BY DERIVATION. docs/onboarding.md is generated from this module, carries the ordered path with commands derived from their authorities, and is linked from the section of DESIGN.md that names the checks. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **toolchain — A build under a distribution-packaged rustc fails in ways that have nothing to do with the toolchain pin.** rust-toolchain.toml is honoured by rustup and ignored by a bare rustc, and nothing in the repository tells a reader that rustup is the assumed installer. The pin is the sole in-repo channel authority, so a reader who never installs rustup silently builds against a different compiler. DOCUMENTED REMEDY. Install rustup first; it reads the pin as a directory override with no further action. Confirm the override is active before building rather than after. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **build — There is no obvious package to build. The binary is named gunbc, the crate directory is a stage0 path, and the package name matches neither.** The package is the v1 seed compiler and the bins are its targets; the naming is a fact about the bootstrap layout rather than about the product. A newcomer cannot derive the invocation from the directory tree. REMEDIED BY DERIVATION. The build step above consumes gunbc.repo_self_build, the single point through which this repository builds its own binaries, so the projected command is the same argv the required lane uses and moves when that authority moves. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **build — A build reports success and the binary is absent from the target path the repository names -- or, worse, PRESENT AND STALE. A cold-start walk found the override directory holding binaries from an earlier run, timestamped forty minutes before the build that had just reported success.** A cargo target directory override redirects artifacts away from the in-tree target path, and the release-path authority names the default location. The two disagree silently, so a successful build and an absent binary are simultaneously true. THE STALE CASE IS THE DANGEROUS ONE: a reader told to list the binaries under the target directory in effect FINDS THEM, and concludes the build worked -- so the instruction to check the artifact rather than the exit code fails too, unless the check also asks whether THIS build wrote them. Compare the file timestamp against when the build ran. DOCUMENTED REMEDY. Check the cargo target directory in effect before concluding a build produced nothing -- but do not simply go and use whatever it points at, because the witnesses in later steps read repo-root/target/release and nothing else. POINT THE BUILD BACK AT THE IN-TREE target/ AND KEEP IT THERE. A stranger who followed an earlier revision of this page to the override directory built successfully and then watched step 5 die at exit 127 on a binary it could not find, which is this page having sent them somewhere its own later steps cannot follow. The neighbouring wall, measured on the same walk: forcing the build local can fail with a permission error naming the override directory, because that path belongs to the environment rather than to you -- which is a second reason the in-tree target is the right destination. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **build — An entry that compiled yesterday refuses today with unresolved-import errors naming standard-library modules the reader never touched.** A previously-built or previously-installed gunbc on the path predates a language capability the corpus now uses, dies inside the standard library before registering the module, and reports the failure as a defect in whatever module the reader is editing. An old local binary silently accepts and rejects a different language than the merge path does, which is how a break reached main. DOCUMENTED REMEDY. Run the binary this clone built, addressed by path, never one resolved from the shell path; rebuild it after pulling. When a diagnostic names a module you did not touch, suspect the compiler before the corpus. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **run-witnesses — The run refuses before doing any work, reporting that the host memory budget is unreadable.** No cgroup memory.high or memory.max binds the process, so the planning allowance is unknown and the run refuses rather than admitting against the widest cap it could have chosen. The authority is explicit that this is not a state an environment integer can rescue: the planning-request variable may only narrow an observed bound, never supply a missing one, and treating it as a rescue converts a typed refusal into an unbounded resolve on an unbounded host. DOCUMENTED REMEDY. There are two environment variables here and only one of them is an answer. GUNBC_MEMORY_BUDGET_BYTES is a PLANNING REQUEST: gunbc.host_budget_source states that it may only narrow an observed bound and can never supply a missing one, so reaching for it to clear this refusal is the escape hatch the refusal exists to refuse -- and the run it admits is SIGKILLed rather than refused, which is strictly worse than the refusal it replaced. GUNBC_BIND_MEMORY_CGROUP_BYTES is the one that binds: v1_compiler.memory_governor apply_memory_cgroup_bind writes an actual memory.max for the process, announces which of its three continuing states it took, and REFUSES rather than proceeding when the controller is unavailable, the tree is unwritable or the request does not fit the machine. So request a real bound, and read the memory-cgroup-bind line the run prints to confirm which state you got: not requested, already bound, or applying. The error text points at the remedy.
- **run-witnesses — The floor fold goes red on a clean checkout, and going red again on a re-run looks like an unstable test suite.** The required floor runs close under the memory line it is admitted against, so an environmental red is an ordinary outcome of the run rather than a statement about the diff. THE GROUNDING IS AN AUTHORITY RATHER THAN HEARSAY, and a reader can check it: gunbc.runner_slot_allocation declares the per-slot ceiling and the throttle line beneath it, and gunbc_runner_slot_memory_max_ruling_note records that the ceiling was RAISED because the floor's measured peak no longer fit under the previous throttle line. That note also carries the reading that matters when you see a peak equal to the throttle line -- it is a censored lower bound, the workload being held rather than measured -- so a run that looks like it fitted exactly did not. DOCUMENTED REMEDY. Read the refusal and classify it before acting, WITH THREE ARMS RATHER THAN TWO: an environmental refusal names the budget; a MISSING BINARY refusal names a claim but was caused by an artifact that was never built; and only what is left is a corpus failure. The two-arm form of this rule was measured wrong on the cold-start walk -- it routed a never-built binary into the corpus arm and sent the reader hunting the corpus for a build problem. Never re-run a corpus or cost refusal to sample a green: that is sampling until the instrument agrees with you, and it destroys the only signal the deficit produces. A TOOLCHAIN CRASH IS THE ONE EXCEPTION and has its own row below. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **change — A hand edit to a generated file is reverted by the next run, or the merge gate refuses a file the author never opened.** Several committed documents are projections of .dag authorities, including DESIGN.md itself, and the generated-artifact phase adjudicates them against those authorities. Editing the projection edits the copy. REMEDIED BY DERIVATION. The change step above names the regeneration recipe as its command, and the build-lane step names the phase whose refusal prints that recipe beside the drifted path, so the loop closes without the reader knowing which files are generated in advance. The error text points at the remedy.
- **build — The build command exits ZERO, prints no error, and produces no binary anywhere -- neither in the in-tree target path nor under the override. Measured on the cold-start walk at five minutes and one second, with a line reporting that zero artifacts were uploaded.** The cargo on the path was a WRAPPER, not cargo: it dispatched the build to a remote runner of a different architecture and returned that runner's success. So the compile genuinely succeeded and the artifact genuinely does not exist on this machine, which is why there is no error text to read. This is the worst available shape for a newcomer -- a zero exit with an empty diagnostic is indistinguishable from a working build until the next step cannot find its input. It is a property of the environment rather than of the command, which is why the page cannot fix it by changing the argv. DOCUMENTED REMEDY. THE OBVIOUS CONTROL FOR THIS DOES NOT WORK AND THE PAGE PREVIOUSLY PRESCRIBED IT, WHICH IS THE WORSE HALF OF THIS ROW. Running the build with a flag that must be rejected was measured TWICE, in two environments, on the same command, and gave OPPOSITE ANSWERS: one session saw the flag refused with a nonzero status, and a cold-start walk saw the remote compiler reject it while the local wrapper still exited 0. The divergence is a sharper finding than either result, because the control's whole job is to establish WHICH COMPILER YOU REACHED -- and a control that answers differently depending on where it runs cannot answer that question anywhere. Do not rely on it, and do not rely on a report that it fired; a wrapper that does not propagate its inner failure cannot be interrogated by exit code at all. USE THE ARTIFACT AS THE ORACLE INSTEAD: build, then list target/release and run what is there. To force the build onto this machine, set the build mode to local and point the target directory back in-tree; a heavily parallel local build may also need its job count capped. AND KNOW THAT THIS CLASS IS ALREADY MODELED, which is the most useful thing on this row: gunbc.instruments.host_prelude WitnessBinRefusalReason separates a build that failed from an artifact that is absent from one that is present but not executable, and its own note names the two incidents that priced those arms -- a build returning success while writing no artifact, and a wrapper silently routing to a remote builder. Those are the two failures a newcomer meets here. The repository knows this class; the hand-rolled build on this page is simply outside the carrier that handles it, so a reader who keeps hitting it should reach for that carrier rather than invent a third account of it. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **run-witnesses — After twenty-plus minutes the lane reports a failed phase and refusals that read as corpus claims. The proximate cause was printed about thirty screens earlier: a witness binary exited 127, no such file or directory.** Some witnesses invoke a separate binary, and the pair this step builds is the pair the required lane's own roster names -- not every declared bin. A binary outside that pair is absent, the invocation fails at the operating-system level, and the refusal surfaces far from the missing artifact and in the vocabulary of a claim. The distance between cause and report is the whole problem: the reader sees a claim and goes to the corpus. THE ASYMMETRY IS REAL AND IT IS NOT A HOLE, AND THE REASON IS WORTH KNOWING RATHER THAN RE-DISCOVERING. The floor lane really does build only the two-bin roster while the build lane builds every declared bin, and required floor witnesses really do invoke a bin outside that pair. What closes it is that the dependency is declared at the CONSUMER instead of in a central roster: gunbc.instruments.host_prelude witness_bin_ensure_built_typed refuses an unenrolled bin, probes the release path for an executable, RUNS cargo for that bin when it is absent rather than merely reporting it missing, and refuses with a typed reason when that build fails. So a roster grep cannot see the dependency, which is why it looks like a gap. That placement is the better shape under DESIGN section 3 -- one roster enumerating every bin any witness might someday need would be a second authority for a fact each witness already holds -- and every non-Ready arm still fails the gate, so the typing buys diagnosis rather than permission. THE CONSEQUENCE FOR THIS PAGE IS THE OPPOSITE OF REASSURING: the local route above hand-rolls its own build and therefore has NONE of that protection, while the lane it imitates has all of it. DOCUMENTED REMEDY. Search the output for a nonzero exit from a binary invocation before reading any refusal as a claim about the corpus. If one is there, build every declared bin target INTO THE IN-TREE target/ -- witnesses resolve binaries there and nowhere else -- and run the lane again. The command, rendered here rather than named so you do not have to open a .dag to get it: `cargo build --release -p v1-compiler --bins`. It is exactly the required build lane prelude. That re-run is not sampling for a green: the input changed. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **build — The compiler itself crashes on a dependency with an internal error, and the same command succeeds on the next attempt with nothing changed.** A transient toolchain crash rather than a fact about this repository. Observed once on the cold-start walk and gone on retry. DOCUMENTED REMEDY. Retry once. THIS IS THE ONE PLACE ON THIS PAGE WHERE RETRYING IS CORRECT, and the boundary is worth holding precisely because the witness rows say the opposite: a compiler crash is infrastructure and the retry costs nothing, while a witness that RAN TO COMPLETION and refused on cost or on a claim must never be re-run to sample a green. The discriminator is whether the instrument reached a verdict. If it crashed, retry; if it answered, read the answer. A second identical crash is no longer transient and is worth reporting rather than retrying a third time. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **change — A fold whose accumulator starts as an empty list compiles into a wall of errors saying a field does not exist on a type the author never wrote, pointing at lines far from the fold.** An empty list literal as a fold seed has no element type to infer from, so the accumulator resolves to the bare free monoid and every later field access on its elements is a field on THAT type. The diagnostics are therefore correct and land nowhere near the cause. Observed while writing this change's own witness: one seeded fold produced eleven field-does-not-exist errors across two functions, none of them on the fold. DOCUMENTED REMEDY. Give the accumulator an element type the seed can carry, or author the fixture as a typed data declaration instead of folding one out of a live roster. When a diagnostic says a field is missing on a type you did not name, read it as an inference failure upstream rather than as a mistake at the line it cites. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **change — A change that adds a generated artifact goes red in CI on a job that is not a required lane, reporting HealAuthorCommitRequired against a path under .github/workflows that the author never opened, with the cause ActionsJobCredentialScopeUnavailable.** The emitted workflow carries a staging line per committed generated artifact, derived from the registry, so registering an artifact CHANGES the workflow. The repairing job would ordinarily regenerate and push that itself, and for workflow files it cannot: the credential an Actions job runs under is not permitted to modify workflows. So it refuses, names the path, and requires the author to carry the emission. Observed on the change that introduced this very page -- one staging line for docs/onboarding.md -- so the mechanism is this row's own receipt rather than a report. DOCUMENTED REMEDY. Run the whole-registry regenerator the refusal names, then commit the workflow it rewrites. THE CAUSE STRING DOES NOT SAY THIS AND IT IS THE WHOLE POINT: a credential-scope refusal on a workflow path is not a defect in your change and not something a retry clears -- it is a step the repairing job is structurally unable to take for you. Confirm the regenerator rewrote ONLY paths you can explain: on this change it left every other generated artifact byte-identical, which is what distinguishes carrying your own emission from importing somebody else's drift. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **propose — A clone over an anonymous transport can fetch and build everything, and cannot push.** Read access is anonymous and write access is not. Nothing in the path up to this point requires an account, so the first credential requirement appears at the moment the work is finished. OPERATOR GATED. whoever administers repository access; provisioning is a separate item and is deliberately not specified here The error text points at the remedy.
- **land — The forge reports the pull request as mergeable while a real conflict against the base exists.** The forge's mergeability field has been observed reporting clean over conflicting trees, so it is not an oracle for whether a merge applies. Reported by a sibling lane in this subtree and carried here because the step it breaks is the last one. DOCUMENTED REMEDY. Establish mergeability locally by writing a merge tree for the head against the base and reading its conflict report, rather than from the forge's summary field. THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.
- **land — The change is reviewed, green and mergeable, and nothing happens.** A review is advisory: it establishes that no blocking defect was found, never that the change is what was asked for. The merge itself is performed under a standing policy by the operator, so the last step of the path is the one step that structurally is not self-serve. OPERATOR GATED. the operator, who performs the merge; this is the single irreducible operator dependency remaining in the path and it is a policy choice rather than a gap THE ERROR TEXT DOES NOT POINT AT THE REMEDY, which is why this row exists.

## Where the path still needs somebody else

Stated as a census rather than as reassurance. These are the steps and obstacles a document cannot discharge, and they are the standing bus-factor risks in this path.

- Step 5, run-witnesses: needs a machine: a host whose cgroup exposes an enforceable memory bound large enough for the floor fold
- Step 9, propose: needs a credential: push access to the repository, or a fork plus the ability to open a pull request from it
- Step 10, land: needs an operator decision: who reviews and who merges; the merge itself is performed by the operator under the standing squash-merge policy

Obstacles whose only honest disposition is that somebody else must act:

- propose: OPERATOR GATED. whoever administers repository access; provisioning is a separate item and is deliberately not specified here
- land: OPERATOR GATED. the operator, who performs the merge; this is the single irreducible operator dependency remaining in the path and it is a policy choice rather than a gap

## The other commands worth knowing

Derived from `gunbc.repo_self_build`, which is the single point through which this repository builds and checks itself.

1. Lint over every target, which a merge is blocked on: `cargo clippy --all-targets -- -D warnings`
2. The package's own unit tests, which are local diligence and are run by no required step: `cargo test --release -p v1-compiler --lib`
3. The source roots every entry is resolved against: `--source-root dag --source-root src/v2`

## Reading DESIGN.md in an order

`DESIGN.md` is reasoned serially from three axioms, and every section is a consequence of the ones before it. So it rewards being read in its own order and punishes being skimmed for the section that seems relevant. What follows is when you need each part, not a summary of any of them — a shorter second account of the axioms would be the exact duplication the document forbids.

- **Building and checks — first day.** It names the checks and the one manual per-clone seed, and it is the section this path is linked from. It is the only section a reader needs before a build.
- **Section 1, the objective — first day.** Three axioms and the three consequences every later section is derived from. Without it the rest of the document reads as a list of preferences rather than a chain, and a reader will argue with conclusions instead of premises.
- **Section 5, fail-closed — before the first commit.** It carries the hard reject: a silent widen, a fabricated default, an uncounted degradation or an escape hatch is refused regardless of what else the change delivers. It is the single most likely reason a first diff is rejected.
- **Sections 2 and 3, redundancy and single authority — before the first commit.** They decide where a fact is allowed to live. A first change usually either restates something that already has an owner or cites a position instead of a symbol, and both are refused on these two sections.
- **Sections 3b and 3c, conformance and consumption — before the first commit.** They are the two questions a reviewer asks of a diff: does it inhabit the models it touches, and who consumes what it adds. A well-shaped declaration nothing reads is refused under the second.
- **Section 4b, the guarantee ladder — before the first commit.** It fixes what a change may claim about its own safety, and it is where the vocabulary a reviewer will use comes from. A claim stated one rung above its executed evidence is the failure mode it names.
- **Section 4, the substrate, and section 4c, annotations — when it bites.** Needed at the first .dag edit rather than at the first read: the closed vocabulary the language is built from, and the rule that a comment is the only place uncarried prose may live.
- **Section 6, how to work — when it bites.** The authorship tells -- scaffold, workaround, for-now, a new artifact with no consumer. Read it when a change starts wanting one of those, which is the moment it is about to be refused.
- **Section 7, self-hosting, and the recurring failure modes — when it bites.** Why Rust is present at all and what the seed is shrinking toward, plus the ledger of classes already found. Deferrable on day one and indispensable once a change touches the compiler.

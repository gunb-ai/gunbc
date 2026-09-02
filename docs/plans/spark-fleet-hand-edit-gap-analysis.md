# Spark fleet: hand-edited state with no modeled authority

**Status 2026-09-02, corrected.** Every row below was established by reading the live
hosts and comparing against the module named as its authority. The instrument for the
desired side is `gunbc.spark.serving_unit_render spark_serving_desired_user_unit_text`;
for the live side, the unit file and `/api/ps` on each host.

## Correction, and it is the most important thing in this document

**The first version of this analysis was measured against the wrong tree, and its two
headline findings were false.** It was derived by running the renderer from a session
branch based on `373b8d11`, and `main` had since advanced. Re-derived against
`origin/main`:

- **Context is NOT diverged.** `spark_serving_desired_context_length` is `1048576` on
  main, matching the live hosts. The claimed `131072` was this branch's stale value; main's
  own annotation records that exact number as the thing it replaced.
- **`OLLAMA_NUM_PARALLEL` is NOT missing.** `extdeps.ollama.server_env` carries it as a
  typed axis, `serving_desired` carries a `PositiveSlotCount` of 4, the renderer emits
  `ollama_num_parallel_env_assignment`, and
  `test.claim.spark.spark_serving_unit_render_witness_test` holds the renderer to it.

So **the freeze recommendation that followed from those two rows is withdrawn.** On those
axes a convergence run would not degrade the fleet; it would agree with it.

The failure was mine and it is worth naming as its own class, because it is the exact
defect this repository spends its review budget on: I ran a real instrument, got a real
number, and read it as a fact about the fleet when it was a fact about **my branch**. A
measurement is only as current as the tree it was taken from, and a session branch is not
the authority. Re-derive against `origin/main` before reporting a divergence.

## Why this document exists

The fleet is being assembled by hand while the model that is supposed to own it is
elsewhere. That is a legitimate way to move, but it has a specific failure mode worth
stating up front, because it is already loaded and armed:

**A convergence run today would degrade the fleet, silently and without failing.** The
modeled desired unit differs from the live unit on both serving hosts in ways that all
point the same direction — narrower context, no concurrency, a different model. Nothing
in that path refuses; the unit rewrites, the service restarts, and it serves. The numbers
every serving decision is currently reasoned from would be invalidated without a single
error.

So this is not a tidiness ledger. It is the list of things that must land before
`gunbc.spark` convergence may be actuated against `srv5`/`srv6` again.

## The divergences

### 1. Context window — WITHDRAWN, see the correction above

Main desires `1048576`, which is what the hosts serve. No divergence.

### 2. Concurrency — WITHDRAWN, see the correction above

Main models the slot count as a typed `PositiveSlotCount` of 4, renders it, and witnesses
the rendering. No divergence.

### 3. Served model — CLOSED

`gunbc.spark.serving_desired` selected a gpt-oss build while both serving hosts ran
DeepSeek-V4, and the two are not variants of one choice — different publisher, family,
artifact and runtime footprint — so no evidence gathered against one answered for the
other.

`spark_serving_desired_manifest` now names the DeepSeek manifest, resolved through
`gunbc.ollama_model_resolution` with the provenance arm that records it as locally
materialized rather than upstream-published, and `gunbc.spark.serving_model_admission`
requires an exact runtime version AND a manifest-identity match before a local artifact is
admitted. The divergence INSIDE the model closed with it: the rendered `Description=` was a
second, stale spelling of the model name and is now derived from
`spark_serving_desired_model_ref` rather than authored beside it.

What this did NOT close, and the reason the ordering below still starts here: the desired
model is now correct, and reproducing that artifact from desired state remains impossible,
which is carried as a declared §4b(3) rung drop rather than as silence.

### 4. Residency: modeled and live both silent on `OLLAMA_KEEP_ALIVE`

Neither the desired unit nor the live unit sets it, so a 94 GB model is evicted after the
default idle period and the next request pays a full reload. Measured today: both `srv5`
and `srv6` reported `none resident` minutes after serving a request. Any latency
measurement that does not declare residency state is measuring two different things
depending on when it ran.

### 5. The second pair is entirely outside the model — AND IS NOT TWO SERVING HOSTS

This section originally described the pair as two independent ollama serving hosts. That
was the state when it was written and it is not the state now, and the difference is
load-bearing rather than a refresh: `192.168.1.232` runs a **llama-server front door** on
`:30000` whose weights live on `192.168.1.233` behind an `ggml-rpc-server` reached over the
RoCE fabric. They are ONE serving unit split across two machines, not two of anything, and
a roster that admits them as two serving cells would plan an 86.7 GB load onto the weights
peer — which has roughly 16 GiB free and swap disabled, so the load is an OOM kill that
takes the front door down with it.

`gunbc.spark.serving_unit_observed` now carries that subject with the split as a
constructor condition. The membership gap below is unchanged and is what remains open;
what changed is what membership would have to mean.

`spark-c2b1` (192.168.1.232) and `spark-ac79` (192.168.1.233) are provisioned, serving,
and unknown to every authority that should own them:

- no `SparkCellRoleAssignment` — `gunbc.spark.cell_role` knows only `srv5` and `srv6`
- no fleet slot, so `gunbc.network_identity_subsumption` cannot bind them
- their static DHCP reservations are refused by design by that same module, so they have
  no modeled network identity even in principle
- different service user (`briansrls`, not `gunbc-automation`)
- different install root (`/usr/local/ollama`, not the install-paths authority's tree)
- different unit name, and on `.232` not an ollama unit at all
- `OLLAMA_KEEP_ALIVE=-1` on the ollama that `.233` still runs alongside its weights role,
  which the modeled unit has no field for — and that ollama is itself the hazard in the
  paragraph above, because it WILL accept a load the weights peer cannot survive

The runtime itself is the one thing that IS aligned: the pinned `v0.32.9` arm64 asset was
installed after verifying its SHA-256 against `extdeps.ollama.binary_release`, so the
runtime materialization identity matches the first pair rather than drifting to `latest`.

### 6. Credentials are hand-carried

`gunbc.spark.credential_workflow` models a `spark-administrator-password` `SecretRef`
resolving through `gunbc.auth.secret_ref_credential`, and its own note says wet entry
points name attempt identity and host on argv and never plaintext passwords. The password
in use today was passed in chat and now exists in at least two session transcripts. SSH
keys were installed by hand on top of it.

The modeled path exists and was bypassed. That is the §6 out-of-band actuation tell in its
plainest form, and the credential should be rotated once the fleet is stable.

### 7. Runner configuration lives in the environment, and the identity model reads argv

`gunbc.model.choice` keys a serving realization on, among other things, a fixed runtime
mode derived from the runner's **argv**. Every runner on this fleet is launched as bare
`ollama serve` with its entire configuration in `Environment=` lines. So two runners
differing in context window and slot count — a material difference the selector exists to
notice — currently hash to the same mode and the same exact configuration.

The argv split is correct as far as it goes and is strictly better than the `{name,
version}` nickname it replaced. On this fleet it is discriminating almost nothing, and
widening the carrier to environment is the highest-value follow-up. It needs a ruling
first on which variables are configuration and which are ambient.

## What has to happen, in order

1. ~~**Decide the served model in the model.**~~ CLOSED — see §3. The desired row names the
   DeepSeek manifest and admission requires an identity match, not a name match.
2. **Model residency.** `OLLAMA_KEEP_ALIVE` appears nowhere on main, so a 94 GB model
   evicts on the default idle timer and every measurement silently depends on when it ran.
3. **Admit the second pair** — cell roles, host identities, and whatever network identity
   can be bound given that static reservations are refused by design. Admission must admit
   ONE serving unit, not two serving cells; see §5.
4. **Widen the realization carrier to environment.** The side chat has since ruled on
   the partition: consume a runtime-owned closed projection of causal configuration, not
   the whole process environment. `OLLAMA_CONTEXT_LENGTH` and `OLLAMA_NUM_PARALLEL` are
   configuration; `OLLAMA_HOST` is deployment identity; `OLLAMA_MODELS` is
   artifact-resolution provenance already subsumed once the artifact digest is joined.
   This is landing in #9897 rather than waiting.
5. **Route credentials through the modeled `SecretRef`** and rotate the current one.

## The standing risk this document is really about

After the correction, and after the served-model divergence closed, the remaining
divergences are narrower still but they are the same class: residency, the second pair,
and credentials are facts about the fleet that no authority owns. The danger is not that the model is wrong on
those axes — it is that it is SILENT on them, so nothing refuses when they drift.

The generalization worth keeping is the one the correction taught: a divergence report is
a measurement, and a measurement is only as current as the tree it was taken from. This
document was itself an instance of the failure it exists to catalogue.

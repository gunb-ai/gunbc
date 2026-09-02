# Spark fleet: hand-edited state with no modeled authority

**Status 2026-09-02.** Every row below was established by reading the live hosts and
comparing against the module named as its authority, not from memory. The instrument for
the desired side is `gunbc.spark.serving_unit_render spark_serving_desired_user_unit_text`;
for the live side, the unit file and `/api/ps` on each host.

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

### 1. Context window: modeled 131072, live 1048576

An 8x difference, and it is the load-bearing one. `gunbc.model.choice` reasons about a
400,000-token floor; the modeled desired state cannot serve it. Converging would take the
fleet below the floor the selector is being built to defend.

### 2. Concurrency: modeled ABSENT, live `OLLAMA_NUM_PARALLEL=4`

`OLLAMA_NUM_PARALLEL` does not appear anywhere in the `.dag` corpus. Not as a stale value
— as no value at all. The renderer emits three `Environment=` lines and this is not one of
them, so convergence does not overwrite the setting, it DELETES it, and the runner falls
back to whatever Ollama defaults to.

This is the setting that decides whether the fleet serves concurrent sessions or
serializes them, which the operator has named as mattering more than throughput.

### 3. Served model: modeled `gpt-oss`, live `hf.co/antirez/deepseek-v4-gguf`

`gunbc.spark.serving_desired` selects a gpt-oss build. Both serving hosts run DeepSeek-V4.
The two are not variants of one choice — different publisher, family, artifact and runtime
footprint — so no evidence gathered against one answers for the other.

Note also a divergence INSIDE the model: the rendered `Description=` says `gpt-oss:20b`
while the row's own annotation records the operator decision as `gpt-oss:120b`. The
description is a second, stale spelling of a fact the row already carries.

### 4. Residency: modeled and live both silent on `OLLAMA_KEEP_ALIVE`

Neither the desired unit nor the live unit sets it, so a 94 GB model is evicted after the
default idle period and the next request pays a full reload. Measured today: both `srv5`
and `srv6` reported `none resident` minutes after serving a request. Any latency
measurement that does not declare residency state is measuring two different things
depending on when it ran.

### 5. The second pair is entirely outside the model

`spark-c2b1` (192.168.1.232) and `spark-ac79` (192.168.1.233) are provisioned, serving,
and unknown to every authority that should own them:

- no `SparkCellRoleAssignment` — `gunbc.spark.cell_role` knows only `srv5` and `srv6`
- no fleet slot, so `gunbc.network_identity_subsumption` cannot bind them
- their static DHCP reservations are refused by design by that same module, so they have
  no modeled network identity even in principle
- different service user (`briansrls`, not `gunbc-automation`)
- different install root (`/usr/local/ollama`, not the install-paths authority's tree)
- different unit name (`ollama.service`, not `gunbc-spark-serving.service`)
- `OLLAMA_KEEP_ALIVE=-1`, which the modeled unit has no field for

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

1. **Freeze convergence.** No `gunbc.spark` serving convergence against `srv5`/`srv6`
   until (2) lands. This is the only item that is urgent rather than important.
2. **Make the live configuration the modeled desire**, or explicitly rule the hand-edit a
   deviation. Context, slot count and residency all need modeled fields; slot count needs
   one invented, since it has no representation at all.
3. **Decide the served model in the model.** If the fleet is moving to a DeepSeek build or
   to dspark, the desired row should say so rather than continuing to name gpt-oss.
4. **Admit the second pair** — cell roles, host identities, and whatever network identity
   can be bound given that static reservations are refused by design.
5. **Widen the realization carrier to environment**, after the ruling in (7).
6. **Route credentials through the modeled `SecretRef`** and rotate the current one.

## The standing risk this document is really about

Items 1 through 4 are one class: the model's desired state has drifted behind the fleet,
so the authority that is supposed to be able to rebuild a host would instead damage it.
That is worse than having no model, because a stale authority is one somebody will
eventually trust.

The honest reading is that `gunbc.spark`'s desired state is currently a record of what the
fleet looked like at one past moment, not a specification of what it should be. Until
items 2 and 3 land it should be treated as observation, not intent.

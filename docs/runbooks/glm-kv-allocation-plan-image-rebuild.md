# GLM arm: rebuild the derived serving image with the KV allocation plan emission patch

Operator recipe. Rebuilds the Group B GLM serving image so the patched vLLM
emits its resolved KV allocation plan at startup (owner directive
2026-09-25, docs/plans/moa-memory-modeling.md §"The allocation-plan
projection"). The patch is reviewable text at
`dag/extdeps/vllm/patches/kv-allocation-plan-emission-8d09804c8.patch`; this
runbook is the build and roll-out procedure. All steps run on the fleet by the
operator — nothing here has been executed as part of authoring the patch.

## AMENDMENT 2026-09-26 (owner directive, docs/plans/shared-pool-byte-correction.md): build v2, not v1

The v1 patch above is SUPERSEDED for new builds by
`dag/extdeps/vllm/patches/kv-allocation-plan-emission-v2-8d09804c8.patch`
(sha256 `8267d86be0b7f2690119ed22d09e3299cea69debc6032c57f6eacf2fec8252c5`).
v2 is a full standalone patch against the SAME pinned base
(`8d09804c877c48165c6ba69bc9dc02d09bae0b83`; the pre-patch sha256 pins are
unchanged: `core.py` `ef709a03...`, `envs.py` `1a2c6121...`) — apply v2
INSTEAD of v1, never both. Every step below applies verbatim with two
substitutions: the patch path becomes the v2 path, and the post-patch pin for
`vllm/v1/engine/core.py` becomes sha256
`7ffe8efa43e532d41722509db90db0c0db57af1b44b2b5dcd2d47a9088c9245e` (the
`vllm/envs.py` post-patch pin is unchanged:
`239d509314d44fd90c0aa2f39e8306c1d5976533763e4c2784d1ed97da774747`). v2 was
witnessed to apply cleanly by `git apply --check` against a fresh checkout at
the pin, and the patched `core.py` parses.

WHAT v2 CHANGES AND WHY: the emission's schema moves to
`kv-allocation-plan/v2` and the record gains the engine's own physical
shared-pool figures — `physical_pool_bytes_per_block`
(`_get_kv_cache_bytes_per_block(scheduler_kv_cache_config.kv_cache_groups)`,
the divisor the allocator used to size the pool) and `allocated_pool_bytes`
(physical_pool_bytes_per_block x num_blocks, the worker's single backing
tensor). vLLM allocates ONE backing tensor at startup and every cache group
overlays it — a block ID owned by any group consumes one block of it — so the
per-group `page_size_bytes` are the LOGICAL per-group charge, not the pool
block cost; v1's record carried only the group figures, and summing them
underpriced the 262144x6 request ~106x (the withdrawn 125,010,432-byte
figure). The corpus capture mint
(`extdeps.vllm.kv_layout` / `gunbc.spark.vllm_allocation_plan_observe`)
accepts BOTH vintages: the two fields are optional-until-emitted, the v1
capture stays readable, and the byte-conservation wall
(`allocated_pool_bytes == allocator_blocks x physical_pool_bytes_per_block`)
is active only for v2.

After the v2 image flies and a bounded run captures a `KV_ALLOCATION_PLAN_JSON`
line: expect the capture to mint with both pool fields present (verify
`physical_pool_bytes_per_block` against TP3's observed arena — the doctrine's
reconciliation implies ~17.1 MiB/block at 1035 blocks against the 17.29 GiB
arena, i.e. allocated_pool_bytes ≈ 17.29 GiB with the arena slack under one
block), and expect the arm-side fold to mint PoolConstructionProved beside the
block wall's CacheFitProved. ANY NEW WET RUN IS THE PARENT'S — this lane runs
nothing; the operator sequence (rebuild, probe, bounded launch holding the
stop legs past the ranks' profile prints per the landed readback bracket,
capture) is fleet-owned on the serving lane's go.


## What is being rebuilt, and why this shape

- **Base**: the recorded derived image `gunbc-vllm-glm53-gb10`, tag
  `derived-c99be081f219d123`, content id `sha256:c7d63d15...` (read the full id
  from the daemon in step 2; `RepoDigests` is empty because it was never
  pushed — pin by content id, per `gunbc.spark.serving_arm`
  `glm53_derived_row`). Its vLLM self-reports `0.1.dev1+g8d09804c8`, the pinned
  source revision `8d09804c877c48165c6ba69bc9dc02d09bae0b83`
  (`extdeps.vllm.runtime_defaults` `Vllm8d09804`).
- **Why a file-copy layer and not a source rebuild**: the patch touches only
  `vllm/v1/engine/core.py` and `vllm/envs.py` — both pure Python, no compiled
  artifacts, no ABI surface. The patched files can be copied over the installed
  package in a derived layer. (The corpus-native source-converge route,
  `gunbc.spark.vllm_runtime_image_build` + `v41_source_patch_converge`, remains
  the authority for patches that touch compiled code; this recipe is the
  surgical path for a pure-Python emission patch, and says so rather than
  pretending a two-hour aarch64 recompile is needed.)

## Pinning records (verify before building)

| artifact | identity |
|---|---|
| patch file | `dag/extdeps/vllm/patches/kv-allocation-plan-emission-8d09804c8.patch` sha256 `5b75b7847d6ef46089d067084c6f92c11cc20c933dbfa7fd65759f5d2e695da9` |
| patched file (post-patch) | `vllm/v1/engine/core.py` sha256 `8a29760b321d8db62d6856711fa99fee942c2c40378e3d265771a80e4512f3c0` |
| patched file (post-patch) | `vllm/envs.py` sha256 `239d509314d44fd90c0aa2f39e8306c1d5976533763e4c2784d1ed97da774747` |
| pre-patch base files (in the patch header) | `core.py` sha256 `ef709a03...`, `envs.py` sha256 `1a2c6121...` |
| engine base revision | `github.com/vllm-project/vllm` @ `8d09804c877c48165c6ba69bc9dc02d09bae0b83` |

The patch applies cleanly at the pinned revision — witnessed by
`git apply --check` against a fresh upstream checkout (dry application check,
recorded in the delivery notes of the lane that authored it).

## Steps (operator, on the fleet)

1. **Materialize the patched source.** On the build host (the GLM lane's build
   host — where the prior derived image was built; the provenance in
   `gunbc.spark.serving_arm` reads docker history on the Group B head
   192.168.1.236):
   ```bash
   git clone --no-checkout --filter=blob:none https://github.com/vllm-project/vllm.git /root/vllm-kv-plan
   cd /root/vllm-kv-plan
   git fetch --depth 1 origin 8d09804c877c48165c6ba69bc9dc02d09bae0b83
   git checkout 8d09804c877c48165c6ba69bc9dc02d09bae0b83
   git apply --check /path/to/gunbc/dag/extdeps/vllm/patches/kv-allocation-plan-emission-8d09804c8.patch
   git apply /path/to/gunbc/dag/extdeps/vllm/patches/kv-allocation-plan-emission-8d09804c8.patch
   sha256sum vllm/v1/engine/core.py vllm/envs.py   # expect the two post-patch pins above
   ```
2. **Read the base image content id** (never trust the tag alone):
   ```bash
   docker image inspect gunbc-vllm-glm53-gb10:derived-c99be081f219d123 --format '{{.Id}}'
   ```
   Confirm it is the id recorded in `glm53_derived_row` before using it as
   `FROM`. Confirm the installed vllm package location (debian images:
   `/usr/local/lib/python3.12/dist-packages/vllm`):
   ```bash
   docker run --rm <base-id> python3 -c 'import vllm, os; print(os.path.dirname(vllm.__file__)); print(vllm.__version__)'
   ```
   Expect the version string `0.1.dev1+g8d09804c8` — if it differs, STOP: the
   base is not the pinned subject.
3. **Stage the build context** (Dockerfile beside the two patched files, paths
   relative to the context):
   ```dockerfile
   FROM <base-id-from-step-2>
   COPY vllm/v1/engine/core.py <pkg>/vllm/v1/engine/core.py
   COPY vllm/envs.py <pkg>/vllm/envs.py
   ```
   where `<pkg>` is the directory printed in step 2.
4. **Build the derived layer:**
   ```bash
   docker build -t gunbc-vllm-glm53-gb10:derived-kvplan-<yyyymmdd> .
   ```
   Tag note: the durable tag authority is
   `gunbc.spark.vllm_runtime_image_build glm53_derived_tag`, whose input set
   today folds the flashinfer wheel set only; adding the patch to that fold is
   a model-lane follow-up, so this interim tag carries the patch name and date
   and must not be read as the converged tag.
5. **Probe the built image before it can serve anything:**
   ```bash
   docker run --rm <new-id> python3 -c '
   import vllm.v1.engine.core as c
   assert c.KV_ALLOCATION_PLAN_LOG_MARKER == "KV_ALLOCATION_PLAN_JSON "
   import vllm.envs as e
   assert e.VLLM_KV_ALLOCATION_PLAN_PATH is None
   print("patch probe OK")'
   ```
   (Imports recompile the two modules' bytecode on first use; the stale
   `RECORD` hashes in dist-info affect `pip install` verification only, not
   imports — stated so the next operator does not chase it.)
6. **Relaunch the arm on the new image** (operator fleet op; the launch plan
   authority is `gunbc.spark.serving_arm_launch`). Add to the launch:
   - `-e VLLM_KV_ALLOCATION_PLAN_IMAGE_DIGEST=<new-image-id>` (self-report the
     record carries beside the observer's authoritative launch reading);
   - optionally a host bind mount and `-e VLLM_KV_ALLOCATION_PLAN_PATH=/out/kv-plan.json`
     if the file channel is wanted; the log channel needs no launch change.
7. **Capture and hand to the model lane.** From the head container's captured
   startup stdout, take the last `KV_ALLOCATION_PLAN_JSON` line (or the file);
   read the startup capacity lines beside it; run
   `gunbc.spark.vllm_allocation_plan_observe`'s fold. EXPECT the mint's
   revision wall to refuse on first real capture: the emission self-reports
   `g8d09804c8...`, a revision the corpus has not modeled. That refusal is the
   wall working; advancing `extdeps.vllm.kv_layout`'s modeled revision to the
   patched source — after reading the emission at that revision — is the model
   lane's act. Record the captured emission bytes and the launch identity with
   the wet receipt bundle.

## What this recipe does not do

- It does not touch systemd units, does not push anywhere (`RepoDigests` stays
  empty; the fleet consumes content ids), and does not change which image the
  corpus authorities describe — the `serving_arm` runtime row still names the
  old derived image until its owner cuts the arm over.
- It does not mint a plan receipt: no real capture exists until step 7 runs,
  and the mint refuses until the modeled revision advances. Any document
  claiming a `ResolvedKvAllocationPlan` for this arm before both has happened
  is a fixture wearing a receipt's name.

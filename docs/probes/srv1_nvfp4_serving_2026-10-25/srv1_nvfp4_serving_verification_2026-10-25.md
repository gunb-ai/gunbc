# srv1 NVFP4 serving verification + FP8 KV/attention test matrix (Blackwell)

**Found:** 2026-10-25 (UTC), read-only probe + benchmark against the **running**
server. No change was made to srv1's serving at any point. Operator request
2026-10-24; dashboard node `adhoc-15068614-20b`; session `sharp-ram-303`.
**Class:** receipt (observing-layer observation, DESIGN §3 "observations are
receipts in the observing layer"). Raw measurement JSON alongside this file in
this directory (`raw_*.json` / `raw_*.jsonl`).

## Question

Is srv1 **serving NVFP4 (FP4) model weights**, and does FP8 KV cache / FP8-capable
attention work on its Blackwell GPU? Measure correctness, throughput, TTFT,
peak memory, max context — keyed by engine version and flags.

## What srv1 is, as observed (engine's own startup log + live state, §4d — not config intent)

| item | observed value | source |
|---|---|---|
| GPU | 1x NVIDIA RTX PRO 6000 Blackwell (sm_120), 97,887 MiB | `nvidia-smi` |
| Driver | 580.178.04 | `nvidia-smi`; `serve/events.log` |
| CUDA | 13.4 (container image) | image tag |
| OS | Ubuntu 26.04 LTS, kernel 7.0.0-27-generic | `uname -a` |
| Engine | vLLM `0.1.dev22105+g5009fa56b` | `pip show vllm` in container |
| Image | `ghcr.io/local-inference-lab/vllm:karmic-kraken-beta-spark-20261007-d071fb3dde0ba979` | `docker inspect` |
| Container | `lil-flash-next` — hand-run `docker run`, **no systemd unit**, `Restart=no`, cgroup parent `srv1-inference.slice` (MemoryMax=80G), last start 2026-10-25T02:26:10Z | `docker inspect`, `systemctl show` |
| Model (served) | `Qwen3.8-Flash-Next` | engine log `served_model_name=` |
| Checkpoint | `local-inference-lab/Qwen3.8-Flash-Next-NVFP4-MXFP8-CSF-QAD`, HF snapshot `dc5b8ab2fa14bc4fecd6886273b7ebc6095662e2` | engine log `model=`; HF cache layout |
| **Weights as loaded** | **NVFP4**: engine log `quantization=nvfp4_csf load_format=nvfp4_csf`; checkpoint config `quant_method=modelopt, quant_algo=MIXED_PRECISION`, MoE experts NVFP4 `group_size=16, weight_scale_encoding=csf`; engine chose `B12X NvFp4 MoE backend` from `[FLASHINFER_TRTLLM, FLASHINFER_CUTEDSL, FLASHINFER_CUTEDSL_BATCHED, FLASHINFER_CUTLASS, VLLM_CUTLASS, MARLIN, HUMMING, EMULATION]` | serve.log engine-init line |
| Activation dtype | `torch.bfloat16` | engine log `dtype=` |
| **KV cache** | **`fp8`** (`kv_cache_dtype=fp8`), block_size 64, GPU KV cache 774,019 tokens / 2.95x concurrency at 262,144 tok per request (engine log, last boot), prefix caching on, LMCache kv connector (`kv_connector=LMCacheConnectorV1`, kv-role `kv_both`; LMCache own log: multiprocess L2, token hasher `chunk_size=4096 blake3`, tensor allocator free 61,440 MB) | engine log / serve.log |
| Attention backend | `AttentionBackendEnum.FLASH_ATTN` (also selected for `MMEncoderAttention`); FLASHINFER appears only for top-p/top-k sampling and as MoE candidates. No attention-backend override flag was passed (auto-selected). | engine log |
| Spec decode | MTP (`method=mtp`, `num_speculative_tokens=3`) | engine log `speculative_config=` |
| Context | `max_model_len=262144` (256k) | engine log |
| Scheduling flags | `gpu_memory_utilization=0.96`, `max_num_batched_tokens=6019`, `max_num_seqs=16`, `enable_chunked_prefill`, `enable_prefix_caching`, `async_scheduling`, `optimization_level=2` | engine log `non-default args` |
| Endpoint | TLS HTTPS `0.0.0.0:18000` (`/v1/models` requires bearer; `/metrics` open) | live probe |
| KV scale | observed value is exactly `cache_dtype=fp8` (engine init line); neither `e4m3` nor `e5m2` appears anywhere in serve.log, so the fp8 subtype is not stated by the instrument — vLLM's documented default for `fp8` is e4m3 (inference, not observation; untestable per-request) | serve.log grep |

**Verdict on the question: YES — srv1 is already serving NVFP4 weights, with
FP8 KV cache and the FP8-capable FLASH_ATTN path active on Blackwell.** No
step-2 change is needed; none was made.

**The modeled home exists, and the container is NOT enrolled there.** The
launch domain for a serving host is `product.fabric.serving_recipe`
(`serving_recipe_flash_next_lab_vllm` — the lab vLLM engine
`extdeps.vllm.local_inference_lab_build`, RTX PRO 6000 Blackwell card class,
published port 18000, driver floor derived from CUDA 13) feeding the
`gunbc.serving` launch legs (`serving_provision_derivation` →
`serving_artifact_placement` + `serving_launch`, process identity in
`replica_endpoint_launch`, routes in `serving_enrollment`). srv1's running
container diverges from that model on three observed points, not because no
model exists:
1. `serving_provision_derive` **refuses the vLLM engine typed**
   (`ServingProvisionRefusedEngineVllm`: "no vllm launch argv is modeled") —
   a declared frontier row in that module; only the ninfer arm has a modeled
   launch (`gunbc.harness.operator_pod_launch operator_pod_ninfer_launch`).
   So the exact flags the running container uses
   (`quantization/load_format=nvfp4_csf`, `kv_cache_dtype=fp8`, MTP x3,
   LMCache connector, `max_model_len=262144`) have **no modeled spelling**.
2. The recipe's weights are `WeightsNamedUnpinned { weights:
   "Qwen3.8-Flash-Next", quantization: none }` with `served_model_id: none` —
   no digest to place, which the derivation also refuses. The running
   checkpoint pin (`…NVFP4-MXFP8-CSF-QAD` @ `dc5b8ab2…`) exists only in the
   container's own mounts.
3. `gunbc.serving.serving_enrollment` keys routes by
   `rented_replica.RentedReplicaIdentity` and carries **no srv1 row** (the
   module is written for rented/flex replicas; the recipe's standing cites
   the Flex L2 template read by crisp-seal-306, 2026-10-19 — an outward-rental
   context, not srv1).

So the launch is a **hand `docker run` with no repo row binding it**: a
declared divergence from `product.fabric.serving_recipe
serving_recipe_flash_next_lab_vllm` + `gunbc.serving.*`, for the specific
reasons above. The engine row closest to modeled, `extdeps.vllm
.local_inference_lab_build` (`local_inference_lab_vllm_release`), carries a
`read_obligation` (source revision, launch flags, driver) which this probe
discharges: revision `5009fa56b`, flags as tabled above, driver 580.178.04.

Host context (untouched): srv1 also runs `actions-runner@srv1-01/-02`,
`gunbc-approval-broker`, `gunbc-ntfy` — verified untouched; only read-only
commands and API calls against port 18000 were executed.

## Test matrix (measured 2026-10-25T07:48–08:00Z, keyed by engine version + flags)

Engine: vLLM `0.1.dev22105+g5009fa56b` (image tag `...spark-20261007-d071fb3dde0ba979`),
checkpoint `dc5b8ab2…`, flags as the table above. All numbers from a
temperature-0 fixed prompt set against the live server; harness committed at
`srv1_serving_bench.py.txt` (this directory; run as python; repo-wide `*.py` ignore applies).

| arm | KV dtype | attention backend | status | correctness (6-prompt set, temp 0) | prefill tok/s | decode tok/s (1-stream) | TTFT | peak GPU mem | max ctx that fits |
|---|---|---|---|---|---|---|---|---|---|
| **A (as running)** | `fp8` | FLASH_ATTN (auto) | **RUN** | 6/6 correct at temp 0; per-prompt answer SHA-256/16 + heads in `raw_eval3.json` | `raw_arm_fp8_kv_flash_attn.json` `prefill-30k`: 5,803 tok, 7,828.1 tok/s (first calibration); `raw_longctx3.jsonl` `prefill-30k`: 30,058 tok, **8,030.8 tok/s**; `raw_cold.json` `cold-prefill-150k`: 228,588 tok, **7,996.2 tok/s** cold (unique content, no prefix cache) | `raw_eval3.json` eval rows, decode tok/s 189.6–528.4 (all six rows, incl. `code-tests` 189.6); `raw_arm_fp8_kv_flash_attn.json` `countdown-200`: 700 completion tokens, 313.5 tok/s sustained, MTP x3 active | `raw_eval3.json`: 0.352–0.442 s (short prompts); `raw_cold.json`: 28.587 s at 228,588 tokens cold | `raw_arm_fp8_kv_flash_attn.json` `peak_gpu_mem_mib`: **93,630 MiB** of 97,887 (engine steady state, incl. CUDA graphs + LMCache) | `raw_longctx3.jsonl`: 259,900 prompt + 16 out → 200 (ctx-260k); 262,129+ → 400 (error below) |
| B: KV `bf16` | `auto`/bf16 | any | **REFUSED — not runnable against the running server** | — | — | — | — | — | — | `kv_cache_dtype` is an **engine-startup** flag: a per-request `{"kv_cache_dtype":"auto"}` POST is **silently ignored** (HTTP 200, served with fp8 KV); running a bf16-KV arm requires restarting the container (`docker rm -f` + re-run with `--kv-cache-dtype auto`) = a serving change, gated on operator go. GPU headroom for a 2nd parallel server: **none** (93.6/97.9 GiB used at idle; slice MemoryMax 80G). |
| C: KV `fp8` x backend `FLASHINFER` override | `fp8` | FLASHINFER | **REFUSED — not runnable against the running server** | — | — | — | — | — | — | `attention_backend` is likewise engine-level: per-request `{"attention_backend":"FLASHINFER"}` POST **silently ignored** (HTTP 200, still FLASH_ATTN); backend override needs `--attention-config` at launch = serving change, same gate. |
| D: KV `fp8_e5m2` | `fp8_e5m2` | — | **REFUSED — variant not started** | — | — | — | — | — | — | no `e5m2` string in serve.log; the running engine only ever initialized the plain `fp8` path; testing e5m2 needs a relaunch (same gate). |

Additional refusals observed while measuring (not skipped — recorded with errors):

- 262,129+ token prompt: `HTTP 400 "This model's maximum context length is
  262144 tokens. However, you requested 16 output tokens and your prompt
  contains at least 262129 input tokens, for a total of at least 262145
  tokens."` (engine tokenizer limit, as intended).
- `POST /v1/tokenize` → `404 Not Found` (this vLLM build serves no tokenize route).
- Thinking model: completions stream `delta.reasoning` with `content: null`;
  a small `max_tokens` is consumed entirely by thinking and the final answer
  truncates (observed on budgets ≤220). Harness captures both channels; the
  correctness rows ran 4x budgets (288–880) so every answer completed.

Aggregate (16 concurrent short requests, arm A): two runs are recorded —
`raw_arm_fp8_kv_flash_attn.json` `burst-16`: 167.3 aggregate decode tok/s
(2.30 s wall, 384 tokens, 0 errors, burst TTFT p50 2.278 s);
`raw_eval3.json` `burst-16`: 183.3 tok/s (2.09 s wall, 384 tokens, 0 errors,
p50 1.804 s). Prefill is chunked (`max_num_batched_tokens=6019`) and runs
concurrently with decode, hence burst decode sits below single-stream.

## Limits of this receipt

- Arm A is measured; arms B–D are **engine-launch-level options** that cannot
  be exercised against a running server — recorded refused with the observed
  error/ignore behavior, per the brief. Their diff against the fp8 baseline
  needs one brief maintenance window (stop `lil-flash-next`, relaunch with the
  arm's flags, re-run this directory's harness — `srv1_serving_bench.py.txt`
  copied to srv1 as `srv1_serving_bench.py`, invoked as
  `python3 srv1_serving_bench.py <key-file> <out.json> --arm <name>` — and
  restore).
- Throughput figures are single-request and 16-burst probes on an idle GPU at
  probe time, not the CI-loaded profile; srv1 runners were idle (load avg 0.01).
- `~180k context` premium target: **fits with margin** — 179,937-token prompt
  returned 200 at ~10.6 s (cached-prefix); cold 228,588-token prefill at 28.6 s.

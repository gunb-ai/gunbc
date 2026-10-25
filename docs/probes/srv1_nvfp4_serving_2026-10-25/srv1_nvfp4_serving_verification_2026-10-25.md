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
| **KV cache** | **`fp8`** (`kv_cache_dtype=fp8`), block_size 64, 774,019 GPU-KV tokens, prefix caching on, LMCache kv connector (`kv_connector=LMCacheConnectorV1`, kv-role `kv_both`, L1 24G / L2 200G ram-disk) | engine log |
| Attention backend | `AttentionBackendEnum.FLASH_ATTN` (also selected for `MMEncoderAttention`); FLASHINFER appears only for top-p/top-k sampling and as MoE candidates. No attention-backend override flag was passed (auto-selected). | engine log |
| Spec decode | MTP (`method=mtp`, `num_speculative_tokens=3`) | engine log `speculative_config=` |
| Context | `max_model_len=262144` (256k) | engine log |
| Scheduling flags | `gpu_memory_utilization=0.96`, `max_num_batched_tokens=6019`, `max_num_seqs=16`, `enable_chunked_prefill`, `enable_prefix_caching`, `async_scheduling`, `optimization_level=2` | engine log `non-default args` |
| Endpoint | TLS HTTPS `0.0.0.0:18000` (`/v1/models` requires bearer; `/metrics` open) | live probe |
| KV scale | `--kv-cache-dtype fp8` maps to the e4m3 path (no `e5m2` string appears anywhere in serve.log) | serve.log grep |

**Verdict on the question: YES — srv1 is already serving NVFP4 weights, with
FP8 KV cache and the FP8-capable FLASH_ATTN path active on Blackwell.** No
step-2 change is needed; none was made. There is **no modeled launch route**
for this container in the repo (no `gunbc.serving` module; `gunbc.spark`
`first_party_serving` is the *retired* DGX Spark 4 target per #13779 and lists
no srv1 serving target; `operator_pod_launch` is a fleet-declaration consumer,
not this launch). The hand launch is therefore a **declared divergence**: the
closest modeled row is `extdeps.vllm.local_inference_lab_build` (image family
`ghcr.io/local-inference-lab/vllm`), whose `read_obligation` (source revision,
launch flags, driver) this probe discharges: revision `5009fa56b`, flags above,
driver 580.178.04.

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
| **A (as running)** | `fp8` | FLASH_ATTN (auto) | **RUN** | 6/6 correct; answer SHA-256/16 in raw_eval3.json (rot13 code, bug fixed, 11:36 meet-time, 92, 4 planets JSON, pytest cases) | **7,833** (209,975 tok prompt, warm-prefix) / **7,996 cold** (228,588 tok, unique content) | 217–528 (short, MTP accept visible at 528 on 25-tok answer); countdown-700tok: **176.4** | 0.30–0.44 s short prompts; 26.8 s @210k tok; 28.6 s @228.6k tok cold | **93,630 MiB** (of 97,887; steady-state engine footprint, incl. CUDA graphs + LMCache) | **259,900 prompt + 16 out = 260k fits** (status 200); 262,129+ refused (below) |
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

Aggregate (16 concurrent short requests, arm A): **183 tok/s aggregate decode**
(2.09 s wall, 384 completion tokens, 0 errors), burst TTFT p50 1.80 s / max 2.08 s.
Prefill is chunked (`max_num_batched_tokens=6019`) and runs concurrently with
decode, hence burst decode sits below single-stream.

## Limits of this receipt

- Arm A is measured; arms B–D are **engine-launch-level options** that cannot
  be exercised against a running server — recorded refused with the observed
  error/ignore behavior, per the brief. Their diff against the fp8 baseline
  needs one brief maintenance window (stop `lil-flash-next`, relaunch with the
  arm's flags, re-run `bench/srv1_serving_bench.py --arm <name>`, restore).
- Throughput figures are single-request and 16-burst probes on an idle GPU at
  probe time, not the CI-loaded profile; srv1 runners were idle (load avg 0.01).
- `~180k context` premium target: **fits with margin** — 179,937-token prompt
  returned 200 at ~10.6 s (cached-prefix); cold 228,588-token prefill at 28.6 s.

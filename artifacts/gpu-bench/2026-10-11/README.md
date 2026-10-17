# Rented-GPU serving benchmarks, 2026-10-11

Serving throughput and capacity for two models on rented vast.ai GPUs, captured to
compare cards for an at-home deployment (power-constrained, single host). Every file
under `runs/` is the verbatim output of the kit in `kit/`; nothing there is authored.
This README is the only authored file: it states provenance, configuration, and which
runs are valid. Numbers quoted below are medians copied from the named `results_*.txt`
for reading convenience; the `results_*.txt` and the raw `*.jsonl` are the authority.

## The instrument (`kit/`)

`kit/` is an **out-of-tree tool captured verbatim for provenance**, not a realization of
anything in this repository. It ran by hand on rented vast.ai boxes; nothing here
invokes, builds, tests or maintains it, and it is frozen at the state that produced
`runs/` (including the `sed`-derived per-run copies of `sample.sh`). Re-deriving a run
means copying `kit/` to a box with the same GPU, image and checkpoint and running the
same command; the scripts are kept so that `runs/` names its producer. `*.py` files are
force-added because the repository `.gitignore` excludes `*.py`.


| step | what it does |
|---|---|
| `setup_ninfer.sh` | builds NInfer at commit `81c8ce09` and fetches `neroued/Qwen3.8-27B-nvfp4-NInfer` |
| `serve_ninfer.sh` / `serve_vllm.sh` | launch one engine on port 8081 (8080 is the image's Jupyter) |
| `auto.sh` (NInfer) / `vauto.sh` (vLLM) | start the server, find the largest KV capacity that starts (NInfer), run `run.sh`, scrape `/metrics` before and after, sample telemetry |
| `run.sh` → `longbench.py` | **short**: rounds of N concurrent 512-token generations from a ~20-token prompt, 4 min; logs aggregate tok/s per round |
| `run.sh` → `longctx.py` | **warm**: warms max(N) sessions of ~174k tokens each, then cycles the N values for 8 min; each round every active session appends a question and streams 512 tokens; logs aggregate, per-stream decode, TTFT |
| `sample.sh` | GPU memory/util/power/clocks/temp and host RAM/swap every 5 s → `telemetry_<label>.csv` |
| `allreduce.py` | NCCL all-reduce latency/bandwidth across 4 KB–256 MB, with and without P2P (dual-GPU boxes) |
| `nvidia-smi dmon -s t -d 1` | per-GPU PCIe rx/tx once a second during dual-GPU runs → `pcie_<label>.log` |

**The warm working set is max(N) sessions, not N.** `longctx.py` touches sessions
0..N−1 every round, so all max(N) warmed sessions stay live for the whole run. A run
is valid only if max(N) × ~176k (prompt + replies) fits in the engine's KV cache;
otherwise LRU eviction recomputes most prompts at every N, including N=1.

## Validity rule

A warm result is **valid** when the run's `/metrics` show ≥ 94% prefix-cache hits and
0 preemptions (vLLM), or 0 replay/snapshot restores (NInfer), and max(N) fits the
logged KV capacity. Short results do not depend on the cache and are valid whenever
the run completed. Over-capacity warm runs are kept beside the valid ones and marked
invalid in the table below.

## Runs

`Q27` = `Qwen3.8-27B` dense (NInfer file above; vLLM `nvidia/Qwen3.8-27B-NVFP4`), NVFP4
weights, MTP 3 draft tokens. `FN` = `Qwen/Qwen3.8-Flash-Next` (125B MoE, 6B active,
51B n-gram table kept in pinned host RAM by vLLM's default `VLLM_PLE_CPU_OFFLOAD`), BF16
KV (vLLM 0.30's QSA backend refuses other KV dtypes), MTP 3, `--mamba-cache-mode align`.
All vLLM runs are 0.30.0 from the `vastai/vllm_v0.30.0-cuda-13.0` image.

| dir | GPU(s), host | model / engine / weights / KV | launch | KV capacity | sessions at 174k | short N=1 / N=8 | warm (max N) agg, TTFT | standing |
|---|---|---|---|---|---|---|---|---|
| `pro5000_48g` | 1× RTX PRO 5000 48 GB, Ryzen 9950X, Gen5 ×8 | Q27 / NInfer / NVFP4 / NVFP4 | 7 lanes, max ctx 240k | 1.30M | 7 | 99 / 544 (N=7) | 169 @7, 2.1 s | valid; replay counters not captured (pre-dates scrape), fit inferred from capacity |
| `pro5000_72g` | 1× RTX PRO 5000 72 GB, EPYC 9734, Gen5 ×16 | Q27 / NInfer / NVFP4 / NVFP4 | 8 lanes, max ctx 240k | 1.92M (engine cap 8 × max ctx) | 8 | 96 / 575 | 163 @8, 4.6 s | valid; counters not captured |
| `pro5000_72g` | same | Q27 / NInfer / NVFP4 / FP8 | 8 lanes, max ctx 262k | 1.45M | 8 | 97 / 584 | 170 @8, 3.3 s | valid; 0 replay, 0 snapshot restore (`metrics_ninfer_fp8kv_pro5000_72g.prom`) |
| `pro6000ws` | 1× RTX PRO 6000 WS, EPYC 7V12, Gen4 ×16 | Q27 / NInfer / NVFP4 / FP8 | 8 lanes, max ctx 262k | 2.10M | 8 | 136 / 775 | 171 @8, 7.2 s | valid; counters not captured |
| `pro6000ws` | same | Q27 / vLLM / NVFP4 / FP8 | util 0.90, 32 seqs | 1.56M | 8 | 120 / 809 | 197 @8, 9.0 s | valid; 94% hits, 0 preemptions |
| `pro6000ws` | same | FN / vLLM / NVFP4 | util 0.90, 8 seqs | — | 0 | — | — | **does not fit**: weights 80.1 GiB on GPU, KV −4.3 GiB (`serve_vllm_flashnext_bf16kv_pro6000ws.log`) |
| `2xpro5000` | 2× RTX PRO 5000 48 GB, EPYC 7K62 (Rome), Gen4 ×16 | Q27 / vLLM TP2 / NVFP4 / FP8 | util 0.90, 32 seqs | 1.46M | 8 requested | 146 / 947 | — | short valid; **warm invalid** (46% hits: 8 × 176k did not fit 1.46M) |
| `2xpro5000_genoa` | 2× RTX PRO 5000 48 GB, EPYC 9354P (Genoa), Gen5 ×16 | Q27 / vLLM TP2 / NVFP4 / FP8 | util 0.90, 16 seqs | 1.50M | 7 | 142 / 971 | 240 @7, 6.4 s | valid; 95% hits, 0 preemptions |
| `2xpro6000maxq` | 2× RTX PRO 6000 Max-Q (300 W), Threadripper PRO 9995WX, Gen5 ×16 | FN / vLLM TP2+EP / NVFP4 | util 0.85, 16 seqs, batched tokens 8192 | 2.54M | 14 | 183 / 906 | 614 @10, 2.1 s | valid; 96.5% hits, 0 preemptions |
| `2xpro6000ws` | 2× RTX PRO 6000 WS (600 W), EPYC 9R14, Gen5 ×16 | FN / vLLM TP2+EP / NVFP4 | same as Max-Q | 2.54M | 14 | 194 / 980 | 636 @10, 2.5 s | valid; 96.7% hits, 0 preemptions |
| `b200` | 1× B200 | FN / vLLM / FP8 (`Qwen/…-FP8`) | util 0.90, 32 seqs | 0.83M | 4 | 204 / 1,174 | — | short valid; **warm invalid** (max N 6 > 4) |
| `b200` | same | FN / vLLM / NVFP4 | util 0.90, 16 seqs | 2.55M | 14 | 229 / 1,402 | 780 @12, 3.1 s | valid; 96.6% hits, 0 preemptions; short N=32 is two waves of 16 |
| `b300` | 1× B300 SXM6 | FN / vLLM / FP8 | util 0.90, 32 seqs | 3.61M | 20 | 223 / 1,195 | 654 @16, 5.6 s | valid; 95.4% hits, 0 preemptions. An earlier run with max N 24 > 20 thrashed and was not retained |

Session counts are for ~174k-token sessions. At the full 262,144-token window each
pool holds fewer (e.g. 1.30M → 4).

## Dual-GPU link (`allreduce_*.txt`, P2P on)

| box | small-message latency | peak bus bandwidth |
|---|---|---|
| `2xpro5000` (Rome, Gen4) | ~50 µs | 21.5 GB/s |
| `2xpro5000_genoa` (Genoa, Gen5) | ~19 µs | 34 GB/s |
| `2xpro6000ws` (EPYC 9R14, Gen5) | ~20 µs | 39 GB/s |
| `2xpro6000maxq` (Threadripper PRO, Gen5) | ~9 µs | 21 GB/s |

These are whole-platform measurements (CPU, board, topology together); they do not
isolate PCIe generation. During the Flash-Next runs, 1 s PCIe samples had a median of
~2.2 GB/s and peaked at 15–17 GB/s per GPU, below each link's measured peak. One-second
sampling cannot see sub-second bursts or time spent waiting on a sync, so this does not
by itself rule out a communication bottleneck.

## Failures kept as evidence

- `failed/<HHMM>_*` — every log `relaunch.sh` moved aside before relaunching at that UTC
  time. Most are failed starts; `pro6000ws/failed/2226_vauto.log` is the completed Q27
  vLLM run's driver log, moved aside when the FN attempt was launched. The failures:
  vLLM out-of-memory at warm-up: `--gpu-memory-utilization 0.95` (27B, 1 GPU)
  and `0.93`/`0.91` (27B, 2× 48 GB) leave no room for a ~1 GiB warm-up buffer; for FN on
  2× 96 GB, FlashInfer's CUTLASS MoE workspace (3.22 GiB at 32,768 batched tokens) is
  allocated outside vLLM's budget and OOMs even at 0.85 — fixed by
  `--max-num-batched-tokens 8192`. `*_oom095.log` are the 0.95 attempts.
- The first `2xpro6000ws` FN run (max N 16 > 11) also thrashed; its results were
  overwritten by the valid re-run and only its driver log under `failed/2247_*` remains.

## Consumer

Nothing in the corpus reads these files yet. The intended consumer is the change that
models the at-home serving-card selection (candidate cards, measured KV capacity and
warm throughput per card and power cap) against the existing decision home
(`std.decision`, `product.capacity`); that change should transcribe the rows it needs
into typed declarations citing this directory. Until then this is a record, not an
input.

## Not established here

- Why warm throughput for Q27 barely varies across cards (169–197 at 7–8 sessions for a
  1.3× spread in short throughput). Cache hits rule out recomputation; prefill share,
  MTP acceptance and queueing were not broken down.
- Wall power. `telemetry_*.csv` records GPU board power only.

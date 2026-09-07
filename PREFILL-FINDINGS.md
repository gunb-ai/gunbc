# The prefill budget is 2048, an exception handler chose it, and it is roughly the right value anyway

Investigation of `serving-prefill-scaling` @ `db02d517058`, 2026-09-07, on the live
group A head (192.168.1.225, spark-3bd5). The pre-sweep engine carried launch
invocation `d140fa39c1174403bf2ce4d7ed8c2d74` — **the same launch the branch's
samples were taken against**, so the baseline joins to them by identity.

Group A was swept across four launches and **restored to its authored unit**
(drop-in removed, `enable_mfu_metrics=False`, budget back to default, KV pool
12,775,604 tokens). Group B was never touched.

---

## 1. The effective batch budget is 2048 tokens

Established six independent ways.

**(a) The engine's own startup banner.** `'compile_ranges_endpoints': [2048]`, and
`vllm/config/vllm.py::_set_compile_ranges` states the binding outright:

    # The upper bound of the compile ranges is the max_num_batched_tokens.
    compile_range_end = self.scheduler_config.max_num_batched_tokens

**(b) Executed re-derivation** of the real `EngineArgs.get_batch_defaults(4)` inside
the serving container: `{LLM_CLASS: 8192, OPENAI_API_SERVER: 2048}` → **2048**.

**(c) A per-step counter.** With `--enable-mfu-metrics`,
`vllm:estimated_flops_per_gpu_total` increments once per engine step in fixed units
of 264,765,440. Its step counts on the control arm are exactly `ceil(tokens/2048)`:

    tokens        511  2048  4098  8186  12372  16510  20072
    flops units     1     1     3     4      7      9     10
    ceil(t/2048)    1     1     3     4      7      9     10

**(d) A hard behavioural ceiling.** `vllm:inter_token_latency_seconds` over 45 hours:
1,454,342 samples, **one above 5 s, none above 7.5 s**. A 20,082-token step would
impose ~21 s on every co-tenant decoder.

**(e) The 158,764-token cold run reconciles, and the mechanism is now known.**
158,764/2048 = 77.5 chunks against 83 observed counter advances; 2048 x 1.055 ms =
2.16 s against the 2.30 s median interval.

The counter does not advance per prefill chunk (see §3), so under an idle engine
that run should have advanced it about twice, not 83 times. The reconciliation is
the co-tenant measurement in §4: **a decoder sharing the replica receives exactly
one token per chunk**, and each token is one output batch. 185.5 s / 2.30 s ~ 81
advances against 77.5 chunks. So the run was NOT idle — its idleness was assumed,
not established — and the 83 advances were a co-tenant's tokens, one per chunk.

This matters for the failure-mode roster. The original quotient 158,764/83 ~ 1,913
was called a fabricated quantity that coincidentally resembled the 2048 default. It
was not a coincidence: a co-tenant decoder beside a chunked prefill emits one output
batch per chunk, so the quotient approximates the chunk size **by construction**,
through a mechanism nobody had identified. The original inference reached the right
answer by a route it misdescribed, and was then withdrawn as wrong.

That is the sharp specimen: **a wrong-grain quotient can land near the true answer,
and the near-miss is exactly what makes it unfalsifiable by inspection** — first as
apparent confirmation, then as apparent refutation.

**(f) The branch's own samples identify it.** Refitting the idle arm as
`wall = a*ceil(T/chunk) + b*T` selects 2048 (rms 18.4 ms) over the affine one-step
model (62.0 ms) and over every other chunk size. The pair already in the dataset:
8117 tokens → 4 steps → 8560 ms; 8281 tokens → 5 steps → 8900 ms. +2% tokens for
+4% time; one-step predicts +173 ms, 2048-chunk predicts +308 ms, observed +340 ms.

## 2. Root cause: an absorbing fallback in upstream vLLM

`vllm/engine/arg_utils.py::get_batch_defaults`:

    try:
        device_memory = current_platform.get_device_total_memory()
        device_name = current_platform.get_device_name().lower()
    except Exception:
        # This is only used to set default_max_num_batched_tokens
        device_memory = 0
        device_name = ""

    if device_memory >= 70 * GiB_bytes and "a100" not in device_name:
        default_max_num_batched_tokens = {LLM_CLASS: 16384, OPENAI_API_SERVER: 8192}
    else:
        default_max_num_batched_tokens = {LLM_CLASS: 8192, OPENAI_API_SERVER: 2048}

On a GB10 the memory is unified and `nvmlDeviceGetMemoryInfo` returns
`NVMLError_NotSupported`. The bare `except` substitutes `device_memory = 0`,
`0 >= 70 GiB` is false, and a 121 GB machine is classified as a small GPU.
`torch.cuda.get_device_properties(0).total_memory` reports 130,663,231,488 bytes
correctly — only the NVML path fails.

DESIGN §5's absorbing fallback exactly: a failure arm that widens to a conservative
answer instead of refusing, with a comment explaining why it was believed harmless.

## 3. The instrument behind the branch's headline correction does not measure steps

`vllm:iteration_tokens_total_count` advances once per **EngineCoreOutputs batch
containing at least one output**, not once per engine step.

`vllm/v1/engine/async_llm.py:663`:

    iteration_stats = (IterationStats() if (log_stats and num_outputs) else None)

`vllm/v1/metrics/loggers.py`, above the histogram observe: `if iteration_stats is
None: return`. A step that only advances a chunked prefill produces no output, so
the counter does not move. A prefill-only request therefore emits **exactly one
output batch** regardless of chunk count — confirmed on every probe in all three
arms, where `iter_delta == 1` even for requests measured at 10 steps.

So `iteration_delta == 1` from 512 to 20,082 tokens is **invariant, not
informative**. `IterationStats`' own docstring says it: "Stats associated with a
single set of EngineCoreOutputs."

Consequences for the branch:
- `spark_group_a_effective_batch_budget = BehaviouralPerIterationCapacityAtLeast
  { computed_tokens: 20082 }` is **refuted**; the value is 2048.
- The withdrawn conclusion in `serving_step_cost.dag` — that 158,764/83 ≈ 1,913
  matches the 2048 fallback — **was correct**, and was withdrawn on a broken instrument.
- `EffectiveSchedulerConfigUnread` is obsolete: the value is in the engine's own
  banner, and the process environment was never where it lived.
- The honest step counter is `vllm:estimated_flops_per_gpu_total` under
  `--enable-mfu-metrics`. That is the instrument this work needed.

## 4. The measured sweep

Four launches on group A, same harness, idle engine, unique nonce per prompt so
nothing is served from cache. Step counts read from the per-step counter.

Three-term fit over all 20 clean probes, `wall = R + S*steps + b*tokens`:

    R  per-request fixed     105.8 ms   (client + API server + tokenize)
    S  per-STEP fixed        189.9 ms
    b  per-token            0.9661 ms/token
    worst residual           201.8 ms   rms 82.2 ms

A 16,400-token cold prefill, by budget:

    budget  steps  pred ms  meas ms  vs 2048   step/stall ms    KV tokens
       512     33    22217        -   +25.8%       685 pred            -
      1024     17    19179        -    +8.6%      1179 pred            -
      2048      9    17660    17759    +0.0%      2168 meas   11,767,856
      4096      5    16900        -    -4.3%      4147 pred            -
      8192      3    16520    16652    -6.5%      8225 meas    8,704,293
     16384      2    16330    16327    -7.5%     15930 meas    5,836,755

The co-tenant stall law `S + b*budget` is validated to within 1% at every measured
budget: predicted 2169/8105/16019 ms against measured 2168/8225/15930 ms.

(512, 1024 and 4096 are model rows, not measurements: the sweep was stopped after
16384 when the harness lost permission to apply further arms. The model behind them
is the one validated to 1% above.)

**Decode alone: 30.1 / 30.1 / 29.5 ms per token across the three arms** — unchanged,
and healthy at ~33 tok/s. There is no decode-speed problem.

## 5. What the sweep settles

**The per-token cost is invariant across a 32x range of batch size.** 0.9661 ms/token
holds from 512 to 16,384 tokens per step. So:

- The ~1 ms/token is **not** a batching, scheduling or MoE-granularity artifact.
  Going from 48 to 384 tokens per routed expert per step changed it by ~2%.
  Review candidate #4 is refuted, and so is the MoE-granularity hypothesis I
  formed before measuring.
- The budget buys **only** per-step overhead amortisation: at most 7.5%, for a 7.3x
  worse interactive stall and half the KV pool.
- **Raising the budget is a bad trade.** The direction that helps interactive
  latency is *downward*, and it is priced: 1024 is modelled at ~8.6% prefill
  throughput for half the stall — a prediction, not a measurement.

The honest decision standing, which is stronger than "the bug accidentally chose the
right answer" and narrower than "2048 is optimal":

> Among the **measured** arms (2048, 8192, 16384), 2048 best preserves interactive
> progress while giving up only a small amount of bulk-prefill throughput relative
> to larger batches. The 1024 trade remains predicted and undecided, so 2048 is not
> established as globally optimal nor as the floor.

The product should eventually pin 2048 **explicitly** rather than depend on an
upstream defect to keep selecting a value that happens to suit the objective. That
needs no restart today — explicit 2048 was measured to be a no-op — it belongs in
the next authored launch.

**The externality mechanism, measured directly.** Chunked prefill runs as a mixed
batch, so a co-tenant decoder receives exactly one token per chunk, at the chunk's
duration:

    decode alone                              30.1 ms/token
    decode during a 16,381-token prefill    2168.2 ms/token     72x

That 72x, not any prefill number, is what the operator experiences as ~0.4 tok/s.

## 6. Runtime provenance: the optimised paths ARE selected

The review's leading concern resolves in favour of the optimised paths:

    Selected DeepGemmFp8BlockScaledMMKernel for Fp8LinearMethod
    DeepGEMM E8M0 enabled on current platform
    Setting kv cache block size to 256 for DEEPSEEK_SPARSE_SWA backend
    Using FP8 indexer cache for Lightning Indexer
    Autotuning DeepSeek V4 SM120 sparse MLA decode with FlashInfer  (30 configs)
    Using ['PYNCCL'] all-reduce for 'tp:0' out of
      ['NCCL_SYMM_MEM','QUICK_REDUCE','FLASHINFER','CUSTOM','SYMM_MEM','PYNCCL']
    Capturing CUDA graphs (mixed prefill-decode, PIECEWISE): 11/11
    quantization=deepseek_v4_fp8, kv_cache_dtype=fp8_ds_mla,
    mode=CompilationMode.VLLM_COMPILE, cudagraph_mode=FULL_AND_PIECEWISE

`CUSTOM` all-reduce losing to `PYNCCL` is correct: four separate machines over RoCE,
and custom all-reduce needs single-node P2P.

## 7. Corrections to the end-to-end review

- "behavioral capacity is at least 20,082 tokens" — refuted; 2048.
- "An explicit 8,192 or 2,048 value would not be a no-op" — inverted. Explicit 2048
  was applied and changed nothing, as predicted. 8192 is the real change, and it is
  the wrong direction.
- The proposed relief (cap at 8192/4096/2048 to cut victim stalls) is already at its
  floor, and every value it names is *larger* than the current one, so it would make
  victim stalls 2-7x worse.
- The sweep upward to 24k/32k/48k/64k/96k/128k/160k would have found nothing: the
  instrument it relies on is invariant.
- Candidates #1 and #4 are not two candidates and are now both closed.
- "It will not solve the ~1.055 ms/token term" — correct, and now measured: the term
  is invariant to batch size over 32x.

## 8. TP=4 against TP=2: the batch axis could not move the per-token term, and this does

Run 2026-09-07 on the same head, holding everything: same image and container, same
model snapshot and tokenizer, same executor, same backend selections, same fresh-nonce
cache standing, batch budget **pinned explicitly at 2048 in both arms**, MFU metrics on
in both, and the same context ceiling of 131,072 — lowered from the production
1,048,576 for BOTH arms, because a TP=2 rank holds roughly half of a ~168.7 GB model
against a ~99.8 GiB budget and the production ceiling is untested there.

    quantity                  TP=4         TP=2      TP=2 against TP=4
    marginal per token      1.053 us     0.786 us     25.4% cheaper
    prefill throughput       949 tok/s   1272 tok/s     34% higher
    decode alone             30.2 ms      40.5 ms       34% WORSE
    co-tenant stall         2146 ms      1629 ms        24% shorter
    KV pool at 131,072    6,175,555    2,310,685        63% LESS

The gain holds at every probe above 2,048 tokens across a 40x range (510 → 20,192
computed tokens) and is only 7% at 510, where the fixed per-request term still
dominates. Step counts from the per-step FLOPs counter matched `ceil(tokens/2048)` in
both arms, so the budget really was pinned and the comparison really is topology.

**Review candidate #3 is confirmed: TP=4 over-shards this prefill.** It is the first
mechanism found that moves the per-token term at all — the batch axis could not touch
it — and the largest single lever measured on this fleet.

**It is not a free win, and the two workloads want opposite topologies.** Decode is
34% worse at TP=2, which is what you would expect: decode is memory-bound per token
and fewer ranks means less aggregate bandwidth to stream weights, while prefill is
compute-and-collective-bound and pays for every extra participant.

### The topologies are not numerically interchangeable

A greedy text comparison would have been worthless here — the two arms diverge at the
first token (" The" against " the") and separate from there, which a near-tie plus
greedy decoding produces from an arbitrarily small logit difference.

So the measurement is **prompt logprobs over one fixed 22-token input**: a single
deterministic forward pass, no sampling, identical bytes in.

    same token sequence:  yes
    max |delta|:          1.517 nats   (a 4.56x ratio on one token's probability)
    mean |delta|:         0.330 nats

That is orders of magnitude above float reduction-order noise. Something about the
sharding changes the computation and not merely its rounding. The candidates are
**unseparated**: per-shard fp8 accumulation, routed-expert selection under a different
partition of the 256 experts, or a shape-dependent kernel choice. The carrier refuses
substitution rather than reporting a warning — a 25% speed win may not silently carry
a numerical change into production.

**Missing control, named rather than glossed:** no arm was repeated, so within-topology
run-to-run variation is unmeasured and the 25.4% carries no interval.

## 9. What is still open

1. **Whether TP=2 fits the production 1,048,576 ceiling at all** — untested, and the
   weights rather than the KV pool are the binding constraint.
2. **Whether the numerical divergence is a defect or an accepted consequence of
   resharding.** This blocks adoption on its own.
3. **Within-topology repeat runs**, to put an interval on the 25.4%.
4. **Ray vs mp executor** (review B4) — cheap A/B, everything else fixed.
5. **The operation ladder** (B6) — GEMM → grouped MoE → sparse attention → layer →
   model, to locate *where inside the step* the recovered 25% was being spent.
6. **Platform** — clocks, thermals, UVM behaviour, driver/firmware parity per rank.

`--enable-mfu-metrics` should be on for all of these: it is the per-step counter, and
it also carries estimated FLOPs and memory traffic per step.

## 9. Suggested carrier changes

- `extdeps.vllm.engine_args`: the branch's `VllmPrefillSchedulingPolicy` is the right
  shape and its withdrawal of the do-not-set row was right for the wrong reason —
  "reduces the KV pool" is TRUE and now measured (11.77M → 8.70M → 5.84M tokens),
  while "prefill is compute-bound so a larger batch buys no throughput" is nearly
  true (7.5% at 8x). Record that omitting the flag resolves to 2048 on a GB10 via
  the NVML gap, and that the effective value is readable from the banner.
- `gunbc.spark.serving_critical_path`: replace
  `BehaviouralPerIterationCapacityAtLeast{20082}` with the read value 2048, and
  carry the measured stall law `S + b*budget`.
- `extdeps.vllm.metrics`: `iteration_tokens_total_count` is not a step counter —
  that belongs beside the existing refusal. `estimated_flops_per_gpu_total` is one.
- `gunbc.recurring_failure_mode`: an upstream instance of the absorbing-fallback
  class — a capability probe that fails, is caught, and silently selects a
  conservative default that then governs production behaviour.

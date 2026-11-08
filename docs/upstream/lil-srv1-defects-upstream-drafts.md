# Upstream issue drafts — lil / engine defects observed on srv1 (LANE F, F24–F26)

**Status: DRAFTS ONLY — not opened.** Filing these is outward-facing. They are handed back to the
parent/operator to open. The substrate facts these cite live in
`extdeps.vllm.lil_serving_defects` (witness: `test.claim.extdeps.lil_serving_defects_witness`).

Common environment for all four (read on the host, not relayed):

- Host `srv1` (nvjet1), two RTX PRO 6000 Blackwell — SM120, **no GPU P2P** (no NVLink).
- Image `docker.iot-bots.li:5555/local-inference-lab:2026-11-03-19-54-02`, vLLM build string
  `0.1.dev22105+g5009fa56b` (present verbatim in each defect log).
- Model profile `qwen38-flash-next`, hardware profile `rtx-pro-6000-pcie`.
- Read 2026-11-08 from `~/lil/serve/serve.log` and from `/opt/lil/runtime/*.py` inside the running
  `lil-flash-next` container.

---

## 1) b12x — PCIe all-reduce init failure hangs TP=2 instead of falling back to NCCL (F24)

**Component:** `b12x` vLLM integration (`b12x/integration/vllm/b12x_pcie_all_reduce.py`,
`b12x/comm/pcie/pcie_oneshot.py`).

**Config that triggers it:** `tensor_parallel_size=2` on two RTX PRO 6000 (SM120) over PCIe, no GPU P2P.

**What we see.** Both tensor-parallel ranks log the same initialization failure and then TP=2 bring-up
hangs to the NCCL watchdog. Exact strings from the log:

```
(Worker pid=456) WARNING [b12x_pcie_all_reduce.py:298] B12X PCIe all-reduce initialization failed on rank 0: failed to exchange CUDA IPC import-open status; CUDA IPC ownership was retained
(Worker pid=457) WARNING [b12x_pcie_all_reduce.py:298] B12X PCIe all-reduce initialization failed on rank 1: failed to exchange CUDA IPC import-open status; CUDA IPC ownership was retained
... Custom allreduce is disabled because your platform lacks GPU P2P capability or P2P test failed.
... Using ['PYNCCL'] all-reduce backends ... out of potential backends: ['FLASHINFER_PCIE_IPC', 'B12X_PCIE', 'B12X_ROCENANTE', 'FLASHINFER', 'NCCL_SYMM_MEM', ...]
```

After the failure is logged, the surviving rank blocks on a **one-element NCCL broadcast inside b12x's
own failure path** and only the 600 s watchdog releases it:

```
[rank1]:[E1108 ... ProcessNCCL] Watchdog caught collective operation timeout:
WorkNCCL(SeqNum=37, OpType=BROADCAST, NumelIn=1, NumelOut=1, Timeout(ms)=600000)
ran for 600017 milliseconds on 2 ranks before timeout.
  File ".../b12x/comm/pcie/pcie_oneshot.py", line 653, in _exchange_setup_failures
    _broadcast_gather_object(...)
```

**Why it hangs (our reading).** The init failure is detected on both ranks, but the exchange of setup
failures uses a broadcast that assumes the peer will post. The rank that retained CUDA IPC ownership
never posts, so the peer's `NumelIn=1` broadcast blocks until the watchdog.

**Expected.** On init failure, abort the B12X PCIe path cleanly (free/drop the retained CUDA IPC
ownership) and let NCCL take the collective immediately — no peer-to-peer broadcast in the failure path
that can deadlock when one rank never posts.

**Related, in the local launcher (not the b12x repo).** The lil `rtx-pro-6000-pcie` hardware profile does
not set the off-switch: the launcher sets `VLLM_ENABLE_PCIE_ALLREDUCE=0` **only** when
`--disable-custom-all-reduce` is passed (`launcher.py:1239-1240`). On a host with no P2P the profile
should derive `VLLM_ENABLE_PCIE_ALLREDUCE=0` from the absence of P2P rather than depend on the operator
passing the flag. We can file this against local-inference-lab if you would rather it not live here.

---

## 2) local-inference-lab / vLLM — NVFP4 main KV cache unsupported for qwen38-flash-next (F25)

**Component:** lil launcher allow-list (`/opt/lil/runtime/launcher.py`) and the vLLM QSA main-KV builder.

**Config that triggers it:** `kv_cache_dtype=nvfp4` on profile `qwen38-flash-next`.

**What we see.** Rejected at launch by the profile:

```
Launch configuration error: qwen38-flash-next supports target KV settings ['fp8', 'fp8_e4m3']; other precisions require separate qualification
```

Pushed past the allow-list, the engine core raises:

```
NotImplementedError: QSA requires BF16 or FP8 E4M3 main KV cache
  (raised through vllm/v1/executor/multiproc_executor.py)
```

**The hard-code.** `launcher.py:2011` is `supported_kv = {"fp8", "fp8_e4m3"}` (a separate branch adds
`nvfp4_ds_mla` for the DS-MLA shape only, not for this model), and `launcher.py:2019` renders the message
above.

**Expected / ask.** Either qualify an NVFP4 main KV cache for the QSA path (it currently builds only BF16
or FP8 E4M3), or document FP8 E4M3 as the **FP floor** for `qwen38-flash-next` and surface that at the
profile with the reason, instead of a raw `NotImplementedError` from the engine.

---

## 3) LMCache — second replica crashes at recurrent-checkpoint bind (F26a)

**Component:** LMCache vLLM integration — `lmcache/integration/vllm/recurrent_checkpoint_connector.py`,
`checkpoint_copy.py`, `lmcache/v1/multiprocess/transfer_context/shm.py`.

**Config that triggers it:** lil `replicas=2` (one container, two single-GPU replica servers behind the
proxy), LMCache `ram-disk`, recurrent checkpoint connector enabled.

**What we see.** The first replica's engine comes up; the second dies at engine-core init:

```
[replica 1] (EngineCore pid=...) ERROR multiproc_executor.py:1073 Error in code: ...
Traceback:
  .../lmcache/integration/vllm/recurrent_checkpoint_connector.py:253  bind_boundary_checkpoint_state
  .../lmcache/integration/vllm/checkpoint_copy.py:59                   transfer.checkpoint_slot_views(...)
  .../lmcache/v1/multiprocess/transfer_context/shm.py:227             raise RuntimeError(
RuntimeError: Checkpoint DMA requires a registered SHM mapping
```

**Why (our reading).** In `replicas>1` bring-up the second engine core reaches
`bind_boundary_checkpoint_state` before its shared-memory checkpoint region is registered, so
`checkpoint_slot_views` raises and the whole replica fails to init.

**Expected / ask.** Register the SHM mapping per replica before binding checkpoint state, or fall back to
a copy path (or disable checkpoint DMA) when no mapping is registered, rather than raising and taking the
replica down. This is the blocker for lil's built-in `REPLICAS=2` mode.

---

## 4) local-inference-lab — replica proxy dials replicas over plain HTTP (F26b)

**Component:** lil `runtime/replica_proxy.py` and `runtime/replicas.py`.

**Config that triggers it:** lil `replicas=2` behind the replica proxy, with the replicas launched
`--ssl-keyfile/--ssl-certfile` (the replicas inherit these TLS flags).

**What we see.** With `replicas=2` the proxy is started as (from `replicas.py:122`):

```
[lil-serve] Proxying 0.0.0.0:18000 to replicas 127.0.0.1:36097, 127.0.0.1:49879
```

`replica_proxy.py` `main()` accepts **only** `--host`, `--port`, `--replica HOST:PORT` — there is no TLS
argument — and `build_app` hard-codes the upstream scheme:

```
# replica_proxy.py:297
bases = [f"http://{upstream}" for upstream in upstreams]
```

So when the replicas serve TLS (they inherit `--ssl-keyfile/--ssl-certfile`), the proxy speaks plaintext
HTTP to a TLS listener. Every forwarded request hits the proxy's client-error branch and returns:

```
{"error": {"message": "replica 0 unavailable: Server disconnected without sending a response.", "type": "proxy_error", "code": 502}}
```

which the client sees as **`Server disconnected`**.

**Expected / ask.** Derive the upstream scheme from whether the replicas were given TLS — thread
`--ssl-keyfile/--ssl-certfile` (or a `--replica-scheme` / `--replica-tls` flag) from `replicas.py` into
`replica_proxy.py` — or serve the replicas on loopback without TLS and keep TLS only on the public proxy
listener. As written, `REPLICAS=2` + a TLS model server cannot serve a single request.

---

### If `REPLICAS=2` were fixed (context for 3 and 4)

lil `REPLICAS=2` is attractive because two replicas under one launcher share **one** host-RAM PLE table
instead of one each (`launcher.py` sets `VLLM_PLE_TABLE_MEMORY=shared` when supported); one replica's PLE
table is ~26.8 GiB on srv1. But this image **cannot** share the PLE table: the launcher only does so when
the installed vLLM carries `VLLM_PLE_SHARED_TABLE_DIR`, which this build lacks, so it sets
`ple_copy_per_replica = True` and warns *"every replica keeps its own host-RAM copy (about 27 GiB each)."*
The 26.8 GiB saving therefore needs **both** the F26a fix **and** shared-PLE-table support; fixing the
checkpoint-DMA crash alone does not recover it.

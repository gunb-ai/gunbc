# Qwen3.8-27B NVFP4 benchmark kit (NInfer vs vLLM)

Rent a dedicated card (vast.ai, *not* a time-sliced pod), Ubuntu with CUDA 13.1, ≥150 GB disk, ≥64 GB RAM.

    scp -P PORT -r ~/gpu-bench root@HOST:/workspace/
    ssh -p PORT root@HOST; cd /workspace/gpu-bench; tmux

## NInfer
    ./setup_ninfer.sh                     # ~10 min build + 24 GB download
    ./serve_ninfer.sh 1300000 8 nvfp4     # in tmux window 1; lower capacity by 50000 if it OOMs
    ./run.sh ninfer_nvfp4                 # in window 2 (short N=1,4,8; warm N=1,4,8)

## vLLM (stop NInfer first; one engine per GPU)
    pip install vllm==0.30.*
    ./serve_vllm.sh fp8_e4m3              # wait for "Application startup complete"
    ./run.sh vllm_fp8 1,8,16,32 1,4,8,14

Results: `results_<label>.txt` (+ raw jsonl). Copy them back, then **destroy the instance**.

## Reference (RTX PRO 6000 96 GB, vast, 2026-10-10)
| | short N=8 | warm 8×174k agg | TTFT |
|---|---|---|---|
| NInfer NVFP4 KV | 774 | 204 | 5.2 s |
| vLLM 4-bit KV   | 418 | 120 | 14.0 s |
RTX 5090 (NInfer NVFP4, 480k KV): 104 tok/s @1, 346 @4.

Plan on the **median**, not p90. Check NInfer's log for `replay` vs `snapshot` counts —
replays mean the KV pool + host pool is too small for the sessions you keep.

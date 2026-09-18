# Architecture roster read from the serving image

Verbatim stdout of `ModelRegistry.get_supported_archs()`, sorted, taken inside the image that
Group B actually serves.

- image tag: `gunbc-vllm-glm53-gb10:derived-c99be081f219d123`
- image config id: `sha256:c7d63d153f991b9f1bd7e6419dcd1a5109746670747be90f4ff6da74f0e8fe84`
- `RepoDigests`: empty — built locally, never pushed, so the config id is the only identity available
- host: 192.168.1.236 (spark-0c75), a live Group B rank
- read: 2026-09-18
- vLLM `0.1.dev1+g8d09804c8`, torch `2.13.0+cu130`
- count: 378
- sha256 over the sorted names joined by `\n`: `2e43451e12bd3e096af99b5c4658c428c5eadf857c644cfa67fdadb7b67802e1`

## The producer, named rather than described

```
docker run --rm --entrypoint python3 gunbc-vllm-glm53-gb10:derived-c99be081f219d123 -c \
  'from vllm.model_executor.models.registry import ModelRegistry as M
   for a in sorted(M.get_supported_archs()): print(a)'
```

## What this establishes and what it does not

It establishes which architectures this build REGISTERS. It does not establish that any of them
loads, that its kernels exist for SM 121, or that a checkpoint served through it — no checkpoint was
loaded and no kernel was executed. In particular `GlmMoeDsaForCausalLM` appearing here is a
registration fact and is not evidence about the NVFP4 W4A4 path, whose own probe
(`torch.ops._C.cutlass_scaled_mm_supports_fp4`) is absent from this build's op namespace and is
therefore UNREAD rather than negative.

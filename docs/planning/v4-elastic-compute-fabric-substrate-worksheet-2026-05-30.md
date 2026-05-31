# [SUPERSEDED] Combined compute + cache worksheet

> **This file is withdrawn.** Exploration §4.0f on **main** (PR #4091) requires **two** Modeling DFS worksheets. Prior §8 on the combined doc is **revoked**.

| Worksheet | Path | Cases |
|-----------|------|-------|
| **A — Compute fabric** | [`v4-elastic-compute-fabric-worksheet-2026-05-30.md`](v4-elastic-compute-fabric-worksheet-2026-05-30.md) | 1–8 |
| **B — Cache interface** | [`v4-elastic-cache-interface-worksheet-2026-05-30.md`](v4-elastic-cache-interface-worksheet-2026-05-30.md) | 9–19 |

Compose only via `ExecutionReceipt<T>.output` and `CachedArtifactReceipt<T>.producer: ProducerReceipt<T>` (internal path uses `ExecutionReceiptRef<T>`) — no cross-imports between substrate modules.

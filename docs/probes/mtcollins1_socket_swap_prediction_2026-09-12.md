# Mt. Collins socket-swap prediction, 2026-09-12

**Question:** move the same 2DRx4 quartet from socket 1 to socket 0 (J1/J3/J5/J7), filling socket 1's odd connectors with originals and retaining eight modules per socket. The preceding socket-1 round restarted after 4m50s with `0fde703b1102`.

**Prediction, committed before intervention:** failure, with byte 3 changing from `3b` to `33`. These are separately falsifiable claims. The candidate schema was stage = byte>>4, socket = (byte>>3)&1, status = byte&7; only the socket hypothesis had experimental support.

**Original alternatives:** `33` was predicted to follow the quartet; `3b` to track location; training would show an interaction; another payload required stopping interpretation. The [outcome](mtcollins1_socket_swap_outcome_2026-09-12.md) corrects this table's overclaim: companion populations changed too, and a configuration restriction can follow the quartet exactly as a defect would. No result isolates one module or establishes a physical fault. Supported shape had not been proved necessary, and no payload-to-DIMM mapping existed.

[Original dated account](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/docs/probes/mtcollins1_socket_swap_prediction_2026-09-12.md). Raw artifacts and modeled evidence are unchanged.

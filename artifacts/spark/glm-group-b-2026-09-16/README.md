# Group B native GLM captures, 2026-09-16

Raw bytes captured from the four-rank native GLM-5.3-Flash arm on srv9/10/11/12
(spark-0c75/3336/b66c/2196) around the transport change of 2026-09-16, for use as
observation fixtures. Nothing here is authored: every file is the verbatim output of
the named command.

Provenance, per file:

| file | command | host | incarnation |
|---|---|---|---|
| `inspect-socket-rank0.json` | `docker inspect glm-native-tp4-socket` | 192.168.1.236 | the incarnation that ran 2026-09-13T06:47 .. 2026-09-16, `Devices=[] CapAdd=null Ulimits=[]`, `--max-num-seqs 1`, `--gpu-memory-utilization 0.84` |
| `inspect-socket-rank1.json` | same | 192.168.1.237 | same |
| `inspect-roce-rank0.json` | `docker inspect glm-native-tp4` | 192.168.1.236 | the incarnation created 2026-09-16, device/IPC_LOCK/memlock present, `--max-num-seqs 16`, `--gpu-memory-utilization 0.8567` |
| `inspect-roce-rank1.json` | same | 192.168.1.237 | same |
| `nccl-socket-rank0.txt` | `docker logs glm-native-tp4-socket \| grep -aE 'NCCL INFO (NET/\|Using network\|Channel 0[0-3]/0 .*via NET)'` | 192.168.1.236 | socket incarnation |
| `nccl-roce-rank0.txt` | `docker logs glm-native-tp4 \| grep` (same filter) | 192.168.1.236 | RoCE incarnation |

RANK ORDER IS NOT ADDRESS ORDER on this group: .236=rank0, .237=rank1, .239=rank2,
.238=rank3. Read the rank from the captured argv, never from the host list.

THE DISCRIMINATING PAIR. The socket capture carries `NET/IB : No device found.` followed
by `NET/Socket : Using [0]enp1s0f1np1`; the RoCE capture carries
`NET/IB: [1] rocep1s0f1:uverbs1:1/RoCE provider=Mlx5 speed=50000` followed by
`NET/IB : Using [0]rocep1s0f1:1/RoCE`. A parser that cannot tell these two apart, or a
readback type that cannot see the device/ulimit difference between the two inspect files,
has not modelled the defect these were captured to make detectable.

The grep filter is recorded above because these are FILTERED captures, not whole logs: a
consumer must not read them as the complete output of the command.

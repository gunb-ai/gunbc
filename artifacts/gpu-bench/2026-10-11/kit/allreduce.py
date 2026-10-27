# usage: torchrun --nproc_per_node=2 allreduce.py  — NCCL all-reduce latency/bandwidth across message sizes (bf16)
import os,time,torch,torch.distributed as dist
dist.init_process_group("nccl"); r=dist.get_rank(); torch.cuda.set_device(r)
if r==0: print(f"{'bytes':>12} {'lat_us':>9} {'busbw_GB/s':>11}")
for p in range(12,29,2):
    n=2**p; x=torch.ones(n//2,dtype=torch.bfloat16,device="cuda")
    it=200 if n<2**22 else 30
    for _ in range(10): dist.all_reduce(x)
    torch.cuda.synchronize(); t=time.perf_counter()
    for _ in range(it): dist.all_reduce(x)
    torch.cuda.synchronize(); dt=(time.perf_counter()-t)/it
    if r==0: print(f"{n:>12} {dt*1e6:>9.1f} {n/dt/1e9:>11.2f}")  # busbw for 2 ranks = algbw*(2*(2-1)/2)
dist.destroy_process_group()

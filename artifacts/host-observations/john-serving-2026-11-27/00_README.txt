host-ba61f9 (john, Supermicro H11SSL unit 4) STRATA SERVING SETUP AND ITS EVIDENCE -- 2026-11-26/27

WHAT THIS IS
  The files behind gunbc.serving.john_strata_bench_receipt and the operator's de facto setup for john
  (gunbc.design_ruling john_de_facto_serving_setup_2026_11_27; the operator's words, 2026-11-27: "could you
  put this in the repo, we can use this as the de facto setup for that machine for now" and "4 slots is
  fine"). Every file is what john or srv2 printed or wrote; nothing in this directory is edited except
  that the Strata configs had their api_key value replaced (*.redacted). SHA256SUMS pins every other file
  here. The earlier hand-install evidence for the same machine is ../john-hand-install-2026-11-26/;
  john-hand-steps.log here is the same step log continued through 05:35Z on 2026-11-27 (its first 14 lines
  are that directory's copy).

THE SETUP AS IT STOOD ~05:30Z 2026-11-27
  GPUs, nvidia-smi PCI order (bus ids read by thermal/ab-20261127T034432Z/run.log; no power-off happened
  between that read and 05:31Z -- fans-ab-*.csv is continuous with no 'No Reading' gap):
    0  Tesla V100-SXM2-16GB  00000000:01:00.0  BLOWER cooler on an SXM2-to-PCIe adapter (operator report)  150 W
    1  Tesla P100-PCIE-16GB  00000000:81:00.0  passive (operator report)                                    125 W
    2  Tesla PG500-216       00000000:C1:00.0  passive; the operator's V100 32 GB (10de:1df0)              100 W
  Each limit is that card's own power.min_limit, set and read back by gpu-power-cap.sh, run by
  gpu-power-cap.service (both here). The ranges (min/default/max) are in thermal/run-20261127T015819Z/
  limits.txt, read with V100-32 and P100 in the other two slots; the ranges are per card, not per slot.
  Driver nvidia-headless-580-server 580.178.04 (proprietary: the open modules do not support Pascal/Maxwell);
  CUDA 12.9 toolkit for the build (CUDA 13 dropped sm_60/sm_70).
  Strata 8c5fafc1 built by producers/strata_build_sm60.sh (STRATA_EXPERIMENTAL_SM60=ON,
  CMAKE_CUDA_ARCHITECTURES=60;70, ken's other STRATA_* options). john-strata.service (here) runs
  serve/server.py --engine strata --config cfg-john3.json on 0.0.0.0:18081, Requires=gpu-power-cap.service,
  MemoryMax=100G. cfg-john3.json.redacted: gpu [0,1,2], parallel 4, layer split auto (the engine chose
  K=19,22 on this layout: strata3-20261127T040802Z/run.log 'split:' line), Qwen3.8-Flash-Next GSQ-RCO IQ3_S
  (the same pack and GGUF as ken). The key file is ~/strata-pool/api_key on john (0600) and is NOT here; an
  unauthenticated request answers 401. john has no row in the srv1 router.
  Fans: BMC fan mode Heavy IO (raw 0x30 0x45 0x01 0x04), chassis zone 1 (FANA/FANB) duty 0x64 (100%),
  CPU zone 0 left to the BMC. chassis-fan-guard.service runs chassis-fan-guard.sh (both here), in band
  over /dev/ipmi0: it re-asserts zone 1 at 0x64 every 10 s and journals each correction. Before it was
  installed on john the same loop ran from srv2 (fan_guard.log here is that loop's correction log).

THE RUNS (all lid on; each llm-logs-results/<run>/gpu.csv is a 1 s nvidia-smi log with columns
  timestamp, index, name, temperature.gpu, power.draw, power.limit, clocks.sm, clocks.mem, utilization.gpu,
  memory.used, clocks_event_reasons.active, then .sw_power_cap, .sw_thermal_slowdown, .hw_slowdown,
  .hw_thermal_slowdown as Active/Not Active; the thermal/ab gpu.csv has a 16th column fan.speed)
  run (dir)                         cards serving (layout)                       fan zone 1 in the measured window
  strata3-20261127T040802Z  DE FACTO V100-16 @01 + P100 @81 + V100-32 @c1, auto  BMC log: 44/44 samples 0x64, guard on
  agg-20261127T042225Z      DE FACTO same, parallel 4, concurrency sweep        BMC log: 75/75 samples 0x64, guard on
  agg-p8-20261127T043648Z   same cards, parallel 8 (cfg-john3-p8)              BMC log: 61/61 samples 0x64, guard on
  strata3-20261127T031708Z  V100-16 @81 + V100-32 @c1 (P100 out), auto          BMC log: 21/21 samples 0x64
  strata3-20261127T035851Z  P100 @81 + V100-32 @c1, V100-16 idle (passive S1)   BMC log: 54 samples, 0x39..0x64 (BMC curve)
  strata3-20261127T021726Z  V100-16 @01 + V100-32 @81 + P100 @c1, auto          NOT LOGGED (see FAN STANDING)
  strata3-20261127T025302Z  same, layer split forced 10,32 (cfg-john3-k10-32)   NOT LOGGED
  thermal/run-20261127T015819Z  fp16 burn, all three, 600 s, layout as 021726Z  NOT LOGGED
  thermal/ab-20261127T034432Z   fp16 burn phase A (passive cards only), stopped  BMC log: 79 samples, 0x2f..0x64 (BMC curve)
                                by the operator after ~6.5 min; phase B never ran
  llama.cpp 3cf03257f, M40 + P100 (warm150k-20261127T011141Z-lidon): cold 150k prefill NOT completed after
  31 min (124928 tokens processed); the evidence is the 01:42:54Z RESULT line of john-hand-steps.log, and
  the run's .err is here. Its gpu.csv and jsonl (empty) are not committed: no row cites a figure from them.
  The 'measured window' is warm150k start -> done in each run.log, or the first -> last gpu.csv sample
  of a sweep. The fan counts are re-derived from fans-ab-*.csv and fans-strata3-*.csv (srv2's BMC poll,
  one line per ~10 s: time, zone0 duty, zone1 duty, fan RPMs, BMC temperatures) by
    cat fans-ab-20261127T034432Z.csv fans-strata3-20261127T031708Z.csv | sort -u |
      awk -F, -v a=FROM -v b=TO '$1>=a && $1<=b {n++; split($3,z,"="); d[z[2]]++} END{print n; for(k in d) print k, d[k]}'

  PER-CARD REDUCTIONS (temperature max and the number of samples each clocks_event_reasons column read
  Active) are re-derived from each gpu.csv by
    awk 'BEGIN{FS=", "} {i=$2; n[i]++; nm[i]=$3; if($4+0>t[i]) t[i]=$4+0; if($12=="Active") pc[i]++;
      if($13=="Active") st[i]++; if($14=="Active") hs[i]++; if($15=="Active") ht[i]++}
      END{for(i in n) print i, nm[i], n[i], t[i], pc[i]+0, st[i]+0, hs[i]+0, ht[i]+0}' gpu.csv
  over the WHOLE file (idle samples before the load included).

PRODUCERS
  producers/ holds the shell drivers as they stood on srv2 when this directory was assembled:
  strata3_run.sh (server start, smoke 17*23, warm150k, 1 s GPU log), serving_ab.sh, strata_build_sm60.sh,
  thermal_run3.sh, thermal_ab.sh. They were edited during the night (the step log names each fix);
  whether john's copy at each run's start was byte-identical to these is NOT checked.
  The Python producers are NOT in this directory: the repository's .gitignore excludes every *.py
  (gunbc.repo_workspace), so they are identified by digest (read 2026-11-27 on srv2):
    strata_bench.py   272 lines  sha256 36f3e6b4ab23ba385cfa791063b4e8bbdeb6ace2c1342d63c37858b63f011aaa
                      (the same digest as the producer of ../h11-r9700-2026-11-12 and of srv1's srv1-agg
                      runs); run as --mode concurrency --conc 1 2 4 8 16 (p8: 4 8 16) --out 1024 --repeats 1:
                      84-token prompts, 1024 tokens out per request, one repeat.
    warm150k_john.py   97 lines  sha256 5691063058a8e72168d9e8ae4f8dd208536a4d6b144a432d3605c6457da93886
                      srv1:~/lil/warm150k.py adapted: plain HTTP, port from env, llama-server /tokenize
                      shape, TPW env override (Strata has no /tokenize; TPW=1.2488 measured with
                      llama-tokenize), TURNS/DELTA successive turns. Shape: a cold ~150k-token prompt, then
                      the warm turn (512 tokens forced out), then turns 3-6 each adding a 2000-token tool
                      delta. john-bringup copy; it was edited between runs (the 01:11:26Z .err is a
                      SyntaxError in an earlier edit).
    burnin.py          28 lines  sha256 185aef36f868a93f42a985817e298ff1756ee3e6c5bba5fa371f7d2bdf3cef9e
                      fp16 matmul burn with a checksum.

FAN STANDING -- READ THIS BEFORE ANY THERMAL ROW (THERMAL-NOTES.txt is the authoritative caveat list)
  Zone 1 was set to 0x64 by hand at 23:09:51Z, 01:52:36Z (after the card-swap power cycle), 03:13Z, ~03:44Z
  (after the second card change) and 04:08:22Z, when the guard took over.
  The BMC does NOT hold that write. fans-ab-20261127T034432Z.csv shows it: zone 1 was written 0x64 at
  ~03:44:07Z and read 0x64 until 03:46:23Z; at 03:46:26Z the BMC replaced it with 0x2f -- 54 s after burn
  phase A had started -- then ramped it back up step by step to 0x64 by 03:47:59Z as the cards heated;
  after the burn stopped (03:52:31Z) it stepped down to 0x39 by 03:58:31Z, and rose to 0x64 again by
  04:01:22Z under the S1 serving load (load start 04:00:03Z). No power cycle and no host write happened in
  that span: the BMC runs a temperature-following curve in Heavy IO mode that rewrites the zone.
  fan_guard.log records six corrections from 0x37..0x43 in the guard's first seven minutes. The duty also
  resets on every power-off. So no run before 04:08:22Z has a held zone; runs 021726Z, 025302Z and the all-three burn
  015819Z have NO fan log at all and PROBABLY ran at 100% under load (the curve raises it under load),
  which is not proven.
  THE 03:13Z 'BMC FAILURE' WAS NOT ONE. At 03:13Z all BMC sensors read 'No Reading' and both zones read
  0x32; the session took that for a dead BMC and issued mc reset cold. The host was POWERED OFF (the
  operator was changing cards; boot 03:12:41Z): silent sensors and the 0x32 duty are this board's normal
  state with the host off or in POST. The cold reset was unnecessary. THERMAL-NOTES.txt carries the
  original note and its 03:23:09Z correction; both are kept.

WHAT THE ROWS DO NOT SAY
  - n = 1 per arm. No row is a distribution; the run-to-run spread on this host is unmeasured.
  - The passive V100-32 cooled far better in slot c1:00 (de facto: 75 C, no thermal slowdown) than in
    81:00 (021726Z: 84 C, sw_thermal_slowdown on 130 of 468 samples). That comparison is CONFOUNDED with
    fan state: the c1:00 runs had zone 1 logged at 100%, the 81:00 runs had no fan log. It is an inference
    about the slot, not a measurement of it.
  - The passive-only S1 run (035851Z) decoded turns 3-6 faster (66-75 tok/s) than the de facto three-card
    run (51-54), on one run each, with its fans on the BMC curve and its P100 in thermal slowdown on 44 of
    340 samples. Nothing here says which layout serves better; the operator chose the three-card one.
  - parallel 8 at concurrency 8 (95.19 tok/s) out-produced every parallel-4 point (best 87.31 at c4). The
    4-slot choice is the operator's; no row derives it from the sweep.
  - Every concurrency row reads gpu_throttle_reasons_seen 0x4 (sw power cap), as expected at the cards'
    minimum limits; power.draw samples above power.limit occur (e.g. V100-16 up to 237.54 W against 150 W
    in 040802Z). nvidia-smi's power.draw definition for these cards was not read, so no claim is made
    about whether the cap bounds peaks.
  - The srv1-agg reference shape (the same strata_bench.py concurrency mode) has no receipt in this
    repository, so no comparison with srv1 is carried.
  - No GPU UUID or board serial was read: the cards have no gunbc.fleet.gpu_device_observation rows yet,
    and 10de:1df0 (PG500-216) has no extdeps.gpu board catalog row.
  - john answers at 192.168.1.240 on DHCP. test.claim.spark.fabric_switch_read_witness's header names
    gunbc-fabric-switch at 192.168.1.240 from a 2026-09-17 capture. Possible address conflict; not resolved.

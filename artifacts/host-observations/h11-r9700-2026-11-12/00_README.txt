host-6ce85c (h11-a16) ONE GIGABYTE AI TOP RADEON AI PRO R9700 — 2026-11-12

WHAT THIS IS
  The captures behind gunbc.fleet.gpu_device_observation's host-6ce85c row and
  gunbc.serving.h11_r9700_strata_bench_receipt. Every file is what the host printed or wrote; nothing here is
  edited. SHA256SUMS pins every other file in this directory.

THE DEVICE READING (read_r9700_device.sh -> r9700_device_reading.txt, 2026-11-12T16:31Z, after the 210 W cap
was written by hand to power1_cap earlier that day)
  read_r9700_device.sh is read-only: lspci, cat of sysfs/hwmon attributes, `sudo -n lspci -vv` for the link
  capability list. It writes nothing.

THE BENCHMARK RUNS (strata_bench.py --mode multiwarm, 4 agents, 32000-token context, run by the operator's
  session on h11 on 2026-11-12; the exact command line was not recorded)
  THE BENCH SCRIPT IS NOT IN THIS DIRECTORY. The repository's generated .gitignore excludes every *.py
  (gunbc.repo_workspace), so the producer is identified by digest instead: h11-a16:~/strata-test/strata_bench.py,
  272 lines, sha256 36f3e6b4ab23ba385cfa791063b4e8bbdeb6ace2c1342d63c37858b63f011aaa (read 2026-11-12). Its
  copy on h11 is the only one; if it changes or is deleted, these rows lose their reproducible producer.
  file stem               engine log                cap in force           power sampled
  r9700-20261112          strata-hip-serve.log      not recorded           no
  r9700-convcache         strata-hip-serve2.log     not recorded           no
  r9700-210w              strata-hip-serve4.log     210 W (operator)       r9700-210w.power.csv
  r9700-300w-measured     strata-hip-serve4.log     300 W (operator)       r9700-300w-measured.power.csv
  The engine config of every run except r9700-convcache is strata-iq3_s.json. r9700-convcache ran with the
  engine's conversation cache on; that config was reverted afterwards and its exact flags were not kept.

  strata_bench.err.log is the stderr of all four runs (byte-identical): the bench's own GPU sampler calls
  nvidia-smi and crashed on this AMD-only host, so the rows carry no GPU telemetry. The power.csv files come
  from pwr_sampler.sh, run beside the bench: one line every 2 s of
    unix_seconds, power1_average (uW, hwmon label PPT), gpu_busy_percent, temp1 edge (mC), temp2 junction (mC),
    temp3 mem (mC), mem_info_vram_used (bytes)
  Note temp2 is the JUNCTION sensor, not memory (hwmon temp2_label = junction, temp3_label = mem).

WHAT THE ROWS DO NOT SAY
  cached_med is 0 on every row: no follow-up turn reused a cached prefix, so turns 2 and 3 are full
  re-prefills of ~31.5k and ~33.9k tokens, not warm turns.

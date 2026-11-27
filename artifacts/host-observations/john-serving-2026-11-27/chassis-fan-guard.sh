#!/bin/bash
# john: hold Supermicro H11 chassis fan zone 1 (FANA/FANB) at 100% in Heavy IO mode, CPU zone 0 on BMC auto.
# The BMC runs a temperature-following curve that lowers zone 1 whenever GPU load drops (measured 2026-11-27), so a
# one-shot set does not hold: re-assert every 10 s, logging each correction to the journal. In-band via /dev/ipmi0.
set -u
until ipmitool raw 0x30 0x45 0x00 >/dev/null 2>&1; do sleep 5; done
mode=$(ipmitool raw 0x30 0x45 0x00 | tr -d ' ')
[ "$mode" = "04" ] || { echo "fan mode was 0x$mode, setting Heavy IO (0x04)"; ipmitool raw 0x30 0x45 0x01 0x04 >/dev/null; sleep 3; }
while true; do
  z=$(ipmitool raw 0x30 0x70 0x66 0x00 0x01 2>/dev/null | tr -d ' ')
  if [ -n "$z" ] && [ "$z" != "64" ]; then echo "zone1 was 0x$z, re-set 0x64"; ipmitool raw 0x30 0x70 0x66 0x01 0x01 0x64 >/dev/null; fi
  sleep 10
done

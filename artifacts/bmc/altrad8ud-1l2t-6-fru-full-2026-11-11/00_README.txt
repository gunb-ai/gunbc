# ALTRAD8UD-1L2T full in-band FRU print (PXE 1d-a)

Host: butch (srv14), 192.168.1.205 — read on the unit itself, in-band over /dev/ipmi0.
Date/time: 2026-11-11 13:55 UTC
Command: sudo ipmitool fru print 0   (run as ubuntu, via sudo; no BMC credential, no -U/-I lanplus)
Scope: read-only; nothing on the box was changed.

The print is the full output for device 0; this board's FRU carries only the four fields
shown (board mfg date/mfr/product/serial and product name). Digest in SHA256SUMS.

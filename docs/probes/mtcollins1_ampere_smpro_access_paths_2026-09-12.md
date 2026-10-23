# Pre-OS observation paths on Ampere Altra / Mt. Collins — what is documented

Written because the operator will have ~20 of these units and the BMC-only paths we tried
tonight were all dead ends. Every row states whether it is DOCUMENTED or INFERRED, and against
which source. Nothing here has been executed: the BMC has been off the network (ARP FAILED on
both 192.168.1.228 and 192.168.1.231) since ~22:15Z.

## The OEM SEL records are Ampere's

Manufacturer ID `0x00CD3A` = 52538 = **Ampere Computing**. Two independent sources: the IANA
Private Enterprise Number registry, and Ampere's own *Altra Family SoC BMC Interface
Specification v1.43*, Table 3, which states the register bytes as `3A CD 00`.

The specification does NOT define the IPMI SEL record format — it defines the SMpro/PMpro I2C
register map. The published byte layout comes instead from Ampere's OpenBMC emitters, which
are the producers of these records on Ampere reference platforms:
`openbmc/meta-ampere/meta-common/recipes-ampere/host/ac01-boot-progress/dimm_train_fail_log.sh`
and `ampere-openbmc/ampere-misc` `altra-host-error-monitor`.

The documented DDR-training byte is `(channel << 4) | type | (socket << 3)`. Under it our
`0x33` reads channel 3 / socket 0 and `0x3b` reads channel 3 / socket 1.

**This is a third independent support for the socket bit**, and it is the strongest of the
three because it is vendor source rather than our own experiment. The first was two
declared-in-advance crossovers; the second was same-second co-emission of the `ff10` pair.

The tail remains UNDECODED and may not be read as a syndrome: our records carry sensor number
`0xDE` where the documented training record uses `0xEB`, and the low nibble of byte 3 is 3
where the documented value is 4 (`BOOT_SYNDROME_DATA`). Two structural mismatches against the
only layout that would license the reading. **This still cannot name a DIMM.**

## The controller cannot decode what the vendor documents

Two different facts, and conflating them is a modelling error. The MegaRAC returns
`advanced_event_description: "unknown"` — a fact about the CONTROLLER, belonging to
`extdeps.bmc.megarac`. The format being externally documented by the producing vendor is a
fact about AMPERE, belonging in an Ampere authority. An absence row in the BMC module must not
be read as "these records are undecodable".

## Register path — the correction that matters

`ipmitool raw 0x3c 0x17` (`cmdScpRead`) was the first recommendation and it is **probably
wrong for this box**. Its Ampere implementation is a `seekg(offset)` against
`/sys/bus/platform/devices/smpro-misc.2.auto/reg`, a vendor-kernel sysfs attribute. It
requires OpenBMC with `smpro-misc` bound. On AMI MegaRAC, opcode `0x3c 0x17` will most likely
hit AMI's own *Get Hardware ID* instead — which would return plausible bytes that mean
something else entirely, the worst possible failure shape.

The realistic path on MegaRAC is the STANDARD IPMI **Master Write-Read** command, `0x06 0x52`,
against the SMpro I2C endpoint. From `aspeed-bmc-ampere-mtjade.dts`: `&i2c2 { smpro@4f …
smpro@4e … }` — so **bus 2, address 0x4F = socket 0 and 0x4E = socket 1**. That is the one
confirmed bus/address pair, and `0x06 0x52` is a standard command rather than a vendor one.

Registers of interest (spec v1.43 Table 16, matching Linux `drivers/misc/smpro-errmon.c`):

- `0xB0` boot stage + status. byte0 status: 0 not started, 1 started, 2 completed OK,
  **3 failed**, 0xFF unsupported. byte1 stage: **3 = DDR initialization, 4 = DDR training
  report status**, 8 UEFI, 9 OS.
- `0xB1` failure bits **and a per-DIMM bitmap**. bit0 generic, bit1 configuration, **bit2
  training failure**, bit3 ECC-init, bit4 no DIMM plugged; bitmap covers MCU0 slot0 … MCU7
  slot1.
- `0xB4` write slot ID to select; `0xB5` read that slot's training syndrome.

**`0xB0` history walking is DESTRUCTIVE.** Reading the stage history requires writing back
`{byte0 = stage just read, byte1 = 0x01}` and re-reading, which advances and consumes the
data. So the register sweep goes LAST, after every non-destructive read is banked.

Syndrome decode, documented and matching spec register 0xB5: bits[1:0] type (1 = PHY training
failure, 2 = DIMM training failure), bits[4:2] physical rank, bits[7:5] syndrome0, bits[15:8]
syndrome1. Byte order is a trap — the spec's byte-swap exemption list names 0xB0/0xB1/0xB3 but
NOT 0xB5, so the syndrome's order must be established against a known-good slot before any
reading is trusted.

## Console routing — the sources disagree, so probe both

Ampere's Altra datasheet Table 5 and the HW Design Spec Table 19 both map the SCP console to
S0_UART1. **Mt. Jade's own OCP Spec v1.0 Table 9 disagrees**, stating:
BMC_UART1 → S0_UART0 (system console, UEFI/OS); BMC_UART2 → **S0_UART3** (SCP FW console);
BMC_UART3 → S0_UART4 (ATF FW console); BMC_UART4 → S1_UART3 (SCP FW console).

A board-specific spec beats a family reference for that board, but the conflict is
unresolved, so a physical tap must probe BOTH S0_UART1 and S0_UART3 (3.3V TTL, 115200).

Two corrections to earlier framing in this session: there is **no PMpro-specific UART** in any
table — "SMpro console" and "SCP console" are one stream. And **Mt. Jade's OCP spec documents
no UART pin header at all**, only JTAG. The 1x3 header and `TS3A44159` mux are
reference-design facts, not confirmed Mt. Collins ones, so the physical-tap path is less
certain than it first appeared.

There IS a documented console-mux OEM command, `0x3c 0xB0`, in the Ampere vendor fork
(`cmdUartSW`, documented example `ipmitool raw 0x3c 0xb0 0x00 0x01`). It shells out to
`ampere_uartmux_ctrl.sh`, so it is very unlikely to exist on MegaRAC. It is cheap to probe and
AMI's OEM set leaves `0x3c 0xB0` unassigned, so a read-shaped probe carries no collision risk.

## Ordered probe plan, non-destructive first

1. `sel elist` and `sel list -v` — the reference implementation logs DECODED strings such as
   "DIMM Slot 3 MCU rank 1: PHY training failure: PHY Write Leveling failure: Slice 5: Upper
   Nibble: No rising edge error". Costs nothing and survives the destructive register read.
2. `sol info 1` (confirm 115.2 kbit), then `sol activate instance=2|3|4` — a different SOL
   instance may carry the SCP console.
3. `raw 0x06 0x52` against bus 2 / 0x4F, reading `0xB2` (current boot stage) first.
4. `0xB1` for the failure bits and per-DIMM bitmap.
5. `0xB4` → `0xB5` per slot, byte order established against a known-good slot first.
6. `0xB0` history walk LAST — it consumes what it reads.
7. Physical UART tap, probing both candidate pins, only if all of the above are dead.

## Standing caveat

`0x2c 0x03 0xae` (SBMR boot progress) is documented but the Ampere OpenBMC manual states
"SBMR related features are not available for Mt. Jade", so it is expected to fail. It is one
command and worth the attempt for the negative.

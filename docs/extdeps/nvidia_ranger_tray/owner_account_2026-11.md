# A running Ranger tray: one owner's account, November 2026

Source for `extdeps.nvidia.ranger_tray`, `extdeps.nvidia.gracebmc_ranger`,
`extdeps.zt_systems.ranger_front_io_board` and the pin rows that cite it.

An owner of a running NVIDIA Ranger compute tray wrote to this repository's operator on
2026-11-16 and 2026-11-17 (three messages) and allowed the content to be recorded here. At the
owner's request the owner and the lab are not named: "an owner of a running Ranger tray" is the
whole attribution. The text below is a transcription of the technical content with identities and
correspondence removed. Nothing here was read by this repository on a tray.

## Message 1, 2026-11-16: the tray, its cables and its power

- The tray is NVIDIA's "TS1C RANGER COMPUTE", P/N 675-24265-2020-T1C, with ZT Systems boards. Its
  cable map is the label inside the lid, H88-1516-001 REV X02.
- The controller is a plug-in management module in the slot printed "J18 DC-SCM" on the tray's
  front IO board (ZT Systems board 1395A3492101, rev X03, also marked BD-00325-003). No part number
  was read on the module itself.
- It carries an ASPEED AST2600 and runs NVIDIA's OpenBMC build GraceBMC-23.06-2-rc1, machine name
  "ranger" (model string "AST2600 Ranger BMC").
- One E1-type P4261 with a 699-2G530-0215-DV5 module runs in the tray.

Cables, from the lid label ("FIO" is the front IO board, "BB" the P4261 baseboard):

| Cable | From | To | Print |
|---|---|---|---|
| CA-03379-001 | BB-J7 | FIO-J12 | MCIO1 CONN |
| CA-03428-001 | BB-J8 | FIO-J13 | MCIO2 CONN |
| CA-03429-001 | BB-J13 | FIO-J20 | MGMT1 CONN |
| CA-03378-001 | BB-J14 | FIO-J19 | MGMT2 CONN |
| CA-03382-001 | power board J4 | BB-J20 | (power) |

The socket prints on the front board agree with the label where photographed (J13 "MCIO2 CONN",
J20 "MGMT1 CONN"), and BB-J8 to FIO-J13 was traced by hand.

Power. The tray's power board is fed with 48 V. There is no reading of the whole tray's input
current (fans and front board included). What exists are the P4261's own two readings, as its FPGA
reports them to the controller, which calls them "HSC 0" and "StandbyHSC 0":

- Standby, host off: HSC 0 = 7 W, StandbyHSC 0 = 27 W. Together about 0.7 A at 48 V.
- Running: StandbyHSC 0 = 37 W. HSC 0 = 162 W right after power-on, 140 to 150 W at idle, and up to
  412 W under load with the owner's own limits set (module 450 W, GPU 300 W). About 3.7 to 3.9 A at
  idle and 9.4 A at that maximum.

No contact level was measured on the running tray, and none is planned. Contacts are numbered as
SFF-TA-1016 does: rows A and B, 1 to 37, row B on the latch side.

## Message 2, 2026-11-16: the controller's lines and their order

From the controller's own start-up scripts and its pin table (GraceBMC-23.06-2-rc1 on this tray),
read on the controller. Not measurements at the sockets: the lines can be named, not which contact
carries which. "out" = the controller drives the board; "in" = the board answers. Levels are logic
levels at the controller.

A. At the controller's own boot, before standby
1. out, all set to 0 (done when FPGA_READY reads 0 at that moment, that is, when the board is not
   already up): BMC_READY, RUN_POWER_EN, LOCAL_RUN_PWR_EN, IOM_PWR_EN, EROT_FPGA_RST_L, FPGA_RST_L,
   BMC_EP_PERST_EN, PWR_BRAKE_L, SHDN_REQ_L, SHDN_FORCE_L, SYS_RST_IN_L, GLOBAL_WP.
2. in: BB_PRSNT_L must read 0 (board present). Nothing else is checked before standby or run
   power: no voltage, part number or FRU check.

B. Standby
3. out: STBY_POWER_EN = 1.
4. in: wait for STBY_POWER_PG = 1 (limit 20 s; on a miss STBY_POWER_EN goes back to 0).
5. out: BMC_EROT_FPGA_SPI_MUX_SEL = 1 (then released), FPGA_EROT_RECOVERY_L = 1.
6. out: PWR_BRAKE_L = 1, SHDN_REQ_L = 1, SHDN_FORCE_L = 1; SYS_RST_IN_L stays 0.
7. out: EROT_FPGA_RST_L = 1, then FPGA_RST_L = 1, with no pause between the two.
8. in: wait for FPGA_READY = 1 (limit about 60 s).
9. Bus work left out: the PCIe reset toward the FPGA is released and a few I2C messages go to the
   board.
10. out: BMC_READY = 1.

C. Run power (the controller's power-on)
11. out: SHDN_REQ_L = 1, SHDN_FORCE_L = 1; SYS_RST_IN_L = 0.
12. out: BMC_EP_PERST_EN = 1.
13. out: LOCAL_RUN_PWR_EN = 1 (by the scripts, the tray's own run power).
14. out: RUN_POWER_EN = 1.
15. in: wait for RUN_POWER_PG = 1 (limit 20 s; on a miss a forced shutdown).
16. IOM_PWR_EN would be raised here if an IO module were present; this tray has none, so it stays 0.
17. out: SYS_RST_IN_L = 1.
18. in: wait for SYS_RST_OUT_L = 1.
19. in: RUN_POWER_PG is read once more 5 s later.

D. Lines the controller only watches (active low; they rest at 1 on the running tray)
- SHDN_OK_L: when it falls, SYS_RST_IN_L = 0 and, 2 s later, RUN_POWER_EN = 0.
- RUN_POWER_FAULT_L, THERM_BB_OVERT_L, THERM_FPGA_OVERT_L: a fall means a hard power-off.
- THERM_BB_WARN_L and FPGA_EROT_FATAL_ERROR_L: a fall is logged.
- RUN_POWER_PG is watched as well: when it falls, SYS_RST_IN_L = 0.

E. Power-off and reset
- Graceful off: SHDN_REQ_L = 0 for 0.5 s, then 1; the board answers on SHDN_OK_L and the rule above
  does the rest.
- Forced off: SHDN_REQ_L = 0 and SHDN_FORCE_L = 0, BMC_EP_PERST_EN = 0; wait for SHDN_OK_L = 0 (up
  to 40 s); 2 s later RUN_POWER_EN = 0; wait for RUN_POWER_PG = 0; LOCAL_RUN_PWR_EN = 0; both SHDN
  lines back to 1.
- Host reset: SYS_RST_IN_L = 0 for 0.1 s, then 1.
- Standby off: all outputs 0, then STBY_POWER_EN = 0.

## Message 3, 2026-11-17: the controller's pins

For every line above: the AST2600 GPIO port and bit, the ball, and the line number, from the
controller's pin table. The number is the line's offset on the controller's main GPIO chip (8
lines per port, port A = 0); in sysfs numbering this firmware adds 816 (gpio-928, gpio-931 and
gpio-860 were read for O0, O3 and F4). Which contact of which connector carries each line is not
in the controller's files, so the table stops at the controller's pins.

Outputs (the controller drives the board)

| Line | Port | Ball | Number |
|---|---|---|---|
| STBY_POWER_EN | W7 | AD8 | 183 |
| BMC_EROT_FPGA_SPI_MUX_SEL | R4 | U25 | 140 |
| FPGA_EROT_RECOVERY_L | R6 | W26 | 142 |
| PWR_BRAKE_L | O2 | AD23 | 114 |
| SHDN_REQ_L | B5 | G26 | 13 |
| SHDN_FORCE_L | Q4 | Y26 | 132 |
| SYS_RST_IN_L | N3 | N24 | 107 |
| EROT_FPGA_RST_L | R5 | T23 | 141 |
| FPGA_RST_L | W5 | AF7 | 181 |
| BMC_READY | V2 | AD14 | 170 |
| BMC_EP_PERST_EN | R2 | V24 | 138 |
| LOCAL_RUN_PWR_EN | O0 | AD26 | 112 |
| RUN_POWER_EN | O3 | AD24 | 115 |
| IOM_PWR_EN | F4 | C22 | 44 |
| GLOBAL_WP | I6 | B16 | 70 |

Inputs the sequence waits on

| Line | Port | Ball | Number |
|---|---|---|---|
| BB_PRSNT_L | P4 | W23 | 124 |
| STBY_POWER_PG | W6 | AD7 | 182 |
| FPGA_READY | R0 | V25 | 136 |
| RUN_POWER_PG | O1 | AD22 | 113 |
| SYS_RST_OUT_L | N2 | N25 | 106 |

Inputs the controller only watches

| Line | Port | Ball | Number |
|---|---|---|---|
| SHDN_OK_L | B4 | J23 | 12 |
| RUN_POWER_FAULT_L | O4 | AD25 | 116 |
| THERM_BB_OVERT_L | O5 | AC22 | 117 |
| THERM_FPGA_OVERT_L | O7 | AC23 | 119 |
| THERM_BB_WARN_L | O6 | AC24 | 118 |
| FPGA_EROT_FATAL_ERROR_L | R7 | U26 | 143 |

LOCAL_RUN_PWR_EN also goes from 0 to 1 on the way to running (step 13, one step before
RUN_POWER_EN). It is a controller output (O0, ball AD26) that the scripts treat as the tray's own
run power. Where it lands has not been measured, so whether it reaches the P4261 or stays on the
power board's side is not known. (An earlier summary of "five lines that change between standby
and running" left it out; this message corrects that.)

## Message 4, 2026-11-17: where the sequence and pin table live, and the bus work before BMC_READY

Sent with the owner's agreement, in answer to two questions.

**Where the sequence and the pin table came from.** All of it is on the tray's controller
(GraceBMC-23.06-2-rc1).

- The sequence up to BMC_READY is `/usr/bin/bmc_ready.sh`. The run-power part after it is
  `/usr/bin/powerctrl.sh`. The standby request and the watchers are small scripts beside them:
  `stbypowerctrl.sh`, `standby_power_status_monitor.sh`, `power_status_monitor.sh`,
  `fpga_ready_monitor.sh`, `shutdown_ok_monitor.sh`.
- The pin table is not a file of its own. The line names are the `gpio-line-names` of the main GPIO
  controller (`gpio@1e780000`, chip 0, 208 lines) in the device tree the firmware boots with. They
  were read on the running controller with `gpioinfo` and `/sys/kernel/debug/gpio`. The port and bit
  follow from the offset (offset divided by 8 gives the letter, the remainder the bit), and the ball
  comes from the pin controller's own pin list in debugfs. There is also
  `/usr/share/gpio-config.json`, but it holds only two alert lines for a monitor.

**Bus work before BMC_READY.** All of it is in `bmc_ready.sh`, after FPGA_READY has read 1 and the
controller's PCIe reset is released, and all of it is on one bus: i2c-2 (adapter
`1e78a180.i2c-bus`).

- The script declares five EEPROM devices to the kernel. Its own comment places them behind the FPGA
  and says they do not answer on a cold start until the FPGA is ready: a 24c02 at 0x4e, and four
  24c64 at 0x50, 0x51, 0x52 and 0x53.
- Just before BMC_READY it writes to target 0x60 on the same bus: four pairs of writes, each a data
  register and then a command register, which the script's comments call SMBPBI registers of the
  FPGA. By those comments they set a thermal alert threshold of 105 C for four modules.
- Nothing else in that script touches I2C before BMC_READY, and the run-power script that follows
  contains no I2C command at all.
- The device tree also declares a 24c02 at 0x67 on that bus. No driver is bound to it and no script
  reads it.

Whether the board starts without the EEPROM declarations or the four threshold settings was not
tested. The voltage and current readings of HSC 0 and StandbyHSC 0 are to follow when the tray's
long job has ended.

## What was not supplied

- A part number for the management module.
- Which contact of BB-J13, BB-J14, BB-J7 or BB-J8 carries which line.
- The signalling voltage of any line.
- The whole tray's input current.
- Voltage and current for HSC 0 and StandbyHSC 0 (promised once the tray is free).
- The register numbers and bytes of the I2C writes before BMC_READY (message 4 gives the bus, the
  devices, the target and the stated purpose).
- Whether the board starts without the EEPROM declarations or the threshold writes (not tested).
- Which files hold the power-off and reset flows.
- Photographs.

## An earlier note from the same owner, 2026-10-31

About a K1-type P4261 in the server chassis it was pulled from (not the Ranger tray above): a PCIe
riser, HEPCBG4143, was cabled to the baseboard's J5 and J6 by cable RRC164-0688, with its top edge
plugged into a separate front board with a coin cell, which appeared to hold the controller and
power the riser slot. A ZT Systems power distribution board (PCB 1395A3492201, label PB-00185-003)
took 48-54 V on two round posts; a plug-in "12V VRM" card stepped that down, and a harness labelled
RWHHW-U220 (PDB-J4 to BB-J20) fed J20 with 12 V. Source for `extdeps.nvidia.p4261_k1_chassis`.

## Permission

"You may record what we send in your public repository. Please do not name the lab or its owner
there; 'an owner of a running Ranger tray' is enough." Repeated with each later message.

## What the owner asked for in return

Where each non-ground contact of BB-J13 and BB-J14 leads on a bare E1 or K1 board (the first part
it reaches), a partial list being useful; and, if easy, the same for the sideband contacts of BB-J7
and BB-J8 (8 to 12 and 26 to 30 of each row). Also whether the rear cable cartridge between the
compute trays and the NVLink switch trays ("Ranger switch tray", 675-24265-3000-T1C) has been seen
anywhere: a part number, a photo, a listing or a teardown.

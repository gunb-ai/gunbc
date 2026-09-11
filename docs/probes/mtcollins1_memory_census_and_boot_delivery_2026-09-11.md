# Mt. Collins unit 1 — memory census, and the boot-delivery defect it uncovered

Session `eager-owl-205`, 2026-09-10T19:00Z to 2026-09-11T01:40Z. Branch
`session/eager-owl-205`, PR gunb-ai/gunbc#10965 (draft, CHANGES_REQUESTED, correctly
held).

This is a probe record, not an authority. Every number in it is either quoted from a
committed artifact or was re-derived while writing this file; nothing is recalled. Where
a claim is inference rather than measurement it says so, because a large part of what
went wrong here was inference presented at the strength of measurement.

---

## 1. What was asked, and what the machine actually gates

The brief was `strategy.year_end_plan item_hw_first_host`: unit 1 through intake phases
1–5, of which this session owned two — Linux EDAC pass clean at nproc 160, and DIMM
population against the J-slot table — landing as one held-unit receipt module. BMC
recredentialing was explicitly **not** in scope and was not attempted.

The commercial weight sits on the DIMM census: it is the due diligence gating a
~320-module fleet RAM purchase across 20 units, and the operator had said they would not
commit capital without it.

---

## 2. Result

**The census is complete.** Both gaps this work initially declared open are closed.

| Question | Answer | Evidence |
|---|---|---|
| `nproc` | **160** | corroborated by `numactl`: 2 nodes × 80 CPUs |
| MemTotal | 262,088,084 kB (~249.9 GiB usable; 256 GiB nominal) | `/proc/meminfo` |
| DIMM population | 16 of 32 populated, all odd connectors, 16 GiB each @ 2666 MT/s, 16 distinct serials | `dmidecode --type 17` |
| Channel map | recovered — see §4 | firmware DRAM training + Ampere GSG Tables 12–13 |
| EDAC topology | `ghes_edac`, 1 controller, **0 csrow nodes** | counted over `find /sys/devices/system/edac` |
| EDAC health | **0 corrected, 0 uncorrected**, before *and* after a named 64 GiB workload | §6 |
| Unit binding | board serial `02030A800TEXFT02L` | host SMBIOS Type 2 **and** controller IPMI FRU |

### Committed artifacts (digests verified at time of writing)

| File | Bytes | SHA-256 |
|---|---|---|
| `artifacts/bmc/mtcollins1-hw-census.txt` | 247,849 | `9ec33e27b225848e2edc7454e0ff21b9081a19396f1a7205fe99529cf842bc8f` |
| `artifacts/bmc/mtcollins1-hw-census-sol.log` | 240,475 | `1e0c4bc3ab7f635f5522c23832e6d5db33d7c3b07e9467cea1f9814167bd0de8` |
| `artifacts/bmc/mtcollins1-attempt3-sol.log` | 205,441 | `c63f5014f47004dcb099c1d306d4afdc97ae20835a8f40e93dd770efb03e051d` |
| `artifacts/bmc/mtcollins1-attempt3-srv2-side.txt` | 1,669 | `e7673b74a9361453440364334d9223706284dc25d66c9f5bde1230c2ec5720ad` |
| `artifacts/bmc/mtcollins1-host-capture.txt` | 270,880 | `a9b880c3ecd9ba611c1df9e7de1ab089839c5d8e92b743bf2e1bcd684100fd2a` |

Receipt module: `dag/gunbc/machine_intake/mtcollins1_memory_census_observation.dag`.
Witness: `dag/test/claim/machine_intake/mtcollins1_memory_census_witness_test.dag`.

**The artifacts retain the BMC credential in `argv`, deliberately.** Two `----ARGV`
lines in `mtcollins1-attempt3-srv2-side.txt` record `ipmitool … -U admin -P admin`. I
redacted this after review flagged it; **the redaction was wrong and has been reverted.**
`admin/admin` is AMI's published MegaRAC factory default and the corpus already models it
as an upstream fact — `extdeps.bmc.megarac` `megarac_factory_login`, whose field is *named*
`published_password` because the value is public. The artifact discloses nothing the repo
does not already state, so redacting it protected nothing and destroyed byte fidelity.

The rule this settles: a credential in a capture is redactable only when it is a **secret**.
The real defect here is the producer — every ad-hoc call passed `-P` on the command line
while the modelled route already uses `-f` with a password file. If the unit's credential is
ever rotated off the published default, that value *is* a secret, must never reach argv, and
any capture containing it is genuinely unpublishable.

---

## 3. The two purchasing conclusions

Both were carried to the operator and accepted.

**Buy DDR4-2666, not 3200.** Every installed module is a 2666 part running at 2666.
Modules added to fill the empty slots share channels with the existing ones, and DDR4
clocks to the common minimum, so a faster grade buys nothing unless all 32 slots are
replaced. Ampere's published "up to DDR4-3200 (2DPC)" is a ceiling, not a guarantee, and
does not change this.

**Do not multiply this unit's rank mix by twenty.** Matching each channel's existing rank
requires **10 × 1Rx4 and 6 × 2Rx8** *for this unit*. That split is idiosyncratic — three
different part numbers across 16 sticks, including one part that appears exactly once.
Ordering ~320 modules against one sample would be ordering blind. Each unit needs its own
census; the census ISO does one unit in a single boot.

---

## 4. The channel map, and why EDAC could not supply it

The brief expected the EDAC csrow/channel tree to carry channel membership. **It does not
exist on this platform**, and this is structural rather than a gap to chase.

`dmesg` carries `GHES: APEI firmware first mode is enabled by APEI bit`. Under ACPI APEI
firmware-first the firmware handles memory errors and hands the OS finished records; the
OS never enumerates the memory controller. `ghes_edac` therefore registers **one**
synthetic controller and mirrors the SMBIOS Type 17 device list into it, purely so a GHES
error record has a DIMM to name. `find /sys/devices/system/edac` returns **zero** paths
containing `csrow` — a counted reading over an enumerated tree, not an inference from a
driver name. No boot changes this without a native Altra EDAC driver, which does not exist
in mainline.

**The channel-resolved reading comes from the Altra boot firmware instead**, and is
strictly richer than a csrow tree because it names slot-within-channel as well as
controller:

```
SK0 MC5 S0: RDIMM[ad:80] 16GB 2666 ECC 2R x8 RCD[32:86] HMA82GR7CJR8N-VK
```

All sixteen modules sit at `S0`, so the unit is uniformly 1DPC and every empty connector
is its channel's `S1`.

### The J-slot join, and what discriminates it

Neither reading prints a J-number, so the connector field is a **join**, declared as one.
The upstream half is already modelled at
`extdeps.ampere.mt_collins_product_brief.memory_population` (Ampere GSG Tables 12–13:
pairs `{J1,J2} … {J31,J32}`, pair *k* on MCU *k*, 1DPC populating the odd member) and was
consumed, not re-transcribed.

Three checks, the first decisive:

1. **A one-of-sixteen part lands on the one agreeing position.** `HMA82GR7CJR8N-VK` occurs
   exactly once in the machine. Firmware places it at `SK0 MC5`; SMBIOS at `DIMM 11`; the
   guide puts MCU5's 1DPC member at J11. No shift or reversal of the sequence reproduces
   that.
2. **SMBIOS's own `Rank` column** reproduces the firmware's 1R/2R sequence position for
   position (`1,2,1,1,2,2,1,2` / `1,1,2,1,1,1,1,2`) — a second field, read by a different
   agent from a different table.
3. **The populated set is exactly the odd connectors**, which the guide specifies
   independently and the firmware corroborates by reporting every module at `S0`.

**Not claimed:** that the board silkscreen reads "J11" at that connector. Nobody looked at
the board; that is a photograph, not a software reading.

### Per-connector table

| J | SMBIOS locator | Channel | Populated | Rank | Part | Empty partner needs |
|---|---|---|---|---|---|---|
| J1 | DIMM 1 | S0/MC0 | 16 GB | 1R | HMA82GR7AFR4N-VK | J2 → 1R |
| J3 | DIMM 3 | S0/MC1 | 16 GB | 2R | HMA82GR7AFR8N-VK | J4 → 2R |
| J5 | DIMM 5 | S0/MC2 | 16 GB | 1R | HMA82GR7AFR4N-VK | J6 → 1R |
| J7 | DIMM 7 | S0/MC3 | 16 GB | 1R | HMA82GR7AFR4N-VK | J8 → 1R |
| J9 | DIMM 9 | S0/MC4 | 16 GB | 2R | HMA82GR7AFR8N-VK | J10 → 2R |
| J11 | DIMM 11 | S0/MC5 | 16 GB | 2R | **HMA82GR7CJR8N-VK** | J12 → 2R |
| J13 | DIMM 13 | S0/MC6 | 16 GB | 1R | HMA82GR7AFR4N-VK | J14 → 1R |
| J15 | DIMM 15 | S0/MC7 | 16 GB | 2R | HMA82GR7AFR8N-VK | J16 → 2R |
| J17 | Socket 1 DIMM 1 | S1/MC0 | 16 GB | 1R | HMA82GR7AFR4N-VK | J18 → 1R |
| J19 | Socket 1 DIMM 3 | S1/MC1 | 16 GB | 1R | HMA82GR7AFR4N-VK | J20 → 1R |
| J21 | Socket 1 DIMM 5 | S1/MC2 | 16 GB | 2R | HMA82GR7AFR8N-VK | J22 → 2R |
| J23 | Socket 1 DIMM 7 | S1/MC3 | 16 GB | 1R | HMA82GR7AFR4N-VK | J24 → 1R |
| J25 | Socket 1 DIMM 9 | S1/MC4 | 16 GB | 1R | HMA82GR7AFR4N-VK | J26 → 1R |
| J27 | Socket 1 DIMM 11 | S1/MC5 | 16 GB | 1R | HMA82GR7AFR4N-VK | J28 → 1R |
| J29 | Socket 1 DIMM 13 | S1/MC6 | 16 GB | 1R | HMA82GR7AFR4N-VK | J30 → 1R |
| J31 | Socket 1 DIMM 15 | S1/MC7 | 16 GB | 2R | HMA82GR7AFR8N-VK | J32 → 2R |

The "empty partner needs" column is the OCP Mt. Jade same-channel rank rule applied to the
measured population. **That rule is not modelled in the corpus** — nothing under
`extdeps.ocp.mt_jade` carries it — so the column is a human conclusion from a document the
repository has not modelled, and the receipt deliberately does not author it as derived.

Serials, manufacturer and configured speed for all 32 connectors are in the committed
capture.

---

## 5. Unit binding

The first capture bound only to the board **model** (kernel `DMI: FOXCONN Mt.
Collins/Mt. Collins`) and said so. The second closes it, and the closure is an **equality
between two independent reads** rather than a stronger assertion of one:

- host, reading its own SMBIOS Type 2 under Linux: `02030A800TEXFT02L`
- controller, via IPMI FRU, days earlier, different transport, different agent
  (`mtcollins1_access_observation`): the same serial

Either alone is an observation of a reading; the two agreeing is an observation of a unit.
Also captured: system serial `MXX2080619`, UUID `d5b80e46-903a-1000-01e6-4cd68a3cc128`
(which matches the GUID in the earlier PXE DHCP capture).

---

## 6. EDAC health — load-backed, and its limits

The first capture read zero counters on a host up ~20 seconds, which bounds nothing: an
idle machine that has touched almost no memory cannot demonstrate its memory is sound.
That was recorded as `EdacSnapshotClean` specifically so it could not be quoted as a
qualification.

The second capture runs a named workload and reads counters on both sides:

```
4 × 16 GiB tmpfs chunks (64 GiB) seeded from /dev/urandom
sha256 hashed twice, passes compared
workload-write-rc=0        workload-digest-stable=yes
EDAC before: 34 counters all 0        EDAC after: 34 counters all 0
mc0 ce_count / ue_count / ce_noinfo_count / ue_noinfo_count: 0 both sides
```

**Why the digest comparison is there.** EDAC counters are the memory controller's own
account of itself; if it fails to notice a fault its counters stay zero and a receipt
reading only those counters reports clean. Hashing 64 GiB twice and requiring the digests
to match tests the **data**, not the controller's opinion of the data.

**What it still is not.** 64 GiB is a quarter of installed memory, the passes take minutes
not hours, and nothing exercises thermal or long-duration modes. This discharges "Linux
EDAC pass clean" for the `hw-first-host` clause. It does **not** discharge a soak, and it
must not be read as satisfying `MemoryAuthoritativeQualify`. The memtest86 pass recorded
earlier in the intake remains the endurance evidence.

---

## 7. The boot-delivery defect

### 7.1 Root cause

IPMI boot parameter 5 carries **two** decisions: the device selector **and** the BIOS boot
type (byte 1 bit 5 — 0 = PC-compatible/legacy, 1 = EFI). The failing runs read back:

```
- BIOS PC Compatible (legacy) boot
- Boot Device Selector : Force Boot from CD/DVD
```

**aarch64 has no legacy BIOS boot path.** A legacy-boot request for the virtual CD is
unsatisfiable, so the firmware falls through to its own boot order — PXE, which nothing on
this segment answers (see §7.4) — and then the UEFI shell.

Measured fix:

```
ipmitool chassis bootdev cdrom options=efiboot,persistent
  → "BIOS EFI boot" / "Force Boot from CD/DVD" / Boot Flag Valid
```

The next reset booted the census image and produced `mtcollins1-host-capture.txt`.

**Scope of that control, stated precisely.** Persistence was held constant between the
failing and succeeding probes, so it proves the **EFI axis** and nothing about persistence.
`persistent` was used for convenience on a machine that was not mine to leave reconfigured,
and the override was cleared at session end (§9.7). **What argv the intake should use, and
what policy governs persistence, are the migration's to state — this record does not
prescribe them.**

### 7.2 The modelling defect behind it

Two, and the second is the more important:

**A.** `efiboot`, `boot_type` and `legacy` appear **nowhere** in `oob_boot_handoff.dag`,
`mtcollins1_actuate.dag` or `extdeps/bmc/ipmi.dag`. The model carries the device selector
alone, so every handoff silently requests the default boot type. It models a two-field
decision as one field. `mtcollins1_boot_readback_rendering` even asserts the expected
string "Force Boot from CD/DVD" and matches it happily — the selector *is* right; the
unsatisfiable half is invisible to the check.

**B.** `oob_boot_handoff.dag:190` mints `BootHandoffCompleted { selected: device }`
immediately after the power action. Success means "override read back as expected + power
command accepted". It **never confirms the host booted that device**. Both preconditions
genuinely held on every failed boot this session — the selector really did say "Force Boot
from CD/DVD", the reset really was accepted — so a handoff that provably booted nothing is
indistinguishable, in the model, from one that worked.

That is DESIGN §5's specification-without-execution: a precondition rendered as an effect.
**It is why a legacy/EFI mismatch survived an entire intake lane**, and fixing the argv
without fixing the postcondition would close this instance and leave the class.

### 7.3 There are two distinct failure modes, not one

An early version of this analysis claimed the boot type explained every failed attempt. It
does not, and the correction came from a reviewer reading my own committed artifact:

| Mode | Symptom | Attempts |
|---|---|---|
| **A** — firmware never selects the CD | `Start PXE over IPv4` → `Shell>`, GRUB never runs | 20:23, 22:44 |
| **B** — firmware *does* select the CD | GRUB runs, kernel starts, then `EFI stub: ERROR: Failed to load initrd: 0x8000000000000001` → `VFS: Unable to mount root fs` | attempt 3 |

Mode B is a different bug. A medium that had vanished could not have served GRUB and a
kernel first. Attempt 3's script reset the host while the redirection was still at `100`
(connecting); GRUB's small reads succeeded and the larger initrd read did not. **Plausible,
not proven** — the artifact shows the initrd error and the timing, it does not isolate the
cause. Recorded at that strength deliberately.

### 7.4 PXE on this segment

Not broken — **not present**. `srv1` and `srv2` both show zero listeners on DHCP/67,
TFTP/69 and proxyDHCP/4011; `dnsmasq`, `tftpd-hpa`, `atftpd`, `isc-dhcp-server` and
`pixiecore` all inactive/absent; no `/srv/tftp`, `/srv/pxe` or `/var/lib/tftpboot`. The
committed receipt `mtcollins1-host-pxe-client.json` already recorded the same conclusion:
the host PXE-requests correctly (`PXEClient:Arch:00011:UNDI:003016`, arch 11 = EFI ARM64)
and the router answers with no next-server and no bootfile-name.

**"Netboot" on this platform means the modelled diskless route, not PXE**:

```
srv2 NFS export /srv/bmc (ro, scoped to 192.168.1.228)
  → MegaRAC remote-media client fetches the image itself
  → virtual CD presented to the host
  → host UEFI → GRUB 2.12 → EFI stub
```

Modelled across `mtcollins1_boot_image_fetch` → `mtcollins1_boot_artifact` →
`mtcollins1_media_attach` / `megarac_media_attach` → `mtcollins1_diskless_boot_receipt`.
Nothing is missing; `mtcollins1_netboot_client_observation` exists precisely to record that
PXE is a dead end here. I spent time re-deriving that already-held negative because my
brief said "PXE netboot" and I took it literally.

---

## 8. Controller-side observations

**The decoded status enum and the readiness rule that were here have been removed.** They
reconstructed the complete MegaRAC state machine from `source.min.js` as served by this
controller — and that file **is not a committed artifact**. This document opens by claiming
every number is artifact-backed or re-derived, so a table whose source was never preserved
was the one thing in here that could not honour that claim. It was also reusable protocol
policy, which belongs to `extdeps.bmc.megarac` keyed to a firmware identity, not to a probe
record.

What survives is what was measured against the live controller and is backed by this
session's own observations:

- **The attach transition, as observed.** 2026-09-10T22:38:16.528Z: status 100 with
  `session_index` 255; `session_index` 0 at +12s with status still 100; status 1 at +15s;
  still status 1 at +4m27s idle and throughout the boot that followed. Recorded as seen.
  **No readiness conclusion is drawn from it here** — which status permits a reset is a
  policy question owned by the migration, against a catalog keyed to firmware identity.
- **`(100, session_index 0)` is a real intermediate**, not a hypothetical: session
  allocation precedes the status flip by about three seconds.

### `PUT /api/settings/media/general` carries no verdict

Three cells measured, including one not previously listed:

| HTTP | Readback | Reality |
|---|---|---|
| 500 | desired | **applied** |
| 500 | unchanged | **not applied** |
| 200 | — | (not a verdict either) |

Both 500 cases occurred the same evening on the same endpoint with no configuration
difference I could find. I could not explain the variation and did not invent a reason.
**The readback comparator is the entire verdict**; the HTTP status is diagnostic only.

Also observed: `start-media` requests without the undocumented key `image_redirection: 1`
were refused with errors 13410/13460; the requests carrying it were accepted. How far that
generalizes is the status/API catalog's to say, not this record's.

### BMC controller reset timings (3 resets, consistent)

```
reset accepted → actually goes down:  13–25s   ← polling inside this window reads the OLD controller
IPMI answering again:                 69–88s
web/API answering again:             240–244s
```

The web stack is the slow, stable one at ~4 minutes. A recovery check that gives up at 90s
will wrongly conclude the controller is dead.


## 9. Errors I made, and what they cost

Recorded because the operator asked to understand the process end to end, and because most
of this session's elapsed time was spent on my mistakes rather than the hardware's.

**1. I asserted a meaning for `redirection_status` that I had not evidenced.** This is the
big one. I read the raw `100 -> 1` transition (§8) as a session dying, built a "~10 second
decay clock" on top of that reading, measured it with controls, and escalated it — sending
the directing session and an analyst into hours of packet captures, BMC resets and
teardown-hunting for a session that went on to serve a full boot. Every boot gate I wrote
was built on the same assertion and refused on the transition that preceded every success.
The defect is not which code means what; it is that I treated my own decoding of an opaque
vendor enum as a fact, and the artifacts never contained a source for it.

**2. I read a flag byte as a session count.** `cd_active_sessions` 128/129 → I claimed a
129-slot pool, "the unit arrived with 128 consumed", and "one virtual-media boot per BMC
reset" as fleet-scale knowledge. `128 = 0x80`, `129 = 0x81`; the neighbouring `cd_port`
reads `0x80000050` = port 80 with bit 31 set. It is a flag, not a count. Retracted.

**3. I overclaimed the root cause.** Said the legacy/EFI axis explained *every* failed
boot; attempt 3 reached GRUB and the kernel, so it cannot. Caught by a reviewer reading my
own committed artifact.

**4. A timestamp the artifact contradicts.** Declared `1789428410000` =
2026-09-14T23:26:50Z for a capture the artifact stamps 2026-09-11T01:26:50Z — wrong by
nearly four days. The digest was correct the whole time, which is the lesson: a digest
proves the bytes are unaltered and proves **nothing** about whether the fields around it
were read out of those bytes.

**5. One attempt identity for two attempts.** The successful capture's header reads
`attempt=3`; so does the failed run. The capture script hard-codes its attempt number, so
a rerun reuses the identity. I obeyed the letter of "never overwrite an earlier attempt" —
the file was preserved — and broke it one level down, where identity lives.

**6. `-P admin` in every ipmitool invocation**, putting the BMC password in argv, while the
modelled route already does the right thing with `-f`.

**7. `options=persistent` throughout**, changing all future boots on a machine that was
not mine to reconfigure. (Verified clear at session end: `Boot Flag Invalid` / `No
override`.)

**8. Operational slips:** raced two scripts against the controller simultaneously for ~1
minute; `pkill -f` patterns that matched my own SSH command line and killed the shell,
twice; polled for a BMC's return inside the window before it had gone down, contaminating
a measurement round.

**9. And the one that frames the rest: I never built the modelled capture producer.** It
was in the directing session's *first* message. I optimised for getting bytes and left the
actuation unreviewable and irreproducible — the exact failure class this lane exists to
eliminate. That limitation is now carried explicitly by
`mtcollins1_acquisition_standing = HistoricalAdHocCapture { reusable_producer_absent: true }`,
which is what lets this evidence land without being mistaken for producer-backed provenance.

---

## 10. Outstanding

**Not mine — being recut by the directing session.** An earlier revision of this section
enumerated that lane's design: the shape of the boot-intent product, how the readback should
refuse, which terminal splits into which, how the request body should be serialized, how
legacy-on-aarch64 should be made unconstructible. **That has been removed, for the same
reason §8's decoder was.** A probe record that specifies another lane's carriers becomes a
second authority for them the moment they land, and enumerating someone else's design in
prose is exactly the coupling this PR was recut to remove — the fact that I happened to
agree with the design does not make it mine to write down.

What this record contributes to that work is the *evidence*: the measured boot flags on
failing and succeeding attempts (§7.1), that the model reported a completed handoff
throughout (§7.2), the two distinct failure modes (§7.3), and the controller observations
in §8. The design conclusions drawn from that evidence belong in the migration's carriers,
where they can be consumed rather than read.

**Open questions:**

- Mode B (initrd load failure) — cause plausible but unproven.
- SOL is unreliable on this controller: it closed mid-POST at 22:45:53 and later delivered
  nothing at all while still accepting keystrokes. Treat it as diagnostic only; the
  delivered image's own receipt landing is a sturdier execution authority.
- `HMA82GR7CJR8N-VK` has no catalog row in `extdeps.memory.sk_hynix`, which blocks a
  `HostMemoryPopulation` for this unit. Deliberately not minted from our own boot log —
  that would render a local report as an upstream fact.
- OCP Mt. Jade same-channel rank rule not modelled in `extdeps.ocp.mt_jade`.
- Twice this session my PR was blocked by `IMPORT-MEMBER-ABSENT` defects on `main` that
  someone else introduced (the colo rename; #10977's fabric imports). The floor lane
  catches them, but only after they are on main and blocking every branch.

---

## 11. What this probe does NOT prescribe

An earlier revision ended with a numbered operating procedure for the next Mt. Collins —
which argv to use, which status to wait for, how long to budget. **That has been removed.**
A probe record that also prescribes protocol becomes a second authority the moment the
boot-path migration lands, and the migration is where the boot-intent product, the readiness
predicate and their admissibility rules are being modelled properly.

The observations above stand on their artifacts. The rules to draw from them are the
migration's to state, in a carrier that can be consumed rather than read.

Two things this record does assert, because they are measurements rather than policy:
PXE is not served on this segment (§7.4), and the boot-type axis was the varied term in the
control that booted (§7.1) — with the explicit caveat that this says nothing about
persistence, and nothing about Attempt 3, which reached GRUB and failed at its initrd.

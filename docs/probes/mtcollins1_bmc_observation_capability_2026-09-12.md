# Mt. Collins BMC — what this controller can and cannot do, read from its own UI source

Answers two questions that were being guessed at: whether UEFI settings are reachable, and
whether any mechanical console capture exists. All from the controller's own `source.min.js`,
not from probing endpoint names.

A THIRD QUESTION WAS ASKED HERE AND SHOULD NOT HAVE BEEN: "why is SOL silent". It presupposed its
own answer. SOL was never silent — a broken collector was — so the section that answered it is
withdrawn below rather than revised, and nothing in this document may be read as establishing a
cause for an absence that did not exist.

## Provenance

`https://192.168.1.228/source.min.js`, fetched 2026-09-12, 6,597,873 bytes,
sha256 `8b5ef64f5bf1df88a0f497887540d198a677e7a6f63f1bc8fc9bbc79e4f54fcd`.
255 distinct `api/` paths extracted to `artifacts/bmc/mtcollins1-megarac-api-surface.txt`,
sha256 `4fc53fea3fe49bc3ed3b967511654223578d59d963546117ab524df11b419bcd`.

This is the file whose ABSENCE forced a retraction on 2026-09-11: a `redirection_status` decoder
was read from it and then withdrawn from the receipt because the bytes were never committed. The
bytes are now committed, so any future reading of this controller's behaviour has a resolvable
source.

## SOL IS NOT SILENT — the collector was, and this section's original claim is withdrawn

**SUPERSEDED BY MEASUREMENT.** This section was headed "Why SOL is silent — not a BMC defect" and
stated flatly that "the cause is host-side console routing". Both the premise and the cause are
withdrawn. SOL on this controller delivers host console bytes: a repaired collector captured 3764
bytes of firmware output across a failing DRAM training boot, including a 16-row per-DIMM table and
typed failure lines. Review 64316 found that this section contradicted the ledger row added in the
same change, and it was right — the corpus cannot answer "what causes SOL silence" two ways.

**THE MEASURED CAUSE, which is not this one.** The collector ran `ipmitool sol activate` with stdin
not a terminal, which warns, then its receive loop reached stdin EOF and it exited `rc=0` after
roughly three seconds of a four-hundred-and-twenty second window, while the wrapper slept out the
remainder and reported the unchanged file. Eight captures across eight hardware configurations were
86 bytes of the instrument's own warning and banner, seven of them byte-identical. Held under a pty
with stdin open, the same client stayed connected 645 seconds and captured host output on the first
attempt. Authority: `gunbc.recurring_failure_mode` `empty_capture_read_as_clean_result`, seventh
instance.

**WHAT SURVIVES, AT ITS ACTUAL STRENGTH.** The BMC-side readings stand and were never in dispute:
`sol info` reports `Enabled: true`, 115.2 kbps volatile and non-volatile, channel 1, port 623, and
sessions establish and accept keystrokes. The boot-parameter readback also stands as a READING —
every actuation carried `Console Redirection control : Console redirection occurs per BIOS
configuration setting`, i.e. request byte `00`, defer to BIOS.

What does NOT follow from that reading is that it explains anything, because there was no silence
to explain. "The BIOS evidently routes console to video" was an inference consumed as a cause
(DESIGN section 4d), built to account for an absence that the instrument manufactured. It is
withdrawn, not merely softened: byte `00` is consistent with a console that prints over SOL
perfectly well, which is what was subsequently observed while that byte's successor was in effect.

IPMI boot parameter 5 data byte 3, bits 1:0 control this: `00` defer, `01` suppress, `10`
request enabled. Set to `0x02` and verified: the readback now renders
`Request console redirection be enabled`. Whether the firmware HONOURS the request is untested
and is a separate fact from the BMC being able to ask.

## UEFI / BIOS settings — NOT REACHABLE

Across all 255 endpoints the only BIOS-adjacent ones are
`maintenance/firmware/upgrade`, `maintenance/hpm/getbiosversions`,
`maintenance/ValidBootfwSignImage` — flashing and version reads. There is no settings surface.
`/redfish/v1/` returns 404; this controller serves no Redfish at all.

CONSEQUENCE FOR CONVERGENCE: UEFI configuration on this platform is reachable ONLY through an
interactive console. Any workflow step requiring a firmware setting is a human-attended barrier
on Mt. Collins, not an automatable action, and should be modelled as one rather than assumed.

## The mechanical console capture that existed all along

```
settings/video/triggers        settings/video/sol-triggers
settings/video/pre-event       settings/sol-recorded-video
logs/video   logs/video-data   logs/video-log   logs/video-log/delete
sol/session  settings/javasol  remote_control/get/kvm/launch
```

Every trigger was `0` on arrival — the feature shipped disabled and had never been used. The
controller can record the host console ITSELF on events, which survives the resets that kill a
live `ipmitool sol activate` session. That is the difference between an instrument that needs a
babysitter and one that does not.

ARMED 2026-09-12, verified by readback on both objects:
- video and SOL recording: chassis power-on, chassis reset, critical, non-recoverable, watchdog
- `video/pre-event`: enable 1, fps 1, duration 10

TWO RECORDERS, DIFFERENT SUBJECTS, and the distinction is load-bearing:
- `video/*` records the FRAMEBUFFER — the content the KVM renders, which works today
- `sol-triggers` records the SOL STREAM — empty until console redirection actually takes effect

So video capture is useful immediately; SOL capture becomes useful only if the firmware honours
the redirection request.

## The observability lesson this session paid for

SOL returned an identical 86-byte banner across eight configurations, four power states and
every reset, while the KVM rendered real output throughout. Inferences were drawn from that
silence — including that failures occurred before console initialisation. THE INSTRUMENT WAS
POINTED ELSEWHERE. A capability model that records `sol_stream: present` without recording
`delivering: no` will mislead the next platform the same way.

## What a 20-unit intake needs before it starts

1. Console capture that does not depend on a live session — now armed here, and it should be
   part of unit bring-up rather than discovered per unit.
2. A diagnostic image whose output does NOT depend on the console. Tonight's census image
   reports over serial only; it produced nothing retrievable because it died at initrd load and
   because serial carries nothing. The NFS share it boots from is writable and was empty.
3. Console redirection requested explicitly at every actuation, not deferred to BIOS.
4. Per-unit capability observation, because none of this is uniform: this controller serves no
   Redfish, exposes no BIOS settings, no per-DIMM telemetry, and cannot decode its own OEM SEL
   records.

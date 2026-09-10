# Recovered: the Mt. Collins host image source

**This directory is preserved provenance, not a build input, and nothing in this repository
executes it.** It is the hand-authored source of the host boot tooling that has started every
micro-VM CI job on `mtcollins1`, recovered on 2026-09-10 from `/var/tmp/gunbc-hostimage` on
**srv2**. It is committed so that the code production actually runs is *readable* while its
replacement is built, and for no other purpose.

## Why this is here at all

`docs/plans/microvm-launch-displacement-analysis.md` established that step 9 of the live
per-attempt path — the jailer exec that starts the VMM — has no `.dag` authority, and that the
producing code appears in **no repository, in any language**. That finding is correct and this
directory does not retire it: recovering the bytes is not the same as modeling the behaviour.

What the analysis got wrong is where the artifacts live, and the correction is load-bearing:

- `gunbc.runner.runner_host_image_store` `host_image_store.location` records the export as
  **`/srv/bmc` on srv1**. There is no `/srv/bmc` on srv1. The export is on **srv2**
  (`/srv/bmc 192.168.1.228(ro,no_subtree_check,insecure,no_root_squash)`), and the seven
  digest-named host ISOs and their `.meta` sidecars are intact there, 6.7 GB.
- That module's own annotation names the BMC's redirection configuration as the authority for
  where images live. The location was instead carried as prose in a `NonEmptyStr`, so it was
  unreachable from the thing that owns it and rotted without either end being touched — a §3
  citation defect. Correcting the string is not the fix; deriving the location is.

## What was recovered

`initramfs/init` is the **template** and is the source of truth: it carries `@@BUILD_ID@@`
placeholders. `irfs/init` is the instantiated copy for the `checkout-2` build and differs from the
template in exactly two lines (the build id at lines 80 and 233). The `.prev`, `.pre-verify` and
`.pre-supervisor` files are hand-versioned generations kept as-found — filename-as-version is the
same nickname defect the image store already records for the ISOs, and they are preserved rather
than tidied because they are evidence of how the current form was arrived at.

| file | lines | subject |
|---|---|---|
| `initramfs/init` | 838 | the host boot script: per-attempt steps 3-9, including the step-9 jailer exec at line 602 |
| `guest/gunbc-runner-init.sh` | 133 | guest-side runner bring-up |
| `build.sh` | 95 | ISO assembly |
| `guest/gunbc-probe.sh` | 78 | guest probe |
| `envelope.sh` | 66 | signed envelope |
| `initramfs.spec.in` | 37 | initramfs spec template |
| `udhcpc.script` | 25 | guest DHCP hook |

`vm.json` is the Firecracker machine config; `cp/cp-signing.pub` is a public key. **No private key
material is committed here.** Two private keys (`cp-signing.key`, `impostor.key`) were found beside
these files in `/var/tmp` on srv2, outside any custody model; they were deliberately excluded, and
their presence on that host is a separate exposure that this commit does not resolve.

## Standing, stated honestly

This is a §6 scaffold tell by construction — hand-authored shell implementing semantics that
belong in `.dag` — and committing it does not make it admissible. It is admitted only as the
readable X of a replacement migration (§3), on the reasoning that a translation cannot be verified
against a subject nobody can see, and that one copy on one unreplicated disk in world-writable
temp is not a place to leave the sole causal path of the CI fleet while that translation is built.

**Dissolution condition.** This directory is deleted when the per-attempt path steps 3-9 are
performed by modeled operations and the host image is built from the model, such that no boot
depends on any file here. Deleting it is not satisfied by a modeled executor existing beside this
tooling: per the displacement doctrine, X's authority must end in one motion, and Y may not
resolve through X. Until then, no file here may be edited to change behaviour — a fix applied
here would make this a second live authority, which is exactly what it exists to prevent.

## Recovered-file digests (sha256, first 16 hex)

```
44cf6c81bb95a3e3  build.sh
3bae750ad15221a3  envelope.sh
7c4f7b0c88d872f0  initramfs/init
5f0c56c901eac0b9  irfs/init
f4cb81830bd434f8  initramfs/init.prev
8f1982f93592107f  initramfs/init.pre-verify
3e3a155dd4e4c49d  initramfs/init.pre-supervisor
be538270d7a4aeb8  guest/gunbc-runner-init.sh
1d97baf57fefaca9  guest/gunbc-probe.sh
22d82a3a448bf543  udhcpc.script
2d97f20a0a47e6bd  initramfs.spec.in
196d781734e8f8a1  vm.json
2fe3e60033e611ad  build.sh.bak
98087d2c9f8f95ac  cp/cp-signing.pub
```

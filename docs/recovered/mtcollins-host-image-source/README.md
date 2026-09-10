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

- `gunbc.runner.runner_host_image_store` `host_image_store` carried a `location` field reading
  **`/srv/bmc` on srv1**. There is no `/srv/bmc` on srv1. The export is on **srv2**
  (`/srv/bmc 192.168.1.228(ro,no_subtree_check,insecure,no_root_squash)`), and the seven
  digest-named host ISOs and their `.meta` sidecars are intact there, 6.7 GB.
- That module's own `attachment_authority` names the BMC's redirection configuration as the
  authority for image presentation, and the same configuration names where images are read from.
  The location was instead spelled as a literal beside it, so it was unreachable from the thing
  that owns it and decayed without either end being touched — a §3 second answer to a question the
  module already routes elsewhere.
- **The field is deleted in this PR, not corrected.** A freshly-true literal rots exactly as the
  last one did and restores the confidence that made this expensive, so the row is uprooted (§3
  delete-first) and the module now answers *what* the store guarantees while refusing to answer
  *where*. Restoring the field takes the capability named in that module: a modeled read of the
  BMC's `remote/configurations` yielding serving host and path as an observation with its read
  time. Nothing short of that.

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

## Integrity

**No digest table is transcribed here.** Fourteen copied hex strings would be numbers with no
producer: nothing recomputes them, so they rot silently the first time a file is touched, and the
one thing a provenance directory must not do is carry an integrity claim nobody can check (DESIGN
§6 — name the instrument, never transcribe its output). An earlier revision of this README did
carry that table, and it listed `build.sh.bak`, which `.gitignore`'s `*.bak` rule had excluded from
the commit — a hand-maintained table asserting a digest for bytes that were not there. The file is
now force-added and the table is gone.

Git already holds the content hash of every file here, and it is the producer:

```
git ls-tree -r HEAD --format='%(objectname) %(path)' docs/recovered/mtcollins-host-image-source
```

To check these against the machine they came from, while `/var/tmp/gunbc-hostimage` still exists on
srv2:

```
ssh srv2 'cd /var/tmp/gunbc-hostimage && git hash-object build.sh envelope.sh initramfs/init \
  irfs/init initramfs.spec.in udhcpc.script vm.json'
```

Those blob hashes are directly comparable to the `git ls-tree` output above. When that build tree is
gone, this directory becomes the only copy and the comparison is no longer available — which is the
condition this directory exists to survive, not one it can check its way out of.

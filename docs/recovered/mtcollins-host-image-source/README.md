# Recovered Mt. Collins host-image source

Recovered on 2026-09-10 from `/var/tmp/gunbc-hostimage` on srv2 so the existing per-attempt boot path can be inspected during replacement. These files are provenance, not build inputs; recovering them does not model their behavior.

`initramfs/init.recovered` is the template (`@@BUILD_ID@@`); `irfs/init.recovered` is the checkout-2 instance. Keep earlier `.prev`, `.pre-verify` and `.pre-supervisor` generations as found. The remaining sources cover ISO assembly, guest initialization/probing, envelope signing, DHCP and initramfs specification; `vm.json` is the machine configuration and `cp/cp-signing.pub` is public. The two private keys found on the source host were excluded; this archive does not resolve their custody exposure.

All recovered scripts have a `.recovered` suffix and mode `100644`. Preserve those properties and do not change behavior here. They discourage accidental execution but cannot make execution impossible.

The image-store location is an observation of BMC redirection configuration, not a second literal in `gunbc.runner.runner_host_image_store`. The original srv1 location was wrong; the recovery found the export on srv2. Restoring a location field requires a modeled, dated read of the BMC's `remote/configurations`.

**Removal condition:** modeled operations perform per-attempt steps 3–9 and generate the host image, no boot depends on these sources, and the replacement does not resolve through the old path. See [launcher displacement](../../plans/microvm-launch-displacement-analysis.md).

**Integrity:** Git identifies the retained bytes:

```sh
git ls-tree -r HEAD --format='%(objectname) %(path)' docs/recovered/mtcollins-host-image-source
```

Where the original build tree still exists, compare its `git hash-object` results to these blobs. Losing that external copy removes the comparison, not the archived evidence.

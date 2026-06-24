# WebUI-KVM virtual-media attach (OpenBMC boards without Redfish VirtualMedia)

`virtual_media_kvm_attach.mjs` headless-drives the OpenBMC **webui-vue** HTML5 UI to
attach an OS ISO as virtual media on boards whose **Redfish VirtualMedia surface is
absent** — e.g. the ASRockRack ALTRAD8UD-1L2T (srv3, BMC `192.168.1.192`, OpenBMC
02.07.00). It is the realization of the install arm that cannot go over Redfish here.

## Why this exists (grounded findings, verified against the live BMC)

- The BMC UI is stock OpenBMC **webui-vue** (Vue SPA, HTML5 — **not** a Java/ActiveX
  applet). Title `OpenBMC Web UI`; virtual-media route `/#/operations/virtual-media`.
- `/redfish/v1/Managers/bmc/VirtualMedia` and `/redfish/v1/Systems/system/VirtualMedia`
  both **404 even with auth**. The webui-vue "remote URI / configured media" path
  (`startLegacy` → Redfish `InsertMedia`) therefore has no device to act on; the page
  shows only **"Load image from web browser"**.
- So the **only** attach mechanism is a **local file streamed over an NBD websocket**
  (`wss://<bmc>/vm/0/0`, the `startVM` path). Consequences:
  - The ISO must be a **local file on the host running this script** — it cannot be
    handed to the BMC as a URL. The script downloads it first.
  - The **NBD websocket *is* the virtual CD**: the browser session must stay alive for
    the entire mount duration. If the process exits, the media detaches. So the script
    holds the session open across the boot trigger and the install.

The grounded surface (route, NBD path, DOM ids) is **not hardcoded** in this script. It
is read from a drift-gated descriptor (`srv3/virtual-media-kvm.yaml`) emitted from the
single authority `dsl/extdeps/bmc/webui_kvm.dag` (via `gunbc/webui_kvm_emit.dag` +
`generated_artifact.dag`). Edit the `.dag`, regenerate, never hand-edit the descriptor.

## Sequence (under `--attach`)

1. Fetch the ISO locally (NBD needs a local file).
2. Log in, open the virtual-media page, select the ISO, click **Start** → NBD stream
   begins; wait for the **Stop** button to confirm **mounted**.
3. Run `--boot-hook-command` **synchronously** (the Redfish boot-once-CD step, owned by
   the BMC-lifecycle lane). **Fail-closed:** exit 0 = boot issued; nonzero ⇒ abort,
   surface the error, and (default) hold the media for inspection — do not tear down.
4. Keep the NBD session alive (`--wait-seconds`, SIGINT to end early) while the host
   installs from the media, then click Stop and exit.

The boot half (BootSourceOverride=Cd/Once + ComputerSystem.Reset) is **not** done here —
it is a Redfish step invoked via `--boot-hook-command`. The NBD-backed media presents to
the host as the CD-ROM device, so boot-once-to-Cd targets it correctly.

## Safety

**Default is dry-run.** Without `--attach` the script logs in, maps the DOM read-only,
prints the attach + boot it *would* perform, downloads nothing, and writes nothing to the
BMC. `--attach` is the single gate for the real attach **and** the boot hook. The BMC
operator is lockout-sensitive; review a dry-run before firing `--attach`.

## Setup

```bash
cd provisioning
npm install            # installs playwright + (postinstall) the chromium browser
```

Headless chromium needs system libraries (`libnss3`, `libgbm1`, `libasound2`, …). On a
minimal/CI host without them, `npx playwright install-deps chromium` (root) installs the
set; without root, stage the matching distro packages into a prefix and export
`LD_LIBRARY_PATH` + a `FONTCONFIG_FILE` pointing at a font dir (chromium aborts on a
missing fontconfig config).

## Usage

```bash
# dry-run (safe; default): map the live UI, print the plan
node virtual_media_kvm_attach.mjs

# live attach + boot (operator-gated; ISO is fetched locally first)
node virtual_media_kvm_attach.mjs --attach \
  --bmc-host https://192.168.1.192 \
  --iso-url http://192.168.1.188/ubuntu-24.04/ubuntu.iso \
  --boot-hook-command 'gunbc run --source-root dsl --entry dsl/gunbc/srv3_boot_once_cd.dag --function srv3_boot_once_cd'
```

Key flags: `--iso-file <path>` (use an already-local ISO), `--no-boot-hook` (attach
only), `--wait-seconds <n>` (post-boot hold; SIGINT ends early), `--bmc-pass`/`$BMC_PASS`.
Run `--help` for the full list. Credentials default to the unrotated factory pair
(`root` / `0penBmc`); pass `--bmc-pass` / `$BMC_PASS` once srv3's BMC password is rotated.

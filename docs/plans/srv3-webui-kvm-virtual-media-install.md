# srv3 WebUI-KVM virtual-media install — design (for review, before implement)

**Status:** design for sign-off. Author stern-crane-163 (under neat-boar-71, BMC onboarding lane). **Prior approaches rejected** — this is a from-scratch redesign that obeys DESIGN.md and the merged host-effect orchestration plan (`docs/plans/host-effect-orchestration.md`).

## 0. The pain (displaced cost)

srv3's BMC is OpenBMC 2.07. Its firmware **does not expose Redfish VirtualMedia** (cited: `b351dac6cf`, `c1f7ed37fe`) and PXE can't cross the Tailscale overlay (no same-L2 / BIOS net stack). The **only** Tailscale-agnostic remote-boot path is the OpenBMC **WebUI** virtual-media page: an NBD-over-websocket "Load image from web browser" mount, driven through the browser SPA. To bring up srv3's OS we must headless-drive that WebUI: log in → open the virtual-media page → attach the local Ubuntu ISO → Start → then fire the boot-once-to-CD hook (already modeled, `gunbc.srv3_boot_once_cd`).

## 1. Why the prior approaches were rejected (do not repeat)

- **Hand-written `provisioning/virtual_media_kvm_attach.mjs`** — install logic written directly in Node/JS. Off-substrate: not the authority, not analyzable, hand-maintained.
- **Embedded-JS-emit (`gunbc.webui_kvm_runner_emit` + `extdeps.browser.session_program`)** — `.dag` that *emits* a `.mjs` by string-concatenating Playwright JS (`op_page_js`, `runner_prelude` returning raw `"function fail(msg){…}"` lines). This is hand-written JS one level removed: the JS body is authored as `.dag` string literals, which is exactly the embedded-shell/JS anti-pattern DESIGN §5.1 / the shell-literal containment guard forbid. It also forks the browser op vocabulary into a second `BrowserPageOp` coproduct alongside the `service browser.Page` ops (§3 nickname).

The root mistake in both: **the install logic lived in JS** (authored or emitted). The fix is that the install logic lives **as a modeled `.dag` program of typed browser operations**, and JS appears **only** below the transport boundary, in a single generic, install-agnostic runner (opaque per the §3 rename test) — never authored per-install, never emitted.

## 2. The substrate already supports this JS-free (key finding)

`dsl/extdeps/browser/browser.dag` is **already on main**: `service browser.Page { operation Goto { … transport shell { argv: ["playwright-runner","goto","{url}", …] } } }` etc. (Page: Goto, Fill, Click, WaitForSelector, UploadFile, Wait, … — the full set the install needs).

The v1 interpreter already dispatches typed service-op calls generically: a `browser.Page.Goto(url: …)` call in a workflow `func`, when run via `gunbc run`, flows `eval_service_call → wet_service_call → dispatch_service_wet → dispatch_shell`, which interpolates the `argv` template from the param env and spawns the process (`v1_interpreter.rs:3490,3675`; `push_shell_argv_tokens`). **No emit step. No JS authored.** The install is a `func` that *calls* the modeled ops in sequence; execution is the existing generic shell transport invoking the generic `playwright-runner` CLI (already the blessed argv on main).

This is the DESIGN §3 shape/transport/policy split, exactly:
- **shape** = `service browser.Page` ops (extdeps/browser, on main) — the parameterized contract.
- **transport** = the generic `playwright-runner` CLI, one of N handlers, opaque below the boundary. Its JS is fixed and install-agnostic (a per-op interpreter), so it is the legitimate realization, not embedded install logic.
- **grounded surface** = the cited OpenBMC virtual-media DOM (selectors/routes/labels) — extdeps, pure data.
- **policy / program** = the install op-sequence — workflow layer (gunbc), references `fleet_intent`.

### 2a. Transport-realization requirement (note, not model)
A browser session is stateful; each `playwright-runner <op>` is a separate process. The generic runner must persist one browser context across op invocations (persistent profile dir / CDP daemon keyed by the `Launch` context handle). This is a transport concern below the §3 boundary — the model is agnostic to it. The runner is provided once, generic; this PR does not author install JS.

## 3. A grounding gap this surfaces (must fix for srv3 to resolve correctly)

`extdeps.bmc.capability.CapabilityVirtualMedia` is anchored on **dmtf.org/redfish** — it means *Redfish* VirtualMedia specifically. `openbmc_2_07_00_capabilities` correctly omits it. But `os_install_mechanism.solve_install_mechanism` then resolves srv3 to **`PxeHttpInstall`** (FirmwareUpdate present but no VM-capable firmware in catalog) — which is wrong: PXE can't cross Tailscale, and the real path is WebUI VirtualMedia.

Root cause = **§3 conflation**: "VirtualMedia (the mechanism)" is fused with "Redfish (one transport for it)". WebUI NBD-over-websocket is a *second transport* of the same mechanism that the model can't express.

**Fix (decompose, don't nickname):**
- Add `CapabilityWebuiVirtualMedia` to `BmcCapability` (distinct grounded capability: browser-driven NBD-over-websocket mount, anchored on the openbmc webui-vue VirtualMedia source). Add it to `openbmc_2_07_00_capabilities` (srv3's firmware *does* expose this).
- Add a `WebuiVirtualMediaInstall` arm to `OsInstallMechanism` and a branch in `solve_install_mechanism`: Redfish VM → `VirtualMediaInstall`; else WebUI VM → `WebuiVirtualMediaInstall`; else firmware-update path; else PXE. srv3 then resolves to `WebuiVirtualMediaInstall`, the modeled path.

This keeps a single authority for "what mechanism does this host use" and makes the WebUI-KVM install the *consequence* of the grounded capability, not a hand-wired choice.

## 4. Files

- `dsl/extdeps/bmc/webui_kvm.dag` *(new, pure data)* — cited grounding of the OpenBMC virtual-media WebUI surface: SPA route, NBD websocket path, transport kind (`NbdWebsocketLocalFile`), and the DOM ids/labels/selectors. Anchored on the openbmc webui-vue `VirtualMedia.vue` source. (The rejected branch's `webui_kvm.dag` was pure JS-free data — that grounding is reused; the JS emitters are dropped.)
- `dsl/extdeps/bmc/capability.dag` *(edit)* — `CapabilityWebuiVirtualMedia` + add to openbmc 2.07 row (§3 above).
- `dsl/gunbc/os_install_mechanism.dag` *(edit)* — `WebuiVirtualMediaInstall` arm + solve branch.
- `dsl/gunbc/srv3_virtual_media_install.dag` *(new)* — the JS-free install program:
  - **pure** helpers deriving concrete args from the grounded surface + bmc host: `login_url(host)`, `virtual_media_url(host)`, `file_input_selector()`, `start_button_selector()`, `stop_button_selector()`. Unit-testable, with discriminating REDs.
  - **effectful** `func srv3_webui_kvm_install(...) -> ProcessExit uses net` that calls `browser.Page.*` ops in sequence using those derived args, fail-closed (a nonzero `playwright-runner` exit propagates as a loud interp error, never a fabricated success). Composes the `gunbc.srv3_boot_once_cd` hook after a confirmed mount.
- `dsl/test/claim/srv3_virtual_media_kvm_witness_test.dag` *(new, `test fn`s)* — green-by-execution witnesses over the pure surface + derivations + the corrected mechanism solve (srv3 ⇒ `WebuiVirtualMediaInstall`); discriminating REDs (wrong selector / wrong route fails).

## 5. Ownership / boundaries (escalation-aware)

`gunbc.host_effect.HostEffect` is a **co-owned coproduct** (smart-newt-512 + neat-boar-71); "no arm lands alone." Browser-driving is naturally a future `BrowserAction` arm of `HostEffect` (the WebUI realization of the VirtualMedia mechanism the host-effect plan §2 calls out). **This PR does NOT add a HostEffect arm** — it lands the install as its own workflow over the existing `browser.Page` service ops, and flags the `BrowserAction`-arm fold as a follow-on co-owned step. This respects ownership and keeps the change additive.

## 6. Proof (the deliverable's green-by-execution)

- `test fn` witnesses: surface grounding, pure arg derivations, corrected `solve_install_mechanism(srv3) == WebuiVirtualMediaInstall`, each with a discriminating RED.
- The effectful `func` is realization (live BMC); it is proven by the same generic shell-dispatch path that `shell.Exec.Run`/`Clock.UnixSecs` use, not a bespoke executor. No JS authored or emitted anywhere in the change.
</content>
</invoke>

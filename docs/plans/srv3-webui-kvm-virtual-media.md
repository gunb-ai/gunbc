# srv3 WebUI-KVM virtual-media install — design for sign-off

**Status:** design-for-review. No implementation until the §5 boundary question is signed.
Lane: `sharp-stag-30` under neat-boar-71 (BMC onboarding). DESIGN refs: §3 (interface ≠ transport ≠
policy; dispatch is realization → peripheral), §4 (one grammar, modeled argv not authored strings),
§5 (fail-closed; green-by-execution; no fabricated success), §6 (model just-in-time; price in
displaced cost).

This note exists because **two prior attempts on this exact task were rejected**, both on the same
boundary: (1) a hand-written 247-line Playwright `.mjs` runner (#5749), and (2) "emit the runner from
the model" — which still produced **embedded JS** out of the model (`op_page_js`/`session_program`,
dec00d4abe). The lesson the rejections encode: **no task-specific JavaScript may be authored OR
emitted.** This design starts from that constraint and derives the shape, then isolates the one
decision a human must make before any code lands.

## 0. The ground truth (why WebUI at all)

srv3's BMC (ASRockRack ALTRAD8UD-1L2T, OpenBMC 2.07.00) exposes **VirtualMedia only via the legacy
WebUI KVM, not over Redfish** — `404` on the Redfish VirtualMedia paths, on the installed *and* the
latest 03.22.00 firmware (`extdeps/bmc/capability.dag`, `openbmc_redfish_vs_webui_virtual_media_doc`,
cited #b351dac6). PXE can't cross the Tailscale overlay (needs BIOS net-stack + same-L2). So the only
**automatable** remote virtual-media path on this board is **driving the WebUI KVM with a browser** —
the manual-attach step made into an effect.

## 1. The §3 framing: WebUI automation is a *transport*, not a new concept

The host-effect interface (`gunbc.host_effect`, #5756) already models the agnostic shape:

```
apply(target: NodeControlPlane, effect: HostEffect, policy: Policy) -> Reconciliation
   NodeControlPlane = HostOs(node) | BmcController(node)
```

"Mount this ISO as a virtual CD and boot-once from it" is **one agnostic BMC effect-kind**. It has two
transports: `RedfishRest` (the `RedfishAction` arm — unavailable on srv3's firmware) and a **new
`WebUiBrowser` transport** (Playwright over the WebUI). Per DESIGN §3 *"the dispatch that selects a
realization is itself realization → it sits peripheral"*: the agnostic effect stays central; the
WebUI step-sequence + the page selectors are **realization detail**, homed in `extdeps`. The firmware
capability solver (`os_install_mechanism.solve_install_mechanism`) is exactly the dispatch that picks
WebUiBrowser **because** `CapabilityVirtualMedia` (the Redfish surface) is false for this board.

This is the principled win: WebUI-KVM is not a bespoke side-path — it is the cell
`(BmcController × WebUiBrowser × VirtualMediaBoot)` of the existing grid. It composes with the
lifecycle rather than forking it.

## 2. Three layers (the import arrow points toward std)

| # | Artifact | Layer | Content |
| - | -------- | ----- | ------- |
| 1 | `extdeps/browser/browser.dag` **(exists, on main)** | extdeps | Playwright as a service: `Launch/Goto/Fill/Click/WaitForSelector/UploadFile/…`, each `transport shell { argv: ["playwright-runner", …] }`. The argv **models the runner CLI's interface** (like `git diff` argv models git) — typed inputs → argv, never an authored shell/JS string. |
| 2 | `extdeps/bmc/webui/openbmc_kvm.dag` **(new)** | extdeps | **Cited descriptor** of the OpenBMC WebUI virtual-media page: login form selectors, the VirtualMedia nav path, the ISO-URL field, the Mount control — as *structured selector facts* citing the OpenBMC WebUI source. DOM selectors are external facts about OpenBMC's UI, so they live in extdeps beside their citation. **Not** JS. |
| 3 | `gunbc/webui_virtual_media_install.dag` **(new)** | product/workflow | The install **kernel**: a `.dag` function that *calls the layer-1 service operations in order* over the layer-2 descriptor — login → navigate → fill ISO URL → mount → set boot-once → power-cycle. This is the "business policy" (which steps, which order) per §3. Zero JS, zero hand-shell: it is a sequence of **modeled service-op calls**, the same call-form as `shell.Exec.Run(...)`. |

`extdeps.browser` currently has **zero consumers**; this kernel is its first real one (so nothing
existing breaks). Service-op calls execute their `transport shell` argv at runtime through the same
path `host_effect_realize` already uses for real `shell.Exec.Run` — so the kernel is **runnable, not
just modeled**, the moment its transport binary exists (→ §5).

## 3. How the kernel avoids both rejected patterns

- **vs. hand-written `.mjs`:** the install logic (step order, selectors, ISO URL) lives in `.dag`
  facts, not a JS file. There is no per-task JS to maintain.
- **vs. embedded-JS-emit:** the kernel never emits JS. It emits **discrete typed service-op
  invocations** (`playwright-runner goto <url>`, `playwright-runner click <selector>`), each argv
  reconstructed from typed inputs by the modeled transport — the §4 "modeled argv, read forward" path,
  not a string the model wrote.
- **Session persistence is a transport concern, not the model's.** Each `playwright-runner <verb>` is
  one process; they share a live browser via the **persistent context keyed by
  `BrowserConfig.profile_path`** (already modeled). The prior attempt leaked this persistence into the
  model as emitted JS; keeping it inside the generic runner is the §3-correct home.

## 4. Witness (DESIGN §5 — green-by-execution)

- **Model-level (lands first, no hardware):** the kernel produces the **ordered intent sequence**
  (the argv list) for srv3's descriptor; a witness asserts the exact sequence + a discriminating RED
  (drop the ISO-URL input → typed refusal, not a half-built sequence; ask for the kernel against a
  `CapabilityVirtualMedia`-true board → dispatch picks RedfishRest, not WebUiBrowser).
- **Live (gated, operator-run):** the same sequence executed against srv3's real WebUI mounts the ISO
  and the host boots the installer. Gated behind the §5 sign-off + a real `playwright-runner`.

## 5. The ONE question that must be signed before I implement

Playwright **is** a JavaScript library — *something* at the very bottom of transport-1 is JS. The two
rejections were of **task-specific** JS. A **generic, fixed, task-agnostic `playwright-runner` CLI**
(a thin 1:1 wrapper exposing `launch/goto/fill/click/wait-for/upload/screenshot` — no install logic,
no selectors, no business policy) is, I argue, the legitimate **transport realization**, exactly as
the `git` C-binary realizes the `git` extdeps shape and `sh` realizes `shell.Exec`. `browser.dag`
on main already commits to this boundary (its argv name `playwright-runner`). But that binary is
**fictional today** (dec00d4abe called it out), and it cannot itself be "modeled away" — Playwright
has no non-JS surface.

**So, sign-off needed (parent / operator):**

1. Is a generic, task-agnostic `playwright-runner` CLI an **acceptable transport realization** (the
   `git`/`sh` analogy), distinct from the rejected task-specific JS? **If yes** → who owns that binary
   (it lives outside the modeled corpus, like any external tool), and the model work in §2 proceeds.
2. **If no** (no JS may exist at all, even generic transport) → then WebUI automation is off the
   table on this stack, and the honest fallback is the firmware/PXE path or a manual attach — which is
   a lifecycle decision above my lane, and I escalate rather than improvise a third rejected shape.

I will **not** write transport-1's binary or the kernel until #1 is answered, because guessing the
boundary is precisely what got the last two attempts rejected.

## Dissolution trigger (DESIGN §6)

Delete this note when the layer-2 descriptor + layer-3 kernel land green-by-execution and the
WebUiBrowser transport is dispatched-to from the host-effect interface — at which point the carrier
(the `.dag` facts + witness) is the authority and this note is redundant.

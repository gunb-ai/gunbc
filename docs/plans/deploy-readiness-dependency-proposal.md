# Deploy readiness dependency — design proposal (for the Slice 3 bump decision)

**Status:** PROPOSAL, not a signed carrier. Authored by royal-carp-451 for calm-ferret-849 (shell→dag lane) review and the operator's Slice 3 bump decision. Does **not** edit the operator-signed `shell_emission_model.dag`; signing this into Slice 3 is the operator's act. No build / no `host_effect.dag` edits until Slice 3 is bumped and the host_effect Phase B gate is opened.

**Scope:** the ONE piece Slice 3's current dissolution plan does not cover — the readback→service-ready **dependency**. Slice 3 as written dissolves heredoc→`Filesystem.Write` and apt/systemctl/tailscale→typed argv; that fixes deploy bugs #1 and #2 but leaves #3 alive.

---

## 1. Problem — the three deploy bugs are one disease (§6 displaced cost)

`live_deploy_apply_script_for(spec)` emits the entire srv1 deploy as **one smuggled bash `String`** run through a single `shell.Exec.Run`. Every deploy main-red so far is a realization detail leaking out of that string:

| # | symptom | un-modeled thing | dissolved by |
|---|---------|------------------|--------------|
| 1 | E2BIG (`os error 7`) — >128KB server.js in argv | script **transport** (argv-embed vs stdin) | Slice 3 (heredoc → `Filesystem.Write`) + already mitigated by #6711/#6731 |
| 2 | `sh: set: Illegal option -o pipefail` | script **receiver** (sh/dash vs bash) | Slice 3 (one typed script receiver) + already patched #6746 |
| 3 | healthz readback `SyntaxError: Unexpected end of JSON input` | readback→service-ready **dependency** | **this proposal** |

Bug #3 is different in kind: it is not a missing typed-effect, it is a **missing dependency edge**. `emit.dag:105` `systemctl restart` and `emit.dag:111` the readback are adjacent doc-lines with nothing between them; `roadmap_static_site.dag:267` is a bare `curl -sf <health_url> | node -e 'JSON.parse(...)'` — no poll, no until, no retry. `systemctl restart` (Type=simple) returns as soon as node is forked, not when it is listening, so the readback races it. **Evidence it is a race, not a hard bug:** two consecutive post-#6746 main deploys — run 29505536620 FAILED at the readback, run 29505859431 SUCCEEDED. Intermittent = a missing synchronization edge by definition.

**Corollary (why converting the bash is not enough):** turning `curl | node JSON.parse` into a typed HTTP GET still races if nothing makes the GET wait for readiness. The fix is a dependency, not a transport swap.

## 2. The model — readiness as a first-class precondition on the dissolved spine

The substrate is already a dependency DAG (DESIGN §4) and there is already a reconcile spine for host effects (`host_effect_apply` / `host_effect_apply_gated`, `Reconciliation<HostEffectIntent, HostEffectEvidence>`). The deploy bypasses it with a hand-emitted string. Once Slice 3 dissolves the deploy into typed host effects, the readback stops being a trailing bash line and becomes a reconciled effect with a **declared precondition**:

- **`ServiceReady(unit, health_endpoint)` — a typed, satisfiable precondition.** Unsatisfied until the service actually serves a parseable health response at `/healthz`. It is the *postcondition of "start the service" made explicit as a dependency* — the edge the raw bash never had.
- **The readback (and the roadmap/dispatch digest checks) `depends-on ServiceReady`.** They cannot run until it holds.
- **The reconcile *satisfies* the precondition — it does not skip it or race it.** The realization of "make `ServiceReady` true" is **poll-until-healthy-or-timeout**, bounded. This is NOT a `sleep`, and NOT code inside the readback — it is the spine driving the precondition to satisfaction before the dependent effect runs (the substrate's bounded-forward execution, §4). The wait falls out of the dependency, it is not hand-authored ordering.
- **Fail-closed on timeout.** If readiness is not reached within the declared bound, the reconcile REFUSES with a typed, located diagnostic — `ServiceNotReady { unit, endpoint, waited, last_probe }` — and never proceeds to the readback, never fabricates convergence. (Contrast today: an instant `JSON.parse('')` throw mislabeled "healthz json parse failed", which blames the parser for a readiness gap.)

### The health probe is a modeled operation, not raw curl+node

- `ServiceReady`'s evidence is a **modeled HTTP GET** (`extdeps/http` — GET `health_endpoint` → typed response) realized via curl as *one* transport, not `curl -sf | node -e`.
- The JSON validity check becomes a **typed response decode** (the existing `roadmap_site_healthz_validate` JSON-parse is the decode's realization), not `node -e 'JSON.parse(...)'`.
- The two digest checks (`/ROADMAP.md`, `/target/roadmap-dispatch.json`) are the same GET→verify shape (§2 one operation, N uses) — each `depends-on ServiceReady`, not N independent racing curls.

## 3. Why the readiness edge earns its place (§6)

- **Displaces bug #3 at the root.** The edge exists in the model, so the race is *unwritable* — not caught-after-the-fact, constructed-away (DESIGN §5: construction over validation).
- **Composes.** Any future "deploy X, then verify X is serving" reuses `ServiceReady`; the project does not accrete a new racing curl per check.
- **No smuggled threshold.** The poll waits exactly until healthy or the declared bound; the bound is a *typed timeout that fails closed*, not a confidence threshold that silently proceeds (§5 — no absorbing fallback).

## 4. Integration — reuse, do not re-coin (§3)

- Reuse `HostEffectIntent` / `HostEffectEvidence` / `Reconciliation` from `host_effect_realize` — `ServiceReady` is a precondition on the intent, its evidence a successful health probe (HTTP 200 + parseable body). Mirrors the existing reconcile evidence pattern.
- One script receiver authority (bash) — the Slice 3 direction #6746 already started; the readiness edge does not fork it.
- DFS the concept DAG before minting: `ServiceReady` / the timeout `Bound` / the probe should reuse existing readiness/precondition or measure/duration types if they exist under `std`/`extdeps` before adding new names. (Flagged for the implementing slice; not pre-decided here.)

## 5. Guardrails honored (calm-ferret-849's constraints)

1. Proposal only — no edit to `shell_emission_model.dag`; the operator signs it into Slice 3.
2. Readiness modeled as a first-class precondition on the DISSOLVED spine — not a sleep, not a curl→typed-GET swap, nothing bound to today's string.
3. Bugs #1/#2/#3 cited as the displaced cost that justifies the edge.
4. No build, no `host_effect.dag` edits, until the operator bumps Slice 3 + opens Phase B.

## 6. Open question for the operator

Slice 3's dissolution is currently gated on host_effect **Phase B** (srv3 `OsInstalled` + Receipt-lock #5725, marked *escalated*). Does the `live_deploy` dissolution + this readiness edge **genuinely** need the full Phase B gate, or can it be decoupled from the srv3-OS-install work it is chained to? Decoupling would let the recurring flaky main-red be killed without waiting on the OS-install lane.

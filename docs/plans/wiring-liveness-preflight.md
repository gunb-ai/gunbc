# Plan — wiring-liveness: the input→output dependence oracle + the execution preflight

**Status:** design + direction · **DESIGN.md + carriers are authority** (§6). Linked from `ROADMAP.md` §4 *Testgen as the bug-class oracle*. Sibling of [testgen-oracle.md](testgen-oracle.md) (the bug-class→mechanism map): this is the **wiring** class worked out in full, and the first real consumer of the affected-set frontier.

**Verified against the live tree 2026-06-23.** Receipts inline; re-check before acting.

## 0. Motivation — a credential that was modeled but never wired

A production rotation run reached `acquire → mint → token-fetch`, then GCP IAM `AddVersion` returned **HTTP 401 `CREDENTIALS_MISSING`** — the REST transport sent the request with **no `Authorization` header**. The fail-closed design held (it stopped at the store, never rotated srv3), but late: the harm surfaced as a remote 401 on a *mutating* call, not as a local refusal before firing.

Root cause is a **model↔realization fork**, not a typo. The §3-correct model declares the credential as a *direct input* — `config { endpoint: "...", auth: Bearer, auth_input: access_token }` (`dag/extdeps/cloud/gcp/iam.dag:78`). `auth_input:` constructs an `svc_auth_input` field on the service-config node (`v1_std_core.rs:2829-2837`). But the interpreter's REST realization, `resolve_auth` (`src/v1/stage0/src/v1_interpreter.rs:3978-4028`), matches only `svc_auth` (scheme) and `svc_auth_source` (an **env-var name**, resolved via `resolve_env_var_token`). It **never reads `svc_auth_input`**. So `auth_input` → `env_var_name = None` → `token = None` → the attach block at `:3921` is skipped → no header → 401. The emit-rust realization is keyed the same way (`emit_rust:19351` branches on `service_config_auth_source`), so `svc_auth_input` is **under-realized across both realizations** — the model field landed ahead of its handlers. Six live services declare `auth_input:` and share the gap (`gcp/iam`, `gcp/ResourceManager`, `gcp/secret_manager`, `llm/anthropic_rest`, `llm/openai_rest`, `github/gists`).

The realization fix is someone else's lane. **This doc is about the class:** a declared input flowed into nothing, the program typechecked, and the only "auth tests" we had — `rest_emit_uses_bearer_auth` and `service_auth_source_reads_env_var` (`src/v1/tests/src/pipeline.rs`) — are `content.contains("Bearer")` greps on **emitted source** (DESIGN §5 *spec-without-execution*): they assert the emitter wrote a string, never that a request carries the header, and they cover the Bearer scheme + EnvVar source, not the `auth_input` path. There was **zero execution-grounded witness** that a declared input reaches its output.

## 1. The principle — wiring is causal dependence, and it is the cache oracle read backwards

A declared input is *wired* iff it **influences the observable output it is declared to feed.** The detector is perturbation-response — the same signal the caching kernel already uses as its purity oracle (DESIGN §6: *byte-identical cached-vs-cold is the purity oracle*). The two are duals on one kernel:

| reading | claim | violation |
| --- | --- | --- |
| **purity** (caching) | *same* input ⇒ *same* output | output depends on something **outside** its declared inputs (impure key) |
| **liveness** (wiring) | *different* input ⇒ *different* output | output depends on **less** than its declared inputs — a declared input is dead |

The cache key **is** the true dependence set. An input present in the declared signature but absent from that set is either (a) a wiring bug — it should matter and doesn't (the auth case), or (b) an over-broad cache key — it shouldn't matter and is keyed anyway. Same content-hash + perturbation machinery, two directions. This is §2-horizontal (*one concept, every breadth*): the wiring oracle is not new infra, it is the purity oracle turned around. (Carrier today: `std.dependency` `DependencyView`/`EffectDependsOn`, already exercised by `src/v2/lens/effect/effect_depends_on.dag`.)

## 2. The decidability split — wall vs confirmation (§5: "never" is the trap)

Fuzzing **cannot prove "unwired"** — non-influence across a finite sample is a sampling argument, not a proof (Rice). So the property splits along DESIGN §5's decidability line, and conflating the two would be the "never" trap:

- **Static dataflow reachability over the Node DAG — decidable, construction-side (the *wall*).** A declared input parameter with **no structural path** to any output/effect node is *unwired*, caught **without running anything**. This is the strong form: the bad state is unwritable. Primitives exist — `node_query` `arrow_domain_named_param_bindings` / `call_argument_targets` / `node_labeled_child_edges` walk param-binding → usage.
- **Perturbation / fuzz — runtime positive confirmation (the *ratchet/witness*).** Perturb the input, observe the output change ⇒ **wired, proven by execution**. This proves the *positive* and detects *candidate*-unwired (output unchanged across a sample), but never *proves* unwired. It is the residue mechanism for what static reach cannot see.

Per [testgen-oracle.md](testgen-oracle.md) §2's trichotomy, wiring is a **"wall after grounding"**: once a realization is modeled in `.dag`, static reachability makes "declared-but-unwired" unwritable; the fuzz-confirmation covers the residue that is still **opaque** — exactly the Rust-seed realizations (`resolve_auth`) that `.dag` reflection cannot see into today. As realizations migrate to `.dag` (§5/§7 self-host) the wall subsumes the residue.

## 3. Two mechanisms

**3a. The wiring-liveness witness (testgen / lens).** The execution-grounded form of §1: for every declared input→output relation, a witness that the input influences the output, RED when it doesn't. This is the generalization the auth case is one instance of — not "test that auth attaches a header" but "test that *every declared input reaches what it feeds*." It routes per the oracle map:

- **structural-reachable, modeled in `.dag`** → a **lens** over the corpus (a declared input with no path to an output = a *missing consumer* = oracle band C). Decidable, no execution.
- **opaque realization (Rust seed)** → a **testgen generator** that emits a perturbation witness: construct the call, perturb the declared input, assert the observable (the constructed request / the effect payload) changes. Discriminating RED = the auth case (perturb `access_token`, request is unchanged because the input is dropped).

**3b. Run the lens at compile time — fail compilation (construction-first); runtime-frontier only for the opaque residue.** The wiring lens is a pure reader over the Node tree, so its natural home is the *floor / compile gate*, not a runtime preflight: a dead wire becomes a **compile diagnostic that fails compilation**, the same way every other lens gates the floor. This is strictly stronger than a pre-execution check (DESIGN §5 construction-first — caught before any execution, statically, with zero runtime cost) and it answers the design question directly: *yes*, for `.dag`-modeled realizations this is one of the lenses and it fails the build, no interpreter-runtime gate needed.

The runtime preflight survives only as the **residue path for opaque realizations** the compile-time lens cannot see into (the Rust seed `resolve_auth`). There it is still negligible compute — not a fixture round-trip per effect, but static reachability over the **affected slice**: `affected_set.dag` already computes a `ReExecFrontier = ChangedSubgraphFrontier{...} | FailClosed{reason}` with `FrontierDependency { dependency: DependencyView, target }` (it isolates "the part of the graph that will run"; selection-as-CI-gate was shelved as a 0-min shadow, [testgen-oracle.md](testgen-oracle.md) §3). Running the wiring check over that frontier before executing a section, `FailClosed` on a dead wire, is the frontier's first paying consumer. As realizations self-host into `.dag` (§5/§7) this residue path dissolves into the compile-time lens — the destination is *fail to compile*, the runtime preflight is the interim.

The smallest sound increment, pure construction-side and near-free, lives one level down in the realization and is worth stating because it bounds the harm immediately even before the lens exists: when a service **declares** auth (`svc_auth` present) but the realization resolved **no** token, **refuse to send** with a typed `AuthDeclaredButUnwired` error rather than silently emitting an unauthenticated request (`dispatch_rest` currently skips the attach block at `:3921` and sends anyway). That single fail-closed guard converts the silent remote 401 into a local, typed, pre-send refusal — the preflight's principle at the narrowest seam.

## 4. Honest boundary

- **The auth realization is opaque to `.dag` today** (Rust seed). The static-reach wall (§2, §3a-lens) cannot see into `resolve_auth` until that realization is in-substrate, so for the *current* fork the catch is the §3a **generator** (perturbation witness) or the §3b fail-closed guard, not the lens. The lens lands by construction as realizations self-host. State this in the plan, don't pretend the wall covers the seed.
- **Liveness ≠ correctness.** Wiring proves the input *reaches* the output, not that the *value* is right (a Bearer token attached but malformed still 401s remotely). Wiring is the floor — "at least the wire exists" — exactly the bar the operator asked for ("ensure wiring actually works as intended"), and the right floor: it is decidable where correctness is not.
- **A legitimately-irrelevant input** (declared, genuinely not feeding any output) is *not* a bug — but it must be **declared** irrelevant, else it is indistinguishable from a dead wire. The lens output for "input X reaches no output" is a diagnostic to *resolve* (wire it, or declare it inert), not an automatic RED — same shape as the dead-scaffold dissolution-trigger discipline (§6).

## 5. Direction (dependency-ordered → ROADMAP §4)

1. **Fail-closed pre-send guard** (realization, narrowest seam) — declared-auth-but-no-token ⇒ typed refusal before send. Bounds the harm class immediately; independent of the lens. **OWNER: handed to the `resolve_auth` realization-fix lane** (the `auth_input` fix), NOT this lane — bundled there because it is adjacent to the actual fix. **FOLLOW-UP (this lane must confirm): verify the guard actually landed** — a declared-auth service whose realization resolves no token must raise a typed `AuthDeclaredButUnwired` *before send*, proven by execution (perturb to drop the token → typed error, not a remote 401). Until that confirmation, this item stays open here as a tracking obligation even though the code lives elsewhere.
2. **Wiring-liveness lens — static reachability, COMPILE-TIME (fail compilation)** (§3a-lens / §3b) — over `.dag`-modeled input→output relations, a declared input with no path to an output ⇒ a compile diagnostic that fails the floor. This is the primary mechanism (construction-first): one of the lenses, no runtime gate. Decidable, no execution. Grows to cover the seed realizations as they self-host.
3. **Wiring-liveness witness — opaque-realization generator** (§3a) — for realizations the compile-time lens cannot see into (the Rust seed), a perturbation witness: construct the call, perturb the declared input, assert the observable changes; RED when an input is dropped. The execution-grounded generalization of the auth case; the cache-purity oracle read backwards (reuse the perturbation kernel, do not re-coin).
4. **Runtime preflight — residue only** (§3b) — run the wiring check over the `affected_set` `ReExecFrontier` before the interpreter executes a section; `FailClosed` on a dead wire in the about-to-run slice. Needed only where a realization stays opaque to compile-time reflection; dissolves into item 2 (the compile-time lens) as realizations self-host. The affected-set frontier's first paying consumer.

**Execution (DESIGN §6, on merge):** these slices fan out to dependent workers in this order — item 2 (compile-time lens, the wall) is the spine; item 3 (opaque-residue generator) and the item-1 follow-up confirmation gate on the realization lane; item 4 (runtime preflight) is last and shrinks over time.

## Dissolution trigger (DESIGN §6)

Delete this doc when: (a) the wiring-liveness oracle is a single carrier shared with the cache-purity oracle (one perturbation kernel, two readings — not two representations); (b) the static-reach lens covers every `.dag`-modeled input→output relation; and (c) the preflight runs over the affected frontier ahead of interpreter execution, fail-closed on a dead wire — at which point "a declared input wired into nothing" is unwritable for modeled realizations and caught-before-firing for the opaque residue, and this tracker is redundant.

# Auth-pattern and R2-origin lanes — state at wind-down, 2026-09-20

The operator wound these lanes down on 2026-09-20 to put the system's capacity behind v1
performance and the v2 migration. This page is the record of what landed, what is left, and what
each remaining item needs — it is the carrier the roadmap rows below point at, not a second plan.

## What landed

- **The durable origin is real.** Cloudflare R2 had never been activated on the account and the
  bucket did not exist; every signed request failed its TLS handshake from three networks. The
  operator activated R2 and created `gunbai-fabric-origin` on 2026-09-20, after which
  `tools.fabric_m0_origin_object_probe` `fetch_absent` and, once the write credential was minted and
  pinned, `roundtrip` both exit 0 against the live bucket.
- **Both origin credentials are minted and pinned** (`gunbc.cloudflare.r2_origin`
  `fabric_durable_origin_standing`), and re-running either mint now refuses rather than minting a
  second token nothing reads.
- **Entitlement and bucket existence are converged** per allocated purpose
  (`gunbc.cloudflare.r2_bucket_ensure`), signed by a separately minted account-scoped bucket-admin
  token; an unentitled account refuses with the dashboard step, because Cloudflare publishes no API
  route to an R2 subscription. The boot-origin bucket was absent and was created by that ensure.
- **The authorization pattern is a modeled selection**
  (`gunbc.auth.authorization_pattern_selection`), with every current privileged-effect site
  classified by executing it (`gunbc.auth.privileged_effect_census`), and the DESIGN §3b
  conformance row extended so a reviewer asks the question of every change.

## What is left, and what each needs

1. **Nine call sites still resolve their credential outside the procedure.** Rostered in order in
   `gunbc.auth.privileged_effect_census` `follow_up_sites`. Seven are one seam each: the call-site
   `GcloudPrintToken` / `read_supplied_access_token` binding becomes
   `gunbc.auth.access_token_source` `resolve_access_token`. A migrated site must leave the roster in
   the same change, or `unreasoned_divergences_join_the_follow_up_roster_exactly` goes red.
2. **The approval authority adapter is unbuilt.** Until a redeemed capability can execute Cloudflare
   `AccountTokens.Create` and GCP IAM / Secret Manager control-plane effects, the two UNBOUND
   `DivergenceReason` rows (`interim_cloudflare_adapter_unlanded`, `interim_gcp_iam_adapter_unlanded`)
   stand, and the interim is the operator's own token in the operator's own session. Relaying a
   token through chat or between sessions is the refused pattern. Depends on the first executing
   approval consumer (mtcollins1 boot).
3. **Seven Secret Manager IAM cells are unconverged** — four for the origin write token, three for
   the bucket-admin token. Their converge entries shell out to `gcloud`, which no reachable host
   has. Until they are applied, a fleet-principal run of the write path cannot reach those secrets.
4. **A census row's characteristics are authored, not derived.** Frequency, reversibility and minted
   reach are authored per row, so a row could be made to conform by editing its inputs. The
   roster preamble's next-rung trigger should name that derivation.
5. **A stale generated workflow passed every gate.** `.github/workflows/fleet-converge.yml` lost its
   new modes to the ours side of a merge; the model was correct, the projection was stale, CI was
   fully green, and two reviewers caught it by reading. No lane regenerates that file and none fails
   on it. Regenerating it needs a host with the admitted memory budget — session containers and the
   remote runners both refuse; on srv1 the login shell has no memory cgroup either, so the entry
   runs under `systemd-run --user --scope -p MemoryMax=…`.
6. **A public wall can red every private lane with no runway.** `TestCodeReferenced` (public) admits
   exceptions only through a ledger in the public repository, keyed by module name, so a private
   overlay cannot be admitted without publishing private names. The private corpus dissolved all 274
   references immediately, which was the right outcome and was affordable — but it was forced. Either
   such walls need a per-source-root admission surface owned by the root that carries the source, or
   landing one needs a declared downstream notice period.

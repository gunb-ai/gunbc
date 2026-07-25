# dag-native SCM — node-grain source control (design seed)

Operator-directed 2026-07-25 (session gitlab-10k-scm-costs-baz90x). Roadmap carrier: §2 group
"SCM — node-grain source control, visibility-first". Economics grounding landed the same session:
DESIGN open thread "SCM economics — the GitLab 10-K corpus" (cited carriers + witnesses).

## 1. Thesis

Merges and conflicts happen at **node identity**, not file/text grain: a conflict is located at a
node, so most textual conflicts (formatting, adjacent edits, reorders) never exist, and the ones
that remain are smaller — fewer conflicts, fewer human/LLM tokens spent resolving them. The git
interface survives as a **compatibility surface**: git trees are ONE storage realization of the
node graph (ingest/emit through the same grammar-read-both-directions machinery, DESIGN §4), so
adoption cost is a realization choice, not a rewrite. The product wedge is workflow value
(visibility, typed secrecy, memoized CI), not serving cost — the GitLab corpus shows serving is
11–13% of revenue at every scale; the money is distribution.

## 2. Architecture (each piece an existing lane, composed)

- **Store**: content-addressed node store; files/repos are storage realizations (module-identity
  storage-binding thread). Binary provenance sorts three ways by construction: *derived* artifacts
  are materialization cache (regenerable, never versioned source); *structured* formats ingest to
  nodes where `DecodeFidelity` is `Lossless` (the LFS dual-store dissolves — no pointer-file
  second system, and chunked dedup replaces LFS's full-file-per-version billing shape);
  *genuinely opaque* leaves are chunked content-addressed bytes, tiered hot/cold.
- **Merge**: keyed diff over node identity — `std.change.keyed_two_way_diff`, the same spine as
  membership-reconcile (§3 one diff authority, never a text fork). Conflict = same-node divergent
  edits; refusal typed and located at the node.
- **Visibility**: the `Reference`/`Publish` grant algebra
  ([node-subtree-visibility-grants](node-subtree-visibility-grants.md)) — implemented FIRST,
  realized over git public/private roots (that doc's P-B / Stage 0), the identical interface
  carried by the native store later. Extended by the **publication ladder** (§3 below).
- **Remote realization = the SCM's CI**: a withheld/locked node executes server-side on the
  execution-as-realization spine (ROADMAP §0-④) — interface public, realization hosted, effect
  envelope declared so admission works without decryption, metered per invocation (protection
  mechanism = billing mechanism). Memoized by content hash: only affected nodes re-run.
- **Serving + economics**: serving surface rides `gunbc serve`/belt B; costs are cited carriers —
  `gunbc.econ.free_tier_serving` floors 10k free users at 2×AX41-1-LTD ≈ 115 EUR/month
  (witnessed), and GitLab's filed floor upper-bounds average serving at ≤2.42 USD/user-year.
  Metered CI compute is the abuse axis (GitLab's 400-free-minutes fence + cryptomining FAQ is the
  receipt): meter compute from day one.

## 3. Publication ladder + locked realizations (extends the visibility doc's §4/§5; design-only until Stage 0 lands)

`Publish` per node is a **rung**, not a boolean — each rung a typed capability for the audience:

| rung | audience keeps |
|---|---|
| full source | read · typecheck · execute · verify |
| emitted artifact only | execute · typecheck (not read) |
| ciphertext + interface (**locked**) | typecheck now · complete/execute on key |
| interface only | typecheck · execute remotely |
| commitment hash | verify identity/churn only |
| absence (default) | nothing — existence hidden |

**Locked nodes** (the operator's unlock-with-a-key design): the subtree ships in-artifact as ONE
ciphertext blob (internal shape hidden; pad to size buckets when size leaks), plaintext residue =
the interface at the cut + the commitment. Per-audience envelope key-wraps live on the
`AudienceScopeTree` (the storage form of the grant); key rotation re-wraps without touching
ciphertext or commitment. Decrypt-then-VERIFY against the commitment before admission — a wrong
key is a typed `KeyMismatch`, never garbage admitted as code (§5). Keyless execution reaching a
locked node refuses (`LockedNodeUnrealizable`), and a witness whose closure is blocked is a
counted `LockedBlocked` state — never failed, never silently skipped. The 99.99% compiles against
interfaces (separate compilation already required by the self-host frontier); the locked
realization arrives out of band, via key.

**Hole residues** (what redaction can never hide): a typecheckable projection irreducibly exposes
the type at the cut (coarsening it spends the public side's typecheck/effect-admission
precision); an executable projection needs a realization binding (key, artifact, or remote); a
verify-only projection needs the commitment. Per-statement holes inside *published executable
bodies* are refused — absence is not semantically neutral in code; the clean collapse is the
interface/realization seam (publish the signature, withhold the body). Data/config nodes redact
cleanly at node grain, and the strongest form is unwritability: `Secret`/`Pii`-typed content can
only inhabit `SecretRef`-shaped nodes, so publication is safe by construction (never a scanner).

**Realization placements** (the hardness dial): client+key = access control for distribution
(NOT DRM — a key-holder can always exfiltrate); remote-only = secrecy by non-delivery (the
consumer never holds the bytes; the interface remains an oracle, priced by metering); attested
TEE = the on-device middle rung, hardness bounded by enclave trust. **Churn blinding**: a naked
commitment leaks when the secret changed (sometimes a feature — audits, embargoes); a salted
(hiding) commitment re-randomized per release makes the public signal carry zero bits while
key-holders still verify; the stable `SecretRef` wrapper is reserved for secrets-as-material —
wrapping CODE in a stable pointer breaks content-addressed determinism and public verifiability
(stable identity × changing secret × public verifiability: pick two). **Crypto-shred** (key
destruction) is the erasure story for PII-typed nodes — the deletion mechanism content-addressed
immutable history otherwise lacks. Uses unlocked, in order: EE-features-in-one-artifact (license
key = typed unlock, no `ee/` fork), embargoed fixes (release the key, not a new artifact),
per-client subtrees, PII crypto-shredding.

## 4. Sequencing (visibility first — operator plan 2026-07-25)

1. **NOW** — roadmap `2-scm-visibility-stage0`: the grant model implemented properly, realized
   over git public/private roots (visibility doc P-A/P-B: two git storage roots, file-grain
   `Publish`, push-time guard, one-time `World` stamp of the existing public corpus).
2. Ladder + locked realizations land as model + witnesses only after Stage 0 (roadmap
   `2-scm-publication-ladder`).
3. Node-grain merge core + git-realization compatibility (`2-scm-node-merge`), riding the
   module-identity storage-binding authority.
4. Remote realization / metered CI (`2-scm-remote-realization`) and the store/serving infra
   (`2-scm-infra-econ`).

## Dissolution trigger (DESIGN §6)

This doc dissolves into the visibility design doc (grant/ladder content), the module-identity
storage-binding design (store content), and registered `gunbc.plan.Plan` rows as each roadmap
node lands with executing witnesses; it must not outlive Stage 1 as a parallel ledger.

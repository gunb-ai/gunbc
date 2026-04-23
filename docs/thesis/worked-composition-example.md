# Worked Composition Example — Five Services, One Program

> **Parent:** `THESIS.md` §"Omni-emission" + §"Correctness is structural,
> not behavioral (meta-claim)"

## What this doc does

One concrete real-world integration surface (an automated GitHub issue
classifier using five external services), modeled in `.dag`, with every
correctness guarantee falling out as a consequence of the structural
modeling itself — no tests required, no per-service glue, no drift as
the surface grows.

**The ah-ha moment.** When services compose, the compiler composes their
guarantees too. The number of services your program touches does not
change the effort required to keep them all honest together.

## The scenario

A scheduled job runs every 5 minutes:

1. Poll GitHub for new issues in a watched repo
2. Send each issue body to Claude for classification (priority, category)
3. Record the classification in Postgres
4. Archive the issue + classification JSON to GCS
5. Post high-priority classifications to a Slack channel

Five external services. Five authentication models. Eight effect
surfaces (read GitHub + write GCS + write Postgres + call Anthropic +
write Slack + time-of-day scheduling + log + trace). Four kinds of
auth (PAT, API key, service account, webhook URL). Typical
small-company integration — boring, real, and bug-prone.

## The five services and their quirks

Each service brings its own compositional model with its own peculiar
constraints:

**GitHub API.** Issues may span multiple pages. `issue.user` can be null
(user deleted their account). `issue.body` can be null, or up to 65536
chars. Rate-limited (5000 req/hour authenticated). GETs safe to retry;
POSTs need idempotency keys.

**Anthropic API.** Context window up to 1M tokens (Opus). `messages`
must be non-empty. Structured output via `tool_use` requires a matching
schema on both sides. Idempotency keys available but optional.

**Postgres.** Schema must match code; migrations are a separate artifact.
UNIQUE violations are runtime failures. Transactions have explicit
commit/rollback. Parameterized queries only.

**GCS.** Object names disallow certain characters (`[`, `#`, `?`, `*`).
Per-object size cap (5 TB). Metadata key/value limits. IAM scopes
required per operation. PUT with key is idempotent.

**Slack.** Webhook URL *is* the secret (no auth header). Block Kit is
nested with character limits at multiple levels — section text ≤ 3000,
total message ≤ 40000, empty blocks = 400. Posts are not idempotent.

All five have completely different failure modes, completely different
type systems, and are usually pulled in as independent Rust / Python
libraries whose type systems do not know about each other. At the point
in your workflow where they interact, a traditional compiler sees no
more than `String`, `Bytes`, `Result`, and `Option`.

## The gunbc declarations (authored once)

```dag
// ─── GitHub API types + service ──────────────────────────────────────────
type IssueState = Open | Closed

type Repository {
  owner:      GitHubUser                  // owner must exist for repo to exist
  name:       BoundedString<100>          // GitHub's repo-name limit
  full_name:  BoundedString<200>          // "{owner}/{name}" — derived
}

type Issue {
  number:      Nat
  title:       BoundedString<256>         // GitHub's issue-title limit
  body:        BoundedString<65536>?      // nullable + length-capped
  state:       IssueState
  user:        GitHubUser?                // null if account deleted
  assignee:    GitHubUser?
  labels:      List<Label>
  created_at:  Timestamp
  updated_at:  Timestamp
}

type GitHubUser {
  login:       BoundedString<40>          // GitHub username limit
  id:          Nat
}

service GitHubApi extends RestService {
  base_url:    "https://api.github.com"
  auth:        GitHubPat | GitHubApp
  rate_limit:  RateLimit<5000, Hour>

  fn list_issues(
    owner:      BoundedString<40>,
    repo:       BoundedString<100>,
    state:      IssueState         = Open,
    since:      Timestamp?         = None,
    page:       Nat                = 1,
    per_page:   Nat<1..100>        = 30,
  ) -> Paginated<Issue>
    via rest::get("/repos/{owner}/{repo}/issues")
}

// ─── Anthropic API types + service ──────────────────────────────────────
type AnthropicModel   = ClaudeOpus4_6 | ClaudeSonnet4_6 | ClaudeHaiku4_5
type MessageRole      = User | Assistant

type ContentBlock = TextContent(BoundedString<200_000>)
                  | ToolUseContent(ToolUse)
                  | ToolResultContent(ToolResult)

type LlmMessage {
  role:    MessageRole
  content: NonEmpty<ContentBlock>         // Anthropic rejects empty content
}

type ToolDefinition {
  name:          BoundedString<64>
  description:   BoundedString<1024>
  input_schema:  JsonSchema               // typed schema, not arbitrary JSON
}

type IssuePriority = Low | Medium | High | Critical
type IssueCategory = Bug | Feature | Docs | Question | Other

type ClassificationOutput {
  priority:  IssuePriority
  category:  IssueCategory
  summary:   BoundedString<500>
}

service AnthropicApi extends RestService {
  base_url: "https://api.anthropic.com/v1"
  auth:     Secret<AnthropicApiKey>

  fn create_message(
    model:       AnthropicModel,
    messages:    NonEmpty<LlmMessage>,
    max_tokens:  Nat<1..max_for_model(model)>,   // refinement per model
    tools:       List<ToolDefinition>     = [],
    system:      BoundedString<200_000>?  = None,
  ) -> MessageResponse
    via rest::post("/messages")
}

// ─── Postgres types + service ───────────────────────────────────────────
// Schema is the type declaration. Migrations emit from this, not the
// other way around.
type IssueClassification {
  repo_full_name:   BoundedString<200>    // shape matches Repository.full_name
  issue_number:     Nat
  classified_at:    Timestamp
  priority:         IssuePriority
  category:         IssueCategory
  summary:          BoundedString<500>
  anthropic_model:  AnthropicModel
  tokens_used:      Nat
}

service IssueClassificationTable extends PostgresTable<IssueClassification> {
  unique_key:  [.repo_full_name, .issue_number]
  indexes:     [(.classified_at, Descending)]
}

// ─── GCS types + service ────────────────────────────────────────────────
service GcsService extends GcpService {
  base_url:        "https://storage.googleapis.com"
  auth:            GcpServiceAccount
  required_scopes: [StorageScope.ObjectsCreate, StorageScope.ObjectsRead]

  fn put_object(
    bucket:       BucketName,
    key:          GcsObjectName,          // excludes invalid chars by type
    body:         Bytes<≤ 5_000_000_000_000>,
    metadata:     Map<BoundedString<128>, BoundedString<2048>>,
    content_type: MimeType,
  ) -> GcsObject
    via rest::put("/upload/storage/v1/b/{bucket}/o")
}

// ─── Slack types + service ──────────────────────────────────────────────
type MarkdownText = BoundedString<3000>   // Slack's per-section limit

type SlackBlock = SectionBlock(SectionBody)
                | DividerBlock
                | ContextBlock(ContextBody)

type SectionBody {
  text:    MarkdownText
  fields:  BoundedList<MarkdownText, 0..10>
}

service SlackWebhook {
  fn post_message(
    url:     Secret<SlackWebhookUrl>,     // URL is the secret
    blocks:  NonEmpty<SlackBlock>,        // empty = 400
    text:    BoundedString<40000>,
  ) -> SlackResponse
    via rest::post(url)
}
```

That's the entire model surface. About 150 lines of declarations. It is
also the complete specification of the system — there is no additional
OpenAPI schema, migration file, type-sharing crate, or mock harness. All
of those are emitted from the above.

## The workflow — user intent (≈30 lines)

```dag
workflow classify_new_issues(config: Config) {
  // 1. List new issues from GitHub (paginated; compiler walks all pages)
  let recent = GitHubApi.list_issues(
    owner:  config.repo.owner,
    repo:   config.repo.name,
    state:  Open,
    since:  config.last_scan_at,
  )                                         // returns Paginated<Issue>

  fold(recent.all_pages(), init: 0, fn: (count, issue) => {
    // 2. Classify via Claude
    let classification = AnthropicApi.create_message(
      model:       ClaudeSonnet4_6,
      system:      Some(config.classifier_system_prompt),
      messages:    [LlmMessage { role: User,
                                 content: [TextContent(format_for_classification(issue))] }],
      max_tokens:  1024,
      tools:       [config.classification_tool],
    )
    let output = extract_classification(classification)

    // 3. Upsert into Postgres (UNIQUE on repo+number makes retry safe)
    IssueClassificationTable.upsert(IssueClassification {
      repo_full_name:   config.repo.full_name,
      issue_number:     issue.number,
      classified_at:    now(),
      priority:         output.priority,
      category:         output.category,
      summary:          output.summary,
      anthropic_model:  ClaudeSonnet4_6,
      tokens_used:      classification.usage.total_tokens,
    })

    // 4. Archive to GCS (idempotent by key)
    let key = GcsObjectName.of([
      config.repo.full_name,
      show(issue.number),
      show_rfc3339(now()) ++ ".json",
    ])
    GcsService.put_object(
      bucket:        config.audit_bucket,
      key:           key,
      body:          encode_json(AuditRecord { issue, classification: output }),
      metadata:      { "priority": show(output.priority) },
      content_type:  MimeType.ApplicationJson,
    )

    // 5. Notify Slack if high priority (non-idempotent)
    match output.priority {
      High | Critical => SlackWebhook.post_message(
        url:     config.high_priority_slack,
        blocks:  build_priority_blocks(issue, output),
        text:    format_slack_fallback(issue, output),
      ),
      _ => (),
    }

    count + 1
  })
}
```

That is the entire program surface against five services. ~30 lines of
workflow + ~150 lines of declarations. This is the complete spec.

## What the compiler proves — downstream of the modeling, not the programmer

Every guarantee below is a structural consequence of the declarations
above, not something the programmer added. Each is either impossible by
construction or a compile error. No tests required.

### Arity propagates across every service boundary

- `Issue.user: GitHubUser?` → any access to `issue.user.login` forces
  pattern-match. You cannot accidentally assume the user exists.
- `Paginated<Issue>` is a Cardinality-bearing container: `.all_pages()`
  returns a flat `List<Issue>` that may be empty; `fold(empty, init, _)`
  returns `init` by law. No "assume at least one issue" bug.
- `NonEmpty<ContentBlock>` on `LlmMessage.content` → you cannot pass
  empty content; Anthropic's 400 becomes a compile error at
  construction site.
- `BoundedList<MarkdownText, 0..10>` on Slack `SectionBody.fields` → a
  list of eleven markdown elements is a compile error.
- Composition: issue → classified → audit → notify, each carries its
  own cardinality; the compiler tracks them through every step. No
  refinement is lost at a boundary.

### Bounded-size limits propagate through composition

- `issue.body: BoundedString<65536>?` is passed to
  `format_for_classification`. The output's bound is *derived* from the
  concatenation algebra, not asserted. If the resulting `TextContent`
  would exceed `BoundedString<200_000>`, the error fires at the concat
  site, not at Anthropic's runtime 400.
- `MarkdownText = BoundedString<3000>`. Slack section text is typed-
  bounded; the block-builder's outputs are type-checked against this
  bound at every step.
- `max_tokens: Nat<1..max_for_model(model)>` — refinement depends on
  the model selected. The compiler knows `Opus = 1_000_000` max;
  `Haiku = 200_000` max. Callers cannot pass 300K to Haiku.

### Idempotency composes automatically from transport + path algebra

| Operation | Structural derivation | Algebra |
|---|---|---|
| `GitHubApi.list_issues` | `rest::get` | Read → **Idempotent** |
| `AnthropicApi.create_message` | POST, but no idempotency-key in request type | **Breaking** |
| `IssueClassificationTable.upsert` | `unique_key` declared → ON CONFLICT semantics | **Idempotent** |
| `GcsService.put_object` | `rest::put` + key in path | Upsert → **Idempotent** |
| `SlackWebhook.post_message` | `rest::post`, no key | **Breaking** |

**Consequence:** if anyone wraps the workflow in `retry_on_failure(...)`,
the compiler errors — Slack is Breaking, so the composition is
unsound. The programmer is told: factor the idempotent prefix, run
Slack post outside the retry boundary. The architectural fix is
structurally required.

### Effects are computed, not asserted

The compiler walks the workflow body and aggregates the effect shapes
up the call graph:

```
workflow classify_new_issues:
  [Http.read(GitHub),
   Http.read(Anthropic), Anthropic.token_spend,
   Http.write(Postgres), Db.upsert(IssueClassificationTable),
   Http.write(GCS), Gcp.storage.objects.create,
   Http.write(Slack), Slack.message.post]
```

This is the enumerated effect set. If the workflow declares a narrower
effect surface in its signature, the compiler compares and errors on
mismatch (class 14, unenumerated effects). If the declaration is broader
than needed, the compiler warns over-declaration.

### Secrets cannot escape the graph

- `config.high_priority_slack: Secret<SlackWebhookUrl>` is an opaque
  nominal type. It flows only to `rest::post(url)` sites.
- `info_log("posted to " ++ show(config.high_priority_slack))` →
  compile error: `Secret<T>` has no `Show` instance; no coercion to
  `String`; no `Debug` derive. Cannot reach the logger.
- `Secret<AnthropicApiKey>` is a distinct nominal type; it does not
  coerce to `Secret<SlackWebhookUrl>` or to `String`. Each service's
  secrets are walled off by type.
- A typed bracket around deployment prevents the emission of any
  deploy artifact (k8s manifest, env var file, terraform plan) that
  would cause a Secret to cross a process-boundary in plaintext.

### IAM scopes derive from reachable operations, exactly

The compiler walks the workflow, collects every service call, and
aggregates the `required_scopes` field of each service binding:

```
Required IAM scopes derived from classify_new_issues:
  GCS:       storage.objects.create   (from GcsService.put_object)
  Postgres:  cloudsql.client          (from Postgres connection pool)
```

That's the complete set. The deploy spec's granted role is checked
against this derived set:

- If the granted role includes these scopes → compile check passes
- If it includes **more** → compile error: over-privileged
- If it includes **less** → compile error: under-privileged

Least-privilege IAM is mechanically enforced. No periodic audit, no
service-account creep, no "oh, we gave it `roles/editor` because it was
easier."

### Cross-service schema consistency is maintained by construction

`BoundedString<200>` on `Repository.full_name` is the same type as
`IssueClassification.repo_full_name` is the same type as the segment in
`GcsObjectName.of([...])`. One declaration. Three call sites.

If someone increases the GitHub bound to `BoundedString<300>` (because
a new username policy), either:

- Postgres migration auto-updates the column type → compatible
- Postgres column stays at 200 → compile error: `BoundedString<300>`
  cannot pass into a `BoundedString<200>` target

The conventional "migrate the schema and hope the code catches up" dance
becomes a compile-time gate. Drift is impossible because there is no
second authority to drift against.

### Exhaustive variant handling across versions

Adding `IssuePriority.Critical` to the enum:

- `match output.priority { High | Critical => ... _ => () }` stays
  valid — the `_` covers new variants.
- Any match site in the codebase that *explicitly* enumerated
  `Low | Medium | High` without `_` fails to compile until `Critical`
  is handled.
- The programmer sees, at compile time, every site that needs updating.
  No forgotten case. No runtime surprise.

### Rate limits produce paced clients automatically

`GitHubApi.rate_limit: RateLimit<5000, Hour>` is structural. The
emitted GitHub client paces calls accordingly; exposes rate-limit
state on `Paginated<Issue>` for dashboards; throws a structured error
(not a 429) only on hard limits. The workflow writer never writes
rate-limit code.

## What the compiler generates — from the same declarations

Every artifact below is emitted from the workflow + service declarations
above. Writing the workflow is the only programmer action. All of the
following are emitted:

- **Rust client stubs** for GitHub, Anthropic, Postgres, GCS, Slack —
  each with typed request/response, typed error unions, rate-limit
  handling
- **Python client stubs** with identical behavior, idiomatic Python
- **TypeScript client stubs** if a browser frontend needs them
- **Go client stubs** for Go-side consumers
- **Postgres migration** (CREATE TABLE, indexes, UNIQUE constraint)
  from `IssueClassification` declaration
- **OpenAPI spec** if the workflow is exposed as an HTTP endpoint
- **Deploy spec** (k8s manifests, IAM role + service account bindings,
  secret references)
- **Retry wrappers** with correct per-op semantics
- **Mock harnesses** per service — usable in tests
- **Auto-generated test claims** for each cardinality transition and
  effect boundary (e.g., "when list_issues returns empty, classify_new
  does nothing," "when Anthropic returns max_tokens error, workflow
  surfaces it typed," "when GCS rejects the path, the workflow fails
  closed")
- **IAM policy document** scoped exactly to reachable operations
- **Structured error union** covering every service's failure modes,
  unified through the thesis's `Result` / `Diagnostic` algebra

The programmer writes none of these. As services are added, each
artifact updates automatically.

## The traditional Rust version (sketch)

For honesty, what the same system looks like in Rust today:

```
Cargo.toml — 15+ dependencies:
  reqwest, serde_json, tokio, sqlx, google-cloud-storage,
  octocrab, anthropic-rs, slack-morphism, secrecy, tracing,
  tokio-retry, anyhow, tower, ...

src/
  github/       types, client, rate-limit glue, pagination       ~300 LOC
  anthropic/    types, client, token-counting, tool-use parsing  ~250 LOC
  db/           schema duplicate, queries, migrations, pool      ~200 LOC
  gcs/          client, path builder, retry wrapper              ~150 LOC
  slack/        block builder, webhook client, escaping rules    ~150 LOC
  workflow/     business logic — maps to the 30 lines of .dag    ~150 LOC
  types/        shared types across five APIs; must stay in sync ~200 LOC
  errors/       error union across five services                 ~100 LOC
  mocks/        per-service test doubles                         ~200 LOC
  tests/        integration tests, fixtures, boundary cases      ~400 LOC

migrations/     Postgres schema migration DSL, hand-written      ~100 LOC
terraform/      IAM policy, service accounts, secret bindings    ~200 LOC
openapi.yaml    if service is exposed; hand-maintained           ~150 LOC

Total: ~2,400 LOC across 20+ files + infra config.
```

Every file must stay in sync with the others. The type-sharing crate
must track all five upstream APIs. The mocks must match the real clients.
The IAM policy must match what the workflow actually reaches. The
migrations must match the Rust type definitions. The tests must cover
the boundaries. None of these are checked by the Rust compiler; they
are checked (often partially) by CI and (always partially) by review.

## Side-by-side bug surface

Each row: a real production bug pattern, how Rust handles it, how gunbc
handles it. "runtime" = it fires when the code runs; "CE" = compile
error; "IBC" = impossible by construction.

| # | Bug | Rust | gunbc |
|---|---|---|---|
| 1 | `issue.user` null → `user.login` access | runtime panic | CE at access site |
| 2 | GitHub returns an extra field not in Rust type | `serde` silently drops; bug hides | CE when declaration updated; all consumers re-checked |
| 3 | Anthropic prompt > context window | runtime 400 | CE if bound derivable; else fail-closed with typed error |
| 4 | GCS path contains `#` (invalid char) | silent upload to wrong path | CE at `GcsObjectName.of([...])` construction |
| 5 | Postgres UNIQUE violation on retry | runtime error caught + re-raised | upsert algebra — retry is safe by construction |
| 6 | Slack webhook URL logged | secret in CloudWatch | CE: `Secret<T>` has no `Show` |
| 7 | Service account has `roles/editor` (over-privileged) | security audit catches it months later | CE: granted scopes ⊋ derived scopes |
| 8 | Rust type drifts from Postgres column type | runtime "column not found" | impossible — single declaration for both |
| 9 | Anthropic `tool_use` returns schema mismatch | runtime deserialization failure | input/output schemas typed; structural match |
| 10 | `retry_on_failure` wraps whole workflow | duplicate Slack posts on transient failure | CE: Slack post is Breaking in composition |
| 11 | Rate limit 429 bursts | hand-rolled backoff code | paced client emitted; no bespoke code |
| 12 | Postgres transaction leaks on exception | pool exhaustion over time | linear `ResourceHandle` — CE if not committed/released |
| 13 | Slack section block > 3000 chars | runtime 400 | CE: `MarkdownText = BoundedString<3000>` |
| 14 | Pagination loop misses the last page | silent missing issues | `Paginated<T>` iterator closes the loop structurally |
| 15 | `issue.body` null → concat produces `"None"` | Display derives stringified debug | CE: `Option<BoundedString<_>>` can't concat without match |

None of the gunbc outcomes require tests. Each is either IBC or a
compile error.

## The ah-ha moment — effort does not scale with services

**Adding a sixth service** — say, a DataDog metrics emitter so we can
dashboard classification throughput:

**In Rust:** add a new crate (~50 LOC of client types), wire it into
the workflow (~30 LOC), update the error union (~20 LOC), add retry
policy (~20 LOC), update mocks (~30 LOC), update IAM if the service is
on GCP (~10 LOC), add tests (~50 LOC). Net: ~200 LOC, 6 files touched,
1 new dependency, and four new places to remember for future changes.
The coupling between DataDog and the other five services is latent:
future maintenance will touch one and silently degrade the others.

**In gunbc:** declare the service (~15 LOC of types — DataDog API
types, the `DataDogMetrics` service binding with its auth and scopes),
add the call in the workflow (~5 LOC). Total: 20 LOC. No new retry
wrapper (emitted). No new error union (emitted). No new mock (emitted).
No IAM update (scope automatically added to derived set). No tests
(emitted from the new cardinality transitions and effect boundaries).

**Integration effort is linear per service in gunbc; it is
super-linear in Rust** because every pairwise interaction (DataDog ×
GitHub, DataDog × Anthropic, ...) is a new bug surface that must be
independently checked.

This is the central point. As your integration surface grows from 5 to
15 to 50 services, Rust's maintenance load grows super-linearly because
every new pair-interaction needs attention. gunbc's maintenance load
stays per-service because the composition is the compiler's problem,
not yours.

## Summary — what falls out, for free

The programmer wrote:

- ~150 lines of type + service declarations
- ~30 lines of workflow business logic
- Zero lines of retry code
- Zero lines of client glue
- Zero lines of schema migration
- Zero lines of IAM policy
- Zero lines of mock harness
- Zero lines of error-union plumbing
- Zero lines of rate-limit handling
- Zero lines of tests

From those 180 lines, the compiler delivers:

- Complete arity propagation (no unexpected empties, null derefs, out-of-bounds)
- Complete idempotency composition (retry wraps are structurally checked)
- Complete effect enumeration (declared = actual, or compile error)
- Complete secret flow tracking (no leaks to logs / outputs / telemetry)
- Complete IAM scope derivation (exact least-privilege, mechanically)
- Complete schema consistency (no drift, client/server/DB agree)
- Complete exhaustiveness (enum additions surface every caller)
- Complete test coverage at service boundaries (auto-generated)
- Complete client emission for every target (Rust / Python / Go / TS)
- Complete error-union tracking (typed per-operation)
- Complete retry semantics per operation (algebra-driven)
- Complete IAM policy document for deploy
- Complete OpenAPI spec (if exposed)

And if the integration surface doubles tomorrow, none of the above gets
worse.

## What this supersedes

This example replaces the class-by-class audit approach of the draft
`impossible-bug-classes.md` doc. Every class in that audit (null
dereference, schema drift, unenumerated effects, retry-unsafe
composition, secret leak, IAM over-privilege, etc.) is a *consequence*
of the structural modeling shown here — not an independently-prevented
list.

The pitch shifts from *"we prevent 26 specific bug classes"* (invites
nit-pick about each one) to *"write the declarations; the correctness
composes"* (invites engagement with the model).

## Honest caveats

Several constructs in this example are the **target state**, not the
state of gunbc's tree today:

- `BoundedString<N>` — type-alias refinement is partially landed via
  DB-3 (`src/v3/std/dimensions.dag` is the live framework; refinement-
  as-constraint is the follow-up work).
- `NonEmpty<T>` as a first-class substrate type — not yet; currently
  `List<T>` + caller-side match on emptiness (class 2 in the audit).
- `Paginated<T>` iterator collapse — conceptual; depends on Cardinality
  refinement composition (class 3 in the audit, GAP).
- `Secret<T>` as a nominal opaque wrapper — not yet; currently
  `Secret = String` alias (class 10 in the audit, GAP).
- IAM scope derivation to deploy spec — not yet; conceptual.
- `RateLimit<N, Window>` producing a paced client — not yet;
  conceptual.
- Effect enumeration walker — not yet (class 5 in the audit, GAP).
- Linear `ResourceHandle` typing — not yet (class 16 in the audit,
  PARTIAL).

Each of these is a named structural path; none requires runtime-only
mitigation. The example is the *destination*. The audit in
`impossible-bug-classes.md` (if it stays) is the current-state ledger.

When each substrate piece lands, it unlocks the corresponding part of
this example. The order of landing does not change the model; it only
changes how much of the example is mechanically enforced today.

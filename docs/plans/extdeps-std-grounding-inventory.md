# extdeps + std grounding-defect inventory

Exhaustive pass over **every dag/extdeps + dag/std module** (451 files, classified by 46 parallel
auditors + a 24-claim adversarial verify pass; 71 agents, 0 errors). Companion to
[formal-concepts extdeps grounding](formal-concepts-extdeps-grounding.md) — the operator directive
(2026-07-24) that *location is cosmetic; grounding is the real defect class*. This doc prices that
class up front so nobody rediscovers it post-flip. Findings are execution-checked, not grep-guessed;
each row carries a suggested grounding and its own dissolve-on.

> **Read the `verify` column first.** On a REFUTED row the flagged defect did **not** hold under
> adversarial re-check — the suggested action is struck and the row is kept only as a recorded
> false-positive. Do not action a REFUTED row. PARTIAL rows are action-with-caveat.

## Summary — 203 defects across 451 modules

| class | total | extdeps | std | what it is |
| --- | --- | --- | --- | --- |
| **s3-fork** | 16 | 6 | 10 | duplicate-models a concept that has another home |
| **std-to-extdeps-table** | 8 | 0 | 8 | cited data rows sitting in std that belong in extdeps |
| **citation-gap** | 20 | 5 | 15 | models a real upstream spec with no cited source/version |
| **numeric-anemia** | 56 | 50 | 6 | bare Int for a dimensional magnitude (time/size/rate) — should ground on `std.measure` |
| **string-anemia** | 86 | 73 | 13 | bare String hiding named structure (the `LGA4926` class) |
| **other** | 17 | 13 | 4 | misc grounding/decomposition notes |
| **total** | 203 | 147 | 56 | |

The spread is the point: grounding defects are **everywhere**, in both trees — location did not
predict them (extdeps carries the bulk simply because it is the larger tree, 349 files to std's 102).
The two most actionable classes (verified below) are the §3-forks and the std→extdeps tables; the
anemia classes are the wide long tail.

## 1. §3-forks — duplicate-modeled concepts (16 flagged · 14 CONFIRMED · 2 refuted)

The real §3 violations: a concept modeled in two homes. Adversarially verified (a legitimate
std-surface / extdeps-cited-rows split is *not* a fork and was refuted). REFUTED rows carry the
refutation so no one re-actions them.

| module | location | the fork (REFUTED = why it's actually fine) | verify |
| --- | --- | --- | --- |
| `std.languages` | data rust_language/rust_spec/rust_* (and go_*/py | std/languages.dag re-models Rust/Go/Python/TS syntax, type-mappings and primitives that dag/extdeps/languages/<lang>/ already model — two homes for one per-language spec (verified: extdeps/l | CONFIRMED |
| `std.types` | Milliseconds (line 151), Seconds (line 152), Dur | Duplicate-models time units that std.measure already owns: std.measure.Millisecond (Measure<Time,Milli,Nat>) and std.measure.Second (Measure<Time,One,Nat>) are the single authority (temporal | CONFIRMED |
| `std.types` | SemVer (line 127) | `SemVer = String` forks extdeps.version.semver, which already models the semver.org spec faithfully as SemVerVersion { major, minor, patch, prerelease, build } cited to `semver.org/`; the St | CONFIRMED |
| `extdeps.formats.elf.segments` | PhdrDecodeOutcome<T> / PhdrDecodeReason | Parallel decode machinery duplicating types.dag's generic ElfDecodeOutcome<T>/ElfDecodeReason: PhdrDecoded/PhdrRejected mirror ElfDecoded/ElfRejected, PhdrTruncated{required_bytes,available_ | CONFIRMED |
| `extdeps.gnu_coreutils` | diff_recursive_flags / diff_recursive_brief_flag | The diff operation is forked here: `diff -r` / `diff -rq` flags live in the coreutils module while extdeps.diffutils (extdeps/shell/diffutils.dag) already models diff as the `shell.Diff.Recu | CONFIRMED |
| `extdeps.gunbc` | packages (List<WorkspacePackage>) / WorkspacePac | ~~A hand-drawn roster of the Rust workspace crate layout (gunbc-*, daglang-* with CrateRole) duplicate-models th~~ REFUTED: The s3-fork test — "BOTH homes define the SAME concept with overlapping/duplicate structure" — is not met. Reading dag/extdeps/gunbc/gunbc.dag: `packa | REFUTED |
| `extdeps.languages.rust.types` | integer_types / float_types | Bare String lists of Rust integer/float type names duplicate the type-name catalog already modeled as RustPrimitive.target_name rows in extdeps.languages.rust.primitives (rust_grounding_prim | CONFIRMED |
| `extdeps.llm.cli` | service claude.Invoke (operation Run, cli.dag:15 | The claude.Invoke.Run shell-transport argv re-spells the exact claude-session invocation flag sequence (permission_args ++ --settings/--effort/--session-id/-n/--append-system-prompt/start_pr | CONFIRMED |
| `std.baseboard` | BaseboardManufacturer = AsrockRack | Re-coins a hardware vendor as a freshly minted single-variant enum; DESIGN §3 dissolves 'vendor' into a single Vendor<Domain> authority whose cited company entities live one-file-each in ext | CONFIRMED |
| `std.markdown_markup` | markdown_inline_to_markup_nodes | Re-implements the MarkdownInline -> HTML-element mapping (code/em/strong/a/img tag map) already present in std.markdown.markdown_inline_to_fragments, differing only in target vocabulary (Mar | CONFIRMED |
| `std.markup` | MarkupNode/ElementNode/TextNode/MarkupAttr vs Fr | Two byte-identical-shaped vocabularies for the same HTML element tree: ElementNode{tag,attrs,children} == Element{tag,attributes,children}, TextNode == FragmentText, MarkupAttr == Attribute  | CONFIRMED |
| `extdeps.cloud.gcp.secret_manager` | SmReplica.location: String | ~~A replica location is a GCP region, a concept already modeled in extdeps.cloud.gcp.gcp (GcpRegion + the cited ~~ REFUTED: The s3-fork test requires both homes to define the SAME concept with overlapping/duplicate structure (a duplicate decl / nickname). That is absent. se | REFUTED |
| `std.bit` | Word8 / Word16 / Word32 / Word64 / Word128 | Five separate hand-rolled width types duplicate the machine-width axis std.machine_constraints already parameterizes as MachineWidth<N> (std.integer: Int8..UInt128 = Compose<Int, MachineWidt | CONFIRMED |
| `std.cache_identity` | CacheInterfaceProduct | "Which cache backend" is modeled twice: a closed enum here vs the branded-string identity std.cache_interface.CacheInterfaceId used throughout CacheInterfaceFacts/CacheLayerPlan. | CONFIRMED |
| `std.filesystem` | fn is_text_encoding(e: Encoding?) | std.filesystem re-defines is_text_encoding, which already exists as std.encoding.is_text_encoding(e: Encoding); this is a second (Optional-lifted) copy of the same text-vs-binary predicate w | CONFIRMED |
| `std.node` | compiler_inductive_fields (InductiveField rows) | Hand-authored table restates the inductive-field structure the Node/InferredNode/MatchPattern/MethodSemantics type declarations already carry (dual representation), using bare-String referen | CONFIRMED |

## 2. std→extdeps tables — cited data in std (8 flagged · 4 CONFIRMED · 3 partial · 1 refuted)

Cited data rows sitting in `dag/std/` that belong downstream as extdeps rows — the framework stays,
the cited table moves. The house exemplars (unicode blocks, RFC method tables). **The suggested
action is authoritative only for CONFIRMED rows**; PARTIAL is action-with-caveat; REFUTED means the
split is already correct — no migration.

| module | location | the table | verify | action |
| --- | --- | --- | --- | --- |
| `std.baseboard` | BaseboardManufacturer (= AsrockRack) and Baseboa | Vendor-specific closed enums naming one real board (ASRock Rack ALTRAD8UD-1L2T) are cited catalog data sitting in std; e | CONFIRMED | Move the vendor model/manufacturer enum values (and the catalog rows) to extdeps/boards; keep only the agnostic shape (ServerBaseboard, generic catalo |
| `std.bmc` | BmcFirmwareFamily = \| OpenBmc | A specific cited firmware project (OpenBMC) pinned as a std enum value and matched-on for dispatch, while the cited Open | PARTIAL | PARTIAL — Home the firmware-family enum values + their dispatch in extdeps/bmc (per DESIGN §3 os pattern: std projects t (caveat: std-to-extdeps-table REFUTED, but a real §3 defect of a DIFFERENT class is present.  REFUT) |
| `std.languages` | all `data` rows (rust_language/go_language/pytho | A dag/std/* module carries the concrete per-language and per-format cited syntax/type/reserved-word tables; §3 keeps the | CONFIRMED | Move every `data *_language/*_spec/*_format/*_ops/*_syntax/*_reserved_words` row into dag/extdeps/languages/{rust,go,python,typescript,yaml,toml,css,j |
| `std.cache_identity` | CacheInterfaceProduct | A dag/std module carries a roster of specific real vendor cache products; per §3 the cited product enumeration is data t | PARTIAL | PARTIAL — Relocate the vendor variants to extdeps (e.g. extdeps/cache/{gha,sccache,buildbuddy,cargo,rustup}.dag as cited (caveat: Read dag/std/cache_identity.dag. The claim's factual predicate holds: CacheInterfaceProduc) |
| `std.os.types` | UbuntuDistributionCatalogRow / WindowsDistributi | Vendor-specific catalog-row shapes and cited distribution variant identities (NobleNumbat2404Lts, Windows1124H2Build2610 | REFUTED | **No migration** — dag/std/os/types.dag contains ONLY `type` declarations (no `data` or `fn` decls at all), so the std module carries ZERO cited data rows — the requirem |
| `std.unicode` | wide_blocks (line 58), zero_width_blocks (line 4 | A dag/std/* module carries cited upstream data rows (Unicode block start/end codepoint ranges from UAX #11 / UCD); per § | CONFIRMED | Move the block-range rows to a cited extdeps module (e.g. extdeps/unicode, cite UAX #11 + Unicode version); keep DisplayWidth, CharClass, in_block, an |
| `std.currency` | type CurrencyCode = Eur \| Usd + fn currency_min | A cited ISO 4217 code list plus its minor-unit exponent data table sits in dag/std/; the framework (Currency Quantity, M | CONFIRMED | Relocate the code enum + minor-unit table to extdeps/currency as cited ISO 4217 rows (beside the extdeps/pricing/* consumers that already import it);  |
| `std.os` | operating_system_surface_ubuntu / _windows / _ma | Cited vendor product-marketing names ("Ubuntu", "Windows 11", "macOS Sequoia") and the Windows "(OS build ...)" version  | PARTIAL | PARTIAL — Add a cited product_name (and OS-build display) field to the extdeps DistributionCatalogRow in extdeps/os/{ubu (caveat: Factual sub-claims are all TRUE but the classification is wrong. std/os.dag (32 lines) car) |

## 3. Citation gaps — real upstream, no cited source/version (20)

Modules modeling a real spec/standard/vendor-API/hardware with no citation. 19 modules
also carry `cited=false` overall: `extdeps.runtime.local`, `extdeps.uri`, `std.approximate_field`, `std.baseboard`, `std.bmc`, `std.cache_identity`, `std.currency`, `std.encoding`, `std.exec_format`, `std.http_path`, `std.languages`, `std.markdown`, `std.markup`, `std.numerical_contract`, `std.os`, `std.os.types`, `std.process`, `std.types`, `std.unicode`.

| module | location | missing citation |
| --- | --- | --- |
| `extdeps.runtime.syscalls.linux` | sys_aarch64_read / sys_aarch64_write / sys_aarch64_mmap | The aarch64 syscall numbers (read=63, write=64, mmap=222, execve=221, exit=93) come from the generic table include/uapi/asm-generic/unistd.h, but the only cited source is the x86-specific arch/x86/... |
| `extdeps.uri` | module extdeps.uri (UriScheme / Uri) | Defines UriScheme and Uri — the URI-scheme/URI-syntax model consumed corpus-wide — with no ExternalAuthority anchor or data-note, while modeling RFC 3986 URI generic syntax and the IANA URI Schemes re |
| `std.approximate_field` | RoundingMode / Precision / NanPolicy / InfinityPolicy / | Models the IEEE 754 floating-point standard verbatim (rounding-direction attributes ToNearestEven/ToPositiveInfinity/…, NaN/infinity/signed-zero/subnormal policies, GradualUnderflow/FlushToZero) but c |
| `std.baseboard` | module std.baseboard (whole module) | The std module names a real vendor board with no data-note / ExternalAuthority; the only citation for ALTRAD8UD-1L2T lives in the extdeps twin, so the std-side vendor identity is uncited. |
| `std.bmc` | BmcProtocol (= Redfish \| Ipmi) and BmcFirmwareFamily ( | Models real industry standards (DMTF Redfish, IPMI) and the OpenBMC firmware project with no data-note or ExternalAuthority in std; the extdeps twins (extdeps/bmc.dag cites the BMC authority, extdeps/ |
| `std.cache_identity` | CacheInterfaceProduct | Enum variants name real upstream vendor cache products (GhaActionsCache=GitHub Actions Cache, SccacheLocal=sccache, BuildbuddyCas=BuildBuddy CAS, CargoTargetDir=Cargo, RustupToolchainStore=rustup) wit |
| `std.currency` | CurrencyCode / currency_minor_unit_exponent | Models ISO 4217 (Eur/Usd are the standard's alphabetic codes; minor_unit_exponent 2/2 is ISO 4217's minor-unit column) with no data-note naming the source or version. |
| `std.encoding` | base64_alphabet / base64_encode / base64_decode / Base6 | The Base64 codec and its two verbatim alphabet strings (Standard and UrlSafe) are exactly the RFC 4648 tables, but the module carries no citation naming RFC 4648 or its version; only a Terminal debt m |
| `std.exec_format` | ExecutableFormatFamily (Elf \| Pe \| MachO) / Relocatio | The surface names three real binary-format specs and models spec-derived structures with no in-module citation; only ELF has a cited downstream home (extdeps/formats/elf cites the gABI), while Pe (PE/ |
| `std.languages` | data rust_reserved_words / go_reserved_words / python_r | Reserved-word lists, naming conventions and syntax templates are taken verbatim from each language's official reference with no data-note naming the upstream source or version. |
| `std.types` | HttpMethod (line 184), HttpStatus (line 118), Transport | std.types defines the HTTP method enum, status-code range, and transport request/response shapes — a real upstream spec (IETF HTTP, RFC 9110) — with no source/version note; the RFC 9110 citation lives |
| `std.unicode` | wide_blocks (line 58), zero_width_blocks (line 42), zer | The codepoint-range block tables and inline ASCII/codepoint ranges are transcribed from the Unicode Standard (UAX #11 East Asian Width + Unicode Character Database block ranges) with no data-note nami |
| `extdeps.bmc.types` | extdeps_external_authority_anchor (module extdeps.bmc.t | Encodes version-specific Redfish schema enum registries (RedfishResetType, RedfishBootSourceOverrideTarget/Enabled/Mode, RedfishPowerState, RedfishPrivilege, RedfishAccountRole) but the anchor `www.dm |
| `extdeps.runtime.errno` | errno_number | Returned values EAGAIN=11/ENOMEM=12 are Linux/glibc ABI facts (asm-generic/errno-base.h); the sole cited source (POSIX errno.h at opengroup) standardizes only the symbolic names, not the numeric value |
| `extdeps.runtime.local` | LocalDevRuntime.adc_env_var | adc_env_var = "GOOGLE_APPLICATION_CREDENTIALS" is Google Cloud's Application Default Credentials convention, but the module's external_authority anchor self-references the gunbc repo (github.com/gunb- |
| `std.http_path` | PathTemplate / UrlPathToken (ParamToken) | Models URL path templating with literal/param tokens (the `{name}` placeholder convention of a real web standard) but carries no data-note citing it. Note: the token decomposition itself is good groun |
| `std.markdown` | module std.markdown (HeadingLevel H1..H6, MarkdownAlign | Models CommonMark plus GitHub-Flavored-Markdown extensions (task lists, tables, column alignments, 6 heading levels) but carries no data-note naming the CommonMark/GFM spec or its version. |
| `std.markup` | escape_text / escape_attribute / escape_url (and unesca | Implements HTML metacharacter escaping (&amp;/&lt;/&gt;/&quot;) and URL percent-encoding (%25/%20/%3C/%22) — real upstream specs — with no citation. |
| `std.numerical_contract` | FmaContractionPolicy / NumericalContract.FloatApproxima | Models IEEE-754 floating-point contract concepts (rounding attribute, fused-multiply-add contraction) with no data-note naming the standard; FMA contraction is a real IEEE-754 / C-standard concept, no |
| `std.process` | exit_code_success / exit_code_general_error / exit_code | Models the reserved process exit-status convention (0 = success, 1 = general error, 2 = misuse of shell builtins) — a real upstream POSIX/shell standard — with no data-note naming the source or versio |
## 4. Numeric anemia — bare Int for a dimensional magnitude (56)

**Recurring root (a §3 fork under the anemia):** `dag/std/types.dag` carries branded-`Int`
time/size aliases (`Milliseconds`, `Seconds` = `Int where range(min:0)`) that FORK
`dag/std/measure.dag`'s dimensional `Measure<Time,…>`. Modules that type a field `Milliseconds`
look grounded but consume the weaker branded-Int, not the measure authority. Fixing the alias fork
at the root dissolves a whole cluster of these at once — grep the offenders below for `_ms`/`_seconds`/
`_bytes` and the `Seconds`/`Milliseconds` types.

High/medium severity (37 of 56):

| module | location | detail | suggested grounding |
| --- | --- | --- | --- |
| `extdeps.cloud.gcp.secret_manager` | max_secret_size_bytes: Int = 65536 | A SIZE (64 KiB, 65536 bytes) carried as a bare Int — exactly the size-with-unit class. | Ground on std.measure.ByteSize (Measure<Memory,One,Nat>); 65536 = 64 Kibi. |
| `std.types` | Duration (line 150), Milliseconds (line 151), Seconds (line  | Time magnitudes modeled as bare `Int where range(min:0)` with the unit smuggled into the type/field name instead of carrying a dimension. | Ground each on std.measure: Duration/Milliseconds -> Millisecond (Measure<Time,Milli,Nat>), Seconds -> Second (Measure<T |
| `extdeps.astronomy.stellar_classification` | TemperatureRange.BoundedRange.min_k / max_k and OpenAbove.mi | Effective temperatures are a physical magnitude (kelvin) carried as bare Int with the unit smuggled into the field name (_k) — the classic unit-in-nam | Ground on std.measure Temperature: min_k/max_k become Measure<Temperature, One, Int> (kelvin), the quantity std.measure  |
| `extdeps.bmc.serial_console` | SolConsoleCaptureReceipt.duration_ms | Time magnitude typed `Milliseconds`, which is a branded `Int where range(min:0)` alias in dag/std/types.dag (line 151), NOT the dimensional Measure<Ti | Retype to `Millisecond` = Measure<Time, Milli, Nat> (dag/std/measure.dag line 407) so the field grounds on the single st |
| `extdeps.bmc.serial_console` | SolConsoleSendIntent.settle_seconds / SolConsoleSendIntent.r | Both time fields typed `Seconds`, a branded `Int where range(min:0)` alias in dag/std/types.dag (line 152), not the dimensional Measure<Time,...> auth | Retype both to `Second` = Measure<Time, One, Nat> (dag/std/measure.dag line 417); root-cause is the std.types branded-In |
| `extdeps.browser` | BrowserConfig.viewport_width / BrowserConfig.viewport_height | Viewport pixel dimensions (1280/900) modeled as bare Int; a rendered display size is a physical magnitude, not a dimensionless cardinal. | Ground on a cited CSS/device-pixel magnitude (a Pixels dimension) via std.measure rather than a bare Int; keep the pixel |
| `extdeps.browser` | service browser.Page.WaitForSelector input timeout_ms: Int = | Selector wait timeout is a TIME in milliseconds carried as a bare Int. | Ground on std.measure Duration; serialize to the argv literal only at the shell transport, not in the modeled input. |
| `extdeps.browser` | service browser.Page.Wait input ms: Int | Explicit delay is a TIME in milliseconds carried as a bare Int. | Ground on std.measure Duration. |
| `extdeps.clock` | Clock.UnixSecs output unix_secs: String | `date +%s` returns Unix epoch seconds — a time magnitude — carried as a bare String, while the sibling Clock.Now operation coerces its stdout to std.t | Coerce unix_secs to std.types.Timestamp (or a std.measure epoch Second, Measure<Time,One,Nat>) mirroring Clock.Now, so b |
| `extdeps.cloud.cloud` | RateLimitPolicy.requests_per_minute: Int | A request RATE carried as a bare Int with the period baked into the field name ('per_minute' is convention standing where a unit type was available). | Ground on a std.measure frequency/rate carrier — Measure<Frequency, PerMinute, Nat> or a Rate<PerMinute> parametric reco |
| `extdeps.cloud.cloud` | CloudCredential.expires_seconds: Int? | Credential lifetime in seconds (a Time magnitude) carried as a bare Int. | Ground on std.measure.Second (Measure<Time,One,Nat>) / a Duration alias. |
| `extdeps.cloud.gcp.gcp` | GoogleOAuth2Refresh200Body.expires_in: Int (and oauth2.Googl | OAuth2 token lifetime in seconds (RFC 6749 expires_in) carried as a bare Int. | Ground on std.measure.Second / Duration while keeping the upstream field name expires_in (faithful-name §3). |
| `extdeps.cloud.gcp.iam` | TokenLifetime.default_seconds: Int | A token lifetime (3600s) carried as a bare Int seconds field. | Ground on std.measure.Second (Measure<Time,One,Nat>) / a Duration alias. |
| `extdeps.cloud.gcp.iam` | TokenLifetime.max_seconds: Int | A max token lifetime (43200s) carried as a bare Int seconds field. | Ground on std.measure.Second / Duration. |
| `extdeps.cloud.gcp.iam` | gcp.IAM.GenerateAccessToken input lifetime_seconds: Int wher | Requested token lifetime in seconds carried as a bare Int (the range refinement bounds it but does not dimension it). | Ground on std.measure.Second / Duration, keeping the range refinement. |
| `extdeps.cloud.gcp.secret_manager` | SmRotationSchedule.rotation_period_seconds: Int? | A rotation period (a Duration in seconds) carried as a bare Int. | Ground on std.measure.Second / Duration. |
| `extdeps.cloud.gcp.sts` | StsTokenResponse.expires_in (and operation Exchange output e | OAuth2/STS token lifetime is a TIME in seconds but is carried as a bare Int, dropping the Duration dimension; the module imports no std.measure. | Ground on std.measure Duration (seconds) — `expires_in: Duration` — so the second-unit is carried, not conventionally im |
| `extdeps.container.oci.descriptor` | OciDescriptor.size | size: Int is the descriptor blob size in bytes (OCI descriptor spec: 'size, in bytes, of the raw content') carried as a bare Int; sibling extdeps modu | Read the raw wire Int at the decode boundary, then carry the validated size as std.measure.ByteSize (Nat-backed) in the  |
| `extdeps.container.oci.linux` | LinuxMemoryResources.limit / LinuxMemoryResources.reservatio | limit: Int? and reservation: Int? are cgroup memory sizes in bytes (config-linux.md memory.limit/reservation) carried as bare Int; the corpus already  | Ground limit and reservation on std.measure.ByteSize? (decode raw wire Int -> ByteSize), matching the docker container_i |
| `extdeps.docker` | DockerMount.TmpfsMount.size_bytes | Bare Int for a byte SIZE (tmpfs mount capacity); a dimensional magnitude modeled as a raw Int while the sibling field digest already grounds on Conten | Import std.measure and type size_bytes as std.measure.ByteSize. |
| `extdeps.ebay.oauth` | EbayOAuthTokenResponse.expires_in / refresh_token_expires_in | OAuth2 expires_in is a token lifetime in SECONDS (RFC 6749 §5.1); modeled as a bare Int (and re-projected as Int in the MintApplicationToken/MintUserT | Import std.measure and type expires_in/refresh_token_expires_in as std.measure.Duration (seconds); apply to the operatio |
| `extdeps.filesystem.filesystem_io` | Filesystem.Write.output.bytes_written | bytes_written: Int is a byte count (a size in bytes — POSIX write() return value), carried dimensionless with no std.measure grounding. | Ground on std.measure.ByteSize (Measure<Memory,One,Nat>); import std.measure and project the transport int through byte_ |
| `extdeps.github.actions` | Job.timeout_minutes / RunStep.timeout_minutes / UsesStep.tim | Job/step timeouts are durations-in-minutes carried as bare Int? (the timeout: Int example); the minute unit lives only in the field name. | Ground on std.measure Duration (with a minutes projection at the YAML emit boundary). |
| `extdeps.github.actions` | ArtifactOp.Upload.retention_days | Artifact retention is a duration-in-days carried as bare Int?; the day unit lives only in the field name. | Ground on std.measure Duration (days projection at emit). |
| `extdeps.github.actions` | max_job_timeout_minutes / default_job_timeout_minutes | The GitHub 360-minute job-timeout ceiling/default are TIME magnitudes carried as bare Int; max_matrix_combinations (256) and MatrixStrategy.max_parall | Ground the two minute constants on std.measure Duration (the 360-minute ceiling is a cited GitHub magnitude); leave the  |
| `extdeps.github.gists` | GistFile.size | `size: Int` is the gist file's size in BYTES (per the GitHub gists API) modeled as a bare dimensionless Int. | Ground on std.measure.ByteSize (type ByteSize = Measure<Memory, One, Nat>, dag/std/measure.dag:178; constructor byte_siz |
| `extdeps.memory.sk_hynix` | hma82gr7afr4n_vk_catalog.data_rate_mts = 2666 (also hma82gr7 | DDR4 transfer rate (2666 MT/s) lands as a bare Int because the DramModuleCatalogRow.data_rate_mts field type is Int (root in extdeps/memory/types.dag) | Reground the field on a std.measure transfer-rate measure so these rows assign e.g. mega_transfers(2666) instead of a ba |
| `extdeps.memory.types` | DramModuleCatalogRow.data_rate_mts: Int | A physical data-transfer rate with the unit (MT/s = mega-transfers per second) baked into the field name and the value left as a bare Int — the interv | Ground on a std.measure transfer-rate/frequency measure, e.g. Measure<Frequency, Mega, Nat> (MT/s as mega events/second) |
| `extdeps.router.verizon_cr1000a` | Cr1000aBroadbandIpv6Snapshot.delegated_prefix_length | A cited IPv6 prefix-delegation length is a bit-count physical magnitude carried as a bare Int, not a dimensionless cardinal (module's own dissolve_on_ | Ground on std.measure BitWidth (import std.measure); parse the prefix length through bit_width at the observation bounda |
| `extdeps.router.verizon_cr1000a` | Cr1000aRouterStatusSnapshot.delegated_prefix_length | Same cited IPv6-PD bit-count magnitude carried as bare Int? — a physical magnitude, not an index/count. | Ground on std.measure BitWidth; single BitWidth authority shared with Cr1000aBroadbandIpv6Snapshot.delegated_prefix_leng |
| `extdeps.sec.types` | XbrlFact.val | A monetary/measured XBRL fact value stored as a bare Float; the upstream companyfacts JSON carries the unit (USD / shares / USD-per-shares) as the enc | Carry the XBRL unit alongside val (a UnitRef / currency tag) or ground val on the monetary type used in extdeps.sec.fact |
| `extdeps.tcgplayer.pricing` | ProductPriceRow.{lowPrice,midPrice,highPrice,marketPrice,dir | Prices are magnitudes with a currency unit (USD) modeled as bare Float; the module itself declares a Scaffold (tcgplayer_pricing_money_measure_groundi | Ground the money fields on a cited Money/Currency measure (std.measure or an extdeps money authority carrying currency + |
| `extdeps.tcgplayer.store` | tcgplayer.Store.UpdateSkuPrice.input.price: Float | SKU price is a currency magnitude modeled as bare Float; the module declares a Scaffold (tcgplayer_store_money_measure_grounding_disposition) on this  | Ground price on the same cited Money/Currency measure as pricing.dag (single money authority), dissolving the scaffold. |
| `extdeps.tcgplayer.tcgplayer` | TcgplayerTokenResponse.expires_in: Int (and tcgplayer.Auth.M | expires_in is the OAuth2 token lifetime in seconds — a Duration — modeled as a bare Int with no scaffold or measure grounding. | Ground expires_in on std.measure.Duration (seconds) so the token-lifetime magnitude carries its unit. |
| `std.cache_interface` | EvictionPolicy.Ttl.days | TTL is expressed as a bare Int count of days (a Duration) while the sibling variant SizeBounded already grounds cap_bytes on std.measure.ByteSize and  | Ground on std.measure Duration, e.g. Ttl { ttl: Duration } (or a cited time magnitude), matching the ByteSize grounding  |
| `std.exec_format` | LoadSegment.file_size / LoadSegment.memory_size / LoadSegmen | Every one of these is a byte-denominated magnitude (segment sizes, byte offsets, virtual addresses, byte alignment) modeled as bare Int; the module do | Import std.measure; ground file_size/memory_size/alignment on ByteSize (Measure<Memory,One,Nat>) and the offset/address  |
| `std.filesystem` | FileEntry.size: Int | size is a file size in bytes modeled as a bare Int; the module does not import std.measure. | Ground size on std.measure.ByteSize (= Measure<Memory, One, Nat>). |

<details><summary>Low-severity numeric-anemia (19)</summary>

| module | location | detail |
| --- | --- | --- |
| `extdeps.bmc.webui.nbd_proxy` | NbdTransmissionRequest.length / NbdTransmissionRequest.offse | Byte-quantity wire fields kept as bare Int even though this same module already grounds export sizes on ByteSize (NbdProxyServeInterface.exp |
| `extdeps.bmc.webui.nbd_proxy_serve` | NbdProxyServeTransportIntent.local_nbd_port / data srv3_nbd_ | NBD local port modeled as bare Int, while the sibling authority extdeps.bmc.virtual_media grounds the identical concept on std.types.Port (N |
| `extdeps.boot.emit` | freestanding_page_align (and byte offsets freestanding_code_ | freestanding_page_align = 4096 is a page/alignment byte AMOUNT (a size), and code_file_offset/phoff are byte offsets, all kept as bare Int w |
| `extdeps.cloud.hetzner` | HetznerCloudServerCatalogRow.hourly / HetznerCloudServerCata | Both fields are money-per-time RATEs (per hour, per month) stored as a plain MoneyAmountMicro amount with the /time dimension dropped, and s |
| `extdeps.container.oci.ctrl_session_witness` | ctrl_session_config_size / ctrl_session_layer0_size | data ctrl_session_config_size: Int = 34025 and ctrl_session_layer0_size: Int = 28122307 are blob byte-sizes carried as bare Int (feeding Oci |
| `extdeps.docker.container_stats` | CpuStats.cpu_percent | cpu_percent: Float? is a percentage/ratio with no Measure carrier; bare Float leaves the unit ambiguous (0-1 normalized vs 0-100 scale). |
| `extdeps.docker.container_stats` | BlkioValue.value | value: Nat carries parent-dependent physical units (bytes under io_service_bytes_recursive, nanoseconds under io_service_time_recursive, pla |
| `extdeps.entropy` | service Urandom.ReadBytes.count (Int) | count is a number of BYTES read from /dev/urandom — a ByteSize magnitude modeled as a bare Int. Self-declared by the module's entropy_count_ |
| `extdeps.formats.elf.primitives` | elf64_ehdr_size / elf64_phdr_size / elf64_shdr_size | Structure-size constants (64/56/64) are byte sizes carried as bare Int and used in bounds arithmetic against element counts; the magic/class |
| `extdeps.formats.elf.relocation` | Elf64Rela.offset | r_offset is a byte offset into the section (ELF Elf64_Off) carried as bare Int; faithfully mirrors the wire struct but the byte dimension is |
| `extdeps.formats.elf.sections` | Elf64Shdr.size / offset / addr / addralign / entsize | Section byte-size (size, entsize), file offset (offset), virtual address (addr) and alignment (addralign) carried as bare Int (Elf64_Xword/O |
| `extdeps.formats.elf.segments` | Elf64Phdr.offset / vaddr / paddr / filesz / memsz / align | Segment file/memory byte sizes (filesz, memsz), file offset (offset), virtual/physical addresses (vaddr/paddr) and alignment (align) carried |
| `extdeps.formats.elf.segments` | PhdrDecodeReason.PhdrTruncated.required_bytes / available_by | Diagnostic byte counts whose field names literally assert a byte dimension, carried as bare Int (the size_bytes: Int pattern). |
| `extdeps.formats.elf.types` | Elf64Ehdr.entry / phoff / shoff / ehsize / phentsize / shent | Entry virtual address (entry), program/section-header file offsets (phoff/shoff, Elf64_Off) and header-entry byte sizes (ehsize/phentsize/sh |
| `extdeps.formats.elf.types` | ElfDecodeReason.Truncated.required_bytes / available_bytes | Diagnostic byte counts whose names assert a byte dimension, carried as bare Int (the size_bytes: Int pattern; identical shape duplicated as  |
| `extdeps.llm.openai` | OpenAiChatCompletion200Body.created (also OpenAiResponses200 | Unix epoch timestamp carried as bare Int; a TIME magnitude. Wire-faithful but ungrounded — the module does not import std.measure. |
| `extdeps.realization.hermetic_fixture` | hermetic_fixture_file_facts.placement.eviction = Ttl { days: | A 30-day freshness/TTL window (a duration) is authored as a bare integer day-count; the std EvictionPolicy.Ttl variant carries days: Int rat |
| `std.approximate_field` | Precision.BinaryPrecision.significand_bits / exponent_bits ( | Bit-widths are an Information magnitude (a count of bits) carried as bare PositiveInt; std.measure already models this exact axis as BitWidt |
| `std.fermi` | FermiTimeout.timeout_ms: Milliseconds | timeout_ms is a time magnitude carried as std.types.Milliseconds, a brand over Int (Int where range(min:0), brand("Milliseconds")), rather t |

</details>

## 5. String anemia — bare String hiding named structure (86)

The `"LGA4926"` → `{package, contact_count}` class: a String whose source encodes decodable
structure. Genuine free-text names/ids are excluded. Highest concentration is extdeps (73).

High/medium severity (21 of 86):

| module | location | the hidden structure | suggested |
| --- | --- | --- | --- |
| `extdeps.apt` | apt_man_url | A https URL carried as a bare NonEmptyStr ("https://manpages.debian.org/testing/apt/apt.8.en.html"), hiding {scheme: Https, locator} that extdeps.uri. | Delete apt_man_url and reference extdeps_external_authority_anchor.uri, or type it Uri via extdeps.uri; do not |
| `extdeps.boards.asrock_rack` | altrad8ud_openbmc_2_07_00_serial_console_probe | A single NonEmptyStr fuses named, decodable facts: 'IPMI SOL Enabled=true, Privilege=ADMINISTRATOR, Payload Port=623 — confirmed live 2026-07-21 via i | Decompose into a modeled SolCapability { enabled: Bool, privilege: BmcPrivilege, payload_port: std.types.Port  |
| `extdeps.boards.asrock_rack` | BmcLiveReadOnlyProbeRow.locator | Locator strings fuse an endpoint path with a probe mode ('/vm/0/0 ws-upgrade', '/nbd/0 non-upgrade GET') into one NonEmptyStr, even though this module | Model as { endpoint: BmcwebNbdProxyEndpointVariant with slot/index bindings, mode: WsUpgrade \| PlainGet }, re |
| `extdeps.boot.linux_x86_boot` | LinuxX86BootImage.cmdline | cmdline is List<String>, hiding the key=value kernel-argument structure that the existing authority extdeps.firmware.kernel_cmdline.KernelCmdlineArg a | Change cmdline to List<KernelCmdlineArg> importing extdeps.firmware.kernel_cmdline, converging on the same aut |
| `extdeps.brew` | brew_install_url / brew_docs | Two URLs stored as bare NonEmptyStr while the grounded Uri{scheme,locator} type is imported and used two lines above for the authority anchor; brew_do | Model both as Uri { scheme: Https, locator: "brew.sh" } / "docs.brew.sh/", and make brew_docs reference the ex |
| `extdeps.browser` | service browser.Page.Goto input wait_until: String = "d | wait_until is Playwright's closed load-state set (domcontentloaded\|load\|networkidle\|commit) hidden inside a bare String. | Model a closed WaitUntilState enum cited to the Playwright page.goto API and project it to the argv string at  |
| `extdeps.container.docker_ce` | DockerCeAptRepo.arch | arch: NonEmptyStr = "arm64" is a bare architecture wire-label; Architecture is a modeled enum (extdeps.toolchain.types, Aarch64/…) and the sibling oci | Type arch as extdeps.toolchain.types.Architecture and derive the debian-arch wire label from a projection fn ( |
| `extdeps.docker.container_stats` | BlkioValue.op | op: String is a closed set of Docker blkio operation names (Read \| Write \| Sync \| Async \| Discard \| Total) modeled as a bare String — no dissolut | Model as a closed enum (BlkioOp = Read \| Write \| Sync \| Async \| Discard \| Total) in extdeps/docker per th |
| `extdeps.git.hooks` | PrePushStdinRow.local_sha / remote_sha (and data zero_s | Commit object IDs (40-hex SHA-1) carried as bare String while extdeps.git already models CommitSha as the single authority; zero_sha = "0000..." is th | Ground local_sha/remote_sha/zero_sha on extdeps.git CommitSha and local_ref/remote_ref on GitRef (import from  |
| `extdeps.gnu_coreutils` | grep_recursive_quiet_flags / grep_recursive_numbered_fl | Opaque argv flag literals ('-rqE','-rnE','-rf') encode named flag sets {recursive, quiet, extended-regexp, line-number, force} as bare strings — the § | Model these as service operations with typed boolean parameters (recursive/quiet/extended_regexp/force) whose  |
| `extdeps.llm.openai` | OpenAiChatCompletion200Message.role (also OpenAiRespons | Response-side role is a bare String while OpenAiChatMessageRole (System\|Developer\|User\|Assistant\|Tool\|Function, defined line 81) is the module's  | Type role as OpenAiChatMessageRole with an *Other{raw: NonEmptyStr} fallback variant per the module's own wire |
| `extdeps.os.ubuntu_seeded_install_media` | UbuntuSeededInstallMediaArtifactRow.grub_kernel_cmdline | A grub kernel cmdline carried as a bare NonEmptyStr, hiding structure that already has a modeled authority: List<KernelCmdlineArg> (extdeps.firmware.k | Type the field List<KernelCmdlineArg> and render via grub_cmdline_arg_render at the emit boundary, so the ';'  |
| `extdeps.router.verizon_cr1000a` | Cr1000aIpv4AddressDistributionRow.expires_in | Cited router-UI lease-expiry text (a TIME magnitude) held as a bare NonEmptyStr? that hides a value+unit; module's dissolve_on_cr1000a_expires_in_ui_t | Parse to std.measure Duration (Second/Minute carrier) at the observation boundary via a typed duration parser  |
| `extdeps.sec.facts` | FiledFactCitation.fiscal_period | fiscal_period: NonEmptyStr holds the SEC XBRL `fp` field, a closed controlled vocabulary ("FY", "Q1".."Q4"), yet is left as a bare string while its si | Model a closed `FiscalPeriod = FullYear \| Q1 \| Q2 \| Q3 \| Q4 \| OtherFiscalPeriod{raw}` in extdeps.sec.type |
| `extdeps.sec.types` | XbrlFact.form | `form: String` holds SEC filing-form codes (10-K/10-Q/8-K) even though this SAME module already defines the closed `FilingForm` enum plus `filing_form | Type the field `form: FilingForm` (project the raw String through `parse_filing_form`); reserve bare String on |
| `extdeps.tcgplayer.pricing` | tcgplayer.Pricing.GetProductPrices.input.product_ids: N | The plural product_ids feeds path /pricing/product/{product_ids}; TCGplayer accepts a comma-joined list of product IDs, so the string hides a List<Int | Model product_ids as List<Int> (or List<ProductId>) and join to the comma-delimited wire form in the transport |
| `std.decl_ref` | DeclarationRef.module_path | module_path: NonEmptyStr is a dotted qualified module path ('std.effects') held as an opaque string, hiding the segment structure QualifiedName alread | Ground module_path on QualifiedName (single authority at src/v2/std/qualified_name.dag); currently blocked onl |
| `std.materialization_ladder` | EvictionClass.SpacePacked.budget (String); also space_p | A persistent-tier space budget is a byte-size magnitude but is carried as a bare String, hiding {value, unit} (a '10G' cache-size cap is really a Byte | Import std.measure and type budget as ByteSize so the size magnitude is decomposed and unit-safe; all other In |
| `std.os.types` | UbuntuDistributionCatalogRow.release_label / kernel_ver | Bare NonEmptyStr hides decodable structure the source encodes: release_label "24.04 LTS (Noble Numbat)" packs {version 24.04, channel LTS, codename No | Ground versions on a structured Version type (major.minor.patch) and split release_label into {version, channe |
| `std.primitives` | PrimitiveContract.work / PrimitiveContract.output_size | Cost-complexity expressions ('n * log(n)', 'end - start', 'min(n, m)', 'max(0, len - n)', 'sum(f_output_sizes)') are encoded as bare Strings, hiding t | Model a structured CostExpr AST (Var \| Add \| Mul \| Min \| Max \| Log over size-variables) reusing std algeb |
| `std.types` | Timestamp (line 148) | `Timestamp = String` represents a temporal instant that encodes decodable structure (ISO 8601 / RFC 3339 date-time) as an opaque String. | Ground on a cited RFC 3339 date-time model or a std.measure Time instant rather than a bare String alias. |

<details><summary>Low-severity string-anemia (65)</summary>

| module | location | detail |
| --- | --- | --- |
| `extdeps.access.aws_iam` | ConditionEntry.operator | AWS IAM condition operators are a closed vocabulary (StringEquals, NumericLessThan, DateGreaterThan, IpAddress, ArnLike, Bool, ... plus ForA |
| `extdeps.apt` | apt_get_binary_path | "/usr/bin/apt-get" is typed NonEmptyStr while the same file types apt_bin_dir/apt_local_bin_dir as FilePath; it is also apt_bin_dir joined w |
| `extdeps.astronomy.stellar_classification` | SpectralClass.main_sequence_share_text | A decodable numeric proportion (e.g. "0.00003%", "76%") is stored as a source-verbatim String; the note self-marks it as a deferred typed ma |
| `extdeps.astronomy.stellar_classification` | SpectralClass.hydrogen_lines | An ordinal strength scale ("Very weak" < "Weak" < "Medium" < "Strong") is kept as a free String rather than a closed ordered enum. |
| `extdeps.astronomy.stellar_classification` | SpectralClass.code | The Harvard class letter is a canonical closed 7-member set (O B A F G K M) modeled as NonEmptyStr, used as the catalog key and matched by s |
| `extdeps.boards.asrock_rack` | BmcLiveReadOnlyProbeRow.probed_at | Calendar date '2026-07-01' carried as a bare NonEmptyStr (decodable Y-M-D structure). |
| `extdeps.browser` | service browser.Page.Goto output final_url / browser.Pa | A URL (scheme/authority/path) is carried as a bare String while a modeled Uri type (extdeps.uri.Uri) already exists. |
| `extdeps.cache.sccache` | SccacheBinaryRelease.version / sccache_toolchain_capabi | The same 0.15.0 release is spelled two ways as bare NonEmptyStr ('v0.15.0' on the release row, '0.15.0' on the ToolchainCapability) — a semv |
| `extdeps.cargo` | CargoProfile (= String) / canonical_profiles | CargoProfile is a bare String alias, and canonical_profiles re-represents the closed built-in profile set (dev/release/test/bench) as a para |
| `extdeps.cargo` | CargoPackage.path / CargoDepSource.LocalPathDep.path | Filesystem paths are carried as bare String even though std FilePath/FilePathParts exists and is already used in this same module (cargo_tar |
| `extdeps.cloud.cloud` | ServiceEndpoint.base_url: String | An endpoint URL carried as a bare String though extdeps.uri.Uri (scheme+locator) is the modeled authority and is already used elsewhere (scc |
| `extdeps.cloud.gcp.gcp` | ServiceAccountEmail = String | A GCP service-account email is a structured id — {account_id}@{project_id}.iam.gserviceaccount.com — decodable into account_id + project_id  |
| `extdeps.cloud.gcp.gcp` | GcpApiEndpoint.base_url: String (and api_endpoints rows | Endpoint URLs carried as bare Strings though extdeps.uri.Uri exists and is used in sccache.dag. |
| `extdeps.cloud.gcp.iam` | GcpGenerateAccessToken200Body.expire_time: String (and  | An RFC3339 expiry timestamp carried as a bare String though std.types.Timestamp exists. |
| `extdeps.cloud.gcp.mock_corpus` | PublishedMockCase.operation_key (all rows, e.g. 'gcloud | operation_key restates service + '.' + operation — both already present as separate fields on the same row — so the composite id is duplicat |
| `extdeps.cloud.gcp.secret_manager` | SmRotationSchedule.next_rotation_time: String? | An RFC3339 timestamp carried as a bare String though std.types.Timestamp exists. |
| `extdeps.cloud.gcp.secret_manager` | GcpSecret.create_time: String | An RFC3339 creation timestamp carried as a bare String. |
| `extdeps.cloud.gcp.secret_manager` | GcpSecretVersion.create_time: String | An RFC3339 creation timestamp carried as a bare String. |
| `extdeps.cloud.gcp.secret_ref` | SecretRef.version | GCP Secret Manager versions are a closed structure — the sentinel 'latest' or a positive integer version number — but are carried as a bare  |
| `extdeps.cloud.gcp.sts` | StsTokenResponse.issued_token_type / StsTokenResponse.t | issued_token_type/token_type are a closed URN vocabulary (urn:ietf:params:oauth:token-type:*) — the module even defines those URN constants  |
| `extdeps.container.docker_ce` | DockerCeAptRepo.keyring_url / DockerCeAptRepo.repo_base | URLs stored as bare NonEmptyStr while the Uri type (extdeps.uri: scheme + locator) is modeled and already imported in this very file for the |
| `extdeps.container.docker_ce` | DockerCeAptRepo.keyring_path | keyring_path: NonEmptyStr = "/etc/apt/keyrings/docker.gpg" is a filesystem path stored as an untyped string while std.types.FilePath is the  |
| `extdeps.dhcp.v4` | MacAddress | MacAddress = NonEmptyStr where brand hides six grounded octets (aa:bb:cc:dd:ee:ff) behind a branded string. |
| `extdeps.dhcp.v4` | Ipv4Address | Ipv4Address = NonEmptyStr where brand hides four grounded octets behind a branded string. |
| `extdeps.docker.container_inspect` | NetworkSettings.gateway / NetworkSettings.ip_address /  | IP and MAC addresses carried as bare String? hide octet structure a typed carrier would capture and validate. |
| `extdeps.docker.container_inspect` | HostConfig.cap_add/cap_drop/exposed_ports/port_bindings | Structured Docker wire shapes (KEY=VALUE env pairs, host:container port specs, source:target:mode volume specs, structured device requests)  |
| `extdeps.docker.endpoint` | docker_default_endpoint (data: String = "unix:///var/ru | Bare String hides decodable structure (scheme=unix + socket path; the union also spans http/https/fd/host:port) while the module already imp |
| `extdeps.ebay.ebay` | production_api_base / sandbox_api_base (data: NonEmptyS | Base API URLs ("https://api.ebay.com", "https://api.sandbox.ebay.com") modeled as bare NonEmptyStr while the module imports Uri; decodable s |
| `extdeps.ebay.inventory` | EbayAmount.value (String) | Monetary amount modeled as a bare String; the paired currency is grounded via std.currency.CurrencyCode but the decimal magnitude itself is  |
| `extdeps.ebay.oauth` | api_scope_root / sell_inventory_scope / user_authorizat | URLs modeled as bare NonEmptyStr while the module imports Uri — same anemic-URL class as ebay.dag/endpoint.dag. (The grant_type constants st |
| `extdeps.filesystem.mock_corpus` | PublishedMockCase.operation_key | operation_key "Filesystem.Write" is a dotted service.operation composite that duplicates the sibling service and operation fields already pr |
| `extdeps.firmware.uefi_shell` | UefiNetworkInterface.ipv4 (and UefiIfconfigAction.Ifcon | IPv4 dotted-quads carried as bare NonEmptyStr; the four-octet structure is decodable and a single authority for the concept already exists i |
| `extdeps.formats.dnsmasq` | DnsmasqDirective.DnsmasqDhcpRangeProxy.network_address | network_address is an IPv4/CIDR network address carried as a bare String; the octet/prefix structure is decodable and an Ipv4Address authori |
| `extdeps.git` | DiffHunk.file_path | A repository path carried as bare String while this module already models FilePath and uses it for the analogous fields (GitDiffStatusEntry. |
| `extdeps.git.versioning` | min_git_version | min_git_version = ">= 2.0.0" restates the version 2.0.0 as a bare constraint string while git_2_0_release: GitReleaseVersion = {2,0,0} is al |
| `extdeps.github.github` | Repository.full_name | `full_name: String` is the `{owner}/{name}` slug — decodable structure already carried by the sibling `owner` and `name` fields, stored as a |
| `extdeps.github.workflow_runs` | WorkflowRun.html_url | html_url is a bare NonEmptyStr URL while the same module already imports and models URLs as Uri{scheme, locator} (used for extdeps_external_ |
| `extdeps.http.client` | http.Client.Get.url (also PostJsonFromFile.url, Downloa | url operation inputs typed NonEmptyStr hide the scheme/authority/path structure that extdeps.uri.Uri (already imported in this file for the  |
| `extdeps.llm.anthropic` | AnthropicModelSpec.model_id | model_id e.g. "claude-opus-4-6-20251101" is a structured id (family/tier/major/minor + a YYYYMMDD snapshot date) carried as a bare String; f |
| `extdeps.os.ubuntu` | noble_numbat_2404_catalog.security_maintenance_until (f | "2029-05-31" is an ISO-8601 calendar date used for EOL/temporal comparison, carried as a bare String; kernel_version "6.8" and libc_version  |
| `extdeps.os.ubuntu_autoinstall` | UbuntuAutoinstallPayload.locale | Bare NonEmptyStr holding a structured POSIX/BCP-47 locale id (e.g. "en_US.UTF-8" = language_territory.codeset); keyboard_layout is the same  |
| `extdeps.os.ubuntu_install_media` | UbuntuInstallMediaArtifactRow.point_release | "24.04.3" is a structured major.minor.patch version carried as bare NonEmptyStr; UbuntuInstallMediaMirrorRow.locator ("releases.ubuntu.com") |
| `extdeps.os.windows` | windows_11_24h2_catalog.end_of_servicing (field Windows | "2026-10-13" is an ISO-8601 calendar date (EOL, used for temporal comparison) carried as a bare String; os_build "26100" and release_label " |
| `extdeps.pricing.hetzner_dedicated` | HetznerDedicatedServerPriceRow.cpu_description | A free-form CPU string ("AMD Ryzen 5 3600 Hexa-Core Matisse (Zen2)") sits beside the structured cpu: CpuFacts field, encoding vendor/model/c |
| `extdeps.router.verizon_cr1000a` | Cr1000aIpv4AddressDistributionRow.host_name_raw | Raw router-UI host-name text held as bare String that the module itself marks for decode via parse_host_name (dissolve_on_cr1000a_host_name_ |
| `extdeps.sec.types` | XbrlFact.fp | `fp: String` encodes the closed fiscal-period set {FY, Q1, Q2, Q3, Q4} as free text, unlike the sibling `FilingForm`/`XbrlTaxonomy` which ar |
| `extdeps.sec.types` | EdgarCalendarDate (type = String) | An ISO-8601 calendar date aliased directly to String; the value decodes to named parts {year, month, day} but the alias hides that structure |
| `extdeps.sec.types` | XbrlFact.accn | `accn: String` is an SEC accession number with a real decodable format (filer-CIK '-' 2-digit-year '-' 6-digit-sequence, e.g. 0000084839-26- |
| `extdeps.tailscale.acl` | TailscaleAclSelector.AclProtoPort.spec (and data dashbo | The proto:port ACL token encodes a protocol and a port number as a bare NonEmptyStr; std.types already exports a Port type (used in serve.da |
| `extdeps.tailscale.acl` | TailscaleAclSelector.AclIpv4.address (and data dashboar | An IPv4 dotted-quad is carried as an opaque NonEmptyStr, hiding its four octets; dashboard_cutover_mac_ip is already scaffold-marked for Sin |
| `extdeps.tcgplayer.tcgplayer` | data production_api_base: NonEmptyStr = "https://api.tc | The API base endpoint is a bare NonEmptyStr that decodes to scheme=Https + host; extdeps.uri.Uri is already imported and used for the author |
| `extdeps.tcgplayer.tcgplayer` | data api_contract_version: NonEmptyStr = "v1.39.0" | The contract version carries decodable semver structure (major.minor.patch) as an opaque string. |
| `extdeps.tools` | InstallSource.SourceGitHubRelease.install_path | Filesystem install path typed NonEmptyStr even though std.types.FilePath is imported and used for ResolvedTool.path in the same module — a p |
| `extdeps.tools.xorriso` | xorriso.Iso operations: iso_path / iso_internal_path /  | Filesystem and in-ISO path inputs typed NonEmptyStr rather than the std.types.FilePath path authority; volume_id is a legitimate opaque ISO9 |
| `extdeps.transports.file` | FileTransportConfig.base_path | Filesystem base path typed as bare String (not even NonEmptyStr) while std.types.FilePath is the modeled path authority. |
| `extdeps.transports.shell` | ShellTransportConfig.working_dir | Working directory (a filesystem path) typed String? rather than the std.types.FilePath path authority. |
| `extdeps.uri` | Uri.locator | locator: NonEmptyStr collapses RFC 3986 authority (host/port/userinfo) + path + query + fragment into one opaque string — the structure RFC  |
| `std.coercion` | CallableRepr.template | Emit code fragments/templates are stored as opaque String (CallableRepr.template, CastSyntax.template, InhabitantDecl.template, TypeCheckpoi |
| `std.hermetic_replay` | PublishedMockCase.operation_key | operation_key: String sits beside `service` and `operation` and encodes their composite — a parallel derived key that hides the {service, op |
| `std.interface_summary` | InterfaceSummary.module_path | module_path: NonEmptyStr is a dotted qualified name stored as a bare string; the dotted-segment structure is decodable and DESIGN §3 explici |
| `std.os.types` | UbuntuDistributionCatalogRow.security_maintenance_until | Support/EOL calendar dates (e.g. "2029-05-31") stored as bare NonEmptyStr hide a structured Date (year-month-day) — a temporal point whose o |
| `std.primitives` | PrimitiveContract.name | The primitive's identity ('char_at', 'hash_combine', 'atom_identity_hash', 'map_keys', 'filesystem_read') is a re-coined String nickname for |
| `std.realization_measurement` | RealizationMeasureEffect.work_shape (ObserveElapsedAtSu | work_shape carries a cost/work-shape descriptor as a bare String, the same cost-algebra lineage as std.primitives.PrimitiveContract.work, hi |
| `std.realize_pack` | RealizeAdvisory.verdict | verdict: String stringifies the already-modeled closed enum RealizeVerdict ('PackedWidth' \| 'MaturationReserve' \| 'BudgetRefused') — a dua |
| `std.types` | Email (line 120), MimeType (line 182) | `Email = String` hides RFC 5322 addr-spec structure (local-part @ domain); `MimeType = String` hides RFC 6838 / IANA media-type structure (t |

</details>

## 6. Other grounding/decomposition notes (17)

| module | location | detail |
| --- | --- | --- |
| `extdeps.github.actions` | upload_artifact_action / download_artifact_action | Each action pin is declared twice with CONFLICTING refs — upload_artifact_action = v2 (line 202) and v4 (line 234); download_artifact_action = v2 (line 206) and v4 (line  |
| `extdeps.git.versioning` | git_release_versioning_spec_url | The spec URL is stored twice: as a structured ExternalAuthority/Uri anchor (extdeps_external_authority_anchor, locator git-scm.com/docs/BreakingChanges.html) and again he |
| `extdeps.gnu_coreutils` | coreutils_docs | Redundant citation representation: `coreutils_docs` is a bare NonEmptyStr copy of the exact URL already carried structurally by the module's ExternalAuthority/Uri anchor  |
| `extdeps.apt` | apt_install_argv vs service apt.PackageManager.Install  | The apt-get install argv ["apt-get","install","--yes",package] is modeled twice: once in the fn apt_install_argv and once in the service Install transport block — two rep |
| `extdeps.diagnostic.mock_corpus` | PublishedMockCase.operation_key | operation_key (e.g. "ssh.Session.Exec") is exactly service + "." + operation, both present as sibling fields in every row — a redundant re-encoding (§2) of already-decomp |
| `extdeps.docker.container_inspect` | ContainerStateDetail | Carries a status: ContainerState coproduct AND parallel running/paused/restarting/dead bools — a state-space conflation permitting illegal states (status=Running, running |
| `extdeps.docker.container_inspect` | HostConfig.memory_swap | ByteSize? (Nat-backed, non-negative) cannot round-trip Docker's -1 'unlimited' sentinel, conflating 'swap disabled' (-1) with 'swap zero' (0) — a fidelity/state-space gap |
| `extdeps.formats.elf.sections` | Elf64Shdr.section_type | section_type is a raw Int even though the sht_* code constants (sht_null/progbits/symtab/strtab) exist and the parallel segments.dag decodes segment_type into the ElfSegm |
| `extdeps.languages.go.primitives` | GoIntegerRangeFact.range_min_inclusive/range_max_inclus | Integer min/max bounds are stored as String literals (e.g. "-128","18446744073709551615") and the same 8 range facts are duplicated across the standalone GoIntegerRangeFa |
| `extdeps.languages.rust.primitives` | RustPrimitive.range_min_inclusive / range_max_inclusive | Each integer type's value bounds are stored as literal String rows ('-128','127',...) even though they are fully derivable from carrier (bit width, already imported via s |
| `extdeps.llm.anthropic_rest` | extdeps_external_authority_anchor (covering operation M | The module's only citation anchors docs.anthropic.com/en/docs/claude-code (the CLI docs, matching operation CliPrompt), but the primary operation Messages models the REST |
| `extdeps.rustup` | rustup_docs | rustup_docs: NonEmptyStr = "https://rust-lang.github.io/rustup/" restates the module's own cited external authority (ExternalAuthority anchor Uri{Https, "rust-lang.github |
| `extdeps.tcgplayer.mock_corpus` | PublishedMockCase.operation_key (all 7 rows, e.g. "tcgp | operation_key is exactly service + "." + operation in every row — a derivable denormalized duplicate of the sibling service/operation fields (§2 redundancy). |
| `std.behavioral` | FailureMode.http_status: Int? | A bare Int for an HTTP status code (an enumerated RFC 9110 code hiding named structure), where std.types already exports a modeled HttpStatus consumed by extdeps/http/ser |
| `std.exec_format` | Relocation.relocation_type: Int | A raw Int relocation-type code that hides the named relocation-type enumeration the ELF/PE specs define (e.g. R_X86_64_*), rather than a modeled cited enum. |
| `std.fermi` | FermiTimeout.label: String | label ("30 seconds", "5 minutes", ...) is a human-readable duplicate representation of timeout_ms — a dual representation of the same magnitude (§2/§5), drifting-prone ra |
| `std.resources` | ResourceHandle.type | Bare String discriminator (alongside bare resource_id/key) names the resource kind as free text rather than referencing a modeled resource, an open nickname surface (§3). |

## Dissolution triggers & how to burn this down

- **§3-forks** dissolve as each duplicate collapses to one authority (delete/redirect one home) —
  the same move the namespace-flip de-fork lane used for Set/Map/Byte/Char. The `std.types` time/size
  alias fork is the highest-leverage single fix (a cluster of numeric-anemia rows dissolve with it).
- **std→extdeps tables** dissolve as each cited table moves to an extdeps row set (framework stays in
  std) — unicode blocks and the RFC-9110 method table are the ready exemplars.
- **citation-gaps** dissolve as each module gains a cited source + pinned version note (the Redfish
  cluster wants one dated DMTF anchor; the 19 uncited modules each want their source named).
- **anemia** dissolves as each field re-grounds on its `std.measure` magnitude or its decomposed type;
  the citation discipline (the surviving gate) reaches these wherever they live — location is cosmetic.

Method note: this is a snapshot classification (grep + read + adversarial verify), not a live lens.
The durable enforcement is the citation + acyclicity discipline; this inventory is the worklist that
discipline has to reach. When a class is burned down its section deletes; when all are, this doc
dissolves into the carriers (§6 — the mark on the carrier is the authority).

---
*Generated from an exhaustive 451-module classification (workflow `extdeps-std-grounding-inventory`,
71 agents, 203 defects, 18/3 CONFIRMED/REFUTED on the verified high-stakes claims). Counts are exact
for this snapshot; severity is the auditor's call.*
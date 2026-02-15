# V2 Worked Examples

**Status**: Draft — January 2026
**Purpose**: Concrete before/after examples showing how existing tool
understandings would be expressed in the V2 type system. Validates the design
in [`v2-contracts-design.md`](./v2-contracts-design.md) against real data.

---

## Example 1: `tool/zstd` — Simple Upsert + Standalone Behavior

### V1 (current)

```rust
static ZSTD_TOOL: Understanding = Understanding {
    id: "tool/zstd",                          // string
    name: "Zstandard Compression Tool",
    kind: SystemKind::Cli,
    behaviors: &[
        Behavior {
            id: "verify",                      // string — ad-hoc name
            upsert_phase: Some(UpsertPhase::Check),  // optional tag
            observed_properties: &[
                ObservedProperty::noted(Property::ReadOnly, "..."),
                ObservedProperty::noted(Property::Deterministic, "..."),
                ObservedProperty::noted(Property::Idempotent, "..."),
            ],
            // ...
        },
        Behavior {
            id: "install_apt",                 // string
            upsert_phase: Some(UpsertPhase::Create),
            observed_properties: &[
                ObservedProperty::noted(Property::WritesWorld, "..."),
                ObservedProperty::noted(Property::Idempotent, "..."),
                ObservedProperty::new(Property::ValidatesWith("verify")),  // string ref
            ],
            // ...
        },
        // install_brew, install_choco — same shape
        Behavior {
            id: "decompress",                  // string
            upsert_phase: None,                // silent non-participation
            observed_properties: &[
                ObservedProperty::noted(Property::WritesWorld, "..."),
                ObservedProperty::noted(Property::Deterministic, "..."),
                ObservedProperty::new(Property::FailsWhen("output exists and force=false")),
            ],
            // ...
        },
    ],
    depends_on: &[],
    // ...
};
```

**Problems visible in this one file:**

1. `id: "verify"` — string, could be `"check"` or `"detect"` with no compiler complaint
2. `upsert_phase: Some(UpsertPhase::Check)` — opt-in tag, no structural link to Create/Resolve
3. `ValidatesWith("verify")` — string reference to another behavior; breaks silently if renamed
4. `FailsWhen("output exists and force=false")` — freeform string; generators can't act on it
5. `upsert_phase: None` on `decompress` — silent non-participation (forgot? or intentional?)
6. No Resolve phase at all — upsert contract is incomplete but compiles fine
7. `ObservedProperty::noted(Property::Idempotent, "...")` — claim with no verification binding

### V2 (proposed)

```rust
use crate::understanding_id;

static ZSTD: ToolUnderstanding = ToolUnderstanding {
    id: understanding_id!("tool/zstd"),       // validated newtype
    name: "Zstandard Compression Tool",
    kind: SystemKind::Cli,

    // Pattern participation is explicit — every pattern addressed
    patterns: Patterns {
        upsert: PatternUse::Applicable(UpsertSpec {
            check: UpsertCheck {
                invocation: Invocation::cli("zstd", &["--version"]),
                exports: ResourceState,       // typed: Exists | Missing | Stale
                properties: &[
                    PropertyClaim {
                        property: Property::ReadOnly,
                        verified_by: Verification::GeneratedTest(readonly_test_spec()),
                    },
                    PropertyClaim {
                        property: Property::Idempotent,
                        verified_by: Verification::GeneratedTest(idempotence_test_spec()),
                    },
                ],
            },
            create: UpsertCreate {
                // Platform-specific strategies — but structurally linked to check/resolve
                strategies: &[
                    InstallStrategy {
                        platform: Platform::Linux,
                        invocation: Invocation::cli("apt-get", &["install", "-y", "zstd"]),
                    },
                    InstallStrategy {
                        platform: Platform::MacOS,
                        invocation: Invocation::cli("brew", &["install", "zstd"]),
                    },
                    InstallStrategy {
                        platform: Platform::Windows,
                        invocation: Invocation::cli("choco", &["install", "-y", "zstandard"]),
                    },
                ],
                imports: ResourceState::Missing,  // typed: only runs when Check exports Missing
                exports: ResourceRef,             // typed: what was installed
                properties: &[
                    PropertyClaim {
                        property: Property::Idempotent,
                        verified_by: Verification::GeneratedTest(idempotence_test_spec()),
                    },
                ],
            },
            resolve: UpsertResolve {
                // Resolve was MISSING in V1 — V2 makes this a compile error
                invocation: Invocation::cli("which", &["zstd"]),
                imports: ResourceStateOrRef,      // typed: works for both paths
                exports: ResolvedHandle,          // typed: path to binary
                properties: &[
                    PropertyClaim {
                        property: Property::ReadOnly,
                        verified_by: Verification::GeneratedTest(readonly_test_spec()),
                    },
                ],
            },
        }),
        lifecycle: PatternUse::NotApplicable,  // explicit: reviewed, doesn't apply
        composition: CompositionSpec::Independent,
    },

    // Standalone behaviors — not part of any pattern
    custom_behaviors: &[
        CustomBehavior {
            role: BehaviorRole::Custom(declare_behavior_id!("tool/zstd/decompress")),
            description: "Decompress a .zst file",
            invocation: Invocation::cli("zstd", &["-d", "<input.zst>"]),
            inputs: &[
                Input::required("input", InputType::Path, "Path to .zst file"),
                Input::optional("output", InputType::Path, "Output path"),
                Input::optional_with_default("force", InputType::Bool, "Overwrite", "false"),
            ],
            outputs: &[
                Output::new("output_path", OutputType::String, "Decompressed file path"),
            ],
            failure_semantics: FailureSemantics::Core(FailureKind::AlreadyExists),
            output_semantics: OutputSemantics::CreatesResource,
            properties: &[
                PropertyClaim {
                    property: Property::Deterministic,
                    verified_by: Verification::GeneratedTest(determinism_test_spec()),
                },
            ],
        },
    ],

    depends_on: &[],
};
```

**What changed:**

| V1 problem | V2 solution |
|---|---|
| `id: "verify"` (string) | No behavior ID in author code — derived from `UpsertPhase::Check` |
| `upsert_phase: Some(...)` (opt-in tag) | `UpsertSpec { check, create, resolve }` — all three required at construction |
| Missing Resolve phase (compiles) | Missing Resolve = won't compile (`UpsertSpec` requires it) |
| `ValidatesWith("verify")` (string ref) | Gone — Check/Create/Resolve are structurally linked by typed imports/exports |
| `FailsWhen("output exists...")` (string) | `FailureKind::AlreadyExists` (Lane A enum) |
| `upsert_phase: None` (silent) | `PatternUse::NotApplicable` (explicit contract assertion) |
| `ObservedProperty::noted(...)` (no verification) | `PropertyClaim { verified_by: ... }` (verification required) |
| 3 separate install behaviors | `strategies: &[InstallStrategy]` — Create is one phase with platform dispatch |

---

## Example 2: `tool/tectonic` — Upsert + Dependencies + Release Spec

### V1 (current, abbreviated)

```rust
static TECTONIC_TOOL: Understanding = Understanding {
    id: "tool/tectonic",
    depends_on: &[
        UnderstandingDependency {
            target: "tool:gh",                    // string, convention-parsed
            reason: "Uses gh CLI for version resolution",
            behaviors: SetSpec::These(&["release_view"]),  // string behavior ref
        },
        UnderstandingDependency {
            target: "github_releases",            // string
            reason: "...",
            behaviors: SetSpec::These(&["download_and_extract_tarball"]),
        },
        // ...
    ],
    behaviors: &[
        // verify (Check), install_script_linux (Create),
        // install_script_macos (Create), install_script_windows (Create),
        // compile (no upsert phase)
    ],
};
```

**Additional V1 problems visible here:**

1. `target: "tool:gh"` — convention-based prefix parsing (`parse_dependency_target`)
2. `behaviors: SetSpec::These(&["release_view"])` — behavior IDs as strings inside dependency scoping
3. No Resolve phase (again)
4. `compile` behavior has `upsert_phase: None` — silent non-participation

### V2 (proposed)

```rust
static TECTONIC: ToolUnderstanding = ToolUnderstanding {
    id: understanding_id!("tool/tectonic"),
    name: "Tectonic LaTeX Engine",
    kind: SystemKind::Cli,

    patterns: Patterns {
        upsert: PatternUse::Applicable(UpsertSpec {
            check: UpsertCheck {
                invocation: Invocation::cli("tectonic", &["--version"]),
                exports: ResourceState,
                properties: &[/* ReadOnly, Deterministic, Idempotent with verification */],
            },
            create: UpsertCreate {
                strategies: &[
                    InstallStrategy {
                        platform: Platform::Linux,
                        invocation: Invocation::generated_from(&RELEASE_SPEC, Platform::Linux),
                    },
                    InstallStrategy {
                        platform: Platform::MacOS,
                        invocation: Invocation::generated_from(&RELEASE_SPEC, Platform::MacOS),
                    },
                    InstallStrategy {
                        platform: Platform::Windows,
                        invocation: Invocation::generated_from(&RELEASE_SPEC, Platform::Windows),
                    },
                ],
                imports: ResourceState::Missing,
                exports: ResourceRef,
                properties: &[/* Idempotent with verification */],
            },
            resolve: UpsertResolve {
                invocation: Invocation::cli("which", &["tectonic"]),
                imports: ResourceStateOrRef,
                exports: ResolvedHandle,
                properties: &[/* ReadOnly with verification */],
            },
        }),
        lifecycle: PatternUse::NotApplicable,
        composition: CompositionSpec::Independent,
    },

    custom_behaviors: &[
        CustomBehavior {
            role: BehaviorRole::Custom(declare_behavior_id!("tool/tectonic/compile")),
            description: "Compile a LaTeX document to PDF",
            invocation: Invocation::cli("tectonic", &["<input.tex>"]),
            inputs: &[
                Input::required("input", InputType::Path, "LaTeX source file"),
                Input::optional_with_default("output_dir", InputType::Path, "Output directory", "."),
                Input::optional_with_default("keep_logs", InputType::Bool, "Keep log files", "false"),
                Input::optional_with_default("synctex", InputType::Bool, "Generate SyncTeX", "false"),
            ],
            outputs: &[
                Output::new("pdf", OutputType::String, "Path to generated PDF"),
            ],
            failure_semantics: FailureSemantics::Core(FailureKind::Custom(
                declare_failure_code!("tool/tectonic/compilation-failed"),
            )),
            output_semantics: OutputSemantics::CreatesResource,
            properties: &[
                PropertyClaim {
                    property: Property::Deterministic,
                    verified_by: Verification::GeneratedTest(determinism_test_spec()),
                },
                PropertyClaim {
                    property: Property::Idempotent,
                    verified_by: Verification::GeneratedTest(idempotence_test_spec()),
                },
            ],
        },
    ],

    depends_on: &[
        Dependency {
            target: DependencyTarget::Tool(understanding_id!("tool/gh")),  // typed, not string
            reason: "Uses gh CLI for version resolution",
            scoped_to: SetSpec::These(&[
                BehaviorRef::from_role(understanding_id!("tool/gh"), BehaviorRole::Custom(
                    declare_behavior_id!("tool/gh/release_view"),
                )),
            ]),
        },
        Dependency {
            target: DependencyTarget::Understanding(understanding_id!("github_releases")),
            reason: "Composable install command generation",
            scoped_to: SetSpec::These(&[
                BehaviorRef::from_role(understanding_id!("github_releases"), BehaviorRole::Custom(
                    declare_behavior_id!("github_releases/download_and_extract_tarball"),
                )),
            ]),
        },
        Dependency {
            target: DependencyTarget::Understanding(understanding_id!("rust_targets")),
            reason: "Rust target triple naming for platform detection",
            scoped_to: SetSpec::Universal,
        },
        Dependency {
            target: DependencyTarget::Understanding(understanding_id!("platform")),
            reason: "Architecture detection",
            scoped_to: SetSpec::Universal,
        },
    ],
};
```

**What changed beyond zstd:**

| V1 problem | V2 solution |
|---|---|
| `target: "tool:gh"` (prefix convention) | `DependencyTarget::Tool(understanding_id!("tool/gh"))` (typed enum) |
| `behaviors: SetSpec::These(&["release_view"])` (string) | `BehaviorRef::from_role(...)` (typed reference) |
| `FailsWhen("LaTeX compilation error")` (string) | `declare_failure_code!("tool/tectonic/compilation-failed")` (Lane B) |

---

## Example 3: `tool/gh` — Mixed Concerns (Upsert + Non-Upsert Behaviors)

This is the most interesting case because `gh` has both upsert behaviors
(install the tool) and non-upsert behaviors (auth, release operations) that
are genuinely different capabilities.

### V1 (current, abbreviated)

```rust
static GH_TOOL: Understanding = Understanding {
    id: "tool/gh",
    behaviors: &[
        Behavior { id: "verify", upsert_phase: Some(UpsertPhase::Check), /* ... */ },
        Behavior { id: "install_apt", upsert_phase: Some(UpsertPhase::Create), /* ... */ },
        Behavior { id: "install_brew", upsert_phase: Some(UpsertPhase::Create), /* ... */ },
        Behavior { id: "install_choco", upsert_phase: Some(UpsertPhase::Create), /* ... */ },
        // These have nothing to do with upsert — but they're in the same flat list
        Behavior { id: "auth_status", upsert_phase: None, /* ... */ },
        Behavior { id: "release_view", upsert_phase: None, /* ... */ },
        Behavior { id: "release_download", upsert_phase: None, /* ... */ },
    ],
    depends_on: &[
        UnderstandingDependency {
            target: "invariant:inv/no-print-secrets",  // string prefix
            reason: "auth_status must not print secrets",
            behaviors: SetSpec::These(&["auth_status"]),
        },
    ],
};
```

**V1 problem specific to this case:** Upsert behaviors and non-upsert
behaviors are in the same flat list with no structural distinction. A reader
must scan `upsert_phase` annotations to understand which behaviors belong
to the install pattern and which are standalone capabilities.

### V2 (proposed)

```rust
static GH: ToolUnderstanding = ToolUnderstanding {
    id: understanding_id!("tool/gh"),
    name: "GitHub CLI",
    kind: SystemKind::Cli,

    // The install-the-tool pattern — structurally separate from capabilities
    patterns: Patterns {
        upsert: PatternUse::Applicable(UpsertSpec {
            check: UpsertCheck {
                invocation: Invocation::cli("gh", &["--version"]),
                exports: ResourceState,
                properties: &[
                    PropertyClaim {
                        property: Property::ReadOnly,
                        verified_by: Verification::GeneratedTest(readonly_test_spec()),
                    },
                    PropertyClaim {
                        property: Property::Idempotent,
                        verified_by: Verification::GeneratedTest(idempotence_test_spec()),
                    },
                ],
            },
            create: UpsertCreate {
                strategies: &[
                    InstallStrategy {
                        platform: Platform::Linux,
                        invocation: Invocation::cli("apt-get", &["install", "-y", "gh"]),
                    },
                    InstallStrategy {
                        platform: Platform::MacOS,
                        invocation: Invocation::cli("brew", &["install", "gh"]),
                    },
                    InstallStrategy {
                        platform: Platform::Windows,
                        invocation: Invocation::cli("choco", &["install", "-y", "gh"]),
                    },
                ],
                imports: ResourceState::Missing,
                exports: ResourceRef,
                properties: &[
                    PropertyClaim {
                        property: Property::Idempotent,
                        verified_by: Verification::GeneratedTest(idempotence_test_spec()),
                    },
                ],
            },
            resolve: UpsertResolve {
                invocation: Invocation::cli("which", &["gh"]),
                imports: ResourceStateOrRef,
                exports: ResolvedHandle,
                properties: &[
                    PropertyClaim {
                        property: Property::ReadOnly,
                        verified_by: Verification::GeneratedTest(readonly_test_spec()),
                    },
                ],
            },
        }),
        lifecycle: PatternUse::NotApplicable,
        composition: CompositionSpec::Independent,
    },

    // Capabilities — not part of any pattern, these are what gh DOES once installed
    custom_behaviors: &[
        CustomBehavior {
            role: BehaviorRole::Custom(declare_behavior_id!("tool/gh/auth_status")),
            description: "Check GitHub authentication status",
            invocation: Invocation::cli("gh", &["auth", "status"]),
            inputs: &[],
            outputs: &[
                Output::new("authenticated", OutputType::Bool, "Whether authenticated"),
                Output::new("user", OutputType::String, "Authenticated username"),
            ],
            failure_semantics: FailureSemantics::Core(FailureKind::PermissionDenied),
            output_semantics: OutputSemantics::PureSignal,
            properties: &[
                PropertyClaim {
                    property: Property::ReadOnly,
                    verified_by: Verification::GeneratedTest(readonly_test_spec()),
                },
            ],
        },
        CustomBehavior {
            role: BehaviorRole::Custom(declare_behavior_id!("tool/gh/release_view")),
            description: "View release information for a repository",
            invocation: Invocation::cli("gh", &["release", "view"]),
            inputs: &[
                Input::optional("tag", InputType::String, "Release tag"),
                Input::required("repo", InputType::String, "Repository (owner/name)"),
                Input::optional("json_fields", InputType::String, "JSON output fields"),
            ],
            outputs: &[
                Output::new("tag_name", OutputType::String, "Release tag"),
                Output::new("assets", OutputType::List, "Release assets"),
            ],
            failure_semantics: FailureSemantics::Core(FailureKind::NotFound),
            output_semantics: OutputSemantics::PureSignal,
            properties: &[
                PropertyClaim {
                    property: Property::ReadOnly,
                    verified_by: Verification::GeneratedTest(readonly_test_spec()),
                },
            ],
        },
        CustomBehavior {
            role: BehaviorRole::Custom(declare_behavior_id!("tool/gh/release_download")),
            description: "Download release assets",
            invocation: Invocation::cli("gh", &["release", "download"]),
            inputs: &[
                Input::required("tag", InputType::String, "Release tag"),
                Input::required("repo", InputType::String, "Repository (owner/name)"),
                Input::optional("pattern", InputType::String, "Asset filename pattern"),
                Input::optional_with_default("dir", InputType::Path, "Download directory", "."),
            ],
            outputs: &[
                Output::new("assets", OutputType::List, "Downloaded asset paths"),
            ],
            failure_semantics: FailureSemantics::Core(FailureKind::NotFound),
            output_semantics: OutputSemantics::CreatesResource,
            properties: &[
                PropertyClaim {
                    property: Property::WritesWorld,
                    verified_by: Verification::Harness(writes_world_harness()),
                },
            ],
        },
    ],

    depends_on: &[
        Dependency {
            target: DependencyTarget::Invariant(invariant_id!("inv/no-print-secrets")),
            reason: "auth_status must not print secrets",
            scoped_to: SetSpec::These(&[
                BehaviorRef::from_role(understanding_id!("tool/gh"), BehaviorRole::Custom(
                    declare_behavior_id!("tool/gh/auth_status"),
                )),
            ]),
        },
    ],
};
```

**What this example demonstrates:**

1. **Structural separation of concerns**: The upsert pattern (install the
   tool) is visually and structurally separate from the tool's capabilities
   (`auth_status`, `release_view`, `release_download`). In V1 these are all
   in the same flat `&[Behavior]` list.

2. **The gh tool is really two things**: an installable resource (upsert)
   and a set of capabilities (custom behaviors). V2 makes this distinction
   explicit in the type system.

3. **Dependency target typing**: `"invariant:inv/no-print-secrets"` (V1
   string with prefix convention) → `DependencyTarget::Invariant(invariant_id!("inv/no-print-secrets"))` (V2 typed enum variant with validated ID).

4. **FailureKind replaces FailsWhen**: `FailsWhen("tag not found")` →
   `FailureKind::NotFound`. Generators can produce meaningful error handling
   without string matching.

---

## Observations Across All Three Examples

### What disappears entirely

- `upsert_phase: Option<UpsertPhase>` — replaced by structural `UpsertSpec`
- `ValidatesWith("verify")` — replaced by structural check/create/resolve linkage
- `FailsWhen(&str)` — replaced by `FailureKind` enum (Lane A) or `declare_failure_code!()` (Lane B)
- `ObservedProperty::noted(Property::X, "reason")` — replaced by `PropertyClaim { verified_by }`
- Convention-parsed dependency targets — replaced by `DependencyTarget` enum
- Behavior `id: &str` for patterned behaviors — derived from role

### What gets simpler

- Platform-specific install behaviors collapse from N separate behaviors
  (each with redundant properties) into one `UpsertCreate` with a
  `strategies` array
- The reader can immediately see the structural shape: upsert pattern here,
  custom capabilities there, dependencies with typed targets

### What gets more verbose

- `PropertyClaim { property, verified_by }` is more verbose than
  `ObservedProperty::noted(Property::X, "reason")`
- `DependencyTarget::Tool(understanding_id!("tool/gh"))` is more verbose than
  `"tool:gh"`
- Every `PatternUse` field must be addressed even for `NotApplicable`

This verbosity is intentional — it's the cost of structural enforcement.
The V1 brevity came from strings and optionality, which is exactly what
allowed silent bugs.

### The missing Resolve phase

All three V1 examples are **missing a Resolve phase**. In V1 this compiles
fine. In V2 it's a type error — `UpsertSpec` requires all three phases. This
single change would have prevented the issue documented in the retrospective
§2 ("Missing Resolve Phase").

---

## What the sub-DAG looks like after lowering

For completeness, here's what `tool/zstd`'s upsert pattern produces when
lowered to the execution layer (Level 3):

```
┌─────────────────────┐
│ tool/zstd/check     │
│ Exports: ResourceState │
└────────┬────────────┘
         │ ResourceState
         ▼
┌─────────────────────┐     ┌─────────────────────┐
│ (conditional gate)  │────▶│ tool/zstd/create     │
│ if Missing          │     │ Imports: Missing      │
└─────────────────────┘     │ Exports: ResourceRef  │
                            └────────┬────────────┘
                                     │ ResourceRef
         ┌───────────────────────────┘
         ▼
┌─────────────────────┐
│ tool/zstd/resolve   │
│ Imports: State|Ref   │
│ Exports: ResolvedHandle │
└─────────────────────┘
```

Plus the standalone `tool/zstd/decompress` node — no edges to the upsert
sub-DAG (it's a custom behavior, not part of any pattern).

This is what `to_blocks()` should produce in V2: not 5 isolated blocks,
but a 3-node sub-DAG with typed edges plus 1 independent block.

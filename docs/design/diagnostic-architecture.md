# Diagnostic Architecture Design

Status: **Proposed** | Owner: CP-48/CP-63 | Depends on: CP-8 (done)

## Problem

The current `Diagnostic` type in `daglang-contract` has `span: Option<Span>` and `file: Option<PathBuf>`. The design promise is that user-facing diagnostics always carry source location, but the optional fields make this unenforceable. Stages can (and do) emit diagnostics without location, which produces unhelpful error output like `[LOW026]: unresolved service call argument` with no file or line.

Additionally, the `LowerError` enum (25+ variants) and `TypeError` enum carry error information outside the `Diagnostic` system entirely — they use `Display` formatting rather than structured diagnostics.

## Goals

1. **Mandatory source location** on user-facing diagnostics
2. **Multi-span support** for errors that reference multiple locations (e.g., "expected X defined here, got Y used here")
3. **Bounded rendering** — predictable error output size
4. **Single diagnostic path** — all stage errors flow through `Diagnostic`, not ad-hoc `Display` impls

## Design

### Phase 1: Split Diagnostic into Located + Internal

```rust
/// User-facing diagnostic — source location is mandatory.
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub primary: LocatedSpan,         // always present
    pub related: SmallVec<[LabeledSpan; 2]>,
    pub context: DiagnosticContext,
    pub help: Option<String>,
}

/// A span with mandatory file and label.
pub struct LocatedSpan {
    pub file: FileId,
    pub span: Span,
    pub label: String,
}

/// A secondary span with role annotation.
pub struct LabeledSpan {
    pub file: FileId,
    pub span: Span,
    pub label: String,
    pub role: SpanRole,
}

pub enum SpanRole {
    /// "first defined here"
    Definition,
    /// "conflicts with this"
    Conflict,
    /// "referenced here"
    Related,
}
```

Key changes from current:
- `span` and `file` are non-optional (moved into `LocatedSpan`)
- `file` uses `FileId` (interned, already defined in contract) instead of `PathBuf`
- `related` uses `SmallVec<[LabeledSpan; 2]>` (bounded, inline for common case)
- `RelatedSpan` renamed to `LabeledSpan` with a `role` discriminant

### Phase 2: FileId resolution table

The `Diagnostics` collection carries a `FileTable` that maps `FileId -> (PathBuf, source_text)`. This avoids cloning `PathBuf` per diagnostic and enables the renderer to show source snippets.

```rust
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
    pub files: FileTable,
}

pub struct FileTable {
    entries: Vec<FileEntry>,
}

pub struct FileEntry {
    pub path: PathBuf,
    // Source text is optional — only loaded when rendering snippets.
    // Not carrying source text through the pipeline.
}

impl FileTable {
    pub fn intern(&mut self, path: PathBuf) -> FileId;
    pub fn resolve(&self, id: FileId) -> &Path;
}
```

### Phase 3: Stage error → Diagnostic conversion

Each stage error type (`LowerError`, `TypeError`, `VerifyError`) gets a `fn to_diagnostic(&self, files: &mut FileTable) -> Diagnostic` method. The stage runner calls this when converting stage-specific errors to the unified `Diagnostics` collection.

```rust
// In daglang-driver, at each stage boundary:
let typed = typecheck(&module_graph).map_err(|errors| {
    let mut diags = Diagnostics::new();
    for err in errors {
        diags.push(err.to_diagnostic(&mut diags.files));
    }
    CompileError::Diagnostics(diags)
})?;
```

This is incremental — each error variant gets a `to_diagnostic` impl one at a time. The `NodeOrigin` on IR nodes provides the source location for lowerer errors.

### Phase 4: Bounded renderer

```rust
pub struct RenderConfig {
    /// Max related spans to show (default: 2)
    pub max_related: usize,
    /// Show source snippets (default: true if source available)
    pub show_snippets: bool,
}
```

Output format:
```
error[LOW026] dsl/tools/makegen.dag:42:5
  |
42|   result = some_service.call(arg: unknown_ref)
  |                               ^^^^^^^^^^^ unresolved argument
  |
  help: check that arg #1 of `some_service.call` matches a declared operation input
```

Bounded: 1 primary span, up to 2 related, 1 help line, up to 2 hint lines. Renderer truncates beyond bounds with `... and N more`.

## Migration Path

1. **Phase 1** (S): Add `LocatedSpan`, `LabeledSpan`, `SpanRole`. Keep old `Diagnostic` fields as deprecated aliases. Add `Diagnostic::located()` constructor that requires `LocatedSpan`.
2. **Phase 2** (S): Add `FileTable` to `Diagnostics`. Update `Diagnostic::new()` to take `FileId` instead of being location-free.
3. **Phase 3** (M): Add `to_diagnostic()` to `LowerError` (29 variants), `TypeError` (35+ variants), `VerifyError` (4 variants). Requires `NodeOrigin` stamping (CP-63).
4. **Phase 4** (S): Bounded renderer. Replace all `eprintln!` in stages with renderer calls.

## What this does NOT cover

- **Rename "Resolve" to "Bind"** — naming change, orthogonal to diagnostic plumbing
- **Warning-level diagnostics** — `Diagnostics` currently only has `errors`. Adding `warnings: Vec<Diagnostic>` is straightforward but deferred until there are actual warnings to emit.
- **IDE protocol (LSP)** — `Diagnostic` maps cleanly to LSP `Diagnostic` but the wire format is deferred.

## Verification

After Phase 1+2:
- `Diagnostic::new()` requires `LocatedSpan` (compile error if omitted)
- `grep -r "span: None" core/daglang/` → 0 hits in non-test code

After Phase 3:
- `grep -r "eprintln!" core/daglang/daglang-lower/src/` → 0 hits
- All `LowerError` variants produce `Diagnostic` with source location

After Phase 4:
- Error output format matches spec above
- No error exceeds 10 lines of output

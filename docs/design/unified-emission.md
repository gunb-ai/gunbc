# Unified Emission Model

> **Goal**: All rendering follows the same IR → Renderer pattern. Rendering
> pipelines are DAGs. Backends are swappable without duplicating "what to
> render" logic.

> **Status update (2026-03-06)**: The Rust-side CI YAML renderer family
> discussed in this audit (`CiRenderer`, `SharedStep`, `WorkflowConfig`) has
> been deleted. The live CI artifact path is now `dsl/config/ci.dag` +
> `dsl/tools/cigen.dag`. References to those Rust CI renderers below are
> historical audit context unless explicitly called out as already resolved.
> For CI artifact generation, the current/future direction is documented in
> `docs/design/ci-rendering-dsl-consolidation.md`; any CI-specific Rust
> renderer target in this document is superseded by that design.

---

## Current State: Thirteen Rendering Islands

### 1. Testgen (the gold standard)
**Location**: `core/codegen/src/testgen/`
**Pattern**: Structured IR → trait-based renderer

```
analyze_dag() → collect_obligations() → build TestFile IR → TestRenderer::render_file() → String
```

- `TestFile` / `TestFn` / `Stmt` / `Expr` / `Assert` — language-neutral IR (~295 lines)
- `TestRenderer` trait — 6 methods (render_file, render_expr, render_stmt, render_assert, render_value, render_import)
- `RustRenderer` — full implementation (630 lines)
- `PythonRenderer` / `TypeScriptRenderer` — stubs (41 lines each)
- `codegen.rs` (4,212 lines) builds IR only; never constructs strings

**What's good**: Complete separation of "what to test" from "how to render."
Adding a new language backend requires zero changes to codegen.rs.

**What's wrong**: The file is named `codegen.rs` inside the `codegen` crate.
The `TestRenderer` trait is private to testgen — not shared with any other
system. Does not use `Renderable`.

### 2. Makegen / Justgen
**Location**: `dsl/tools/{makegen,justgen}.dag`, `dsl/extdeps/{make_render,justfile_render}.dag`, `gunbc-app/src/makegen/`
**Pattern**: shared build-target data → DSL leaf serializer → string artifact

```
config.build_targets + discover_tools() + extdeps.build_targets
  → tools.makegen / tools.justgen
  → extdeps.make_render / extdeps.justfile_render
  → String
```

- `dsl/extdeps/build_targets.dag` — shared repo build/workflow target schema
- `dsl/extdeps/make_render.dag` — final Makefile syntax leaf serializer
- `dsl/extdeps/justfile_render.dag` — final Justfile syntax leaf serializer
- `gunbc-app/src/makegen/shared.rs` — evaluates DSL render fns for Rust-side ratchets/tests

**What's good**: The live repo render path is now DSL-owned, shared-schema-first,
and compositional. Makegen and Justgen no longer duplicate the repo build-target
model or inline their syntax helpers in the tool wrapper.

**What's wrong**: There is still no richer typed Makefile/Justfile document IR
consumed end-to-end. The unused `extdeps.make` / `extdeps.justfile` experiment
was intentionally deleted rather than kept as an anemic stub, so the current
state is "shared build-target schema + leaf serializers", not full document IR.

### 3. Terminal Progress Display
**Location**: `core/exec/src/`
**Pattern**: State machine → ANSI string construction (no IR, no trait)

```
DAG execution → DagProgress state machine → TerminalRenderer writes ANSI directly → TTY
```

- `DagProgress` — state machine tracking node/edge states (Pending→Running→Completed/Failed/Skipped)
- `TerminalRenderer` — consumes `DagLayout` + `DagProgress` + `SymbolSet`, writes ANSI escape codes
- `TerminalProfile` — detects TTY, CI, Unicode support, viewport size
- Animation is a post-execution replay, not live rendering
- Three modes: Standard, Dynamic, Compact — selected by profile, hardcoded in renderer

**What's good**: Sophisticated capability detection. Power-flow edge animation.
Graceful degradation (Emoji → Unicode → ASCII).

**What's wrong**: Entirely hardcoded to ANSI terminal output. No IR between
"what to display" and "how to display it." Cannot produce: HTML visualization,
plain-text CI summary, SVG export, web dashboard, or test snapshot without
forking the entire renderer. The `FrameLoop` trait decouples timing from
rendering but doesn't decouple content from format.

### 4. CI Workflow Rendering (historical Rust path; now deleted)
**Historical location**: `core/ir/src/transport/ci/`
**Historical pattern**: SharedStep IR → provider trait → YAML string

```
DAG → dag_to_shared_steps() → SharedStep[] → CiRenderer::render() → YAML string
```

- `SharedStep` enum — language-neutral (Checkout, Run, DagStep, DagRun)
- `CiRenderer` trait — provider_id(), render(), output_path()
- `GitHubActionsProvider` / `GitLabCiProvider` — each maps SharedSteps to provider YAML
- `dag_to_shared_steps()` is shared logic

**What was good**: The deleted Rust path had a clean two-layer split and made
provider-specific YAML differences explicit.

**What's wrong / what remains**: The repo no longer uses this path. The live
problem moved left into `dsl/tools/cigen.dag`, which still serializes provider
YAML with string-heavy helpers instead of building typed workflow/job/step
values first.

### 5. CLI Generation
**Location**: `core/codegen/src/cli_gen.rs`
**Pattern**: Template functions → string concatenation (no IR, no trait)

```
ToolMeta + CliEntrypoint[] → generate_cli() → format!() with embedded Rust templates → String
```

- `ToolMeta` / `CliEntrypoint` / `CliBoundary` — configuration structs (not IR)
- 1,001 lines of template functions building Rust source via `format!()`
- Two modes: normal (single execution) and step (per-node CI steps)

**What's good**: Simple, works, rarely changes.

**What's wrong**: Could use the test IR (Expr/Stmt) and RustRenderer — it's
generating Rust code the same way testgen does, but via string concatenation
instead of structured IR.

### 6. Clippy Config Generation
**Location**: `lib/tools/clippy/src/config.rs`
**Pattern**: Renderable trait + string concatenation

- `ClippyConfigRenderer` implements `Renderable` (line 368)
- `generate_clippy_toml()` builds TOML via `push_str()` + `format!()`
- Grouped disallowed methods, category comments, crate allowance docs

**What's wrong**: TOML content is string-built, not structured. Same class of
problem as makegen.

### 7. Pragma Artifact Generation
**Location**: `gunbc-app/src/policy/pragma.rs`
**Pattern**: Manual header + string concatenation (no trait)

- `render_disallowed_methods_allowlist()` → `tools/disallowed-methods-allowlist.txt`
- `render_pragma_lint_policy()` → `tools/pragma-lint-policy.txt`
- Both produce "Generated by / DO NOT EDIT" headers manually — does NOT use `Renderable`

**What's wrong**: Bypasses `Renderable` entirely. Hand-rolls the header that
`Renderable` standardizes. The pragma binary mixes `Renderable` (for
clippy.toml) with non-`Renderable` (for the two text files) in the same codepath.

### 8. DAG Code Generation
**Location**: `core/codegen/src/dag_gen.rs`
**Pattern**: format!() templates (no IR, no trait)

- `generate_graph_rs()` produces complete Rust source files
- Generates an enum, an `Executable` impl, and a graph builder function
- Embeds "DO NOT EDIT" header inline via `format!()` — does NOT use `Renderable`

**What's wrong**: Generates Rust source code the same way testgen does (via
string concatenation) and CLI gen does (via format!() templates), but is a
third independent system. Could use the Code IR + `RustCodeRenderer<M>`.

### 9. CI Report Rendering
**Location**: `gunbc-app/src/ci/ops.rs`
**Pattern**: format!() inside a DAG operation (no IR, no trait)

- `CIOp::Report` builds a structured plain-text report (`CI Report\n=========\n...`)
- Conditionally appends failure detail sections
- Classified as a "pure operation" rather than a rendering concern

**What's wrong**: This IS a rendering concern — it produces structured output
with sections, headers, and conditional content. Should be a `Document` with
`StructuredBlock` sections rendered via `PlainText` medium.

### 10. Markdown Generation
**Location**: `lib/markdown/src/lib.rs`
**Pattern**: push_str() + format!() (no IR, no trait)

- `render_code_snapshot()` → markdown with fenced code blocks per file
- `render_diff_snapshot()` → markdown with fenced diff blocks per file
- Wrapped as `MarkdownOp` DAG operations

**What's wrong**: Generates markdown with no connection to any rendering
abstraction. Should use `MarkupNode` IR + `MarkupRenderer<M>`.

### 11. LLM Prompt Construction
**Location**: `lib/review/src/lib.rs`
**Pattern**: Vec<String>.join() (no IR, no trait)

- `execute_prepare_review_prompt()` builds a structured prompt for LLM review
- Includes review criteria, JSON output format, per-check questions with examples
- Prompts are output artifacts consumed by an external system

**What's wrong**: Prompt format is significant to system behavior but has no
rendering abstraction. A structured prompt IR would make format changes safer
and enable alternative prompt formats (e.g., XML-tagged for Claude).

### 12. CI Workflow Commands
**Location**: `core/ir/src/transport/ci/providers/{github,gitlab,plain}.rs`
**Pattern**: CiProvider::format() trait method (separate from CiRenderer)

- `CiProvider::format()` formats `WorkflowCommand` enums into provider-specific strings
- GitHub: `::group::`, `::error file=X,line=Y::`, `::add-mask::`
- GitLab: ANSI escape sequences for collapsible sections
- Plain: `=== group ===`, `[ERROR]`

**What's wrong**: This is a rendering concern distinct from `CiRenderer` (which
generates YAML). Runtime command formatting should flow through the same
medium model — `CiProvider::format()` is really a `StructuredRenderer`
specialization.

### 13. WorkflowConfig YAML (legacy, deleted)
**Historical location**: `core/ir/src/transport/github_actions.rs`
**Pattern**: Renderable trait + string building

- `WorkflowConfig` implemented `Renderable`
- Generates GitHub Actions YAML via string building
- Existed alongside the newer `CiRenderer` trait system
- Was redundant with `GitHubActionsProvider`

**What's wrong**: Same output type (GitHub Actions YAML) produced by two
independent systems (`WorkflowConfig::render()` and `GitHubActionsProvider::render()`).
That duplication has now been removed, but it remains a useful example of a
renderer bridge surviving past its migration window.

---

## The Problem

### Fragmentation is worse than it looks

The initial inventory identified 5 rendering systems with 4 different traits.
The full audit reveals **13 rendering systems** — and 8 of them have no trait
at all. Only 4 go through `Renderable`, and of those, 2 others produce the
same "Generated by / DO NOT EDIT" header manually without using it.

| Category | Count | Systems |
|----------|-------|---------|
| Has a trait | 4 | Testgen (TestRenderer), Makegen/gitignore/clippy (Renderable), Terminal (FrameLoop), CI commands (CiProvider) |
| No trait, has IR | 0 | — |
| No trait, no IR | 8 | CLI gen, pragma text files, DAG code gen, CI report, markdown, LLM prompts, dag_gen, CI YAML generation (`tools/cigen.dag`) |

### Policy lives inside renderers

Makegen is the clearest example, but it's not unique. The problem isn't just
"string concat instead of IR" — it's **model/policy/presentation mixed in the
render step**:

- **PrepLevel → deps mapping** is hardcoded inside `render_meta_target()`.
  That's policy living in codegen instead of being derived from the model.
- **Tool targets blanket-depend on `ensure-codegen`**, even for tools that
  don't need generated code. That's policy by blanket rule, not per-tool
  declaration.
- **Meta-target deps are unverified strings** (e.g., `"testgen-check"`,
  `"fmt-fix"`) — renames break at runtime, not compile-time.

The fix isn't just "use an IR instead of strings." It's: **structure should
end at the last possible moment.** Model what depends on what. Derive policy
from the model. Then render to Makefile/Justfile/Taskfile syntax at the very
end. The renderer should receive a fully-resolved dependency graph and only
decide how to *express* it — never what it *means*.

### Terminal is outside the DAG

The most user-visible output — the animated DAG visualization — is not modeled
as a DAG. It can't be tested with the same dry-run infrastructure. It can't
produce alternative formats. It can't be composed with other rendering.

### Render ≈ Upsert, but we don't see it

Consider what happens when we generate a test file:

```
1. Check if file exists and hash matches (read-only)
2. Build the content (pure transformation)
3. Write the file (boundary I/O)
```

This is **Upsert**: Check → Create → Resolve. But testgen doesn't use
UpsertBuilder. It runs outside the DAG entirely, in imperative Rust code. Same
for makegen, CI gen, CLI gen.

### Generated artifacts aren't verified

CI doesn't yet verify that generated files match their generator output. The
first hand-edit silently reintroduces drift. The Emit pattern should close this
loop: every emitted artifact gets a content hash recorded in a manifest, and CI
verifies the manifest matches the regenerated output. This is the behavioral
complement to the rendering unification — not just "all rendering flows through
the same model" but "all generated artifacts are provably fresh."

### Missing pattern: Emit

The DAG system has patterns for control flow (Branch, Loop, Retry, While, Poll),
idempotent creation (Upsert), and transactional operations (Transaction,
Atomic). It's missing an **Emit** pattern for "transform data into a
target-specific format and write it somewhere."

Every rendering system follows this shape:

```
┌────────────┐     ┌──────────┐     ┌────────────┐     ┌────────────┐
│  Prepare   │ ──▶ │  Format  │ ──▶ │   Write    │ ──▶ │  Verify    │
│  (pure)    │     │  (pure)  │     │ (boundary) │     │ (manifest) │
└────────────┘     └──────────┘     └────────────┘     └────────────┘
   Build IR      Apply renderer    TransportOps::Execute  Record hash
```

But this isn't formalized. Each system builds this pipeline ad-hoc in
imperative code, and none of them record a verification hash.

---

## Proposed Model

### Core Insight 1: Rendering is a DAG

Every emission pipeline is a DAG of pure transformations ending at a transport
boundary. We should model it that way.

### Core Insight 2: Renderers form a dependency tree over output media

Rendering isn't a flat set of unrelated format backends. It's a two-axis tree:

1. **Output medium** — *how* content reaches the user (text strings, graphics
   primitives, audio, etc.)
2. **Domain layer** — *what kind* of content (code, markup, structured data,
   terminal frames)

Domain layers are generic over the output medium. The same `CodeRenderer` that
produces Rust source as a string can also *draw* a Rust AST as a visual
diagram — same structural logic, different leaf rendering.

```
                        OutputMedium
                       /            \
              TextMedium            GraphicsMedium
             /    |    \             /      |
          Ansi  Plain  Html      Svg    Canvas  ...
            \    |    /             \      |
             ────┬────               ──┬───
                 │                     │
         ┌──────┴──────────────────────┴──────┐
         │         Domain Layers              │
         │  (generic over OutputMedium)       │
         │                                    │
         │   CodeRenderer<M: OutputMedium>    │
         │   MarkupRenderer<M: OutputMedium>  │
         │   StructuredRenderer<M: OutputMedium>│
         │   FrameRenderer<M: OutputMedium>   │
         └────────────────────────────────────┘
                 │
         ┌──────┴──────┐
         │  Concrete   │
         │  Impls      │
         │             │
         │  RustCodeRenderer<Ansi>     → highlighted terminal output  │
         │  RustCodeRenderer<Html>     → highlighted HTML             │
         │  RustCodeRenderer<Svg>      → visual AST diagram (future) │
         │  GfmMarkupRenderer<Plain>   → plain markdown text         │
         │  MakefileRenderer<Ansi>     → colored Makefile preview    │
         └─────────────────────────────────────────────────────────┘
```

Every renderer eventually bottoms out at an **OutputMedium** — the leaf that
knows how to express a styled span in its target format. A `RustCodeRenderer`
doesn't know whether it's producing ANSI strings or SVG glyphs; it delegates
to `CodeRenderer` which delegates to the medium. A `GfmMarkupRenderer`
doesn't know whether headings become `#` characters or drawn rectangles; the
medium decides.

This means:
- **All text rendering is uniform.** Indentation, line wrapping, span
  concatenation — one `TextMedium` implementation, used everywhere that
  produces strings.
- **All graphics rendering is uniform.** Glyph placement, bounding boxes,
  color mapping — one `GraphicsMedium` implementation (future), used everywhere
  that produces visual output.
- **All code languages share structure.** Imports, blocks, expressions,
  statements — `CodeRenderer` handles the skeleton; language-specific renderers
  fill in syntax; the medium handles output format.
- **All markup languages share structure.** Headings, lists, emphasis, links —
  `MarkupRenderer` handles the document model; flavor-specific renderers handle
  syntax differences; the medium handles output format.
- **All structured documents share structure.** Sections, key-value pairs,
  targets with dependencies — `StructuredRenderer` handles the shape; the
  medium handles whether that's tab-indented Makefile text or a visual
  dependency graph.
- **Adding a new medium unlocks all domains.** Implement `GraphicsMedium` once
  and every domain renderer — Code, Markup, Structured, Frame — can target it
  without modification.

### Render IR: medium-agnostic content, medium-specific output

The IR captures **what** to render — content and structure — without assuming
**how** it will be expressed. The same IR flows through a text medium (producing
strings) or a graphics medium (producing visual primitives). The IR is layered:
each domain layer introduces its own node types that compose from a shared
content layer.

```rust
// ── Content layer (medium-agnostic) ────────────────────────────────
// The "what." Describes styled content without assuming output format.
// These types are consumed by every medium — text, graphics, future.

/// A span of content with semantic styling.
/// A TextMedium renders this as ANSI/HTML/plain characters.
/// A GraphicsMedium renders this as positioned glyphs with color fills.
pub struct Span {
    pub text: String,
    pub style: SpanStyle,
}

pub struct SpanStyle {
    pub color: Option<SemanticColor>,  // From symbols.rs — already exists
    pub bold: bool,
    pub italic: bool,
    pub symbol: Option<SymbolId>,      // From symbols.rs — medium resolves via SymbolSet + Tier
}

/// A logical line of content.
pub struct Line {
    pub spans: Vec<Span>,
    pub indent: usize,                 // Logical indent level, not chars/pixels
}

/// A block of lines.
pub struct Block {
    pub lines: Vec<Line>,
}

// ── Code layer (composes Content) ──────────────────────────────────
// Language-neutral code structure. TestFile/TestFn/Stmt/Expr live here.
// The existing test_ir types are already this layer — they just need
// to compose Span for their leaf values instead of raw strings.

// (TestFile, TestFn, Stmt, Expr, Assert — already exist, become the
// canonical Code IR. No new types needed; they gain Span leaves.)

// ── Markup layer (composes Content) ────────────────────────────────
// Document structure for markdown-family languages.

pub enum MarkupNode {
    Heading { level: u8, content: Vec<Span> },
    Paragraph(Vec<Span>),
    List { ordered: bool, items: Vec<Vec<MarkupNode>> },
    CodeBlock { language: Option<String>, body: Block },
    Table { headers: Vec<Vec<Span>>, rows: Vec<Vec<Vec<Span>>> },
    ThematicBreak,
    BlockQuote(Vec<MarkupNode>),
}

// ── Structured layer (composes Content) ────────────────────────────
// Key-value targets, sections, categorized lists.

pub struct Target {
    pub name: String,
    pub deps: Vec<String>,
    pub body: Block,
}

pub struct Category {
    pub name: String,
    pub source: String,
    pub items: Vec<String>,
    pub rationale: String,
}

pub enum StructuredBlock {
    Target(Target),
    Category(Category),
    Section { heading: Option<String>, blocks: Vec<StructuredBlock> },
    Content(Block),
    Blank,
}

// ── Frame layer (composes Content) ─────────────────────────────────
// Frame-based output with cursor control.

pub struct Frame {
    pub lines: Vec<Line>,
    pub cursor_action: CursorAction,  // Overwrite, Append, Clear
}

// ── Document layer (top-level, composes all above) ─────────────────
// A complete renderable artifact.

pub struct Document {
    pub path: Option<String>,
    pub header: Option<FileHeader>,
    pub body: DocumentBody,
}

pub enum DocumentBody {
    Code(TestFile),
    Markup(Vec<MarkupNode>),
    Structured(Vec<StructuredBlock>),
    Frames(Vec<Frame>),
    Raw(String),
}
```

The key design choice: `Span`, `Line`, `Block` are **medium-agnostic content
primitives**. They use semantic styling (`SemanticColor`, not RGB;
`SymbolId`, not a char literal; `indent` as a level, not a character count).
Each medium interprets these semantics in its own way:

| Content primitive | TextMedium | GraphicsMedium |
|-------------------|-----------|----------------|
| `Span { bold: true }` | `\x1b[1m` or `<b>` | Heavier font weight |
| `SemanticColor::Success` | Green ANSI code | Green fill |
| `SymbolId::Checkmark` | `✓` or `[x]` | Checkmark glyph/icon |
| `Line { indent: 2 }` | 8 spaces or 2 tabs | 2 × indent_px offset |
| `Block` | Newline-joined lines | Vertically stacked line boxes |

This means the IR is write-once. A codegen pass builds a `Document` containing
`Span`/`Line`/`Block` nodes. That same document flows unchanged to any medium.

### Renderer traits: OutputMedium root, domain layers generic over it

The trait hierarchy has two axes: the **medium** (how to express primitives)
and the **domain** (what structural logic to apply). Domain traits are generic
over the medium.

```rust
// ═══════════════════════════════════════════════════════════════════
// Axis 1: Output Medium — the leaf of the rendering tree
// ═══════════════════════════════════════════════════════════════════

/// The root trait. Every renderer bottoms out here.
/// Associated type `Output` is the medium's native format.
pub trait OutputMedium {
    /// What this medium produces. String for text, RenderSurface for graphics.
    type Output;

    fn render_span(&self, span: &Span) -> Self::Output;
    fn render_line(&self, line: &Line) -> Self::Output;
    fn render_block(&self, block: &Block) -> Self::Output;

    /// Compose multiple outputs into one (join lines, stack boxes, etc.)
    fn compose(&self, parts: Vec<Self::Output>) -> Self::Output;
}

/// Text medium: Output = String.
/// All text-based rendering (ANSI, HTML, plain) implements this.
pub trait TextMedium: OutputMedium<Output = String> {}

/// Graphics medium: Output = RenderSurface.
/// All visual rendering (SVG, Canvas, PDF) implements this.
/// Stub — no implementations yet. The trait exists so domain layers
/// can be generic over it from day one.
pub trait GraphicsMedium: OutputMedium<Output = RenderSurface> {}

/// Placeholder for graphics output. Will hold positioned glyphs,
/// bounding boxes, connection lines, etc.
pub struct RenderSurface {
    pub elements: Vec<GraphicsElement>,
}

/// Stub enum for graphics primitives.
pub enum GraphicsElement {
    /// Positioned text with font/size/color
    Glyph { text: String, x: f64, y: f64, style: SpanStyle },
    /// A rectangle (code block background, node box, etc.)
    Rect { x: f64, y: f64, w: f64, h: f64, fill: Option<SemanticColor> },
    /// A line/arrow (edge, dependency, flow)
    Path { points: Vec<(f64, f64)>, stroke: Option<SemanticColor> },
}

// ═══════════════════════════════════════════════════════════════════
// Axis 2: Domain Layers — generic over OutputMedium
// ═══════════════════════════════════════════════════════════════════

/// Code layer: language-specific syntax, generic over medium.
pub trait CodeRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_file(&self, file: &TestFile) -> M::Output;
    fn render_expr(&self, expr: &Expr) -> M::Output;
    fn render_stmt(&self, stmt: &Stmt) -> M::Output;
    fn render_import(&self, import: &Import) -> M::Output;
}

/// Markup layer: document formatting, generic over medium.
pub trait MarkupRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_node(&self, node: &MarkupNode) -> M::Output;
    fn render_document(&self, nodes: &[MarkupNode]) -> M::Output;
}

/// Structured layer: targets/sections, generic over medium.
pub trait StructuredRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_target(&self, target: &Target) -> M::Output;
    fn render_category(&self, category: &Category) -> M::Output;
    fn render_block(&self, block: &StructuredBlock) -> M::Output;
}

/// Frame layer: temporal output, generic over medium.
pub trait FrameRenderer<M: OutputMedium> {
    fn medium(&self) -> &M;
    fn render_frame(&self, frame: &Frame) -> M::Output;
}

/// Top-level: renders a complete Document, generic over medium.
pub trait DocumentRenderer<M: OutputMedium> {
    fn render(&self, doc: &Document) -> M::Output;
}
```

Concrete implementations compose medium + domain:

**Text medium implementations** (Phase 1 — all implemented):

| Impl | Trait | What it adds |
|------|-------|-------------|
| `AnsiText` | `TextMedium` | ANSI escape codes, color, bold |
| `PlainText` | `TextMedium` | No escapes, CI-friendly |
| `HtmlText` | `TextMedium` | `<span>` tags, CSS classes |

**Graphics medium implementations** (stubs — trait + types exist, no impls yet):

| Impl | Trait | What it would add |
|------|-------|-------------------|
| `SvgGraphics` | `GraphicsMedium` | SVG elements, viewBox layout |
| `CanvasGraphics` | `GraphicsMedium` | Canvas draw commands |
| `PdfGraphics` | `GraphicsMedium` | PDF content streams |

**Domain implementations** (generic over medium):

| Impl | Domain | Example instantiations |
|------|--------|----------------------|
| `RustCodeRenderer<M>` | `CodeRenderer<M>` | `<AnsiText>` → terminal, `<HtmlText>` → web, `<SvgGraphics>` → visual AST |
| `PythonCodeRenderer<M>` | `CodeRenderer<M>` | `<PlainText>` → file output |
| `GfmMarkupRenderer<M>` | `MarkupRenderer<M>` | `<PlainText>` → `.md` file, `<HtmlText>` → rendered preview |
| `MakefileRenderer<M>` | `StructuredRenderer<M>` | `<PlainText>` → Makefile, `<AnsiText>` → colored preview |
| `YamlRenderer<M>` | `StructuredRenderer<M>` | `<PlainText>` → CI YAML |
| `AnsiFrameRenderer` | `FrameRenderer<AnsiText>` | Terminal animation |
| `PlainFrameRenderer` | `FrameRenderer<PlainText>` | CI log output |

The composability is the point: `RustCodeRenderer<AnsiText>` produces
syntax-highlighted terminal output. `RustCodeRenderer<HtmlText>` produces
syntax-highlighted HTML. `RustCodeRenderer<SvgGraphics>` (future) draws
the Rust AST as a visual diagram. Same structural logic — `render_file`,
`render_expr`, `render_stmt` — different output medium. The domain renderer
never calls `format!()` or builds a string directly; it calls
`self.medium().render_span()` and `self.medium().compose()` and gets back
whatever `M::Output` is.

### Emission Registry (`core/ir/src/emit_registry.rs`)

The model proposes types and a pattern, but doesn't say **where the system
learns what exists**. Without a registry, you get the same drift: someone adds
a new generator and forgets to wire it into Make/CI/verification.

The emission registry follows the same principle as testgen's registry: **metadata
only, no function refs** (avoids circular deps). It advertises "things that can
be emitted" and lets downstream consumers (makegen, CI gen, verification)
auto-derive targets.

```rust
/// What an emission pipeline produces. Registered once, consumed everywhere.
pub struct ArtifactDef {
    pub id: ArtifactId,               // e.g., "testgen:llm_ops"
    pub path: String,                 // output path relative to workspace root
    pub format: FormatId,             // "rust", "makefile", "yaml", "toml", ...
    pub generator: &'static str,     // "testgen", "makegen", "ci_gen", ...
    pub regenerate_command: &'static str,  // "make testgen", "make codegen", ...
}

/// Maps FormatId → medium-specific renderers.
pub struct FormatRegistry { ... }

/// All known emission targets. Populated via inventory.
pub struct EmissionRegistry { ... }
```

Invariants enforced by tests:
- Every `ArtifactDef.format` must exist in `FormatRegistry`
- Every tool that declares testgen targets must have matching emission defs
- Every emission pipeline must declare its write resource (filesystem / stdout)
- `make list-artifacts` dumps the full registry for human inspection

This is what enables "add a DAG once, everything updates": the emission
registry is the single source of truth that makegen, CI gen, and verification
all consume. No manual target lists.

### Crate boundary constraint

> **RenderIR cannot depend on codegen IR types** unless those types move into
> a shared crate/module.

`DocumentBody::Code(TestFile)` implies `core/ir` depends on `core/codegen`
(where `TestFile` lives). That's a dependency inversion — `core/codegen`
depends on `core/ir`, not the other way.

Two coherent options:

**Option A (preferred): move Code IR into `core/ir`.**
`TestFile`/`TestFn`/`Stmt`/`Expr`/`Assert` move to `core/ir/src/code_ir.rs`.
They're already language-neutral and have no codegen-specific deps. Testgen,
cli_gen, dag_gen all depend on `core/ir` for the Code IR. This is the same
move we did for `ResourceId` → `core/infra`: pull the type to the lowest
crate that needs it.

**Option B: keep RenderIR generic, lower separately.**
`DocumentBody` doesn't embed `TestFile` directly. Instead, codegen produces
`TestFile`, a lowering pass converts `TestFile → Document<StructuredBlock>`,
and the renderer only sees the lowered form. Cleaner layering but more
indirection.

Phase 2 must resolve this before Code IR integration. Option A is the path of
least resistance given the existing crate structure.

### Data IR for structured formats (`core/ir/src/data_ir.rs`)

`Target { body: Block }` is sufficient for Makefile rules, but CI jobs are
nested maps with steps, env, conditions, etc. Flattening them to text blocks
too early recreates the string template trap.

For YAML/TOML/JSON sharing, the Structured layer needs a **data DOM**:

```rust
/// A structured data value. Renderable to YAML, TOML, JSON, etc.
pub enum DataValue {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<DataValue>),
    Map(Vec<(String, DataValue)>),  // ordered to preserve key order
}

/// A data node with optional annotation (comment, anchor, etc.)
pub struct DataNode {
    pub value: DataValue,
    pub comment: Option<String>,
}
```

`YamlRenderer<M>` renders `DataValue` to YAML syntax. `TomlRenderer<M>`
renders it to TOML. `JsonRenderer<M>` renders it to JSON. The CI step
builder produces `DataValue::Map(...)` with the full job structure — env,
steps, matrix, conditions — and the renderer only decides syntax.

`Target` then composes `DataValue`:

```rust
pub struct Target {
    pub name: String,
    pub deps: Vec<TargetRef>,    // typed, not strings
    pub body: DataValue,          // structured, not text
    pub metadata: DataValue,      // format-specific annotations
}
```

This means the same CI job data can render to GitHub Actions YAML *and*
GitLab CI YAML *and* a visual graph, because it's structured data all the
way down.

### Streaming vs file rendering

`OutputMedium::Output = String` works for file-like outputs. It doesn't work
for terminal frames with timing, cursor control, and incremental writes.

The `OutputMedium` trait already has an associated `Output` type, so this
is handled naturally:

- `TextMedium` → `Output = String` (buffered, for files)
- `GraphicsMedium` → `Output = RenderSurface` (scene graph, for visual)

For streaming, `FrameRenderer<M>` doesn't call `render() -> String` and
then write the whole string. It calls `render_frame()` per frame, with the
medium writing incrementally. For `AnsiText`, this means:

```rust
impl FrameRenderer<AnsiText> for AnsiFrameRenderer {
    fn render_frame(&self, frame: &Frame, sink: &mut dyn Write) {
        // cursor control + incremental line writes
        // sink is the TTY or a test buffer
    }
}
```

For test snapshots, the sink is a `Vec<u8>` — capture the output, assert
against golden files. This matches the existing progress display's
"render into a buffer for snapshot tests" pattern.

### `DocumentBody::Raw(String)` governance

`Raw(String)` is the escape hatch that lets the old world re-enter. It's
necessary during migration (incremental adoption) but must not persist.

Rules:
- `Raw` is only allowed in leaf migrations or where a proper IR is not yet defined
- A test counts `Raw` usage across the codebase — this count must not increase
- After Phase 5, `Raw` usage should be zero (or have explicit justification comments)
- Each `Raw` site has a tracking comment: `// TODO(emission): replace with StructuredBlock`

The Definition of Done (criterion 10) already requires "no ad-hoc emission
outside the renderer model" — this makes `Raw(String)` = 0 an explicit
part of that.

### Emit Pattern (`core/ir/src/patterns/emit.rs`)

A new pattern builder that models the full emission pipeline as a SubDag:

```
EmitBuilder::new("write_tests")
    .with_prepare(TestgenOp::BuildTestFile)   // pure: DAG → TestFile IR
    .with_format(TestgenOp::RenderToRust)     // pure: TestFile → M::Output
    .with_hash(HashContentSha256)             // pure: compute content hash
    .with_skip_if_unchanged()                 // pure: compare to existing file hash
    .with_write(TransportOps::Execute)        // boundary: write file (skipped if unchanged)
    .with_verify()                            // record (path, content_hash, input_hash) → manifest
    .with_diff(DiffSummary)                   // boundary: report what changed (dev ergonomics)
    .build()
```

The full pipeline:

```
┌──────────┐   ┌──────────┐   ┌────────┐   ┌───────────┐   ┌───────┐   ┌────────┐   ┌──────┐
│ Prepare  │──▶│  Format  │──▶│  Hash  │──▶│ Skip-if-  │──▶│ Write │──▶│ Verify │──▶│ Diff │
│  (pure)  │   │  (pure)  │   │ (pure) │   │ unchanged │   │(bound)│   │(record)│   │(show)│
└──────────┘   └──────────┘   └────────┘   └───────────┘   └───────┘   └────────┘   └──────┘
  Build IR     Apply renderer  Content    Compare to        Write if   Record in    Show
               via Medium      hash       existing file     changed    manifest     changes
```

This addresses the full lifecycle:
- **Rendering becomes a first-class DAG citizen**
- **Dry-run mode** automatically intercepts the Write step
- **Skip-if-unchanged** avoids unnecessary writes (no spurious mtime bumps)
- **Content hash** enables staleness detection without re-rendering
- **The Format step is swappable** (render to Rust vs. Python vs. TypeScript)
- **The Prepare step is reusable** across formats
- **Verify** records `(path, content_hash, input_hash)` to `.emit-manifest.json`
  — CI catches hand-edits and stale generators
- **Diff** reports what changed for developer ergonomics (`make codegen` shows a summary)

The manifest records both **output hash** (does the file match the generator?)
and optionally **input hash** (did the generator's inputs change?). This
connects to the broader input freshness story without blocking it.

### Terminal as a DAG

The animated progress display becomes a rendering pipeline:

```
ExecuteDAG → CollectProgress → ComputeLayout → BuildFrames → EmitFrames
  (exec)       (pure)           (pure)          (pure)        (boundary)
```

Where `BuildFrames` produces `Vec<Frame>` (medium-agnostic `Line`/`Span`
content with `CursorAction`) and `EmitFrames` writes them via a
`FrameRenderer<M>`. Today `M = AnsiText` for terminals, `M = PlainText` for
CI. Tomorrow `M = SvgGraphics` for visual replays — same frame IR, different
medium.

---

## Migration Path

### Existing inventory (what must be fully replaced)

Every item below must be migrated or explicitly removed. No dual-path code.

**Old traits at audit time (5; CI cleanup already removed one family):**

| Trait | Location | Impls | Phase removed |
|-------|----------|-------|---------------|
| `Renderable` | `core/ir/src/render.rs` | `MakefileRenderer`, `ClippyConfigRenderer`, `WorkflowConfig` (historical), `IgnoreCategorySet` | 3 |
| `TestRenderer` | `core/codegen/src/testgen/render.rs` | `RustRenderer`, `PythonRenderer`, `TypeScriptRenderer` | 2 |
| `CiRenderer` | `core/ir/src/transport/ci/render.rs` | `GitHubActionsProvider`, `GitLabCiProvider` (historical; deleted 2026-03-06) | 5 |
| `FrameLoop` | `core/exec/src/render.rs` | `TerminalRenderer<W>` | 4 |
| `CiProvider` | `core/ir/src/transport/ci/providers/` | `GitHubProvider`, `GitLabProvider`, `PlainProvider` | 5 |

**Old IRs (2):**

| IR | Location | Consumers | Replaced by |
|----|----------|-----------|-------------|
| `SharedStep` | `core/ir/src/transport/ci/render.rs` | 12 files at audit time (historical; deleted 2026-03-06) | `StructuredBlock::Target` + `YamlRenderer` |
| `IgnoreCategory` | `gunbc-app/src/makegen/gitignore.rs` | gitignore generation | `Category` IR |

**Old rendering structs (3):**

| Struct | Location | Replaced by |
|--------|----------|-------------|
| `TerminalRenderer<W>` | `core/exec/src/render.rs` | `FrameRenderer<AnsiText>` |
| `Animation` | `core/exec/src/render.rs` | Animation state moves into `Frame` timing metadata |
| `RenderMode` | `core/exec/src/render.rs` | Medium selection + layout mode (not rendering logic) |

**Trait-less emission sites (8) — no trait, no IR, must be migrated:**

| Site | Location | Generates | Phase migrated |
|------|----------|-----------|---------------|
| Pragma allowlist | `gunbc-app/src/policy/pragma.rs` | `disallowed-methods-allowlist.txt` | 3 |
| Pragma lint policy | `gunbc-app/src/policy/pragma.rs` | `pragma-lint-policy.txt` | 3 |
| DAG code gen | `core/codegen/src/dag_gen.rs` | `graph.rs` Rust source | 2 |
| CI report | `gunbc-app/src/ci/ops.rs` | Plain-text CI report | 3 |
| Markdown snapshots | `lib/markdown/src/lib.rs` | Code/diff markdown blocks | 5 |
| LLM prompts | `lib/review/src/lib.rs` | Structured prompts | 5 |
| CI workflow commands | `core/ir/src/transport/ci/providers/` | Runtime `::group::` / ANSI commands | 5 |
| WorkflowConfig YAML | `core/ir/src/transport/github_actions.rs` | GitHub Actions YAML (redundant) | 5 |

**Existing infra that survives (already medium-agnostic):**

| Type | Location | Why it stays |
|------|----------|-------------|
| `SemanticColor` | `core/ir/src/symbols.rs` | Already semantic — `SpanStyle.color` references it directly |
| `SymbolId` | `core/ir/src/symbols.rs` | Already abstract — `SpanStyle.symbol` references it directly |
| `SymbolSet` + `Tier` | `core/ir/src/symbols.rs` | Medium resolves SymbolId via Tier — this IS the medium's job |
| `DagProgress` | `core/exec/src/progress.rs` | Execution state machine, not rendering — stays as input to Frame building |
| `DagLayout` | `core/ir/src/layout.rs` | Spatial computation — already has `ViewportUnit::Chars` vs `Pixels` |
| `ProgressObserver` | `core/exec/src/progress.rs` | Execution observation — feeds DagProgress, not tied to rendering |
| `FileWriter` | `core/codegen/src/file_writer.rs` | Write infrastructure (dry-run vs real) — stays, used by Emit pattern |
| `Template` | `core/codegen/src/template.rs` | `{{var}}` substitution utility — unused today, may serve as text helper |

---

### Phase 1: OutputMedium + content IR + graphics stubs (non-breaking)

**Add:**
- `Span`, `SpanStyle`, `Line`, `Block` content primitives to `core/ir/src/render_ir.rs`
- `SpanStyle` references **existing** `SemanticColor` and `SymbolId` from `symbols.rs` — no duplication
- `OutputMedium` trait with associated `Output` type
- `TextMedium` marker trait + `AnsiText` and `PlainText` impls (`Output = String`)
- `GraphicsMedium` marker trait + `RenderSurface`/`GraphicsElement` stubs (no impls yet)
- `Document`, `DocumentBody` top-level types

**Remove:** nothing — purely additive, existing systems unchanged

**Invariant:** `SemanticColor` and `SymbolId` are used from `symbols.rs`, not redefined.
`DagLayout::Viewport` with `ViewportUnit::{Chars, Pixels}` is acknowledged as
the existing medium-aware spatial layer — graphics medium will use `Pixels`.

**Files**: `core/ir/src/render_ir.rs` (new), `core/ir/src/render.rs` (evolve), `core/ir/src/lib.rs`

**Test**: types compile, `AnsiText`/`PlainText` pass round-trip span tests, graphics stubs exist

### Phase 2: Code layer — migrate ALL Rust code generation

This phase covers **every system that generates Rust source code**: testgen,
DAG code gen, and CLI gen. All three use `format!()` string templates today.
After this phase, all Rust emission flows through `CodeRenderer<M>`.

**Add:**
- `CodeRenderer<M: OutputMedium>` trait in `core/ir/src/render.rs`
- `RustCodeRenderer<M>` implementing `CodeRenderer<M>`
- `PythonCodeRenderer<M>` (stub, same scope as current `PythonRenderer`)
- `TypeScriptCodeRenderer<M>` (stub, same scope as current `TypeScriptRenderer`)
- Rename `testgen/codegen.rs` → `testgen/emit.rs`
- TestFile/Expr/Stmt gain `Span` leaves (replacing raw strings at boundaries)
- DAG code gen (`dag_gen.rs`) rewritten to build Code IR + `RustCodeRenderer<PlainText>`
- CLI gen (`cli_gen.rs`) rewritten to build Code IR + `RustCodeRenderer<PlainText>`

**Remove:**
- `TestRenderer` trait — deleted entirely from `core/codegen/src/testgen/render.rs`
- `RustRenderer` struct — replaced by `RustCodeRenderer<M>`
- `PythonRenderer` struct — replaced by `PythonCodeRenderer<M>`
- `TypeScriptRenderer` struct — replaced by `TypeScriptCodeRenderer<M>`
- All 7 downstream files importing `TestRenderer` updated to `CodeRenderer<M>`
- All `format!()` Rust source templates in `dag_gen.rs` — replaced by Code IR
- All `format!()` Rust source templates in `cli_gen.rs` — replaced by Code IR

**Verify:** `RustCodeRenderer<M>` compiles for *any* `M: OutputMedium` — graphics
medium is valid even with no impl. Generated test files byte-identical.
Generated `graph.rs` files byte-identical. Generated CLI files byte-identical.

**Zero stragglers:** `grep -r "TestRenderer\|RustRenderer\b" core/codegen/` returns nothing.
No `format!()` calls in `dag_gen.rs` or `cli_gen.rs` that construct Rust syntax.

**Files**: `core/codegen/src/testgen/emit.rs`, `core/codegen/src/testgen/render.rs`,
`core/codegen/src/testgen/render_rust.rs`, `core/codegen/src/testgen/render_python.rs`,
`core/codegen/src/testgen/render_ts.rs`, `core/codegen/src/dag_gen.rs`,
`core/codegen/src/cli_gen.rs`, `core/ir/src/render.rs`

### Phase 3: Structured layer — migrate ALL Renderable impls + trait-less structured emitters

This phase covers **every system that generates structured non-code artifacts**:
Makefiles, .gitignore, clippy.toml, pragma text files, CI reports. After this
phase, all structured emission flows through `StructuredRenderer<M>`.

**Critical: separate model from policy from presentation in makegen.**
Today, `render_meta_target()` hardcodes PrepLevel → deps mapping (policy),
tool targets blanket-depend on `ensure-codegen` (policy), and meta-target deps
are unverified strings like `"testgen-check"` (stringly modeling). The fix:

1. **Model**: typed `TargetRef` / dependency graph — what depends on what
2. **Policy**: derived from per-tool declarations — which deps apply
3. **Presentation**: `MakefileRenderer<M>` receives a resolved graph — how to express it

The renderer should never decide what depends on what. It should only decide
how to *express* a fully-resolved dependency graph in Makefile/Justfile/Taskfile
syntax. Structure ends at the last possible moment.

**Add:**
- `StructuredRenderer<M: OutputMedium>` trait
- `Target`, `Category`, `StructuredBlock` IR types in `render_ir.rs`
- `TargetRef` type for compile-time validated target dependencies (replaces raw strings)
- `MakefileRenderer<M>` implementing `StructuredRenderer<M>`
- `ClippyConfigRenderer<M>` implementing `StructuredRenderer<M>`
- `IgnoreCategorySet<M>` implementing `StructuredRenderer<M>`
- `CiReportRenderer<M>` implementing `StructuredRenderer<M>` — CI report as sections
- `DocumentRenderer<M>` trait — replaces `Renderable`'s header-generation role
- `FileHeader` struct (generator_name, regenerate_command, comment_prefix) — the
  useful part of `Renderable`, now data instead of trait
- Pragma allowlist/policy rendering migrated through `DocumentRenderer<M>` + `FileHeader`

**Remove:**
- `Renderable` trait — deleted entirely from `core/ir/src/render.rs`
- Old `MakefileRenderer` struct (not generic) — replaced by `MakefileRenderer<M>`
- Old `ClippyConfigRenderer` struct — replaced
- Old `WorkflowConfig` `Renderable` impl — replaced (its CI rendering moves to Phase 5)
- Old `IgnoreCategorySet` `Renderable` impl — replaced
- `IgnoreCategory` struct — replaced by `Category` IR type
- Manual "Generated by" headers in `pragma.rs` — replaced by `FileHeader`
- Ad-hoc `format!()` CI report in `ci/ops.rs` — replaced by `CiReportRenderer<M>`
- All 9 downstream files importing `Renderable` updated
- All hardcoded PrepLevel → deps mappings in render functions — replaced by typed graph

**Verify:** `MakefileRenderer<PlainText>` produces byte-identical Makefiles.
`ClippyConfigRenderer<PlainText>` produces byte-identical clippy configs.
Pragma text files byte-identical. CI report output byte-identical.

**Zero stragglers:** `grep -r "Renderable\b\|IgnoreCategory\b" --include='*.rs'` returns
nothing. No manual "Generated by" / "DO NOT EDIT" string construction outside `FileHeader`.

**Files**: `core/ir/src/render.rs`, `core/ir/src/render_ir.rs`,
`gunbc-app/src/makegen/render.rs`, `gunbc-app/src/makegen/gitignore.rs`,
`lib/tools/clippy/src/config.rs`, `gunbc-app/src/policy/pragma.rs`,
`gunbc-app/src/ci/ops.rs`

### Phase 4: Frame layer — migrate ALL terminal rendering

**Add:**
- `FrameRenderer<M: OutputMedium>` trait
- `Frame` IR type (medium-agnostic `Line`/`Span` + `CursorAction`)
- `AnsiFrameRenderer` implementing `FrameRenderer<AnsiText>`
- `PlainFrameRenderer` implementing `FrameRenderer<PlainText>`
- Frame-building pure function: `DagProgress` + `DagLayout` + `SymbolSet` → `Vec<Frame>`
- Animation timing metadata on `Frame` (replaces `Animation` struct's role)

**Remove:**
- `TerminalRenderer<W>` struct — deleted entirely
- `FrameLoop` trait — deleted (replaced by `FrameRenderer<M>` + frame-building function)
- `Animation` struct — timing/state absorbed into frame-building logic
- `RenderMode` enum — rendering style becomes medium selection + layout configuration
- All `impl FrameLoop for TerminalRenderer` code

**Survives (not removed):**
- `DagProgress` — stays as execution state machine input
- `DagLayout` — stays as spatial computation input
- `SymbolSet` — stays; `AnsiText` medium uses it to resolve `SymbolId` → char
- `ProgressObserver` — stays; feeds `DagProgress`

**Verify:** `make gist --dry-run` visual output identical. CI plain-text output identical.

**Zero stragglers:** `grep -r "TerminalRenderer\|FrameLoop\b" core/exec/` returns nothing.

**Files**: `core/exec/src/render.rs`, `core/exec/src/display.rs`, `core/exec/src/lib.rs`

### Phase 5: Emit pattern + CI + markup + remaining emitters

This is the final migration phase. After this, **every emission site in the
codebase** flows through `OutputMedium`. This phase also introduces the Emit
pattern (rendering-as-DAG) and the verification manifest.

**Add:**
- `EmitBuilder` pattern in `core/ir/src/patterns/emit.rs`
- Emit manifest: every `EmitBuilder` records content hash of output → `.emit-manifest.json`
- CI verification step: recompute hashes, compare to manifest, fail on drift
- `MarkupRenderer<M>` trait + `MarkupNode` IR
- `YamlRenderer<M>` implementing `StructuredRenderer<M>` for CI workflow YAML
- `GitHubActionsRenderer<M>` implementing CI-specific logic over `StructuredRenderer<M>`
- `GitLabCiRenderer<M>` implementing CI-specific logic over `StructuredRenderer<M>`
- Markdown snapshots (`lib/markdown/src/lib.rs`) rewritten to use `MarkupNode` IR +
  `MarkupRenderer<PlainText>`
- LLM prompt construction (`lib/review/src/lib.rs`) rewritten to use `MarkupNode` IR
  (prompts are structured documents — headings, lists, code blocks)
- CI workflow command formatting (`CiProvider::format()`) migrated to
  `StructuredRenderer<M>` — GitHub uses `PlainText` (annotations are text),
  GitLab uses `AnsiText` (collapsible sections use ANSI)
- `WorkflowConfig` YAML rendering consolidated into `GitHubActionsRenderer<M>` —
  the redundant parallel renderer is deleted

**Remove:**
- `CiRenderer` trait — deleted entirely from `core/ir/src/transport/ci/render.rs`
- `CiProvider` trait (format method) — deleted; runtime commands go through `StructuredRenderer`
- `SharedStep` enum — deleted; CI steps become `Target`/`StructuredBlock` IR nodes
- `dag_to_shared_steps()` — replaced by DAG → `Vec<StructuredBlock>` builder
- `GitHubActionsProvider` struct — replaced by `GitHubActionsRenderer<M>`
- `GitLabCiProvider` struct — replaced by `GitLabCiRenderer<M>`
- `PlainProvider` struct — replaced by `StructuredRenderer<PlainText>`
- `WorkflowConfig` struct — consolidated into `GitHubActionsRenderer<M>`
- `RenderConfig` — fields absorbed into `Document` metadata + `FileHeader`
- Ad-hoc `push_str()` markdown in `lib/markdown/src/lib.rs` — replaced by `MarkupNode` IR
- Ad-hoc `Vec<String>.join()` prompts in `lib/review/src/lib.rs` — replaced by `MarkupNode` IR
- All 12 downstream files importing `SharedStep`/`CiRenderer` updated

**Remove `WorkflowConfig`'s old `Renderable` impl** if not already removed in Phase 3
(it implements both `Renderable` and produces YAML — both paths must be gone).

**Verify:** Generated CI YAML byte-identical. Markdown snapshots byte-identical.
LLM prompts byte-identical. CI workflow command output byte-identical.
Emit manifest correctly records content hashes for all generated artifacts.

**Zero stragglers:** `grep -r "CiRenderer\|SharedStep\|dag_to_shared_steps\|CiProvider" --include='*.rs'`
returns nothing. `grep -r "Renderable\|TestRenderer\|FrameLoop" --include='*.rs'` returns nothing.
No `push_str()` or `format!()` building markdown, YAML, or prompt content outside renderers.

**Files**: `core/ir/src/patterns/emit.rs`, `core/ir/src/transport/ci/`,
`core/ir/src/transport/github_actions.rs`, `lib/markdown/src/lib.rs`,
`lib/review/src/lib.rs`, `core/codegen/src/main.rs`

### Phase 6 (future): Graphics medium

- Implement `SvgGraphics` as first `GraphicsMedium` impl
- All existing `CodeRenderer<M>`, `StructuredRenderer<M>`, `FrameRenderer<M>` become
  instantiable with `<SvgGraphics>` — no changes to domain renderers
- `RustCodeRenderer<SvgGraphics>` draws AST as visual diagram
- `FrameRenderer<SvgGraphics>` produces SVG animation of DAG execution
- `DagLayout` already supports `ViewportUnit::Pixels` — graphics medium uses it directly
- **Files**: `core/ir/src/render_svg.rs` (new), no changes to domain renderers

---

## Cross-Cutting Concerns

The emission model intersects with other architectural themes. These are not
fully solved by the emission work alone, but the emission model must be
designed so it doesn't *block* them, and where possible, *enables* them.

### Environment/context flows through the medium, not the renderer

Renderers need ambient state: TTY capabilities, viewport dimensions, Unicode
tier, color support, working directory, CI detection. Today each renderer
acquires this ad-hoc (`TerminalProfile::detect()`, env var reads, etc.).

The `OutputMedium` model solves this: **the medium holds the context**. An
`AnsiText` instance is constructed with TTY info, viewport, and tier — domain
renderers never touch ambient state directly. They call
`self.medium().render_span()` and the medium resolves `SemanticColor::Success`
to green ANSI or `[PASS]` plain text based on its own capabilities.

This is critical because if the medium doesn't own context, renderers will
reinvent context acquisition, and we're back to the same "env access leaks
into business logic" anti-pattern. The rule:

> **Domain renderers are pure functions of (IR, Medium).** They never call
> `std::env`, detect TTY, or read capabilities. The Medium is the only
> boundary-aware component.

### Model-driven target generation (testgen → makegen/CI auto-wiring)

`all_testgen_targets()` already produces the registry of test generation
targets, but makegen and CI gen don't consume it — they maintain parallel
lists of targets manually. This is "model exists but doesn't drive everything
yet," which is exactly how drift re-enters.

Phase 3 (Structured layer) must wire this: `MakefileRenderer<M>` receives
its target list from the model (testgen registry, tool registry, etc.), not
from hardcoded lists. The `TargetRef` typed dependency system enables this —
you can't accidentally forget a target if the model generates the list.

### Input freshness vs output verification (two distinct problems)

The Emit pattern's `.emit-manifest.json` solves **output verification**: "does
the generated file match what the generator would produce right now?" This
catches hand-edits.

But there's a separate problem: **input freshness** — "did the inputs to the
generator change since last run?" The current codegen freshness model uses
glob patterns to discover inputs; if inputs change outside those globs, stale
artifacts slip through.

The structural fix (content-hash manifest of *inputs*) is orthogonal to the
emission model but complementary. The emission model should not block it.
Concretely: `EmitBuilder` should accept an optional input hash alongside the
output hash, so a single manifest can record both "what went in" and "what
came out." This is not required for emission Done but is the natural extension.

### Build resource chain (adjacent, not blocked)

The "build artifacts as resources" design (URGENT, unassigned) describes the
deeper problem: build artifacts should be DAG-modeled resources with typed
provide/need chains, so `make test-fix` doesn't use `testgen-check` when it
should regenerate.

The Emit pattern is a step toward this — it models generation as a DAG
pipeline with Prepare → Format → Write → Verify. But it doesn't fully solve
the resource chain (which requires mode flow: Verify vs Ensure through
dependency chains, not just per-artifact).

The emission model should be compatible with the resource chain when it lands:
an Emit pipeline should be expressible as a resource provider (the Write step
provides the artifact resource, downstream nodes consume it). This is a
future integration point, not a Phase 1-5 deliverable.

### Coverage matrix

Where each emission system stands relative to the four structural properties
the model requires:

| System | Has typed IR | Has pluggable renderer | Emission is a DAG pipeline | Output verified |
|--------|:---:|:---:|:---:|:---:|
| Testgen (tests) | yes | yes (multi-lang incomplete) | no | no |
| Testgen (dag_gen) | no | no | no | no |
| Testgen (cli_gen) | no | no | no | no |
| Makegen (Makefile) | no | no | no | no |
| Makegen (gitignore) | partial (IgnoreCategory) | no | no | no |
| Clippy config | no | no | no | no |
| Pragma text files | no | no | no | no |
| Terminal progress | yes (DagProgress + DagLayout) | partial (terminal only) | partial | no |
| CI YAML (GitHub) | partial (`config.ci` + provider schema) | no live repo-YAML trait | no | no |
| CI YAML (GitLab) | partial (`config.ci` + provider schema) | no live repo-YAML trait | no | no |
| CI report | no | no | no | no |
| Markdown snapshots | no | no | no | no |
| LLM prompts | no | no | no | no |

After Phase 5, every row should be: **yes | yes | yes | yes**.

---

## Definition of Done

This work is **done** when all of the following are true. Each criterion is
mechanically verifiable — no judgement calls.

### 1. Build passes

```bash
cargo test --workspace        # all tests pass
cargo clippy --all-targets -- -D warnings  # no warnings
```

### 2. Zero behavioral regression

Every artifact the codebase generates must be **byte-identical** before and
after migration. All 13 emission sites covered:

| # | Artifact | Old system | New system | Verification |
|---|----------|-----------|-----------|-------------|
| 1 | `*_generated_tests.rs` | TestRenderer | `RustCodeRenderer<PlainText>` | `diff` before/after |
| 2 | Makefile | Renderable + string concat | `MakefileRenderer<PlainText>` | `diff` before/after |
| 3 | .gitignore | Renderable (IgnoreCategorySet) | `StructuredRenderer<PlainText>` | `diff` before/after |
| 4 | clippy.toml | Renderable (ClippyConfigRenderer) | `StructuredRenderer<PlainText>` | `diff` before/after |
| 5 | Terminal frames | TerminalRenderer + FrameLoop | `FrameRenderer<AnsiText>` | Visual + snapshot |
| 6 | CI YAML (GitHub) | CiRenderer (GitHubActionsProvider) | `GitHubActionsRenderer<PlainText>` | `diff` before/after |
| 7 | CI YAML (GitLab) | CiRenderer (GitLabCiProvider) | `GitLabCiRenderer<PlainText>` | `diff` before/after |
| 8 | Generated CLI source | format!() templates | `RustCodeRenderer<PlainText>` | `diff` before/after |
| 9 | Generated `graph.rs` | format!() templates (dag_gen) | `RustCodeRenderer<PlainText>` | `diff` before/after |
| 10 | Pragma text files | Manual header + push_str() | `DocumentRenderer<PlainText>` | `diff` before/after |
| 11 | CI report | format!() in DAG op | `StructuredRenderer<PlainText>` | `diff` before/after |
| 12 | Markdown snapshots | push_str() (MarkdownOp) | `MarkupRenderer<PlainText>` | `diff` before/after |
| 13 | LLM prompts | Vec<String>.join() | `MarkupRenderer<PlainText>` | `diff` before/after |

Plus CI workflow commands (GitHub `::group::`, GitLab ANSI sections, plain
`=== group ===`) and the legacy `WorkflowConfig` YAML (consolidated into #6).

### 3. All old rendering traits deleted

```
grep -r "trait Renderable\b"    --include='*.rs'  → 0 matches
grep -r "trait TestRenderer\b"  --include='*.rs'  → 0 matches
grep -r "trait CiRenderer\b"    --include='*.rs'  → 0 matches
grep -r "trait FrameLoop\b"     --include='*.rs'  → 0 matches
grep -r "trait CiProvider\b"    --include='*.rs'  → 0 matches (format() method)
```

### 4. All old IRs deleted

```
grep -r "enum SharedStep\b"       --include='*.rs'  → 0 matches
grep -r "struct IgnoreCategory\b" --include='*.rs'  → 0 matches
```

### 5. All old rendering structs deleted

```
grep -r "struct TerminalRenderer\b" --include='*.rs'  → 0 matches
grep -r "struct Animation\b" core/exec/              → 0 matches
grep -r "enum RenderMode\b"        --include='*.rs'  → 0 matches
grep -r "struct WorkflowConfig\b"  --include='*.rs'  → 0 matches (consolidated)
grep -r "struct RenderConfig\b"    --include='*.rs'  → 0 matches (absorbed into Document)
```

### 6. No imports of removed types

```
grep -r "use.*Renderable\b"     --include='*.rs'  → 0 matches
grep -r "use.*TestRenderer\b"   --include='*.rs'  → 0 matches
grep -r "use.*CiRenderer\b"     --include='*.rs'  → 0 matches
grep -r "use.*FrameLoop\b"      --include='*.rs'  → 0 matches
grep -r "use.*SharedStep\b"     --include='*.rs'  → 0 matches
grep -r "use.*CiProvider\b"     --include='*.rs'  → 0 matches
```

### 7. Unified model fully in place

```
grep -r "trait OutputMedium\b"   --include='*.rs'  → 1 match (definition)
grep -r "impl.*TextMedium\b"    --include='*.rs'  → 3 matches (AnsiText, PlainText, HtmlText)
grep -r "trait GraphicsMedium\b" --include='*.rs'  → 1 match (stub, 0 impls)
grep -r "trait CodeRenderer\b"   --include='*.rs'  → 1 match (definition)
grep -r "trait StructuredRenderer\b" --include='*.rs' → 1 match (definition)
grep -r "trait FrameRenderer\b"  --include='*.rs'  → 1 match (definition)
grep -r "trait MarkupRenderer\b" --include='*.rs'  → 1 match (definition)
grep -r "trait DocumentRenderer\b" --include='*.rs' → 1 match (definition)
grep -r "struct EmitBuilder\b"   --include='*.rs'  → 1 match (definition)
```

### 8. Graphics medium stubs wired correctly

```
grep -r "struct RenderSurface\b"  --include='*.rs'  → 1 match (stub)
grep -r "enum GraphicsElement\b"  --include='*.rs'  → 1 match (stub)
```

And the following must **compile** (even with no graphics impl):

```rust
fn assert_generic_over_graphics<M: OutputMedium>() {
    // These types exist and are generic over any medium:
    let _: fn(&dyn CodeRenderer<M>);
    let _: fn(&dyn StructuredRenderer<M>);
    let _: fn(&dyn FrameRenderer<M>);
    let _: fn(&dyn MarkupRenderer<M>);
    let _: fn(&dyn DocumentRenderer<M>);
}
```

### 9. Existing medium-agnostic infra preserved

```
grep -r "enum SemanticColor\b"   --include='*.rs'  → 1 match (symbols.rs)
grep -r "enum SymbolId\b"        --include='*.rs'  → 1 match (symbols.rs)
grep -r "struct SymbolSet\b"     --include='*.rs'  → 1 match (symbols.rs)
grep -r "struct DagLayout\b"     --include='*.rs'  → 1 match (layout.rs)
grep -r "struct DagProgress\b"   --include='*.rs'  → 1 match (progress.rs)
grep -r "trait ProgressObserver\b" --include='*.rs' → 1 match (progress.rs)
```

### 10. No ad-hoc emission outside the renderer model

No code path constructs artifact content via direct `format!()`, `push_str()`,
`write!()`, or string concatenation. Every emission site flows through:

```
Content IR (Span/Line/Block/...) → Domain Renderer<M> → M::Output
```

Specifically:
- No manual "Generated by" / "DO NOT EDIT" headers — all go through `FileHeader`
- No `format!()` constructing Rust/Python/TypeScript syntax — all go through `CodeRenderer<M>`
- No `push_str()` building markdown — all go through `MarkupRenderer<M>`
- No string concat building Makefile/YAML/TOML — all go through `StructuredRenderer<M>`
- No ANSI escape code construction outside `AnsiText::render_span()` (except GitLab CI, which
  uses `AnsiText` medium through the renderer)

### 11. Policy separated from presentation in makegen

- Makefile target dependencies are `TargetRef` (typed), not raw strings
- PrepLevel → deps mapping is derived from per-tool declarations, not hardcoded in renderer
- Tool targets declare their codegen dependencies explicitly (not blanket `ensure-codegen`)
- `MakefileRenderer<M>` receives a fully-resolved dependency graph — never decides what depends on what

### 12. Emit manifest enables generated artifact verification

- `.emit-manifest.json` exists with `(path, content_hash, input_hash)` for every emitted artifact
- CI can run `make verify-generated` which recomputes hashes and compares to manifest
- A hand-edited generated file causes CI failure with a clear message
- Skip-if-unchanged prevents spurious mtime bumps on identical regeneration
- The manifest is itself a generated file tracked by the Emit pattern

### 13. Emission Registry in place

- `EmissionRegistry` lists all `ArtifactDef`s via inventory — no manual target lists
- Every `ArtifactDef.format` has a matching entry in `FormatRegistry`
- Makegen auto-derives targets from the registry (not hardcoded)
- CI gen auto-derives jobs from the registry (not hardcoded)
- `make list-artifacts` dumps the full registry for human inspection

### 14. Crate boundary respected

- Code IR types (`TestFile`, `Stmt`, `Expr`, `Assert`, etc.) live in `core/ir` (not `core/codegen`)
- `core/ir` does not depend on `core/codegen` — no dependency inversion
- `DocumentBody::Code(TestFile)` is valid because both types are in `core/ir`

### 15. No `DocumentBody::Raw(String)` remaining

- `grep -r "Raw(" core/ lib/ gunbc-app/ --include='*.rs'` in emission-related code → 0 matches
  (or each remaining usage has an explicit `// JUSTIFIED:` comment with rationale)
- A test asserts `Raw` usage count does not increase

### 16. Data IR used for structured formats

- CI job definitions are `DataValue::Map(...)` — not flattened to text
- Same `DataValue` renders to GitHub YAML and GitLab YAML via different renderers
- `YamlRenderer<M>` and `TomlRenderer<M>` consume `DataValue`, not custom structs

---

## What This Enables

Once done, the unified model provides:

| Today | With Unified Model |
|-------|-------------------|
| 13 emission sites, 5 traits, 8 with no trait | 1 `OutputMedium` root, 5 domain renderer traits, 0 ad-hoc emitters |
| Terminal output is ANSI-only | Same frame IR renders to ANSI, HTML, plain-text, SVG |
| Makefile generation mixes model/policy/presentation | Model → policy → presentation cleanly separated; `TargetRef` typed deps |
| CI YAML is disconnected from other rendering | Same `DataValue` IR used by makegen and CI gen |
| 3 systems generate Rust via format!() | All Rust emission through `RustCodeRenderer<M>` |
| Markdown built with push_str() | Markdown through `MarkupRenderer<M>` — reusable for docs, PRs, prompts |
| LLM prompts are ad-hoc string joins | Prompts are structured `MarkupNode` documents |
| Hand-edited generated files drift silently | Emit manifest + CI verification catches drift |
| Rendering happens outside DAGs | Rendering pipelines are DAGs with skip-if-unchanged + diff reporting |
| Adding HTML dashboard requires new system | Implement `HtmlText: TextMedium`, all domain layers get HTML for free |
| Visual AST diagram = entirely new system | `RustCodeRenderer<SvgGraphics>` = visual AST, same domain logic |
| Adding a new output format touches every renderer | Implement one `OutputMedium`, all domain renderers compose it |
| Adding a new generator requires manual Make/CI wiring | Register `ArtifactDef` once, Make/CI targets auto-derive |
| CI YAML flattened to strings early | `DataValue` DOM preserves structure; YAML/TOML/JSON renderers share it |

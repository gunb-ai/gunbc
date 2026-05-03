//! PB-Bootstrap-Process Gap 4 build-step producer.
//!
//! Implements **R3 row 85 / PB #1560 Gap 4** at the build-step level per the
//! decision in `docs/decisions/r3-row85-method-template-read-surface.md` and
//! the dispatch unparking the architecture once
//! [`PR #1575`](https://github.com/gunb-ai/gunbc/pull/1575) landed the
//! ephemeral source-root mechanism in v2.
//!
//! ## What this is
//!
//! A producer that takes the canonical `MethodTemplateContract` row authority
//! (from `src/v3/std/{rust,python,go}_method_template_contracts.dag` projected
//! via [`crate::pb_method_template_projection`]) and writes a build-time-
//! ephemeral `.dag` file at `<out_dir>/generated/method_template_projection.dag`
//! that v2 can consume via the source-root mechanism (`--source-root <out_dir>`,
//! `compile_dag_named_with_source_roots`, `run_self_compile_with_extra_source_roots`).
//!
//! ## What this is not
//!
//! - **Not a committed `.dag`**: the producer writes into a caller-supplied
//!   directory which must be ephemeral (temp dir / `OUT_DIR`). The repository
//!   never tracks the generated file.
//! - **Not a `v3.std.*` import bridge**: the produced `.dag` declares its own
//!   module (`generated.method_template_projection`) and uses only kernel
//!   shapes (`Map<String, String>`). v2 imports the generated module directly,
//!   not the v3 row authorities.
//! - **Not a hand-authored second authority**: the producer's only template-
//!   text source is the typed `MethodTemplateContract` rows. There is no
//!   parallel `Map<String, String>` authority anywhere in `src/v2/` or `dsl/`.
//! - **Not Gap 5**: the producer emits the legacy-shaped `Map<String, String>`
//!   adapter so leaf-emit consumers can flip imports without a substrate
//!   rewrite. Gap 5 (`LanguageSpec.method_templates` structural change) is
//!   sequenced after Grounding's leaf-migration consumes this surface.
//! - **Not higher-order rows**: only `MethodEmitTemplate::Single` rows land in
//!   the generated maps. Higher-order rows live structurally in the
//!   `MethodTemplateContract` substrate; serving them through a Map would
//!   reintroduce the legacy parallel-table pattern. Higher-order migration is
//!   a Gap 5 follow-up.
//!
//! ## Authority chain
//!
//! 1. `src/v3/std/{rust,python,go}_method_template_contracts.dag` — canonical
//!    typed `MethodTemplateContract` rows.
//! 2. `crate::pb_method_template_projection::method_template_contract_rows` —
//!    typed projection over the bootstrap `Dag`. Validates Stratum-A-grade
//!    `MethodDeclaration` data bindings, closed-record schema, sum identity.
//! 3. **This module** — adapts the typed projection to a v2-importable
//!    `Map<String, String>` shape and writes it to an ephemeral `.dag`.
//! 4. v2 consumes via `--source-root <ephemeral>` per #1575.
//!
//! Drift between the typed substrate and the generated map is structurally
//! impossible: both share `method_template_contract_rows` as the single
//! source of row text.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::dag::Dag;
use crate::pb_method_template_projection::{
    method_declaration_carrier_id, method_template_contract_rows,
    validate_method_declaration_data_binding, MethodEmitTemplateProjection,
    MethodTemplateProjectionError, MethodTemplateTarget,
};

/// File path, relative to `out_dir`, where the producer writes the generated
/// projection. Stable so v2 callers can cite it directly when configuring
/// `--source-root`.
pub const GENERATED_PROJECTION_RELATIVE_PATH: &str = "generated/method_template_projection.dag";

/// Module declaration name in the generated `.dag` file. v2 callers import
/// from this name (e.g.
/// `import generated.method_template_projection { rust_method_template_emit }`).
pub const GENERATED_MODULE_NAME: &str = "generated.method_template_projection";

/// Per-target name of the `Map<String, String>` data declaration emitted into
/// the generated `.dag`. Distinct from legacy
/// `extdeps.languages.{rust,python,go}.emit::*_method_templates` so the
/// generated module does not collide with the legacy hand-authored authority.
pub fn generated_map_declaration_name(target: MethodTemplateTarget) -> &'static str {
    match target {
        MethodTemplateTarget::Rust => "rust_method_template_emit",
        MethodTemplateTarget::Python => "python_method_template_emit",
        MethodTemplateTarget::Go => "go_method_template_emit",
    }
}

/// Fail-closed producer error. `MethodTemplateProjectionError` is wrapped so
/// any structural violation in the typed projection chain (closed schema,
/// Stratum-A-grade `MethodDeclaration` bindings, payload shape, per-target
/// uniqueness) surfaces here as a typed value (per `INVARIANTS.md` C-8).
#[derive(Debug)]
pub enum MethodTemplateProjectionDagEmitError {
    /// Underlying typed projection failed for `target`.
    Projection {
        target: MethodTemplateTarget,
        cause: MethodTemplateProjectionError,
    },
    /// Resolving the `MethodDeclaration` carrier failed (substrate degenerate
    /// case; should be impossible if `regen_bootstrap --verify` is green).
    MethodDeclarationCarrier(MethodTemplateProjectionError),
    /// Recovering the method-name `String` from a row's `MethodRef.decl`
    /// failed Stratum A's data-binding contract. This should be unreachable
    /// because [`method_template_contract_rows`] already validated the
    /// binding — but the carrier surfaces violations rather than panicking.
    MethodNameRecoveryFailed {
        target: MethodTemplateTarget,
        row_index: usize,
        cause: crate::pb_method_template_projection::MethodDeclarationBindingViolation,
    },
    /// Filesystem I/O failure (mkdir / write).
    Io {
        path: PathBuf,
        cause: std::io::Error,
    },
}

impl std::fmt::Display for MethodTemplateProjectionDagEmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MethodTemplateProjectionDagEmitError {}

/// Targets covered by the generated projection. The set mirrors the closed
/// per-target row-list authorities in `src/v3/std/`; new targets land by
/// adding a row-list authority + extending [`MethodTemplateTarget`].
const TARGETS: &[MethodTemplateTarget] = &[
    MethodTemplateTarget::Rust,
    MethodTemplateTarget::Python,
    MethodTemplateTarget::Go,
];

/// Write the generated `.dag` file to `<out_dir>/<GENERATED_PROJECTION_RELATIVE_PATH>`.
/// Caller is responsible for `out_dir` being ephemeral (temp dir / `OUT_DIR`)
/// and for cleaning it up.
///
/// Returns the absolute path written on success. On any structural mismatch
/// in the typed projection chain, returns a typed
/// [`MethodTemplateProjectionDagEmitError`] without writing a partial file.
pub fn write_method_template_projection_dag(
    dag: &Dag,
    out_dir: &Path,
) -> Result<PathBuf, MethodTemplateProjectionDagEmitError> {
    // Resolve the MethodDeclaration carrier once; reused for every row.
    let method_declaration_id = method_declaration_carrier_id(dag)
        .map_err(MethodTemplateProjectionDagEmitError::MethodDeclarationCarrier)?;

    // Build the per-target adapter map in source-stable order. BTreeMap
    // produces deterministic .dag bytes regardless of how the substrate
    // happens to iterate; deterministic output is necessary for
    // build-pipeline reproducibility.
    let mut per_target: Vec<(MethodTemplateTarget, BTreeMap<String, String>)> =
        Vec::with_capacity(TARGETS.len());
    for target in TARGETS.iter().copied() {
        let rows = method_template_contract_rows(dag, target)
            .map_err(|cause| MethodTemplateProjectionDagEmitError::Projection { target, cause })?;
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for (row_index, row) in rows.iter().enumerate() {
            // Higher-order rows are deliberately skipped — the legacy
            // Map<String, String> shape doesn't carry the inline/fn-ref
            // distinction. Migrating higher-order rows is a Gap 5 follow-up.
            let MethodEmitTemplateProjection::Single { template } = &row.emit_template else {
                continue;
            };
            let name = validate_method_declaration_data_binding(
                dag,
                row.dag_method,
                method_declaration_id,
            )
            .map_err(|cause| {
                MethodTemplateProjectionDagEmitError::MethodNameRecoveryFailed {
                    target,
                    row_index,
                    cause,
                }
            })?;
            // Per-target uniqueness by `dag_method` was already validated by
            // `method_template_contract_rows`; the same `name` cannot appear
            // twice within one target's map.
            map.insert(name, template.clone());
        }
        per_target.push((target, map));
    }

    let content = render_dag(&per_target);

    let target_path = out_dir.join(GENERATED_PROJECTION_RELATIVE_PATH);
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|cause| {
            MethodTemplateProjectionDagEmitError::Io {
                path: parent.to_path_buf(),
                cause,
            }
        })?;
    }
    std::fs::write(&target_path, content).map_err(|cause| {
        MethodTemplateProjectionDagEmitError::Io {
            path: target_path.clone(),
            cause,
        }
    })?;
    Ok(target_path)
}

/// Render the `.dag` source bytes for the populated per-target maps.
///
/// Output format is stable for build-pipeline reproducibility:
///
/// ```text
/// // AUTO-GENERATED — do not commit. Regenerate via …
/// // …
/// module generated.method_template_projection
///
/// data rust_method_template_emit: Map<String, String> = {
///   "all": "({recv}.iter().all(…))",
///   "count": "({recv}.len() as i64)",
///   …
/// }
///
/// data python_method_template_emit: Map<String, String> = { … }
/// data go_method_template_emit: Map<String, String> = { … }
/// ```
fn render_dag(per_target: &[(MethodTemplateTarget, BTreeMap<String, String>)]) -> String {
    let mut s = String::new();
    s.push_str(
        "// AUTO-GENERATED by `pb_method_template_projection_dag_emit::write_method_template_projection_dag`.\n\
         // Do not commit; this file lives in a build-time-ephemeral dependency root\n\
         // (temp dir / OUT_DIR) per R3 row 85 / PB #1560 Gap 4.\n\
         //\n\
         // Single authority for row text:\n\
         //   `src/v3/std/{rust,python,go}_method_template_contracts.dag`\n\
         //\n\
         // Only `MethodEmitTemplate::Single` rows are projected here. Higher-\n\
         // order rows (inline / fn-ref split) live structurally in the typed\n\
         // `MethodTemplateContract` carrier and migrate via Gap 5.\n",
    );
    s.push('\n');
    s.push_str("module ");
    s.push_str(GENERATED_MODULE_NAME);
    s.push('\n');
    for (target, map) in per_target {
        s.push('\n');
        s.push_str("data ");
        s.push_str(generated_map_declaration_name(*target));
        s.push_str(": Map<String, String> = {\n");
        for (name, template) in map {
            s.push_str("  \"");
            s.push_str(&escape_dag_string(name));
            s.push_str("\": \"");
            s.push_str(&escape_dag_string(template));
            s.push_str("\",\n");
        }
        s.push_str("}\n");
    }
    s
}

/// Escape a `String` for placement inside a v2 `.dag` string literal.
///
/// v2's grammar escapes `\\` for backslash, `\"` for double quote, and —
/// crucially for template-text projection — `\{` and `\}` for brace pairs
/// (the legacy authorities at `dsl/extdeps/languages/{python,go}/emit.dag`
/// store templates as e.g. `"len(\{recv\})"`). Without the brace escapes
/// v2 treats `{recv}` as variable interpolation and emits "undefined
/// variable" diagnostics.
fn escape_dag_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            other => out.push(other),
        }
    }
    out
}

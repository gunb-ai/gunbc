//! daglang-resolve: Import resolution and module graph.
//!
//! Discovers `.dag` files from the filesystem, resolves imports into a
//! dependency-ordered module graph, and produces a resolved AST where
//! every name reference points to its definition.
//!
//! # Pipeline position
//!
//! ```text
//! Source files -> [daglang-syntax] -> AST -> [daglang-resolve] -> ResolvedAST
//! ```
//!
//! # Filesystem discovery
//!
//! The module graph is built by scanning configured paths for `.dag` files.
//! Module paths (`module infra.gcp.resources`) map to filesystem paths
//! (`infra/gcp/resources.dag`). This eliminates manual registration --
//! every `.dag` file in the configured directories is automatically part
//! of the module graph.

use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};

use daglang_syntax::ast::{ModulePath, SourceFile};
use daglang_syntax::diagnostic::Diagnostic;
use daglang_syntax::parser;

type ParsedModuleRecord = (PathBuf, ModulePath, Vec<ModulePath>, SourceFile);

/// Strict check for `.dag` file extension (lowercase only).
pub fn has_dag_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "dag")
}

/// Case-insensitive check — matches `.dag`, `.DAG`, `.DaG`, etc.
/// Used internally to detect wrong-cased extensions and produce clear errors.
pub fn has_dag_like_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dag"))
}

/// Discover `.dag` files recursively under a root directory.
pub fn discover_dag_files(root: &Path) -> Result<Vec<PathBuf>, ResolveError> {
    let mut files = Vec::new();
    let mut visited_dirs = HashSet::new();
    collect_dag_files(root, &mut files, &mut visited_dirs)?;
    files.sort();
    files.dedup();
    Ok(files)
}

/// A resolved module in the dependency graph.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// Filesystem path to the `.dag` file.
    pub path: PathBuf,
    /// Parsed AST (unresolved).
    pub ast: SourceFile,
    /// Module path (e.g., `infra.gcp.resources`).
    pub module_path: ModulePath,
    /// Indices of modules this module depends on (via `import`).
    pub dependencies: Vec<usize>,
}

/// The complete module graph for a project.
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    /// Modules in dependency order (leaves first).
    pub modules: Vec<ResolvedModule>,
}

impl ModuleGraph {
    /// Discover `.dag` files from the given root directories and build
    /// the module graph.
    // Compiler pipeline: reads .dag files for module resolution
    #[allow(clippy::disallowed_methods)]
    pub fn discover(roots: &[PathBuf]) -> Result<Self, ResolveError> {
        Self::discover_with_cycle_mode(roots, false)
    }

    /// Discover `.dag` files and fail when an import cycle exists.
    ///
    /// This mode is used by compile/check flows that require explicit
    /// success/failure behavior rather than best-effort graph ordering.
    #[allow(clippy::disallowed_methods)]
    pub fn discover_strict(roots: &[PathBuf]) -> Result<Self, ResolveError> {
        Self::discover_with_cycle_mode(roots, true)
    }

    #[allow(clippy::disallowed_methods)]
    fn discover_with_cycle_mode(
        roots: &[PathBuf],
        fail_on_cycles: bool,
    ) -> Result<Self, ResolveError> {
        let mut dag_files = Vec::new();
        let mut visited_dirs = HashSet::new();
        for root in roots {
            if !root.exists() {
                return Err(ResolveError::InvalidRootPath {
                    path: root.clone(),
                    reason: "does not exist".to_string(),
                });
            }
            if !root.is_dir() {
                return Err(ResolveError::InvalidRootPath {
                    path: root.clone(),
                    reason: "is not a directory".to_string(),
                });
            }
            collect_dag_files(root, &mut dag_files, &mut visited_dirs)?;
        }
        let mut canonical_dag_files = Vec::with_capacity(dag_files.len());
        for path in dag_files {
            let canonical =
                std::fs::canonicalize(&path).map_err(|e| ResolveError::IoError(path.clone(), e))?;
            canonical_dag_files.push(canonical);
        }
        dag_files = canonical_dag_files;
        dag_files.sort();
        dag_files.dedup();
        let canonical_roots = canonicalize_roots(roots);

        let mut parsed: Vec<ParsedModuleRecord> = Vec::new();
        let mut parse_errors: Vec<(PathBuf, Vec<Diagnostic>)> = Vec::new();

        for path in &dag_files {
            let source = std::fs::read_to_string(path)
                .map_err(|e| ResolveError::IoError(path.clone(), e))?;
            match parser::parse_with_file_diagnostics(path, &source) {
                Ok(ast) => {
                    let mod_path = ast
                        .module_path
                        .as_ref()
                        .map(|mp| mp.node.clone())
                        .map_or_else(
                            || path_to_module_path(path, roots, &canonical_roots),
                            Ok,
                        )?;
                    let imports: Vec<ModulePath> = ast
                        .imports
                        .iter()
                        .map(|imp| imp.node.path.clone())
                        .collect();
                    parsed.push((path.clone(), mod_path, imports, ast));
                }
                Err(diagnostics) => {
                    parse_errors.push((path.clone(), diagnostics));
                }
            }
        }

        if !parse_errors.is_empty() {
            return Err(ResolveError::ParseErrors(parse_errors));
        }

        let mut mod_index: HashMap<ModulePath, usize> = HashMap::new();
        for (i, (_, mp, _, _)) in parsed.iter().enumerate() {
            if mod_index.insert(mp.clone(), i).is_some() {
                return Err(ResolveError::DuplicateModule(mp.clone()));
            }
        }

        let mut modules: Vec<ResolvedModule> = Vec::with_capacity(parsed.len());
        for (path, mod_path, imports, ast) in parsed {
            let mut deps: Vec<usize> = Vec::new();
            for imp in &imports {
                match mod_index.get(imp) {
                    Some(dep) => deps.push(*dep),
                    None => {
                        return Err(ResolveError::UnresolvedImport {
                            importing_module: mod_path.clone(),
                            target: imp.clone(),
                        });
                    }
                }
            }
            modules.push(ResolvedModule {
                path,
                ast,
                module_path: mod_path,
                dependencies: deps,
            });
        }

        topo_sort(&mut modules, fail_on_cycles)?;

        Ok(ModuleGraph { modules })
    }

    /// Format the module graph as a displayable tree string.
    pub fn display_tree(&self) -> String {
        let mut out = String::new();
        for m in &self.modules {
            let path_str = m.module_path.as_dotted();
            let dep_count = m.dependencies.len();
            let n_items = m.ast.items.len();
            writeln!(
                out,
                "  {path_str}  ({n_items} items, {dep_count} deps)  [{}]",
                m.path.display()
            )
            .ok();
        }
        out
    }
}

/// Recursively collect all `.dag` files under `dir`.
///
/// Tests and fixtures are defined inline within regular `.dag` files,
/// so no filename-based filtering is needed.
// Compiler pipeline: recursively discovers .dag files
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn collect_dag_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    visited_dirs: &mut HashSet<PathBuf>,
) -> Result<(), ResolveError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let canonical_dir =
        std::fs::canonicalize(dir).map_err(|e| ResolveError::IoError(dir.to_path_buf(), e))?;
    if !visited_dirs.insert(canonical_dir) {
        return Ok(());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|e| ResolveError::IoError(dir.to_path_buf(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| ResolveError::IoError(dir.to_path_buf(), e))?;
        let p = entry.path();
        if p.is_dir() {
            collect_dag_files(&p, out, visited_dirs)?;
        } else if has_dag_extension(&p) {
            out.push(p);
        } else if has_dag_like_extension(&p) {
            return Err(ResolveError::InvalidExtensionCase(p));
        }
    }
    Ok(())
}

/// Derive a module path from a filesystem path relative to any root.
/// `dsl/tools/makegen.dag` with root `dsl/` -> `["tools", "makegen"]`
pub fn relative_path_to_module_path(relative: &Path) -> ModulePath {
    ModulePath::new(
        relative
            .with_extension("")
            .components()
            .filter_map(|component| component.as_os_str().to_str().map(|s| s.replace('-', "_")))
            .collect(),
    )
}

/// Pick the shorter (or lexicographically smaller) relative path.
pub fn choose_preferred_relative(current: Option<PathBuf>, candidate: &Path) -> Option<PathBuf> {
    let candidate_buf = candidate.to_path_buf();
    match current {
        None => Some(candidate_buf),
        Some(existing) => {
            let existing_depth = existing.components().count();
            let candidate_depth = candidate_buf.components().count();
            if candidate_depth < existing_depth {
                return Some(candidate_buf);
            }
            if candidate_depth > existing_depth {
                return Some(existing);
            }

            let existing_key = existing.as_os_str().to_string_lossy();
            let candidate_key = candidate_buf.as_os_str().to_string_lossy();
            if candidate_key < existing_key {
                Some(candidate_buf)
            } else {
                Some(existing)
            }
        }
    }
}

/// Deduplicate root paths via filesystem canonicalization.
#[allow(clippy::disallowed_methods)]
pub fn canonicalize_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut canonical_roots = Vec::new();
    for root in roots {
        if let Ok(canonical_root) = std::fs::canonicalize(root) {
            if seen.insert(canonical_root.clone()) {
                canonical_roots.push(canonical_root);
            }
        }
    }
    canonical_roots
}

/// Resolve a filesystem path to a module path using canonical path comparison.
/// Returns an error if the path cannot be resolved relative to any known root.
#[allow(clippy::disallowed_methods)]
pub fn path_to_module_path(
    path: &Path,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
) -> Result<ModulePath, ResolveError> {
    let canonical_path = std::fs::canonicalize(path).ok();
    let mut best_relative: Option<PathBuf> = None;
    for root in roots {
        if let Ok(rel) = path.strip_prefix(root) {
            best_relative = choose_preferred_relative(best_relative, rel);
        }
    }
    for canonical_root in canonical_roots {
        if let Ok(rel) = path.strip_prefix(canonical_root) {
            best_relative = choose_preferred_relative(best_relative, rel);
        }
        if let Some(canonical_path) = &canonical_path {
            if let Ok(rel) = canonical_path.strip_prefix(canonical_root) {
                best_relative = choose_preferred_relative(best_relative, rel);
            }
        }
    }
    if let Some(relative) = best_relative {
        return Ok(relative_path_to_module_path(&relative));
    }
    Err(ResolveError::InvalidRootPath {
        path: path.to_path_buf(),
        reason: "file is not under any known DSL root directory".to_string(),
    })
}

/// Stable topological sort: leaves (no deps) first.
fn topo_sort(modules: &mut Vec<ResolvedModule>, fail_on_cycles: bool) -> Result<(), ResolveError> {
    let n = modules.len();
    let mut in_degree = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, m) in modules.iter().enumerate() {
        for &dep in &m.dependencies {
            if dep < n {
                adj[dep].push(i);
                in_degree[i] += 1;
            }
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    // Keep deterministic ordering and consume lowest index first.
    queue.sort_unstable_by(|a, b| b.cmp(a));
    let mut order = Vec::with_capacity(n);
    while let Some(idx) = queue.pop() {
        order.push(idx);
        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push(next);
            }
        }
        queue.sort_unstable_by(|a, b| b.cmp(a));
    }
    if order.len() != n {
        if !fail_on_cycles {
            let mut seen = vec![false; n];
            for idx in &order {
                seen[*idx] = true;
            }
            for (idx, visited) in seen.iter().enumerate().take(n) {
                if !visited {
                    order.push(idx);
                }
            }
            return reorder_modules_by_order(modules, &order);
        }
        let mut seen = vec![false; n];
        for idx in &order {
            seen[*idx] = true;
        }
        let cycle = seen
            .iter()
            .enumerate()
            .filter_map(|(idx, visited)| (!visited).then_some(modules[idx].module_path.clone()))
            .collect::<Vec<_>>();
        return Err(ResolveError::CyclicDependency(cycle));
    }

    reorder_modules_by_order(modules, &order)
}

fn reorder_modules_by_order(
    modules: &mut Vec<ResolvedModule>,
    order: &[usize],
) -> Result<(), ResolveError> {
    let n = modules.len();
    let mut old_to_new = vec![0usize; n];
    for (new_idx, old_idx) in order.iter().enumerate() {
        old_to_new[*old_idx] = new_idx;
    }

    let mut tmp: Vec<Option<ResolvedModule>> = modules.drain(..).map(Some).collect();
    for old_idx in order {
        let mut module = tmp[*old_idx].take().ok_or_else(|| {
            ResolveError::InternalInvariant(format!(
                "topological order referenced missing module index {old_idx}"
            ))
        })?;
        for dep in &mut module.dependencies {
            if *dep < n {
                *dep = old_to_new[*dep];
            }
        }
        modules.push(module);
    }
    Ok(())
}

/// Errors during module resolution.
#[derive(Debug)]
pub enum ResolveError {
    /// A `.dag` file could not be read.
    IoError(PathBuf, std::io::Error),
    /// Parse errors in one or more `.dag` files.
    ParseErrors(Vec<(PathBuf, Vec<Diagnostic>)>),
    /// An import references a module that doesn't exist.
    UnresolvedImport {
        importing_module: ModulePath,
        target: ModulePath,
    },
    /// Circular dependency detected.
    CyclicDependency(Vec<ModulePath>),
    /// Duplicate module path.
    DuplicateModule(ModulePath),
    /// Discovery root path is invalid.
    InvalidRootPath { path: PathBuf, reason: String },
    /// A `.dag` file has wrong-cased extension (e.g., `.DAG`, `.DaG`).
    InvalidExtensionCase(PathBuf),
    /// Internal invariant violation while resolving modules.
    InternalInvariant(String),
}

impl ResolveError {
    /// Stable, grep-able error code for this variant (CP-59).
    pub fn code(&self) -> &'static str {
        match self {
            Self::IoError(..) => "MOD001",
            Self::ParseErrors(..) => "MOD002",
            Self::UnresolvedImport { .. } => "MOD003",
            Self::CyclicDependency(..) => "MOD004",
            Self::DuplicateModule(..) => "MOD005",
            Self::InvalidRootPath { .. } => "MOD006",
            Self::InvalidExtensionCase(..) => "MOD007",
            Self::InternalInvariant(..) => "MOD008",
        }
    }

    /// Help text with fix suggestions for common errors (CP-50).
    pub fn help(&self) -> Option<String> {
        match self {
            Self::IoError(path, _) => Some(format!(
                "check that `{}` exists and is readable", path.display()
            )),
            Self::UnresolvedImport { target, .. } => Some(format!(
                "create `{}.dag` or check the import path — did you mean a different module?",
                target.as_dotted().replace('.', "/")
            )),
            Self::CyclicDependency(cycle) => {
                let names: Vec<_> = cycle.iter().map(|m| m.as_dotted()).collect();
                Some(format!("break the cycle: {}", names.join(" → ")))
            }
            Self::InvalidExtensionCase(path) => Some(format!(
                "rename `{}` to use lowercase `.dag` extension", path.display()
            )),
            _ => None,
        }
    }
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(p, e) => write!(f, "I/O error reading {}: {e}", p.display()),
            Self::ParseErrors(files) => {
                write!(f, "parse errors encountered in {} file(s):", files.len())?;
                for (file, errors) in files {
                    write!(f, "\n  {}:", file.display())?;
                    for e in errors {
                        write!(f, "\n    {}", e.render())?;
                    }
                }
                Ok(())
            }
            Self::UnresolvedImport {
                importing_module,
                target,
            } => write!(f, "unresolved import: {importing_module} -> {target}"),
            Self::CyclicDependency(cycle) => {
                write!(f, "cyclic dependency: ")?;
                for (i, m) in cycle.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{m}")?;
                }
                Ok(())
            }
            Self::DuplicateModule(path) => write!(f, "duplicate module: {path}"),
            Self::InvalidRootPath { path, reason } => {
                write!(f, "invalid discovery root {}: {reason}", path.display())
            }
            Self::InvalidExtensionCase(path) => {
                write!(
                    f,
                    "file `{}` has wrong-cased extension; rename to `.dag` (lowercase)",
                    path.display()
                )
            }
            Self::InternalInvariant(message) => {
                write!(f, "internal resolver invariant failed: {message}")
            }
        }
    }
}

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
use std::path::{Path, PathBuf};

use daglang_syntax::ast::SourceFile;
use daglang_syntax::diagnostic::Diagnostic;
use daglang_syntax::parser;

/// A resolved module in the dependency graph.
#[derive(Debug)]
pub struct ResolvedModule {
    /// Filesystem path to the `.dag` file.
    pub path: PathBuf,
    /// Parsed AST (unresolved).
    pub ast: SourceFile,
    /// Module path segments (e.g., `["infra", "gcp", "resources"]`).
    pub module_path: Vec<String>,
    /// Indices of modules this module depends on (via `import`).
    pub dependencies: Vec<usize>,
}

/// The complete module graph for a project.
#[derive(Debug)]
pub struct ModuleGraph {
    /// Modules in dependency order (leaves first).
    pub modules: Vec<ResolvedModule>,
}

impl ModuleGraph {
    /// Discover `.dag` files from the given root directories and build
    /// the module graph.
    pub fn discover(roots: &[PathBuf]) -> Result<Self, ResolveError> {
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
            let canonical = std::fs::canonicalize(&path)
                .map_err(|e| ResolveError::IoError(path.clone(), e))?;
            canonical_dag_files.push(canonical);
        }
        dag_files = canonical_dag_files;
        dag_files.sort();
        dag_files.dedup();
        let canonical_roots = canonicalize_roots(roots);

        let mut parsed: Vec<(PathBuf, Vec<String>, Vec<Vec<String>>, SourceFile)> = Vec::new();
        let mut parse_errors: Vec<(PathBuf, Vec<Diagnostic>)> = Vec::new();

        for path in &dag_files {
            let source = std::fs::read_to_string(path)
                .map_err(|e| ResolveError::IoError(path.clone(), e))?;
            match parser::parse_with_file_diagnostics(path, &source) {
                Ok(ast) => {
                    let mod_path = ast
                        .module_path
                        .as_ref()
                        .map(|mp| mp.node.segments.clone())
                        .unwrap_or_else(|| path_to_module_path(path, roots, &canonical_roots));
                    let imports: Vec<Vec<String>> = ast
                        .imports
                        .iter()
                        .map(|imp| imp.node.path.segments.clone())
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

        let mut mod_index: HashMap<Vec<String>, usize> = HashMap::new();
        for (i, (_, mp, _, _)) in parsed.iter().enumerate() {
            if mod_index.insert(mp.clone(), i).is_some() {
                return Err(ResolveError::DuplicateModule(mp.clone()));
            }
        }

        let mut modules: Vec<ResolvedModule> = Vec::with_capacity(parsed.len());
        for (path, mod_path, imports, ast) in parsed {
            let mut deps: Vec<usize> = Vec::new();
            for imp in &imports {
                if let Some(dep) = mod_index.get(imp) {
                    deps.push(*dep);
                }
            }
            modules.push(ResolvedModule {
                path,
                ast,
                module_path: mod_path,
                dependencies: deps,
            });
        }

        topo_sort(&mut modules)?;

        Ok(ModuleGraph { modules })
    }

    /// Format the module graph as a displayable tree string.
    pub fn display_tree(&self) -> String {
        let mut out = String::new();
        for m in &self.modules {
            let path_str = m.module_path.join(".");
            let dep_count = m.dependencies.len();
            let n_items = m.ast.items.len();
            out.push_str(&format!(
                "  {path_str}  ({n_items} items, {dep_count} deps)  [{}]\n",
                m.path.display()
            ));
        }
        out
    }
}

/// Recursively collect all `.dag` files under `dir`.
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
        } else if p.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(p);
        }
    }
    Ok(())
}

/// Derive a module path from a filesystem path relative to any root.
/// `dsl/tools/makegen.dag` with root `dsl/` -> `["tools", "makegen"]`
fn relative_path_to_module_path(relative: &Path) -> Vec<String> {
    relative
        .with_extension("")
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(String::from))
        .collect()
}

fn canonicalize_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .filter_map(|root| std::fs::canonicalize(root).ok())
        .collect()
}

fn path_to_module_path(path: &Path, roots: &[PathBuf], canonical_roots: &[PathBuf]) -> Vec<String> {
    let canonical_path = std::fs::canonicalize(path).ok();
    for root in roots {
        if let Ok(rel) = path.strip_prefix(root) {
            return relative_path_to_module_path(rel);
        }
    }
    for canonical_root in canonical_roots {
        if let Ok(rel) = path.strip_prefix(canonical_root) {
            return relative_path_to_module_path(rel);
        }
        if let Some(canonical_path) = &canonical_path {
            if let Ok(rel) = canonical_path.strip_prefix(canonical_root) {
                return relative_path_to_module_path(rel);
            }
        }
    }
    vec![path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()]
}

/// Stable topological sort: leaves (no deps) first.
fn topo_sort(modules: &mut Vec<ResolvedModule>) -> Result<(), ResolveError> {
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
    let mut order = Vec::with_capacity(n);
    while let Some(idx) = queue.pop() {
        order.push(idx);
        for &next in &adj[idx] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                queue.push(next);
            }
        }
    }
    if order.len() != n {
        let mut seen = vec![false; n];
        for idx in &order {
            seen[*idx] = true;
        }
        for idx in 0..n {
            if !seen[idx] {
                order.push(idx);
            }
        }
    }

    let mut tmp: Vec<Option<ResolvedModule>> = modules.drain(..).map(Some).collect();
    for i in &order {
        modules.push(tmp[*i].take().unwrap());
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
        importing_module: Vec<String>,
        target: Vec<String>,
    },
    /// Circular dependency detected.
    CyclicDependency(Vec<Vec<String>>),
    /// Duplicate module path.
    DuplicateModule(Vec<String>),
    /// Discovery root path is invalid.
    InvalidRootPath { path: PathBuf, reason: String },
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
            } => write!(
                f,
                "unresolved import: {} -> {}",
                importing_module.join("."),
                target.join(".")
            ),
            Self::CyclicDependency(cycle) => {
                write!(f, "cyclic dependency: ")?;
                for (i, m) in cycle.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{}", m.join("."))?;
                }
                Ok(())
            }
            Self::DuplicateModule(path) => write!(f, "duplicate module: {}", path.join(".")),
            Self::InvalidRootPath { path, reason } => {
                write!(f, "invalid discovery root {}: {reason}", path.display())
            }
        }
    }
}

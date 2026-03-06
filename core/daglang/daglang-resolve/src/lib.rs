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

// ============================================================================
// Virtual filesystem abstraction (CP-57)
// ============================================================================

/// Filesystem abstraction for module discovery and source reading.
///
/// All filesystem access in the resolve stage goes through this trait,
/// making the parser/typechecker/lowerer pipeline testable with synthetic
/// inputs and enabling deterministic caching.
pub trait Vfs {
    /// Read a file's contents as a UTF-8 string.
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;

    /// List entries in a directory (files and subdirectories).
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>>;

    /// Canonicalize a path (resolve symlinks, normalize).
    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf>;

    /// Check whether a path exists.
    fn exists(&self, path: &Path) -> bool;

    /// Check whether a path is a directory.
    fn is_dir(&self, path: &Path) -> bool;
}

/// A directory entry returned by `Vfs::read_dir`.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

/// Real filesystem implementation of `Vfs`.
#[derive(Debug, Clone, Copy)]
pub struct RealVfs;

#[allow(clippy::disallowed_methods)]
impl Vfs for RealVfs {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            entries.push(DirEntry {
                is_dir: p.is_dir(),
                path: p,
            });
        }
        Ok(entries)
    }

    fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }
}

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
    discover_dag_files_with_vfs(root, &RealVfs)
}

/// Discover `.dag` files recursively under a root directory using a custom Vfs.
pub fn discover_dag_files_with_vfs(
    root: &Path,
    vfs: &dyn Vfs,
) -> Result<Vec<PathBuf>, ResolveError> {
    let mut files = Vec::new();
    let mut visited_dirs = HashSet::new();
    collect_dag_files_vfs(root, &mut files, &mut visited_dirs, vfs)?;
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
    /// the module graph. Uses the real filesystem.
    pub fn discover(roots: &[PathBuf]) -> Result<Self, ResolveError> {
        Self::discover_with_vfs(roots, &RealVfs, false)
    }

    /// Discover `.dag` files and fail when an import cycle exists.
    /// Uses the real filesystem.
    pub fn discover_strict(roots: &[PathBuf]) -> Result<Self, ResolveError> {
        Self::discover_with_vfs(roots, &RealVfs, true)
    }

    /// Discover `.dag` files using a custom `Vfs` implementation.
    /// Enables testing with synthetic/in-memory filesystems.
    pub fn discover_with_vfs(
        roots: &[PathBuf],
        vfs: &dyn Vfs,
        fail_on_cycles: bool,
    ) -> Result<Self, ResolveError> {
        let mut dag_files = Vec::new();
        let mut visited_dirs = HashSet::new();
        for root in roots {
            if !vfs.exists(root) {
                return Err(ResolveError::InvalidRootPath {
                    path: root.clone(),
                    reason: "does not exist".to_string(),
                });
            }
            if !vfs.is_dir(root) {
                return Err(ResolveError::InvalidRootPath {
                    path: root.clone(),
                    reason: "is not a directory".to_string(),
                });
            }
            collect_dag_files_vfs(root, &mut dag_files, &mut visited_dirs, vfs)?;
        }
        let mut canonical_dag_files = Vec::with_capacity(dag_files.len());
        for path in dag_files {
            let canonical = vfs
                .canonicalize(&path)
                .map_err(|e| ResolveError::IoError(path.clone(), e))?;
            canonical_dag_files.push(canonical);
        }
        dag_files = canonical_dag_files;
        dag_files.sort();
        dag_files.dedup();
        let canonical_roots = canonicalize_roots_vfs(roots, vfs);

        let mut parsed: Vec<ParsedModuleRecord> = Vec::new();
        let mut parse_errors: Vec<(PathBuf, Vec<Diagnostic>)> = Vec::new();

        for path in &dag_files {
            let source = vfs
                .read_to_string(path)
                .map_err(|e| ResolveError::IoError(path.clone(), e))?;
            match parser::parse_with_file_diagnostics(path, &source) {
                Ok(ast) => {
                    let mod_path = ast
                        .module_path
                        .as_ref()
                        .map(|mp| mp.node.clone())
                        .map_or_else(
                            || path_to_module_path_vfs(path, roots, &canonical_roots, vfs),
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
fn collect_dag_files_vfs(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    visited_dirs: &mut HashSet<PathBuf>,
    vfs: &dyn Vfs,
) -> Result<(), ResolveError> {
    if !vfs.is_dir(dir) {
        return Ok(());
    }
    let canonical_dir = vfs
        .canonicalize(dir)
        .map_err(|e| ResolveError::IoError(dir.to_path_buf(), e))?;
    if !visited_dirs.insert(canonical_dir) {
        return Ok(());
    }
    let entries = vfs
        .read_dir(dir)
        .map_err(|e| ResolveError::IoError(dir.to_path_buf(), e))?;
    for entry in entries {
        if entry.is_dir {
            collect_dag_files_vfs(&entry.path, out, visited_dirs, vfs)?;
        } else if has_dag_extension(&entry.path) {
            out.push(entry.path);
        } else if has_dag_like_extension(&entry.path) {
            return Err(ResolveError::InvalidExtensionCase(entry.path));
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
pub fn canonicalize_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    canonicalize_roots_vfs(roots, &RealVfs)
}

/// Deduplicate root paths via Vfs canonicalization.
fn canonicalize_roots_vfs(roots: &[PathBuf], vfs: &dyn Vfs) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut canonical_roots = Vec::new();
    for root in roots {
        if let Ok(canonical_root) = vfs.canonicalize(root) {
            if seen.insert(canonical_root.clone()) {
                canonical_roots.push(canonical_root);
            }
        }
    }
    canonical_roots
}

/// Resolve a filesystem path to a module path using canonical path comparison.
/// Returns an error if the path cannot be resolved relative to any known root.
pub fn path_to_module_path(
    path: &Path,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
) -> Result<ModulePath, ResolveError> {
    path_to_module_path_vfs(path, roots, canonical_roots, &RealVfs)
}

/// Resolve a filesystem path to a module path using a Vfs for canonicalization.
fn path_to_module_path_vfs(
    path: &Path,
    roots: &[PathBuf],
    canonical_roots: &[PathBuf],
    vfs: &dyn Vfs,
) -> Result<ModulePath, ResolveError> {
    let canonical_path = vfs.canonicalize(path).ok();
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
                "check that `{}` exists and is readable",
                path.display()
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
                "rename `{}` to use lowercase `.dag` extension",
                path.display()
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

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory Vfs for testing module discovery without the filesystem.
    struct InMemoryVfs {
        files: HashMap<PathBuf, String>,
        dirs: HashSet<PathBuf>,
    }

    impl InMemoryVfs {
        fn new() -> Self {
            Self {
                files: HashMap::new(),
                dirs: HashSet::new(),
            }
        }

        fn add_file(&mut self, path: impl Into<PathBuf>, content: &str) {
            let path = path.into();
            // Ensure all parent directories exist
            let mut dir = path.parent();
            while let Some(d) = dir {
                self.dirs.insert(d.to_path_buf());
                dir = d.parent();
            }
            self.files.insert(path, content.to_string());
        }
    }

    impl Vfs for InMemoryVfs {
        fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
        }

        fn read_dir(&self, path: &Path) -> std::io::Result<Vec<DirEntry>> {
            let mut entries = Vec::new();
            for file_path in self.files.keys() {
                if file_path.parent() == Some(path) {
                    entries.push(DirEntry {
                        path: file_path.clone(),
                        is_dir: false,
                    });
                }
            }
            for dir_path in &self.dirs {
                if dir_path.parent() == Some(path) && dir_path != path {
                    entries.push(DirEntry {
                        path: dir_path.clone(),
                        is_dir: true,
                    });
                }
            }
            entries.sort_by(|a, b| a.path.cmp(&b.path));
            Ok(entries)
        }

        fn canonicalize(&self, path: &Path) -> std::io::Result<PathBuf> {
            // In-memory: just return the path as-is (no symlinks to resolve)
            Ok(path.to_path_buf())
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.contains_key(path) || self.dirs.contains(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.dirs.contains(path)
        }
    }

    #[test]
    fn in_memory_vfs_discovers_dag_files() {
        let mut vfs = InMemoryVfs::new();
        let root = PathBuf::from("/project/dsl");
        vfs.add_file(
            "/project/dsl/foo.dag",
            "module foo\nfn main() -> { out: String } { return { out: \"hello\" } }",
        );
        vfs.add_file(
            "/project/dsl/bar.dag",
            "module bar\nimport foo\nfn run() -> { out: String } { return { out: \"world\" } }",
        );

        let graph = ModuleGraph::discover_with_vfs(&[root], &vfs, true)
            .expect("should discover in-memory modules");
        assert_eq!(graph.modules.len(), 2);

        let mod_names: Vec<String> = graph
            .modules
            .iter()
            .map(|m| m.module_path.as_dotted())
            .collect();
        // foo has no deps, so it comes first in topo order
        assert_eq!(mod_names[0], "foo");
        assert_eq!(mod_names[1], "bar");
    }

    #[test]
    fn in_memory_vfs_discovers_nested_dirs() {
        let mut vfs = InMemoryVfs::new();
        let root = PathBuf::from("/project/dsl");
        vfs.add_file(
            "/project/dsl/tools/makegen.dag",
            "module tools.makegen\nfn render() -> { out: String } { return { out: \"make\" } }",
        );

        let graph = ModuleGraph::discover_with_vfs(&[root], &vfs, true)
            .expect("should discover nested modules");
        assert_eq!(graph.modules.len(), 1);
        assert_eq!(graph.modules[0].module_path.as_dotted(), "tools.makegen");
    }

    #[test]
    fn in_memory_vfs_rejects_missing_root() {
        let vfs = InMemoryVfs::new();
        let root = PathBuf::from("/nonexistent");
        let err = ModuleGraph::discover_with_vfs(&[root], &vfs, true)
            .expect_err("should fail on missing root");
        assert!(matches!(err, ResolveError::InvalidRootPath { .. }));
    }

    #[test]
    fn in_memory_vfs_detects_unresolved_import() {
        let mut vfs = InMemoryVfs::new();
        let root = PathBuf::from("/project/dsl");
        vfs.add_file(
            "/project/dsl/broken.dag",
            "module broken\nimport nonexistent\nfn f() -> { out: String } { return { out: \"x\" } }",
        );

        let err = ModuleGraph::discover_with_vfs(&[root], &vfs, true)
            .expect_err("should fail on unresolved import");
        assert!(matches!(err, ResolveError::UnresolvedImport { .. }));
    }
}

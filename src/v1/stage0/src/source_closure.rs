//! Parse-then-derive closure assembly: which source files belong in a compile.
//!
//! This replaces the import-driven closure builders (`extract_imports` and its
//! four forked copies) and the pre-parse byte scanners in `cli_run`
//! (`referenced_module_paths_in_text`). It is homed here, outside `cli_run.rs`,
//! deliberately: that file is being deleted wholesale on a sibling lane, and
//! closure assembly must survive that cut.
//!
//! WHY NOT A TEXT SCAN. `referenced_module_paths_in_text` runs BEFORE parsing
//! and longest-matches dotted identifiers against a module-name index, so it
//! cannot distinguish a reference from prose — the in-tree receipt is the
//! English word `edge` in a source annotation binding to `fn edge` in
//! `v2.test.manual.ownership_movable` and pulling that module into an unrelated
//! entry's pool. DESIGN §4: in a closed system a heuristic is never necessary,
//! the richer source always exists. Here the richer source is the parsed tree.
//!
//! WHY NOT IMPORT-DRIVEN. The import statement did two separable jobs: make a
//! name visible (now unique-on-chain's job) and pull a file into the compile
//! (this module's job). Deleting the statement obsoletes only the first.
//!
//! THE CONSTRUCTION. Parsing is per-file and needs no closure, so the fixpoint
//! is well-founded: parse the entry standalone, read the names it actually
//! references out of the `Node` tree, map each through a declaration index to
//! the module that declares it, load those, repeat until nothing new appears.
//!
//! PRECISION, STATED HONESTLY. The reference set is every name occurring in the
//! parsed tree, so a local binder that happens to share a declaration's name
//! pulls that declaration's module. That is a structural OVER-approximation
//! computed as the answer — the model's precision frontier — not an absorbing
//! fallback: it never widens on failure, and it is bounded by names that
//! actually occur in the tree. It is strictly narrower than the text scan it
//! replaces, which could not even exclude prose. Narrowing it further is a
//! resolution-grain question (bind occurrences to declarations, then close over
//! the bound edges only) and belongs with the resolver, not here.
//!
//! MIGRATION. This is seed Rust because a new `.dag` module needs the two-round
//! regen bootstrap and regen is suspended for the duration of the import cut.
//! The shape is deliberately fold-like so it moves to `.dag` with a thin host
//! seam for the filesystem read.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::v1_compiler_compile::SourceFile;
use crate::v1_std_core::{
    byte_to_line_col, diagnostic_to_message, diagnostic_to_span, inferred_to_node, module_items,
    node_field_roles, Connective, ExprData, MatchPattern, Node, NodeFieldRole,
};

/// Maps every declared name to the module declaring it, and every module to its
/// file. Built once over a set of source roots; the expensive half (parsing
/// every candidate file) is paid here so that each individual closure stays
/// small.
pub struct DeclarationIndex {
    /// declared name -> module paths declaring it (plural: a homonym is a real
    /// state, and collapsing it here would silently pick a winner before the
    /// resolver ever sees the ambiguity)
    by_name: HashMap<String, BTreeSet<String>>,
    /// module path -> source file
    modules: HashMap<String, Rc<SourceFile>>,
}

impl DeclarationIndex {
    pub fn modules_declaring(&self, name: &str) -> Option<&BTreeSet<String>> {
        self.by_name.get(name)
    }

    pub fn source_for_module(&self, module_path: &str) -> Option<&Rc<SourceFile>> {
        self.modules.get(module_path)
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn name_count(&self) -> usize {
        self.by_name.len()
    }

    /// How many distinct names are declared by more than one module, and the
    /// worst case. A bare reference to such a name pulls EVERY declarer,
    /// because this layer must not pick a winner, so these are what multiply
    /// closure width.
    pub fn homonym_stats(&self) -> (usize, usize, Vec<(String, usize)>) {
        let mut multi = 0usize;
        let mut worst: Vec<(String, usize)> = Vec::new();
        for (name, mods) in self.by_name.iter() {
            if mods.len() > 1 {
                multi += 1;
                worst.push((name.clone(), mods.len()));
            }
        }
        worst.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        worst.truncate(15);
        (multi, self.by_name.len(), worst)
    }
}

/// Typed, located refusal for the fail-closed regen / multi-entry path.
///
/// The older `dag_files_under` / `build_declaration_index` / `closure_for_entry`
/// surfaces stay for test helpers that already assert `unparsed` empty. Regen
/// must not inherit their silent continues: unreadable dirs, unparsable reached
/// sources, and duplicate module paths are refusals, never skips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosureRefusal {
    MissingRoot {
        path: String,
    },
    ReadDir {
        path: String,
        error: String,
    },
    ReadFile {
        path: String,
        error: String,
    },
    DuplicateModule {
        module_path: String,
        first: String,
        second: String,
    },
    ParseFailed {
        path: String,
        message: String,
    },
    EmptyModulePath {
        path: String,
    },
    DeclaredModuleMissingSource {
        module_path: String,
        referenced_name: String,
    },
    ImportOracle {
        git_ref: String,
        path: String,
        error: String,
    },
}

impl std::fmt::Display for ClosureRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClosureRefusal::MissingRoot { path } => {
                write!(f, "source closure: source root does not exist: {path}")
            }
            ClosureRefusal::ReadDir { path, error } => {
                write!(f, "source closure: read dir {path}: {error}")
            }
            ClosureRefusal::ReadFile { path, error } => {
                write!(f, "source closure: read {path}: {error}")
            }
            ClosureRefusal::DuplicateModule {
                module_path,
                first,
                second,
            } => write!(
                f,
                "source closure: duplicate module path `{module_path}`: {first} and {second}"
            ),
            ClosureRefusal::ParseFailed { path, message } => {
                write!(f, "source closure: parse failed at {path}: {message}")
            }
            ClosureRefusal::EmptyModulePath { path } => {
                write!(f, "source closure: empty module path at {path}")
            }
            ClosureRefusal::DeclaredModuleMissingSource {
                module_path,
                referenced_name,
            } => write!(
                f,
                "source closure: name `{referenced_name}` declared in `{module_path}` but that module has no source row"
            ),
            ClosureRefusal::ImportOracle {
                git_ref,
                path,
                error,
            } => write!(
                f,
                "source closure: import oracle `{git_ref}` failed for {path}: {error}"
            ),
        }
    }
}

/// Every `.dag` file under `roots`, deepest-first stable order.
pub fn dag_files_under(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots {
        if root.exists() {
            collect_dag_files(root, &mut out);
        }
    }
    out.sort();
    out
}

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

/// Fail-closed walk: missing roots, unreadable directories, and IO errors refuse.
pub fn dag_files_under_fail_closed(roots: &[PathBuf]) -> Result<Vec<PathBuf>, ClosureRefusal> {
    let mut out = Vec::new();
    for root in roots {
        if !root.exists() {
            return Err(ClosureRefusal::MissingRoot {
                path: root.display().to_string(),
            });
        }
        collect_dag_files_fail_closed(root, &mut out)?;
    }
    out.sort();
    Ok(out)
}

fn collect_dag_files_fail_closed(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ClosureRefusal> {
    let entries = std::fs::read_dir(dir).map_err(|e| ClosureRefusal::ReadDir {
        path: dir.display().to_string(),
        error: e.to_string(),
    })?;
    let mut collected: Vec<_> =
        entries
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ClosureRefusal::ReadDir {
                path: dir.display().to_string(),
                error: e.to_string(),
            })?;
    collected.sort_by_key(|entry| entry.file_name());
    for entry in collected {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files_fail_closed(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

/// Collect every name occurring anywhere in a parsed tree.
///
/// Walks all structural positions rather than a hand-picked subset, so a new
/// `Node` field cannot silently drop references. Includes binder and
/// declaration names; `referenced_names` is what subtracts them.
pub fn names_in_tree(node: &Rc<Node>, out: &mut HashSet<String>) {
    if !node.name.is_empty() {
        out.insert(node.name.clone());
    }
    for child in node.children.iter() {
        names_in_tree(child, out);
    }
    for param in node.params.iter() {
        names_in_tree(param, out);
    }
    for used in node.uses.iter() {
        names_in_tree(used, out);
    }
    for prop in node.properties.iter() {
        names_in_tree(prop, out);
    }
    if let Some(body) = node.body.as_ref() {
        names_in_tree(body, out);
    }
    if let Some(transport) = node.transport.as_ref() {
        names_in_tree(transport, out);
    }
    if let Some(annotation) = node.type_annotation.as_ref() {
        names_in_tree(annotation, out);
    }
    if let Some(inferred) = node.inferred.clone() {
        if let Some(type_node) = inferred_to_node(inferred) {
            names_in_tree(&type_node, out);
        }
    }
    if let Some(pattern) = node.match_pattern.as_ref() {
        names_in_pattern(pattern, out);
    }
}

fn names_in_pattern(pattern: &MatchPattern, out: &mut HashSet<String>) {
    match pattern {
        MatchPattern::Bind { declaration } => names_in_tree(declaration, out),
        MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
        } => {
            if !name.is_empty() {
                out.insert(name.clone());
            }
            if let Some(parent) = parent_enum {
                if !parent.is_empty() {
                    out.insert(parent.clone());
                }
            }
            for binding in field_bindings.iter() {
                names_in_tree(binding, out);
            }
        }
        MatchPattern::LitPattern { .. } | MatchPattern::Wildcard => {}
    }
}

/// The names a module declares at its own top level, including variant arms.
///
/// An item's children are its VARIANTS only when the item is a coproduct; for
/// a record they are its FIELDS, which are not module-scope names. The guard is
/// the same one `v1.compiler.resolve get_variant_names` applies, and dropping it
/// is not a small imprecision: field names like `num`, `name`, `value` and `id`
/// recur across the corpus, so indexing them as declarations made any tree that
/// merely mentioned such a name pull every module declaring a field of that
/// name, and the closure degenerated to nearly the whole corpus.
pub fn declared_names(module: &Rc<Node>) -> Vec<String> {
    let mut out = Vec::new();
    for item in module_items(module.clone()).iter() {
        if !item.name.is_empty() {
            out.push(item.name.clone());
        }
        if item.connective != Connective::Disj {
            continue;
        }
        for variant in item.children.iter() {
            if !variant.name.is_empty() {
                out.push(variant.name.clone());
            }
        }
    }
    out
}

/// Every name a parsed tree references, collected from REFERENCE POSITIONS
/// only.
///
/// This is the difference between a closure and the corpus. The first version
/// collected every name occurring anywhere, so a `let` binder, a parameter, or
/// a record-literal FIELD LABEL that happened to share a declaration's name
/// dragged that declaration's module in; because the corpus is densely
/// interconnected the result closed over nearly all of it. Subtracting binders
/// afterwards was a blunter form of the same idea and still left 1503 of 3711
/// modules for a five-line entry.
///
/// `ExprData` already discriminates what each node IS, so the precise question
/// is answerable directly: a name is a reference when it appears as a variable,
/// a call or method-call callee, or a record-literal type name, or anywhere
/// inside a type annotation. It is NOT a reference when it is a `let` binder or
/// a field-access label -- those name something bound or projected here, not a
/// declaration elsewhere.
///
/// Still an over-approximation, and deliberately so: a bare `ExprVar` that
/// resolves to a local binding is indistinguishable from one that resolves
/// cross-module without doing resolution, and resolution needs the closure.
/// The residue is bounded by names in reference position and is never widened
/// on failure.
pub fn referenced_names(module: &Rc<Node>) -> HashSet<String> {
    let mut out = HashSet::new();
    collect_references(module, false, &mut out);

    let mut binders: HashSet<String> = declared_names(module).into_iter().collect();
    collect_param_binders(module, &mut binders);
    binders.insert(module.name.clone());

    out.retain(|name| !binders.contains(name));
    out
}

fn name_is_reference_position(node: &Rc<Node>, in_type_position: bool) -> bool {
    if in_type_position {
        return true;
    }
    matches!(
        &*node.expr_data,
        ExprData::ExprVar { .. }
            | ExprData::ExprCall { .. }
            | ExprData::ExprMethodCall { .. }
            | ExprData::ExprRecordLit { .. }
    )
}

fn collect_references(node: &Rc<Node>, in_type_position: bool, out: &mut HashSet<String>) {
    if !node.name.is_empty() && name_is_reference_position(node, in_type_position) {
        out.insert(node.name.clone());
    }
    if let ExprData::ExprRecordLit {
        parent_enum: Some(parent),
    } = &*node.expr_data
    {
        out.insert(parent.clone());
    }

    // Positions that carry Node trees but are not in `node_field_roles`.
    // Dropping them would shrink the read set (a silent miss). They stay
    // walked until the role map names them.
    //
    // `inferred` is the parse-time home of a fn/alias return type
    // (`Resolved { node }`). A module named only in `-> T` is otherwise
    // invisible — the same omission class as match_pattern, just not yet
    // in the role map.
    if let Some(annotation) = node.type_annotation.as_ref() {
        collect_references(annotation, true, out);
    }
    if let Some(inferred) = node.inferred.clone() {
        if let Some(type_node) = inferred_to_node(inferred) {
            collect_references(&type_node, true, out);
        }
    }
    for used in node.uses.iter() {
        collect_references(used, in_type_position, out);
    }
    for prop in node.properties.iter() {
        collect_references(prop, in_type_position, out);
    }
    if let Some(transport) = node.transport.as_ref() {
        collect_references(transport, in_type_position, out);
    }

    // Authority: `v1.std.core` `node_field_roles`. Every SubValueField /
    // ChildrenListField must have an arm so a new field cannot silently drop
    // references (the match_pattern hole this branch inherited from the same
    // omitted walk in dag_collect_support and dag_collect).
    for (field, role) in node_field_roles().iter() {
        match (field.as_str(), role) {
            ("children", _) => {
                for child in node.children.iter() {
                    collect_references(child, in_type_position, out);
                }
            }
            ("params", _) => {
                // A parameter's own name is a binder, but its CHILDREN carry
                // its declared type -- parsed params hold the type there
                // rather than in type_annotation, which is measured, not
                // assumed: `fn f(x: T)` parses to a param named `x` with
                // type_annotation None and one child. Walking those children
                // in expression position collected nothing.
                for param in node.params.iter() {
                    for declared_type in param.children.iter() {
                        collect_references(declared_type, true, out);
                    }
                    if let Some(annotation) = param.type_annotation.as_ref() {
                        collect_references(annotation, true, out);
                    }
                    for nested in param.params.iter() {
                        collect_references(nested, in_type_position, out);
                    }
                }
            }
            ("body", NodeFieldRole::SubValueField) => {
                if let Some(body) = node.body.as_ref() {
                    collect_references(body, false, out);
                }
            }
            ("expr_data", NodeFieldRole::SubValueField) => {
                // Classification carrier, not a Node tree. Names come from
                // `node.name` and the ExprRecordLit parent_enum arm above.
            }
            ("match_pattern", NodeFieldRole::SubValueField) => {
                if let Some(pattern) = node.match_pattern.as_ref() {
                    collect_pattern_references(pattern, out);
                }
            }
            (other, NodeFieldRole::SubValueField | NodeFieldRole::ChildrenListField) => {
                panic!(
                    "source_closure collect_references: node_field_roles has unhandled sub-value field `{other}`"
                );
            }
            (_, NodeFieldRole::MetadataField) => {}
        }
    }
}

fn collect_pattern_references(pattern: &MatchPattern, out: &mut HashSet<String>) {
    match pattern {
        MatchPattern::Bind { declaration } => {
            collect_references(declaration, false, out);
        }
        MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
        } => {
            if !name.is_empty() {
                out.insert(name.clone());
            }
            if let Some(parent) = parent_enum {
                if !parent.is_empty() {
                    out.insert(parent.clone());
                }
            }
            for binding in field_bindings.iter() {
                collect_references(binding, false, out);
            }
        }
        MatchPattern::LitPattern { .. } | MatchPattern::Wildcard => {}
    }
}

fn collect_param_binders(node: &Rc<Node>, binders: &mut HashSet<String>) {
    for param in node.params.iter() {
        if !param.name.is_empty() {
            binders.insert(param.name.clone());
        }
        collect_param_binders(param, binders);
    }
    for child in node.children.iter() {
        collect_param_binders(child, binders);
    }
    if let Some(body) = node.body.as_ref() {
        collect_param_binders(body, binders);
    }
}

pub fn parse_file(path: &str, content: &str) -> Option<Rc<Node>> {
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), path.to_string());
    let source_index =
        crate::v1_std_core::build_newline_index(path.to_string(), content.to_string());
    let mut source_indices = im::HashMap::new();
    source_indices.insert(path.to_string(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    result.module.clone()
}

/// Same rule as `cli_run::extract_module_path`: a `.dag` fragment with no
/// leading `module` declaration is a parse fixture, not a seed. Indexing or
/// compiling it would refuse `expected keyword 'module'` on files such as
/// `src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag`.
fn source_declares_module_line(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return true;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            return false;
        }
    }
    false
}

/// Parse a source file, refusing on a diagnostic or a missing/empty module.
pub fn parse_file_admitted(path: &str, content: &str) -> Result<Rc<Node>, ClosureRefusal> {
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), path.to_string());
    let source_index =
        crate::v1_std_core::build_newline_index(path.to_string(), content.to_string());
    let mut source_indices = im::HashMap::new();
    source_indices.insert(path.to_string(), source_index.clone());
    let result = crate::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    if let Some(err) = result.error.as_ref() {
        let span = diagnostic_to_span(err.diagnostic.clone());
        let loc = byte_to_line_col(source_index, span.start);
        return Err(ClosureRefusal::ParseFailed {
            path: path.to_string(),
            message: format!(
                "{}:{}: {}",
                loc.line,
                loc.col,
                diagnostic_to_message(err.diagnostic.clone())
            ),
        });
    }
    match result.module.clone() {
        Some(module) if !module.name.is_empty() => Ok(module),
        Some(_) => Err(ClosureRefusal::EmptyModulePath {
            path: path.to_string(),
        }),
        None => Err(ClosureRefusal::ParseFailed {
            path: path.to_string(),
            message: "parse returned no module".to_string(),
        }),
    }
}

/// True when `rel` is under `dag/test/`. That tree is a witness corpus, not a
/// seed input: it is never an entry (entries are `src/v1/**.dag`), and putting
/// it in the declaration index would (1) refuse regen on a leftover `import`
/// in a claim file — measured: `dag/test/claim/root_d_checkpoint_scalar_declared_arity_witness_test.dag`
/// — and (2) homonym-pull test modules into the READ set, a silent widen of
/// the stub-dissolution class. Production leftovers still refuse.
pub fn relpath_is_dag_witness_tree(rel: &str) -> bool {
    let n = rel.replace('\\', "/");
    n == "dag/test" || n.starts_with("dag/test/")
}

/// Fail-closed declaration index: missing roots, IO failures, parse failures,
/// empty module paths, and duplicate module paths all refuse. This is the
/// regen authority; the tuple-returning builder below stays for test helpers
/// that already assert `unparsed` empty.
pub fn admit_declaration_index(
    roots: &[PathBuf],
    workspace_root: &Path,
) -> Result<DeclarationIndex, ClosureRefusal> {
    let files = dag_files_under_fail_closed(roots)?;
    admit_declaration_index_from_files(&files, workspace_root)
}

/// Regen seed index: same fail-closed walk as `admit_declaration_index`, minus
/// the dag witness tree (see `relpath_is_dag_witness_tree`).
pub fn admit_regen_declaration_index(
    roots: &[PathBuf],
    workspace_root: &Path,
) -> Result<DeclarationIndex, ClosureRefusal> {
    let files = dag_files_under_fail_closed(roots)?;
    let seed_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| {
            let rel = path
                .strip_prefix(workspace_root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            !relpath_is_dag_witness_tree(&rel)
        })
        .collect();
    admit_declaration_index_from_files(&seed_files, workspace_root)
}

pub fn admit_declaration_index_from_files(
    files: &[PathBuf],
    workspace_root: &Path,
) -> Result<DeclarationIndex, ClosureRefusal> {
    let mut by_name: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut modules: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for file_path in files {
        let content = std::fs::read_to_string(file_path).map_err(|e| ClosureRefusal::ReadFile {
            path: file_path.display().to_string(),
            error: e.to_string(),
        })?;
        let rel = file_path
            .strip_prefix(workspace_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();
        if !source_declares_module_line(&content) {
            continue;
        }
        let module = parse_file_admitted(&rel, &content)?;
        let module_path = module.name.clone();
        if let Some(existing) = modules.get(&module_path) {
            return Err(ClosureRefusal::DuplicateModule {
                module_path,
                first: existing.path.clone(),
                second: rel,
            });
        }
        for name in declared_names(&module) {
            by_name.entry(name).or_default().insert(module_path.clone());
        }
        modules.insert(module_path, Rc::new(SourceFile { path: rel, content }));
    }
    Ok(DeclarationIndex { by_name, modules })
}

/// Build the declaration index over `roots`.
///
/// A file that does not parse contributes NOTHING and is not silently treated
/// as empty-but-fine: it is returned in `unparsed` so the caller can refuse or
/// report rather than compile against a pool with a hole in it. Dropping it
/// quietly would be the empty-observation narrow DESIGN §5 forbids — "this file
/// declares nothing" and "I could not read this file" are different states.
pub fn build_declaration_index(
    roots: &[PathBuf],
    workspace_root: &Path,
) -> (DeclarationIndex, Vec<String>) {
    let mut by_name: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut modules: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut unparsed: Vec<String> = Vec::new();

    for file_path in dag_files_under(roots) {
        let Ok(content) = std::fs::read_to_string(&file_path) else {
            unparsed.push(file_path.to_string_lossy().to_string());
            continue;
        };
        let rel = file_path
            .strip_prefix(workspace_root)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();
        let Some(module) = parse_file(&rel, &content) else {
            unparsed.push(rel);
            continue;
        };
        let module_path = module.name.clone();
        if module_path.is_empty() {
            unparsed.push(rel);
            continue;
        }
        for name in declared_names(&module) {
            by_name.entry(name).or_default().insert(module_path.clone());
        }
        modules.insert(module_path, Rc::new(SourceFile { path: rel, content }));
    }

    (DeclarationIndex { by_name, modules }, unparsed)
}

/// The transitive closure of source files an entry actually references.
///
/// Fixpoint over parsed trees: parse what is in the closure, read the names it
/// references, pull in every module declaring one of those names, repeat. A
/// name declared by several modules pulls ALL of them, so the resolver sees the
/// genuine ambiguity and can refuse, rather than this layer picking a winner.
/// Same fixpoint, but reporting which NAMES pulled how many modules.
///
/// Width alone cannot say whether a wide closure is a genuine dependency fan-out
/// or one boilerplate name dragging hundreds of modules in a single step. This
/// attributes each pull to the name responsible, so the answer is measured
/// rather than inferred.
pub fn closure_for_entry_attributed(
    entry_path: &str,
    entry_content: &str,
    index: &DeclarationIndex,
) -> (Vec<Rc<SourceFile>>, Vec<(String, usize)>) {
    let mut attribution: HashMap<String, usize> = HashMap::new();
    let sources = closure_inner(entry_path, entry_content, index, &mut attribution);
    let mut ranked: Vec<(String, usize)> = attribution.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(15);
    (sources, ranked)
}

pub fn closure_for_entry(
    entry_path: &str,
    entry_content: &str,
    index: &DeclarationIndex,
) -> Vec<Rc<SourceFile>> {
    let mut attribution = HashMap::new();
    closure_inner(entry_path, entry_content, index, &mut attribution)
}

fn closure_inner(
    entry_path: &str,
    entry_content: &str,
    index: &DeclarationIndex,
    attribution: &mut HashMap<String, usize>,
) -> Vec<Rc<SourceFile>> {
    let entry_source = Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    });

    let mut in_closure: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut own_modules: HashSet<String> = HashSet::new();
    let mut frontier: Vec<Rc<SourceFile>> = vec![entry_source.clone()];
    let mut scanned: HashSet<String> = HashSet::new();

    while let Some(source) = frontier.pop() {
        if !scanned.insert(source.path.clone()) {
            continue;
        }
        let Some(tree) = parse_file(&source.path, &source.content) else {
            continue;
        };
        own_modules.insert(tree.name.clone());

        for name in referenced_names(&tree) {
            // A QUALIFIED reference names its declaring module directly: the
            // prefix IS the module path. The index is keyed on simple
            // declaration names, so `std.algebra.FieldOfFractions` never
            // matches `by_name` -- without this arm the dependency is silently
            // dropped and the closure comes back short. Qualification is what
            // replaces `import`, so this is the closure's only remaining way to
            // follow a cross-module edge.
            let qualified_owner = name
                .rfind('.')
                .map(|i| &name[..i])
                .filter(|prefix| index.source_for_module(prefix).is_some());
            let declaring: BTreeSet<String> =
                match (qualified_owner, index.modules_declaring(&name)) {
                    (Some(owner), _) => std::iter::once(owner.to_string()).collect(),
                    (None, Some(found)) => found.clone(),
                    (None, None) => continue,
                };
            for module_path in &declaring {
                if own_modules.contains(module_path) || in_closure.contains_key(module_path) {
                    continue;
                }
                let Some(dep) = index.source_for_module(module_path) else {
                    continue;
                };
                in_closure.insert(module_path.clone(), dep.clone());
                *attribution.entry(name.clone()).or_insert(0) += 1;
                frontier.push(dep.clone());
            }
        }
    }

    let mut out: Vec<Rc<SourceFile>> = in_closure.into_values().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.push(entry_source);
    out
}

/// Multi-entry reference closure: seed the frontier with every entry and run
/// ONE fixpoint. Regen has ~68 `src/v1` entries; calling `closure_for_entry`
/// once per entry would re-parse shared modules 68 times.
///
/// Reached sources that do not parse, and declaration-index rows with no
/// source, refuse. A name with no declaration candidate is skipped — that is
/// the precision frontier (locals, builtins not in the index), not a silent
/// IO/parse miss.
///
/// Cross-module edges are qualified names only (`container.member`). Bare-name
/// pulls were measured and refused as READ policies: homonym-all 1203,
/// unique-bare 1130, unique+std-homonym 1086. All three compiled modules
/// that main's import closure never admitted (or, for homonyms, both forks
/// of `OccurrenceId`), which made names ambiguous rather than resolving.
/// Regen READ therefore follows main's import lists (see
/// `cli_run::regen_input_sources`); this walk stays the namespace-cut
/// reference closure for callers that already write `container.member`.
pub fn closure_for_entries(
    entries: &[(String, String)],
    index: &DeclarationIndex,
) -> Result<Vec<Rc<SourceFile>>, ClosureRefusal> {
    let mut attribution = HashMap::new();
    closure_from_seeds(entries, index, &mut attribution)
}

/// The namespace-cut dependency edge: a qualified name's prefix that is a
/// known module path. Tries longest prefix first (`std.foo.Bar.Baz` prefers
/// module `std.foo.Bar` if it exists, else `std.foo`).
fn qualified_declaring_module<'a>(name: &'a str, index: &DeclarationIndex) -> Option<&'a str> {
    let mut end = name.len();
    while let Some(dot) = name[..end].rfind('.') {
        let prefix = &name[..dot];
        if index.source_for_module(prefix).is_some() {
            return Some(prefix);
        }
        end = dot;
    }
    None
}

pub fn closure_for_entries_attributed(
    entries: &[(String, String)],
    index: &DeclarationIndex,
) -> Result<(Vec<Rc<SourceFile>>, Vec<(String, usize)>), ClosureRefusal> {
    let mut attribution: HashMap<String, usize> = HashMap::new();
    let sources = closure_from_seeds(entries, index, &mut attribution)?;
    let mut ranked: Vec<(String, usize)> = attribution.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(15);
    Ok((sources, ranked))
}

fn closure_from_seeds(
    entries: &[(String, String)],
    index: &DeclarationIndex,
    attribution: &mut HashMap<String, usize>,
) -> Result<Vec<Rc<SourceFile>>, ClosureRefusal> {
    let mut in_closure: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut frontier: Vec<Rc<SourceFile>> = Vec::new();
    let mut scanned: HashSet<String> = HashSet::new();

    for (path, content) in entries {
        let source = Rc::new(SourceFile {
            path: path.clone(),
            content: content.clone(),
        });
        frontier.push(source);
    }

    while let Some(source) = frontier.pop() {
        if !scanned.insert(source.path.clone()) {
            continue;
        }
        let tree = parse_file_admitted(&source.path, &source.content)?;
        in_closure
            .entry(tree.name.clone())
            .or_insert_with(|| source.clone());

        for name in referenced_names(&tree) {
            let Some(owner) = qualified_declaring_module(&name, index) else {
                continue;
            };
            if in_closure.contains_key(owner) {
                continue;
            }
            let Some(dep) = index.source_for_module(owner) else {
                return Err(ClosureRefusal::DeclaredModuleMissingSource {
                    module_path: owner.to_string(),
                    referenced_name: name.clone(),
                });
            };
            in_closure.insert(owner.to_string(), dep.clone());
            *attribution.entry(name.clone()).or_insert(0) += 1;
            frontier.push(dep.clone());
        }
    }

    let mut out: Vec<Rc<SourceFile>> = in_closure.into_values().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Multi-entry closure whose edges come from an external import-list oracle,
/// not from names in this tree. Namespace-cut sources have no `import`
/// statements; main's lists are the exact READ set that compiled (142 files
/// at the measured green main, including `std.occurrence_identity` and
/// excluding `std.observation`). This tree supplies file bytes; the oracle
/// supplies which modules those bytes used to name.
pub fn closure_for_entries_following_imports<F>(
    entries: &[(String, String)],
    index: &DeclarationIndex,
    mut imports_of: F,
) -> Result<Vec<Rc<SourceFile>>, ClosureRefusal>
where
    F: FnMut(&str) -> Result<Vec<String>, ClosureRefusal>,
{
    let mut in_closure: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut frontier: Vec<String> = Vec::new();
    let mut scanned: HashSet<String> = HashSet::new();

    for (path, content) in entries {
        let tree = parse_file_admitted(path, content)?;
        let source = Rc::new(SourceFile {
            path: path.clone(),
            content: content.clone(),
        });
        in_closure
            .entry(tree.name.clone())
            .or_insert_with(|| source.clone());
        frontier.push(path.clone());
    }

    while let Some(path) = frontier.pop() {
        if !scanned.insert(path.clone()) {
            continue;
        }
        for module_path in imports_of(&path)? {
            if in_closure.contains_key(&module_path) {
                continue;
            }
            let Some(dep) = index.source_for_module(&module_path) else {
                return Err(ClosureRefusal::DeclaredModuleMissingSource {
                    module_path: module_path.clone(),
                    referenced_name: format!("import {module_path}"),
                });
            };
            in_closure.insert(module_path, dep.clone());
            frontier.push(dep.path.clone());
        }
    }

    let mut out: Vec<Rc<SourceFile>> = in_closure.into_values().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};
    use std::fs;
    use std::path::PathBuf;

    fn fixture_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gunbc-source-closure-{tag}-{}", std::process::id()))
    }

    fn write(root: &std::path::Path, rel: &str, content: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir fixture");
        fs::write(&path, content).expect("write fixture");
    }

    /// A module referenced ONLY from a match-arm pattern must enter the
    /// closure. `Node.match_pattern` is a `SubValueField` in
    /// `v1.std.core` `node_field_roles` (same category as `body`); a walk
    /// that enumerates children/params/body/uses/properties/type_annotation/
    /// transport and skips it drops every qualified constructor that lives
    /// only in pattern position — the reference form this branch produces.
    ///
    /// Discriminating: the entry names `test.pattern_only_dep.PatternOnlyArm`
    /// solely inside `match`, never as a type, call, or record literal.
    /// Control: `test.unrelated` is never named and must stay out.
    #[test]
    fn pattern_only_reference_pulls_declaring_module() {
        let root = fixture_root("pattern-only");
        let _ = fs::remove_dir_all(&root);
        write(
            &root,
            "dep.dag",
            "module test.pattern_only_dep\n\
             type PatternOnlyMark = | PatternOnlyArm\n",
        );
        write(
            &root,
            "unrelated.dag",
            "module test.unrelated\n\
             type UnrelatedMark = | UnrelatedArm\n",
        );
        write(
            &root,
            "entry.dag",
            "module test.pattern_only_entry\n\
             fn classify(x: Int) -> Int {\n\
               match x {\n\
                 test.pattern_only_dep.PatternOnlyArm => 1\n\
                 _ => 0\n\
               }\n\
             }\n",
        );

        let (index, unparsed) = build_declaration_index(&[root.clone()], &root);
        assert!(unparsed.is_empty(), "fixture must parse: {unparsed:?}");
        assert!(
            index.source_for_module("test.pattern_only_dep").is_some(),
            "dep module must be in the declaration index"
        );

        let entry_path = "entry.dag";
        let entry_content = fs::read_to_string(root.join(entry_path)).expect("read entry");
        let tree = parse_file(entry_path, &entry_content).expect("entry parses");
        let names = referenced_names(&tree);
        assert!(
            names.iter().any(|n| n.contains("PatternOnlyArm")
                || n == "test.pattern_only_dep"
                || n.starts_with("test.pattern_only_dep.")),
            "pattern-only constructor must appear in referenced_names; got {names:?}"
        );

        let sources = closure_for_entry(entry_path, &entry_content, &index);
        let paths: BTreeSet<String> = sources.iter().map(|s| s.path.clone()).collect();
        let _ = fs::remove_dir_all(&root);

        assert!(
            paths
                .iter()
                .any(|p| p.ends_with("dep.dag") || p == "dep.dag"),
            "pattern-only reference must pull test.pattern_only_dep: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.ends_with("unrelated.dag")),
            "unrelated module must stay out: {paths:?}"
        );
    }

    /// Fn return types live in `Node.inferred`, which is not in
    /// `node_field_roles`. A module named only in `-> T` must still be pulled.
    #[test]
    fn return_type_only_reference_pulls_declaring_module() {
        let root = fixture_root("return-type-only");
        let _ = fs::remove_dir_all(&root);
        write(
            &root,
            "dep.dag",
            "module test.return_type_dep\ntype ReturnOnlyMark = Int\n",
        );
        write(
            &root,
            "unrelated.dag",
            "module test.return_unrelated\ntype U = Int\n",
        );
        write(
            &root,
            "entry.dag",
            "module test.return_type_entry\n\
             fn classify() -> test.return_type_dep.ReturnOnlyMark { 0 }\n",
        );

        let (index, unparsed) = build_declaration_index(&[root.clone()], &root);
        assert!(unparsed.is_empty(), "fixture must parse: {unparsed:?}");
        let entry_content = fs::read_to_string(root.join("entry.dag")).expect("read entry");
        let tree = parse_file("entry.dag", &entry_content).expect("entry parses");
        let names = referenced_names(&tree);
        assert!(
            names.iter().any(|n| n.contains("ReturnOnlyMark")
                || n == "test.return_type_dep"
                || n.starts_with("test.return_type_dep.")),
            "return-type constructor must appear in referenced_names; got {names:?}"
        );
        let sources = closure_for_entry("entry.dag", &entry_content, &index);
        let paths: BTreeSet<String> = sources.iter().map(|s| s.path.clone()).collect();
        let _ = fs::remove_dir_all(&root);
        assert!(
            paths
                .iter()
                .any(|p| p.ends_with("dep.dag") || p == "dep.dag"),
            "return-type-only reference must pull test.return_type_dep: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.ends_with("unrelated.dag")),
            "unrelated module must stay out: {paths:?}"
        );
    }

    #[test]
    fn dag_witness_tree_relpath_is_excluded() {
        assert!(relpath_is_dag_witness_tree(
            "dag/test/claim/leftover_import.dag"
        ));
        assert!(relpath_is_dag_witness_tree("dag/test"));
        assert!(!relpath_is_dag_witness_tree("dag/std/syntax.dag"));
        assert!(!relpath_is_dag_witness_tree(
            "src/v1/tests/claim/emitter.dag"
        ));
        assert!(!relpath_is_dag_witness_tree("dag/testing/foo.dag"));
    }

    #[test]
    fn moduleless_fixture_is_skipped_not_refused() {
        let root = fixture_root("moduleless");
        let _ = fs::remove_dir_all(&root);
        write(&root, "entry.dag", "module test.ml_entry\ntype T = Int\n");
        write(&root, "frag.dag", "data x: Int = 0\n");
        let index = admit_declaration_index(&[root.clone()], &root).expect("moduleless skip");
        let _ = fs::remove_dir_all(&root);
        assert_eq!(index.module_count(), 1);
    }

    #[test]
    fn missing_root_refuses() {
        let missing = fixture_root("missing-root");
        let _ = fs::remove_dir_all(&missing);
        let err = match admit_declaration_index(&[missing.clone()], &missing) {
            Err(e) => e,
            Ok(_) => panic!("missing root must refuse"),
        };
        assert!(
            matches!(err, ClosureRefusal::MissingRoot { .. }),
            "got {err}"
        );
    }

    #[test]
    fn duplicate_module_path_refuses() {
        let root = fixture_root("dup-module");
        let _ = fs::remove_dir_all(&root);
        write(&root, "a.dag", "module test.dup\ntype A = Int\n");
        write(&root, "b.dag", "module test.dup\ntype B = Int\n");
        let err = match admit_declaration_index(&[root.clone()], &root) {
            Err(e) => e,
            Ok(_) => panic!("duplicate module path must refuse"),
        };
        let _ = fs::remove_dir_all(&root);
        assert!(
            matches!(err, ClosureRefusal::DuplicateModule { .. }),
            "got {err}"
        );
    }

    #[test]
    fn multi_entry_union_runs_one_fixpoint() {
        let root = fixture_root("multi-entry");
        let _ = fs::remove_dir_all(&root);
        write(
            &root,
            "shared.dag",
            "module test.shared\ntype SharedMark = | SharedArm\n",
        );
        write(
            &root,
            "one.dag",
            "module test.one\nfn f(x: test.shared.SharedMark) -> Int { 1 }\n",
        );
        write(
            &root,
            "two.dag",
            "module test.two\nfn g(x: test.shared.SharedMark) -> Int { 2 }\n",
        );
        write(
            &root,
            "unrelated.dag",
            "module test.unrelated\ntype U = Int\n",
        );

        let index = admit_declaration_index(&[root.clone()], &root).expect("index");
        let one = fs::read_to_string(root.join("one.dag")).unwrap();
        let two = fs::read_to_string(root.join("two.dag")).unwrap();
        let sources =
            closure_for_entries(&[("one.dag".into(), one), ("two.dag".into(), two)], &index)
                .expect("multi-entry closure");
        let paths: BTreeSet<String> = sources.iter().map(|s| s.path.clone()).collect();
        let _ = fs::remove_dir_all(&root);

        assert!(paths.contains("one.dag"), "{paths:?}");
        assert!(paths.contains("two.dag"), "{paths:?}");
        assert!(
            paths.contains("shared.dag"),
            "shared dep must be pulled: {paths:?}"
        );
        assert!(
            !paths.contains("unrelated.dag"),
            "unrelated must stay out: {paths:?}"
        );
    }

    /// Import-oracle edges, not names in this tree, decide READ. The entry
    /// has no qualified reference to `test.dep`; the oracle lists that
    /// import; the dep enters and the unreferenced module stays out.
    #[test]
    fn import_oracle_pulls_listed_module_only() {
        let root = fixture_root("import-oracle");
        let _ = fs::remove_dir_all(&root);
        write(&root, "dep.dag", "module test.dep\ntype T = Int\n");
        write(
            &root,
            "unrelated.dag",
            "module test.unrelated\ntype U = Int\n",
        );
        write(
            &root,
            "entry.dag",
            "module test.entry\nfn f(x: T) -> T { x }\n",
        );
        let index = admit_declaration_index(&[root.clone()], &root).expect("index");
        let entry = fs::read_to_string(root.join("entry.dag")).unwrap();
        let mut imports: HashMap<String, Vec<String>> = HashMap::new();
        imports.insert("entry.dag".into(), vec!["test.dep".into()]);
        imports.insert("dep.dag".into(), vec![]);
        let sources =
            closure_for_entries_following_imports(&[("entry.dag".into(), entry)], &index, |path| {
                Ok(imports.get(path).cloned().unwrap_or_default())
            })
            .expect("oracle closure");
        let paths: BTreeSet<String> = sources.iter().map(|s| s.path.clone()).collect();
        let _ = fs::remove_dir_all(&root);
        assert!(paths.contains("entry.dag"), "{paths:?}");
        assert!(
            paths.contains("dep.dag"),
            "oracle import must pull dep: {paths:?}"
        );
        assert!(
            !paths.contains("unrelated.dag"),
            "unlisted module must stay out: {paths:?}"
        );
    }

    /// A bare name must not pull a homonymous declarer — that over-approx
    /// compiled dag/extdeps into the seed.
    #[test]
    fn bare_homonym_off_seed_surface_is_not_pulled() {
        let root = fixture_root("bare-homonym");
        let _ = fs::remove_dir_all(&root);
        write(&root, "other.dag", "module test.other\ntype T = Int\n");
        write(
            &root,
            "elsewhere.dag",
            "module test.elsewhere\ntype T = Int\n",
        );
        write(
            &root,
            "entry.dag",
            "module test.entry\nfn f(x: T) -> T { x }\n",
        );
        let index = admit_declaration_index(&[root.clone()], &root).expect("index");
        let entry = fs::read_to_string(root.join("entry.dag")).unwrap();
        let sources = closure_for_entries(&[("entry.dag".into(), entry)], &index).expect("closure");
        let paths: BTreeSet<String> = sources.iter().map(|s| s.path.clone()).collect();
        let _ = fs::remove_dir_all(&root);
        assert!(paths.contains("entry.dag"), "{paths:?}");
        assert!(
            !paths.contains("other.dag") && !paths.contains("elsewhere.dag"),
            "off-surface homonym T must not pull either declarer: {paths:?}"
        );
    }

    /// Regression: a qualified `let` binder (`let v1.compiler.coercion.is_copy =`)
    /// is not a legal local. The seed must parse after that residue is restored
    /// to an unqualified binder.
    #[test]
    fn emit_rust_seed_parses() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../05_emit_rust.dag");
        let content = fs::read_to_string(&path).expect("read 05_emit_rust.dag");
        match parse_file_admitted("src/v1/05_emit_rust.dag", &content) {
            Ok(_) => {}
            Err(err) => panic!("emit_rust seed parse refusal: {err}"),
        }
    }
}

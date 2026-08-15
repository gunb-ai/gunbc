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
use crate::v1_std_core::{module_items, Connective, Node};

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

/// Every name a parsed tree references FREELY: occurring names minus the names
/// the tree itself binds.
///
/// This is the difference between a closure and the corpus. Pulling on every
/// occurring name means a parameter, a `let` binder, or a record-literal field
/// label that happens to share a declaration's name drags that declaration's
/// module in, and because the corpus is densely interconnected the result
/// closes over nearly all of it. Measured before this narrowing: a five-line
/// entry produced a closure wide enough that compiling it was a whole-corpus
/// typecheck.
///
/// Binders subtracted here: the module's own declarations (including coproduct
/// variant arms) and every parameter name at any depth. Names bound inside
/// bodies by other forms are NOT yet subtracted, so this remains an
/// over-approximation — a narrower one, computed as the answer, never widened
/// on failure.
pub fn referenced_names(module: &Rc<Node>) -> HashSet<String> {
    let mut occurring = HashSet::new();
    names_in_tree(module, &mut occurring);

    let mut binders: HashSet<String> = declared_names(module).into_iter().collect();
    collect_param_binders(module, &mut binders);
    binders.insert(module.name.clone());

    occurring.retain(|name| !binders.contains(name));
    occurring
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

fn parse_file(path: &str, content: &str) -> Option<Rc<Node>> {
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), path.to_string());
    let source_index =
        crate::v1_std_core::build_newline_index(path.to_string(), content.to_string());
    let mut source_indices = im::HashMap::new();
    source_indices.insert(path.to_string(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    result.module.clone()
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
pub fn closure_for_entry(
    entry_path: &str,
    entry_content: &str,
    index: &DeclarationIndex,
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
            let Some(declaring) = index.modules_declaring(&name) else {
                continue;
            };
            for module_path in declaring {
                if own_modules.contains(module_path) || in_closure.contains_key(module_path) {
                    continue;
                }
                let Some(dep) = index.source_for_module(module_path) else {
                    continue;
                };
                in_closure.insert(module_path.clone(), dep.clone());
                frontier.push(dep.clone());
            }
        }
    }

    let mut out: Vec<Rc<SourceFile>> = in_closure.into_values().collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.push(entry_source);
    out
}

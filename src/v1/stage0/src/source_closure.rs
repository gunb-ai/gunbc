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
use crate::v1_std_core::{module_items, Connective, ExprData, MatchPattern, Node};

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
    if let Some(pattern) = node.match_pattern.as_ref() {
        names_in_pattern(pattern, out);
    }
}

fn names_in_pattern(pattern: &Rc<MatchPattern>, out: &mut HashSet<String>) {
    match &**pattern {
        MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
        } => {
            if !name.is_empty() {
                out.insert(name.clone());
            }
            if let Some(parent) = parent_enum {
                out.insert(parent.clone());
            }
            for binding in field_bindings.iter() {
                names_in_tree(binding, out);
            }
        }
        MatchPattern::Bind { declaration } => names_in_tree(declaration, out),
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

    // A type annotation subtree is entirely type references.
    if let Some(annotation) = node.type_annotation.as_ref() {
        collect_references(annotation, true, out);
    }

    for child in node.children.iter() {
        collect_references(child, in_type_position, out);
    }
    // A parameter's own name is a binder, but its CHILDREN carry its declared
    // type -- parsed params hold the type there rather than in type_annotation,
    // which is measured, not assumed: `fn f(x: T)` parses to a param named `x`
    // with type_annotation None and one child. Walking those children in
    // expression position (the earlier shape) collected nothing, so every
    // parameter-type dependency was silently missing from the closure.
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
    for used in node.uses.iter() {
        collect_references(used, in_type_position, out);
    }
    for prop in node.properties.iter() {
        collect_references(prop, in_type_position, out);
    }
    if let Some(body) = node.body.as_ref() {
        collect_references(body, false, out);
    }
    if let Some(transport) = node.transport.as_ref() {
        collect_references(transport, in_type_position, out);
    }
    // A match pattern is a REFERENCE POSITION, and it was the one the walk
    // missed entirely. `match x { Present { v } => ... }` names `Present` --
    // and, through parent_enum, the type that declares it -- nowhere else in
    // the node, because a pattern hangs off match_pattern rather than off
    // children/body. So a variant referenced ONLY in a pattern never pulled its
    // declaring module into the closure, and the module then resolved without
    // it. That is not a missing edge in the abstract: it is measured as
    // `variant 'Empty'/'Cons' not found in type 'List'`,
    // `non-exhaustive match: missing variant(s) Present, Absent`, and
    // `variant 'Accepted'/'Rejected' not found in type 'String'` in the live
    // corpus -- roughly forty diagnostics from this one omission.
    //
    // Bind and Wildcard introduce binders rather than referencing anything, so
    // they contribute no edge; LitPattern names no declaration.
    if let Some(pattern) = node.match_pattern.as_ref() {
        collect_pattern_references(pattern, out);
    }
}

fn collect_pattern_references(pattern: &Rc<MatchPattern>, out: &mut HashSet<String>) {
    match &**pattern {
        MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
        } => {
            if !name.is_empty() {
                out.insert(name.clone());
            }
            if let Some(parent) = parent_enum {
                out.insert(parent.clone());
            }
            // A field binding is a binder, but it may carry a nested pattern or
            // a declared type of its own.
            for binding in field_bindings.iter() {
                collect_references(binding, false, out);
            }
        }
        MatchPattern::Bind { declaration } => {
            collect_references(declaration, false, out);
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
            // A DOTTED NAME WHOSE PREFIX IS NOT A MODULE is a projection off a
            // declared root rather than a qualified declaration: `Filesystem.Read`
            // names an operation of the service `Filesystem`, and `Filesystem` is
            // what some module declares. Without this arm the whole dotted string
            // matches nothing -- neither `source_for_module("Filesystem")` nor
            // `modules_declaring("Filesystem.Read")` -- and the edge is dropped
            // silently, so a module reached ONLY through a service call never
            // enters the closure and every call on it reports `undefined
            // variable` at the root segment. The root segment is looked up
            // exactly as a bare reference would be, so this adds no name that a
            // bare spelling could not already reach.
            let declaring: BTreeSet<String> =
                match (qualified_owner, index.modules_declaring(&name)) {
                    (Some(owner), _) => std::iter::once(owner.to_string()).collect(),
                    (None, Some(found)) => found.clone(),
                    (None, None) => {
                        let root = name.split('.').next().unwrap_or(&name);
                        match if root == name {
                            None
                        } else {
                            index.modules_declaring(root)
                        } {
                            Some(found) => found.clone(),
                            None => continue,
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
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

    /// RETAINED EVIDENCE, NO EXECUTING CONSUMER. CI does not run `cargo test`
    /// (operator ruling 2026-07-11) and gunbc#8146 deleted the v1 Rust suite.
    /// This `#[test]` is not coverage. Restoration trigger: the first of
    /// (a) `src/v1` enters a witness-executing source-root set, (b) this
    /// control is re-expressed as a floor witness the current roots reach, or
    /// (c) the v1 seed regains an executing test home. Until then the
    /// match-pattern walk in `collect_references` is landed and UNWITNESSED;
    /// this fn is the receipt.
    ///
    /// A module referenced ONLY from a match-arm pattern must enter the
    /// closure. Discriminating: the entry names
    /// `test.pattern_only_dep.PatternOnlyArm` solely inside `match`, never as
    /// a type, call, or record literal. Negative control: `test.unrelated` is
    /// never named and must stay out.
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
}

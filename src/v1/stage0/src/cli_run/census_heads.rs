// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 9 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::clone_on_copy,  // 6
    clippy::type_complexity,  // 1
    dead_code,  // 2
    unused_imports,  // 0 -- pre-existing
)]
// cli_run.rs is this module's PARENT, and an `#![allow]` there reaches every module
// under it -- the same cascade this commit removed at the crate root, one level down.
// These are the names its roster carries that this module does not trip, restored to
// warn so `-D warnings` still judges them here. A name moves from this list to the
// allow list above only with a counted site, never silently.
#![warn(
    clippy::assertions_on_constants,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::disallowed_macros,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    unused_mut
)]

use super::*;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::module_path_index::{
    parse_module_binding, ModuleBindingOutcome, ModuleBindingRefusal, ParsedModuleBinding,
};
use crate::shared_typecheck_store::{self, SharedTypecheckCaches};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::{
    lookup_binding_by_name, lookup_type_by_name, qualified_all_but_last, symbol_index_insert,
    symbol_index_lookup, GlobalBareLookupState, SymbolIndex, TypeEnv,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_lookup::global_bare_callable_node;
use crate::v1_compiler_infer_method::infer_builtin_call_type;
use crate::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_interpreter::str_value;
use crate::v1_interpreter::Value;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, block_stmts,
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    empty_intern_table, empty_node_list, expr_call_func_at, expr_method_name_at, expr_var_name_at,
    field_access_base, field_access_field_at, field_init_node_name_at, field_init_node_value,
    has_child_named, inferred_to_node, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, make_error_node,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span,
    param_node_name_at, param_node_type_expr, Cardinality, CompilerDiagnostic, Connective,
    ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex,
    Node,
};
use serde::Serialize;

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_fn_stand_in_for_test() -> Rc<Node> {
    crate::v1_compiler_parse::census_heads_body_stand_in()
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn census_heads_module_node_for_test(module: Rc<Node>) -> Rc<Node> {
    census_heads_module_node(module)
}

pub(crate) fn census_heads_module_node(module: Rc<Node>) -> Rc<Node> {
    crate::v1_compiler_compile::census_heads_module_node(module)
}

/// One module read BOTH ways and normalized by the census, so the two readings can be
/// compared as an identity rather than described as similar.
///
/// `census_heads_module_node` is applied to both sides. That is what makes the comparison
/// meaningful rather than trivially false: the body slot is the one slot the heads reading
/// deliberately fills differently, the normalizer overwrites it on both sides, and what
/// remains is exactly the declaration heads the pool census consumes. A skip that swallowed
/// a declaration, mis-counted a brace depth, or left the token stream one token off changes
/// the head list and diverges here.
///
/// Both readings start from the SAME intern-table snapshot and neither writes back, so the
/// sides are symmetric — a difference is the reading, never the order they ran in.
pub(crate) fn census_heads_both_readings(
    index: &MultiEntryIndex,
    source: &Rc<v1_compiler_compile::SourceFile>,
) -> (
    (Result<Rc<Node>, String>, u128),
    (Result<Rc<Node>, String>, u128),
) {
    let table = index.intern_table.borrow().clone();
    let read = |heads_only: bool| -> (Result<Rc<Node>, String>, u128) {
        let tokens = v1_compiler_tokenize::tokenize(
            source.content.clone(),
            source.path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let nl_index = build_newline_index(source.path.clone(), source.content.clone());
        let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
            let mut m = HashMap::new();
            m.insert(source.path.clone(), nl_index);
            m
        });
        // Only the parse is inside the timer: tokenize, newline index and setup are
        // identical work in both readings and sit outside it on purpose.
        let started = std::time::Instant::now();
        let parsed = if heads_only {
            v1_compiler_parse::parse_heads_with_table(tokens, single_si, table.clone())
        } else {
            v1_compiler_parse::parse_with_table(tokens, single_si, table.clone())
        };
        let nanos = started.elapsed().as_nanos();
        if let Some(err) = &parsed.result.error {
            return (Err(diagnostic_to_message(err.diagnostic.clone())), nanos);
        }
        match &parsed.result.module {
            Some(module) => (Ok(census_heads_module_node(module.clone())), nanos),
            None => (Err("no module in parse result".to_string()), nanos),
        }
    };
    (read(false), read(true))
}

#[cfg(test)]
mod heads_reading_item_boundary_tests {
    //! A DATA VALUE ENDS WHERE AN ITEM BEGINS, and "an item begins" is the grammar's own answer:
    //! its item forms plus the test marker `parse_item` peels first. The heads reading used a
    //! hand-written keyword list with no test marker, so the test item below was consumed into the
    //! data value above it and its name vanished from every pool census on that reading. The
    //! declaration-name population of `heads_reading_differential` is this class's corpus-wide
    //! reading; this is its smallest specimen. Local diligence only: the Rust unit run is off the
    //! merge path (`gunbc.rung_drop` `rust_unit_tests_off_the_merge_path`).
    use super::*;

    fn heads_names(content: &str) -> Vec<String> {
        let path = "dag/test/fixture/heads_after_data_provider_test.dag".to_string();
        let tokens = v1_compiler_tokenize::tokenize(
            content.to_string(),
            path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let mut si = HashMap::new();
        si.insert(
            path.clone(),
            build_newline_index(path.clone(), content.to_string()),
        );
        let parsed = v1_compiler_parse::parse_heads_with_table(
            tokens,
            Rc::new(si),
            crate::v1_std_core::empty_intern_table(),
        );
        assert!(
            parsed.result.error.is_none(),
            "heads reading refused the specimen"
        );
        let module = parsed.result.module.clone().expect("module");
        module
            .children
            .iter()
            .map(|item| item.name.clone())
            .collect()
    }

    #[test]
    fn a_test_item_after_a_data_value_is_its_own_item() {
        let names = heads_names(include_str!(
            "../../../../../dag/test/fixture/heads_after_data_provider_test.dag"
        ));
        assert!(
            names.iter().any(|n| n == "heads_after_data_marked_probe"),
            "the test item after the data value was swallowed: {names:?}"
        );
        assert!(
            names.iter().any(|n| n == "heads_before_data_marked_probe"),
            "the control row before the data value must also be read: {names:?}"
        );
    }
}

/// THE POOL CENSUS'S READING OF ONE FILE, projected from the file-local heads reading
/// `pool_acquire::heads_reading_for` holds rather than parsed a second time.
///
/// The census threads one intern table and one occurrence-ordinal space across the pool, and the
/// closure parses continue from it, so its nodes are not the file-local reading's values. The
/// projection between them is TOTAL, because both of the reading's space-dependent facts are
/// derived from the file alone plus the space's state on entry:
///   - OCCURRENCE IDS are allocated sequentially from the entry base (`alloc_occurrence_id`,
///     `occurrence_allocator_after_index`), so a local id `k` is pool id `base + k`, and the
///     allocator leaves at `base + local_next`.
///   - IDENTS are intern ids. The local table interns this file's strings in first-sight order,
///     which is exactly the order the threaded parse interns the ones the pool has not seen, so
///     interning the local strings in local-id order into the incoming table reproduces the
///     threaded table, and `relabel[k]` is the pool id of local ident `k`.
///
/// Anything the parser does not produce refuses instead of being guessed at: a projected
/// occurrence, or expression data carrying semantic payload. The receipt that
/// the projection is the threaded reading is
/// `heads_projection_tests` (fixture) and `entry_resolve::heads_projection_live_differential`
/// (live pool).
pub(crate) fn project_heads_reading(
    local: &pool_acquire::HeadsReading,
    incoming: &Rc<InternTable>,
) -> Result<(crate::v1_compiler_parse::ParseResult, Rc<InternTable>), String> {
    let base = incoming.authored_token_ordinals.allocator.next_id;
    let mut table = incoming.clone();
    let mut relabel: Vec<i64> = Vec::with_capacity(local.local_strings.len());
    for s in local.local_strings.iter() {
        let r = crate::v1_std_core::intern(table.clone(), s.clone());
        relabel.push(r.id);
        table = r.table.clone();
    }
    let local_next = local.local_next;
    let table = crate::v1_std_core::intern_table_with_authored_token_ordinals(
        table,
        crate::std_occurrence_identity::authored_token_ordinal_space_from_allocator(
            crate::std_occurrence_identity::OccurrenceIdAllocator {
                next_id: base + local_next,
            },
        ),
    );
    let module = match &local.module {
        Some(m) => Some(project_node(m, base, &relabel)?),
        None => None,
    };
    Ok((
        crate::v1_compiler_parse::ParseResult {
            module,
            error: local.error.clone(),
        },
        table,
    ))
}

fn project_node(n: &Rc<Node>, base: i64, relabel: &[i64]) -> Result<Rc<Node>, String> {
    use crate::std_occurrence_identity::{NodeOccurrenceIdentity, OccurrenceId};
    use crate::v1_std_core::{ExprData, InferredNode};
    // EXHAUSTIVE BY CONSTRUCTION: every `Node` field, and every arm of every enum below that can
    // hold a node or an id, is named with no `..` and no wildcard, so a field or variant added
    // later fails to compile here rather than passing through with file-local ids.
    let Node {
        occurrence_identity,
        name,
        ident,
        span,
        ident_span,
        children,
        connective,
        params,
        inferred,
        return_cardinality,
        uses,
        body,
        transport,
        properties,
        type_annotation,
        is_self_recursive,
        has_non_tail_self_call,
        match_pattern,
        module_item_kind,
        declaration_marker,
        declaration,
        expr_data,
    } = &**n;
    let refuse = |what: &str| {
        Err(format!(
            "heads projection refused: node '{name}' carries {what}, which the heads parser does \
             not produce"
        ))
    };
    let occurrence_identity = match &**occurrence_identity {
        NodeOccurrenceIdentity::OccurrenceSynthetic => occurrence_identity.clone(),
        NodeOccurrenceIdentity::OccurrenceMinted { id } => {
            Rc::new(NodeOccurrenceIdentity::OccurrenceMinted {
                id: OccurrenceId {
                    value: base + id.value,
                },
            })
        }
        NodeOccurrenceIdentity::OccurrenceProjected { .. } => {
            return refuse("a projected occurrence")
        }
        NodeOccurrenceIdentity::OccurrencePending { .. } => return refuse("a pending occurrence"),
    };
    let payload_free = match &**expr_data {
        ExprData::NoExprData
        | ExprData::ExprLiteral { value: _ }
        | ExprData::ExprError {
            kind: _,
            message: _,
        }
        | ExprData::ExprMatch
        | ExprData::ExprIf
        | ExprData::ExprLet
        | ExprData::ExprRecordLit { parent_enum: _ }
        | ExprData::ExprListLit
        | ExprData::ExprUnaryOp { op: _ }
        | ExprData::ExprLambda
        | ExprData::ExprStringInterp
        | ExprData::ExprBlock
        | ExprData::ExprCast
        | ExprData::ExprForEach
        | ExprData::ExprIndex
        | ExprData::ExprSlice
        | ExprData::ExprReturn => true,
        ExprData::ExprElaboratedLiteral {
            value: _,
            elaboration: _,
        } => false,
        ExprData::ExprVar { binding_kind } => binding_kind.is_none(),
        ExprData::ExprFieldAccess { summary } => summary.is_none(),
        ExprData::ExprCall {
            call_semantics,
            descent_evidence,
        } => call_semantics.is_none() && descent_evidence.is_none(),
        ExprData::ExprMethodCall { method_semantics } => method_semantics.is_none(),
        ExprData::ExprBinOp {
            op: _,
            algebra_field,
            operand,
        } => algebra_field.is_none() && operand.is_none(),
    };
    if !payload_free {
        return refuse("semantic expression data");
    }
    // Declaration identity is written by resolve, never by the parser.
    if declaration.is_some() {
        return refuse("a resolved declaration identity");
    }
    let ident = match ident {
        Some(k) => Some(*relabel.get(*k as usize).ok_or_else(|| {
            format!(
                "heads projection refused: node '{name}' ident {k} is outside the file's intern \
                 table"
            )
        })?),
        None => None,
    };
    let list = |v: &Rc<im::Vector<Rc<Node>>>| -> Result<Rc<im::Vector<Rc<Node>>>, String> {
        v.iter()
            .map(|c| project_node(c, base, relabel))
            .collect::<Result<im::Vector<_>, _>>()
            .map(Rc::new)
    };
    let opt = |o: &Option<Rc<Node>>| -> Result<Option<Rc<Node>>, String> {
        o.as_ref()
            .map(|c| project_node(c, base, relabel))
            .transpose()
    };
    // The parser records a written type expression as `Resolved { node }`: one more nested node.
    let inferred = match inferred.as_deref() {
        None => None,
        Some(InferredNode::Resolved { node }) => Some(Rc::new(InferredNode::Resolved {
            node: project_node(node, base, relabel)?,
        })),
        Some(InferredNode::CompilerError {
            message: _,
            span: _,
        })
        | Some(InferredNode::TypeVariable { id: _ })
        | Some(InferredNode::Divergent) => inferred.clone(),
    };
    let match_pattern = match match_pattern.as_deref() {
        None => None,
        Some(MatchPattern::Bind { declaration }) => Some(Rc::new(MatchPattern::Bind {
            declaration: project_node(declaration, base, relabel)?,
        })),
        Some(MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
            parent_identity,
        }) => Some(Rc::new(MatchPattern::VariantPattern {
            name: name.clone(),
            parent_enum: parent_enum.clone(),
            field_bindings: list(field_bindings)?,
            parent_identity: parent_identity.clone(),
        })),
        Some(MatchPattern::LitPattern { value: _ }) | Some(MatchPattern::Wildcard) => {
            match_pattern.clone()
        }
    };
    Ok(Rc::new(Node {
        occurrence_identity,
        name: name.clone(),
        ident,
        span: span.clone(),
        ident_span: ident_span.clone(),
        children: list(children)?,
        connective: connective.clone(),
        params: list(params)?,
        inferred,
        return_cardinality: return_cardinality.clone(),
        uses: list(uses)?,
        body: opt(body)?,
        transport: opt(transport)?,
        properties: list(properties)?,
        type_annotation: opt(type_annotation)?,
        is_self_recursive: *is_self_recursive,
        has_non_tail_self_call: *has_non_tail_self_call,
        match_pattern,
        module_item_kind: module_item_kind.clone(),
        declaration_marker: declaration_marker.clone(),
        declaration: None,
        expr_data: expr_data.clone(),
    }))
}

/// Thread `files` through the pool census's two readings in order -- the threaded parse the census
/// used to take, and the projection of the file-local reading it takes now -- and return every
/// divergence at identity grain: the whole projected module Node (every occurrence id and ident),
/// the refusal, and the intern table and occurrence allocator each file leaves behind.
#[cfg(test)]
pub(crate) fn heads_projection_divergences(files: &[(String, String)]) -> (usize, Vec<String>) {
    let mut threaded = crate::v1_std_core::empty_intern_table();
    let mut projected = crate::v1_std_core::empty_intern_table();
    let mut divergent = Vec::new();
    for (path, content) in files {
        let tokens = pool_acquire::tokens_for(path, content);
        let mut si = HashMap::new();
        si.insert(path.clone(), pool_acquire::newline_index_for(path, content));
        let old = v1_compiler_parse::parse_heads_with_table(tokens, Rc::new(si), threaded.clone());
        let local = pool_acquire::heads_reading_for(path, content);
        let (new, table) = match project_heads_reading(&local, &projected) {
            Ok(v) => v,
            Err(e) => {
                divergent.push(format!("{path}: {e}"));
                return (files.len(), divergent);
            }
        };
        if old.result.module != new.module {
            divergent.push(format!("{path}: module node"));
        }
        if old.result.error != new.error {
            divergent.push(format!("{path}: refusal"));
        }
        if old.intern_table != table {
            divergent.push(format!("{path}: intern table / occurrence allocator"));
        }
        threaded = old.intern_table.clone();
        projected = table;
    }
    (files.len(), divergent)
}

#[cfg(test)]
mod heads_projection_tests {
    use super::*;

    fn fixture() -> Vec<(String, String)> {
        [
            ("dag/test/fixture/proj_a.dag", "module proj.a\n\ntype Shape = Circle | Square\n\ntype Box<T> { value: T }\n\nfn area(s: Shape) -> Int {\n  match s {\n    Circle => 1,\n    Square => 2,\n  }\n}\n"),
            ("dag/test/fixture/proj_b.dag", "module proj.b\n\nfn area(b: Box<Int>) -> Int {\n  1\n}\n\ndata limit: Int = 3\n"),
            ("dag/test/fixture/proj_c.dag", "module proj.c\n\ntype Shape = Circle | Triangle\n\nfn fresh_name(x: Shape, y: Box<Shape>) -> Shape {\n  x\n}\n"),
            ("dag/test/fixture/proj_bad.dag", "module proj.bad\n\nfn broken( -> Int {\n  1\n}\n"),
        ]
        .iter()
        .map(|(p, c)| (p.to_string(), c.to_string()))
        .collect()
    }

    /// Strings repeat across files (`Shape`, `area`, `Box`), so identity only holds if idents are
    /// relabeled into the pool table and occurrence ids offset by what earlier files allocated;
    /// the malformed head checks that the refusal and the table it leaves agree too.
    #[test]
    fn projected_heads_equal_the_threaded_parse_at_identity_grain() {
        let (n, divergent) = heads_projection_divergences(&fixture());
        assert_eq!(n, 4);
        assert!(divergent.is_empty(), "divergent: {divergent:?}");
    }

    /// The red the comparison exists to catch: taking the file-local reading as the census
    /// reading, with no projection, diverges from the threaded parse from the second file on.
    #[test]
    fn the_unprojected_local_reading_is_not_the_threaded_reading() {
        let files = fixture();
        let (a, b) = (&files[0], &files[1]);
        let mut si = HashMap::new();
        si.insert(a.0.clone(), pool_acquire::newline_index_for(&a.0, &a.1));
        let first = v1_compiler_parse::parse_heads_with_table(
            pool_acquire::tokens_for(&a.0, &a.1),
            Rc::new(si),
            crate::v1_std_core::empty_intern_table(),
        );
        let mut si = HashMap::new();
        si.insert(b.0.clone(), pool_acquire::newline_index_for(&b.0, &b.1));
        let threaded = v1_compiler_parse::parse_heads_with_table(
            pool_acquire::tokens_for(&b.0, &b.1),
            Rc::new(si),
            first.intern_table.clone(),
        );
        let local = pool_acquire::heads_reading_for(&b.0, &b.1);
        assert_ne!(threaded.result.module, local.module);
        let (projected, _) = project_heads_reading(&local, &first.intern_table).unwrap();
        assert_eq!(threaded.result.module, projected.module);
    }
}

#![allow(clippy::disallowed_macros)]

// XL-0T-CUTOVER CENSUS INSTRUMENT. It answers ONE question and it is the question the cutover
// cannot be decided without: over the COMPLETE production TypeOccurrence population, what does
// X (the module-wide bare-name TypeEnv/precedence path repaired by #9813) bind, what does Y
// (std.occurrence_binding_candidates.resolve_type_reference_containment_binding, the exact
// per-occurrence containment authority) bind, and where do they disagree?
//
// WHY A SEPARATE INSTRUMENT AND NOT A TEST. The denominator is the corpus, not a fixture: the
// subject is "every production TypeOccurrence still obtaining its declaration identity through
// the flat TypeEnv path", which is a population fact about the tree at a commit. It is named
// here as a producer that re-derives it (DESIGN §6 "name the instrument, never transcribe its
// output") so no number this reports is ever copied into prose as an authority.
//
// IT DOES NOT NARROW. Every reference in the population lands in exactly one printed class,
// including every Y refusal arm, and the class totals are asserted to sum to the denominator --
// a census that quietly drops the references it could not classify is the absorbing arm
// (DESIGN §5), so an unclassifiable reference is its own counted, named class and the process
// exits non-zero if the sum does not close.
//
// THE GROUNDING IS NOT THIS INSTRUMENT'S CHOICE. DeclarationExposureGrounding is a PARAMETER of
// the modeled exposure authority, and for a WHOLE-GRAPH type-reference binding context the
// modeled producer -- gunbc.type_reference_binding_context
// build_graph_type_reference_binding_inputs -- declares NamespaceStructuralRootExposure. That is
// the primary column here. The other two groundings are reported beside it as a SENSITIVITY
// reading, because the cross-module visibility question is exactly what the cut turns on and an
// instrument that showed one column would pre-decide it.
//
// THE ROW PROJECTION IS A RE-DERIVATION AND SAYS SO. `inputs_for_module` below transposes one
// module's transport into the three input row lists. That transposition is already modeled twice
// in .dag (gunbc.type_reference_binding_context occurrence_binding_inputs_from_module_transport
// and v1.gunbc.occurrence_binding_parser_walk occurrence_binding_inputs_from_transport), and this
// is a third copy in the host -- a DESIGN section 3 defect, admitted rather than hidden, because
// neither .dag module is on the stage0 emission roster and a host instrument cannot call an
// unemitted authority. It delegates the one fact that is not transposition -- exposure -- to the
// single authority std.occurrence_binding_candidates declaration_exposure_from_containment, so
// the duplicated part carries no decision. DISSOLVE-ON: gunbc.type_reference_binding_context is
// emitted into stage0 and this binary calls build_graph_type_reference_binding_inputs directly.
//
// IT IS A TWO-READER COMPARISON, WHICH IS NOT CORRECTNESS EVIDENCE. Agreement between X and Y
// says the two readers concur, never that either is right -- the failure class
// gunbc.recurring_failure_mode disagreement_census_blind_to_agreed_wrong names this
// exactly, with a receipt from this same subject area. Read OldAndNewAgree as "the cut does not
// move this occurrence", never as "this occurrence binds correctly today".
//
// RUN IT IN ONE REMOTE DISPATCH (runners are amd64; a binary built there will not execute in an
// arm64 session):
//
//   ctrl-build --remote -- bash -lc \
//     'cargo build --release -p v1-compiler --bin type_occurrence_binding_census \
//      && ./target/release/type_occurrence_binding_census dag'
//
// Roots are argv (default: the DAG_PARSE_SWEEP_ROOTS roster). USE A DISCRIMINATING CONTROL: a
// census over a corpus it never read reports the same clean sum as one that read everything, so
// confirm the printed denominator moves when a root is added or removed.

use std::collections::HashMap;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::std_occurrence_binding_candidates::{
    declaration_exposure_from_containment, occurrence_candidate_index_build,
    resolve_reference_via_structural_candidates, AuthoredOrderRow, DeclarationExposureGrounding,
    DeclarationExposureRow, OccurrenceBindingCandidateInputs, OccurrenceCandidateIndex,
    OccurrenceCandidateIndexBuild, OccurrenceModulePathRow, ReferenceBindingProjection,
};
use v1_compiler::std_occurrence_identity::{
    NodeOccurrenceIdentity, OccurrenceCategory, OccurrenceIndex, OccurrenceIndexEntry,
    OccurrenceTransport, ReferenceOccurrence,
};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_infer_env::{
    lookup_type_by_name, resolved_node_is_kernel_identity_for_name,
};
use v1_compiler::v1_compiler_parse::{parse_with_table, parse_with_table_ready_module_path};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table};

/// One census class. `key` is the printed identity; classes are declared here rather than
/// minted at the match site so the closing sum is over a fixed roster.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ClassKey(String);

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("type_occurrence_binding_census: {msg}");
    ExitCode::from(1)
}

fn collect_dag_files(
    workspace: &std::path::Path,
    root: &str,
) -> Result<Vec<std::path::PathBuf>, String> {
    let root_dir = workspace.join(root);
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    let mut stack = vec![root_dir.clone()];
    while let Some(dir) = stack.pop() {
        let read_dir =
            std::fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for entry in read_dir {
            let entry = entry.map_err(|e| format!("read_dir entry in {}: {e}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                // tests/fixtures holds deliberately malformed parser inputs; they are not
                // production type occurrences. Same exclusion the parse sweep applies.
                if path.file_name().map(|n| n == "fixtures").unwrap_or(false)
                    && dir.file_name().map(|n| n == "tests").unwrap_or(false)
                {
                    continue;
                }
                stack.push(path);
            } else if path.extension().map(|ext| ext == "dag").unwrap_or(false) {
                out.push(path);
            }
        }
    }
    if out.is_empty() {
        // An empty walk reports the same clean sum as a complete one (DESIGN §5).
        return Err(format!("root {root} contributed no .dag files"));
    }
    out.sort();
    Ok(out)
}

/// Y inputs for one module's own transport, under one declared grounding. Module paths and
/// authored order come from the parser's index entries; exposure is delegated, unchanged, to
/// the single exposure authority.
/// The three input row lists one module contributes, named because the triple is the shape
/// `OccurrenceBindingCandidateInputs` is assembled from and a bare tuple of three vectors says
/// nothing about which is which.
type ModuleInputRows = (
    Vec<Rc<OccurrenceModulePathRow>>,
    Vec<Rc<DeclarationExposureRow>>,
    Vec<Rc<AuthoredOrderRow>>,
);

fn inputs_for_module(
    module_path: &str,
    transport: &OccurrenceTransport,
    grounding: DeclarationExposureGrounding,
) -> ModuleInputRows {
    let mut module_paths = Vec::new();
    let mut order = Vec::new();
    for entry in transport.index.entries.iter() {
        module_paths.push(Rc::new(OccurrenceModulePathRow {
            occurrence: entry.projection.occurrence,
            module_path: module_path.to_string(),
        }));
        order.push(Rc::new(AuthoredOrderRow {
            occurrence: entry.projection.occurrence,
            ordinal: v1_compiler::std_occurrence_identity::AuthoredTokenOrdinal {
                value: entry.projection.diagnostic_span.start,
            },
        }));
    }
    let mut exposures = Vec::new();
    for declaration in transport.declarations.iter() {
        exposures.push(Rc::new(DeclarationExposureRow {
            occurrence: declaration.occurrence,
            exposure: declaration_exposure_from_containment(
                module_path.to_string(),
                declaration.containment.clone(),
                grounding,
            ),
        }));
    }
    (module_paths, exposures, order)
}

/// What X answered for one reference, at declaration-identity grain.
enum XAnswer {
    /// X bound the reference to an authored declaration carrying this occurrence id.
    Declaration(i64),
    /// X bound the reference to the kernel identity installed for this spelling. The kernel
    /// node is synthetic: it has no authored occurrence, so it is NOT comparable to a Y
    /// declaration id and must never be netted against one.
    Kernel,
    /// X bound the reference to a node with no authored occurrence identity that is not the
    /// kernel identity for this spelling (a normalized/expanded node). Counted separately
    /// rather than folded into Kernel, because the two have different owners.
    SyntheticNonKernel,
    Unresolved,
}

fn x_answer(env: &Rc<v1_compiler::v1_compiler_infer_env::TypeEnv>, name: &str) -> XAnswer {
    match lookup_type_by_name(env.clone(), name.to_string()) {
        None => XAnswer::Unresolved,
        Some(node) => match &*node.occurrence_identity {
            NodeOccurrenceIdentity::OccurrenceMinted { id } => XAnswer::Declaration(id.value),
            NodeOccurrenceIdentity::OccurrenceProjected { id, .. } => {
                XAnswer::Declaration(id.value)
            }
            NodeOccurrenceIdentity::OccurrenceSynthetic => {
                if resolved_node_is_kernel_identity_for_name(node.clone(), name.to_string()) {
                    XAnswer::Kernel
                } else {
                    XAnswer::SyntheticNonKernel
                }
            }
        },
    }
}

/// What Y answered, at declaration-identity grain. Every refusal arm keeps its own name.
enum YAnswer {
    Declaration(i64),
    Unbound,
    Ambiguous,
    Refused(&'static str),
}

fn y_answer(index: &Rc<OccurrenceCandidateIndex>, reference: &Rc<ReferenceOccurrence>) -> YAnswer {
    match &*resolve_reference_via_structural_candidates(index.clone(), reference.clone()) {
        ReferenceBindingProjection::ReferenceBindingProjectionBound { provider } => {
            YAnswer::Declaration(provider.declaration_occurrence.value)
        }
        ReferenceBindingProjection::ReferenceBindingProjectionUnbound { .. } => YAnswer::Unbound,
        ReferenceBindingProjection::ReferenceBindingProjectionAmbiguous { .. } => {
            YAnswer::Ambiguous
        }
        ReferenceBindingProjection::ReferenceBindingProjectionTransportRefused { .. } => {
            YAnswer::Refused("TransportRefused")
        }
        ReferenceBindingProjection::ReferenceBindingProjectionModulePathRefused { .. } => {
            YAnswer::Refused("ModulePathRefused")
        }
        ReferenceBindingProjection::ReferenceBindingProjectionExposureRefused { .. } => {
            YAnswer::Refused("ExposureRefused")
        }
        ReferenceBindingProjection::ReferenceBindingProjectionAuthoredOrderRefused { .. } => {
            YAnswer::Refused("AuthoredOrderRefused")
        }
        ReferenceBindingProjection::ReferenceBindingProjectionDeclarationBucketRefused {
            ..
        } => YAnswer::Refused("DeclarationBucketRefused"),
        ReferenceBindingProjection::ReferenceBindingProjectionModulePathMissing { .. } => {
            YAnswer::Refused("ModulePathMissing")
        }
        ReferenceBindingProjection::ReferenceBindingProjectionWrongCategory { .. } => {
            YAnswer::Refused("WrongCategory")
        }
    }
}

/// WHAT THIS CENSUS DOES NOT MEASURE, printed WITH the numbers rather than filed beside them.
/// A disclosure that lives only in a report or a message is not attached to the figure a later
/// reader quotes, and the figures here are exactly the kind that get quoted.
fn print_reading_disclosures() {
    println!("\nREAD THESE WITH THE NUMBERS ABOVE:");
    println!(
        "  X COLUMN IS THE POST-REWIRE PERSISTED ENV -- the one EMISSION reads. \
v1.compiler.infer rewire_type_env_import_str_binding_identity rewrites module envs after \
build_type_env; PRE-REWIRE INFERENCE ANSWERS ARE NOT MEASURED HERE. The two once disagreed on \
this exact subject (139 of 146 E0308 rows, frontier receipt 1), so 'they agree now' is an \
assumption this instrument does not test."
    );
    println!(
        "  THIS IS A TWO-READER COMPARISON, NOT CORRECTNESS EVIDENCE \
(gunbc.recurring_failure_mode disagreement_census_blind_to_agreed_wrong). OldAndNewAgree means \
THE CUT DOES NOT MOVE THIS OCCURRENCE -- never that the occurrence binds correctly today. Both \
readers wrong together scores as agreement and is invisible here by construction."
    );
    println!(
        "  UNDECIDED IS A FINDING, NOT NOISE. Every NewRefused_* arm and both Unclassifiable_* \
classes are part of the denominator. Do not take a ratio over the total without naming the \
undecided fraction beside it (gunbc.recurring_failure_mode: a census with an unresolvable \
fraction of its population reports a denominator it does not have)."
    );
    println!();
}

fn classify(x: &XAnswer, y: &YAnswer) -> ClassKey {
    let key = match (x, y) {
        (XAnswer::Declaration(a), YAnswer::Declaration(b)) if a == b => "OldAndNewAgree",
        (XAnswer::Declaration(_), YAnswer::Declaration(_)) => "OldAndNewDisagree",
        (XAnswer::Declaration(_), YAnswer::Unbound) => "OldBinds_NewUnresolved",
        (XAnswer::Declaration(_), YAnswer::Ambiguous) => "OldBinds_NewAmbiguous",
        (XAnswer::Unresolved, YAnswer::Declaration(_)) => "OldUnresolved_NewBinds",
        (XAnswer::Unresolved, YAnswer::Unbound) => "OldUnresolved_NewUnresolved",
        (XAnswer::Unresolved, YAnswer::Ambiguous) => "OldUnresolved_NewAmbiguous",
        (XAnswer::Kernel, YAnswer::Declaration(_)) => "OldKernel_NewBinds",
        (XAnswer::Kernel, YAnswer::Unbound) => "OldKernel_NewUnresolved",
        (XAnswer::Kernel, YAnswer::Ambiguous) => "OldKernel_NewAmbiguous",
        (XAnswer::SyntheticNonKernel, YAnswer::Declaration(_)) => "OldSynthetic_NewBinds",
        (XAnswer::SyntheticNonKernel, YAnswer::Unbound) => "OldSynthetic_NewUnresolved",
        (XAnswer::SyntheticNonKernel, YAnswer::Ambiguous) => "OldSynthetic_NewAmbiguous",
        (_, YAnswer::Refused(arm)) => return ClassKey(format!("NewRefused_{arm}")),
    };
    ClassKey(key.to_string())
}

struct Sample {
    module: String,
    name: String,
    span_start: i64,
}

/// THE DENOMINATOR, MEASURED WITHOUT A COMPILE. Parsing is per file and its peak memory is one
/// file, so this mode enumerates the COMPLETE production TypeOccurrence population over any root
/// set -- including the population no single whole-corpus `compile_to_resolved` can reach,
/// because that compile is OOM-killed on this corpus (measured: `src/v2` alone, 1320 files,
/// killed at 93s during reconcile with signal 9).
///
/// IT EXISTS SO THE COMPARISON MODE CANNOT QUIETLY BECOME ITS OWN DENOMINATOR. The comparison
/// needs a typed graph and therefore a compile, so it is bounded to what fits; this mode is not,
/// so the fraction of the population a comparison run actually covered is a measured quantity
/// rather than an assumption. A census whose denominator is "what I was able to compile" is the
/// absorbing arm (DESIGN section 5), and the roster already carries the class this defends
/// against: gunbc.recurring_failure_mode a census with an unresolvable fraction of its
/// population reports a denominator it does not have.
fn run_denominator(workspace: &std::path::Path, roots: &[String]) -> ExitCode {
    let mut files = 0usize;
    let mut parse_refused = 0usize;
    let mut type_refs = 0usize;
    let mut type_decls = 0usize;
    let mut qualified_type_refs = 0usize;
    let mut by_category: HashMap<&'static str, usize> = HashMap::new();
    let mut per_root: Vec<(String, usize, usize)> = Vec::new();

    for root in roots {
        let paths = match collect_dag_files(workspace, root) {
            Ok(p) => p,
            Err(e) => return fail(e),
        };
        let root_files = paths.len();
        let mut root_type_refs = 0usize;
        for path in paths {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(&path)
                .display()
                .to_string();
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => return fail(format!("read {}: {e}", path.display())),
            };
            files += 1;
            let index = build_newline_index(rel.clone(), content.clone());
            let source_indices = v1_compiler::v1_rt::rc_map_insert(
                v1_compiler::v1_rt::rc_empty_map::<
                    String,
                    Rc<v1_compiler::v1_std_core::NewlineIndex>,
                >(),
                rel.clone(),
                index,
            );
            let parsed = parse_with_table(
                tokenize(content, rel.clone()),
                source_indices,
                empty_intern_table(),
            );
            // A file that does not parse contributes NO occurrences and is counted as its own
            // class: an unparsed file and a file with no type references must not share a number.
            if parse_with_table_ready_module_path(parsed.clone()).is_none() {
                parse_refused += 1;
                continue;
            }
            let transport = parsed.occurrence_transport.clone();
            let names: HashMap<i64, String> = transport
                .index
                .entries
                .iter()
                .map(|e| {
                    (
                        e.projection.occurrence.value,
                        e.projection.authored_name.clone(),
                    )
                })
                .collect();
            for r in transport.references.iter() {
                let k = category_label(&r.category);
                *by_category.entry(k).or_default() += 1;
                if matches!(r.category, OccurrenceCategory::TypeOccurrence) {
                    type_refs += 1;
                    root_type_refs += 1;
                    if names
                        .get(&r.occurrence.value)
                        .map(|n| n.contains('.'))
                        .unwrap_or(false)
                    {
                        qualified_type_refs += 1;
                    }
                }
            }
            for d in transport.declarations.iter() {
                if matches!(d.category, OccurrenceCategory::TypeOccurrence) {
                    type_decls += 1;
                }
            }
        }
        per_root.push((root.clone(), root_files, root_type_refs));
    }

    println!("MODE denominator (parse only, no compile)");
    for (root, n, refs) in per_root.iter() {
        println!("  root {root}: files={n} type_occurrence_references={refs}");
    }
    let mut cats: Vec<_> = by_category.into_iter().collect();
    cats.sort();
    println!("  files={files} parse_refused={parse_refused}");
    println!("  reference_occurrences_by_category={cats:?}");
    println!("  type_occurrence_declarations={type_decls}");
    println!(
        "DENOMINATOR type_occurrence_references={type_refs} (of which qualified spelling: {qualified_type_refs})"
    );
    if parse_refused > 0 {
        println!("  NOTE: {parse_refused} file(s) did not parse and contributed zero occurrences; that is a separate finding, not a zero.");
    }
    print_reading_disclosures();
    ExitCode::SUCCESS
}

fn category_label(c: &OccurrenceCategory) -> &'static str {
    match c {
        OccurrenceCategory::LexicalValueOccurrence => "LexicalValue",
        OccurrenceCategory::TypeOccurrence => "Type",
        OccurrenceCategory::CallableOccurrence => "Callable",
        OccurrenceCategory::ConstructorOccurrence => "Constructor",
        OccurrenceCategory::NamespaceSegmentOccurrence => "NamespaceSegment",
        OccurrenceCategory::FieldOccurrence => "Field",
        OccurrenceCategory::MethodOccurrence => "Method",
    }
}

fn main() -> ExitCode {
    let workspace = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => return fail(format!("current_dir: {e}")),
    };
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let denominator_only = args.first().map(|a| a == "--denominator").unwrap_or(false);
    if denominator_only {
        args.remove(0);
    }
    let roots: Vec<String> = if args.is_empty() {
        v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS
            .iter()
            .map(|r| r.to_string())
            .collect()
    } else {
        args
    };

    if denominator_only {
        return run_denominator(&workspace, &roots);
    }

    let mut sources: Vec<Rc<SourceFile>> = Vec::new();
    for root in &roots {
        let files = match collect_dag_files(&workspace, root) {
            Ok(f) => f,
            Err(e) => return fail(e),
        };
        for path in files {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => return fail(format!("read {}: {e}", path.display())),
            };
            sources.push(Rc::new(SourceFile {
                path: path
                    .strip_prefix(&workspace)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                content,
            }));
        }
    }
    println!("census roots={:?} source_files={}", roots, sources.len());

    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let Some(graph) = resolved.graph.clone() else {
        return fail("frontend produced no graph; census cannot enumerate its denominator");
    };
    println!(
        "typed_modules={} frontend_diagnostics={}",
        graph.modules.len(),
        resolved.diagnostics.len()
    );

    // ONE program-wide transport. The frontend threads a single occurrence allocator across all
    // files (v1.compiler.compile front_end_sources), so occurrence ids are already globally
    // unique and no rekey is performed here -- a rekey would mint a second identity authority.
    let mut all_entries: Vec<Rc<OccurrenceIndexEntry>> = Vec::new();
    let mut all_declarations = Vec::new();
    let mut all_references = Vec::new();
    // reference occurrence value -> owning module path (from the module whose transport carried
    // it), so the X side is asked of the SAME module's environment production uses.
    let mut module_of_reference: HashMap<i64, String> = HashMap::new();
    let mut env_of_module: HashMap<String, Rc<v1_compiler::v1_compiler_infer_env::TypeEnv>> =
        HashMap::new();
    let mut modules_without_transport = 0usize;

    // Primary first: the grounding the modeled whole-graph producer declares.
    let mut grounding_inputs: Vec<(DeclarationExposureGrounding, (Vec<_>, Vec<_>, Vec<_>))> = vec![
        (
            DeclarationExposureGrounding::NamespaceStructuralRootExposure,
            (Vec::new(), Vec::new(), Vec::new()),
        ),
        (
            DeclarationExposureGrounding::ModuleLocalMemberExposure,
            (Vec::new(), Vec::new(), Vec::new()),
        ),
        (
            DeclarationExposureGrounding::CrossFileProviderExportedExposure,
            (Vec::new(), Vec::new(), Vec::new()),
        ),
    ];

    for module in graph.modules.iter() {
        let module_path = module.type_env.module_path.clone();
        env_of_module.insert(module_path.clone(), module.type_env.clone());
        let Some(transport) = module.occurrence_transport.clone() else {
            modules_without_transport += 1;
            continue;
        };
        for entry in transport.index.entries.iter() {
            all_entries.push(entry.clone());
        }
        for declaration in transport.declarations.iter() {
            all_declarations.push(declaration.clone());
        }
        for reference in transport.references.iter() {
            module_of_reference.insert(reference.occurrence.value, module_path.clone());
            all_references.push(reference.clone());
        }
        for (grounding, acc) in grounding_inputs.iter_mut() {
            let (m, e, o) = inputs_for_module(&module_path, &transport, *grounding);
            acc.0.extend(m);
            acc.1.extend(e);
            acc.2.extend(o);
        }
    }

    let transport = Rc::new(OccurrenceTransport {
        index: Rc::new(OccurrenceIndex {
            entries: Rc::new(all_entries.into()),
        }),
        declarations: Rc::new(all_declarations.into()),
        references: Rc::new(all_references.clone().into()),
    });

    // THE DENOMINATOR. Every TypeOccurrence reference in the program-wide transport, and the
    // count of those whose owning module carries no TypeEnv is reported rather than skipped.
    let type_references: Vec<Rc<ReferenceOccurrence>> = all_references
        .iter()
        .filter(|r| matches!(r.category, OccurrenceCategory::TypeOccurrence))
        .cloned()
        .collect();
    let by_category = {
        let mut m: HashMap<&'static str, usize> = HashMap::new();
        for r in all_references.iter() {
            let k = category_label(&r.category);
            *m.entry(k).or_default() += 1;
        }
        m
    };
    println!(
        "reference_population={} modules_without_transport={} by_category={:?}",
        all_references.len(),
        modules_without_transport,
        {
            let mut v: Vec<_> = by_category.into_iter().collect();
            v.sort();
            v
        }
    );
    println!(
        "DENOMINATOR type_occurrence_references={}",
        type_references.len()
    );
    print_reading_disclosures();

    let name_of: HashMap<i64, (String, i64)> = transport
        .index
        .entries
        .iter()
        .map(|e| {
            (
                e.projection.occurrence.value,
                (
                    e.projection.authored_name.clone(),
                    e.projection.diagnostic_span.start,
                ),
            )
        })
        .collect();

    let mut exit = ExitCode::SUCCESS;
    for (grounding, (module_paths, exposures, order)) in grounding_inputs.into_iter() {
        let label = match grounding {
            DeclarationExposureGrounding::ModuleLocalMemberExposure => "ModuleLocalMemberExposure",
            DeclarationExposureGrounding::CrossFileProviderExportedExposure => {
                "CrossFileProviderExportedExposure"
            }
            DeclarationExposureGrounding::NamespaceStructuralRootExposure => {
                "NamespaceStructuralRootExposure"
            }
        };
        let inputs = Rc::new(OccurrenceBindingCandidateInputs {
            module_paths: Rc::new(module_paths.into()),
            exposure_rows: Rc::new(exposures.into()),
            authored_order_rows: Rc::new(order.into()),
        });
        // ONE index build for the whole population. resolve_type_reference_containment_binding
        // rebuilds the index per reference by construction, which is O(population) per
        // reference; the census consumes the same delegate it calls with the index built once,
        // and pre-filters the category the typed entry point refuses.
        let index = match &*occurrence_candidate_index_build(transport.clone(), inputs.clone()) {
            OccurrenceCandidateIndexBuild::OccurrenceCandidateIndexReady { index } => index.clone(),
            other => {
                println!("[{label}] INDEX BUILD REFUSED: {other:?}");
                exit = ExitCode::from(1);
                continue;
            }
        };

        let mut counts: HashMap<ClassKey, usize> = HashMap::new();
        let mut samples: HashMap<ClassKey, Vec<Sample>> = HashMap::new();
        let mut qualified_counts: HashMap<ClassKey, usize> = HashMap::new();
        let mut unclassifiable_missing_env = 0usize;
        let mut unclassifiable_missing_name = 0usize;

        for reference in type_references.iter() {
            let Some((name, span_start)) = name_of.get(&reference.occurrence.value).cloned() else {
                unclassifiable_missing_name += 1;
                continue;
            };
            let Some(module_path) = module_of_reference.get(&reference.occurrence.value) else {
                unclassifiable_missing_env += 1;
                continue;
            };
            let Some(env) = env_of_module.get(module_path) else {
                unclassifiable_missing_env += 1;
                continue;
            };
            let x = x_answer(env, &name);
            let y = y_answer(&index, reference);
            let class = classify(&x, &y);
            *counts.entry(class.clone()).or_default() += 1;
            if name.contains('.') {
                *qualified_counts.entry(class.clone()).or_default() += 1;
            }
            let bucket = samples.entry(class).or_default();
            if bucket.len() < 5 {
                bucket.push(Sample {
                    module: module_path.clone(),
                    name: name.clone(),
                    span_start,
                });
            }
        }

        println!("\n=== grounding {label} ===");
        let mut rows: Vec<(ClassKey, usize)> = counts.clone().into_iter().collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut classified = 0usize;
        for (class, n) in rows.iter() {
            classified += n;
            let q = qualified_counts.get(class).copied().unwrap_or(0);
            println!("{:>8}  {:<34} (qualified spelling: {})", n, class.0, q);
            for s in samples.get(class).into_iter().flatten() {
                println!("           e.g. {}::{} @{}", s.module, s.name, s.span_start);
            }
        }
        println!(
            "{:>8}  {:<34}",
            unclassifiable_missing_name, "Unclassifiable_NoIndexEntry"
        );
        println!(
            "{:>8}  {:<34}",
            unclassifiable_missing_env, "Unclassifiable_NoModuleEnv"
        );
        let total = classified + unclassifiable_missing_name + unclassifiable_missing_env;
        // THE SUM CLOSES OR THE CENSUS REFUSES. A partition that does not add up to the
        // denominator has silently narrowed its own subject.
        if total != type_references.len() {
            println!(
                "[{label}] REFUSED: class total {total} != denominator {}",
                type_references.len()
            );
            exit = ExitCode::from(1);
        } else {
            println!("[{label}] partition closes: {total} == denominator");
        }
    }

    exit
}

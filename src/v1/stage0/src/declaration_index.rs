//! ONE per-module declaration index, built where each module is already parsed.
//!
//! WHY THIS EXISTS, and why one module rather than three checks. DESIGN carries two
//! next-rung triggers naming the same construction. §6, on the deleted inert-lens and
//! construction-justification censuses: "an authorship fact belongs on the module's own
//! declaration, checked at ingestion where the module is parsed anyway — one module's facts
//! from one module's source — rather than reconstructed corpus-wide by a consumer that
//! wanted something else." §3's cited-symbol rung-drop row, on the deleted
//! `--required-cited-symbol` job: "this row retires when the citation wall is re-derived
//! where the operator's own framing puts it — *you would just make them a normal compiler
//! error* — checked at ingestion, on the module whose source carries the citation, from
//! that module's own text, rather than reconstructed corpus-wide by a second job. That is
//! the same next-rung trigger §6 already names for module authorship facts, and the two
//! should land together rather than each rebuilding a corpus walk."
//!
//! THE COST SHAPE BOTH TRIGGERS COMPLAIN ABOUT. Every mechanism wanting a per-module fact
//! acquired the whole corpus: `decl_facts(roots)` re-parses every `.dag` file into a FLAT
//! `Vec<DeclFact>`, `module_declaration_facts(roots)` walks them again for a flat `Vec` of
//! module rows, and the cited-symbol resolver answers each reference by a LINEAR SCAN over
//! both. §6's cost-shape defect exactly — unit of computation the world, unit of fact one
//! module — and why the checks kept being authored as separate corpus-wide jobs.
//!
//! WHAT IS CONSTRUCTED HERE. `run_dag_parse_sweep` already parses every `.dag` file under
//! `DAG_PARSE_SWEEP_ROOTS`, once, in parallel, on every required run — and threw the tree
//! away. This module turns that parse into one `ModuleDeclarationRecord` per module: what it
//! DECLARES, what it CLAIMS from other modules (import members), what it CITES
//! (`DeclarationRef` literals in its own text), and its authorship fact. The integrity
//! questions are answered from that single index by keyed lookup, not linear scan:
//!
//!   1. import-member claim integrity — `import m { X }` where `m` declares no `X`
//!   2. the cited-symbol wall — an authored `DeclarationRef` naming a symbol that
//!      does not resolve (§3: cite the symbol, not the position)
//!   3. module authorship — a top-level lens with no `construction_justification`
//!   4. cited-authority reachability — a non-fixture module cited as a fact's home by
//!      another non-fixture module, while no authored import edge reaches that home
//!
//! WHAT THIS IS NOT. Not a widening of the required floor's source roots; the objection
//! `gunbc.ci_layer_roots` `v1_dead_witness_tree_triage_receipt_remainder` raises against that
//! cannot reach it, as it cannot reach the parse sweep it rides on: NOTHING HERE RESOLVES
//! ACROSS FILES IN THE COMPILER'S SENSE. Each record derives from one file's own tree; the
//! index maps the module path a file DECLARES to that file's facts. Two roots colliding on a
//! last segment re-bind nothing — no bare reference is resolved, only fully qualified module
//! paths are looked up, and a module path is unique or a duplicate the index reports.
//!
//! WHY EVERY ROSTER ROW NAMES ITS CITING MODULE, AND NOT ONLY ITS TARGET. The three suppression
//! rosters below used to be keyed `(module, decl, field)` — the TARGET — so one row exempted
//! EVERY citation of that target, corpus-wide, while it stood: an open wall in a direction
//! nothing could observe. A patch could author a BRAND NEW dangling `DeclarationRef` naming any
//! enrolled target, from any module, and the wall would silently decline to judge it — decidable
//! from the patch alone (new site, no row named it), so rot admitted by the mechanism built to
//! refuse rot.
//!
//! IT WAS OCCUPIED, NOT MERELY REACHABLE, which settled the grain. Measured over the live corpus
//! through `DAG_PARSE_SWEEP_ROOTS`, the 70 target-keyed rows covered 87 refusing sites, and seven
//! targets were cited from more than one module: `gunbc.host_effect` `host_effect_apply` from
//! three (`extdeps.github.actions_runner`, `gunbc.executor_privileged_operation`,
//! `gunbc.runner_slot_provision`), `std.bytes` `builtin_function_registry` from three,
//! `extdeps.network.mac` `parse_mac_address` from two (`extdeps.dhcp.v4` and a witness), and
//! four more from two apiece. (That measurement stands; the `parse_mac_address` example has since
//! been DISCHARGED, not falsified — the module landed, the witness citations resolve and their
//! rows are deleted, and `extdeps.dhcp.v4` stopped citing it when its frontier trigger was
//! re-pointed off the artifact onto the capability. Noted so a reader does not grep for a
//! two-site collision that is gone.) Every extra site was suppressed by a row authored about a
//! different module.
//!
//! THE ROSTERS ARE RE-DERIVED FROM THAT MEASUREMENT, AND THE FIRST DERIVATION WAS TAKEN OVER THE
//! WRONG DENOMINATOR — the same class this module keeps catching. The sweep's roots are
//! `src/v1`, `dag` and `src/v2`; the first measurement used only the last two, so five sites in
//! modules the narrow walk never read were absent from the rosters and the required run refused
//! them. A roster derived from a subset of its subject is not smaller, it is wrong.
//!
//! So a row is `(citing_module, in_declaration, module, decl, field)` and exempts THE SITE THAT
//! AUTHORED IT. Both inverse arms read that same identity: a suppression arm and a staleness
//! arm keyed differently is the desynchronization `corpus_findings` carries a receipt for.
//!
//! THE DECLARATION IN THE ROW IS THE SECOND HALF OF THE SAME REPAIR, A DISCLOSED RESIDUE BEFORE
//! A CLOSED ONE (review 56227). Keyed on the citing MODULE alone, a row still covered every
//! citation of that target ANYWHERE IN THAT MODULE, so a new dangling citation BESIDE an
//! enrolled one was suppressed — the same fail-open one level in. A residue with an available
//! identity is not a residue: `record_from_module` already iterates top-level items, so the
//! enclosing declaration's name costs one string at extraction. It is a NAME, reachable from the
//! containment tree, not the positional citation DESIGN §3 forbids — an offset would be finer
//! and would rot on any edit above the line.
//!
//! WHAT REMAINS (a closed residue is not a total one): two citations of ONE target inside ONE
//! declaration still share a row. Only a position separates those, and a position is what this
//! grain exists not to be, so this is a ceiling, not a stall — the next rung is a citation
//! carrying an occurrence ordinal within its declaration, which the ingestion record could hold
//! but no measured site needs today.
//!
//! It is also not the compiler's own name resolution. `v1.03_resolve` refuses `MissingExport`
//! for an import member inside a COMPILE CLOSURE; this index answers the same question over the
//! whole authored corpus — DESIGN's 2026-08-25 row records `gunbc.auth.credentials` standing on
//! main with four hard errors because NO CLOSURE REACHES IT. An orphan module's import claims
//! are checked here and nowhere else.

// CLIPPY ROSTER -- 2 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::empty_line_after_doc_comments,  // 1
    clippy::items_after_test_module,  // 1
)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

use crate::std_occurrence_identity::{OccurrenceCategory, OccurrenceTransport};
use crate::v1_rt::VecCompat;
use crate::v1_std_core::{
    authored_name_at, expr_literal_string_optional, import_is_all, import_specific_names_at,
    module_imports, module_items, Connective, ExprData, MatchPattern, NewlineIndex, Node,
    SourceSpan,
};

/// A span COPIED OUT of the parse tree rather than referenced into it.
///
/// The sweep parses each file on its own thread and hands the record across a thread
/// boundary; the `Rc`-shaped parse tree cannot cross one. Copying the two fields a location
/// needs loses nothing — holding the whole tree alive to carry an offset would keep the
/// corpus resident for a `usize`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceLocation {
    pub file: String,
    pub offset: i64,
}

fn source_location_of(span: &Rc<SourceSpan>) -> SourceLocation {
    SourceLocation {
        file: span.file.clone(),
        offset: span.start,
    }
}

/// The fixed-name declaration every top-level lens module must carry (DESIGN §6).
pub const CONSTRUCTION_JUSTIFICATION_DECL: &str = "construction_justification";

/// The record type name whose literals are citations. Both authoring forms are
/// captured: the record literal and the `std.decl_ref` constructors over it.
const DECLARATION_REF_TYPE: &str = "DeclarationRef";
const DECL_REF_FN: &str = "decl_ref";
const DECL_FIELD_REF_FN: &str = "decl_field_ref";

/// One authored citation, located in the module whose source carries it.
#[derive(Debug, Clone, PartialEq)]
pub struct CitedSymbol {
    /// The top-level declaration in the CITING module whose subtree carries this citation.
    ///
    /// The finest STABLE site identity an ingestion record can hold. A byte offset would be
    /// finer and is refused: DESIGN §3 forbids a positional citation because it rots on any
    /// edit above the line, and a roster keyed on one would go stale with nobody touching
    /// either end. A declaration name is reachable from the containment tree the namespace
    /// authority walks — the same kind of identity a citation is.
    pub in_declaration: String,
    pub module_path: String,
    pub decl_name: String,
    /// `Some(field)` for a `NamedField` citation, `None` for `WholeDeclaration`.
    pub field: Option<String>,
    pub location: SourceLocation,
}

/// One `import m { A, B }` claim, with each member located at its own name.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportClaim {
    pub target: String,
    pub members: Vec<(String, SourceLocation)>,
    pub location: SourceLocation,
}

/// One module's facts, derived from that one module's source.
#[derive(Debug, Clone)]
pub struct ModuleDeclarationRecord {
    pub module_path: String,
    pub rel_path: String,
    /// Top-level item names authored in this module.
    pub declared: BTreeSet<String>,
    /// Coproduct variant names, which the import surface also exports.
    pub variants: BTreeSet<String>,
    /// Names this module re-exports because its own imports list them. Kept apart from
    /// `declared`: an import member may legitimately be a re-export, while a CITATION naming
    /// a re-export names the wrong authority (§3 — a fact's home is the declaring module).
    pub reexported: BTreeSet<String>,
    /// Declaration name -> the field names reachable one level inside it. Answers
    /// `NamedField` citations without a second pass.
    pub decl_fields: BTreeMap<String, BTreeSet<String>>,
    pub imports: Vec<ImportClaim>,
    pub cited: Vec<CitedSymbol>,
    /// Callee spellings authored in this module. Used only to partition cited authorities by
    /// whether the citing module also calls the cited declaration; it is not a resolver.
    pub called: BTreeSet<String>,
    /// The authored NAME OCCURRENCES in this module's own tree that name something the module
    /// reaches, paired with the top-level declaration whose subtree carries it:
    /// `(in_declaration, spelling)`.
    ///
    /// WHY A NAME OCCURRENCE AND NOT A SEMANTIC REFERENCE. Derived by walking the parsed tree
    /// — parse-then-derive, as DESIGN prescribes after the raw-text scanner family was ruled a
    /// heuristic — but NOT a resolution: a parameter name and a `let` binder land here beside
    /// a genuine reference, because telling those apart is the resolver's job and this index
    /// resolves nothing across files.
    ///
    /// THE OVER-COLLECTION IS BOUNDED, and the earlier reasoning for leaving it unbounded is
    /// refuted: that it is SYMMETRIC across the two trees the one consumer
    /// (`namespace_wave_admission`) compares, so a spelling denoting nothing on both sides
    /// contributes no delta. A symmetric COLLECTOR does not give a symmetric VERDICT — the
    /// supplier set is a function of the CORPUS, so deleting an unrelated declaration moves it
    /// under every site merely spelling the same word. The measured specimen, the two kinds
    /// now excluded, and the remaining members of the class are on
    /// `collect_reference_occurrences`.
    ///
    /// Dotted spellings are recorded WHOLE as well as by segment, so `v2.std.node.Hash` is
    /// observable as naming the module `v2.std.node`, not only four unrelated segments.
    pub referenced: BTreeSet<(String, String)>,
    /// AUTHORED TYPE REFERENCES, TAKEN FROM THE PARSER'S OWN `OccurrenceTransport` RATHER THAN
    /// RE-DERIVED FROM THE `Node`. A peer of `referenced`, not a widening: different
    /// authorities, different precision, and fusing them would hide which is which.
    ///
    /// `referenced` is this module's lossy walk over the final tree -- it cannot see a type
    /// parked in the `inferred` slot, and over-collects binders and labels. This set is the
    /// parser's answer: `stamp_parsed_inferred` stamps a declared type as
    /// `ParsedOccurrenceReference { TypeOccurrence }` with a minted identity, the authored
    /// spelling, and containment, read back here.
    ///
    /// WHY A PEER AND NOT A MERGE. A Node-reading projection reproducing these entries would be
    /// a second authority for a fact the parser owns exactly -- agreeing today, silently
    /// diverging tomorrow, undetected. A separate field lets a reader tell authored from
    /// reconstructed, and the Node walk can shrink toward zero as positions gain transport
    /// entries.
    ///
    /// Keyed `(enclosing declaration, authored spelling)` like `referenced`, so a consumer
    /// wanting every authored reference takes the union without knowing which channel supplied
    /// which row.
    pub authored_type_references: BTreeSet<(String, String)>,
    pub declares_construction_justification: bool,
    /// Whether this module is a witness or fixture carrier. See `module_is_fixture_carrier`.
    pub is_fixture_carrier: bool,
}

/// The names an `import m { X }` may legitimately claim — the same surface
/// `v1.03_resolve` `get_exported_names` admits, minus the kernel types it appends
/// (those are never authored as import members and are handled by the caller).
pub fn import_surface_has(record: &ModuleDeclarationRecord, name: &str) -> bool {
    record.declared.contains(name)
        || record.variants.contains(name)
        || record.reexported.contains(name)
}

/// A typed, located integrity finding. One variant per QUESTION, never per site, so a
/// reader can tell which wall fired without parsing the message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclarationIntegrityKind {
    /// `import m { X }` and `m` declares, re-exports and varies no `X`.
    ImportMemberAbsent,
    /// An authored `DeclarationRef` whose module path names no module in the corpus.
    CitedModuleAbsent,
    /// An authored `DeclarationRef` whose module exists and declares no such symbol.
    CitedDeclarationAbsent,
    /// A `NamedField` citation whose declaration carries no such field.
    CitedFieldAbsent,
    /// A top-level lens module carrying no `construction_justification` declaration.
    LensAuthorshipAbsent,
    /// Two swept files declaring one module path — the index cannot key on it.
    DuplicateModuleDeclaration,
    /// A `PRE_EXISTING_CITATION_DEBT` row whose citation no longer refuses.
    CitationDebtRowStale,
    /// A `PLANTED_CONTROL_CITATIONS` row whose citation stopped refusing — the control is no
    /// longer discriminating. The inverse reading of the same trigger as the row above.
    PlantedControlNoLongerRefuses,
}

pub fn integrity_kind_label(kind: &DeclarationIntegrityKind) -> &'static str {
    match kind {
        DeclarationIntegrityKind::ImportMemberAbsent => "IMPORT-MEMBER-ABSENT",
        DeclarationIntegrityKind::CitedModuleAbsent => "CITED-MODULE-ABSENT",
        DeclarationIntegrityKind::CitedDeclarationAbsent => "CITED-DECLARATION-ABSENT",
        DeclarationIntegrityKind::CitedFieldAbsent => "CITED-FIELD-ABSENT",
        DeclarationIntegrityKind::LensAuthorshipAbsent => "LENS-AUTHORSHIP-ABSENT",
        DeclarationIntegrityKind::DuplicateModuleDeclaration => "DUPLICATE-MODULE",
        DeclarationIntegrityKind::CitationDebtRowStale => "CITATION-DEBT-ROW-STALE",
        DeclarationIntegrityKind::PlantedControlNoLongerRefuses => "PLANTED-CONTROL-RESOLVES",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeclarationIntegrityFinding {
    pub kind: DeclarationIntegrityKind,
    pub rel_path: String,
    /// Char offset into `rel_path`'s source, rendered as line:col by the caller that
    /// holds the newline index. `None` where the span names another file.
    pub offset: Option<i64>,
    pub message: String,
}

/// Whether a module is a WITNESS OR FIXTURE CARRIER, and therefore whether its authored
/// `DeclarationRef`s are CLAIMS or FIXTURE DATA.
///
/// THE ONE DISTINCTION THE CITATION WALL CANNOT DO WITHOUT, found by measurement. Over the
/// live corpus the wall refuses in two different populations: the rot §3's rule exists to
/// catch (a citation naming a module the floor cut deleted), and DELIBERATELY FALSE TEXT — a
/// witness proving the resolver refuses an absent module must AUTHOR one, so
/// `test.claim.annotation_carrier` cites `extdeps.network.mac` `parse_mac_addres` on purpose,
/// one letter short, and refusing it would refuse the evidence for the wall's own mechanism.
/// Not a leniency carve-out: the difference between a claim and an input, decidable from the
/// carrier's identity.
///
/// NOT AN AUTHORED EXEMPTION LIST — read off the module's own path, the same `_test.dag`
/// suffix `cli_run` `is_test_dag` uses corpus-wide, widened by the `test` namespace segment so
/// a fixture module without the suffix (`test.fixture.decl_facts_reflection.specimens`) lands
/// in the same class. The population is COUNTED, not dropped: `citations_in_fixtures` is
/// reported beside the enrolled count, so a green cannot read as covering what it excluded.
///
/// WHAT IT COSTS: a genuinely stale citation authored inside a witness module is not refused.
/// Strictly better than what it replaces — `decl_facts` did not INDEX test modules at all, so
/// citations INTO them refused spuriously and needed an authored outside-index disposition;
/// here they resolve, and only citations FROM them are unenrolled.
pub fn module_is_fixture_carrier(module_path: &str, rel_path: &str) -> bool {
    rel_path.ends_with("_test.dag") || module_path.split('.').any(|segment| segment == "test")
}

/// Whether a module path is a top-level lens — the population DESIGN §6's authorship
/// obligation covers. A `*_test` module beside a lens is a witness carrier, not a lens.
pub fn is_top_level_lens_module(module_path: &str) -> bool {
    let Some(tail) = module_path.strip_prefix("v2.lens.") else {
        return false;
    };
    !tail.contains('.') && !tail.ends_with("_test")
}

/// Walk one node's whole subtree, visiting every reachable child edge.
fn for_each_node(node: &Rc<Node>, visit: &mut impl FnMut(&Rc<Node>)) {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        visit(node);
        for c in node.children.iter() {
            for_each_node(c, visit);
        }
        for c in node.params.iter() {
            for_each_node(c, visit);
        }
        for c in node.properties.iter() {
            for_each_node(c, visit);
        }
        for c in node.uses.iter() {
            for_each_node(c, visit);
        }
        if let Some(b) = node.body.as_ref() {
            for_each_node(b, visit);
        }
        if let Some(t) = node.transport.as_ref() {
            for_each_node(t, visit);
        }
        if let Some(t) = node.type_annotation.as_ref() {
            for_each_node(t, visit);
        }
    })
}

/// The whole dotted spelling a field-access spine authors, or `None` when the node is not
/// the head of one.
///
/// `a.b.c` parses as receiver `a` under two `ExprFieldAccess` nodes, so an ordinary walk sees
/// the SEGMENTS and not the SPELLING. A module reference is a spelling — `v2.std.node` names a
/// module, `node` alone names nothing — so a segment-only reader cannot see which modules a
/// body reaches. The spine stops at the first receiver that is not a name or field access, so
/// `foo(x).bar` yields nothing rather than a fabricated `foo.bar`.
fn dotted_chain(
    node: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    if !matches!(&*node.expr_data, ExprData::ExprFieldAccess { .. }) {
        return None;
    }
    let field = authored_name_at(source_indices.clone(), node.clone());
    if field.is_empty() {
        return None;
    }
    let receiver = node.children.first()?;
    let prefix = match &*receiver.expr_data {
        ExprData::ExprFieldAccess { .. } => dotted_chain(receiver, source_indices)?,
        ExprData::ExprVar { .. } => {
            let name = authored_name_at(source_indices.clone(), receiver.clone());
            if name.is_empty() {
                return None;
            }
            name
        }
        _ => return None,
    };
    Some(format!("{prefix}.{field}"))
}

fn is_record_literal(node: &Rc<Node>) -> bool {
    matches!(&*node.expr_data, ExprData::ExprRecordLit { .. })
}

fn is_call(node: &Rc<Node>) -> bool {
    matches!(&*node.expr_data, ExprData::ExprCall { .. })
}

/// The value expression of a field-init / argument node. Both shapes are one child.
fn bound_value(node: &Rc<Node>) -> Option<&Rc<Node>> {
    node.children.first()
}

/// A `DeclarationRef { module_path: "..", decl_name: "..", field: .. }` literal.
///
/// A field whose value is NOT a string literal yields no citation — fail-OPEN on purpose: a
/// computed module path is not a citation this index can resolve, and refusing it would refuse
/// a construction the substrate allows. Recorded as a coverage boundary, not a silent skip:
/// `citation_sites` and `resolvable_citations` are reported separately so a green names both
/// denominators.
fn citation_from_record_literal(node: &Rc<Node>, in_declaration: &str) -> Option<CitedSymbol> {
    if node.name != DECLARATION_REF_TYPE || !is_record_literal(node) {
        return None;
    }
    let mut module_path = None;
    let mut decl_name = None;
    let mut field = None;
    for f in node.children.iter() {
        let value = bound_value(f)?;
        match f.name.as_str() {
            "module_path" => module_path = expr_literal_string_optional(value.clone()),
            "decl_name" => decl_name = expr_literal_string_optional(value.clone()),
            "field" => field = named_field_name(value),
            _ => {}
        }
    }
    Some(CitedSymbol {
        in_declaration: in_declaration.to_string(),
        module_path: module_path?,
        decl_name: decl_name?,
        field,
        location: source_location_of(&node.span),
    })
}

/// `NamedField { field_name: "x" }` -> `Some("x")`; `WholeDeclaration` -> `None`.
fn named_field_name(value: &Rc<Node>) -> Option<String> {
    if value.name != "NamedField" {
        return None;
    }
    value
        .children
        .iter()
        .find(|f| f.name == "field_name")
        .and_then(bound_value)
        .and_then(|v| expr_literal_string_optional(v.clone()))
}

/// `decl_ref(m, n)` / `decl_field_ref(m, n, f)` — the constructors `std.decl_ref` owns.
/// Arguments are read positionally OR by name, because both spellings are authored.
fn citation_from_constructor_call(node: &Rc<Node>, in_declaration: &str) -> Option<CitedSymbol> {
    if !is_call(node) {
        return None;
    }
    let with_field = match node.name.as_str() {
        DECL_REF_FN => false,
        DECL_FIELD_REF_FN => true,
        _ => return None,
    };
    let arg = |index: usize, name: &str| -> Option<String> {
        node.children
            .iter()
            .find(|a| a.name == name)
            .or_else(|| node.children.get(index).filter(|a| a.name.is_empty()))
            .and_then(bound_value)
            .and_then(|v| expr_literal_string_optional(v.clone()))
    };
    Some(CitedSymbol {
        in_declaration: in_declaration.to_string(),
        module_path: arg(0, "module_path")?,
        decl_name: arg(1, "decl_name")?,
        field: if with_field {
            Some(arg(2, "field_name")?)
        } else {
            None
        },
        location: source_location_of(&node.span),
    })
}

/// The field names a `NamedField` citation may legitimately name inside a declaration.
///
/// WHY THE DECLARATION'S WHOLE SUBTREE AND NOT ITS DIRECT CHILDREN. The first version read
/// one level down and produced FABRICATED REFUSALS over the live corpus: `extdeps.llm.anthropic`
/// cites `cache_control` of `AnthropicTextBlock`, a field of a VARIANT of a coproduct, two
/// levels down; `std.disposition` `Disposition` `marker` is the same shape; a `data`
/// declaration's fields live inside its initializer, deeper still. A refusal firing because
/// the reader did not descend is worse than no check — its remedy is deleting a correct
/// citation.
///
/// WHAT THIS GIVES UP: the set is the union over the subtree, so a citation naming a field of
/// a DIFFERENT variant of the same coproduct resolves. A real weakening confined to this arm —
/// the module and declaration arms are exact. Next rung: a field lookup descending the
/// declared TYPE rather than the declaration's text, which needs the inferred tree this
/// ingestion walk deliberately does not build.
fn declaration_field_names(
    item: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for_each_node(item, &mut |node| {
        // A record literal's field initializers and a `type`'s declared fields are both named
        // child nodes one step below their parent. BOTH NAME READINGS ARE TAKEN:
        // `make_field_init_node` stamps `.name` directly; a declared field's name is recovered
        // from its ident span as the rest of the frontend does; the two are not
        // interchangeable across node families.
        for child in node.children.iter() {
            if !child.name.is_empty() {
                out.insert(child.name.clone());
            }
            let authored = authored_name_at(source_indices.clone(), child.clone());
            if !authored.is_empty() {
                out.insert(authored);
            }
        }
    });
    out
}

/// One declaration's REFERENCE occurrences, recorded into `out` as `(in_declaration, spelling)`.
///
/// WHY SELECTIVE BY NODE KIND. The first version took every node's authored name unfiltered,
/// arguing over-collection is harmless because SYMMETRIC across the two trees the wave wall
/// compares. REFUTED BY A MEASURED SPECIMEN: symmetry of the COLLECTOR does not give symmetry
/// of the VERDICT, because the supplier set is a function of the corpus, not the site. On
/// gunbc#9106 a witness module deleted a helper `fn live_tree_declined_entries` and kept twelve
/// RECORD FIELD LABELS spelling the same word. The labels never bound to the helper and need no
/// supplier, yet each was collected as a reference, reported base `{that module}` -> head `{}`,
/// and the wall raised twelve `NewUnresolvedness` rows against a correct cut — a delta true
/// about the declaration and false about every site it names.
///
/// THE FIX IS THE SHAPE THE `cited` COLLECTOR ON THE SAME TREE ALREADY USES — decide by node
/// kind, not by name — and exactly two kinds are excluded here:
///
///   * A RECORD LITERAL'S FIELD LABELS. `ExprRecordLit`'s children are its field initializers
///     and nothing else, so the label is decidable from the parent's kind. The initializer's
///     VALUE is still walked, because that is where a reference lives.
///   * A FIELD PROJECTION'S MEMBER NAME. `f.widget` names a field of the value `f`, not a
///     declaration `widget`. The SPELLING is kept — `dotted_chain` records `f.widget` whole,
///     which `module_prefix_of` needs to tell a module-qualified reference
///     (`probe.home.widget`) from an ordinary projection; the wall keys on the last segment
///     either way, so a qualified reference keeps its leaf.
///
/// THE REMAINING MEMBERS OF THE CLASS, each excluded by a structural discriminator derived
/// against a fixture — the parent kinds carrying them also carry children that ARE real
/// references, so every rule below suppresses ONLY the label or binder and keeps the
/// genuine-reference children walked:
///
///   * A RECORD TYPE DECLARATION'S FIELD LABEL. `field_to_child_node` builds it with the
///     declared type parked in `inferred`, nothing in `children`/`params`, no expr data, no
///     connective — a shape no reference node has (a refinement's base type is a
///     `leaf_type_node` with `inferred: None`, so it stays collected). The declared TYPE lives
///     in `inferred`, which this walk never visits; it reaches consumers through
///     `authored_type_references`, so that channel must be unioned wherever `referenced` is
///     read.
///   * A NAMED CALL ARGUMENT'S LABEL. `ExprCall`'s children are its argument nodes, the same
///     parent-kind rule as the record literal; the callee spelling is on the call node and the
///     argument VALUE is still walked. A positional argument's node has no ident span and
///     contributed nothing already.
///   * A PARAMETER BINDER. Everything directly on the `params` edge declares a name — value
///     parameter or generic type parameter — so the edge passes the label flag; the declared
///     type is `children[0]` and is still walked, the flag being consumed at one level.
///   * A COPRODUCT'S VARIANT NAMES IN THE DECLARATION. `Connective::Disj` is set only by the
///     two coproduct item builders, so a `Disj` parent's direct children are variant
///     declarations — already exported via `variants` — and their payload field nodes are
///     still walked (their labels then fall to the field rule).
fn collect_reference_occurrences(
    node: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
    in_declaration: &str,
    ident_is_a_binder_or_label: bool,
    out: &mut BTreeSet<(String, String)>,
) {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let is_projection_member = matches!(&*node.expr_data, ExprData::ExprFieldAccess { .. });
        // The `field_to_child_node` shape: declared type in `inferred`, no children, no
        // params, no expr data, no connective, an authored ident. `inferred: Some` alone
        // separates it from every reference node; the rest pins the constructor.
        let is_declared_field_label = node.inferred.is_some()
            && node.children.is_empty()
            && node.params.is_empty()
            && matches!(&*node.expr_data, ExprData::NoExprData)
            && node.connective == Connective::NoConnective
            && node.ident_span.is_some();
        if !ident_is_a_binder_or_label && !is_projection_member && !is_declared_field_label {
            let name = authored_name_at(source_indices.clone(), node.clone());
            if !name.is_empty() {
                out.insert((in_declaration.to_string(), name));
            }
        }
        if let Some(chain) = dotted_chain(node, source_indices) {
            out.insert((in_declaration.to_string(), chain));
        }
        let children_are_field_labels =
            is_record_literal(node) || is_call(node) || node.connective == Connective::Disj;
        for c in node.children.iter() {
            collect_reference_occurrences(
                c,
                source_indices,
                in_declaration,
                children_are_field_labels,
                out,
            );
        }
        for c in node.params.iter() {
            collect_reference_occurrences(c, source_indices, in_declaration, true, out);
        }
        for c in node.properties.iter() {
            collect_reference_occurrences(c, source_indices, in_declaration, false, out);
        }
        for c in node.uses.iter() {
            collect_reference_occurrences(c, source_indices, in_declaration, false, out);
        }
        if let Some(b) = node.body.as_ref() {
            collect_reference_occurrences(b, source_indices, in_declaration, false, out);
        }
        if let Some(t) = node.transport.as_ref() {
            collect_reference_occurrences(t, source_indices, in_declaration, false, out);
        }
        if let Some(t) = node.type_annotation.as_ref() {
            collect_reference_occurrences(t, source_indices, in_declaration, false, out);
        }

        // THE VARIANT-PATTERN CONSTRUCTOR NAME -- THE ONE AUTHORED REFERENCE WITH NO
        // TRANSPORT ENTRY TO CONSUME, AND THE ONLY REASON THIS FUNCTION READS A NAME AT ALL.
        //
        // The operator ruling: authored references come from the parser's
        // `OccurrenceTransport`, never from re-reading the final `Node` -- the parser stamps
        // the fact exactly, so a Node-reading projection is a SECOND AUTHORITY free to
        // diverge. `authored_type_references` below obeys that.
        //
        // THAT RATIONALE HAS NO REFERENT HERE, checked rather than argued:
        //   `MatchPattern::VariantPattern { name: String, .. }` -- the head is a String
        //   `stamp_parsed_pattern`'s VariantPattern arm stamps `field_bindings` ONLY
        //   `ConstructorOccurrence` is stamped NOWHERE in `v1_compiler_parse`
        //   occurrence ids are minted per `Node`, so a String can never carry one
        // The parser mints nothing for this position, so there is no first authority to be
        // second to: reading the authored String is the ONLY derivation. A prohibition whose
        // reason does not apply is not extended by its letter -- that would forbid the only
        // available construction in favour of one that does not exist.
        //
        // THE TRANSPORT REPAIR IS BLOCKED ON A RULING, NOT ON A RISK. The objection to stamping
        // the head -- occurrence ids come from a sequential allocator, so a new stamp shifts
        // every later id -- is ANSWERED: `v2.std.node` `content_hash` folds node kind, edge
        // labels and child hashes, NOT the occurrence id, so a shifted id cannot move a hash;
        // and `v2.workflow.legacy_binding_delta` states outright that an `OccurrenceId` is not
        // a stable cross-compile name BECAUSE the counter is consumed in DFS order and encodes
        // walk position -- naming inserted tokens as exactly the edit that shifts ids. The
        // corpus declares that dependency illegitimate, not merely absent.
        //
        // What remains is an authority question: stamping a new occurrence is a change to the
        // parser. This block is provisional pending that ruling; when it comes the deletion is
        // this `if` and nothing else -- the name then arrives in `authored_type_references`
        // and the consumer already reading the union does not change.
        //
        // A pattern head is unreachable by the seven-slot walk above even in principle: a raw
        // `String` on `MatchPattern`, not a `Node`. So a module whose only use of an imported
        // coproduct is naming its variants in match arms contributed NOTHING to `referenced`,
        // and the gate concluded no name here resolved into the target.
        //
        // ONLY `name`. NOT `parent_enum`: v1.02_parse writes `parent_enum: none` and INFERENCE
        // fills it later, so collecting it would mint a membership fact from a compiler
        // consequence rather than authored source -- the failure the operator ruling forbids,
        // through the one field of this enum that looks authored and is not.
        //
        // NOT `field_bindings` either. Those are BINDERS -- they declare names -- and the
        // recursion below would otherwise report a pattern's own bound variables as references
        // into whatever module spells them the same way.
        if let Some(pattern) = node.match_pattern.as_ref() {
            if let MatchPattern::VariantPattern { name, .. } = &**pattern {
                if !name.is_empty() {
                    out.insert((in_declaration.to_string(), name.clone()));
                }
            }
        }
    })
}

/// The parser's own authored type references, read back out of the transport it already
/// produced for this file.
///
/// THIS RECONSTRUCTS NOTHING. `stamp_parsed_inferred` stamps a declared type as
/// `ParsedOccurrenceReference { TypeOccurrence }`; the spelling is the index entry's
/// `OccurrenceProjection.authored_name`, the enclosing declaration the second element of the
/// reference's `containment.ancestors`. Every value is looked up, none derived from the `Node`.
///
/// WHY `ancestors[1]` IS THE ENCLOSING DECLARATION. `stamp_parsed_node` pushes its own
/// occurrence onto the ancestor list before descending, so ancestors run outermost-first from
/// the stamp root; the sweep stamps one file per transport rooted at its module node, so `[0]`
/// is the module and `[1]` the module-scope item containing the reference. Fewer than two
/// ancestors means not inside a module-scope declaration: SKIPPED, not attributed to the empty
/// string, which would key a row no consumer can join and read as a real reference belonging to
/// a declaration that does not exist.
///
/// THE CATEGORY FILTER IS THE POINT. Only `TypeOccurrence` references are taken; folding in the
/// transport's lexical-value, field and namespace-segment occurrences would reintroduce the
/// over-collection `referenced` is criticised for, with the parser's authority attached --
/// worse than the walk.
fn authored_type_references_from_transport(
    transport: &Rc<OccurrenceTransport>,
    declared: &BTreeSet<String>,
) -> BTreeSet<(String, String)> {
    let mut by_id: HashMap<i64, String> = HashMap::new();
    for entry in transport.index.entries.iter() {
        by_id.insert(
            entry.projection.occurrence.value,
            entry.projection.authored_name.clone(),
        );
    }

    let mut out = BTreeSet::new();
    for reference in transport.references.iter() {
        if reference.category != OccurrenceCategory::TypeOccurrence {
            continue;
        }
        let spelling = match by_id.get(&reference.occurrence.value) {
            Some(name) if !name.is_empty() => name.clone(),
            _ => continue,
        };
        let enclosing = match reference.containment.ancestors.get(1) {
            Some(ancestor) => match by_id.get(&ancestor.value) {
                Some(name) if !name.is_empty() => name.clone(),
                _ => continue,
            },
            None => continue,
        };
        // ONLY REFERENCES ENCLOSED BY A DECLARATION THIS MODULE DECLARES -- load-bearing: its
        // absence broke a live arm in the dangerous direction.
        //
        // The transport stamps an IMPORT MEMBER NAME as a `TypeOccurrence` reference too,
        // enclosed by the import's target: `import probe.other { gadget }` yields
        // `("probe.other", "gadget")` on the fixture that caught it. Folded in, every import is
        // "bound through" by its own member and `UnusedSubjectMembershipRemoved` can never fire
        // again -- the wall permanently quiet about unused membership while looking more
        // precise. Strictly worse than the false green this change closes: one wrong verdict
        // versus a disposition that can no longer fire.
        //
        // A module-scope declaration name is the right filter, not a proxy: the key IS
        // `(enclosing declaration, spelling)`, and an import is not a declaration. Anything
        // enclosed by something this module does not declare is dropped rather than attributed.
        if !declared.contains(&enclosing) {
            continue;
        }
        out.insert((enclosing, spelling));
    }
    out
}

/// One module's record, from that one module's parse tree. No corpus, no resolution.
pub fn record_from_module(
    module: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
    rel_path: &str,
    transport: &Rc<OccurrenceTransport>,
) -> ModuleDeclarationRecord {
    let module_path = authored_name_at(source_indices.clone(), module.clone());
    let mut declared = BTreeSet::new();
    let mut variants = BTreeSet::new();
    let mut decl_fields = BTreeMap::new();
    for item in module_items(module.clone()).iter() {
        let name = authored_name_at(source_indices.clone(), item.clone());
        if name.is_empty() {
            continue;
        }
        if item.connective == Connective::Disj {
            for v in item.children.iter() {
                let vname = authored_name_at(source_indices.clone(), v.clone());
                if !vname.is_empty() {
                    variants.insert(vname);
                }
            }
        }
        decl_fields.insert(name.clone(), declaration_field_names(item, source_indices));
        declared.insert(name);
    }

    let mut reexported = BTreeSet::new();
    let mut imports = Vec::new();
    for imp in module_imports(module.clone()).iter() {
        let target = authored_name_at(source_indices.clone(), imp.clone());
        if import_is_all(imp.clone()) {
            imports.push(ImportClaim {
                target,
                members: Vec::new(),
                location: source_location_of(&imp.span),
            });
            continue;
        }
        let mut members = Vec::new();
        for (i, name) in import_specific_names_at(imp.clone(), source_indices.clone())
            .iter()
            .enumerate()
        {
            if name.is_empty() {
                continue;
            }
            reexported.insert(name.clone());
            let location = imp
                .children
                .get(i)
                .map(|c| source_location_of(&c.span))
                .unwrap_or_else(|| source_location_of(&imp.span));
            members.push((name.clone(), location));
        }
        imports.push(ImportClaim {
            target,
            members,
            location: source_location_of(&imp.span),
        });
    }

    let mut cited = Vec::new();
    for item in module_items(module.clone()).iter() {
        // The enclosing declaration is known HERE and nowhere deeper: carrying its name into
        // the subtree walk costs one string and closes the "two citations in one module share
        // one row" residue.
        let in_declaration = authored_name_at(source_indices.clone(), item.clone());
        for_each_node(item, &mut |node| {
            if let Some(c) = citation_from_record_literal(node, &in_declaration)
                .or_else(|| citation_from_constructor_call(node, &in_declaration))
            {
                cited.push(c);
            }
        });
    }

    let mut referenced = BTreeSet::new();
    let mut called = BTreeSet::new();
    for item in module_items(module.clone()).iter() {
        let in_declaration = authored_name_at(source_indices.clone(), item.clone());
        collect_reference_occurrences(
            item,
            source_indices,
            &in_declaration,
            false,
            &mut referenced,
        );
        for_each_node(item, &mut |node| {
            if is_call(node) && !node.name.is_empty() {
                called.insert(node.name.clone());
                if let Some(tail) = node.name.rsplit('.').next() {
                    called.insert(tail.to_string());
                }
            }
        });
    }

    ModuleDeclarationRecord {
        referenced,
        called,
        authored_type_references: authored_type_references_from_transport(transport, &declared),
        declares_construction_justification: declared.contains(CONSTRUCTION_JUSTIFICATION_DECL),
        is_fixture_carrier: module_is_fixture_carrier(&module_path, rel_path),
        module_path,
        rel_path: rel_path.to_string(),
        declared,
        variants,
        reexported,
        decl_fields,
        imports,
        cited,
    }
}

/// The corpus-shaped view: module path -> that module's own record. Built by INSERTION
/// from the parse sweep, never by a walk of its own.
#[derive(Debug, Clone, Default)]
pub struct DeclarationIndex {
    modules: BTreeMap<String, ModuleDeclarationRecord>,
    duplicates: Vec<(String, String, String)>,
    /// The first segment of every module path the sweep observed. It is what makes an
    /// absent cited module DECIDABLE rather than an authored exception list — see
    /// `citation_is_outside_index`.
    namespace_roots: BTreeSet<String>,
}

/// The denominators a green must name (DESIGN §5 — a green that cannot say what it
/// covered is an instrument failure wearing coverage's clothes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationIndexPopulation {
    pub modules: usize,
    pub declarations: usize,
    pub import_members: usize,
    pub citations: usize,
    /// Citations suppressed by an enumerated `PRE_EXISTING_CITATION_DEBT` row. Counted, so
    /// a reader can watch the contract shrink rather than take the roster on trust.
    ///
    /// THE `!is_fixture_carrier` FILTER THAT USED TO GUARD THIS IS GONE, a property of the row
    /// grain, not of today's roster contents. A row names its citing module, so enrolment is
    /// answered by the row itself; a carrier-shaped pre-filter could only change the answer by
    /// disagreeing with the rows — the paired-arm desynchronization this module carries a
    /// receipt for. Were a fixture citer enrolled tomorrow, counting it is the CORRECT reading
    /// of this field, whose subject is the roster and not the carrier.
    pub citations_pre_existing_debt: usize,
    /// Citations authored inside a witness or fixture carrier, where deliberately false
    /// text is the evidence rather than a defect. Counted, never silently dropped.
    pub citations_in_fixtures: usize,
    /// Citations naming a namespace no swept module declares — hand-Rust and other
    /// universes. Counted rather than dropped, so a green names what it did NOT cover.
    pub citations_outside_index: usize,
    /// Distinct in-corpus modules cited as authorities from ordinary authored modules, but
    /// reached by no authored import edge. A citation asserts that the target is a fact's
    /// home; without an import edge no consumer closure typechecks that home. Identities are
    /// carried rather than only a count so the unobserved remainder cannot be mistaken for a
    /// specimen list or a percentage (DESIGN's third emit-stage escape mode).
    pub cited_authorities_without_import_edges: Vec<String>,
    /// The retained identities split by whether any citing module syntactically calls the cited
    /// declaration. The called arm is an under-declaration candidate, not proof: indirection can
    /// hide a call and a same-spelled callee can be unrelated. Both arms still lack compile
    /// coverage; gunbc#9453 demonstrates the import-edge repair for one called specimen only.
    pub cited_and_called_without_import_edges: Vec<String>,
    pub cited_not_called_without_import_edges: Vec<String>,
    /// Retained citees under `dag/`, the FIRST and therefore entry-producing root of the
    /// existing `--source-root dag --source-root src/v2` corpus compile.
    pub cited_authorities_under_primary_dag_entry_root: Vec<String>,
    /// Retained citees under `src/v2/`, which that invocation indexes only as a dependency
    /// pool. With no inbound import edge these identities are structurally unreachable there.
    pub cited_authorities_in_src_v2_dependency_pool_only: Vec<String>,
    /// Import members admitted ONLY because they name a kernel type, over a target that
    /// declares no such name. Counted rather than skipped — see `import_member_findings`.
    pub import_members_kernel_named: usize,
    pub lens_modules: usize,
}

pub fn index_insert(index: &mut DeclarationIndex, record: ModuleDeclarationRecord) {
    if record.module_path.is_empty() {
        return;
    }
    if let Some(prior) = index.modules.get(&record.module_path) {
        index.duplicates.push((
            record.module_path.clone(),
            prior.rel_path.clone(),
            record.rel_path.clone(),
        ));
        return;
    }
    if let Some(root) = record.module_path.split('.').next() {
        index.namespace_roots.insert(root.to_string());
    }
    index.modules.insert(record.module_path.clone(), record);
}

pub fn index_get<'a>(
    index: &'a DeclarationIndex,
    module_path: &str,
) -> Option<&'a ModuleDeclarationRecord> {
    index.modules.get(module_path)
}

/// Identity-grain complement of a required lane's semantic-resolution population.
///
/// Both inputs name observations made by the lane: `admitted_module_identities` comes from
/// source-root ingest, while `judged_module_identities` names modules for which the lane completed
/// strict semantic resolution and typechecking to a typed verdict, whether positive or negative.
/// A module that aborts before producing a typed result is not judged. The caller keeps in-run
/// judgments distinct from content-addressed cross-process judgments. The broader parse-sweep
/// index is deliberately not a denominator, and import edges are deliberately absent: neither
/// establishes that the lane judged a module.
pub fn modules_unresolved_by_lane(
    admitted_module_identities: Vec<String>,
    judged_module_identities: &[String],
) -> Vec<String> {
    module_identity_difference(admitted_module_identities, judged_module_identities)
}

fn module_identity_difference(
    declared_module_identities: Vec<String>,
    judged_module_identities: &[String],
) -> Vec<String> {
    let judged: BTreeSet<&str> = judged_module_identities
        .iter()
        .map(String::as_str)
        .collect();
    declared_module_identities
        .into_iter()
        .filter(|identity| !judged.contains(identity.as_str()))
        .collect()
}

#[cfg(test)]
mod lane_resolution_join_tests {
    use super::module_identity_difference;

    #[test]
    fn identical_populations_have_empty_difference() {
        let declared = vec!["probe.alpha".to_string(), "probe.beta".to_string()];
        assert_eq!(
            module_identity_difference(declared.clone(), &declared),
            Vec::<String>::new()
        );
    }

    #[test]
    fn difference_enumerates_the_missing_identity() {
        let declared = vec!["probe.alpha".to_string(), "probe.beta".to_string()];
        assert_eq!(
            module_identity_difference(declared, &["probe.alpha".to_string()]),
            vec!["probe.beta".to_string()]
        );
    }
}

pub fn index_records(index: &DeclarationIndex) -> Vec<&ModuleDeclarationRecord> {
    index.modules.values().collect()
}

pub fn index_population(index: &DeclarationIndex) -> DeclarationIndexPopulation {
    let imported_modules: BTreeSet<String> = index
        .modules
        .values()
        .filter(|record| !record.is_fixture_carrier)
        .flat_map(|record| record.imports.iter())
        .filter_map(|import| {
            resolve_cited_module(index, &import.target).map(|target| target.module_path.clone())
        })
        .collect();
    let retained_citations: Vec<(&ModuleDeclarationRecord, &CitedSymbol, String)> = index
        .modules
        .values()
        .filter(|record| !record.is_fixture_carrier)
        .flat_map(|record| record.cited.iter().map(move |cited| (record, cited)))
        .filter_map(|(record, cited)| {
            resolve_cited_module(index, &cited.module_path)
                .filter(|target| {
                    !target.is_fixture_carrier
                        && target.module_path != record.module_path
                        && !imported_modules.contains(&target.module_path)
                })
                .map(|target| (record, cited, target.module_path.clone()))
        })
        .collect();
    let cited_authorities_without_import_edges: Vec<String> = retained_citations
        .iter()
        .map(|(_, _, target)| target.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let cited_and_called_without_import_edges: Vec<String> = retained_citations
        .iter()
        .filter(|(record, cited, _)| record.called.contains(&cited.decl_name))
        .map(|(_, _, target)| target.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let cited_not_called_without_import_edges: Vec<String> = cited_authorities_without_import_edges
        .iter()
        .filter(|target| !cited_and_called_without_import_edges.contains(target))
        .cloned()
        .collect();
    let cited_authorities_under_primary_dag_entry_root: Vec<String> =
        cited_authorities_without_import_edges
            .iter()
            .filter(|module_path| {
                index
                    .modules
                    .get(*module_path)
                    .is_some_and(|record| record.rel_path.starts_with("dag/"))
            })
            .cloned()
            .collect();
    let cited_authorities_in_src_v2_dependency_pool_only: Vec<String> =
        cited_authorities_without_import_edges
            .iter()
            .filter(|module_path| {
                index
                    .modules
                    .get(*module_path)
                    .is_some_and(|record| record.rel_path.starts_with("src/v2/"))
            })
            .cloned()
            .collect();
    DeclarationIndexPopulation {
        modules: index.modules.len(),
        declarations: index.modules.values().map(|r| r.declared.len()).sum(),
        import_members: index
            .modules
            .values()
            .flat_map(|r| r.imports.iter())
            .map(|i| i.members.len())
            .sum(),
        citations: index.modules.values().map(|r| r.cited.len()).sum(),
        citations_pre_existing_debt: index
            .modules
            .values()
            .flat_map(|r| r.cited.iter().map(move |c| (r, c)))
            .filter(|(r, c)| citation_in_roster(&r.module_path, c, PRE_EXISTING_CITATION_DEBT))
            .count(),
        citations_in_fixtures: index
            .modules
            .values()
            .filter(|r| r.is_fixture_carrier)
            .map(|r| r.cited.len())
            .sum(),
        citations_outside_index: index
            .modules
            .values()
            .flat_map(|r| r.cited.iter())
            .filter(|c| citation_is_outside_index(index, &c.module_path))
            .count(),
        cited_authorities_without_import_edges,
        cited_and_called_without_import_edges,
        cited_not_called_without_import_edges,
        cited_authorities_under_primary_dag_entry_root,
        cited_authorities_in_src_v2_dependency_pool_only,
        import_members_kernel_named: import_member_kernel_named_count(index),
        lens_modules: index
            .modules
            .keys()
            .filter(|m| is_top_level_lens_module(m))
            .count(),
    }
}

/// A citation may name a module by its LOGICAL path — `v2.` stripped — the identity
/// `decl_facts` published and the corpus is authored against. Both spellings resolve to the
/// one module; the fallback only ever finds a module that really declares itself `v2.x`.
pub(crate) fn resolve_cited_module<'a>(
    index: &'a DeclarationIndex,
    module_path: &str,
) -> Option<&'a ModuleDeclarationRecord> {
    index
        .modules
        .get(module_path)
        .or_else(|| index.modules.get(&format!("v2.{module_path}")))
}

/// WHETHER AN ABSENT CITED MODULE IS A REFUSAL OR A COVERAGE BOUNDARY, decided from the
/// index rather than from an authored allowlist.
///
/// The corpus cites modules that are not `.dag` — `v1_compiler.cli_run` and siblings name
/// hand-Rust, a namespace no `.dag` file declares. Such a citation is TRUE and UNRESOLVABLE at
/// once, which `std.decl_ref` `CitationIndexCoverage` models: resolution answers "does this
/// name a real declaration", coverage "is it inside the universe the index covers". Refusing
/// them would refuse correct citations.
///
/// The decidable line is the NAMESPACE ROOT. If some swept module declares the citation's
/// first segment, the citation is inside the `.dag` namespace and its module must exist — so
/// a DELETED `.dag` module still refuses, the rot §3's rule exists to catch. If no module
/// declares that root, the citation names another universe and is counted as outside the
/// index, never dropped.
///
/// NOT A THRESHOLD OR A HEURISTIC (DESIGN §4 — a heuristic is never necessary in a closed
/// system): a namespace root is declared by a swept module or it is not; the predicate is
/// total and derived from the same one index.
fn citation_is_outside_index(index: &DeclarationIndex, module_path: &str) -> bool {
    if resolve_cited_module(index, module_path).is_some() {
        return false;
    }
    let root = module_path.split('.').next().unwrap_or(module_path);
    !index.namespace_roots.contains(root)
}

/// Whether an import member is admitted ONLY by the kernel-type escape below.
///
/// A REAL HOLE IN THE WALL, THE ONE PLACE THIS FILE DOES NOT CLOSE. `v1.03_resolve`
/// `get_exported_names` appends `map_keys(kernel_type_set)` to every module's export surface,
/// so EVERY module exports every kernel type name; `import m { Int }` is admitted whatever `m`
/// is, and the index must admit it too or refuse source the compiler accepts.
///
/// MEASURED against the installed compiler on a throwaway source root: a module whose whole
/// body is one `fn` returning `Int`, imported as
/// `import extdeps.probe_missing_anchor { Int, String, Bool }`, compiled to 6 files with 0
/// diagnostics. The zero is readable because the nonzero ran beside it — the same root with
/// `{ probe_absent_member_RED }` refused with a located `MissingExport` at the member token.
/// The wall is live and this class walks through it.
///
/// THE LIVE SPECIMEN IS NOT SYNTHETIC. `std.types` declares no `Int`, `String` or `Float` —
/// it names them as KEYS of `kernel_type_set`, a different fact — and hundreds of modules
/// author `import std.types { Int, String }`. Every such claim is false about `std.types` and
/// always was; nobody noticed because the escape makes it unfalsifiable.
///
/// WHY COUNTED RATHER THAN REFUSED. Refusing changes what the SEED COMPILER ACCEPTS —
/// `get_exported_names` is the authority and editing it is `NewLanguageBehavior`, which
/// `gunbc.v1_maintenance_standing` refuses, a refusal dominating every admission. What this
/// may do, and what DESIGN §5 requires of any widening arm, is make the frequency OBSERVABLE:
/// a bare `continue` zeroes the deficit by construction so the class never ranks for fixing
/// (§6 prices by displaced cost; a masked cost displaces nothing). Counted, it can be watched,
/// prioritized, and burned down.
///
/// NEXT-RUNG TRIGGER: `get_exported_names` grounds its export surface in DEFINITIONS rather
/// than appending the kernel set to every module, at which point this predicate deletes and
/// the members it admits become ordinary `ImportMemberAbsent` findings.
fn import_member_is_kernel_named(target: &ModuleDeclarationRecord, name: &str) -> bool {
    !import_surface_has(target, name) && crate::std_types::kernel_type_set().contains_key(name)
}

fn import_member_kernel_named_count(index: &DeclarationIndex) -> usize {
    let mut n = 0;
    for record in index.modules.values() {
        for claim in &record.imports {
            let Some(target) = index.modules.get(&claim.target) else {
                continue;
            };
            for (name, _) in &claim.members {
                if import_member_is_kernel_named(target, name) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// (1) Import-member claim integrity.
///
/// A target ABSENT from the index is not reported here — a denominator reason, not leniency:
/// the sweep's roots are the authored `.dag` roots, so an absent target is a module never
/// observed, and "member absent" over it would assert a fact about text never read. Module
/// existence is a different question — the compiler's `UnresolvedImport`.
///
/// The kernel-type escape is the one admission that is NOT a fact about the target module;
/// counted as `import_members_kernel_named`, receipt on `import_member_is_kernel_named`.
pub fn import_member_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    let mut out = Vec::new();
    for record in index.modules.values() {
        for claim in &record.imports {
            let Some(target) = index.modules.get(&claim.target) else {
                continue;
            };
            for (name, location) in &claim.members {
                if import_surface_has(target, name) {
                    continue;
                }
                if import_member_is_kernel_named(target, name) {
                    continue;
                }
                out.push(DeclarationIntegrityFinding {
                    kind: DeclarationIntegrityKind::ImportMemberAbsent,
                    rel_path: record.rel_path.clone(),
                    offset: span_offset_in(location, &record.rel_path),
                    message: format!(
                        "`{}` imports `{name}` from `{}`, which declares no such name",
                        record.module_path, claim.target
                    ),
                });
            }
        }
    }
    out
}

/// THE PRE-EXISTING CITATION DEBT, ENUMERATED AT SITE GRAIN.
///
/// WHY A ROSTER EXISTS. Nothing has checked an authored citation since 2026-08-23, when the
/// operator removed the `cited-symbol` job (DESIGN records the drop and states its future
/// exposure as unbounded). This wall's first execution lands on a corpus accumulating exactly
/// the class §3's rule names; every row is a REAL DEFECT, a citation naming a declaration that
/// does not exist. Measured on the live tree: 42 sites, each named by its authoring module and
/// declaration.
///
/// WHY NOT REPAIRED HERE. A citation's repair is a judgement about what its author MEANT, and
/// guessing is how a stale citation becomes a confidently wrong one.
/// `extdeps.docker.container_stats` `Stats` is probably `ContainerStats`; `gunbc.ci_workflow`
/// was deleted by the floor cut and its citation may want a different module or none. 38
/// subjects with 38 owners, bundled into the wall's own change, would make the wall
/// unreviewable.
///
/// WHY A DEBT CONTRACT AND NOT A QUIETER SNAPSHOT. DESIGN §5 admits a monotone debt contract
/// on four conditions, all met: the subject universe is INDEPENDENTLY DISCOVERED (by this
/// index, from the corpus's own text); CLOSED (every authored citation in a non-fixture
/// module); membership checked at IDENTITY GRAIN, not count; direction one-way — see
/// `citation_debt_findings`, which REFUSES a row that no longer reproduces. That is the teeth:
/// repairing a citation forces deleting its row, so the roster only shrinks, and a rotted
/// roster stops the line like a violation.
///
/// THE FOUR PLANTED CONTROLS ARE NOT IN THIS ROSTER, AND THE PROSE THAT SAID THEY WERE WAS
/// FALSE. An earlier revision read "FOUR ROWS AT THE END ARE NOT DEBT ... they are the DELETED
/// census's own planted controls ... they leave when the lens does". Enumerating all 38 rows
/// finds no such row — no `G1_planted` target, no `synthetic` namespace; the last four are
/// ordinary debt. The rows were described, never added.
///
/// Caught by the wall's first corpus run, which reported all four controls as ordinary
/// refusals: `synthetic.g1_planted_module_absent_control_RED`, two `v2.std.node` declarations
/// and one `NodeKind` field. Recording the claim as FALSE rather than silently correcting it is
/// the point — a stale statement inside the carrier built to stop stale statements is the
/// specimen.
///
/// They now live in `PLANTED_CONTROL_CITATIONS`, a different KIND of roster: debt shrinks to
/// empty, controls never retire, and the two arms read one trigger in opposite directions.

/// THE LENS IS DELETED AS OF 2026-08-26, AND THE HAND-OFF THIS COMMENT LEFT WAS HALF RIGHT.
/// It read "its deletion cascades through sixteen witnesses ... they delete with the lens, not
/// before it"; the staged form was correct — the wall and its predecessor's funeral belonged
/// in separate diffs — but the population was wrong.
///
/// SIXTEEN WAS THE COUNT AT #7707, when the lens landed. The file grew twice after (#8673
/// enrolled roster_registry, #8775 the two instance-gap carriers) and carried 27 `test fn`
/// identities when this comment was written beside it.
///
/// AND "DEAD" WAS TRUE OF THE LENS AND FALSE OF ITS WITNESSES. Of the 27, only 6 touch one of
/// the seven symbols the witness file imported FROM the lens. Six more call
/// `resolve_declaration_ref` directly — in `v2.std.decl_ref_resolution`, which SURVIVES with
/// four other consumers — so they are the only executing evidence for a live authority's
/// five-arm refusal, and §4b(4) keeps them enrolled; they moved to
/// `test.claim.long.decl_ref_resolution_witness_test`. The remaining 15 are population and
/// projection claims about the carriers that PROJECT `DeclarationRef`s, moved to
/// `test.claim.long.carrier_reference_integrity_witness_test`.
///
/// So the disposition is three-way where this comment admitted two: the lens dies, six
/// witnesses die with it, twelve rehome onto subjects that outlive it.
/// Citations INSIDE fixture and witness carriers that do not resolve, enumerated at identity
/// grain because carrier identity is not a licence.
///
/// WHY THIS EXISTS, AND WHY WHAT IT REPLACED WAS WRONG (review 55939). Both citation arms used
/// to skip every citation in a module `module_is_fixture_carrier` answered true for. The
/// justification was real — a witness proving the resolver refuses an absent symbol must author
/// one, so its false citation is evidence, not defect — but the EXEMPTION WAS KEYED ON THE
/// MODULE while the justification is a property of the CITATION. Carrier identity establishes
/// that SOME citations there are deliberately false, never ALL.
///
/// THE ROWS ARE DERIVED FROM THE INDEX, NOT FROM DIAGNOSTIC TEXT; the first attempt parsed
/// rendered messages and silently dropped the FIELD, because a `CitedDeclarationAbsent`
/// message never prints one. Rows for `NamedField` citations whose DECLARATION is absent then
/// matched nothing, and the corpus run reported the citation as refusing AND its row as spent —
/// the paired-inverse-arm desynchronization this module documents, firing on its author. Five
/// further identities are deliberately NOT here: they refuse but are enrolled in
/// `PRE_EXISTING_CITATION_DEBT` or `PLANTED_CONTROL_CITATIONS`, and a second row would be
/// duplicate authority with a double stale-arm report.
///
/// MEASURED, WHICH SETTLED IT: of 161 citations authored inside fixture carriers, 128 RESOLVE
/// — ordinary citations of real authorities that happen to live in a test module. The
/// module-grained skip shielded all 128 to protect at most 33. Counting the excluded
/// population as `in_fixtures` did not restore integrity; a counted hole is still a hole.
///
/// AND THE HOLE WAS OCCUPIED, not merely reachable — a defect, not a disclosed boundary. Two
/// rows below are ordinary staleness unrelated to fixture intent:
///   - `dag.test.claim.witness_purpose_taxonomy_witness` is a `dag.`-prefixed module path no
///     module declares; the real module is `test.claim.witness_purpose_taxonomy_witness`. A
///     plain typo, invisible while the skip stood.
///   - `std.disposition` `Disposition` `marker` names a real authority and declaration with
///     an absent field, cited twice from `v2.lens/disposition_redundancy_test.dag`.
///
/// WHAT THIS ROSTER IS AND IS NOT. An identity-grain exemption, not a debt contract — the
/// difference is condition 3 of §5's four: a debt roster's terminal state is EMPTY and this
/// one's is not (a planted control such as `NoSuchDecl_G1_RED` is permanent by design). What it
/// shares with the debt contract is what makes either safe: MONOTONE, and REFUSES WHEN SPENT.
/// A row whose citation stops refusing is reported by the same inverse arm that polices
/// `PRE_EXISTING_CITATION_DEBT`, so the roster cannot rot into things that used to be true,
/// and deleting a row while its citation still refuses turns it into an ordinary finding that
/// stops the line.
///
/// THE ROWS ARE NOT CLASSIFIED into deliberate-control versus genuine-staleness: the two
/// specimens above are decidable by inspection; the rest are not sorted, because deciding what
/// a witness's author MEANT is the judgement §5 warns turns a stale citation into a confidently
/// wrong one. Next rung: a witness declaring its planted controls as typed rows, at which point
/// the deliberate half becomes derivable and only the genuine half survives here as debt.
const FIXTURE_CARRIER_CITATION_EXEMPTIONS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "test.claim.altra_placement_witness",
        "w_a_plan_from_another_design_is_refused_as_incomparable",
        "product.altra_motherboard.minimal_design",
        "some_other_board",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "bound_condition_does_not_fire_on_a_near_miss_ref",
        "extdeps.network.mac",
        "parse_mac_addres",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "bound_condition_does_not_fire_on_a_near_miss_ref",
        "extdeps.network.max",
        "parse_mac_address",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "planted_bound_row_ref",
        "test.fixture.frontier",
        "planted_bound_trigger",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "planted_rows",
        "test.fixture.frontier",
        "planted_subject",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "planted_rows",
        "test.fixture.frontier",
        "planted_unbound_subject",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "frontier_expiry_fired_trigger_absent_from_rows_is_clean",
        "test.fixture.frontier",
        "never_a_trigger_here",
        "",
    ),
    (
        "test.claim.capability_binding_witness",
        "witness_the_bob_requirement_is_still_provisional",
        "gunbc.capability_binding",
        "x",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "live_discriminator_rows",
        "test.claim.dissolution_census_mechanism_witness_test",
        "absent_subject_row",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "live_discriminator_rows",
        "test.claim.dissolution_census_mechanism_witness_test",
        "present_subject_row",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "live_discriminator_rows",
        "test.fixture.dissolution_live_discriminator.subject",
        "absent_subject_declaration",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_bound_ref",
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_bound_target",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_retires_ref",
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_retires_target",
        "",
    ),
    (
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_rows",
        "test.claim.dissolution_census_mechanism_witness_test",
        "planted_unbound_target",
        "",
    ),
    (
        "test.claim.keyed_roster_witness",
        "frontier_rows_keyed_build_admits_distinct_decl_fields",
        "synthetic.mod",
        "HostConfig",
        "",
    ),
    (
        "test.claim.keyed_roster_witness",
        "frontier_rows_keyed_build_admits_distinct_decl_fields",
        "synthetic.mod",
        "HostConfig",
        "memory_swap",
    ),
    (
        "test.claim.language_target_subject_registration",
        "red_unregistered_subject_ref_is_not_reported_as_present",
        "extdeps.languages.nonexistent.subject",
        "nonexistent_language",
        "",
    ),
    (
        "test.claim.long.carrier_reference_integrity_witness_test",
        "carrier_ref_refusal_count_red_on_dangling_fixture",
        "v2.std.node",
        "NoSuchDecl_G1_RED",
        "",
    ),
    (
        "test.claim.long.decl_ref_resolution_witness_test",
        "decl_ref_refuses_ambiguous_binding",
        "v2.std.node",
        "g1_ambiguous",
        "",
    ),
    (
        "test.claim.long.decl_ref_resolution_witness_test",
        "decl_ref_refuses_declaration_absent",
        "v2.std.node",
        "NoSuchDecl_G1_RED",
        "",
    ),
    (
        "test.claim.long.decl_ref_resolution_witness_test",
        "decl_ref_refuses_module_absent",
        "synthetic.g1_module_absent_RED",
        "any",
        "",
    ),
    (
        "test.claim.long.decl_ref_resolution_witness_test",
        "decl_ref_refuses_named_field_absent",
        "v2.std.node",
        "NodeKind",
        "NoSuchField_G1_RED",
    ),
    (
        "test.claim.long.carrier_reference_integrity_witness_test",
        "instance_gap_carrier_dangling_caller_red_control_refuses",
        "v2.compiler.parse",
        "G1_planted_instance_gap_caller_control_RED",
        "",
    ),
    (
        "test.claim.long.carrier_reference_integrity_witness_test",
        "instance_gap_membership_dangling_producer_red_control_refuses",
        "gunbc.publication_policy",
        "G1_planted_instance_gap_producer_control_RED",
        "",
    ),
    (
        "test.claim.long.v1_complexity_capability_census_resolution_test",
        "census_absent_declaration_refuses",
        "v1.compiler.complexity",
        "this_declaration_does_not_exist_in_the_seed",
        "",
    ),
    (
        "test.claim.long.v1_complexity_capability_census_resolution_test",
        "census_absent_module_refuses",
        "v1.compiler.this_module_does_not_exist",
        "classify_complexity",
        "",
    ),
    (
        "test.claim.long.v1_complexity_capability_census_resolution_test",
        "census_prose_only_name_does_not_resolve",
        "v1.compiler.complexity",
        "Derived",
        "",
    ),
    (
        "test.claim.primitive_identity_join_witness_test",
        "w_unknown_realization_refuses",
        "std.primitive_identity",
        "missing_handler",
        "",
    ),
    (
        "test.claim.primitive_projection_authority_witness_test",
        "planted_registry_row",
        "test.fixture",
        "planted",
        "",
    ),
    (
        "test.claim.repository_census_observation_witness",
        "witness_a_declaration_field_selector_reaches_the_identity",
        "test.claim.repository_census_observation_witness",
        "census_witness_classifier",
        "rows",
    ),
    (
        "test.claim.witness_purpose_taxonomy_witness",
        "fixture_population_ref",
        "dag.test.claim.witness_purpose_taxonomy_witness",
        "witness_purpose_taxonomy_witness_note",
        "",
    ),
    (
        "test.fixture.decl_facts_reflection.specimens",
        "scaffold_named_field_bind",
        "test.fixture.decl_facts_reflection.specimens",
        "named_field_anchor",
        "dissolves_to",
    ),
    (
        "test.fixture.scaffold_disposition_census.pool.specimens",
        "dangling_bind_target_specimen",
        "test.fixture.scaffold_disposition_census.pool.specimens",
        "no_such_declaration_G1_dangling_bind_control_RED",
        "",
    ),
    (
        "v2.test.lens_cost.valuation",
        "effect_operation",
        "v2.test.lens_cost.valuation",
        "unmodelled_effect_specimen",
        "",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_present_successor_locator",
        "std.disposition",
        "Disposition",
        "marker",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_red_scaffold",
        "std.disposition",
        "Disposition",
        "marker",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_region1_real_successor_present",
        "extdeps.llm.anthropic_messages_api",
        "AnthropicTextBlock",
        "cache_control",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_region_budget_per_service_overhead_locator",
        "gunbc.ci_floor_measurement",
        "gunbc_ci_managed_host_quiescent_meminfo_read",
        "",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_region_bytes_synthetic_present",
        "std.bytes",
        "builtin_function_registry",
        "",
    ),
    (
        "v2.test.lens_disposition_redundancy.disposition_redundancy_test",
        "redundancy_region_rust_gates_synthetic_present",
        "tools.rust_stage0_gates",
        "per_unit_test_selector",
        "",
    ),
];

const PRE_EXISTING_CITATION_DEBT: &[(&str, &str, &str, &str, &str)] = &[
    (
        "extdeps.docker.container_inspect",
        "container_inspect_error_responses_frontier_rows",
        "extdeps.docker.container_inspect",
        "Inspect",
        "",
    ),
    (
        "extdeps.docker.container_stats",
        "container_stats_error_responses_frontier_rows",
        "extdeps.docker.container_stats",
        "Stats",
        "",
    ),
    (
        "extdeps.docker.container_stats",
        "cpu_percent_ratio_carrier_frontier_rows",
        "extdeps.docker.container_stats",
        "ContainerStats",
        "cpu_percent",
    ),
    (
        "extdeps.github.actions_runner",
        "actions_runner_base_dir_ensure_script_scaffold",
        "gunbc.host_effect",
        "host_effect_apply",
        "",
    ),
    (
        "extdeps.github.actions_runner",
        "actions_runner_slot_extract_script_scaffold",
        "gunbc.host_effect",
        "host_effect_apply",
        "",
    ),
    (
        "extdeps.git.publication_transport",
        "extdeps_model_scope",
        "extdeps.git.publication_transport",
        "PublicationTransport",
        "",
    ),
    // RE-KEYED, NOT GROWN (DCH-1). These five rows are the same debt they were: the shape they
    // cite moved WHOLE from `extdeps.llm.anthropic` to `extdeps.llm.anthropic_messages_api`, so
    // the citing and cited module names changed and the debt did not. Row count, subject and
    // dissolution are identical; deleting them instead would have left five CITED-FIELD-ABSENT
    // refusals, and paying them here would have folded an unrelated modeling climb into a
    // replacement migration. They still retire the one way this roster allows — when the cited
    // optional fields are modeled on the specification's blocks.
    (
        "extdeps.llm.anthropic_messages_api",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic_messages_api",
        "AnthropicImageBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic_messages_api",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic_messages_api",
        "AnthropicTextBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic_messages_api",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic_messages_api",
        "AnthropicTextBlock",
        "citations",
    ),
    (
        "extdeps.llm.anthropic_messages_api",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic_messages_api",
        "AnthropicToolReferenceBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic_messages_api",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic_messages_api",
        "CacheControl",
        "ttl",
    ),
    (
        "extdeps.network.ipv6",
        "ipv6_text_codec_staged_frontier_rows",
        "extdeps.network.ipv6",
        "parse_ipv6_address",
        "",
    ),
    (
        "extdeps.tcgplayer.store",
        "tcgplayer_store_money_measure_grounding_disposition",
        "extdeps.tcgplayer.store",
        "UpdateSkuPrice",
        "price",
    ),
    (
        "gunbc.ci_floor_measurement",
        "gunbc_ci_legacy_host_fixed_overhead_disposition",
        "gunbc.ci_floor_measurement",
        "gunbc_ci_legacy_host_modeled_residents",
        "",
    ),
    (
        "gunbc.ci_floor_measurement",
        "gunbc_ci_managed_host_fixed_overhead_disposition",
        "gunbc.ci_floor_measurement",
        "gunbc_ci_managed_host_quiescent_meminfo_read",
        "",
    ),
    (
        "gunbc.claude_setup_token_enrollment",
        "claude_enrollment_exact_version_read_back_scaffold",
        "extdeps.cloud.gcp.secret_manager",
        "AccessVersion",
        "",
    ),
    (
        "gunbc.claude_setup_token_enrollment",
        "claude_enrollment_secret_manager_add_version_scaffold",
        "extdeps.cloud.gcp.secret_manager",
        "AddVersion",
        "",
    ),
    (
        "gunbc.emit_summary_map_consumer_partition",
        "emit_summary_map_consumers",
        "v1.compiler.infer_emit_info",
        "type_summary_reaches_fn",
        "",
    ),
    (
        "gunbc.empty_decl_file_checkpoint_bypass",
        "empty_decl_file_bypass_instances",
        "v1.compiler.05_emit",
        "emit_literal",
        "",
    ),
    (
        "gunbc.executor_privileged_operation",
        "executor_privileged_operation_shell_scaffold",
        "gunbc.host_effect",
        "host_effect_apply",
        "",
    ),
    (
        "gunbc.fabric_capacity_class_gap",
        "no_class_carrier",
        "product.fabric.supply",
        "Offer",
        "",
    ),
    (
        "gunbc.fabric_capacity_class_gap",
        "protection_has_no_reach",
        "gunbc.ci_runner_placement",
        "ci_runner_placement_authority",
        "",
    ),
    (
        "gunbc.fabric_capacity_class_gap",
        "protection_has_no_reach",
        "gunbc.fleet_host_budget",
        "fleet_host_budget_authority",
        "",
    ),
    (
        "gunbc.host_budget_source",
        "host_budget_source_seed_mirror_disposition",
        "gunbc.host_budget_source",
        "host_budget_source_emitted_into_stage0",
        "",
    ),
    (
        "gunbc.language_source_scaffold_index",
        "compiler_tests_harness_trigger",
        "v2.test.workflow.claim_witness_corpus_ci_runner",
        "ClaimWitnessCorpusClaimRunRow",
        "",
    ),
    (
        "gunbc.runner_connectivity_repair_plan",
        "runner_jit_installation_token_cache_scaffold",
        "gunbc.runner_lifecycle",
        "EnsureRunnerJitWrapper",
        "",
    ),
    (
        "gunbc.runner_slot_provision",
        "runner_slot_provision_scaffold",
        "gunbc.host_effect",
        "host_effect_apply",
        "",
    ),
    (
        "gunbc.self_host_promotion_obligations",
        "frontier_numerator_admits_seed_evidence",
        "v2.compiler.self_host.emitter_producer_provenance",
        "v2_self_hosted_promotions",
        "",
    ),
    (
        "gunbc.srv3_os_install_actuate_scope",
        "srv3_os_install_actuate_credential_source_scaffold",
        "gunbc.roadmap_authority",
        "roadmap_document",
        "",
    ),
    (
        "gunbc.tailscale_acl_phase2_credential",
        "tailscale_acl_phase2_live_write_disposition",
        "gunbc.tailscale_acl_phase2_credential",
        "tailscale_acl_upsert_wet",
        "",
    ),
    (
        "product.build_selection",
        "bandwidth_axis",
        "product.build_selection",
        "build_memory_bandwidth_axis",
        "",
    ),
    (
        "product.build_selection",
        "capacity_axis",
        "product.build_selection",
        "build_memory_capacity_axis",
        "",
    ),
    (
        "product.build_selection",
        "cash_axis",
        "product.build_selection",
        "build_incremental_cash_axis",
        "",
    ),
    (
        "product.build_selection",
        "ceiling_axis",
        "product.build_selection",
        "build_constructible_ceiling_axis",
        "",
    ),
    (
        "product.build_selection",
        "wall_power_axis",
        "product.build_selection",
        "build_wall_power_axis",
        "",
    ),
    (
        "std.bytes",
        "bytes_seam_host_realization_marker",
        "std.bytes",
        "builtin_function_registry",
        "",
    ),
    (
        "std.citation",
        "citation_cit2_mediawiki_provider_observation_scaffold",
        "extdeps.mediawiki",
        "extdeps_external_authority_anchor",
        "",
    ),
    (
        "std.encoding",
        "utf8_decode_bytes_host_realization_marker",
        "std.bytes",
        "builtin_function_registry",
        "",
    ),
    (
        "tools.rust_stage0_gates",
        "unit_must_run_staged_note",
        "tools.rust_stage0_gates",
        "per_unit_test_selector",
        "",
    ),
];

/// Whether a citation names one of the enumerated pre-existing targets.
/// Whether a citation is enrolled in `roster`, at SITE grain — the citing module included.
/// See the module header for why the target alone is not an identity a roster may key on.
///
/// The roster is a parameter because an enrolled-debt roster is a fact about ONE corpus, and
/// a predicate reading it from module scope makes its behaviour unauthorable by any fixture.
fn citation_in_roster(
    citing_module: &str,
    cited: &CitedSymbol,
    roster: &[(&str, &str, &str, &str, &str)],
) -> bool {
    roster
        .iter()
        .any(|row| *row == citation_site(citing_module, cited))
}

/// A citation's SITE identity — who cites, and what is cited. Every roster row is one of these;
/// the first field is the whole content of this change: a row exempts the site that authored
/// it, never the target it names.
fn citation_site<'a>(
    citing_module: &'a str,
    cited: &'a CitedSymbol,
) -> (&'a str, &'a str, &'a str, &'a str, &'a str) {
    (
        citing_module,
        cited.in_declaration.as_str(),
        cited.module_path.as_str(),
        cited.decl_name.as_str(),
        cited.field.as_deref().unwrap_or(""),
    )
}

/// THE CONTRACT'S OWN REFUSAL — a roster row that no longer reproduces.
///
/// Without this the roster is the "quieter snapshot" DESIGN §5 rejects: rows survive their
/// subjects, the population drifts from the corpus, and a reader takes a memory for a
/// measurement. With it the roster is monotone by construction: removing a violation means
/// repairing it AND deleting its row, and keeping a row means the violation is still there.
pub fn citation_debt_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    citation_debt_findings_against(index, PRE_EXISTING_CITATION_DEBT)
}

/// THE DELETED CENSUS'S PLANTED CONTROLS — DELIBERATELY FALSE CITATIONS, NOT DEBT.
///
/// `v2.lens.cited_symbol_resolution` authors citations that MUST NOT RESOLVE: the
/// discriminating evidence that a resolver refuses, one per refusal arm;
/// `cited_symbol_planted_control_home_note` records the operator ruling (2026-08-25) rehoming
/// them there as machine evidence rather than documents. Refusing them would refuse the
/// evidence for the wall's own mechanism.
///
/// THEY ARE NOT ENROLLED AS DEBT, AND THE DISTINCTION IS THE WHOLE POINT.
/// `PRE_EXISTING_CITATION_DEBT` may only SHRINK: a row leaves when its citation is repaired.
/// These never retire — DESIGN §4b(4): an expecting-red probe that greens flips to a permanent
/// regression control rather than being deleted. In the debt roster they would make "only
/// shrinks" false of four rows, and a contract with silent permanent members is not one.
///
/// SO THE STALENESS ARM IS INVERTED HERE, the carrier's whole content: a debt row refuses when
/// its citation STOPS refusing; a control row refuses on the same event for the opposite
/// reason — a resolving control is no longer discriminating and the mechanism it proves has
/// quietly lost its evidence. Same trigger, opposite meaning: two carriers, not one with a
/// flag.
///
/// THE NAMED TERMINUS HAS FIRED, 2026-08-26. What stood here said the four identities were a
/// deliberate, transient second representation of controls `v2.lens.cited_symbol_resolution`
/// also held, terminus that lens's deletion. This change IS that deletion: the lens is gone
/// and these rows are the sole authority.
///
/// THE DEADNESS MEASUREMENT IS KEPT, because it licensed the cut and held up. Of the 27
/// symbols unique to that lens, the only outside references were one prose row in
/// `gunbc.roster_registry`, two prose mentions in fast witnesses (a `String` note and a `//`
/// comment), and nine real uses in its own `long/`-homed witness — declined before the fold,
/// never executing. NO EXECUTING witness called any function that lens declared. Dead, not
/// competing; §3's attractor argument did not bite.
///
/// WHAT THE CUT OWED THIS ROSTER, AND PAID: the four identities are the SURVIVING authority.
/// The deletion removed the dead copy and none of the evidence — DESIGN §4b(4) keeps a
/// discriminating control enrolled when its machinery goes; treating the controls as part of
/// the funeral would have erased the four probes proving this wall's refusal arms are real.
/// The seven debt rows below were RE-POINTED rather than deleted in the same change: their
/// citations are deliberately false and moved with their witnesses into
/// `test.claim.long.decl_ref_resolution_witness_test` and
/// `test.claim.long.carrier_reference_integrity_witness_test`. A deleted row would be a
/// silently dropped obligation; a stale one would refuse, which is how the contract catches
/// exactly this.
///
/// FOUND BY MEASUREMENT, AND THE PROSE THAT SHOULD HAVE SAID SO WAS FALSE. The roster's doc
/// comment claimed "FOUR ROWS AT THE END ARE NOT DEBT ... the deleted census's own planted
/// controls". Enumerating all 38 rows finds no such row: described, never added. The first
/// corpus run reported all four controls as ordinary refusals, which caught the claim. A false
/// statement inside the carrier built to stop false statements is the specimen this change
/// exists to make impossible; recorded here rather than quietly corrected.
/// RE-OCCUPIED BY #10706's OutsideModeledGuarantee stamps (repair on the same subject as the
/// parse/call-shape floor red). Those stamps cite a `required_capability` that MUST stay
/// absent: `guarantee_boundary_still_outside` is true only on `DeclarationRefDeclarationAbsent`,
/// and `construction_justification_rule` says authoring the capability makes the stamp wrong.
/// The citations are therefore deliberately false — planted controls, not debt and not
/// missing declarations. Enrolling them here is the other half #10706 omitted: without these
/// rows the declarations phase refuses the same absences the join requires.
///
/// THE ARM'S OWN FIXTURE EVIDENCE DOES NOT LIVE IN THIS ROSTER.
/// `planted_control_findings_against` takes the roster as a parameter, and
/// `a_planted_control_that_still_refuses_is_healthy` /
/// `a_planted_control_that_resolves_has_lost_its_power_and_refuses` drive both directions from
/// controlled fixtures authoring their own rows. §4b(4): a climb dissolves production
/// machinery, never that evidence.
///
/// Site grain: `(citing_module, in_declaration, cited_module, cited_decl, field)`. Two
/// DeclarationRef literals inside one stamp that name the same absent symbol share one row.
const PLANTED_CONTROL_CITATIONS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "v2.lens.enforcement.complexity_contract_subject",
        "complexity_optimality_boundary",
        "v2.lens.enforcement.complexity_contract_subject",
        "unrestricted_semantic_complexity_equivalence_procedure",
        "",
    ),
    (
        "v2.lens.grounding",
        "grounding_name_only_residual_boundary",
        "v2.lens.grounding",
        "confirm_judge_should_ground",
        "",
    ),
    (
        "v2.lens.synthesis",
        "synthesis_rice_residual_boundary",
        "v2.lens.synthesis",
        "unrestricted_cheaper_equivalent",
        "",
    ),
    (
        "v2.test.claim.construction_justification.outside_modeled_guarantee_witness_test",
        "nonexistent_capability_ref",
        "v2.lens.cost",
        "capability_absent_from_decl_facts",
        "",
    ),
];

/// A control that has STOPPED refusing has lost its discriminating power, and that is a red in
/// its own right — the inverse of a spent debt row, and the reason these are a separate roster.
pub fn planted_control_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    planted_control_findings_against(index, PLANTED_CONTROL_CITATIONS)
}

pub fn planted_control_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    let still_refusing = refusing_sites(index);
    roster
        .iter()
        .filter(|row| !still_refusing.contains(&site_owned(row)))
        .map(|(citer, in_decl, module, decl, field)| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::PlantedControlNoLongerRefuses,
            rel_path: "src/v1/stage0/src/declaration_index.rs".to_string(),
            offset: None,
            message: format!(
                "PLANTED_CONTROL_CITATIONS lists `{citer}` `{in_decl}` citing `{module}` `{decl}`{} as a \
                 control that must NOT resolve, and it no longer refuses — the control has \
                 lost its discriminating power and the mechanism it proves is now unevidenced",
                if field.is_empty() {
                    String::new()
                } else {
                    format!(" field `{field}`")
                }
            ),
        })
        .collect()
}

/// The debt join, over an EXPLICIT roster.
///
/// The roster is a parameter rather than a constant read from inside, which is what makes this
/// arm's red authorable at the fixture boundary. A fixture tree of a few `probe.*` modules
/// joined against the 42-row production roster makes every row trivially absent, and the arm
/// reports 38 stale rows saying nothing about the fixture. Passing the roster lets a fixture
/// author a ONE-ROW roster and plant both directions — a row whose citation still refuses
/// (live, no finding) and one whose does not (spent, one finding) — the discriminating pair,
/// unauthorable while the roster is baked in.
pub fn citation_debt_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    citation_debt_findings_named(index, roster, "PRE_EXISTING_CITATION_DEBT")
}

/// The same arm, told which roster carried the row, so the diagnostic names the list the
/// reader must edit rather than sending them to a file that does not contain the row.
pub fn citation_debt_findings_named(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str, &str, &str)],
    roster_name: &str,
) -> Vec<DeclarationIntegrityFinding> {
    let live = refusing_sites(index);
    roster
        .iter()
        .filter(|row| !live.contains(&site_owned(row)))
        .map(|(citer, in_decl, module, decl, field)| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::CitationDebtRowStale,
            rel_path: "src/v1/stage0/src/declaration_index.rs".to_string(),
            offset: None,
            message: format!(
                "{roster_name} still lists `{citer}` `{in_decl}` citing `{module}` `{decl}`{} — that \
                 citation no longer refuses, so the row is spent and must be deleted; the \
                 roster only shrinks",
                if field.is_empty() {
                    String::new()
                } else {
                    format!(" field `{field}`")
                }
            ),
        })
        .collect()
}

/// (2) The cited-symbol wall — §3's cite-the-symbol rule, executing.
///
/// A citation resolves against the DECLARED surface of the module it names, never its
/// re-exports: a citation names where a fact LIVES, and a module merely importing a name is
/// not its authority (§3).
///
/// ONE RESOLUTION, TWO CONSUMERS. The wall and the debt contract both read
/// `citation_resolution_refusal` — a second predicate agreeing today is a fork disagreeing
/// later, the objection `v2.lens.cited_symbol_resolution` recorded against a second resolver.
pub fn cited_symbol_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    cited_symbol_findings_against(index, &[])
}

/// The citation wall, with the ENROLLED-DEBT roster passed in.
///
/// SAME CLASS AS THE DEBT JOIN BELOW, FOUND BY ASKING THE QUESTION ONE LEVEL UP. That arm read
/// its roster from a constant inside the function; this one suppressed citations against the
/// same constant, silently, unvariable by any fixture. Suppression is the more dangerous: a
/// spurious refusal is loud, a spurious SUPPRESSION is a citation the wall quietly declines to
/// judge — the arm could stop enrolling a whole class with every fixture still passing.
///
/// So the roster is a parameter here too, defaulting to EMPTY rather than the production
/// constant — the denominator argument again: an arbitrary swept tree has no pre-existing
/// debt, so `index_findings` judges every citation, and only `corpus_findings` — whose subject
/// IS this repository — passes the roster suppressing this repository's enrolled rows. One
/// roster, reachable from one place.
///
/// THE RULE THE EMPTY DEFAULT ENCODES, stated so a later reader does not flip it back for
/// convenience: **a policy roster passed as a parameter defaults to the IDENTITY ELEMENT OF
/// THE JUDGMENT, never to the production value.** Empty means judge everything, the strictest
/// answer, so a caller forgetting the roster gets MORE refusals. Defaulted to the production
/// roster, the forgetful caller gets silent suppression — the defect this parameter closes,
/// reintroduced through the default. That makes this a construction move, not a tidier
/// signature: the fail-closed direction is a property of the type's default, not of
/// discipline.
///
/// The seam this creates — that `corpus_findings` really passes the production roster — needs
/// its own evidence, and has it:
/// `corpus_findings_is_wired_to_the_production_suppression_roster` declares a module the
/// roster names and requires the same tree refused unenrolled and suppressed enrolled.
pub fn cited_symbol_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    let mut out = Vec::new();
    for record in index.modules.values() {
        for cited in &record.cited {
            if citation_in_roster(&record.module_path, cited, roster) {
                continue;
            }
            if let Some(finding) = citation_resolution_refusal(index, record, cited) {
                out.push(finding);
            }
        }
    }
    out
}

/// The typed refusal one citation earns, or `None` if it resolves.
fn citation_resolution_refusal(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    cited: &CitedSymbol,
) -> Option<DeclarationIntegrityFinding> {
    let Some(target) = resolve_cited_module(index, &cited.module_path) else {
        if citation_is_outside_index(index, &cited.module_path) {
            return None;
        }
        return Some(DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::CitedModuleAbsent,
            rel_path: record.rel_path.clone(),
            offset: span_offset_in(&cited.location, &record.rel_path),
            message: format!(
                "`{}` cites `{}` `{}`, and no module declares `{}`",
                record.module_path, cited.module_path, cited.decl_name, cited.module_path
            ),
        });
    };
    if !target.declared.contains(&cited.decl_name) && !target.variants.contains(&cited.decl_name) {
        return Some(DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::CitedDeclarationAbsent,
            rel_path: record.rel_path.clone(),
            offset: span_offset_in(&cited.location, &record.rel_path),
            message: format!(
                "`{}` cites `{}` `{}`, which that module does not declare",
                record.module_path, cited.module_path, cited.decl_name
            ),
        });
    }
    if let Some(field) = cited.field.as_ref() {
        let present = target
            .decl_fields
            .get(&cited.decl_name)
            .is_some_and(|fields| fields.contains(field));
        if !present {
            return Some(DeclarationIntegrityFinding {
                kind: DeclarationIntegrityKind::CitedFieldAbsent,
                rel_path: record.rel_path.clone(),
                offset: span_offset_in(&cited.location, &record.rel_path),
                message: format!(
                    "`{}` cites field `{field}` of `{}` `{}`, which has no such field",
                    record.module_path, cited.module_path, cited.decl_name
                ),
            });
        }
    }
    None
}

/// (3) Module authorship — one module's fact, from one module's source.
pub fn lens_authorship_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    index
        .modules
        .values()
        .filter(|r| is_top_level_lens_module(&r.module_path))
        .filter(|r| !r.declares_construction_justification)
        .map(|r| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::LensAuthorshipAbsent,
            rel_path: r.rel_path.clone(),
            offset: None,
            message: format!(
                "lens `{}` declares no `{CONSTRUCTION_JUSTIFICATION_DECL}` — a lens is \
                 validation, and DESIGN §6 requires it to record why construction was \
                 unavailable",
                r.module_path
            ),
        })
        .collect()
}

pub fn duplicate_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    index
        .duplicates
        .iter()
        .map(|(module_path, first, second)| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::DuplicateModuleDeclaration,
            rel_path: second.clone(),
            offset: None,
            message: format!("module `{module_path}` is also declared by `{first}`"),
        })
        .collect()
}

/// Every finding, ordered so the report is stable across runs.
/// Every finding DERIVABLE FROM THE INDEX ITSELF — the four arms whose subject is the tree
/// that was swept, whatever tree that is.
///
/// THE DEBT ARM IS DELIBERATELY NOT HERE — a denominator distinction. The other four arms ask
/// ABOUT THE SWEPT MODULES (does this import member exist, does this citation resolve, does
/// this lens carry its authorship fact), answered by the modules in front of them, so
/// meaningful over any tree. `PRE_EXISTING_CITATION_DEBT` is a fact about ONE SPECIFIC CORPUS
/// — the repository's own — and joined against another tree it answers a question nobody
/// asked: a fixture tree of `probe.*` modules makes all 42 rows trivially absent, so the arm
/// reports 38 spent rows saying nothing about the fixture and drowning every real finding.
///
/// That is the failure `review 55817` found: with the debt arm folded in here, every fixture in
/// `tests/declaration_index_integrity.rs` received 42 findings it did not plant, the
/// planted-red and positive-control assertions could not pass, and the §4b fixture-boundary
/// evidence did not execute. The repair is not to gate the arm on a corpus-shape signal — a
/// smuggled heuristic (§4: a heuristic is never necessary in a closed system) — but to put the
/// arm where its denominator is, in `corpus_findings` below, with a roster parameter so its
/// own red stays authorable.
pub fn index_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    let mut out = duplicate_findings(index);
    out.extend(import_member_findings(index));
    out.extend(cited_symbol_findings(index));
    out.extend(lens_authorship_findings(index));
    out.sort();
    out
}

/// PAIRED INVERSE ARMS MUST RUN IN ONE REPORT OVER ONE SUBJECT SET.
///
/// Four arms in two inverse pairs, the pairing load-bearing. `cited_symbol_findings_against`
/// SUPPRESSES what a roster enrolls; `citation_debt_findings` refuses a roster row nothing
/// enrolls; `planted_control_findings` reads the same trigger as the latter in the opposite
/// direction. Each arm is a partial answer, individually honest.
///
/// SPLITTING A PAIR ACROSS SEPARATE CHECKS DOES NOT WEAKEN IT, IT DESTROYS IT — different jobs,
/// cadences, or roster arguments all have that effect, usually for good job-granularity
/// reasons. The receipt: one roster row with an empty field where its citation carries
/// `NamedField { "price" }` desynchronized the arms — the suppression arm reported the citation
/// as UNENROLLED DEBT while the staleness arm reported its row as SPENT, in one run. Both
/// locally correct, both wrong. NEITHER COULD DETECT IT ALONE; the only observable is the two
/// answers present together and disagreeing.
///
/// So the arms share this one entry point and subject set, and the desynchronization is
/// asserted rather than left to a reader noticing two report lines:
/// `a_roster_row_on_the_wrong_identity_desynchronizes_both_arms` plants the exact mismatch and
/// requires BOTH findings, then repairs the row and requires NEITHER.
///
/// What a run OVER THIS REPOSITORY must answer: everything the index derives, plus the debt
/// contract, whose subject universe is this corpus. This is what the required run and the
/// standalone sweep call; nothing else should.
pub fn corpus_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    let mut out = duplicate_findings(index);
    out.extend(import_member_findings(index));
    out.extend(cited_symbol_findings_against(
        index,
        &[
            PRE_EXISTING_CITATION_DEBT,
            PLANTED_CONTROL_CITATIONS,
            FIXTURE_CARRIER_CITATION_EXEMPTIONS,
        ]
        .concat()[..],
    ));
    out.extend(lens_authorship_findings(index));
    out.extend(citation_debt_findings(index));
    out.extend(citation_debt_findings_named(
        index,
        FIXTURE_CARRIER_CITATION_EXEMPTIONS,
        "FIXTURE_CARRIER_CITATION_EXEMPTIONS",
    ));
    out.extend(planted_control_findings(index));
    out.sort();
    out
}

/// A span's offset when it names the file the record came from; `None` otherwise, so a
/// synthetic span renders as the path alone rather than as a fabricated `1:1`.
fn span_offset_in(location: &SourceLocation, rel_path: &str) -> Option<i64> {
    if location.file == rel_path
        || location.file.ends_with(rel_path)
        || rel_path.ends_with(&location.file)
    {
        Some(location.offset)
    } else {
        None
    }
}

/// Render one finding as a typed, LOCATED refusal line.
///
/// The offset becomes `line:col` by reading the offending file only — a finding is rare, and
/// holding the corpus's text resident so a green run could render nothing is the cost shape
/// this module exists to remove. A finding with no offset renders as the path alone, never a
/// fabricated `1:1`.
pub fn render_finding(
    workspace: &std::path::Path,
    finding: &DeclarationIntegrityFinding,
) -> String {
    let located = match finding.offset {
        Some(offset) => match std::fs::read_to_string(workspace.join(&finding.rel_path)) {
            Ok(content) => {
                let index =
                    crate::v1_std_core::build_newline_index(finding.rel_path.clone(), content);
                let lc = crate::v1_std_core::byte_to_line_col(index, offset);
                format!("{}:{}:{}", finding.rel_path, lc.line, lc.col)
            }
            Err(_) => finding.rel_path.clone(),
        },
        None => finding.rel_path.clone(),
    };
    format!(
        "{} {}: {}",
        integrity_kind_label(&finding.kind),
        located,
        finding.message
    )
}

/// Every citation SITE in the corpus that currently refuses, as owned identities.
///
/// ONE SET, BOTH INVERSE ARMS. The debt arm and the planted-control arm read it in opposite
/// directions and must read the SAME subject set — two separately-built sets are the easiest
/// way to reintroduce the desynchronization receipt on `corpus_findings`.
fn refusing_sites(index: &DeclarationIndex) -> BTreeSet<(String, String, String, String, String)> {
    let mut out = BTreeSet::new();
    for record in index.modules.values() {
        for cited in &record.cited {
            if citation_resolution_refusal(index, record, cited).is_some() {
                let (a, b, c, d, e) = citation_site(&record.module_path, cited);
                out.insert((
                    a.to_string(),
                    b.to_string(),
                    c.to_string(),
                    d.to_string(),
                    e.to_string(),
                ));
            }
        }
    }
    out
}

fn site_owned(row: &(&str, &str, &str, &str, &str)) -> (String, String, String, String, String) {
    (
        row.0.to_string(),
        row.1.to_string(),
        row.2.to_string(),
        row.3.to_string(),
        row.4.to_string(),
    )
}

// GATE 1 OF THE IMPORT-DECLARATION CUT: an import declaration must have ZERO binding authority.
//
// IT LIVES HERE, AND AS A TEST, FOR A REASON THAT WAS RULED ON. An earlier attempt widened the
// multi-module compile fixture's public result with a ResolvedCallTarget carrier so a `.dag` witness
// could read the bound declaration. That was REFUSED on PURPOSE admission: v1_maintenance_standing's
// refused classes DOMINATE the admitted axis, and PublicSurfaceGrowth applied. The carrier was also
// unnecessary -- compile_to_resolved, ResolvedGraph, Node.expr_data and CallSemantics are all
// reachable from a crate test, so the observation type can be PRIVATE TO THIS MODULE and the seed
// grows by nothing. The `.dag` route is separately closed: a nested compile from `.dag` refuses with
// NoSuchField Node.ident, which is why both existing nested-compile instruments route through a host
// builtin -- that explains the closure, it does not license a builtin when a Rust test suffices.
//
// WHY THE SIGNATURES ARE IDENTICAL. cut.a and cut.b declare ONE spelling with the same arity and
// types, so an illicit selection stays COMPILE-GREEN and only the resolver's own recorded target
// separates them. Accept/refuse cannot express this subject: it sees a MISSING selection, never a
// WRONG one. Nothing here reads diagnostics, counts, emitted Rust, or a rendered string.
#[cfg(test)]
mod import_binding_authority_tests {
    use crate::v1_compiler_compile::{compile_to_resolved, SourceFile};
    use crate::v1_std_core::{CallSemantics, CallTargetIdentity, ExprData, Node};
    use std::rc::Rc;

    // The pipeline takes the persistent vector, not std's.
    use im::Vector;

    /// Every distinct state the observation can be in. PRIVATE, so this costs no seed surface.
    ///
    /// `CallSemanticsAbsent` STAYS DISTINCT FROM `TargetUndetermined`. "The call carries no
    /// semantics at all" and "lookup ran and could not determine a target" are different facts, and
    /// collapsing them would let a DELETED SELECTOR or a SKIPPED PHASE pass as successful
    /// nonbinding -- which is exactly the false green the cut must not be able to produce.
    ///
    /// `FunctionValueCall` is its own arm because `CallSemantics::target()` PANICS on that variant.
    /// Matching the constructor is therefore not a stylistic choice over calling the accessor; the
    /// accessor is unsafe here by construction.
    #[derive(Debug, Clone, PartialEq)]
    enum ObservedTarget {
        SourceDeclarationTarget {
            owner_module_path: String,
            decl_name: String,
        },
        RuntimePrimitiveTarget {
            primitive_name: String,
        },
        TargetUndetermined,
        FunctionValueCall,
        CallSemanticsAbsent,
        NoCallNodeFound,
        CompileRefused {
            diagnostic_count: usize,
        },
    }

    fn source(path: &str, content: &str) -> Rc<SourceFile> {
        Rc::new(SourceFile {
            path: path.to_string(),
            content: content.to_string(),
        })
    }

    /// Depth-first over every child list a call can hide in. The direct child of a call argument is
    /// a wrapper whose children carry the value, so a shallow walk misses real calls.
    fn find_first_call(node: &Rc<Node>, out: &mut Vec<Rc<Node>>) {
        if matches!(*node.expr_data, ExprData::ExprCall { .. }) {
            out.push(node.clone());
        }
        for child in node.children.iter() {
            find_first_call(child, out);
        }
        for param in node.params.iter() {
            find_first_call(param, out);
        }
        if let Some(body) = node.body.as_ref() {
            find_first_call(body, out);
        }
    }

    /// The observation, read from what the resolver itself recorded.
    fn observe_bare_call(entry: &str) -> ObservedTarget {
        let sources = Rc::new(Vector::from(vec![
            source(
                "cut/a.dag",
                "module cut.a\n\nfn cut_probe(cut_v: Int) -> Int { cut_v }\n",
            ),
            source(
                "cut/b.dag",
                "module cut.b\n\nfn cut_probe(cut_v: Int) -> Int { cut_v }\n",
            ),
            source(
                "cut/c.dag",
                "module cut.c\n\nfn cut_unrelated_marker(cut_w: Int) -> Int { cut_w }\n",
            ),
            source("cut/entry.dag", entry),
        ]));
        let resolved = compile_to_resolved(sources);
        let graph = match resolved.graph.as_ref() {
            Some(g) => g.clone(),
            // A REFUSAL IS ITS OWN STATE, never folded into "no target". Reporting a refused
            // compile as nonbinding would let the cut green by breaking the compiler.
            None => {
                return ObservedTarget::CompileRefused {
                    diagnostic_count: resolved.diagnostics.len(),
                }
            }
        };

        let mut calls: Vec<Rc<Node>> = Vec::new();
        for module in graph.modules.iter() {
            if module.module.name != "cut.entry" {
                continue;
            }
            for item in module.items.iter() {
                find_first_call(item, &mut calls);
            }
        }

        let call = match calls.first() {
            Some(c) => c.clone(),
            None => return ObservedTarget::NoCallNodeFound,
        };

        let semantics = match &*call.expr_data {
            ExprData::ExprCall { call_semantics, .. } => call_semantics.clone(),
            _ => return ObservedTarget::NoCallNodeFound,
        };

        let semantics = match semantics {
            Some(s) => s,
            None => return ObservedTarget::CallSemanticsAbsent,
        };

        let target = match &*semantics {
            CallSemantics::PlainCallSemantics { target } => target.clone(),
            CallSemantics::ResolvedDirectCallSemantics { target, .. } => target.clone(),
            CallSemantics::LookupCallSemantics { target } => target.clone(),
            CallSemantics::FunctionValueCallSemantics => return ObservedTarget::FunctionValueCall,
        };

        match &*target {
            CallTargetIdentity::SourceDeclarationCall {
                owner_module_path,
                decl_name,
            } => ObservedTarget::SourceDeclarationTarget {
                owner_module_path: owner_module_path.clone(),
                decl_name: decl_name.clone(),
            },
            CallTargetIdentity::RuntimePrimitiveCall { primitive_name, .. } => {
                ObservedTarget::RuntimePrimitiveTarget {
                    primitive_name: primitive_name.clone(),
                }
            }
            CallTargetIdentity::CallableTargetUndetermined => ObservedTarget::TargetUndetermined,
        }
    }

    /// The builtin-rival subjects. `string_length` is a real builtin, so a bare call to it can
    /// reach `builtin_callable_candidates` -- which is what makes the third visibility state
    /// observable at all.
    const ENTRY_BUILTIN_NO_IMPORTS: &str =
        "module cut.entry\n\nfn cut_use() -> Int { string_length(cut_s: \"ab\") }\n";
    const ENTRY_BUILTIN_UNLISTED: &str = "module cut.entry\n\nimport cut.c { cut_unrelated_marker }\n\nfn cut_use() -> Int { string_length(cut_s: \"ab\") }\n";
    const ENTRY_BUILTIN_UNLISTED_WITH_RIVAL: &str = "module cut.entry\n\nimport cut.c { cut_unrelated_marker }\n\nfn string_length(cut_s: String) -> Int { 7 }\n\nfn cut_use() -> Int { string_length(cut_s: \"ab\") }\n";
    const ENTRY_BUILTIN_RIVAL_NO_IMPORTS: &str = "module cut.entry\n\nfn string_length(cut_s: String) -> Int { 7 }\n\nfn cut_use() -> Int { string_length(cut_s: \"ab\") }\n";

    const ENTRY_IMPORTS_A: &str = "module cut.entry\n\nimport cut.a { cut_probe }\n\nfn cut_use() -> Int { cut_probe(cut_v: 1) }\n";
    const ENTRY_IMPORTS_B: &str = "module cut.entry\n\nimport cut.b { cut_probe }\n\nfn cut_use() -> Int { cut_probe(cut_v: 1) }\n";
    const ENTRY_IMPORTS_NONE: &str =
        "module cut.entry\n\nfn cut_use() -> Int { cut_probe(cut_v: 1) }\n";
    const ENTRY_IMPORTS_AC: &str = "module cut.entry\n\nimport cut.a { cut_probe }\nimport cut.c { cut_unrelated_marker }\n\nfn cut_use() -> Int { cut_probe(cut_v: 1) }\n";
    const ENTRY_IMPORTS_CA: &str = "module cut.entry\n\nimport cut.c { cut_unrelated_marker }\nimport cut.a { cut_probe }\n\nfn cut_use() -> Int { cut_probe(cut_v: 1) }\n";

    /// CONTROL 0 -- THE FIXTURE IS LIVE. Without this, every arm agreeing on `NoCallNodeFound`
    /// would satisfy the invariant for the worst possible reason.
    #[test]
    fn the_bare_call_is_observed_at_all() {
        assert_eq!(
            observe_bare_call(ENTRY_IMPORTS_A),
            ObservedTarget::SourceDeclarationTarget {
                owner_module_path: "cut.a".to_string(),
                decl_name: "cut_probe".to_string(),
            }
        );
    }

    /// THE QUALIFIED CONTROL: the exact constructor AND payload, not a rendered string.
    #[test]
    fn the_reported_owner_is_the_resolved_one_and_not_the_consumer() {
        match observe_bare_call(ENTRY_IMPORTS_B) {
            ObservedTarget::SourceDeclarationTarget {
                owner_module_path, ..
            } => {
                assert_ne!(owner_module_path, "cut.entry");
            }
            other => panic!("expected a source declaration target, got {other:?}"),
        }
    }

    /// ORDER IS ALREADY INERT, and this arm is GREEN TODAY. It stays enrolled as an ordinary
    /// regression control rather than retiring because it happens to pass (DESIGN §4b(4)).
    #[test]
    fn reordering_import_declarations_does_not_change_the_bare_call() {
        assert_eq!(
            observe_bare_call(ENTRY_IMPORTS_AC),
            observe_bare_call(ENTRY_IMPORTS_CA)
        );
    }

    /// THE MEASURED PRE-CUT STATE, ASSERTED AS WHAT IT IS RATHER THAN AS THE INVARIANT.
    ///
    /// This is NOT the gate-1 invariant and must not be read as one. `rust-unit-tests` is a `needs`
    /// of the required aggregate, so a test that is red today would red the required lane for
    /// everyone; the gate-1 invariant is therefore DEMONSTRATED at review time and lands WITH the
    /// cut. What this records is that the three arms are DISTINGUISHABLE today -- which is the fact
    /// that makes the invariant's RED authorable at all. A wall whose red was never authorable is a
    /// decoration, and worse than absent because it gets cited as coverage.
    ///
    /// It is deliberately NOT written as `imports_still_decide_binding`: an inverted green test
    /// ratifies the defect as the assertion, and when it flips the red is ambiguous between "the cut
    /// landed" and "something else broke".
    #[test]
    fn the_three_arms_are_distinguishable_before_the_cut() {
        let with_a = observe_bare_call(ENTRY_IMPORTS_A);
        let with_b = observe_bare_call(ENTRY_IMPORTS_B);
        let with_none = observe_bare_call(ENTRY_IMPORTS_NONE);
        assert_ne!(with_a, with_b, "import selection is live today");
        assert_ne!(with_a, with_none, "import presence is live today");
    }

    /// A `TypeEnv` identical in every field except `authored_import_names`. The two arms of the
    /// gate below are built only through this, so "the arms differ in one field" is enforced by
    /// construction rather than asserted in a comment.
    fn type_env_with_authored(names: &[&str]) -> Rc<crate::v1_compiler_infer_env::TypeEnv> {
        let base = crate::v1_compiler_infer_env::empty_type_env();
        let mut authored = crate::v1_rt::empty_map::<String, bool>();
        for n in names {
            authored = crate::v1_rt::map_insert(authored, n.to_string(), true);
        }
        Rc::new(crate::v1_compiler_infer_env::TypeEnv {
            authored_import_names: Rc::new(authored),
            ..(*base).clone()
        })
    }

    /// The declaration must sit in a PARENT env: `callable_lookup_over_candidates` checks
    /// `func_env.local` first and returns immediately on a hit, so a local declaration never
    /// reaches the authored-membership branch. `v2.std.collection` / `map_get` is not an arbitrary
    /// choice -- it is the specimen carrying `DivergentProjection` fidelity, which is the condition
    /// `declared_candidate_rivals_the_builtin` requires.
    fn parent_env_declaring_map_get() -> Rc<crate::v1_compiler_infer_sigs::ResolvedFuncEnv> {
        use crate::v1_compiler_infer_sigs::{ResolvedFormals, ResolvedFuncEnv, ResolvedFuncSig};
        let sig = Rc::new(ResolvedFuncSig {
            name: "map_get".to_string(),
            params: crate::v1_std_core::empty_node_list(),
            // This synthetic declaration deliberately has no parameters. Its empty formal
            // population is kernel-grounded, not awaiting a module-context resolution pass.
            resolved_formals: Rc::new(ResolvedFormals::KernelGroundedFormals {
                formals: Rc::new(im::Vector::new()),
            }),
            inferred: crate::v1_std_core::unit_type(),
            is_async: false,
            output_provenance: Rc::new(im::Vector::new()),
            variant_provenance: crate::v1_rt::rc_empty_map(),
        });
        let parent = Rc::new(ResolvedFuncEnv {
            name: "v2.std.collection".to_string(),
            local: Rc::new(crate::v1_rt::map_insert(
                crate::v1_rt::empty_map(),
                "map_get".to_string(),
                sig,
            )),
            parents: Rc::new(im::Vector::new()),
        });
        Rc::new(ResolvedFuncEnv {
            name: "cut.entry".to_string(),
            local: crate::v1_rt::rc_empty_map(),
            parents: Rc::new(im::Vector::from(vec![parent])),
        })
    }

    /// THE AUTHORED-MEMBERSHIP DECISION GATE, OBSERVED AT THE FuncSigLookup SEAM.
    ///
    /// WHY NOT AT TARGET IDENTITY -- MEASURED, NOT ASSUMED. Two arms built over `CallTargetIdentity`
    /// were executed and BOTH FAILED TO DISCRIMINATE, for two independent structural reasons. A
    /// locally declared rival never reaches the visibility branch at all, because
    /// `callable_lookup_over_candidates` checks `func_env.local` FIRST and returns on a hit. And on
    /// the zero-declared arm the builtin IS admitted, but a lone builtin is deliberately mapped back
    /// to `FuncSigUnresolved` (the registry carries no declared parameter list), after which
    /// `builtin_call_target_or_undetermined` reconstructs `RuntimePrimitiveCall` FROM THE NAME. So
    /// both the lookup verdict and the projected target collapse the difference in that population.
    /// That is an OBSERVATION-BOUNDARY defect, not a bad fixture spelling.
    ///
    /// AND IT DOES NOT MEAN THE POLICY IS DEAD. That convenient reading is refuted by a specimen in
    /// the tree: `map_get` is declared at `v2.std.collection` with `DivergentProjection` fidelity in
    /// `std.primitive_projection`, which is exactly the condition
    /// `declared_candidate_rivals_the_builtin` tests. So the collision is real, reachable from
    /// authored source, and already witnessed by `callable_candidate_ambiguity_witness_test`.
    ///
    /// THIS CONTROL ISOLATES THE CAUSE. Both arms share the same `ResolvedFuncEnv` carrying one
    /// PARENT declaration (parent, not local, or the early return fires) and the same `TypeEnv` in
    /// every other field. They differ in `authored_import_names` AND NOTHING ELSE, so a difference
    /// in the verdict is attributable to authored membership alone.
    #[test]
    fn authored_membership_alone_decides_the_callable_verdict() {
        use crate::v1_compiler_infer_lookup::callable_lookup_over_candidates;
        use crate::v1_compiler_infer_sigs::{CallableIdentity, FuncSigLookup};

        let env = parent_env_declaring_map_get();

        let named = callable_lookup_over_candidates(
            env.clone(),
            type_env_with_authored(&["map_get"]),
            "map_get".to_string(),
        );
        let omitted = callable_lookup_over_candidates(
            env,
            type_env_with_authored(&["cut_unrelated_marker"]),
            "map_get".to_string(),
        );

        // THE DIRECTION IS ASSERTED, NOT MERE INEQUALITY. A control demanding only that the arms
        // differ would pass if BOTH moved to the wrong verdict together -- the shared-corruption
        // shape a comparison cannot detect.
        match &*named {
            FuncSigLookup::FuncSigResolved { declared, .. } => {
                assert_eq!(declared.owner_module_path, "v2.std.collection");
                assert_eq!(declared.decl_name, "map_get");
            }
            other => panic!(
                "naming the callable must suppress the builtin co-candidate and resolve to the \
                 declaration, got {other:?}"
            ),
        }

        // AND THE AMBIGUOUS ARM IS CHECKED BY ITS EXACT IDENTITY SET, never by a count and never by
        // list order. A count would pass on a substitution, and an order-sensitive comparison would
        // make candidate ordering -- which nothing declares to be semantic -- part of the assertion.
        match &*omitted {
            FuncSigLookup::FuncSigAmbiguous { candidates, .. } => {
                let mut observed: Vec<String> = candidates
                    .iter()
                    .map(|c| match &*c.identity {
                        CallableIdentity::DeclaredCallable { identity } => format!(
                            "declared:{}:{}",
                            identity.owner_module_path, identity.decl_name
                        ),
                        CallableIdentity::BuiltinCallable { primitive_name } => {
                            format!("builtin:{primitive_name}")
                        }
                    })
                    .collect();
                observed.sort();
                observed.dedup();
                assert_eq!(
                    observed,
                    vec![
                        "builtin:map_get".to_string(),
                        "declared:v2.std.collection:map_get".to_string(),
                    ],
                    "the two rival authorities must both be present, and only those two"
                );
            }
            other => panic!(
                "omitting it from a NONEMPTY authored list must admit the builtin rival and go \
                 ambiguous, got {other:?}"
            ),
        }
    }
}

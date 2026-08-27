//! ONE per-module declaration index, built where each module is already parsed.
//!
//! WHY THIS EXISTS AT ALL, and why it is one module rather than three checks.
//! DESIGN carries two next-rung triggers that name the same construction from two
//! directions. §6, on the deleted inert-lens and construction-justification censuses:
//! "an authorship fact belongs on the module's own declaration, checked at ingestion
//! where the module is parsed anyway — one module's facts from one module's source —
//! rather than reconstructed corpus-wide by a consumer that wanted something else."
//! §3's cited-symbol rung-drop row, on the deleted `--required-cited-symbol` job:
//! "this row retires when the citation wall is re-derived where the operator's own
//! framing puts it — *you would just make them a normal compiler error* — checked at
//! ingestion, on the module whose source carries the citation, from that module's own
//! text, rather than reconstructed corpus-wide by a second job. That is the same
//! next-rung trigger §6 already names for module authorship facts, and the two should
//! land together rather than each rebuilding a corpus walk."
//!
//! THE COST SHAPE THE TWO TRIGGERS ARE BOTH COMPLAINING ABOUT. Every mechanism that
//! wanted a per-module fact acquired the whole corpus to get it: `decl_facts(roots)`
//! walks and re-parses every `.dag` file to produce a FLAT `Vec<DeclFact>`, and
//! `module_declaration_facts(roots)` walks them again for a flat `Vec` of module rows;
//! the cited-symbol resolver then answers each reference by a LINEAR SCAN over both.
//! That is §6's cost-shape defect exactly — the unit of computation was the world and
//! the unit of fact was one module — and it is why the checks kept being authored as
//! separate corpus-wide jobs instead of as ingestion facts.
//!
//! WHAT IS CONSTRUCTED HERE. `run_dag_parse_sweep` already parses every `.dag` file
//! under `DAG_PARSE_SWEEP_ROOTS`, once, in parallel, on every required run — and threw
//! the parse tree away. This module turns that one existing parse into one
//! `ModuleDeclarationRecord` per module: what the module DECLARES, what it CLAIMS from
//! other modules (its import members), what it CITES (`DeclarationRef` literals in its
//! own text), and the authorship fact it carries. Three integrity questions are then
//! answered from that single index, by keyed lookup rather than by linear scan:
//!
//!   1. import-member claim integrity — `import m { X }` where `m` declares no `X`
//!   2. the cited-symbol wall — an authored `DeclarationRef` naming a symbol that
//!      does not resolve (§3: cite the symbol, not the position)
//!   3. module authorship — a top-level lens with no `construction_justification`
//!
//! WHAT THIS IS NOT. It is not a widening of the required floor's source roots, and
//! the objection `gunbc.ci_layer_roots` `v1_dead_witness_tree_triage_receipt_remainder`
//! raises against that cannot reach it, for the same reason it cannot reach the parse
//! sweep it rides on: NOTHING HERE RESOLVES ACROSS FILES IN THE COMPILER'S SENSE. Each
//! record is derived from one file's own tree; the index is a map from the module path
//! that file DECLARES to that file's own facts. Two roots colliding on a last segment
//! re-bind nothing, because no bare reference is ever resolved — only fully qualified
//! module paths are looked up, and a module path is unique or it is a duplicate the
//! index reports.
//!
//! WHY EVERY ROSTER ROW NAMES ITS CITING MODULE, AND NOT ONLY ITS TARGET.
//! The three suppression rosters below used to be keyed `(module, decl, field)` — the TARGET
//! of the citation. One row therefore exempted EVERY citation of that target, corpus-wide, for
//! as long as the row stood. That is not a narrower wall, it is an open one in a direction
//! nothing could observe: a patch could author a BRAND NEW dangling `DeclarationRef` naming any
//! enrolled target, from any module, and the wall would silently decline to judge it. The
//! violation is decidable from that patch alone — the site is new, and no row named it — so the
//! class was rot admitted by the mechanism built to refuse rot.
//!
//! IT WAS OCCUPIED, NOT MERELY REACHABLE, which is what settled the grain rather than the
//! argument. Measured over the live corpus through `DAG_PARSE_SWEEP_ROOTS`, the 70 target-keyed
//! rows covered 87 refusing sites, and seven targets were already cited from more than one
//! module: `gunbc.host_effect` `host_effect_apply` from three (`extdeps.github.actions_runner`,
//! `gunbc.executor_privileged_operation`, `gunbc.runner_slot_provision`), `std.bytes`
//! `builtin_function_registry` from three, `extdeps.network.mac` `parse_mac_address` from two
//! (`extdeps.dhcp.v4` and a witness), and four more from two apiece. Every one of those extra
//! sites was being suppressed by a row authored about a different module.
//!
//! THE ROSTERS ARE RE-DERIVED FROM THAT MEASUREMENT, AND THE FIRST DERIVATION WAS TAKEN OVER THE
//! WRONG DENOMINATOR — recorded because it is the same class this module keeps catching. The
//! sweep's roots are `src/v1`, `dag` and `src/v2`; the first measurement used only the last two,
//! so five sites authored in modules the narrow walk never read were absent from the rosters and
//! the required run refused them. A roster derived from a subset of the subject it governs is
//! not a smaller roster, it is a wrong one.
//!
//! So a row is `(citing_module, in_declaration, module, decl, field)` and it exempts THE SITE
//! THAT AUTHORED IT.
//! Both inverse arms read that same identity, because a suppression arm and a staleness arm
//! keyed differently is the desynchronization `corpus_findings` already carries a receipt for.
//!
//! THE DECLARATION IN THE ROW IS THE SECOND HALF OF THE SAME REPAIR, AND IT WAS A DISCLOSED
//! RESIDUE BEFORE IT WAS A CLOSED ONE (review 56227). Keyed on the citing MODULE alone, a row
//! still covered every citation of that target ANYWHERE IN THAT MODULE, so a new dangling
//! citation authored BESIDE an enrolled one was suppressed — the same fail-open one level in.
//! The reviewer's objection was that a residue with an available identity is not a residue, and
//! that is right: `record_from_module` already iterates top-level items, so the enclosing
//! declaration's name costs one string at extraction. It is a NAME, reachable from the
//! containment tree, and therefore not the positional citation DESIGN §3 forbids — an offset
//! would be finer and would rot on any edit above the line.
//!
//! WHAT REMAINS, stated because a closed residue must not be reported as a total one: two
//! citations of ONE target inside ONE declaration still share a row. Nothing short of a
//! position separates those, and a position is the thing this grain exists not to be, so this
//! is a ceiling rather than a stall — the class's next rung is a citation carrying an
//! occurrence ordinal within its declaration, which the ingestion record could hold but which
//! no measured site needs today.
//!
//! It is also not the compiler's own name resolution. `v1.03_resolve` already refuses
//! `MissingExport` for an import member inside a COMPILE CLOSURE; this index answers the
//! same question over the whole authored corpus, which is where the difference lives —
//! DESIGN's 2026-08-25 row records `gunbc.auth.credentials` standing on main with four
//! hard errors because NO CLOSURE REACHES IT. An orphan module's import claims are
//! checked here and nowhere else.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::v1_rt::VecCompat;
use crate::v1_std_core::{
    authored_name_at, expr_literal_string_optional, import_is_all, import_specific_names_at,
    module_imports, module_items, Connective, ExprData, NewlineIndex, Node, SourceSpan,
};

/// A span COPIED OUT of the parse tree rather than referenced into it.
///
/// The sweep parses each file on its own thread and hands the record back across a
/// thread boundary; the parse tree is `Rc`-shaped and cannot cross one. Copying the two
/// fields a location actually needs is not a loss — a record is a fact about a module,
/// and holding the whole tree alive to carry an offset would keep the entire corpus
/// resident for the sake of a `usize`.
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
    /// It is the finest STABLE site identity an ingestion record can hold. A byte offset would
    /// be finer and is refused on principle: DESIGN §3 forbids a positional citation precisely
    /// because it rots on any edit above the line, and a suppression roster keyed on one would
    /// go stale without anyone touching either end. A declaration name is reachable from the
    /// containment tree the namespace authority already walks, so it is the same kind of
    /// identity a citation itself is.
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
    /// `declared` deliberately: an import member may legitimately be a re-export, while a
    /// CITATION naming a re-export names the wrong authority (§3 — a fact's home is the
    /// module that declares it), so the two questions must not read one set.
    pub reexported: BTreeSet<String>,
    /// Declaration name -> the field names reachable one level inside it. Answers
    /// `NamedField` citations without a second pass.
    pub decl_fields: BTreeMap<String, BTreeSet<String>>,
    pub imports: Vec<ImportClaim>,
    pub cited: Vec<CitedSymbol>,
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
/// THIS IS THE ONE DISTINCTION THE CITATION WALL CANNOT DO WITHOUT, and it was found by
/// measurement rather than anticipated. Over the live corpus the wall reports refusals in
/// two utterly different populations. One is the rot §3's rule exists to catch — a citation
/// naming a module the floor cut deleted. The other is DELIBERATELY FALSE TEXT: a witness
/// that proves the resolver refuses an absent module has to AUTHOR an absent module, so
/// `test.claim.annotation_carrier` cites `extdeps.network.mac` `parse_mac_addres` on
/// purpose, one letter short, and a wall that refused it would be refusing the discriminating
/// evidence for its own mechanism. That is not a leniency carve-out; it is the difference
/// between a claim and an input, and it is decidable from the carrier's own identity.
///
/// It is NOT AN AUTHORED EXEMPTION LIST — the property is read off the module's own path,
/// the same `_test.dag` suffix `cli_run` `is_test_dag` already uses corpus-wide, widened by
/// the `test` namespace segment so a fixture module that does not carry the suffix
/// (`test.fixture.decl_facts_reflection.specimens`) lands in the same class as the witnesses
/// beside it. And the population is COUNTED, not dropped: `citations_in_fixtures` is
/// reported beside the enrolled count, so nobody reads a green as covering what it excluded.
///
/// WHAT IT COSTS, stated because it is a real hole and not a free win: a genuinely stale
/// citation authored inside a witness module is not refused. That is strictly better than
/// the mechanism it replaces — `decl_facts` did not INDEX test modules at all, so citations
/// INTO them refused spuriously and needed an authored outside-index disposition to survive;
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
/// A field whose value is NOT a string literal yields no citation, and that is the
/// fail-OPEN direction on purpose: a computed module path is not a citation this index
/// can resolve, and refusing it would refuse a construction the substrate allows. It is
/// recorded as a coverage boundary rather than as a silent skip — `citation_sites` and
/// `resolvable_citations` are reported separately so a green names both denominators.
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
/// WHY THIS IS THE DECLARATION'S WHOLE SUBTREE AND NOT ITS DIRECT CHILDREN. The first
/// version read one level down and produced FABRICATED REFUSALS over the live corpus,
/// measured rather than predicted: `extdeps.llm.anthropic` cites `cache_control` of
/// `AnthropicTextBlock`, which is a field of a VARIANT of a coproduct, so it sits two
/// levels down; `std.disposition` `Disposition` `marker` is the same shape; and a `data`
/// declaration's fields live inside its initializer expression, deeper still. A refusal
/// that fires because the reader did not descend is worse than no check at all, because
/// its remedy is to delete a correct citation.
///
/// WHAT THIS DELIBERATELY GIVES UP, stated rather than left to be discovered: the set is
/// the union over the declaration's subtree, so a citation naming a field that belongs to
/// a DIFFERENT variant of the same coproduct resolves. That is a real weakening of this
/// arm and it is confined to it — the module and declaration arms are exact. The next rung
/// is a field lookup that descends the declared TYPE rather than the declaration's text,
/// which needs the inferred tree this ingestion walk deliberately does not build.
fn declaration_field_names(
    item: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for_each_node(item, &mut |node| {
        // A record literal's field initializers, and a `type`'s declared fields, are both
        // named child nodes one step below their parent. BOTH NAME READINGS ARE TAKEN:
        // `make_field_init_node` stamps `.name` directly, while a declared field's name is
        // recovered from its ident span the way the rest of the frontend does it, and the
        // two are not interchangeable across node families.
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

/// One module's record, from that one module's parse tree. No corpus, no resolution.
pub fn record_from_module(
    module: &Rc<Node>,
    source_indices: &Rc<im::HashMap<String, Rc<NewlineIndex>>>,
    rel_path: &str,
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
        // The enclosing declaration is known HERE and nowhere deeper: this loop already
        // iterates top-level items, so carrying its name into the subtree walk costs one
        // string and closes the "two citations in one module share one row" residue.
        let in_declaration = authored_name_at(source_indices.clone(), item.clone());
        for_each_node(item, &mut |node| {
            if let Some(c) = citation_from_record_literal(node, &in_declaration)
                .or_else(|| citation_from_constructor_call(node, &in_declaration))
            {
                cited.push(c);
            }
        });
    }

    ModuleDeclarationRecord {
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
    /// THE `!is_fixture_carrier` FILTER THAT USED TO GUARD THIS IS GONE, and its absence is a
    /// property of the row grain rather than of the current roster's contents. A row now names
    /// its citing module, so "is this citation enrolled in the debt roster" is answered by the
    /// row itself; a carrier-shaped pre-filter could only ever change the answer by disagreeing
    /// with the rows, which is the paired-arm desynchronization this module already carries a
    /// receipt for. It is not safe merely because today's debt rows happen to name no fixture
    /// citer: were one enrolled there tomorrow, counting it is the CORRECT reading of this
    /// field, whose subject is the roster and not the carrier.
    pub citations_pre_existing_debt: usize,
    /// Citations authored inside a witness or fixture carrier, where deliberately false
    /// text is the evidence rather than a defect. Counted, never silently dropped.
    pub citations_in_fixtures: usize,
    /// Citations naming a namespace no swept module declares — hand-Rust and other
    /// universes. Counted rather than dropped, so a green names what it did NOT cover.
    pub citations_outside_index: usize,
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

pub fn index_records(index: &DeclarationIndex) -> Vec<&ModuleDeclarationRecord> {
    index.modules.values().collect()
}

pub fn index_population(index: &DeclarationIndex) -> DeclarationIndexPopulation {
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
        import_members_kernel_named: import_member_kernel_named_count(index),
        lens_modules: index
            .modules
            .keys()
            .filter(|m| is_top_level_lens_module(m))
            .count(),
    }
}

/// A citation may name a module by its LOGICAL path — the `v2.` prefix stripped — because
/// that is the identity `decl_facts` published and the corpus is authored against it. Both
/// spellings resolve to the one module; neither is fabricated, since the fallback only ever
/// finds a module that really declares itself `v2.x`.
fn resolve_cited_module<'a>(
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
/// The corpus cites modules that are not `.dag` at all — `v1_compiler.cli_run` and its
/// siblings name hand-Rust, and no `.dag` file declares that namespace. A citation to one
/// of those is TRUE and UNRESOLVABLE at the same time, which `std.decl_ref`
/// `CitationIndexCoverage` already models: resolution answers "does this name a real
/// declaration", coverage answers "is this declaration inside the universe the index
/// covers". Refusing them would refuse correct citations.
///
/// The decidable line is the NAMESPACE ROOT. If some swept module declares the citation's
/// first segment, the citation is inside the `.dag` namespace and its module must exist —
/// so a DELETED `.dag` module still refuses, which is exactly the rot §3's rule exists to
/// catch. If no module declares that root, the citation names another universe and is
/// counted as outside the index, never silently dropped.
///
/// THIS IS NOT A THRESHOLD OR A HEURISTIC (DESIGN §4 — a heuristic is never necessary in a
/// closed system). A namespace root is either declared by a module in the swept corpus or
/// it is not; the predicate is total and is derived from the same one index.
fn citation_is_outside_index(index: &DeclarationIndex, module_path: &str) -> bool {
    if resolve_cited_module(index, module_path).is_some() {
        return false;
    }
    let root = module_path.split('.').next().unwrap_or(module_path);
    !index.namespace_roots.contains(root)
}

/// Whether an import member is admitted ONLY by the kernel-type escape below.
///
/// THIS IS A REAL HOLE IN THE WALL AND IT IS THE ONE PLACE THIS FILE DOES NOT CLOSE.
/// `v1.03_resolve` `get_exported_names` appends `map_keys(kernel_type_set)` to every
/// module's export surface, so EVERY module in the corpus exports every kernel type name.
/// The claim `import m { Int }` is therefore admitted whatever `m` is, and the index has to
/// admit it too or it would refuse source the compiler accepts.
///
/// MEASURED, not inferred, against the installed compiler on a throwaway source root: a
/// module whose entire body is one `fn` returning `Int` was imported as
/// `import extdeps.probe_missing_anchor { Int, String, Bool }` and compiled to 6 files with
/// 0 diagnostics. The zero is readable because the nonzero was run beside it — the same
/// source root with `{ probe_absent_member_RED }` refused with a located `MissingExport` at
/// the member token. So the wall is live and this specific class walks through it.
///
/// THE LIVE SPECIMEN IS NOT SYNTHETIC. `std.types` declares no `Int`, no `String` and no
/// `Float` anywhere — it names them as KEYS of `kernel_type_set`, which is a different fact
/// — and hundreds of modules author `import std.types { Int, String }`. Every one of those
/// claims is false about `std.types` and always has been, and the reason nobody noticed is
/// precisely that the escape makes the claim unfalsifiable.
///
/// WHY IT IS COUNTED RATHER THAN REFUSED HERE. Refusing it is a change to what the SEED
/// COMPILER ACCEPTS — `get_exported_names` is the authority and editing it is
/// `NewLanguageBehavior`, which `gunbc.v1_maintenance_standing` refuses and which a refusal
/// dominates every admission of. So this change may not close it. What it may do, and what
/// DESIGN §5 requires of any arm that widens, is make the frequency OBSERVABLE: a bare
/// `continue` zeroes the deficit's frequency by construction, so the class can never rank
/// for fixing (§6 prices by displaced cost, and a masked cost displaces nothing). Counted,
/// it is a number that can be watched, prioritized, and burned down.
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
/// A target ABSENT from the index is not reported here, and the reason is a denominator one
/// rather than leniency: the sweep's roots are the authored `.dag` roots, so a target absent
/// from them is a module this index never observed, and reporting "member absent" over it
/// would assert a fact about a module whose text was never read. That is a different
/// question — module existence — and it is the compiler's `UnresolvedImport`.
///
/// The kernel-type escape is the one admission this function makes that is NOT a fact about
/// the target module; it is counted as `import_members_kernel_named` and its receipt is on
/// `import_member_is_kernel_named`.
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
/// WHY A ROSTER EXISTS AT ALL. Nothing has checked an authored citation since 2026-08-23,
/// when the operator removed the `cited-symbol` job (DESIGN records that drop and states its
/// future exposure as unbounded). This wall's first execution therefore lands on a corpus
/// that has been accumulating the exact class §3's rule names, and every row below is a REAL
/// DEFECT: a citation naming a declaration that does not exist. Measured on the live tree,
/// they are 42 sites, each named by the module and declaration that authored it.
///
/// WHY THEY ARE NOT REPAIRED HERE. The repair for a citation is a judgement about what its
/// author MEANT, and guessing it is how a stale citation becomes a confidently wrong one.
/// `extdeps.docker.container_stats` `Stats` is probably `ContainerStats`; `gunbc.ci_workflow`
/// was deleted by the floor cut and its citation may want a different module or no citation
/// at all. Those are 38 separate subjects with 38 separate owners, and bundling them into
/// the change that builds the wall would make the wall unreviewable.
///
/// WHY THIS IS A DEBT CONTRACT AND NOT A QUIETER SNAPSHOT. DESIGN §5 admits a monotone debt
/// contract on four conditions, and all four are met here rather than argued: the subject
/// universe is INDEPENDENTLY DISCOVERED (by this index, from the corpus's own text, not
/// hand-listed); it is CLOSED (every authored citation in a non-fixture module); membership
/// is checked at IDENTITY GRAIN, not by count; and the direction is one-way — see
/// `citation_debt_findings`, which REFUSES a row that no longer reproduces. That last one is
/// the teeth: repairing a citation forces deleting its row, so the roster can only shrink,
/// and a roster that has rotted stops the line exactly like a violation does.
///
/// THE FOUR PLANTED CONTROLS ARE NOT IN THIS ROSTER, AND THE PROSE THAT SAID THEY WERE WAS
/// FALSE. An earlier revision of this comment read "FOUR ROWS AT THE END ARE NOT DEBT ... they
/// are the DELETED census's own planted controls ... they leave when the lens does". Enumerating
/// all 38 rows finds no such row — no `G1_planted` target, no `synthetic` namespace, and the
/// actual last four are ordinary debt. The rows were never added, only described.
///
/// It was caught by the wall's first corpus run, which reported all four controls as ordinary
/// refusals: `synthetic.g1_planted_module_absent_control_RED`, two `v2.std.node` declarations
/// and one `NodeKind` field. Recording that the claim was FALSE rather than silently correcting
/// it is the point — a stale statement inside the carrier built to stop stale statements is the
/// specimen, and deleting it quietly would lose the finding.
///
/// They now live in `PLANTED_CONTROL_CITATIONS`, which is a different KIND of roster and not a
/// tidier corner of this one: debt shrinks to empty, controls never retire, and the two arms
/// read the same trigger in opposite directions. See that constant.

/// THE LENS IS DELETED AS OF 2026-08-26, AND THE HAND-OFF THIS COMMENT LEFT WAS HALF RIGHT.
/// It read "its deletion cascades through sixteen witnesses ... they delete with the lens,
/// not before it", and the staged form it invoked was correct — the wall and its
/// predecessor's funeral did belong in separate diffs. What it got wrong is the population.
///
/// SIXTEEN WAS THE COUNT AT #7707, when the lens landed. The file grew twice after that
/// (#8673 enrolled roster_registry, #8775 the two instance-gap carriers) and carried 27
/// `test fn` identities by the time this comment was written beside it.
///
/// AND "DEAD" WAS TRUE OF THE LENS AND FALSE OF ITS WITNESSES. Measured against the seven
/// symbols the witness file imported FROM the lens, only 6 of the 27 touch one. Six more
/// call `resolve_declaration_ref` directly — which lives in `v2.std.decl_ref_resolution`,
/// a module that SURVIVES with four other consumers — so they are the only executing
/// evidence for a live authority's five-arm refusal, and §4b(4) keeps them enrolled rather
/// than deleting them with the machinery that climbed. They moved to
/// `test.claim.long.decl_ref_resolution_witness_test`. The remaining 15 are population and
/// projection claims about the carriers that PROJECT `DeclarationRef`s, and they moved to
/// `test.claim.long.carrier_reference_integrity_witness_test`.
///
/// So the disposition is three-way and this comment admitted only two arms: the lens dies,
/// six witnesses die with it, twelve rehome onto subjects that outlive it.
/// Citations INSIDE fixture and witness carriers that do not resolve, enumerated at identity
/// grain because carrier identity is not a licence.
///
/// WHY THIS EXISTS, AND WHY THE THING IT REPLACED WAS WRONG (review 55939). Both citation arms
/// used to skip every citation in a module `module_is_fixture_carrier` answered true for. The
/// justification was real — a witness proving the resolver refuses an absent symbol has to
/// author an absent symbol, so its false citation is its evidence rather than its defect — but
/// the EXEMPTION WAS KEYED ON THE MODULE while the justification is a property of the
/// CITATION. Carrier identity establishes that SOME citations there are deliberately false,
/// never that ALL are.
///
/// THE ROWS ARE DERIVED FROM THE INDEX, NOT FROM DIAGNOSTIC TEXT, and the first attempt got
/// that wrong in a way worth recording: extracting the identities by parsing rendered
/// messages silently dropped the FIELD, because a `CitedDeclarationAbsent` message never
/// prints one. Rows for citations carrying a `NamedField` whose DECLARATION is absent then
/// matched nothing, and the corpus run reported the same citation as BOTH refusing and its
/// row as spent — the paired-inverse-arm desynchronization this module already documents,
/// firing on its author. Five further identities are deliberately NOT here: they refuse but
/// are already enrolled in `PRE_EXISTING_CITATION_DEBT` or `PLANTED_CONTROL_CITATIONS`, and a
/// second row for one citation would be duplicate authority with a double stale-arm report.
///
/// MEASURED, WHICH IS WHAT SETTLED IT RATHER THAN THE ARGUMENT: of 161 citations authored
/// inside fixture carriers, 128 RESOLVE. They are ordinary citations of real authorities that
/// happen to live in a test module, and the module-grained skip was shielding all 128 in order
/// to protect at most 33. Counting the excluded population as `in_fixtures` did not restore
/// integrity; a counted hole is still a hole.
///
/// AND THE HOLE WAS OCCUPIED, not merely reachable, which is the difference between a
/// disclosed boundary and a defect. Two of the rows below are ordinary staleness that had
/// nothing to do with fixture intent:
///   - `dag.test.claim.witness_purpose_taxonomy_witness` is a `dag.`-prefixed module path no
///     module declares; the real module is `test.claim.witness_purpose_taxonomy_witness`. A
///     plain typo, invisible for as long as the skip stood.
///   - `std.disposition` `Disposition` `marker` names a real authority and a real declaration
///     with an absent field, cited twice from `v2.lens/disposition_redundancy_test.dag`.
///
/// WHAT THIS ROSTER IS AND IS NOT. It is an identity-grain exemption, not a debt contract, and
/// the difference is condition 3 of §5's four: a debt roster's terminal state is EMPTY, and
/// this one's is not — a planted control such as `NoSuchDecl_G1_RED` is permanent by design
/// and will never be repaired. What it shares with the debt contract is the property that
/// makes either safe: it is MONOTONE and it REFUSES WHEN SPENT. A row whose citation stops
/// refusing is reported by the same inverse arm that polices `PRE_EXISTING_CITATION_DEBT`, so
/// the roster cannot rot into a list of things that used to be true, and a citation cannot be
/// silently un-checked by deleting its row — deleting a row while the citation still refuses
/// turns it into an ordinary finding that stops the line.
///
/// THE ROWS ARE NOT CLASSIFIED into deliberate-control versus genuine-staleness, and that is
/// stated rather than hidden: the two specimens above are named because they are decidable by
/// inspection, and the remaining rows are not sorted, because deciding what a witness's author
/// MEANT is the judgement §5 warns turns a stale citation into a confidently wrong one. The
/// next rung is a witness declaring its planted controls as typed rows, at which point the
/// deliberate half becomes derivable and only the genuine half survives here as debt.
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
        "extdeps.network.mac",
        "parse_mac_address",
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
        "bound_condition_pends_until_its_declaration_appears",
        "extdeps.network.mac",
        "parse_mac_address",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "frontier_expiry_fired_row_still_present_reds",
        "extdeps.network.mac",
        "parse_mac_address",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "frontier_expiry_fired_trigger_absent_from_rows_is_clean",
        "extdeps.network.mac",
        "already_deleted_frontier_unit",
        "",
    ),
    (
        "test.claim.annotation_carrier",
        "unbound_condition_cannot_be_forced_to_fire_by_any_present_decls",
        "extdeps.network.mac",
        "parse_mac_address",
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
        "extdeps.llm.anthropic",
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
        "extdeps.dhcp.v4",
        "mac_address_anemic_brand_frontier_rows",
        "extdeps.network.mac",
        "parse_mac_address",
        "",
    ),
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
    (
        "extdeps.llm.anthropic",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic",
        "AnthropicImageBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic",
        "AnthropicTextBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic",
        "AnthropicTextBlock",
        "citations",
    ),
    (
        "extdeps.llm.anthropic",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic",
        "AnthropicToolReferenceBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic",
        "structural_coverage_gap_anthropic_tool_result_nested_block_wire_payloads",
        "extdeps.llm.anthropic",
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
        "gunbc.ci_heal_credential",
        "ci_heal_job_ref",
        "gunbc.ci_workflow",
        "ci_heal_generated_artifacts_job",
        "",
    ),
    (
        "gunbc.ci_heal_credential",
        "ci_heal_workflow_ref",
        "gunbc.ci_workflow",
        "ci_workflow",
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
/// The roster is a parameter for the reason both callers now state: an enrolled-debt roster
/// is a fact about ONE corpus, and a predicate that reads it from module scope makes its own
/// behaviour unauthorable by any fixture.
fn citation_in_roster(
    citing_module: &str,
    cited: &CitedSymbol,
    roster: &[(&str, &str, &str, &str, &str)],
) -> bool {
    roster
        .iter()
        .any(|row| *row == citation_site(citing_module, cited))
}

/// A citation's SITE identity — who cites, and what is cited. Every roster row is one of
/// these, and the first field is the whole content of this change: a row exempts the site
/// that authored it, never the target it names.
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
/// Without this the roster is the "quieter snapshot" DESIGN §5 rejects: rows would survive
/// their subjects, the population would drift from the corpus, and a reader would take the
/// list for a measurement when it had become a memory. With it the roster is monotone by
/// construction: the only way to remove a violation is to repair it AND delete its row, and
/// the only way to keep a row is for the violation to still be there.
pub fn citation_debt_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    citation_debt_findings_against(index, PRE_EXISTING_CITATION_DEBT)
}

/// THE DELETED CENSUS'S PLANTED CONTROLS — DELIBERATELY FALSE CITATIONS, NOT DEBT.
///
/// `v2.lens.cited_symbol_resolution` authors citations that MUST NOT RESOLVE: they are the
/// discriminating evidence that a resolver refuses, one per refusal arm, and
/// `cited_symbol_planted_control_home_note` records the operator ruling (2026-08-25) that
/// rehomed them there as machine evidence rather than documents. A wall that refused them
/// would be refusing the evidence for its own mechanism.
///
/// THEY ARE NOT ENROLLED AS DEBT, AND THE DISTINCTION IS THE WHOLE POINT. `PRE_EXISTING_CITATION_DEBT`
/// is a monotone contract that may only SHRINK: a row leaves when someone repairs its citation.
/// These never retire — DESIGN §4b(4) is explicit that an expecting-red probe which greens
/// flips to a permanent regression control rather than being deleted. Putting them in the debt
/// roster would have made "the roster only shrinks" false of four of its rows, and a contract
/// with silent permanent members is not a contract.
///
/// SO THE STALENESS ARM IS INVERTED HERE, and that inversion is the carrier's whole content: a
/// debt row refuses when its citation STOPS refusing; a control row refuses when its citation
/// stops refusing TOO, but for the opposite reason — a control that resolves is no longer
/// discriminating, and the mechanism it exists to prove has quietly lost its evidence. Same
/// trigger, opposite meaning, so they are two carriers rather than one with a flag.
///
/// THE NAMED TERMINUS HAS FIRED, 2026-08-26, AND THIS PARAGRAPH RECORDS IT RATHER THAN STILL
/// PREDICTING IT. What stood here said the four identities were a deliberate, transient second
/// representation of controls `v2.lens.cited_symbol_resolution` also held, and named that lens's
/// deletion as the terminus. This change IS that deletion, so the duplication is over: the lens
/// is gone and these rows are the sole authority, exactly as the clause below anticipated.
///
/// THE DEADNESS MEASUREMENT IS KEPT, because it is what licensed the cut and it held up. Of the
/// 27 symbols unique to that lens, the only references outside it were one prose row in
/// `gunbc.roster_registry`, two prose mentions in fast witnesses (a `String` note and a `//`
/// comment), and nine real uses in its own `long/`-homed witness — declined before the fold, so
/// never executing. NO EXECUTING witness called any function that lens declared. It was dead, not
/// competing, and §3's attractor argument did not bite.
///
/// WHAT THE CUT OWED THIS ROSTER, AND PAID: the four identities are the SURVIVING authority. The
/// lens's deletion removed the dead copy and none of the evidence — DESIGN §4b(4) keeps a
/// discriminating control enrolled when its machinery goes, and treating those controls as part
/// of the funeral would have erased the four probes that prove this wall's refusal arms are real.
/// The seven debt rows below were RE-POINTED rather than deleted in the same change, for the same
/// reason: their citations are deliberately false and moved with their witnesses into
/// `test.claim.long.decl_ref_resolution_witness_test` and
/// `test.claim.long.carrier_reference_integrity_witness_test`. A deleted row would have been a
/// silently dropped obligation; a stale one would have refused, which is how the contract is
/// supposed to catch exactly this.
///
/// FOUND BY MEASUREMENT, AND THE PROSE THAT SHOULD HAVE SAID SO WAS FALSE. The roster's own
/// doc comment claimed "FOUR ROWS AT THE END ARE NOT DEBT ... the deleted census's own planted
/// controls". Enumerating all 38 rows finds no such row: the rows were never added, only
/// described. The first corpus run reported all four controls as ordinary refusals, which is
/// how the claim was caught. A false statement inside the carrier built to stop false
/// statements is the specimen this whole change exists to make impossible, and it is recorded
/// here rather than quietly corrected.
/// EMPTY AS OF 2026-08-26, AND EMPTY IS NOT DEAD. All four rows named citations authored
/// inside `v2.lens.cited_symbol_resolution`, and the comment above them said in terms that
/// they "delete with the lens, not before it". This change is that deletion, so emptying the
/// roster is the scheduled event rather than a judgement call: with the lens gone the
/// citations are gone, and every row would report `PlantedControlNoLongerRefuses` — the
/// inverse arm working, not a regression.
///
/// THE ROSTER STAYS AND THE ARM STAYS. DESIGN's reachability-read-as-occupancy row asks three
/// questions and only the first two decide whether a guard should exist: the mechanism can
/// still produce this state (any future control row), and it can still classify an element of
/// this operation's denominator (every authored citation). Current occupancy is zero. Yes /
/// yes / zero is a healthy guard being quiet, and deleting the arm because nothing lands in
/// it today would remove a live wall while looking principled.
///
/// THE ARM'S OWN EVIDENCE DOES NOT LIVE IN THIS ROSTER, which is what makes emptying it
/// cheap rather than a loss. `planted_control_findings_against` takes the roster as a
/// parameter, and `a_planted_control_that_still_refuses_is_healthy` /
/// `a_planted_control_that_resolves_has_lost_its_power_and_refuses` drive both directions
/// from controlled fixtures that author their own rows. So the RED that proves this arm
/// works stays enrolled and executing with an empty constant (§4b(4): a climb dissolves the
/// production machinery, never the evidence).
///
/// WHAT REPLACED THE REFUSAL COVERAGE the four rows carried. Each named one refusal arm of
/// the cited-symbol wall. Three of those arms already had controlled fixtures in
/// `tests/declaration_index_integrity.rs` (`import_member_absent_is_refused_and_located`,
/// `stale_citation_is_refused`,
/// `citation_to_a_deleted_module_is_refused_and_a_foreign_namespace_is_not`). The FOURTH,
/// `CitedFieldAbsent`, had none — measured, the string did not occur in that file — so
/// `citation_to_an_absent_field_is_refused_and_a_present_field_is_not` was authored in the
/// same change that empties this roster. A controlled fixture that authors both input and
/// expected population is the stronger oracle anyway (§5); a planted row over the live corpus
/// only ever asserted that one hand-authored citation still refuses.
const PLANTED_CONTROL_CITATIONS: &[(&str, &str, &str, &str, &str)] = &[];

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
/// The roster is a parameter rather than a constant read from inside, and that is what makes
/// this arm's red authorable at the fixture boundary at all. A fixture tree contains a
/// handful of `probe.*` modules; joined against the 42-row production roster, every row is
/// trivially absent and the arm reports 38 stale rows that say nothing about the fixture.
/// Passing the roster lets a fixture author a ONE-ROW roster and plant both directions of the
/// contract — a row whose citation still refuses (live, no finding) and a row whose citation
/// does not (spent, one finding) — which is the discriminating pair, and it is unauthorable
/// while the roster is baked in.
pub fn citation_debt_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    citation_debt_findings_named(index, roster, "PRE_EXISTING_CITATION_DEBT")
}

/// The same arm, told which roster carried the row, so the diagnostic names the list the
/// reader must edit. A spent-row message that names the wrong roster sends the reader to a
/// file that does not contain the row.
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
/// A citation resolves against the DECLARED surface of the module it names, never against
/// that module's re-exports: a citation names where a fact LIVES, and a module that merely
/// imports a name is not its authority (§3).
///
/// ONE RESOLUTION, TWO CONSUMERS. The wall and the debt contract both read
/// `citation_resolution_refusal` — a second predicate that agreed today is a fork that
/// disagrees later, which is the objection `v2.lens.cited_symbol_resolution` already
/// recorded against writing a second resolver beside the first.
pub fn cited_symbol_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    cited_symbol_findings_against(index, &[])
}

/// The citation wall, with the ENROLLED-DEBT roster passed in.
///
/// SAME CLASS AS THE DEBT JOIN BELOW, FOUND BY ASKING THE QUESTION ONE LEVEL UP. That arm was
/// defective because its roster was a constant read from inside the function; this one
/// suppressed citations against the same constant, silently, and no fixture could vary it
/// either. The suppression is the more dangerous of the two, because a spurious refusal is
/// loud while a spurious SUPPRESSION is a citation the wall quietly declines to judge — so
/// the arm could have stopped enrolling a whole class and every fixture would still pass.
///
/// The roster is therefore a parameter here too, and the default is EMPTY rather than the
/// production constant. That is the denominator argument again: an arbitrary swept tree has
/// no pre-existing debt, so `index_findings` judges every citation it finds, and only
/// `corpus_findings` — whose subject IS this repository — passes the roster that suppresses
/// this repository's enrolled rows. One roster, reachable from one place.
///
/// THE RULE THE EMPTY DEFAULT ENCODES, stated because it is doing more work than
/// "parameterize it" suggests and a later reader will otherwise flip it back for convenience:
/// **a policy roster passed as a parameter defaults to the IDENTITY ELEMENT OF THE JUDGMENT,
/// never to the production value.** Empty means judge everything, which is the strictest
/// answer, so a caller who forgets to pass a roster gets MORE refusals. Default it to the
/// production roster instead and the forgetful caller gets silent suppression — precisely the
/// defect this parameter exists to close, reintroduced through the default. That is what makes
/// this a construction move rather than a tidier signature: the fail-closed direction is a
/// property of the type's default, not of anyone's discipline.
///
/// The seam this creates — that `corpus_findings` really does pass the production roster — is
/// itself a fact needing evidence, and it has some:
/// `corpus_findings_is_wired_to_the_production_suppression_roster` declares a module the
/// roster names and requires the same tree to be refused unenrolled and suppressed enrolled.
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
/// THE DEBT ARM IS DELIBERATELY NOT HERE, and it is a denominator distinction rather than a
/// tidying one. The other four arms ask questions ABOUT THE SWEPT MODULES: does this import
/// member exist, does this citation resolve, does this lens carry its authorship fact. Every
/// one is answered by the modules in front of it, so the answer is meaningful over any tree.
/// `PRE_EXISTING_CITATION_DEBT` is a fact about ONE SPECIFIC CORPUS — the repository's own —
/// and joining it against some other tree does not produce a weaker answer, it produces an
/// answer to a question nobody asked: a fixture tree of `probe.*` modules makes all 42 rows
/// trivially absent, so the arm reports 38 spent rows that say nothing about the fixture and
/// drown every real finding beside them.
///
/// That is the failure `review 55817` found, and it was a real one: with the debt arm folded
/// in here, every fixture in `tests/declaration_index_integrity.rs` received 42 findings it
/// did not plant, so the planted-red and positive-control assertions could not pass and the
/// §4b fixture-boundary evidence this change rests on did not execute. The repair is not to
/// gate the arm on a corpus-shape signal — that would be a smuggled heuristic (§4: a
/// heuristic is never necessary in a closed system) — but to put the arm where its
/// denominator is, in `corpus_findings` below, and to give it a roster parameter so its own
/// red stays authorable.
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
/// This function carries four arms that come in two inverse pairs, and the pairing is
/// load-bearing rather than incidental. `cited_symbol_findings_against` SUPPRESSES what a roster
/// enrolls; `citation_debt_findings` refuses a roster row nothing enrolls. `planted_control_findings`
/// reads the same trigger as the latter in the opposite direction. Each arm is a partial answer
/// and each is individually honest.
///
/// SPLITTING A PAIR ACROSS SEPARATE CHECKS DOES NOT WEAKEN IT, IT DESTROYS IT — different jobs,
/// different cadences, or different roster arguments all have the same effect, and the reasons
/// for doing it are usually good ones about job granularity. The receipt: one roster row carrying
/// an empty field where its citation carries `NamedField { "price" }` desynchronized the two
/// arms, so the suppression arm reported the citation as UNENROLLED DEBT while the staleness arm
/// reported its row as SPENT — contradictory answers about one citation in one run. Both arms
/// were locally correct. Both were wrong. NEITHER COULD DETECT IT ALONE, because each is right
/// about its own half; the only observable is the two answers being present together and
/// disagreeing.
///
/// So the arms share this one entry point and this one subject set, and the desynchronization is
/// asserted rather than left to a reader noticing two lines in a report:
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
/// The offset is turned into `line:col` by reading the offending file only — a finding is
/// rare, and holding the whole corpus's text resident so a green run could render nothing
/// would be paying the corpus for a fact about one module, which is the cost shape this
/// module exists to remove. A finding with no offset renders as the path alone rather
/// than as a fabricated `1:1`.
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
/// directions and must read the SAME subject set — the desynchronization receipt on
/// `corpus_findings` is what happens when two locally-correct arms disagree about one
/// citation, and two separately-built sets are the easiest way to reintroduce it.
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

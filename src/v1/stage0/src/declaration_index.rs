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
fn citation_from_record_literal(node: &Rc<Node>) -> Option<CitedSymbol> {
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
fn citation_from_constructor_call(node: &Rc<Node>) -> Option<CitedSymbol> {
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
        for_each_node(item, &mut |node| {
            if let Some(c) =
                citation_from_record_literal(node).or_else(|| citation_from_constructor_call(node))
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
            .filter(|r| !r.is_fixture_carrier)
            .flat_map(|r| r.cited.iter())
            .filter(|c| citation_in_roster(c, PRE_EXISTING_CITATION_DEBT))
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

/// THE PRE-EXISTING CITATION DEBT, ENUMERATED AT IDENTITY GRAIN.
///
/// WHY A ROSTER EXISTS AT ALL. Nothing has checked an authored citation since 2026-08-23,
/// when the operator removed the `cited-symbol` job (DESIGN records that drop and states its
/// future exposure as unbounded). This wall's first execution therefore lands on a corpus
/// that has been accumulating the exact class §3's rule names, and every row below is a REAL
/// DEFECT: a citation naming a declaration that does not exist. Measured on the live tree,
/// they are 38 distinct targets across 46 sites.
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

/// WHY THE LENS IS NOT DELETED IN THIS CHANGE, stated plainly rather than left as an
/// omission. `v2.lens.cited_symbol_resolution` is invoked by nothing: the operator removed
/// its CI job on 2026-08-23 and this change removes `--required-cited-symbol`, its last
/// caller. So it is dead code, not a competing authority — §3's attractor argument is about
/// a live X answering the same question, and after this change nothing asks it anything.
/// Its deletion cascades through sixteen witnesses, the lens registry, the deferral freeze
/// and the doc-graph population, which is a second cut with its own review surface, and
/// bundling it here would put the wall and its predecessor's funeral in one unreviewable
/// diff. That is the staged form §3's replacement doctrine admits, and these four rows are
/// its declared residue: they delete with the lens, not before it.
const PRE_EXISTING_CITATION_DEBT: &[(&str, &str, &str)] = &[
    ("extdeps.cloud.gcp.secret_manager", "AccessVersion", ""),
    ("extdeps.cloud.gcp.secret_manager", "AddVersion", ""),
    ("extdeps.docker.container_inspect", "Inspect", ""),
    (
        "extdeps.docker.container_stats",
        "ContainerStats",
        "cpu_percent",
    ),
    ("extdeps.docker.container_stats", "Stats", ""),
    (
        "extdeps.git.publication_transport",
        "PublicationTransport",
        "",
    ),
    (
        "extdeps.llm.anthropic",
        "AnthropicImageBlock",
        "cache_control",
    ),
    (
        "extdeps.llm.anthropic",
        "AnthropicTextBlock",
        "cache_control",
    ),
    ("extdeps.llm.anthropic", "AnthropicTextBlock", "citations"),
    (
        "extdeps.llm.anthropic",
        "AnthropicToolReferenceBlock",
        "cache_control",
    ),
    ("extdeps.llm.anthropic", "CacheControl", "ttl"),
    ("extdeps.mediawiki", "extdeps_external_authority_anchor", ""),
    ("extdeps.network.ipv6", "parse_ipv6_address", ""),
    ("extdeps.network.mac", "parse_mac_address", ""),
    ("extdeps.tcgplayer.store", "UpdateSkuPrice", "price"),
    (
        "gunbc.ci_floor_measurement",
        "gunbc_ci_legacy_host_modeled_residents",
        "",
    ),
    (
        "gunbc.ci_floor_measurement",
        "gunbc_ci_managed_host_quiescent_meminfo_read",
        "",
    ),
    (
        "gunbc.ci_runner_placement",
        "ci_runner_placement_authority",
        "",
    ),
    ("gunbc.ci_workflow", "ci_heal_generated_artifacts_job", ""),
    ("gunbc.ci_workflow", "ci_workflow", ""),
    ("gunbc.fleet_host_budget", "fleet_host_budget_authority", ""),
    (
        "gunbc.host_budget_source",
        "host_budget_source_emitted_into_stage0",
        "",
    ),
    ("gunbc.host_effect", "host_effect_apply", ""),
    ("gunbc.roadmap_authority", "roadmap_document", ""),
    ("gunbc.runner_lifecycle", "EnsureRunnerJitWrapper", ""),
    (
        "gunbc.tailscale_acl_phase2_credential",
        "tailscale_acl_upsert_wet",
        "",
    ),
    (
        "product.build_selection",
        "build_constructible_ceiling_axis",
        "",
    ),
    ("product.build_selection", "build_incremental_cash_axis", ""),
    ("product.build_selection", "build_memory_bandwidth_axis", ""),
    ("product.build_selection", "build_memory_capacity_axis", ""),
    ("product.build_selection", "build_wall_power_axis", ""),
    ("product.fabric.supply", "Offer", ""),
    ("std.bytes", "builtin_function_registry", ""),
    ("tools.rust_stage0_gates", "per_unit_test_selector", ""),
    ("v1.compiler.05_emit", "emit_literal", ""),
    ("v1.compiler.infer_emit_info", "type_summary_reaches_fn", ""),
    (
        "v2.compiler.self_host.emitter_producer_provenance",
        "v2_self_hosted_promotions",
        "",
    ),
    (
        "v2.test.workflow.claim_witness_corpus_ci_runner",
        "ClaimWitnessCorpusClaimRunRow",
        "",
    ),
];

/// Whether a citation names one of the enumerated pre-existing targets.
/// Whether a citation is enrolled in `roster`, at identity grain.
///
/// The roster is a parameter for the reason both callers now state: an enrolled-debt roster
/// is a fact about ONE corpus, and a predicate that reads it from module scope makes its own
/// behaviour unauthorable by any fixture.
fn citation_in_roster(cited: &CitedSymbol, roster: &[(&str, &str, &str)]) -> bool {
    let field = cited.field.as_deref().unwrap_or("");
    roster.iter().any(|(module, decl, f)| {
        *module == cited.module_path && *decl == cited.decl_name && *f == field
    })
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
/// THIS IS A SECOND REPRESENTATION OF FOUR IDENTITIES THE DEAD LENS ALSO CARRIES, AND THE
/// DUPLICATION IS DELIBERATE, TRANSIENT, AND HAS A NAMED TERMINUS. `v2.lens.cited_symbol_resolution`
/// holds the same four controls as `cited_symbol_planted_controls`. Measured rather than assumed,
/// because "it is dead" is exactly the claim shape this branch already got wrong once: of the 27
/// symbols unique to that lens, the only references outside it are one prose row in
/// `gunbc.roster_registry`, two prose mentions in fast witnesses (a `String` note and a `//`
/// comment), and nine real uses in `test.claim.long.cited_symbol_resolution_witness_test` — which
/// is a `long/` home and is declined before the fold, so it never executes. NO EXECUTING witness
/// calls any function that lens declares. So it is dead, not competing, and §3's attractor
/// argument does not bite.
///
/// WHAT THIS ROSTER OWES THE CUT THAT DELETES IT: these four identities are the SURVIVING
/// authority. The lens's deletion is a removal of the dead copy, never of the evidence — DESIGN
/// §4b(4) keeps a discriminating control enrolled when its machinery goes, and deleting the lens
/// while treating its controls as part of the funeral would erase the four probes that prove this
/// wall's refusal arms are real. They live here now; that is the point of moving them.
///
/// FOUND BY MEASUREMENT, AND THE PROSE THAT SHOULD HAVE SAID SO WAS FALSE. The roster's own
/// doc comment claimed "FOUR ROWS AT THE END ARE NOT DEBT ... the deleted census's own planted
/// controls". Enumerating all 38 rows finds no such row: the rows were never added, only
/// described. The first corpus run reported all four controls as ordinary refusals, which is
/// how the claim was caught. A false statement inside the carrier built to stop false
/// statements is the specimen this whole change exists to make impossible, and it is recorded
/// here rather than quietly corrected.
const PLANTED_CONTROL_CITATIONS: &[(&str, &str, &str)] = &[
    ("synthetic.g1_planted_module_absent_control_RED", "any", ""),
    (
        "v2.std.node",
        "G1_planted_declaration_absent_control_RED",
        "",
    ),
    ("v2.std.node", "G1_planted_ambiguous_control_RED", ""),
    (
        "v2.std.node",
        "NodeKind",
        "G1_planted_field_absent_control_RED",
    ),
];

/// A control that has STOPPED refusing has lost its discriminating power, and that is a red in
/// its own right — the inverse of a spent debt row, and the reason these are a separate roster.
pub fn planted_control_findings(index: &DeclarationIndex) -> Vec<DeclarationIntegrityFinding> {
    planted_control_findings_against(index, PLANTED_CONTROL_CITATIONS)
}

pub fn planted_control_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    let mut still_refusing: BTreeSet<(String, String, String)> = BTreeSet::new();
    for record in index.modules.values() {
        for cited in &record.cited {
            if citation_resolution_refusal(index, record, cited).is_some() {
                still_refusing.insert((
                    cited.module_path.clone(),
                    cited.decl_name.clone(),
                    cited.field.clone().unwrap_or_default(),
                ));
            }
        }
    }
    roster
        .iter()
        .filter(|(module, decl, field)| {
            !still_refusing.contains(&(module.to_string(), decl.to_string(), field.to_string()))
        })
        .map(|(module, decl, field)| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::PlantedControlNoLongerRefuses,
            rel_path: "src/v1/stage0/src/declaration_index.rs".to_string(),
            offset: None,
            message: format!(
                "PLANTED_CONTROL_CITATIONS lists `{module}` `{decl}`{} as a control that must \
                 NOT resolve, and it no longer refuses — the control has lost its \
                 discriminating power and the mechanism it proves is now unevidenced",
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
/// handful of `probe.*` modules; joined against the 38-row production roster, every row is
/// trivially absent and the arm reports 38 stale rows that say nothing about the fixture.
/// Passing the roster lets a fixture author a ONE-ROW roster and plant both directions of the
/// contract — a row whose citation still refuses (live, no finding) and a row whose citation
/// does not (spent, one finding) — which is the discriminating pair, and it is unauthorable
/// while the roster is baked in.
pub fn citation_debt_findings_against(
    index: &DeclarationIndex,
    roster: &[(&str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    let mut live: BTreeSet<(String, String, String)> = BTreeSet::new();
    for record in index.modules.values() {
        if record.is_fixture_carrier {
            continue;
        }
        for cited in &record.cited {
            if citation_resolution_refusal(index, record, cited).is_some() {
                live.insert((
                    cited.module_path.clone(),
                    cited.decl_name.clone(),
                    cited.field.clone().unwrap_or_default(),
                ));
            }
        }
    }
    roster
        .iter()
        .filter(|(module, decl, field)| {
            !live.contains(&(module.to_string(), decl.to_string(), field.to_string()))
        })
        .map(|(module, decl, field)| DeclarationIntegrityFinding {
            kind: DeclarationIntegrityKind::CitationDebtRowStale,
            rel_path: "src/v1/stage0/src/declaration_index.rs".to_string(),
            offset: None,
            message: format!(
                "PRE_EXISTING_CITATION_DEBT still lists `{module}` `{decl}`{} — that citation \
                 no longer refuses, so the row is spent and must be deleted; the roster only \
                 shrinks",
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
    roster: &[(&str, &str, &str)],
) -> Vec<DeclarationIntegrityFinding> {
    let mut out = Vec::new();
    for record in index.modules.values() {
        // A witness's deliberately false citation is its evidence, not its defect.
        if record.is_fixture_carrier {
            continue;
        }
        for cited in &record.cited {
            if citation_in_roster(cited, roster) {
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
/// answer to a question nobody asked: a fixture tree of `probe.*` modules makes all 38 rows
/// trivially absent, so the arm reports 38 spent rows that say nothing about the fixture and
/// drown every real finding beside them.
///
/// That is the failure `review 55817` found, and it was a real one: with the debt arm folded
/// in here, every fixture in `tests/declaration_index_integrity.rs` received 38 findings it
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
        &[PRE_EXISTING_CITATION_DEBT, PLANTED_CONTROL_CITATIONS].concat()[..],
    ));
    out.extend(lens_authorship_findings(index));
    out.extend(citation_debt_findings(index));
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

use im::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    build_newline_index, diagnostic_to_message, diagnostic_to_span, node_name_span,
    CompilerDiagnostic, SourceSpan,
};

/// One parse-derived module⇄path row for manifest emission (host binding authority).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedModuleBinding {
    pub module_path: String,
    pub ident_span: Rc<SourceSpan>,
}

/// A non-refusing outcome of the module-index walk.
///
/// `ModuleBindingUnclassified` is deliberately NOT called "valid moduleless". Its
/// inhabitants include admitted PARSE FAILURES — a file whose leading `module `
/// header the legacy scan did not recognize is placed here whether it is a genuine
/// fragment or unparseable source, and the two are indistinguishable (see
/// `module_declaration_line_present`). Naming it "valid" would assert a semantic
/// classification the index cannot observe, which is the same conflation this
/// change exists to expose. The name reports what was observed: no binding was
/// produced, and why is unclassified.
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleBindingOutcome {
    Bound(ParsedModuleBinding),
    ModuleBindingUnclassified,
}

/// A parse that FAILED: typed, located, and carrying the parser's own diagnostic
/// rather than a rendered string (§5 — the line stops with a located refusal).
#[derive(Debug, Clone)]
pub struct ModuleBindingRefusal {
    /// Repository-relative where derivable, so two same-basename files are
    /// distinguishable and the location stays host-independent.
    pub source_path: String,
    pub diagnostic: Rc<CompilerDiagnostic>,
}

impl ModuleBindingRefusal {
    pub fn span(&self) -> Rc<SourceSpan> {
        diagnostic_to_span(self.diagnostic.clone())
    }

    /// Rendering is for display only; the typed diagnostic above is the authority.
    pub fn rendered(&self) -> String {
        let span = self.span();
        format!(
            "{}:{}-{}: {}",
            self.source_path,
            span.start,
            span.end,
            diagnostic_to_message(self.diagnostic.clone())
        )
    }
}

/// Returns true when the first non-blank, non-comment line starts with `module `.
///
/// RETAINED, AND THE REASON MATTERS. The intent was to delete this and decide the
/// moduleless case from the parse result. That is NOT POSSIBLE TODAY, established
/// by execution rather than argued: the bootstrap parser is module-mandatory, so a
/// valid moduleless fragment does not parse-with-no-module — it FAILS, with
/// `expected keyword 'module'`. `Ok(None)` therefore never came from "parsed, no
/// module"; that state is unreachable. Removing this scan refused every moduleless
/// fragment, including `tests/fixtures/fact_cardinality_split_brace.dag`, which sits
/// under `regen_source_roots`.
///
/// Classifying on the typed diagnostic instead does not separate them either: a
/// fragment yields `expected keyword 'module', found keyword 'type'` at offset 0,
/// and a file whose header is preceded by an unsupported block comment yields
/// `expected keyword 'module', found Slash` at offset 0 — the same class at the
/// same position.
///
/// So the conflation below is REAL and stays: a parse failure with no textual
/// header is still read as a moduleless fragment, silently. NEXT-RUNG TRIGGER: a
/// parser affordance that can parse a fragment, or a distinct diagnostic for
/// "no header at all" versus "header did not parse". Both are v1 parser changes and
/// are refused under `gunbc.v1_maintenance_standing` (NewLanguageBehavior /
/// SeedFeatureCompletion), so this does not climb inside the seed.
fn module_declaration_line_present(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module ") {
            return true;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    false
}

/// Repo-relative when the path sits under the workspace root, else the path as
/// given. Keys the tokenizer and newline index so spans name this exact file.
fn source_key(path: &Path) -> String {
    let ws = crate::cli_run::workspace_root();
    path.strip_prefix(&ws)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Parse a `.dag` file's module declaration through the v1 bootstrap parser.
///
/// `Err(_)`              — parse failed AND a textual `module ` header is present:
///                         typed, located refusal carrying the parser's own
///                         diagnostic. This arm previously PANICKED with a
///                         rendered string, discarding the span.
/// `Ok(ModuleBindingUnclassified)`
///                       — the leading-header scan recognized no module
///                         declaration and no binding was produced. Inhabitants
///                         include genuine fragments AND parse failures; the index
///                         cannot tell them apart. Unchanged by this PR.
/// `Ok(Bound(_))`        — parsed, declared a module.
///
/// SCOPE: this is the panic arm only. The silent arm is NOT closed here, and the
/// reason is a parser limitation recorded above, not an oversight.
pub fn parse_module_binding(
    path: &Path,
    content: &str,
) -> Result<ModuleBindingOutcome, ModuleBindingRefusal> {
    let key = source_key(path);
    let tokens = tokenize(content.to_string(), key.clone());
    let source_index = build_newline_index(key.clone(), content.to_string());
    let mut indices = HashMap::new();
    indices.insert(key.clone(), source_index);
    let source_indices = Rc::new(indices);
    let result = parse(tokens, source_indices);
    if let Some(err) = result.error.as_ref() {
        if module_declaration_line_present(content) {
            return Err(ModuleBindingRefusal {
                source_path: key,
                diagnostic: err.diagnostic.clone(),
            });
        }
        // UNCHANGED, and still a conflation: see `module_declaration_line_present`.
        // What this change fixes is the OTHER arm — a header-bearing file that fails
        // to parse now produces a typed located refusal instead of an untyped panic.
        return Ok(ModuleBindingOutcome::ModuleBindingUnclassified);
    }
    let Some(module) = result.module.as_ref() else {
        return Ok(ModuleBindingOutcome::ModuleBindingUnclassified);
    };
    // NOTE: an empty parsed module name is also unclassified, preserving prior
    // behavior exactly. It is a third candidate refusal, deliberately unchanged —
    // this change is bounded to the recognized-header parse-failure arm.
    if module.name.is_empty() {
        return Ok(ModuleBindingOutcome::ModuleBindingUnclassified);
    }
    Ok(ModuleBindingOutcome::Bound(ParsedModuleBinding {
        module_path: module.name.clone(),
        ident_span: node_name_span(module.clone()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The four cases the partition must keep separate. Cases 1 and 2 are the
    // discriminating pair: BOTH are parse failures, they differ only in whether a
    // textual `module ` header is present, and before this change case 1 panicked
    // while case 2 silently returned absence. They must now be the same class.

    // Case 1 — malformed WITH a module header: located refusal, no panic.
    #[test]
    fn malformed_with_header_refuses_located() {
        let path = Path::new("fixture.dag");
        // A coproduct written with the value keyword: the real specimen shape.
        let src = "module probe.bad\n\ndata BadScope\n  = | OnlyVariant\n";
        let refusal = parse_module_binding(path, src).expect_err("must refuse");
        assert_eq!(refusal.source_path, "fixture.dag");
        assert!(
            refusal.span().end >= refusal.span().start,
            "refusal must carry a located span"
        );
    }

    // Case 2 — malformed WITHOUT a recognized header: lands unclassified.
    //
    // This pins the RESIDUE rather than the fix. It is the silent arm, and it is
    // deliberately still open: the bootstrap parser cannot distinguish this from a
    // genuine fragment (both yield `expected keyword 'module'` at offset 0). If a
    // parser affordance ever separates them, this assertion FLIPS to expect an
    // Err, and it should — that is the signal the next rung became reachable.
    #[test]
    fn malformed_without_header_lands_unclassified_not_refused() {
        let path = Path::new("fixture.dag");
        let src = "/* unsupported */\nmodule probe.silentbad\n\nfn f() -> Int {\n  1\n}\n";
        assert_eq!(
            parse_module_binding(path, src).expect("unclassified today"),
            ModuleBindingOutcome::ModuleBindingUnclassified,
            "residue: a parse failure with no recognized header is not distinguishable \
             from a fragment, so it lands unclassified rather than refusing"
        );
    }

    // Case 3 — a GENUINE fragment: also unclassified. Same arm as case 2, which is
    // precisely the residue: this test and the one above cannot be told apart by
    // the index, and both assertions are identical by necessity, not by accident.
    #[test]
    fn genuine_fragment_also_lands_unclassified() {
        let path = Path::new("fixture.dag");
        assert_eq!(
            parse_module_binding(path, "type Foo { x: Int }\n").expect("must parse"),
            ModuleBindingOutcome::ModuleBindingUnclassified
        );
    }

    // Case 4 — valid module: binding with identity and location.
    #[test]
    fn valid_module_binds_with_location() {
        let path = Path::new("fixture.dag");
        match parse_module_binding(path, "module v1.test.fixture\n").expect("must parse") {
            ModuleBindingOutcome::Bound(b) => {
                assert_eq!(b.module_path, "v1.test.fixture");
                assert_eq!(b.ident_span.start, 7);
            }
            other => panic!("expected a binding, got {:?}", other),
        }
    }

    // Same-basename files must not produce indistinguishable locations.
    #[test]
    fn refusal_path_distinguishes_same_basename_files() {
        let src = "module probe.bad\n\ndata BadScope\n  = | OnlyVariant\n";
        let a = parse_module_binding(Path::new("/w/alpha/types.dag"), src).expect_err("refuse");
        let b = parse_module_binding(Path::new("/w/beta/types.dag"), src).expect_err("refuse");
        assert_ne!(
            a.source_path, b.source_path,
            "two same-basename files must be distinguishable"
        );
    }

    #[test]
    fn parse_orchestration_dag_binding() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("../../../src/v2/std/orchestration.dag");
        let content = std::fs::read_to_string(&path).expect("read orchestration.dag");
        match parse_module_binding(&path, &content).expect("orchestration.dag must parse") {
            ModuleBindingOutcome::Bound(b) => assert_eq!(b.module_path, "v2.std.orchestration"),
            other => panic!("unexpected: {:?}", other),
        }
    }
}

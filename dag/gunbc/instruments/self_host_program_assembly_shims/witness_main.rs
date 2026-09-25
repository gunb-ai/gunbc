use im::vector as vec;
use std::rc::Rc;

use v1_compiled::v2_compiler_name_resolve::{Admission, ResolutionSubject};
use v1_compiled::v2_compiler_program_assembly as emitted;
use v1_compiled::v2_compiler_source_authority::DagSourceReadWitness;
use v1_compiled::v2_compiler_target_carriers::lossless_source;
use v1_compiled::v2_extdeps_languages_dag::dag_language_model;
use v1_compiled::v2_std_artifact::{Artifact, ArtifactKind};
use v1_compiled::v2_std_cross_tree_import_model::SourceRootRef;
use v1_compiled::v2_std_diagnostic::Outcome;
use v1_compiled::v2_std_qualified_name::qualified_name_from_dotted_string;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// program_assembly_prepare_once_note against a hand copy in v1_compiler, which said nothing about
// what assembly does. Its expected verdicts are read off the authority instead:
// src/v2/compiler/program_assembly.dag assemble_program_from_ingest tokenizes, parses, normalizes
// and resolves every read in the ingest, so a well-formed one-module ingest is Accepted, and a
// read the .dag grammar cannot parse fails the whole assembly (ProgramAssemblyFoldFailed), never a
// per-module silent skip.

const MODULE: &str = "self_host.program_assembly_witness";
const WELL_FORMED: &str =
    "module self_host.program_assembly_witness\nfn add(x: Int, y: Int) -> Int { x + y }\n";
const UNPARSEABLE: &str = "module self_host.program_assembly_witness\nfn add(x: Int, y: Int -> Int {\n";

// Accepted, or the refusal reasons in chain order.
fn assemble(text: &str) -> Result<(), Vec<String>> {
    let read = Rc::new(DagSourceReadWitness {
        source: lossless_source(text.to_string()),
        artifact: Rc::new(Artifact {
            kind: ArtifactKind::SourceFile,
            id: "self_host_program_assembly_witness_artifact".to_string(),
            file_path: "self_host/program_assembly_witness.dag".to_string(),
        }),
        compilation_unit: "self_host_program_assembly_witness_unit".to_string(),
        source_root: SourceRootRef::V2Tree,
    });
    let admission = Rc::new(Admission {
        subject: Rc::new(ResolutionSubject {
            name: qualified_name_from_dotted_string(MODULE.to_string()),
        }),
        imports: Rc::new(vec![]),
    });
    let outcome = emitted::assemble_program_from_ingest(Rc::new(vec![read]), admission, dag_language_model());
    match &*outcome {
        Outcome::Accepted { .. } => Ok(()),
        Outcome::Rejected { diagnostics } => Err(std::iter::once(&diagnostics.head)
            .chain(diagnostics.tail.iter())
            .map(|d| d.reason.clone())
            .collect()),
    }
}

// The planted fault feeds the unparseable read where the well-formed one is expected, so the
// fault run goes red only if emitted assembly really refuses it.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let valid = assemble(if inject_fault { UNPARSEABLE } else { WELL_FORMED });
    let valid_accepts = valid.is_ok();
    println!("assemble well_formed inject_fault={inject_fault} accepts={valid_accepts}");
    // The ROUTE, not only the verdict: the refusal must carry the parse stage's syntax error, so a
    // refusal reached for some other reason (a grammar-level residue riding the chain) does not
    // count.
    let invalid = assemble(UNPARSEABLE);
    let invalid_refuses = matches!(&invalid, Err(reasons) if reasons.iter().any(|r| r == "parse_e1_syntax_error"));
    let distinct: std::collections::BTreeSet<&String> = invalid.as_ref().err().into_iter().flatten().collect();
    println!("assemble unparseable distinct_reasons={distinct:?} refuses_with_syntax_error={invalid_refuses}");
    if valid_accepts && invalid_refuses {
        println!("SELF_HOST_PROGRAM_ASSEMBLY_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_PROGRAM_ASSEMBLY_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}

// Lane scratch (never lands; cited receipt tool): print ONLY the emitted `fn probe_case`
// window for the four unary-prefix discriminator bodies, so the claim's substring poles are
// calibrated by observation (E3: bang poles true, minus poles false => both minus substrings
// were guesses; E4/E5 showed crate-level dumps flood any pipe).
// Imports follow src/v1/stage0/src/bin/bootstrap_witness.rs (the in-tree compile_sources caller).
use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn dump(label: &str, module: &str, params: &str, ret: &str, body: &str) {
    let src = format!("module {module}\n\nfn probe_case({params}) -> {ret} {{\n  {body}\n}}\n");
    let sources: Vec<Rc<SourceFile>> = vec![Rc::new(SourceFile {
        path: format!("probe_{label}.dag"),
        content: src,
    })];
    let result = compile_sources(Rc::new(sources.into()), RenderTarget::Rust);
    let mut found = false;
    for f in result.files.iter() {
        if let Some(idx) = f.content.find("fn probe_case") {
            found = true;
            let mut end = (idx + 320).min(f.content.len());
            while end > idx && !f.content.is_char_boundary(end) {
                end -= 1;
            }
            let win = &f.content[idx..end];
            println!("BEGIN {label} | body: {body} | file {}", f.path);
            println!("{win}");
            println!("END {label}");
        }
    }
    if !found {
        println!(
            "BEGIN {label} | body: {body} | NO probe_case IN ANY FILE ({} files)",
            result.files.len()
        );
        println!("END {label}");
    }
}

fn main() {
    dump(
        "minus_fixed",
        "probe.unary_prefix.minus",
        "",
        "Int",
        "-1 - 2 - 3",
    );
    dump(
        "minus_bug",
        "probe.unary_prefix.minusbug",
        "",
        "Int",
        "-(1 - 2 - 3)",
    );
    dump(
        "bang_fixed",
        "probe.unary_prefix.bang",
        "t: Bool, f: Bool",
        "Bool",
        "!t && f && t",
    );
    dump(
        "bang_bug",
        "probe.unary_prefix.bangbug",
        "t: Bool, f: Bool",
        "Bool",
        "!(t && f && t)",
    );
}

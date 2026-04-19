// Unified lens-regen driver. Narrow host shim for
// `src/v3/compiler/regen.dag`: reads every
// `data <name>_entry: LensRegistryEntry` record out of the bootstrap
// Dag, compiles the referenced `.dag` lens, and writes the
// `emit_rust_module` projection to the declared output path. Adding
// a new lens is an edit to `regen.dag`, not to this driver.
//
// Usage:
//   cargo run -p v3-compiler --bin regen_lens
//     → regenerates every lens in the registry.
//   cargo run -p v3-compiler --bin regen_lens -- --lens cost
//     → regenerates only the entry whose `name` field is "cost".

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::emit_rust::emit_rust_module;

const LENS_REGISTRY_ENTRY_TYPE: &str = "LensRegistryEntry";

struct Entry {
    name: String,
    lens_file: String,
    generated_file: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let requested_name = parse_args()?;

    let dag = Dag::new();
    if !dag.diagnostics().is_empty() {
        return Err(format!(
            "bootstrap Dag carries {} diagnostic(s): {:#?}",
            dag.diagnostics().len(),
            dag.diagnostics()
        ));
    }

    let entries = read_registry(&dag)?;
    if entries.is_empty() {
        return Err("lens registry is empty; check `src/v3/compiler/regen.dag`".to_string());
    }

    let root = workspace_root();
    let mut processed = 0usize;
    for entry in &entries {
        if let Some(requested) = &requested_name {
            if entry.name != *requested {
                continue;
            }
        }
        regen_entry(&root, entry)?;
        processed += 1;
    }

    if processed == 0 {
        let known: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        return Err(format!(
            "no lens entry matches `--lens {}`; known: {}",
            requested_name.unwrap_or_default(),
            known.join(", ")
        ));
    }
    Ok(())
}

fn parse_args() -> Result<Option<String>, String> {
    let mut requested: Option<String> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lens" => {
                requested = Some(
                    args.next()
                        .ok_or_else(|| "--lens requires a name".to_string())?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(requested)
}

fn workspace_root() -> PathBuf {
    // src/v3/compiler/Cargo.toml lives 3 levels below the workspace
    // root; the registry entries record paths relative to that root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn read_registry(dag: &Dag) -> Result<Vec<Entry>, String> {
    let entry_type_id = dag
        .declaration_by_name(LENS_REGISTRY_ENTRY_TYPE)
        .map(|decl| decl.id)
        .ok_or_else(|| format!("missing `{LENS_REGISTRY_ENTRY_TYPE}` in bootstrap Dag"))?;

    let mut entries = Vec::new();
    for decl in dag.declarations() {
        if decl.meta_tag != Some(entry_type_id) {
            continue;
        }
        let binding_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            return Err(format!(
                "lens registry entry `{binding_name}` must carry a structural value body"
            ));
        };
        entries.push(Entry {
            name: require_string(fields, "name", binding_name)?,
            lens_file: require_string(fields, "lens_file", binding_name)?,
            generated_file: require_string(fields, "generated_file", binding_name)?,
        });
    }
    Ok(entries)
}

fn require_string(
    fields: &[(String, FieldValue)],
    label: &str,
    binding_name: &str,
) -> Result<String, String> {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            format!("lens registry entry `{binding_name}` is missing String field `{label}`")
        })
}

fn regen_entry(root: &Path, entry: &Entry) -> Result<(), String> {
    let lens_path = root.join(&entry.lens_file);
    let out_path = root.join(&entry.generated_file);

    let source = std::fs::read_to_string(&lens_path)
        .map_err(|e| format!("read {}: {e}", lens_path.display()))?;
    let dag = compile_to_dag(&source, &entry.lens_file)
        .map_err(|e| format!("compile {}: {e:?}", entry.lens_file))?;
    let raw = emit_rust_module(&dag).map_err(|e| format!("emit {}: {e:?}", entry.lens_file))?;
    let header = format!(
        "// AUTO-GENERATED from `{}` via\n\
         // `emit_rust_module`. Regenerate instead of hand-editing.\n\n",
        entry.lens_file
    );
    let combined = format!("{header}{raw}");
    let formatted = rustfmt_stdin(&combined)?;
    std::fs::write(&out_path, &formatted)
        .map_err(|e| format!("write {}: {e}", out_path.display()))?;
    println!("wrote {}", out_path.display());
    Ok(())
}

fn rustfmt_stdin(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .map_err(|e| format!("write rustfmt stdin: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("rustfmt output not utf8: {e}"))
}

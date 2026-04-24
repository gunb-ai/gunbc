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

use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};
use v3_compiler::emit_rust::emit_rust_module;
use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::{
    compile_to_dag, patch_lower_helpers_generated_type_alias_refinement, CompileError,
};

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

    let mut entries: Vec<Entry> = Vec::new();
    // Dedup maps: record the FIRST binding that introduced each key,
    // so a duplicate error names both the existing owner and the
    // colliding binding. `--lens <name>` is a singleton key and
    // `generated_file` is the output an entry writes to; either
    // collision would make the registry ambiguous or let two
    // entries race to clobber the same file.
    let mut seen_name: HashMap<String, String> = HashMap::new();
    let mut seen_generated_file: HashMap<String, String> = HashMap::new();

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
        let entry = Entry {
            name: require_string(fields, "name", binding_name)?,
            lens_file: require_string(fields, "lens_file", binding_name)?,
            generated_file: require_string(fields, "generated_file", binding_name)?,
        };

        if let Some(prior_binding) = seen_name.get(&entry.name) {
            return Err(format!(
                "lens registry has duplicate `name` field `{name}`: first declared by `{prior_binding}`, re-declared by `{binding_name}`",
                name = entry.name,
            ));
        }
        if let Some(prior_binding) = seen_generated_file.get(&entry.generated_file) {
            return Err(format!(
                "lens registry has duplicate `generated_file` path `{path}`: first declared by `{prior_binding}`, re-declared by `{binding_name}`",
                path = entry.generated_file,
            ));
        }
        seen_name.insert(entry.name.clone(), binding_name.to_string());
        seen_generated_file.insert(entry.generated_file.clone(), binding_name.to_string());
        entries.push(entry);
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
    // Single-authority gate: the registry's `generated_file` path
    // must also be registered in `REGEN_OUTPUTS` (surfaced as
    // `v3_compiler::generated_files::GENERATED_FILES`). SG-0 treats
    // that manifest as the sole producer-owned partition; if
    // `regen.dag` and `REGEN_OUTPUTS` drift, the driver would
    // silently write to a path the SG-0 census doesn't know about.
    // Fail closed here rather than rely on the downstream census to
    // notice — the error points the reviewer at the two authorities
    // that must stay in lockstep.
    if !GENERATED_FILES.iter().any(|p| *p == entry.generated_file) {
        return Err(format!(
            "registry / manifest drift: `regen.dag` declares \
             `generated_file = \"{path}\"` (lens `{lens}`) but the \
             path is not registered in `REGEN_OUTPUTS` in \
             `src/v3/compiler/build.rs`. Add the path to `REGEN_OUTPUTS` \
             (or remove it from `regen.dag`) so the two authorities stay \
             in lockstep. Both are SG-0's producer-owned manifest; \
             writing to a path outside the manifest would be silent \
             drift.",
            path = entry.generated_file,
            lens = entry.name,
        ));
    }

    let lens_path = root.join(&entry.lens_file);
    let out_path = root.join(&entry.generated_file);

    let source = std::fs::read_to_string(&lens_path)
        .map_err(|e| format!("read {}: {e}", lens_path.display()))?;
    let dag = compile_to_dag(&source, &entry.lens_file).map_err(|e| match e {
        CompileError::Semantic(dag) => format!(
            "compile {}: {}",
            entry.lens_file,
            dag.diagnostics()
                .iter()
                .map(|d| format!("{d:?}"))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        other => format!("compile {}: {other:?}", entry.lens_file),
    })?;
    let raw = emit_rust_module(&dag).map_err(|e| format!("emit {}: {e:?}", entry.lens_file))?;
    let header = format!(
        "// AUTO-GENERATED from `{}` via\n\
         // `emit_rust_module`. Regenerate instead of hand-editing.\n\n",
        entry.lens_file
    );
    let combined = format!("{header}{raw}");
    let mut formatted = rustfmt_stdin(&combined)?;
    if entry.generated_file.ends_with("lower_helpers_generated.rs") {
        formatted = patch_lower_helpers_generated_type_alias_refinement(&formatted);
        formatted = rustfmt_stdin(&formatted)?;
    }
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

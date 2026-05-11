use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use v3_compiler::{
    compare_stage_snapshots, compile_stage_snapshots, compile_to_dag,
    dag::{Dag, FieldValue, LiteralBits, ValueBody},
    default_fixed_point_source,
    emit_rust::emit_rust_module,
    generated_files::GENERATED_FILES,
    CompileError, FixedPointMismatch, StageSnapshotError,
};

const LENS_REGISTRY_ENTRY_TYPE: &str = "LensRegistryEntry";

struct LensEntry {
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
    let mut raw_args = env::args().skip(1);
    if matches!(raw_args.next().as_deref(), Some("lens")) {
        return run_lens(raw_args);
    }

    let mut source_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut inject_stage: Option<String> = env::var("GUNBC_REGEN_INJECT_STAGE").ok();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "lens" => unreachable!("handled before fixed-point arg parsing"),
            "--source" => {
                source_path = Some(
                    args.next()
                        .ok_or_else(|| "--source requires a path".to_string())?,
                );
            }
            "--file" => {
                file_name = Some(
                    args.next()
                        .ok_or_else(|| "--file requires a name".to_string())?,
                );
            }
            "--inject-stage" => {
                inject_stage = Some(
                    args.next()
                        .ok_or_else(|| "--inject-stage requires a stage name".to_string())?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let (source, file) = match source_path {
        Some(path) => {
            let source = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read {}: {err}", path))?;
            let file = file_name.unwrap_or_else(|| {
                Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("input.v3")
                    .to_string()
            });
            (source, file)
        }
        None => (
            default_fixed_point_source().to_string(),
            file_name.unwrap_or_else(|| "fixed_point_input.v3".to_string()),
        ),
    };

    let pass1 = compile_stage_snapshots(&source, &file).map_err(render_snapshot_error)?;
    let mut pass2 = compile_stage_snapshots(&source, &file).map_err(render_snapshot_error)?;

    if let Some(stage_name) = inject_stage {
        let snapshot = pass2
            .iter_mut()
            .find(|snapshot| snapshot.stage == stage_name)
            .ok_or_else(|| format!("unknown stage `{stage_name}`"))?;
        snapshot
            .bytes
            .extend_from_slice(b"\n# synthetic divergence\n");
        snapshot.dag = None;
    }

    compare_stage_snapshots(&pass1, &pass2).map_err(render_mismatch)?;
    println!("fixed-point verified across {} stages", pass1.len());
    Ok(())
}

fn run_lens(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let requested_name = parse_lens_args(&mut args)?;

    let dag = Dag::new();
    if !dag.diagnostics().is_empty() {
        return Err(format!(
            "bootstrap Dag carries {} diagnostic(s): {:#?}",
            dag.diagnostics().len(),
            dag.diagnostics()
        ));
    }

    let entries = read_lens_registry(&dag)?;
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
        regen_lens_entry(&root, entry)?;
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

fn parse_lens_args(args: &mut impl Iterator<Item = String>) -> Result<Option<String>, String> {
    let mut requested: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lens" => {
                requested = Some(
                    args.next()
                        .ok_or_else(|| "--lens requires a name".to_string())?,
                );
            }
            other => return Err(format!("unknown lens argument `{other}`")),
        }
    }
    Ok(requested)
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn read_lens_registry(dag: &Dag) -> Result<Vec<LensEntry>, String> {
    let entry_type_id = dag
        .declaration_by_name(LENS_REGISTRY_ENTRY_TYPE)
        .map(|decl| decl.id)
        .ok_or_else(|| format!("missing `{LENS_REGISTRY_ENTRY_TYPE}` in bootstrap Dag"))?;

    let mut entries: Vec<LensEntry> = Vec::new();
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
        let entry = LensEntry {
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

fn regen_lens_entry(root: &Path, entry: &LensEntry) -> Result<(), String> {
    if !GENERATED_FILES.iter().any(|p| *p == entry.generated_file) {
        return Err(format!(
            "registry / manifest drift: `regen.dag` declares \
             `generated_file = \"{path}\"` (lens `{lens}`) but the \
             path is not registered in `REGEN_OUTPUTS` in \
             `src/v3/compiler/build.rs`.",
            path = entry.generated_file,
            lens = entry.name,
        ));
    }

    let lens_path = root.join(&entry.lens_file);
    let out_path = root.join(&entry.generated_file);

    let source =
        fs::read_to_string(&lens_path).map_err(|e| format!("read {}: {e}", lens_path.display()))?;
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
    let formatted = rustfmt_stdin(&format!("{header}{raw}"))?;
    fs::write(&out_path, &formatted).map_err(|e| format!("write {}: {e}", out_path.display()))?;
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

fn render_snapshot_error(error: StageSnapshotError) -> String {
    match error {
        StageSnapshotError::Compile(error) => format!("compile failed: {error:?}"),
        StageSnapshotError::Emit(error) => format!("emit failed: {error:?}"),
        StageSnapshotError::Pipeline(error) => format!("pipeline authority failed: {error}"),
    }
}

fn render_mismatch(mismatch: FixedPointMismatch) -> String {
    format!(
        "FIXED-POINT FAILURE at stage `{}`: {}",
        mismatch.stage, mismatch.detail
    )
}

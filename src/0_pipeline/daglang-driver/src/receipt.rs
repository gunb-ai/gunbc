use super::*;

use sha2::{Digest, Sha256};

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub(crate) fn compute_source_digest_from_module_graph(module_graph: &ModuleGraph) -> String {
    let mut source_hashes: Vec<String> = module_graph
        .modules
        .iter()
        .map(|module| {
            format!(
                "{}:{}",
                module.path.display(),
                sha256_hex(module.source.as_bytes())
            )
        })
        .collect();
    source_hashes.sort();
    sha256_hex(source_hashes.join("\n").as_bytes())
}

/// Compute a deterministic compilation receipt from the compilation artifacts.
///
/// The receipt contains content-addressable digests for source files, the
/// canonical IR, and the emit manifest. Two compilations of the same input
/// MUST produce identical receipts — this is the determinism contract.
pub(crate) fn compute_receipt(
    dag: &Dag<LoweredOp>,
    emitted: &EmissionBundle,
    emit_manifest_path: &str,
    source_digest: &str,
) -> Result<CompileReceipt, CompileError> {
    let canonical_json = daglang_lower::canonical_ir_json(dag)
        .map_err(|e| CompileError::Message(format!("failed to render canonical IR JSON: {e}")))?;
    let program_ir_digest = sha256_hex(canonical_json.as_bytes());

    let emit_manifest_digest = emitted
        .files
        .iter()
        .find(|f| f.path == emit_manifest_path)
        .map(|f| sha256_hex(f.content.as_bytes()))
        .ok_or_else(|| {
            CompileError::Message(format!(
                "emit manifest `{emit_manifest_path}` missing from emitted files"
            ))
        })?;

    Ok(CompileReceipt {
        source_digest: source_digest.to_string(),
        program_ir_digest,
        emit_manifest_digest,
    })
}

/// Compute the source digest for a compilation context without performing
/// the full compilation pipeline (C26).
///
/// Discovers the module graph and computes a content-addressable SHA-256
/// digest from the already loaded module sources. This is much cheaper than
/// full compilation (parse + typecheck + lower + emit).
pub fn compute_source_digest_for_context(context: &DriverContext) -> Result<String, CompileError> {
    let module_graph = discover_module_graph_for_context(context)?;
    Ok(compute_source_digest_from_module_graph(&module_graph))
}

/// Compute a source digest from a list of source file paths.
///
/// Returns SHA-256 of sorted path:hash pairs.
/// Returns an error if any file cannot be read.
///
/// Build-time filesystem access (compiler bootstrap exception).
pub fn compute_source_digest(source_paths: &[PathBuf]) -> Result<String, std::io::Error> {
    let mut source_hashes: Vec<String> = Vec::with_capacity(source_paths.len());
    for path in source_paths {
        #[allow(clippy::disallowed_methods)]
        let content = std::fs::read(path).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("cannot read {} for source digest: {e}", path.display()),
            )
        })?;
        source_hashes.push(format!("{}:{}", path.display(), sha256_hex(&content)));
    }
    source_hashes.sort();
    Ok(sha256_hex(source_hashes.join("\n").as_bytes()))
}

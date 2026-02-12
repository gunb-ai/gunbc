//! Preflight helpers for ensuring lint state before running binaries.
//!
//! This enforces an "upsert lint" policy:
//! - If lint inputs are fresh, do nothing.
//! - If stale or missing, run codegen/testgen/pragma + clippy fix/lint,
//!   then update the resource manifest.
//!
//! Preflight commands always use `--release` to reuse cached release artifacts
//! and avoid triggering full debug rebuilds in CI.

use crate::ops::execute_request;
use crate::TransportIo;
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand, Warnings};
use gunbc_ir::resource::{
    load_manifest_default, save_manifest_default, ContentHash, ExecMode, ManagedResource,
    ManifestEntry, ResourceDef, ResourceError, ResourceIo, ResourceManifest, ResourceState,
};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::ResourceId;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PREFLIGHT_SKIP_BINARIES: &[&str] = &[
    "gunbc-codegen",
    "gunbc-codegen-dag",
    "gunbc-testgen",
    "gunbc-pragma",
    "gunbc-makegen",
];

const PREFLIGHT_ENV_DISABLE: &str = "GUNBC_PREFLIGHT_DISABLE";

/// Ensure lint is fresh (run lint-upsert if stale/missing).
pub fn ensure_lint_upsert() -> Result<(), String> {
    if should_skip_preflight() {
        return Ok(());
    }

    let io = TransportIo::new();
    let resource = LintResource::new();

    let mut manifest = load_manifest_default(&io)
        .map_err(|e| format!("preflight: manifest load failed: {}", e))?;

    let state = resource.check_state(&manifest, &io);
    if state.is_fresh() {
        return Ok(());
    }
    if state.is_error() {
        return Err(format!("preflight: lint state error: {}", state));
    }

    println!("preflight: lint-upsert ({})", state);

    resource
        .acquire(ExecMode::Ensure, &mut manifest, &io)
        .map_err(|e| format!("preflight: lint-upsert failed: {}", e))?;

    save_manifest_default(&io, &manifest)
        .map_err(|e| format!("preflight: manifest save failed: {}", e))?;

    Ok(())
}

fn should_skip_preflight() -> bool {
    if std::env::var(PREFLIGHT_ENV_DISABLE).is_ok() {
        return true;
    }

    let Some(name) = current_binary_name() else {
        return false;
    };
    PREFLIGHT_SKIP_BINARIES.iter().any(|skip| *skip == name)
}

fn current_binary_name() -> Option<String> {
    let from_exe = std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|s| s.to_string_lossy().to_string()));
    if from_exe.is_some() {
        return from_exe;
    }

    std::env::args().next().and_then(|arg0| {
        Path::new(&arg0)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
    })
}

#[derive(Clone)]
struct LintResource {
    def: ResourceDef,
}

impl LintResource {
    fn new() -> Self {
        Self {
            def: ResourceDef::new(ResourceId::build("lint_upsert")),
        }
    }
}

impl ManagedResource for LintResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn check_state(&self, manifest: &ResourceManifest, io: &dyn ResourceIo) -> ResourceState {
        let entry = match manifest.get(self.resource_id()) {
            Some(e) => e,
            None => return ResourceState::Missing,
        };

        let files = match list_tracked_files(io) {
            Ok(f) if !f.is_empty() => f,
            Ok(_) => {
                return ResourceState::error(
                    "no tracked lint inputs found (are you in the repo root?)",
                )
            }
            Err(e) => return ResourceState::error(format!("list inputs failed: {}", e)),
        };

        if let Some(prev_files) = &entry.input_files {
            let curr_files: Vec<String> = files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            if &curr_files != prev_files {
                return ResourceState::stale(
                    "tracked file set changed",
                    entry.key.clone(),
                    ContentHash::empty(),
                );
            }
        } else if files.len() != entry.input_file_count {
            return ResourceState::stale(
                "tracked file count changed",
                entry.key.clone(),
                ContentHash::empty(),
            );
        }

        let created_at = millis_to_system_time(entry.created_at);
        for path in &files {
            match io.file_mtime(path) {
                Ok(modified) => {
                    if modified > created_at {
                        return ResourceState::stale(
                            format!("file newer than manifest: {}", path.display()),
                            entry.key.clone(),
                            ContentHash::empty(),
                        );
                    }
                }
                Err(e) => {
                    if let ResourceError::Io(io_err) = &e {
                        if io_err.kind() == std::io::ErrorKind::NotFound {
                            return ResourceState::stale(
                                format!("tracked file missing: {}", path.display()),
                                entry.key.clone(),
                                ContentHash::empty(),
                            );
                        }
                    }
                    return ResourceState::error(format!(
                        "failed to stat {}: {}",
                        path.display(),
                        e
                    ));
                }
            }
        }

        ResourceState::Fresh
    }

    fn create(
        &self,
        _manifest: &ResourceManifest,
        io: &dyn ResourceIo,
    ) -> Result<ManifestEntry, ResourceError> {
        run_lint_upsert(self.resource_id())?;

        let files = list_tracked_files(io)?;
        let key = compute_lint_key(io, &files)?;
        let file_list: Vec<String> = files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Ok(ManifestEntry::new(key, file_list.len()).with_input_files(file_list))
    }
}

fn list_tracked_files(io: &dyn ResourceIo) -> Result<Vec<PathBuf>, ResourceError> {
    let root = repo_root(io)?;
    let root_str = root.to_string_lossy().to_string();

    let args = vec![
        "-C".to_string(),
        root_str,
        "ls-files".to_string(),
        "-z".to_string(),
        "--".to_string(),
        "**/*.rs".to_string(),
        "**/Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        "clippy.toml".to_string(),
        "rustfmt.toml".to_string(),
        "rust-toolchain".to_string(),
        "rust-toolchain.toml".to_string(),
        "deny.toml".to_string(),
        ".cargo/config".to_string(),
        ".cargo/config.toml".to_string(),
    ];

    let output = io.command_output("git", &args)?;
    let mut files = Vec::new();
    for part in output.split(|b| *b == 0) {
        if part.is_empty() {
            continue;
        }
        let rel = std::str::from_utf8(part)
            .map_err(|e| ResourceError::Io(std::io::Error::other(e.to_string())))?;
        if rel.trim().is_empty() {
            continue;
        }
        files.push(root.join(rel));
    }

    files.sort();
    Ok(files)
}

fn repo_root(io: &dyn ResourceIo) -> Result<PathBuf, ResourceError> {
    let output = io.command_output(
        "git",
        &["rev-parse".to_string(), "--show-toplevel".to_string()],
    )?;
    let root = String::from_utf8(output)
        .map_err(|e| ResourceError::Io(std::io::Error::other(e.to_string())))?;
    let root = root.trim();
    if root.is_empty() {
        return Err(ResourceError::Io(std::io::Error::other(
            "git repo root not found",
        )));
    }
    Ok(PathBuf::from(root))
}

fn compute_lint_key(io: &dyn ResourceIo, files: &[PathBuf]) -> Result<ContentHash, ResourceError> {
    let mut hash_builder = gunbc_ir::resource::HashBuilder::new();
    hash_builder = hash_builder.update(b"lint-upsert\0");

    for path in files {
        match io.read_file(path) {
            Ok(contents) => {
                hash_builder = hash_builder.update_file_content(path, &contents);
            }
            Err(_) => {
                hash_builder = hash_builder.update(b"missing\0");
                hash_builder = hash_builder.update(path.to_string_lossy().as_bytes());
            }
        }
    }

    if let Ok(stdout) = io.command_output("rustc", &["--version".to_string()]) {
        hash_builder =
            hash_builder.update_command_output_bytes("rustc", &["--version".to_string()], &stdout);
    } else {
        hash_builder = hash_builder.update(b"rustc:missing\0");
    }

    if let Ok(stdout) = io.command_output("cargo", &["clippy".to_string(), "--version".to_string()])
    {
        hash_builder = hash_builder.update_command_output_bytes(
            "cargo",
            &["clippy".to_string(), "--version".to_string()],
            &stdout,
        );
    } else {
        hash_builder = hash_builder.update(b"cargo-clippy:missing\0");
    }

    Ok(hash_builder.finalize())
}

fn run_lint_upsert(resource_id: &ResourceId) -> Result<(), ResourceError> {
    // Always use --release for preflight commands. These tools (codegen, testgen,
    // pragma) don't benefit from debug mode, and using release ensures we reuse
    // cached artifacts when CI runs with --release (avoiding a 30+ min rebuild).

    let steps: &[(&str, CargoCommand)] = &[
        (
            "codegen-dag",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "codegen-dag",
                "dag",
            )))
            .release(),
        ),
        (
            "testgen",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed("testgen", "dag")))
                .release(),
        ),
        (
            "pragma",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed("pragma", "dag")))
                .release(),
        ),
    ];

    let total = steps.len() + 1; // +1 for clippy
    for (i, (label, cmd)) in steps.iter().enumerate() {
        eprint!("  [{}/{}] {}...", i + 1, total, label);
        let start = std::time::Instant::now();
        run_cargo_command(resource_id, cmd)?;
        eprintln!(" {:.1}s", start.elapsed().as_secs_f64());
    }

    // clippy check: cargo clippy -- -D warnings
    // Note: no --all-targets for speed; CI still catches test-only lint issues
    eprint!("  [{}/{}] clippy...", total, total);
    let clippy_start = std::time::Instant::now();
    let clippy_check = CargoCommand::new(Subcommand::Clippy).warnings(Warnings::Deny);
    let clippy_result = run_cargo_command_response(resource_id, &clippy_check)?;

    if !clippy_result.success() {
        eprintln!(" fix needed ({:.1}s)", clippy_start.elapsed().as_secs_f64());
        eprint!("  [{}/{}] clippy --fix...", total, total);
        let fix_start = std::time::Instant::now();

        // clippy fix: cargo clippy --fix --workspace --allow-dirty --allow-staged -- -D warnings
        let clippy_fix = CargoCommand::new(Subcommand::Clippy)
            .fix()
            .workspace()
            .allow_dirty()
            .allow_staged()
            .warnings(Warnings::Deny);
        run_cargo_command(resource_id, &clippy_fix)?;

        // verify fix worked
        let verify_result = run_cargo_command_response(resource_id, &clippy_check)?;
        if !verify_result.success() {
            eprintln!(" failed ({:.1}s)", fix_start.elapsed().as_secs_f64());
            return Err(ResourceError::CreateFailed(
                resource_id.clone(),
                format!(
                    "cargo clippy failed after fix (exit {})\n{}",
                    verify_result.exit_code, verify_result.stderr
                ),
            ));
        }
        eprintln!(" {:.1}s", fix_start.elapsed().as_secs_f64());
    } else {
        eprintln!(" {:.1}s", clippy_start.elapsed().as_secs_f64());
    }

    Ok(())
}

/// Run a cargo command, failing on non-zero exit.
fn run_cargo_command(
    resource_id: &ResourceId,
    cmd: &CargoCommand,
) -> Result<(), ResourceError> {
    let response = run_cargo_command_response(resource_id, cmd)?;
    if response.success() {
        Ok(())
    } else {
        Err(ResourceError::CreateFailed(
            resource_id.clone(),
            format!(
                "command failed (exit {}): {}\n{}",
                response.exit_code,
                cmd.to_shell(),
                response.stderr
            ),
        ))
    }
}

/// Run a cargo command and return the response.
fn run_cargo_command_response(
    resource_id: &ResourceId,
    cmd: &CargoCommand,
) -> Result<gunbc_ir::transport::ShellResponse, ResourceError> {
    // Preflight steps compile + run cargo binaries; in CI with cold caches
    // this can take well over 5 minutes. Use FermiCost::L (30 min).
    const PREFLIGHT_TIMEOUT_MS: u64 = 1_800_000; // FermiCost::L = 30 min

    let mut request = cmd.to_shell_request().timeout(PREFLIGHT_TIMEOUT_MS);
    // Disable recursive preflight in child processes
    request = request.env(PREFLIGHT_ENV_DISABLE, "1");

    let response = execute_request(&TransportRequest::Shell(request))
        .map_err(|e| ResourceError::CreateFailed(resource_id.clone(), e.to_string()))?;

    match response {
        TransportResponse::Shell(shell) => Ok(shell),
        other => Err(ResourceError::CreateFailed(
            resource_id.clone(),
            format!(
                "unexpected transport response for shell: {:?}",
                std::mem::discriminant(&other)
            ),
        )),
    }
}

fn millis_to_system_time(millis: i64) -> SystemTime {
    if millis >= 0 {
        UNIX_EPOCH + Duration::from_millis(millis as u64)
    } else {
        UNIX_EPOCH
    }
}

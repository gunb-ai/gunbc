//! Preflight helpers for ensuring lint state before running binaries.
//!
//! This enforces an "upsert lint" policy:
//! - If lint inputs are fresh, do nothing.
//! - If stale or missing, run codegen/testgen/pragma + clippy fix/lint,
//!   then update the resource manifest.
//!
//! Preflight commands use the default (debug) profile so they share the
//! same compilation cache as dev builds — no separate release recompilation.

use crate::ops::execute_request;
use crate::TransportIo;
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand, Warnings};
use gunbc_ir::resource::{
    load_manifest_default, save_manifest_default, ContentHash, ManagedResource, ManifestEntry,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest, ResourceState,
};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use gunbc_ir::ResourceId;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Binaries that should never trigger preflight (they ARE preflight tools).
const PREFLIGHT_SKIP_BINARIES: &[&str] = &[
    "gunbc-codegen",
    "gunbc-codegen-dag",
    "gunbc-testgen",
    "gunbc-pragma",
    "gunbc-makegen",
];

/// Ensure lint is fresh (run lint-upsert if stale/missing).
pub fn ensure_lint_upsert() -> Result<(), String> {
    ensure_lint_upsert_with_observer(None)
}

/// Ensure lint is fresh with optional CI context for structured output.
///
/// Deprecated: use [`ensure_lint_upsert_with_observer`] instead.
/// This adapter wraps the CiContext in a `PreflightObserver`.
pub fn ensure_lint_upsert_with_ci(ci: Option<&mut gunbc_exec::CiContext>) -> Result<(), String> {
    match ci {
        Some(c) => ensure_lint_upsert_with_observer(Some(c)),
        None => ensure_lint_upsert_with_observer(None),
    }
}

/// Ensure lint is fresh with optional observer for structured output.
///
/// When a [`PreflightObserver`] is provided, preflight emits structured
/// events for steps, completion, and errors. Without an observer, preflight
/// runs silently (output only on state change via the `println!` in
/// `ensure_lint_upsert_manifest_state`).
pub fn ensure_lint_upsert_with_observer(
    observer: Option<&mut dyn gunbc_exec::PreflightObserver>,
) -> Result<(), String> {
    if should_skip_preflight() {
        return Ok(());
    }

    let preflight_start = std::time::Instant::now();

    let io = TransportIo::new();
    let resource = LintResource::new();

    let mut manifest = load_manifest_default(&io)
        .map_err(|e| format!("preflight: manifest load failed: {}", e))?;

    if let Some(obs) = observer {
        obs.on_preflight_start("preflight");
        let updated = match ensure_lint_upsert_manifest_state(&io, &mut manifest, &resource, |id| {
            run_lint_upsert(id, Some(obs))
        }) {
            Ok(updated) => updated,
            Err(msg) => {
                obs.on_preflight_error("preflight", &msg);
                return Err(msg);
            }
        };

        if updated {
            if let Err(e) = save_manifest_default(&io, &manifest) {
                let msg = format!("preflight: manifest save failed: {}", e);
                obs.on_preflight_error("preflight", &msg);
                return Err(msg);
            }
        }

        obs.on_preflight_complete("preflight", preflight_start.elapsed());
        return Ok(());
    }

    let updated = ensure_lint_upsert_manifest_state(&io, &mut manifest, &resource, |id| {
        run_lint_upsert(id, None)
    })?;

    if updated {
        save_manifest_default(&io, &manifest)
            .map_err(|e| format!("preflight: manifest save failed: {}", e))?;
    }

    Ok(())
}

fn ensure_lint_upsert_manifest_state<F>(
    io: &dyn ResourceIo,
    manifest: &mut ResourceManifest,
    resource: &LintResource,
    mut lint_runner: F,
) -> Result<bool, String>
where
    F: FnMut(&ResourceId) -> Result<(), ResourceError>,
{
    let state = resource.check_state(manifest, io);
    if state.is_fresh() {
        return Ok(false);
    }
    if state.is_error() {
        return Err(format!("preflight: lint state error: {}", state));
    }

    println!("preflight: lint-upsert ({})", state);
    lint_runner(resource.resource_id())
        .map_err(|e| format!("preflight: lint-upsert failed: {}", e))?;

    upsert_lint_manifest_entry(io, manifest, resource)?;
    Ok(true)
}

fn upsert_lint_manifest_entry(
    io: &dyn ResourceIo,
    manifest: &mut ResourceManifest,
    resource: &LintResource,
) -> Result<(), String> {
    let files = list_tracked_files(io)
        .map_err(|e| format!("preflight: failed to list tracked files: {}", e))?;
    let key = compute_lint_key(io, &files)
        .map_err(|e| format!("preflight: failed to compute lint key: {}", e))?;
    let file_list: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    manifest.insert(
        resource.resource_id().clone(),
        ManifestEntry::new(key, file_list.len()).with_input_files(file_list),
    );
    Ok(())
}

/// Skip preflight when the current binary IS a preflight tool (prevents recursion).
fn should_skip_preflight() -> bool {
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
        run_lint_upsert(self.resource_id(), None)?;

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

fn run_lint_upsert(
    resource_id: &ResourceId,
    mut observer: Option<&mut dyn gunbc_exec::PreflightObserver>,
) -> Result<(), ResourceError> {
    let steps: &[(&str, CargoCommand)] = &[
        (
            "codegen-dag",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed(
                "codegen-dag",
                "dag",
            ))),
        ),
        (
            "testgen",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed("testgen", "dag"))),
        ),
        (
            "pragma",
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed("pragma", "dag"))),
        ),
    ];

    let total = steps.len() + 2; // +1 for clippy, +1 for test gate
    for (i, (label, cmd)) in steps.iter().enumerate() {
        if let Some(ref mut obs) = observer {
            obs.on_preflight_step(i + 1, total, label);
        } else {
            eprint!("  [{}/{}] {}...", i + 1, total, label);
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
        let start = std::time::Instant::now();
        let result = run_cargo_command(resource_id, cmd);
        let elapsed = start.elapsed();
        if let Some(ref mut obs) = observer {
            obs.on_preflight_step_complete(label, elapsed);
        } else {
            eprintln!(" {:.1}s", elapsed.as_secs_f64());
        }
        result?;
    }

    // clippy check: cargo clippy -- -D warnings
    // Note: no --all-targets for speed; CI still catches test-only lint issues
    let clippy_step = total - 1;
    if let Some(ref mut obs) = observer {
        obs.on_preflight_step(clippy_step, total, "clippy");
    } else {
        eprint!("  [{}/{}] clippy...", clippy_step, total);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }
    let mut clippy_elapsed = Duration::ZERO;
    let clippy_outcome: Result<(), ResourceError> = (|| {
        let clippy_start = std::time::Instant::now();
        let clippy_check = CargoCommand::new(Subcommand::Clippy).warnings(Warnings::Deny);
        let clippy_result = run_cargo_command_response(resource_id, &clippy_check)?;

        if !clippy_result.success() {
            eprintln!(" fix needed ({:.1}s)", clippy_start.elapsed().as_secs_f64());
            eprint!("  [{}/{}] clippy --fix...", clippy_step, total);
            let _ = std::io::Write::flush(&mut std::io::stderr());
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
                        "cargo clippy failed after fix (exit {})\n{}\n{}",
                        verify_result.exit_code, verify_result.stdout, verify_result.stderr
                    ),
                ));
            }
            eprintln!(" {:.1}s", fix_start.elapsed().as_secs_f64());
            clippy_elapsed = clippy_start.elapsed();
        } else {
            eprintln!(" {:.1}s", clippy_start.elapsed().as_secs_f64());
            clippy_elapsed = clippy_start.elapsed();
        }
        Ok(())
    })();
    if let Some(ref mut obs) = observer {
        obs.on_preflight_step_complete("clippy", clippy_elapsed);
    }
    clippy_outcome?;

    // Test gate: compile all workspace lib test targets without executing
    // them. This catches contract/compile mismatches (including stale
    // generated tests) while avoiding long-running runtime execution in
    // local preflight.
    if let Some(ref mut obs) = observer {
        obs.on_preflight_step(total, total, "test --lib --no-run");
    } else {
        eprint!("  [{}/{}] test --lib --no-run...", total, total);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }
    let test_outcome: Result<(), ResourceError> = (|| {
        let test_start = std::time::Instant::now();
        let test_cmd = preflight_test_gate_command();
        run_cargo_command_with_env(resource_id, &test_cmd, &[("GUNBC_TEST_MAX_COST", "S")])?;
        let elapsed = test_start.elapsed();
        if let Some(ref mut obs) = observer {
            obs.on_preflight_step_complete("test --lib --no-run", elapsed);
        } else {
            eprintln!(" {:.1}s", elapsed.as_secs_f64());
        }
        Ok(())
    })();
    if test_outcome.is_err() {
        if let Some(ref mut obs) = observer {
            obs.on_preflight_step_complete("test --lib --no-run", Duration::ZERO);
        }
    }
    test_outcome?;

    Ok(())
}

fn preflight_test_gate_command() -> CargoCommand {
    CargoCommand::new(Subcommand::Test)
        .workspace()
        .lib_only()
        .no_run()
        .warnings(Warnings::Deny)
}

/// Run a cargo command, failing on non-zero exit.
fn run_cargo_command(resource_id: &ResourceId, cmd: &CargoCommand) -> Result<(), ResourceError> {
    run_cargo_command_with_env(resource_id, cmd, &[])
}

/// Run a cargo command with extra environment variables, failing on non-zero exit.
fn run_cargo_command_with_env(
    resource_id: &ResourceId,
    cmd: &CargoCommand,
    extra_env: &[(&str, &str)],
) -> Result<(), ResourceError> {
    let response = run_cargo_command_response_with_env(resource_id, cmd, extra_env)?;
    if response.success() {
        Ok(())
    } else {
        // Include both stdout and stderr — cargo test writes test failure
        // details (which tests failed, panic messages) to stdout, while
        // stderr only gets the terse "error: test failed" line.
        let mut detail = String::new();
        if !response.stdout.is_empty() {
            detail.push_str(&response.stdout);
        }
        if !response.stderr.is_empty() {
            if !detail.is_empty() {
                detail.push('\n');
            }
            detail.push_str(&response.stderr);
        }
        Err(ResourceError::CreateFailed(
            resource_id.clone(),
            format!(
                "command failed (exit {}): {}\n{}",
                response.exit_code,
                cmd.to_shell(),
                detail
            ),
        ))
    }
}

/// Run a cargo command and return the response.
fn run_cargo_command_response(
    resource_id: &ResourceId,
    cmd: &CargoCommand,
) -> Result<gunbc_ir::transport::ShellResponse, ResourceError> {
    run_cargo_command_response_with_env(resource_id, cmd, &[])
}

/// Run a cargo command with extra environment variables and return the response.
fn run_cargo_command_response_with_env(
    resource_id: &ResourceId,
    cmd: &CargoCommand,
    extra_env: &[(&str, &str)],
) -> Result<gunbc_ir::transport::ShellResponse, ResourceError> {
    // Preflight steps compile + run cargo binaries; in CI with cold caches
    // this can take well over 5 minutes. Use FermiCost::L (30 min).
    const PREFLIGHT_TIMEOUT_MS: u64 = 1_800_000; // FermiCost::L = 30 min
    const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

    let mut request = cmd.to_shell_request().timeout(PREFLIGHT_TIMEOUT_MS);
    for (k, v) in extra_env {
        request = request.env(*k, *v);
    }

    // Long-running commands (especially test/clippy) can look frozen in
    // non-interactive logs. Emit coarse elapsed heartbeats while the command runs.
    let still_running = Arc::new(AtomicBool::new(true));
    let heartbeat_running = Arc::clone(&still_running);
    let heartbeat_start = std::time::Instant::now();
    let heartbeat = thread::spawn(move || loop {
        thread::sleep(HEARTBEAT_INTERVAL);
        if !heartbeat_running.load(Ordering::Relaxed) {
            break;
        }
        eprint!(" {:.0}s", heartbeat_start.elapsed().as_secs_f64());
        let _ = std::io::Write::flush(&mut std::io::stderr());
    });

    let response = execute_request(&TransportRequest::Shell(request))
        .map_err(|e| ResourceError::CreateFailed(resource_id.clone(), e.to_string()));

    still_running.store(false, Ordering::Relaxed);
    let _ = heartbeat.join();
    let response = response?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::HashMap;
    use std::io;
    use std::path::{Path, PathBuf};

    #[derive(Default)]
    struct FakeIo {
        files: HashMap<PathBuf, Vec<u8>>,
        mtimes: HashMap<PathBuf, SystemTime>,
        command_outputs: HashMap<(String, Vec<String>), Vec<u8>>,
    }

    impl FakeIo {
        fn insert_file(&mut self, path: &str, contents: &[u8], mtime: SystemTime) {
            let path = PathBuf::from(path);
            self.files.insert(path.clone(), contents.to_vec());
            self.mtimes.insert(path, mtime);
        }

        fn insert_command_output(&mut self, command: &str, args: &[String], output: &[u8]) {
            self.command_outputs
                .insert((command.to_string(), args.to_vec()), output.to_vec());
        }
    }

    impl ResourceIo for FakeIo {
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, ResourceError> {
            self.files.get(path).cloned().ok_or_else(|| {
                ResourceError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("missing file: {}", path.display()),
                ))
            })
        }

        fn write_file(&self, path: &Path, _contents: &[u8]) -> Result<(), ResourceError> {
            Err(ResourceError::Io(io::Error::other(format!(
                "unexpected write_file call for {}",
                path.display()
            ))))
        }

        fn file_exists(&self, path: &Path) -> Result<bool, ResourceError> {
            Ok(self.files.contains_key(path))
        }

        fn glob_paths(&self, _pattern: &str) -> Result<Vec<PathBuf>, ResourceError> {
            Ok(Vec::new())
        }

        fn command_output(&self, command: &str, args: &[String]) -> Result<Vec<u8>, ResourceError> {
            self.command_outputs
                .get(&(command.to_string(), args.to_vec()))
                .cloned()
                .ok_or_else(|| {
                    ResourceError::Io(io::Error::other(format!(
                        "unexpected command: {} {:?}",
                        command, args
                    )))
                })
        }

        fn file_mtime(&self, path: &Path) -> Result<SystemTime, ResourceError> {
            self.mtimes.get(path).copied().ok_or_else(|| {
                ResourceError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("missing mtime: {}", path.display()),
                ))
            })
        }
    }

    fn string_args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    fn tracked_files_args(repo_root: &str) -> Vec<String> {
        vec![
            "-C".to_string(),
            repo_root.to_string(),
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
        ]
    }

    fn configured_fake_io() -> FakeIo {
        let mut io = FakeIo::default();
        io.insert_command_output(
            "git",
            &string_args(&["rev-parse", "--show-toplevel"]),
            b"/repo\n",
        );
        io.insert_command_output(
            "git",
            &tracked_files_args("/repo"),
            b"src/main.rs\0Cargo.toml\0",
        );
        io.insert_command_output("rustc", &string_args(&["--version"]), b"rustc 1.90.0\n");
        io.insert_command_output(
            "cargo",
            &string_args(&["clippy", "--version"]),
            b"clippy 1.90.0\n",
        );
        let old_mtime = UNIX_EPOCH + Duration::from_millis(5_000);
        io.insert_file("/repo/src/main.rs", b"fn main() {}\n", old_mtime);
        io.insert_file(
            "/repo/Cargo.toml",
            b"[package]\nname = \"demo\"\n",
            old_mtime,
        );
        io
    }

    #[test]
    fn ensure_manifest_state_missing_runs_and_records_manifest_entry() {
        let io = configured_fake_io();
        let resource = LintResource::new();
        let mut manifest = ResourceManifest::new();
        let calls = Cell::new(0usize);

        let updated = ensure_lint_upsert_manifest_state(&io, &mut manifest, &resource, |_| {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .expect("missing state should be upserted");

        assert!(updated);
        assert_eq!(calls.get(), 1, "lint upsert runner should run once");

        let entry = manifest
            .get(resource.resource_id())
            .expect("manifest entry should be written");
        assert_eq!(entry.input_file_count, 2);
        assert_eq!(
            entry
                .input_files
                .as_ref()
                .expect("input files should be stored"),
            &vec![
                "/repo/Cargo.toml".to_string(),
                "/repo/src/main.rs".to_string()
            ]
        );
    }

    #[test]
    fn ensure_manifest_state_fresh_does_not_run_or_mutate_manifest() {
        let io = configured_fake_io();
        let resource = LintResource::new();
        let files = list_tracked_files(&io).expect("tracked file list");
        let key = compute_lint_key(&io, &files).expect("compute key");
        let file_list: Vec<String> = files
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        let mut manifest = ResourceManifest::new();
        manifest.insert(
            resource.resource_id().clone(),
            ManifestEntry::new(key.clone(), file_list.len())
                .with_input_files(file_list)
                .with_timestamp(10_000),
        );
        let calls = Cell::new(0usize);

        let updated = ensure_lint_upsert_manifest_state(&io, &mut manifest, &resource, |_| {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .expect("fresh state should succeed");

        assert!(!updated);
        assert_eq!(calls.get(), 0, "lint upsert runner should not run");
        assert_eq!(
            manifest
                .get(resource.resource_id())
                .expect("entry remains")
                .key,
            key
        );
    }

    #[test]
    fn ensure_manifest_state_runner_failure_does_not_write_manifest_entry() {
        let io = configured_fake_io();
        let resource = LintResource::new();
        let resource_id = resource.resource_id().clone();
        let mut manifest = ResourceManifest::new();
        let calls = Cell::new(0usize);

        let err = ensure_lint_upsert_manifest_state(&io, &mut manifest, &resource, |_| {
            calls.set(calls.get() + 1);
            Err(ResourceError::CreateFailed(
                resource_id.clone(),
                "simulated failure".to_string(),
            ))
        })
        .expect_err("runner failure should bubble up");

        assert_eq!(
            calls.get(),
            1,
            "lint upsert runner should still be attempted"
        );
        assert!(
            err.contains("preflight: lint-upsert failed:"),
            "error should include preflight context, got: {}",
            err
        );
        assert!(
            manifest.get(resource.resource_id()).is_none(),
            "failed run should not write manifest entry"
        );
    }

    #[test]
    fn preflight_test_gate_uses_compile_only_workspace_libs() {
        let cmd = preflight_test_gate_command();
        assert_eq!(
            cmd.to_args(),
            vec!["cargo", "test", "--workspace", "--lib", "--no-run",]
        );
        assert_eq!(
            cmd.env(),
            vec![("RUSTFLAGS".to_string(), "-D warnings".to_string())]
        );
    }
}

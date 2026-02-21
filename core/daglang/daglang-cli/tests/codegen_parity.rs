// Test infrastructure: filesystem access for generated artifacts.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use daglang_driver::DriverContext;
use gunbc_dag::resolve_lowered_dag;
use gunbc_exec::{execute_with_mode_and_inputs, BoundaryMocks, ExecutionMode};
use gunbc_ir::ToolchainCommands;
use gunbc_ir::Value;
use gunbc_ir::WorkspaceLayout;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    static WORKSPACE_ROOT: OnceLock<PathBuf> = OnceLock::new();
    WORKSPACE_ROOT
        .get_or_init(|| {
            WorkspaceLayout::from_env_manifest_dir()
                .expect("resolve workspace layout")
                .workspace_root
        })
        .clone()
}

fn daglang_bin() -> &'static str {
    env!("CARGO_BIN_EXE_daglang")
}

fn unique_workspace_target_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    workspace_root().join("target").join(format!(
        "daglang_codegen_parity_{name}_{}_{}",
        std::process::id(),
        nanos
    ))
}

fn compile_module_for_target(relative_module: &str, target: &str, out_dir: &Path) {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(relative_module)
        .arg("--target")
        .arg(target)
        .arg("--out")
        .arg(out_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile for codegen parity");
    assert!(
        output.status.success(),
        "compile {relative_module} --target {target} should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_makegen_for_target(target: &str, out_dir: &Path) {
    compile_module_for_target("dsl/tools/makegen.dag", target, out_dir);
}

fn compile_makegen_layer1_rust(out_dir: &Path) {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg("dsl/tools/makegen.dag")
        .arg("--target")
        .arg("rust")
        .arg("--layer")
        .arg("1")
        .arg("--out")
        .arg(out_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --target rust --layer 1");
    assert!(
        output.status.success(),
        "compile --target rust --layer 1 should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_module_layer1_rust(relative_module: &str, out_dir: &Path) {
    let output = Command::new(daglang_bin())
        .arg("compile")
        .arg(relative_module)
        .arg("--target")
        .arg("rust")
        .arg("--layer")
        .arg("1")
        .arg("--out")
        .arg(out_dir)
        .current_dir(workspace_root())
        .output()
        .expect("failed to run daglang compile --target rust --layer 1");
    assert!(
        output.status.success(),
        "compile {relative_module} --target rust --layer 1 should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn read_target_manifest(out_dir: &Path, target: &str) -> String {
    let manifest_path = out_dir
        .join("target")
        .join("generated")
        .join(target)
        .join("progress_manifest.txt");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
        panic!(
            "failed to read progress manifest for target `{target}` at {}: {error}",
            manifest_path.display()
        )
    });
    normalize_manifest_text(&manifest)
}

fn normalize_manifest_text(manifest: &str) -> String {
    manifest
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "// progress-manifest"
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_makefile_text(content: &str) -> String {
    content.replace("\r\n", "\n").trim_end().to_string()
}

fn command_exists(name: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .is_ok_and(|status| status.success())
}

fn mips_runtime_available() -> bool {
    let toolchain = ToolchainCommands::mips_linux_gnu();
    let emulator = toolchain
        .emulator
        .clone()
        .unwrap_or_else(|| "qemu-mips".to_string());
    command_exists(&toolchain.assembler)
        && command_exists(&toolchain.linker)
        && command_exists(&emulator)
}

fn c_runtime_available() -> bool {
    command_exists("cc")
}

fn c_runtime_with_curl_headers_available() -> bool {
    if !c_runtime_available() {
        return false;
    }
    let probe_dir = unique_workspace_target_dir("c_runtime_curl_probe");
    if std::fs::create_dir_all(&probe_dir).is_err() {
        return false;
    }
    let source = probe_dir.join("probe.c");
    let bin = probe_dir.join("probe_bin");
    if std::fs::write(
        &source,
        "#include <curl/curl.h>\nint main(void) { return 0; }\n",
    )
    .is_err()
    {
        let _ = std::fs::remove_dir_all(&probe_dir);
        return false;
    }
    let available = Command::new("cc")
        .arg(&source)
        .arg("-o")
        .arg(&bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    let _ = std::fs::remove_dir_all(&probe_dir);
    available
}

fn generated_cli_bindings(main_rs: &str) -> Vec<(String, String)> {
    main_rs
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("input_mocks.set_input(\"")?;
            let (node_id, after_node) = rest.split_once("\", \"")?;
            let (port_name, _) = after_node.split_once("\", Value::Str(val.clone()));")?;
            Some((node_id.to_string(), port_name.to_string()))
        })
        .collect()
}

#[derive(Debug)]
enum RuntimeOutcome {
    Ran { stdout: String, stderr: String },
    Skipped { reason: String },
}

fn run_makegen_generated_rust_layer1(crate_out_dir: &Path) -> RuntimeOutcome {
    let generated_main = match std::fs::read_to_string(crate_out_dir.join("src/main.rs")) {
        Ok(content) => content,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("missing generated src/main.rs: {error}"),
            };
        }
    };
    let bindings = generated_cli_bindings(&generated_main);
    if bindings.is_empty() {
        return RuntimeOutcome::Skipped {
            reason: "generated rust layer1 crate exposed zero CLI bindings".to_string(),
        };
    }

    let output_dir = unique_workspace_target_dir("runtime_rust_makegen_out");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create runtime output dir: {error}"),
        };
    }
    let generated_path = output_dir.join("Makefile.generated");
    let generated_path_arg = generated_path.display().to_string();

    let mut run_cmd = Command::new("cargo");
    if let Err(error) = std::fs::copy(
        workspace_root().join("Cargo.lock"),
        crate_out_dir.join("Cargo.lock"),
    ) {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!("failed to stage Cargo.lock for generated crate: {error}"),
        };
    }
    run_cmd
        .arg("run")
        .arg("--quiet")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(crate_out_dir.join("Cargo.toml"))
        .arg("--")
        .current_dir(workspace_root());
    for _ in &bindings {
        run_cmd.arg(&generated_path_arg);
    }

    let run_output = match run_cmd.output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated rust layer1 crate: {error}"),
            };
        }
    };

    let stderr = String::from_utf8_lossy(&run_output.stderr).into_owned();
    if !run_output.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!("generated rust layer1 run failed: {stderr}"),
        };
    }

    let generated_content = match std::fs::read_to_string(&generated_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!(
                    "generated rust layer1 run did not write {}: {error}",
                    generated_path.display()
                ),
            };
        }
    };

    let _ = std::fs::remove_dir_all(&output_dir);
    RuntimeOutcome::Ran {
        stdout: generated_content,
        stderr,
    }
}

fn run_makegen_interpreter() -> RuntimeOutcome {
    let output_dir = unique_workspace_target_dir("runtime_makegen_interpreter_out");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create interpreter runtime output dir: {error}"),
        };
    }
    let generated_path = output_dir.join("Makefile.generated");
    let run_output = match Command::new(daglang_bin())
        .arg("run")
        .arg("--output")
        .arg(&generated_path)
        .arg("dsl/tools/makegen.dag")
        .current_dir(workspace_root())
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute daglang interpreter run: {error}"),
            };
        }
    };
    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr).into_owned();
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!("daglang interpreter run failed: {stderr}"),
        };
    }
    let generated_content = match std::fs::read_to_string(&generated_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!(
                    "daglang interpreter run did not write {}: {error}",
                    generated_path.display()
                ),
            };
        }
    };
    let stdout = String::from_utf8_lossy(&run_output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&run_output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&output_dir);
    RuntimeOutcome::Ran {
        stdout: generated_content,
        stderr: format!("{stdout}{stderr}"),
    }
}

fn module_entrypoint_id(relative_module: &str) -> String {
    let module_name = relative_module
        .trim_start_matches("dsl/")
        .trim_end_matches(".dag")
        .replace('/', ".");
    let function_name = module_name
        .rsplit('.')
        .next()
        .unwrap_or(module_name.as_str())
        .to_string();
    format!("{module_name}::{function_name}")
}

fn apply_module_entrypoint_inputs(
    mocks: &mut BoundaryMocks,
    relative_module: &str,
    inputs: &[(&str, &str)],
) {
    let entrypoint = module_entrypoint_id(relative_module);
    let entrypoint_dot = entrypoint.replace("::", ".");
    let module_stub = entrypoint.replace("::", "_").replace('.', "_");
    for (port, value) in inputs {
        mocks.set_input(&entrypoint, *port, Value::Str((*value).to_string()));
        mocks.set_input(&entrypoint_dot, *port, Value::Str((*value).to_string()));
        mocks.set_input(
            &format!("param_source_{module_stub}_{port}"),
            *port,
            Value::Str((*value).to_string()),
        );
    }
}

fn run_module_interpreter_execution_nodes(
    relative_module: &str,
    inputs: &[(&str, &str)],
) -> Result<Vec<String>, String> {
    let mut input_mocks = BoundaryMocks::new();
    apply_module_entrypoint_inputs(&mut input_mocks, relative_module, inputs);
    run_module_interpreter_execution_nodes_with_mocks(relative_module, input_mocks)
}

fn run_module_interpreter_execution_nodes_with_mocks(
    relative_module: &str,
    input_mocks: BoundaryMocks,
) -> Result<Vec<String>, String> {
    let context = DriverContext {
        roots: vec![workspace_root().join("dsl")],
        target_file: Some(workspace_root().join(relative_module)),
    };
    let output = daglang_driver::compile_from_context(&context)
        .map_err(|error| format!("compile failed for {relative_module}: {error}"))?;
    let resolved = resolve_lowered_dag(&output.lowered_dag)
        .map_err(|error| format!("resolve failed for {relative_module}: {error}"))?;
    let mut dry_run_boundary_mocks = BoundaryMocks::new();
    for node in &resolved.nodes {
        for output_port in &node.outputs {
            dry_run_boundary_mocks.set_value(&node.id.0, &output_port.name.0, Value::Skipped);
        }
    }
    let execution = execute_with_mode_and_inputs(
        &resolved,
        ExecutionMode::DryRun(dry_run_boundary_mocks),
        Some(&input_mocks),
    )
    .map_err(|error| format!("execute failed for {relative_module}: {error}"))?;
    let nodes = execution
        .entries
        .iter()
        .map(|entry| entry.node_id.clone())
        .collect::<Vec<_>>();
    Ok(normalize_execution_nodes(nodes))
}

fn parse_generated_execution_nodes(stdout: &str, stderr: &str) -> Vec<String> {
    let mut nodes = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[') {
            if let Some((node, _)) = rest.split_once("] intercepted=") {
                nodes.push(node.to_string());
            }
        }
    }
    nodes.sort();
    nodes.dedup();
    normalize_execution_nodes(nodes)
}

fn normalize_execution_nodes(mut nodes: Vec<String>) -> Vec<String> {
    nodes.retain(|node| node != "fs_env");
    nodes.sort();
    nodes.dedup();
    nodes
}

fn run_makegen_generated_go(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("go") {
        return RuntimeOutcome::Skipped {
            reason: "go toolchain not available on PATH".to_string(),
        };
    }

    let go_dir = native_out_dir.join("target/generated/go");
    let main_go = go_dir.join("main.go");
    if !main_go.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated go source: {}", main_go.display()),
        };
    }

    let output_dir = unique_workspace_target_dir("runtime_go_makegen_out");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create go runtime output dir: {error}"),
        };
    }
    let output_path = output_dir.join("Makefile.generated");

    let cache_root = native_out_dir.join(".go-cache");
    let run_output = match Command::new("go")
        .arg("run")
        .arg("main.go")
        .arg(&output_path)
        .current_dir(&go_dir)
        .env("GOCACHE", cache_root.join("build"))
        .env("GOMODCACHE", cache_root.join("mod"))
        .env("GOPATH", cache_root.join("path"))
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated go binary: {error}"),
            };
        }
    };

    if !run_output.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated go run failed: {}",
                String::from_utf8_lossy(&run_output.stderr)
            ),
        };
    }

    let generated_content = match std::fs::read_to_string(&output_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!(
                    "generated go run did not write {}: {error}",
                    output_path.display()
                ),
            };
        }
    };

    let _ = std::fs::remove_dir_all(&output_dir);
    RuntimeOutcome::Ran {
        stdout: generated_content,
        stderr: String::from_utf8_lossy(&run_output.stderr).into_owned(),
    }
}

fn run_makegen_generated_c(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("gcc") {
        return RuntimeOutcome::Skipped {
            reason: "gcc toolchain not available on PATH".to_string(),
        };
    }

    let c_dir = native_out_dir.join("target/generated/c");
    let main_c = c_dir.join("main.c");
    if !main_c.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated c source: {}", main_c.display()),
        };
    }

    let output_dir = unique_workspace_target_dir("runtime_c_makegen_out");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create c runtime output dir: {error}"),
        };
    }
    let output_path = output_dir.join("Makefile.generated");

    let app_path = c_dir.join("parity_app");
    let build = match Command::new("gcc")
        .arg("-std=c11")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-o")
        .arg(&app_path)
        .arg(&main_c)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke gcc for generated c source: {error}"),
            };
        }
    };
    if !build.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c compile failed: {}",
                String::from_utf8_lossy(&build.stderr)
            ),
        };
    }

    let run = match Command::new(&app_path).arg(&output_path).output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated c binary: {error}"),
            };
        }
    };
    if !run.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c binary exited nonzero: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }

    let generated_content = match std::fs::read_to_string(&output_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!(
                    "generated c run did not write {}: {error}",
                    output_path.display()
                ),
            };
        }
    };

    let _ = std::fs::remove_dir_all(&output_dir);
    RuntimeOutcome::Ran {
        stdout: generated_content,
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_makegen_generated_c_with_asan_ubsan(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("gcc") {
        return RuntimeOutcome::Skipped {
            reason: "gcc toolchain not available on PATH".to_string(),
        };
    }

    let c_dir = native_out_dir.join("target/generated/c");
    let main_c = c_dir.join("main.c");
    if !main_c.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated c source: {}", main_c.display()),
        };
    }

    let output_dir = unique_workspace_target_dir("runtime_c_makegen_asan_ubsan_out");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create c runtime output dir: {error}"),
        };
    }
    let output_path = output_dir.join("Makefile.generated");

    let app_path = c_dir.join("parity_asan_ubsan_app");
    let build = match Command::new("gcc")
        .arg("-std=c11")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-fsanitize=address,undefined")
        .arg("-fno-omit-frame-pointer")
        .arg("-g")
        .arg("-O1")
        .arg("-o")
        .arg(&app_path)
        .arg(&main_c)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke gcc for generated c source: {error}"),
            };
        }
    };
    if !build.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c asan+ubsan compile failed: {}",
                String::from_utf8_lossy(&build.stderr)
            ),
        };
    }

    let run = match Command::new(&app_path).arg(&output_path).output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated c binary: {error}"),
            };
        }
    };
    if !run.status.success() {
        let _ = std::fs::remove_dir_all(&output_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c asan+ubsan binary exited nonzero: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }

    let generated_content = match std::fs::read_to_string(&output_path) {
        Ok(content) => content,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&output_dir);
            return RuntimeOutcome::Skipped {
                reason: format!(
                    "generated c run did not write {}: {error}",
                    output_path.display()
                ),
            };
        }
    };

    let _ = std::fs::remove_dir_all(&output_dir);
    RuntimeOutcome::Ran {
        stdout: generated_content,
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_makegen_generated_mips(native_out_dir: &Path) -> RuntimeOutcome {
    let toolchain = ToolchainCommands::mips_linux_gnu();
    let emulator = toolchain
        .emulator
        .clone()
        .unwrap_or_else(|| "qemu-mips".to_string());
    if !(command_exists(&toolchain.assembler)
        && command_exists(&toolchain.linker)
        && command_exists(&emulator))
    {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "MIPS assembler/linker/runtime not available (need {}, {}, {})",
                toolchain.assembler, toolchain.linker, emulator
            ),
        };
    }

    let mips_dir = native_out_dir.join("target/generated/mips");
    let main_s = mips_dir.join("main.s");
    if !main_s.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated mips source: {}", main_s.display()),
        };
    }

    let obj_path = mips_dir.join("main.o");
    let bin_path = mips_dir.join("main.bin");

    let assemble = match Command::new(&toolchain.assembler)
        .arg("-o")
        .arg(&obj_path)
        .arg(&main_s)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke mips assembler: {error}"),
            };
        }
    };
    if !assemble.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "mips assembly failed: {}",
                String::from_utf8_lossy(&assemble.stderr)
            ),
        };
    }

    let link = match Command::new(&toolchain.linker)
        .arg("-e")
        .arg("main")
        .arg("-o")
        .arg(&bin_path)
        .arg(&obj_path)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke mips linker: {error}"),
            };
        }
    };
    if !link.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "mips link failed: {}",
                String::from_utf8_lossy(&link.stderr)
            ),
        };
    }

    let run = match Command::new(&emulator).arg(&bin_path).output() {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute mips binary under qemu: {error}"),
            };
        }
    };
    if !run.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "{emulator} execution failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }

    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_infra_generated_rust_layer1(crate_out_dir: &Path) -> RuntimeOutcome {
    if !crate_out_dir.join("Cargo.toml").is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "missing generated rust layer1 Cargo.toml at {}",
                crate_out_dir.join("Cargo.toml").display()
            ),
        };
    }
    if let Err(error) = std::fs::copy(
        workspace_root().join("Cargo.lock"),
        crate_out_dir.join("Cargo.lock"),
    ) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to stage Cargo.lock for generated crate: {error}"),
        };
    }
    let run_output = match Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(crate_out_dir.join("Cargo.toml"))
        .arg("--")
        .arg("dev")
        .arg("local")
        .arg("secret:github-token")
        .arg("")
        .arg("")
        .arg("false")
        .current_dir(workspace_root())
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated rust layer1 crate: {error}"),
            };
        }
    };
    if !run_output.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated rust layer1 run failed: {}",
                String::from_utf8_lossy(&run_output.stderr)
            ),
        };
    }
    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run_output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run_output.stderr).into_owned(),
    }
}

fn run_generated_rust_layer1_with_args(crate_out_dir: &Path, args: &[&str]) -> RuntimeOutcome {
    if !crate_out_dir.join("Cargo.toml").is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "missing generated rust layer1 Cargo.toml at {}",
                crate_out_dir.join("Cargo.toml").display()
            ),
        };
    }
    if let Err(error) = std::fs::copy(
        workspace_root().join("Cargo.lock"),
        crate_out_dir.join("Cargo.lock"),
    ) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to stage Cargo.lock for generated crate: {error}"),
        };
    }
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
        .arg("--quiet")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(crate_out_dir.join("Cargo.toml"))
        .arg("--")
        .current_dir(workspace_root());
    for arg in args {
        cmd.arg(arg);
    }
    let run_output = match cmd.output() {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated rust layer1 crate: {error}"),
            };
        }
    };
    if !run_output.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated rust layer1 run failed: {}",
                String::from_utf8_lossy(&run_output.stderr)
            ),
        };
    }
    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run_output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run_output.stderr).into_owned(),
    }
}

fn run_infra_generated_go(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("go") {
        return RuntimeOutcome::Skipped {
            reason: "go toolchain not available on PATH".to_string(),
        };
    }
    let go_dir = native_out_dir.join("target/generated/go");
    let main_go = go_dir.join("main.go");
    if !main_go.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated go source: {}", main_go.display()),
        };
    }
    match Command::new("go")
        .arg("run")
        .arg("main.go")
        .current_dir(&go_dir)
        .output()
    {
        Ok(output) if output.status.success() => RuntimeOutcome::Ran {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Ok(output) => RuntimeOutcome::Skipped {
            reason: format!(
                "generated go run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        },
        Err(error) => RuntimeOutcome::Skipped {
            reason: format!("failed to execute generated go binary: {error}"),
        },
    }
}

fn run_infra_generated_c(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("cc") {
        return RuntimeOutcome::Skipped {
            reason: "C compiler `cc` not available on PATH".to_string(),
        };
    }
    let c_dir = native_out_dir.join("target/generated/c");
    let main_c = c_dir.join("main.c");
    if !main_c.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated c source: {}", main_c.display()),
        };
    }
    let out_dir = unique_workspace_target_dir("runtime_infra_c_out");
    if let Err(error) = std::fs::create_dir_all(&out_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create C runtime output dir: {error}"),
        };
    }
    let bin_path = out_dir.join("infra_c_bin");
    let compile = match Command::new("cc")
        .arg("main.c")
        .arg("-o")
        .arg(&bin_path)
        .current_dir(&c_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to compile generated c binary: {error}"),
            };
        }
    };
    if !compile.status.success() {
        let _ = std::fs::remove_dir_all(&out_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c compile failed: {}",
                String::from_utf8_lossy(&compile.stderr)
            ),
        };
    }
    let run = match Command::new(&bin_path).output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated c binary: {error}"),
            };
        }
    };
    let _ = std::fs::remove_dir_all(&out_dir);
    if !run.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c binary failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }
    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_generated_c_with_asan(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("cc") {
        return RuntimeOutcome::Skipped {
            reason: "C compiler `cc` not available on PATH".to_string(),
        };
    }
    let c_dir = native_out_dir.join("target/generated/c");
    let main_c = c_dir.join("main.c");
    if !main_c.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated c source: {}", main_c.display()),
        };
    }
    let out_dir = unique_workspace_target_dir("runtime_c_asan_out");
    if let Err(error) = std::fs::create_dir_all(&out_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create C ASAN runtime output dir: {error}"),
        };
    }
    let bin_path = out_dir.join("asan_bin");
    let compile = match Command::new("cc")
        .arg("main.c")
        .arg("-fsanitize=address")
        .arg("-fno-omit-frame-pointer")
        .arg("-g")
        .arg("-O1")
        .arg("-o")
        .arg(&bin_path)
        .current_dir(&c_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke C compiler for ASAN build: {error}"),
            };
        }
    };
    if !compile.status.success() {
        let _ = std::fs::remove_dir_all(&out_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c ASAN compile failed: {}",
                String::from_utf8_lossy(&compile.stderr)
            ),
        };
    }
    let run = match Command::new(&bin_path).output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated c ASAN binary: {error}"),
            };
        }
    };
    let _ = std::fs::remove_dir_all(&out_dir);
    if !run.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c ASAN binary failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }
    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_generated_c_with_asan_ubsan(native_out_dir: &Path) -> RuntimeOutcome {
    if !command_exists("cc") {
        return RuntimeOutcome::Skipped {
            reason: "C compiler `cc` not available on PATH".to_string(),
        };
    }
    let c_dir = native_out_dir.join("target/generated/c");
    let main_c = c_dir.join("main.c");
    if !main_c.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated c source: {}", main_c.display()),
        };
    }
    let out_dir = unique_workspace_target_dir("runtime_c_asan_ubsan_out");
    if let Err(error) = std::fs::create_dir_all(&out_dir) {
        return RuntimeOutcome::Skipped {
            reason: format!("failed to create C ASAN+UBSAN runtime output dir: {error}"),
        };
    }
    let bin_path = out_dir.join("asan_ubsan_bin");
    let compile = match Command::new("cc")
        .arg("main.c")
        .arg("-fsanitize=address,undefined")
        .arg("-fno-omit-frame-pointer")
        .arg("-g")
        .arg("-O1")
        .arg("-o")
        .arg(&bin_path)
        .current_dir(&c_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke C compiler for ASAN+UBSAN build: {error}"),
            };
        }
    };
    if !compile.status.success() {
        let _ = std::fs::remove_dir_all(&out_dir);
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c ASAN+UBSAN compile failed: {}",
                String::from_utf8_lossy(&compile.stderr)
            ),
        };
    }
    let run = match Command::new(&bin_path).output() {
        Ok(output) => output,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&out_dir);
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute generated c ASAN+UBSAN binary: {error}"),
            };
        }
    };
    let _ = std::fs::remove_dir_all(&out_dir);
    if !run.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "generated c ASAN+UBSAN binary failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }
    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

fn run_infra_generated_mips(native_out_dir: &Path) -> RuntimeOutcome {
    let toolchain = ToolchainCommands::mips_linux_gnu();
    let emulator = toolchain
        .emulator
        .clone()
        .unwrap_or_else(|| "qemu-mips".to_string());
    if !(command_exists(&toolchain.assembler)
        && command_exists(&toolchain.linker)
        && command_exists(&emulator))
    {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "MIPS assembler/linker/runtime not available (need {}, {}, {})",
                toolchain.assembler, toolchain.linker, emulator
            ),
        };
    }

    let mips_dir = native_out_dir.join("target/generated/mips");
    let main_s = mips_dir.join("main.s");
    if !main_s.is_file() {
        return RuntimeOutcome::Skipped {
            reason: format!("missing generated mips source: {}", main_s.display()),
        };
    }

    let obj_path = mips_dir.join("main.o");
    let bin_path = mips_dir.join("main.bin");

    let assemble = match Command::new(&toolchain.assembler)
        .arg("-o")
        .arg(&obj_path)
        .arg(&main_s)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke mips assembler: {error}"),
            };
        }
    };
    if !assemble.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "mips assembly failed: {}",
                String::from_utf8_lossy(&assemble.stderr)
            ),
        };
    }

    let link = match Command::new(&toolchain.linker)
        .arg("-e")
        .arg("main")
        .arg("-o")
        .arg(&bin_path)
        .arg(&obj_path)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to invoke mips linker: {error}"),
            };
        }
    };
    if !link.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "mips link failed: {}",
                String::from_utf8_lossy(&link.stderr)
            ),
        };
    }

    let run = match Command::new(&emulator).arg(&bin_path).output() {
        Ok(output) => output,
        Err(error) => {
            return RuntimeOutcome::Skipped {
                reason: format!("failed to execute mips binary under qemu: {error}"),
            };
        }
    };
    if !run.status.success() {
        return RuntimeOutcome::Skipped {
            reason: format!(
                "{emulator} execution failed: {}",
                String::from_utf8_lossy(&run.stderr)
            ),
        };
    }

    RuntimeOutcome::Ran {
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    }
}

#[test]
fn makegen_manifest_parity_across_rust_go_c_mips_targets() {
    let out_root = unique_workspace_target_dir("manifest_parity");
    let targets = ["rust", "go", "c", "mips"];
    let mut manifests = BTreeMap::<String, String>::new();

    for target in targets {
        let target_out = out_root.join(target);
        compile_makegen_for_target(target, &target_out);
        manifests.insert(
            target.to_string(),
            read_target_manifest(&target_out, target),
        );
    }

    let rust_manifest = manifests
        .get("rust")
        .expect("manifest map should contain rust output")
        .clone();
    for target in ["go", "c", "mips"] {
        let target_manifest = manifests
            .get(target)
            .unwrap_or_else(|| panic!("manifest map should contain {target} output"));
        assert_eq!(
            &rust_manifest, target_manifest,
            "progress manifest parity mismatch: rust != {target}"
        );
    }

    std::fs::remove_dir_all(&out_root).expect("failed to cleanup manifest parity output root");
}

#[test]
fn sdlc_manifest_parity_across_rust_go_c_mips_targets() {
    let out_root = unique_workspace_target_dir("sdlc_manifest_parity");
    let targets = ["rust", "go", "c", "mips"];
    let mut manifests = BTreeMap::<String, String>::new();

    for target in targets {
        let target_out = out_root.join(target);
        compile_module_for_target("dsl/pipelines/sdlc.dag", target, &target_out);
        manifests.insert(
            target.to_string(),
            read_target_manifest(&target_out, target),
        );
    }

    let rust_manifest = manifests
        .get("rust")
        .expect("manifest map should contain rust output")
        .clone();
    for target in ["go", "c", "mips"] {
        let target_manifest = manifests
            .get(target)
            .unwrap_or_else(|| panic!("manifest map should contain {target} output"));
        assert_eq!(
            &rust_manifest, target_manifest,
            "sdlc progress manifest parity mismatch: rust != {target}"
        );
    }

    std::fs::remove_dir_all(&out_root).expect("failed to cleanup sdlc manifest parity output root");
}

#[test]
fn sdlc_control_plane_manifest_parity_across_rust_go_c_mips_targets() {
    let out_root = unique_workspace_target_dir("sdlc_control_plane_manifest_parity");
    let targets = ["rust", "go", "c", "mips"];
    let mut manifests = BTreeMap::<String, String>::new();

    for target in targets {
        let target_out = out_root.join(target);
        compile_module_for_target("dsl/services/sdlc/control_plane.dag", target, &target_out);
        manifests.insert(
            target.to_string(),
            read_target_manifest(&target_out, target),
        );
    }

    let rust_manifest = manifests
        .get("rust")
        .expect("manifest map should contain rust output")
        .clone();
    for target in ["go", "c", "mips"] {
        let target_manifest = manifests
            .get(target)
            .unwrap_or_else(|| panic!("manifest map should contain {target} output"));
        assert_eq!(
            &rust_manifest, target_manifest,
            "sdlc control-plane progress manifest parity mismatch: rust != {target}"
        );
    }

    std::fs::remove_dir_all(&out_root)
        .expect("failed to cleanup sdlc control-plane manifest parity output root");
}

#[test]
fn infra_tool_manifest_parity_across_rust_go_c_mips_targets() {
    let out_root = unique_workspace_target_dir("infra_tool_manifest_parity");
    let targets = ["rust", "go", "c", "mips"];
    let mut manifests = BTreeMap::<String, String>::new();

    for target in targets {
        let target_out = out_root.join(target);
        compile_module_for_target("dsl/tools/infra.dag", target, &target_out);
        manifests.insert(
            target.to_string(),
            read_target_manifest(&target_out, target),
        );
    }

    let rust_manifest = manifests
        .get("rust")
        .expect("manifest map should contain rust output")
        .clone();
    for target in ["go", "c", "mips"] {
        let target_manifest = manifests
            .get(target)
            .unwrap_or_else(|| panic!("manifest map should contain {target} output"));
        assert_eq!(
            &rust_manifest, target_manifest,
            "infra tool progress manifest parity mismatch: rust != {target}"
        );
    }

    std::fs::remove_dir_all(&out_root).expect("failed to cleanup infra tool manifest parity output root");
}

#[test]
fn design_tool_manifest_parity_across_rust_go_c_mips_targets() {
    let out_root = unique_workspace_target_dir("design_tool_manifest_parity");
    let targets = ["rust", "go", "c", "mips"];
    let mut manifests = BTreeMap::<String, String>::new();

    for target in targets {
        let target_out = out_root.join(target);
        compile_module_for_target("dsl/tools/design.dag", target, &target_out);
        manifests.insert(
            target.to_string(),
            read_target_manifest(&target_out, target),
        );
    }

    let rust_manifest = manifests
        .get("rust")
        .expect("manifest map should contain rust output")
        .clone();
    for target in ["go", "c", "mips"] {
        let target_manifest = manifests
            .get(target)
            .unwrap_or_else(|| panic!("manifest map should contain {target} output"));
        assert_eq!(
            &rust_manifest, target_manifest,
            "design tool progress manifest parity mismatch: rust != {target}"
        );
    }

    std::fs::remove_dir_all(&out_root).expect("failed to cleanup design tool manifest parity output root");
}

#[test]
fn makegen_runtime_smoke_per_target_with_toolchain_awareness() {
    let native_out_root = unique_workspace_target_dir("runtime_native");
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1");

    compile_makegen_for_target("go", &native_out_root.join("go"));
    compile_makegen_for_target("c", &native_out_root.join("c"));
    compile_makegen_for_target("mips", &native_out_root.join("mips"));
    compile_makegen_layer1_rust(&rust_layer1_out);

    let rust = run_makegen_generated_rust_layer1(&rust_layer1_out);
    let go = run_makegen_generated_go(&native_out_root.join("go"));
    let c = run_makegen_generated_c(&native_out_root.join("c"));
    let mips = run_makegen_generated_mips(&native_out_root.join("mips"));

    let rust_makefile = match rust {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains(".PHONY:"),
                "rust layer1 runtime should emit makegen makefile content"
            );
            assert!(
                stderr.contains("execution completed"),
                "rust layer1 runtime should log execution completion"
            );
            normalize_makefile_text(&stdout)
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("rust runtime parity smoke should not skip: {reason}");
        }
    };

    for (target, outcome) in [("go", go), ("c", c), ("mips", mips)] {
        match outcome {
            RuntimeOutcome::Ran { stdout, .. } => {
                let target_makefile = normalize_makefile_text(&stdout);
                assert_eq!(
                    rust_makefile, target_makefile,
                    "makegen runtime parity mismatch: rust != {target}"
                );
            }
            RuntimeOutcome::Skipped { reason } => {
                eprintln!("SKIP {target} runtime parity: {reason}");
            }
        }
    }

    std::fs::remove_dir_all(&native_out_root).expect("failed to cleanup native runtime out root");
    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 runtime out root");
}

#[test]
fn makegen_runtime_differential_interpreter_vs_generated_rust_layer1() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_makegen_diff");
    compile_makegen_layer1_rust(&rust_layer1_out);

    let generated = run_makegen_generated_rust_layer1(&rust_layer1_out);
    let interpreter = run_makegen_interpreter();

    let generated_makefile = match generated {
        RuntimeOutcome::Ran { stdout, .. } => normalize_makefile_text(&stdout),
        RuntimeOutcome::Skipped { reason } => {
            panic!("generated rust layer1 runtime should not skip in differential test: {reason}");
        }
    };

    let interpreter_makefile = match interpreter {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stderr.contains("OK: run mode=real"),
                "interpreter run should report successful execution: {stderr}"
            );
            normalize_makefile_text(&stdout)
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("daglang interpreter differential run should not skip: {reason}");
        }
    };

    assert_eq!(
        interpreter_makefile, generated_makefile,
        "interpreter and generated rust layer1 outputs must match exactly for makegen"
    );

    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 makegen differential out root");
}

#[test]
fn makegen_runtime_differential_interpreter_vs_generated_native_backends() {
    let native_out_root = unique_workspace_target_dir("runtime_native_makegen_diff");
    compile_makegen_for_target("go", &native_out_root.join("go"));
    compile_makegen_for_target("c", &native_out_root.join("c"));
    compile_makegen_for_target("mips", &native_out_root.join("mips"));

    let interpreter = run_makegen_interpreter();
    let interpreter_makefile = match interpreter {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stderr.contains("OK: run mode=real"),
                "interpreter run should report successful execution: {stderr}"
            );
            normalize_makefile_text(&stdout)
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("daglang interpreter differential run should not skip: {reason}");
        }
    };

    let go = run_makegen_generated_go(&native_out_root.join("go"));
    let c = run_makegen_generated_c(&native_out_root.join("c"));
    let mips = run_makegen_generated_mips(&native_out_root.join("mips"));

    for (target, outcome) in [("go", go), ("c", c), ("mips", mips)] {
        match outcome {
            RuntimeOutcome::Ran { stdout, .. } => {
                let target_makefile = normalize_makefile_text(&stdout);
                assert_eq!(
                    interpreter_makefile, target_makefile,
                    "interpreter runtime differential mismatch: interpreter != {target}"
                );
            }
            RuntimeOutcome::Skipped { reason } => {
                eprintln!("SKIP interpreter vs {target} differential: {reason}");
            }
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native makegen differential out root");
}

#[test]
fn makegen_c_runtime_asan_ubsan_differential_matches_interpreter() {
    let native_out_root = unique_workspace_target_dir("runtime_native_makegen_c_asan_ubsan_diff");
    compile_makegen_for_target("c", &native_out_root.join("c"));

    let interpreter = run_makegen_interpreter();
    let interpreter_makefile = match interpreter {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stderr.contains("OK: run mode=real"),
                "interpreter run should report successful execution: {stderr}"
            );
            normalize_makefile_text(&stdout)
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("daglang interpreter differential run should not skip: {reason}");
        }
    };

    let c = run_makegen_generated_c_with_asan_ubsan(&native_out_root.join("c"));
    match c {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                !stderr.contains("AddressSanitizer") && !stderr.contains("runtime error:"),
                "makegen c asan+ubsan differential should not report sanitizer violations: {stderr}"
            );
            let c_makefile = normalize_makefile_text(&stdout);
            assert_eq!(
                interpreter_makefile, c_makefile,
                "interpreter runtime differential mismatch: interpreter != c(asan+ubsan)"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP interpreter vs c(asan+ubsan) differential: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native makegen c asan+ubsan differential out root");
}

#[test]
fn design_tool_rust_layer1_execution_trace_matches_interpreter() {
    let module = "dsl/tools/design.dag";
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_trace_diff_design");
    compile_module_layer1_rust(module, &rust_layer1_out);
    let generated = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    let (generated_stdout, generated_stderr) = match generated {
        RuntimeOutcome::Ran { stdout, stderr } => (stdout, stderr),
        RuntimeOutcome::Skipped { reason } => {
            panic!("generated rust layer1 runtime should not skip for {module}: {reason}");
        }
    };
    let generated_nodes = parse_generated_execution_nodes(&generated_stdout, &generated_stderr);
    assert!(
        !generated_nodes.is_empty(),
        "generated rust layer1 runtime should emit execution trace for {module}"
    );

    let interpreter_nodes = run_module_interpreter_execution_nodes(module, &[])
        .unwrap_or_else(|error| panic!("interpreter run should succeed for {module}: {error}"));
    assert_eq!(
        interpreter_nodes, generated_nodes,
        "execution trace differential mismatch for {module}"
    );
    std::fs::remove_dir_all(&rust_layer1_out).unwrap_or_else(|error| {
        panic!(
            "failed to cleanup rust layer1 trace differential out root for {module}: {error}"
        )
    });
}

#[test]
fn infra_tool_rust_layer1_execution_trace_matches_interpreter() {
    let module = "dsl/tools/infra.dag";
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_trace_diff_infra");
    compile_module_layer1_rust(module, &rust_layer1_out);
    let generated = run_infra_generated_rust_layer1(&rust_layer1_out);
    let (generated_stdout, generated_stderr) = match generated {
        RuntimeOutcome::Ran { stdout, stderr } => (stdout, stderr),
        RuntimeOutcome::Skipped { reason } => {
            panic!("generated rust layer1 runtime should not skip for {module}: {reason}");
        }
    };
    let generated_nodes = parse_generated_execution_nodes(&generated_stdout, &generated_stderr);
    assert!(
        !generated_nodes.is_empty(),
        "generated rust layer1 runtime should emit execution trace for {module}"
    );

    let mut mocks = BoundaryMocks::new();
    mocks.set_input("tools.infra.infra", "environment", Value::Str("dev".to_string()));
    mocks.set_input("tools.infra::infra", "environment", Value::Str("dev".to_string()));
    mocks.set_input("tools.infra.infra", "runtime", Value::Str("local".to_string()));
    mocks.set_input("tools.infra::infra", "runtime", Value::Str("local".to_string()));
    mocks.set_input(
        "tools.infra.infra",
        "spec_targets",
        Value::List(vec![Value::Str("secret:github-token".to_string())]),
    );
    mocks.set_input(
        "tools.infra::infra",
        "spec_targets",
        Value::List(vec![Value::Str("secret:github-token".to_string())]),
    );
    mocks.set_input("tools.infra.infra", "request_token", Value::Str(String::new()));
    mocks.set_input("tools.infra::infra", "request_token", Value::Str(String::new()));
    mocks.set_input("tools.infra.infra", "request_url", Value::Str(String::new()));
    mocks.set_input("tools.infra::infra", "request_url", Value::Str(String::new()));
    mocks.set_input("tools.infra.infra", "allow_impersonation", Value::Bool(false));
    mocks.set_input("tools.infra::infra", "allow_impersonation", Value::Bool(false));
    mocks.set_input("tools.infra.infra", "execute", Value::Bool(false));
    mocks.set_input("tools.infra::infra", "execute", Value::Bool(false));
    let interpreter_nodes = run_module_interpreter_execution_nodes_with_mocks(module, mocks)
    .unwrap_or_else(|error| panic!("interpreter run should succeed for {module}: {error}"));
    assert_eq!(
        interpreter_nodes, generated_nodes,
        "execution trace differential mismatch for {module}"
    );
    std::fs::remove_dir_all(&rust_layer1_out).unwrap_or_else(|error| {
        panic!(
            "failed to cleanup rust layer1 trace differential out root for {module}: {error}"
        )
    });
}

#[test]
fn sdlc_control_plane_rust_layer1_execution_trace_matches_interpreter() {
    let module = "dsl/services/sdlc/control_plane.dag";
    let rust_layer1_out =
        unique_workspace_target_dir("runtime_rust_layer1_trace_diff_sdlc_control_plane");
    compile_module_layer1_rust(module, &rust_layer1_out);
    let generated = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    let (generated_stdout, generated_stderr) = match generated {
        RuntimeOutcome::Ran { stdout, stderr } => (stdout, stderr),
        RuntimeOutcome::Skipped { reason } => {
            panic!("generated rust layer1 runtime should not skip for {module}: {reason}");
        }
    };
    let generated_nodes = parse_generated_execution_nodes(&generated_stdout, &generated_stderr);
    assert!(
        !generated_nodes.is_empty(),
        "generated rust layer1 runtime should emit execution trace for {module}"
    );

    let interpreter_nodes = run_module_interpreter_execution_nodes(module, &[])
        .unwrap_or_else(|error| panic!("interpreter run should succeed for {module}: {error}"));
    assert_eq!(
        interpreter_nodes, generated_nodes,
        "execution trace differential mismatch for {module}"
    );
    std::fs::remove_dir_all(&rust_layer1_out).unwrap_or_else(|error| {
        panic!(
            "failed to cleanup rust layer1 trace differential out root for {module}: {error}"
        )
    });
}

#[test]
fn sdlc_pipeline_rust_layer1_execution_trace_matches_interpreter() {
    let module = "dsl/pipelines/sdlc.dag";
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_trace_diff_sdlc_pipeline");
    compile_module_layer1_rust(module, &rust_layer1_out);
    let generated = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    let (generated_stdout, generated_stderr) = match generated {
        RuntimeOutcome::Ran { stdout, stderr } => (stdout, stderr),
        RuntimeOutcome::Skipped { reason } => {
            panic!("generated rust layer1 runtime should not skip for {module}: {reason}");
        }
    };
    let generated_nodes = parse_generated_execution_nodes(&generated_stdout, &generated_stderr);
    assert!(
        !generated_nodes.is_empty(),
        "generated rust layer1 runtime should emit execution trace for {module}"
    );

    let interpreter_nodes = run_module_interpreter_execution_nodes(module, &[])
        .unwrap_or_else(|error| panic!("interpreter run should succeed for {module}: {error}"));
    assert_eq!(
        interpreter_nodes, generated_nodes,
        "execution trace differential mismatch for {module}"
    );
    std::fs::remove_dir_all(&rust_layer1_out).unwrap_or_else(|error| {
        panic!(
            "failed to cleanup rust layer1 trace differential out root for {module}: {error}"
        )
    });
}

#[test]
fn infra_runtime_smoke_rust_layer1_executes_entrypoint() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_infra");
    compile_module_layer1_rust("dsl/tools/infra.dag", &rust_layer1_out);

    let rust = run_infra_generated_rust_layer1(&rust_layer1_out);
    match rust {
        RuntimeOutcome::Ran { stderr, .. } => {
            assert!(
                stderr.contains("execution completed: 1 nodes executed"),
                "rust infra runtime should execute one node: {stderr}"
            );
            assert!(
                stderr.contains("[tools.infra::infra]"),
                "rust infra runtime should execute tools.infra::infra: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("rust infra runtime smoke should not skip: {reason}");
        }
    }

    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 infra runtime out root");
}

#[test]
fn sdlc_pipeline_layer1_rust_compiles_for_exec_runtime() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_sdlc_pipeline");
    compile_module_layer1_rust("dsl/pipelines/sdlc.dag", &rust_layer1_out);
    assert!(
        rust_layer1_out.join("Cargo.toml").is_file(),
        "rust layer1 compile should emit Cargo.toml for sdlc pipeline"
    );
    assert!(
        rust_layer1_out.join("src/main.rs").is_file(),
        "rust layer1 compile should emit src/main.rs for sdlc pipeline"
    );
    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 sdlc pipeline out root");
}

#[test]
fn sdlc_control_plane_layer1_rust_compiles_for_exec_runtime() {
    let rust_layer1_out =
        unique_workspace_target_dir("runtime_rust_layer1_sdlc_control_plane");
    compile_module_layer1_rust("dsl/services/sdlc/control_plane.dag", &rust_layer1_out);
    assert!(
        rust_layer1_out.join("Cargo.toml").is_file(),
        "rust layer1 compile should emit Cargo.toml for sdlc control-plane service"
    );
    assert!(
        rust_layer1_out.join("src/main.rs").is_file(),
        "rust layer1 compile should emit src/main.rs for sdlc control-plane service"
    );
    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 sdlc control-plane out root");
}

#[test]
fn sdlc_pipeline_runtime_smoke_rust_layer1_executes_entrypoint() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_sdlc_pipeline");
    compile_module_layer1_rust("dsl/pipelines/sdlc.dag", &rust_layer1_out);

    let rust = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    match rust {
        RuntimeOutcome::Ran { stderr, .. } => {
            assert!(
                stderr.contains("execution completed: 57 nodes executed"),
                "rust sdlc pipeline runtime should execute expected node count: {stderr}"
            );
            assert!(
                stderr.contains("[pipelines.sdlc::sdlc]"),
                "rust sdlc pipeline runtime should execute pipeline entrypoint: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("rust sdlc pipeline runtime smoke should not skip: {reason}");
        }
    }

    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 sdlc pipeline runtime out root");
}

#[test]
fn sdlc_control_plane_runtime_smoke_rust_layer1_executes_entrypoint() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_sdlc_control_plane");
    compile_module_layer1_rust("dsl/services/sdlc/control_plane.dag", &rust_layer1_out);

    let rust = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    match rust {
        RuntimeOutcome::Ran { stderr, .. } => {
            assert!(
                stderr.contains("execution completed: 15 nodes executed"),
                "rust sdlc control-plane runtime should execute expected node count: {stderr}"
            );
            assert!(
                stderr.contains("prepare_transport_services_sdlc_control_plane"),
                "rust sdlc control-plane runtime should execute service transport path: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("rust sdlc control-plane runtime smoke should not skip: {reason}");
        }
    }

    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 sdlc control-plane runtime out root");
}

#[test]
fn design_tool_runtime_smoke_rust_layer1_executes_entrypoint() {
    let rust_layer1_out = unique_workspace_target_dir("runtime_rust_layer1_design_tool");
    compile_module_layer1_rust("dsl/tools/design.dag", &rust_layer1_out);

    let rust = run_generated_rust_layer1_with_args(&rust_layer1_out, &[]);
    match rust {
        RuntimeOutcome::Ran { stderr, .. } => {
            assert!(
                stderr.contains("execution completed"),
                "rust design tool runtime should execute successfully: {stderr}"
            );
            assert!(
                stderr.contains("tools.design::generate_design"),
                "rust design tool runtime should execute design callable path: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("rust design runtime smoke should not skip: {reason}");
        }
    }

    std::fs::remove_dir_all(&rust_layer1_out)
        .expect("failed to cleanup rust layer1 design tool runtime out root");
}

#[test]
fn infra_runtime_smoke_go_and_c_emit_runnable_binaries() {
    let native_out_root = unique_workspace_target_dir("runtime_native_infra");
    compile_module_for_target("dsl/tools/infra.dag", "go", &native_out_root.join("go"));
    compile_module_for_target("dsl/tools/infra.dag", "c", &native_out_root.join("c"));

    let go = run_infra_generated_go(&native_out_root.join("go"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go infra runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP infra go runtime smoke: {reason}");
        }
    }

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c infra runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP infra c runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native infra runtime out root");
}

#[test]
fn infra_runtime_smoke_mips_emits_runnable_binary_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_infra_mips");
    compile_module_for_target("dsl/tools/infra.dag", "mips", &native_out_root.join("mips"));

    let mips = run_infra_generated_mips(&native_out_root.join("mips"));
    match mips {
        RuntimeOutcome::Ran { .. } => {
            // Generated MIPS scaffold exits successfully; stdout is backend-specific.
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP infra mips runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native infra mips runtime out root");
}

#[test]
fn sdlc_pipeline_runtime_smoke_go_and_c_emit_runnable_binaries() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline");
    compile_module_for_target("dsl/pipelines/sdlc.dag", "go", &native_out_root.join("go"));
    compile_module_for_target("dsl/pipelines/sdlc.dag", "c", &native_out_root.join("c"));

    let go = run_infra_generated_go(&native_out_root.join("go"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go sdlc pipeline runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc pipeline go runtime smoke: {reason}");
        }
    }

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c sdlc pipeline runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc pipeline c runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc pipeline runtime out root");
}

#[test]
fn sdlc_pipeline_go_runtime_executes_when_go_available() {
    if !command_exists("go") {
        eprintln!("SKIP sdlc pipeline go runtime strict check: go toolchain not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_go_strict");
    compile_module_for_target("dsl/pipelines/sdlc.dag", "go", &native_out_root.join("go"));
    let go = run_infra_generated_go(&native_out_root.join("go"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go sdlc pipeline runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("sdlc pipeline go runtime should not skip when go is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc pipeline go runtime out root");
}

#[test]
fn infra_go_runtime_executes_when_go_available() {
    if !command_exists("go") {
        eprintln!("SKIP infra go runtime strict check: go toolchain not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_infra_go_strict");
    compile_module_for_target("dsl/tools/infra.dag", "go", &native_out_root.join("go"));
    let go = run_infra_generated_go(&native_out_root.join("go"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go infra runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("infra go runtime should not skip when go is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict infra go runtime out root");
}

#[test]
fn sdlc_control_plane_go_runtime_executes_when_go_available() {
    if !command_exists("go") {
        eprintln!("SKIP sdlc control-plane go runtime strict check: go toolchain not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_control_plane_go_strict");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "go",
        &native_out_root.join("go"),
    );
    let go = run_infra_generated_go(&native_out_root.join("go"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go sdlc control-plane runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("sdlc control-plane go runtime should not skip when go is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc control-plane go runtime out root");
}

#[test]
fn design_tool_go_runtime_executes_when_go_available() {
    if !command_exists("go") {
        eprintln!("SKIP design tool go runtime strict check: go toolchain not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_design_go_strict");
    compile_module_for_target("dsl/tools/design.dag", "go", &native_out_root.join("go"));
    let go = run_infra_generated_go(&native_out_root.join("go"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go design runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("design tool go runtime should not skip when go is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict design go runtime out root");
}

#[test]
fn design_tool_c_runtime_executes_when_cc_available() {
    if !c_runtime_available() {
        eprintln!("SKIP design tool c runtime strict check: C compiler not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_design_c_strict");
    compile_module_for_target("dsl/tools/design.dag", "c", &native_out_root.join("c"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c design runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("design tool c runtime should not skip when C compiler is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict design c runtime out root");
}

#[test]
fn infra_c_runtime_executes_when_cc_and_curl_headers_available() {
    if !c_runtime_with_curl_headers_available() {
        eprintln!("SKIP infra c runtime strict check: C compiler/curl headers not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_infra_c_strict");
    compile_module_for_target("dsl/tools/infra.dag", "c", &native_out_root.join("c"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c infra runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("infra c runtime should not skip when C compiler/curl headers are available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict infra c runtime out root");
}

#[test]
fn sdlc_pipeline_c_runtime_executes_when_cc_and_curl_headers_available() {
    if !c_runtime_with_curl_headers_available() {
        eprintln!(
            "SKIP sdlc pipeline c runtime strict check: C compiler/curl headers not available"
        );
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_c_strict");
    compile_module_for_target("dsl/pipelines/sdlc.dag", "c", &native_out_root.join("c"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c sdlc pipeline runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!("sdlc pipeline c runtime should not skip when C compiler/curl headers are available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc pipeline c runtime out root");
}

#[test]
fn sdlc_control_plane_c_runtime_executes_when_cc_and_curl_headers_available() {
    if !c_runtime_with_curl_headers_available() {
        eprintln!(
            "SKIP sdlc control-plane c runtime strict check: C compiler/curl headers not available"
        );
        return;
    }

    let native_out_root =
        unique_workspace_target_dir("runtime_native_sdlc_control_plane_c_strict");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "c",
        &native_out_root.join("c"),
    );
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c sdlc control-plane runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            panic!(
                "sdlc control-plane c runtime should not skip when C compiler/curl headers are available: {reason}"
            );
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc control-plane c runtime out root");
}

#[test]
fn infra_mips_runtime_executes_when_mips_toolchain_available() {
    if !mips_runtime_available() {
        eprintln!("SKIP infra mips runtime strict check: mips toolchain/runtime not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_infra_mips_strict");
    compile_module_for_target("dsl/tools/infra.dag", "mips", &native_out_root.join("mips"));
    let mips = run_infra_generated_mips(&native_out_root.join("mips"));

    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            panic!("infra mips runtime should not skip when toolchain is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict infra mips runtime out root");
}

#[test]
fn sdlc_pipeline_mips_runtime_executes_when_mips_toolchain_available() {
    if !mips_runtime_available() {
        eprintln!("SKIP sdlc pipeline mips runtime strict check: mips toolchain/runtime not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_mips_strict");
    compile_module_for_target(
        "dsl/pipelines/sdlc.dag",
        "mips",
        &native_out_root.join("mips"),
    );
    let mips = run_infra_generated_mips(&native_out_root.join("mips"));

    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            panic!("sdlc pipeline mips runtime should not skip when toolchain is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc pipeline mips runtime out root");
}

#[test]
fn sdlc_control_plane_mips_runtime_executes_when_mips_toolchain_available() {
    if !mips_runtime_available() {
        eprintln!("SKIP sdlc control-plane mips runtime strict check: mips toolchain/runtime not available");
        return;
    }

    let native_out_root =
        unique_workspace_target_dir("runtime_native_sdlc_control_plane_mips_strict");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "mips",
        &native_out_root.join("mips"),
    );
    let mips = run_infra_generated_mips(&native_out_root.join("mips"));

    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            panic!(
                "sdlc control-plane mips runtime should not skip when toolchain is available: {reason}"
            );
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict sdlc control-plane mips runtime out root");
}

#[test]
fn design_tool_mips_runtime_executes_when_mips_toolchain_available() {
    if !mips_runtime_available() {
        eprintln!("SKIP design tool mips runtime strict check: mips toolchain/runtime not available");
        return;
    }

    let native_out_root = unique_workspace_target_dir("runtime_native_design_mips_strict");
    compile_module_for_target("dsl/tools/design.dag", "mips", &native_out_root.join("mips"));
    let mips = run_infra_generated_mips(&native_out_root.join("mips"));

    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            panic!("design tool mips runtime should not skip when toolchain is available: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup strict design mips runtime out root");
}

#[test]
fn sdlc_pipeline_runtime_smoke_mips_emits_runnable_binary_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_mips");
    compile_module_for_target(
        "dsl/pipelines/sdlc.dag",
        "mips",
        &native_out_root.join("mips"),
    );

    let mips = run_infra_generated_mips(&native_out_root.join("mips"));
    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc pipeline mips runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc pipeline mips runtime out root");
}

#[test]
fn sdlc_control_plane_runtime_smoke_go_and_c_emit_runnable_binaries() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_control_plane");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "go",
        &native_out_root.join("go"),
    );
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "c",
        &native_out_root.join("c"),
    );

    let go = run_infra_generated_go(&native_out_root.join("go"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go sdlc control-plane runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc control-plane go runtime smoke: {reason}");
        }
    }

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c sdlc control-plane runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc control-plane c runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc control-plane runtime out root");
}

#[test]
fn sdlc_control_plane_runtime_smoke_mips_emits_runnable_binary_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_control_plane_mips");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "mips",
        &native_out_root.join("mips"),
    );

    let mips = run_infra_generated_mips(&native_out_root.join("mips"));
    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc control-plane mips runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc control-plane mips runtime out root");
}

#[test]
fn design_tool_runtime_smoke_go_and_c_emit_runnable_binaries() {
    let native_out_root = unique_workspace_target_dir("runtime_native_design_tool");
    compile_module_for_target("dsl/tools/design.dag", "go", &native_out_root.join("go"));
    compile_module_for_target("dsl/tools/design.dag", "c", &native_out_root.join("c"));

    let go = run_infra_generated_go(&native_out_root.join("go"));
    let c = run_infra_generated_c(&native_out_root.join("c"));

    match go {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated go backend"),
                "generated go design runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP design tool go runtime smoke: {reason}");
        }
    }

    match c {
        RuntimeOutcome::Ran { stdout, .. } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated c design runtime should print backend banner: {stdout}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP design tool c runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native design runtime out root");
}

#[test]
fn design_tool_runtime_smoke_mips_emits_runnable_binary_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_design_tool_mips");
    compile_module_for_target("dsl/tools/design.dag", "mips", &native_out_root.join("mips"));

    let mips = run_infra_generated_mips(&native_out_root.join("mips"));
    match mips {
        RuntimeOutcome::Ran { .. } => {}
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP design tool mips runtime smoke: {reason}");
        }
    }

    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native design mips runtime out root");
}

#[test]
fn infra_c_runtime_asan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_infra_c_asan");
    compile_module_for_target("dsl/tools/infra.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated infra c asan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("ERROR: AddressSanitizer"),
                "infra c asan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP infra c asan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native infra c asan out root");
}

#[test]
fn sdlc_pipeline_c_runtime_asan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_c_asan");
    compile_module_for_target("dsl/pipelines/sdlc.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated sdlc pipeline c asan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("ERROR: AddressSanitizer"),
                "sdlc pipeline c asan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc pipeline c asan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc pipeline c asan out root");
}

#[test]
fn sdlc_control_plane_c_runtime_asan_smoke_when_available() {
    let native_out_root =
        unique_workspace_target_dir("runtime_native_sdlc_control_plane_c_asan");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "c",
        &native_out_root.join("c"),
    );
    match run_generated_c_with_asan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated sdlc control-plane c asan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("ERROR: AddressSanitizer"),
                "sdlc control-plane c asan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc control-plane c asan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc control-plane c asan out root");
}

#[test]
fn design_tool_c_runtime_asan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_design_tool_c_asan");
    compile_module_for_target("dsl/tools/design.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated design tool c asan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("ERROR: AddressSanitizer"),
                "design tool c asan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP design tool c asan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native design tool c asan out root");
}

#[test]
fn infra_c_runtime_asan_ubsan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_infra_c_asan_ubsan");
    compile_module_for_target("dsl/tools/infra.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan_ubsan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated infra c asan+ubsan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("AddressSanitizer") && !stderr.contains("runtime error:"),
                "infra c asan+ubsan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP infra c asan+ubsan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native infra c asan+ubsan out root");
}

#[test]
fn sdlc_pipeline_c_runtime_asan_ubsan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_sdlc_pipeline_c_asan_ubsan");
    compile_module_for_target("dsl/pipelines/sdlc.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan_ubsan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated sdlc pipeline c asan+ubsan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("AddressSanitizer") && !stderr.contains("runtime error:"),
                "sdlc pipeline c asan+ubsan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc pipeline c asan+ubsan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc pipeline c asan+ubsan out root");
}

#[test]
fn sdlc_control_plane_c_runtime_asan_ubsan_smoke_when_available() {
    let native_out_root =
        unique_workspace_target_dir("runtime_native_sdlc_control_plane_c_asan_ubsan");
    compile_module_for_target(
        "dsl/services/sdlc/control_plane.dag",
        "c",
        &native_out_root.join("c"),
    );
    match run_generated_c_with_asan_ubsan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated sdlc control-plane c asan+ubsan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("AddressSanitizer") && !stderr.contains("runtime error:"),
                "sdlc control-plane c asan+ubsan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP sdlc control-plane c asan+ubsan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native sdlc control-plane c asan+ubsan out root");
}

#[test]
fn design_tool_c_runtime_asan_ubsan_smoke_when_available() {
    let native_out_root = unique_workspace_target_dir("runtime_native_design_tool_c_asan_ubsan");
    compile_module_for_target("dsl/tools/design.dag", "c", &native_out_root.join("c"));
    match run_generated_c_with_asan_ubsan(&native_out_root.join("c")) {
        RuntimeOutcome::Ran { stdout, stderr } => {
            assert!(
                stdout.contains("daglang generated c backend"),
                "generated design tool c asan+ubsan runtime should print backend banner: {stdout}"
            );
            assert!(
                !stderr.contains("AddressSanitizer") && !stderr.contains("runtime error:"),
                "design tool c asan+ubsan smoke should not report sanitizer violations: {stderr}"
            );
        }
        RuntimeOutcome::Skipped { reason } => {
            eprintln!("SKIP design tool c asan+ubsan smoke: {reason}");
        }
    }
    std::fs::remove_dir_all(&native_out_root)
        .expect("failed to cleanup native design tool c asan+ubsan out root");
}

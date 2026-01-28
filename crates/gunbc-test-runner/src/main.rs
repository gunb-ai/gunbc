use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

struct CargoInvocation {
    program: &'static str,
    args_prefix: Vec<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = env::current_dir().map_err(|e| format!("failed to get cwd: {e}"))?;
    let vendor = root.join("vendor");
    if !vendor.is_dir() {
        return Err("vendor/ not found. Run: tools/buck_bootstrap.sh".to_string());
    }

    let cargo_home = resolve_path(&root, "CARGO_HOME", "buck-out/cargo-home");
    let rustup_home = resolve_path(&root, "RUSTUP_HOME", "buck-out/rustup-home");
    let cargo_target_dir = resolve_path(&root, "CARGO_TARGET_DIR", "buck-out/cargo-target");
    let gen_dir = root.join("buck-out/gen");

    fs::create_dir_all(&cargo_home)
        .map_err(|e| format!("failed to create {}: {e}", cargo_home.display()))?;
    fs::create_dir_all(&rustup_home)
        .map_err(|e| format!("failed to create {}: {e}", rustup_home.display()))?;
    fs::create_dir_all(&cargo_target_dir)
        .map_err(|e| format!("failed to create {}: {e}", cargo_target_dir.display()))?;
    fs::create_dir_all(&gen_dir)
        .map_err(|e| format!("failed to create {}: {e}", gen_dir.display()))?;

    write_cargo_config(&cargo_home, &vendor)?;

    let cargo = resolve_cargo_cmd()?;
    let base_env = vec![
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("RUSTUP_HOME".to_string(), rustup_home.display().to_string()),
        ("CARGO_TARGET_DIR".to_string(), cargo_target_dir.display().to_string()),
        ("CARGO_NET_OFFLINE".to_string(), "true".to_string()),
    ];

    run_cargo(
        &cargo,
        &base_env,
        &[
            "run",
            "-p",
            "gunbc-deps",
            "--",
            "--entry",
            "buck_test",
            "--mode",
            "check",
        ],
    )?;

    let gen_out = gen_dir.join("generated_tests.rs");
    run_cargo(
        &cargo,
        &base_env,
        &[
            "run",
            "-p",
            "gunbc-testgen",
            "--",
            "--out",
            gen_out
                .to_str()
                .ok_or("generated tests path is not valid UTF-8")?,
        ],
    )?;

    let mut test_env = base_env.clone();
    test_env.push((
        "GUNBC_GENERATED_TESTS_DIR".to_string(),
        gen_dir.display().to_string(),
    ));

    run_cargo(
        &cargo,
        &test_env,
        &["test", "--workspace", "--offline", "--locked"],
    )?;

    Ok(())
}

fn resolve_path(root: &Path, var: &str, default_rel: &str) -> PathBuf {
    match env::var(var) {
        Ok(value) => PathBuf::from(value),
        Err(_) => root.join(default_rel),
    }
}

fn write_cargo_config(cargo_home: &Path, vendor: &Path) -> Result<(), String> {
    let config = format!(
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"{}\"\n",
        vendor.display()
    );
    let config_path = cargo_home.join("config.toml");
    fs::write(&config_path, config)
        .map_err(|e| format!("failed to write {}: {e}", config_path.display()))?;
    Ok(())
}

fn resolve_cargo_cmd() -> Result<CargoInvocation, String> {
    let override_toolchain = env::var("BUCK_RUSTUP_TOOLCHAIN")
        .ok()
        .or_else(|| env::var("RUSTUP_TOOLCHAIN").ok());

    if let Some(toolchain) = override_toolchain {
        if !command_success("rustup", &["--version"]) {
            return Err("rustup not found. Install rustup or unset BUCK_RUSTUP_TOOLCHAIN.".into());
        }
        let list = rustup_toolchain_list()?;
        if !toolchain_installed(&list, &toolchain) {
            return Err(format!(
                "rustup toolchain '{}' not installed. Run: rustup toolchain install {}",
                toolchain, toolchain
            ));
        }
        return Ok(CargoInvocation {
            program: "cargo",
            args_prefix: vec![format!("+{}", toolchain)],
        });
    }

    if command_success("rustup", &["--version"]) {
        if command_success("rustup", &["show", "active-toolchain"]) {
            return Ok(CargoInvocation {
                program: "cargo",
                args_prefix: Vec::new(),
            });
        }
        let list = rustup_toolchain_list()?;
        if toolchain_installed(&list, "stable") {
            return Ok(CargoInvocation {
                program: "cargo",
                args_prefix: vec!["+stable".to_string()],
            });
        }
        return Err("no default rustup toolchain. Run: rustup default stable".into());
    }

    if command_success("cargo", &["--version"]) {
        return Ok(CargoInvocation {
            program: "cargo",
            args_prefix: Vec::new(),
        });
    }

    Err("cargo not found. Install Rust toolchain or set BUCK_RUSTUP_TOOLCHAIN.".into())
}

fn rustup_toolchain_list() -> Result<String, String> {
    let output = Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .map_err(|e| format!("failed to run rustup toolchain list: {e}"))?;
    if !output.status.success() {
        return Err("rustup toolchain list failed".to_string());
    }
    String::from_utf8(output.stdout)
        .map_err(|e| format!("rustup toolchain list returned invalid UTF-8: {e}"))
}

fn toolchain_installed(list: &str, toolchain: &str) -> bool {
    list.lines().any(|line| line.starts_with(toolchain))
}

fn command_success(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_cargo(
    cargo: &CargoInvocation,
    envs: &[(String, String)],
    args: &[&str],
) -> Result<(), String> {
    let mut cmd = Command::new(cargo.program);
    cmd.args(&cargo.args_prefix);
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    let status = cmd
        .status()
        .map_err(|e| format!("failed to launch {}: {e}", cargo.program))?;
    if !status.success() {
        let mut rendered = cargo.args_prefix.clone();
        rendered.extend(args.iter().map(|arg| (*arg).to_string()));
        return Err(format!(
            "command failed: {} {}",
            cargo.program,
            rendered.join(" ")
        ));
    }
    Ok(())
}

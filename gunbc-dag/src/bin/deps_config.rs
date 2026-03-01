//! gunbc-deps-config main entry point.
//!
//! Ensures or verifies that `deps.toml` matches the canonical tool registry.

#![deny(dead_code)]
use gunbc_cli::{parse, CliParam, ParamType};
use gunbc_codegen::file_writer::{format_diff, FileWriter};
use gunbc_dag::deps_config_resource_def;
use gunbc_dag::resources::DEPS_CONFIG_OUTPUT_PATH;
use gunbc_deps::generate_deps_toml_from_registry;
use gunbc_exec::{print_attention, AttentionLevel};
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_lib_transport::TransportIo;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let schema = vec![
        CliParam::new("mode", ParamType::Str).default("ensure"),
        CliParam::new("path", ParamType::Str)
            .short('p')
            .default(DEPS_CONFIG_OUTPUT_PATH),
    ];
    let parsed = match parse(&argv, &schema) {
        Ok(parsed) => parsed,
        Err(error) => {
            print_attention(
                AttentionLevel::Error,
                "deps-config argument parsing failed",
                &error.to_string(),
            );
            process::exit(1);
        }
    };
    if parsed.help {
        print_help();
        return;
    }

    let dry_run = parsed.dry_run;
    let mode_raw = parsed
        .values
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("ensure");
    let resource_mode = ExecMode::parse_strict(mode_raw).unwrap_or_else(|_| {
        print_attention(
            AttentionLevel::Error,
            "deps-config --mode is invalid",
            "expected one of: ensure, verify",
        );
        process::exit(1);
    });
    let path = parsed
        .values
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(DEPS_CONFIG_OUTPUT_PATH)
        .to_string();

    let expected = generate_deps_toml_from_registry();
    let io = TransportIo::new();

    if resource_mode == ExecMode::Verify {
        verify_mode(&io, &path, &expected);
        return;
    }

    println!("deps-config");
    println!("  path: {}", path);
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!("  resource_mode: {}", resource_mode);
    println!();

    let writer = FileWriter::new(dry_run, &io);
    match writer.write_if_changed(Path::new(&path), expected) {
        Ok(result) => {
            if result.changed {
                if dry_run {
                    println!("(dry-run) deps-config would update {}", result.path);
                } else if result.written {
                    println!("deps-config updated {}", result.path);
                }
            } else {
                println!("deps-config: {} is already up to date", result.path);
            }
        }
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "deps-config --mode=ensure failed",
                &e.to_string(),
            );
            process::exit(1);
        }
    }

    if !dry_run {
        update_manifest_after_deps_config(&path);
    }
}

fn verify_mode(io: &dyn ResourceIo, path: &str, expected: &str) {
    let path_obj = Path::new(path);
    let actual_bytes = match io.read_file(path_obj) {
        Ok(bytes) => bytes,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "deps-config --mode=verify: drift detected",
                &format!("MISSING  {path}\nerror: {e}"),
            );
            eprintln!();
            eprintln!("To fix:");
            if path == DEPS_CONFIG_OUTPUT_PATH {
                eprintln!("  make deps-config");
            } else {
                eprintln!(
                    "  cargo run -p gunbc-dag --bin gunbc-deps-config -- --path {} --mode=ensure",
                    path
                );
            }
            process::exit(1);
        }
    };

    let actual = match String::from_utf8(actual_bytes) {
        Ok(content) => content,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "deps-config --mode=verify failed",
                &format!("{path} is not valid UTF-8: {e}"),
            );
            process::exit(1);
        }
    };

    if actual == expected {
        println!("deps-config --mode=verify: 1 file up to date");
        return;
    }

    print_attention(
        AttentionLevel::Error,
        "deps-config --mode=verify: drift detected",
        &format!("DRIFT  {path}"),
    );
    eprintln!();
    eprintln!("--- Drift diff (expected vs disk) ---");
    eprintln!("{}", format_diff(&actual, expected));
    eprintln!();
    eprintln!("To fix:");
    if path == DEPS_CONFIG_OUTPUT_PATH {
        eprintln!("  make deps-config");
    } else {
        eprintln!(
            "  cargo run -p gunbc-dag --bin gunbc-deps-config -- --path {} --mode=ensure",
            path
        );
    }
    process::exit(1);
}

fn update_manifest_after_deps_config(path: &str) {
    if path != DEPS_CONFIG_OUTPUT_PATH {
        println!(
            "Skipping resource manifest update for non-canonical deps config path: {}",
            path
        );
        return;
    }

    println!();
    println!("Updating resource manifest...");

    #[derive(Clone)]
    struct DepsConfigResource {
        def: ResourceDef,
        outputs: Vec<PathBuf>,
    }

    impl ManagedResource for DepsConfigResource {
        fn definition(&self) -> &ResourceDef {
            &self.def
        }

        fn create(
            &self,
            manifest: &ResourceManifest,
            io: &dyn ResourceIo,
        ) -> Result<ManifestEntry, ResourceError> {
            let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
            Ok(ManifestEntry::new(key, file_count)
                .with_outputs(self.outputs.clone())
                .with_input_files(input_files))
        }
    }

    let resource = DepsConfigResource {
        def: deps_config_resource_def(),
        outputs: vec![PathBuf::from(DEPS_CONFIG_OUTPUT_PATH)],
    };
    let io = TransportIo::new();

    match update_resource_manifest(&resource, &io) {
        Ok(()) => {
            println!("Resource manifest updated.");
        }
        Err(ManifestUpdateError::Load(e)) => {
            eprintln!("Failed to load manifest: {e}");
        }
        Err(ManifestUpdateError::Save(e)) => {
            eprintln!("Failed to write manifest: {e}");
        }
        Err(ManifestUpdateError::Acquire(e)) => {
            eprintln!("Failed to update manifest: {e}");
        }
    }
}

fn print_help() {
    println!("deps-config - Generate or verify deps.toml from tool registry");
    println!();
    println!("USAGE:");
    println!("    deps-config [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -p, --path <VAL>     deps.toml path (default: deps.toml)");
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    --mode=MODE          Resource mode: verify (CI) or ensure (default)");
    println!("    -h, --help           Print this help");
}

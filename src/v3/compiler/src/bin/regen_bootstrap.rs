use std::path::PathBuf;

use v3_compiler::{
    compile_full_bootstrap_dag_from_std_seed,
    compile_full_bootstrap_without_runtime_mirrors_dag_from_std_seed, compile_std_bootstrap_dag,
    generated_files::GENERATED_FILES, render_bootstrap_generated_rs,
    render_bootstrap_std_generated_rs,
};

const GENERATED_STD_FILE: &str = "src/v3/compiler/src/bootstrap_std_generated.rs";
const GENERATED_FULL_FILE: &str = "src/v3/compiler/src/bootstrap_generated.rs";
const GENERATED_NO_RUNTIME_MIRRORS_FILE: &str =
    "src/v3/compiler/src/bootstrap_generated_without_runtime_mirrors.rs";

fn main() {
    for generated_file in [
        GENERATED_STD_FILE,
        GENERATED_FULL_FILE,
        GENERATED_NO_RUNTIME_MIRRORS_FILE,
    ] {
        assert!(
            GENERATED_FILES.contains(&generated_file),
            "`regen_bootstrap` writes `{generated_file}` but that path is not \
             registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
        );
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let std_dag = compile_std_bootstrap_dag();
    let std_formatted = render_bootstrap_std_generated_rs(&std_dag)
        .unwrap_or_else(|e| panic!("regen_bootstrap std: {e}"));
    write_generated(&manifest_dir, "bootstrap_std_generated.rs", &std_formatted);

    let full_dag = compile_full_bootstrap_dag_from_std_seed(std_dag.clone());
    let full_formatted = render_bootstrap_generated_rs(
        &full_dag,
        "dsl/std/*.dag + src/v3/std/*.dag + src/v3/spec/*.dag + src/v3/compiler/*.dag minus tokenize.dag",
        "bootstrapped_fixture_dag",
    )
    .unwrap_or_else(|e| panic!("regen_bootstrap full: {e}"));
    write_generated(&manifest_dir, "bootstrap_generated.rs", &full_formatted);

    let full_no_runtime_mirrors_dag =
        compile_full_bootstrap_without_runtime_mirrors_dag_from_std_seed(std_dag);
    let full_no_runtime_mirrors_formatted = render_bootstrap_generated_rs(
        &full_no_runtime_mirrors_dag,
        "dsl/std/*.dag + src/v3/std/*.dag + src/v3/spec/*.dag + src/v3/compiler/*.dag minus tokenize.dag and runtime_mirrors.dag",
        "bootstrapped_fixture_without_runtime_mirrors_dag",
    )
    .unwrap_or_else(|e| panic!("regen_bootstrap no-runtime-mirrors: {e}"));
    write_generated(
        &manifest_dir,
        "bootstrap_generated_without_runtime_mirrors.rs",
        &full_no_runtime_mirrors_formatted,
    );
}

fn write_generated(manifest_dir: &PathBuf, file_name: &str, contents: &str) {
    let out_path = manifest_dir.join("src").join(file_name);
    std::fs::write(&out_path, contents).unwrap_or_else(|e| panic!("write {file_name}: {e}"));
    println!("wrote {}", out_path.display());
}

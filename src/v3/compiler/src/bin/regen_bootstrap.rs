use std::path::PathBuf;

use v3_compiler::{
    compile_std_bootstrap_dag, generated_files::GENERATED_FILES, render_bootstrap_std_generated_rs,
};

const GENERATED_FILE: &str = "src/v3/compiler/src/bootstrap_std_generated.rs";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_bootstrap` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let dag = compile_std_bootstrap_dag();
    let formatted =
        render_bootstrap_std_generated_rs(&dag).unwrap_or_else(|e| panic!("regen_bootstrap: {e}"));

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("bootstrap_std_generated.rs");
    std::fs::write(&out_path, formatted).expect("write bootstrap_std_generated.rs");
    println!("wrote {}", out_path.display());
}

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=GUNBC_GENERATED_TESTS_DIR");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR missing");
    let dest = Path::new(&out_dir).join("generated_tests.rs");

    let content = match env::var("GUNBC_GENERATED_TESTS_DIR") {
        Ok(dir) => {
            let src = Path::new(&dir).join("generated_tests.rs");
            println!("cargo:rerun-if-changed={}", src.display());
            match fs::read_to_string(&src) {
                Ok(s) => s,
                Err(_) => stub("generated tests path set but file missing"),
            }
        }
        Err(_) => stub("generated tests not configured"),
    };

    fs::write(&dest, content).expect("failed to write generated_tests.rs");
}

fn stub(reason: &str) -> String {
    format!(
        "#[test]\nfn generated_tests_skipped() {{\n    eprintln!(\"{}\");\n}}\n",
        reason
    )
}

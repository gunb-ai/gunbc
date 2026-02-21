use std::path::PathBuf;
fn main() {
    let cwd = std::env::current_dir().unwrap();
    let root = cwd.join("dsl");
    let relative = PathBuf::from("shared/dag_util.dag");
    let candidate = root.join(&relative);
    println!("Candidate {} exists: {}", candidate.display(), candidate.is_file());
}

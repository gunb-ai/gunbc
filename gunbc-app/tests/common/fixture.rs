#![allow(clippy::disallowed_methods)]

use gunbc_test::unique_temp_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CliTestContext {
    root: PathBuf,
    bin_path: &'static str,
}

impl CliTestContext {
    pub fn new(name: &str, bin_path: &'static str) -> Self {
        let root = unique_temp_dir(name);
        std::fs::create_dir_all(&root).expect("CliTestContext failed to create temp directory");
        Self { root, bin_path }
    }

    pub fn command(&self) -> Command {
        let mut cmd = Command::new(self.bin_path);
        cmd.env("HOME", &self.root);
        cmd.env("USERPROFILE", &self.root);
        cmd
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn join(&self, rel_path: impl AsRef<Path>) -> PathBuf {
        self.root.join(rel_path)
    }

    pub fn write_file(&self, rel_path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let target = self.join(rel_path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("CliTestContext create_dir_all");
        }
        std::fs::write(&target, contents).expect("CliTestContext write file");
    }

    pub fn create_dir_all(&self, rel_path: impl AsRef<Path>) {
        std::fs::create_dir_all(self.join(rel_path)).expect("CliTestContext create_dir_all");
    }

    pub fn read_to_string(&self, rel_path: impl AsRef<Path>) -> String {
        std::fs::read_to_string(self.join(rel_path)).expect("CliTestContext read_to_string")
    }

    pub fn remove_dir_all(&self, rel_path: impl AsRef<Path>) {
        std::fs::remove_dir_all(self.join(rel_path)).expect("CliTestContext remove_dir_all");
    }

    pub fn remove_file(&self, rel_path: impl AsRef<Path>) {
        std::fs::remove_file(self.join(rel_path)).expect("CliTestContext remove_file");
    }
}

impl Drop for CliTestContext {
    fn drop(&mut self) {
        if self.root.exists() {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CliTestContext;

    #[test]
    fn helper_methods_round_trip_filesystem_state() {
        let ctx = CliTestContext::new("fixture_helpers", "/bin/true");
        let _ = ctx.command();
        assert!(ctx.path().exists(), "context root should exist");

        ctx.create_dir_all("nested/path");
        let file_path = ctx.join("nested/path/value.txt");
        ctx.write_file("nested/path/value.txt", "hello");
        assert_eq!(
            ctx.read_to_string("nested/path/value.txt"),
            "hello",
            "write/read helper round-trip should match"
        );
        assert!(file_path.exists(), "helper write should create file");

        ctx.remove_file("nested/path/value.txt");
        assert!(
            !file_path.exists(),
            "helper remove_file should remove target file"
        );
        ctx.remove_dir_all("nested");
        assert!(
            !ctx.join("nested").exists(),
            "helper remove_dir_all should remove nested tree"
        );
    }
}

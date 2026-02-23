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
        if self.bin_path.ends_with("gunbc-sdlc") {
            // SDLC compiled profile preflight expects local credential env vars.
            cmd.env("GITHUB_TOKEN", "test-gh-token");
            cmd.env("CODEX_API_KEY", "test-codex-token");
        }
        cmd
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.root
    }

    #[allow(dead_code)]
    pub fn join(&self, rel_path: impl AsRef<Path>) -> PathBuf {
        self.root.join(rel_path)
    }

    #[allow(dead_code)]
    pub fn write_file(&self, rel_path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let target = self.join(rel_path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("CliTestContext create_dir_all");
        }
        std::fs::write(&target, contents).expect("CliTestContext write file");
    }

    #[allow(dead_code)]
    pub fn create_dir_all(&self, rel_path: impl AsRef<Path>) {
        std::fs::create_dir_all(self.join(rel_path)).expect("CliTestContext create_dir_all");
    }

    #[allow(dead_code)]
    pub fn read_to_string(&self, rel_path: impl AsRef<Path>) -> String {
        std::fs::read_to_string(self.join(rel_path)).expect("CliTestContext read_to_string")
    }

    #[allow(dead_code)]
    pub fn remove_dir_all(&self, rel_path: impl AsRef<Path>) {
        std::fs::remove_dir_all(self.join(rel_path)).expect("CliTestContext remove_dir_all");
    }

    #[allow(dead_code)]
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

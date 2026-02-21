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

}

impl Drop for CliTestContext {
    fn drop(&mut self) {
        if self.root.exists() {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

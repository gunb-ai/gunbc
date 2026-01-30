# TODO — gunbc

## Generic CLI Tool Interface

A trait for all CLI tools (gh, git, cargo, etc.) to share upsert mechanics.
Initial implementation in `core/ir/src/transport/github/cli.rs` establishes the pattern.

```rust
trait CliTool {
    fn id(&self) -> &str;
    fn command(&self) -> &str;
    fn verify_command(&self) -> &str;
    fn min_version(&self) -> &str;
    fn install_methods(&self) -> Vec<(Platform, InstallMethod)>;
    fn is_installed(&self) -> bool;
}

enum InstallMethod {
    Apt { packages: &[&str] },
    Brew { packages: &[&str] },
    Cargo { packages: &[&str] },
    Script { script: &str },
    GithubRelease { url_template: &str },
}
```

Tasks:
- [ ] Define `CliTool` trait in `core/ir/src/transport/`
- [ ] Move `InstallMethod` enum from `github/cli.rs` to shared location
- [ ] Refactor `GitHubCLI` to implement trait
- [ ] Add implementations for git, cargo
- [ ] Generate deps.toml from trait implementations
- [ ] Integrate with deps tool upsert mechanics

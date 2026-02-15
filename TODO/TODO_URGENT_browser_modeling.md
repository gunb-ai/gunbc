# URGENT: Cross-Platform Browser Open Modeling

## Problem

Browser opening is currently implemented inline in `gunbc-dag/src/dag_viz/graph.rs` (`execute_open_browser`, lines 556-597). It handles WSL/macOS/Linux but:

1. **Not shared** -- only usable by dag-viz, not by other tools
2. **Platform enum is incomplete** -- `lib/tools/deps/src/platform.rs` has `Platform::Linux` but does not distinguish WSL from native Linux
3. **No environment modeling** -- no concept of containers, WSL, or other runtime environments layered on top of OS platforms
4. **Compile-time vs runtime detection mismatch** -- macOS uses `cfg!(target_os)` (compile-time) while WSL uses `WSL_DISTRO_NAME` env var (runtime)

## Current Implementation

```rust
// gunbc-dag/src/dag_viz/graph.rs
fn execute_open_browser(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let is_wsl = std::env::var("WSL_DISTRO_NAME").is_ok();
    if is_wsl {
        ShellRequest::new("wslview").arg(abs_path)  // Windows browser via WSL
    } else if cfg!(target_os = "macos") {
        ShellRequest::new("open").arg(file_path)     // macOS default browser
    } else {
        ShellRequest::new("xdg-open").arg(file_path) // Linux default browser
    }
}
```

## Requirements

1. **Shared utility** in `lib/primitives` (or similar) for cross-platform browser open
2. **Proper Platform + Environment enum**:
   - Platform: Linux, macOS, Windows
   - Environment: Native, WSL (WSL1, WSL2), Docker, Podman, GitHub Actions, etc.
   - Already modeled in gunb.ai/the-gunbai -- reference those implementations
3. **Runtime detection** for environments (WSL, containers) on top of compile-time OS detection
4. **Browser open command resolution** based on (Platform, Environment) pair:
   - (Linux, WSL) -> `wslview` (opens in Windows host browser)
   - (Linux, Native) -> `xdg-open`
   - (macOS, Native) -> `open`
   - (Windows, Native) -> `start`
   - (Linux, Docker) -> no-op or error (no browser available)
5. **DAG-native**: Should be expressible as a transport operation or utility node, not just a Rust function

## References

- gunb.ai platform modeling
- the-gunbai environment detection
- Current `Platform` enum: `lib/tools/deps/src/platform.rs`
- Current inline implementation: `gunbc-dag/src/dag_viz/graph.rs:556-597`

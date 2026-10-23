use v1_compiler::codex_app_server_stdio_session;

fn main() {
    let code = codex_app_server_stdio_session::run_cli_main();
    std::process::exit(code);
}

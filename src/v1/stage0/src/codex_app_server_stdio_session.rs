//! Typed Codex app-server --stdio session driver (#8166 RejectAndFinishNow).

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const THREAD_START_TIMEOUT: Duration = Duration::from_secs(30);
const TURN_TERMINAL_TIMEOUT: Duration = Duration::from_secs(600);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub const EXIT_THREAD_START_TIMEOUT: i32 = 97;
pub const EXIT_TURN_TERMINAL_TIMEOUT: i32 = 98;

pub struct CodexStdioSessionRequest<'a> {
    pub executable: &'a Path,
    pub codex_home: &'a Path,
    pub cwd: &'a Path,
    pub preamble_lines: &'a [String],
    pub user_input: &'a str,
    pub client_command_id: &'a str,
    pub client_command_id_path: &'a Path,
    pub thread_start_rpc_id: i64,
    pub turn_start_wire_id: i64,
}

pub struct CodexStdioSessionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_codex_app_server_stdio_session(
    req: CodexStdioSessionRequest<'_>,
) -> Result<CodexStdioSessionResult, String> {
    if !req.executable.is_file() {
        return Err(format!(
            "codex executable missing at {}",
            req.executable.display()
        ));
    }

    std::fs::write(req.client_command_id_path, req.client_command_id)
        .map_err(|e| format!("persist client_command_id: {e}"))?;

    let mut fds = [0i32, 0i32];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) != 0 {
            return Err(format!("pipe stdin: {}", std::io::Error::last_os_error()));
        }
    }
    use std::os::unix::io::FromRawFd;
    let stdin_rx_file = unsafe { File::from_raw_fd(fds[0]) };
    let stdin_tx_file = unsafe { File::from_raw_fd(fds[1]) };

    let stdout_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let mut child = spawn_codex_child(&req, stdin_rx_file)?;
    let pid = child.id();

    let stdout_reader = child
        .stdout
        .take()
        .map(|out| spawn_stream_reader(out, Arc::clone(&stdout_buf), "stdout"));
    let stderr_reader = child
        .stderr
        .take()
        .map(|err| spawn_stream_reader(err, Arc::clone(&stderr_buf), "stderr"));

    let stdin_tx_file = stdin_tx_file;
    let preamble: Vec<String> = req.preamble_lines.to_vec();
    let user_input = req.user_input.to_string();
    let thread_start_rpc_id = req.thread_start_rpc_id;
    let turn_start_wire_id = req.turn_start_wire_id;

    let stdout_buf_for_writer = Arc::clone(&stdout_buf);
    let stdin_result = thread::spawn(move || {
        let mut stdin = stdin_tx_file;
        for line in &preamble {
            if writeln!(stdin, "{line}").is_err() {
                return Err("write preamble line failed".to_string());
            }
        }

        let thread_id = poll_thread_start_id(
            &stdout_buf_for_writer,
            thread_start_rpc_id,
            THREAD_START_TIMEOUT,
        )?;

        let turn_line = build_turn_start_line(turn_start_wire_id, &thread_id, &user_input)?;
        if writeln!(stdin, "{turn_line}").is_err() {
            return Err("write turn/start failed".to_string());
        }

        let terminal = poll_turn_terminal(&stdout_buf_for_writer, TURN_TERMINAL_TIMEOUT)?;
        stdin.flush().ok();
        drop(stdin);
        Ok(terminal)
    })
    .join()
    .map_err(|_| "stdin writer panicked".to_string())?;

    if let Some(h) = stdout_reader {
        let _ = h.join();
    }
    if let Some(h) = stderr_reader {
        let _ = h.join();
    }

    let mut exit_code = child
        .wait()
        .map_err(|e| format!("wait codex child: {e}"))?
        .code()
        .unwrap_or(1);

    kill_process_group(pid);

    let stdout = stdout_buf.lock().unwrap().clone();
    let stderr = stderr_buf.lock().unwrap().clone();

    match stdin_result {
        Err(why) if why.starts_with("thread/start") => {
            exit_code = EXIT_THREAD_START_TIMEOUT;
            Ok(CodexStdioSessionResult {
                exit_code,
                stdout,
                stderr: format!("{stderr}\n{why}"),
            })
        }
        Err(why) if why.starts_with("turn terminal") => {
            exit_code = EXIT_TURN_TERMINAL_TIMEOUT;
            Ok(CodexStdioSessionResult {
                exit_code,
                stdout,
                stderr: format!("{stderr}\n{why}"),
            })
        }
        Err(why) => Err(why),
        Ok(terminal_ok) => {
            if !terminal_ok {
                exit_code = EXIT_TURN_TERMINAL_TIMEOUT;
            }
            Ok(CodexStdioSessionResult {
                exit_code,
                stdout,
                stderr,
            })
        }
    }
}

fn spawn_stream_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    buf: Arc<Mutex<String>>,
    label: &'static str,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut lines = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            match lines.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(mut guard) = buf.lock() {
                        guard.push_str(&line);
                    }
                }
                Err(e) => {
                    if let Ok(mut guard) = buf.lock() {
                        guard.push_str(&format!("[drain {label} error: {e}]\n"));
                    }
                    break;
                }
            }
        }
    })
}

fn spawn_codex_child(req: &CodexStdioSessionRequest<'_>, stdin_rx: File) -> Result<Child, String> {
    use std::os::unix::process::CommandExt;

    let mut cmd = std::process::Command::new(req.executable);
    cmd.arg("app-server")
        .arg("--stdio")
        .stdin(Stdio::from(stdin_rx))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("CODEX_HOME", req.codex_home)
        .current_dir(req.cwd);

    unsafe {
        cmd.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn()
        .map_err(|e| format!("spawn codex app-server: {e}"))
}

fn kill_process_group(pid: u32) {
    unsafe {
        let _ = libc::kill(pid as i32, libc::SIGTERM);
        let _ = libc::kill(-(pid as i32), libc::SIGTERM);
    }
}

fn poll_thread_start_id(
    stdout_buf: &Arc<Mutex<String>>,
    rpc_id: i64,
    timeout: Duration,
) -> Result<String, String> {
    let deadline = Instant::now() + timeout;
    let id_needle = format!("\"id\":{rpc_id}");
    loop {
        let snapshot = stdout_buf.lock().unwrap().clone();
        for line in snapshot.lines() {
            if line.contains(&id_needle) {
                if let Some(id) = extract_thread_id_from_line(line) {
                    return Ok(id);
                }
            }
        }
        if Instant::now() >= deadline {
            return Err("thread/start: timed out waiting for rpc response".to_string());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn extract_thread_id_from_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let result = parsed.get("result")?;
    let thread = result.get("thread")?;
    let id = thread.get("id")?;
    id.as_str().map(|s| s.to_string())
}

fn poll_turn_terminal(stdout_buf: &Arc<Mutex<String>>, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let snapshot = stdout_buf.lock().unwrap().clone();
        for line in snapshot.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(method) = parsed.get("method").and_then(|m| m.as_str()) {
                    if method == "turn/completed" || method == "turn/failed" {
                        return Ok(true);
                    }
                }
            }
        }
        if Instant::now() >= deadline {
            return Err(
                "turn terminal: timed out waiting for turn/completed or turn/failed".to_string(),
            );
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn build_turn_start_line(
    turn_start_wire_id: i64,
    thread_id: &str,
    user_input: &str,
) -> Result<String, String> {
    let params = serde_json::json!({
        "threadId": thread_id,
        "input": [{
            "type": "text",
            "text": user_input,
        }],
    });
    let line = serde_json::json!({
        "jsonrpc": "2.0",
        "id": turn_start_wire_id,
        "method": "turn/start",
        "params": params,
    });
    serde_json::to_string(&line).map_err(|e| format!("serialize turn/start: {e}"))
}

pub fn run_cli_main() -> i32 {
    let mut executable = None;
    let mut codex_home = None;
    let mut cwd = None;
    let mut preamble_file = None;
    let mut user_input = None;
    let mut client_command_id = None;
    let mut client_command_id_path = None;
    let mut thread_start_id: i64 = 4;
    let mut turn_start_id: i64 = 5;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--executable" => {
                i += 1;
                executable = args.get(i).cloned();
            }
            "--codex-home" => {
                i += 1;
                codex_home = args.get(i).cloned();
            }
            "--cwd" => {
                i += 1;
                cwd = args.get(i).cloned();
            }
            "--preamble-file" => {
                i += 1;
                preamble_file = args.get(i).cloned();
            }
            "--user-input" => {
                i += 1;
                user_input = args.get(i).cloned();
            }
            "--client-command-id" => {
                i += 1;
                client_command_id = args.get(i).cloned();
            }
            "--client-command-id-path" => {
                i += 1;
                client_command_id_path = args.get(i).cloned();
            }
            "--thread-start-id" => {
                i += 1;
                match args.get(i) {
                    None => {
                        eprintln!(
                            "codex_app_server_stdio_session: missing value for --thread-start-id"
                        );
                        return 2;
                    }
                    Some(s) => match s.parse::<i64>() {
                        Ok(id) => thread_start_id = id,
                        Err(e) => {
                            eprintln!(
                                "codex_app_server_stdio_session: invalid --thread-start-id {:?}: {e}",
                                s
                            );
                            return 2;
                        }
                    },
                }
            }
            "--turn-start-id" => {
                i += 1;
                match args.get(i) {
                    None => {
                        eprintln!(
                            "codex_app_server_stdio_session: missing value for --turn-start-id"
                        );
                        return 2;
                    }
                    Some(s) => match s.parse::<i64>() {
                        Ok(id) => turn_start_id = id,
                        Err(e) => {
                            eprintln!(
                                "codex_app_server_stdio_session: invalid --turn-start-id {:?}: {e}",
                                s
                            );
                            return 2;
                        }
                    },
                }
            }
            _ => {}
        }
        i += 1;
    }

    if executable.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --executable");
        return 2;
    }
    if codex_home.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --codex-home");
        return 2;
    }
    if cwd.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --cwd");
        return 2;
    }
    if preamble_file.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --preamble-file");
        return 2;
    }
    if user_input.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --user-input");
        return 2;
    }
    if client_command_id.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --client-command-id");
        return 2;
    }
    if client_command_id_path.is_none() {
        eprintln!("codex_app_server_stdio_session: missing --client-command-id-path");
        return 2;
    }

    let preamble_content = match std::fs::read_to_string(preamble_file.as_ref().unwrap()) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("codex_app_server_stdio_session: read preamble file: {e}");
            return 1;
        }
    };

    let preamble_lines: Vec<String> = preamble_content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    let req = CodexStdioSessionRequest {
        executable: Path::new(executable.as_ref().unwrap()),
        codex_home: Path::new(codex_home.as_ref().unwrap()),
        cwd: Path::new(cwd.as_ref().unwrap()),
        preamble_lines: &preamble_lines,
        user_input: user_input.as_ref().unwrap(),
        client_command_id: client_command_id.as_ref().unwrap(),
        client_command_id_path: Path::new(client_command_id_path.as_ref().unwrap()),
        thread_start_rpc_id: thread_start_id,
        turn_start_wire_id: turn_start_id,
    };

    match run_codex_app_server_stdio_session(req) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            result.exit_code
        }
        Err(why) => {
            eprintln!("codex_app_server_stdio_session: {why}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_turn_start_line_escapes_json_in_user_input() {
        let line = build_turn_start_line(5, "thread-1", "say \"hi\" and \\ backslash").unwrap();
        assert!(line.contains(r#"\"hi\""#));
        assert!(line.contains(r#"\\"#));
    }
}

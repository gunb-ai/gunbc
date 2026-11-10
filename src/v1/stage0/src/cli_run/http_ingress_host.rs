//! THE SEED'S REALIZATION OF AN HTTP INGRESS BINDING (`gunbc.http_ingress`).
//!
//! `gunbc run` evaluates an entry function once. When its answer is a `gunbc.http_ingress`
//! `HttpIngressAdmission`, this module realizes it: a refused admission exits nonzero before any
//! listener exists, and a bound one is served -- bind, accept, and one independent evaluation of
//! the binding's handler per admitted request (`gunbc.plans.converged_ingress`). There is no serve
//! verb: serving is a deployment fact about an ordinary entry, carried by the binding.
//!
//! THIS MODULE DECIDES NOTHING ABOUT A REQUEST. It performs only what no fold can: bind and accept
//! the listener, read the kernel-attested peer of a unix connection, read bytes within the limits
//! the binding hands it, arm the per-request evaluation budget around the handler call, and write
//! the bytes the response folds return. Every decision between those reads and writes is a fold in
//! `gunbc.http_ingress` or `gunbc.native_serve` that this module evaluates in the loaded program:
//! the handler's argument plan (`http_ingress_fact_plan`), whether a head is admissible and how many
//! body bytes follow (`http_ingress_frame`), which routes answer before anything is evaluated and
//! with what bytes (`http_ingress_pre_evaluation_routes`, evaluated once at startup, so `/livez`
//! never enters the evaluator), the handler's arguments (`http_ingress_arguments`), what a read failure
//! or handler error means (`http_ingress_read_refused`, `http_ingress_handler_error`,
//! `http_ingress_not_a_response`), and the response head's exact bytes (`http_ingress_outgoing`,
//! `http_ingress_wire_head`). The one consequence rendered here is a budget refusal, because the
//! evaluation that would have rendered it is the one that was aborted (`serve_budget_refusal`).

use super::serve_budget_refusal;
use crate::v1_interpreter::{self, InterpContext, Value};
use std::io::{BufRead, Read, Write};

/// What an entry's answer says about ingress: not a binding at all, a refused binding, or a bound
/// one to realize.
pub enum IngressAnswer {
    NotABinding,
    Refused(String),
    Bound(Value),
}

/// The declaration an ingress answer must inhabit. A runtime variant carries only its spellings, so
/// the spelling is not the identity: the arm must be declared, in this run's closure, by exactly this
/// module's coproduct.
const INGRESS_MODULE: &str = "gunbc.http_ingress";
const INGRESS_COPRODUCT: &str = "HttpIngressAdmission";

/// Classify an entry's answer. A value is an ingress binding only when its arm is declared solely by
/// `gunbc.http_ingress` `HttpIngressAdmission` in the run's closure. A same-spelled lookalike declared
/// elsewhere is an ordinary answer, and a closure in which both declare the arm refuses, because the
/// value could denote either.
pub fn ingress_answer(value: &Value, ctx: &InterpContext) -> IngressAnswer {
    let Value::Variant {
        type_name,
        variant_name,
        fields,
    } = value
    else {
        return IngressAnswer::NotABinding;
    };
    if !ctx.sym_eq(*type_name, INGRESS_COPRODUCT) {
        return IngressAnswer::NotABinding;
    }
    let arm = ctx.resolve(*variant_name);
    let declarers = ctx.variant_arm_declarers(&arm);
    let ours = (INGRESS_MODULE.to_string(), INGRESS_COPRODUCT.to_string());
    if !declarers.contains(&ours) {
        return IngressAnswer::NotABinding;
    }
    if declarers.len() > 1 {
        return IngressAnswer::Refused(format!(
            "the answer's arm `{arm}` is declared by more than one coproduct in this closure ({}), so it cannot be read as {INGRESS_MODULE} {INGRESS_COPRODUCT}",
            declarers
                .iter()
                .map(|(m, c)| format!("{m} {c}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if arm == "HttpIngressBound" {
        match ctx.field(fields, "binding") {
            Some(binding) => IngressAnswer::Bound(binding.clone()),
            None => IngressAnswer::Refused("HttpIngressBound carries no binding".to_string()),
        }
    } else {
        match ctx.field(fields, "reason") {
            Some(Value::Str(s)) => IngressAnswer::Refused(s.to_string()),
            _ => IngressAnswer::Refused(format!("HttpIngressAdmission::{arm} is not a binding")),
        }
    }
}

/// Evaluate one fold of the loaded program with named arguments.
fn fold(ctx: &InterpContext, name: &str, args: Vec<(&str, Value)>) -> Result<Value, String> {
    let named: Vec<(Option<String>, Value)> = args
        .into_iter()
        .map(|(n, v)| (Some(n.to_string()), v))
        .collect();
    v1_interpreter::run_in_context_with_args(ctx, name, &named, true)
        .map_err(|e| format!("{name}: {e}"))
}

fn fold_int(ctx: &InterpContext, name: &str, binding: &Value) -> Result<i64, String> {
    match &fold(ctx, name, vec![("b", binding.clone())])? {
        Value::Int(n) => Ok(*n),
        other => Err(format!(
            "{name} answered {}, not an Int",
            ctx.format_value(other)
        )),
    }
}

fn fold_text(ctx: &InterpContext, name: &str, args: Vec<(&str, Value)>) -> Result<String, String> {
    match &fold(ctx, name, args)? {
        Value::Str(s) => Ok(s.to_string()),
        other => Err(format!(
            "{name} answered {}, not a String",
            ctx.format_value(other)
        )),
    }
}

fn str_list(items: &[String]) -> Value {
    v1_interpreter::list_value(
        items
            .iter()
            .map(v1_interpreter::str_value)
            .collect::<Vec<_>>(),
    )
}

// ONE CONNECTION SHAPE FOR BOTH LISTENERS. The request read and the response write are the same
// bytes over TCP and over a unix socket; only binding, accepting and the peer differ.
trait IngressConnection: Read + Write {
    fn set_read_timeout_for_request(&self, d: std::time::Duration) -> std::io::Result<()>;
}

impl IngressConnection for std::net::TcpStream {
    fn set_read_timeout_for_request(&self, d: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(d))
    }
}

impl IngressConnection for std::os::unix::net::UnixStream {
    fn set_read_timeout_for_request(&self, d: std::time::Duration) -> std::io::Result<()> {
        self.set_read_timeout(Some(d))
    }
}

enum IngressListener {
    Tcp(std::net::TcpListener),
    /// The listener, its path, and the path's lock, held for the listener's lifetime.
    Unix(std::os::unix::net::UnixListener, String, std::fs::File),
}

/// Bind the binding's listener. A unix socket path that already exists is removed only when it IS
/// a socket (a previous process's); any other file at that path refuses, so a mistyped path never
/// deletes data. The socket file's mode is the binding's (`http_ingress_unix_socket_mode`).
fn bind_listener(ctx: &InterpContext, listener: &Value) -> Result<IngressListener, String> {
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = listener
    else {
        return Err(format!(
            "listener {} is not a variant",
            ctx.format_value(listener)
        ));
    };
    if ctx.sym_eq(*variant_name, "HttpIngressTcp") {
        let host = match ctx.field(fields, "host") {
            Some(Value::Str(s)) => s.to_string(),
            _ => return Err("HttpIngressTcp carries no host".to_string()),
        };
        let port = match ctx.field(fields, "port") {
            Some(Value::Int(n)) => {
                u16::try_from(*n).map_err(|_| format!("port {n} out of range"))?
            }
            _ => return Err("HttpIngressTcp carries no port".to_string()),
        };
        return bind_tcp(&host, port);
    }
    let path = match ctx.field(fields, "path") {
        Some(Value::Str(s)) => s.to_string(),
        _ => return Err("HttpIngressUnixSocket carries no path".to_string()),
    };
    let mode = match ctx.field(fields, "mode") {
        Some(Value::Int(n)) => {
            u32::try_from(*n).map_err(|_| format!("socket mode {n} out of range"))?
        }
        _ => return Err("HttpIngressUnixSocket carries no mode".to_string()),
    };
    bind_unix(&path, mode)
}

fn bind_tcp(host: &str, port: u16) -> Result<IngressListener, String> {
    std::net::TcpListener::bind((host, port))
        .map(IngressListener::Tcp)
        .map_err(|e| format!("failed to bind {host}:{port}: {e}"))
}

/// THE SOCKET PATH IS OWNED BY WHOEVER HOLDS ITS LOCK: an exclusive flock on `<path>.lock`, taken
/// before the path is looked at and held for the listener's whole lifetime, so two realizations
/// started on one path cannot both judge it stale and the second never unlinks the first's socket
/// (side-chat review 5786866091). The same protocol as the native door's rendered main.
fn lock_socket_path(path: &str) -> Result<std::fs::File, String> {
    use std::os::unix::io::AsRawFd;
    let lock_path = format!("{path}.lock");
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| format!("could not open the socket's lock {lock_path}: {e}"))?;
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let e = std::io::Error::last_os_error();
        return Err(if e.kind() == std::io::ErrorKind::WouldBlock {
            format!("unix socket {path} is held by another realization (its lock {lock_path} is taken); refusing to replace it")
        } else {
            format!("could not lock {lock_path}: {e}")
        });
    }
    Ok(lock)
}

fn bind_unix(path: &str, mode: u32) -> Result<IngressListener, String> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};
    let lock = lock_socket_path(path)?;
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if !meta.file_type().is_socket() {
            return Err(format!(
                "unix socket {path} exists and is not a socket; refusing to replace it"
            ));
        }
        // Stale only when SHOWN stale: a refused connection means no process listens there. A live
        // listener is never displaced, and a socket whose staleness cannot be shown is refused.
        match std::os::unix::net::UnixStream::connect(path) {
            Ok(_) => {
                return Err(format!(
                    "unix socket {path} has a live listener; refusing to displace it"
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {}
            Err(e) => {
                return Err(format!(
                    "unix socket {path} could not be shown stale ({e}); refusing to replace it"
                ))
            }
        }
        std::fs::remove_file(path)
            .map_err(|e| format!("could not remove the stale socket {path}: {e}"))?;
    }
    let listener = std::os::unix::net::UnixListener::bind(path)
        .map_err(|e| format!("failed to bind unix socket {path}: {e}"))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .map_err(|e| format!("could not set the socket {path} to mode {mode:o}: {e}"))?;
    Ok(IngressListener::Unix(listener, path.to_string(), lock))
}

impl IngressListener {
    fn bound_address(&self) -> std::io::Result<(String, i64)> {
        match self {
            IngressListener::Tcp(l) => l
                .local_addr()
                .map(|a| (a.ip().to_string(), i64::from(a.port()))),
            IngressListener::Unix(_, path, _) => Ok((format!("unix:{path}"), 0)),
        }
    }

    // THE PEER IS READ FROM THE KERNEL, NEVER FROM THE REQUEST. A unix connection whose peer
    // credentials cannot be read or resolved to an account is dropped here, before any request is
    // read: an unattested peer on the socket that exists to attest peers is a refusal.
    fn accept(&self) -> std::io::Result<(Box<dyn IngressConnection>, String)> {
        match self {
            IngressListener::Tcp(l) => l
                .accept()
                .map(|(s, _)| (Box::new(s) as Box<dyn IngressConnection>, String::new())),
            IngressListener::Unix(l, _, _) => {
                let (s, _) = l.accept()?;
                let user = peer_user(&s)?;
                Ok((Box::new(s) as Box<dyn IngressConnection>, user))
            }
        }
    }
}

// The kernel-attested peer uid of a unix-socket connection. Linux answers through SO_PEERCRED; macOS
// has no SO_PEERCRED and answers the same fact through getpeereid. Any other target refuses.
#[cfg(target_os = "linux")]
fn peer_uid(s: &std::os::unix::net::UnixStream) -> std::io::Result<libc::uid_t> {
    use std::os::unix::io::AsRawFd;
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let rc = unsafe {
        libc::getsockopt(
            s.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut libc::ucred as *mut libc::c_void,
            &mut len,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(cred.uid)
}

#[cfg(target_os = "macos")]
fn peer_uid(s: &std::os::unix::net::UnixStream) -> std::io::Result<libc::uid_t> {
    use std::os::unix::io::AsRawFd;
    let mut uid: libc::uid_t = 0;
    let mut gid: libc::gid_t = 0;
    if unsafe { libc::getpeereid(s.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(uid)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn peer_uid(_s: &std::os::unix::net::UnixStream) -> std::io::Result<libc::uid_t> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unix-socket peer credentials are not modeled for this target",
    ))
}

fn peer_user(s: &std::os::unix::net::UnixStream) -> std::io::Result<String> {
    let uid = peer_uid(s)?;
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buf = vec![0 as libc::c_char; 4096];
    let mut result: *mut libc::passwd = std::ptr::null_mut();
    let rc = unsafe { libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result) };
    if rc != 0 || result.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("peer uid {uid} resolves to no account"),
        ));
    }
    let name = unsafe { std::ffi::CStr::from_ptr(pwd.pw_name) };
    Ok(name.to_string_lossy().into_owned())
}

/// One line of the request head, terminators stripped, charged against the head budget. `Ok(None)`
/// is a clean close or an idle timeout before any byte: nothing was asked.
fn read_line(reader: &mut impl BufRead, budget: &mut usize) -> Result<Option<String>, String> {
    let mut bytes = Vec::new();
    match reader.read_until(b'\n', &mut bytes) {
        Ok(0) => return Ok(None),
        Ok(n) if n > *budget => {
            return Err("the request head exceeds the serve head limit".to_string())
        }
        Ok(n) => *budget -= n,
        Err(e)
            if bytes.is_empty()
                && matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
        {
            return Ok(None)
        }
        Err(e) => return Err(format!("read: {e}")),
    }
    if bytes.ends_with(b"\n") {
        bytes.pop();
    }
    if bytes.ends_with(b"\r") {
        bytes.pop();
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "the request head is not utf-8".to_string())
}

/// One row of the graph's pre-evaluation route table (`http_ingress_pre_evaluation_routes`), its
/// response already rendered to bytes at startup.
struct PreEvaluationRoute {
    method: String,
    target: String,
    bytes: Vec<u8>,
}

struct Limits {
    head: usize,
    body: usize,
    read_timeout: std::time::Duration,
}

/// Why a request's evaluation produced no bytes: the per-request budget was crossed (rendered as the
/// budget refusal once the scope has closed), or a fold of the loaded program could not answer.
enum RequestFailure {
    Budget(v1_interpreter::InterpError),
    Defect(String),
}

impl From<String> for RequestFailure {
    fn from(s: String) -> Self {
        RequestFailure::Defect(s)
    }
}

/// [`fold`] inside the per-request budget: a crossed budget stays typed so it can be answered.
fn eval(
    ctx: &InterpContext,
    name: &str,
    args: Vec<(&str, Value)>,
) -> Result<Value, RequestFailure> {
    let named: Vec<(Option<String>, Value)> = args
        .into_iter()
        .map(|(n, v)| (Some(n.to_string()), v))
        .collect();
    v1_interpreter::run_in_context_with_args(ctx, name, &named, true).map_err(|e| {
        if serve_budget_refusal::serve_budget_refusal_from_exceeded(&e).is_some() {
            RequestFailure::Budget(e)
        } else {
            RequestFailure::Defect(format!("{name}: {e}"))
        }
    })
}

/// The request's head as read, before anything is evaluated.
enum HeadRead {
    Idle,
    Refused(String),
    PreRendered(usize),
    Head {
        request_line: String,
        header_lines: Vec<String>,
    },
}

fn read_head(
    reader: &mut impl BufRead,
    limits: &Limits,
    routes: &[PreEvaluationRoute],
) -> HeadRead {
    let mut budget = limits.head;
    let request_line = match read_line(reader, &mut budget) {
        Ok(Some(line)) => line,
        Ok(None) => return HeadRead::Idle,
        Err(reason) => return HeadRead::Refused(reason),
    };
    // THE PRE-EVALUATION TABLE IS CONSULTED BEFORE ANYTHING IS EVALUATED. The comparison is the
    // request line's method and target against the graph's rows, exactly; the host chooses no route.
    let mut tokens = request_line.split_whitespace();
    if let (Some(method), Some(target)) = (tokens.next(), tokens.next()) {
        if let Some(i) = routes
            .iter()
            .position(|r| r.method == method && r.target == target)
        {
            return HeadRead::PreRendered(i);
        }
    }
    let mut header_lines = Vec::new();
    loop {
        match read_line(reader, &mut budget) {
            Ok(Some(line)) if line.is_empty() => break,
            Ok(Some(line)) => header_lines.push(line),
            Ok(None) => {
                return HeadRead::Refused("connection closed before end of headers".to_string())
            }
            Err(reason) => return HeadRead::Refused(reason),
        }
    }
    HeadRead::Head {
        request_line,
        header_lines,
    }
}

/// Everything a request's bytes depend on, evaluated inside the one per-request budget: framing,
/// the body read the frame fold sized, the handler's arguments, the handler, and the response bytes.
#[allow(clippy::too_many_arguments)]
fn evaluate_request(
    ctx: &InterpContext,
    binding: &Value,
    facts: &Value,
    handler: &str,
    reader: &mut impl Read,
    request_line: &str,
    header_lines: &[String],
    peer: &str,
) -> Result<Vec<u8>, RequestFailure> {
    let frame = eval(
        ctx,
        "http_ingress_frame",
        vec![
            ("b", binding.clone()),
            ("request_line", v1_interpreter::str_value(request_line)),
            ("header_lines", str_list(header_lines)),
        ],
    )?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &frame
    else {
        return Err(format!("http_ingress_frame answered {}", ctx.format_value(&frame)).into());
    };
    let response = if !ctx.sym_eq(*variant_name, "NativeServeFramed") {
        ctx.field(fields, "response")
            .cloned()
            .ok_or_else(|| format!("http_ingress_frame answered {}", ctx.format_value(&frame)))?
    } else {
        let head = ctx
            .field(fields, "head")
            .cloned()
            .ok_or_else(|| "NativeServeFramed carries no head".to_string())?;
        let body_length = match ctx.field(fields, "body_length") {
            Some(Value::Int(n)) => usize::try_from(*n).map_err(|_| format!("body_length {n}"))?,
            _ => {
                return Err("NativeServeFramed carries no body_length"
                    .to_string()
                    .into())
            }
        };
        let mut body_bytes = vec![0u8; body_length];
        let body = match reader.read_exact(&mut body_bytes) {
            Err(e) => Err(format!("read body: {e}")),
            Ok(()) => {
                String::from_utf8(body_bytes).map_err(|_| "the body is not utf-8".to_string())
            }
        };
        match body {
            Err(reason) => eval(
                ctx,
                "http_ingress_read_refused",
                vec![("reason", v1_interpreter::str_value(reason))],
            )?,
            Ok(body) => answer_with_handler(ctx, binding, facts, handler, head, body, peer)?,
        }
    };
    render_response_in(ctx, response)
}

/// The handler's named arguments, as the arguments fold planned them. Each arm unwraps to its value;
/// the host chooses no name and no value.
fn handler_args(
    ctx: &InterpContext,
    arguments: &Value,
) -> Result<Vec<(Option<String>, Value)>, String> {
    let Value::List(items) = arguments else {
        return Err(format!(
            "http_ingress_arguments answered {}",
            ctx.format_value(arguments)
        ));
    };
    items
        .iter()
        .map(|item| match item {
            Value::Variant { fields, .. } => {
                let name = match ctx.field(fields, "name") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => return Err("an ingress argument carries no name".to_string()),
                };
                let value = ctx
                    .field(fields, "value")
                    .cloned()
                    .ok_or_else(|| format!("ingress argument {name} carries no value"))?;
                Ok((Some(name), value))
            }
            other => Err(format!(
                "ingress argument {} is not a variant",
                ctx.format_value(other)
            )),
        })
        .collect()
}

/// Render a response's bytes: the outgoing fold judges it, the head fold renders the head, and the
/// host appends the body's octets.
fn render_response(ctx: &InterpContext, response: Value) -> Result<Vec<u8>, String> {
    fold(ctx, "http_ingress_outgoing", vec![("r", response)]).and_then(|outgoing| {
        let body = match &outgoing {
            Value::Record { fields, .. } => match ctx.field(fields, "body") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err("NativeServeOutgoing carries no body".to_string()),
            },
            other => {
                return Err(format!(
                    "http_ingress_outgoing answered {}",
                    ctx.format_value(other)
                ))
            }
        };
        let head = fold_text(
            ctx,
            "http_ingress_wire_head",
            vec![
                ("o", outgoing),
                ("body_octets", Value::Int(body.len() as i64)),
            ],
        )?;
        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(body.as_bytes());
        Ok(bytes)
    })
}

fn render_response_in(ctx: &InterpContext, response: Value) -> Result<Vec<u8>, RequestFailure> {
    let outgoing = eval(ctx, "http_ingress_outgoing", vec![("r", response)])?;
    let body = match &outgoing {
        Value::Record { fields, .. } => match ctx.field(fields, "body") {
            Some(Value::Str(s)) => s.to_string(),
            _ => return Err("NativeServeOutgoing carries no body".to_string().into()),
        },
        other => {
            return Err(
                format!("http_ingress_outgoing answered {}", ctx.format_value(other)).into(),
            )
        }
    };
    let head = match &eval(
        ctx,
        "http_ingress_wire_head",
        vec![
            ("o", outgoing),
            ("body_octets", Value::Int(body.len() as i64)),
        ],
    )? {
        Value::Str(s) => s.to_string(),
        other => {
            return Err(format!(
                "http_ingress_wire_head answered {}",
                ctx.format_value(other)
            )
            .into())
        }
    };
    let mut bytes = head.into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    Ok(bytes)
}

fn write_bytes(stream: &mut dyn IngressConnection, bytes: &[u8]) {
    if let Err(e) = stream.write_all(bytes) {
        eprintln!("http-ingress: write error: {e}");
    }
}

/// Realize a bound `HttpIngressBinding`. Returns only on a startup refusal (exit status 1 or 2 and
/// the reason); once the listener serves, it never returns.
pub fn realize(ctx: &InterpContext, binding: Value) -> (i32, String) {
    v1_interpreter::with_active_context(ctx, || realize_in(ctx, binding))
}

fn realize_in(ctx: &InterpContext, binding: Value) -> (i32, String) {
    let startup = || -> Result<_, String> {
        let handler = fold_text(
            ctx,
            "http_ingress_handler_identity",
            vec![("b", binding.clone())],
        )?;
        // FAIL FAST ON THE DECLARED KEY SET. The fold names this host consults by name are
        // declared ONCE in .dag (gunbc.http_ingress `data http_ingress_host_read_folds` + its
        // reader fold); the infer-side registry (src/v1/04_infer.dag `runtime_key_readers`,
        // instance `loaded_index_keyed_call`) seeds them into the demanded realization -- and
        // the host reads THE SAME ROW here, at the top of startup, before its first real consult
        // (this replaced the srv1 crash-loop: the demanded grain realized none of these folds,
        // and the first consult aborted the unit ~110s in with one opaque loaded-index miss).
        // If this execution's loaded index is missing any declared key -- or the argv-carried
        // handler -- refuse HERE, naming EVERY missing key.
        let mut missing: Vec<String> =
            match &fold(ctx, "http_ingress_host_read_folds_reader", vec![])? {
                Value::List(items) => items
                    .iter()
                    .filter_map(|item| match item {
                        Value::Str(s) => Some(s.as_ref().to_string()),
                        _ => None,
                    })
                    .filter(|name| !v1_interpreter::function_exists(ctx, name))
                    .collect(),
                other => {
                    return Err(format!(
                        "http_ingress_host_read_folds_reader answered a non-list: {}",
                        ctx.format_value(other)
                    ))
                }
            };
        if !v1_interpreter::qualified_function_exists(ctx, &handler) {
            missing.push(handler.clone());
        }
        if !missing.is_empty() {
            return Err(format!(
                "runtime-keyed reads not realized by this execution's loaded index: {}",
                missing.join(", ")
            ));
        }
        let limits = Limits {
            head: usize::try_from(fold_int(ctx, "http_ingress_head_limit_bytes", &binding)?)
                .map_err(|e| e.to_string())?,
            body: usize::try_from(fold_int(ctx, "http_ingress_body_limit_bytes", &binding)?)
                .map_err(|e| e.to_string())?,
            read_timeout: std::time::Duration::from_millis(
                u64::try_from(fold_int(ctx, "http_ingress_read_timeout_ms", &binding)?)
                    .map_err(|e| e.to_string())?,
            ),
        };
        let cpu = u64::try_from(fold_int(ctx, "http_ingress_cpu_limit_ms", &binding)?)
            .map_err(|e| e.to_string())?;
        let wall = u64::try_from(fold_int(ctx, "http_ingress_wall_limit_ms", &binding)?)
            .map_err(|e| e.to_string())?;
        // THE ARGUMENT PLAN IS DERIVED ONCE, before the listener binds, from the handler's declared
        // parameters. A handler absent from the closure, or a parameter naming no fact, refuses here.
        let declared = v1_interpreter::declared_parameter_names(ctx, &handler)
            .ok_or_else(|| format!("no function `{handler}` in the run's closure (the binding names a module and handler; the binding module must import that module)"))?;
        let plan = fold(
            ctx,
            "http_ingress_fact_plan",
            vec![("b", binding.clone()), ("declared", str_list(&declared))],
        )?;
        let facts = match &plan {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "HttpIngressFactsPlanned") => ctx
                .field(fields, "facts")
                .cloned()
                .ok_or_else(|| "HttpIngressFactsPlanned carries no facts".to_string())?,
            Value::Variant { fields, .. } => {
                return Err(match ctx.field(fields, "reason") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => ctx.format_value(&plan),
                })
            }
            other => {
                return Err(format!(
                    "http_ingress_fact_plan answered {}",
                    ctx.format_value(other)
                ))
            }
        };
        let listener_value = match &binding {
            Value::Record { fields, .. } => ctx
                .field(fields, "listener")
                .cloned()
                .ok_or_else(|| "the binding carries no listener".to_string())?,
            other => {
                return Err(format!(
                    "the binding {} is not a record",
                    ctx.format_value(other)
                ))
            }
        };
        // WHAT A LISTENER MAY CARRY is the graph's (gunbc.http_ingress http_ingress_listener_refusal),
        // judged before any bind or path mutation, because a constructor may build its binding
        // directly; the native door's start consumes the same fold.
        let refusal = fold(
            ctx,
            "http_ingress_listener_refusal",
            vec![("listener", listener_value.clone())],
        )?;
        match &refusal {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Present") => {
                return Err(match ctx.field(fields, "value") {
                    Some(Value::Str(s)) => s.to_string(),
                    _ => ctx.format_value(&refusal),
                })
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Absent") => {}
            other => {
                return Err(format!(
                    "http_ingress_listener_refusal answered {}",
                    ctx.format_value(other)
                ))
            }
        }
        let listener = bind_listener(ctx, &listener_value)?;
        let (bound_host, bound_port) = listener
            .bound_address()
            .map_err(|e| format!("bound, but could not read the bound address: {e}"))?;
        let announcement = fold_text(
            ctx,
            "http_ingress_announcement",
            vec![
                ("b", binding.clone()),
                ("bound_host", v1_interpreter::str_value(&bound_host)),
                ("bound_port", Value::Int(bound_port)),
            ],
        )?;
        let table = fold(
            ctx,
            "http_ingress_pre_evaluation_routes",
            vec![
                ("b", binding.clone()),
                ("bound_host", v1_interpreter::str_value(&bound_host)),
                ("bound_port", Value::Int(bound_port)),
            ],
        )?;
        let Value::List(rows) = &table else {
            return Err(format!(
                "http_ingress_pre_evaluation_routes answered {}",
                ctx.format_value(&table)
            ));
        };
        let mut routes = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            let Value::Record { fields, .. } = row else {
                return Err(format!(
                    "pre-evaluation route {} is not a record",
                    ctx.format_value(row)
                ));
            };
            let text = |name: &str| match ctx.field(fields, name) {
                Some(Value::Str(s)) => Ok(s.to_string()),
                _ => Err(format!("a pre-evaluation route carries no {name}")),
            };
            let response = ctx
                .field(fields, "response")
                .cloned()
                .ok_or_else(|| "a pre-evaluation route carries no response".to_string())?;
            routes.push(PreEvaluationRoute {
                method: text("method")?,
                target: text("target")?,
                bytes: render_response(ctx, response)?,
            });
        }
        Ok((
            handler,
            limits,
            cpu,
            wall,
            facts,
            listener,
            routes,
            announcement,
        ))
    };
    let (handler, limits, cpu, wall, facts, listener, routes, announcement) = match startup() {
        Ok(parts) => parts,
        Err(reason) => return (1, format!("REFUSED: ingress binding: {reason}")),
    };
    eprintln!("{announcement}");
    let armed = serve_budget_refusal::serve_armed_contract(handler.clone(), Some(cpu), Some(wall));
    loop {
        let (mut conn, peer) = match listener.accept() {
            Ok(accepted) => accepted,
            Err(e) => {
                eprintln!("http-ingress: accept error: {e}");
                continue;
            }
        };
        let stream: &mut dyn IngressConnection = &mut *conn;
        if let Err(e) = stream.set_read_timeout_for_request(limits.read_timeout) {
            if let Some(bytes) = read_refused_bytes(ctx, format!("set_read_timeout: {e}")) {
                write_bytes(stream, &bytes);
            }
            continue;
        }
        let bytes = {
            let mut reader =
                std::io::BufReader::new((&mut *stream).take((limits.head + limits.body) as u64));
            match read_head(&mut reader, &limits, &routes) {
                HeadRead::Idle => None,
                HeadRead::PreRendered(i) => Some(routes[i].bytes.clone()),
                // Nothing request-dependent has been evaluated: the head did not arrive whole.
                HeadRead::Refused(reason) => read_refused_bytes(ctx, reason),
                HeadRead::Head {
                    request_line,
                    header_lines,
                } => {
                    // ONE BUDGET PER REQUEST, OVER EVERY EVALUATION ITS BYTES DEPEND ON -- framing,
                    // arguments, the handler and the response folds -- armed once the head is read so
                    // no socket wait is charged to it, and restored when it ends: `ctx` outlives every
                    // request, so a leaked deadline would refuse every later one.
                    let outcome = {
                        let _budget = ctx.enter_evaluation_budget(
                            serve_budget_refusal::serve_contract_entry(&armed),
                            serve_budget_refusal::serve_contract_cpu_limit_ms(&armed),
                            serve_budget_refusal::serve_contract_wall_limit_ms(&armed),
                        );
                        evaluate_request(
                            ctx,
                            &binding,
                            &facts,
                            serve_budget_refusal::serve_contract_entry(&armed),
                            &mut reader,
                            &request_line,
                            &header_lines,
                            &peer,
                        )
                    };
                    match outcome {
                        Ok(bytes) => Some(bytes),
                        Err(RequestFailure::Budget(e)) => budget_refused_bytes(ctx, &armed, &e),
                        Err(RequestFailure::Defect(e)) => {
                            eprintln!("http-ingress: request folds failed, connection closed unanswered: {e}");
                            None
                        }
                    }
                }
            }
        };
        if let Some(bytes) = bytes {
            write_bytes(stream, &bytes);
        }
    }
}

fn read_refused_bytes(ctx: &InterpContext, reason: String) -> Option<Vec<u8>> {
    fold(
        ctx,
        "http_ingress_read_refused",
        vec![("reason", v1_interpreter::str_value(reason))],
    )
    .and_then(|r| render_response(ctx, r))
    .map_err(|e| {
        eprintln!("http-ingress: response folds failed, connection closed unanswered: {e}")
    })
    .ok()
}

/// The budget refusal, rendered after the request's scope has closed. The single armed contract
/// makes a refusal naming another entry unreachable; it is answered as a handler error rather than
/// rendered under the wrong subject.
fn budget_refused_bytes(
    ctx: &InterpContext,
    armed: &serve_budget_refusal::ServeArmedContract,
    e: &v1_interpreter::InterpError,
) -> Option<Vec<u8>> {
    let response = match serve_budget_refusal::serve_budget_refusal_from_exceeded(e) {
        Some(refusal) if serve_budget_refusal::serve_refusal_names_contract(&refusal, armed) => {
            eprintln!(
                "{}",
                serve_budget_refusal::serve_budget_refusal_diagnostic_line(&refusal)
            );
            fold(
                ctx,
                "http_ingress_budget_refused",
                vec![(
                    "machine_body",
                    v1_interpreter::str_value(
                        serve_budget_refusal::serve_budget_refusal_machine_body(&refusal),
                    ),
                )],
            )
        }
        _ => fold(
            ctx,
            "http_ingress_handler_error",
            vec![(
                "reason",
                v1_interpreter::str_value(
                    "a budget refusal named an entry other than the armed handler",
                ),
            )],
        ),
    };
    response
        .and_then(|r| render_response(ctx, r))
        .map_err(|e| {
            eprintln!("http-ingress: response folds failed, connection closed unanswered: {e}")
        })
        .ok()
}

/// The handler call, by the handler's MODULE-QUALIFIED identity (`gunbc.http_ingress`
/// `http_ingress_handler_identity`), so a binding naming one module's handler can never run another
/// module's declaration of the same bare name.
fn answer_with_handler(
    ctx: &InterpContext,
    binding: &Value,
    facts: &Value,
    handler: &str,
    head: Value,
    body: String,
    peer: &str,
) -> Result<Value, RequestFailure> {
    let arguments = eval(
        ctx,
        "http_ingress_arguments",
        vec![
            ("b", binding.clone()),
            ("facts", facts.clone()),
            ("head", head),
            ("body", v1_interpreter::str_value(&body)),
            ("peer_user", v1_interpreter::str_value(peer)),
        ],
    )?;
    let args = handler_args(ctx, &arguments)?;
    match v1_interpreter::run_in_context_with_args(ctx, handler, &args, true) {
        Ok(value) => match &value {
            Value::Record { type_name, .. } if ctx.sym_eq(*type_name, "ServeWireResponse") => {
                Ok(value)
            }
            other => eval(
                ctx,
                "http_ingress_not_a_response",
                vec![(
                    "rendered",
                    v1_interpreter::str_value(ctx.format_value(other)),
                )],
            ),
        },
        Err(e) if serve_budget_refusal::serve_budget_refusal_from_exceeded(&e).is_some() => {
            Err(RequestFailure::Budget(e))
        }
        Err(e) => eval(
            ctx,
            "http_ingress_handler_error",
            vec![("reason", v1_interpreter::str_value(e.to_string()))],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_socket_path(tag: &str) -> String {
        let dir =
            std::env::temp_dir().join(format!("gunbc-ingress-door-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("door.sock").to_string_lossy().into_owned()
    }

    // The kernel names the connecting account, and it is THIS process's account -- the value the
    // handler receives as peer_user, never anything the request carried.
    #[test]
    fn a_unix_listener_attests_the_connecting_account() {
        let path = temp_socket_path("peer");
        let listener = bind_unix(&path, 0o666).expect("bind");
        let client = std::thread::spawn({
            let path = path.clone();
            move || std::os::unix::net::UnixStream::connect(&path).expect("connect")
        });
        let (_conn, peer) = listener.accept().expect("accept");
        let _ = client.join();
        let uid = unsafe { libc::geteuid() };
        let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
        let mut buf = vec![0 as libc::c_char; 4096];
        let mut result: *mut libc::passwd = std::ptr::null_mut();
        unsafe { libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr(), buf.len(), &mut result) };
        assert!(!result.is_null(), "this process's own uid resolves");
        let me = unsafe { std::ffi::CStr::from_ptr(pwd.pw_name) }
            .to_string_lossy()
            .into_owned();
        assert_eq!(peer, me);
        assert_eq!(listener.bound_address().unwrap().0, format!("unix:{path}"));
    }

    // A stale socket from a previous process is replaced; any other file at the path refuses and
    // survives, so a mistyped listener never deletes data. The socket takes the binding's mode.
    #[test]
    fn a_unix_listener_replaces_only_a_socket_and_takes_the_binding_mode() {
        use std::os::unix::fs::PermissionsExt;
        let path = temp_socket_path("replace");
        drop(bind_unix(&path, 0o666).expect("first bind"));
        assert!(
            bind_unix(&path, 0o666).is_ok(),
            "a stale socket is replaced"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o666
        );
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, b"data").unwrap();
        assert!(bind_unix(&path, 0o666).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"data");
    }

    // A socket a process still listens on is live, not stale, even when that process never took the
    // path's lock: a start refuses and the listener keeps its name, so a connection by path still
    // reaches it (side-chat review 5785782995). A listener that took the lock is refused by the lock
    // first (a_held_socket_lock_refuses_a_second_start_and_leaves_the_path_untouched).
    #[test]
    fn a_live_unix_listener_is_never_displaced() {
        let path = temp_socket_path("live");
        let held = std::os::unix::net::UnixListener::bind(&path).expect("a live listener");
        let refused = bind_unix(&path, 0o666);
        assert!(
            matches!(&refused, Err(reason) if reason.contains("has a live listener")),
            "a live listener is refused, not displaced"
        );
        let _client = std::os::unix::net::UnixStream::connect(&path)
            .expect("the path still names a listener");
        assert!(
            held.accept().is_ok(),
            "the connection reached the original listener"
        );
        drop(held);
        std::fs::remove_file(&path).unwrap();
    }

    // A REALIZATION THAT HOLDS THE PATH'S LOCK OWNS THE PATH, even while the path still holds a stale
    // socket it has not yet replaced: a second start refuses and leaves that socket untouched, which is
    // the interleaving point at which an unlocked stale check let the second start unlink the first's
    // socket (side-chat review 5786866091).
    #[test]
    fn a_held_socket_lock_refuses_a_second_start_and_leaves_the_path_untouched() {
        use std::os::unix::fs::MetadataExt;
        let path = temp_socket_path("locked");
        drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
        let before = std::fs::metadata(&path).unwrap().ino();
        let held = lock_socket_path(&path).expect("the first realization takes the lock");
        let refused = bind_unix(&path, 0o666);
        assert!(
            matches!(&refused, Err(reason) if reason.contains("is held by another realization")),
            "a second start refuses while the lock is held"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().ino(),
            before,
            "the path was not touched"
        );
        drop(held);
        assert!(
            bind_unix(&path, 0o666).is_ok(),
            "once released, the stale socket is replaced"
        );
    }

    // TWO STARTS AT ONCE ON ONE STALE PATH: at most one binds, and the one that bound is still the
    // listener a connection by path reaches.
    #[test]
    fn concurrent_starts_on_one_stale_path_bind_at_most_once() {
        let path = temp_socket_path("race");
        for _ in 0..50 {
            drop(std::os::unix::net::UnixListener::bind(&path).ok());
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
            let starts: Vec<_> = (0..2)
                .map(|_| {
                    let (path, barrier) = (path.clone(), barrier.clone());
                    std::thread::spawn(move || {
                        barrier.wait();
                        bind_unix(&path, 0o666)
                    })
                })
                .collect();
            let results: Vec<_> = starts.into_iter().map(|s| s.join().unwrap()).collect();
            let bound: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
            assert!(bound.len() <= 1, "two starts both bound one path");
            if let Some(IngressListener::Unix(winner, _, _)) = bound.first() {
                let _client = std::os::unix::net::UnixStream::connect(&path)
                    .expect("the path names a listener");
                assert!(winner.accept().is_ok(), "the path still reaches the winner");
            }
            drop(results);
        }
    }

    // A CONSTRUCTOR THAT BUILDS ITS BINDING ITSELF with a socket mode no socket can carry is refused by
    // the graph's listener fold before anything touches the path: no socket and no lock file appear
    // (side-chat review 5786866091; the native start consumes the same fold).
    #[test]
    fn a_directly_built_mode_no_socket_can_carry_refuses_before_the_path_is_touched() {
        let root = crate::cli_run::workspace_root();
        let roots = vec![
            root.join("dag").display().to_string(),
            root.join("src/v2").display().to_string(),
        ];
        let entry = root
            .join("dag/test/fixture/serve_seam_direct_ingress.dag")
            .display()
            .to_string();
        let (graph, source_indices) =
            crate::cli_run::resolve_entry_graph(&roots, &entry).expect("the fixture resolves");
        let ctx = crate::cli_run::make_eval_context(
            graph.as_ref(),
            source_indices,
            v1_interpreter::ExecutionMode::Wet,
        );
        let path = temp_socket_path("direct");
        let _ = std::fs::remove_file(&path);
        let admission = v1_interpreter::run_in_context_with_args(
            &ctx,
            "serve_seam_direct_mode_ingress",
            &[(
                Some("socket_path".to_string()),
                v1_interpreter::str_value(&path),
            )],
            true,
        )
        .expect("the constructor answers");
        let IngressAnswer::Bound(binding) = ingress_answer(&admission, &ctx) else {
            panic!("the constructor answers a binding")
        };
        // A realization that does NOT refuse serves forever, so a regression must not hang the run:
        // past the deadline the watchdog ends the test binary loudly.
        let served = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let watched = served.clone();
        let watched_path = path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(60));
            if watched.load(std::sync::atomic::Ordering::SeqCst) {
                eprintln!("a_directly_built_mode_no_socket_can_carry_refuses_before_the_path_is_touched: the binding was SERVED at {watched_path} instead of refused");
                std::process::abort();
            }
        });
        let (status, reason) = realize(&ctx, binding);
        served.store(false, std::sync::atomic::Ordering::SeqCst);
        assert_ne!(status, 0);
        assert!(
            reason.contains("listener socket mode out of range: 4096"),
            "refused by the shared fold: {reason}"
        );
        assert!(!std::path::Path::new(&path).exists(), "no socket was bound");
        assert!(
            !std::path::Path::new(&format!("{path}.lock")).exists(),
            "the path was not locked either"
        );
    }

    // A TCP listener attests nothing: its peer is empty, and gunbc.http_ingress
    // http_ingress_fact_plan refuses a handler declaring peer_user behind one before it binds.
    #[test]
    fn a_tcp_listener_attests_no_peer() {
        let listener = bind_tcp("127.0.0.1", 0).expect("bind");
        let port = u16::try_from(listener.bound_address().unwrap().1).unwrap();
        let client = std::thread::spawn(move || {
            std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect")
        });
        let (_conn, peer) = listener.accept().expect("accept");
        let _ = client.join();
        assert_eq!(peer, "");
    }

    // An idle connection is not a request: no bytes before the timeout reads as nothing asked.
    #[test]
    fn a_line_read_distinguishes_nothing_asked_from_a_head_over_its_limit() {
        let mut idle = std::io::BufReader::new(&b""[..]);
        let mut budget = 10;
        assert_eq!(read_line(&mut idle, &mut budget), Ok(None));
        let mut long = std::io::BufReader::new(&b"GET /aaaaaaaaaaaaaaaa HTTP/1.1\r\n"[..]);
        let mut small = 8;
        assert!(read_line(&mut long, &mut small).is_err());
        let mut ok = std::io::BufReader::new(&b"GET / HTTP/1.1\r\n"[..]);
        let mut room = 64;
        assert_eq!(
            read_line(&mut ok, &mut room),
            Ok(Some("GET / HTTP/1.1".to_string()))
        );
        assert_eq!(room, 64 - 16);
    }
}

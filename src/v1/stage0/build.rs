use std::path::Path;
use std::time::Instant;

// THE CENSUS OF RECURSIVE VALUE WALKERS, KEYED ON A PARSE. `main` REFUSES THE BUILD when it finds one,
// so every lane that compiles v1-compiler -- the required lint step among them -- enforces it on every
// merge. It lives here rather than in src/ because every file under src/ is a rostered stage0 surface
// (v2.compiler.self_host.stage0_crate_layout), and a census needs no new seed module.
//
// A function that takes the interpreter's `Value` and calls itself recurses once per level of a
// value, and a value's depth is bounded by the heap, not by any call limit
// (gunbc.recurring_failure_mode recursion_over_value_depth_uncounted_by_the_call_limit). Every such
// self-call must sit INSIDE a closure handed to `value_depth_guarded`, so each level re-checks the
// host stack. The sources are parsed with syn, so comments and string literals cannot satisfy the
// check, and the guard must lexically wrap the recursive call rather than merely appear in the body.
// A file that does not parse refuses rather than being skipped. REACH, stated rather than implied:
// DIRECT self-recursion of free functions and methods (`name(..)`, `Self::name(..)`,
// `self.name(..)`) over a parameter whose type names `Value` outside serde_json. A mutually recursive
// pair, recursion through an operator (`==` reaching PartialEq), or a `Value` reached through a type
// alias is not seen; the Value trait impls (PartialEq, Display, Debug) are guarded by hand and Drop
// is iterative.

/// `path::fn` for every recursive Value walker under `root` whose self-call is not wrapped by the
/// guard, plus every source file that does not parse.
fn unguarded_recursive_value_walkers(root: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            findings.push(format!("{}: unreadable source directory", dir.display()));
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = match std::fs::read_to_string(&path) {
                    Ok(text) => text,
                    Err(e) => {
                        findings.push(format!("{}: unreadable source file: {e}", path.display()));
                        continue;
                    }
                };
                match judge_source(text) {
                    Ok(unguarded) => findings.extend(
                        unguarded
                            .into_iter()
                            .map(|name| format!("{}::{name}", path.display())),
                    ),
                    Err(e) => findings.push(format!("{}: does not parse: {e}", path.display())),
                }
            }
        }
    }
    findings.sort();
    findings
}

/// Host stack syn is granted per level of bracket nesting. syn's parser and visitor recurse once per
/// nested expression, and the generated emitter mirror nests ~2,000 deep: on the build script's
/// default 8 MiB main stack that overflowed, which puts the cost above 4 KiB per level in a debug
/// build script. 64 KiB is sixteen times that bound; the reservation is virtual, so pages a parse does
/// not touch are never committed.
const SYN_STACK_BYTES_PER_NESTING_LEVEL: usize = 64 * 1024;

/// Parses and judges one source file on a thread whose stack is sized from THAT file's measured
/// bracket nesting, rather than on a stack picked once for every input: the recursion's depth is a
/// property of the source, so its stack is derived from the source.
fn judge_source(text: String) -> Result<Vec<String>, String> {
    let stack = (bracket_nesting_depth(&text) + 64) * SYN_STACK_BYTES_PER_NESTING_LEVEL;
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            let file = syn::parse_file(&text).map_err(|e| e.to_string())?;
            let mut census = WalkerCensus::default();
            syn::visit::Visit::visit_file(&mut census, &file);
            Ok(census.unguarded)
        })
        .map_err(|e| format!("could not start the census thread: {e}"))?
        .join()
        .map_err(|_| "the census thread panicked".to_string())?
}

/// Deepest `(`/`[`/`{` nesting, counted iteratively. Brackets inside literals are counted too, which
/// only over-estimates the depth and so only enlarges the stack.
fn bracket_nesting_depth(text: &str) -> usize {
    let (mut depth, mut deepest) = (0usize, 0usize);
    for c in text.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

#[derive(Default)]
struct WalkerCensus {
    unguarded: Vec<String>,
}

impl<'ast> syn::visit::Visit<'ast> for WalkerCensus {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.judge(&item.sig, &item.block);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.judge(&item.sig, &item.block);
        syn::visit::visit_impl_item_fn(self, item);
    }
}

impl WalkerCensus {
    fn judge(&mut self, sig: &syn::Signature, body: &syn::Block) {
        let takes_value = sig.inputs.iter().any(|input| match input {
            syn::FnArg::Typed(pat) => type_names_interpreter_value(&pat.ty),
            syn::FnArg::Receiver(_) => false,
        });
        if !takes_value {
            return;
        }
        let mut calls = SelfCalls {
            name: sig.ident.to_string(),
            guard_depth: 0,
            unguarded: 0,
        };
        syn::visit::Visit::visit_block(&mut calls, body);
        if calls.unguarded > 0 {
            self.unguarded.push(sig.ident.to_string());
        }
    }
}

/// Whether a parameter type mentions a path ending in `Value` that is not serde_json's.
fn type_names_interpreter_value(ty: &syn::Type) -> bool {
    struct Finder(bool);
    impl<'ast> syn::visit::Visit<'ast> for Finder {
        fn visit_path(&mut self, path: &'ast syn::Path) {
            let is_value = path.segments.last().is_some_and(|s| s.ident == "Value");
            let is_serde_json = path.segments.iter().any(|s| s.ident == "serde_json");
            if is_value && !is_serde_json {
                self.0 = true;
            }
            syn::visit::visit_path(self, path);
        }
    }
    let mut finder = Finder(false);
    syn::visit::Visit::visit_type(&mut finder, ty);
    finder.0
}

/// Counts how many of a function's calls to itself are outside every closure handed to
/// `value_depth_guarded`.
struct SelfCalls {
    name: String,
    guard_depth: usize,
    unguarded: usize,
}

impl SelfCalls {
    fn record(&mut self) {
        if self.guard_depth == 0 {
            self.unguarded += 1;
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for SelfCalls {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let callee = match &*call.func {
            syn::Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        };
        if callee.as_deref() == Some(self.name.as_str()) {
            self.record();
        }
        if callee.as_deref() == Some("value_depth_guarded") {
            self.visit_expr(&call.func);
            self.guard_depth += 1;
            for arg in &call.args {
                self.visit_expr(arg);
            }
            self.guard_depth -= 1;
        } else {
            syn::visit::visit_expr_call(self, call);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method == self.name.as_str()
            && matches!(&*call.receiver, syn::Expr::Path(p) if p.path.is_ident("self"))
        {
            self.record();
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    // A nested fn item is judged on its own by WalkerCensus; its calls are not this body's.
    fn visit_item_fn(&mut self, _item: &'ast syn::ItemFn) {}
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("gunbc build: Cargo did not supply CARGO_MANIFEST_DIR to the build script");
    let started = Instant::now();
    let unguarded = unguarded_recursive_value_walkers(&Path::new(&manifest_dir).join("src"));
    let elapsed_ms = started.elapsed().as_millis();
    println!("cargo:warning=unguarded_recursive_value_walkers elapsed_ms={elapsed_ms}");
    assert!(
        unguarded.is_empty(),
        "gunbc build: recursive Value walkers that do not run under value_depth_guarded would abort \
         the process on a deep value (gunbc.recurring_failure_mode \
         recursion_over_value_depth_uncounted_by_the_call_limit): {unguarded:?}"
    );
}

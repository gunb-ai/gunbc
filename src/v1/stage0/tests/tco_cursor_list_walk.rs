//! The cursor realization of a self-tail-recursive list walk (gunbc#13883): `Cons { tail }` of a
//! parameter that is only ever walked becomes an index into the unchanged base list, and anything
//! that could observe the tail or the base keeps the `skip` realization. Each red below is a shape
//! the cursor must REFUSE; the positive arms are built with rustc and RUN, so a wrong emptiness
//! test past the end (depth 2 over an odd-length list) fails by execution, not by spelling.
//!
//! Run with cargo test -p v1-compiler --test tco_cursor_list_walk. Required CI does not execute
//! Rust integration tests (gunbc.rung_drop v2_native_route_off_the_merge_path); the required
//! `witnesses` steps emit, build and start two binaries and run no test, so this is on-demand
//! evidence, not merge-path coverage. The decision it pins is the seed emitter's
//! `v1.compiler.emit_rust` `tco_cursor_plan`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::diagnostic_to_message;

const SOURCE: &str = r#"module cursor_probe

import std.algebra { Cons, Empty, FreeMonoid }

fn walk_pos(xs: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: h, tail: t } => walk_pos(t, acc + h)
  }
}

fn walk_escape(xs: FreeMonoid<Int>, acc: Int) -> FreeMonoid<Int> {
  match xs {
    Empty => Empty
    Cons { head: h, tail: t } => if h == 0 { t } else { walk_escape(t, acc + h) }
  }
}

fn walk_xs_after(xs: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: h, tail: t } => walk_xs_after(t, acc + h + (xs |> count))
  }
}

fn walk_other_pos(xs: FreeMonoid<Int>, ys: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: h, tail: t } => walk_other_pos(ys, t, acc + h)
  }
}

fn dot_lockstep(xs: FreeMonoid<Int>, ys: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: a, tail: xt } =>
      match ys {
        Empty => acc
        Cons { head: b, tail: yt } => dot_lockstep(xt, yt, acc + a * b)
      }
  }
}

fn dot_reset(xs: FreeMonoid<Int>, ys: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: a, tail: xt } =>
      match ys {
        Empty => dot_reset(xt, Empty, acc + a)
        Cons { head: b, tail: yt } => dot_reset(xt, yt, acc + a * b)
      }
  }
}

fn walk_depth2(xs: FreeMonoid<Int>, acc: Int) -> Int {
  match xs {
    Empty => acc
    Cons { head: a, tail: Cons { head: b, tail: r } } => walk_depth2(r, acc + a + b)
    Cons { head: a, tail: Empty } => acc + a
  }
}
"#;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// Cargo leaves one rlib per build of the crate in deps/, and nothing names which one this test
// executable was linked against, so more than one is a refusal and not a pick: a guess could link the
// specimen against a build with other features and still compile. Remedy: remove the stale rlibs
// (`cargo clean -p v1-compiler`) or run this test without a separate `cargo build` of the crate.
fn unique_rlib(deps: &Path, name: &str) -> PathBuf {
    let prefix = format!("lib{name}-");
    let matches: Vec<_> = std::fs::read_dir(deps)
        .expect("read test dependency directory")
        .map(|entry| entry.expect("read dependency entry").path())
        .filter(|path| {
            let filename = path.file_name().unwrap().to_string_lossy();
            filename.starts_with(&prefix) && filename.ends_with(".rlib")
        })
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "ambiguous or absent {name} artifact: {matches:?}; remove the stale rlibs (cargo clean -p v1-compiler) and rerun"
    );
    matches[0].clone()
}

fn module_path(root: &Path, m: &str) -> PathBuf {
    let mut parts = m.split('.');
    let top = parts.next().unwrap();
    let mut p = if top == "v2" {
        root.join("src/v2")
    } else {
        root.join("dag").join(top)
    };
    for r in parts {
        p = p.join(r);
    }
    p.set_extension("dag");
    p
}

/// The import closure of `std.algebra`: the production definition of `List`/`Empty`/`Cons`, which a
/// one-file specimen cannot resolve.
fn algebra_closure(root: &Path) -> Vec<Rc<SourceFile>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut todo = vec!["std.algebra".to_string()];
    let mut out = Vec::new();
    while let Some(m) = todo.pop() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let path = module_path(root, &m);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("import ") {
                todo.push(
                    rest.chars()
                        .take_while(|c| !c.is_whitespace() && *c != '{')
                        .collect(),
                );
            }
        }
        out.push(Rc::new(SourceFile {
            path: path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            content,
        }));
    }
    out
}

fn fn_text<'a>(emitted: &'a str, name: &str) -> &'a str {
    let at = emitted
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("no fn {name} in emitted module"));
    let rest = &emitted[at..];
    let end = rest[1..]
        .find("\nfn ")
        .or_else(|| rest[1..].find("\npub fn "))
        .map(|e| e + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn cursor_is_taken_exactly_where_the_tail_is_unobservable() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut sources = algebra_closure(&repo);
    sources.push(Rc::new(SourceFile {
        path: "cursor_probe.dag".into(),
        content: SOURCE.into(),
    }));
    let result = compile_sources(Rc::new(sources.into_iter().collect()), RenderTarget::Rust);
    let modules: Vec<_> = result
        .files
        .iter()
        .filter(|file| file.path == "src/cursor_probe.rs")
        .collect();
    assert_eq!(
        modules.len(),
        1,
        "emission did not produce the specimen: {:?}",
        (
            result
                .files
                .iter()
                .map(|f| f.path.clone())
                .collect::<Vec<_>>(),
            result
                .diagnostics
                .iter()
                .map(|d| diagnostic_to_message(d.diagnostic.clone()))
                .filter(|m| !m.contains("std.algebra"))
                .take(8)
                .collect::<Vec<_>>()
        )
    );
    let emitted = &modules[0].content;
    for (name, cursor) in [
        ("walk_pos", true),
        ("walk_depth2", true),
        ("dot_lockstep", true),
        ("dot_reset", true),
        ("walk_escape", false),
        ("walk_xs_after", false),
        ("walk_other_pos", false),
    ] {
        let text = fn_text(emitted, name);
        assert_eq!(
            text.contains("__tco_cursor"),
            cursor,
            "{name}: cursor realization expected={cursor}\n{text}"
        );
        if !cursor {
            assert!(
                text.contains("skip("),
                "{name} must keep the skip realization\n{text}"
            );
        }
    }

    // Two lists walked in lockstep: each is a cursor, named by its own param, and neither pays `skip`.
    for name in ["dot_lockstep", "dot_reset"] {
        let text = fn_text(emitted, name);
        for cursor in ["__tco_cursor_xs", "__tco_cursor_ys"] {
            assert!(text.contains(cursor), "{name}: expected {cursor}\n{text}");
        }
        assert!(
            !text.contains("skip("),
            "{name}: a cursor walk must not skip\n{text}"
        );
    }

    let root = std::env::temp_dir().join(format!("gunbc-cursor-{}", std::process::id()));
    std::fs::create_dir(&root).expect("create isolated specimen directory");
    let scratch = Scratch(root);
    std::fs::write(scratch.0.join("cursor_probe.rs"), emitted).expect("retain emitted bytes");
    std::fs::write(
        scratch.0.join("main.rs"),
        r#"pub use v1_compiler::*;
mod cursor_probe;
fn main() {
    use cursor_probe::*;
    use std::rc::Rc;
    for n in 0..9i64 {
        let xs: im::Vector<i64> = (1..=n).collect();
        let want: i64 = xs.iter().sum();
        assert_eq!(walk_pos(Rc::new(xs.clone()), 0), want, "walk_pos n={n}");
        assert_eq!(walk_depth2(Rc::new(xs.clone()), 0), want, "walk_depth2 n={n}");
        assert_eq!(walk_xs_after(Rc::new(xs.clone()), 0), want + n * (n + 1) / 2, "walk_xs_after n={n}");
        let zs: im::Vector<i64> = (1..=n).collect();
        let squares: i64 = xs.iter().map(|x| x * x).sum();
        assert_eq!(dot_lockstep(Rc::new(xs.clone()), Rc::new(zs.clone()), 0), squares, "dot_lockstep n={n}");
        assert_eq!(dot_reset(Rc::new(xs.clone()), Rc::new(zs.clone()), 0), squares, "dot_reset n={n}");
        let short: im::Vector<i64> = (1..=n / 2).collect();
        let short_dot: i64 = (1..=n / 2).map(|x| x * x).sum::<i64>() + (n / 2 + 1..=n).sum::<i64>();
        assert_eq!(dot_reset(Rc::new(xs.clone()), Rc::new(short.clone()), 0), short_dot, "dot_reset short n={n}");
        assert_eq!(dot_lockstep(Rc::new(xs.clone()), Rc::new(short.clone()), 0), (1..=n / 2).map(|x| x * x).sum::<i64>(), "dot_lockstep short n={n}");
        let ys: im::Vector<i64> = im::Vector::new();
        let other = walk_other_pos(Rc::new(xs.clone()), Rc::new(ys), 0);
        assert!(other >= 0);
    }
    let with_zero: im::Vector<i64> = im::vector![3, 0, 5, 6];
    assert_eq!(*walk_escape(Rc::new(with_zero), 0), im::vector![5, 6]);
}
"#,
    )
    .expect("write harness around emitted module");
    let executable = std::env::current_exe().expect("locate test executable");
    let deps = executable.parent().expect("test dependency directory");
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(scratch.0.join("main.rs"))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!(
            "v1_compiler={}",
            unique_rlib(deps, "v1_compiler").display()
        ))
        .arg("--extern")
        .arg(format!("im={}", unique_rlib(deps, "im").display()))
        .arg("--extern")
        .arg(format!("serde={}", unique_rlib(deps, "serde").display()))
        .arg("-o")
        .arg(scratch.0.join("specimen"))
        .output()
        .expect("invoke rustc on emitted specimen");
    assert!(
        output.status.success(),
        "emitted Rust refused: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(scratch.0.join("specimen"))
        .output()
        .expect("execute compiled specimen");
    assert!(
        run.status.success(),
        "walk behavior failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}

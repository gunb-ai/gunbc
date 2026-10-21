use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use v1_compiled::extdeps_filesystem_filesystem_io::{Filesystem, FilesystemExactRead};
use v1_compiled::gunbc_source_root_read::{
    source_root_read, source_root_read_file, SourceRootFiles,
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "gunbc-filesystem-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn text(&self) -> String {
        self.0.to_str().unwrap().to_owned()
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn filesystem() -> Filesystem {
    Filesystem::new(Default::default())
}

#[tokio::test(flavor = "current_thread")]
async fn missing_read_is_a_typed_absence() {
    let scratch = Scratch::new();
    let path = scratch.0.join("missing.dag").to_str().unwrap().to_owned();
    let observed = source_root_read_file(path.clone(), &filesystem())
        .await
        .unwrap();
    assert!(
        matches!(&*observed, FilesystemExactRead::FilesystemExactPathAbsent { path: p } if p == &path)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn nested_sources_are_sorted_and_empty_files_are_present() {
    let scratch = Scratch::new();
    std::fs::create_dir(scratch.0.join("nested.dag")).unwrap();
    std::fs::write(scratch.0.join("z.dag"), "last").unwrap();
    std::fs::write(scratch.0.join("nested.dag/a.dag"), "").unwrap();
    std::fs::write(scratch.0.join("ignored.txt"), "ignored").unwrap();
    let observed = source_root_read(Rc::new(im::vector![scratch.text()]), &filesystem())
        .await
        .unwrap();
    match &*observed {
        SourceRootFiles::SourceRootFilesRead { files } => {
            assert_eq!(files.len(), 2);
            assert!(files[0].path.ends_with("nested.dag/a.dag"));
            assert_eq!(files[0].content, "");
            assert_eq!(files[1].content, "last");
        }
        refused => panic!("{refused:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn absent_root_and_file_as_root_refuse() {
    let scratch = Scratch::new();
    let file = scratch.0.join("file.dag");
    std::fs::write(&file, "").unwrap();
    for (path, expected) in [
        (scratch.0.join("absent"), "not_found"),
        (file, "not_a_directory"),
    ] {
        let path = path.to_str().unwrap().to_owned();
        let observed = source_root_read(Rc::new(im::vector![path.clone()]), &filesystem())
            .await
            .unwrap();
        assert!(
            matches!(&*observed, SourceRootFiles::SourceRootFilesRefused { path: p, kind, .. } if p == &path && kind == expected),
            "{observed:?}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn unreadable_text_refuses_without_publishing_partial_ingest() {
    let scratch = Scratch::new();
    std::fs::write(scratch.0.join("a.dag"), "valid").unwrap();
    std::fs::write(scratch.0.join("b.dag"), [0xff]).unwrap();
    let observed = source_root_read(Rc::new(im::vector![scratch.text()]), &filesystem())
        .await
        .unwrap();
    assert!(
        matches!(&*observed, SourceRootFiles::SourceRootFilesRefused { path, kind, cause } if path.ends_with("b.dag") && kind == "other" && !cause.is_empty()),
        "{observed:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn listing_encoding_refusal_is_not_an_empty_directory() {
    let scratch = Scratch::new();
    std::fs::write(scratch.0.join("line\nbreak.dag"), "").unwrap();
    let observed = source_root_read(Rc::new(im::vector![scratch.text()]), &filesystem())
        .await
        .unwrap();
    assert!(
        matches!(&*observed, SourceRootFiles::SourceRootFilesRefused { kind, cause, .. } if kind == "other" && cause.contains("line feed")),
        "{observed:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn tail_recursive_acquisition_handles_a_large_population() {
    let scratch = Scratch::new();
    for n in 0..4096 {
        std::fs::write(scratch.0.join(format!("{n:04}.dag")), n.to_string()).unwrap();
    }
    let observed = source_root_read(Rc::new(im::vector![scratch.text()]), &filesystem())
        .await
        .unwrap();
    assert!(
        matches!(&*observed, SourceRootFiles::SourceRootFilesRead { files } if files.len() == 4096 && files[4095].content == "4095")
    );
}

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("pristine-cli-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        git(&path, &["init", "-q"]);
        git(&path, &["config", "user.email", "test@example.invalid"]);
        git(&path, &["config", "user.name", "CLI test"]);
        fs::write(path.join("seed"), "baseline\n").unwrap();
        git(&path, &["add", "seed"]);
        git(&path, &["commit", "-qm", "baseline"]);
        Self(path)
    }

    fn add(&self, name: &str, contents: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
        git(&self.0, &["add", "--", name]);
        git(&self.0, &["commit", "-qm", "change file"]);
    }

    fn delete(&self, name: &str) {
        git(&self.0, &["rm", "-q", "--", name]);
        git(&self.0, &["commit", "-qm", "delete file"]);
    }

    fn base(&self) -> String {
        git_output(&self.0, &["rev-parse", "HEAD"])
    }

    fn check(&self, base: &str, title: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_pristine"))
            .current_dir(&self.0)
            .args(["check", "--base", base, "--title", title])
            .output()
            .unwrap()
    }

    fn state(&self) -> String {
        format!(
            "{}\n{}",
            self.base(),
            git_output(
                &self.0,
                &["status", "--porcelain=v1", "--untracked-files=all"]
            )
        )
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

#[test]
fn docs_only_and_empty_diffs_are_clean() {
    let repo = Repo::new();
    let base = repo.base();
    repo.add("docs/guide.MD", "guide\n");
    assert_eq!(
        repo.check(&base, "docs: update guide").status.code(),
        Some(0)
    );
    assert_eq!(repo.check(&base, "docs: update guide").stdout, b"");
    let empty = Repo::new();
    assert_eq!(
        empty
            .check(&empty.base(), "docs: nothing changed")
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn documentation_directories_and_root_readmes_are_clean() {
    for path in [
        "docs/guide.rst",
        "documentation/manual.adoc",
        "README-extra.MD",
    ] {
        let repo = Repo::new();
        let base = repo.base();
        repo.add(path, "content\n");
        let result = repo.check(&base, "docs: update documentation");
        assert_eq!(
            result.status.code(),
            Some(0),
            "{path}: {}",
            text(&result.stderr)
        );
        assert_eq!(result.stdout, b"");
    }
}

#[test]
fn mixed_paths_report_rule_and_every_source_path() {
    let repo = Repo::new();
    let base = repo.base();
    repo.add("src/main.rs", "main\n");
    repo.add("build file.rs", "build\n");
    repo.add("docs/guide.md", "guide\n");
    let result = repo.check(&base, "docs: update docs");
    assert_eq!(result.status.code(), Some(1));
    let out = text(&result.stdout);
    assert!(out.contains("src/main.rs"));
    assert!(out.contains("build file.rs"));
    assert!(out.to_lowercase().contains("docs:"));
    assert!(!out.contains("docs/guide.md"));
    assert!(result.stderr.is_empty());
}

#[test]
fn nonmatching_prefixes_are_unsupported_not_scope_judgments() {
    let repo = Repo::new();
    let base = repo.base();
    repo.add("src/main.rs", "main\n");
    for title in ["docs-update: update", "feat: add feature"] {
        let result = repo.check(&base, title);
        assert_eq!(result.status.code(), Some(0));
        assert!(text(&result.stdout).to_lowercase().contains("not applied"));
        assert!(!text(&result.stdout).contains("src/main.rs"));
    }
}

#[test]
fn outside_repository_and_invalid_base_are_operational_errors() {
    let outside = std::env::temp_dir().join(format!("pristine-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_pristine"))
        .current_dir(&outside)
        .args(["check", "--base", "HEAD", "--title", "docs: test"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!result.stderr.is_empty());
    fs::remove_dir_all(&outside).unwrap();

    let repo = Repo::new();
    let result = repo.check("no-such-ref", "docs: test");
    assert_eq!(result.status.code(), Some(2));
    assert!(!result.stderr.is_empty());
    assert!(text(&result.stderr).to_lowercase().contains("base"));
}

#[test]
fn check_does_not_mutate_head_index_or_worktree() {
    let repo = Repo::new();
    let base = repo.base();
    repo.add("src/main.rs", "source\n");
    let before = repo.state();
    let _ = repo.check(&base, "docs: check");
    assert_eq!(repo.state(), before);
}

#[test]
fn unicode_and_space_paths_are_preserved() {
    let repo = Repo::new();
    let base = repo.base();
    repo.add("src/naïve file.rs", "source\n");
    let result = repo.check(&base, "docs: test");
    assert_eq!(result.status.code(), Some(1));
    assert!(text(&result.stdout).contains("src/naïve file.rs"));
}

#[test]
fn deleted_source_path_is_reported() {
    let repo = Repo::new();
    repo.add("src/old.rs", "source\n");
    let base = repo.base();
    repo.delete("src/old.rs");
    let result = repo.check(&base, "docs: remove source");
    assert_eq!(result.status.code(), Some(1));
    assert!(text(&result.stdout).contains("src/old.rs"));
}

#[test]
fn help_is_successful() {
    let result = Command::new(env!("CARGO_BIN_EXE_pristine"))
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert!(text(&result.stdout).contains("pristine check"));
}

use std::env;
use std::process::{self, Command};

fn fail(message: impl AsRef<str>) -> ! {
    eprintln!("pristine: {}", message.as_ref());
    process::exit(2);
}

fn git(args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(output.stdout)
}

fn is_documentation(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.starts_with("docs/") || lower.starts_with("documentation/") {
        return true;
    }
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    if [
        "readme",
        "changelog",
        "contributing",
        "code_of_conduct",
        "security",
        "license",
    ]
    .iter()
    .any(|family| basename.starts_with(family))
    {
        return true;
    }
    [".md", ".mdx", ".rst", ".adoc"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        println!("Usage: pristine check --base <ref> --title <title>\n       pristine --help");
        return Ok(());
    }
    if args.first().map(String::as_str) != Some("check") {
        return Err("usage: pristine check --base <ref> --title <title>".into());
    }
    let mut base = None;
    let mut title = None;
    let mut i = 1;
    while i < args.len() {
        let dest = match args[i].as_str() {
            "--base" => &mut base,
            "--title" => &mut title,
            _ => return Err(format!("unknown argument: {}", args[i])),
        };
        i += 1;
        if i == args.len() || args[i].starts_with("--") {
            return Err("option requires a value".into());
        }
        if dest.replace(args[i].clone()).is_some() {
            return Err("option specified more than once".into());
        }
        i += 1;
    }
    let base = base.ok_or("missing --base <ref>")?;
    let title = title.ok_or("missing --title <title>")?;
    git(&["rev-parse", "--show-toplevel"]).map_err(|e| format!("Git inspection failed: {e}"))?;
    let base = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        "--end-of-options",
        &format!("{base}^{{commit}}"),
    ])
    .map_err(|_| format!("base ref is not a valid commit: {base}"))?;
    let base = std::str::from_utf8(&base)
        .map_err(|_| "Git returned an invalid base commit")?
        .trim();
    let paths = git(&[
        "diff",
        "--no-renames",
        "--name-only",
        "-z",
        &format!("{base}...HEAD"),
    ])
    .map_err(|e| format!("Git diff failed: {e}"))?;
    let paths = paths
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| std::str::from_utf8(path).map_err(|_| "Git path is not valid UTF-8"))
        .collect::<Result<Vec<_>, _>>()?;
    if !title.starts_with("docs:") {
        println!("Rule not applied: title must begin with docs:.");
        return Ok(());
    }
    let findings: Vec<_> = paths
        .into_iter()
        .filter(|path| !is_documentation(path))
        .collect();
    if !findings.is_empty() {
        println!(
            "docs: titles should change documentation paths only; non-documentation paths found:"
        );
        for path in findings {
            println!("  {path}");
        }
        process::exit(1);
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        fail(error);
    }
}

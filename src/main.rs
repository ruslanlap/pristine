use std::env;
use std::fs;
use std::path::Path;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Github,
    Agent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopePrefix {
    Docs,
    Ci,
    Test,
    Style,
}

impl ScopePrefix {
    fn from_title(title: &str) -> Option<Self> {
        let lower = title.trim_start().to_ascii_lowercase();
        for (name, variant) in [
            ("docs", Self::Docs),
            ("ci", Self::Ci),
            ("test", Self::Test),
            ("style", Self::Style),
        ] {
            if let Some(rest) = lower.strip_prefix(name) {
                if rest.starts_with(':') {
                    return Some(variant);
                }
                if let Some(after_paren) = rest.strip_prefix('(') {
                    if let Some((_, after_close)) = after_paren.split_once(')') {
                        if after_close.starts_with(':') || after_close.starts_with("!:") {
                            return Some(variant);
                        }
                    }
                }
                if rest.starts_with("!:") {
                    return Some(variant);
                }
            }
        }
        None
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Docs => "docs:",
            Self::Ci => "ci:",
            Self::Test => "test:",
            Self::Style => "style:",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Docs => "documentation",
            Self::Ci => "CI/CD configuration",
            Self::Test => "test",
            Self::Style => "style/formatting",
        }
    }
}

#[derive(Default)]
struct CustomConfig {
    docs: Vec<String>,
    ci: Vec<String>,
    test: Vec<String>,
    style: Vec<String>,
    allow_doc_configs: Option<bool>,
}

impl CustomConfig {
    fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read config {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, String> {
        let mut cfg = Self::default();
        let mut current_section = String::new();
        for (line_idx, raw_line) in content.lines().enumerate() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                if key == "allow_doc_configs" {
                    match val {
                        "true" => cfg.allow_doc_configs = Some(true),
                        "false" => cfg.allow_doc_configs = Some(false),
                        _ => {
                            return Err(format!(
                                "invalid boolean at line {}: {val}",
                                line_idx + 1
                            ))
                        }
                    }
                    continue;
                }
                let target_key = if current_section.is_empty() {
                    key
                } else if key == "allow" || key == "paths" {
                    current_section.as_str()
                } else {
                    key
                };
                let list = parse_toml_string_array(val).ok_or_else(|| {
                    format!("invalid array syntax at line {}: {val}", line_idx + 1)
                })?;
                match target_key {
                    "docs" => cfg.docs.extend(list),
                    "ci" => cfg.ci.extend(list),
                    "test" => cfg.test.extend(list),
                    "style" => cfg.style.extend(list),
                    _ => {}
                }
            }
        }
        Ok(cfg)
    }

    fn patterns_for(&self, prefix: ScopePrefix) -> &[String] {
        match prefix {
            ScopePrefix::Docs => &self.docs,
            ScopePrefix::Ci => &self.ci,
            ScopePrefix::Test => &self.test,
            ScopePrefix::Style => &self.style,
        }
    }
}

fn parse_toml_string_array(val: &str) -> Option<Vec<String>> {
    let val = val.trim();
    if !val.starts_with('[') || !val.ends_with(']') {
        return None;
    }
    let inner = val[1..val.len() - 1].trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut items = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if (part.starts_with('"') && part.ends_with('"'))
            || (part.starts_with('\'') && part.ends_with('\''))
        {
            items.push(part[1..part.len() - 1].to_owned());
        } else {
            return None;
        }
    }
    Some(items)
}

fn matches_custom_pattern(path: &str, pattern: &str) -> bool {
    let path_lower = path.to_ascii_lowercase();
    let pat_lower = pattern.to_ascii_lowercase();
    if let Some(dir_prefix) = pat_lower.strip_suffix("/**") {
        return path_lower.starts_with(&format!("{dir_prefix}/")) || path_lower == dir_prefix;
    }
    if let Some(dir_prefix) = pat_lower.strip_suffix("/*") {
        if let Some(rest) = path_lower.strip_prefix(&format!("{dir_prefix}/")) {
            return !rest.contains('/');
        }
        return false;
    }
    if let Some(suffix) = pat_lower.strip_prefix('*') {
        return path_lower.ends_with(suffix);
    }
    if pat_lower.ends_with('/') {
        return path_lower.starts_with(&pat_lower);
    }
    path_lower == pat_lower
}

fn is_doc_config(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        basename,
        ".markdownlint.json"
            | ".markdownlint.yaml"
            | ".markdownlint.yml"
            | ".markdownlint-cli2.jsonc"
            | ".markdownlint-cli2.yaml"
            | ".markdownlintignore"
            | ".cspell.json"
            | "cspell.json"
            | ".cspell.yaml"
            | ".vale.ini"
            | "mkdocs.yml"
            | "docusaurus.config.js"
            | "docusaurus.config.ts"
            | "book.toml"
            | ".lychee.toml"
            | "lychee.toml"
    )
}

fn is_documentation(path: &str, allow_doc_configs: bool) -> bool {
    let lower = path.to_ascii_lowercase();
    if allow_doc_configs && is_doc_config(path) {
        return true;
    }
    if lower.starts_with("docs/") || lower.starts_with("documentation/") {
        return true;
    }
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    if !lower.contains('/')
        && [
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

fn is_ci_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.starts_with(".github/")
        || lower.starts_with(".circleci/")
        || lower.starts_with(".buildkite/")
    {
        return true;
    }
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    matches!(
        basename,
        ".gitlab-ci.yml"
            | "jenkinsfile"
            | ".travis.yml"
            | "azure-pipelines.yml"
            | "action.yml"
            | "action.yaml"
            | "dockerfile"
            | "docker-compose.yml"
            | "docker-compose.yaml"
            | "justfile"
            | "makefile"
    ) || basename.starts_with("dockerfile.")
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    if lower.starts_with("tests/")
        || lower.starts_with("test/")
        || lower.starts_with("__tests__/")
        || lower.starts_with("e2e/")
        || lower.starts_with("benches/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
    {
        return true;
    }
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    basename.starts_with("test_")
        || [
            "_test.rs",
            "_tests.rs",
            "_test.go",
            "_test.py",
            ".test.ts",
            ".test.tsx",
            ".test.js",
            ".test.jsx",
            ".spec.ts",
            ".spec.tsx",
            ".spec.js",
            ".spec.jsx",
            ".snap",
        ]
        .iter()
        .any(|suffix| basename.ends_with(suffix))
}

fn is_style_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(&lower);
    if [
        ".css", ".scss", ".sass", ".less", ".svg", ".ico", ".png", ".jpg", ".jpeg", ".webp",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
    {
        return true;
    }
    matches!(
        basename,
        ".prettierrc"
            | ".prettierrc.json"
            | ".prettierrc.yml"
            | ".prettierrc.yaml"
            | ".prettierrc.js"
            | "prettier.config.js"
            | ".editorconfig"
            | "rustfmt.toml"
            | ".rustfmt.toml"
            | ".clang-format"
            | "biome.json"
            | "biome.jsonc"
    )
}

fn is_allowed_path(
    prefix: ScopePrefix,
    path: &str,
    allow_doc_configs: bool,
    custom_cfg: &CustomConfig,
) -> bool {
    if custom_cfg
        .patterns_for(prefix)
        .iter()
        .any(|pat| matches_custom_pattern(path, pat))
    {
        return true;
    }
    match prefix {
        ScopePrefix::Docs => is_documentation(path, allow_doc_configs),
        ScopePrefix::Ci => is_ci_path(path),
        ScopePrefix::Test => is_test_path(path),
        ScopePrefix::Style => is_style_path(path),
    }
}

struct ChangedEntry {
    path: String,
    is_symlink_or_submodule: bool,
}

fn parse_raw_diff(raw: &[u8]) -> Result<Vec<ChangedEntry>, String> {
    let mut entries = Vec::new();
    let mut tokens = raw.split(|b| *b == 0).filter(|t| !t.is_empty());
    while let Some(meta) = tokens.next() {
        let meta_str =
            std::str::from_utf8(meta).map_err(|_| "Git diff metadata is not valid UTF-8")?;
        let path_bytes = tokens
            .next()
            .ok_or("Unexpected end of git diff --raw output")?;
        let path = std::str::from_utf8(path_bytes)
            .map_err(|_| "Git path is not valid UTF-8")?
            .to_owned();
        // Format of metadata: ":<old_mode> <new_mode> <old_sha> <new_sha> <status>"
        let parts: Vec<&str> = meta_str.trim_start_matches(':').split_whitespace().collect();
        let mut is_symlink_or_submodule = false;
        if parts.len() >= 2 {
            let old_mode = parts[0];
            let new_mode = parts[1];
            if old_mode == "120000"
                || new_mode == "120000"
                || old_mode == "160000"
                || new_mode == "160000"
            {
                is_symlink_or_submodule = true;
            }
        }
        entries.push(ChangedEntry {
            path,
            is_symlink_or_submodule,
        });
    }
    Ok(entries)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

fn is_shallow_repository() -> bool {
    git(&["rev-parse", "--is-shallow-repository"])
        .ok()
        .and_then(|out| String::from_utf8(out).ok())
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}

fn print_help() {
    println!(
        "Usage: pristine check --base <ref> --title <title> [options]\n\
         \x20      pristine --help | --version\n\n\
         Options:\n\
         \x20 --base <ref>            Base Git commit or branch reference\n\
         \x20 --title <title>         Pull request or commit title (e.g. \"docs: update README\")\n\
         \x20 --format <fmt>          Output format: text (default), json, github, agent\n\
         \x20 --agent                 Shorthand for --format agent (structured AI remediation prompt)\n\
         \x20 --allow-doc-configs     Allow documentation tool configs (.markdownlint.json, .cspell.json, etc.)\n\
         \x20 --config <path>         Path to pristine.toml configuration file"
    );
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        print_help();
        return Ok(());
    }
    if args == ["--version"] || args == ["-V"] {
        println!("pristine {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.first().map(String::as_str) != Some("check") {
        return Err("usage: pristine check --base <ref> --title <title>".into());
    }
    let mut base = None;
    let mut title = None;
    let mut format = None;
    let mut config_path = None;
    let mut allow_doc_configs = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--allow-doc-configs" => {
                allow_doc_configs = true;
                i += 1;
                continue;
            }
            "--agent" | "--agent-friendly" => {
                if format.replace(OutputFormat::Agent).is_some() {
                    return Err("output format specified more than once".into());
                }
                i += 1;
                continue;
            }
            "--format" => {
                i += 1;
                if i == args.len() || args[i].starts_with("--") {
                    return Err("option requires a value".into());
                }
                let parsed = match args[i].as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "github" => OutputFormat::Github,
                    "agent" => OutputFormat::Agent,
                    other => {
                        return Err(format!(
                            "unknown format: {other} (expected text, json, github, or agent)"
                        ))
                    }
                };
                if format.replace(parsed).is_some() {
                    return Err("option specified more than once".into());
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        let dest = match args[i].as_str() {
            "--base" => &mut base,
            "--title" => &mut title,
            "--config" => &mut config_path,
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
    let base_arg = base.ok_or("missing --base <ref>")?;
    let title = title.ok_or("missing --title <title>")?;
    let format = format.unwrap_or(OutputFormat::Text);

    let toplevel_bytes = git(&["rev-parse", "--show-toplevel"])
        .map_err(|e| format!("Git inspection failed: {e}"))?;
    let toplevel = std::str::from_utf8(&toplevel_bytes)
        .map_err(|_| "Git repository path is not valid UTF-8")?
        .trim();

    let custom_cfg = if let Some(ref p) = config_path {
        CustomConfig::load(Path::new(p))?
    } else {
        let default_toml = Path::new(toplevel).join("pristine.toml");
        if default_toml.is_file() {
            CustomConfig::load(&default_toml)?
        } else {
            CustomConfig::default()
        }
    };
    if let Some(cfg_allow) = custom_cfg.allow_doc_configs {
        allow_doc_configs = allow_doc_configs || cfg_allow;
    }

    let base_resolved = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        "--end-of-options",
        &format!("{base_arg}^{{commit}}"),
    ])
    .map_err(|_| {
        if is_shallow_repository() {
            format!(
                "base ref is not a valid commit: {base_arg} (shallow clone detected; Hint: set fetch-depth: 0 in actions/checkout or fetch the base branch first)"
            )
        } else {
            format!("base ref is not a valid commit: {base_arg}")
        }
    })?;
    let base_sha = std::str::from_utf8(&base_resolved)
        .map_err(|_| "Git returned an invalid base commit")?
        .trim();

    let raw_diff = git(&[
        "diff",
        "--raw",
        "--no-renames",
        "-z",
        &format!("{base_sha}...HEAD"),
    ])
    .map_err(|e| {
        if is_shallow_repository()
            && (e.contains("no merge base") || e.contains("Invalid symmetric difference"))
        {
            format!(
                "Git diff failed: {e} (shallow clone detected; Hint: set fetch-depth: 0 in actions/checkout so Git can compute the merge-base)"
            )
        } else {
            format!("Git diff failed: {e}")
        }
    })?;

    let entries = parse_raw_diff(&raw_diff)?;
    let Some(scope) = ScopePrefix::from_title(&title) else {
        match format {
            OutputFormat::Json => {
                println!(
                    "{{\"status\":\"skipped\",\"title\":\"{}\",\"findings\":[]}}",
                    json_escape(&title)
                );
            }
            OutputFormat::Github | OutputFormat::Agent => {}
            OutputFormat::Text => {
                println!("Rule not applied: title must begin with docs: (or ci:, test:, style:).");
            }
        }
        return Ok(());
    };

    let findings: Vec<String> = entries
        .into_iter()
        .filter(|entry| {
            if entry.is_symlink_or_submodule {
                return true;
            }
            !is_allowed_path(scope, &entry.path, allow_doc_configs, &custom_cfg)
        })
        .map(|entry| entry.path)
        .collect();

    if !findings.is_empty() {
        match format {
            OutputFormat::Text => {
                println!(
                    "{} titles should change {} paths only; non-{} paths found:",
                    scope.as_str(),
                    scope.label(),
                    scope.label()
                );
                for path in &findings {
                    println!("  {path}");
                }
            }
            OutputFormat::Json => {
                let items: Vec<String> = findings
                    .iter()
                    .map(|p| format!("\"{}\"", json_escape(p)))
                    .collect();
                println!(
                    "{{\"status\":\"violation\",\"scope\":\"{}\",\"title\":\"{}\",\"findings\":[{}]}}",
                    scope.as_str(),
                    json_escape(&title),
                    items.join(",")
                );
            }
            OutputFormat::Github => {
                for path in &findings {
                    println!(
                        "::error file={},title=Pristine Scope Violation::Non-{} file '{}' modified in a '{}' PR.",
                        path,
                        scope.label(),
                        path,
                        scope.as_str()
                    );
                }
            }
            OutputFormat::Agent => {
                let list = findings.join(", ");
                println!(
                    "[PRISTINE VIOLATION] Branch/PR intent is '{}', but non-{} path(s) [{}] were modified. Action required: Revert [{}] or change the PR/commit prefix to 'feat:' / 'fix:' / 'chore:'.",
                    scope.as_str(),
                    scope.label(),
                    list,
                    list
                );
            }
        }
        process::exit(1);
    }

    if format == OutputFormat::Json {
        println!(
            "{{\"status\":\"clean\",\"scope\":\"{}\",\"title\":\"{}\",\"findings\":[]}}",
            scope.as_str(),
            json_escape(&title)
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        fail(error);
    }
}

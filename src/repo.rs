use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::models::{TemplateCategory, TemplateInfo};

const GITHUB_GITIGNORE_REPO: &str = "https://github.com/github/gitignore.git";

/// Manager for the local cache/storage of github/gitignore templates repository
#[derive(Debug, Clone)]
pub struct GitignoreRepo {
    templates_dir: PathBuf,
}

impl GitignoreRepo {
    /// Initialize with standard templates path (~/.config/ign/templates or $IGN_TEMPLATES_DIR)
    pub fn new() -> Result<Self> {
        let templates_dir = if let Ok(custom) = std::env::var("IGN_TEMPLATES_DIR") {
            PathBuf::from(custom)
        } else if let Ok(custom) = std::env::var("IGN_CACHE_DIR") {
            PathBuf::from(custom)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("ign").join("templates")
        } else if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.config_dir().join("ign").join("templates")
        } else {
            PathBuf::from(".ign").join("templates")
        };
        Ok(Self { templates_dir })
    }

    /// Initialize with a custom templates directory (useful for testing)
    pub fn with_templates_dir<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            templates_dir: path.into(),
        }
    }

    /// Backwards compatibility constructor alias for tests and external callers
    pub fn with_cache_dir<P: Into<PathBuf>>(path: P) -> Self {
        Self::with_templates_dir(path)
    }

    /// Return the path to the local repository templates directory
    pub fn templates_dir(&self) -> &Path {
        &self.templates_dir
    }

    /// Backwards compatibility alias for templates_dir
    pub fn cache_dir(&self) -> &Path {
        &self.templates_dir
    }

    /// Check if the local templates repository exists, contains a git repository, and has gitignore templates
    pub fn is_cached(&self) -> bool {
        if !self.templates_dir.is_dir() || !self.templates_dir.join(".git").exists() {
            return false;
        }

        WalkDir::new(&self.templates_dir)
            .into_iter()
            .filter_entry(|e| !is_hidden_or_git(e))
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_type().is_file()
                    && e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.ends_with(".gitignore"))
                        .unwrap_or(false)
            })
    }

    /// Ensure the repository cache is ready.
    /// If not cached/initialized, returns a friendly error directing the user to initialize.
    pub fn ensure_cache_ready(&self) -> Result<()> {
        if !self.is_cached() {
            return Err(anyhow!(
                "Template library is not initialized or empty at '{}'.\nPlease run `ign template init` (or `ign update`) to clone templates from GitHub.",
                self.templates_dir.display()
            ));
        }

        Ok(())
    }

    /// Explicitly initialize the local templates repository by cloning from GitHub
    pub fn init_repo(&self) -> Result<()> {
        Self::check_git_available()?;

        if let Some(parent) = self.templates_dir.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directory for templates: {:?}",
                    parent
                )
            })?;
        }

        // Clean up any incomplete directory if .git doesn't exist
        if self.templates_dir.exists() && !self.templates_dir.join(".git").exists() {
            let _ = fs::remove_dir_all(&self.templates_dir);
        }

        println!(
            "Fetching template library from {}...",
            GITHUB_GITIGNORE_REPO
        );

        let output = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg(GITHUB_GITIGNORE_REPO)
            .arg(&self.templates_dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .with_context(|| {
                "Failed to execute `git clone` command. Please verify `git` is installed and available in PATH."
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "Failed to clone GitHub gitignore repository (exit code: {:?}):\n{}\n\nPlease check your network connection or configure a proxy if needed.",
                output.status.code(),
                stderr.trim()
            ));
        }

        println!(
            "Successfully initialized gitignore template library at {:?}",
            self.templates_dir
        );
        Ok(())
    }

    /// Backwards-compatible alias for init_repo
    pub fn clone_repo(&self) -> Result<()> {
        self.init_repo()
    }

    /// Manually update the local templates repository via git pull / fetch & reset.
    /// If uninitialized, prompts and clones the repository.
    pub fn update_repo(&self) -> Result<()> {
        Self::check_git_available()?;

        if !self.is_cached() {
            println!("Template repository is not initialized. Initializing template library...");
            return self.init_repo();
        }

        println!(
            "Updating gitignore template library at {:?}...",
            self.templates_dir
        );

        // Try `git pull --ff-only` first
        let pull_output = Command::new("git")
            .arg("-C")
            .arg(&self.templates_dir)
            .args(["pull", "--ff-only"])
            .output()
            .with_context(|| "Failed to execute `git pull` in templates directory")?;

        if pull_output.status.success() {
            println!("Gitignore template library is up to date.");
            return Ok(());
        }

        // Fallback: fetch shallow and reset to origin's default branch
        println!("Standard pull failed, resetting to latest remote HEAD...");
        let fetch_output = Command::new("git")
            .arg("-C")
            .arg(&self.templates_dir)
            .args(["fetch", "--depth=1", "origin"])
            .output()
            .with_context(|| "Failed to execute `git fetch` in templates directory")?;

        if !fetch_output.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_output.stderr);
            return Err(anyhow!(
                "Failed to fetch updates from GitHub gitignore repository:\n{}",
                stderr.trim()
            ));
        }

        let reset_output = Command::new("git")
            .arg("-C")
            .arg(&self.templates_dir)
            .args(["reset", "--hard", "origin/HEAD"])
            .output()
            .with_context(|| "Failed to execute `git reset` in templates directory")?;

        if !reset_output.status.success() {
            // Try reset to origin/main or origin/master
            let _ = Command::new("git")
                .arg("-C")
                .arg(&self.templates_dir)
                .args(["reset", "--hard", "origin/main"])
                .output();
        }

        println!("Successfully updated gitignore templates to latest version.");
        Ok(())
    }

    /// Backwards-compatible alias for update_repo
    pub fn update(&self) -> Result<()> {
        self.update_repo()
    }

    /// List all available templates in the repository
    pub fn list_templates(&self) -> Result<Vec<TemplateInfo>> {
        self.ensure_cache_ready()?;

        let mut templates = Vec::new();

        for entry in WalkDir::new(&self.templates_dir)
            .into_iter()
            .filter_entry(|e| !is_hidden_or_git(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".gitignore") {
                        if let Ok(rel_path) = path.strip_prefix(&self.templates_dir) {
                            let rel_str = rel_path.to_string_lossy().to_string();
                            let clean_name = extract_clean_name(&rel_str);
                            let category = TemplateCategory::from_relative_path(&rel_str);

                            templates.push(TemplateInfo {
                                name: clean_name,
                                path: path.to_path_buf(),
                                category,
                                relative_path: rel_str,
                            });
                        }
                    }
                }
            }
        }

        // Sort alphabetically by category, then by name
        templates.sort_by(|a, b| {
            let cat_cmp = (a.category as u8).cmp(&(b.category as u8));
            if cat_cmp == std::cmp::Ordering::Equal {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else {
                cat_cmp
            }
        });

        Ok(templates)
    }

    /// Fuzzy search templates matching a query
    pub fn search_templates(&self, query: &str) -> Result<Vec<(TemplateInfo, i64)>> {
        let all_templates = self.list_templates()?;
        let matcher = SkimMatcherV2::default();
        let query_normalized = normalize_string(query);

        let mut matched: Vec<(TemplateInfo, i64)> = all_templates
            .into_iter()
            .filter_map(|t| {
                // Check if query matches alias directly
                if let Some(target) = resolve_smart_alias(query) {
                    if normalize_string(&t.name) == normalize_string(target) {
                        return Some((t, 1000));
                    }
                }

                // Check exact normalized match
                let t_norm = normalize_string(&t.name);
                if t_norm == query_normalized {
                    return Some((t, 900));
                }

                // Fuzzy match against full name
                if let Some(score) = matcher.fuzzy_match(&t.name, query) {
                    return Some((t, score));
                }

                // Fuzzy match against last segment (e.g. "macOS" in "Global/macOS")
                if let Some(last_seg) = t.name.split('/').last() {
                    if let Some(score) = matcher.fuzzy_match(last_seg, query) {
                        return Some((t, score));
                    }
                }

                None
            })
            .collect();

        matched.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(matched)
    }

    /// Find template info by query, supporting case-insensitivity and smart aliases
    pub fn find_template(&self, query: &str) -> Result<TemplateInfo> {
        let all = self.list_templates()?;
        let query_norm = normalize_string(query);

        // 1. Check smart alias mapping first
        if let Some(aliased_name) = resolve_smart_alias(query) {
            let alias_norm = normalize_string(aliased_name);
            if let Some(t) = all.iter().find(|t| normalize_string(&t.name) == alias_norm) {
                return Ok(t.clone());
            }
        }

        // 2. Exact match against normalized name (e.g. "global/macos" or "rust")
        if let Some(t) = all.iter().find(|t| normalize_string(&t.name) == query_norm) {
            return Ok(t.clone());
        }

        // 3. Match against basename (e.g. "macos" matches "Global/macOS", "jetbrains" matches "Global/JetBrains")
        if let Some(t) = all.iter().find(|t| {
            if let Some(base) = t.name.split('/').last() {
                normalize_string(base) == query_norm
            } else {
                false
            }
        }) {
            return Ok(t.clone());
        }

        // 4. Try fuzzy matching as fallback
        let results = self.search_templates(query)?;
        if let Some((best, score)) = results.first() {
            if *score > 30 {
                return Ok(best.clone());
            }
        }

        Err(anyhow!(
            "Template '{}' not found. Run `ign search {}` or `ign list` to view available templates.",
            query,
            query
        ))
    }

    /// Get raw content of a template by name or alias
    pub fn get_template_content(&self, query: &str) -> Result<(TemplateInfo, String)> {
        let info = self.find_template(query)?;
        let content = fs::read_to_string(&info.path).with_context(|| {
            format!("Failed to read template file from {:?}", info.path)
        })?;
        Ok((info, content))
    }

    fn check_git_available() -> Result<()> {
        let output = Command::new("git").arg("--version").output();
        match output {
            Ok(out) if out.status.success() => Ok(()),
            _ => Err(anyhow!(
                "Git executable was not found on your system. Please install Git and make sure it is in your PATH."
            )),
        }
    }
}

/// Helper: Extract clean name without `.gitignore` extension
fn extract_clean_name(rel_path: &str) -> String {
    let normalized = rel_path.replace('\\', "/");
    normalized
        .strip_suffix(".gitignore")
        .unwrap_or(&normalized)
        .to_string()
}

/// Helper: Normalize string for comparison (lowercase, stripped of punctuation)
fn normalize_string(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(['_', '-', ' ', '.', '/'], "")
}

/// Resolve popular shorthand aliases to standard repository template names
fn resolve_smart_alias(query: &str) -> Option<&'static str> {
    let norm = normalize_string(query);
    match norm.as_str() {
        "macos" | "osx" | "mac" | "darwin" | "apple" => Some("Global/macOS"),
        "vscode" | "visualstudiocode" | "vs-code" | "code" => Some("Global/Visual_Studio_Code"),
        "idea" | "intellij" | "jetbrains" | "pycharm" | "clion" | "webstorm" | "goland"
        | "rider" | "phpstorm" | "rubymine" => Some("Global/JetBrains"),
        "windows" | "win" | "win32" | "win64" => Some("Global/Windows"),
        "linux" => Some("Global/Linux"),
        "vim" | "nvim" | "neovim" => Some("Global/Vim"),
        "emacs" => Some("Global/Emacs"),
        "sublime" | "sublimetext" => Some("Global/SublimeText"),
        "eclipse" => Some("Global/Eclipse"),
        "xcode" => Some("Global/Xcode"),
        "node" | "nodejs" | "npm" | "yarn" | "pnpm" | "bun" => Some("Node"),
        "python" | "py" | "pip" | "venv" | "conda" => Some("Python"),
        "rust" | "rs" | "cargo" => Some("Rust"),
        "go" | "golang" => Some("Go"),
        "java" | "jvm" | "maven" | "mvn" | "gradle" => Some("Java"),
        "c++" | "cpp" | "cplusplus" => Some("C++"),
        "c" => Some("C"),
        "c#" | "csharp" | "cs" | "dotnet" => Some("VisualStudio"),
        "ruby" | "rb" | "gem" | "rails" => Some("Ruby"),
        "php" | "composer" => Some("PHP"),
        "swift" => Some("Swift"),
        "kotlin" | "kt" => Some("Kotlin"),
        "dart" | "flutter" => Some("Dart"),
        "android" => Some("Android"),
        "unity" => Some("Unity"),
        "unreal" | "unrealengine" | "ue4" | "ue5" => Some("UnrealEngine"),
        "elixir" | "ex" | "phoenix" => Some("Elixir"),
        "haskell" | "hs" | "cabal" | "stack" => Some("Haskell"),
        "lua" => Some("Lua"),
        "r" => Some("R"),
        "scala" | "sbt" => Some("Scala"),
        "tex" | "latex" => Some("TeX"),
        "zig" => Some("Zig"),
        _ => None,
    }
}

/// Helper: Filter out hidden files and .git directory
fn is_hidden_or_git(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == ".git")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_clean_name() {
        assert_eq!(extract_clean_name("Rust.gitignore"), "Rust");
        assert_eq!(extract_clean_name("Global/macOS.gitignore"), "Global/macOS");
        assert_eq!(
            extract_clean_name("community/Elixir/Phoenix.gitignore"),
            "community/Elixir/Phoenix"
        );
    }

    #[test]
    fn test_smart_aliases() {
        assert_eq!(resolve_smart_alias("macos"), Some("Global/macOS"));
        assert_eq!(resolve_smart_alias("mac"), Some("Global/macOS"));
        assert_eq!(resolve_smart_alias("vscode"), Some("Global/Visual_Studio_Code"));
        assert_eq!(resolve_smart_alias("idea"), Some("Global/JetBrains"));
        assert_eq!(resolve_smart_alias("rust"), Some("Rust"));
        assert_eq!(resolve_smart_alias("python"), Some("Python"));
    }
}

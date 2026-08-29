use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Category classification of a GitHub gitignore template
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    Language,
    Global,
    Community,
}

impl TemplateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateCategory::Language => "Language",
            TemplateCategory::Global => "Global",
            TemplateCategory::Community => "Community",
        }
    }

    /// Determine category based on the relative path in the github/gitignore repository
    pub fn from_relative_path(path: &str) -> Self {
        let normalized = path.replace('\\', "/");
        if normalized.starts_with("Global/") || normalized.starts_with("global/") {
            TemplateCategory::Global
        } else if normalized.starts_with("community/") || normalized.starts_with("Community/") {
            TemplateCategory::Community
        } else {
            TemplateCategory::Language
        }
    }
}

impl fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Metadata and location information for an individual .gitignore template
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Clean display/query name, e.g. "Rust", "Global/macOS", "community/Elixir/Phoenix"
    pub name: String,
    /// Absolute filesystem path to the template file in cache
    pub path: PathBuf,
    /// Category (Language, Global, Community)
    pub category: TemplateCategory,
    /// Relative path inside the repository, e.g. "Rust.gitignore", "Global/macOS.gitignore"
    pub relative_path: String,
}

/// Representation of an applied/parsed .gitignore file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedGitignore {
    /// Ordered list of template names included in this .gitignore
    pub templates: Vec<String>,
    /// When the file was first created by ign
    pub created_at: Option<DateTime<Utc>>,
    /// When the file was last updated by ign
    pub updated_at: Option<DateTime<Utc>>,
    /// Custom user rules or pre-existing unmanaged gitignore content
    pub custom_content: Option<String>,
    /// Map of template name to raw template content blocks
    pub template_blocks: HashMap<String, String>,
}

impl Default for AppliedGitignore {
    fn default() -> Self {
        Self {
            templates: Vec::new(),
            created_at: None,
            updated_at: None,
            custom_content: None,
            template_blocks: HashMap::new(),
        }
    }
}

/// Status summary for `ign status` / `ign info`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitignoreStatus {
    pub file_exists: bool,
    pub path: String,
    pub managed_by_ign: bool,
    pub templates_count: usize,
    pub templates: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub has_custom_rules: bool,
    pub custom_rules_lines: usize,
    pub custom_rules: Option<String>,
    pub total_lines: usize,
}


use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ign",
    author = "Skyxim",
    version,
    about = "A fast, flexible, and feature-rich .gitignore CLI manager based on https://github.com/github/gitignore",
    long_about = "ign is a command-line tool written in Rust that lets you search, inspect, initialize, merge, and manage .gitignore files directly using the official GitHub gitignore template library (https://github.com/github/gitignore)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output results in JSON format
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new .gitignore file with chosen templates
    #[command(alias = "i", alias = "new", alias = "create")]
    Init(InitArgs),

    /// Add one or more templates into existing .gitignore
    #[command(alias = "a", alias = "append")]
    Add(AddArgs),

    /// Remove one or more templates from existing .gitignore
    #[command(alias = "rm", alias = "delete")]
    Remove(RemoveArgs),

    /// List all available templates from the repository
    #[command(alias = "ls")]
    List(ListArgs),

    /// Search templates with fuzzy matching and smart aliases
    #[command(alias = "s", alias = "find")]
    Search(SearchArgs),

    /// Show the raw content of a specific template
    #[command(alias = "cat", alias = "view", alias = "get")]
    Show(ShowArgs),

    /// Show status and details of the current .gitignore file
    #[command(alias = "st", alias = "info", alias = "check")]
    Status(StatusArgs),

    /// Manage local gitignore template library (init, update, path, status)
    #[command(alias = "tpl")]
    Template(TemplateArgs),

    /// Update local templates library from GitHub (alias for `template update`)
    #[command(alias = "sync", alias = "pull")]
    Update,
}

#[derive(Args, Debug)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub command: TemplateCommands,
}

#[derive(Subcommand, Debug)]
pub enum TemplateCommands {
    /// Initialize template library from GitHub
    #[command(alias = "i", alias = "clone")]
    Init,

    /// Update template library from GitHub
    #[command(alias = "sync", alias = "pull")]
    Update,

    /// Show local storage path of template library
    #[command(alias = "dir")]
    Path,

    /// Show template library version and status
    #[command(alias = "st", alias = "info")]
    Status,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Template names to apply (e.g. rust, macos, vscode, python)
    #[arg(value_name = "TEMPLATES")]
    pub templates: Vec<String>,

    /// Run interactive multi-select menu to pick templates
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Force overwrite existing .gitignore file without preserving existing content
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Target file path (defaults to .gitignore)
    #[arg(short = 'o', long = "output", default_value = ".gitignore")]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Template names to add (e.g. macos, jetbrains)
    #[arg(value_name = "TEMPLATES")]
    pub templates: Vec<String>,

    /// Interactively select templates to add
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Target .gitignore file path (defaults to .gitignore)
    #[arg(short = 'p', long = "path", default_value = ".gitignore")]
    pub path: PathBuf,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Template names to remove from .gitignore (e.g. macos, python)
    #[arg(required = true, value_name = "TEMPLATES")]
    pub templates: Vec<String>,

    /// Target .gitignore file path (defaults to .gitignore)
    #[arg(short = 'p', long = "path", default_value = ".gitignore")]
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CategoryFilter {
    #[value(alias = "lang")]
    Language,
    #[value(alias = "glob")]
    Global,
    #[value(alias = "comm")]
    Community,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter templates by category (language, global, community)
    #[arg(short = 'c', long = "category")]
    pub category: Option<CategoryFilter>,

    /// Filter templates by name or keyword
    #[arg(short = 'f', long = "filter")]
    pub filter: Option<String>,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Query string to fuzzy search templates
    #[arg(required = true)]
    pub query: String,

    /// Maximum number of search results to return
    #[arg(short = 'l', long = "limit", default_value_t = 25)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Template name or alias (e.g. rust, macos, vscode)
    #[arg(required = true)]
    pub template: String,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Path to .gitignore file to inspect (defaults to .gitignore)
    #[arg(short = 'p', long = "path", default_value = ".gitignore")]
    pub path: PathBuf,
}

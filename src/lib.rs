pub mod cli;
pub mod interactive;
pub mod models;
pub mod parser;
pub mod repo;

pub use cli::Cli;
pub use models::{AppliedGitignore, GitignoreStatus, TemplateCategory, TemplateInfo};
pub use parser::GitignoreParser;
pub use repo::GitignoreRepo;

use anyhow::{anyhow, Result};
use colored::*;
use std::fs;

use crate::cli::{
    AddArgs, CategoryFilter, Commands, InitArgs, ListArgs, RemoveArgs, SearchArgs, ShowArgs,
    StatusArgs, TemplateArgs, TemplateCommands,
};
use crate::interactive::select_templates_interactively;

pub fn run() {
    let cli = <Cli as clap::Parser>::parse();
    if let Err(err) = run_with_cli(cli) {
        eprintln!("{} {}", "Error:".bright_red().bold(), err);
        std::process::exit(1);
    }
}

pub fn run_with_cli(cli: Cli) -> Result<()> {
    let json_mode = cli.json;
    let repo = GitignoreRepo::new()?;

    match cli.command {
        Commands::Init(args) => handle_init(args, &repo, json_mode),
        Commands::Add(args) => handle_add(args, &repo, json_mode),
        Commands::Remove(args) => handle_remove(args, json_mode),
        Commands::List(args) => handle_list(args, &repo, json_mode),
        Commands::Search(args) => handle_search(args, &repo, json_mode),
        Commands::Show(args) => handle_show(args, &repo),
        Commands::Status(args) => handle_status(args, json_mode),
        Commands::Template(args) => handle_template(args, &repo, json_mode),
        Commands::Update => handle_template_update(&repo),
    }
}

fn handle_init(args: InitArgs, repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let mut template_names = args.templates;

    if template_names.is_empty() || args.interactive {
        let all_templates = repo.list_templates()?;
        let selected = select_templates_interactively(&all_templates)?;
        if selected.is_empty() && template_names.is_empty() {
            return Err(anyhow!("No templates were selected. Operation aborted."));
        }
        for s in selected {
            if !template_names.contains(&s) {
                template_names.push(s);
            }
        }
    }

    if template_names.is_empty() {
        return Err(anyhow!(
            "No templates specified. Provide template names (e.g. `ign init rust macos`) or use `--interactive`."
        ));
    }

    // Resolve template contents
    let mut resolved_templates = Vec::new();
    for name in &template_names {
        let (info, content) = repo.get_template_content(name)?;
        resolved_templates.push((info.name, content));
    }

    // Prepare applied gitignore
    let mut applied = if args.force || !args.output.exists() {
        AppliedGitignore::default()
    } else {
        GitignoreParser::read_from_file(&args.output)?
    };

    let added = GitignoreParser::add_templates(&mut applied, resolved_templates);
    let rendered = GitignoreParser::render(&applied);

    fs::write(&args.output, rendered)?;

    if json_mode {
        let status = GitignoreParser::get_status(&args.output, &applied);
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!(
            "{} Successfully initialized {} with {} templates:",
            "✔".bright_green().bold(),
            args.output.display().to_string().bright_cyan(),
            added.len().to_string().bright_yellow().bold()
        );
        for name in &added {
            println!("  • {}", name.bright_green());
        }
        if let Some(ref custom) = applied.custom_content {
            if !custom.trim().is_empty() {
                println!(
                    "  {} Preserved existing custom rules ({} lines)",
                    "ℹ".bright_blue(),
                    custom.lines().count()
                );
            }
        }
    }

    Ok(())
}

fn handle_add(args: AddArgs, repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let mut template_names = args.templates;

    if template_names.is_empty() || args.interactive {
        let all_templates = repo.list_templates()?;
        let selected = select_templates_interactively(&all_templates)?;
        if selected.is_empty() && template_names.is_empty() {
            return Err(anyhow!("No templates were selected. Operation aborted."));
        }
        for s in selected {
            if !template_names.contains(&s) {
                template_names.push(s);
            }
        }
    }

    if template_names.is_empty() {
        return Err(anyhow!(
            "No templates specified. Run `ign add <TEMPLATE>...` or `ign add -i`."
        ));
    }

    let mut applied = GitignoreParser::read_from_file(&args.path)?;

    let mut resolved_templates = Vec::new();
    for name in &template_names {
        let (info, content) = repo.get_template_content(name)?;
        resolved_templates.push((info.name, content));
    }

    let added = GitignoreParser::add_templates(&mut applied, resolved_templates);
    let rendered = GitignoreParser::render(&applied);

    fs::write(&args.path, rendered)?;

    if json_mode {
        let status = GitignoreParser::get_status(&args.path, &applied);
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!(
            "{} Added templates to {}:",
            "✔".bright_green().bold(),
            args.path.display().to_string().bright_cyan()
        );
        for name in &added {
            println!("  + {}", name.bright_green());
        }
    }

    Ok(())
}

fn handle_remove(args: RemoveArgs, json_mode: bool) -> Result<()> {
    if !args.path.exists() {
        return Err(anyhow!(
            "Gitignore file '{}' does not exist.",
            args.path.display()
        ));
    }

    let mut applied = GitignoreParser::read_from_file(&args.path)?;
    let removed = GitignoreParser::remove_templates(&mut applied, &args.templates);

    if removed.is_empty() {
        if json_mode {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "warning",
                    "message": "No matching templates found to remove",
                    "requested": args.templates,
                    "active_templates": applied.templates
                }))?
            );
        } else {
            println!(
                "{} No matching templates found in {} to remove. Current templates: {:?}",
                "⚠".bright_yellow().bold(),
                args.path.display().to_string().bright_cyan(),
                applied.templates
            );
        }
        return Ok(());
    }

    let rendered = GitignoreParser::render(&applied);
    fs::write(&args.path, rendered)?;

    if json_mode {
        let status = GitignoreParser::get_status(&args.path, &applied);
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!(
            "{} Removed templates from {}:",
            "✔".bright_green().bold(),
            args.path.display().to_string().bright_cyan()
        );
        for name in &removed {
            println!("  - {}", name.bright_red());
        }
    }

    Ok(())
}

fn handle_list(args: ListArgs, repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let mut templates = repo.list_templates()?;

    if let Some(cat) = args.category {
        let target_category = match cat {
            CategoryFilter::Language => TemplateCategory::Language,
            CategoryFilter::Global => TemplateCategory::Global,
            CategoryFilter::Community => TemplateCategory::Community,
        };
        templates.retain(|t| t.category == target_category);
    }

    if let Some(ref filter_keyword) = args.filter {
        let fk = filter_keyword.to_lowercase();
        templates.retain(|t| {
            t.name.to_lowercase().contains(&fk) || t.relative_path.to_lowercase().contains(&fk)
        });
    }

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&templates)?);
        return Ok(());
    }

    println!(
        "{} Found {} available templates:",
        "📦".bright_cyan(),
        templates.len().to_string().bright_yellow().bold()
    );

    let mut current_cat: Option<TemplateCategory> = None;
    for t in &templates {
        if current_cat != Some(t.category) {
            current_cat = Some(t.category);
            println!(
                "\n{} {}:",
                "▶".bright_magenta(),
                t.category.as_str().bold().underline()
            );
        }
        println!(
            "  • {:<35} {}",
            t.name.bright_white(),
            t.relative_path.bright_black()
        );
    }

    println!("\nUse `ign show <NAME>` to inspect, or `ign init <NAME>...` to generate.");
    Ok(())
}

fn handle_search(args: SearchArgs, repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let results = repo.search_templates(&args.query)?;
    let limited: Vec<_> = results.into_iter().take(args.limit).collect();

    if json_mode {
        let json_results: Vec<_> = limited
            .iter()
            .map(|(info, score)| {
                serde_json::json!({
                    "name": info.name,
                    "category": info.category,
                    "relative_path": info.relative_path,
                    "score": score
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_results)?);
        return Ok(());
    }

    if limited.is_empty() {
        println!(
            "{} No templates found matching query '{}'.",
            "⚠".bright_yellow(),
            args.query.bright_cyan()
        );
        return Ok(());
    }

    println!(
        "{} Search results for '{}' (showing top {}):",
        "🔍".bright_cyan(),
        args.query.bright_green().bold(),
        limited.len()
    );

    for (info, score) in limited {
        println!(
            "  [{}] {:<35} {} {}",
            info.category.as_str().bright_blue(),
            info.name.bright_white().bold(),
            format!("(score: {})", score).bright_black(),
            info.relative_path.bright_black()
        );
    }

    Ok(())
}

fn handle_show(args: ShowArgs, repo: &GitignoreRepo) -> Result<()> {
    let (info, content) = repo.get_template_content(&args.template)?;
    println!(
        "{} Template: {} ({}) [{}]",
        "#".bright_magenta().bold(),
        info.name.bright_green().bold(),
        info.relative_path.bright_black(),
        info.category.as_str().bright_blue()
    );
    println!("--------------------------------------------------------------------------------");
    print!("{}", content);
    if !content.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn handle_status(args: StatusArgs, json_mode: bool) -> Result<()> {
    let path = &args.path;
    let applied = GitignoreParser::read_from_file(path)?;
    let status = GitignoreParser::get_status(path, &applied);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!(
        "{} Gitignore Status: {}",
        "📋".bright_cyan(),
        path.display().to_string().bright_yellow().bold()
    );
    println!("--------------------------------------------------");
    println!(
        "  • Exists:            {}",
        if status.file_exists {
            "Yes".bright_green()
        } else {
            "No".bright_red()
        }
    );
    println!(
        "  • Managed by ign:   {}",
        if status.managed_by_ign {
            "Yes".bright_green()
        } else {
            "No (Plain/Unmanaged)".bright_yellow()
        }
    );
    println!(
        "  • Applied Templates: {}",
        status.templates_count.to_string().bright_cyan()
    );

    if !status.templates.is_empty() {
        for t in &status.templates {
            println!("    - {}", t.bright_green());
        }
    }

    if let Some(created) = status.created_at {
        println!(
            "  • Created At:        {}",
            created.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_black()
        );
    }
    if let Some(updated) = status.updated_at {
        println!(
            "  • Updated At:        {}",
            updated.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_black()
        );
    }

    println!(
        "  • Custom Rules:      {}",
        if status.has_custom_rules {
            format!("Yes ({} lines)", status.custom_rules_lines).bright_green()
        } else {
            "None".bright_black()
        }
    );
    println!("  • Total Lines:       {}", status.total_lines);

    Ok(())
}

fn handle_template(args: TemplateArgs, repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    match args.command {
        TemplateCommands::Init => handle_template_init(repo),
        TemplateCommands::Update => handle_template_update(repo),
        TemplateCommands::Path => handle_template_path(repo, json_mode),
        TemplateCommands::Status => handle_template_status(repo, json_mode),
    }
}

fn handle_template_init(repo: &GitignoreRepo) -> Result<()> {
    if repo.is_cached() {
        println!(
            "{} Template library is already initialized at {:?}.\nRun `ign template update` (or `ign update`) to fetch the latest templates.",
            "ℹ".bright_blue(),
            repo.templates_dir()
        );
        return Ok(());
    }
    repo.init_repo()
}

fn handle_template_update(repo: &GitignoreRepo) -> Result<()> {
    repo.update_repo()
}

fn handle_template_path(repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let path = repo.templates_dir();
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "path": path.to_string_lossy(),
                "exists": path.exists(),
                "is_initialized": repo.is_cached()
            }))?
        );
    } else {
        println!("{}", path.display());
    }
    Ok(())
}

fn handle_template_status(repo: &GitignoreRepo, json_mode: bool) -> Result<()> {
    let is_init = repo.is_cached();
    let path = repo.templates_dir();

    if !is_init {
        if json_mode {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "path": path.to_string_lossy(),
                    "initialized": false,
                    "templates_count": 0,
                    "message": "Template library not initialized. Run `ign template init` to clone."
                }))?
            );
        } else {
            println!(
                "{} Template Library Status: {}",
                "📦".bright_cyan(),
                path.display().to_string().bright_yellow().bold()
            );
            println!("--------------------------------------------------");
            println!("  • Initialized:       {}", "No".bright_red());
            println!(
                "  • Action Required:   Please run `{}` to initialize.",
                "ign template init".bright_green().bold()
            );
        }
        return Ok(());
    }

    let templates = repo.list_templates().unwrap_or_default();
    let lang_count = templates
        .iter()
        .filter(|t| t.category == TemplateCategory::Language)
        .count();
    let global_count = templates
        .iter()
        .filter(|t| t.category == TemplateCategory::Global)
        .count();
    let community_count = templates
        .iter()
        .filter(|t| t.category == TemplateCategory::Community)
        .count();

    // Try to get git commit info
    let git_commit = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["log", "-1", "--format=%h - %s (%cr)"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        });

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "path": path.to_string_lossy(),
                "initialized": true,
                "total_templates": templates.len(),
                "categories": {
                    "language": lang_count,
                    "global": global_count,
                    "community": community_count
                },
                "last_commit": git_commit
            }))?
        );
    } else {
        println!(
            "{} Template Library Status: {}",
            "📦".bright_cyan(),
            path.display().to_string().bright_green().bold()
        );
        println!("--------------------------------------------------");
        println!("  • Initialized:       {}", "Yes".bright_green());
        println!(
            "  • Total Templates:   {}",
            templates.len().to_string().bright_yellow().bold()
        );
        println!("    - Language:        {}", lang_count.to_string().bright_cyan());
        println!("    - Global:          {}", global_count.to_string().bright_cyan());
        println!(
            "    - Community:       {}",
            community_count.to_string().bright_cyan()
        );
        if let Some(commit) = git_commit {
            println!("  • Latest Commit:     {}", commit.bright_black());
        }
    }

    Ok(())
}

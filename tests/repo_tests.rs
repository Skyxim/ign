use ign::models::TemplateCategory;
use ign::repo::GitignoreRepo;
use std::fs;

#[test]
fn test_category_from_path() {
    assert_eq!(
        TemplateCategory::from_relative_path("Rust.gitignore"),
        TemplateCategory::Language
    );
    assert_eq!(
        TemplateCategory::from_relative_path("Global/macOS.gitignore"),
        TemplateCategory::Global
    );
    assert_eq!(
        TemplateCategory::from_relative_path("community/Elixir/Phoenix.gitignore"),
        TemplateCategory::Community
    );
    assert_eq!(
        TemplateCategory::from_relative_path("community/DotNet/WPF.gitignore"),
        TemplateCategory::Community
    );
}

#[test]
fn test_repo_mock_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_path = temp_dir.path().join("templates");
    fs::create_dir_all(&cache_path).unwrap();

    // Create fake git directory to mock git clone
    fs::create_dir_all(cache_path.join(".git")).unwrap();

    // Create mock templates
    fs::write(
        cache_path.join("Rust.gitignore"),
        "/target\nCargo.lock\n",
    )
    .unwrap();
    fs::write(
        cache_path.join("Python.gitignore"),
        "*.pyc\n__pycache__/\n",
    )
    .unwrap();

    let global_dir = cache_path.join("Global");
    fs::create_dir_all(&global_dir).unwrap();
    fs::write(global_dir.join("macOS.gitignore"), ".DS_Store\n").unwrap();
    fs::write(global_dir.join("JetBrains.gitignore"), ".idea/\n*.iml\n").unwrap();
    fs::write(
        global_dir.join("Visual_Studio_Code.gitignore"),
        ".vscode/\n",
    )
    .unwrap();

    let community_dir = cache_path.join("community").join("Rust");
    fs::create_dir_all(&community_dir).unwrap();
    fs::write(
        community_dir.join("Bevy.gitignore"),
        "assets/imported\n",
    )
    .unwrap();

    let repo = GitignoreRepo::with_templates_dir(&cache_path);
    assert!(repo.is_cached());
    assert_eq!(repo.templates_dir(), cache_path.as_path());

    // Test list_templates
    let templates = repo.list_templates().unwrap();
    assert_eq!(templates.len(), 6);

    let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
    assert!(names.contains(&"Rust".to_string()));
    assert!(names.contains(&"Python".to_string()));
    assert!(names.contains(&"Global/macOS".to_string()));
    assert!(names.contains(&"Global/JetBrains".to_string()));
    assert!(names.contains(&"Global/Visual_Studio_Code".to_string()));
    assert!(names.contains(&"community/Rust/Bevy".to_string()));

    // Test find_template and smart aliases
    let macos = repo.find_template("macos").unwrap();
    assert_eq!(macos.name, "Global/macOS");

    let vscode = repo.find_template("vscode").unwrap();
    assert_eq!(vscode.name, "Global/Visual_Studio_Code");

    let idea = repo.find_template("idea").unwrap();
    assert_eq!(idea.name, "Global/JetBrains");

    let jetbrains = repo.find_template("jetbrains").unwrap();
    assert_eq!(jetbrains.name, "Global/JetBrains");

    let rust = repo.find_template("rust").unwrap();
    assert_eq!(rust.name, "Rust");

    // Test get_template_content
    let (info, content) = repo.get_template_content("macos").unwrap();
    assert_eq!(info.name, "Global/macOS");
    assert_eq!(content, ".DS_Store\n");

    // Test search_templates
    let search_res = repo.search_templates("mac").unwrap();
    assert!(!search_res.is_empty());
    assert_eq!(search_res[0].0.name, "Global/macOS");

    // Test ensure_cache_ready succeeds when cached
    let ensure_res = repo.ensure_cache_ready();
    assert!(ensure_res.is_ok());
}

#[test]
fn test_uninitialized_cache_behavior() {
    let temp_dir = tempfile::tempdir().unwrap();
    let uninit_path = temp_dir.path().join("empty_templates");

    let repo = GitignoreRepo::with_templates_dir(&uninit_path);
    assert!(!repo.is_cached());

    // ensure_cache_ready should return error with helpful instructions
    let err = repo.ensure_cache_ready().unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("Template library is not initialized or empty"));
    assert!(err_str.contains("ign template init"));
    assert!(err_str.contains("ign update"));

    // list_templates should also fail with the same message
    let list_err = repo.list_templates().unwrap_err();
    assert!(list_err.to_string().contains("Template library is not initialized or empty"));

    // Directory with .git but no .gitignore files should still not be considered ready/cached
    fs::create_dir_all(uninit_path.join(".git")).unwrap();
    assert!(!repo.is_cached());

    // Once a .gitignore file is added, it is cached
    fs::write(uninit_path.join("Rust.gitignore"), "/target\n").unwrap();
    assert!(repo.is_cached());
    assert!(repo.ensure_cache_ready().is_ok());
}

#[test]
fn test_custom_env_var_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let custom_path = temp_dir.path().join("custom_templates");
    std::env::set_var("IGN_TEMPLATES_DIR", custom_path.to_str().unwrap());

    let repo = GitignoreRepo::new().unwrap();
    assert_eq!(repo.templates_dir(), custom_path.as_path());

    std::env::remove_var("IGN_TEMPLATES_DIR");
}

#[test]
fn test_cli_template_subcommand_parsing() {
    use clap::Parser;
    use ign::cli::{Cli, Commands, TemplateCommands};

    // test `ign template init` and alias `tpl i`
    let cli = Cli::try_parse_from(["ign", "template", "init"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Init)),
        _ => panic!("Expected Commands::Template"),
    }

    let cli = Cli::try_parse_from(["ign", "tpl", "i"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Init)),
        _ => panic!("Expected Commands::Template"),
    }

    // test `ign template update` and alias `tpl update`
    let cli = Cli::try_parse_from(["ign", "template", "update"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Update)),
        _ => panic!("Expected Commands::Template"),
    }

    // test `ign template path` and alias `tpl dir`
    let cli = Cli::try_parse_from(["ign", "template", "path"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Path)),
        _ => panic!("Expected Commands::Template"),
    }

    let cli = Cli::try_parse_from(["ign", "tpl", "dir"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Path)),
        _ => panic!("Expected Commands::Template"),
    }

    // test `ign template status` and alias `tpl st`
    let cli = Cli::try_parse_from(["ign", "template", "status"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Status)),
        _ => panic!("Expected Commands::Template"),
    }

    let cli = Cli::try_parse_from(["ign", "tpl", "st"]).unwrap();
    match cli.command {
        Commands::Template(args) => assert!(matches!(args.command, TemplateCommands::Status)),
        _ => panic!("Expected Commands::Template"),
    }

    // test top-level `ign update`
    let cli = Cli::try_parse_from(["ign", "update"]).unwrap();
    assert!(matches!(cli.command, Commands::Update));

    // verify --no-update is no longer recognized
    let res = Cli::try_parse_from(["ign", "--no-update", "list"]);
    assert!(res.is_err());
}

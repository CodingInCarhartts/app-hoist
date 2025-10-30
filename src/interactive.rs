use crate::cli::{CacheCommand, TemplateCommand};
use crate::docker;
use crate::models::ProjectType;
use crate::multi_project;
use crate::package;
use crate::project;
use crate::template;
use inquire::{Confirm, Select, Text};

#[derive(Debug, Clone)]
enum MainMenuChoice {
    PackageManagement,
    ProjectManagement,
    DockerOperations,
    MultiProjectOperations,
    TemplateOperations,
    CacheOperations,
    Help,
    Exit,
}

impl std::fmt::Display for MainMenuChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuChoice::PackageManagement => {
                write!(f, "📦 Package Management - Hoist executables/packages")
            }
            MainMenuChoice::ProjectManagement => {
                write!(f, "🏗️  Project Management - Manage development projects")
            }
            MainMenuChoice::DockerOperations => {
                write!(f, "🐳 Docker Operations - Container management")
            }
            MainMenuChoice::MultiProjectOperations => write!(
                f,
                "🔄 Multi-Project Operations - Parallel project management"
            ),
            MainMenuChoice::TemplateOperations => {
                write!(f, "📋 Template Operations - Project scaffolding")
            }
            MainMenuChoice::CacheOperations => {
                write!(f, "💾 Cache Operations - Manage cached data")
            }
            MainMenuChoice::Help => write!(f, "❓ Help/About - Information and help"),
            MainMenuChoice::Exit => write!(f, "🚪 Exit - Quit app-hoist"),
        }
    }
}

pub async fn run_interactive_mode() -> anyhow::Result<()> {
    println!("🚀 Welcome to app-hoist interactive mode!");
    println!("==========================================");
    println!("Select an option below to get started.\n");

    loop {
        let choices = vec![
            MainMenuChoice::PackageManagement,
            MainMenuChoice::ProjectManagement,
            MainMenuChoice::DockerOperations,
            MainMenuChoice::MultiProjectOperations,
            MainMenuChoice::TemplateOperations,
            MainMenuChoice::CacheOperations,
            MainMenuChoice::Help,
            MainMenuChoice::Exit,
        ];

        let selection = Select::new("What would you like to do?", choices).prompt()?;

        match selection {
            MainMenuChoice::PackageManagement => {
                handle_package_management().await?;
            }
            MainMenuChoice::ProjectManagement => {
                handle_project_management().await?;
            }
            MainMenuChoice::DockerOperations => {
                handle_docker_operations().await?;
            }
            MainMenuChoice::MultiProjectOperations => {
                handle_multi_project_operations().await?;
            }
            MainMenuChoice::TemplateOperations => {
                handle_template_operations()?;
            }
            MainMenuChoice::CacheOperations => {
                handle_cache_operations()?;
            }
            MainMenuChoice::Help => {
                show_help();
            }
            MainMenuChoice::Exit => {
                println!("👋 Goodbye! Thanks for using app-hoist.");
                break;
            }
        }

        // Ask if user wants to continue
        if !Confirm::new("Would you like to perform another operation?")
            .with_default(true)
            .prompt()?
        {
            println!("👋 Goodbye! Thanks for using app-hoist.");
            break;
        }
        println!(); // Add spacing
    }

    Ok(())
}

async fn handle_package_management() -> anyhow::Result<()> {
    println!("📦 Package Management");
    println!("Hoist executables and packages to make them available system-wide.\n");

    let package_name = Text::new("Enter the name of the package/executable to hoist:").prompt()?;

    let dry_run = Confirm::new("Dry run? (Show what would be done without executing)")
        .with_default(false)
        .prompt()?;

    package::handle_package_mode(&package_name, dry_run)?;
    Ok(())
}

async fn handle_project_management() -> anyhow::Result<()> {
    println!("🏗️  Project Management");
    println!("Manage development projects (Rust, Go, Python, JavaScript/TypeScript).\n");

    // Auto-detect current directory
    let current_dir = std::env::current_dir()?;
    let current_path = current_dir.to_string_lossy();

    println!("Current directory: {}", current_path);

    // Try to detect project type
    let detected_type = detect_project_in_current_dir()?;

    match detected_type {
        Some(project_type) => {
            println!("✅ Detected {} project in current directory", project_type);

            // Show available operations for this project type
            let available_ops = get_available_operations(&project_type);
            if !available_ops.is_empty() {
                println!("📋 Available operations: {}", available_ops.join(", "));
            }

            let use_current = Confirm::new(&format!("Use current directory ({})?", current_path))
                .with_default(true)
                .prompt()?;

            if use_current {
                project::handle_project_mode(&current_path, false)?;
            } else {
                let path_input = Text::new("Enter project path:")
                    .with_default(".")
                    .prompt()?;
                let path = expand_tilde(&path_input)?;
                project::handle_project_mode(&path, false)?;
            }
        }
        None => {
            println!("❌ No project detected in current directory");
            println!(
                "💡 Supported project types: Rust (Cargo.toml), Go (go.mod), Python (pyproject.toml/uv), JavaScript/TypeScript (package.json)"
            );

            let path_input = Text::new("Enter project path:")
                .with_default(".")
                .prompt()?;
            let path = expand_tilde(&path_input)?;
            project::handle_project_mode(&path, false)?;
        }
    }

    Ok(())
}

async fn handle_docker_operations() -> anyhow::Result<()> {
    println!("🐳 Docker Operations");
    println!("Choose between direct Docker commands or managing Docker-enabled projects.\n");

    let docker_choices = vec!["Direct Docker Commands", "Docker Project Management"];

    let selection = Select::new("Select Docker operation type:", docker_choices).prompt()?;

    match selection {
        "Direct Docker Commands" => {
            let command =
                Text::new("Enter Docker command (e.g., 'ps -a', 'images', 'system prune'):")
                    .prompt()?;

            let dry_run = Confirm::new("Dry run?").with_default(false).prompt()?;

            docker::handle_direct_docker_mode(&command, dry_run)?;
        }
        "Docker Project Management" => {
            let path_input = Text::new("Enter path to Docker-enabled project:")
                .with_default(".")
                .prompt()?;
            let path = expand_tilde(&path_input)?;

            let dry_run = Confirm::new("Dry run?").with_default(false).prompt()?;

            docker::handle_docker_project_mode(&path, dry_run)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn handle_multi_project_operations() -> anyhow::Result<()> {
    println!("🔄 Multi-Project Operations");
    println!("Run operations across multiple projects in parallel.\n");

    let mut paths = Vec::new();

    println!("Enter project paths (one per line, leave empty to finish):");
    loop {
        let path_input = Text::new(&format!(
            "Project path {} (leave empty to finish):",
            paths.len() + 1
        ))
        .prompt()?;

        if path_input.trim().is_empty() {
            break;
        }

        let path = expand_tilde(&path_input)?;

        // Validate path exists
        if !std::path::Path::new(&path).exists() {
            println!("⚠️  Warning: Path '{}' does not exist", path);
            let continue_anyway = Confirm::new("Continue anyway?")
                .with_default(false)
                .prompt()?;
            if !continue_anyway {
                continue;
            }
        }

        paths.push(path);

        if paths.len() >= 10 {
            println!("⚠️  Maximum of 10 projects reached");
            break;
        }
    }

    if paths.is_empty() {
        println!("❌ No paths provided. Operation cancelled.");
        return Ok(());
    }

    println!(
        "📂 Selected {} projects for parallel operations:",
        paths.len()
    );
    for (i, path) in paths.iter().enumerate() {
        println!("  {}. {}", i + 1, path);
    }
    println!();

    let dry_run = Confirm::new("Dry run (preview commands without executing)?")
        .with_default(false)
        .prompt()?;

    multi_project::handle_multi_project_mode(&paths, dry_run).await?;
    Ok(())
}

fn handle_template_operations() -> anyhow::Result<()> {
    println!("📋 Template Operations");
    println!("Manage project templates for quick scaffolding.\n");

    let template_choices = vec![
        "List Available Templates",
        "Initialize Project from Template",
        "Create Template from Project",
        "Search Templates",
    ];

    let selection = Select::new("Select template operation:", template_choices).prompt()?;

    match selection {
        "List Available Templates" => {
            let cmd = TemplateCommand::List;
            handle_template_mode(&cmd)?;
        }
        "Initialize Project from Template" => {
            let template = Text::new("Enter template name:").prompt()?;

            let target_input = Text::new("Enter target directory:")
                .with_default(".")
                .prompt()?;
            let target = expand_tilde(&target_input)?;

            let cmd = TemplateCommand::Init { template, target };
            handle_template_mode(&cmd)?;
        }
        "Create Template from Project" => {
            let name = Text::new("Enter template name:").prompt()?;

            let source_input = Text::new("Enter source project path:")
                .with_default(".")
                .prompt()?;
            let source = expand_tilde(&source_input)?;

            let cmd = TemplateCommand::Create { name, source };
            handle_template_mode(&cmd)?;
        }
        "Search Templates" => {
            let query = Text::new("Enter search query:").prompt()?;

            let cmd = TemplateCommand::Search { query };
            handle_template_mode(&cmd)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn handle_cache_operations() -> anyhow::Result<()> {
    println!("💾 Cache Operations");
    println!("Manage cached project detection data.\n");

    let cache_choices = vec![
        "Show Cache Statistics",
        "Clear All Cache",
        "Invalidate Specific Path",
    ];

    let selection = Select::new("Select cache operation:", cache_choices).prompt()?;

    match selection {
        "Show Cache Statistics" => {
            let cmd = CacheCommand::Stats;
            handle_cache_mode(&cmd)?;
        }
        "Clear All Cache" => {
            let confirm = Confirm::new("Are you sure you want to clear all cached data?")
                .with_default(false)
                .prompt()?;

            if confirm {
                let cmd = CacheCommand::Clear;
                handle_cache_mode(&cmd)?;
            } else {
                println!("Operation cancelled.");
            }
        }
        "Invalidate Specific Path" => {
            let path_input = Text::new("Enter path to invalidate:").prompt()?;
            let path = expand_tilde(&path_input)?;

            let cmd = CacheCommand::Invalidate { path };
            handle_cache_mode(&cmd)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn show_help() {
    println!("❓ App-Hoist Help");
    println!("=================");
    println!();
    println!("App-hoist is a dynamic CLI tool for managing packages, projects, and containers.");
    println!();
    println!("🎯 Main Features:");
    println!("  • Package Management: Hoist executables and packages system-wide");
    println!("  • Project Management: Manage development projects (Rust, Go, Python, JS/TS)");
    println!("  • Docker Operations: Direct Docker commands and containerized projects");
    println!("  • Multi-Project: Run operations across multiple projects in parallel");
    println!("  • Templates: Project scaffolding and boilerplate management");
    println!("  • Cache: Intelligent caching for fast project detection");
    println!();
    println!("💡 Pro Tips:");
    println!("  • Use dry-run mode to preview commands before execution");
    println!("  • Auto-detection works in project directories");
    println!("  • Multi-project operations run in parallel for speed");
    println!("  • Templates help you quickly scaffold new projects");
    println!();
    println!("📚 For more information, visit: https://github.com/sst/opencode");
    println!();
}

fn get_available_operations(project_type: &ProjectType) -> Vec<String> {
    match project_type {
        ProjectType::Rust => vec!["run", "build", "test", "check", "clippy", "install"],
        ProjectType::Go => vec!["run", "build", "test", "tidy", "get"],
        ProjectType::Uv | ProjectType::Venv | ProjectType::Generic => {
            vec!["run", "sync/add/remove"]
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            vec!["run", "install", "add", "test", "build"]
        }
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

fn detect_project_in_current_dir() -> anyhow::Result<Option<ProjectType>> {
    let current_dir = std::env::current_dir()?;

    // Quick detection logic (simplified version of what's in project.rs)
    let pyproject_path = current_dir.join("pyproject.toml");
    let uv_lock_path = current_dir.join("uv.lock");
    let activate_path = current_dir.join("bin").join("activate");
    let go_mod_path = current_dir.join("go.mod");
    let cargo_toml_path = current_dir.join("Cargo.toml");
    let package_json_path = current_dir.join("package.json");
    let tsconfig_path = current_dir.join("tsconfig.json");

    // Check for uv project
    if pyproject_path.exists() {
        let has_uv_section = std::fs::read_to_string(&pyproject_path)
            .map(|content| content.contains("[tool.uv]"))
            .unwrap_or(false);
        let has_uv_lock = uv_lock_path.exists();

        if has_uv_section || has_uv_lock {
            return Ok(Some(ProjectType::Uv));
        }
    }

    // Check for venv
    if activate_path.exists() {
        return Ok(Some(ProjectType::Venv));
    }

    // Check for Go project
    if go_mod_path.exists() {
        return Ok(Some(ProjectType::Go));
    }

    // Check for Rust project
    if cargo_toml_path.exists() {
        return Ok(Some(ProjectType::Rust));
    }

    // Check for JavaScript/TypeScript project
    if package_json_path.exists() {
        if tsconfig_path.exists() {
            return Ok(Some(ProjectType::TypeScript));
        } else {
            return Ok(Some(ProjectType::JavaScript));
        }
    }

    // Generic Python project
    if pyproject_path.exists() {
        return Ok(Some(ProjectType::Generic));
    }

    Ok(None)
}

// Re-export the handler functions from main.rs for reuse
fn handle_template_mode(command: &TemplateCommand) -> anyhow::Result<()> {
    match command {
        TemplateCommand::List => {
            let templates = template::list_available_templates()?;
            if templates.is_empty() {
                println!("No templates found. Create your first template with:");
                println!("  app-hoist template create <name>");
            } else {
                println!("Available templates:");
                for template in templates {
                    println!("  - {}", template);
                }
            }
        }
        TemplateCommand::Init { template, target } => {
            template::init_project_from_template(template, target)?;
        }
        TemplateCommand::Create { name, source } => {
            template::create_template_from_project(source, name)?;
        }
        TemplateCommand::Search { query } => {
            let templates = template::list_available_templates()?;
            let matches: Vec<_> = templates
                .into_iter()
                .filter(|t| t.to_lowercase().contains(&query.to_lowercase()))
                .collect();

            if matches.is_empty() {
                println!("No templates found matching '{}'", query);
            } else {
                println!("Templates matching '{}':", query);
                for template in matches {
                    println!("  - {}", template);
                }
            }
        }
    }
    Ok(())
}

fn handle_cache_mode(command: &CacheCommand) -> anyhow::Result<()> {
    let mut cache_manager = crate::cache::CacheManager::new()?;

    match command {
        CacheCommand::Stats => {
            let stats = cache_manager.stats();
            println!("{}", stats);
        }
        CacheCommand::Clear => {
            cache_manager.clear_all()?;
            println!("✅ All cache cleared");
        }
        CacheCommand::Invalidate { path } => {
            cache_manager.invalidate(path)?;
            println!("✅ Cache invalidated for: {}", path);
        }
    }

    Ok(())
}

fn expand_tilde(path: &str) -> anyhow::Result<String> {
    if path.starts_with("~") {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        Ok(path.replacen("~", &home, 1))
    } else {
        Ok(path.to_string())
    }
}

mod cache;
mod cli;
mod docker;
mod interactive;
mod models;
mod multi_project;
mod package;
mod project;
mod template;
mod utils;

use crate::cli::{AppCommand, Args, CacheCommand, TemplateCommand};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(app_cmd) = &args.command {
        match app_cmd {
            AppCommand::Template(template_cmd) => {
                // Template mode
                handle_template_mode(template_cmd)?;
            }
            AppCommand::Cache(cache_cmd) => {
                // Cache mode
                handle_cache_mode(cache_cmd)?;
            }
        }
    } else {
        match (
            &args.package,
            &args.path,
            &args.docker,
            &args.docker_path,
            &args.multi_path,
        ) {
            (Some(package), None, None, None, None) => {
                // Tool mode: hoist a package/executable
                package::handle_package_mode(package, args.dry_run)?;
            }
            (None, Some(path), None, None, None) => {
                // Project mode: manage a project (Python, Go, Rust, or JS/TS)
                project::handle_project_mode(path, args.dry_run)?;
            }
            (None, None, Some(cmd), None, None) => {
                // Direct Docker mode: execute Docker commands directly
                docker::handle_direct_docker_mode(cmd, args.dry_run)?;
            }
            (None, None, None, Some(path), None) => {
                // Docker project mode: manage Docker-enabled projects
                docker::handle_docker_project_mode(path, args.dry_run)?;
            }
            (None, None, None, None, Some(paths)) => {
                // Multi-project mode: run operations on multiple projects in parallel
                multi_project::handle_multi_project_mode(paths, args.dry_run).await?;
            }
            (None, None, None, None, None) => {
                // Interactive mode: no arguments provided, show interactive menu
                interactive::run_interactive_mode().await?;
            }
            _ => {
                anyhow::bail!(
                    "Invalid argument combination. Use:\n\
                      --package <name> for executables\n\
                      --path <directory> for projects\n\
                      --docker <command> for direct Docker operations\n\
                      --docker-path <directory> for Docker projects\n\
                      --multi-path <path1> <path2> ... for parallel operations\n\
                      template <subcommand> for template operations\n\
                      \n\
                      Or run without arguments for interactive mode."
                );
            }
        }
    }

    Ok(())
}

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

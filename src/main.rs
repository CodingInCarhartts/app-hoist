mod cli;
mod docker;
mod models;
mod package;
mod project;
mod template;
mod utils;

use crate::cli::{Args, TemplateCommand};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(template_cmd) = &args.template {
        // Template mode
        handle_template_mode(template_cmd)?;
    } else {
        match (&args.package, &args.path, &args.docker, &args.docker_path) {
            (Some(package), None, None, None) => {
                // Tool mode: hoist a package/executable
                package::handle_package_mode(package, args.dry_run)?;
            }
            (None, Some(path), None, None) => {
                // Project mode: manage a project (Python, Go, Rust, or JS/TS)
                project::handle_project_mode(path, args.dry_run)?;
            }
            (None, None, Some(cmd), None) => {
                // Direct Docker mode: execute Docker commands directly
                docker::handle_direct_docker_mode(cmd, args.dry_run)?;
            }
            (None, None, None, Some(path)) => {
                // Docker project mode: manage Docker-enabled projects
                docker::handle_docker_project_mode(path, args.dry_run)?;
            }
            _ => {
                anyhow::bail!(
                    "Invalid argument combination. Use:\n\
                     --package <name> for executables\n\
                     --path <directory> for projects\n\
                     --docker <command> for direct Docker operations\n\
                     --docker-path <directory> for Docker projects\n\
                     template <subcommand> for template operations"
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
            let matches: Vec<_> = templates.into_iter()
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

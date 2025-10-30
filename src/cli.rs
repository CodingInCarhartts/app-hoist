use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hoist")]
#[command(about = "Dynamic CLI command builder for packages and projects")]
pub struct Args {
    /// Name of the package/executable to hoist
    #[arg(short, long)]
    pub package: Option<String>,

    /// Path to the project directory
    #[arg(long)]
    pub path: Option<String>,

    /// Execute Docker commands directly
    #[arg(long)]
    pub docker: Option<String>,

    /// Manage Docker-enabled projects
    #[arg(long)]
    pub docker_path: Option<String>,

    /// Template operations
    #[command(subcommand)]
    pub template: Option<TemplateCommand>,

    /// Dry run: show the command without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,
    /// Initialize a project from a template
    Init {
        /// Name of the template to use
        template: String,
        /// Target directory for the new project
        #[arg(default_value = ".")]
        target: String,
    },
    /// Create a template from an existing project
    Create {
        /// Name for the new template
        name: String,
        /// Path to the project to create template from
        #[arg(default_value = ".")]
        source: String,
    },
    /// Search for templates
    Search {
        /// Search query
        query: String,
    },
}

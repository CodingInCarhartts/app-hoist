mod cli;
mod models;
mod package;
mod project;
mod utils;

use crate::cli::Args;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match (&args.package, &args.path) {
        (Some(package), None) => {
            // Tool mode: hoist a package/executable
            package::handle_package_mode(package, args.dry_run)?;
        }
        (None, Some(path)) => {
            // Project mode: manage a project (Python or Go)
            project::handle_project_mode(path, args.dry_run)?;
        }
        (Some(_), Some(_)) => {
            anyhow::bail!(
                "Cannot specify both --package and --path. Use --package for executables or --path for projects."
            );
        }
        (None, None) => {
            anyhow::bail!("Must specify either --package <name> or --path <directory>");
        }
    }

    Ok(())
}

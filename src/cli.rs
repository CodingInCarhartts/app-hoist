use clap::Parser;

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

    /// Dry run: show the command without executing
    #[arg(long)]
    pub dry_run: bool,
}

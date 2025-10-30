use clap::Parser;
use inquire::{MultiSelect, Text};
use regex::Regex;
use std::process::Command;

#[derive(Debug)]
struct OptionInfo {
    flags: Vec<String>,
    description: String,
    requires_value: bool,
}

#[derive(Parser)]
#[command(name = "hoist")]
#[command(about = "Dynamic CLI command builder for project executables")]
struct Args {
    /// Name of the project to hoist
    #[arg(short, long)]
    project_name: String,

    /// Dry run: show the command without executing
    #[arg(long)]
    dry_run: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Hoisting project: {}", args.project_name);

    // Discover the executable
    let executable = find_executable(&args.project_name)?;

    // Get help output
    let help_output = get_help_output(&executable)?;

    // Parse options from help
    let options = parse_options(&help_output)?;

    println!("Found {} options", options.len());

    let selected_options = if args.dry_run {
        println!("Dry run: skipping interactive selection, using no arguments.");
        Vec::new()
    } else if options.is_empty() {
        println!("No options found, proceeding with no arguments.");
        Vec::new()
    } else {
        // Interactive selection
        select_options(&options)?
    };

    // Build the command
    let command_args = build_command(&selected_options)?;

    // Execute the command
    if args.dry_run {
        println!("Dry run: {} {}", executable, command_args.join(" "));
    } else {
        execute_command(&executable, &command_args)?;
    }

    Ok(())
}

fn find_executable(name: &str) -> anyhow::Result<String> {
    // Try to run 'which' to find the executable
    let output = Command::new("which")
        .arg(name)
        .output()?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(path)
    } else {
        anyhow::bail!("Executable '{}' not found in PATH", name);
    }
}

fn get_help_output(executable: &str) -> anyhow::Result<String> {
    let output = Command::new(executable)
        .arg("--help")
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        anyhow::bail!("Failed to get help output from {}", executable);
    }
}

fn parse_options(help_text: &str) -> anyhow::Result<Vec<OptionInfo>> {
    let lines: Vec<&str> = help_text.lines().collect();
    let mut options = Vec::new();
    let mut in_options = false;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        if line.trim() == "Options:" {
            in_options = true;
            i += 1;
            continue;
        }

        if !in_options {
            i += 1;
            continue;
        }

        // Check if this is a flag line (starts with 2 spaces and contains -)
        if line.starts_with("  ") && line.trim().starts_with('-') {
            // Parse the flag line
            let flag_part = line.trim();
            let (flags, requires_value) = parse_flag_line(flag_part);

            // Collect description from subsequent lines
            let mut description = String::new();
            i += 1;
            while i < lines.len() && lines[i].starts_with("          ") {
                description.push_str(lines[i].trim());
                description.push(' ');
                i += 1;
            }

            if !flags.is_empty() {
                options.push(OptionInfo {
                    flags,
                    description: description.trim().to_string(),
                    requires_value,
                });
            }
        } else {
            i += 1;
        }
    }

    // If no options found with new method, try fallback regex for single-line formats
    if options.is_empty() {
        options = parse_options_fallback(help_text)?;
    }

    Ok(options)
}

fn parse_flag_line(line: &str) -> (Vec<String>, bool) {
    // Examples: "-c, --config <CONFIG>" or "--init"
    let mut flags = Vec::new();
    let mut requires_value = false;

    // Split by comma to handle multiple flags
    for part in line.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }

        // Split by space and take the flag part
        let flag = part.split_whitespace().next().unwrap_or(part);
        flags.push(flag.to_string());

        // Check if this part indicates a value is required
        if part.contains('<') {
            requires_value = true;
        }
    }

    (flags, requires_value)
}

fn parse_options_fallback(help_text: &str) -> anyhow::Result<Vec<OptionInfo>> {
    let mut options = Vec::new();

    // Fallback regex for single-line formats (like grep)
    let option_regex = Regex::new(r"^\s*([-\w\s,]+?)\s{2,}(.+)$")?;

    for line in help_text.lines() {
        if let Some(captures) = option_regex.captures(line) {
            let flags_str = captures.get(1).unwrap().as_str();
            let description = captures.get(2).unwrap().as_str().trim();

            // Split flags by comma and clean up
            let flags: Vec<String> = flags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !flags.is_empty() {
                // Simple heuristic: if description contains <value> or [value], it requires a value
                let requires_value = description.contains('<') || description.contains('[');

                options.push(OptionInfo {
                    flags,
                    description: description.to_string(),
                    requires_value,
                });
            }
        }
    }

    Ok(options)
}

fn select_options(options: &[OptionInfo]) -> anyhow::Result<Vec<(String, Option<String>)>> {
    // Create a list of option descriptions for selection
    let option_texts: Vec<String> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| format!("[{}] {}: {}", i, opt.flags.join(", "), opt.description))
        .collect();

    // Use MultiSelect to let user choose options
    let selected_texts = MultiSelect::new("Select options to include:", option_texts)
        .prompt()?;

    let mut selected = Vec::new();

    for text in selected_texts {
        // Extract the index from [idx]
        if let Some(start) = text.find('[') {
            if let Some(end) = text.find(']') {
                if let Ok(idx) = text[start+1..end].parse::<usize>() {
                    if let Some(opt) = options.get(idx) {
                        let flag = opt.flags[0].clone(); // Use the first flag

                        let value = if opt.requires_value {
                            // Ask for value
                            Some(Text::new(&format!("Enter value for {}:", flag)).prompt()?)
                        } else {
                            None
                        };

                        selected.push((flag, value));
                    }
                }
            }
        }
    }

    Ok(selected)
}

fn build_command(selected: &[(String, Option<String>)]) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    for (flag, value) in selected {
        args.push(flag.clone());
        if let Some(val) = value {
            args.push(val.clone());
        }
    }

    Ok(args)
}

fn execute_command(executable: &str, args: &[String]) -> anyhow::Result<()> {
    println!("Executing: {} {}", executable, args.join(" "));

    let mut command = Command::new(executable);
    command.args(args);

    let status = command.status()?;

    if status.success() {
        println!("Command executed successfully");
    } else {
        println!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

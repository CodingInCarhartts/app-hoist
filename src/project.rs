use crate::models::{OptionInfo, ProjectType};
use crate::utils::{execute_project_command, select_options};

pub fn handle_project_mode(path: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("Managing project: {}", path);

    // Detect project type
    let project_type = detect_project_type(path)?;

    // Detect entry point
    let entry_point = detect_entry_point(path)?;

    // Get options based on type
    let options = get_project_options(&project_type, &entry_point)?;

    println!(
        "Detected {} project with {} options",
        project_type,
        options.len()
    );

    let selected_options = if dry_run {
        println!("Dry run: skipping interactive selection, using no arguments.");
        Vec::new()
    } else if options.is_empty() {
        println!("No options available, proceeding with no arguments.");
        Vec::new()
    } else {
        // Interactive selection
        select_options(&options)?
    };

    // Build the command
    let (executable, command_args) = build_project_command(&project_type, path, &selected_options)?;

    // Execute the command
    if command_args.is_empty() {
        println!("No command to execute. Select options to perform actions.");
    } else if dry_run {
        println!("Dry run: {} {}", executable, command_args.join(" "));
    } else {
        execute_project_command(&executable, &command_args, path)?;
    }

    Ok(())
}

fn detect_project_type(path: &str) -> anyhow::Result<ProjectType> {
    // Check for uv project
    let pyproject_path = format!("{}/pyproject.toml", path);
    let uv_lock_path = format!("{}/uv.lock", path);
    if std::path::Path::new(&pyproject_path).exists() {
        // Check for [tool.uv] section OR uv.lock file
        let has_uv_section = std::fs::read_to_string(&pyproject_path)
            .map(|content| content.contains("[tool.uv]"))
            .unwrap_or(false);
        let has_uv_lock = std::path::Path::new(&uv_lock_path).exists();

        if has_uv_section || has_uv_lock {
            return Ok(ProjectType::Uv);
        }
    }

    // Check for venv
    let activate_path = format!("{}/bin/activate", path);
    if std::path::Path::new(&activate_path).exists() {
        return Ok(ProjectType::Venv);
    }

    // Generic Python project
    if std::path::Path::new(&pyproject_path).exists() {
        return Ok(ProjectType::Generic);
    }

    Ok(ProjectType::Generic)
}

fn detect_entry_point(path: &str) -> anyhow::Result<String> {
    let candidates = ["app.py", "main.py", "__main__.py"];

    for candidate in &candidates {
        let full_path = format!("{}/{}", path, candidate);
        if std::path::Path::new(&full_path).exists() {
            return Ok(candidate.to_string());
        }
    }

    // Default to app.py if none found
    Ok("app.py".to_string())
}

fn get_project_options(
    project_type: &ProjectType,
    entry_point: &str,
) -> anyhow::Result<Vec<OptionInfo>> {
    let mut options = Vec::new();

    match project_type {
        ProjectType::Uv => {
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({})", entry_point),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["sync".to_string()],
                description: "Sync dependencies".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["add".to_string()],
                description: "Install a package".to_string(),
                requires_value: true,
            });
            options.push(OptionInfo {
                flags: vec!["remove".to_string()],
                description: "Uninstall a package".to_string(),
                requires_value: true,
            });
        }
        ProjectType::Venv => {
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({})", entry_point),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["install".to_string()],
                description: "Install a package".to_string(),
                requires_value: true,
            });
            options.push(OptionInfo {
                flags: vec!["uninstall".to_string()],
                description: "Uninstall a package".to_string(),
                requires_value: true,
            });
        }
        ProjectType::Generic => {
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({})", entry_point),
                requires_value: false,
            });
        }
    }

    Ok(options)
}

fn build_project_command(
    project_type: &ProjectType,
    path: &str,
    selected: &[(String, Option<String>)],
) -> anyhow::Result<(String, Vec<String>)> {
    match project_type {
        ProjectType::Uv => {
            if selected.iter().any(|(flag, _)| flag == "run") {
                // For run command, use uv run <entry_point>
                let entry_point = detect_entry_point(path)?;
                Ok(("uv".to_string(), vec!["run".to_string(), entry_point]))
            } else {
                // For other commands (sync, add, etc.), use uv --project <path> <command>
                let mut args = vec!["--project".to_string(), path.to_string()];
                for (flag, value) in selected {
                    args.push(flag.clone());
                    if let Some(val) = value {
                        args.push(val.clone());
                    }
                }
                Ok(("uv".to_string(), args))
            }
        }
        ProjectType::Venv => {
            let mut command_parts = Vec::new();
            for (flag, value) in selected {
                match flag.as_str() {
                    "run" => {
                        command_parts.push(format!("python {}", detect_entry_point(path)?));
                    }
                    "install" => {
                        if let Some(pkg) = value {
                            command_parts.push(format!("pip install {}", pkg));
                        }
                    }
                    "uninstall" => {
                        if let Some(pkg) = value {
                            command_parts.push(format!("pip uninstall {}", pkg));
                        }
                    }
                    _ => {}
                }
            }
            let full_command = format!("source bin/activate && {}", command_parts.join(" && "));
            Ok(("bash".to_string(), vec!["-c".to_string(), full_command]))
        }
        ProjectType::Generic => {
            let mut args = Vec::new();
            for (flag, _) in selected {
                if flag == "run" {
                    args.push(detect_entry_point(path)?.to_string());
                }
            }
            Ok(("python".to_string(), args))
        }
    }
}

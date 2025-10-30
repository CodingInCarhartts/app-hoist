use crate::models::{OptionInfo, ProjectType};
use crate::utils::{execute_project_command, select_options};

pub fn handle_project_mode(path: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("Managing project: {}", path);

    // Detect project type
    let project_type = detect_project_type(path)?;

    // Detect entry point
    let entry_point = detect_entry_point(path)?;

    // Get options based on type
    let options = get_project_options(&project_type, &entry_point, path)?;

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
        // Special handling for Go build command
        if project_type == ProjectType::Go && selected_options.iter().any(|(flag, _)| flag == "build") {
            execute_go_build_with_install(&executable, &command_args, path)?;
        } else {
            execute_project_command(&executable, &command_args, path)?;
        }
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

    // Check for Go project
    let go_mod_path = format!("{}/go.mod", path);
    if std::path::Path::new(&go_mod_path).exists() {
        return Ok(ProjectType::Go);
    }

    // Check for Rust project
    let cargo_toml_path = format!("{}/Cargo.toml", path);
    if std::path::Path::new(&cargo_toml_path).exists() {
        return Ok(ProjectType::Rust);
    }

    // Check for JavaScript/TypeScript project
    let package_json_path = format!("{}/package.json", path);
    if std::path::Path::new(&package_json_path).exists() {
        // Check for TypeScript
        let tsconfig_path = format!("{}/tsconfig.json", path);
        if std::path::Path::new(&tsconfig_path).exists() {
            return Ok(ProjectType::TypeScript);
        } else {
            return Ok(ProjectType::JavaScript);
        }
    }

    // Generic Python project
    if std::path::Path::new(&pyproject_path).exists() {
        return Ok(ProjectType::Generic);
    }

    Ok(ProjectType::Generic)
}

fn detect_entry_point(path: &str) -> anyhow::Result<String> {
    // Check if this is a Go project
    let go_mod_path = format!("{}/go.mod", path);
    if std::path::Path::new(&go_mod_path).exists() {
        let go_candidates = ["main.go", "cmd/main.go"];

        for candidate in &go_candidates {
            let full_path = format!("{}/{}", path, candidate);
            if std::path::Path::new(&full_path).exists() {
                return Ok(candidate.to_string());
            }
        }

        // Default to current directory for Go
        return Ok(".".to_string());
    }

    // Check if this is a Rust project
    let cargo_toml_path = format!("{}/Cargo.toml", path);
    if std::path::Path::new(&cargo_toml_path).exists() {
        return Ok(".".to_string());  // Run current directory for Rust
    }

    // Check if this is a JavaScript/TypeScript project
    let package_json_path = format!("{}/package.json", path);
    if std::path::Path::new(&package_json_path).exists() {
        return Ok(".".to_string());  // Run with package manager
    }

    // Python project detection
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
    path: &str,
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
        ProjectType::Go => {
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({})", entry_point),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build and install the application".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["test".to_string()],
                description: "Run tests".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["tidy".to_string()],
                description: "Clean up dependencies".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["get".to_string()],
                description: "Add a dependency".to_string(),
                requires_value: true,
            });
        }
        ProjectType::Rust => {
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({})", entry_point),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build the project".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["install".to_string()],
                description: "Build and install to ~/.cargo/bin".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["test".to_string()],
                description: "Run tests".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["check".to_string()],
                description: "Check code without building".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["clippy".to_string()],
                description: "Run linter".to_string(),
                requires_value: false,
            });
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            let pm = detect_package_manager(path);
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: format!("Run the app ({} start)", pm),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["install".to_string()],
                description: format!("Install dependencies ({} install)", pm),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["add".to_string()],
                description: format!("Add package ({} add)", pm),
                requires_value: true,
            });
            options.push(OptionInfo {
                flags: vec!["test".to_string()],
                description: format!("Run tests ({} test)", pm),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: format!("Build project ({} run build)", pm),
                requires_value: false,
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
        ProjectType::Go => {
            let mut args = Vec::new();
            for (flag, value) in selected {
                match flag.as_str() {
                    "run" => {
                        args.push("run".to_string());
                        args.push(detect_entry_point(path)?);
                    }
                    "build" => {
                        // For build, we'll handle this specially in execution
                        args.push("build".to_string());
                        args.push("-o".to_string());
                        let binary_name = detect_binary_name(path)?;
                        let temp_path = format!("/tmp/{}", binary_name);
                        args.push(temp_path);
                        args.push(".".to_string());
                    }
                    "test" => {
                        args.push("test".to_string());
                        args.push("./...".to_string());
                    }
                    "tidy" => {
                        args.push("mod".to_string());
                        args.push("tidy".to_string());
                    }
                    "get" => {
                        if let Some(pkg) = value {
                            args.push("get".to_string());
                            args.push(pkg.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ok(("go".to_string(), args))
        }
        ProjectType::Rust => {
            let mut args = Vec::new();
            for (flag, _) in selected {
                match flag.as_str() {
                    "run" => {
                        args.push("run".to_string());
                        args.push("--bin".to_string());
                        args.push(detect_rust_binary_name(path)?);
                    }
                    "build" => {
                        args.push("build".to_string());
                        args.push("--release".to_string());
                    }
                    "install" => {
                        args.push("install".to_string());
                        args.push("--path".to_string());
                        args.push(".".to_string());
                    }
                    "test" => {
                        args.push("test".to_string());
                    }
                    "check" => {
                        args.push("check".to_string());
                    }
                    "clippy" => {
                        args.push("clippy".to_string());
                    }
                    _ => {}
                }
            }
            Ok(("cargo".to_string(), args))
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            let pm = detect_package_manager(path);
            let mut args = vec![pm];

            for (flag, value) in selected {
                match flag.as_str() {
                    "run" => {
                        args.push("start".to_string());
                    }
                    "install" => {
                        args.push("install".to_string());
                    }
                    "add" => {
                        args.push("add".to_string());
                        if let Some(pkg) = value {
                            args.push(pkg.clone());
                        }
                    }
                    "test" => {
                        args.push("test".to_string());
                    }
                    "build" => {
                        args.push("run".to_string());
                        args.push("build".to_string());
                    }
                    _ => {}
                }
            }
            Ok(("npx".to_string(), args))
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

fn detect_binary_name(path: &str) -> anyhow::Result<String> {
    // Try to read from go.mod
    let go_mod_path = format!("{}/go.mod", path);
    if let Ok(content) = std::fs::read_to_string(&go_mod_path) {
        // Parse module name from "module github.com/user/repo"
        for line in content.lines() {
            if line.starts_with("module ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // Extract repo name from github.com/user/repo
                    let module_path = parts[1];
                    if let Some(repo_name) = module_path.split('/').next_back() {
                        return Ok(repo_name.to_string());
                    }
                }
            }
        }
    }

    // Fallback: use directory name
    let dir_name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app");

    Ok(dir_name.to_string())
}

fn extract_binary_path_from_args(args: &[String]) -> anyhow::Result<String> {
    // Find the -o flag and get the next argument
    for (i, arg) in args.iter().enumerate() {
        if arg == "-o" && i + 1 < args.len() {
            return Ok(args[i + 1].clone());
        }
    }
    anyhow::bail!("Could not find output path in build arguments");
}

fn detect_package_manager(path: &str) -> String {
    // Check for lock files to determine package manager
    let yarn_lock = format!("{}/yarn.lock", path);
    let pnpm_lock = format!("{}/pnpm-lock.yaml", path);
    let _package_lock = format!("{}/package-lock.json", path);

    if std::path::Path::new(&yarn_lock).exists() {
        "yarn".to_string()
    } else if std::path::Path::new(&pnpm_lock).exists() {
        "pnpm".to_string()
    } else {
        "npm".to_string()  // default
    }
}

fn detect_rust_binary_name(path: &str) -> anyhow::Result<String> {
    // Parse from Cargo.toml [package] name
    let cargo_toml_path = format!("{}/Cargo.toml", path);
    if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name = ") {
                let name_part = trimmed.split_once('=').unwrap().1.trim();
                let name = name_part.trim_matches('"').trim_matches('\'');
                return Ok(name.to_string());
            }
        }
    }

    // Fallback to directory name
    let dir_name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app");

    Ok(dir_name.to_string())
}

fn execute_go_build_with_install(executable: &str, args: &[String], path: &str) -> anyhow::Result<()> {
    use std::process::Command;

    // Step 1: Build the binary
    println!("Building Go application...");
    let mut build_cmd = Command::new(executable);
    build_cmd.args(args).current_dir(path);

    let build_status = build_cmd.status()?;
    if !build_status.success() {
        anyhow::bail!("Build failed with exit code: {:?}", build_status.code());
    }

    // Step 2: Detect the binary path from the build command
    let binary_path = extract_binary_path_from_args(args)?;

    // Step 3: Determine final installation name
    let install_name = detect_binary_name(path)?;
    let install_path = format!("/usr/bin/{}", install_name);

    // Step 4: Check if binary exists before moving
    if !std::path::Path::new(&binary_path).exists() {
        anyhow::bail!("Built binary not found at: {}", binary_path);
    }

    // Step 5: Move to /usr/bin (requires sudo)
    println!("Installing {} to {}...", install_name, install_path);
    let install_status = Command::new("sudo")
        .args(["mv", &binary_path, &install_path])
        .status()?;

    if !install_status.success() {
        anyhow::bail!("Installation failed. You may need to run with sudo or check permissions.");
    }

    // Step 6: Verify installation
    let which_output = Command::new("which").arg(&install_name).output()?;
    if which_output.status.success() {
        println!("✅ Successfully installed {} and added to PATH!", install_name);
        println!("You can now run: {}", install_name);
    } else {
        println!("⚠️  Binary installed but may not be in PATH. Try: export PATH=$PATH:/usr/bin");
    }

    Ok(())
}

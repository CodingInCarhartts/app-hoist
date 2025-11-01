use crate::cache::{CacheManager, ProjectCache};
use crate::models::{OptionInfo, ProjectType};
use crate::utils::{execute_project_command_async, select_options};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn handle_multi_project_mode(paths: &[String], dry_run: bool) -> anyhow::Result<()> {
    println!("Managing {} projects in parallel", paths.len());

    // Initialize cache manager
    let mut cache_manager = CacheManager::new()?;

    // Create multi-progress bar
    let multi_progress = Arc::new(MultiProgress::new());

    // Limit concurrent operations to prevent overwhelming the system
    let semaphore = Arc::new(Semaphore::new(num_cpus::get()));

    // Collect all project information
    let mut project_infos = Vec::new();
    for path in paths {
        let path_clone = path.clone();

        // Try to get cached project info first
        let cached_info = if let Ok(Some(cached)) = cache_manager.get(&path_clone) {
            Some((path_clone.clone(), cached.project_type, cached.entry_point))
        } else {
            None
        };

        let project_info = if let Some(info) = cached_info {
            info
        } else {
            // Detect project type and entry point
            let project_type = detect_project_type(&path_clone)?;
            let entry_point = detect_entry_point(&path_clone)?;

            // Cache the results
            let cache = ProjectCache::new(project_type.clone(), entry_point.clone());
            let _ = cache_manager.set(path_clone.clone(), cache);

            (path_clone, project_type, entry_point)
        };

        project_infos.push(project_info);
    }

    // Get common options across all projects (intersection of available options)
    let common_options = if project_infos.is_empty() {
        Vec::new()
    } else {
        let (first_path, first_type, first_entry) = &project_infos[0];

        let mut common_opts = get_project_options(first_type, first_entry, first_path)?;

        // Filter to only options that exist in all projects
        for (path, project_type, entry_point) in &project_infos[1..] {
            let project_opts = get_project_options(project_type, entry_point, path)?;
            let project_flags: std::collections::HashSet<_> = project_opts
                .iter()
                .flat_map(|opt| opt.flags.iter())
                .collect();

            common_opts.retain(|opt| opt.flags.iter().any(|flag| project_flags.contains(flag)));
        }

        common_opts
    };

    println!(
        "Found {} common operations across all projects",
        common_options.len()
    );

    let selected_options = if dry_run {
        println!("Dry run: skipping interactive selection, using no arguments.");
        Vec::new()
    } else if common_options.is_empty() {
        println!("No common options available, proceeding with no arguments.");
        Vec::new()
    } else {
        // Interactive selection
        select_options(&common_options)?
    };

    if selected_options.is_empty() {
        println!("No operations selected. Exiting.");
        return Ok(());
    }

    // Execute operations in parallel
    let mut handles = Vec::new();

    for project_info in project_infos {
        let (path, project_type, entry_point) = project_info;
        let selected_opts = selected_options.clone();
        let dry_run_flag = dry_run;
        let multi_pb = Arc::clone(&multi_progress);
        let sem = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Create progress bar for this project
            let pb = multi_pb.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap(),
            );
            pb.set_message(format!("Processing {}", path));

            let result = execute_project_operations(
                &path,
                &project_type,
                &entry_point,
                &selected_opts,
                dry_run_flag,
                &pb,
            )
            .await;

            match &result {
                Ok(_) => {
                    pb.finish_with_message(format!("✅ {} completed", path));
                }
                Err(e) => {
                    pb.finish_with_message(format!("❌ {} failed: {}", path, e));
                }
            }

            result
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }

    // Check for failures
    let failures: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();

    if failures.is_empty() {
        println!("✅ All operations completed successfully!");
    } else {
        println!("❌ {} operations failed", failures.len());
        for failure in failures {
            eprintln!("Error: {}", failure);
        }
        anyhow::bail!("Some operations failed");
    }

    Ok(())
}

async fn execute_project_operations(
    path: &str,
    project_type: &ProjectType,
    _entry_point: &str,
    selected_options: &[(String, Option<String>)],
    dry_run: bool,
    pb: &ProgressBar,
) -> anyhow::Result<()> {
    // Build command for this project type
    let (executable, args) = build_project_command(project_type, path, selected_options)?;

    if args.is_empty() {
        pb.set_message(format!("{}: No command to execute", path));
        return Ok(());
    }

    if dry_run {
        pb.set_message(format!(
            "{}: Dry run - {} {}",
            path,
            executable,
            args.join(" ")
        ));
        return Ok(());
    }

    // Execute the command asynchronously
    execute_project_command_async(&executable, &args, path, pb).await?;

    Ok(())
}

fn build_project_command(
    project_type: &ProjectType,
    path: &str,
    selected: &[(String, Option<String>)],
) -> anyhow::Result<(String, Vec<String>)> {
    // This is a simplified version - we could reuse the logic from project.rs
    // but for now, let's implement basic support for common operations

    match project_type {
        ProjectType::Rust => {
            let mut args = Vec::new();
            for (flag, _) in selected {
                match flag.as_str() {
                    "build" => {
                        args.push("build".to_string());
                        args.push("--release".to_string());
                    }
                    "test" => {
                        args.push("test".to_string());
                    }
                    "check" => {
                        args.push("check".to_string());
                    }
                    _ => {}
                }
            }
            if !args.is_empty() {
                Ok(("cargo".to_string(), args))
            } else {
                Ok(("cargo".to_string(), vec![]))
            }
        }
        ProjectType::Go => {
            let mut args = Vec::new();
            for (flag, _) in selected {
                match flag.as_str() {
                    "build" => {
                        args.push("build".to_string());
                        args.push(".".to_string());
                    }
                    "test" => {
                        args.push("test".to_string());
                        args.push("./...".to_string());
                    }
                    _ => {}
                }
            }
            if !args.is_empty() {
                Ok(("go".to_string(), args))
            } else {
                Ok(("go".to_string(), vec![]))
            }
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            let pm = detect_package_manager(path);
            let mut args = vec![pm];

            for (flag, _) in selected {
                match flag.as_str() {
                    "install" => {
                        args.push("install".to_string());
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
            if args.len() > 1 {
                Ok(("npx".to_string(), args))
            } else {
                Ok(("npx".to_string(), vec![]))
            }
        }
        ProjectType::Uv => {
            let mut args = vec!["--project".to_string(), path.to_string()];
            for (flag, _) in selected {
                if flag.as_str() == "sync" {
                    args.push("sync".to_string());
                }
            }
            if args.len() > 2 {
                Ok(("uv".to_string(), args))
            } else {
                Ok(("uv".to_string(), vec![]))
            }
        }
        _ => {
            // For other project types, return empty command for now
            Ok(("".to_string(), vec![]))
        }
    }
}

fn detect_package_manager(path: &str) -> String {
    // Check for lock files to determine package manager
    let yarn_lock = format!("{}/yarn.lock", path);
    let pnpm_lock = format!("{}/pnpm-lock.yaml", path);

    if std::path::Path::new(&yarn_lock).exists() {
        "yarn".to_string()
    } else if std::path::Path::new(&pnpm_lock).exists() {
        "pnpm".to_string()
    } else {
        "npm".to_string() // default
    }
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
        return Ok(".".to_string()); // Run current directory for Rust
    }

    // Check if this is a JavaScript/TypeScript project
    let package_json_path = format!("{}/package.json", path);
    if std::path::Path::new(&package_json_path).exists() {
        return Ok(".".to_string()); // Run with package manager
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
    _entry_point: &str,
    _path: &str,
) -> anyhow::Result<Vec<OptionInfo>> {
    let mut options = Vec::new();

    match project_type {
        ProjectType::Uv => {
            options.push(OptionInfo {
                flags: vec!["sync".to_string()],
                description: "Sync dependencies".to_string(),
                requires_value: false,
            });
        }
        ProjectType::Go => {
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build the application".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["test".to_string()],
                description: "Run tests".to_string(),
                requires_value: false,
            });
        }
        ProjectType::Rust => {
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build the project".to_string(),
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
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            options.push(OptionInfo {
                flags: vec!["install".to_string()],
                description: "Install dependencies".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["test".to_string()],
                description: "Run tests".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build project".to_string(),
                requires_value: false,
            });
        }
        _ => {} // No common options for other types
    }

    Ok(options)
}

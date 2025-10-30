use crate::models::OptionInfo;
use crate::utils::{select_options, execute_project_command};
use anyhow::anyhow;
use std::path::Path;
use std::process::Command;

pub fn handle_direct_docker_mode(command: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("Executing Docker command: {}", command);

    if dry_run {
        println!("Dry run: docker {}", command);
        return Ok(());
    }

    // Parse the command and execute it
    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return Err(anyhow!("Empty Docker command"));
    }

    let mut docker_cmd = Command::new("docker");
    docker_cmd.args(&args[1..]); // Skip "docker" if it was included

    let status = docker_cmd.status()?;
    if !status.success() {
        return Err(anyhow!("Docker command failed with exit code: {:?}", status.code()));
    }

    Ok(())
}

pub fn handle_docker_project_mode(path: &str, dry_run: bool) -> anyhow::Result<()> {
    println!("Managing Docker project: {}", path);

    // Detect Docker context
    let context = detect_docker_context(path)?;

    // Get options based on context
    let options = get_docker_options(&context)?;

    println!("Detected {} Docker setup with {} options", context, options.len());

    let selected_options = if dry_run {
        println!("Dry run: skipping interactive selection, using no arguments.");
        Vec::new()
    } else if options.is_empty() {
        println!("No Docker options available.");
        Vec::new()
    } else {
        select_options(&options)?
    };

    // Build and execute commands
    for (flag, value) in selected_options {
        let (command, args) = build_docker_command(&context, path, &flag, value.as_deref())?;

        if dry_run {
            println!("Dry run: {} {}", command, args.join(" "));
        } else {
            execute_project_command(&command, &args, path)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum DockerContext {
    SingleImage,
    Compose,
    Hybrid,
}

impl std::fmt::Display for DockerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockerContext::SingleImage => write!(f, "single image"),
            DockerContext::Compose => write!(f, "compose"),
            DockerContext::Hybrid => write!(f, "hybrid"),
        }
    }
}

fn detect_docker_context(path: &str) -> anyhow::Result<DockerContext> {
    let dockerfile_path = format!("{}/Dockerfile", path);
    let compose_path = format!("{}/docker-compose.yml", path);

    let has_dockerfile = Path::new(&dockerfile_path).exists();
    let has_compose = Path::new(&compose_path).exists();

    match (has_dockerfile, has_compose) {
        (true, true) => Ok(DockerContext::Hybrid),
        (true, false) => Ok(DockerContext::SingleImage),
        (false, true) => Ok(DockerContext::Compose),
        (false, false) => Err(anyhow!("No Dockerfile or docker-compose.yml found in {}", path)),
    }
}

fn get_docker_options(context: &DockerContext) -> anyhow::Result<Vec<OptionInfo>> {
    let mut options = Vec::new();

    match context {
        DockerContext::SingleImage => {
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build Docker image".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["run".to_string()],
                description: "Run container".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["shell".to_string()],
                description: "Access container shell".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["logs".to_string()],
                description: "Show container logs".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["push".to_string()],
                description: "Push image to registry".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["pull".to_string()],
                description: "Pull image from registry".to_string(),
                requires_value: false,
            });
        }
        DockerContext::Compose => {
            options.push(OptionInfo {
                flags: vec!["up".to_string()],
                description: "Start services".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["down".to_string()],
                description: "Stop services".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["build".to_string()],
                description: "Build services".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["logs".to_string()],
                description: "Show service logs".to_string(),
                requires_value: false,
            });
            options.push(OptionInfo {
                flags: vec!["shell".to_string()],
                description: "Access service shell".to_string(),
                requires_value: true, // service name
            });
        }
        DockerContext::Hybrid => {
            // Include both single image and compose options
            options.extend(get_docker_options(&DockerContext::SingleImage)?);
            options.extend(get_docker_options(&DockerContext::Compose)?);
        }
    }

    Ok(options)
}

fn build_docker_command(
    context: &DockerContext,
    path: &str,
    flag: &str,
    value: Option<&str>,
) -> anyhow::Result<(String, Vec<String>)> {
    match context {
        DockerContext::SingleImage => {
            let image_name = generate_image_name(path);
            match flag {
                "build" => Ok(("docker".to_string(), vec!["build".to_string(), "-t".to_string(), image_name, ".".to_string()])),
                "run" => Ok(("docker".to_string(), vec!["run".to_string(), "-it".to_string(), "--rm".to_string(), image_name])),
                "shell" => Ok(("docker".to_string(), vec!["run".to_string(), "-it".to_string(), "--rm".to_string(), image_name, "/bin/bash".to_string()])),
                "logs" => {
                    // For logs, we need to find the running container
                    // This is a simplified version - in practice you'd need to track container names
                    Ok(("docker".to_string(), vec!["ps".to_string(), "-f".to_string(), format!("ancestor={}", image_name)]))
                }
                "push" => Ok(("docker".to_string(), vec!["push".to_string(), image_name])),
                "pull" => Ok(("docker".to_string(), vec!["pull".to_string(), image_name])),
                _ => Err(anyhow!("Unknown Docker command: {}", flag)),
            }
        }
        DockerContext::Compose => {
            match flag {
                "up" => Ok(("docker-compose".to_string(), vec!["up".to_string(), "-d".to_string()])),
                "down" => Ok(("docker-compose".to_string(), vec!["down".to_string()])),
                "build" => Ok(("docker-compose".to_string(), vec!["build".to_string()])),
                "logs" => Ok(("docker-compose".to_string(), vec!["logs".to_string(), "-f".to_string()])),
                "shell" => {
                    if let Some(service) = value {
                        Ok(("docker-compose".to_string(), vec!["exec".to_string(), service.to_string(), "/bin/bash".to_string()]))
                    } else {
                        Err(anyhow!("Service name required for shell command"))
                    }
                }
                _ => Err(anyhow!("Unknown Docker Compose command: {}", flag)),
            }
        }
        DockerContext::Hybrid => {
            // For hybrid, try compose first, then fall back to single image
            if matches!(flag, "up" | "down" | "build" | "logs" | "shell") {
                build_docker_command(&DockerContext::Compose, path, flag, value)
            } else {
                build_docker_command(&DockerContext::SingleImage, path, flag, value)
            }
        }
    }
}

fn generate_image_name(path: &str) -> String {
    let dir_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("app");

    format!("{}-app", dir_name.to_lowercase())
}
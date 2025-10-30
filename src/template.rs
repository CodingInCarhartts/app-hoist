use anyhow::anyhow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub name: String,
    pub description: String,
    pub language: String,
    pub tags: Vec<String>,
    pub variables: HashMap<String, TemplateVariable>,
}

#[derive(Debug, Clone)]
pub struct TemplateVariable {
    pub description: String,
    pub default: String,
}

pub fn list_available_templates() -> anyhow::Result<Vec<String>> {
    let template_dir = get_template_dir()?;
    if !template_dir.exists() {
        return Ok(vec![]);
    }

    let mut templates = vec![];
    for entry in fs::read_dir(template_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                templates.push(name.to_string());
            }
        }
    }

    Ok(templates)
}

pub fn init_project_from_template(template_name: &str, target_path: &str) -> anyhow::Result<()> {
    let template_dir = get_template_dir()?.join(template_name);
    if !template_dir.exists() {
        return Err(anyhow!("Template '{}' not found", template_name));
    }

    // Load template config
    let config_path = template_dir.join("template.toml");
    let config = if config_path.exists() {
        load_template_config(&config_path)?
    } else {
        // Create default config
        TemplateConfig {
            name: template_name.to_string(),
            description: format!("{} template", template_name),
            language: "unknown".to_string(),
            tags: vec![],
            variables: HashMap::new(),
        }
    };

    // Collect variable values
    let variables = collect_template_variables(&config)?;

    // Copy and process template files
    copy_template_files(&template_dir, target_path, &variables)?;

    println!("✅ Successfully initialized project from template '{}'", template_name);
    println!("📁 Project created at: {}", target_path);

    Ok(())
}

pub fn create_template_from_project(project_path: &str, template_name: &str) -> anyhow::Result<()> {
    let template_dir = get_template_dir()?.join(template_name);
    if template_dir.exists() {
        return Err(anyhow!("Template '{}' already exists", template_name));
    }

    // Create template directory
    fs::create_dir_all(&template_dir)?;

    // Copy project files (excluding common ignore patterns)
    copy_project_to_template(project_path, &template_dir)?;

    // Create basic template config
    let config = TemplateConfig {
        name: template_name.to_string(),
        description: format!("Template created from {}", project_path),
        language: detect_project_language(project_path)?,
        tags: vec!["custom".to_string()],
        variables: HashMap::new(),
    };

    save_template_config(&template_dir.join("template.toml"), &config)?;

    println!("✅ Successfully created template '{}' from project", template_name);
    println!("📁 Template stored at: {}", template_dir.display());

    Ok(())
}

fn get_template_dir() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".app-hoist").join("templates"))
}

fn load_template_config(path: &Path) -> anyhow::Result<TemplateConfig> {
    let content = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&content)?;

    let name = value.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let description = value.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let language = value.get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let tags = value.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect()
        )
        .unwrap_or_default();

    let variables = if let Some(vars_table) = value.get("variables").and_then(|v| v.as_table()) {
        let mut vars = HashMap::new();
        for (key, var_value) in vars_table {
            if let Some(var_table) = var_value.as_table() {
                let var_desc = var_table.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let var_default = var_table.get("default")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                vars.insert(key.clone(), TemplateVariable {
                    description: var_desc,
                    default: var_default,
                });
            }
        }
        vars
    } else {
        HashMap::new()
    };

    Ok(TemplateConfig {
        name,
        description,
        language,
        tags,
        variables,
    })
}

fn save_template_config(path: &Path, config: &TemplateConfig) -> anyhow::Result<()> {
    let mut value = toml::value::Table::new();

    value.insert("name".to_string(), toml::Value::String(config.name.clone()));
    value.insert("description".to_string(), toml::Value::String(config.description.clone()));
    value.insert("language".to_string(), toml::Value::String(config.language.clone()));

    let tags_array: Vec<toml::Value> = config.tags.iter()
        .map(|tag| toml::Value::String(tag.clone()))
        .collect();
    value.insert("tags".to_string(), toml::Value::Array(tags_array));

    let mut vars_table = toml::value::Table::new();
    for (key, var) in &config.variables {
        let mut var_table = toml::value::Table::new();
        var_table.insert("description".to_string(), toml::Value::String(var.description.clone()));
        var_table.insert("default".to_string(), toml::Value::String(var.default.clone()));
        vars_table.insert(key.clone(), toml::Value::Table(var_table));
    }
    value.insert("variables".to_string(), toml::Value::Table(vars_table));

    let content = toml::to_string_pretty(&toml::Value::Table(value))?;
    fs::write(path, content)?;

    Ok(())
}

fn collect_template_variables(config: &TemplateConfig) -> anyhow::Result<HashMap<String, String>> {
    let mut variables = HashMap::new();

    // Add built-in variables
    variables.insert("project_name".to_string(), config.name.clone());
    variables.insert("year".to_string(), chrono::Utc::now().format("%Y").to_string());

    // Collect user-defined variables
    for (key, var_config) in &config.variables {
        let value = inquire::Text::new(&var_config.description)
            .with_default(&var_config.default)
            .prompt()
            .unwrap_or_else(|_| {
                // Fallback to default value if interactive prompt fails
                println!("Using default value for '{}': {}", key, var_config.default);
                var_config.default.clone()
            });
        variables.insert(key.clone(), value);
    }

    Ok(variables)
}

fn copy_template_files(template_dir: &Path, target_path: &str, variables: &HashMap<String, String>) -> anyhow::Result<()> {
    let target_path = Path::new(target_path);

    for entry in walkdir::WalkDir::new(template_dir) {
        let entry = entry?;
        let path = entry.path();

        // Skip template config files
        if path.file_name().unwrap_or_default() == "template.toml" {
            continue;
        }

        // Calculate relative path from template directory
        let relative_path = path.strip_prefix(template_dir)?;
        let target_file = target_path.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_file)?;
        } else {
            // Read and process template file
            let content = fs::read_to_string(path)?;
            let processed_content = process_template_content(&content, variables)?;

            // Ensure parent directory exists
            if let Some(parent) = target_file.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&target_file, processed_content)?;
        }
    }

    Ok(())
}

fn process_template_content(content: &str, variables: &HashMap<String, String>) -> anyhow::Result<String> {
    let mut result = content.to_string();

    // Simple variable substitution: {{variable_name}}
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    Ok(result)
}

fn copy_project_to_template(project_path: &str, template_dir: &Path) -> anyhow::Result<()> {
    let ignore_patterns = [
        ".git",
        "node_modules",
        "target",
        "__pycache__",
        ".DS_Store",
        "*.log",
        ".env",
    ];

    for entry in walkdir::WalkDir::new(project_path) {
        let entry = entry?;
        let path = entry.path();

        // Skip ignored files/directories
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if ignore_patterns.iter().any(|pattern| {
            if pattern.starts_with("*.") {
                file_name.ends_with(&pattern[1..])
            } else {
                file_name == *pattern
            }
        }) {
            continue;
        }

        // Skip hidden files/directories
        if file_name.starts_with('.') && file_name != ".gitignore" {
            continue;
        }

        // Calculate relative path and target
        let relative_path = path.strip_prefix(project_path)?;
        let target_path = template_dir.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            fs::copy(path, &target_path)?;
        }
    }

    Ok(())
}

fn detect_project_language(project_path: &str) -> anyhow::Result<String> {
    // Simple language detection based on files present
    let project_path = Path::new(project_path);

    if project_path.join("Cargo.toml").exists() {
        Ok("rust".to_string())
    } else if project_path.join("go.mod").exists() {
        Ok("go".to_string())
    } else if project_path.join("package.json").exists() {
        if project_path.join("tsconfig.json").exists() {
            Ok("typescript".to_string())
        } else {
            Ok("javascript".to_string())
        }
    } else if project_path.join("pyproject.toml").exists() ||
              project_path.join("setup.py").exists() ||
              project_path.join("requirements.txt").exists() {
        Ok("python".to_string())
    } else {
        Ok("unknown".to_string())
    }
}
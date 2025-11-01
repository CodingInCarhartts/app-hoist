#[derive(Debug)]
pub struct OptionInfo {
    pub flags: Vec<String>,
    pub description: String,
    pub requires_value: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CliArg {
    pub name: String,
    pub long: Option<String>,
    pub short: Option<char>,
    pub requires_value: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProjectType {
    Uv,
    Venv,
    Generic,
    Go,
    Rust,
    JavaScript,
    TypeScript,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Uv => write!(f, "UV"),
            ProjectType::Venv => write!(f, "venv"),
            ProjectType::Generic => write!(f, "generic Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::JavaScript => write!(f, "JavaScript"),
            ProjectType::TypeScript => write!(f, "TypeScript"),
        }
    }
}

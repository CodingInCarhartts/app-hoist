#[derive(Debug)]
pub struct OptionInfo {
    pub flags: Vec<String>,
    pub description: String,
    pub requires_value: bool,
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Uv,
    Venv,
    Generic,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Uv => write!(f, "UV"),
            ProjectType::Venv => write!(f, "venv"),
            ProjectType::Generic => write!(f, "generic Python"),
        }
    }
}

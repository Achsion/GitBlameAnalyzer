use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project_dir: String,
    pub project_files: ProjectFileConfig,
    pub author_mapping: Vec<AuthorAlias>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorAlias {
    pub author: String,
    pub map_to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFileConfig {
    #[serde(with = "serde_regex")]
    pub blacklist: Vec<Regex>,
}

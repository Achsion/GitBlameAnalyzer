use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project_dir: String,
    pub author_mapping: Vec<AuthorAlias>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorAlias {
    pub author: String,
    pub map_to: String
}

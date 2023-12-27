use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct ProjectCache {
    pub repository_id: String,
    pub variants: Vec<ConfigVariantCache>,
}

pub struct ConfigVariantCache {
    pub config_hash: String,
    pub last_updated: DateTime<Utc>,
    pub project_files: Vec<ProjectFile>,
}

pub struct ProjectFile {
    pub count_map: HashMap<String, u128>,
}

// need to check stuff before this can be fully done
// worst case (which would be the case until I implement this: a full re-analyzation has to be done)
//TODO: last git commit to each division (this only if following can be done:)
//follow up TODO: then, if the config hash matches:
//                - fetch all files that received changes since that commit ======> what if a commit was done BEFORE master merge and it then got merged into master? this may break stuff
//                - load cache config
//                - re-analyze files with changes
//                - override config with those changes ===> what if a commit deleted a file??

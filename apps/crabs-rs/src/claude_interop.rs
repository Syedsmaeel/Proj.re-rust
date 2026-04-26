use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BashInput {
    pub command: String,
    pub timeout: Option<u32>,
    pub description: Option<String>,
    pub run_in_background: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReadInput {
    pub file_path: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEditInput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
}

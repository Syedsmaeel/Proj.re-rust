use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub stages: Vec<String>,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub stage: String,
    pub image: String,
    pub script: Vec<String>,
}

pub fn generate_gitlab_ci(config: &PipelineConfig) -> String {
    let mut yaml = String::from("stages:\n");
    for stage in &config.stages {
        yaml.push_str(&format!("  - {}\n", stage));
    }
    yaml.push('\n');

    for job in &config.jobs {
        yaml.push_str(&format!("{}:\n", job.name));
        yaml.push_str(&format!("  stage: {}\n", job.stage));
        yaml.push_str(&format!("  image: {}\n", job.image));
        yaml.push_str("  script:\n");
        for script in &job.script {
            yaml.push_str(&format!("    - {}\n", script));
        }
    }
    yaml
}

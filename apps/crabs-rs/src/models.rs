use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ServiceConfig {
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<String>>,
}

impl ServiceConfig {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
            ..Default::default()
        }
    }

    pub fn port(mut self, host: u16, container: u16) -> Self {
        let ports = self.ports.get_or_insert(Vec::new());
        ports.push(format!("{}:{}", host, container));
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        let env = self.env.get_or_insert(HashMap::new());
        env.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StackConfig {
    pub name: String,
    pub services: HashMap<String, ServiceConfig>,
}

impl StackConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            services: HashMap::new(),
        }
    }

    pub fn add_service(mut self, name: &str, service: ServiceConfig) -> Self {
        self.services.insert(name.to_string(), service);
        self
    }
}

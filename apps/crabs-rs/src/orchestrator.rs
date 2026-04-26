use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::image::CreateImageOptions;
use futures_util::stream::TryStreamExt;
use crate::models::ServiceConfig;
use std::collections::HashMap;

use bollard::network::CreateNetworkOptions;

pub struct Orchestrator {
    docker: Docker,
}

impl Orchestrator {
    pub fn new() -> Result<Self, bollard::errors::Error> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    pub async fn create_network(&self, name: &str) -> Result<(), bollard::errors::Error> {
        println!("Creating network: {}", name);
        let config = CreateNetworkOptions {
            name: name.to_string(),
            check_duplicate: true,
            ..Default::default()
        };
        self.docker.create_network(config).await?;
        Ok(())
    }

    pub async fn pull_image(&self, image: &str) -> Result<(), bollard::errors::Error> {
        println!("Pulling image: {}", image);
        self.docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: image,
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;
        Ok(())
    }

    pub async fn start_service(&self, name: &str, config: &ServiceConfig, network: Option<&str>) -> Result<String, bollard::errors::Error> {
        self.pull_image(&config.image).await?;

        let mut env_list = Vec::new();
        if let Some(env) = &config.env {
            for (k, v) in env {
                env_list.push(format!("{}={}", k, v));
            }
        }

        let mut host_config = bollard::service::HostConfig::default();
        if let Some(network_name) = network {
            host_config.network_mode = Some(network_name.to_string());
        }

        if let Some(ports) = &config.ports {
            let mut port_bindings = HashMap::new();
            for port_map in ports {
                let parts: Vec<&str> = port_map.split(':').collect();
                if parts.len() == 2 {
                    let host_port = parts[0];
                    let container_port = parts[1];
                    port_bindings.insert(
                        format!("{}/tcp", container_port),
                        Some(vec![bollard::service::PortBinding {
                            host_ip: None,
                            host_port: Some(host_port.to_string()),
                        }]),
                    );
                }
            }
            host_config.port_bindings = Some(port_bindings);
        }

        let container_config = Config {
            image: Some(config.image.clone()),
            cmd: config.command.clone(),
            env: Some(env_list),
            host_config: Some(host_config),
            ..Default::default()
        };

        let container = self.docker
            .create_container(
                Some(CreateContainerOptions {
                    name: name.to_string(),
                    ..Default::default()
                }),
                container_config,
            )
            .await?;

        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        Ok(container.id)
    }

    pub async fn stop_service(&self, name: &str) -> Result<(), bollard::errors::Error> {
        println!("Stopping service: {}", name);
        self.docker.stop_container(name, None).await?;
        self.docker.remove_container(name, None).await?;
        Ok(())
    }

    pub async fn remove_network(&self, name: &str) -> Result<(), bollard::errors::Error> {
        println!("Removing network: {}", name);
        self.docker.remove_network(name).await?;
        Ok(())
    }
}

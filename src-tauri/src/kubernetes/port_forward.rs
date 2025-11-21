// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kubernetes Port Forward Module
// Handles port forwarding functionality

use kube::{Api, Client};
use k8s_openapi::api::core::v1::{Pod, Service};
use anyhow::{Result, anyhow};
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct PortForwardSession {
    pub session_id: String,
    pub resource_type: String, // "pod" or "service"
    pub resource_name: String,
    pub namespace: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub container_name: Option<String>,
}

impl PortForwardSession {
    pub fn new(
        resource_type: String,
        resource_name: String,
        namespace: String,
        local_port: u16,
        remote_port: u16,
        container_name: Option<String>,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            resource_type,
            resource_name,
            namespace,
            local_port,
            remote_port,
            container_name,
        }
    }
    
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.local_port)
    }
}

// Start port forward session
// Returns session ID - actual forwarding will be implemented with WebSocket
pub async fn start_port_forward_session(
    client: &Client,
    resource_type: &str,
    resource_name: &str,
    namespace: &str,
    local_port: u16,
    remote_port: u16,
    container_name: Option<&str>,
) -> Result<PortForwardSession> {
    info!("Starting port forward: {} {}:{}/{} -> localhost:{}", 
          resource_type, namespace, resource_name, remote_port, local_port);
    
    // Verify resource exists
    match resource_type {
        "pod" => {
            let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            pods_api.get(resource_name).await
                .map_err(|e| anyhow!("Pod not found: {}", e))?;
        }
        "service" => {
            let services_api: Api<Service> = Api::namespaced(client.clone(), namespace);
            services_api.get(resource_name).await
                .map_err(|e| anyhow!("Service not found: {}", e))?;
        }
        _ => return Err(anyhow!("Invalid resource type: {}", resource_type)),
    }
    
    // Check if local port is available
    match TcpListener::bind(format!("127.0.0.1:{}", local_port)).await {
        Ok(_) => {
            // Port is available, drop the listener
        }
        Err(e) => {
            return Err(anyhow!("Local port {} is not available: {}", local_port, e));
        }
    }
    
    let session = PortForwardSession::new(
        resource_type.to_string(),
        resource_name.to_string(),
        namespace.to_string(),
        local_port,
        remote_port,
        container_name.map(|s| s.to_string()),
    );
    
    info!("Created port forward session: {}", session.session_id);
    Ok(session)
}


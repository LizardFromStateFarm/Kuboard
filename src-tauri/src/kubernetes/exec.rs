// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kubernetes Exec Module
// Handles pod exec functionality with WebSocket streaming

use kube::{Api, Client};
use k8s_openapi::api::core::v1::Pod;
use anyhow::{Result, anyhow};
use tracing::info;

#[derive(Clone)]
pub struct ExecSession {
    pub session_id: String,
    pub pod_name: String,
    pub namespace: String,
    pub container_name: Option<String>,
}

impl ExecSession {
    pub fn new(pod_name: String, namespace: String, container_name: Option<String>) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            pod_name,
            namespace,
            container_name,
        }
    }
}

// Start exec session - returns session ID
// The actual streaming will be handled via a separate command that uses Tauri events
pub async fn start_exec_session(
    client: &Client,
    pod_name: &str,
    namespace: &str,
    container_name: Option<&str>,
    _command: Option<Vec<String>>,
    _tty: bool,
) -> Result<ExecSession> {
    info!("Starting exec session for pod: {}/{}", namespace, pod_name);
    
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
    
    // Verify pod exists
    pods_api.get(pod_name).await
        .map_err(|e| anyhow!("Pod not found: {}", e))?;
    
    let session = ExecSession::new(
        pod_name.to_string(),
        namespace.to_string(),
        container_name.map(|s| s.to_string()),
    );
    
    info!("Created exec session: {}", session.session_id);
    Ok(session)
}


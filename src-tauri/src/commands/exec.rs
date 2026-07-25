// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Tauri Commands Module
// This module contains all Tauri command functions with kuboard_ prefix

use tauri::State;
use kube::Api;
use kube::api::DeleteParams;
use k8s_openapi::api::{
    apps::v1::{Deployment, ReplicaSet, StatefulSet, DaemonSet},
    batch::v1::{CronJob, Job},
    core::v1::{Node, Namespace, Pod, Service, ConfigMap, Secret, Endpoints},
};
use tracing::{error, info, warn};

use crate::app_state::AppState;
use crate::types::*;
use crate::kubernetes::{
    kuboard_load_kubeconfig,
    kuboard_create_client_from_context,
    kuboard_calculate_cluster_metrics,
};
use crate::metrics::{
    kuboard_fetch_node_metrics_real,
    kuboard_fetch_node_metrics_history,
    kuboard_fetch_pod_metrics_real,
    kuboard_fetch_pod_metrics_history,
    kuboard_check_metrics_server_availability,
};
use crate::kubernetes::{kuboard_fetch_pod_events, kuboard_fetch_pod_logs};
use crate::kubernetes::exec::start_exec_session;
use crate::kubernetes::port_forward::start_port_forward_session;
use serde_json::json;



// Pod Exec Commands
// Note: Full exec implementation requires WebSocket support for bidirectional streaming
// This creates a session that can be used for streaming
#[tauri::command]
pub async fn kuboard_exec_into_pod(
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
    command: Option<Vec<String>>,
    tty: Option<bool>,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("Exec into pod: {}/{} (container: {:?})", namespace, pod_name, container_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Start exec session
    let session = start_exec_session(
        client,
        &pod_name,
        &namespace,
        container_name.as_deref(),
        command.clone(),
        tty.unwrap_or(true),
    ).await.map_err(|e| format!("Failed to start exec session: {}", e))?;

    // Store session in app state
    {
        let mut sessions = state.exec_sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());
    }

    // TODO: Implement WebSocket streaming
    // For now, return session info
    // The frontend will need to connect to a streaming endpoint
    
    Ok(json!({
        "sessionId": session.session_id,
        "podName": session.pod_name,
        "namespace": session.namespace,
        "containerName": session.container_name,
        "status": "connected",
        "message": "Exec session created. Full streaming support coming soon."
    }))
}


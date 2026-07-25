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

    Ok(json!({
        "status": "success",
        "sessionId": session.session_id
    }))
}

#[tauri::command]
pub async fn kuboard_exec_command(
    pod_name: String,
    namespace: String,
    container_name: Option<String>,
    command: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Exec command in pod: {}/{} (container: {:?}): {}", namespace, pod_name, container_name, command);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);

    let mut attach_params = kube::api::AttachParams::default()
        .stdout(true)
        .stderr(true)
        .stdin(false);

    if let Some(ref c) = container_name {
        if !c.trim().is_empty() {
            attach_params = attach_params.container(c);
        }
    }

    let shell_options = vec![
        vec!["sh", "-c", &command],
        vec!["/bin/sh", "-c", &command],
        vec!["/bin/bash", "-c", &command],
        vec!["/bin/ash", "-c", &command],
    ];

    let mut last_err = String::new();
    let mut attached_opt = None;

    for cmd_vec in shell_options {
        match pods_api.exec(&pod_name, cmd_vec, &attach_params).await {
            Ok(attached) => {
                attached_opt = Some(attached);
                break;
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }

    if attached_opt.is_none() {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(attached) = pods_api.exec(&pod_name, parts, &attach_params).await {
                attached_opt = Some(attached);
            }
        }
    }

    let mut attached = match attached_opt {
        Some(a) => a,
        None => return Err(format!("Exec error: Container shell executable not found (sh/bash/ash). {}", last_err)),
    };

    let mut output = String::new();
    if let Some(mut stdout) = attached.stdout() {
        use tokio::io::AsyncReadExt;
        let mut buffer = Vec::new();
        let _ = stdout.read_to_end(&mut buffer).await;
        output = String::from_utf8_lossy(&buffer).to_string();
    }
    if let Some(mut stderr) = attached.stderr() {
        use tokio::io::AsyncReadExt;
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer).await;
        let err_str = String::from_utf8_lossy(&buffer).to_string();
        if !err_str.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&err_str);
        }
    }

    Ok(output)
}

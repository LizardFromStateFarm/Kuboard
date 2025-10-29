// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Tauri Commands Module
// This module contains all Tauri command functions with kuboard_ prefix

use tauri::State;
use kube::Api;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Node, Namespace, Pod, Service, ConfigMap, Secret},
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
use crate::kubernetes::kuboard_fetch_pod_events;

// Context Management Commands
#[tauri::command]
pub async fn kuboard_list_contexts(state: State<'_, AppState>) -> Result<ContextListResponse, String> {
    info!("Listing Kubernetes contexts");
    
    let kubeconfig = match kuboard_load_kubeconfig().await {
        Ok(config) => {
            *state.kubeconfig.write().await = Some(config.clone());
            config
        }
        Err(e) => {
            error!("Failed to load kubeconfig: {}", e);
            return Err(format!("Failed to load kubeconfig: {}", e));
        }
    };

    let current_context = kubeconfig.current_context.clone();
    let mut contexts = Vec::new();

    // Simple iteration over contexts
    for context_entry in &kubeconfig.contexts {
        let context_name = &context_entry.name;
        
        if let Some(context) = &context_entry.context {
            contexts.push(KubeContext {
                name: context_name.clone(),
                cluster: context.cluster.clone(),
                user: context.user.clone().unwrap_or_default(),
                namespace: context.namespace.clone(),
                is_current: current_context.as_ref() == Some(context_name),
            });
        }
    }

    // Automatically set the current context if one exists and no context is currently set
    if let Some(_current_context_name) = &current_context {
        let _current_state = state.current_context.read().await;
        // Don't auto-select context - let user choose
        // if current_state.is_none() {
        //     drop(current_state); // Release the read lock
        //     
        //     // Set the current context automatically
        //     match kuboard_create_client_from_context(&kubeconfig, current_context_name).await {
        //         Ok(client) => {
        //             *state.current_client.write().await = Some(client);
        //             *state.current_context.write().await = Some(current_context_name.clone());
        //             info!("Automatically set current context to: {}", current_context_name);
        //         }
        //         Err(e) => {
        //             warn!("Failed to automatically set current context '{}': {}", current_context_name, e);
        //         }
        //     }
        // }
    }

    Ok(ContextListResponse {
        contexts,
        current_context,
    })
}

#[tauri::command]
pub async fn kuboard_set_context(context_name: String, state: State<'_, AppState>) -> Result<String, String> {
    info!("Setting context to: {}", context_name);
    
    let kubeconfig = state.kubeconfig.read().await;
    let kubeconfig = kubeconfig
        .as_ref()
        .ok_or_else(|| "Kubeconfig not loaded. Call list_contexts first.".to_string())?;

    // Verify the context exists
    let context_exists = kubeconfig.contexts
        .iter()
        .any(|c| c.name == context_name);

    if !context_exists {
        return Err(format!("Context '{}' not found", context_name));
    }

    // Create client for the new context
    match kuboard_create_client_from_context(kubeconfig, &context_name).await {
        Ok(client) => {
            *state.current_client.write().await = Some(client);
            *state.current_context.write().await = Some(context_name.clone());
            Ok(format!("Context switched to: {}", context_name))
        }
        Err(e) => {
            error!("Failed to create client for context '{}': {}", context_name, e);
            Err(format!("Failed to switch context: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_current_context(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let current_context = state.current_context.read().await.clone();
    Ok(current_context)
}

// Cluster Overview Commands
#[tauri::command]
pub async fn kuboard_get_cluster_overview(state: State<'_, AppState>) -> Result<ClusterOverview, String> {
    info!("Getting cluster overview");
    
    // Check if we have a client, if not try to set the current context automatically
    let client_guard = state.current_client.read().await;
    let client = if client_guard.is_some() {
        client_guard.as_ref().unwrap().clone()
    } else {
        drop(client_guard); // Release the read lock
        
        // Try to automatically set the current context from kubeconfig
        let kubeconfig_guard = state.kubeconfig.read().await;
        if let Some(kubeconfig) = kubeconfig_guard.as_ref() {
            if let Some(current_context) = &kubeconfig.current_context {
                match kuboard_create_client_from_context(kubeconfig, current_context).await {
                    Ok(client) => {
                        *state.current_client.write().await = Some(client.clone());
                        *state.current_context.write().await = Some(current_context.clone());
                        info!("Automatically set context to: {}", current_context);
                        client
                    }
                    Err(e) => {
                        return Err(format!("Failed to automatically set context: {}", e));
                    }
                }
            } else {
                return Err("No current context found in kubeconfig. Please set a context first.".to_string());
            }
        } else {
            return Err("No kubeconfig loaded. Please call list_contexts first.".to_string());
        }
    };

    let context_name = state.current_context.read().await
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Get cluster info
    let cluster_info = ClusterInfo {
        name: context_name,
        server: "unknown".to_string(), // Client doesn't expose apiserver_url in this version
        version: None,
    };

    // Count nodes
    let nodes_api: Api<Node> = Api::all(client.clone());
    let node_count = match nodes_api.list(&Default::default()).await {
        Ok(nodes) => nodes.items.len(),
        Err(e) => {
            warn!("Failed to get nodes: {}", e);
            0
        }
    };

    // Count namespaces
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let namespace_count = match namespaces_api.list(&Default::default()).await {
        Ok(namespaces) => namespaces.items.len(),
        Err(e) => {
            warn!("Failed to get namespaces: {}", e);
            0
        }
    };

    // Count pods
    let pods_api: Api<Pod> = Api::all(client.clone());
    let pod_count = match pods_api.list(&Default::default()).await {
        Ok(pods) => pods.items.len(),
        Err(e) => {
            warn!("Failed to get pods: {}", e);
            0
        }
    };

    // Count deployments
    let deployments_api: Api<Deployment> = Api::all(client.clone());
    let deployment_count = match deployments_api.list(&Default::default()).await {
        Ok(deployments) => deployments.items.len(),
        Err(e) => {
            warn!("Failed to get deployments: {}", e);
            0
        }
    };

    // Try to get Kubernetes version
    let kubernetes_version = match client.apiserver_version().await {
        Ok(version) => Some(format!("{}.{}", version.major, version.minor)),
        Err(e) => {
            warn!("Failed to get Kubernetes version: {}", e);
            None
        }
    };

    // Calculate cluster metrics
    let cluster_metrics = match kuboard_calculate_cluster_metrics(&client).await {
        Ok(metrics) => Some(metrics),
        Err(e) => {
            warn!("Failed to calculate cluster metrics: {}", e);
            None
        }
    };

    Ok(ClusterOverview {
        cluster_info,
        node_count,
        namespace_count,
        pod_count,
        deployment_count,
        kubernetes_version,
        cluster_metrics,
    })
}

// Resource Commands
#[tauri::command]
pub async fn kuboard_get_nodes(state: State<'_, AppState>) -> Result<Vec<Node>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let nodes_api: Api<Node> = Api::all(client.clone());
    match nodes_api.list(&Default::default()).await {
        Ok(nodes) => Ok(nodes.items),
        Err(e) => Err(format!("Failed to get nodes: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_namespaces(state: State<'_, AppState>) -> Result<Vec<Namespace>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    match namespaces_api.list(&Default::default()).await {
        Ok(namespaces) => Ok(namespaces.items),
        Err(e) => Err(format!("Failed to get namespaces: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_pods(state: State<'_, AppState>) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::all(client.clone());
    match pods_api.list(&Default::default()).await {
        Ok(pods) => Ok(pods.items),
        Err(e) => Err(format!("Failed to get pods: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_deployments(state: State<'_, AppState>) -> Result<Vec<Deployment>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::all(client.clone());
    match deployments_api.list(&Default::default()).await {
        Ok(deployments) => Ok(deployments.items),
        Err(e) => Err(format!("Failed to get deployments: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_services(state: State<'_, AppState>) -> Result<Vec<Service>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::all(client.clone());
    match services_api.list(&Default::default()).await {
        Ok(services) => Ok(services.items),
        Err(e) => Err(format!("Failed to get services: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_configmaps(state: State<'_, AppState>) -> Result<Vec<ConfigMap>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let configmaps_api: Api<ConfigMap> = Api::all(client.clone());
    match configmaps_api.list(&Default::default()).await {
        Ok(configmaps) => Ok(configmaps.items),
        Err(e) => Err(format!("Failed to get configmaps: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_secrets(state: State<'_, AppState>) -> Result<Vec<Secret>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let secrets_api: Api<Secret> = Api::all(client.clone());
    match secrets_api.list(&Default::default()).await {
        Ok(secrets) => Ok(secrets.items),
        Err(e) => Err(format!("Failed to get secrets: {}", e)),
    }
}

// Metrics Commands - Real Implementation
#[tauri::command]
pub async fn kuboard_get_node_metrics(node_name: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching real-time metrics for node: {}", node_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Check if metrics server is available
    match kuboard_check_metrics_server_availability(client).await {
        Ok(true) => {
            info!("Metrics server is available, fetching real metrics");
        }
        Ok(false) => {
            warn!("Metrics server is not available, using mock data");
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}, using mock data", e);
        }
    }

    // Fetch real metrics (with fallback to mock data)
    match kuboard_fetch_node_metrics_real(client, &node_name).await {
        Ok(metrics) => {
            let response = serde_json::json!({
                "cpu": {
                    "usage": format!("{}m", (metrics.cpu_usage_cores * 1000.0) as i32),
                    "usage_percent": metrics.cpu_usage_percent
                },
                "memory": {
                    "usage": format!("{:.1}Gi", metrics.memory_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
                    "usage_percent": metrics.memory_usage_percent
                },
                "disk": {
                    "usage": format!("{:.1}Gi", metrics.disk_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
                    "usage_percent": metrics.disk_usage_percent
                },
                "timestamp": metrics.timestamp,
                "is_mock_data": metrics.is_mock_data
            });
            
            Ok(response)
        }
        Err(e) => {
            error!("Failed to fetch metrics for node {}: {}", node_name, e);
            Err(format!("Failed to fetch metrics: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_node_metrics_history(
    node_name: String, 
    duration_minutes: u32, 
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    info!("Fetching {} minutes of metrics history for node: {}", duration_minutes, node_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Fetch historical metrics
    match kuboard_fetch_node_metrics_history(client, &node_name, duration_minutes).await {
        Ok(history) => {
            let json_history: Vec<serde_json::Value> = history.into_iter().map(|data_point| {
                serde_json::json!({
                    "timestamp": data_point.timestamp,
                    "cpu": {
                        "usage": format!("{}m", (data_point.cpu_usage_cores * 1000.0) as i32),
                        "usage_percent": data_point.cpu_usage_percent
                    },
                    "memory": {
                        "usage": format!("{:.1}Gi", data_point.memory_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
                        "usage_percent": data_point.memory_usage_percent
                    },
                    "disk": {
                        "usage": format!("{:.1}Gi", data_point.disk_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
                        "usage_percent": data_point.disk_usage_percent
                    },
                    "is_mock_data": data_point.is_mock_data
                })
            }).collect();
            
            Ok(json_history)
        }
        Err(e) => {
            error!("Failed to fetch metrics history for node {}: {}", node_name, e);
            Err(format!("Failed to fetch metrics history: {}", e))
        }
    }
}

// Check metrics server availability
#[tauri::command]
pub async fn kuboard_check_metrics_availability(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    match kuboard_check_metrics_server_availability(client).await {
        Ok(available) => {
            let response = serde_json::json!({
                "available": available,
                "using_mock_data": !available
            });
            Ok(response)
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}", e);
            let response = serde_json::json!({
                "available": false,
                "using_mock_data": true
            });
            Ok(response)
        }
    }
}

// Pod metrics commands
#[tauri::command]
pub async fn kuboard_get_pod_metrics(podName: String, namespace: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching real-time metrics for pod: {}/{}", namespace, podName);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Check if metrics server is available
    match kuboard_check_metrics_server_availability(client).await {
        Ok(true) => {
            info!("Metrics server is available, fetching real metrics");
        }
        Ok(false) => {
            warn!("Metrics server is not available, using mock data");
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}, using mock data", e);
        }
    }

    // Fetch real metrics (with fallback to mock data)
    match kuboard_fetch_pod_metrics_real(client, &podName, &namespace).await {
        Ok(metrics) => {
            info!("✅ Successfully fetched real pod metrics for: {}/{}", namespace, podName);
            Ok(serde_json::to_value(metrics).unwrap())
        }
        Err(e) => {
            warn!("Failed to fetch real pod metrics, using mock data: {}", e);
            // Generate mock data as fallback
            let mock_metrics = crate::metrics::generate_mock_metrics_data_point();
            Ok(serde_json::to_value(mock_metrics).unwrap())
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_pod_metrics_history(
    podName: String,
    namespace: String,
    durationMinutes: u32,
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    info!("Fetching {} minutes of pod metrics history for: {}/{}", durationMinutes, namespace, podName);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Check if metrics server is available
    match kuboard_check_metrics_server_availability(client).await {
        Ok(true) => {
            info!("Metrics server is available, fetching real metrics history");
        }
        Ok(false) => {
            warn!("Metrics server is not available, using mock data");
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}, using mock data", e);
        }
    }

    // Fetch real metrics history (with fallback to mock data)
    match kuboard_fetch_pod_metrics_history(client, &podName, &namespace, durationMinutes).await {
        Ok(history) => {
            info!("✅ Successfully fetched real pod metrics history for: {}/{}", namespace, podName);
            let json_history: Vec<serde_json::Value> = history.into_iter()
                .map(|dp| serde_json::to_value(dp).unwrap())
                .collect();
            Ok(json_history)
        }
        Err(e) => {
            warn!("Failed to fetch real pod metrics history, using mock data: {}", e);
            // Generate mock data as fallback
            let mock_history = crate::metrics::generate_mock_metrics_history(durationMinutes);
            let json_history: Vec<serde_json::Value> = mock_history.into_iter()
                .map(|dp| serde_json::to_value(dp).unwrap())
                .collect();
            Ok(json_history)
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_pod_events(
    podName: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    info!("Fetching events for pod: {}/{}", namespace, podName);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    match kuboard_fetch_pod_events(client, &podName, &namespace).await {
        Ok(events) => {
            info!("✅ Successfully fetched events for pod: {}/{}", namespace, podName);
            let json_events: Vec<serde_json::Value> = events.into_iter()
                .map(|event| serde_json::to_value(event).unwrap())
                .collect();
            Ok(json_events)
        }
        Err(e) => {
            error!("Failed to fetch events for pod: {}/{}: {}", namespace, podName, e);
            Err(e.to_string())
        }
    }
}

// Cluster-wide metrics command
#[tauri::command]
pub async fn kuboard_get_cluster_metrics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching cluster-wide metrics");
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get all nodes
    let nodes_api: Api<Node> = Api::all(client.clone());
    let nodes = match nodes_api.list(&Default::default()).await {
        Ok(nodes) => nodes.items,
        Err(e) => {
            error!("Failed to get nodes for cluster metrics: {}", e);
            return Err(format!("Failed to get nodes: {}", e));
        }
    };

    // Calculate cluster-wide totals
    let mut total_cpu_cores = 0.0;
    let mut total_memory_bytes = 0u64;
    let mut total_disk_bytes = 0u64;
    let mut total_cpu_usage = 0.0;
    let mut total_memory_usage = 0u64;
    let mut total_disk_usage = 0u64;

    // Check if metrics server is available
    let metrics_available = kuboard_check_metrics_server_availability(client).await.unwrap_or(false);

    for node in &nodes {
        // Parse node capacity
        if let Some(capacity) = &node.status.as_ref().and_then(|s| s.capacity.as_ref()) {
            // CPU capacity
            if let Some(cpu_quantity) = capacity.get("cpu") {
                if let Ok(cpu_cores) = parse_cpu_capacity(&cpu_quantity.0) {
                    total_cpu_cores += cpu_cores;
                }
            }
            
            // Memory capacity
            if let Some(memory_quantity) = capacity.get("memory") {
                if let Ok(memory_bytes) = parse_memory_capacity(&memory_quantity.0) {
                    total_memory_bytes += memory_bytes;
                }
            }
            
            // Disk capacity
            if let Some(disk_quantity) = capacity.get("ephemeral-storage") {
                if let Ok(disk_bytes) = parse_memory_capacity(&disk_quantity.0) {
                    total_disk_bytes += disk_bytes;
                }
            }
        }

        // Get usage from metrics server if available
        if metrics_available {
            if let Some(node_name) = node.metadata.name.as_ref() {
                match kuboard_fetch_node_metrics_real(client, node_name).await {
                    Ok(metrics) => {
                        total_cpu_usage += metrics.cpu_usage_cores;
                        total_memory_usage += metrics.memory_usage_bytes;
                        total_disk_usage += metrics.disk_usage_bytes;
                    }
                    Err(e) => {
                        warn!("Failed to get metrics for node {}: {}", node_name, e);
                    }
                }
            }
        }
    }

    // If metrics server not available, calculate from pod requests/limits
    if !metrics_available {
        let pods_api: Api<Pod> = Api::all(client.clone());
        if let Ok(pods) = pods_api.list(&Default::default()).await {
            for pod in &pods.items {
                if let Some(spec) = &pod.spec {
                    for container in &spec.containers {
                        // CPU requests
                        if let Some(requests) = &container.resources.as_ref().and_then(|r| r.requests.as_ref()) {
                            if let Some(cpu_quantity) = requests.get("cpu") {
                                if let Ok(cpu_cores) = parse_cpu_capacity(&cpu_quantity.0) {
                                    total_cpu_usage += cpu_cores;
                                }
                            }
                        }
                        
                        // Memory requests
                        if let Some(requests) = &container.resources.as_ref().and_then(|r| r.requests.as_ref()) {
                            if let Some(memory_quantity) = requests.get("memory") {
                                if let Ok(memory_bytes) = parse_memory_capacity(&memory_quantity.0) {
                                    total_memory_usage += memory_bytes;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Calculate percentages
    let cpu_usage_percent = if total_cpu_cores > 0.0 {
        (total_cpu_usage / total_cpu_cores * 100.0).min(100.0)
    } else {
        0.0
    };

    let memory_usage_percent = if total_memory_bytes > 0 {
        (total_memory_usage as f64 / total_memory_bytes as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let disk_usage_percent = if total_disk_bytes > 0 {
        (total_disk_usage as f64 / total_disk_bytes as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let response = serde_json::json!({
        "cpu": {
            "total_cores": total_cpu_cores,
            "used_cores": total_cpu_usage,
            "usage_percent": cpu_usage_percent
        },
        "memory": {
            "total_bytes": total_memory_bytes,
            "used_bytes": total_memory_usage,
            "usage_percent": memory_usage_percent
        },
        "disk": {
            "total_bytes": total_disk_bytes,
            "used_bytes": total_disk_usage,
            "usage_percent": disk_usage_percent
        },
        "nodes_count": nodes.len(),
        "metrics_available": metrics_available
    });

    Ok(response)
}

// Helper functions for parsing capacity strings
fn parse_cpu_capacity(cpu_str: &str) -> Result<f64, String> {
    let cpu_str = cpu_str.trim();
    
    if cpu_str.ends_with('m') {
        let millicores_str = cpu_str.trim_end_matches('m');
        let millicores = millicores_str.parse::<f64>()
            .map_err(|e| format!("Invalid CPU millicores '{}': {}", cpu_str, e))?;
        Ok(millicores / 1000.0)
    } else {
        cpu_str.parse::<f64>()
            .map_err(|e| format!("Invalid CPU cores '{}': {}", cpu_str, e))
    }
}

fn parse_memory_capacity(memory_str: &str) -> Result<u64, String> {
    let memory_str = memory_str.trim();
    
    if memory_str.ends_with("Ki") {
        let kibibytes_str = memory_str.trim_end_matches("Ki");
        let kibibytes = kibibytes_str.parse::<f64>()
            .map_err(|e| format!("Invalid memory KiB '{}': {}", memory_str, e))?;
        Ok((kibibytes * 1024.0) as u64)
    } else if memory_str.ends_with("Mi") {
        let mebibytes_str = memory_str.trim_end_matches("Mi");
        let mebibytes = mebibytes_str.parse::<f64>()
            .map_err(|e| format!("Invalid memory MiB '{}': {}", memory_str, e))?;
        Ok((mebibytes * 1024.0 * 1024.0) as u64)
    } else if memory_str.ends_with("Gi") {
        let gibibytes_str = memory_str.trim_end_matches("Gi");
        let gibibytes = gibibytes_str.parse::<f64>()
            .map_err(|e| format!("Invalid memory GiB '{}': {}", memory_str, e))?;
        Ok((gibibytes * 1024.0 * 1024.0 * 1024.0) as u64)
    } else {
        memory_str.parse::<u64>()
            .map_err(|e| format!("Invalid memory bytes '{}': {}", memory_str, e))
    }
}
// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Tauri Commands Module
// This module contains all Tauri command functions with kuboard_ prefix

use tauri::State;
use kube::Api;
use kube::api::DeleteParams;
use k8s_openapi::api::{
    apps::v1::{Deployment, ReplicaSet},
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
use serde_json::json;

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
pub async fn kuboard_get_service(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Service, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    match services_api.get(&name).await {
        Ok(service) => Ok(service),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Service {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_service_endpoints(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Endpoints, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let endpoints_api: Api<Endpoints> = Api::namespaced(client.clone(), &namespace);
    match endpoints_api.get(&name).await {
        Ok(endpoints) => Ok(endpoints),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Endpoints {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service endpoints: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_replicasets(state: State<'_, AppState>) -> Result<Vec<ReplicaSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::all(client.clone());
    match replicasets_api.list(&Default::default()).await {
        Ok(replicasets) => Ok(replicasets.items),
        Err(e) => Err(format!("Failed to get replicasets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<ReplicaSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    match replicasets_api.get(&name).await {
        Ok(replicaset) => Ok(replicaset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("ReplicaSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get replicaset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_replicaset(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<ReplicaSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current replicaset
    let mut replicaset = match replicasets_api.get(&name).await {
        Ok(rs) => rs,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("ReplicaSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get replicaset: {}", e)),
    };

    // Update replica count
    if let Some(spec) = replicaset.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("ReplicaSet spec is missing".to_string());
    }

    // Apply the update
    match replicasets_api.replace(&name, &Default::default(), &replicaset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale replicaset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the replicaset to find its selector
    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let replicaset = match replicasets_api.get(&name).await {
        Ok(rs) => rs,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("ReplicaSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get replicaset: {}", e)),
    };

    // Get selector from replicaset
    let selector = match replicaset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("ReplicaSet has no spec".to_string()),
    };

    // List pods with matching labels
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pods = match pods_api.list(&Default::default()).await {
        Ok(pod_list) => pod_list.items,
        Err(e) => return Err(format!("Failed to list pods: {}", e)),
    };

    // Filter pods by selector
    let matching_pods: Vec<Pod> = pods
        .into_iter()
        .filter(|pod| {
            if let Some(pod_labels) = pod.metadata.labels.as_ref() {
                if let Some(match_labels) = selector.match_labels.as_ref() {
                    match_labels.iter().all(|(key, value)| {
                        pod_labels.get(key).map_or(false, |v| v == value)
                    })
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect();

    Ok(matching_pods)
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
            warn!("Metrics server is not available");
            return Err("Metrics server is not available".to_string());
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}", e);
            return Err(format!("Error checking metrics server availability: {}", e));
        }
    }

    // Fetch real metrics
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
                "available": available
            });
            Ok(response)
        }
        Err(e) => {
            error!("Error checking metrics server availability: {}", e);
            Err(format!("Error checking metrics server availability: {}", e))
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

    // Fetch real metrics
    match kuboard_fetch_pod_metrics_real(client, &podName, &namespace).await {
        Ok(metrics) => {
            info!("✅ Successfully fetched real pod metrics for: {}/{}", namespace, podName);
            Ok(serde_json::to_value(metrics).unwrap())
        }
        Err(e) => {
            error!("Failed to fetch real pod metrics for {}/{}: {}", namespace, podName, e);
            Err(format!("Failed to fetch pod metrics: {}", e))
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
            warn!("Metrics server is not available");
            return Err("Metrics server is not available".to_string());
        }
        Err(e) => {
            warn!("Error checking metrics server availability: {}", e);
            return Err(format!("Error checking metrics server availability: {}", e));
        }
    }

    // Fetch real metrics history
    match kuboard_fetch_pod_metrics_history(client, &podName, &namespace, durationMinutes).await {
        Ok(history) => {
            info!("✅ Successfully fetched real pod metrics history for: {}/{}", namespace, podName);
            let json_history: Vec<serde_json::Value> = history.into_iter()
                .map(|dp| serde_json::to_value(dp).unwrap())
                .collect();
            Ok(json_history)
        }
        Err(e) => {
            error!("Failed to fetch real pod metrics history for {}/{}: {}", namespace, podName, e);
            Err(format!("Failed to fetch pod metrics history: {}", e))
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

#[tauri::command]
pub async fn kuboard_get_pod_logs(
    podName: String,
    namespace: String,
    containerName: Option<String>,
    tailLines: Option<u32>,
    follow: Option<bool>,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Fetching logs for pod: {}/{}", namespace, podName);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    match kuboard_fetch_pod_logs(client, &podName, &namespace, containerName.as_deref(), tailLines, follow.unwrap_or(false)).await {
        Ok(logs) => {
            info!("✅ Successfully fetched logs for pod: {}/{}", namespace, podName);
            Ok(logs)
        }
        Err(e) => {
            error!("Failed to fetch logs for pod: {}/{}: {}", namespace, podName, e);
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

// Pod Actions Commands
#[tauri::command]
pub async fn kuboard_delete_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.delete(&pod_name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} deleted successfully", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Treat 404 as successful deletion (already gone)
            warn!("Pod {}/{} not found during delete - treating as already deleted", namespace, pod_name);
            Ok(format!("Pod {}/{} not found (already deleted)", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to delete pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to delete pod: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_restart_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Restarting pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    // Delete directly to trigger recreation by controller. If already gone, treat as success.
    match pods_api.delete(&pod_name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully restarted (deleted for recreation) pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} restarted (deleted for recreation)", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Pod {}/{} not found during restart - treating as already restarted", namespace, pod_name);
            Ok(format!("Pod {}/{} not found (already restarted)", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to restart pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to restart pod: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_pod_yaml(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Getting YAML for pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.get(&pod_name).await {
        Ok(pod) => {
            // Convert to JSON first, then format as YAML-like structure
            // Note: We'll use JSON for now, YAML can be added later if needed
            match serde_json::to_string_pretty(&pod) {
                Ok(json) => {
                    info!("✅ Successfully retrieved pod data: {}/{}", namespace, pod_name);
                    Ok(json)
                }
                Err(e) => {
                    error!("Failed to serialize pod to JSON: {}", e);
                    Err(format!("Failed to serialize pod: {}", e))
                }
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to get pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to get pod: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_update_pod_from_yaml(
    pod_name: String,
    namespace: String,
    yaml_content: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Updating pod from YAML: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    // Parse JSON/YAML content
    let mut updated_pod: Pod = match serde_json::from_str(&yaml_content) {
        Ok(pod) => pod,
        Err(e) => {
            error!("Failed to parse YAML/JSON: {}", e);
            return Err(format!("Invalid YAML/JSON format: {}", e));
        }
    };
    
    // Verify the pod name matches (metadata.name is Option<String>)
    match &updated_pod.metadata.name {
        Some(name) if name != &pod_name => {
            return Err(format!("Pod name mismatch: expected {}, got {}", 
                pod_name, name));
        }
        None => {
            // If name is None, set it to the expected name
            updated_pod.metadata.name = Some(pod_name.clone());
        }
        _ => {} // Name matches or will be set
    }
    
    // Replace the pod
    match pods_api.replace(&pod_name, &Default::default(), &updated_pod).await {
        Ok(_) => {
            info!("✅ Successfully updated pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} updated successfully", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to update pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to update pod: {}", e))
        }
    }
}

// Pod Watch Commands
#[tauri::command]
pub async fn kuboard_start_pod_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting pod watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.pod_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Pod watch started successfully");
            Ok("Pod watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start pod watch: {}", e);
            Err(format!("Failed to start pod watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_pod_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping pod watch");

    let mut watcher_guard = state.pod_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Pod watch stopped");
    Ok("Pod watch stopped".to_string())
}

// Resource Describe Commands
#[tauri::command]
pub async fn kuboard_describe_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("Describing pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.get(&pod_name).await {
        Ok(pod) => {
            // Get pod events
            let events = kuboard_fetch_pod_events(client, &pod_name, &namespace).await.unwrap_or_default();
            
            // Build describe output structure
            let describe = json!({
                "name": pod.metadata.name.as_ref().unwrap_or(&"Unknown".to_string()),
                "namespace": pod.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                "labels": pod.metadata.labels.as_ref().unwrap_or(&std::collections::BTreeMap::new()),
                "annotations": pod.metadata.annotations.as_ref().unwrap_or(&std::collections::BTreeMap::new()),
                "status": {
                    "phase": pod.status.as_ref().and_then(|s| s.phase.as_ref()).unwrap_or(&"Unknown".to_string()),
                    "podIP": pod.status.as_ref().and_then(|s| s.pod_ip.as_ref()).unwrap_or(&"None".to_string()),
                    "hostIP": pod.status.as_ref().and_then(|s| s.host_ip.as_ref()).unwrap_or(&"None".to_string()),
                    "nodeName": pod.spec.as_ref().and_then(|s| s.node_name.as_ref()).unwrap_or(&"None".to_string()),
                    "qosClass": pod.status.as_ref().and_then(|s| s.qos_class.as_ref()).unwrap_or(&"Unknown".to_string()),
                    "startTime": pod.status.as_ref().and_then(|s| s.start_time.as_ref()).map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                },
                "conditions": pod.status.as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .map(|conditions| conditions.iter().map(|c| json!({
                        "type": c.type_,
                        "status": c.status,
                        "reason": c.reason.as_ref().unwrap_or(&"None".to_string()),
                        "message": c.message.as_ref().unwrap_or(&"None".to_string()),
                        "lastTransitionTime": c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "containers": pod.spec.as_ref()
                    .map(|s| s.containers.iter().map(|c| {
                        let status = pod.status.as_ref()
                            .and_then(|s| s.container_statuses.as_ref())
                            .and_then(|statuses| statuses.iter().find(|cs| cs.name == c.name));
                        json!({
                            "name": c.name,
                            "image": c.image,
                            "imagePullPolicy": c.image_pull_policy.as_ref().unwrap_or(&"IfNotPresent".to_string()),
                            "resources": c.resources.as_ref().map(|r| json!({
                                "requests": r.requests.as_ref().map(|reqs| reqs.iter().map(|(k, v)| (k, v.0.clone())).collect::<std::collections::BTreeMap<_, _>>()).unwrap_or_default(),
                                "limits": r.limits.as_ref().map(|lims| lims.iter().map(|(k, v)| (k, v.0.clone())).collect::<std::collections::BTreeMap<_, _>>()).unwrap_or_default(),
                            })),
                            "ports": c.ports.as_ref().map(|ports| ports.iter().map(|p| json!({
                                "name": p.name.as_ref().unwrap_or(&"None".to_string()),
                                "containerPort": p.container_port,
                                "protocol": p.protocol.as_ref().unwrap_or(&"TCP".to_string()),
                            })).collect::<Vec<_>>()).unwrap_or_default(),
                            "env": c.env.as_ref().map(|envs| envs.iter().map(|e| {
                                let mut env_json = serde_json::Map::new();
                                env_json.insert("name".to_string(), json!(e.name));
                                env_json.insert("value".to_string(), json!(e.value.as_ref().unwrap_or(&"None".to_string())));
                                if let Some(vf) = e.value_from.as_ref() {
                                    let mut value_from_json = serde_json::Map::new();
                                    if let Some(fr) = vf.field_ref.as_ref() {
                                        let mut field_ref_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                                        field_ref_json.insert("fieldPath".to_string(), serde_json::Value::String(fr.field_path.clone()));
                                        value_from_json.insert("fieldRef".to_string(), serde_json::Value::Object(field_ref_json));
                                    }
                                    env_json.insert("valueFrom".to_string(), json!(value_from_json));
                                }
                                json!(env_json)
                            }).collect::<Vec<_>>()).unwrap_or_default(),
                            "status": status.map(|s| json!({
                                "ready": s.ready,
                                "restartCount": s.restart_count,
                                "state": {
                                    "running": s.state.as_ref().and_then(|st| st.running.as_ref()).map(|_| json!({"startedAt": "Running"})),
                                    "waiting": s.state.as_ref().and_then(|st| st.waiting.as_ref()).map(|w| json!({
                                        "reason": w.reason.as_ref().unwrap_or(&"None".to_string()),
                                        "message": w.message.as_ref().unwrap_or(&"None".to_string()),
                                    })),
                                    "terminated": s.state.as_ref().and_then(|st| st.terminated.as_ref()).map(|t| json!({
                                        "reason": t.reason.as_ref().unwrap_or(&"None".to_string()),
                                        "exitCode": t.exit_code,
                                        "startedAt": t.started_at.as_ref().map(|dt| dt.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                                        "finishedAt": t.finished_at.as_ref().map(|dt| dt.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                                    })),
                                },
                            })).unwrap_or(json!({})),
                        })
                    }).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "volumes": pod.spec.as_ref()
                    .and_then(|s| s.volumes.as_ref())
                    .map(|volumes| volumes.iter().map(|v| json!({
                        "name": v.name,
                        "type": if v.config_map.is_some() { "ConfigMap" } 
                               else if v.secret.is_some() { "Secret" }
                               else if v.persistent_volume_claim.is_some() { "PVC" }
                               else if v.empty_dir.is_some() { "EmptyDir" }
                               else { "Other" },
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "tolerations": pod.spec.as_ref()
                    .and_then(|s| s.tolerations.as_ref())
                    .map(|tolerations| tolerations.iter().map(|t| json!({
                        "key": t.key.as_ref().unwrap_or(&"".to_string()),
                        "operator": t.operator.as_ref().unwrap_or(&"Equal".to_string()),
                        "value": t.value.as_ref().unwrap_or(&"None".to_string()),
                        "effect": t.effect.as_ref().unwrap_or(&"None".to_string()),
                        "tolerationSeconds": t.toleration_seconds,
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "events": events.iter().map(|e| json!({
                    "type": e.type_,
                    "reason": e.reason,
                    "message": e.message,
                    "count": e.count,
                    "firstTimestamp": e.first_timestamp.as_deref().unwrap_or("None"),
                    "lastTimestamp": e.last_timestamp.as_deref().unwrap_or("None"),
                })).collect::<Vec<_>>(),
                "metadata": {
                    "uid": pod.metadata.uid.as_ref().unwrap_or(&"None".to_string()),
                    "resourceVersion": pod.metadata.resource_version.as_ref().unwrap_or(&"None".to_string()),
                    "creationTimestamp": pod.metadata.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                    "generation": pod.metadata.generation.unwrap_or(0),
                },
            });
            
            info!("✅ Successfully described pod: {}/{}", namespace, pod_name);
            Ok(describe)
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to describe pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to describe pod: {}", e))
        }
    }
}
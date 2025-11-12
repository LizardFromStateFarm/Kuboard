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

// Deployment Commands
#[tauri::command]
pub async fn kuboard_get_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    match deployments_api.get(&name).await {
        Ok(deployment) => Ok(deployment),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Deployment {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_deployment(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment
    let mut deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Update replica count
    if let Some(spec) = deployment.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("Deployment spec is missing".to_string());
    }

    // Apply the update
    match deployments_api.replace(&name, &Default::default(), &deployment).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_rollback_deployment(
    name: String,
    namespace: String,
    _revision: Option<i64>,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment (verify it exists)
    let _deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // For rollback, we need to use the rollout subresource
    // This is a simplified version - in production, you'd use kubectl rollout undo
    // For now, we'll return an error indicating this needs kubectl
    Err("Rollback requires kubectl rollout undo command. This feature will be enhanced in Phase 2.".to_string())
}

#[tauri::command]
pub async fn kuboard_restart_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment
    let mut deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = deployment.spec.as_mut().ok_or_else(|| "Deployment spec is missing".to_string())?;
    let metadata = spec.template.metadata.get_or_insert_with(Default::default);
    let annotations = metadata.annotations.get_or_insert_with(Default::default);
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    annotations.insert(
        "kubectl.kubernetes.io/restartedAt".to_string(),
        timestamp.to_string(),
    );

    // Apply the update
    match deployments_api.replace(&name, &Default::default(), &deployment).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_deployment_replicasets(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<ReplicaSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the deployment to find its selector
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Get selector from deployment (currently unused, filtering by owner reference instead)
    let _selector = match deployment.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("Deployment has no spec".to_string()),
    };

    // List all replicasets in namespace
    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let replicasets = match replicasets_api.list(&Default::default()).await {
        Ok(rs_list) => rs_list.items,
        Err(e) => return Err(format!("Failed to list replicasets: {}", e)),
    };

    // Filter replicasets by owner reference (owned by this deployment)
    let matching_replicasets: Vec<ReplicaSet> = replicasets
        .into_iter()
        .filter(|rs| {
            if let Some(owner_refs) = rs.metadata.owner_references.as_ref() {
                owner_refs.iter().any(|owner| {
                    owner.kind == "Deployment" && owner.name == name
                })
            } else {
                false
            }
        })
        .collect();

    Ok(matching_replicasets)
}

#[tauri::command]
pub async fn kuboard_get_deployment_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the deployment to find its selector
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Get selector from deployment
    let selector = match deployment.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("Deployment has no spec".to_string()),
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

// StatefulSet Commands
#[tauri::command]
pub async fn kuboard_get_statefulsets(state: State<'_, AppState>) -> Result<Vec<StatefulSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::all(client.clone());
    match statefulsets_api.list(&Default::default()).await {
        Ok(statefulsets) => Ok(statefulsets.items),
        Err(e) => Err(format!("Failed to get statefulsets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    match statefulsets_api.get(&name).await {
        Ok(statefulset) => Ok(statefulset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("StatefulSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_statefulset(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current statefulset
    let mut statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Update replica count
    if let Some(spec) = statefulset.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("StatefulSet spec is missing".to_string());
    }

    // Apply the update
    match statefulsets_api.replace(&name, &Default::default(), &statefulset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_restart_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current statefulset
    let mut statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = statefulset.spec.as_mut().ok_or_else(|| "StatefulSet spec is missing".to_string())?;
    let metadata = spec.template.metadata.get_or_insert_with(Default::default);
    let annotations = metadata.annotations.get_or_insert_with(Default::default);
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    annotations.insert(
        "kubectl.kubernetes.io/restartedAt".to_string(),
        timestamp.to_string(),
    );

    // Apply the update
    match statefulsets_api.replace(&name, &Default::default(), &statefulset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the statefulset to find its selector
    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    let statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Get selector from statefulset
    let selector = match statefulset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("StatefulSet has no spec".to_string()),
    };

    // List pods with matching labels
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pods = match pods_api.list(&Default::default()).await {
        Ok(pod_list) => pod_list.items,
        Err(e) => return Err(format!("Failed to list pods: {}", e)),
    };

    // Filter pods by selector and sort by ordinal (StatefulSet pods are named with ordinal suffix)
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

    // Sort by pod name (which contains ordinal) for StatefulSet ordering
    let mut sorted_pods = matching_pods;
    sorted_pods.sort_by(|a, b| {
        let name_a = a.metadata.name.as_deref().unwrap_or("");
        let name_b = b.metadata.name.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    Ok(sorted_pods)
}

// DaemonSet Commands
#[tauri::command]
pub async fn kuboard_get_daemonsets(state: State<'_, AppState>) -> Result<Vec<DaemonSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::all(client.clone());
    match daemonsets_api.list(&Default::default()).await {
        Ok(daemonsets) => Ok(daemonsets.items),
        Err(e) => Err(format!("Failed to get daemonsets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<DaemonSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    match daemonsets_api.get(&name).await {
        Ok(daemonset) => Ok(daemonset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("DaemonSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get daemonset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_restart_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<DaemonSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current daemonset
    let mut daemonset = match daemonsets_api.get(&name).await {
        Ok(ds) => ds,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("DaemonSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get daemonset: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = daemonset.spec.as_mut().ok_or_else(|| "DaemonSet spec is missing".to_string())?;
    let metadata = spec.template.metadata.get_or_insert_with(Default::default);
    let annotations = metadata.annotations.get_or_insert_with(Default::default);
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    annotations.insert(
        "kubectl.kubernetes.io/restartedAt".to_string(),
        timestamp.to_string(),
    );

    // Apply the update
    match daemonsets_api.replace(&name, &Default::default(), &daemonset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart daemonset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the daemonset to find its selector
    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    let daemonset = match daemonsets_api.get(&name).await {
        Ok(ds) => ds,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("DaemonSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get daemonset: {}", e)),
    };

    // Get selector from daemonset
    let selector = match daemonset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("DaemonSet has no spec".to_string()),
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

    // Sort by node name, then by pod name for consistent ordering
    let mut sorted_pods = matching_pods;
    sorted_pods.sort_by(|a, b| {
        let node_a = a.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or("");
        let node_b = b.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or("");
        match node_a.cmp(node_b) {
            std::cmp::Ordering::Equal => {
                let name_a = a.metadata.name.as_deref().unwrap_or("");
                let name_b = b.metadata.name.as_deref().unwrap_or("");
                name_a.cmp(name_b)
            }
            other => other,
        }
    });

    Ok(sorted_pods)
}

// CronJob Commands
#[tauri::command]
pub async fn kuboard_get_cronjobs(state: State<'_, AppState>) -> Result<Vec<CronJob>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::all(client.clone());
    match cronjobs_api.list(&Default::default()).await {
        Ok(cronjobs) => Ok(cronjobs.items),
        Err(e) => Err(format!("Failed to get cronjobs: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    match cronjobs_api.get(&name).await {
        Ok(cronjob) => Ok(cronjob),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("CronJob {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_trigger_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Job, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get the cronjob to extract its job template
    let cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Extract job template from cronjob spec
    let job_template = match cronjob.spec.as_ref() {
        Some(spec) => &spec.job_template,
        None => return Err("CronJob has no spec".to_string()),
    };

    // Create a new Job from the template
    let mut job_metadata = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
        name: Some(format!("{}-manual-{}", name, chrono::Utc::now().timestamp())),
        namespace: Some(namespace.clone()),
        ..Default::default()
    };

    // Copy labels from job template metadata if present
    if let Some(template_metadata) = job_template.metadata.as_ref() {
        if let Some(labels) = template_metadata.labels.as_ref() {
            job_metadata.labels = Some(labels.clone());
        }
    }

    let job = Job {
        metadata: job_metadata,
        spec: job_template.spec.clone(),
        ..Default::default()
    };

    // Create the job
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    match jobs_api.create(&Default::default(), &job).await {
        Ok(created_job) => Ok(created_job),
        Err(e) => Err(format!("Failed to trigger cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_suspend_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get current cronjob
    let mut cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Set suspend to true
    if let Some(spec) = cronjob.spec.as_mut() {
        spec.suspend = Some(true);
    } else {
        return Err("CronJob has no spec".to_string());
    }

    // Apply the update
    match cronjobs_api.replace(&name, &Default::default(), &cronjob).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to suspend cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_resume_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get current cronjob
    let mut cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Set suspend to false
    if let Some(spec) = cronjob.spec.as_mut() {
        spec.suspend = Some(false);
    } else {
        return Err("CronJob has no spec".to_string());
    }

    // Apply the update
    match cronjobs_api.replace(&name, &Default::default(), &cronjob).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to resume cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob_jobs(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Job>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Verify the cronjob exists
    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    match cronjobs_api.get(&name).await {
        Ok(_) => {},
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    }

    // List all jobs in the namespace
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let jobs = match jobs_api.list(&Default::default()).await {
        Ok(job_list) => job_list.items,
        Err(e) => return Err(format!("Failed to list jobs: {}", e)),
    };

    // Filter jobs by owner reference (jobs created by this cronjob)
    let matching_jobs: Vec<Job> = jobs
        .into_iter()
        .filter(|job| {
            if let Some(owner_refs) = job.metadata.owner_references.as_ref() {
                owner_refs.iter().any(|owner| {
                    owner.kind == "CronJob" && 
                    owner.name == name &&
                    owner.controller == Some(true)
                })
            } else {
                false
            }
        })
        .collect();

    // Sort by creation timestamp (newest first)
    let mut sorted_jobs = matching_jobs;
    sorted_jobs.sort_by(|a, b| {
        let time_a = a.metadata.creation_timestamp.as_ref()
            .map(|ts| ts.0.timestamp())
            .unwrap_or(0);
        let time_b = b.metadata.creation_timestamp.as_ref()
            .map(|ts| ts.0.timestamp())
            .unwrap_or(0);
        time_b.cmp(&time_a) // Reverse order (newest first)
    });

    Ok(sorted_jobs)
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

// Delete Commands for All Resource Types
#[tauri::command]
pub async fn kuboard_delete_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting deployment: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    match deployments_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted deployment: {}/{}", namespace, name);
            Ok(format!("Deployment {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Deployment {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("Deployment {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete deployment {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete deployment: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting statefulset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    match statefulsets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted statefulset: {}/{}", namespace, name);
            Ok(format!("StatefulSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("StatefulSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("StatefulSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete statefulset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete statefulset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting daemonset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    match daemonsets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted daemonset: {}/{}", namespace, name);
            Ok(format!("DaemonSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("DaemonSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("DaemonSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete daemonset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete daemonset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_replicaset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting replicaset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    match replicasets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted replicaset: {}/{}", namespace, name);
            Ok(format!("ReplicaSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("ReplicaSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("ReplicaSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete replicaset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete replicaset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_service(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting service: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    
    match services_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted service: {}/{}", namespace, name);
            Ok(format!("Service {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Service {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("Service {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete service {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete service: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting cronjob: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    match cronjobs_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted cronjob: {}/{}", namespace, name);
            Ok(format!("CronJob {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("CronJob {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("CronJob {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete cronjob {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete cronjob: {}", e))
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

// YAML Get Commands for All Resource Types
#[tauri::command]
pub async fn kuboard_get_deployment_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    match deployments_api.get(&name).await {
        Ok(deployment) => {
            match serde_json::to_string_pretty(&deployment) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize deployment: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Deployment {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get deployment: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    match statefulsets_api.get(&name).await {
        Ok(statefulset) => {
            match serde_json::to_string_pretty(&statefulset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize statefulset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("StatefulSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get statefulset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    match daemonsets_api.get(&name).await {
        Ok(daemonset) => {
            match serde_json::to_string_pretty(&daemonset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize daemonset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("DaemonSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get daemonset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    match replicasets_api.get(&name).await {
        Ok(replicaset) => {
            match serde_json::to_string_pretty(&replicaset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize replicaset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("ReplicaSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get replicaset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_service_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    
    match services_api.get(&name).await {
        Ok(service) => {
            match serde_json::to_string_pretty(&service) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize service: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Service {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    match cronjobs_api.get(&name).await {
        Ok(cronjob) => {
            match serde_json::to_string_pretty(&cronjob) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize cronjob: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("CronJob {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get cronjob: {}", e))
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

// Deployment Watch Commands
#[tauri::command]
pub async fn kuboard_start_deployment_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting deployment watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.deployment_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Deployment watch started successfully");
            Ok("Deployment watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start deployment watch: {}", e);
            Err(format!("Failed to start deployment watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_deployment_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping deployment watch");

    let mut watcher_guard = state.deployment_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Deployment watch stopped");
    Ok("Deployment watch stopped".to_string())
}

// StatefulSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_statefulset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting statefulset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.statefulset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ StatefulSet watch started successfully");
            Ok("StatefulSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start statefulset watch: {}", e);
            Err(format!("Failed to start statefulset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_statefulset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping statefulset watch");

    let mut watcher_guard = state.statefulset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ StatefulSet watch stopped");
    Ok("StatefulSet watch stopped".to_string())
}

// DaemonSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_daemonset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting daemonset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.daemonset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ DaemonSet watch started successfully");
            Ok("DaemonSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start daemonset watch: {}", e);
            Err(format!("Failed to start daemonset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_daemonset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping daemonset watch");

    let mut watcher_guard = state.daemonset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ DaemonSet watch stopped");
    Ok("DaemonSet watch stopped".to_string())
}

// ReplicaSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_replicaset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting replicaset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.replicaset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ ReplicaSet watch started successfully");
            Ok("ReplicaSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start replicaset watch: {}", e);
            Err(format!("Failed to start replicaset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_replicaset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping replicaset watch");

    let mut watcher_guard = state.replicaset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ ReplicaSet watch stopped");
    Ok("ReplicaSet watch stopped".to_string())
}

// Service Watch Commands
#[tauri::command]
pub async fn kuboard_start_service_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting service watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.service_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Service watch started successfully");
            Ok("Service watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start service watch: {}", e);
            Err(format!("Failed to start service watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_service_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping service watch");

    let mut watcher_guard = state.service_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Service watch stopped");
    Ok("Service watch stopped".to_string())
}

// CronJob Watch Commands
#[tauri::command]
pub async fn kuboard_start_cronjob_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting cronjob watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.cronjob_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ CronJob watch started successfully");
            Ok("CronJob watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start cronjob watch: {}", e);
            Err(format!("Failed to start cronjob watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_cronjob_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping cronjob watch");

    let mut watcher_guard = state.cronjob_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ CronJob watch stopped");
    Ok("CronJob watch stopped".to_string())
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
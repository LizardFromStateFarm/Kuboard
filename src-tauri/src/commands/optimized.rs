// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Optimized Commands for Faster Context Switching and Loading
// This module contains performance-optimized versions of key functions

use crate::app_state::AppState;
use crate::types::*;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::{Node, Namespace, Pod};
use k8s_openapi::api::apps::v1::Deployment;
use std::sync::Arc;
use tauri::State;
use tracing::{info, warn, error};

// Cache for frequently accessed data
#[derive(Debug, Clone)]
pub struct ClusterCache {
    pub overview: Option<ClusterOverview>,
    pub nodes: Option<Vec<Node>>,
    pub namespaces: Option<Vec<Namespace>>,
    pub pods: Option<Vec<Pod>>,
    pub deployments: Option<Vec<Deployment>>,
    pub last_updated: std::time::SystemTime,
    pub context_name: String,
}

impl ClusterCache {
    pub fn new() -> Self {
        Self {
            overview: None,
            nodes: None,
            namespaces: None,
            pods: None,
            deployments: None,
            last_updated: std::time::SystemTime::now(),
            context_name: String::new(),
        }
    }

    pub fn is_valid(&self, context_name: &str) -> bool {
        self.context_name == context_name && 
        self.last_updated.elapsed().unwrap_or_default().as_secs() < 30 // 30 second cache
    }

    pub fn invalidate(&mut self) {
        self.overview = None;
        self.nodes = None;
        self.namespaces = None;
        self.pods = None;
        self.deployments = None;
        self.last_updated = std::time::SystemTime::now();
    }
}

// Optimized context switching with caching
#[tauri::command]
pub async fn kuboard_set_context_optimized(
    context_name: String, 
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Setting context to: {} (optimized)", context_name);
    
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
            
            // Invalidate cache for new context
            if let Some(cache) = state.cluster_cache.write().await.as_mut() {
                cache.invalidate();
                cache.context_name = context_name.clone();
            }
            
            Ok(format!("Context switched to: {}", context_name))
        }
        Err(e) => {
            error!("Failed to create client for context '{}': {}", context_name, e);
            Err(format!("Failed to switch context: {}", e))
        }
    }
}

// Optimized cluster overview with parallel API calls
#[tauri::command]
pub async fn kuboard_get_cluster_overview_optimized(
    state: State<'_, AppState>
) -> Result<ClusterOverview, String> {
    info!("Getting cluster overview (optimized)");
    
    let context_name = state.current_context.read().await
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Check cache first
    if let Some(cache) = state.cluster_cache.read().await.as_ref() {
        if cache.is_valid(&context_name) {
            if let Some(overview) = &cache.overview {
                info!("Returning cached cluster overview");
                return Ok(overview.clone());
            }
        }
    }

    // Get client
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Create API clients
    let nodes_api: Api<Node> = Api::all(client.clone());
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());
    let deployments_api: Api<Deployment> = Api::all(client.clone());

    // Parallel API calls for better performance
    let (nodes_result, namespaces_result, pods_result, deployments_result, version_result) = tokio::join!(
        nodes_api.list(&Default::default()),
        namespaces_api.list(&Default::default()),
        pods_api.list(&Default::default()),
        deployments_api.list(&Default::default()),
        client.apiserver_version()
    );

    // Process results
    let node_count = nodes_result.map(|nodes| nodes.items.len()).unwrap_or(0);
    let namespace_count = namespaces_result.map(|namespaces| namespaces.items.len()).unwrap_or(0);
    let pod_count = pods_result.map(|pods| pods.items.len()).unwrap_or(0);
    let deployment_count = deployments_result.map(|deployments| deployments.items.len()).unwrap_or(0);
    let kubernetes_version = version_result.ok().map(|v| format!("{}.{}", v.major, v.minor));

    // Get cluster info
    let cluster_info = ClusterInfo {
        name: context_name.clone(),
        server: "unknown".to_string(),
        version: kubernetes_version.clone(),
    };

    // Try to get cluster metrics (non-blocking)
    let cluster_metrics = match kuboard_calculate_cluster_metrics(client).await {
        Ok(metrics) => Some(metrics),
        Err(e) => {
            warn!("Failed to get cluster metrics: {}", e);
            None
        }
    };

    let overview = ClusterOverview {
        cluster_info,
        node_count,
        namespace_count,
        pod_count,
        deployment_count,
        kubernetes_version,
        cluster_metrics,
    };

    // Cache the result
    if let Some(cache) = state.cluster_cache.write().await.as_mut() {
        cache.overview = Some(overview.clone());
        cache.context_name = context_name;
        cache.last_updated = std::time::SystemTime::now();
    }

    Ok(overview)
}

// Optimized resource loading with caching
#[tauri::command]
pub async fn kuboard_get_nodes_optimized(
    state: State<'_, AppState>
) -> Result<Vec<Node>, String> {
    let context_name = state.current_context.read().await
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Check cache first
    if let Some(cache) = state.cluster_cache.read().await.as_ref() {
        if cache.is_valid(&context_name) {
            if let Some(nodes) = &cache.nodes {
                info!("Returning cached nodes");
                return Ok(nodes.clone());
            }
        }
    }

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let nodes_api: Api<Node> = Api::all(client.clone());
    match nodes_api.list(&Default::default()).await {
        Ok(nodes) => {
            let nodes = nodes.items;
            
            // Cache the result
            if let Some(cache) = state.cluster_cache.write().await.as_mut() {
                cache.nodes = Some(nodes.clone());
                cache.context_name = context_name;
                cache.last_updated = std::time::SystemTime::now();
            }
            
            Ok(nodes)
        }
        Err(e) => Err(format!("Failed to get nodes: {}", e)),
    }
}

// Batch resource loading for better performance
#[tauri::command]
pub async fn kuboard_get_all_resources_optimized(
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    let context_name = state.current_context.read().await
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Create API clients
    let nodes_api: Api<Node> = Api::all(client.clone());
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());
    let deployments_api: Api<Deployment> = Api::all(client.clone());

    // Parallel API calls
    let (nodes_result, namespaces_result, pods_result, deployments_result) = tokio::join!(
        nodes_api.list(&Default::default()),
        namespaces_api.list(&Default::default()),
        pods_api.list(&Default::default()),
        deployments_api.list(&Default::default())
    );

    // Process results
    let nodes = nodes_result.map(|nodes| nodes.items).unwrap_or_default();
    let namespaces = namespaces_result.map(|namespaces| namespaces.items).unwrap_or_default();
    let pods = pods_result.map(|pods| pods.items).unwrap_or_default();
    let deployments = deployments_result.map(|deployments| deployments.items).unwrap_or_default();

    // Cache all results
    if let Some(cache) = state.cluster_cache.write().await.as_mut() {
        cache.nodes = Some(nodes.clone());
        cache.namespaces = Some(namespaces.clone());
        cache.pods = Some(pods.clone());
        cache.deployments = Some(deployments.clone());
        cache.context_name = context_name;
        cache.last_updated = std::time::SystemTime::now();
    }

    Ok(serde_json::json!({
        "nodes": nodes,
        "namespaces": namespaces,
        "pods": pods,
        "deployments": deployments
    }))
}

// Helper function to create client from context (reused from existing code)
async fn kuboard_create_client_from_context(
    kubeconfig: &kube::Config,
    context_name: &str,
) -> Result<Client, String> {
    // Implementation would be the same as in the existing code
    // This is a placeholder for the actual implementation
    Err("Not implemented".to_string())
}

// Helper function to calculate cluster metrics (reused from existing code)
async fn kuboard_calculate_cluster_metrics(
    _client: &Client,
) -> Result<ClusterMetrics, String> {
    // Implementation would be the same as in the existing code
    // This is a placeholder for the actual implementation
    Err("Not implemented".to_string())
}

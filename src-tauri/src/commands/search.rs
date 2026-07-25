// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::Api;
use kube::api::ListParams;
use k8s_openapi::api::{
    core::v1::{Pod, Service, Node, Namespace},
    apps::v1::{Deployment, StatefulSet, DaemonSet},
};
use serde::{Serialize, Deserialize};
use tracing::error;

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub name: String,
    pub namespace: Option<String>,
    pub kind: String,
}

#[tauri::command]
pub async fn kuboard_search_resources(
    state: State<'_, AppState>
) -> Result<Vec<SearchResult>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let mut results = Vec::new();

    // Fetch common resources in parallel (simplified for now, but could be more robust)
    // In a real large cluster, this might be slow, so we could optimize or cache
    
    // Namespaces
    let ns_api: Api<Namespace> = Api::all(client.clone());
    if let Ok(list) = ns_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: None,
                kind: "Namespace".to_string(),
            });
        }
    }

    // Nodes
    let node_api: Api<Node> = Api::all(client.clone());
    if let Ok(list) = node_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: None,
                kind: "Node".to_string(),
            });
        }
    }

    // Pods
    let pod_api: Api<Pod> = Api::all(client.clone());
    if let Ok(list) = pod_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: item.metadata.namespace,
                kind: "Pod".to_string(),
            });
        }
    }

    // Deployments
    let deploy_api: Api<Deployment> = Api::all(client.clone());
    if let Ok(list) = deploy_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: item.metadata.namespace,
                kind: "Deployment".to_string(),
            });
        }
    }

    // Services
    let svc_api: Api<Service> = Api::all(client.clone());
    if let Ok(list) = svc_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: item.metadata.namespace,
                kind: "Service".to_string(),
            });
        }
    }

    // StatefulSets
    let sts_api: Api<StatefulSet> = Api::all(client.clone());
    if let Ok(list) = sts_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: item.metadata.namespace,
                kind: "StatefulSet".to_string(),
            });
        }
    }

    // DaemonSets
    let ds_api: Api<DaemonSet> = Api::all(client.clone());
    if let Ok(list) = ds_api.list(&ListParams::default()).await {
        for item in list.items {
            results.push(SearchResult {
                name: item.metadata.name.unwrap_or_default(),
                namespace: item.metadata.namespace,
                kind: "DaemonSet".to_string(),
            });
        }
    }

    Ok(results)
}

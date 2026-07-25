// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::{Api, ResourceExt, api::ListParams};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relationship: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[tauri::command]
pub async fn kuboard_get_resource_graph(
    kind: String,
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<ResourceGraph, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let mut nodes: Vec<GraphNode> = Vec::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    // Start with the root resource
    // For simplicity in this version, we'll focus on the Workload -> Pod -> Service chain
    
    // 1. If it's a Deployment/StatefulSet/DaemonSet, find its Pods
    // 2. If it's a Pod, find its Service
    // 3. Find Ingresses pointing to Services
    
    // Let's implement a more general approach:
    // Fetch all common resources in the namespace and build the map locally
    
    let pods_api: Api<kube::core::DynamicObject> = Api::namespaced_with(client.clone(), &namespace, &kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk("", "v1", "Pod")));
    let svcs_api: Api<kube::core::DynamicObject> = Api::namespaced_with(client.clone(), &namespace, &kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk("", "v1", "Service")));
    let deploys_api: Api<kube::core::DynamicObject> = Api::namespaced_with(client.clone(), &namespace, &kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk("apps", "v1", "Deployment")));
    let rs_api: Api<kube::core::DynamicObject> = Api::namespaced_with(client.clone(), &namespace, &kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk("apps", "v1", "ReplicaSet")));

    // Fetch lists
    let pods = pods_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let svcs = svcs_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let deploys = deploys_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let rss = rs_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    // Add all these to nodes
    for p in &pods {
        nodes.push(GraphNode {
            id: format!("Pod/{}", p.name_any()),
            name: p.name_any(),
            kind: "Pod".to_string(),
            namespace: Some(namespace.clone()),
            status: "Running".to_string(), // Simplified
        });
    }

    for d in &deploys {
        nodes.push(GraphNode {
            id: format!("Deployment/{}", d.name_any()),
            name: d.name_any(),
            kind: "Deployment".to_string(),
            namespace: Some(namespace.clone()),
            status: "Ready".to_string(),
        });
    }

    for r in &rss {
        nodes.push(GraphNode {
            id: format!("ReplicaSet/{}", r.name_any()),
            name: r.name_any(),
            kind: "ReplicaSet".to_string(),
            namespace: Some(namespace.clone()),
            status: "Ready".to_string(),
        });
    }

    for s in &svcs {
        nodes.push(GraphNode {
            id: format!("Service/{}", s.name_any()),
            name: s.name_any(),
            kind: "Service".to_string(),
            namespace: Some(namespace.clone()),
            status: "Active".to_string(),
        });
    }

    // Build edges based on OwnerReferences
    for p in &pods {
        if let Some(owners) = &p.metadata.owner_references {
            for owner in owners {
                edges.push(GraphEdge {
                    from: format!("{}/{}", owner.kind, owner.name),
                    to: format!("Pod/{}", p.name_any()),
                    relationship: "Owns".to_string(),
                });
            }
        }
    }

    for r in &rss {
        if let Some(owners) = &r.metadata.owner_references {
            for owner in owners {
                edges.push(GraphEdge {
                    from: format!("{}/{}", owner.kind, owner.name),
                    to: format!("ReplicaSet/{}", r.name_any()),
                    relationship: "Owns".to_string(),
                });
            }
        }
    }

    // Build edges based on Service Selectors
    for s in &svcs {
        if let Some(spec) = s.data.get("spec") {
            if let Some(selector) = spec.get("selector").and_then(|v| v.as_object()) {
                // Find pods that match this selector
                for p in &pods {
                    if let Some(labels) = &p.metadata.labels {
                        let mut matches = true;
                        for (k, v) in selector {
                            if labels.get(k).map(|s| s.as_str()) != v.as_str() {
                                matches = false;
                                break;
                            }
                        }
                        if matches && !selector.is_empty() {
                            edges.push(GraphEdge {
                                from: format!("Service/{}", s.name_any()),
                                to: format!("Pod/{}", p.name_any()),
                                relationship: "Selects".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Filter the graph to only include nodes reachable from the target resource
    // (Or just return the whole namespace graph for now)
    
    Ok(ResourceGraph { nodes, edges })
}

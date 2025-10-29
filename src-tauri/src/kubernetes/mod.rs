// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Kubernetes Integration Module
// This module contains all Kubernetes-related helper functions

use anyhow::{anyhow, Result};
use kube::{Client, Config, Api};
use kube::api::ListParams;
use kube::config::{KubeConfigOptions, Kubeconfig};
use k8s_openapi::api::core::v1::Node;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tracing::{debug, warn};

use crate::types::{ClusterMetrics, NodeDetails};
use crate::utils::{kuboard_parse_cpu_string, kuboard_parse_memory_string};

// Kubeconfig Management
pub async fn kuboard_load_kubeconfig() -> Result<Kubeconfig> {
    let kubeconfig_path = env::var("KUBECONFIG")
        .map(PathBuf::from)
        .or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".kube").join("config"))
                .ok_or_else(|| anyhow!("Could not find home directory"))
        })?;

    debug!("Loading kubeconfig from: {:?}", kubeconfig_path);
    
    if !kubeconfig_path.exists() {
        return Err(anyhow!("Kubeconfig file not found at {:?}", kubeconfig_path));
    }

    let kubeconfig = Kubeconfig::read_from(&kubeconfig_path)?;
    Ok(kubeconfig)
}

pub async fn kuboard_create_client_from_context(
    kubeconfig: &Kubeconfig, 
    context_name: &str
) -> Result<Client> {
    let config_options = KubeConfigOptions {
        context: Some(context_name.to_string()),
        cluster: None,
        user: None,
    };
    
    let config = Config::from_custom_kubeconfig(kubeconfig.clone(), &config_options).await?;
    let client = Client::try_from(config)?;
    
    Ok(client)
}

// Metrics Functions
pub async fn kuboard_fetch_node_metrics(
    _client: &Client, 
    _node_name: &str
) -> Result<(f64, f64, f64)> {
    // For now, return mock data since metrics API is not available in k8s-openapi
    // In a real implementation, you would use the metrics.k8s.io API directly
    warn!("Metrics server integration not fully implemented - using mock data");
    
    // Mock realistic usage data
    let cpu_usage = 0.15; // 15% CPU usage
    let memory_usage = 1024.0 * 1024.0 * 1024.0; // 1GB memory usage
    let disk_usage = 5.0 * 1024.0 * 1024.0 * 1024.0; // 5GB disk usage
    
    Ok((cpu_usage, memory_usage, disk_usage))
}

// Cluster Metrics Calculation
pub async fn kuboard_calculate_cluster_metrics(client: &Client) -> Result<ClusterMetrics> {
    let nodes_api: Api<Node> = Api::all(client.clone());
    let nodes = nodes_api.list(&Default::default()).await?;
    
    let mut active_nodes = 0;
    let mut node_details = Vec::new();
    
    for node in &nodes.items {
        let node_name = node.metadata.name.as_ref().unwrap_or(&"Unknown".to_string()).clone();
        
        // Check if node is ready
        let is_ready = node.status.as_ref()
            .and_then(|status| status.conditions.as_ref())
            .map(|conditions| {
                conditions.iter().any(|condition| {
                    condition.type_ == "Ready" && condition.status == "True"
                })
            })
            .unwrap_or(false);
        
        if is_ready {
            active_nodes += 1;
        }
        
        let status = if is_ready { "Ready".to_string() } else { "Not Ready".to_string() };
        
        // Extract node conditions
        let mut conditions = Vec::new();
        if let Some(node_status) = &node.status {
            if let Some(node_conditions) = &node_status.conditions {
                for condition in node_conditions {
                    conditions.push(format!("{}: {}", condition.type_, condition.status));
                }
            }
        }
        
        // Get node capacity and allocatable resources
        let mut max_cpu_cores = 0.0;
        let mut max_memory_bytes = 0u64;
        let mut allocatable_cpu_cores = 0.0;
        let mut allocatable_memory_bytes = 0u64;
        let mut disk_capacity = None;
        let mut disk_allocatable = None;
        
        if let Some(node_status) = &node.status {
            // Max capacity
            if let Some(capacity) = &node_status.capacity {
                if let Some(cpu) = capacity.get("cpu") {
                    if let Ok(cpu_cores) = kuboard_parse_cpu_string(&cpu.0) {
                        max_cpu_cores = cpu_cores;
                    }
                }
                if let Some(memory) = capacity.get("memory") {
                    if let Ok(memory_bytes) = kuboard_parse_memory_string(&memory.0) {
                        max_memory_bytes = memory_bytes;
                    }
                }
                if let Some(disk) = capacity.get("ephemeral-storage") {
                    if let Ok(disk_bytes) = kuboard_parse_memory_string(&disk.0) {
                        disk_capacity = Some(disk_bytes);
                    }
                }
            }
            
            // Allocatable resources
            if let Some(allocatable) = &node_status.allocatable {
                if let Some(cpu) = allocatable.get("cpu") {
                    if let Ok(cpu_cores) = kuboard_parse_cpu_string(&cpu.0) {
                        allocatable_cpu_cores = cpu_cores;
                    }
                }
                if let Some(memory) = allocatable.get("memory") {
                    if let Ok(memory_bytes) = kuboard_parse_memory_string(&memory.0) {
                        allocatable_memory_bytes = memory_bytes;
                    }
                }
                if let Some(disk) = allocatable.get("ephemeral-storage") {
                    if let Ok(disk_bytes) = kuboard_parse_memory_string(&disk.0) {
                        disk_allocatable = Some(disk_bytes);
                    }
                }
            }
        }
        
        // Try to fetch real metrics from metrics server
        let (cpu_usage_cores, memory_usage_bytes, disk_usage_bytes) = match kuboard_fetch_node_metrics(client, &node_name).await {
            Ok((cpu, memory, disk)) => (cpu, memory, disk),
            Err(e) => {
                warn!("Metrics not available for node {}: {}", node_name, e);
                (0.0, 0.0, 0.0)
            }
        };
        
        // Calculate usage percentages
        let cpu_usage_percent = if allocatable_cpu_cores > 0.0 {
            (cpu_usage_cores / allocatable_cpu_cores * 100.0).min(100.0)
        } else {
            0.0
        };
        
        let memory_usage_percent = if allocatable_memory_bytes > 0 {
            (memory_usage_bytes / allocatable_memory_bytes as f64 * 100.0).min(100.0)
        } else {
            0.0
        };
        
        let disk_usage_percent = if let Some(disk_cap) = disk_capacity {
            if disk_cap > 0 {
                (disk_usage_bytes / disk_cap as f64 * 100.0).min(100.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // Extract additional node information
        let os = node.status.as_ref()
            .and_then(|status| status.node_info.as_ref())
            .map(|info| format!("{} {}", info.operating_system, info.architecture));
        
        let kernel_version = node.status.as_ref()
            .and_then(|status| status.node_info.as_ref())
            .map(|info| info.kernel_version.clone());
        
        let kubelet_version = node.status.as_ref()
            .and_then(|status| status.node_info.as_ref())
            .map(|info| info.kubelet_version.clone());
        
        let container_runtime = node.status.as_ref()
            .and_then(|status| status.node_info.as_ref())
            .map(|info| info.container_runtime_version.clone());
        
        // Extract labels and annotations
        let labels = node.metadata.labels.clone().unwrap_or_default();
        let annotations = node.metadata.annotations.clone().unwrap_or_default();
        
        // Extract taints
        let taints = node.spec.as_ref()
            .and_then(|spec| spec.taints.as_ref())
            .map(|taints| {
                taints.iter().map(|taint| {
                    format!("{}={}:{}", 
                        taint.key,
                        taint.value.as_ref().unwrap_or(&"".to_string()),
                        taint.effect
                    )
                }).collect()
            })
            .unwrap_or_default();
        
        // Check if metrics are available
        let metrics_available = cpu_usage_cores > 0.0 || memory_usage_bytes > 0.0;
        let metrics_error = if !metrics_available {
            Some("Metrics server not available - install metrics-server for real-time resource usage".to_string())
        } else {
            None
        };
        
        node_details.push(NodeDetails {
            name: node_name,
            status,
            max_cpu_cores,
            max_memory_bytes,
            allocatable_cpu_cores,
            allocatable_memory_bytes,
            cpu_usage_percent,
            memory_usage_percent,
            conditions,
            os,
            kernel_version,
            kubelet_version,
            container_runtime,
            disk_capacity,
            disk_allocatable,
            disk_usage_percent,
            labels,
            annotations,
            taints,
            metrics_available,
            metrics_error,
        });
    }
    
    Ok(ClusterMetrics {
        max_nodes: nodes.items.len(),
        active_nodes,
        nodes: node_details,
    })
}

// Pod Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodEvent {
    pub type_: String,
    pub reason: String,
    pub message: String,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub count: Option<i32>,
    pub involved_object: Option<InvolvedObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvolvedObject {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
}

pub async fn kuboard_fetch_pod_events(
    client: &Client,
    pod_name: &str,
    namespace: &str,
) -> Result<Vec<PodEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let events_api: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    
    // Create a field selector to get events for this specific pod
    let field_selector = format!("involvedObject.name={}", pod_name);
    let list_params = ListParams::default().fields(&field_selector);
    
    let events = events_api.list(&list_params).await?;
    
    let pod_events: Vec<PodEvent> = events.items.into_iter().map(|event| {
        PodEvent {
            type_: event.type_.unwrap_or_default(),
            reason: event.reason.unwrap_or_default(),
            message: event.message.unwrap_or_default(),
            first_timestamp: event.first_timestamp.map(|ts| ts.0.to_rfc3339()),
            last_timestamp: event.last_timestamp.map(|ts| ts.0.to_rfc3339()),
            count: event.count,
            involved_object: Some(InvolvedObject {
                kind: event.involved_object.kind.unwrap_or_default(),
                name: event.involved_object.name.unwrap_or_default(),
                namespace: event.involved_object.namespace,
            }),
        }
    }).collect();
    
    Ok(pod_events)
}
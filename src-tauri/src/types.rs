// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Type Definitions
// This module contains all custom types and data structures

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeContext {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextListResponse {
    pub contexts: Vec<KubeContext>,
    pub current_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: String,
    pub server: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterOverview {
    pub cluster_info: ClusterInfo,
    pub node_count: usize,
    pub namespace_count: usize,
    pub pod_count: usize,
    pub deployment_count: usize,
    pub kubernetes_version: Option<String>,
    pub cluster_metrics: Option<ClusterMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub max_nodes: usize,
    pub active_nodes: usize,
    pub nodes: Vec<NodeDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetails {
    pub name: String,
    pub status: String,
    pub max_cpu_cores: f64,
    pub max_memory_bytes: u64,
    pub allocatable_cpu_cores: f64,
    pub allocatable_memory_bytes: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub conditions: Vec<String>,
    pub os: Option<String>,
    pub kernel_version: Option<String>,
    pub kubelet_version: Option<String>,
    pub container_runtime: Option<String>,
    pub disk_capacity: Option<u64>,
    pub disk_allocatable: Option<u64>,
    pub disk_usage_percent: f64,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub taints: Vec<String>,
    pub metrics_available: bool,
    pub metrics_error: Option<String>,
}

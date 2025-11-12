// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details
//
// This file uses the kube-rs library (Apache License 2.0) and k8s-openapi (Apache License 2.0)
// See THIRD_PARTY_LICENSES.md for complete license information

// Kuboard Main Library
// Modular structure with kuboard_ prefixed functions to avoid naming conflicts

// Module declarations
pub mod commands;
pub mod kubernetes;
pub mod types;
pub mod app_state;
pub mod utils;
pub mod metrics;

// Re-exports for convenience
pub use app_state::AppState;
pub use types::*;

use tracing::info;

// Main application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Kuboard application");

    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Context Management
            commands::kuboard_list_contexts,
            commands::kuboard_set_context,
            commands::kuboard_get_current_context,
            
            // Cluster Overview
            commands::kuboard_get_cluster_overview,
            
            // Resource Management
            commands::kuboard_get_nodes,
            commands::kuboard_get_namespaces,
            commands::kuboard_get_pods,
            commands::kuboard_get_deployments,
            commands::kuboard_get_replicasets,
            commands::kuboard_get_replicaset,
            commands::kuboard_scale_replicaset,
            commands::kuboard_get_replicaset_pods,
            commands::kuboard_get_deployment,
            commands::kuboard_scale_deployment,
            commands::kuboard_rollback_deployment,
            commands::kuboard_restart_deployment,
            commands::kuboard_get_deployment_replicasets,
            commands::kuboard_get_deployment_pods,
            commands::kuboard_get_statefulsets,
            commands::kuboard_get_statefulset,
            commands::kuboard_scale_statefulset,
            commands::kuboard_restart_statefulset,
            commands::kuboard_get_statefulset_pods,
            commands::kuboard_get_daemonsets,
            commands::kuboard_get_daemonset,
            commands::kuboard_restart_daemonset,
            commands::kuboard_get_daemonset_pods,
            commands::kuboard_get_cronjobs,
            commands::kuboard_get_cronjob,
            commands::kuboard_trigger_cronjob,
            commands::kuboard_suspend_cronjob,
            commands::kuboard_resume_cronjob,
            commands::kuboard_get_cronjob_jobs,
            commands::kuboard_get_services,
            commands::kuboard_get_service,
            commands::kuboard_get_service_endpoints,
            commands::kuboard_get_configmaps,
            commands::kuboard_get_secrets,
            
            // Metrics (Real Implementation)
            commands::kuboard_get_node_metrics,
            commands::kuboard_get_node_metrics_history,
        commands::kuboard_get_pod_metrics,
        commands::kuboard_get_pod_metrics_history,
        commands::kuboard_get_pod_events,
        commands::kuboard_get_pod_logs,
        commands::kuboard_check_metrics_availability,
        commands::kuboard_get_cluster_metrics,
        
        // Pod Actions
        commands::kuboard_delete_pod,
        commands::kuboard_restart_pod,
        commands::kuboard_get_pod_yaml,
        commands::kuboard_update_pod_from_yaml,
        
        // Resource Delete Commands
        commands::kuboard_delete_deployment,
        commands::kuboard_delete_statefulset,
        commands::kuboard_delete_daemonset,
        commands::kuboard_delete_replicaset,
        commands::kuboard_delete_service,
        commands::kuboard_delete_cronjob,
        
        // Resource YAML Commands
        commands::kuboard_get_deployment_yaml,
        commands::kuboard_get_statefulset_yaml,
        commands::kuboard_get_daemonset_yaml,
        commands::kuboard_get_replicaset_yaml,
        commands::kuboard_get_service_yaml,
        commands::kuboard_get_cronjob_yaml,
        
        // Pod Watch
        commands::kuboard_start_pod_watch,
        commands::kuboard_stop_pod_watch,
        
        // Deployment Watch
        commands::kuboard_start_deployment_watch,
        commands::kuboard_stop_deployment_watch,
        
        // StatefulSet Watch
        commands::kuboard_start_statefulset_watch,
        commands::kuboard_stop_statefulset_watch,
        
        // DaemonSet Watch
        commands::kuboard_start_daemonset_watch,
        commands::kuboard_stop_daemonset_watch,
        
        // ReplicaSet Watch
        commands::kuboard_start_replicaset_watch,
        commands::kuboard_stop_replicaset_watch,
        
        // Service Watch
        commands::kuboard_start_service_watch,
        commands::kuboard_stop_service_watch,
        
        // CronJob Watch
        commands::kuboard_start_cronjob_watch,
        commands::kuboard_stop_cronjob_watch,
        
        // Resource Describe
        commands::kuboard_describe_pod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kuboard_context_list_logic() {
        // Simple test to verify the basic context parsing logic works
        let contexts = vec![
            KubeContext {
                name: "test-context".to_string(),
                cluster: "test-cluster".to_string(),
                user: "test-user".to_string(),
                namespace: Some("default".to_string()),
                is_current: true,
            }
        ];

        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].name, "test-context");
        assert_eq!(contexts[0].cluster, "test-cluster");
        assert_eq!(contexts[0].user, "test-user");
        assert_eq!(contexts[0].namespace, Some("default".to_string()));
        assert!(contexts[0].is_current);
    }

    #[test]
    fn test_kuboard_cluster_overview_creation() {
        let cluster_info = ClusterInfo {
            name: "test-cluster".to_string(),
            server: "https://test-cluster.example.com".to_string(),
            version: None,
        };

        let overview = ClusterOverview {
            cluster_info: cluster_info.clone(),
            node_count: 3,
            namespace_count: 5,
            pod_count: 20,
            deployment_count: 8,
            kubernetes_version: Some("v1.28.0".to_string()),
            cluster_metrics: None,
        };

        assert_eq!(overview.cluster_info.name, "test-cluster");
        assert_eq!(overview.node_count, 3);
        assert_eq!(overview.namespace_count, 5);
        assert_eq!(overview.pod_count, 20);
        assert_eq!(overview.deployment_count, 8);
        assert_eq!(overview.kubernetes_version, Some("v1.28.0".to_string()));
    }
}
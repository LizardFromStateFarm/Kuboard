// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Application State Management
// This module contains the application state and related functionality

use kube::{Client, config::Kubeconfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::kubernetes::watch::{
    PodWatcher, DeploymentWatcher, StatefulSetWatcher, DaemonSetWatcher,
    ReplicaSetWatcher, ServiceWatcher, CronJobWatcher
};
// use crate::commands::optimized::ClusterCache;

#[derive(Clone)]
pub struct AppState {
    pub current_client: Arc<RwLock<Option<Client>>>,
    pub current_context: Arc<RwLock<Option<String>>>,
    pub kubeconfig: Arc<RwLock<Option<Kubeconfig>>>,
    pub pod_watcher: Arc<RwLock<PodWatcher>>,
    pub deployment_watcher: Arc<RwLock<DeploymentWatcher>>,
    pub statefulset_watcher: Arc<RwLock<StatefulSetWatcher>>,
    pub daemonset_watcher: Arc<RwLock<DaemonSetWatcher>>,
    pub replicaset_watcher: Arc<RwLock<ReplicaSetWatcher>>,
    pub service_watcher: Arc<RwLock<ServiceWatcher>>,
    pub cronjob_watcher: Arc<RwLock<CronJobWatcher>>,
    // pub cluster_cache: Arc<RwLock<Option<ClusterCache>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_client: Arc::new(RwLock::new(None)),
            current_context: Arc::new(RwLock::new(None)),
            kubeconfig: Arc::new(RwLock::new(None)),
            pod_watcher: Arc::new(RwLock::new(PodWatcher::new())),
            deployment_watcher: Arc::new(RwLock::new(DeploymentWatcher::new())),
            statefulset_watcher: Arc::new(RwLock::new(StatefulSetWatcher::new())),
            daemonset_watcher: Arc::new(RwLock::new(DaemonSetWatcher::new())),
            replicaset_watcher: Arc::new(RwLock::new(ReplicaSetWatcher::new())),
            service_watcher: Arc::new(RwLock::new(ServiceWatcher::new())),
            cronjob_watcher: Arc::new(RwLock::new(CronJobWatcher::new())),
            // cluster_cache: Arc::new(RwLock::new(Some(ClusterCache::new()))),
        }
    }
}

// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Application State Management
// This module contains the application state and related functionality

use kube::{Client, config::Kubeconfig};
use std::sync::Arc;
use tokio::sync::RwLock;
// use crate::commands::optimized::ClusterCache;

#[derive(Clone)]
pub struct AppState {
    pub current_client: Arc<RwLock<Option<Client>>>,
    pub current_context: Arc<RwLock<Option<String>>>,
    pub kubeconfig: Arc<RwLock<Option<Kubeconfig>>>,
    // pub cluster_cache: Arc<RwLock<Option<ClusterCache>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_client: Arc::new(RwLock::new(None)),
            current_context: Arc::new(RwLock::new(None)),
            kubeconfig: Arc::new(RwLock::new(None)),
            // cluster_cache: Arc::new(RwLock::new(Some(ClusterCache::new()))),
        }
    }
}

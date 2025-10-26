// Kuboard UI - TypeScript Type Definitions

export interface KubeContext {
  name: string;
  cluster: string;
  user: string;
  namespace?: string;
  is_current?: boolean;
}

export interface ClusterInfo {
  name: string;
  server: string;
  version?: string;
}

export interface NodeDetails {
  name: string;
  status: string;
  max_cpu_cores: number;
  max_memory_bytes: number;
  allocatable_cpu_cores: number;
  allocatable_memory_bytes: number;
  cpu_usage_percent: number;
  memory_usage_percent: number;
  conditions: string[];
  // Additional node information
  os?: string;
  kernel_version?: string;
  kubelet_version?: string;
  container_runtime?: string;
  disk_capacity?: number;
  disk_allocatable?: number;
  disk_usage_percent: number;
  labels: Record<string, string>;
  annotations: Record<string, string>;
  taints: string[];
  metrics_available: boolean;
  metrics_error?: string;
}

export interface ClusterMetrics {
  max_nodes: number;
  active_nodes: number;
  nodes: NodeDetails[];
}

export interface ClusterOverview {
  cluster_info: ClusterInfo;
  node_count: number;
  namespace_count: number;
  pod_count: number;
  deployment_count: number;
  kubernetes_version?: string;
  cluster_metrics?: ClusterMetrics;
}

export interface ContextListResponse {
  contexts: KubeContext[];
  current_context?: string;
}

export interface MetricsDataPoint {
  timestamp: number;
  cpu_usage_cores: number;
  memory_usage_bytes: number;
  disk_usage_bytes: number;
  cpu_usage_percent: number;
  memory_usage_percent: number;
  disk_usage_percent: number;
  is_mock_data: boolean;
}

export type ResourceTab = 'cpu' | 'memory' | 'disk';
export type ExpandedPanel = 'nodes' | 'namespaces' | 'pods' | 'deployments' | null;

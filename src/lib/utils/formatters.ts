// Kuboard UI - Data Formatting Utilities

/**
 * Format memory bytes into human-readable string
 * @param bytes - Number of bytes
 * @returns Formatted memory string (e.g., "1.5 GB")
 */
export function formatMemory(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/**
 * Format CPU cores into human-readable string
 * @param cores - Number of CPU cores
 * @returns Formatted CPU string (e.g., "2.5 cores")
 */
export function formatCPU(cores: number): string {
  return cores.toFixed(1) + ' cores';
}

/**
 * Format percentage with specified decimal places
 * @param value - Percentage value (0-100)
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted percentage string
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return value.toFixed(decimals) + '%';
}

/**
 * Format date string into localized date
 * @param dateString - ISO date string
 * @returns Formatted date string
 */
export function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString();
}

/**
 * Format timestamp into localized time
 * @param timestamp - Unix timestamp in seconds
 * @returns Formatted time string
 */
export function formatTime(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleTimeString();
}

/**
 * Format duration in minutes to human-readable string
 * @param minutes - Duration in minutes
 * @returns Formatted duration string
 */
export function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} min`;
  } else if (minutes < 1440) { // Less than 24 hours
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
  } else {
    const days = Math.floor(minutes / 1440);
    const remainingHours = Math.floor((minutes % 1440) / 60);
    return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
  }
}

/**
 * Format resource quantity string (e.g., "150m", "1Gi")
 * @param quantity - Kubernetes quantity string
 * @returns Formatted resource string
 */
export function formatResourceQuantity(quantity: string): string {
  // Handle millicores (m)
  if (quantity.endsWith('m')) {
    const value = parseFloat(quantity.slice(0, -1));
    return value >= 1000 ? `${(value / 1000).toFixed(1)} cores` : `${value}m`;
  }
  
  // Handle memory units
  if (quantity.endsWith('Ki')) {
    const value = parseFloat(quantity.slice(0, -2));
    return `${value} KiB`;
  }
  if (quantity.endsWith('Mi')) {
    const value = parseFloat(quantity.slice(0, -2));
    return `${value} MiB`;
  }
  if (quantity.endsWith('Gi')) {
    const value = parseFloat(quantity.slice(0, -2));
    return `${value} GiB`;
  }
  if (quantity.endsWith('Ti')) {
    const value = parseFloat(quantity.slice(0, -2));
    return `${value} TiB`;
  }
  
  // Handle decimal cores
  if (!isNaN(parseFloat(quantity))) {
    const value = parseFloat(quantity);
    return value >= 1 ? `${value} cores` : `${(value * 1000).toFixed(0)}m`;
  }
  
  return quantity;
}

/**
 * Truncate text to specified length with ellipsis
 * @param text - Text to truncate
 * @param maxLength - Maximum length
 * @returns Truncated text
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 3) + '...';
}

/**
 * Format status for display
 * @param status - Status string
 * @returns Formatted status string
 */
export function formatStatus(status: string): string {
  return status.toLowerCase().replace(/\s+/g, '-');
}

/**
 * Get status color class
 * @param status - Status string
 * @returns CSS class name for status
 */
export function getStatusClass(status: string): string {
  const normalized = status.toLowerCase();
  
  if (normalized.includes('ready') || normalized.includes('running') || normalized.includes('available')) {
    return 'status-ready';
  }
  if (normalized.includes('pending') || normalized.includes('waiting')) {
    return 'status-pending';
  }
  if (normalized.includes('failed') || normalized.includes('error') || normalized.includes('unhealthy')) {
    return 'status-error';
  }
  
  return 'status-unknown';
}

/**
 * Format node conditions for display
 * @param conditions - Array of condition strings
 * @returns Formatted conditions string
 */
export function formatNodeConditions(conditions: string[]): string {
  return conditions.join(', ');
}

/**
 * Format labels for display
 * @param labels - Object of label key-value pairs
 * @returns Formatted labels string
 */
export function formatLabels(labels: Record<string, string>): string {
  return Object.entries(labels)
    .map(([key, value]) => `${key}: ${value}`)
    .join(', ');
}

/**
 * Format annotations for display
 * @param annotations - Object of annotation key-value pairs
 * @returns Formatted annotations string
 */
export function formatAnnotations(annotations: Record<string, string>): string {
  return Object.entries(annotations)
    .map(([key, value]) => `${key}: ${value}`)
    .join(', ');
}

/**
 * Format taints for display
 * @param taints - Array of taint strings
 * @returns Formatted taints string
 */
export function formatTaints(taints: string[]): string {
  return taints.join(', ');
}

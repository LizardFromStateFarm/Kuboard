// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Utility Functions
// This module contains helper functions and utilities

use anyhow::Result;

/// Parses CPU string (e.g., "1000m", "1") into CPU cores as f64
pub fn kuboard_parse_cpu_string(cpu_str: &str) -> Result<f64> {
    if cpu_str.ends_with('m') {
        // Millicores (e.g., "1000m" = 1 core)
        let millicores = cpu_str.trim_end_matches('m').parse::<f64>()?;
        Ok(millicores / 1000.0)
    } else {
        // Cores (e.g., "2")
        Ok(cpu_str.parse::<f64>()?)
    }
}

/// Parses memory string (e.g., "8Gi", "8192Mi") into bytes as u64
pub fn kuboard_parse_memory_string(memory_str: &str) -> Result<u64> {
    let memory_str = memory_str.trim();
    if memory_str.ends_with("Gi") {
        let gib = memory_str.trim_end_matches("Gi").parse::<f64>()?;
        Ok((gib * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if memory_str.ends_with("Mi") {
        let mib = memory_str.trim_end_matches("Mi").parse::<f64>()?;
        Ok((mib * 1024.0 * 1024.0) as u64)
    } else if memory_str.ends_with("Ki") {
        let kib = memory_str.trim_end_matches("Ki").parse::<f64>()?;
        Ok((kib * 1024.0) as u64)
    } else {
        // Assume bytes
        Ok(memory_str.parse::<u64>()?)
    }
}

/// Formats bytes into human-readable memory string
pub fn kuboard_format_memory(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Formats CPU cores into human-readable string
pub fn kuboard_format_cpu(cores: f64) -> String {
    if cores < 1.0 {
        format!("{:.0}m", cores * 1000.0)
    } else {
        format!("{:.1}", cores)
    }
}

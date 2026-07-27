use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct GrafanaConfig {
    pub url: String,
    pub api_token: Option<String>,
}

#[tauri::command]
pub async fn kuboard_grafana_test_connection(
    url: String,
    api_token: Option<String>,
) -> Result<bool, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let target_url = format!("{}/api/health", url.trim_end_matches('/'));
    let mut req = client.get(&target_url);

    if let Some(token) = &api_token {
        if !token.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token.trim()));
        }
    }

    match req.send().await {
        Ok(res) => Ok(res.status().is_success()),
        Err(e) => Err(format!("Grafana connection failed: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_grafana_discover_datasources(
    url: String,
    api_token: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let target_url = format!("{}/api/datasources", url.trim_end_matches('/'));
    let mut req = client.get(&target_url);

    if let Some(token) = &api_token {
        if !token.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token.trim()));
        }
    }

    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Grafana API returned HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn kuboard_grafana_query_promql(
    url: String,
    api_token: Option<String>,
    datasource_id: Option<i64>,
    query: String,
    start_time: i64,
    end_time: i64,
    step: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let base_url = url.trim_end_matches('/');
    let target_url = if let Some(ds_id) = datasource_id {
        format!("{}/api/datasources/proxy/{}/api/v1/query_range", base_url, ds_id)
    } else {
        format!("{}/api/v1/query_range", base_url)
    };

    let step_str = step.unwrap_or_else(|| "15s".to_string());

    let mut req = client.get(&target_url)
        .query(&[
            ("query", query.as_str()),
            ("start", &start_time.to_string()),
            ("end", &end_time.to_string()),
            ("step", &step_str),
        ]);

    if let Some(token) = &api_token {
        if !token.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", token.trim()));
        }
    }

    let response = req.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Grafana PromQL query failed with HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(json)
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Request for health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub server: String,
}

/// Response from health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub server_name: String,
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_check: String, // RFC3339 format
    pub error_count: u64,
    pub error_rate: f64, // 0.0-1.0
    pub details: Option<String>,
}

/// Health status of a server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Server is healthy (response time < 500ms, error rate < 5%)
    Healthy,
    /// Server performance is degraded (response time < 2000ms, error rate < 20%)
    Degraded,
    /// Server is unhealthy (response time >= 2000ms or error rate >= 20%)
    Unhealthy,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Internal health history for a server
#[derive(Debug, Clone)]
pub struct HealthHistory {
    pub server_name: String,
    pub recent_checks: Vec<HealthCheckResult>,
    pub max_history: usize,
}

impl HealthHistory {
    /// Create a new health history
    pub fn new(server_name: String, max_history: usize) -> Self {
        Self {
            server_name,
            recent_checks: Vec::new(),
            max_history,
        }
    }

    /// Add a new check result, maintaining max_history limit
    pub fn add_result(&mut self, result: HealthCheckResult) {
        self.recent_checks.push(result);

        // Maintain circular buffer by removing oldest entries
        while self.recent_checks.len() > self.max_history {
            self.recent_checks.remove(0);
        }
    }

    /// Calculate error count and error rate
    pub fn calculate_error_stats(&self) -> (u64, f64) {
        if self.recent_checks.is_empty() {
            return (0, 0.0);
        }

        let error_count = self.recent_checks.iter().filter(|r| !r.success).count() as u64;
        let total_count = self.recent_checks.len() as u64;
        let error_rate = error_count as f64 / total_count as f64;

        (error_count, error_rate)
    }
}

/// Single health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

impl HealthCheckResult {
    /// Create a successful health check result
    pub fn success(response_time_ms: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            success: true,
            response_time_ms,
            error_message: None,
        }
    }

    /// Create a failed health check result
    pub fn failure(response_time_ms: u64, error_message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            success: false,
            response_time_ms,
            error_message: Some(error_message),
        }
    }
}

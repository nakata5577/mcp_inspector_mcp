use crate::client::ClientManager;
use crate::models::{
    HealthCheckResponse, HealthCheckResult, HealthHistory, HealthStatus, Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Service for health checking MCP servers
///
/// This service maintains a history of health check results for each server
/// and uses that history to calculate error rates and determine health status.
pub struct HealthChecker {
    client_manager: Arc<ClientManager>,
    history: Arc<RwLock<HashMap<String, HealthHistory>>>,
    max_history: usize,
}

impl HealthChecker {
    /// Create a new HealthChecker
    ///
    /// # Arguments
    /// * `client_manager` - Manager for MCP client connections
    ///
    /// # Returns
    /// A new HealthChecker instance with default settings (max_history = 100)
    pub fn new(client_manager: Arc<ClientManager>) -> Self {
        Self {
            client_manager,
            history: Arc::new(RwLock::new(HashMap::new())),
            max_history: 100,
        }
    }

    /// Perform a health check on the specified server
    ///
    /// This method:
    /// 1. Sends a ping request to the server
    /// 2. Measures the response time
    /// 3. Records the result in history
    /// 4. Calculates error rate from recent history
    /// 5. Determines health status based on response time and error rate
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to check
    ///
    /// # Returns
    /// A HealthCheckResponse containing status, response time, and error statistics
    ///
    /// # Errors
    /// Returns an error if:
    /// - The server is not found in configuration
    /// - Client creation fails
    pub async fn check_health(&self, server_name: &str) -> Result<HealthCheckResponse> {
        // Start timer for response time measurement
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        // Get client and perform ping
        let client = self.client_manager.get_stdio_client(server_name).await?;
        let ping_result = client.ping().await;

        // Calculate response time
        let response_time_ms = start_time.elapsed().as_millis() as u64;

        // Create check result
        let check_result = match ping_result {
            Ok(_) => HealthCheckResult::success(response_time_ms),
            Err(e) => HealthCheckResult::failure(response_time_ms, e.to_string()),
        };

        // Update history
        self.update_history(server_name, check_result.clone())
            .await;

        // Calculate error statistics from history
        let (error_count, error_rate) = self.calculate_error_rate(server_name).await;

        // Determine health status
        let status = Self::determine_status(response_time_ms, error_rate);

        // Build response
        Ok(HealthCheckResponse {
            server_name: server_name.to_string(),
            status,
            response_time_ms,
            last_check: timestamp.to_rfc3339(),
            error_count,
            error_rate,
            details: check_result.error_message,
        })
    }

    /// Update health history for a server
    ///
    /// This method adds a new check result to the server's history,
    /// maintaining the circular buffer by removing oldest entries
    /// when the maximum size is reached.
    async fn update_history(&self, server_name: &str, result: HealthCheckResult) {
        let mut history_map = self.history.write().await;

        let history = history_map
            .entry(server_name.to_string())
            .or_insert_with(|| HealthHistory::new(server_name.to_string(), self.max_history));

        history.add_result(result);
    }

    /// Calculate error count and error rate from recent history
    ///
    /// # Arguments
    /// * `server_name` - Name of the server
    ///
    /// # Returns
    /// A tuple of (error_count, error_rate) where error_rate is between 0.0 and 1.0
    async fn calculate_error_rate(&self, server_name: &str) -> (u64, f64) {
        let history_map = self.history.read().await;

        if let Some(history) = history_map.get(server_name) {
            history.calculate_error_stats()
        } else {
            (0, 0.0)
        }
    }

    /// Determine health status based on response time and error rate
    ///
    /// Status thresholds:
    /// - Healthy: response_time < 500ms AND error_rate < 5%
    /// - Degraded: response_time < 2000ms AND error_rate < 20%
    /// - Unhealthy: response_time >= 2000ms OR error_rate >= 20%
    ///
    /// # Arguments
    /// * `response_time_ms` - Response time in milliseconds
    /// * `error_rate` - Error rate between 0.0 and 1.0
    ///
    /// # Returns
    /// The determined health status
    fn determine_status(response_time_ms: u64, error_rate: f64) -> HealthStatus {
        if response_time_ms >= 2000 || error_rate >= 0.2 {
            HealthStatus::Unhealthy
        } else if response_time_ms >= 500 || error_rate >= 0.05 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get the current health history for a server (for testing/debugging)
    #[cfg(test)]
    pub async fn get_history(&self, server_name: &str) -> Option<HealthHistory> {
        let history_map = self.history.read().await;
        history_map.get(server_name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_status_healthy() {
        // Response time < 500ms, error rate < 5%
        let status = HealthChecker::determine_status(400, 0.04);
        assert_eq!(status, HealthStatus::Healthy);

        // Edge case: exactly at threshold
        let status = HealthChecker::determine_status(499, 0.049);
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_determine_status_degraded() {
        // Response time between 500ms and 2000ms
        let status = HealthChecker::determine_status(1000, 0.04);
        assert_eq!(status, HealthStatus::Degraded);

        // Error rate between 5% and 20%
        let status = HealthChecker::determine_status(400, 0.1);
        assert_eq!(status, HealthStatus::Degraded);

        // Edge case: exactly at degraded threshold
        let status = HealthChecker::determine_status(500, 0.05);
        assert_eq!(status, HealthStatus::Degraded);
    }

    #[test]
    fn test_determine_status_unhealthy() {
        // Response time >= 2000ms
        let status = HealthChecker::determine_status(2000, 0.0);
        assert_eq!(status, HealthStatus::Unhealthy);

        // Error rate >= 20%
        let status = HealthChecker::determine_status(100, 0.2);
        assert_eq!(status, HealthStatus::Unhealthy);

        // Both conditions met
        let status = HealthChecker::determine_status(3000, 0.5);
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_history_circular_buffer() {
        let mut history = HealthHistory::new("test-server".to_string(), 3);

        // Add 5 results to a buffer with max_history=3
        for i in 0..5 {
            history.add_result(HealthCheckResult::success(100 + i));
        }

        // Should only keep the last 3 results
        assert_eq!(history.recent_checks.len(), 3);
        assert_eq!(history.recent_checks[0].response_time_ms, 102);
        assert_eq!(history.recent_checks[1].response_time_ms, 103);
        assert_eq!(history.recent_checks[2].response_time_ms, 104);
    }

    #[test]
    fn test_health_history_error_stats() {
        let mut history = HealthHistory::new("test-server".to_string(), 10);

        // Add 10 results: 3 failures, 7 successes
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::failure(200, "error 1".to_string()));
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::failure(200, "error 2".to_string()));
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::failure(200, "error 3".to_string()));
        history.add_result(HealthCheckResult::success(100));
        history.add_result(HealthCheckResult::success(100));

        let (error_count, error_rate) = history.calculate_error_stats();
        assert_eq!(error_count, 3);
        assert!((error_rate - 0.3).abs() < 0.01); // 3/10 = 0.3
    }

    #[test]
    fn test_health_history_empty() {
        let history = HealthHistory::new("test-server".to_string(), 10);
        let (error_count, error_rate) = history.calculate_error_stats();
        assert_eq!(error_count, 0);
        assert_eq!(error_rate, 0.0);
    }
}

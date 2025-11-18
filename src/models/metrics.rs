//! Performance metrics data structures
//!
//! This module provides data structures for collecting and aggregating
//! performance metrics of MCP tool executions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a metric record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricStatus {
    /// Request completed successfully
    Success,
    /// Request failed with an error
    Error,
}

impl fmt::Display for MetricStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricStatus::Success => write!(f, "Success"),
            MetricStatus::Error => write!(f, "Error"),
        }
    }
}

/// A single metric record for a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Name of the MCP server
    pub server_name: String,
    /// Name of the tool executed
    pub tool_name: String,
    /// Timestamp of the execution
    pub timestamp: DateTime<Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Status of the execution
    pub status: MetricStatus,
    /// Whether the response was served from cache
    pub cache_hit: bool,
    /// Whether an existing connection was reused
    pub connection_reused: bool,
}

impl Metrics {
    /// Creates a new Metrics record
    pub fn new(
        server_name: String,
        tool_name: String,
        response_time_ms: u64,
        status: MetricStatus,
        cache_hit: bool,
        connection_reused: bool,
    ) -> Self {
        Self {
            server_name,
            tool_name,
            timestamp: Utc::now(),
            response_time_ms,
            status,
            cache_hit,
            connection_reused,
        }
    }

    /// Returns true if this metric represents a successful execution
    pub fn is_success(&self) -> bool {
        self.status == MetricStatus::Success
    }

    /// Returns true if this metric represents an error
    pub fn is_error(&self) -> bool {
        self.status == MetricStatus::Error
    }
}

/// Time window for aggregated metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeWindow {
    /// Last hour
    LastHour,
    /// Last 24 hours
    Last24Hours,
    /// Last 7 days
    Last7Days,
    /// Custom duration in seconds
    Custom(u64),
}

impl TimeWindow {
    /// Returns the duration in seconds
    pub fn duration_secs(&self) -> u64 {
        match self {
            TimeWindow::LastHour => 3600,
            TimeWindow::Last24Hours => 86400,
            TimeWindow::Last7Days => 604800,
            TimeWindow::Custom(secs) => *secs,
        }
    }
}

impl fmt::Display for TimeWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeWindow::LastHour => write!(f, "Last Hour"),
            TimeWindow::Last24Hours => write!(f, "Last 24 Hours"),
            TimeWindow::Last7Days => write!(f, "Last 7 Days"),
            TimeWindow::Custom(secs) => write!(f, "Last {} seconds", secs),
        }
    }
}

/// Statistics for response times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeStats {
    /// Minimum response time in milliseconds
    pub min: u64,
    /// Maximum response time in milliseconds
    pub max: u64,
    /// Average response time in milliseconds
    pub avg: f64,
    /// 50th percentile (median) in milliseconds
    pub p50: u64,
    /// 95th percentile in milliseconds
    pub p95: u64,
    /// 99th percentile in milliseconds
    pub p99: u64,
}

impl Default for ResponseTimeStats {
    fn default() -> Self {
        Self {
            min: 0,
            max: 0,
            avg: 0.0,
            p50: 0,
            p95: 0,
            p99: 0,
        }
    }
}

impl ResponseTimeStats {

    /// Creates a ResponseTimeStats from a single value
    pub fn from_single(value: u64) -> Self {
        Self {
            min: value,
            max: value,
            avg: value as f64,
            p50: value,
            p95: value,
            p99: value,
        }
    }
}

/// Aggregated metrics over a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Name of the MCP server
    pub server_name: String,
    /// Name of the tool (None for server-level aggregation)
    pub tool_name: Option<String>,
    /// Time window for aggregation
    pub time_window: TimeWindow,
    /// Response time statistics
    pub response_time: ResponseTimeStats,
    /// Throughput in requests per second
    pub throughput: f64,
    /// Error rate as a percentage (0-100)
    pub error_rate: f64,
    /// Cache hit rate as a percentage (0-100)
    pub cache_hit_rate: f64,
    /// Connection reuse rate as a percentage (0-100)
    pub connection_reuse_rate: f64,
    /// Total number of requests
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
}

impl AggregatedMetrics {
    /// Creates a new AggregatedMetrics with default values
    pub fn new(server_name: String, tool_name: Option<String>, time_window: TimeWindow) -> Self {
        Self {
            server_name,
            tool_name,
            time_window,
            response_time: ResponseTimeStats::default(),
            throughput: 0.0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
            connection_reuse_rate: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_status_display() {
        assert_eq!(MetricStatus::Success.to_string(), "Success");
        assert_eq!(MetricStatus::Error.to_string(), "Error");
    }

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new(
            "test_server".to_string(),
            "test_tool".to_string(),
            123,
            MetricStatus::Success,
            true,
            false,
        );

        assert_eq!(metrics.server_name, "test_server");
        assert_eq!(metrics.tool_name, "test_tool");
        assert_eq!(metrics.response_time_ms, 123);
        assert_eq!(metrics.status, MetricStatus::Success);
        assert!(metrics.cache_hit);
        assert!(!metrics.connection_reused);
    }

    #[test]
    fn test_metrics_is_success() {
        let success_metrics = Metrics::new(
            "server".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        assert!(success_metrics.is_success());
        assert!(!success_metrics.is_error());
    }

    #[test]
    fn test_metrics_is_error() {
        let error_metrics = Metrics::new(
            "server".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Error,
            false,
            false,
        );
        assert!(error_metrics.is_error());
        assert!(!error_metrics.is_success());
    }

    #[test]
    fn test_time_window_duration_secs() {
        assert_eq!(TimeWindow::LastHour.duration_secs(), 3600);
        assert_eq!(TimeWindow::Last24Hours.duration_secs(), 86400);
        assert_eq!(TimeWindow::Last7Days.duration_secs(), 604800);
        assert_eq!(TimeWindow::Custom(1800).duration_secs(), 1800);
    }

    #[test]
    fn test_time_window_display() {
        assert_eq!(TimeWindow::LastHour.to_string(), "Last Hour");
        assert_eq!(TimeWindow::Last24Hours.to_string(), "Last 24 Hours");
        assert_eq!(TimeWindow::Last7Days.to_string(), "Last 7 Days");
        assert_eq!(TimeWindow::Custom(1800).to_string(), "Last 1800 seconds");
    }

    #[test]
    fn test_response_time_stats_default() {
        let stats = ResponseTimeStats::default();
        assert_eq!(stats.min, 0);
        assert_eq!(stats.max, 0);
        assert_eq!(stats.avg, 0.0);
        assert_eq!(stats.p50, 0);
        assert_eq!(stats.p95, 0);
        assert_eq!(stats.p99, 0);
    }

    #[test]
    fn test_response_time_stats_from_single() {
        let stats = ResponseTimeStats::from_single(100);
        assert_eq!(stats.min, 100);
        assert_eq!(stats.max, 100);
        assert_eq!(stats.avg, 100.0);
        assert_eq!(stats.p50, 100);
        assert_eq!(stats.p95, 100);
        assert_eq!(stats.p99, 100);
    }

    #[test]
    fn test_aggregated_metrics_new() {
        let metrics = AggregatedMetrics::new(
            "test_server".to_string(),
            Some("test_tool".to_string()),
            TimeWindow::LastHour,
        );

        assert_eq!(metrics.server_name, "test_server");
        assert_eq!(metrics.tool_name, Some("test_tool".to_string()));
        assert_eq!(metrics.time_window, TimeWindow::LastHour);
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.throughput, 0.0);
    }
}

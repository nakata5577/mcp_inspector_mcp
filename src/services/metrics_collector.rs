//! Metrics collector service
//!
//! This module provides functionality to collect and aggregate performance metrics
//! for MCP tool executions.

use crate::models::{AggregatedMetrics, MetricStatus, Metrics, ResponseTimeStats, TimeWindow};
use chrono::{Duration, Utc};
use hdrhistogram::Histogram;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Maximum number of metrics to keep in the circular buffer
const MAX_METRICS_BUFFER_SIZE: usize = 10_000;

/// Histogram configuration for response time tracking
/// We track response times from 1ms to 60 seconds with 3 significant digits
const HISTOGRAM_MIN: u64 = 1;
const HISTOGRAM_MAX: u64 = 60_000;
const HISTOGRAM_SIGFIG: u8 = 3;

/// Inner state of the metrics collector
struct MetricsCollectorState {
    /// Circular buffer of metrics (most recent MAX_METRICS_BUFFER_SIZE entries)
    metrics_buffer: VecDeque<Metrics>,
}

impl MetricsCollectorState {
    fn new() -> Self {
        Self {
            metrics_buffer: VecDeque::with_capacity(MAX_METRICS_BUFFER_SIZE),
        }
    }

    /// Adds a metric to the buffer, removing the oldest if at capacity
    fn add_metric(&mut self, metric: Metrics) {
        if self.metrics_buffer.len() >= MAX_METRICS_BUFFER_SIZE {
            self.metrics_buffer.pop_front();
        }
        self.metrics_buffer.push_back(metric);
    }

    /// Returns metrics within the specified time window
    fn get_metrics_in_window(&self, window: TimeWindow) -> Vec<Metrics> {
        let cutoff_time = Utc::now() - Duration::seconds(window.duration_secs() as i64);
        self.metrics_buffer
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .cloned()
            .collect()
    }
}

/// Service for collecting and aggregating performance metrics
#[derive(Clone)]
pub struct MetricsCollector {
    state: Arc<Mutex<MetricsCollectorState>>,
}

impl MetricsCollector {
    /// Creates a new MetricsCollector
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MetricsCollectorState::new())),
        }
    }

    /// Records a metric for a tool execution
    pub fn record_metric(
        &self,
        server_name: String,
        tool_name: String,
        response_time_ms: u64,
        status: MetricStatus,
        cache_hit: bool,
        connection_reused: bool,
    ) {
        let metric = Metrics::new(
            server_name,
            tool_name,
            response_time_ms,
            status,
            cache_hit,
            connection_reused,
        );

        let mut state = match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Metrics collector mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        state.add_metric(metric);
    }

    /// Returns the total number of metrics currently stored
    pub fn total_metrics(&self) -> usize {
        let state = match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Metrics collector mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        state.metrics_buffer.len()
    }

    /// Returns all metrics within the specified time window
    pub fn get_metrics(&self, window: TimeWindow) -> Vec<Metrics> {
        let state = match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Metrics collector mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        state.get_metrics_in_window(window)
    }

    /// Calculates aggregated metrics for a specific server and optional tool
    pub fn aggregate_metrics(
        &self,
        server_name: &str,
        tool_name: Option<&str>,
        window: TimeWindow,
    ) -> AggregatedMetrics {
        let metrics = self.get_metrics(window);

        // Filter metrics by server and tool
        let filtered: Vec<_> = metrics
            .iter()
            .filter(|m| {
                m.server_name == server_name
                    && tool_name.is_none_or(|tool| m.tool_name == tool)
            })
            .collect();

        if filtered.is_empty() {
            return AggregatedMetrics::new(
                server_name.to_string(),
                tool_name.map(|s| s.to_string()),
                window,
            );
        }

        let total_requests = filtered.len() as u64;
        let successful_requests = filtered.iter().filter(|m| m.is_success()).count() as u64;
        let failed_requests = total_requests - successful_requests;
        let cache_hits = filtered.iter().filter(|m| m.cache_hit).count() as u64;
        let connection_reuses = filtered.iter().filter(|m| m.connection_reused).count() as u64;

        // Calculate response time statistics
        let response_time = self.calculate_response_time_stats(&filtered);

        // Calculate rates
        let duration_secs = window.duration_secs() as f64;
        let throughput = if duration_secs > 0.0 {
            total_requests as f64 / duration_secs
        } else {
            0.0
        };

        let error_rate = if total_requests > 0 {
            (failed_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let cache_hit_rate = if total_requests > 0 {
            (cache_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let connection_reuse_rate = if total_requests > 0 {
            (connection_reuses as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        AggregatedMetrics {
            server_name: server_name.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            time_window: window,
            response_time,
            throughput,
            error_rate,
            cache_hit_rate,
            connection_reuse_rate,
            total_requests,
            successful_requests,
            failed_requests,
        }
    }

    /// Calculates response time statistics including percentiles
    fn calculate_response_time_stats(&self, metrics: &[&Metrics]) -> ResponseTimeStats {
        if metrics.is_empty() {
            return ResponseTimeStats::default();
        }

        if metrics.len() == 1 {
            return ResponseTimeStats::from_single(metrics[0].response_time_ms);
        }

        // Create histogram for percentile calculation
        let mut histogram = Histogram::<u64>::new_with_bounds(
            HISTOGRAM_MIN,
            HISTOGRAM_MAX,
            HISTOGRAM_SIGFIG,
        )
        .or_else(|_| Histogram::new(HISTOGRAM_SIGFIG))
        .expect("Failed to create histogram - this should never fail with valid SIGFIG");

        let mut min = u64::MAX;
        let mut max = u64::MIN;
        let mut sum: u64 = 0;

        for metric in metrics {
            let value = metric.response_time_ms;
            min = min.min(value);
            max = max.max(value);
            sum += value;

            // Record in histogram, clamping to valid range
            let clamped_value = value.clamp(HISTOGRAM_MIN, HISTOGRAM_MAX);
            let _ = histogram.record(clamped_value);
        }

        let count = metrics.len() as u64;
        let avg = sum as f64 / count as f64;

        let p50 = histogram.value_at_percentile(50.0);
        let p95 = histogram.value_at_percentile(95.0);
        let p99 = histogram.value_at_percentile(99.0);

        ResponseTimeStats {
            min,
            max,
            avg,
            p50,
            p95,
            p99,
        }
    }

    /// Returns aggregated metrics grouped by server
    pub fn aggregate_by_server(&self, window: TimeWindow) -> HashMap<String, AggregatedMetrics> {
        let metrics = self.get_metrics(window);
        let mut servers: HashMap<String, Vec<&Metrics>> = HashMap::new();

        for metric in &metrics {
            servers
                .entry(metric.server_name.clone())
                .or_default()
                .push(metric);
        }

        servers.into_keys().map(|server_name| {
                let aggregated = self.aggregate_metrics(&server_name, None, window);
                (server_name, aggregated)
            })
            .collect()
    }

    /// Returns aggregated metrics grouped by tool for a specific server
    pub fn aggregate_by_tool(
        &self,
        server_name: &str,
        window: TimeWindow,
    ) -> HashMap<String, AggregatedMetrics> {
        let metrics = self.get_metrics(window);
        let mut tools: HashMap<String, Vec<&Metrics>> = HashMap::new();

        for metric in metrics.iter().filter(|m| m.server_name == server_name) {
            tools
                .entry(metric.tool_name.clone())
                .or_default()
                .push(metric);
        }

        tools.into_keys().map(|tool_name| {
                let aggregated = self.aggregate_metrics(server_name, Some(&tool_name), window);
                (tool_name, aggregated)
            })
            .collect()
    }

    /// Clears all stored metrics
    #[cfg(test)]
    pub fn clear(&self) {
        let mut state = match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Metrics collector mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        state.metrics_buffer.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.total_metrics(), 0);
    }

    #[test]
    fn test_record_metric() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "test_server".to_string(),
            "test_tool".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        assert_eq!(collector.total_metrics(), 1);
    }

    #[test]
    fn test_record_multiple_metrics() {
        let collector = MetricsCollector::new();
        for i in 0..10 {
            collector.record_metric(
                "server".to_string(),
                "tool".to_string(),
                100 + i,
                MetricStatus::Success,
                false,
                false,
            );
        }
        assert_eq!(collector.total_metrics(), 10);
    }

    #[test]
    fn test_circular_buffer_limit() {
        let collector = MetricsCollector::new();
        for i in 0..MAX_METRICS_BUFFER_SIZE + 100 {
            collector.record_metric(
                "server".to_string(),
                "tool".to_string(),
                i as u64,
                MetricStatus::Success,
                false,
                false,
            );
        }
        assert_eq!(collector.total_metrics(), MAX_METRICS_BUFFER_SIZE);
    }

    #[test]
    fn test_get_metrics_empty() {
        let collector = MetricsCollector::new();
        let metrics = collector.get_metrics(TimeWindow::LastHour);
        assert_eq!(metrics.len(), 0);
    }

    #[test]
    fn test_get_metrics_within_window() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        let metrics = collector.get_metrics(TimeWindow::LastHour);
        assert_eq!(metrics.len(), 1);
    }

    #[test]
    fn test_aggregate_metrics_empty() {
        let collector = MetricsCollector::new();
        let aggregated = collector.aggregate_metrics("server", None, TimeWindow::LastHour);
        assert_eq!(aggregated.total_requests, 0);
        assert_eq!(aggregated.throughput, 0.0);
    }

    #[test]
    fn test_aggregate_metrics_single() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Success,
            true,
            false,
        );

        let aggregated = collector.aggregate_metrics("server", None, TimeWindow::LastHour);
        assert_eq!(aggregated.total_requests, 1);
        assert_eq!(aggregated.successful_requests, 1);
        assert_eq!(aggregated.failed_requests, 0);
        assert_eq!(aggregated.response_time.min, 100);
        assert_eq!(aggregated.response_time.max, 100);
        assert_eq!(aggregated.response_time.avg, 100.0);
    }

    #[test]
    fn test_aggregate_metrics_multiple() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Success,
            true,
            false,
        );
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            200,
            MetricStatus::Success,
            false,
            true,
        );
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            300,
            MetricStatus::Error,
            false,
            false,
        );

        let aggregated = collector.aggregate_metrics("server", None, TimeWindow::LastHour);
        assert_eq!(aggregated.total_requests, 3);
        assert_eq!(aggregated.successful_requests, 2);
        assert_eq!(aggregated.failed_requests, 1);
        assert_eq!(aggregated.response_time.min, 100);
        assert_eq!(aggregated.response_time.max, 300);
        assert!((aggregated.response_time.avg - 200.0).abs() < 0.01);
        assert!((aggregated.error_rate - 33.33).abs() < 0.1);
        assert!((aggregated.cache_hit_rate - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_aggregate_metrics_filter_by_tool() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server".to_string(),
            "tool1".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        collector.record_metric(
            "server".to_string(),
            "tool2".to_string(),
            200,
            MetricStatus::Success,
            false,
            false,
        );

        let aggregated = collector.aggregate_metrics("server", Some("tool1"), TimeWindow::LastHour);
        assert_eq!(aggregated.total_requests, 1);
        assert_eq!(aggregated.response_time.min, 100);
    }

    #[test]
    fn test_calculate_response_time_stats_empty() {
        let collector = MetricsCollector::new();
        let stats = collector.calculate_response_time_stats(&[]);
        assert_eq!(stats.min, 0);
        assert_eq!(stats.max, 0);
        assert_eq!(stats.avg, 0.0);
    }

    #[test]
    fn test_calculate_response_time_stats_percentiles() {
        let collector = MetricsCollector::new();
        // Add 100 metrics with values from 1 to 100
        for i in 1..=100 {
            collector.record_metric(
                "server".to_string(),
                "tool".to_string(),
                i,
                MetricStatus::Success,
                false,
                false,
            );
        }

        let metrics = collector.get_metrics(TimeWindow::LastHour);
        let metric_refs: Vec<_> = metrics.iter().collect();
        let stats = collector.calculate_response_time_stats(&metric_refs);

        assert_eq!(stats.min, 1);
        assert_eq!(stats.max, 100);
        assert!((stats.avg - 50.5).abs() < 1.0);
        // Percentiles should be approximately correct
        assert!(stats.p50 >= 45 && stats.p50 <= 55);
        assert!(stats.p95 >= 90 && stats.p95 <= 100);
        assert!(stats.p99 >= 95 && stats.p99 <= 100);
    }

    #[test]
    fn test_aggregate_by_server() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server1".to_string(),
            "tool".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        collector.record_metric(
            "server2".to_string(),
            "tool".to_string(),
            200,
            MetricStatus::Success,
            false,
            false,
        );

        let aggregated = collector.aggregate_by_server(TimeWindow::LastHour);
        assert_eq!(aggregated.len(), 2);
        assert!(aggregated.contains_key("server1"));
        assert!(aggregated.contains_key("server2"));
    }

    #[test]
    fn test_aggregate_by_tool() {
        let collector = MetricsCollector::new();
        collector.record_metric(
            "server".to_string(),
            "tool1".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );
        collector.record_metric(
            "server".to_string(),
            "tool2".to_string(),
            200,
            MetricStatus::Success,
            false,
            false,
        );

        let aggregated = collector.aggregate_by_tool("server", TimeWindow::LastHour);
        assert_eq!(aggregated.len(), 2);
        assert!(aggregated.contains_key("tool1"));
        assert!(aggregated.contains_key("tool2"));
    }
}

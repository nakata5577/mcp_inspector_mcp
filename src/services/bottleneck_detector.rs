//! Bottleneck detector service
//!
//! This module provides functionality to detect performance bottlenecks
//! by analyzing aggregated metrics and identifying anomalies.

use crate::models::{AggregatedMetrics, TimeWindow};
use crate::services::MetricsCollector;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for bottleneck detection thresholds
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Response time threshold in milliseconds (default: 1000ms)
    pub response_time_threshold_ms: u64,
    /// Error rate threshold as a decimal (default: 0.05 = 5%)
    pub error_rate_threshold: f64,
    /// Throughput threshold in requests per second (default: 1.0)
    pub throughput_threshold: f64,
    /// Memory usage threshold in MB (optional)
    pub memory_threshold_mb: Option<u64>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            response_time_threshold_ms: 1000,
            error_rate_threshold: 0.05,
            throughput_threshold: 1.0,
            memory_threshold_mb: None,
        }
    }
}

/// Type of bottleneck detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BottleneckType {
    /// High response time detected
    HighResponseTime,
    /// High error rate detected
    HighErrorRate,
    /// Low throughput detected
    LowThroughput,
    /// High memory usage detected
    HighMemoryUsage,
}

/// Severity level of the bottleneck
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Warning level (100-150% of threshold)
    Warning,
    /// Critical level (>150% of threshold)
    Critical,
}

/// A detected bottleneck with context and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Name of the MCP server
    pub server_name: String,
    /// Name of the tool (None for server-level bottleneck)
    pub tool_name: Option<String>,
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Severity level
    pub severity: Severity,
    /// Current measured value
    pub current_value: f64,
    /// Configured threshold
    pub threshold: f64,
    /// When the bottleneck was detected
    pub detected_at: DateTime<Utc>,
    /// Recommendation for addressing the bottleneck
    pub recommendation: String,
}

impl Bottleneck {
    /// Formats the bottleneck as a human-readable string
    pub fn format_alert(&self) -> String {
        let severity_icon = match self.severity {
            Severity::Warning => "⚠️",
            Severity::Critical => "🔴",
        };

        let tool_info = if let Some(ref tool) = self.tool_name {
            format!(" | Tool: {}", tool)
        } else {
            String::new()
        };

        let (value_str, threshold_str) = match self.bottleneck_type {
            BottleneckType::HighResponseTime => (
                format!("{}ms", self.current_value as u64),
                format!("{}ms", self.threshold as u64),
            ),
            BottleneckType::HighErrorRate => (
                format!("{:.1}%", self.current_value * 100.0),
                format!("{:.1}%", self.threshold * 100.0),
            ),
            BottleneckType::LowThroughput => (
                format!("{:.2} req/s", self.current_value),
                format!("{:.2} req/s", self.threshold),
            ),
            BottleneckType::HighMemoryUsage => (
                format!("{}MB", self.current_value as u64),
                format!("{}MB", self.threshold as u64),
            ),
        };

        format!(
            r#"{}  BOTTLENECK DETECTED

Server: {}{}
Type: {:?} | Severity: {:?}
Current: {} | Threshold: {}
Detected at: {}

Recommendation:
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"#,
            severity_icon,
            self.server_name,
            tool_info,
            self.bottleneck_type,
            self.severity,
            value_str,
            threshold_str,
            self.detected_at.format("%Y-%m-%dT%H:%M:%SZ"),
            self.recommendation
        )
    }
}

/// Service for detecting performance bottlenecks
pub struct BottleneckDetector {
    /// Reference to the metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Detection configuration
    config: DetectionConfig,
}

impl BottleneckDetector {
    /// Creates a new BottleneckDetector with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `metrics_collector` - Arc reference to the metrics collector
    /// * `config` - Detection configuration with thresholds
    pub fn new(metrics_collector: Arc<MetricsCollector>, config: DetectionConfig) -> Self {
        Self {
            metrics_collector,
            config,
        }
    }

    /// Detects bottlenecks across all servers and tools in the time window
    ///
    /// # Arguments
    ///
    /// * `time_window` - Time window to analyze
    ///
    /// # Returns
    ///
    /// A vector of detected bottlenecks
    pub fn detect_bottlenecks(&self, time_window: TimeWindow) -> Vec<Bottleneck> {
        let server_aggregates = self.metrics_collector.aggregate_by_server(time_window);
        let mut bottlenecks = Vec::new();

        // Check server-level bottlenecks
        for (server_name, metrics) in &server_aggregates {
            bottlenecks.extend(self.analyze_metrics(metrics));

            // Also check tool-level bottlenecks for each server
            let tool_aggregates = self
                .metrics_collector
                .aggregate_by_tool(server_name, time_window);

            for metrics in tool_aggregates.values() {
                bottlenecks.extend(self.analyze_metrics(metrics));
            }
        }

        bottlenecks
    }

    /// Detects bottlenecks for a specific server in the time window
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the server to analyze
    /// * `time_window` - Time window to analyze
    ///
    /// # Returns
    ///
    /// A vector of detected bottlenecks for the specified server
    pub fn detect_server_bottlenecks(
        &self,
        server_name: &str,
        time_window: TimeWindow,
    ) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Check server-level metrics
        let server_metrics = self
            .metrics_collector
            .aggregate_metrics(server_name, None, time_window);
        bottlenecks.extend(self.analyze_metrics(&server_metrics));

        // Check tool-level metrics
        let tool_aggregates = self
            .metrics_collector
            .aggregate_by_tool(server_name, time_window);

        for metrics in tool_aggregates.values() {
            bottlenecks.extend(self.analyze_metrics(metrics));
        }

        bottlenecks
    }

    /// Analyzes a single aggregated metrics entry for bottlenecks
    fn analyze_metrics(&self, metrics: &AggregatedMetrics) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Check response time
        if let Some(bottleneck) = self.check_response_time(metrics) {
            bottlenecks.push(bottleneck);
        }

        // Check error rate
        if let Some(bottleneck) = self.check_error_rate(metrics) {
            bottlenecks.push(bottleneck);
        }

        // Check throughput
        if let Some(bottleneck) = self.check_throughput(metrics) {
            bottlenecks.push(bottleneck);
        }

        bottlenecks
    }

    /// Checks if response time exceeds threshold
    fn check_response_time(&self, metrics: &AggregatedMetrics) -> Option<Bottleneck> {
        let p99_ms = metrics.response_time.p99;
        let threshold_ms = self.config.response_time_threshold_ms;

        if p99_ms > threshold_ms {
            let severity = self.calculate_severity(p99_ms as f64, threshold_ms as f64);
            let recommendation = self.generate_recommendation(&BottleneckType::HighResponseTime);

            Some(Bottleneck {
                server_name: metrics.server_name.clone(),
                tool_name: metrics.tool_name.clone(),
                bottleneck_type: BottleneckType::HighResponseTime,
                severity,
                current_value: p99_ms as f64,
                threshold: threshold_ms as f64,
                detected_at: Utc::now(),
                recommendation,
            })
        } else {
            None
        }
    }

    /// Checks if error rate exceeds threshold
    fn check_error_rate(&self, metrics: &AggregatedMetrics) -> Option<Bottleneck> {
        // Error rate in metrics is stored as percentage (0-100), convert to decimal
        let error_rate = metrics.error_rate / 100.0;
        let threshold = self.config.error_rate_threshold;

        if error_rate > threshold {
            let severity = self.calculate_severity(error_rate, threshold);
            let recommendation = self.generate_recommendation(&BottleneckType::HighErrorRate);

            Some(Bottleneck {
                server_name: metrics.server_name.clone(),
                tool_name: metrics.tool_name.clone(),
                bottleneck_type: BottleneckType::HighErrorRate,
                severity,
                current_value: error_rate,
                threshold,
                detected_at: Utc::now(),
                recommendation,
            })
        } else {
            None
        }
    }

    /// Checks if throughput is below threshold
    fn check_throughput(&self, metrics: &AggregatedMetrics) -> Option<Bottleneck> {
        let throughput = metrics.throughput;
        let threshold = self.config.throughput_threshold;

        if throughput < threshold {
            let severity = self.calculate_severity(threshold, throughput);
            let recommendation = self.generate_recommendation(&BottleneckType::LowThroughput);

            Some(Bottleneck {
                server_name: metrics.server_name.clone(),
                tool_name: metrics.tool_name.clone(),
                bottleneck_type: BottleneckType::LowThroughput,
                severity,
                current_value: throughput,
                threshold,
                detected_at: Utc::now(),
                recommendation,
            })
        } else {
            None
        }
    }

    /// Calculates severity based on how much the current value exceeds the threshold
    ///
    /// # Arguments
    ///
    /// * `current` - Current measured value
    /// * `threshold` - Configured threshold
    ///
    /// # Returns
    ///
    /// Severity level (Warning for 100-150%, Critical for >150%)
    fn calculate_severity(&self, current: f64, threshold: f64) -> Severity {
        let ratio = current / threshold;

        if ratio >= 1.5 {
            Severity::Critical
        } else {
            Severity::Warning
        }
    }

    /// Generates a recommendation string based on bottleneck type
    ///
    /// # Arguments
    ///
    /// * `bottleneck_type` - Type of bottleneck detected
    ///
    /// # Returns
    ///
    /// A formatted recommendation string with actionable advice
    fn generate_recommendation(&self, bottleneck_type: &BottleneckType) -> String {
        match bottleneck_type {
            BottleneckType::HighResponseTime => {
                r#"- Check server logs for errors or slow operations
- Consider increasing timeout settings
- Review tool implementation for performance issues
- Monitor system resources (CPU, memory, disk I/O)"#
                    .to_string()
            }
            BottleneckType::HighErrorRate => {
                r#"- Review error logs for common error patterns
- Check network connectivity and server availability
- Verify input parameters and request format
- Consider implementing retry logic with backoff"#
                    .to_string()
            }
            BottleneckType::LowThroughput => {
                r#"- Consider parallel processing implementation
- Adjust batch sizes for better efficiency
- Review caching strategy
- Increase resource pool sizes"#
                    .to_string()
            }
            BottleneckType::HighMemoryUsage => {
                r#"- Check for memory leaks
- Optimize large data structures
- Tune garbage collection settings
- Consider streaming processing approach"#
                    .to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResponseTimeStats;

    fn create_test_metrics_collector() -> Arc<MetricsCollector> {
        Arc::new(MetricsCollector::new())
    }

    fn create_test_aggregated_metrics(
        server_name: &str,
        tool_name: Option<&str>,
        p99_ms: u64,
        error_rate: f64,
        throughput: f64,
    ) -> AggregatedMetrics {
        AggregatedMetrics {
            server_name: server_name.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            time_window: TimeWindow::LastHour,
            response_time: ResponseTimeStats {
                min: 10,
                max: p99_ms,
                avg: (p99_ms / 2) as f64,
                p50: p99_ms / 2,
                p95: (p99_ms as f64 * 0.95) as u64,
                p99: p99_ms,
            },
            throughput,
            error_rate,
            cache_hit_rate: 0.0,
            connection_reuse_rate: 0.0,
            total_requests: 100,
            successful_requests: ((100.0 - error_rate) as u64),
            failed_requests: error_rate as u64,
        }
    }

    #[test]
    fn test_detect_high_response_time_warning() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 1200, 0.0, 10.0);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::HighResponseTime);
        assert_eq!(bottlenecks[0].severity, Severity::Warning);
        assert_eq!(bottlenecks[0].current_value, 1200.0);
        assert_eq!(bottlenecks[0].threshold, 1000.0);
    }

    #[test]
    fn test_detect_high_response_time_critical() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 2000, 0.0, 10.0);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::HighResponseTime);
        assert_eq!(bottlenecks[0].severity, Severity::Critical);
        assert_eq!(bottlenecks[0].current_value, 2000.0);
    }

    #[test]
    fn test_no_bottleneck_when_below_threshold() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 500, 0.0, 10.0);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 0);
    }

    #[test]
    fn test_detect_high_error_rate_warning() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        // Error rate in metrics is percentage (0-100), so 6.0 = 6%
        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 500, 6.0, 10.0);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::HighErrorRate);
        assert_eq!(bottlenecks[0].severity, Severity::Warning);
    }

    #[test]
    fn test_detect_high_error_rate_critical() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        // 10% error rate (threshold is 5%)
        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 500, 10.0, 10.0);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::HighErrorRate);
        assert_eq!(bottlenecks[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_low_throughput_warning() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 500, 0.0, 0.8);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::LowThroughput);
        assert_eq!(bottlenecks[0].severity, Severity::Warning);
    }

    #[test]
    fn test_detect_low_throughput_critical() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let metrics = create_test_aggregated_metrics("test_server", Some("test_tool"), 500, 0.0, 0.5);

        let bottlenecks = detector.analyze_metrics(&metrics);

        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].bottleneck_type, BottleneckType::LowThroughput);
        assert_eq!(bottlenecks[0].severity, Severity::Critical);
    }

    #[test]
    fn test_calculate_severity_warning() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let severity = detector.calculate_severity(1200.0, 1000.0);
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn test_calculate_severity_critical() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let severity = detector.calculate_severity(1600.0, 1000.0);
        assert_eq!(severity, Severity::Critical);
    }

    #[test]
    fn test_generate_recommendation() {
        let collector = create_test_metrics_collector();
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(collector, config);

        let rec = detector.generate_recommendation(&BottleneckType::HighResponseTime);
        assert!(rec.contains("Check server logs"));

        let rec = detector.generate_recommendation(&BottleneckType::HighErrorRate);
        assert!(rec.contains("Review error logs"));

        let rec = detector.generate_recommendation(&BottleneckType::LowThroughput);
        assert!(rec.contains("parallel processing"));

        let rec = detector.generate_recommendation(&BottleneckType::HighMemoryUsage);
        assert!(rec.contains("memory leaks"));
    }
}

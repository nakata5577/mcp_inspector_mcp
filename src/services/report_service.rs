//! Report generation service
//!
//! This module provides functionality to generate performance reports in multiple formats:
//! - Console: Human-readable table format
//! - JSON: Programmatic access format
//! - HTML: Browser-viewable format

use crate::models::{AggregatedMetrics, TimeWindow};
use crate::services::MetricsCollector;
use anyhow::Result;
use chrono::Utc;
use serde_json;
use std::sync::Arc;

/// Escapes HTML special characters to prevent XSS attacks
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Report format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Console table format
    Console,
    /// JSON format
    Json,
    /// HTML format
    Html,
}

/// Service for generating performance reports
pub struct ReportService {
    metrics_collector: Arc<MetricsCollector>,
}

impl ReportService {
    /// Creates a new ReportService
    ///
    /// # Arguments
    ///
    /// * `metrics_collector` - Arc reference to MetricsCollector
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self { metrics_collector }
    }

    /// Generates a comprehensive report for all servers
    ///
    /// # Arguments
    ///
    /// * `time_window` - Time window for the report
    /// * `format` - Output format (Console, JSON, or HTML)
    ///
    /// # Returns
    ///
    /// Formatted report string
    pub fn generate_report(&self, time_window: TimeWindow, format: ReportFormat) -> Result<String> {
        let server_metrics = self.metrics_collector.aggregate_by_server(time_window);

        let mut all_metrics = Vec::new();
        for (server_name, server_agg) in server_metrics {
            // Add server-level metrics
            all_metrics.push(server_agg);

            // Add tool-level metrics for this server
            let tool_metrics = self.metrics_collector.aggregate_by_tool(&server_name, time_window);
            all_metrics.extend(tool_metrics.into_values());
        }

        match format {
            ReportFormat::Console => Ok(self.format_console_report(&all_metrics, time_window)),
            ReportFormat::Json => self.format_json_report(&all_metrics),
            ReportFormat::Html => Ok(self.format_html_report(&all_metrics, time_window)),
        }
    }

    /// Generates a report for a specific server
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the server
    /// * `time_window` - Time window for the report
    /// * `format` - Output format (Console, JSON, or HTML)
    ///
    /// # Returns
    ///
    /// Formatted report string
    pub fn generate_server_report(
        &self,
        server_name: &str,
        time_window: TimeWindow,
        format: ReportFormat,
    ) -> Result<String> {
        let server_agg = self
            .metrics_collector
            .aggregate_metrics(server_name, None, time_window);
        let tool_metrics = self
            .metrics_collector
            .aggregate_by_tool(server_name, time_window);

        let mut metrics = vec![server_agg];
        metrics.extend(tool_metrics.into_values());

        match format {
            ReportFormat::Console => Ok(self.format_console_report(&metrics, time_window)),
            ReportFormat::Json => self.format_json_report(&metrics),
            ReportFormat::Html => Ok(self.format_html_report(&metrics, time_window)),
        }
    }

    /// Generates a report for a specific tool within a server
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the server
    /// * `tool_name` - Name of the tool
    /// * `time_window` - Time window for the report
    /// * `format` - Output format (Console, JSON, or HTML)
    ///
    /// # Returns
    ///
    /// Formatted report string
    pub fn generate_tool_report(
        &self,
        server_name: &str,
        tool_name: &str,
        time_window: TimeWindow,
        format: ReportFormat,
    ) -> Result<String> {
        let tool_agg = self
            .metrics_collector
            .aggregate_metrics(server_name, Some(tool_name), time_window);
        let metrics = vec![tool_agg];

        match format {
            ReportFormat::Console => Ok(self.format_console_report(&metrics, time_window)),
            ReportFormat::Json => self.format_json_report(&metrics),
            ReportFormat::Html => Ok(self.format_html_report(&metrics, time_window)),
        }
    }

    /// Formats metrics as a console table
    fn format_console_report(&self, metrics: &[AggregatedMetrics], time_window: TimeWindow) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "=== Performance Report ({}) ===\n\n",
            self.format_time_window_label(time_window)
        ));

        if metrics.is_empty() {
            output.push_str("No metrics available for the specified time window.\n");
            return output;
        }

        // Group metrics by server
        let mut server_groups: std::collections::HashMap<String, Vec<&AggregatedMetrics>> =
            std::collections::HashMap::new();
        for metric in metrics {
            server_groups
                .entry(metric.server_name.clone())
                .or_default()
                .push(metric);
        }

        // Sort servers by name
        let mut server_names: Vec<_> = server_groups.keys().cloned().collect();
        server_names.sort();

        for server_name in server_names {
            let server_metrics = server_groups.get(&server_name).unwrap();

            // Find server-level metric (tool_name is None)
            let server_level = server_metrics
                .iter()
                .find(|m| m.tool_name.is_none())
                .copied();

            if let Some(server_agg) = server_level {
                output.push_str(&format!("Server: {}\n", server_name));
                output.push_str(&self.format_metric_details(server_agg, 2));
                output.push('\n');
            }

            // Tool-level metrics
            let mut tool_metrics: Vec<_> = server_metrics
                .iter()
                .filter(|m| m.tool_name.is_some())
                .copied()
                .collect();

            // Sort tools by name
            tool_metrics.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));

            for tool_agg in tool_metrics {
                if let Some(tool_name) = &tool_agg.tool_name {
                    output.push_str(&format!("  Tool: {}\n", tool_name));
                    output.push_str(&self.format_metric_details(tool_agg, 4));
                    output.push('\n');
                }
            }
        }

        output
    }

    /// Formats individual metric details for console output
    fn format_metric_details(&self, metric: &AggregatedMetrics, indent: usize) -> String {
        let prefix = " ".repeat(indent);
        let mut output = String::new();

        output.push_str(&format!(
            "{}Response Time: min={}ms, max={}ms, avg={:.1}ms, p50={}ms, p95={}ms, p99={}ms\n",
            prefix,
            metric.response_time.min,
            metric.response_time.max,
            metric.response_time.avg,
            metric.response_time.p50,
            metric.response_time.p95,
            metric.response_time.p99
        ));

        output.push_str(&format!(
            "{}Throughput: {:.2} req/sec\n",
            prefix, metric.throughput
        ));

        output.push_str(&format!(
            "{}Error Rate: {:.2}%\n",
            prefix, metric.error_rate
        ));

        output.push_str(&format!(
            "{}Cache Hit Rate: {:.2}%\n",
            prefix, metric.cache_hit_rate
        ));

        output.push_str(&format!(
            "{}Connection Reuse Rate: {:.2}%\n",
            prefix, metric.connection_reuse_rate
        ));

        output.push_str(&format!(
            "{}Requests: total={}, success={}, failed={}\n",
            prefix, metric.total_requests, metric.successful_requests, metric.failed_requests
        ));

        output
    }

    /// Formats metrics as JSON
    fn format_json_report(&self, metrics: &[AggregatedMetrics]) -> Result<String> {
        let json = serde_json::to_string_pretty(metrics)?;
        Ok(json)
    }

    /// Formats metrics as HTML
    fn format_html_report(&self, metrics: &[AggregatedMetrics], time_window: TimeWindow) -> String {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!(
            "  <title>Performance Report - {}</title>\n",
            html_escape(&self.format_time_window_label(time_window))
        ));
        html.push_str("  <style>\n");
        html.push_str(self.get_html_styles());
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Title
        html.push_str(&format!(
            "  <h1>Performance Report ({})</h1>\n",
            html_escape(&self.format_time_window_label(time_window))
        ));

        if metrics.is_empty() {
            html.push_str("  <p>No metrics available for the specified time window.</p>\n");
        } else {
            // Group metrics by server
            let mut server_groups: std::collections::HashMap<String, Vec<&AggregatedMetrics>> =
                std::collections::HashMap::new();
            for metric in metrics {
                server_groups
                    .entry(metric.server_name.clone())
                    .or_default()
                    .push(metric);
            }

            // Sort servers by name
            let mut server_names: Vec<_> = server_groups.keys().cloned().collect();
            server_names.sort();

            for server_name in server_names {
                let server_metrics = server_groups.get(&server_name).unwrap();

                html.push_str(&format!("  <h2>Server: {}</h2>\n", html_escape(&server_name)));

                // Find server-level metric
                if let Some(server_agg) = server_metrics.iter().find(|m| m.tool_name.is_none()) {
                    html.push_str("  <h3>Server Overview</h3>\n");
                    html.push_str(&self.format_html_metrics_table(server_agg));
                }

                // Tool-level metrics
                let mut tool_metrics: Vec<_> = server_metrics
                    .iter()
                    .filter(|m| m.tool_name.is_some())
                    .copied()
                    .collect();
                tool_metrics.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));

                if !tool_metrics.is_empty() {
                    html.push_str("  <h3>Tool Metrics</h3>\n");
                    for tool_agg in tool_metrics {
                        if let Some(tool_name) = &tool_agg.tool_name {
                            html.push_str(&format!("  <h4>{}</h4>\n", html_escape(tool_name)));
                            html.push_str(&self.format_html_metrics_table(tool_agg));
                        }
                    }
                }
            }
        }

        // HTML footer
        html.push_str(&format!(
            "  <footer>Generated at: {}</footer>\n",
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    /// Formats a single metric as an HTML table
    fn format_html_metrics_table(&self, metric: &AggregatedMetrics) -> String {
        let mut html = String::new();

        html.push_str("  <table class=\"metrics-table\">\n");
        html.push_str("    <tr><th>Metric</th><th>Value</th></tr>\n");

        // Response time stats
        html.push_str(&format!(
            "    <tr><td>Response Time (min)</td><td>{} ms</td></tr>\n",
            metric.response_time.min
        ));
        html.push_str(&format!(
            "    <tr><td>Response Time (max)</td><td>{} ms</td></tr>\n",
            metric.response_time.max
        ));
        html.push_str(&format!(
            "    <tr><td>Response Time (avg)</td><td>{:.1} ms</td></tr>\n",
            metric.response_time.avg
        ));
        html.push_str(&format!(
            "    <tr><td>Response Time (p50)</td><td>{} ms</td></tr>\n",
            metric.response_time.p50
        ));
        html.push_str(&format!(
            "    <tr><td>Response Time (p95)</td><td>{} ms</td></tr>\n",
            metric.response_time.p95
        ));
        html.push_str(&format!(
            "    <tr><td>Response Time (p99)</td><td>{} ms</td></tr>\n",
            metric.response_time.p99
        ));

        // Other metrics
        html.push_str(&format!(
            "    <tr><td>Throughput</td><td>{:.2} req/sec</td></tr>\n",
            metric.throughput
        ));
        html.push_str(&format!(
            "    <tr><td>Error Rate</td><td>{:.2}%</td></tr>\n",
            metric.error_rate
        ));
        html.push_str(&format!(
            "    <tr><td>Cache Hit Rate</td><td>{:.2}%</td></tr>\n",
            metric.cache_hit_rate
        ));
        html.push_str(&format!(
            "    <tr><td>Connection Reuse Rate</td><td>{:.2}%</td></tr>\n",
            metric.connection_reuse_rate
        ));
        html.push_str(&format!(
            "    <tr><td>Total Requests</td><td>{}</td></tr>\n",
            metric.total_requests
        ));
        html.push_str(&format!(
            "    <tr><td>Successful Requests</td><td>{}</td></tr>\n",
            metric.successful_requests
        ));
        html.push_str(&format!(
            "    <tr><td>Failed Requests</td><td>{}</td></tr>\n",
            metric.failed_requests
        ));

        html.push_str("  </table>\n");

        html
    }

    /// Returns CSS styles for HTML reports
    fn get_html_styles(&self) -> &'static str {
        r#"
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
      max-width: 1200px;
      margin: 0 auto;
      padding: 20px;
      background-color: #f5f5f5;
    }
    h1 {
      color: #333;
      border-bottom: 3px solid #0066cc;
      padding-bottom: 10px;
    }
    h2 {
      color: #444;
      border-bottom: 2px solid #0088ee;
      padding-bottom: 8px;
      margin-top: 30px;
    }
    h3 {
      color: #555;
      margin-top: 20px;
    }
    h4 {
      color: #666;
      margin-top: 15px;
      margin-bottom: 10px;
    }
    .metrics-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
      background-color: white;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
    .metrics-table th {
      background-color: #0066cc;
      color: white;
      padding: 12px;
      text-align: left;
      font-weight: 600;
    }
    .metrics-table td {
      padding: 10px 12px;
      border-bottom: 1px solid #ddd;
    }
    .metrics-table tr:hover {
      background-color: #f0f8ff;
    }
    .metrics-table tr:last-child td {
      border-bottom: none;
    }
    footer {
      margin-top: 40px;
      padding-top: 20px;
      border-top: 1px solid #ccc;
      color: #666;
      font-size: 0.9em;
      text-align: center;
    }
"#
    }

    /// Formats a time window as a human-readable label
    fn format_time_window_label(&self, window: TimeWindow) -> String {
        match window {
            TimeWindow::LastHour => "Last Hour".to_string(),
            TimeWindow::Last24Hours => "Last 24 Hours".to_string(),
            TimeWindow::Last7Days => "Last 7 Days".to_string(),
            TimeWindow::Custom(secs) => {
                format!("Last {} seconds", secs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MetricStatus;

    fn create_test_collector_with_data() -> Arc<MetricsCollector> {
        let collector = Arc::new(MetricsCollector::new());

        // Add test data
        collector.record_metric(
            "server1".to_string(),
            "tool1".to_string(),
            100,
            MetricStatus::Success,
            true,
            false,
        );
        collector.record_metric(
            "server1".to_string(),
            "tool1".to_string(),
            200,
            MetricStatus::Success,
            false,
            true,
        );
        collector.record_metric(
            "server1".to_string(),
            "tool2".to_string(),
            150,
            MetricStatus::Success,
            true,
            true,
        );
        collector.record_metric(
            "server2".to_string(),
            "tool3".to_string(),
            300,
            MetricStatus::Error,
            false,
            false,
        );

        collector
    }

    #[test]
    fn test_report_service_new() {
        let collector = Arc::new(MetricsCollector::new());
        let service = ReportService::new(collector);
        assert!(std::ptr::eq(
            &*service.metrics_collector,
            &*service.metrics_collector
        ));
    }

    #[test]
    fn test_generate_console_report_empty() {
        let collector = Arc::new(MetricsCollector::new());
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Console)
            .unwrap();

        assert!(report.contains("Performance Report"));
        assert!(report.contains("No metrics available"));
    }

    #[test]
    fn test_generate_console_report_with_data() {
        let collector = create_test_collector_with_data();
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Console)
            .unwrap();

        assert!(report.contains("Performance Report"));
        assert!(report.contains("Server: server1"));
        assert!(report.contains("Server: server2"));
        assert!(report.contains("Tool: tool1"));
        assert!(report.contains("Tool: tool2"));
        assert!(report.contains("Tool: tool3"));
        assert!(report.contains("Response Time:"));
        assert!(report.contains("Throughput:"));
        assert!(report.contains("Error Rate:"));
    }

    #[test]
    fn test_generate_json_report_empty() {
        let collector = Arc::new(MetricsCollector::new());
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Json)
            .unwrap();

        let parsed: Vec<AggregatedMetrics> = serde_json::from_str(&report).unwrap();
        assert_eq!(parsed.len(), 0);
    }

    #[test]
    fn test_generate_json_report_with_data() {
        let collector = create_test_collector_with_data();
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Json)
            .unwrap();

        let parsed: Vec<AggregatedMetrics> = serde_json::from_str(&report).unwrap();
        assert!(!parsed.is_empty());

        // Verify structure
        for metric in &parsed {
            assert!(!metric.server_name.is_empty());
        }
    }

    #[test]
    fn test_generate_html_report_empty() {
        let collector = Arc::new(MetricsCollector::new());
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Html)
            .unwrap();

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("<html"));
        assert!(report.contains("Performance Report"));
        assert!(report.contains("No metrics available"));
        assert!(report.contains("</html>"));
    }

    #[test]
    fn test_generate_html_report_with_data() {
        let collector = create_test_collector_with_data();
        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Html)
            .unwrap();

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("Performance Report"));
        assert!(report.contains("Server: server1"));
        assert!(report.contains("Server: server2"));
        assert!(report.contains("tool1"));
        assert!(report.contains("tool2"));
        assert!(report.contains("tool3"));
        assert!(report.contains("<table"));
        assert!(report.contains("Response Time"));
        assert!(report.contains("</html>"));
    }

    #[test]
    fn test_generate_server_report_console() {
        let collector = create_test_collector_with_data();
        let service = ReportService::new(collector);
        let report = service
            .generate_server_report("server1", TimeWindow::LastHour, ReportFormat::Console)
            .unwrap();

        assert!(report.contains("Server: server1"));
        assert!(report.contains("Tool: tool1"));
        assert!(report.contains("Tool: tool2"));
        assert!(!report.contains("server2"));
        assert!(!report.contains("tool3"));
    }

    #[test]
    fn test_generate_tool_report_console() {
        let collector = create_test_collector_with_data();
        let service = ReportService::new(collector);
        let report = service
            .generate_tool_report("server1", "tool1", TimeWindow::LastHour, ReportFormat::Console)
            .unwrap();

        assert!(report.contains("Performance Report"));
        assert!(report.contains("Response Time:"));
        assert!(report.contains("Throughput:"));
    }

    #[test]
    fn test_format_time_window_label() {
        let collector = Arc::new(MetricsCollector::new());
        let service = ReportService::new(collector);

        assert_eq!(
            service.format_time_window_label(TimeWindow::LastHour),
            "Last Hour"
        );
        assert_eq!(
            service.format_time_window_label(TimeWindow::Last24Hours),
            "Last 24 Hours"
        );
        assert_eq!(
            service.format_time_window_label(TimeWindow::Last7Days),
            "Last 7 Days"
        );
    }

    #[test]
    fn test_report_format_equality() {
        assert_eq!(ReportFormat::Console, ReportFormat::Console);
        assert_eq!(ReportFormat::Json, ReportFormat::Json);
        assert_eq!(ReportFormat::Html, ReportFormat::Html);
        assert_ne!(ReportFormat::Console, ReportFormat::Json);
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("normal"), "normal");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"quote\""), "&quot;quote&quot;");
        assert_eq!(html_escape("'single'"), "&#x27;single&#x27;");
        assert_eq!(
            html_escape("<script>alert('XSS')</script>"),
            "&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_html_report_xss_protection() {
        let collector = Arc::new(MetricsCollector::new());
        collector.record_metric(
            "<script>alert('xss')</script>".to_string(),
            "<img src=x>".to_string(),
            100,
            MetricStatus::Success,
            false,
            false,
        );

        let service = ReportService::new(collector);
        let report = service
            .generate_report(TimeWindow::LastHour, ReportFormat::Html)
            .unwrap();

        // Ensure that dangerous characters are escaped
        assert!(!report.contains("<script>"));
        assert!(!report.contains("<img"));
        assert!(report.contains("&lt;script&gt;"));
        assert!(report.contains("&lt;img"));
    }
}

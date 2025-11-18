use chrono::{DateTime, Local};
use colored::*;
use serde_json::Value;

/// デバッグログの最大ペイロードサイズ（デフォルト: 4KB）
const DEFAULT_MAX_PAYLOAD_SIZE: usize = 4096;

/// デバッグロガーの設定
#[derive(Debug, Clone)]
pub struct DebugLoggerConfig {
    /// カラー出力を有効化するか
    pub color_output: bool,
    /// 最大ペイロードサイズ（バイト）
    pub max_payload_size: usize,
    /// 大きなペイロードを自動的にトランケートするか
    pub truncate_large_payloads: bool,
}

impl Default for DebugLoggerConfig {
    fn default() -> Self {
        Self {
            color_output: true,
            max_payload_size: DEFAULT_MAX_PAYLOAD_SIZE,
            truncate_large_payloads: true,
        }
    }
}

/// デバッグログを整形して出力するロガー
pub struct DebugLogger {
    config: DebugLoggerConfig,
}

impl DebugLogger {
    /// 新しいデバッグロガーを作成
    ///
    /// # Arguments
    /// * `config` - ロガーの設定
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::debug_logger::{DebugLogger, DebugLoggerConfig};
    ///
    /// let config = DebugLoggerConfig::default();
    /// let logger = DebugLogger::new(config);
    /// ```
    pub fn new(config: DebugLoggerConfig) -> Self {
        Self { config }
    }

    /// デフォルト設定でデバッグロガーを作成
    pub fn with_defaults() -> Self {
        Self::new(DebugLoggerConfig::default())
    }

    /// JSONRPCリクエストをログ出力
    ///
    /// # Arguments
    /// * `server_name` - サーバー名
    /// * `method` - メソッド名
    /// * `request_id` - リクエストID
    /// * `request_body` - リクエストボディ（JSON）
    /// * `timestamp` - タイムスタンプ
    pub fn log_request(
        &self,
        server_name: &str,
        method: &str,
        request_id: &str,
        request_body: &Value,
        timestamp: DateTime<Local>,
    ) {
        let separator = self.make_separator();
        let timestamp_str = self.format_timestamp(timestamp);

        let header = if self.config.color_output {
            format!(
                "{}\n{} {}  [{}]\n{}",
                separator,
                "📤 REQUEST".bright_cyan().bold(),
                "",
                timestamp_str.bright_black(),
                separator
            )
        } else {
            format!(
                "{}\n📤 REQUEST  [{}]\n{}",
                separator, timestamp_str, separator
            )
        };

        eprintln!("{}", header);

        // サーバー名とメソッド情報
        if self.config.color_output {
            eprintln!("{}: {}", "Server".bright_yellow(), server_name);
            eprintln!("{}: {}", "Method".bright_yellow(), method);
            eprintln!("{}: {}", "Request ID".bright_yellow(), request_id);
        } else {
            eprintln!("Server: {}", server_name);
            eprintln!("Method: {}", method);
            eprintln!("Request ID: {}", request_id);
        }

        eprintln!();

        // リクエストボディ
        let formatted = self.format_json(request_body);
        eprintln!("{}", formatted);

        eprintln!("{}\n", separator);
    }

    /// JSONRPCレスポンスをログ出力
    ///
    /// # Arguments
    /// * `server_name` - サーバー名
    /// * `request_id` - リクエストID
    /// * `response_body` - レスポンスボディ（JSON）
    /// * `timestamp` - タイムスタンプ
    /// * `elapsed_ms` - 経過時間（ミリ秒）
    /// * `is_success` - 成功かどうか
    pub fn log_response(
        &self,
        server_name: &str,
        request_id: &str,
        response_body: &Value,
        timestamp: DateTime<Local>,
        elapsed_ms: u128,
        is_success: bool,
    ) {
        let separator = self.make_separator();
        let timestamp_str = self.format_timestamp(timestamp);

        let header = if self.config.color_output {
            format!(
                "{}\n{} {}  [{}] ({}ms)\n{}",
                separator,
                "📥 RESPONSE".bright_magenta().bold(),
                "",
                timestamp_str.bright_black(),
                elapsed_ms.to_string().bright_green(),
                separator
            )
        } else {
            format!(
                "{}\n📥 RESPONSE  [{}] ({}ms)\n{}",
                separator, timestamp_str, elapsed_ms, separator
            )
        };

        eprintln!("{}", header);

        // サーバー名とステータス
        if self.config.color_output {
            eprintln!("{}: {}", "Server".bright_yellow(), server_name);
            eprintln!("{}: {}", "Request ID".bright_yellow(), request_id);

            let status_label = if is_success {
                "✅ Success".green()
            } else {
                "❌ Error".red()
            };
            eprintln!("{}: {}", "Status".bright_yellow(), status_label);
        } else {
            eprintln!("Server: {}", server_name);
            eprintln!("Request ID: {}", request_id);
            let status_label = if is_success { "Success" } else { "Error" };
            eprintln!("Status: {}", status_label);
        }

        eprintln!();

        // レスポンスボディ
        let formatted = self.format_json(response_body);
        eprintln!("{}", formatted);

        eprintln!("{}\n", separator);
    }

    /// JSONを整形
    ///
    /// トランケート設定に応じて、大きなJSONを切り詰めます。
    fn format_json(&self, value: &Value) -> String {
        let formatted = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());

        if self.config.truncate_large_payloads && formatted.len() > self.config.max_payload_size {
            self.truncate(&formatted, self.config.max_payload_size)
        } else {
            formatted
        }
    }

    /// 文字列をトランケート
    ///
    /// 指定されたサイズを超える場合、切り詰めて "... (truncated)" を追加します。
    fn truncate(&self, text: &str, max_size: usize) -> String {
        if text.len() <= max_size {
            return text.to_string();
        }

        let truncated_marker = "\n... (truncated)";
        let available_size = max_size.saturating_sub(truncated_marker.len());

        // UTF-8境界を考慮して切り詰め
        let mut end_index = available_size.min(text.len());
        while !text.is_char_boundary(end_index) && end_index > 0 {
            end_index -= 1;
        }

        format!("{}{}", &text[..end_index], truncated_marker)
    }

    /// タイムスタンプを整形
    fn format_timestamp(&self, timestamp: DateTime<Local>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    /// 区切り線を作成
    fn make_separator(&self) -> String {
        "━".repeat(70)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_debug_logger_creation() {
        let logger = DebugLogger::with_defaults();
        assert!(logger.config.color_output);
        assert_eq!(logger.config.max_payload_size, DEFAULT_MAX_PAYLOAD_SIZE);
        assert!(logger.config.truncate_large_payloads);
    }

    #[test]
    fn test_format_json() {
        let logger = DebugLogger::with_defaults();
        let value = json!({"key": "value", "nested": {"a": 1, "b": 2}});

        let formatted = logger.format_json(&value);
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));
        assert!(formatted.contains("nested"));
    }

    #[test]
    fn test_truncate_small_text() {
        let logger = DebugLogger::with_defaults();
        let text = "Hello, World!";
        let truncated = logger.truncate(text, 100);

        assert_eq!(truncated, text);
    }

    #[test]
    fn test_truncate_large_text() {
        let logger = DebugLogger::with_defaults();
        let text = "a".repeat(1000);
        let truncated = logger.truncate(&text, 50);

        assert!(truncated.len() <= 50);
        assert!(truncated.contains("(truncated)"));
    }

    #[test]
    fn test_truncate_utf8_boundary() {
        let logger = DebugLogger::with_defaults();
        // 日本語文字列（UTF-8で複数バイト）
        let text = "こんにちは世界".repeat(20);
        let truncated = logger.truncate(&text, 50);

        // トランケート後も有効なUTF-8であることを確認
        assert!(truncated.len() <= 50);
        assert!(truncated.contains("(truncated)"));
    }

    #[test]
    fn test_format_timestamp() {
        let logger = DebugLogger::with_defaults();
        let timestamp = Local::now();
        let formatted = logger.format_timestamp(timestamp);

        // フォーマットが正しいかチェック（YYYY-MM-DD HH:MM:SS.mmm）
        assert!(formatted.contains('-'));
        assert!(formatted.contains(':'));
        assert!(formatted.contains('.'));
    }

    #[test]
    fn test_make_separator() {
        let logger = DebugLogger::with_defaults();
        let separator = logger.make_separator();

        // UTF-8で "━" は3バイトなので、70文字 × 3バイト = 210バイト
        assert_eq!(separator.chars().count(), 70);
        assert!(separator.chars().all(|c| c == '━'));
    }

    #[test]
    fn test_log_request_no_panic() {
        let logger = DebugLogger::with_defaults();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": "test"}
        });

        // パニックしないことを確認（出力はstderrに行く）
        logger.log_request(
            "test_server",
            "tools/call",
            "req-123",
            &request,
            Local::now(),
        );
    }

    #[test]
    fn test_log_response_success_no_panic() {
        let logger = DebugLogger::with_defaults();
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"status": "ok"}
        });

        logger.log_response(
            "test_server",
            "req-123",
            &response,
            Local::now(),
            150,
            true,
        );
    }

    #[test]
    fn test_log_response_error_no_panic() {
        let logger = DebugLogger::with_defaults();
        let response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"code": -32600, "message": "Invalid Request"}
        });

        logger.log_response(
            "test_server",
            "req-123",
            &response,
            Local::now(),
            50,
            false,
        );
    }

    #[test]
    fn test_custom_config() {
        let config = DebugLoggerConfig {
            color_output: false,
            max_payload_size: 1024,
            truncate_large_payloads: false,
        };
        let logger = DebugLogger::new(config);

        assert!(!logger.config.color_output);
        assert_eq!(logger.config.max_payload_size, 1024);
        assert!(!logger.config.truncate_large_payloads);
    }

    #[test]
    fn test_format_json_large_payload_with_truncation() {
        let config = DebugLoggerConfig {
            color_output: false,
            max_payload_size: 100,
            truncate_large_payloads: true,
        };
        let logger = DebugLogger::new(config);

        let large_value = json!({
            "data": "x".repeat(500)
        });

        let formatted = logger.format_json(&large_value);
        assert!(formatted.len() <= 100);
        assert!(formatted.contains("(truncated)"));
    }

    #[test]
    fn test_format_json_large_payload_without_truncation() {
        let config = DebugLoggerConfig {
            color_output: false,
            max_payload_size: 100,
            truncate_large_payloads: false,
        };
        let logger = DebugLogger::new(config);

        let large_value = json!({
            "data": "x".repeat(500)
        });

        let formatted = logger.format_json(&large_value);
        assert!(formatted.len() > 100);
        assert!(!formatted.contains("(truncated)"));
    }

    #[test]
    fn test_colorized_output_enabled() {
        let logger = DebugLogger::with_defaults();
        assert!(logger.config.color_output);
        // カラー出力が有効な場合、内部的にcoloredクレートが使用される
        // 実際の出力確認は視覚的なテストが必要
    }

    #[test]
    fn test_colorized_output_disabled() {
        let config = DebugLoggerConfig {
            color_output: false,
            max_payload_size: DEFAULT_MAX_PAYLOAD_SIZE,
            truncate_large_payloads: true,
        };
        let logger = DebugLogger::new(config);
        assert!(!logger.config.color_output);
    }
}

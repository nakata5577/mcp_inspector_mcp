use serde::{Deserialize, Serialize};

/// MCPツール実行の設定
///
/// タイムアウト、リトライなどの実行時パラメータを管理します。
/// 設定の優先順位: config.json > 環境変数 > デフォルト値
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// ツール実行のタイムアウト時間（ミリ秒）
    #[serde(default = "default_tool_timeout")]
    pub tool_timeout_ms: u64,

    /// サーバー接続のタイムアウト時間（ミリ秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,

    /// 失敗時のリトライ回数
    #[serde(default)]
    pub retry_count: u32,

    /// タイムアウト発生時に自動的にリトライするか
    #[serde(default)]
    pub auto_retry_on_timeout: bool,

    /// デバッグモードを有効化するか（詳細なログ出力）
    #[serde(default)]
    pub verbose: bool,
}

fn default_tool_timeout() -> u64 {
    30000
}

fn default_connection_timeout() -> u64 {
    5000
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            tool_timeout_ms: default_tool_timeout(),
            connection_timeout_ms: default_connection_timeout(),
            retry_count: 0,
            auto_retry_on_timeout: false,
            verbose: false,
        }
    }
}

impl ExecutionConfig {
    /// 環境変数から設定を読み込む
    ///
    /// 後方互換性のため、以下の環境変数をサポートします：
    /// - `MCP_TOOL_TIMEOUT_MS`: ツールタイムアウト
    /// - `MCP_CONNECTION_TIMEOUT_MS`: 接続タイムアウト
    /// - `MCP_RETRY_COUNT`: リトライ回数
    /// - `MCP_AUTO_RETRY`: 自動リトライフラグ
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::models::ExecutionConfig;
    ///
    /// let config = ExecutionConfig::from_env();
    /// assert!(config.tool_timeout_ms > 0);
    /// ```
    pub fn from_env() -> Self {
        Self {
            tool_timeout_ms: std::env::var("MCP_TOOL_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_tool_timeout),
            connection_timeout_ms: std::env::var("MCP_CONNECTION_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_connection_timeout),
            retry_count: std::env::var("MCP_RETRY_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            auto_retry_on_timeout: std::env::var("MCP_AUTO_RETRY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            verbose: std::env::var("MCP_VERBOSE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
        }
    }

    /// config.jsonの設定と環境変数をマージ
    ///
    /// config.jsonに値が設定されていればそれを使用し、
    /// デフォルト値のままであれば環境変数を優先します。
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::models::ExecutionConfig;
    ///
    /// let config = ExecutionConfig::default().merge_with_env();
    /// ```
    pub fn merge_with_env(self) -> Self {
        let env_config = Self::from_env();

        // config.jsonの値がデフォルト値でなければそれを使用、
        // デフォルト値の場合は環境変数の値を使用
        Self {
            tool_timeout_ms: if self.tool_timeout_ms != default_tool_timeout() {
                self.tool_timeout_ms
            } else {
                env_config.tool_timeout_ms
            },
            connection_timeout_ms: if self.connection_timeout_ms != default_connection_timeout() {
                self.connection_timeout_ms
            } else {
                env_config.connection_timeout_ms
            },
            retry_count: if self.retry_count != 0 {
                self.retry_count
            } else {
                env_config.retry_count
            },
            auto_retry_on_timeout: self.auto_retry_on_timeout || env_config.auto_retry_on_timeout,
            verbose: self.verbose || env_config.verbose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = ExecutionConfig::default();
        assert_eq!(config.tool_timeout_ms, 30000);
        assert_eq!(config.connection_timeout_ms, 5000);
        assert_eq!(config.retry_count, 0);
        assert!(!config.auto_retry_on_timeout);
        assert!(!config.verbose);
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = ExecutionConfig {
            tool_timeout_ms: 60000,
            connection_timeout_ms: 10000,
            retry_count: 3,
            auto_retry_on_timeout: true,
            verbose: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool_timeout_ms, 60000);
        assert_eq!(deserialized.connection_timeout_ms, 10000);
        assert_eq!(deserialized.retry_count, 3);
        assert!(deserialized.auto_retry_on_timeout);
        assert!(deserialized.verbose);
    }

    #[test]
    fn test_deserialize_with_missing_fields() {
        // 一部のフィールドが欠けている場合、デフォルト値が使用される
        let json = r#"{"tool_timeout_ms": 45000}"#;
        let config: ExecutionConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.tool_timeout_ms, 45000);
        assert_eq!(config.connection_timeout_ms, 5000); // default
        assert_eq!(config.retry_count, 0); // default
        assert!(!config.auto_retry_on_timeout); // default
        assert!(!config.verbose); // default
    }

    #[test]
    fn test_from_env_with_no_env_vars() {
        // 環境変数が設定されていない場合、デフォルト値が使用される
        std::env::remove_var("MCP_TOOL_TIMEOUT_MS");
        std::env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
        std::env::remove_var("MCP_RETRY_COUNT");
        std::env::remove_var("MCP_AUTO_RETRY");
        std::env::remove_var("MCP_VERBOSE");

        let config = ExecutionConfig::from_env();
        assert_eq!(config.tool_timeout_ms, 30000);
        assert_eq!(config.connection_timeout_ms, 5000);
        assert_eq!(config.retry_count, 0);
        assert!(!config.auto_retry_on_timeout);
        assert!(!config.verbose);
    }

    #[test]
    fn test_merge_with_env() {
        // 環境変数をセット
        std::env::set_var("MCP_TOOL_TIMEOUT_MS", "45000");
        std::env::set_var("MCP_CONNECTION_TIMEOUT_MS", "8000");

        // config.jsonからロードした値（一部デフォルト）
        let config = ExecutionConfig {
            tool_timeout_ms: 60000, // 明示的に設定された値
            connection_timeout_ms: 5000, // デフォルト値
            retry_count: 0,
            auto_retry_on_timeout: false,
            verbose: false,
        };

        let merged = config.merge_with_env();

        // config.jsonの明示的な値が優先される
        assert_eq!(merged.tool_timeout_ms, 60000);
        // デフォルト値の場合は環境変数が優先される
        assert_eq!(merged.connection_timeout_ms, 8000);

        // クリーンアップ
        std::env::remove_var("MCP_TOOL_TIMEOUT_MS");
        std::env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    }

    #[test]
    #[serial_test::serial]
    fn test_verbose_from_env() {
        // Clear any existing value first
        std::env::remove_var("MCP_VERBOSE");

        std::env::set_var("MCP_VERBOSE", "true");
        let config = ExecutionConfig::from_env();
        assert!(config.verbose, "MCP_VERBOSE=true should set verbose to true");

        std::env::set_var("MCP_VERBOSE", "false");
        let config = ExecutionConfig::from_env();
        assert!(!config.verbose, "MCP_VERBOSE=false should set verbose to false");

        std::env::remove_var("MCP_VERBOSE");
    }

    #[test]
    fn test_verbose_from_config() {
        let json = r#"{"verbose": true}"#;
        let config: ExecutionConfig = serde_json::from_str(json).unwrap();
        assert!(config.verbose);

        let json = r#"{"verbose": false}"#;
        let config: ExecutionConfig = serde_json::from_str(json).unwrap();
        assert!(!config.verbose);
    }

    #[test]
    fn test_verbose_merge_priority() {
        // config.jsonでverbose=trueの場合、環境変数に関係なく優先される
        std::env::set_var("MCP_VERBOSE", "false");
        let config = ExecutionConfig {
            verbose: true,
            ..Default::default()
        };
        let merged = config.merge_with_env();
        assert!(merged.verbose);

        // config.jsonでverbose=false、環境変数でverbose=trueの場合、環境変数が優先される
        std::env::set_var("MCP_VERBOSE", "true");
        let config = ExecutionConfig {
            verbose: false,
            ..Default::default()
        };
        let merged = config.merge_with_env();
        assert!(merged.verbose);

        std::env::remove_var("MCP_VERBOSE");
    }
}

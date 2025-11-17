use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::execution_config::ExecutionConfig;

/// `.inspector/config.json`ファイルのルート構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 登録されているMCPサーバーのリスト
    pub servers: Vec<ServerEntry>,
    /// ロギング設定
    pub logging: LoggingSettings,
    /// 実行設定（タイムアウト、リトライなど）
    #[serde(default)]
    pub execution_config: ExecutionConfig,
}

/// MCPサーバーエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    /// サーバー識別名（一意である必要がある）
    pub name: String,
    /// トランスポートタイプ（現在は "stdio" のみサポート）
    pub transport: String,
    /// 実行可能ファイルのパス
    pub command: String,
    /// コマンドライン引数（オプション）
    #[serde(default)]
    pub args: Vec<String>,
    /// 環境変数（オプション）
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// ロギング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// ロギングバックエンドタイプ（"memory" または "persistent"）
    pub backend: String,
    /// データベースファイルパス（persistent使用時）
    pub db_path: String,
    /// 最大ログ保持数
    pub max_logs: usize,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            servers: vec![],
            logging: LoggingSettings {
                backend: "memory".to_string(),
                db_path: "./data/logs.db".to_string(),
                max_logs: 10000,
            },
            execution_config: ExecutionConfig::default(),
        }
    }
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            db_path: "./data/logs.db".to_string(),
            max_logs: 10000,
        }
    }
}

impl ServerEntry {
    /// 新しいサーバーエントリを作成
    pub fn new(
        name: String,
        transport: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            transport,
            command,
            args,
            env,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProjectConfig::default();
        assert_eq!(config.servers.len(), 0);
        assert_eq!(config.logging.backend, "memory");
        assert_eq!(config.logging.max_logs, 10000);
    }

    #[test]
    fn test_server_entry_creation() {
        let entry = ServerEntry::new(
            "test_server".to_string(),
            "stdio".to_string(),
            "/path/to/server".to_string(),
            vec![],
            HashMap::new(),
        );
        assert_eq!(entry.name, "test_server");
        assert_eq!(entry.transport, "stdio");
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = ProjectConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProjectConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.servers.len(), config.servers.len());
    }
}

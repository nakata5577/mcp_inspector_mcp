use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::execution_config::ExecutionConfig;
use super::project_config::{LoggingSettings, ServerEntry};

/// プロファイル名を表す型エイリアス
pub type ProfileName = String;

/// プロファイル設定（config.{profile}.jsonファイルの構造）
/// ProjectConfigと同じ構造を持つが、プロファイル管理の文脈で使用される
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// 登録されているMCPサーバーのリスト
    pub servers: Vec<ServerEntry>,
    /// ロギング設定
    pub logging: LoggingSettings,
    /// 実行設定（タイムアウト、リトライなど）
    #[serde(default)]
    pub execution_config: ExecutionConfig,
    /// プロファイル固有のメタデータ（オプション）
    #[serde(default)]
    pub metadata: ProfileMetadata,
}

/// プロファイルのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileMetadata {
    /// プロファイルの説明
    #[serde(default)]
    pub description: Option<String>,
    /// プロファイルのタグ（例: "production", "testing"）
    #[serde(default)]
    pub tags: Vec<String>,
    /// 作成日時（ISO 8601形式）
    #[serde(default)]
    pub created_at: Option<String>,
    /// 最終更新日時（ISO 8601形式）
    #[serde(default)]
    pub updated_at: Option<String>,
    /// その他のカスタムフィールド
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// プロファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// プロファイル名
    pub name: ProfileName,
    /// プロファイル設定ファイルのパス
    pub config_path: String,
    /// プロファイルが存在するか
    pub exists: bool,
    /// プロファイルの説明（メタデータから取得）
    pub description: Option<String>,
    /// プロファイルのタグ
    pub tags: Vec<String>,
}

impl ProfileConfig {
    /// デフォルトのProfileConfigを作成
    pub fn default_profile() -> Self {
        Self {
            servers: vec![],
            logging: LoggingSettings::default(),
            execution_config: ExecutionConfig::default(),
            metadata: ProfileMetadata::default(),
        }
    }

    /// 開発環境向けのProfileConfigを作成
    pub fn dev_profile() -> Self {
        let mut config = Self::default_profile();
        config.execution_config.verbose = true;
        config.execution_config.tool_timeout_ms = 10000;
        config.execution_config.connection_timeout_ms = 10000;
        config.metadata.description = Some("Development environment configuration".to_string());
        config.metadata.tags = vec!["dev".to_string(), "development".to_string()];
        config
    }

    /// ステージング環境向けのProfileConfigを作成
    pub fn staging_profile() -> Self {
        let mut config = Self::default_profile();
        config.execution_config.verbose = false;
        config.execution_config.tool_timeout_ms = 8000;
        config.execution_config.connection_timeout_ms = 8000;
        config.execution_config.retry_count = 2;
        config.metadata.description = Some("Staging environment configuration".to_string());
        config.metadata.tags = vec!["staging".to_string(), "pre-production".to_string()];
        config
    }

    /// 本番環境向けのProfileConfigを作成
    pub fn prod_profile() -> Self {
        let mut config = Self::default_profile();
        config.execution_config.verbose = false;
        config.execution_config.tool_timeout_ms = 5000;
        config.execution_config.connection_timeout_ms = 5000;
        config.execution_config.retry_count = 3;
        config.execution_config.auto_retry_on_timeout = true;
        config.logging.backend = "persistent".to_string();
        config.metadata.description = Some("Production environment configuration".to_string());
        config.metadata.tags = vec!["prod".to_string(), "production".to_string()];
        config
    }

    /// CI/CD環境向けのProfileConfigを作成
    pub fn ci_profile() -> Self {
        let mut config = Self::default_profile();
        config.execution_config.verbose = true;
        config.execution_config.tool_timeout_ms = 15000;
        config.execution_config.connection_timeout_ms = 15000;
        config.execution_config.retry_count = 1;
        config.execution_config.auto_retry_on_timeout = false;
        config.metadata.description = Some("CI/CD environment configuration".to_string());
        config.metadata.tags = vec!["ci".to_string(), "continuous-integration".to_string()];
        config
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> Result<(), String> {
        // サーバー名の重複チェック
        let mut seen_names = std::collections::HashSet::new();
        for server in &self.servers {
            if !seen_names.insert(&server.name) {
                return Err(format!("Duplicate server name: {}", server.name));
            }
        }

        // サーバートランスポートタイプのバリデーション
        for server in &self.servers {
            if server.transport != "stdio" {
                return Err(format!(
                    "Unsupported transport type '{}' for server '{}'",
                    server.transport, server.name
                ));
            }
            if server.command.is_empty() {
                return Err(format!("Empty command for server '{}'", server.name));
            }
        }

        // ロギング設定のバリデーション
        if self.logging.backend != "memory" && self.logging.backend != "persistent" {
            return Err(format!("Invalid logging backend: {}", self.logging.backend));
        }

        if self.logging.max_logs == 0 {
            return Err("max_logs must be greater than 0".to_string());
        }

        // 実行設定のバリデーション
        if self.execution_config.tool_timeout_ms == 0 {
            return Err("tool_timeout_ms must be greater than 0".to_string());
        }

        if self.execution_config.connection_timeout_ms == 0 {
            return Err("connection_timeout_ms must be greater than 0".to_string());
        }

        Ok(())
    }

    /// JSONからProfileConfigを読み込む
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// ProfileConfigをJSON文字列に変換
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// YAMLからProfileConfigを読み込む
    pub fn from_yaml_str(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// ProfileConfigをYAML文字列に変換
    pub fn to_yaml_string(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

impl ProfileInfo {
    /// 新しいProfileInfoを作成
    pub fn new(
        name: ProfileName,
        config_path: String,
        exists: bool,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            name,
            config_path,
            exists,
            description,
            tags,
        }
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self::default_profile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = ProfileConfig::default_profile();
        assert_eq!(profile.servers.len(), 0);
        assert_eq!(profile.logging.backend, "memory");
        assert!(!profile.execution_config.verbose);
    }

    #[test]
    fn test_dev_profile() {
        let profile = ProfileConfig::dev_profile();
        assert!(profile.execution_config.verbose);
        assert_eq!(profile.execution_config.tool_timeout_ms, 10000);
        assert!(profile.metadata.description.is_some());
        assert!(profile.metadata.tags.contains(&"dev".to_string()));
    }

    #[test]
    fn test_staging_profile() {
        let profile = ProfileConfig::staging_profile();
        assert!(!profile.execution_config.verbose);
        assert_eq!(profile.execution_config.retry_count, 2);
        assert!(profile.metadata.tags.contains(&"staging".to_string()));
    }

    #[test]
    fn test_prod_profile() {
        let profile = ProfileConfig::prod_profile();
        assert!(!profile.execution_config.verbose);
        assert_eq!(profile.execution_config.retry_count, 3);
        assert!(profile.execution_config.auto_retry_on_timeout);
        assert_eq!(profile.logging.backend, "persistent");
        assert!(profile.metadata.tags.contains(&"prod".to_string()));
    }

    #[test]
    fn test_ci_profile() {
        let profile = ProfileConfig::ci_profile();
        assert!(profile.execution_config.verbose);
        assert_eq!(profile.execution_config.retry_count, 1);
        assert!(profile.metadata.tags.contains(&"ci".to_string()));
    }

    #[test]
    fn test_validate_success() {
        let profile = ProfileConfig::default_profile();
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_server_name() {
        let mut profile = ProfileConfig::default_profile();
        profile.servers = vec![
            ServerEntry::new(
                "server1".to_string(),
                "stdio".to_string(),
                "cmd1".to_string(),
                vec![],
                HashMap::new(),
            ),
            ServerEntry::new(
                "server1".to_string(),
                "stdio".to_string(),
                "cmd2".to_string(),
                vec![],
                HashMap::new(),
            ),
        ];
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_transport() {
        let mut profile = ProfileConfig::default_profile();
        profile.servers = vec![ServerEntry::new(
            "server1".to_string(),
            "http".to_string(), // unsupported
            "cmd1".to_string(),
            vec![],
            HashMap::new(),
        )];
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_validate_empty_command() {
        let mut profile = ProfileConfig::default_profile();
        profile.servers = vec![ServerEntry::new(
            "server1".to_string(),
            "stdio".to_string(),
            "".to_string(), // empty
            vec![],
            HashMap::new(),
        )];
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_logging_backend() {
        let mut profile = ProfileConfig::default_profile();
        profile.logging.backend = "invalid".to_string();
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_validate_zero_max_logs() {
        let mut profile = ProfileConfig::default_profile();
        profile.logging.max_logs = 0;
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut profile = ProfileConfig::default_profile();
        profile.execution_config.tool_timeout_ms = 0;
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_json_roundtrip() {
        let profile = ProfileConfig::dev_profile();
        let json = profile.to_json_string().unwrap();
        let deserialized = ProfileConfig::from_json_str(&json).unwrap();
        assert_eq!(deserialized.execution_config.verbose, profile.execution_config.verbose);
        assert_eq!(deserialized.execution_config.tool_timeout_ms, profile.execution_config.tool_timeout_ms);
    }

    #[test]
    fn test_profile_info_creation() {
        let info = ProfileInfo::new(
            "dev".to_string(),
            ".inspector/config.dev.json".to_string(),
            true,
            Some("Development profile".to_string()),
            vec!["dev".to_string()],
        );
        assert_eq!(info.name, "dev");
        assert!(info.exists);
        assert!(info.description.is_some());
    }
}

use crate::models::{InspectorError, ProfileConfig, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// 設定のエクスポート/インポート形式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// JSON形式
    Json,
    /// YAML形式
    Yaml,
}

impl FromStr for ConfigFormat {
    type Err = InspectorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            _ => Err(InspectorError::Config(format!(
                "Unsupported format: {}. Supported formats: json, yaml",
                s
            ))),
        }
    }
}

impl ConfigFormat {
    /// ファイル拡張子から形式を推測
    pub fn from_path(path: &Path) -> Result<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Ok(Self::Json),
            Some("yaml") | Some("yml") => Ok(Self::Yaml),
            Some(ext) => Err(InspectorError::Config(format!(
                "Unsupported file extension: .{}",
                ext
            ))),
            None => Err(InspectorError::Config(
                "No file extension found".to_string()
            )),
        }
    }
}

/// 設定のインポート/エクスポートサービス
pub struct ConfigImportExport;

impl ConfigImportExport {
    /// 設定をエクスポート
    ///
    /// # Arguments
    /// * `config` - エクスポートするProfileConfig
    /// * `output_path` - 出力ファイルのパス
    /// * `format` - エクスポート形式（Noneの場合はパスから推測）
    ///
    /// # Returns
    /// エクスポートしたバイト数
    pub fn export_config(
        config: &ProfileConfig,
        output_path: &Path,
        format: Option<ConfigFormat>,
    ) -> Result<usize> {
        // バリデーション
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Config validation failed: {}", e)
            ))?;

        // フォーマットを決定
        let format = match format {
            Some(f) => f,
            None => ConfigFormat::from_path(output_path)?,
        };

        // シリアライズ
        let content = match format {
            ConfigFormat::Json => serde_json::to_string_pretty(config)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to serialize config to JSON: {}", e)
                ))?,
            ConfigFormat::Yaml => serde_yaml::to_string(config)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to serialize config to YAML: {}", e)
                ))?,
        };

        // ファイルに書き込み
        // Create parent directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to create directory: {}", e)
                ))?;
        }

        fs::write(output_path, &content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to write config to {}: {}", output_path.display(), e)
            ))?;

        tracing::info!(
            "Config exported to {} ({} bytes, format: {:?})",
            output_path.display(),
            content.len(),
            format
        );

        Ok(content.len())
    }

    /// 設定をインポート
    ///
    /// # Arguments
    /// * `input_path` - インポート元ファイルのパス
    /// * `format` - インポート形式（Noneの場合はパスから推測）
    ///
    /// # Returns
    /// インポートしたProfileConfig
    pub fn import_config(
        input_path: &Path,
        format: Option<ConfigFormat>,
    ) -> Result<ProfileConfig> {
        // ファイルの存在チェック
        if !input_path.exists() {
            return Err(InspectorError::Config(format!(
                "Input file does not exist: {}",
                input_path.display()
            )));
        }

        // フォーマットを決定
        let format = match format {
            Some(f) => f,
            None => ConfigFormat::from_path(input_path)?,
        };

        // ファイルを読み込み
        let content = fs::read_to_string(input_path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read config from {}: {}", input_path.display(), e)
            ))?;

        // デシリアライズ
        let config: ProfileConfig = match format {
            ConfigFormat::Json => serde_json::from_str(&content)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to parse config from JSON: {}", e)
                ))?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to parse config from YAML: {}", e)
                ))?,
        };

        // バリデーション
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Imported config validation failed: {}", e)
            ))?;

        tracing::info!(
            "Config imported from {} (format: {:?})",
            input_path.display(),
            format
        );

        Ok(config)
    }

    /// 設定をバリデーション（ファイルから読み込んで検証）
    ///
    /// # Arguments
    /// * `input_path` - 検証対象ファイルのパス
    /// * `format` - ファイル形式（Noneの場合はパスから推測）
    ///
    /// # Returns
    /// バリデーション結果とエラーメッセージ
    pub fn validate_config_file(
        input_path: &Path,
        format: Option<ConfigFormat>,
    ) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // インポートして自動的にバリデーション
        match Self::import_config(input_path, format) {
            Ok(config) => {
                // 追加の警告チェック
                if config.servers.is_empty() {
                    warnings.push("No servers configured".to_string());
                }

                if config.execution_config.tool_timeout_ms < 1000 {
                    warnings.push(format!(
                        "Tool timeout is very low: {}ms",
                        config.execution_config.tool_timeout_ms
                    ));
                }

                if config.execution_config.connection_timeout_ms < 1000 {
                    warnings.push(format!(
                        "Connection timeout is very low: {}ms",
                        config.execution_config.connection_timeout_ms
                    ));
                }

                if config.logging.max_logs < 100 {
                    warnings.push(format!(
                        "Max logs is very low: {}",
                        config.logging.max_logs
                    ));
                }

                Ok(warnings)
            }
            Err(e) => Err(e),
        }
    }

    /// 2つの設定の差分を計算
    ///
    /// # Arguments
    /// * `config1` - 比較元の設定
    /// * `config2` - 比較先の設定
    ///
    /// # Returns
    /// 差分のサマリー情報
    pub fn diff_configs(config1: &ProfileConfig, config2: &ProfileConfig) -> ConfigDiff {
        let mut diff = ConfigDiff {
            servers_added: config2.servers.len().saturating_sub(config1.servers.len()),
            servers_removed: config1.servers.len().saturating_sub(config2.servers.len()),
            ..Default::default()
        };

        // サーバー名の比較
        let names1: std::collections::HashSet<_> = config1.servers.iter().map(|s| &s.name).collect();
        let names2: std::collections::HashSet<_> = config2.servers.iter().map(|s| &s.name).collect();

        for name in names2.difference(&names1) {
            diff.server_changes.push(format!("+ Server added: {}", name));
        }

        for name in names1.difference(&names2) {
            diff.server_changes.push(format!("- Server removed: {}", name));
        }

        // ロギング設定の比較
        if config1.logging.backend != config2.logging.backend {
            diff.logging_changes.push(format!(
                "Backend: {} -> {}",
                config1.logging.backend, config2.logging.backend
            ));
        }

        if config1.logging.max_logs != config2.logging.max_logs {
            diff.logging_changes.push(format!(
                "Max logs: {} -> {}",
                config1.logging.max_logs, config2.logging.max_logs
            ));
        }

        // 実行設定の比較
        if config1.execution_config.verbose != config2.execution_config.verbose {
            diff.execution_changes.push(format!(
                "Verbose: {} -> {}",
                config1.execution_config.verbose, config2.execution_config.verbose
            ));
        }

        if config1.execution_config.tool_timeout_ms != config2.execution_config.tool_timeout_ms {
            diff.execution_changes.push(format!(
                "Tool timeout: {}ms -> {}ms",
                config1.execution_config.tool_timeout_ms, config2.execution_config.tool_timeout_ms
            ));
        }

        if config1.execution_config.connection_timeout_ms != config2.execution_config.connection_timeout_ms {
            diff.execution_changes.push(format!(
                "Connection timeout: {}ms -> {}ms",
                config1.execution_config.connection_timeout_ms, config2.execution_config.connection_timeout_ms
            ));
        }

        if config1.execution_config.retry_count != config2.execution_config.retry_count {
            diff.execution_changes.push(format!(
                "Retry count: {} -> {}",
                config1.execution_config.retry_count, config2.execution_config.retry_count
            ));
        }

        if config1.execution_config.auto_retry_on_timeout != config2.execution_config.auto_retry_on_timeout {
            diff.execution_changes.push(format!(
                "Auto retry: {} -> {}",
                config1.execution_config.auto_retry_on_timeout, config2.execution_config.auto_retry_on_timeout
            ));
        }

        diff
    }

    /// 設定のドライラン（実際には適用せずに差分を表示）
    ///
    /// # Arguments
    /// * `current_config` - 現在の設定
    /// * `import_path` - インポート予定のファイルパス
    /// * `format` - ファイル形式（Noneの場合はパスから推測）
    ///
    /// # Returns
    /// 適用予定の差分
    pub fn dry_run_import(
        current_config: &ProfileConfig,
        import_path: &Path,
        format: Option<ConfigFormat>,
    ) -> Result<ConfigDiff> {
        let new_config = Self::import_config(import_path, format)?;
        Ok(Self::diff_configs(current_config, &new_config))
    }
}

/// 設定の差分情報
#[derive(Debug, Default, Clone)]
pub struct ConfigDiff {
    /// 追加されたサーバー数
    pub servers_added: usize,
    /// 削除されたサーバー数
    pub servers_removed: usize,
    /// サーバーの変更詳細
    pub server_changes: Vec<String>,
    /// ロギング設定の変更
    pub logging_changes: Vec<String>,
    /// 実行設定の変更
    pub execution_changes: Vec<String>,
}

impl ConfigDiff {
    /// 変更があるかチェック
    pub fn has_changes(&self) -> bool {
        self.servers_added > 0
            || self.servers_removed > 0
            || !self.server_changes.is_empty()
            || !self.logging_changes.is_empty()
            || !self.execution_changes.is_empty()
    }

    /// 差分の総数を取得
    pub fn total_changes(&self) -> usize {
        self.server_changes.len()
            + self.logging_changes.len()
            + self.execution_changes.len()
    }

    /// 差分を人間が読みやすい形式でフォーマット
    pub fn format(&self) -> String {
        let mut output = String::new();

        if !self.has_changes() {
            output.push_str("No changes detected.\n");
            return output;
        }

        output.push_str(&format!("Total changes: {}\n\n", self.total_changes()));

        if !self.server_changes.is_empty() {
            output.push_str("Server changes:\n");
            for change in &self.server_changes {
                output.push_str(&format!("  {}\n", change));
            }
            output.push('\n');
        }

        if !self.logging_changes.is_empty() {
            output.push_str("Logging changes:\n");
            for change in &self.logging_changes {
                output.push_str(&format!("  {}\n", change));
            }
            output.push('\n');
        }

        if !self.execution_changes.is_empty() {
            output.push_str("Execution changes:\n");
            for change in &self.execution_changes {
                output.push_str(&format!("  {}\n", change));
            }
            output.push('\n');
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProfileConfig;
    use tempfile::TempDir;

    #[test]
    fn test_config_format_from_str() {
        assert_eq!("json".parse::<ConfigFormat>().unwrap(), ConfigFormat::Json);
        assert_eq!("yaml".parse::<ConfigFormat>().unwrap(), ConfigFormat::Yaml);
        assert_eq!("yml".parse::<ConfigFormat>().unwrap(), ConfigFormat::Yaml);
        assert!("xml".parse::<ConfigFormat>().is_err());
    }

    #[test]
    fn test_config_format_from_path() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.json")).unwrap(),
            ConfigFormat::Json
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yaml")).unwrap(),
            ConfigFormat::Yaml
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yml")).unwrap(),
            ConfigFormat::Yaml
        );
        assert!(ConfigFormat::from_path(Path::new("config.xml")).is_err());
        assert!(ConfigFormat::from_path(Path::new("config")).is_err());
    }

    #[test]
    fn test_export_import_json() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config_export.json");

        let config = ProfileConfig::dev_profile();

        // エクスポート
        let size = ConfigImportExport::export_config(&config, &output_path, Some(ConfigFormat::Json))
            .unwrap();
        assert!(size > 0);
        assert!(output_path.exists());

        // インポート
        let imported = ConfigImportExport::import_config(&output_path, Some(ConfigFormat::Json))
            .unwrap();
        assert_eq!(imported.execution_config.verbose, config.execution_config.verbose);
        assert_eq!(imported.execution_config.tool_timeout_ms, config.execution_config.tool_timeout_ms);
    }

    #[test]
    fn test_export_import_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("config_export.yaml");

        let config = ProfileConfig::staging_profile();

        // エクスポート
        let size = ConfigImportExport::export_config(&config, &output_path, Some(ConfigFormat::Yaml))
            .unwrap();
        assert!(size > 0);
        assert!(output_path.exists());

        // インポート
        let imported = ConfigImportExport::import_config(&output_path, Some(ConfigFormat::Yaml))
            .unwrap();
        assert_eq!(imported.execution_config.retry_count, config.execution_config.retry_count);
    }

    #[test]
    fn test_export_format_auto_detection() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("config.json");
        let yaml_path = temp_dir.path().join("config.yaml");

        let config = ProfileConfig::prod_profile();

        // JSON自動検出
        ConfigImportExport::export_config(&config, &json_path, None).unwrap();
        assert!(json_path.exists());

        // YAML自動検出
        ConfigImportExport::export_config(&config, &yaml_path, None).unwrap();
        assert!(yaml_path.exists());
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = ConfigImportExport::import_config(
            Path::new("/nonexistent/config.json"),
            Some(ConfigFormat::Json),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config = ProfileConfig::default_profile();
        ConfigImportExport::export_config(&config, &config_path, Some(ConfigFormat::Json))
            .unwrap();

        let warnings = ConfigImportExport::validate_config_file(&config_path, Some(ConfigFormat::Json))
            .unwrap();

        // デフォルト設定にはサーバーがないため警告が出る
        assert!(warnings.iter().any(|w| w.contains("No servers")));
    }

    #[test]
    fn test_diff_configs_no_change() {
        let config1 = ProfileConfig::dev_profile();
        let config2 = config1.clone();

        let diff = ConfigImportExport::diff_configs(&config1, &config2);
        assert!(!diff.has_changes());
        assert_eq!(diff.total_changes(), 0);
    }

    #[test]
    fn test_diff_configs_with_changes() {
        let config1 = ProfileConfig::dev_profile();
        let mut config2 = config1.clone();
        config2.execution_config.verbose = false;
        config2.execution_config.tool_timeout_ms = 20000;
        config2.logging.max_logs = 5000;

        let diff = ConfigImportExport::diff_configs(&config1, &config2);
        assert!(diff.has_changes());
        assert!(diff.total_changes() > 0);
        assert!(!diff.execution_changes.is_empty());
        assert!(!diff.logging_changes.is_empty());
    }

    #[test]
    fn test_diff_format() {
        let config1 = ProfileConfig::dev_profile();
        let mut config2 = config1.clone();
        config2.execution_config.verbose = false;

        let diff = ConfigImportExport::diff_configs(&config1, &config2);
        let formatted = diff.format();

        assert!(formatted.contains("Total changes:"));
        assert!(formatted.contains("Execution changes:"));
        assert!(formatted.contains("Verbose:"));
    }

    #[test]
    fn test_dry_run_import() {
        let temp_dir = TempDir::new().unwrap();
        let import_path = temp_dir.path().join("new_config.json");

        let current_config = ProfileConfig::dev_profile();
        let mut new_config = current_config.clone();
        new_config.execution_config.tool_timeout_ms = 15000;

        ConfigImportExport::export_config(&new_config, &import_path, Some(ConfigFormat::Json))
            .unwrap();

        let diff = ConfigImportExport::dry_run_import(
            &current_config,
            &import_path,
            Some(ConfigFormat::Json),
        )
        .unwrap();

        assert!(diff.has_changes());
        assert!(!diff.execution_changes.is_empty());
    }
}

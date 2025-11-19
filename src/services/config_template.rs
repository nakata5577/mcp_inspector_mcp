use crate::models::{InspectorError, ProfileConfig, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// テンプレート名を表す型エイリアス
pub type TemplateName = String;

/// プリセットテンプレートの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetTemplate {
    /// 最小限の設定
    Minimal,
    /// 開発環境向け設定
    Development,
    /// 本番環境向け設定
    Production,
    /// CI/CD環境向け設定
    CI,
}

impl FromStr for PresetTemplate {
    type Err = InspectorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(Self::Minimal),
            "development" | "dev" => Ok(Self::Development),
            "production" | "prod" => Ok(Self::Production),
            "ci" | "ci-cd" => Ok(Self::CI),
            _ => Err(InspectorError::Config(format!(
                "Unknown preset template: {}. Available: minimal, development, production, ci",
                s
            ))),
        }
    }
}

impl PresetTemplate {
    /// プリセットテンプレートの名前を取得
    pub fn name(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Development => "development",
            Self::Production => "production",
            Self::CI => "ci",
        }
    }

    /// プリセットテンプレートの説明を取得
    pub fn description(&self) -> &'static str {
        match self {
            Self::Minimal => "Minimal configuration with basic settings",
            Self::Development => "Development environment with verbose logging and relaxed timeouts",
            Self::Production => "Production environment with optimized settings and persistent logging",
            Self::CI => "CI/CD environment with extended timeouts and verbose output",
        }
    }

    /// プリセットテンプレートからProfileConfigを生成
    pub fn to_config(&self) -> ProfileConfig {
        match self {
            Self::Minimal => ProfileConfig::default_profile(),
            Self::Development => ProfileConfig::dev_profile(),
            Self::Production => ProfileConfig::prod_profile(),
            Self::CI => ProfileConfig::ci_profile(),
        }
    }

    /// すべてのプリセットテンプレートを取得
    pub fn all() -> Vec<Self> {
        vec![Self::Minimal, Self::Development, Self::Production, Self::CI]
    }
}

/// テンプレート情報
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// テンプレート名
    pub name: TemplateName,
    /// テンプレートの説明
    pub description: String,
    /// プリセットテンプレートかどうか
    pub is_preset: bool,
    /// カスタムテンプレートの場合のファイルパス
    pub file_path: Option<PathBuf>,
}

/// 設定テンプレート管理サービス
pub struct ConfigTemplate {
    templates_dir: PathBuf,
}

impl ConfigTemplate {
    /// 新しいConfigTemplateを作成
    ///
    /// `.inspector/templates`ディレクトリが存在しない場合は自動的に作成する
    pub fn new() -> Result<Self> {
        let templates_dir = Self::ensure_templates_dir()?;
        Ok(Self { templates_dir })
    }

    /// `.inspector/templates`ディレクトリのパスを取得し、存在しない場合は作成
    fn ensure_templates_dir() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()
            .map_err(|e| InspectorError::Config(format!("Failed to get current directory: {}", e)))?;

        let inspector_dir = current_dir.join(".inspector");
        let templates_dir = inspector_dir.join("templates");

        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to create templates directory: {}", e)
                ))?;
        }

        Ok(templates_dir)
    }

    /// テンプレート名からファイルパスを取得
    fn get_template_path(&self, template_name: &str) -> PathBuf {
        self.templates_dir.join(format!("{}.json", template_name))
    }

    /// プリセットテンプレート一覧を取得
    pub fn list_preset_templates(&self) -> Vec<TemplateInfo> {
        PresetTemplate::all()
            .iter()
            .map(|preset| TemplateInfo {
                name: preset.name().to_string(),
                description: preset.description().to_string(),
                is_preset: true,
                file_path: None,
            })
            .collect()
    }

    /// カスタムテンプレート一覧を取得
    pub fn list_custom_templates(&self) -> Result<Vec<TemplateInfo>> {
        let mut templates = Vec::new();

        if !self.templates_dir.exists() {
            return Ok(templates);
        }

        let entries = fs::read_dir(&self.templates_dir)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read templates directory: {}", e)
            ))?;

        for entry in entries {
            let entry = entry.map_err(|e| InspectorError::Config(
                format!("Failed to read directory entry: {}", e)
            ))?;

            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.ends_with(".json") {
                        let template_name = filename
                            .strip_suffix(".json")
                            .unwrap_or("")
                            .to_string();

                        if !template_name.is_empty() {
                            // テンプレートから説明を読み込み
                            let description = self.load_template_description(&path)
                                .unwrap_or_else(|_| "Custom template".to_string());

                            templates.push(TemplateInfo {
                                name: template_name,
                                description,
                                is_preset: false,
                                file_path: Some(path),
                            });
                        }
                    }
                }
            }
        }

        templates.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(templates)
    }

    /// すべてのテンプレート一覧を取得（プリセット + カスタム）
    pub fn list_all_templates(&self) -> Result<Vec<TemplateInfo>> {
        let mut templates = self.list_preset_templates();
        templates.extend(self.list_custom_templates()?);
        Ok(templates)
    }

    /// テンプレートから説明を読み込む
    fn load_template_description(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read template from {}: {}", path.display(), e)
            ))?;

        let config: ProfileConfig = serde_json::from_str(&content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to parse template: {}", e)
            ))?;

        Ok(config.metadata.description.unwrap_or_else(|| "Custom template".to_string()))
    }

    /// テンプレートを適用して設定を生成
    ///
    /// # Arguments
    /// * `template_name` - テンプレート名（プリセットまたはカスタム）
    ///
    /// # Returns
    /// 生成されたProfileConfig
    pub fn apply_template(&self, template_name: &str) -> Result<ProfileConfig> {
        // プリセットテンプレートをチェック
        if let Ok(preset) = template_name.parse::<PresetTemplate>() {
            return Ok(preset.to_config());
        }

        // カスタムテンプレートを読み込み
        let template_path = self.get_template_path(template_name);
        if !template_path.exists() {
            return Err(InspectorError::Config(format!(
                "Template '{}' does not exist",
                template_name
            )));
        }

        let content = fs::read_to_string(&template_path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read template '{}': {}", template_name, e)
            ))?;

        let config: ProfileConfig = serde_json::from_str(&content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to parse template '{}': {}", template_name, e)
            ))?;

        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Template '{}' validation failed: {}", template_name, e)
            ))?;

        Ok(config)
    }

    /// カスタムテンプレートを作成
    ///
    /// # Arguments
    /// * `template_name` - テンプレート名
    /// * `config` - テンプレートとして保存する設定
    ///
    /// # Errors
    /// 同名のテンプレートが既に存在する場合、バリデーションエラーの場合にエラーを返す
    pub fn create_custom_template(&self, template_name: &str, config: &ProfileConfig) -> Result<()> {
        // プリセットテンプレート名との重複チェック
        if template_name.parse::<PresetTemplate>().is_ok() {
            return Err(InspectorError::Config(format!(
                "Template name '{}' conflicts with a preset template",
                template_name
            )));
        }

        // 既存のカスタムテンプレートとの重複チェック
        let template_path = self.get_template_path(template_name);
        if template_path.exists() {
            return Err(InspectorError::Config(format!(
                "Custom template '{}' already exists",
                template_name
            )));
        }

        // バリデーション
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Template validation failed: {}", e)
            ))?;

        // ファイルに保存
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| InspectorError::Config(
                format!("Failed to serialize template: {}", e)
            ))?;

        fs::write(&template_path, content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to write template '{}': {}", template_name, e)
            ))?;

        tracing::info!("Custom template '{}' created at {}", template_name, template_path.display());

        Ok(())
    }

    /// カスタムテンプレートを削除
    ///
    /// # Arguments
    /// * `template_name` - テンプレート名
    ///
    /// # Errors
    /// プリセットテンプレートを削除しようとした場合、テンプレートが存在しない場合にエラーを返す
    pub fn delete_custom_template(&self, template_name: &str) -> Result<()> {
        // プリセットテンプレートは削除不可
        if template_name.parse::<PresetTemplate>().is_ok() {
            return Err(InspectorError::Config(format!(
                "Cannot delete preset template '{}'",
                template_name
            )));
        }

        let template_path = self.get_template_path(template_name);
        if !template_path.exists() {
            return Err(InspectorError::Config(format!(
                "Custom template '{}' does not exist",
                template_name
            )));
        }

        fs::remove_file(&template_path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to delete template '{}': {}", template_name, e)
            ))?;

        tracing::info!("Custom template '{}' deleted", template_name);

        Ok(())
    }

    /// テンプレートを表示（JSON形式）
    ///
    /// # Arguments
    /// * `template_name` - テンプレート名
    ///
    /// # Returns
    /// テンプレートのJSON文字列
    pub fn show_template(&self, template_name: &str) -> Result<String> {
        let config = self.apply_template(template_name)?;
        config.to_json_string()
            .map_err(|e| InspectorError::Config(
                format!("Failed to serialize template: {}", e)
            ))
    }
}

impl Default for ConfigTemplate {
    fn default() -> Self {
        Self::new().expect("Failed to create ConfigTemplate")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> (TempDir, ConfigTemplate) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        let template_mgr = ConfigTemplate::new().unwrap();
        (temp_dir, template_mgr)
    }

    #[test]
    fn test_preset_template_from_str() {
        assert_eq!("minimal".parse::<PresetTemplate>().unwrap(), PresetTemplate::Minimal);
        assert_eq!("development".parse::<PresetTemplate>().unwrap(), PresetTemplate::Development);
        assert_eq!("dev".parse::<PresetTemplate>().unwrap(), PresetTemplate::Development);
        assert_eq!("production".parse::<PresetTemplate>().unwrap(), PresetTemplate::Production);
        assert_eq!("prod".parse::<PresetTemplate>().unwrap(), PresetTemplate::Production);
        assert_eq!("ci".parse::<PresetTemplate>().unwrap(), PresetTemplate::CI);
        assert!("unknown".parse::<PresetTemplate>().is_err());
    }

    #[test]
    fn test_preset_template_properties() {
        let minimal = PresetTemplate::Minimal;
        assert_eq!(minimal.name(), "minimal");
        assert!(!minimal.description().is_empty());

        let config = minimal.to_config();
        assert_eq!(config.servers.len(), 0);
    }

    #[test]
    fn test_list_preset_templates() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let presets = template_mgr.list_preset_templates();
        assert_eq!(presets.len(), 4); // minimal, development, production, ci

        for preset in &presets {
            assert!(preset.is_preset);
            assert!(preset.file_path.is_none());
            assert!(!preset.description.is_empty());
        }
    }

    #[test]
    fn test_apply_preset_template() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let config = template_mgr.apply_template("development").unwrap();
        assert!(config.execution_config.verbose);

        let config = template_mgr.apply_template("production").unwrap();
        assert_eq!(config.logging.backend, "persistent");
    }

    #[test]
    fn test_create_custom_template() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        template_mgr.create_custom_template("my_template", &config).unwrap();

        let loaded = template_mgr.apply_template("my_template").unwrap();
        assert_eq!(loaded.execution_config.verbose, config.execution_config.verbose);
    }

    #[test]
    fn test_create_custom_template_duplicate() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        template_mgr.create_custom_template("my_template", &config).unwrap();

        // 重複作成はエラー
        assert!(template_mgr.create_custom_template("my_template", &config).is_err());
    }

    #[test]
    fn test_create_custom_template_preset_name_conflict() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let config = ProfileConfig::dev_profile();

        // プリセット名との競合はエラー
        assert!(template_mgr.create_custom_template("minimal", &config).is_err());
        assert!(template_mgr.create_custom_template("development", &config).is_err());
    }

    #[test]
    fn test_list_custom_templates() {
        let (_temp_dir, template_mgr) = setup_test_env();

        // 初期状態では空
        let customs = template_mgr.list_custom_templates().unwrap();
        assert_eq!(customs.len(), 0);

        // カスタムテンプレートを作成
        template_mgr.create_custom_template("template1", &ProfileConfig::dev_profile()).unwrap();
        template_mgr.create_custom_template("template2", &ProfileConfig::staging_profile()).unwrap();

        let customs = template_mgr.list_custom_templates().unwrap();
        assert_eq!(customs.len(), 2);

        for custom in &customs {
            assert!(!custom.is_preset);
            assert!(custom.file_path.is_some());
        }
    }

    #[test]
    fn test_list_all_templates() {
        let (_temp_dir, template_mgr) = setup_test_env();

        template_mgr.create_custom_template("my_template", &ProfileConfig::dev_profile()).unwrap();

        let all = template_mgr.list_all_templates().unwrap();
        assert!(all.len() >= 5); // 4 presets + 1 custom

        let preset_count = all.iter().filter(|t| t.is_preset).count();
        let custom_count = all.iter().filter(|t| !t.is_preset).count();

        assert_eq!(preset_count, 4);
        assert_eq!(custom_count, 1);
    }

    #[test]
    fn test_delete_custom_template() {
        let (_temp_dir, template_mgr) = setup_test_env();

        template_mgr.create_custom_template("my_template", &ProfileConfig::dev_profile()).unwrap();

        // 削除
        template_mgr.delete_custom_template("my_template").unwrap();

        // 削除後は存在しない
        assert!(template_mgr.apply_template("my_template").is_err());
    }

    #[test]
    fn test_delete_preset_template_fails() {
        let (_temp_dir, template_mgr) = setup_test_env();

        // プリセットテンプレートは削除不可
        assert!(template_mgr.delete_custom_template("minimal").is_err());
        assert!(template_mgr.delete_custom_template("development").is_err());
    }

    #[test]
    fn test_show_template() {
        let (_temp_dir, template_mgr) = setup_test_env();

        let json = template_mgr.show_template("development").unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("execution_config"));
        assert!(json.contains("logging"));
    }

    #[test]
    fn test_apply_nonexistent_template() {
        let (_temp_dir, template_mgr) = setup_test_env();

        assert!(template_mgr.apply_template("nonexistent").is_err());
    }
}

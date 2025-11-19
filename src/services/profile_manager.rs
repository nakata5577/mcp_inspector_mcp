use crate::models::{InspectorError, ProfileConfig, ProfileInfo, ProfileName, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// プロファイル管理サービス
///
/// プロファイルの読み込み、切替、検証、一覧表示などの機能を提供
pub struct ProfileManager {
    inspector_dir: PathBuf,
}

impl ProfileManager {
    /// 新しいProfileManagerを作成
    ///
    /// `.inspector`ディレクトリが存在しない場合は自動的に作成する
    pub fn new() -> Result<Self> {
        let inspector_dir = Self::ensure_inspector_dir()?;
        Ok(Self { inspector_dir })
    }

    /// `.inspector`ディレクトリのパスを取得し、存在しない場合は作成
    fn ensure_inspector_dir() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()
            .map_err(|e| InspectorError::Config(format!("Failed to get current directory: {}", e)))?;

        let inspector_dir = current_dir.join(".inspector");

        if !inspector_dir.exists() {
            fs::create_dir_all(&inspector_dir)
                .map_err(|e| InspectorError::Config(
                    format!("Failed to create .inspector directory: {}", e)
                ))?;

            // Windowsの場合は隠しフォルダ属性を設定
            #[cfg(windows)]
            {
                use std::process::Command;
                let dir_path = inspector_dir.to_string_lossy().to_string();
                let _ = Command::new("attrib")
                    .args(["+H", &dir_path])
                    .output();
            }
        }

        Ok(inspector_dir)
    }

    /// プロファイル名から設定ファイルのパスを取得
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名（空の場合はデフォルト設定ファイル）
    ///
    /// # Returns
    /// 設定ファイルのフルパス
    fn get_profile_path(&self, profile_name: &str) -> PathBuf {
        if profile_name.is_empty() {
            self.inspector_dir.join("config.json")
        } else {
            self.inspector_dir.join(format!("config.{}.json", profile_name))
        }
    }

    /// プロファイル設定を読み込む
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名（空の場合はデフォルト設定）
    ///
    /// # Returns
    /// 読み込まれたProfileConfig
    ///
    /// # Errors
    /// ファイルが存在しない、読み込みエラー、パースエラーの場合にエラーを返す
    pub fn load_profile(&self, profile_name: &str) -> Result<ProfileConfig> {
        let config_path = self.get_profile_path(profile_name);

        if !config_path.exists() {
            return Err(InspectorError::Config(format!(
                "Profile '{}' does not exist at {}",
                profile_name,
                config_path.display()
            )));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read profile '{}': {}", profile_name, e)
            ))?;

        let config: ProfileConfig = serde_json::from_str(&content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to parse profile '{}': {}", profile_name, e)
            ))?;

        // バリデーション
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Profile '{}' validation failed: {}", profile_name, e)
            ))?;

        Ok(config)
    }

    /// プロファイル設定を保存
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名（空の場合はデフォルト設定）
    /// * `config` - 保存するProfileConfig
    ///
    /// # Errors
    /// バリデーションエラー、シリアライズエラー、書き込みエラーの場合にエラーを返す
    pub fn save_profile(&self, profile_name: &str, config: &ProfileConfig) -> Result<()> {
        // バリデーション
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Profile '{}' validation failed: {}", profile_name, e)
            ))?;

        let config_path = self.get_profile_path(profile_name);

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| InspectorError::Config(
                format!("Failed to serialize profile '{}': {}", profile_name, e)
            ))?;

        fs::write(&config_path, content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to write profile '{}': {}", profile_name, e)
            ))?;

        tracing::info!("Profile '{}' saved to {}", profile_name, config_path.display());

        Ok(())
    }

    /// プロファイルが存在するかチェック
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名
    ///
    /// # Returns
    /// プロファイルが存在する場合はtrue
    pub fn profile_exists(&self, profile_name: &str) -> bool {
        self.get_profile_path(profile_name).exists()
    }

    /// 利用可能なプロファイルの一覧を取得
    ///
    /// # Returns
    /// ProfileInfoのベクター（デフォルト設定も含む）
    pub fn list_profiles(&self) -> Result<Vec<ProfileInfo>> {
        let mut profiles = Vec::new();

        // デフォルト設定（config.json）をチェック
        let default_path = self.inspector_dir.join("config.json");
        if default_path.exists() {
            let config = self.load_profile_from_path(&default_path)?;
            profiles.push(ProfileInfo::new(
                "default".to_string(),
                default_path.to_string_lossy().to_string(),
                true,
                config.metadata.description.clone(),
                config.metadata.tags.clone(),
            ));
        }

        // config.*.jsonファイルを検索
        let entries = fs::read_dir(&self.inspector_dir)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read .inspector directory: {}", e)
            ))?;

        for entry in entries {
            let entry = entry.map_err(|e| InspectorError::Config(
                format!("Failed to read directory entry: {}", e)
            ))?;

            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();

                    // config.{profile}.json形式のファイルを抽出
                    if filename.starts_with("config.") && filename.ends_with(".json") && filename != "config.json" {
                        let profile_name = filename
                            .strip_prefix("config.")
                            .and_then(|s| s.strip_suffix(".json"))
                            .unwrap_or("")
                            .to_string();

                        if !profile_name.is_empty() {
                            let config = self.load_profile_from_path(&path)?;
                            profiles.push(ProfileInfo::new(
                                profile_name,
                                path.to_string_lossy().to_string(),
                                true,
                                config.metadata.description.clone(),
                                config.metadata.tags.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // プロファイル名でソート
        profiles.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(profiles)
    }

    /// パスからプロファイル設定を読み込む（内部用）
    fn load_profile_from_path(&self, path: &Path) -> Result<ProfileConfig> {
        let content = fs::read_to_string(path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to read profile from {}: {}", path.display(), e)
            ))?;

        let config: ProfileConfig = serde_json::from_str(&content)
            .map_err(|e| InspectorError::Config(
                format!("Failed to parse profile from {}: {}", path.display(), e)
            ))?;

        Ok(config)
    }

    /// プロファイルをバリデーション
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名
    ///
    /// # Returns
    /// バリデーション結果（成功時はOk(())）
    pub fn validate_profile(&self, profile_name: &str) -> Result<()> {
        let config = self.load_profile(profile_name)?;
        config.validate()
            .map_err(|e| InspectorError::Config(
                format!("Profile '{}' validation failed: {}", profile_name, e)
            ))?;
        Ok(())
    }

    /// プロファイルを削除
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名（デフォルト設定は削除不可）
    ///
    /// # Errors
    /// デフォルト設定を削除しようとした場合、ファイルが存在しない場合にエラーを返す
    pub fn delete_profile(&self, profile_name: &str) -> Result<()> {
        if profile_name.is_empty() || profile_name == "default" {
            return Err(InspectorError::Config(
                "Cannot delete default profile".to_string()
            ));
        }

        let config_path = self.get_profile_path(profile_name);

        if !config_path.exists() {
            return Err(InspectorError::Config(format!(
                "Profile '{}' does not exist",
                profile_name
            )));
        }

        fs::remove_file(&config_path)
            .map_err(|e| InspectorError::Config(
                format!("Failed to delete profile '{}': {}", profile_name, e)
            ))?;

        tracing::info!("Profile '{}' deleted", profile_name);

        Ok(())
    }

    /// デフォルト設定またはプロファイルを読み込む（環境変数も考慮）
    ///
    /// 読み込み優先順位:
    /// 1. 引数で指定されたprofile_name
    /// 2. 環境変数MCP_PROFILE
    /// 3. デフォルト設定（config.json）
    ///
    /// # Arguments
    /// * `profile_name` - プロファイル名（Noneの場合は環境変数またはデフォルト）
    ///
    /// # Returns
    /// 読み込まれたProfileConfigとプロファイル名のタプル
    pub fn load_active_profile(&self, profile_name: Option<&str>) -> Result<(ProfileConfig, ProfileName)> {
        // 引数で指定されたプロファイルを優先
        if let Some(name) = profile_name {
            if !name.is_empty() {
                let config = self.load_profile(name)?;
                return Ok((config, name.to_string()));
            }
        }

        // 環境変数MCP_PROFILEをチェック
        if let Ok(env_profile) = std::env::var("MCP_PROFILE") {
            if !env_profile.is_empty() && self.profile_exists(&env_profile) {
                let config = self.load_profile(&env_profile)?;
                tracing::info!("Using profile '{}' from MCP_PROFILE environment variable", env_profile);
                return Ok((config, env_profile));
            }
        }

        // デフォルト設定を読み込む
        let default_path = self.inspector_dir.join("config.json");
        if default_path.exists() {
            let config = self.load_profile("")?;
            return Ok((config, "default".to_string()));
        }

        // デフォルト設定が存在しない場合は作成
        let default_config = ProfileConfig::default_profile();
        self.save_profile("", &default_config)?;
        Ok((default_config, "default".to_string()))
    }

    /// プロファイルのクローンを作成
    ///
    /// # Arguments
    /// * `source_profile` - コピー元プロファイル名
    /// * `dest_profile` - コピー先プロファイル名
    ///
    /// # Errors
    /// コピー元が存在しない、コピー先が既に存在する場合にエラーを返す
    pub fn clone_profile(&self, source_profile: &str, dest_profile: &str) -> Result<()> {
        if dest_profile.is_empty() || dest_profile == "default" {
            return Err(InspectorError::Config(
                "Cannot overwrite default profile".to_string()
            ));
        }

        if self.profile_exists(dest_profile) {
            return Err(InspectorError::Config(format!(
                "Profile '{}' already exists",
                dest_profile
            )));
        }

        let config = self.load_profile(source_profile)?;
        self.save_profile(dest_profile, &config)?;

        tracing::info!("Profile '{}' cloned to '{}'", source_profile, dest_profile);

        Ok(())
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ProfileManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;

    fn setup_test_env() -> (TempDir, ProfileManager) {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();
        let manager = ProfileManager::new().unwrap();
        (temp_dir, manager)
    }

    #[test]
    fn test_new_profile_manager() {
        let (_temp_dir, manager) = setup_test_env();
        assert!(manager.inspector_dir.exists());
    }

    #[test]
    fn test_save_and_load_profile() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        manager.save_profile("dev", &config).unwrap();

        let loaded = manager.load_profile("dev").unwrap();
        assert!(loaded.execution_config.verbose);
        assert_eq!(loaded.execution_config.tool_timeout_ms, 10000);
    }

    #[test]
    fn test_profile_exists() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        manager.save_profile("dev", &config).unwrap();

        assert!(manager.profile_exists("dev"));
        assert!(!manager.profile_exists("nonexistent"));
    }

    #[test]
    fn test_list_profiles() {
        let (_temp_dir, manager) = setup_test_env();

        // デフォルト設定を作成
        let default_config = ProfileConfig::default_profile();
        manager.save_profile("", &default_config).unwrap();

        // 複数のプロファイルを作成
        manager.save_profile("dev", &ProfileConfig::dev_profile()).unwrap();
        manager.save_profile("staging", &ProfileConfig::staging_profile()).unwrap();
        manager.save_profile("prod", &ProfileConfig::prod_profile()).unwrap();

        let profiles = manager.list_profiles().unwrap();
        assert!(profiles.len() >= 4); // default + dev + staging + prod

        // プロファイル名でソートされていることを確認
        let names: Vec<_> = profiles.iter().map(|p| &p.name).collect();
        assert!(names.contains(&&"default".to_string()));
        assert!(names.contains(&&"dev".to_string()));
        assert!(names.contains(&&"staging".to_string()));
        assert!(names.contains(&&"prod".to_string()));
    }

    #[test]
    fn test_validate_profile_success() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        manager.save_profile("dev", &config).unwrap();

        assert!(manager.validate_profile("dev").is_ok());
    }

    #[test]
    fn test_validate_profile_failure() {
        let (_temp_dir, manager) = setup_test_env();

        let mut config = ProfileConfig::dev_profile();
        config.execution_config.tool_timeout_ms = 0; // invalid
        manager.save_profile("invalid", &config).unwrap_err();
    }

    #[test]
    fn test_delete_profile() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        manager.save_profile("dev", &config).unwrap();

        assert!(manager.profile_exists("dev"));
        manager.delete_profile("dev").unwrap();
        assert!(!manager.profile_exists("dev"));
    }

    #[test]
    fn test_delete_default_profile_fails() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::default_profile();
        manager.save_profile("", &config).unwrap();

        assert!(manager.delete_profile("default").is_err());
        assert!(manager.delete_profile("").is_err());
    }

    #[test]
    fn test_clone_profile() {
        let (_temp_dir, manager) = setup_test_env();

        let config = ProfileConfig::dev_profile();
        manager.save_profile("dev", &config).unwrap();

        manager.clone_profile("dev", "dev2").unwrap();

        let cloned = manager.load_profile("dev2").unwrap();
        assert!(cloned.execution_config.verbose);
        assert_eq!(cloned.execution_config.tool_timeout_ms, 10000);
    }

    #[test]
    fn test_clone_profile_already_exists() {
        let (_temp_dir, manager) = setup_test_env();

        manager.save_profile("dev", &ProfileConfig::dev_profile()).unwrap();
        manager.save_profile("staging", &ProfileConfig::staging_profile()).unwrap();

        assert!(manager.clone_profile("dev", "staging").is_err());
    }

    #[test]
    fn test_load_active_profile_with_argument() {
        let (_temp_dir, manager) = setup_test_env();

        manager.save_profile("dev", &ProfileConfig::dev_profile()).unwrap();

        let (config, name) = manager.load_active_profile(Some("dev")).unwrap();
        assert_eq!(name, "dev");
        assert!(config.execution_config.verbose);
    }

    #[test]
    fn test_load_active_profile_with_env() {
        let (_temp_dir, manager) = setup_test_env();

        manager.save_profile("staging", &ProfileConfig::staging_profile()).unwrap();

        env::set_var("MCP_PROFILE", "staging");
        let (config, name) = manager.load_active_profile(None).unwrap();
        env::remove_var("MCP_PROFILE");

        assert_eq!(name, "staging");
        assert_eq!(config.execution_config.retry_count, 2);
    }

    #[test]
    fn test_load_active_profile_default() {
        let (_temp_dir, manager) = setup_test_env();

        let (config, name) = manager.load_active_profile(None).unwrap();
        assert_eq!(name, "default");
        assert_eq!(config.servers.len(), 0);
    }
}

use crate::models::{InspectorError, ProjectConfig, Result, ServerEntry};
use std::fs;
use std::path::PathBuf;

/// `.inspector/config.json`ファイルのパスを取得
fn get_config_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| InspectorError::Config(format!("カレントディレクトリの取得に失敗: {}", e)))?;

    let inspector_dir = current_dir.join(".inspector");
    let config_path = inspector_dir.join("config.json");

    Ok(config_path)
}

/// `.inspector`ディレクトリが存在しない場合は作成
fn ensure_inspector_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .map_err(|e| InspectorError::Config(format!("カレントディレクトリの取得に失敗: {}", e)))?;

    let inspector_dir = current_dir.join(".inspector");

    if !inspector_dir.exists() {
        fs::create_dir_all(&inspector_dir)
            .map_err(|e| InspectorError::Config(
                format!(".inspectorディレクトリの作成に失敗: {}", e)
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

/// 設定ファイルを読み込む。存在しない場合はデフォルト設定で作成
pub fn load_config() -> Result<ProjectConfig> {
    ensure_inspector_dir()?;
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // デフォルト設定を作成して保存
        let default_config = ProjectConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| InspectorError::Config(
            format!("設定ファイルの読み込みに失敗: {}", e)
        ))?;

    let config: ProjectConfig = serde_json::from_str(&content)
        .map_err(|e| InspectorError::Config(
            format!("設定ファイルのパースに失敗: {}", e)
        ))?;

    Ok(config)
}

/// 設定ファイルを保存
pub fn save_config(config: &ProjectConfig) -> Result<()> {
    ensure_inspector_dir()?;
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| InspectorError::Config(
            format!("設定ファイルのシリアライズに失敗: {}", e)
        ))?;

    fs::write(&config_path, content)
        .map_err(|e| InspectorError::Config(
            format!("設定ファイルの書き込みに失敗: {}", e)
        ))?;

    Ok(())
}

/// サーバー設定を追加
pub fn add_server(entry: ServerEntry) -> Result<()> {
    let mut config = load_config()?;

    // 同名のサーバーが既に存在する場合はエラー
    if config.servers.iter().any(|s| s.name == entry.name) {
        return Err(InspectorError::Config(
            format!("サーバー '{}' は既に存在します", entry.name)
        ));
    }

    config.servers.push(entry);
    save_config(&config)?;

    Ok(())
}

/// サーバー設定を削除
pub fn remove_server(name: &str) -> Result<()> {
    let mut config = load_config()?;

    let original_len = config.servers.len();
    config.servers.retain(|s| s.name != name);

    if config.servers.len() == original_len {
        return Err(InspectorError::Config(
            format!("サーバー '{}' が見つかりません", name)
        ));
    }

    save_config(&config)?;

    Ok(())
}

/// サーバー一覧を取得
pub fn list_servers() -> Result<Vec<ServerEntry>> {
    let config = load_config()?;
    Ok(config.servers)
}

/// 特定のサーバーを名前で取得
pub fn get_server(name: &str) -> Result<Option<ServerEntry>> {
    let config = load_config()?;
    Ok(config.servers.into_iter().find(|s| s.name == name))
}

#[cfg(test)]
mod tests {
    // Note: テストは環境の競合を避けるため、統合テストに移行しました
    // 統合テストは tests/config_manager_test.rs を参照してください
}

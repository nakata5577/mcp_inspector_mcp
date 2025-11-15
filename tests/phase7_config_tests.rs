//! Phase 7 単体テスト: 環境変数ベース設定管理
//!
//! このテストスイートは、Phase 7.1で実装された環境変数からの設定読み込み機能を検証します。
//!
//! テスト対象:
//! - InspectorConfig::from_env() - サーバー設定の読み込み
//! - LoggingConfig::from_env() - ログ設定の読み込み
//! - 入力バリデーション
//! - デフォルト値の適用
//! - エラーハンドリング
//!
//! 注意: 環境変数を使用するため、並列実行を防ぐため `-- --test-threads=1` で実行することを推奨

use mcp_inspector_mcp::models::{InspectorConfig, LoggingConfig};
use std::env;
use std::sync::Mutex;

// 環境変数の競合を防ぐためのグローバルロック
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ================================================================================
// テストヘルパー関数
// ================================================================================

/// テスト用に環境変数を安全に設定・削除するヘルパー構造体
struct EnvGuard {
    vars: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self {
        // 既存の環境変数をクリア
        let known_vars = [
            "MCP_INSPECTOR_SERVERS",
            "MCP_LOGGING_BACKEND",
            "MCP_LOGGING_DB_PATH",
            "MCP_LOGGING_MAX_LOGS",
        ];
        for var in &known_vars {
            env::remove_var(var);
        }

        Self { vars: Vec::new() }
    }

    fn set(&mut self, key: &str, value: &str) {
        env::set_var(key, value);
        self.vars.push(key.to_string());
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for var in &self.vars {
            env::remove_var(var);
        }
    }
}

// ================================================================================
// InspectorConfig::from_env() のテスト
// ================================================================================

#[test]
fn test_parse_servers_from_env_valid() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 有効なJSON配列を設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[{"name":"test","transport":"stdio","command":"test.exe","args":[],"env":{}}]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(
        result.is_ok(),
        "有効なJSON設定のパースが失敗しました: {:?}",
        result
    );

    let config = result.unwrap();
    assert_eq!(config.servers.len(), 1, "サーバー数が期待値と一致しません");
    assert_eq!(
        config.servers[0].name, "test",
        "サーバー名が期待値と一致しません"
    );
    assert_eq!(
        config.servers[0].params.command, "test.exe",
        "コマンドが期待値と一致しません"
    );
}

#[test]
fn test_parse_servers_from_env_invalid_json() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 不正なJSON文字列を設定
    env_guard.set("MCP_INSPECTOR_SERVERS", "not valid json");
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(result.is_err(), "不正なJSONが受け入れられてしまいました");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to parse"),
        "エラーメッセージに'Failed to parse'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_parse_servers_from_env_empty_array() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 空のJSON配列を設定
    env_guard.set("MCP_INSPECTOR_SERVERS", "[]");
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(result.is_err(), "空の配列が受け入れられてしまいました");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("at least one server"),
        "エラーメッセージに'at least one server'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_parse_servers_from_env_missing_name() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // nameフィールドを空文字列にしたサーバー設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[{"name":"","transport":"stdio","command":"test.exe","args":[],"env":{}}]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(
        result.is_err(),
        "空のname フィールドが受け入れられてしまいました"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("empty 'name' field"),
        "エラーメッセージに'empty 'name' field'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_parse_servers_from_env_missing_command() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // commandフィールドを空文字列にしたサーバー設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[{"name":"test","transport":"stdio","command":"","args":[],"env":{}}]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(
        result.is_err(),
        "空のcommandフィールドが受け入れられてしまいました"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("empty 'command' field"),
        "エラーメッセージに'empty 'command' field'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_parse_servers_from_env_multiple_servers() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 複数サーバーの設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[
            {"name":"server1","transport":"stdio","command":"cmd1.exe","args":["arg1"],"env":{}},
            {"name":"server2","transport":"stdio","command":"cmd2.exe","args":["arg2","arg3"],"env":{"KEY":"VALUE"}}
        ]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = InspectorConfig::from_env();
    assert!(result.is_ok(), "複数サーバー設定のパースが失敗しました");

    let config = result.unwrap();
    assert_eq!(config.servers.len(), 2, "サーバー数が2であるべきです");

    // 1つ目のサーバー
    assert_eq!(config.servers[0].name, "server1");
    assert_eq!(config.servers[0].params.command, "cmd1.exe");
    assert_eq!(config.servers[0].params.args.len(), 1);

    // 2つ目のサーバー
    assert_eq!(config.servers[1].name, "server2");
    assert_eq!(config.servers[1].params.command, "cmd2.exe");
    assert_eq!(config.servers[1].params.args.len(), 2);
    assert_eq!(
        config.servers[1].params.env.get("KEY"),
        Some(&"VALUE".to_string())
    );
}

#[test]
fn test_parse_servers_from_env_not_set() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _env_guard = EnvGuard::new();

    // MCP_INSPECTOR_SERVERSを設定しない
    let result = InspectorConfig::from_env();
    assert!(
        result.is_err(),
        "環境変数未設定時にエラーが返されませんでした"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("MCP_INSPECTOR_SERVERS"),
        "エラーメッセージに'MCP_INSPECTOR_SERVERS'が含まれていません: {}",
        error_msg
    );
}

// ================================================================================
// LoggingConfig::from_env() のテスト
// ================================================================================

#[test]
fn test_logging_config_from_env_memory() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // メモリバックエンドを設定
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let result = LoggingConfig::from_env();
    assert!(
        result.is_ok(),
        "メモリバックエンド設定のパースが失敗しました"
    );

    let config = result.unwrap();
    assert_eq!(
        config.backend,
        mcp_inspector_mcp::models::LoggingBackend::Memory,
        "バックエンドがMemoryであるべきです"
    );
    assert_eq!(
        config.max_logs, 10000,
        "デフォルトのmax_logsは10000であるべきです"
    );
    assert!(
        config.db_path.is_none(),
        "メモリバックエンドではdb_pathはNoneであるべきです"
    );
}

#[test]
fn test_logging_config_from_env_persistent() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 永続バックエンドを設定
    env_guard.set("MCP_LOGGING_BACKEND", "persistent");
    env_guard.set("MCP_LOGGING_DB_PATH", "./test.db");

    let result = LoggingConfig::from_env();
    assert!(result.is_ok(), "永続バックエンド設定のパースが失敗しました");

    let config = result.unwrap();
    assert_eq!(
        config.backend,
        mcp_inspector_mcp::models::LoggingBackend::Persistent,
        "バックエンドがPersistentであるべきです"
    );
    assert_eq!(
        config.db_path,
        Some("./test.db".to_string()),
        "db_pathが設定されているべきです"
    );
}

#[test]
fn test_logging_config_from_env_defaults() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _env_guard = EnvGuard::new();

    // ログ関連の環境変数を設定しない（デフォルト値を使用）
    let result = LoggingConfig::from_env();
    assert!(result.is_ok(), "デフォルト設定のパースが失敗しました");

    let config = result.unwrap();
    assert_eq!(
        config.backend,
        mcp_inspector_mcp::models::LoggingBackend::Memory,
        "デフォルトバックエンドはMemoryであるべきです"
    );
    assert_eq!(
        config.max_logs, 10000,
        "デフォルトのmax_logsは10000であるべきです"
    );
}

#[test]
fn test_logging_config_persistent_missing_db_path() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 永続バックエンドを設定するがdb_pathを設定しない
    env_guard.set("MCP_LOGGING_BACKEND", "persistent");

    let result = LoggingConfig::from_env();
    assert!(
        result.is_err(),
        "db_path未設定時にエラーが返されませんでした"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("MCP_LOGGING_DB_PATH") || error_msg.contains("db_path"),
        "エラーメッセージに'MCP_LOGGING_DB_PATH'または'db_path'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_logging_config_custom_max_logs() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // カスタムmax_logsを設定
    env_guard.set("MCP_LOGGING_BACKEND", "memory");
    env_guard.set("MCP_LOGGING_MAX_LOGS", "5000");

    let result = LoggingConfig::from_env();
    assert!(result.is_ok(), "カスタムmax_logs設定のパースが失敗しました");

    let config = result.unwrap();
    assert_eq!(config.max_logs, 5000, "max_logsが5000であるべきです");
}

#[test]
fn test_logging_config_invalid_max_logs() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 無効なmax_logs（数値でない）を設定
    env_guard.set("MCP_LOGGING_BACKEND", "memory");
    env_guard.set("MCP_LOGGING_MAX_LOGS", "not_a_number");

    let result = LoggingConfig::from_env();
    assert!(
        result.is_err(),
        "無効なmax_logsが受け入れられてしまいました"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Failed to parse MCP_LOGGING_MAX_LOGS"),
        "エラーメッセージに'Failed to parse MCP_LOGGING_MAX_LOGS'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_logging_config_invalid_backend() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 無効なバックエンドタイプを設定
    env_guard.set("MCP_LOGGING_BACKEND", "invalid_backend");

    let result = LoggingConfig::from_env();
    assert!(
        result.is_err(),
        "無効なバックエンドタイプが受け入れられてしまいました"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Invalid MCP_LOGGING_BACKEND"),
        "エラーメッセージに'Invalid MCP_LOGGING_BACKEND'が含まれていません: {}",
        error_msg
    );
}

#[test]
fn test_logging_config_case_insensitive_backend() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 大文字小文字混在のバックエンド名
    env_guard.set("MCP_LOGGING_BACKEND", "MEMORY");

    let result = LoggingConfig::from_env();
    assert!(result.is_ok(), "大文字バックエンド名のパースが失敗しました");

    let config = result.unwrap();
    assert_eq!(
        config.backend,
        mcp_inspector_mcp::models::LoggingBackend::Memory,
        "大文字でもMemoryバックエンドとして認識されるべきです"
    );
}

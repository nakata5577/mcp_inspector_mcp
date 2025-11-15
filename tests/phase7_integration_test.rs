//! Phase 7 統合テスト: 環境変数ベース設定管理
//!
//! このテストスイートは、Phase 7で実装された環境変数ベースの設定管理と
//! 統合動作を検証します。
//!
//! テスト対象:
//! - 環境変数からの設定読み込み
//! - 複数サーバー設定の統合動作
//! - ログ設定とサーバー設定の統合
//! - エンドツーエンドのシナリオ

use mcp_inspector_mcp::models::{InspectorConfig, LoggingBackend};
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
// 環境変数ベース設定のテスト
// ================================================================================

#[test]
fn test_env_config_with_multiple_servers() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 複数のサーバー設定を含むJSON配列
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[
            {
                "name":"server-alpha",
                "transport":"stdio",
                "command":"alpha.exe",
                "args":["--verbose"],
                "env":{"ALPHA_MODE":"production"}
            },
            {
                "name":"server-beta",
                "transport":"stdio",
                "command":"beta.exe",
                "args":["--debug","--port=8080"],
                "env":{"BETA_KEY":"secret123"}
            },
            {
                "name":"server-gamma",
                "transport":"stdio",
                "command":"gamma.exe",
                "args":[],
                "env":{}
            }
        ]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let config = InspectorConfig::from_env().expect("複数サーバー設定の読み込み失敗");

    // すべてのサーバーが正しくロードされることを確認
    assert_eq!(config.servers.len(), 3, "3つのサーバーが設定されるべきです");

    // 1つ目のサーバー検証
    assert_eq!(config.servers[0].name, "server-alpha");
    assert_eq!(config.servers[0].params.command, "alpha.exe");
    assert_eq!(config.servers[0].params.args.len(), 1);
    assert_eq!(config.servers[0].params.args[0], "--verbose");
    assert_eq!(
        config.servers[0].params.env.get("ALPHA_MODE"),
        Some(&"production".to_string())
    );

    // 2つ目のサーバー検証
    assert_eq!(config.servers[1].name, "server-beta");
    assert_eq!(config.servers[1].params.command, "beta.exe");
    assert_eq!(config.servers[1].params.args.len(), 2);
    assert_eq!(
        config.servers[1].params.env.get("BETA_KEY"),
        Some(&"secret123".to_string())
    );

    // 3つ目のサーバー検証
    assert_eq!(config.servers[2].name, "server-gamma");
    assert_eq!(config.servers[2].params.command, "gamma.exe");
    assert!(config.servers[2].params.args.is_empty());
    assert!(config.servers[2].params.env.is_empty());
}

#[test]
fn test_env_config_complete_scenario() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // サーバー設定 + ログ設定の統合テスト
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[
            {
                "name":"production-server",
                "transport":"stdio",
                "command":"npx",
                "args":["-y","@modelcontextprotocol/server-everything"],
                "env":{"NODE_ENV":"production"}
            }
        ]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "persistent");
    env_guard.set("MCP_LOGGING_DB_PATH", "./test_integration.db");
    env_guard.set("MCP_LOGGING_MAX_LOGS", "50000");

    let config = InspectorConfig::from_env().expect("完全なシナリオ設定の読み込み失敗");

    // サーバー設定の検証
    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].name, "production-server");
    assert_eq!(config.servers[0].params.command, "npx");
    assert_eq!(config.servers[0].params.args.len(), 2);
    assert_eq!(
        config.servers[0].params.env.get("NODE_ENV"),
        Some(&"production".to_string())
    );

    // ログ設定の検証
    assert_eq!(
        config.logging.backend,
        LoggingBackend::Persistent,
        "ログバックエンドがPersistentであるべきです"
    );
    assert_eq!(
        config.logging.db_path,
        Some("./test_integration.db".to_string()),
        "DBパスが設定されているべきです"
    );
    assert_eq!(
        config.logging.max_logs, 50000,
        "max_logsが50000であるべきです"
    );
}

// ================================================================================
// エッジケースと実用シナリオのテスト
// ================================================================================

#[test]
fn test_server_with_complex_args() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 複雑な引数を持つサーバー設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[
            {
                "name":"complex-args-server",
                "transport":"stdio",
                "command":"node",
                "args":[
                    "server.js",
                    "--config=/path/to/config.json",
                    "--port=3000",
                    "--verbose",
                    "--features=feature1,feature2,feature3"
                ],
                "env":{
                    "NODE_ENV":"development",
                    "DEBUG":"app:*",
                    "API_KEY":"test-key-123"
                }
            }
        ]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");

    let config = InspectorConfig::from_env().expect("複雑な引数設定の読み込み失敗");

    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].params.args.len(), 5);
    assert_eq!(config.servers[0].params.args[0], "server.js");
    assert_eq!(
        config.servers[0].params.args[1],
        "--config=/path/to/config.json"
    );
    assert_eq!(config.servers[0].params.env.len(), 3);
}

#[test]
fn test_logging_config_integration_memory_backend() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // メモリバックエンドでの統合テスト
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[{"name":"mem-server","transport":"stdio","command":"test.exe","args":[],"env":{}}]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "memory");
    env_guard.set("MCP_LOGGING_MAX_LOGS", "20000");

    let config = InspectorConfig::from_env().expect("メモリバックエンド統合設定の読み込み失敗");

    assert_eq!(config.logging.backend, LoggingBackend::Memory);
    assert_eq!(config.logging.max_logs, 20000);
    assert!(config.logging.db_path.is_none());
}

#[test]
fn test_realistic_production_config() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 実際の本番環境に近い設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[
            {
                "name":"filesystem-server",
                "transport":"stdio",
                "command":"npx",
                "args":["-y","@modelcontextprotocol/server-filesystem","/workspace"],
                "env":{}
            },
            {
                "name":"git-server",
                "transport":"stdio",
                "command":"npx",
                "args":["-y","@modelcontextprotocol/server-git"],
                "env":{"GIT_AUTHOR_NAME":"CI Bot"}
            },
            {
                "name":"database-server",
                "transport":"stdio",
                "command":"python",
                "args":["db_server.py","--connection=postgresql://localhost/mydb"],
                "env":{"DB_PASSWORD":"secret"}
            }
        ]"#,
    );
    env_guard.set("MCP_LOGGING_BACKEND", "persistent");
    env_guard.set("MCP_LOGGING_DB_PATH", "./production_logs.db");
    env_guard.set("MCP_LOGGING_MAX_LOGS", "100000");

    let config = InspectorConfig::from_env().expect("本番設定の読み込み失敗");

    // 3つのサーバーが正しく設定されていることを確認
    assert_eq!(config.servers.len(), 3);

    // 各サーバーの名前を確認
    let server_names: Vec<&str> = config.servers.iter().map(|s| s.name.as_str()).collect();
    assert!(server_names.contains(&"filesystem-server"));
    assert!(server_names.contains(&"git-server"));
    assert!(server_names.contains(&"database-server"));

    // ログ設定の確認
    assert_eq!(config.logging.backend, LoggingBackend::Persistent);
    assert_eq!(
        config.logging.db_path,
        Some("./production_logs.db".to_string())
    );
    assert_eq!(config.logging.max_logs, 100000);
}

#[test]
fn test_minimal_valid_config() {
    let _lock = ENV_LOCK.lock().unwrap();
    let mut env_guard = EnvGuard::new();

    // 最小限の有効な設定
    env_guard.set(
        "MCP_INSPECTOR_SERVERS",
        r#"[{"name":"minimal","transport":"stdio","command":"cmd.exe","args":[],"env":{}}]"#,
    );

    let config = InspectorConfig::from_env().expect("最小設定の読み込み失敗");

    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].name, "minimal");

    // ログ設定はデフォルト値
    assert_eq!(config.logging.backend, LoggingBackend::Memory);
    assert_eq!(config.logging.max_logs, 10000);
}

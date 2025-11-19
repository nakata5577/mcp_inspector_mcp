//! Task 1.4: ExecutionConfig統合テスト
//!
//! ExecutionConfigのすべての機能をテストします：
//! - デフォルト値
//! - 環境変数からの読み込み
//! - JSONシリアライゼーション/デシリアライゼーション
//! - config.jsonと環境変数のマージ
//! - エッジケースとエラーハンドリング

use mcp_inspector_mcp::models::ExecutionConfig;
use serial_test::serial;
use std::env;

// =============================================================================
// 基本機能テスト
// =============================================================================

#[test]
fn test_default_execution_config() {
    let config = ExecutionConfig::default();

    assert_eq!(config.tool_timeout_ms, 30000, "Default tool timeout should be 30 seconds");
    assert_eq!(config.connection_timeout_ms, 5000, "Default connection timeout should be 5 seconds");
    assert_eq!(config.retry_count, 0, "Default retry count should be 0");
    assert!(!config.auto_retry_on_timeout, "Default auto_retry should be false");
}

#[test]
fn test_execution_config_json_serialization() {
    let config = ExecutionConfig {
        tool_timeout_ms: 60000,
        connection_timeout_ms: 10000,
        retry_count: 3,
        auto_retry_on_timeout: true,
        verbose: false,
    };

    // JSON化して再度デシリアライズ
    let json_str = serde_json::to_string(&config).expect("Failed to serialize ExecutionConfig");
    let deserialized: ExecutionConfig = serde_json::from_str(&json_str)
        .expect("Failed to deserialize ExecutionConfig");

    assert_eq!(deserialized.tool_timeout_ms, 60000);
    assert_eq!(deserialized.connection_timeout_ms, 10000);
    assert_eq!(deserialized.retry_count, 3);
    assert!(deserialized.auto_retry_on_timeout);
}

#[test]
fn test_execution_config_partial_json() {
    // 一部のフィールドのみ指定されたJSONからデシリアライズ
    let json = r#"{"tool_timeout_ms": 45000}"#;
    let config: ExecutionConfig = serde_json::from_str(json)
        .expect("Failed to deserialize partial JSON");

    assert_eq!(config.tool_timeout_ms, 45000);
    assert_eq!(config.connection_timeout_ms, 5000); // default
    assert_eq!(config.retry_count, 0); // default
    assert!(!config.auto_retry_on_timeout); // default
}

#[test]
fn test_execution_config_empty_json() {
    // 空のJSONオブジェクトからデシリアライズ（全てデフォルト値）
    let json = r#"{}"#;
    let config: ExecutionConfig = serde_json::from_str(json)
        .expect("Failed to deserialize empty JSON");

    assert_eq!(config.tool_timeout_ms, 30000);
    assert_eq!(config.connection_timeout_ms, 5000);
    assert_eq!(config.retry_count, 0);
    assert!(!config.auto_retry_on_timeout);
}

// =============================================================================
// 環境変数テスト
// =============================================================================

#[test]
#[serial]
fn test_from_env_with_all_vars() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // テスト用の環境変数を設定
    env::set_var("MCP_TOOL_TIMEOUT_MS", "60000");
    env::set_var("MCP_CONNECTION_TIMEOUT_MS", "10000");
    env::set_var("MCP_RETRY_COUNT", "5");
    env::set_var("MCP_AUTO_RETRY", "true");

    let config = ExecutionConfig::from_env();

    assert_eq!(config.tool_timeout_ms, 60000);
    assert_eq!(config.connection_timeout_ms, 10000);
    assert_eq!(config.retry_count, 5);
    assert!(config.auto_retry_on_timeout);

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");
}

#[test]
#[serial]
fn test_from_env_with_no_vars() {
    // 環境変数を削除してデフォルト値を確認
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    let config = ExecutionConfig::from_env();

    assert_eq!(config.tool_timeout_ms, 30000);
    assert_eq!(config.connection_timeout_ms, 5000);
    assert_eq!(config.retry_count, 0);
    assert!(!config.auto_retry_on_timeout);
}

#[test]
#[serial]
fn test_from_env_with_partial_vars() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // 一部の環境変数のみ設定
    env::set_var("MCP_TOOL_TIMEOUT_MS", "45000");

    let config = ExecutionConfig::from_env();

    assert_eq!(config.tool_timeout_ms, 45000);
    assert_eq!(config.connection_timeout_ms, 5000); // default
    assert_eq!(config.retry_count, 0); // default
    assert!(!config.auto_retry_on_timeout); // default

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
}

#[test]
#[serial]
fn test_from_env_with_invalid_values() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // 無効な値を設定した場合、デフォルト値が使用される
    env::set_var("MCP_TOOL_TIMEOUT_MS", "invalid_number");
    env::set_var("MCP_RETRY_COUNT", "not_a_number");
    env::set_var("MCP_AUTO_RETRY", "not_a_boolean");

    let config = ExecutionConfig::from_env();

    assert_eq!(config.tool_timeout_ms, 30000); // default (parse failed)
    assert_eq!(config.retry_count, 0); // default (parse failed)
    assert!(!config.auto_retry_on_timeout); // default (parse failed)

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");
}

// =============================================================================
// マージ機能テスト（config.json + 環境変数）
// =============================================================================

#[test]
#[serial]
fn test_merge_config_overrides_env() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // 環境変数を設定
    env::set_var("MCP_TOOL_TIMEOUT_MS", "45000");
    env::set_var("MCP_CONNECTION_TIMEOUT_MS", "8000");

    // config.jsonから読み込んだと仮定（明示的な値）
    let config = ExecutionConfig {
        tool_timeout_ms: 60000, // config.jsonの値
        connection_timeout_ms: 5000, // デフォルト値のまま
        retry_count: 0,
        auto_retry_on_timeout: false,
        verbose: false,
    };

    let merged = config.merge_with_env();

    // config.jsonの明示的な値が優先される
    assert_eq!(merged.tool_timeout_ms, 60000, "Config value should override env");
    // デフォルト値の場合は環境変数が優先される
    assert_eq!(merged.connection_timeout_ms, 8000, "Env should override default");

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
}

#[test]
#[serial]
fn test_merge_all_defaults_uses_env() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // 環境変数を設定
    env::set_var("MCP_TOOL_TIMEOUT_MS", "55000");
    env::set_var("MCP_CONNECTION_TIMEOUT_MS", "7000");
    env::set_var("MCP_RETRY_COUNT", "2");

    // デフォルト設定（config.jsonが空の場合）
    let config = ExecutionConfig::default();
    let merged = config.merge_with_env();

    // 全てデフォルトなので、環境変数が優先される
    assert_eq!(merged.tool_timeout_ms, 55000);
    assert_eq!(merged.connection_timeout_ms, 7000);
    assert_eq!(merged.retry_count, 2);

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
}

#[test]
#[serial]
fn test_merge_auto_retry_logic() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // auto_retry_on_timeoutは特別なロジック（論理OR）
    env::set_var("MCP_AUTO_RETRY", "false");

    let config1 = ExecutionConfig {
        tool_timeout_ms: 30000,
        connection_timeout_ms: 5000,
        retry_count: 0,
        auto_retry_on_timeout: true, // config.jsonで有効
        verbose: false,
    };

    let merged1 = config1.merge_with_env();
    assert!(merged1.auto_retry_on_timeout, "Config true should remain true");

    // 逆のケース
    env::set_var("MCP_AUTO_RETRY", "true");

    let config2 = ExecutionConfig {
        tool_timeout_ms: 30000,
        connection_timeout_ms: 5000,
        retry_count: 0,
        auto_retry_on_timeout: false, // config.jsonで無効
        verbose: false,
    };

    let merged2 = config2.merge_with_env();
    assert!(merged2.auto_retry_on_timeout, "Env true should enable auto_retry");

    // クリーンアップ
    env::remove_var("MCP_AUTO_RETRY");
}

// =============================================================================
// エッジケースとバリデーション
// =============================================================================

#[test]
fn test_execution_config_extreme_timeout_values() {
    // 極端に小さい値
    let config_small = ExecutionConfig {
        tool_timeout_ms: 1, // 1ミリ秒
        connection_timeout_ms: 1,
        retry_count: 0,
        auto_retry_on_timeout: false,
        verbose: false,
    };

    // JSON化しても値は保持される
    let json = serde_json::to_string(&config_small).unwrap();
    let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_timeout_ms, 1);
    assert_eq!(deserialized.connection_timeout_ms, 1);

    // 極端に大きい値（u64の上限に近い）
    let config_large = ExecutionConfig {
        tool_timeout_ms: 3_600_000, // 1時間
        connection_timeout_ms: 600_000, // 10分
        retry_count: 100,
        auto_retry_on_timeout: true,
        verbose: false,
    };

    let json = serde_json::to_string(&config_large).unwrap();
    let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_timeout_ms, 3_600_000);
    assert_eq!(deserialized.connection_timeout_ms, 600_000);
    assert_eq!(deserialized.retry_count, 100);
}

#[test]
fn test_execution_config_zero_timeout() {
    // タイムアウト0は技術的に有効（即座にタイムアウト）
    let config = ExecutionConfig {
        tool_timeout_ms: 0,
        connection_timeout_ms: 0,
        retry_count: 0,
        auto_retry_on_timeout: false,
        verbose: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_timeout_ms, 0);
    assert_eq!(deserialized.connection_timeout_ms, 0);
}

#[test]
fn test_execution_config_high_retry_count() {
    // 高いリトライ回数
    let config = ExecutionConfig {
        tool_timeout_ms: 30000,
        connection_timeout_ms: 5000,
        retry_count: 999,
        auto_retry_on_timeout: true,
        verbose: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.retry_count, 999);
}

// =============================================================================
// 実用的なシナリオテスト
// =============================================================================

#[test]
fn test_production_config_scenario() {
    // 本番環境での典型的な設定
    let config = ExecutionConfig {
        tool_timeout_ms: 120_000, // 2分
        connection_timeout_ms: 10_000, // 10秒
        retry_count: 3,
        auto_retry_on_timeout: true,
        verbose: false,
    };

    // 設定が正しく保存・復元できることを確認
    let json = serde_json::to_string_pretty(&config).unwrap();
    let restored: ExecutionConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.tool_timeout_ms, 120_000);
    assert_eq!(restored.connection_timeout_ms, 10_000);
    assert_eq!(restored.retry_count, 3);
    assert!(restored.auto_retry_on_timeout);
}

#[test]
fn test_development_config_scenario() {
    // 開発環境での典型的な設定（短いタイムアウト）
    let config = ExecutionConfig {
        tool_timeout_ms: 5_000, // 5秒
        connection_timeout_ms: 1_000, // 1秒
        retry_count: 0,
        auto_retry_on_timeout: false,
        verbose: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let restored: ExecutionConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.tool_timeout_ms, 5_000);
    assert_eq!(restored.connection_timeout_ms, 1_000);
    assert_eq!(restored.retry_count, 0);
    assert!(!restored.auto_retry_on_timeout);
}

#[test]
#[serial]
fn test_config_priority_chain() {
    // 全ての環境変数をクリーンアップしてからテスト開始
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
    env::remove_var("MCP_CONNECTION_TIMEOUT_MS");
    env::remove_var("MCP_RETRY_COUNT");
    env::remove_var("MCP_AUTO_RETRY");

    // 設定の優先順位チェーン: config.json > 環境変数 > デフォルト

    // Step 1: デフォルト
    let default_config = ExecutionConfig::default();
    assert_eq!(default_config.tool_timeout_ms, 30000);

    // Step 2: 環境変数で上書き
    env::set_var("MCP_TOOL_TIMEOUT_MS", "50000");
    let env_config = ExecutionConfig::from_env();
    assert_eq!(env_config.tool_timeout_ms, 50000);

    // Step 3: config.jsonで上書き（merge_with_env）
    let json_config = ExecutionConfig {
        tool_timeout_ms: 70000,
        connection_timeout_ms: 5000,
        retry_count: 0,
        auto_retry_on_timeout: false,
        verbose: false,
    };
    let final_config = json_config.merge_with_env();
    assert_eq!(final_config.tool_timeout_ms, 70000, "Config.json should have highest priority");

    // クリーンアップ
    env::remove_var("MCP_TOOL_TIMEOUT_MS");
}

// =============================================================================
// Clone/Debugトレイトのテスト
// =============================================================================

#[test]
fn test_execution_config_clone() {
    let config1 = ExecutionConfig {
        tool_timeout_ms: 40000,
        connection_timeout_ms: 6000,
        retry_count: 2,
        auto_retry_on_timeout: true,
        verbose: false,
    };

    let config2 = config1.clone();

    assert_eq!(config1.tool_timeout_ms, config2.tool_timeout_ms);
    assert_eq!(config1.connection_timeout_ms, config2.connection_timeout_ms);
    assert_eq!(config1.retry_count, config2.retry_count);
    assert_eq!(config1.auto_retry_on_timeout, config2.auto_retry_on_timeout);
}

#[test]
fn test_execution_config_debug() {
    let config = ExecutionConfig::default();
    let debug_str = format!("{:?}", config);

    // Debug出力に主要フィールドが含まれていることを確認
    assert!(debug_str.contains("tool_timeout_ms"));
    assert!(debug_str.contains("connection_timeout_ms"));
    assert!(debug_str.contains("retry_count"));
    assert!(debug_str.contains("auto_retry_on_timeout"));
}

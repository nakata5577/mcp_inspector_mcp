//! Phase 5 統合テスト: ログ永続化機能
//!
//! このテストでは、以下を検証します：
//! - Memory backendでの動作
//! - Persistent backendでの動作
//! - 設定ファイルからの初期化
//! - バックエンド切替
//! - サーバー再起動後のログ保持

use anyhow::Result;
use mcp_inspector_mcp::models::{
    LoggingBackend, LoggingConfig, SamplingContent, SamplingLogEntry, SamplingMessage,
    SamplingStatus,
};
use mcp_inspector_mcp::services::create_logger;
use tempfile::TempDir;

/// ヘルパー関数: テスト用のログエントリを作成
fn create_test_entry(server: &str, timestamp: &str, status: SamplingStatus) -> SamplingLogEntry {
    SamplingLogEntry {
        id: format!("{}:{}", server, timestamp),
        timestamp: timestamp.to_string(),
        status,
        messages: vec![SamplingMessage {
            role: "user".to_string(),
            content: SamplingContent {
                content_type: "text".to_string(),
                text: Some("test message".to_string()),
            },
        }],
        model_preferences: None,
        system_prompt: None,
        max_tokens: Some(100),
        error: None,
        response: Some("test response".to_string()),
    }
}

#[test]
fn test_memory_backend_integration() -> Result<()> {
    // Memory backendの設定
    let config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    // Loggerを作成
    let logger = create_logger(&config)?;

    // ログを追加
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;

    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;

    // ログを取得
    let logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(logs.len(), 2);

    // ステータスフィルタ
    let success_logs = logger.get_logs("test_server", 10, "success")?;
    assert_eq!(success_logs.len(), 1);

    let failed_logs = logger.get_logs("test_server", 10, "failed")?;
    assert_eq!(failed_logs.len(), 1);

    // カウント
    assert_eq!(logger.count_logs("test_server")?, 2);

    Ok(())
}

#[test]
fn test_persistent_backend_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    // Persistent backendの設定
    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };

    // Loggerを作成
    let logger = create_logger(&config)?;

    // ログを追加
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;

    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;

    // ログを取得
    let logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(logs.len(), 2);

    // カウント
    assert_eq!(logger.count_logs("test_server")?, 2);

    Ok(())
}

#[test]
fn test_persistence_across_restarts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };

    // 最初のインスタンス: ログを追加
    {
        let logger = create_logger(&config)?;
        logger.add_log(create_test_entry(
            "test_server",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ))?;
        logger.add_log(create_test_entry(
            "test_server",
            "2025-01-15T12:01:00Z",
            SamplingStatus::Failed,
        ))?;
    }

    // 2番目のインスタンス: ログが保持されていることを確認
    {
        let logger = create_logger(&config)?;
        let logs = logger.get_logs("test_server", 10, "all")?;
        assert_eq!(logs.len(), 2, "Logs should persist across restarts");

        // 追加のログを書き込み
        logger.add_log(create_test_entry(
            "test_server",
            "2025-01-15T12:02:00Z",
            SamplingStatus::Success,
        ))?;
    }

    // 3番目のインスタンス: 合計3件のログがあることを確認
    {
        let logger = create_logger(&config)?;
        let logs = logger.get_logs("test_server", 10, "all")?;
        assert_eq!(logs.len(), 3, "All logs should persist");
    }

    Ok(())
}

#[test]
fn test_backend_switching() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    // Memory backendでログを追加
    let memory_config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };
    let memory_logger = create_logger(&memory_config)?;
    memory_logger.add_log(create_test_entry(
        "server1",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;

    // Persistent backendに切り替え
    let persistent_config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };
    let persistent_logger = create_logger(&persistent_config)?;
    persistent_logger.add_log(create_test_entry(
        "server1",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;

    // Memory loggerのログは1件のまま
    assert_eq!(memory_logger.count_logs("server1")?, 1);

    // Persistent loggerのログは1件（別のストレージ）
    assert_eq!(persistent_logger.count_logs("server1")?, 1);

    Ok(())
}

#[test]
fn test_log_rotation_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    // 小さいmax_logsで設定
    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 3,
    };

    let logger = create_logger(&config)?;

    // 5件のログを追加（max_logsを超過）
    for i in 0..5 {
        logger.add_log(create_test_entry(
            "test_server",
            &format!("2025-01-15T12:0{}:00Z", i),
            SamplingStatus::Success,
        ))?;
    }

    // 最新の3件のみが保持されていることを確認
    let logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(logs.len(), 3, "Should keep only max_logs entries");

    // 最新のログが保持されていることを確認
    let timestamps: Vec<String> = logs.iter().map(|l| l.timestamp.clone()).collect();
    assert!(timestamps.contains(&"2025-01-15T12:02:00Z".to_string()));
    assert!(timestamps.contains(&"2025-01-15T12:03:00Z".to_string()));
    assert!(timestamps.contains(&"2025-01-15T12:04:00Z".to_string()));

    // 古いログは削除されている
    assert!(!timestamps.contains(&"2025-01-15T12:00:00Z".to_string()));
    assert!(!timestamps.contains(&"2025-01-15T12:01:00Z".to_string()));

    Ok(())
}

#[test]
fn test_multiple_servers() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 異なるサーバーのログを追加
    logger.add_log(create_test_entry(
        "server1",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;
    logger.add_log(create_test_entry(
        "server2",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;
    logger.add_log(create_test_entry(
        "server1",
        "2025-01-15T12:02:00Z",
        SamplingStatus::Success,
    ))?;

    // サーバーごとのログを確認
    assert_eq!(logger.count_logs("server1")?, 2);
    assert_eq!(logger.count_logs("server2")?, 1);

    let server1_logs = logger.get_logs("server1", 10, "all")?;
    assert_eq!(server1_logs.len(), 2);

    let server2_logs = logger.get_logs("server2", 10, "all")?;
    assert_eq!(server2_logs.len(), 1);

    Ok(())
}

#[test]
fn test_config_validation() {
    // Persistent backendでdb_pathがない場合
    let invalid_config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: None,
        max_logs: 100,
    };

    let result = create_logger(&invalid_config);
    assert!(
        result.is_err(),
        "Should fail without db_path for persistent backend"
    );

    // Memory backendではdb_pathがなくてもOK
    let valid_config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    let result = create_logger(&valid_config);
    assert!(result.is_ok(), "Memory backend should work without db_path");
}

#[test]
fn test_memory_backend_status_filtering() -> Result<()> {
    let config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 複数のステータスのログを追加
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:02:00Z",
        SamplingStatus::Pending,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:03:00Z",
        SamplingStatus::Success,
    ))?;

    // ステータスフィルタのテスト
    let all_logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(all_logs.len(), 4, "Should return all logs");

    let success_logs = logger.get_logs("test_server", 10, "success")?;
    assert_eq!(success_logs.len(), 2, "Should return only success logs");

    let failed_logs = logger.get_logs("test_server", 10, "failed")?;
    assert_eq!(failed_logs.len(), 1, "Should return only failed logs");

    // Note: "pending" filter is not explicitly supported in the current implementation,
    // so it falls through to the default case which returns all logs
    let pending_logs = logger.get_logs("test_server", 10, "pending")?;
    assert_eq!(
        pending_logs.len(),
        4,
        "Unsupported status filter returns all logs"
    );

    Ok(())
}

#[test]
fn test_persistent_backend_status_filtering() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 複数のステータスのログを追加
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:00:00Z",
        SamplingStatus::Success,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:01:00Z",
        SamplingStatus::Failed,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:02:00Z",
        SamplingStatus::Pending,
    ))?;
    logger.add_log(create_test_entry(
        "test_server",
        "2025-01-15T12:03:00Z",
        SamplingStatus::Success,
    ))?;

    // ステータスフィルタのテスト
    let all_logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(all_logs.len(), 4, "Should return all logs");

    let success_logs = logger.get_logs("test_server", 10, "success")?;
    assert_eq!(success_logs.len(), 2, "Should return only success logs");

    let failed_logs = logger.get_logs("test_server", 10, "failed")?;
    assert_eq!(failed_logs.len(), 1, "Should return only failed logs");

    // Note: "pending" filter is not explicitly supported in the current implementation,
    // so it falls through to the default case which returns all logs
    let pending_logs = logger.get_logs("test_server", 10, "pending")?;
    assert_eq!(
        pending_logs.len(),
        4,
        "Unsupported status filter returns all logs"
    );

    Ok(())
}

#[test]
fn test_memory_backend_limit() -> Result<()> {
    let config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 10件のログを追加
    for i in 0..10 {
        logger.add_log(create_test_entry(
            "test_server",
            &format!("2025-01-15T12:{:02}:00Z", i),
            SamplingStatus::Success,
        ))?;
    }

    // limitパラメータのテスト
    let logs_5 = logger.get_logs("test_server", 5, "all")?;
    assert_eq!(logs_5.len(), 5, "Should return only 5 logs");

    let logs_3 = logger.get_logs("test_server", 3, "all")?;
    assert_eq!(logs_3.len(), 3, "Should return only 3 logs");

    Ok(())
}

#[test]
fn test_persistent_backend_limit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_db");

    let config = LoggingConfig {
        backend: LoggingBackend::Persistent,
        db_path: Some(db_path.to_str().unwrap().to_string()),
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 10件のログを追加
    for i in 0..10 {
        logger.add_log(create_test_entry(
            "test_server",
            &format!("2025-01-15T12:{:02}:00Z", i),
            SamplingStatus::Success,
        ))?;
    }

    // limitパラメータのテスト
    let logs_5 = logger.get_logs("test_server", 5, "all")?;
    assert_eq!(logs_5.len(), 5, "Should return only 5 logs");

    let logs_3 = logger.get_logs("test_server", 3, "all")?;
    assert_eq!(logs_3.len(), 3, "Should return only 3 logs");

    Ok(())
}

#[test]
fn test_empty_server_logs() -> Result<()> {
    let config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 存在しないサーバーのログを取得
    let logs = logger.get_logs("nonexistent_server", 10, "all")?;
    assert_eq!(
        logs.len(),
        0,
        "Should return empty list for nonexistent server"
    );

    assert_eq!(
        logger.count_logs("nonexistent_server")?,
        0,
        "Count should be 0 for nonexistent server"
    );

    Ok(())
}

#[test]
fn test_log_entry_structure() -> Result<()> {
    let config = LoggingConfig {
        backend: LoggingBackend::Memory,
        db_path: None,
        max_logs: 100,
    };

    let logger = create_logger(&config)?;

    // 詳細なログエントリを追加
    let entry = SamplingLogEntry {
        id: "test_server:2025-01-15T12:00:00Z".to_string(),
        timestamp: "2025-01-15T12:00:00Z".to_string(),
        status: SamplingStatus::Success,
        messages: vec![
            SamplingMessage {
                role: "user".to_string(),
                content: SamplingContent {
                    content_type: "text".to_string(),
                    text: Some("Hello".to_string()),
                },
            },
            SamplingMessage {
                role: "assistant".to_string(),
                content: SamplingContent {
                    content_type: "text".to_string(),
                    text: Some("Hi there".to_string()),
                },
            },
        ],
        model_preferences: None,
        system_prompt: Some("You are helpful".to_string()),
        max_tokens: Some(1000),
        error: None,
        response: Some("Response text".to_string()),
    };

    logger.add_log(entry.clone())?;

    let logs = logger.get_logs("test_server", 10, "all")?;
    assert_eq!(logs.len(), 1);

    let retrieved = &logs[0];
    assert_eq!(retrieved.id, entry.id);
    assert_eq!(retrieved.timestamp, entry.timestamp);
    assert_eq!(retrieved.status, entry.status);
    assert_eq!(retrieved.messages.len(), 2);
    assert_eq!(retrieved.system_prompt, Some("You are helpful".to_string()));
    assert_eq!(retrieved.max_tokens, Some(1000));
    assert_eq!(retrieved.response, Some("Response text".to_string()));

    Ok(())
}

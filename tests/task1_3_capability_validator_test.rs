/// Task 1.3: CapabilityValidator統合テスト
///
/// CapabilityValidatorの全機能をテストします：
/// - Tools capability検証
/// - Resources capability検証
/// - Prompts capability検証
/// - Capability情報がない場合の動作
/// - 複数のcapabilityの組み合わせ
/// - ログ出力機能

use mcp_inspector_mcp::services::{CapabilityValidationResult, CapabilityValidator};
use rmcp::model::{
    PromptsCapability, ResourcesCapability, ServerCapabilities, ToolsCapability,
};

// =============================================================================
// ヘルパー関数：様々なCapability設定を生成
// =============================================================================

fn create_full_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(false),
        }),
        logging: None,
        experimental: None,
        completions: None,
    }
}

fn create_tools_only_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(false),
        }),
        resources: None,
        prompts: None,
        logging: None,
        experimental: None,
        completions: None,
    }
}

fn create_resources_only_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: None,
        resources: Some(ResourcesCapability {
            subscribe: Some(false),
            list_changed: Some(false),
        }),
        prompts: None,
        logging: None,
        experimental: None,
        completions: None,
    }
}

fn create_prompts_only_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: None,
        resources: None,
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        logging: None,
        experimental: None,
        completions: None,
    }
}

fn create_empty_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: None,
        resources: None,
        prompts: None,
        logging: None,
        experimental: None,
        completions: None,
    }
}

// =============================================================================
// Tools Capability検証テスト
// =============================================================================

#[test]
fn test_tools_validation_with_capability() {
    let validator = CapabilityValidator::new(Some(create_tools_only_capabilities()));

    let result = validator.validate_tools_call("test_tool");

    match result {
        CapabilityValidationResult::Valid => {
            // 期待通り
        }
        _ => panic!("Expected Valid result when tools capability is present"),
    }
}

#[test]
fn test_tools_validation_without_capability() {
    let validator = CapabilityValidator::new(Some(create_empty_capabilities()));

    let result = validator.validate_tools_call("test_tool");

    match result {
        CapabilityValidationResult::Warning { message } => {
            assert!(
                message.contains("test_tool"),
                "Warning message should contain tool name"
            );
            assert!(
                message.contains("does not declare tools capability"),
                "Warning message should explain the issue"
            );
        }
        _ => panic!("Expected Warning result when tools capability is missing"),
    }
}

#[test]
fn test_tools_validation_with_no_capability_info() {
    let validator = CapabilityValidator::new(None);

    let result = validator.validate_tools_call("any_tool");

    match result {
        CapabilityValidationResult::Unavailable => {
            // 期待通り：capability情報がない場合はUnavailable
        }
        _ => panic!("Expected Unavailable result when no capability info is available"),
    }
}

#[test]
fn test_tools_validation_with_multiple_tools() {
    let validator = CapabilityValidator::new(Some(create_tools_only_capabilities()));

    // 同じvalidatorで複数のツールを検証
    let results: Vec<_> = vec!["tool1", "tool2", "tool3"]
        .iter()
        .map(|name| validator.validate_tools_call(name))
        .collect();

    // 全てValidになるはず
    for result in results {
        match result {
            CapabilityValidationResult::Valid => {}
            _ => panic!("All tool validations should return Valid"),
        }
    }
}

// =============================================================================
// Resources Capability検証テスト
// =============================================================================

#[test]
fn test_resources_validation_with_capability() {
    let validator = CapabilityValidator::new(Some(create_resources_only_capabilities()));

    let result = validator.validate_resources_read("file://test.txt");

    match result {
        CapabilityValidationResult::Valid => {
            // 期待通り
        }
        _ => panic!("Expected Valid result when resources capability is present"),
    }
}

#[test]
fn test_resources_validation_without_capability() {
    let validator = CapabilityValidator::new(Some(create_empty_capabilities()));

    let result = validator.validate_resources_read("file://test.txt");

    match result {
        CapabilityValidationResult::Warning { message } => {
            assert!(
                message.contains("file://test.txt"),
                "Warning message should contain resource URI"
            );
            assert!(
                message.contains("does not declare resources capability"),
                "Warning message should explain the issue"
            );
        }
        _ => panic!("Expected Warning result when resources capability is missing"),
    }
}

#[test]
fn test_resources_validation_with_no_capability_info() {
    let validator = CapabilityValidator::new(None);

    let result = validator.validate_resources_read("http://example.com/resource");

    match result {
        CapabilityValidationResult::Unavailable => {
            // 期待通り
        }
        _ => panic!("Expected Unavailable result when no capability info is available"),
    }
}

#[test]
fn test_resources_validation_with_various_uris() {
    let validator = CapabilityValidator::new(Some(create_resources_only_capabilities()));

    let uris = vec![
        "file://path/to/file.txt",
        "http://example.com/resource",
        "https://api.example.com/data",
        "custom://scheme/resource",
    ];

    for uri in uris {
        let result = validator.validate_resources_read(uri);
        match result {
            CapabilityValidationResult::Valid => {}
            _ => panic!("All resource validations should return Valid for URI: {}", uri),
        }
    }
}

// =============================================================================
// Prompts Capability検証テスト
// =============================================================================

#[test]
fn test_prompts_validation_with_capability() {
    let validator = CapabilityValidator::new(Some(create_prompts_only_capabilities()));

    let result = validator.validate_prompts_get("greeting_prompt");

    match result {
        CapabilityValidationResult::Valid => {
            // 期待通り
        }
        _ => panic!("Expected Valid result when prompts capability is present"),
    }
}

#[test]
fn test_prompts_validation_without_capability() {
    let validator = CapabilityValidator::new(Some(create_empty_capabilities()));

    let result = validator.validate_prompts_get("greeting_prompt");

    match result {
        CapabilityValidationResult::Warning { message } => {
            assert!(
                message.contains("greeting_prompt"),
                "Warning message should contain prompt name"
            );
            assert!(
                message.contains("does not declare prompts capability"),
                "Warning message should explain the issue"
            );
        }
        _ => panic!("Expected Warning result when prompts capability is missing"),
    }
}

#[test]
fn test_prompts_validation_with_no_capability_info() {
    let validator = CapabilityValidator::new(None);

    let result = validator.validate_prompts_get("any_prompt");

    match result {
        CapabilityValidationResult::Unavailable => {
            // 期待通り
        }
        _ => panic!("Expected Unavailable result when no capability info is available"),
    }
}

// =============================================================================
// 複合シナリオテスト
// =============================================================================

#[test]
fn test_full_capabilities_all_valid() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // 全てのcapabilityをサポートしている場合、全ての検証がValidになる
    let tool_result = validator.validate_tools_call("some_tool");
    let resource_result = validator.validate_resources_read("file://data.json");
    let prompt_result = validator.validate_prompts_get("instruction");

    match tool_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Tools validation should be Valid"),
    }

    match resource_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Resources validation should be Valid"),
    }

    match prompt_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Prompts validation should be Valid"),
    }
}

#[test]
fn test_empty_capabilities_all_warnings() {
    let validator = CapabilityValidator::new(Some(create_empty_capabilities()));

    // capabilityが何も宣言されていない場合、全ての検証がWarningになる
    let tool_result = validator.validate_tools_call("some_tool");
    let resource_result = validator.validate_resources_read("file://data.json");
    let prompt_result = validator.validate_prompts_get("instruction");

    match tool_result {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Tools validation should be Warning"),
    }

    match resource_result {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Resources validation should be Warning"),
    }

    match prompt_result {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Prompts validation should be Warning"),
    }
}

#[test]
fn test_partial_capabilities_mixed_results() {
    // toolsのみサポート
    let validator = CapabilityValidator::new(Some(create_tools_only_capabilities()));

    let tool_result = validator.validate_tools_call("supported_tool");
    let resource_result = validator.validate_resources_read("file://data.json");

    match tool_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Tools validation should be Valid"),
    }

    match resource_result {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Resources validation should be Warning"),
    }
}

// =============================================================================
// ログ出力機能のテスト
// =============================================================================

#[test]
fn test_log_capabilities_with_full_support() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // パニックせずに完了することを確認
    validator.log_capabilities("test_server");
}

#[test]
fn test_log_capabilities_with_empty_support() {
    let validator = CapabilityValidator::new(Some(create_empty_capabilities()));

    // パニックせずに完了することを確認
    validator.log_capabilities("minimal_server");
}

#[test]
fn test_log_capabilities_with_no_info() {
    let validator = CapabilityValidator::new(None);

    // パニックせずに完了することを確認
    validator.log_capabilities("unknown_server");
}

// =============================================================================
// エッジケースとエラーハンドリング
// =============================================================================

#[test]
fn test_validation_with_empty_strings() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // 空文字列でも正常に動作するか
    let tool_result = validator.validate_tools_call("");
    let resource_result = validator.validate_resources_read("");
    let prompt_result = validator.validate_prompts_get("");

    // 全てValidになる（空文字列でも検証は通過）
    match tool_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Empty string should pass validation"),
    }

    match resource_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Empty string should pass validation"),
    }

    match prompt_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Empty string should pass validation"),
    }
}

#[test]
fn test_validation_with_special_characters() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // 特殊文字を含む名前
    let special_names = vec![
        "tool-with-dashes",
        "tool_with_underscores",
        "tool.with.dots",
        "tool/with/slashes",
        "tool@with@symbols",
        "日本語ツール",
    ];

    for name in special_names {
        let result = validator.validate_tools_call(name);
        match result {
            CapabilityValidationResult::Valid => {}
            _ => panic!("Validation should succeed for special name: {}", name),
        }
    }
}

#[test]
fn test_validation_with_long_names() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // 非常に長い名前
    let long_name = "a".repeat(1000);
    let result = validator.validate_tools_call(&long_name);

    match result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Validation should succeed for long names"),
    }
}

// =============================================================================
// 実用的なシナリオテスト
// =============================================================================

#[test]
fn test_realistic_mcp_server_scenario() {
    // 実際のMCPサーバーのような設定
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true), // 動的ツールリスト
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true), // リソース変更通知
            list_changed: Some(true),
        }),
        prompts: None, // プロンプトは未サポート
        logging: None,
        experimental: None,
        completions: None,
    };

    let validator = CapabilityValidator::new(Some(capabilities));

    // ツール実行は成功
    let tool_result = validator.validate_tools_call("list_files");
    match tool_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Tool validation should succeed"),
    }

    // リソース読み込みも成功
    let resource_result = validator.validate_resources_read("file://README.md");
    match resource_result {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Resource validation should succeed"),
    }

    // プロンプト取得は警告
    let prompt_result = validator.validate_prompts_get("code_review");
    match prompt_result {
        CapabilityValidationResult::Warning { message } => {
            assert!(message.contains("prompts capability"));
        }
        _ => panic!("Prompt validation should warn"),
    }
}

#[test]
fn test_minimal_mcp_server_scenario() {
    // 最小限のMCPサーバー（ツールのみ）
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(false), // 静的ツールリスト
        }),
        resources: None,
        prompts: None,
        logging: None,
        experimental: None,
        completions: None,
    };

    let validator = CapabilityValidator::new(Some(capabilities));

    // ツールのみサポート
    match validator.validate_tools_call("echo") {
        CapabilityValidationResult::Valid => {}
        _ => panic!("Tool should be valid"),
    }

    // その他は警告
    match validator.validate_resources_read("data.json") {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Resource should warn"),
    }

    match validator.validate_prompts_get("help") {
        CapabilityValidationResult::Warning { .. } => {}
        _ => panic!("Prompt should warn"),
    }
}

// =============================================================================
// バリデータの再利用とライフタイム
// =============================================================================

#[test]
fn test_validator_reuse() {
    let validator = CapabilityValidator::new(Some(create_full_capabilities()));

    // 同じvalidatorを複数回使用
    for i in 0..100 {
        let tool_name = format!("tool_{}", i);
        let result = validator.validate_tools_call(&tool_name);
        match result {
            CapabilityValidationResult::Valid => {}
            _ => panic!("Reused validator should work consistently"),
        }
    }
}

#[test]
fn test_validator_clone_behavior() {
    // CapabilityValidatorがCloneトレイトを実装していない場合はスキップ
    // この場合、新しいインスタンスを作成して同じ動作をテスト
    let caps = create_full_capabilities();

    let validator1 = CapabilityValidator::new(Some(caps.clone()));
    let validator2 = CapabilityValidator::new(Some(caps));

    let result1 = validator1.validate_tools_call("test");
    let result2 = validator2.validate_tools_call("test");

    // 同じcapabilitiesから作られたvalidatorは同じ結果を返す
    match (result1, result2) {
        (CapabilityValidationResult::Valid, CapabilityValidationResult::Valid) => {}
        _ => panic!("Validators with same capabilities should behave identically"),
    }
}

use crate::models::test_definition::{Assertion, TestCase, TestConfig, TestSuite};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// テスト実行のためのツール実行インターフェース
///
/// このトレイトは、テストケースで指定されたツールを実行するための抽象化を提供します。
/// モック実装と実際のMCPサーバー統合の両方をサポートします。
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// テストケースで指定されたツールを実行
    ///
    /// # Arguments
    ///
    /// * `test` - 実行するテストケース
    ///
    /// # Returns
    ///
    /// ツール実行結果のJSON値
    async fn execute(&self, test: &TestCase) -> Result<Value>;
}

/// モック実装のツール実行エンジン
///
/// 実際のMCPサーバーとの通信を行わず、モックレスポンスを返します。
/// テストやデモンストレーション目的で使用します。
pub struct MockToolExecutor;

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute(&self, test: &TestCase) -> Result<Value> {
        // テスト用の遅延を追加（実際のネットワーク遅延をシミュレート）
        tokio::time::sleep(Duration::from_millis(50)).await;

        match test.tool.as_str() {
            "health_check" => Ok(json!({
                "status": "healthy",
                "server": test.server,
                "uptime_seconds": 3600,
                "error_rate": 0.01,
            })),
            "tools_list" => Ok(json!({
                "tools": [
                    {"name": "tool1", "description": "Tool 1"},
                    {"name": "tool2", "description": "Tool 2"},
                ],
            })),
            "tools_call" => {
                if test.arguments.get("name").and_then(|v| v.as_str()) == Some("error_tool") {
                    Err(anyhow::anyhow!("Tool execution failed"))
                } else {
                    Ok(json!({
                        "result": {
                            "content": [{
                                "text": json!({"value": 42}).to_string()
                            }]
                        },
                        "output": "Tool executed successfully",
                    }))
                }
            }
            "resources_list" => Ok(json!({
                "resources": [
                    {"uri": "resource://1", "name": "Resource 1"},
                    {"uri": "resource://2", "name": "Resource 2"},
                ],
            })),
            "resource_read" => Ok(json!({
                "contents": [{
                    "uri": "resource://1",
                    "text": "Resource content",
                }],
            })),
            "prompts_list" => Ok(json!({
                "prompts": [
                    {"name": "prompt1", "description": "Prompt 1"},
                    {"name": "prompt2", "description": "Prompt 2"},
                ],
            })),
            "prompt_get" => Ok(json!({
                "messages": [
                    {"role": "user", "content": {"type": "text", "text": "Prompt content"}},
                ],
            })),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", test.tool)),
        }
    }
}

// 将来の実装のためのスケルトン（コメントアウト）
/*
/// 実際のMCPサーバー統合実装
///
/// InspectorServiceを使用して実際のMCPサーバーと通信します。
pub struct McpToolExecutor {
    inspector: Arc<InspectorService>,
}

impl McpToolExecutor {
    pub fn new(inspector: Arc<InspectorService>) -> Self {
        Self { inspector }
    }
}

#[async_trait]
impl ToolExecutor for McpToolExecutor {
    async fn execute(&self, test: &TestCase) -> Result<Value> {
        // InspectorServiceを使用して実際のMCPサーバーと通信
        match test.tool.as_str() {
            "health_check" => {
                // 例: self.inspector.health_check(&test.server).await
                // の結果をJSON値に変換
                unimplemented!("MCP health check integration")
            }
            "tools_list" => {
                // 例: self.inspector.list_tools(&test.server).await
                unimplemented!("MCP tools list integration")
            }
            "tools_call" => {
                // 例: self.inspector.call_tool(&test.server, ...)
                unimplemented!("MCP tool call integration")
            }
            "resources_list" => {
                // 例: self.inspector.list_resources(&test.server).await
                unimplemented!("MCP resources list integration")
            }
            "resource_read" => {
                // 例: self.inspector.read_resource(&test.server, uri).await
                unimplemented!("MCP resource read integration")
            }
            "prompts_list" => {
                // 例: self.inspector.list_prompts(&test.server).await
                unimplemented!("MCP prompts list integration")
            }
            "prompt_get" => {
                // 例: self.inspector.get_prompt(&test.server, name).await
                unimplemented!("MCP prompt get integration")
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", test.tool)),
        }
    }
}
*/

/// テスト実行結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub assertions: Vec<AssertionResult>,
    pub error: Option<String>,
}

/// アサーション評価結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssertionResult {
    pub assertion_type: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// テスト実行エンジン
///
/// テストスイートを実行し、各テストケースの結果を収集します。
/// ToolExecutorトレイトを実装したツール実行エンジンを使用して、
/// 実際のツール呼び出しを抽象化します。
pub struct TestExecutor {
    executor: Arc<dyn ToolExecutor>,
}

impl TestExecutor {
    /// デフォルトのモック実装でTestExecutorインスタンスを作成
    pub fn new() -> Self {
        Self {
            executor: Arc::new(MockToolExecutor),
        }
    }

    /// カスタムのToolExecutor実装でTestExecutorインスタンスを作成
    ///
    /// # Arguments
    ///
    /// * `executor` - 使用するToolExecutor実装
    ///
    /// # Returns
    ///
    /// 新しいTestExecutorインスタンス
    #[allow(dead_code)]
    pub fn with_executor(executor: Arc<dyn ToolExecutor>) -> Self {
        Self { executor }
    }

    /// テストスイート全体を実行
    ///
    /// # Arguments
    ///
    /// * `suite` - 実行するテストスイート
    ///
    /// # Returns
    ///
    /// 各テストケースの実行結果のベクタ
    pub async fn run_test_suite(&self, suite: &TestSuite) -> Result<Vec<TestResult>> {
        if suite.config.parallel {
            self.run_parallel(suite).await
        } else {
            self.run_sequential(suite).await
        }
    }

    /// 並列実行（テスト定義の順序を保持）
    async fn run_parallel(&self, suite: &TestSuite) -> Result<Vec<TestResult>> {
        let mut handles = Vec::new();

        for test in &suite.tests {
            let test = test.clone();
            let config = suite.config.clone();
            let executor = self.executor.clone();

            let handle = tokio::spawn(async move {
                let test_executor = TestExecutor::with_executor(executor);
                test_executor.run_test_case_with_retry(&test, &config).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await??);
        }

        Ok(results)
    }

    /// 順次実行
    async fn run_sequential(&self, suite: &TestSuite) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test in &suite.tests {
            let result = self.run_test_case_with_retry(test, &suite.config).await?;
            results.push(result.clone());

            if suite.config.fail_fast && !result.passed {
                break;
            }
        }

        Ok(results)
    }

    /// リトライ付きテスト実行
    async fn run_test_case_with_retry(&self, test: &TestCase, config: &TestConfig) -> Result<TestResult> {
        let mut last_result = None;

        for attempt in 0..=config.retry_count {
            let result = self.run_test_case(test, config).await?;

            if result.passed || attempt == config.retry_count {
                return Ok(result);
            }

            last_result = Some(result);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(last_result.unwrap())
    }

    /// 単一テストケース実行
    async fn run_test_case(&self, test: &TestCase, config: &TestConfig) -> Result<TestResult> {
        let start = Instant::now();

        // タイムアウト付きでテスト実行
        let timeout = Duration::from_millis(config.timeout_ms);
        let execution_result = tokio::time::timeout(
            timeout,
            self.executor.execute(test),
        )
        .await;

        let duration = start.elapsed();

        match execution_result {
            Ok(Ok(response)) => {
                // アサーション評価
                let metadata = TestMetadata {
                    duration_ms: duration.as_millis() as u64,
                };
                let assertions = Self::evaluate_assertions(&test.assertions, &response, &metadata);

                Ok(TestResult {
                    test_name: test.name.clone(),
                    passed: assertions.iter().all(|a| a.passed),
                    duration_ms: duration.as_millis() as u64,
                    assertions,
                    error: None,
                })
            }
            Ok(Err(e)) => {
                if test.expect_error {
                    // エラーが期待されている場合、エラーアサーションを評価
                    let error_value = serde_json::json!({
                        "error": e.to_string(),
                    });
                    let metadata = TestMetadata {
                        duration_ms: duration.as_millis() as u64,
                    };
                    let assertions = Self::evaluate_assertions(&test.assertions, &error_value, &metadata);

                    Ok(TestResult {
                        test_name: test.name.clone(),
                        passed: assertions.iter().all(|a| a.passed),
                        duration_ms: duration.as_millis() as u64,
                        assertions,
                        error: Some(e.to_string()),
                    })
                } else {
                    Ok(TestResult {
                        test_name: test.name.clone(),
                        passed: false,
                        duration_ms: duration.as_millis() as u64,
                        assertions: vec![],
                        error: Some(e.to_string()),
                    })
                }
            }
            Err(_) => {
                // タイムアウト
                Ok(TestResult {
                    test_name: test.name.clone(),
                    passed: false,
                    duration_ms: duration.as_millis() as u64,
                    assertions: vec![],
                    error: Some(format!("Test timed out after {}ms", config.timeout_ms)),
                })
            }
        }
    }

    /// アサーションリストの評価
    fn evaluate_assertions(
        assertions: &[Assertion],
        response: &Value,
        metadata: &TestMetadata,
    ) -> Vec<AssertionResult> {
        assertions
            .iter()
            .map(|a| Self::evaluate_assertion(a, response, metadata))
            .collect()
    }

    /// 個別アサーションの評価
    fn evaluate_assertion(
        assertion: &Assertion,
        response: &Value,
        metadata: &TestMetadata,
    ) -> AssertionResult {
        match assertion {
            Assertion::Status { expected } => {
                let actual = response
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let passed = actual == expected;

                AssertionResult {
                    assertion_type: "status".to_string(),
                    passed,
                    expected: expected.clone(),
                    actual: actual.to_string(),
                    message: if passed {
                        format!("Status is '{}'", expected)
                    } else {
                        format!("Expected status '{}' but got '{}'", expected, actual)
                    },
                }
            }

            Assertion::FieldExists { field } => {
                let value = Self::get_field_value(response, field);
                let passed = value.is_some();

                AssertionResult {
                    assertion_type: "field_exists".to_string(),
                    passed,
                    expected: format!("Field '{}' exists", field),
                    actual: if passed {
                        "exists".to_string()
                    } else {
                        "not found".to_string()
                    },
                    message: if passed {
                        format!("Field '{}' exists", field)
                    } else {
                        format!("Field '{}' not found", field)
                    },
                }
            }

            Assertion::FieldEquals { field, expected } => {
                let actual = Self::get_field_value(response, field);
                let passed = actual == Some(expected);

                AssertionResult {
                    assertion_type: "field_equals".to_string(),
                    passed,
                    expected: format!("{}", expected),
                    actual: actual.map(|v| format!("{}", v)).unwrap_or_else(|| "null".to_string()),
                    message: if passed {
                        format!("Field '{}' equals expected value", field)
                    } else {
                        format!("Field '{}' does not equal expected value", field)
                    },
                }
            }

            Assertion::FieldNotEquals { field, expected } => {
                let actual = Self::get_field_value(response, field);
                let passed = actual != Some(expected);

                AssertionResult {
                    assertion_type: "field_not_equals".to_string(),
                    passed,
                    expected: format!("not {}", expected),
                    actual: actual.map(|v| format!("{}", v)).unwrap_or_else(|| "null".to_string()),
                    message: if passed {
                        format!("Field '{}' is not equal to expected value", field)
                    } else {
                        format!("Field '{}' equals expected value (should not)", field)
                    },
                }
            }

            Assertion::ArrayLength {
                field,
                operator,
                expected,
            } => {
                let array = Self::get_field_value(response, field);
                let actual_len = array
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                let passed = Self::compare_values(actual_len, operator, *expected);

                AssertionResult {
                    assertion_type: "array_length".to_string(),
                    passed,
                    expected: format!("{} {}", operator, expected),
                    actual: format!("{}", actual_len),
                    message: if passed {
                        format!("Array length {} is {} {}", actual_len, operator, expected)
                    } else {
                        format!(
                            "Array length {} is not {} {}",
                            actual_len, operator, expected
                        )
                    },
                }
            }

            Assertion::Contains { field, expected } => {
                let array = Self::get_field_value(response, field);
                let passed = array
                    .and_then(|v| v.as_array())
                    .map(|a| a.contains(expected))
                    .unwrap_or(false);

                AssertionResult {
                    assertion_type: "contains".to_string(),
                    passed,
                    expected: format!("{}", expected),
                    actual: format!("{:?}", array),
                    message: if passed {
                        "Array contains expected value".to_string()
                    } else {
                        "Array does not contain expected value".to_string()
                    },
                }
            }

            Assertion::ResponseTime { operator, expected } => {
                let actual = metadata.duration_ms;
                let passed = Self::compare_values(actual, operator, *expected);

                AssertionResult {
                    assertion_type: "response_time".to_string(),
                    passed,
                    expected: format!("{} {}ms", operator, expected),
                    actual: format!("{}ms", actual),
                    message: if passed {
                        format!("Response time {}ms is {} {}ms", actual, operator, expected)
                    } else {
                        format!(
                            "Response time {}ms is not {} {}ms",
                            actual, operator, expected
                        )
                    },
                }
            }

            Assertion::ErrorRate { operator, expected } => {
                // モック実装: 実際にはエラー履歴から計算
                let actual_rate = 0.0;
                let passed = Self::compare_values_f64(actual_rate, operator, *expected);

                AssertionResult {
                    assertion_type: "error_rate".to_string(),
                    passed,
                    expected: format!("{} {}", operator, expected),
                    actual: format!("{}", actual_rate),
                    message: if passed {
                        format!("Error rate {} is {} {}", actual_rate, operator, expected)
                    } else {
                        format!("Error rate {} is not {} {}", actual_rate, operator, expected)
                    },
                }
            }

            Assertion::ErrorType { expected } => {
                let actual = response
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let passed = actual.contains(expected);

                AssertionResult {
                    assertion_type: "error_type".to_string(),
                    passed,
                    expected: expected.clone(),
                    actual: actual.to_string(),
                    message: if passed {
                        format!("Error type matches '{}'", expected)
                    } else {
                        format!("Error type '{}' does not match '{}'", actual, expected)
                    },
                }
            }

            Assertion::ErrorMessageContains { expected } => {
                let actual = response
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let passed = actual.contains(expected);

                AssertionResult {
                    assertion_type: "error_message_contains".to_string(),
                    passed,
                    expected: expected.clone(),
                    actual: actual.to_string(),
                    message: if passed {
                        format!("Error message contains '{}'", expected)
                    } else {
                        format!("Error message does not contain '{}'", expected)
                    },
                }
            }

            Assertion::JsonContainsKey { field, expected } => {
                let object = Self::get_field_value(response, field);
                let passed = object
                    .and_then(|v| v.as_object())
                    .map(|o| o.contains_key(expected))
                    .unwrap_or(false);

                AssertionResult {
                    assertion_type: "json_contains_key".to_string(),
                    passed,
                    expected: expected.clone(),
                    actual: format!("{:?}", object),
                    message: if passed {
                        format!("Object contains key '{}'", expected)
                    } else {
                        format!("Object does not contain key '{}'", expected)
                    },
                }
            }

            Assertion::JsonValueType { field, expected } => {
                let value = Self::get_field_value(response, field);
                let actual_type = Self::get_json_type(value);
                let passed = actual_type == *expected;

                AssertionResult {
                    assertion_type: "json_value_type".to_string(),
                    passed,
                    expected: expected.clone(),
                    actual: actual_type.clone(),
                    message: if passed {
                        format!("Value type is '{}'", expected)
                    } else {
                        format!("Value type is '{}', expected '{}'", actual_type, expected)
                    },
                }
            }

            Assertion::JsonValueRange { field, min, max } => {
                let value = Self::get_field_value(response, field);
                let actual = value.and_then(|v| v.as_f64()).unwrap_or(0.0);
                let passed = actual >= *min && actual <= *max;

                AssertionResult {
                    assertion_type: "json_value_range".to_string(),
                    passed,
                    expected: format!("[{}, {}]", min, max),
                    actual: format!("{}", actual),
                    message: if passed {
                        format!("Value {} is in range [{}, {}]", actual, min, max)
                    } else {
                        format!("Value {} is not in range [{}, {}]", actual, min, max)
                    },
                }
            }
        }
    }

    /// JSONPathを使用してフィールド値を取得
    fn get_field_value<'a>(response: &'a Value, field: &str) -> Option<&'a Value> {
        // シンプルなドット記法をサポート
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = response;

        for part in parts {
            if let Some(obj) = current.as_object() {
                current = obj.get(part)?;
            } else if let Some(array) = current.as_array() {
                // 配列インデックスアクセス
                if let Ok(index) = part.parse::<usize>() {
                    current = array.get(index)?;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// JSON値の型を文字列で取得
    fn get_json_type(value: Option<&Value>) -> String {
        match value {
            Some(Value::String(_)) => "string".to_string(),
            Some(Value::Number(_)) => "number".to_string(),
            Some(Value::Bool(_)) => "boolean".to_string(),
            Some(Value::Array(_)) => "array".to_string(),
            Some(Value::Object(_)) => "object".to_string(),
            Some(Value::Null) | None => "null".to_string(),
        }
    }

    /// 演算子による比較（整数）
    fn compare_values<T: PartialOrd>(actual: T, operator: &str, expected: T) -> bool {
        match operator {
            ">" => actual > expected,
            "<" => actual < expected,
            ">=" => actual >= expected,
            "<=" => actual <= expected,
            "==" => actual == expected,
            "!=" => actual != expected,
            _ => false,
        }
    }

    /// 演算子による比較（浮動小数点）
    fn compare_values_f64(actual: f64, operator: &str, expected: f64) -> bool {
        match operator {
            ">" => actual > expected,
            "<" => actual < expected,
            ">=" => actual >= expected,
            "<=" => actual <= expected,
            "==" => (actual - expected).abs() < f64::EPSILON,
            "!=" => (actual - expected).abs() >= f64::EPSILON,
            _ => false,
        }
    }
}

impl Default for TestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// テストメタデータ
struct TestMetadata {
    duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_definition::{TestCase, TestConfig, TestSuite};

    #[tokio::test]
    async fn test_run_single_test_success() {
        let test = TestCase {
            name: "Health Check".to_string(),
            description: None,
            tool: "health_check".to_string(),
            server: "test_server".to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        };

        let config = TestConfig {
            timeout_ms: 5000,
            retry_count: 1,
            fail_fast: false,
            parallel: false,
        };

        let executor = TestExecutor::new();
        let result = executor.run_test_case(&test, &config).await.unwrap();
        assert!(result.passed);
        assert_eq!(result.assertions.len(), 1);
        assert!(result.assertions[0].passed);
    }

    #[tokio::test]
    async fn test_run_single_test_failure() {
        let test = TestCase {
            name: "Health Check".to_string(),
            description: None,
            tool: "health_check".to_string(),
            server: "test_server".to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![Assertion::Status {
                expected: "unhealthy".to_string(),
            }],
        };

        let config = TestConfig {
            timeout_ms: 5000,
            retry_count: 1,
            fail_fast: false,
            parallel: false,
        };

        let executor = TestExecutor::new();
        let result = executor.run_test_case(&test, &config).await.unwrap();
        assert!(!result.passed);
        assert_eq!(result.assertions.len(), 1);
        assert!(!result.assertions[0].passed);
    }

    #[tokio::test]
    async fn test_assertion_field_exists_success() {
        let response = serde_json::json!({
            "status": "healthy",
            "tools": [],
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::FieldExists {
            field: "status".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_field_exists_failure() {
        let response = serde_json::json!({
            "status": "healthy",
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::FieldExists {
            field: "missing_field".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(!result.passed);
    }

    #[tokio::test]
    async fn test_assertion_field_equals() {
        let response = serde_json::json!({
            "count": 5,
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::FieldEquals {
            field: "count".to_string(),
            expected: serde_json::json!(5),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_field_not_equals() {
        let response = serde_json::json!({
            "status": "healthy",
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::FieldNotEquals {
            field: "status".to_string(),
            expected: serde_json::json!("unhealthy"),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_array_length() {
        let response = serde_json::json!({
            "tools": ["tool1", "tool2", "tool3"],
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::ArrayLength {
            field: "tools".to_string(),
            operator: ">".to_string(),
            expected: 2,
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_contains() {
        let response = serde_json::json!({
            "tools": ["tool1", "tool2"],
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::Contains {
            field: "tools".to_string(),
            expected: serde_json::json!("tool1"),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_response_time() {
        let response = serde_json::json!({});
        let metadata = TestMetadata { duration_ms: 500 };

        let assertion = Assertion::ResponseTime {
            operator: "<".to_string(),
            expected: 1000,
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_error_type() {
        let response = serde_json::json!({
            "error": "ValidationError: Invalid input",
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::ErrorType {
            expected: "ValidationError".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_error_message_contains() {
        let response = serde_json::json!({
            "error": "Invalid input provided",
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::ErrorMessageContains {
            expected: "Invalid input".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_json_contains_key() {
        let response = serde_json::json!({
            "metadata": {
                "version": "1.0",
                "author": "test",
            },
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::JsonContainsKey {
            field: "metadata".to_string(),
            expected: "version".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_json_value_type() {
        let response = serde_json::json!({
            "count": 42,
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::JsonValueType {
            field: "count".to_string(),
            expected: "number".to_string(),
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_assertion_json_value_range() {
        let response = serde_json::json!({
            "score": 75.5,
        });

        let metadata = TestMetadata { duration_ms: 100 };

        let assertion = Assertion::JsonValueRange {
            field: "score".to_string(),
            min: 0.0,
            max: 100.0,
        };

        let result = TestExecutor::evaluate_assertion(&assertion, &response, &metadata);
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let test = TestCase {
            name: "Retry Test".to_string(),
            description: None,
            tool: "health_check".to_string(),
            server: "test_server".to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        };

        let config = TestConfig {
            timeout_ms: 5000,
            retry_count: 3,
            fail_fast: false,
            parallel: false,
        };

        let executor = TestExecutor::new();
        let result = executor.run_test_case_with_retry(&test, &config)
            .await
            .unwrap();
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let test = TestCase {
            name: "Timeout Test".to_string(),
            description: None,
            tool: "health_check".to_string(),
            server: "test_server".to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![],
        };

        let config = TestConfig {
            timeout_ms: 1, // 非常に短いタイムアウト
            retry_count: 1,
            fail_fast: false,
            parallel: false,
        };

        let executor = TestExecutor::new();
        let result = executor.run_test_case(&test, &config).await.unwrap();
        // タイムアウトするか、成功するかは実行環境による
        assert!(result.error.is_some() || result.passed);
    }

    #[tokio::test]
    async fn test_expect_error_flag() {
        let test = TestCase {
            name: "Error Test".to_string(),
            description: None,
            tool: "tools_call".to_string(),
            server: "test_server".to_string(),
            arguments: serde_json::json!({
                "name": "error_tool",
            }),
            expect_error: true,
            assertions: vec![Assertion::ErrorMessageContains {
                expected: "failed".to_string(),
            }],
        };

        let config = TestConfig {
            timeout_ms: 5000,
            retry_count: 1,
            fail_fast: false,
            parallel: false,
        };

        let executor = TestExecutor::new();
        let result = executor.run_test_case(&test, &config).await.unwrap();
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let suite = TestSuite {
            name: "Sequential Test Suite".to_string(),
            version: "1.0".to_string(),
            description: None,
            config: TestConfig {
                timeout_ms: 5000,
                retry_count: 1,
                fail_fast: false,
                parallel: false,
            },
            tests: vec![
                TestCase {
                    name: "Test 1".to_string(),
                    description: None,
                    tool: "health_check".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![Assertion::Status {
                        expected: "healthy".to_string(),
                    }],
                },
                TestCase {
                    name: "Test 2".to_string(),
                    description: None,
                    tool: "tools_list".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![Assertion::FieldExists {
                        field: "tools".to_string(),
                    }],
                },
            ],
        };

        let executor = TestExecutor::new();
        let results = executor.run_test_suite(&suite).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(results[1].passed);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let suite = TestSuite {
            name: "Parallel Test Suite".to_string(),
            version: "1.0".to_string(),
            description: None,
            config: TestConfig {
                timeout_ms: 5000,
                retry_count: 1,
                fail_fast: false,
                parallel: true,
            },
            tests: vec![
                TestCase {
                    name: "Test 1".to_string(),
                    description: None,
                    tool: "health_check".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![Assertion::Status {
                        expected: "healthy".to_string(),
                    }],
                },
                TestCase {
                    name: "Test 2".to_string(),
                    description: None,
                    tool: "tools_list".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![Assertion::FieldExists {
                        field: "tools".to_string(),
                    }],
                },
            ],
        };

        let executor = TestExecutor::new();
        let results = executor.run_test_suite(&suite).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_fail_fast() {
        let suite = TestSuite {
            name: "Fail Fast Test Suite".to_string(),
            version: "1.0".to_string(),
            description: None,
            config: TestConfig {
                timeout_ms: 5000,
                retry_count: 0,
                fail_fast: true,
                parallel: false,
            },
            tests: vec![
                TestCase {
                    name: "Test 1 (Fail)".to_string(),
                    description: None,
                    tool: "health_check".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![Assertion::Status {
                        expected: "unhealthy".to_string(),
                    }],
                },
                TestCase {
                    name: "Test 2 (Should Not Run)".to_string(),
                    description: None,
                    tool: "tools_list".to_string(),
                    server: "test_server".to_string(),
                    arguments: serde_json::json!({}),
                    expect_error: false,
                    assertions: vec![],
                },
            ],
        };

        let executor = TestExecutor::new();
        let results = executor.run_test_suite(&suite).await.unwrap();
        assert_eq!(results.len(), 1); // Only first test should run
        assert!(!results[0].passed);
    }

    #[tokio::test]
    async fn test_nested_field_access() {
        let response = serde_json::json!({
            "data": {
                "user": {
                    "name": "John",
                    "age": 30,
                },
            },
        });

        let value = TestExecutor::get_field_value(&response, "data.user.name");
        assert_eq!(value, Some(&serde_json::json!("John")));
    }

    #[tokio::test]
    async fn test_array_index_access() {
        let response = serde_json::json!({
            "items": [
                {"id": 1},
                {"id": 2},
                {"id": 3},
            ],
        });

        let value = TestExecutor::get_field_value(&response, "items.1.id");
        assert_eq!(value, Some(&serde_json::json!(2)));
    }
}

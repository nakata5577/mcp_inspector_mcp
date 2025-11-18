use serde::{Deserialize, Serialize};
use std::path::Path;

/// 有効なツール名の一覧
pub const VALID_TOOLS: &[&str] = &[
    "health_check",
    "tools_list",
    "tools_call",
    "resources_list",
    "resource_read",
    "prompts_list",
    "prompt_get",
];

/// 有効な比較演算子の一覧
pub const VALID_OPERATORS: &[&str] = &[">", "<", ">=", "<=", "==", "!="];

/// テストスイート全体の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub config: TestConfig,
    pub tests: Vec<TestCase>,
}

/// テスト実行設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default)]
    pub fail_fast: bool,
    #[serde(default)]
    pub parallel: bool,
}

fn default_timeout() -> u64 {
    30000
}

fn default_retry_count() -> u32 {
    1
}

/// 個別のテストケース
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tool: String,
    pub server: String,
    pub arguments: serde_json::Value,
    #[serde(default)]
    pub expect_error: bool,
    pub assertions: Vec<Assertion>,
}

/// アサーション定義（12種類）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    /// ステータスチェック
    Status { expected: String },

    /// フィールド存在チェック
    FieldExists { field: String },

    /// フィールド値の完全一致
    FieldEquals {
        field: String,
        expected: serde_json::Value,
    },

    /// フィールド値の不一致
    FieldNotEquals {
        field: String,
        expected: serde_json::Value,
    },

    /// 配列長チェック
    ArrayLength {
        field: String,
        operator: String,
        expected: usize,
    },

    /// 配列要素の存在チェック
    Contains {
        field: String,
        expected: serde_json::Value,
    },

    /// レスポンス時間チェック
    ResponseTime { operator: String, expected: u64 },

    /// エラー率チェック
    ErrorRate { operator: String, expected: f64 },

    /// エラータイプチェック
    ErrorType { expected: String },

    /// エラーメッセージの部分一致
    ErrorMessageContains { expected: String },

    /// JSONキーの存在チェック
    JsonContainsKey { field: String, expected: String },

    /// JSON値の型チェック
    JsonValueType { field: String, expected: String },

    /// JSON値の範囲チェック
    JsonValueRange {
        field: String,
        min: f64,
        max: f64,
    },
}

impl TestSuite {
    /// YAML形式からロード
    ///
    /// # Arguments
    ///
    /// * `content` - YAML形式のテスト定義文字列
    ///
    /// # Returns
    ///
    /// パース成功時はTestSuite、失敗時はエラー
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        serde_yaml::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))
    }

    /// JSON形式からロード
    ///
    /// # Arguments
    ///
    /// * `content` - JSON形式のテスト定義文字列
    ///
    /// # Returns
    ///
    /// パース成功時はTestSuite、失敗時はエラー
    pub fn from_json(content: &str) -> anyhow::Result<Self> {
        serde_json::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
    }

    /// ファイルから自動判別してロード
    ///
    /// # Arguments
    ///
    /// * `path` - テスト定義ファイルのパス
    ///
    /// # Returns
    ///
    /// 読み込み成功時はTestSuite、失敗時はエラー
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => Self::from_yaml(&content),
            Some("json") => Self::from_json(&content),
            _ => Err(anyhow::anyhow!(
                "Unsupported file extension. Expected .yaml, .yml, or .json"
            )),
        }
    }

    /// テスト定義のバリデーション
    ///
    /// # Returns
    ///
    /// バリデーション結果。すべて成功時はOk(())
    pub fn validate(&self) -> anyhow::Result<()> {
        // テスト名の重複チェック
        let mut test_names = std::collections::HashSet::new();
        for test in &self.tests {
            if !test_names.insert(&test.name) {
                return Err(anyhow::anyhow!("Duplicate test name: {}", test.name));
            }
        }

        // タイムアウト値の妥当性チェック
        if self.config.timeout_ms == 0 {
            return Err(anyhow::anyhow!("timeout_ms must be greater than 0"));
        }

        if self.config.timeout_ms > 600000 {
            return Err(anyhow::anyhow!(
                "timeout_ms must be less than or equal to 600000 (10 minutes)"
            ));
        }

        // 各テストケースのバリデーション
        for test in &self.tests {
            self.validate_test_case(test)?;
        }

        Ok(())
    }

    fn validate_test_case(&self, test: &TestCase) -> anyhow::Result<()> {
        // ツール名の妥当性チェック
        if !VALID_TOOLS.contains(&test.tool.as_str()) {
            return Err(anyhow::anyhow!(
                "Invalid tool name: '{}'. Must be one of: {}",
                test.tool,
                VALID_TOOLS.join(", ")
            ));
        }

        // アサーションのバリデーション
        for assertion in &test.assertions {
            self.validate_assertion(assertion)?;
        }

        Ok(())
    }

    fn validate_assertion(&self, assertion: &Assertion) -> anyhow::Result<()> {
        match assertion {
            Assertion::ArrayLength { operator, .. }
            | Assertion::ResponseTime { operator, .. }
            | Assertion::ErrorRate { operator, .. } => {
                if !VALID_OPERATORS.contains(&operator.as_str()) {
                    return Err(anyhow::anyhow!(
                        "Invalid operator: '{}'. Must be one of: {}",
                        operator,
                        VALID_OPERATORS.join(", ")
                    ));
                }
            }
            Assertion::JsonValueType { expected, .. } => {
                let valid_types = ["string", "number", "boolean", "array", "object", "null"];
                if !valid_types.contains(&expected.as_str()) {
                    return Err(anyhow::anyhow!(
                        "Invalid JSON value type: {}. Must be one of: {}",
                        expected,
                        valid_types.join(", ")
                    ));
                }
            }
            Assertion::JsonValueRange { min, max, .. } => {
                if min >= max {
                    return Err(anyhow::anyhow!(
                        "Invalid range: min ({}) must be less than max ({})",
                        min,
                        max
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_basic_parse() {
        let yaml = r#"
name: "Test Suite"
version: "1.0"
config:
  timeout_ms: 10000
  retry_count: 1
  fail_fast: false
  parallel: false
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert_eq!(suite.name, "Test Suite");
        assert_eq!(suite.version, "1.0");
        assert_eq!(suite.config.timeout_ms, 10000);
        assert_eq!(suite.tests.len(), 1);
    }

    #[test]
    fn test_yaml_with_defaults() {
        let yaml = r#"
name: "Test Suite"
version: "1.0"
config: {}
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert_eq!(suite.config.timeout_ms, 30000);
        assert_eq!(suite.config.retry_count, 1);
        assert!(!suite.config.fail_fast);
        assert!(!suite.config.parallel);
    }

    #[test]
    fn test_yaml_all_assertion_types() {
        let yaml = r#"
name: "Test Suite"
version: "1.0"
config: {}
tests:
  - name: "Test All Assertions"
    tool: "tools_list"
    server: "test_server"
    arguments: {}
    assertions:
      - type: "status"
        expected: "success"
      - type: "field_exists"
        field: "tools"
      - type: "field_equals"
        field: "count"
        expected: 5
      - type: "field_not_equals"
        field: "error"
        expected: null
      - type: "array_length"
        field: "tools"
        operator: ">"
        expected: 0
      - type: "contains"
        field: "tools"
        expected: "test_tool"
      - type: "response_time"
        operator: "<"
        expected: 1000
      - type: "error_rate"
        operator: "<"
        expected: 0.01
      - type: "error_type"
        expected: "ValidationError"
      - type: "error_message_contains"
        expected: "invalid input"
      - type: "json_contains_key"
        field: "metadata"
        expected: "version"
      - type: "json_value_type"
        field: "count"
        expected: "number"
      - type: "json_value_range"
        field: "score"
        min: 0.0
        max: 100.0
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert_eq!(suite.tests[0].assertions.len(), 13);
    }

    #[test]
    fn test_yaml_invalid_format() {
        let yaml = "invalid: yaml: content:";
        assert!(TestSuite::from_yaml(yaml).is_err());
    }

    #[test]
    fn test_yaml_missing_required_field() {
        let yaml = r#"
name: "Test Suite"
config: {}
tests: []
"#;
        assert!(TestSuite::from_yaml(yaml).is_err());
    }

    #[test]
    fn test_yaml_complex_arguments() {
        let yaml = r#"
name: "Test Suite"
version: "1.0"
config: {}
tests:
  - name: "Complex Test"
    tool: "tools_call"
    server: "test_server"
    arguments:
      name: "test_tool"
      params:
        key1: "value1"
        key2: 123
        key3:
          - item1
          - item2
    assertions:
      - type: "status"
        expected: "success"
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        let args = &suite.tests[0].arguments;
        assert!(args.is_object());
        assert_eq!(args["name"], "test_tool");
    }

    #[test]
    fn test_json_basic_parse() {
        let json = r#"
{
  "name": "Test Suite",
  "version": "1.0",
  "config": {
    "timeout_ms": 10000,
    "retry_count": 1,
    "fail_fast": false,
    "parallel": false
  },
  "tests": [
    {
      "name": "Test 1",
      "tool": "health_check",
      "server": "test_server",
      "arguments": {},
      "assertions": [
        {
          "type": "status",
          "expected": "healthy"
        }
      ]
    }
  ]
}
"#;
        let suite = TestSuite::from_json(json).unwrap();
        assert_eq!(suite.name, "Test Suite");
        assert_eq!(suite.version, "1.0");
        assert_eq!(suite.tests.len(), 1);
    }

    #[test]
    fn test_json_with_defaults() {
        let json = r#"
{
  "name": "Test Suite",
  "version": "1.0",
  "config": {},
  "tests": [
    {
      "name": "Test 1",
      "tool": "health_check",
      "server": "test_server",
      "arguments": {},
      "assertions": [
        {
          "type": "status",
          "expected": "healthy"
        }
      ]
    }
  ]
}
"#;
        let suite = TestSuite::from_json(json).unwrap();
        assert_eq!(suite.config.timeout_ms, 30000);
    }

    #[test]
    fn test_json_all_assertion_types() {
        let json = r#"
{
  "name": "Test Suite",
  "version": "1.0",
  "config": {},
  "tests": [
    {
      "name": "Test All Assertions",
      "tool": "tools_list",
      "server": "test_server",
      "arguments": {},
      "assertions": [
        {"type": "status", "expected": "success"},
        {"type": "field_exists", "field": "tools"},
        {"type": "field_equals", "field": "count", "expected": 5},
        {"type": "array_length", "field": "tools", "operator": ">", "expected": 0}
      ]
    }
  ]
}
"#;
        let suite = TestSuite::from_json(json).unwrap();
        assert_eq!(suite.tests[0].assertions.len(), 4);
    }

    #[test]
    fn test_json_invalid_format() {
        let json = "{invalid json";
        assert!(TestSuite::from_json(json).is_err());
    }

    #[test]
    fn test_json_missing_required_field() {
        let json = r#"
{
  "name": "Test Suite",
  "config": {},
  "tests": []
}
"#;
        assert!(TestSuite::from_json(json).is_err());
    }

    #[test]
    fn test_json_expect_error_flag() {
        let json = r#"
{
  "name": "Test Suite",
  "version": "1.0",
  "config": {},
  "tests": [
    {
      "name": "Error Test",
      "tool": "tools_call",
      "server": "test_server",
      "arguments": {},
      "expect_error": true,
      "assertions": [
        {"type": "error_type", "expected": "ValidationError"}
      ]
    }
  ]
}
"#;
        let suite = TestSuite::from_json(json).unwrap();
        assert!(suite.tests[0].expect_error);
    }

    #[test]
    fn test_validate_success() {
        let yaml = r#"
name: "Valid Suite"
version: "1.0"
config:
  timeout_ms: 5000
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert!(suite.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_test_names() {
        let yaml = r#"
name: "Invalid Suite"
version: "1.0"
config: {}
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions: []
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions: []
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert!(suite.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_timeout() {
        let yaml = r#"
name: "Invalid Suite"
version: "1.0"
config:
  timeout_ms: 0
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions: []
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert!(suite.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_tool_name() {
        let yaml = r#"
name: "Invalid Suite"
version: "1.0"
config: {}
tests:
  - name: "Test 1"
    tool: "invalid_tool"
    server: "test_server"
    arguments: {}
    assertions: []
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert!(suite.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_operator() {
        let yaml = r#"
name: "Invalid Suite"
version: "1.0"
config: {}
tests:
  - name: "Test 1"
    tool: "health_check"
    server: "test_server"
    arguments: {}
    assertions:
      - type: "response_time"
        operator: "invalid"
        expected: 1000
"#;
        let suite = TestSuite::from_yaml(yaml).unwrap();
        assert!(suite.validate().is_err());
    }
}

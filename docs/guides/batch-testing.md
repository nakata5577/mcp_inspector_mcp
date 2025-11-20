# バッチテストガイド

**対象バージョン**: v0.4.0+
**最終更新**: 2025-11-20

---

## 目次

- [概要](#概要)
- [テスト定義](#テスト定義)
- [アサーション](#アサーション)
- [テスト実行](#テスト実行)
- [レポート形式](#レポート形式)
- [CI/CD統合](#cicd統合)
- [実践的な使用例](#実践的な使用例)
- [ベストプラクティス](#ベストプラクティス)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

バッチテスト機能は、MCPサーバーの自動テストを実現する強力な機能です。YAML/JSON形式でテストを定義し、CI/CD環境に統合することで、継続的な品質保証を実現します。

### バッチテスト機能の特徴

- **構造化されたテスト定義**: YAML/JSON形式
- **柔軟なアサーション**: 12種類の検証方法
- **並列/順次実行**: テストの実行モードを選択可能
- **多形式レポート**: Console/JSON/JUnit XML
- **CI/CD統合**: GitHub Actions/GitLab CI対応
- **リトライ機能**: テスト失敗時の自動リトライ

### ユースケース

- **回帰テスト**: 既存機能の動作を継続的に検証
- **統合テスト**: 複数のツールを組み合わせたシナリオテスト
- **パフォーマンステスト**: レスポンスタイムの検証
- **エラーハンドリングテスト**: 異常系の動作確認
- **CI/CD**: 自動テストによる品質保証

---

## テスト定義

テストスイートはYAML/JSON形式で定義します。

### 基本構造

```yaml
# test_suite.yaml
name: "Test Suite Name"
version: "1.0"
description: "Test suite description"

config:
  timeout_ms: 30000        # テストケースのタイムアウト（ミリ秒）
  retry_count: 1           # 失敗時のリトライ回数
  fail_fast: false         # 1つ失敗したら即座に停止
  parallel: true           # 並列実行を有効化

tests:
  - name: "Test Case 1"
    description: "Test case description"
    tool: "tool_name"
    server: "server_name"
    arguments: {}
    assertions:
      - type: "status"
        expected: "success"
```

### フィールド説明

#### トップレベル

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ✅ | テストスイート名 |
| `version` | string | ❌ | テストスイートのバージョン |
| `description` | string | ❌ | テストスイートの説明 |
| `config` | object | ❌ | テスト実行設定 |
| `tests` | array | ✅ | テストケース配列 |

#### config オブジェクト

| フィールド | 型 | デフォルト | 説明 |
|-----------|-----|-----------|------|
| `timeout_ms` | integer | 30000 | タイムアウト（ミリ秒） |
| `retry_count` | integer | 0 | リトライ回数 |
| `fail_fast` | boolean | false | 失敗時に即座に停止 |
| `parallel` | boolean | false | 並列実行 |

#### tests 配列要素

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | ✅ | テストケース名 |
| `description` | string | ❌ | テストケースの説明 |
| `tool` | string | ✅ | 実行するMCPツール名 |
| `server` | string | ✅ | 対象サーバー名 |
| `arguments` | object | ❌ | ツールに渡す引数 |
| `expect_error` | boolean | ❌ | エラーを期待する（デフォルト: false） |
| `assertions` | array | ✅ | アサーション配列 |

### YAML形式の例

```yaml
name: "Fundamental Analysis Server Test Suite"
version: "1.0"
description: "Comprehensive test suite for fundamental analysis server"

config:
  timeout_ms: 30000
  retry_count: 1
  fail_fast: false
  parallel: true

tests:
  - name: "Server Health Check"
    description: "Verify server is responsive"
    tool: "health_check"
    server: "fundamental_analysis"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
      - type: "response_time"
        operator: "<"
        expected: 500

  - name: "Calculate RSI for AAPL"
    description: "Test RSI calculation tool"
    tool: "tools_call"
    server: "fundamental_analysis"
    arguments:
      tool_name: "calculate_rsi"
      arguments:
        symbol: "AAPL"
        period: 14
    assertions:
      - type: "json_value_type"
        field: "result.content[0].text.rsi"
        expected: "number"
      - type: "json_value_range"
        field: "result.content[0].text.rsi"
        min: 0
        max: 100
```

### JSON形式の例

```json
{
  "name": "Fundamental Analysis Server Test Suite",
  "version": "1.0",
  "description": "Comprehensive test suite",
  "config": {
    "timeout_ms": 30000,
    "retry_count": 1,
    "fail_fast": false,
    "parallel": true
  },
  "tests": [
    {
      "name": "Server Health Check",
      "description": "Verify server is responsive",
      "tool": "health_check",
      "server": "fundamental_analysis",
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
```

---

## アサーション

バッチテストでは12種類のアサーションが利用可能です。

### 1. status - ステータス値の検証

ヘルスチェックのステータスを検証します。

```yaml
assertions:
  - type: "status"
    expected: "healthy"  # healthy/degraded/unhealthy
```

### 2. field_exists - フィールドの存在確認

指定したフィールドが存在するか確認します。

```yaml
assertions:
  - type: "field_exists"
    field: "tools"
```

### 3. field_equals - フィールドの値が一致

フィールドの値が期待値と完全一致するか確認します。

```yaml
assertions:
  - type: "field_equals"
    field: "server"
    expected: "fundamental_analysis"
```

### 4. field_not_equals - フィールドの値が不一致

フィールドの値が期待値と異なるか確認します。

```yaml
assertions:
  - type: "field_not_equals"
    field: "error"
    expected: null
```

### 5. array_length - 配列長の検証

配列の長さを検証します。

```yaml
assertions:
  - type: "array_length"
    field: "tools"
    operator: ">"  # >, >=, <, <=, ==
    expected: 0
```

### 6. contains - 配列が特定値を含むか

配列に特定の値が含まれているか確認します。

```yaml
assertions:
  - type: "contains"
    field: "tools[*].name"
    expected: "calculate_rsi"
```

### 7. response_time - レスポンスタイムの検証

レスポンスタイムを検証します。

```yaml
assertions:
  - type: "response_time"
    operator: "<"  # >, >=, <, <=, ==
    expected: 500  # ミリ秒
```

### 8. error_rate - エラー率の検証

エラー率を検証します。

```yaml
assertions:
  - type: "error_rate"
    operator: "<"
    expected: 0.05  # 5%
```

### 9. error_type - エラータイプの検証

エラーの種類を検証します（`expect_error: true`時に使用）。

```yaml
assertions:
  - type: "error_type"
    expected: "Timeout"  # Timeout/ServerCrash/InvalidResponse/etc.
```

### 10. error_message_contains - エラーメッセージ検証

エラーメッセージに特定文字列が含まれているか確認します。

```yaml
assertions:
  - type: "error_message_contains"
    expected: "connection"
```

### 11. json_contains_key - JSONキーの存在確認

JSON文字列内に特定のキーが存在するか確認します。

```yaml
assertions:
  - type: "json_contains_key"
    field: "result.content[0].text"
    expected: "rsi"
```

### 12. json_value_type - JSON値の型検証

JSON値の型を検証します。

```yaml
assertions:
  - type: "json_value_type"
    field: "result.content[0].text.rsi"
    expected: "number"  # string/number/boolean/array/object/null
```

### 13. json_value_range - 数値範囲の検証

JSON数値が範囲内にあるか検証します。

```yaml
assertions:
  - type: "json_value_range"
    field: "result.content[0].text.rsi"
    min: 0
    max: 100
```

### JSONPath記法

フィールド指定にはJSONPath記法を使用できます:

- `result.content[0].text` - ネストされたフィールド
- `tools[*].name` - 配列のすべての要素
- `tools[0].name` - 配列の特定要素

---

## テスト実行

テストスイートの実行方法を説明します。

### 基本的な実行

```bash
mcp_inspector_mcp test run --suite tests/basic_test.yaml
```

### 並列実行

```bash
mcp_inspector_mcp test run --suite tests/advanced_test.yaml --parallel
```

### レポート形式の指定

```bash
# Console形式（デフォルト）
mcp_inspector_mcp test run --suite tests/basic_test.yaml

# JSON形式
mcp_inspector_mcp test run --suite tests/basic_test.yaml \
  --report-format json --output results.json

# JUnit XML形式
mcp_inspector_mcp test run --suite tests/ci_test.yaml \
  --report-format junit --output results.xml
```

### タイムアウトの指定

```bash
# テストスイート全体のタイムアウト
mcp_inspector_mcp test run --suite tests/basic_test.yaml --timeout 60000
```

### 特定のテストケースのみ実行

```bash
# テストケース名で指定
mcp_inspector_mcp test run --suite tests/basic_test.yaml \
  --test-case "Server Health Check"
```

### dry-run モード

```bash
# テストを実行せずに検証のみ
mcp_inspector_mcp test validate --suite tests/basic_test.yaml
```

---

## レポート形式

バッチテストでは3種類のレポート形式をサポートします。

### Console形式

人間が読みやすいターミナル出力です。

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Test Suite: Fundamental Analysis Server Test Suite
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Server Health Check (234ms)
   ✓ status == "healthy"
   ✓ response_time < 500ms (actual: 234ms)

✅ Calculate RSI for AAPL (456ms)
   ✓ json_value_type "result.content[0].text.rsi" == "number"
   ✓ json_value_range "result.content[0].text.rsi" 0-100 (actual: 65.3)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 2 tests
Passed: 2 tests (100%)
Failed: 0 tests
Duration: 690ms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### JSON形式

プログラムで処理しやすい形式です。

```json
{
  "test_suite": "Fundamental Analysis Server Test Suite",
  "version": "1.0",
  "total": 2,
  "passed": 2,
  "failed": 0,
  "duration_ms": 690,
  "tests": [
    {
      "name": "Server Health Check",
      "status": "passed",
      "duration_ms": 234,
      "assertions": [
        {
          "type": "status",
          "passed": true,
          "expected": "healthy",
          "actual": "healthy"
        }
      ]
    }
  ]
}
```

### JUnit XML形式

CI/CD統合用の標準形式です。

```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Fundamental Analysis Server Test Suite" tests="2" failures="0" time="0.690">
  <testsuite name="Fundamental Analysis Server Test Suite" tests="2" failures="0" time="0.690">
    <testcase name="Server Health Check" classname="fundamental_analysis" time="0.234">
      <system-out>
        ✓ status == "healthy"
        ✓ response_time &lt; 500ms (actual: 234ms)
      </system-out>
    </testcase>
    <testcase name="Calculate RSI for AAPL" classname="fundamental_analysis" time="0.456">
      <system-out>
        ✓ json_value_type "result.content[0].text.rsi" == "number"
        ✓ json_value_range "result.content[0].text.rsi" 0-100 (actual: 65.3)
      </system-out>
    </testcase>
  </testsuite>
</testsuites>
```

---

## CI/CD統合

### GitHub Actions

`.github/workflows/mcp_test.yml`:

```yaml
name: MCP Server Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install MCP Inspector
        run: |
          cargo install --path .

      - name: Run MCP Tests
        run: |
          mcp_inspector_mcp test run \
            --suite tests/ci_test.yaml \
            --report-format junit \
            --output test-results.xml

      - name: Publish Test Results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: test-results.xml

      - name: Upload Test Results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: test-results.xml
```

### GitLab CI

`.gitlab-ci.yml`:

```yaml
stages:
  - test

mcp_test:
  stage: test
  image: rust:latest
  script:
    - cargo install --path .
    - mcp_inspector_mcp test run \
        --suite tests/ci_test.yaml \
        --report-format junit \
        --output test-results.xml
  artifacts:
    when: always
    reports:
      junit: test-results.xml
    paths:
      - test-results.xml
```

### 終了コード

テスト結果は終了コードで判定できます:

- `0`: すべてのテストが成功
- `1`: 1つ以上のテストが失敗
- `2`: テスト定義エラー
- `3`: 実行エラー

**例**:

```bash
mcp_inspector_mcp test run --suite tests/ci_test.yaml
if [ $? -eq 0 ]; then
  echo "All tests passed!"
else
  echo "Tests failed!"
  exit 1
fi
```

---

## 実践的な使用例

### 例1: ヘルスチェックテスト

すべてのMCPサーバーが正常に動作しているか確認します。

**tests/health_check.yaml:**

```yaml
name: "Health Check Test Suite"
version: "1.0"
description: "Verify all servers are healthy"

config:
  timeout_ms: 10000
  parallel: true

tests:
  - name: "Fundamental Analysis Server Health"
    tool: "health_check"
    server: "fundamental_analysis"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
      - type: "response_time"
        operator: "<"
        expected: 500

  - name: "Technical Analysis Server Health"
    tool: "health_check"
    server: "technical_analysis"
    arguments: {}
    assertions:
      - type: "status"
        expected: "healthy"
      - type: "response_time"
        operator: "<"
        expected: 500
```

### 例2: ツール機能テスト

特定のツールが正しく動作することを検証します。

**tests/rsi_calculation.yaml:**

```yaml
name: "RSI Calculation Test Suite"
version: "1.0"

tests:
  - name: "Calculate RSI for AAPL"
    tool: "tools_call"
    server: "fundamental_analysis"
    arguments:
      tool_name: "calculate_rsi"
      arguments:
        symbol: "AAPL"
        period: 14
    assertions:
      - type: "field_exists"
        field: "result.content[0].text"
      - type: "json_contains_key"
        field: "result.content[0].text"
        expected: "rsi"
      - type: "json_value_type"
        field: "result.content[0].text.rsi"
        expected: "number"
      - type: "json_value_range"
        field: "result.content[0].text.rsi"
        min: 0
        max: 100

  - name: "Calculate RSI for Invalid Symbol"
    tool: "tools_call"
    server: "fundamental_analysis"
    arguments:
      tool_name: "calculate_rsi"
      arguments:
        symbol: "INVALID_SYMBOL_12345"
        period: 14
    expect_error: true
    assertions:
      - type: "error_type"
        expected: "ServerError"
      - type: "error_message_contains"
        expected: "symbol"
```

### 例3: パフォーマンステスト

レスポンスタイムが許容範囲内であることを確認します。

**tests/performance.yaml:**

```yaml
name: "Performance Test Suite"
version: "1.0"

config:
  retry_count: 3  # 3回までリトライ

tests:
  - name: "tools_list Performance"
    tool: "tools_list"
    server: "fundamental_analysis"
    arguments: {}
    assertions:
      - type: "response_time"
        operator: "<"
        expected: 200  # 200ms以内

  - name: "health_check Performance"
    tool: "health_check"
    server: "fundamental_analysis"
    arguments: {}
    assertions:
      - type: "response_time"
        operator: "<"
        expected: 500  # 500ms以内
```

---

## ベストプラクティス

### 1. テストスイートの構成

- **目的別に分割**: health_check.yaml、functional_tests.yaml、performance_tests.yaml
- **サーバー別に分割**: fundamental_analysis_tests.yaml、technical_analysis_tests.yaml
- **CI用テストを作成**: ci_test.yaml（高速かつ重要なテストのみ）

### 2. テストケース設計

- **独立性**: 各テストは他のテストに依存しない
- **再現性**: 何度実行しても同じ結果になる
- **明確な命名**: テストケース名で何をテストしているか分かる
- **適切なタイムアウト**: 環境に合わせた値を設定

### 3. アサーション設計

- **複数のアサーション**: 1つのテストケースで複数の側面を検証
- **適切なアサーションタイプ**: 目的に合ったアサーションを選択
- **エラーメッセージ**: 失敗時に原因が分かるアサーション

### 4. CI/CD統合

- **fail_fast無効化**: CI環境ではすべてのテストを実行
- **JUnit XML出力**: CI/CDツールとの統合を容易に
- **アーティファクト保存**: テスト結果を保存

### 5. メンテナンス

- **定期的なレビュー**: テストスイートの見直し
- **不要なテスト削除**: 古いテストケースの整理
- **ドキュメント更新**: テストスイートの説明を最新に保つ

---

## トラブルシューティング

### テストが失敗する

**症状**: テストが予期せず失敗する

**原因と対策**:

1. **タイムアウト**: タイムアウト値を延長
   ```yaml
   config:
     timeout_ms: 60000
   ```

2. **サーバー未起動**: サーバーが正しく起動しているか確認
   ```bash
   mcp_inspector_mcp health_check --server fundamental_analysis
   ```

3. **アサーションエラー**: アサーションの条件を確認
   ```bash
   # デバッグモードで実行
   mcp_inspector_mcp --verbose test run --suite tests/basic_test.yaml
   ```

### テスト定義のエラー

**症状**: テスト定義ファイルが読み込めない

**対策**:

```bash
# バリデーションを実行
mcp_inspector_mcp test validate --suite tests/basic_test.yaml

# YAMLの構文確認
yamllint tests/basic_test.yaml
```

### 並列実行の問題

**症状**: 並列実行時にテストが不安定になる

**対策**:

```yaml
# 並列実行を無効化
config:
  parallel: false
```

---

## 参考リンク

- [README.md](../../README.md): 全体的な使い方
- [デバッグモードガイド](./debug-mode.md): デバッグ方法
- [パフォーマンスモニタリングガイド](./performance-monitoring.md): 性能分析
- [テストスイート例](../../examples/test_suites/): サンプルテストスイート

---

**最終更新**: 2025-11-20
**対象バージョン**: v0.4.0+

# Phase 3 詳細実行計画書

## 概要

### フェーズ情報
- **フェーズ名**: 機能拡張フェーズ (v0.4.0)
- **期間**: 3週間 (2025-11-27 〜 2025-12-15)
- **目的**: エンタープライズグレードツールへの進化

### 背景
MCP Inspector MCP Server (v0.3.1) は、タイムアウト処理、エラーハンドリング、Capability検証など堅牢な基盤を確立し、Phase 2では包括的なドキュメント整備により新規ユーザーの学習曲線を大幅に改善しました。Phase 3では、この強固な基盤の上にエンタープライズ環境で求められる高度な機能を追加し、MCP Inspector MCPをプロフェッショナルツールとして完成させます。

### ビジョン
開発者が本番環境でMCP Inspector MCPを信頼して使用でき、継続的インテグレーション（CI/CD）に組み込める、業界標準のデバッグ・監視ツールを実現する。

### Phase 3の目標

**主要目標:**
1. **デバッグ効率の劇的向上**: 詳細ログとリクエスト/レスポンス可視化による問題解決時間50%削減
2. **自動化テストの実現**: バッチテスト機能によるCI/CD統合と回帰テスト自動化
3. **ユーザー体験の改善**: インタラクティブモードの強化により操作効率30%向上
4. **パフォーマンス可視化**: メトリクス収集により性能問題の早期発見を実現
5. **運用柔軟性の向上**: プロファイル機能により環境切替を容易化

---

## タスク一覧

### Task 3.1: デバッグモードの実装 (4日間)

#### 目標
- 開発者が問題の根本原因を迅速に特定できる詳細なデバッグ情報を提供
- リクエスト/レスポンスの完全な可視化
- タイムスタンプと経過時間によるパフォーマンス分析

#### 詳細作業項目

**3.1.1 --verboseフラグの追加 (6時間)**

**実装内容:**
- コマンドライン引数に`--verbose`フラグを追加
- 環境変数`MCP_VERBOSE`のサポート
- 設定ファイル（`config.json`）での`verbose`設定サポート
- 優先順位: CLI引数 > 環境変数 > config.json > デフォルト値

**実装詳細:**
- `src/models/execution_config.rs`に`verbose: bool`フィールド追加
- `src/main.rs`でCLI引数パース処理追加
- グローバル`VERBOSE_MODE`フラグの実装（`once_cell::sync::Lazy`使用）

**テスト:**
- CLI引数パーステスト（3件）
- 環境変数読み込みテスト（2件）
- 設定ファイル読み込みテスト（2件）

**3.1.2 リクエスト/レスポンスの整形表示 (8時間)**

**実装内容:**
- JSONRPCリクエストの整形出力
- JSONRPCレスポンスの整形出力
- カラー表示のサポート（ANSIエスケープシーケンス）
- 大きなペイロードの自動トランケート機能（設定可能な最大サイズ）

**表示フォーマット:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📤 REQUEST  [2025-11-27 10:30:45.123]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Tool: tools/call
Request ID: req-abc123

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate_rsi",
    "arguments": {
      "symbol": "AAPL",
      "period": 14
    }
  }
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📥 RESPONSE [2025-11-27 10:30:45.456] (333ms)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Request ID: req-abc123
Status: ✅ Success

{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"symbol\":\"AAPL\",\"rsi\":65.3}"
      }
    ]
  }
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**実装詳細:**
- `src/services/debug_logger.rs`: 新規ファイル（約350行）
  - `DebugLogger`構造体
  - `log_request()`メソッド
  - `log_response()`メソッド
  - `format_json()`メソッド（整形処理）
  - `colorize()`メソッド（カラー表示）
  - `truncate()`メソッド（トランケート処理）

**依存クレート:**
- `colored = "2"`: ANSIカラー表示
- `serde_json = "1"`: JSON整形

**設定項目:**
```json
{
  "debug": {
    "verbose": true,
    "color_output": true,
    "max_payload_size": 4096,
    "truncate_large_payloads": true
  }
}
```

**テスト:**
- リクエスト整形テスト（5件）
- レスポンス整形テスト（5件）
- カラー表示テスト（3件）
- トランケートテスト（3件）

**3.1.3 タイムスタンプと経過時間の表示 (4時間)**

**実装内容:**
- リクエスト送信時刻の記録
- レスポンス受信時刻の記録
- 経過時間の計算と表示
- マイクロ秒精度のタイムスタンプ
- ISO 8601フォーマットでの表示

**実装詳細:**
- `std::time::Instant`による高精度タイマー
- `chrono::Utc`によるタイムスタンプ生成
- `src/services/timing_tracker.rs`: 新規ファイル（約150行）
  - `TimingTracker`構造体
  - `start()`メソッド: タイミング計測開始
  - `end()`メソッド: タイミング計測終了
  - `elapsed()`メソッド: 経過時間取得

**表示例:**
```
⏱️  Timing Information
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Request Sent:     2025-11-27T10:30:45.123456Z
Response Received: 2025-11-27T10:30:45.456789Z
Elapsed Time:     333.333ms
```

**テスト:**
- タイミング計測テスト（4件）
- タイムスタンプフォーマットテスト（3件）

**3.1.4 デバッグログの詳細化 (6時間)**

**実装内容:**
- 接続ライフサイクルのトレース（接続開始、接続成功、切断）
- キャッシュヒット/ミスのトレース
- エラー発生時のスタックトレース
- 内部状態のダンプ機能

**ログレベル:**
- `TRACE`: 最も詳細なログ（全JSONRPC通信をダンプ）
- `DEBUG`: デバッグ情報（関数呼び出し、状態変化）
- `INFO`: 通常の操作ログ（ツール実行開始/完了）
- `WARN`: 警告（Capabilityミスマッチ、タイムアウト警告）
- `ERROR`: エラー（接続失敗、ツール実行エラー）

**実装詳細:**
- `RUST_LOG`環境変数の活用
- カスタムログフォーマッタの実装
- 構造化ログ（JSON形式）のサポート

**ログフォーマット例:**
```
[2025-11-27T10:30:45.123Z DEBUG mcp_inspector_mcp::client::stdio_client] Connecting to server 'fundamental_analysis'
[2025-11-27T10:30:45.234Z INFO  mcp_inspector_mcp::client::stdio_client] Successfully connected to 'fundamental_analysis' (111ms)
[2025-11-27T10:30:45.235Z DEBUG mcp_inspector_mcp::services::response_cache] Cache miss for tools_list on server 'fundamental_analysis'
[2025-11-27T10:30:45.456Z DEBUG mcp_inspector_mcp::services::response_cache] Cached tools_list for server 'fundamental_analysis' (TTL: 300s)
[2025-11-27T10:30:45.457Z INFO  mcp_inspector_mcp::services::inspector] tools_list completed (222ms)
```

**構造化ログ（JSON形式）例:**
```json
{
  "timestamp": "2025-11-27T10:30:45.123Z",
  "level": "INFO",
  "target": "mcp_inspector_mcp::services::inspector",
  "message": "tools_list completed",
  "fields": {
    "server": "fundamental_analysis",
    "elapsed_ms": 222,
    "cache_hit": false
  }
}
```

**実装詳細:**
- `env_logger`から`tracing`への移行
- `tracing-subscriber`による構造化ログ
- `tracing-appender`によるログファイル出力

**依存クレート:**
- `tracing = "0.1"`: 構造化トレーシング
- `tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }`
- `tracing-appender = "0.2"`: ログファイル出力

**テスト:**
- ログ出力テスト（6件）
- 構造化ログパーステスト（4件）

#### 成果物
- `src/models/debug_config.rs`: デバッグ設定（約80行）
- `src/services/debug_logger.rs`: デバッグロガー（約350行）
- `src/services/timing_tracker.rs`: タイミング追跡（約150行）
- 修正ファイル: `src/main.rs`, `src/services/inspector.rs`, `src/client/stdio_client.rs`
- テストファイル: `tests/task3_1_debug_mode_test.rs`（約250行、計27テスト）

#### 成功基準
- `--verbose`フラグが正常に動作すること
- リクエスト/レスポンスが見やすく整形されること
- タイムスタンプと経過時間が正確に表示されること
- デバッグログが問題特定に有用であること
- 全テストが成功すること（27/27）
- ドキュメント更新（README.md、チュートリアルへの追記）

---

### Task 3.2: バッチテスト機能 (5日間)

#### 目標
- CI/CD環境でMCP Inspector MCPを使用した自動テストを実現
- 回帰テストの自動化により品質保証を強化
- テスト結果の可視化と分析を容易化

#### 詳細作業項目

**3.2.1 テスト定義フォーマット（YAML/JSON）の設計 (8時間)**

**テスト定義ファイル仕様:**

YAML形式:
```yaml
# test_suite.yaml
name: "Fundamental Analysis Server Test Suite"
version: "1.0"
description: "Comprehensive test suite for fundamental analysis server"

config:
  timeout_ms: 30000
  retry_count: 1
  fail_fast: false  # 1つ失敗してもテスト継続
  parallel: true    # 並列実行

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
      - type: "error_rate"
        operator: "<"
        expected: 0.05

  - name: "Get Tools List"
    description: "Verify server provides expected tools"
    tool: "tools_list"
    server: "fundamental_analysis"
    arguments: {}
    assertions:
      - type: "field_exists"
        field: "tools"
      - type: "array_length"
        field: "tools"
        operator: ">"
        expected: 0
      - type: "contains"
        field: "tools[*].name"
        expected: "calculate_rsi"

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

  - name: "Error Handling: Invalid Symbol"
    description: "Verify error handling for invalid input"
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

JSON形式（YAMLと同等）:
```json
{
  "name": "Fundamental Analysis Server Test Suite",
  "version": "1.0",
  "description": "Comprehensive test suite for fundamental analysis server",
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

**アサーションタイプ一覧:**
- `status`: ステータス値の検証
- `field_exists`: フィールドの存在確認
- `field_equals`: フィールドの値が期待値と一致
- `field_not_equals`: フィールドの値が期待値と不一致
- `array_length`: 配列長の検証
- `contains`: 配列が特定値を含むか
- `response_time`: レスポンスタイムの検証
- `error_rate`: エラー率の検証
- `error_type`: エラータイプの検証
- `error_message_contains`: エラーメッセージに特定文字列を含むか
- `json_contains_key`: JSON内のキーの存在確認
- `json_value_type`: JSON値の型検証
- `json_value_range`: 数値範囲の検証

**実装詳細:**
- `src/models/test_definition.rs`: テスト定義データモデル（約300行）
  - `TestSuite`構造体
  - `TestCase`構造体
  - `Assertion`構造体（enum）
  - Deserialize実装（YAML/JSON）

**依存クレート:**
- `serde_yaml = "0.9"`: YAML パース
- `serde_json = "1"`: JSON パース

**テスト:**
- YAML パーステスト（6件）
- JSON パーステスト（6件）
- バリデーションテスト（5件）

**3.2.2 テスト実行エンジンの実装 (12時間)**

**実装内容:**
- テスト定義ファイルの読み込み
- テストケースの順次/並列実行
- アサーションの評価
- テスト結果の収集
- テスト失敗時のリトライロジック

**実装詳細:**
- `src/services/test_executor.rs`: テスト実行エンジン（約600行）
  - `TestExecutor`構造体
  - `run_test_suite()`メソッド: テストスイート実行
  - `run_test_case()`メソッド: 個別テストケース実行
  - `evaluate_assertion()`メソッド: アサーション評価
  - `retry_test()`メソッド: リトライロジック
  - 並列実行サポート（`tokio::task::JoinSet`）

**アサーション評価ロジック:**
```rust
impl TestExecutor {
    async fn evaluate_assertion(
        &self,
        assertion: &Assertion,
        response: &serde_json::Value,
        metadata: &TestMetadata,
    ) -> AssertionResult {
        match assertion {
            Assertion::FieldExists { field } => {
                // フィールドの存在確認（JSONPath使用）
                let exists = jsonpath::select(response, field)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);

                AssertionResult {
                    assertion_type: "field_exists".to_string(),
                    passed: exists,
                    expected: field.clone(),
                    actual: format!("exists: {}", exists),
                    message: if exists {
                        format!("Field '{}' exists", field)
                    } else {
                        format!("Field '{}' does not exist", field)
                    },
                }
            }

            Assertion::ResponseTime { operator, expected } => {
                // レスポンスタイム検証
                let actual_ms = metadata.elapsed_ms;
                let passed = match operator.as_str() {
                    "<" => actual_ms < *expected,
                    "<=" => actual_ms <= *expected,
                    ">" => actual_ms > *expected,
                    ">=" => actual_ms >= *expected,
                    "==" => actual_ms == *expected,
                    _ => false,
                };

                AssertionResult {
                    assertion_type: "response_time".to_string(),
                    passed,
                    expected: format!("{} {} ms", operator, expected),
                    actual: format!("{} ms", actual_ms),
                    message: if passed {
                        format!("Response time {} ms is {} {} ms", actual_ms, operator, expected)
                    } else {
                        format!("Response time {} ms is not {} {} ms", actual_ms, operator, expected)
                    },
                }
            }

            // その他のアサーションタイプ...
        }
    }
}
```

**依存クレート:**
- `jsonpath-rust = "0.3"`: JSONPath評価
- `regex = "1"`: 正規表現マッチング

**テスト:**
- テスト実行テスト（10件）
- アサーション評価テスト（15件）
- リトライロジックテスト（5件）

**3.2.3 CI/CD統合サポート (6時間)**

**実装内容:**
- 終了コード（Exit Code）の適切な設定
  - 全テスト成功: 0
  - テスト失敗: 1
  - 設定エラー: 2
  - 実行エラー: 3
- 環境変数による設定オーバーライド
- CI/CD向けのログフォーマット（簡潔な出力）

**GitHub Actions統合例:**
```yaml
# .github/workflows/mcp_server_test.yml
name: MCP Server Integration Test

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

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Build MCP Inspector
        run: |
          cd mcp_inspector_mcp
          cargo build --release

      - name: Start MCP Server
        run: |
          # 検査対象のMCPサーバーを起動
          ./target/release/fundamental_analysis_server &
          sleep 2

      - name: Run Batch Tests
        run: |
          cd mcp_inspector_mcp
          ./target/release/mcp_inspector_mcp batch-test \
            --test-file tests/test_suite.yaml \
            --report-format junit \
            --report-output test-results/junit.xml

      - name: Publish Test Results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: test-results/junit.xml
```

**GitLab CI統合例:**
```yaml
# .gitlab-ci.yml
stages:
  - test

mcp_integration_test:
  stage: test
  image: rust:latest
  script:
    - cd mcp_inspector_mcp
    - cargo build --release
    - ./target/release/fundamental_analysis_server &
    - sleep 2
    - ./target/release/mcp_inspector_mcp batch-test \
        --test-file tests/test_suite.yaml \
        --report-format junit \
        --report-output test-results/junit.xml
  artifacts:
    when: always
    reports:
      junit: test-results/junit.xml
```

**実装詳細:**
- `src/main.rs`: `batch-test`サブコマンド追加
- CI/CD向けログフォーマッタ
- 環境変数によるタイムアウト設定（`MCP_TEST_TIMEOUT_MS`）

**テスト:**
- CI/CD統合テスト（3件）

**3.2.4 JUnit XML形式レポート出力 (8時間)**

**実装内容:**
- JUnit XML形式でのテスト結果出力
- テストスイート、テストケース、アサーションの階層構造
- 失敗時のスタックトレース出力
- 実行時間の記録

**JUnit XML出力例:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Fundamental Analysis Server Test Suite" tests="4" failures="1" errors="0" time="1.234">
  <testsuite name="Fundamental Analysis Server Test Suite" tests="4" failures="1" errors="0" time="1.234" timestamp="2025-11-27T10:30:45Z">

    <testcase name="Server Health Check" classname="fundamental_analysis.health" time="0.123">
      <system-out>
        Server: fundamental_analysis
        Status: healthy
        Response Time: 123ms
      </system-out>
    </testcase>

    <testcase name="Get Tools List" classname="fundamental_analysis.tools" time="0.234">
      <system-out>
        Server: fundamental_analysis
        Tools Found: 5
        Tools: [calculate_rsi, calculate_dcf, get_financial_ratios, ...]
      </system-out>
    </testcase>

    <testcase name="Calculate RSI for AAPL" classname="fundamental_analysis.tools.calculate_rsi" time="0.456">
      <system-out>
        Server: fundamental_analysis
        Tool: calculate_rsi
        Symbol: AAPL
        Result: {"rsi": 65.3, "status": "overbought"}
      </system-out>
    </testcase>

    <testcase name="Error Handling: Invalid Symbol" classname="fundamental_analysis.tools.calculate_rsi" time="0.421">
      <failure message="Assertion failed: error_type" type="AssertionError">
        Expected error type: ServerError
        Actual error type: InvalidResponse

        Full Error:
        {
          "error": {
            "type": "InvalidResponse",
            "message": "Symbol 'INVALID_SYMBOL_12345' not found"
          }
        }
      </failure>
      <system-out>
        Server: fundamental_analysis
        Tool: calculate_rsi
        Symbol: INVALID_SYMBOL_12345
      </system-out>
    </testcase>

  </testsuite>
</testsuites>
```

**実装詳細:**
- `src/services/report_generator.rs`: レポート生成（約400行）
  - `ReportGenerator`構造体
  - `generate_junit_report()`メソッド
  - XML生成（`quick-xml`クレート使用）

**依存クレート:**
- `quick-xml = "0.31"`: XML生成

**テスト:**
- JUnit XML生成テスト（8件）

**3.2.5 テスト結果の集計と可視化 (6時間)**

**実装内容:**
- コンソール出力での結果サマリー
- HTMLレポート生成（オプション）
- テスト結果のJSON形式出力

**コンソール出力例:**
```
╔════════════════════════════════════════════════════════════════╗
║         MCP Inspector Batch Test Results                      ║
╚════════════════════════════════════════════════════════════════╝

Test Suite: Fundamental Analysis Server Test Suite
Duration: 1.234s

Tests: 4
  ✅ Passed: 3
  ❌ Failed: 1
  ⏭️  Skipped: 0

Success Rate: 75.0%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Test Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Server Health Check (123ms)
   All assertions passed

✅ Get Tools List (234ms)
   All assertions passed

✅ Calculate RSI for AAPL (456ms)
   All assertions passed

❌ Error Handling: Invalid Symbol (421ms)
   Assertion failed: error_type
   Expected: ServerError
   Actual: InvalidResponse

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Assertions: 12
  ✅ Passed: 11
  ❌ Failed: 1

Average Response Time: 308.5ms
Fastest Test: Server Health Check (123ms)
Slowest Test: Calculate RSI for AAPL (456ms)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**HTMLレポート（オプション）:**
- Bootstrap/Tailwindベースのレスポンシブデザイン
- テスト結果の視覚的な表示
- フィルタリングとソート機能
- グラフ（成功率、レスポンスタイム分布）

**JSON形式出力例:**
```json
{
  "test_suite": {
    "name": "Fundamental Analysis Server Test Suite",
    "version": "1.0",
    "timestamp": "2025-11-27T10:30:45Z",
    "duration_ms": 1234,
    "summary": {
      "total_tests": 4,
      "passed": 3,
      "failed": 1,
      "skipped": 0,
      "success_rate": 0.75
    },
    "tests": [
      {
        "name": "Server Health Check",
        "status": "passed",
        "duration_ms": 123,
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
}
```

**実装詳細:**
- `src/services/result_formatter.rs`: 結果フォーマッター（約300行）
  - コンソール出力
  - JSON出力
  - HTML生成（オプション）

**依存クレート:**
- `prettytable-rs = "0.10"`: テーブル表示
- `askama = "0.12"`: HTMLテンプレート（オプション）

**テスト:**
- フォーマット出力テスト（6件）

#### 成果物
- `src/models/test_definition.rs`: テスト定義（約300行）
- `src/services/test_executor.rs`: テスト実行エンジン（約600行）
- `src/services/report_generator.rs`: レポート生成（約400行）
- `src/services/result_formatter.rs`: 結果フォーマッター（約300行）
- `src/main.rs`: `batch-test`サブコマンド追加
- テストファイル: `tests/task3_2_batch_test.rs`（約400行、計53テスト）
- サンプルテスト定義: `examples/test_suites/basic_test.yaml`, `examples/test_suites/advanced_test.yaml`
- CI/CD統合例: `.github/workflows/mcp_test.yml`, `.gitlab-ci.yml`

#### 成功基準
- YAML/JSON形式のテスト定義が正しくパースされること
- テストが順次/並列で実行されること
- アサーションが正確に評価されること
- JUnit XML形式のレポートが正しく生成されること
- CI/CDパイプラインに統合できること
- 全テストが成功すること（53/53）
- ドキュメント更新（バッチテスト機能の詳細ガイド）

---

### Task 3.3: インタラクティブモード改善 (4日間)

#### 目標
- ユーザーの操作効率を30%向上させる
- コマンド入力のミスを削減
- JSON入力の利便性を向上

#### 詳細作業項目

**3.3.1 タブ補完機能の実装 (10時間)**

**実装内容:**
- MCPツール名のタブ補完
- サーバー名のタブ補完
- 引数名のタブ補完
- ファイルパスのタブ補完

**補完候補の取得:**
- ツール名: `tools_list`の結果をキャッシュして使用
- サーバー名: `.inspector/config.json`から取得
- 引数名: ツールのinput_schemaから取得

**実装詳細:**
- `rustyline`クレートの`Completer`トレイト実装
- `src/interactive/completer.rs`: 新規ファイル（約450行）
  - `McpCompleter`構造体
  - `complete()`メソッド: 補完候補を生成
  - コンテキストに応じた補完ロジック

**補完例:**
```
> tools_l[TAB]
> tools_list

> tools_list --server fund[TAB]
> tools_list --server fundamental_analysis

> tools_call --server fundamental_analysis --tool calc[TAB]
> tools_call --server fundamental_analysis --tool calculate_rsi
```

**依存クレート:**
- `rustyline = "12"`: REPLライブラリ（タブ補完、履歴機能）

**テスト:**
- 補完候補生成テスト（12件）

**3.3.2 コマンド履歴の保存と再利用 (6時間)**

**実装内容:**
- コマンド履歴の永続化
- 上下矢印キーで履歴を辿る
- Ctrl+Rで履歴検索
- 履歴ファイルの自動管理（最大1000件）

**履歴ファイル:**
- 保存場所: `.inspector/history.txt`
- フォーマット: プレーンテキスト（1行1コマンド）
- ローテーション: 1000件を超えると古いものから削除

**実装詳細:**
- `rustyline`の`History`機能を使用
- `src/interactive/history_manager.rs`: 新規ファイル（約200行）
  - `HistoryManager`構造体
  - `load_history()`メソッド
  - `save_history()`メソッド
  - 自動ローテーション

**使用例:**
```
> tools_list --server fundamental_analysis
[上矢印キーで前のコマンドを呼び出し]
> tools_list --server fundamental_analysis
[編集して再実行]
> tools_list --server technical_analysis
```

**履歴検索例:**
```
[Ctrl+R入力]
(reverse-i-search)`rsi': tools_call --server fundamental_analysis --tool calculate_rsi --arguments '{"symbol":"AAPL","period":14}'
```

**テスト:**
- 履歴保存テスト（4件）
- 履歴読み込みテスト（4件）
- ローテーションテスト（2件)

**3.3.3 JSON入力の構文チェック (6時間)**

**実装内容:**
- JSON入力のリアルタイム構文チェック
- エラー箇所の明示
- 修正提案の表示

**実装詳細:**
- `src/interactive/json_validator.rs`: 新規ファイル（約250行）
  - `JsonValidator`構造体
  - `validate()`メソッド: JSON構文検証
  - `highlight_error()`メソッド: エラー箇所の強調表示
  - `suggest_fix()`メソッド: 修正提案

**検証例:**
```
> tools_call --server fundamental_analysis --tool calculate_rsi --arguments '{"symbol":"AAPL","period":14'
❌ JSON Syntax Error: EOF while parsing an object at line 1 column 33

{"symbol":"AAPL","period":14
                              ^ Expected '}' here

Suggestion: Add closing brace '}'
Corrected JSON: {"symbol":"AAPL","period":14}

Do you want to use the corrected JSON? [Y/n]: y
```

**一般的なJSONエラーの検出:**
- 閉じ括弧の欠如
- カンマの欠如/余分
- クォートの欠如
- エスケープ文字の誤り
- 型の不一致

**テスト:**
- JSON検証テスト（10件）
- エラー検出テスト（8件）
- 修正提案テスト（6件）

**3.3.4 オートコンプリート機能 (6時間)**

**実装内容:**
- ツールinput_schemaに基づいた引数のオートコンプリート
- 型に応じた入力支援（string、number、boolean、array、object）
- デフォルト値の提案

**実装詳細:**
- `src/interactive/auto_complete.rs`: 新規ファイル（約350行）
  - `AutoCompleter`構造体
  - `suggest_arguments()`メソッド: 引数候補を提案
  - `generate_template()`メソッド: JSONテンプレート生成

**使用例:**
```
> tools_call --server fundamental_analysis --tool calculate_rsi
? Enter arguments (or press Enter for template):
[Enter押下]

Generated template based on input_schema:
{
  "symbol": "",      // string (required)
  "period": 14       // integer (optional, default: 14)
}

Edit template? [Y/n]: y
[エディタが開く]
```

**input_schemaからのテンプレート生成:**
```json
// Input Schema:
{
  "type": "object",
  "properties": {
    "symbol": {
      "type": "string",
      "description": "Stock ticker symbol"
    },
    "period": {
      "type": "integer",
      "description": "RSI period",
      "default": 14
    }
  },
  "required": ["symbol"]
}

// Generated Template:
{
  "symbol": "",      // string (required) - Stock ticker symbol
  "period": 14       // integer (optional) - RSI period
}
```

**テスト:**
- テンプレート生成テスト（8件）
- 引数提案テスト（6件）

#### 成果物
- `src/interactive/completer.rs`: タブ補完（約450行）
- `src/interactive/history_manager.rs`: 履歴管理（約200行）
- `src/interactive/json_validator.rs`: JSON検証（約250行）
- `src/interactive/auto_complete.rs`: オートコンプリート（約350行）
- `src/interactive/mod.rs`: インタラクティブモードエントリーポイント（約150行）
- テストファイル: `tests/task3_3_interactive_mode_test.rs`（約350行、計46テスト）

#### 成功基準
- タブ補完が正常に動作すること
- コマンド履歴が保存・再利用できること
- JSON構文エラーが適切に検出されること
- オートコンプリートが有用であること
- ユーザー体験が向上すること（操作効率30%向上をユーザーテストで検証）
- 全テストが成功すること（46/46）
- ドキュメント更新（インタラクティブモード使用ガイド）

---

### Task 3.4: パフォーマンスモニタリング (3日間)

#### 目標
- パフォーマンス問題の早期発見
- レスポンスタイムの継続的な監視
- ボトルネック検出の自動化

#### 詳細作業項目

**3.4.1 パフォーマンスメトリクス収集 (8時間)**

**収集メトリクス:**
- **レスポンスタイム**: ツール実行時間（min、max、avg、p50、p95、p99）
- **スループット**: 1秒あたりのリクエスト数
- **エラー率**: エラー発生率（%）
- **キャッシュヒット率**: キャッシュの効率
- **接続プール利用率**: 接続の再利用率
- **メモリ使用量**: プロセスメモリ使用量
- **CPU使用率**: CPU利用率（オプション）

**実装詳細:**
- `src/services/metrics_collector.rs`: 新規ファイル（約500行）
  - `MetricsCollector`構造体
  - `record_request()`メソッド: リクエストメトリクス記録
  - `record_response()`メソッド: レスポンスメトリクス記録
  - `calculate_percentiles()`メソッド: パーセンタイル計算
  - 循環バッファによる履歴管理（最新10,000件）

**メトリクスデータ構造:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub server_name: String,
    pub tool_name: String,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub status: MetricStatus,  // Success/Error
    pub cache_hit: bool,
    pub connection_reused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub server_name: String,
    pub tool_name: Option<String>,
    pub time_window: TimeWindow,
    pub response_time: ResponseTimeStats,
    pub throughput: f64,  // requests/sec
    pub error_rate: f64,  // 0.0 - 1.0
    pub cache_hit_rate: f64,  // 0.0 - 1.0
    pub connection_reuse_rate: f64,  // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeStats {
    pub min: u64,
    pub max: u64,
    pub avg: f64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}
```

**依存クレート:**
- `hdrhistogram = "7"`: パーセンタイル計算
- `sysinfo = "0.30"`: システムメトリクス（メモリ、CPU）

**テスト:**
- メトリクス記録テスト（6件）
- パーセンタイル計算テスト（4件）
- 集計テスト（5件）

**3.4.2 レスポンスタイムの測定と記録 (6時間)**

**実装内容:**
- ツール実行時間の自動測定
- サーバーごと、ツールごとのレスポンスタイム記録
- 時系列データの保存

**実装詳細:**
- `src/services/inspector.rs`に統合
- すべてのツール実行時に自動記録
- `MetricsCollector`への委譲

**記録例:**
```
[2025-11-27T10:30:45.123Z] fundamental_analysis | tools_list | 234ms | Success | Cache: Hit
[2025-11-27T10:30:46.456Z] fundamental_analysis | tools_call | 456ms | Success | Cache: Miss
[2025-11-27T10:30:47.789Z] fundamental_analysis | health_check | 123ms | Success | Cache: Miss
[2025-11-27T10:30:50.012Z] technical_analysis | tools_list | 567ms | Error | Cache: Miss
```

**テスト:**
- レスポンスタイム測定テスト（5件）

**3.4.3 統計レポートの生成 (8時間)**

**実装内容:**
- 時間範囲別の統計レポート（過去1時間、24時間、7日間）
- サーバーごと、ツールごとの統計
- 比較レポート（前期間との比較）

**レポート例:**
```
╔════════════════════════════════════════════════════════════════╗
║         Performance Metrics Report                             ║
╚════════════════════════════════════════════════════════════════╝

Server: fundamental_analysis
Time Window: Last 24 hours (2025-11-26 10:00:00 - 2025-11-27 10:00:00)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Response Time Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tool: tools_list
  Total Requests: 1,234
  Min: 87ms
  Max: 2,345ms
  Avg: 234.5ms
  P50: 198ms
  P95: 456ms
  P99: 789ms

Tool: tools_call
  Total Requests: 567
  Min: 123ms
  Max: 5,678ms
  Avg: 678.9ms
  P50: 456ms
  P95: 1,234ms
  P99: 2,345ms

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Throughput & Error Rate
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Throughput: 0.75 requests/sec
Error Rate: 2.3% (42 errors / 1,801 total)
Cache Hit Rate: 78.5%
Connection Reuse Rate: 92.1%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Comparison with Previous Period
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Response Time (Avg): 234.5ms (↓ 12.3% from 267.8ms)
Error Rate: 2.3% (↓ 0.5% from 2.8%)
Throughput: 0.75 req/s (↑ 15.4% from 0.65 req/s)
```

**実装詳細:**
- `src/services/report_service.rs`: レポート生成（約400行）
  - `generate_performance_report()`メソッド
  - `compare_periods()`メソッド: 期間比較
  - テキスト/JSON/CSV形式の出力

**テスト:**
- レポート生成テスト（8件）

**3.4.4 ボトルネック検出機能 (6時間)**

**実装内容:**
- 自動的なボトルネック検出
- 異常なレスポンスタイムのアラート
- パフォーマンス劣化の検出

**検出ロジック:**
- **異常なレスポンスタイム**: P95が過去平均の150%を超える
- **エラー率の上昇**: エラー率が過去平均の200%を超える
- **スループット低下**: スループットが過去平均の50%を下回る

**アラート例:**
```
⚠️  PERFORMANCE ALERT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Detected: Slow Response Time
Server: fundamental_analysis
Tool: tools_call
Time: 2025-11-27T10:30:45Z

Current P95: 2,345ms
Historical P95 (7d avg): 1,234ms
Deviation: +90.0%

Possible Causes:
  • Server performance degradation
  • Network latency
  • Heavy server load
  • Database slow query

Recommended Actions:
  1. Check server health with health_check tool
  2. Review server logs with logging_messages tool
  3. Verify network connectivity
  4. Consider scaling server resources
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**実装詳細:**
- `src/services/bottleneck_detector.rs`: ボトルネック検出（約300行）
  - `BottleneckDetector`構造体
  - `detect_anomalies()`メソッド
  - `generate_alert()`メソッド
  - `suggest_actions()`メソッド

**テスト:**
- ボトルネック検出テスト（6件）

#### 成果物
- `src/services/metrics_collector.rs`: メトリクス収集（約500行）
- `src/services/report_service.rs`: レポート生成（約400行）
- `src/services/bottleneck_detector.rs`: ボトルネック検出（約300行）
- `src/models/metrics.rs`: メトリクスデータモデル（約200行）
- テストファイル: `tests/task3_4_performance_monitoring_test.rs`（約300行、計34テスト）

#### 成功基準
- メトリクスが正確に収集されること
- 統計レポートが正しく生成されること
- ボトルネックが検出されること
- パフォーマンス問題の早期発見が可能になること
- 全テストが成功すること（34/34）
- ドキュメント更新（パフォーマンスモニタリングガイド）

---

### Task 3.5: 構成管理の拡張 (2日間)

#### 目標
- 環境別設定の容易化
- 設定の再利用性向上
- 設定管理の柔軟性向上

#### 詳細作業項目

**3.5.1 プロファイル機能（dev/staging/prod）の実装 (8時間)**

**実装内容:**
- プロファイルベースの設定管理
- 環境別設定ファイル（`config.dev.json`, `config.staging.json`, `config.prod.json`）
- プロファイル切替機能

**設定ファイル構造:**
```
.inspector/
├── config.json             # デフォルト設定
├── config.dev.json         # 開発環境設定
├── config.staging.json     # ステージング環境設定
├── config.prod.json        # 本番環境設定
└── profiles/               # 追加プロファイル（オプション）
    ├── config.ci.json
    └── config.local.json
```

**プロファイル設定例:**

`config.dev.json`:
```json
{
  "profile": "dev",
  "servers": [
    {
      "name": "fundamental_analysis",
      "transport": "stdio",
      "command": "C:/dev/fa/target/debug/fa.exe",
      "args": ["--debug"],
      "env": {
        "LOG_LEVEL": "debug",
        "API_ENDPOINT": "http://localhost:8080"
      }
    }
  ],
  "logging": {
    "backend": "memory",
    "max_logs": 1000
  },
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 0
  },
  "debug": {
    "verbose": true,
    "color_output": true
  }
}
```

`config.prod.json`:
```json
{
  "profile": "prod",
  "servers": [
    {
      "name": "fundamental_analysis",
      "transport": "stdio",
      "command": "C:/production/fa/fa.exe",
      "args": [],
      "env": {
        "LOG_LEVEL": "info",
        "API_ENDPOINT": "https://api.production.com"
      }
    }
  ],
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs_prod.db",
    "max_logs": 100000
  },
  "execution_config": {
    "tool_timeout_ms": 30000,
    "connection_timeout_ms": 5000,
    "retry_count": 2,
    "auto_retry_on_timeout": true
  },
  "debug": {
    "verbose": false,
    "color_output": false
  }
}
```

**プロファイル選択方法:**
1. CLI引数: `--profile dev`
2. 環境変数: `MCP_PROFILE=dev`
3. デフォルト: `config.json`

**実装詳細:**
- `src/models/profile_config.rs`: プロファイル設定（約250行）
  - `ProfileConfig`構造体
  - `load_profile()`メソッド
  - プロファイル検証
- `src/services/config_manager.rs`: プロファイル管理機能追加（+150行）

**テスト:**
- プロファイル読み込みテスト（6件）
- プロファイル切替テスト（4件）

**3.5.2 環境別設定切り替え (4時間)**

**実装内容:**
- 実行時のプロファイル切替
- プロファイル一覧表示
- プロファイル検証

**CLI例:**
```bash
# プロファイル指定で起動
mcp_inspector_mcp --profile dev

# 環境変数で指定
MCP_PROFILE=staging mcp_inspector_mcp

# プロファイル一覧表示
mcp_inspector_mcp --list-profiles

# プロファイル検証
mcp_inspector_mcp --validate-profile prod
```

**プロファイル一覧表示例:**
```
Available Profiles:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  dev        - Development environment
               Servers: 2
               Logging: Memory backend
               Debug: Enabled

  staging    - Staging environment
               Servers: 3
               Logging: Persistent backend
               Debug: Disabled

  prod       - Production environment
               Servers: 5
               Logging: Persistent backend
               Debug: Disabled

  default    - Default configuration
               Servers: 1
               Logging: Memory backend
               Debug: Disabled

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Current Profile: dev
```

**実装詳細:**
- `src/main.rs`: プロファイル選択ロジック追加
- CLI引数パース拡張

**テスト:**
- プロファイル切替テスト（4件）

**3.5.3 設定のインポート/エクスポート (6時間)**

**実装内容:**
- 設定のエクスポート（JSON/YAML形式）
- 設定のインポート
- 設定のバリデーション
- 設定の差分表示

**CLI例:**
```bash
# 設定のエクスポート
mcp_inspector_mcp config export --output config_backup.json

# YAMLフォーマットでエクスポート
mcp_inspector_mcp config export --output config_backup.yaml --format yaml

# 設定のインポート
mcp_inspector_mcp config import --input config_backup.json

# インポート前に差分表示
mcp_inspector_mcp config import --input config_backup.json --dry-run

# 設定のバリデーション
mcp_inspector_mcp config validate --input config_backup.json
```

**差分表示例:**
```
Configuration Diff:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Servers:
  + Added: technical_analysis
  ~ Modified: fundamental_analysis
      command: C:/old/fa.exe → C:/new/fa.exe
  - Removed: deprecated_server

Logging:
  ~ Modified:
      backend: memory → persistent
      db_path: (not set) → ./data/logs.db

Execution Config:
  ~ Modified:
      tool_timeout_ms: 30000 → 60000
      retry_count: 0 → 2

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Apply these changes? [y/N]:
```

**実装詳細:**
- `src/services/config_import_export.rs`: インポート/エクスポート（約400行）
  - `export_config()`メソッド
  - `import_config()`メソッド
  - `validate_config()`メソッド
  - `diff_configs()`メソッド

**依存クレート:**
- `similar = "2"`: 差分計算

**テスト:**
- エクスポートテスト（4件）
- インポートテスト（5件）
- 差分表示テスト（4件）

**3.5.4 設定テンプレート機能 (6時間)**

**実装内容:**
- プリセットテンプレートの提供
- カスタムテンプレートの作成
- テンプレートからの設定生成

**プリセットテンプレート:**
1. **minimal**: 最小限の設定
2. **development**: 開発環境向け設定
3. **production**: 本番環境向け設定
4. **ci**: CI/CD環境向け設定

**CLI例:**
```bash
# テンプレート一覧表示
mcp_inspector_mcp config template list

# テンプレートから設定生成
mcp_inspector_mcp config template apply --template development --output config.dev.json

# カスタムテンプレートの作成
mcp_inspector_mcp config template create --name my_template --from config.json

# テンプレートの表示
mcp_inspector_mcp config template show --template production
```

**テンプレート例:**

`minimal`テンプレート:
```json
{
  "servers": [
    {
      "name": "example_server",
      "transport": "stdio",
      "command": "/path/to/server",
      "args": [],
      "env": {}
    }
  ],
  "logging": {
    "backend": "memory",
    "max_logs": 1000
  }
}
```

`production`テンプレート:
```json
{
  "servers": [],
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs.db",
    "max_logs": 100000
  },
  "execution_config": {
    "tool_timeout_ms": 30000,
    "connection_timeout_ms": 5000,
    "retry_count": 2,
    "auto_retry_on_timeout": true
  },
  "debug": {
    "verbose": false,
    "color_output": false
  },
  "performance": {
    "metrics_enabled": true,
    "report_interval_sec": 3600
  }
}
```

**実装詳細:**
- `src/services/config_template.rs`: テンプレート管理（約350行）
  - プリセットテンプレートの定義
  - テンプレート適用
  - カスタムテンプレート管理

**テストupgrade:**
- テンプレート適用テスト（6件）

#### 成果物
- `src/models/profile_config.rs`: プロファイル設定（約250行）
- `src/services/config_import_export.rs`: インポート/エクスポート（約400行）
- `src/services/config_template.rs`: テンプレート管理（約350行）
- `src/services/config_manager.rs`: プロファイル管理機能追加（+150行）
- テストファイル: `tests/task3_5_config_management_test.rs`（約250行、計33テスト）
- テンプレートファイル: `templates/minimal.json`, `templates/development.json`, `templates/production.json`, `templates/ci.json`

#### 成功基準
- プロファイル切替が正常に動作すること
- 設定のインポート/エクスポートが正常に動作すること
- テンプレート機能が有用であること
- 設定管理が容易になること
- 全テストが成功すること（33/33）
- ドキュメント更新（構成管理ガイド）

---

### Task 3.6: Phase 3統合テストとリリース (3日間)

#### 目標
- Phase 3で追加されたすべての機能の統合検証
- パフォーマンステスト実施
- ドキュメント最終更新
- v0.4.0リリース準備

#### 詳細作業項目

**3.6.1 全機能の統合テスト (10時間)**

**テスト項目:**
1. **デバッグモード統合テスト**
   - `--verbose`フラグの動作確認
   - リクエスト/レスポンス表示の検証
   - タイミング情報の正確性確認
   - ログ出力の検証

2. **バッチテスト統合テスト**
   - YAML/JSONテスト定義の実行
   - アサーション評価の正確性
   - JUnit XMLレポート生成
   - CI/CD統合動作確認

3. **インタラクティブモード統合テスト**
   - タブ補完機能の動作確認
   - コマンド履歴機能の検証
   - JSON構文チェックの検証
   - オートコンプリートの動作確認

4. **パフォーマンスモニタリング統合テスト**
   - メトリクス収集の正確性
   - レポート生成の検証
   - ボトルネック検出の動作確認

5. **構成管理統合テスト**
   - プロファイル切替の検証
   - インポート/エクスポートの動作確認
   - テンプレート機能の検証

**実装詳細:**
- `tests/phase3_integration_test.rs`: 統合テスト（約800行、計45テスト）
- 各機能の組み合わせテスト
- エンドツーエンドシナリオテスト

**テストシナリオ例:**
```rust
#[tokio::test]
async fn test_full_debugging_workflow() {
    // 1. デバッグモードで起動
    let config = load_config_with_verbose(true);

    // 2. ツール一覧取得（リクエスト/レスポンス表示確認）
    let result = tools_list(&config, "fundamental_analysis").await;
    assert!(result.is_ok());

    // 3. ツール実行（タイミング情報確認）
    let start = Instant::now();
    let result = tools_call(&config, "fundamental_analysis", "calculate_rsi", args).await;
    let elapsed = start.elapsed();
    assert!(result.is_ok());
    assert!(elapsed.as_millis() > 0);

    // 4. パフォーマンスレポート生成
    let metrics = get_metrics(&config, "fundamental_analysis").await;
    assert!(metrics.response_time.avg > 0.0);
}
```

**テスト成功基準:**
- 全統合テストが成功すること（45/45）
- エッジケースでもクラッシュしないこと
- エラーメッセージがわかりやすいこと

**3.6.2 パフォーマンステスト (6時間)**

**テスト項目:**
1. **レスポンスタイムテスト**
   - 各ツールのレスポンスタイム測定
   - 目標値との比較

2. **スループットテスト**
   - 連続リクエスト処理能力測定
   - 並列リクエスト処理能力測定

3. **メモリ使用量テスト**
   - 長時間実行時のメモリリーク確認
   - メトリクス収集時のメモリ使用量

4. **ストレステスト**
   - 大量リクエスト処理
   - タイムアウト設定の検証

**パフォーマンス目標:**
- ツール一覧取得: < 500ms（キャッシュミス時）
- ツール実行: < 3000ms（サーバー依存）
- ヘルスチェック: < 200ms
- バッチテスト実行: < 10秒（10テストケース）
- メモリ使用量: < 100MB（通常動作時）

**実装詳細:**
- `tests/phase3_performance_test.rs`: パフォーマンステスト（約400行、計20テスト）
- ベンチマークツール（`criterion`使用）

**依存クレート:**
- `criterion = "0.5"`: ベンチマーク

**テスト成功基準:**
- すべてのパフォーマンス目標を達成すること
- Phase 2比でパフォーマンス劣化がないこと
- メモリリークがないこと

**3.6.3 ドキュメント更新 (8時間)**

**更新対象:**
1. **README.md**
   - Phase 3機能の追加
   - 使用例の追加
   - トラブルシューティングの更新

2. **チュートリアル**
   - Advanced Usageチュートリアルの拡張
   - デバッグモードの使用例追加
   - バッチテストガイドの追加

3. **API仕様書**
   - 新規追加機能のAPI仕様記載
   - パフォーマンスメトリクスの説明

4. **新規ドキュメント**
   - `docs/guides/debug-mode.md`: デバッグモード詳細ガイド（約3,000文字）
   - `docs/guides/batch-testing.md`: バッチテスト完全ガイド（約4,000文字）
   - `docs/guides/interactive-mode.md`: インタラクティブモード使用ガイド（約2,500文字）
   - `docs/guides/performance-monitoring.md`: パフォーマンスモニタリングガイド（約3,500文字）
   - `docs/guides/configuration-management.md`: 構成管理ガイド（約3,000文字）

**ドキュメント成功基準:**
- 全機能がドキュメント化されていること
- コード例が正確であること
- 新規ユーザーが理解できる内容であること

**3.6.4 CHANGELOG更新とリリースノート作成 (4時間)**

**CHANGELOG.md更新:**
- Phase 3の変更内容を追加
- 新機能の詳細説明
- 破壊的変更の明記（該当する場合）
- 移行ガイドの提供

**リリースノート作成:**
- v0.4.0のハイライト
- 主要な改善点の要約
- 既存ユーザー向けの情報
- 新規ユーザー向けの情報

**リリースノート例:**
```markdown
# Release v0.4.0 - Enterprise-Grade Features

## 🎉 Highlights

MCP Inspector MCP v0.4.0は、エンタープライズ環境で求められる高度な機能を追加し、プロフェッショナルツールとして完成しました。

### 🔍 Debug Mode
- `--verbose`フラグによる詳細デバッグ情報
- リクエスト/レスポンスの整形表示
- タイムスタンプと経過時間の表示
- 構造化ログ（JSON形式）のサポート

### 🧪 Batch Testing
- YAML/JSON形式のテスト定義
- CI/CD統合サポート
- JUnit XML形式レポート出力
- 自動アサーション評価

### 💬 Enhanced Interactive Mode
- タブ補完機能
- コマンド履歴の保存と再利用
- JSON入力の構文チェック
- オートコンプリート機能

### 📊 Performance Monitoring
- パフォーマンスメトリクス収集
- レスポンスタイム測定と記録
- 統計レポート生成
- ボトルネック検出

### ⚙️ Advanced Configuration Management
- プロファイル機能（dev/staging/prod）
- 環境別設定切り替え
- 設定のインポート/エクスポート
- 設定テンプレート機能

## 📈 Performance Improvements
- メトリクス収集のオーバーヘッド < 1%
- バッチテスト実行の高速化
- メモリ効率の改善

## 🐛 Bug Fixes
- (該当する場合、バグ修正を記載)

## 📚 Documentation
- 5つの新規ガイド追加（合計16,000文字）
- チュートリアルの拡張
- README.mdの更新

## 🔄 Migration Guide

v0.3.xからの移行は、後方互換性を維持しているため、設定変更なしで動作します。

新機能を使用する場合:
1. デバッグモード: `--verbose`フラグを追加
2. バッチテスト: テスト定義ファイルを作成
3. プロファイル: `.inspector/config.dev.json`等を作成

詳細は[Migration Guide](docs/migration/v0.3_to_v0.4.md)を参照してください。

## 🙏 Acknowledgments
- すべてのコントリビューターに感謝
- フィードバックを提供してくださったユーザーに感謝
```

**実装詳細:**
- `CHANGELOG.md`の更新
- `docs/releases/v0.4.0.md`: リリースノート作成
- `docs/migration/v0.3_to_v0.4.md`: 移行ガイド作成

**3.6.5 品質保証とレビュー (6時間)**

**品質保証項目:**
1. **コード品質**
   - `cargo clippy`警告ゼロ
   - `cargo fmt`フォーマット準拠
   - コードレビュー実施

2. **テストカバレッジ**
   - 単体テスト: 全機能カバー
   - 統合テスト: 主要シナリオカバー
   - パフォーマンステスト: 全目標達成

3. **ドキュメント品質**
   - 技術的正確性
   - 表記の統一性
   - リンク切れチェック

4. **リリース準備**
   - バージョン番号の更新（`Cargo.toml`）
   - Gitタグの作成
   - リリースビルドの作成

**実装詳細:**
- コードレビューチェックリスト
- ドキュメントレビューチェックリスト
- リリースチェックリスト

#### 成果物
- `tests/phase3_integration_test.rs`: 統合テスト（約800行、計45テスト）
- `tests/phase3_performance_test.rs`: パフォーマンステスト（約400行、計20テスト）
- `docs/guides/debug-mode.md`: デバッグモードガイド（約3,000文字）
- `docs/guides/batch-testing.md`: バッチテストガイド（約4,000文字）
- `docs/guides/interactive-mode.md`: インタラクティブモードガイド（約2,500文字）
- `docs/guides/performance-monitoring.md`: パフォーマンスモニタリングガイド（約3,500文字）
- `docs/guides/configuration-management.md`: 構成管理ガイド（約3,000文字）
- `docs/releases/v0.4.0.md`: リリースノート
- `docs/migration/v0.3_to_v0.4.md`: 移行ガイド
- 更新されたCHANGELOG.md
- 更新されたREADME.md

#### 成功基準
- 全統合テストが成功すること（45/45）
- 全パフォーマンステストが目標達成すること（20/20）
- ドキュメントが完全であること
- CHANGELOGが更新されていること
- リリースノートが作成されていること
- コード品質基準を満たすこと（Clippy警告ゼロ）
- v0.4.0リリース準備完了

---

## 成功基準（Phase 3全体）

### 定量的指標

**コード実装:**
- **新規コード行数**: 約10,000行
- **新規ファイル数**: 約25ファイル
- **修正ファイル数**: 約15ファイル
- **新規テスト数**: 約260テスト
- **テスト成功率**: 100%（260/260）
- **コードカバレッジ**: 85%以上

**ドキュメント:**
- **新規ドキュメント文字数**: 約19,000文字
  - デバッグモードガイド: 3,000文字
  - バッチテストガイド: 4,000文字
  - インタラクティブモードガイド: 2,500文字
  - パフォーマンスモニタリングガイド: 3,500文字
  - 構成管理ガイド: 3,000文字
  - リリースノート: 約1,500文字
  - 移行ガイド: 約1,500文字
- **更新ドキュメント**: README.md、CHANGELOG.md、チュートリアル

**パフォーマンス:**
- デバッグモードのオーバーヘッド: < 5%
- メトリクス収集のオーバーヘッド: < 1%
- バッチテスト実行速度: 10テストケース < 10秒
- メモリ使用量: < 100MB（通常動作時）

### 定性的指標

**技術的品質:**
- すべての新機能が堅牢に動作すること
- エラーハンドリングが適切であること
- コードが保守しやすいこと
- テストが信頼できること

**ユーザー体験:**
- デバッグ効率が向上すること（問題解決時間50%削減目標）
- インタラクティブモードの操作性が向上すること（操作効率30%向上目標）
- バッチテストがCI/CDに容易に統合できること
- ドキュメントがわかりやすいこと

**プロフェッショナル性:**
- エンタープライズ環境で使用できる品質であること
- CI/CD統合が実用的であること
- パフォーマンスモニタリングが有用であること
- 構成管理が柔軟であること

### ユーザビリティ指標

**新規ユーザーテスト:**
- デバッグモード使用開始: 5分以内（目標達成率: 80%以上）
- バッチテスト作成と実行: 30分以内（目標達成率: 70%以上）
- インタラクティブモード習得: 10分以内（目標達成率: 80%以上）
- プロファイル切替: 5分以内（目標達成率: 90%以上）

**ユーザー満足度:** 4.5/5.0以上（テスター評価）

---

## 技術要件

### 新規依存クレート

| クレート | バージョン | 用途 |
|---------|-----------|------|
| `colored` | 2 | ANSIカラー表示 |
| `tracing` | 0.1 | 構造化トレーシング |
| `tracing-subscriber` | 0.3 | トレーシングサブスクライバー |
| `tracing-appender` | 0.2 | ログファイル出力 |
| `serde_yaml` | 0.9 | YAML パース |
| `jsonpath-rust` | 0.3 | JSONPath評価 |
| `quick-xml` | 0.31 | XML生成 |
| `rustyline` | 12 | REPLライブラリ |
| `hdrhistogram` | 7 | パーセンタイル計算 |
| `sysinfo` | 0.30 | システムメトリクス |
| `similar` | 2 | 差分計算 |
| `prettytable-rs` | 0.10 | テーブル表示 |
| `askama` | 0.12 | HTMLテンプレート（オプション） |
| `criterion` | 0.5 | ベンチマーク（dev-dependencies） |

### Rust バージョン要件
- **最小バージョン**: 1.70
- **推奨バージョン**: 1.75以上

### ディレクトリ構造（Phase 3追加分）

```
mcp_inspector_mcp/
├── src/
│   ├── models/
│   │   ├── debug_config.rs         # デバッグ設定
│   │   ├── test_definition.rs      # テスト定義
│   │   ├── metrics.rs              # メトリクスデータモデル
│   │   └── profile_config.rs       # プロファイル設定
│   ├── services/
│   │   ├── debug_logger.rs         # デバッグロガー
│   │   ├── timing_tracker.rs       # タイミング追跡
│   │   ├── test_executor.rs        # テスト実行エンジン
│   │   ├── report_generator.rs     # レポート生成
│   │   ├── result_formatter.rs     # 結果フォーマッター
│   │   ├── metrics_collector.rs    # メトリクス収集
│   │   ├── report_service.rs       # レポートサービス
│   │   ├── bottleneck_detector.rs  # ボトルネック検出
│   │   ├── config_import_export.rs # インポート/エクスポート
│   │   └── config_template.rs      # テンプレート管理
│   ├── interactive/
│   │   ├── mod.rs                  # モジュールルート
│   │   ├── completer.rs            # タブ補完
│   │   ├── history_manager.rs      # 履歴管理
│   │   ├── json_validator.rs       # JSON検証
│   │   └── auto_complete.rs        # オートコンプリート
│   └── main.rs                     # エントリーポイント（拡張）
├── tests/
│   ├── task3_1_debug_mode_test.rs
│   ├── task3_2_batch_test.rs
│   ├── task3_3_interactive_mode_test.rs
│   ├── task3_4_performance_monitoring_test.rs
│   ├── task3_5_config_management_test.rs
│   ├── phase3_integration_test.rs
│   └── phase3_performance_test.rs
├── examples/
│   └── test_suites/
│       ├── basic_test.yaml
│       ├── advanced_test.yaml
│       └── ci_test.yaml
├── templates/
│   ├── minimal.json
│   ├── development.json
│   ├── production.json
│   └── ci.json
├── docs/
│   ├── guides/
│   │   ├── debug-mode.md
│   │   ├── batch-testing.md
│   │   ├── interactive-mode.md
│   │   ├── performance-monitoring.md
│   │   └── configuration-management.md
│   ├── releases/
│   │   └── v0.4.0.md
│   └── migration/
│       └── v0.3_to_v0.4.md
└── .github/workflows/
    └── mcp_test.yml                # CI/CD統合例
```

---

## 品質基準

### コードの品質
- **正確性**: すべての機能が仕様通りに動作する
- **堅牢性**: エッジケースやエラーケースを適切に処理する
- **保守性**: コードが理解しやすく、変更しやすい
- **テスト可能性**: すべての機能が単体テスト可能
- **パフォーマンス**: 性能目標を達成する

### ドキュメントの品質
- **正確性**: 技術的に正確で、実装と一致している
- **完全性**: すべての機能がドキュメント化されている
- **明確性**: 初心者から上級者まで理解できる
- **実用性**: 実際のユースケースに基づいた内容
- **最新性**: 常に最新の実装を反映している

### テストの品質
- **網羅性**: すべての機能とエッジケースをカバー
- **信頼性**: テストが安定して成功する
- **明確性**: テストの意図が明確
- **保守性**: テストコードが理解しやすい

---

## スケジュール詳細

### 第1週: 2025-11-27 (水) 〜 2025-11-30 (土)

**2025-11-27 (水): Task 3.1開始 - デバッグモード (Day 1/4)**
- 午前: --verboseフラグの追加（6時間）
  - CLI引数パース実装
  - 環境変数サポート
  - 設定ファイル統合
  - テスト（7件）
- 午後: リクエスト/レスポンス整形表示開始（4時間）
  - DebugLogger構造体設計
  - リクエスト整形ロジック実装開始

**2025-11-28 (木): Task 3.1継続 - デバッグモード (Day 2/4)**
- 午前: リクエスト/レスポンス整形表示継続（4時間）
  - レスポンス整形ロジック実装
  - カラー表示機能実装
  - トランケート機能実装
  - テスト（16件）
- 午後: タイムスタンプと経過時間表示（4時間）
  - TimingTracker実装
  - タイムスタンプフォーマット実装
  - テスト（7件）

**2025-11-29 (金): Task 3.1継続 - デバッグモード (Day 3/4)**
- 午前: デバッグログの詳細化（3時間）
  - `env_logger`から`tracing`への移行
  - 構造化ログ実装
- 午後: デバッグログの詳細化継続とテスト（5時間）
  - ログフォーマッタ実装
  - ログファイル出力実装
  - テスト（10件）

**2025-11-30 (土): Task 3.1完了とTask 3.2開始 (Day 4/4 → Day 1/5)**
- 午前: Task 3.1最終調整とレビュー（2時間）
  - 統合テスト
  - ドキュメント更新
- 午後: Task 3.2開始 - バッチテスト（6時間）
  - テスト定義フォーマット設計
  - データモデル実装開始

### 第2週: 2025-12-01 (日) 〜 2025-12-04 (水)

**2025-12-01 (日): Task 3.2継続 - バッチテスト (Day 2/5)**
- 午前: テスト定義フォーマット完成（2時間）
  - YAML/JSONパーサー実装
  - バリデーション実装
  - テスト（17件）
- 午後: テスト実行エンジン実装開始（6時間）
  - TestExecutor構造体設計
  - テストケース実行ロジック実装開始

**2025-12-02 (月): Task 3.2継続 - バッチテスト (Day 3/5)**
- 終日: テスト実行エンジン実装継続（8時間）
  - アサーション評価ロジック実装
  - リトライロジック実装
  - 並列実行サポート実装
  - テスト（30件）

**2025-12-03 (火): Task 3.2継続 - バッチテスト (Day 4/5)**
- 午前: CI/CD統合サポート（6時間）
  - 終了コード設定
  - 環境変数オーバーライド
  - CI/CD向けログフォーマット
  - GitHub Actions/GitLab CI例作成
  - テスト（3件）
- 午後: JUnit XMLレポート出力開始（2時間）
  - ReportGenerator設計

**2025-12-04 (水): Task 3.2継続 - バッチテスト (Day 5/5)**
- 午前: JUnit XMLレポート出力完成（6時間）
  - XML生成ロジック実装
  - テスト（8件）
- 午後: テスト結果集計と可視化（2時間）
  - ResultFormatter実装開始

### 第3週: 2025-12-05 (木) 〜 2025-12-08 (日)

**2025-12-05 (木): Task 3.2完了とTask 3.3開始**
- 午前: Task 3.2最終調整（4時間）
  - テスト結果集計と可視化完成
  - 統合テスト
  - ドキュメント更新
- 午後: Task 3.3開始 - インタラクティブモード (Day 1/4, 4時間)
  - タブ補完機能設計
  - McpCompleter実装開始

**2025-12-06 (金): Task 3.3継続 - インタラクティブモード (Day 2/4)**
- 午前: タブ補完機能完成（6時間）
  - 補完候補生成ロジック実装
  - テスト（12件）
- 午後: コマンド履歴保存と再利用（2時間）
  - HistoryManager実装開始

**2025-12-07 (土): Task 3.3継続 - インタラクティブモード (Day 3/4)**
- 午前: コマンド履歴完成（4時間）
  - 履歴保存/読み込み実装
  - 自動ローテーション実装
  - テスト（10件）
- 午後: JSON構文チェック（4時間）
  - JsonValidator実装
  - エラー検出ロジック実装
  - 修正提案ロジック実装

**2025-12-08 (日): Task 3.3完了とTask 3.4開始**
- 午前: JSON構文チェック完成とオートコンプリート（4時間）
  - テスト（24件）
  - AutoCompleter実装
- 午後: Task 3.4開始 - パフォーマンスモニタリング (Day 1/3, 4時間)
  - MetricsCollector設計と実装開始

### 第4週: 2025-12-09 (月) 〜 2025-12-11 (水)

**2025-12-09 (月): Task 3.4継続 - パフォーマンスモニタリング (Day 2/3)**
- 午前: MetricsCollector完成（4時間）
  - パーセンタイル計算実装
  - 循環バッファ実装
  - テスト（15件）
- 午後: レスポンスタイム測定と統計レポート（4時間）
  - ReportService実装開始

**2025-12-10 (火): Task 3.4完了とTask 3.5開始**
- 午前: 統計レポート完成とボトルネック検出（6時間）
  - ReportService完成
  - BottleneckDetector実装
  - テスト（19件）
- 午後: Task 3.5開始 - 構成管理の拡張 (Day 1/2, 2時間)
  - プロファイル機能設計
  - ProfileConfig実装開始

**2025-12-11 (水): Task 3.5継続 - 構成管理の拡張 (Day 2/2)**
- 午前: プロファイル機能完成（4時間）
  - プロファイル読み込み実装
  - プロファイル切替実装
  - テスト（10件）
- 午後: インポート/エクスポートとテンプレート（4時間）
  - ConfigImportExport実装
  - ConfigTemplate実装
  - テスト（23件）

### 第5週: 2025-12-12 (木) 〜 2025-12-15 (日)

**2025-12-12 (木): Task 3.6開始 - 統合テストとリリース (Day 1/3)**
- 午前: 全機能の統合テスト（6時間）
  - デバッグモード統合テスト
  - バッチテスト統合テスト
  - インタラクティブモード統合テスト
- 午後: パフォーマンスモニタリング・構成管理統合テスト（2時間）

**2025-12-13 (金): Task 3.6継続 - 統合テストとリリース (Day 2/3)**
- 午前: パフォーマンステスト（6時間）
  - レスポンスタイムテスト
  - スループットテスト
  - メモリ使用量テスト
  - ストレステスト
- 午後: ドキュメント更新開始（2時間）
  - README.md更新
  - チュートリアル更新開始

**2025-12-14 (土): Task 3.6継続 - 統合テストとリリース (Day 3/3)**
- 午前: ドキュメント更新継続（6時間）
  - チュートリアル完成
  - API仕様書更新
  - 新規ガイド作成（5本）
- 午後: CHANGELOG・リリースノート作成（2時間）

**2025-12-15 (日): Phase 3完了とリリース**
- 午前: 品質保証とレビュー（4時間）
  - コード品質チェック（Clippy、fmt）
  - テストカバレッジ確認
  - ドキュメントレビュー
- 午後: リリース準備とタグ作成（4時間）
  - バージョン番号更新
  - リリースビルド作成
  - Gitタグ作成
  - **v0.4.0リリース完了 🎉**

### マイルストーン
- **2025-11-30 (土) EOD**: Task 3.1完了（デバッグモード）
- **2025-12-04 (水) EOD**: Task 3.2完了（バッチテスト）
- **2025-12-08 (日) EOD**: Task 3.3完了（インタラクティブモード）
- **2025-12-10 (火) EOD**: Task 3.4完了（パフォーマンスモニタリング）
- **2025-12-11 (水) EOD**: Task 3.5完了（構成管理）
- **2025-12-15 (日) EOD**: **Phase 3完了、v0.4.0リリース** ✅

---

## リスクと対策

### リスク1: テスト実行エンジンの複雑性

**影響**: Task 3.2の遅延、バッチテスト機能の品質低下
**確率**: 中
**対策**:
- 段階的な実装（最小機能 → 拡張機能）
- 十分な単体テストの作成
- 早期のプロトタイプ作成と検証
- 複雑なアサーションは後回しにする柔軟性

### リスク2: インタラクティブモードのUX設計

**影響**: Task 3.3のユーザー満足度低下
**確率**: 中
**対策**:
- 早期のユーザーフィードバック収集
- 既存のREPLツール（例: IPython、irb）のUX参考
- プロトタイプによる早期検証
- オプション機能として段階的にリリース

### リスク3: パフォーマンスメトリクス収集のオーバーヘッド

**影響**: Task 3.4実装後のパフォーマンス劣化
**確率**: 低
**対策**:
- 効率的なデータ構造の使用（循環バッファ、HdrHistogram）
- サンプリングレート調整機能の実装
- メトリクス収集の有効/無効切替機能
- 早期のパフォーマンステスト実施

### リスク4: スケジュール遅延

**影響**: リリース日の延期、Phase 4への影響
**確率**: 中
**対策**:
- 優先順位の明確化（Must-have vs Nice-to-have）
- バッファ時間の確保（各タスクに10%の余裕）
- 毎日の進捗確認とスケジュール調整
- 必要に応じて機能の段階的リリース

### リスク5: 依存クレートの互換性問題

**影響**: ビルドエラー、実装の遅延
**確率**: 低
**対策**:
- 安定版クレートの使用
- 事前の互換性検証
- Cargo.lockの活用
- 代替クレートの調査

### リスク6: CI/CD統合の複雑性

**影響**: Task 3.2のCI/CD統合機能の遅延
**確率**: 中
**対策**:
- 主要CI/CDツール（GitHub Actions、GitLab CI）に絞る
- シンプルな統合例から開始
- 詳細なドキュメントの提供
- コミュニティフィードバックによる改善

---

## 依存関係

### 内部依存
- **Phase 2の完了**: ドキュメント整備が前提
- **Phase 1の安定性**: v0.3.1の堅牢なエラーハンドリング
- **既存機能との統合**: 各新機能が既存機能と協調動作すること

### 外部依存
- **Rustツールチェイン**: 1.70以上
- **依存クレートの安定性**: 主要クレートが安定版であること
- **CI/CDツール**: GitHub Actions、GitLab CI等のアクセス

### タスク間依存関係
```
Task 3.1 (デバッグモード) ← 独立
    ↓
Task 3.2 (バッチテスト) ← Task 3.1のログ機能を活用
    ↓
Task 3.3 (インタラクティブモード) ← 独立
    ↓
Task 3.4 (パフォーマンスモニタリング) ← Task 3.1のログ機能を活用
    ↓
Task 3.5 (構成管理) ← 独立
    ↓
Task 3.6 (統合テスト) ← すべてのタスクに依存
```

**並行実装の可能性:**
- Task 3.1とTask 3.3は並行実装可能
- Task 3.4とTask 3.5は並行実装可能
- ただし、リソースの制約により順次実装を推奨

---

## プロジェクト管理

### コミュニケーション
- **進捗報告**: 毎日の簡易報告と週次の詳細報告
- **課題共有**: GitHubイシューでの課題管理
- **コードレビュー**: プルリクエストによるレビュープロセス

### 品質管理
- **コードレビュー**: 各タスク完了時にレビュー実施
- **自動テスト**: CIによる自動テスト実行
- **パフォーマンステスト**: 定期的なパフォーマンス測定
- **ドキュメントレビュー**: 技術的正確性と可読性の確認

### 進捗管理
- **GitHub Projects**: タスク管理とカンバンボード
- **マイルストーン**: 週次のマイルストーン設定
- **進捗レポート**: 週次の詳細レポート作成

---

## 成果物チェックリスト

### コード

**新規ファイル:**
- [ ] `src/models/debug_config.rs`
- [ ] `src/models/test_definition.rs`
- [ ] `src/models/metrics.rs`
- [ ] `src/models/profile_config.rs`
- [ ] `src/services/debug_logger.rs`
- [ ] `src/services/timing_tracker.rs`
- [ ] `src/services/test_executor.rs`
- [ ] `src/services/report_generator.rs`
- [ ] `src/services/result_formatter.rs`
- [ ] `src/services/metrics_collector.rs`
- [ ] `src/services/report_service.rs`
- [ ] `src/services/bottleneck_detector.rs`
- [ ] `src/services/config_import_export.rs`
- [ ] `src/services/config_template.rs`
- [ ] `src/interactive/mod.rs`
- [ ] `src/interactive/completer.rs`
- [ ] `src/interactive/history_manager.rs`
- [ ] `src/interactive/json_validator.rs`
- [ ] `src/interactive/auto_complete.rs`

**修正ファイル:**
- [ ] `src/main.rs`
- [ ] `src/services/inspector.rs`
- [ ] `src/client/stdio_client.rs`
- [ ] `Cargo.toml` (依存クレート追加、バージョン更新)

**テストファイル:**
- [ ] `tests/task3_1_debug_mode_test.rs`
- [ ] `tests/task3_2_batch_test.rs`
- [ ] `tests/task3_3_interactive_mode_test.rs`
- [ ] `tests/task3_4_performance_monitoring_test.rs`
- [ ] `tests/task3_5_config_management_test.rs`
- [ ] `tests/phase3_integration_test.rs`
- [ ] `tests/phase3_performance_test.rs`

### ドキュメント

**新規ガイド:**
- [ ] `docs/guides/debug-mode.md` (約3,000文字)
- [ ] `docs/guides/batch-testing.md` (約4,000文字)
- [ ] `docs/guides/interactive-mode.md` (約2,500文字)
- [ ] `docs/guides/performance-monitoring.md` (約3,500文字)
- [ ] `docs/guides/configuration-management.md` (約3,000文字)

**リリース関連:**
- [ ] `docs/releases/v0.4.0.md` (リリースノート)
- [ ] `docs/migration/v0.3_to_v0.4.md` (移行ガイド)
- [ ] `CHANGELOG.md` (Phase 3更新)

**更新ドキュメント:**
- [ ] `README.md` (Phase 3機能追加、使用例更新)
- [ ] `docs/tutorials/advanced-usage.md` (Phase 3機能の追記)
- [ ] `docs/api/tools.md` (新機能のAPI仕様追加)

### サンプルとテンプレート

**テスト定義:**
- [ ] `examples/test_suites/basic_test.yaml`
- [ ] `examples/test_suites/advanced_test.yaml`
- [ ] `examples/test_suites/ci_test.yaml`

**設定テンプレート:**
- [ ] `templates/minimal.json`
- [ ] `templates/development.json`
- [ ] `templates/production.json`
- [ ] `templates/ci.json`

**CI/CD統合:**
- [ ] `.github/workflows/mcp_test.yml` (GitHub Actions例)
- [ ] `.gitlab-ci.yml` (GitLab CI例、ルートディレクトリ)

### 品質保証

**テスト:**
- [ ] 全単体テストが成功（260/260）
- [ ] 全統合テストが成功（45/45）
- [ ] 全パフォーマンステストが目標達成（20/20）
- [ ] コードカバレッジ85%以上

**コード品質:**
- [ ] `cargo clippy`警告ゼロ
- [ ] `cargo fmt`フォーマット準拠
- [ ] コードレビュー完了

**ドキュメント品質:**
- [ ] 技術的正確性確認
- [ ] 表記の統一性確認
- [ ] リンク切れゼロ
- [ ] コード例が実行可能

---

## 付録

### 用語集

**Phase 3で導入される用語:**
- **Verbose Mode**: 詳細なデバッグ情報を出力するモード
- **Batch Test**: 複数のテストケースを一括実行する機能
- **Test Definition**: テストケースを定義するYAML/JSONファイル
- **Assertion**: テスト結果を検証する条件
- **JUnit XML**: テスト結果を記録する標準XML形式
- **Tab Completion**: タブキーによる入力補完機能
- **Auto-complete**: 自動的に候補を提案する機能
- **Metrics**: パフォーマンス測定値
- **Percentile**: パーセンタイル（P50、P95、P99等）
- **Bottleneck**: 性能のボトルネック
- **Profile**: 環境別の設定セット
- **Template**: 設定のひな形

### 参考リソース

**Rust関連:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [tokio Documentation](https://docs.rs/tokio/)

**MCP関連:**
- [Model Context Protocol 公式サイト](https://modelcontextprotocol.io/)
- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp/)

**テスト関連:**
- [JUnit XML Format](https://llg.cubic.org/docs/junit/)
- [YAML Specification](https://yaml.org/spec/)

**CI/CD関連:**
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitLab CI/CD Documentation](https://docs.gitlab.com/ee/ci/)

**UX/UI関連:**
- [rustyline Documentation](https://docs.rs/rustyline/)
- [colored Documentation](https://docs.rs/colored/)

### レビュー担当者

**コードレビュー:** Rust開発者、MCP経験者
**ドキュメントレビュー:** テクニカルライター
**ユーザビリティテスト:** 新規ユーザー3〜5名
**パフォーマンステスト:** パフォーマンスエンジニア

---

## バージョン履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|----------|------|---------|-------|
| 1.0 | 2025-11-18 | 初版作成 | Tech Writer |

---

**このドキュメントは、MCP Inspector MCP Server v0.4.0 (Phase 3: 機能拡張フェーズ) の詳細実行計画書です。**

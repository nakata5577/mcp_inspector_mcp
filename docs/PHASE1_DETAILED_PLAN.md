# Phase 1 詳細実行計画書

**フェーズ名**: 安定化フェーズ (v0.3.1)
**期間**: 2週間 (2025-11-18 〜 2025-12-01)
**目的**: 残存バグの完全解決とエラーハンドリング改善
**目標**: ユーザー評価 9.0/10以上の安定版リリース

---

## 目次

1. [Phase 1 概要](#1-phase-1-概要)
2. [作業スケジュール](#2-作業スケジュール)
3. [タスク詳細](#3-タスク詳細)
4. [開発環境準備](#4-開発環境準備)
5. [テスト計画](#5-テスト計画)
6. [リリース手順](#6-リリース手順)

---

## 1. Phase 1 概要

### 1.1 主要目標

1. **list_filesツール問題の完全解決** (Priority: Critical)
2. **エラーレポートの構造化** (Priority: High)
3. **Capability検証と警告機能** (Priority: Medium)
4. **タイムアウト設定のカスタマイズ** (Priority: Medium)
5. **統合テストとリリース** (Priority: Critical)

### 1.2 成功基準

**必須条件**:
- ✅ list_filesツールが正常動作（成功率95%以上）
- ✅ エラーメッセージが構造化され、詳細情報を含む
- ✅ Capability矛盾時に警告が表示される
- ✅ 全既存機能が継続動作（リグレッションなし）
- ✅ パフォーマンス劣化なし（±10%以内）

**推奨条件**:
- ⭐ テストカバレッジ70%以上
- ⭐ ユーザー評価9.0/10以上

### 1.3 リソース配分

| タスク | 担当 | 工数 |
|-------|-----|------|
| Task 1.1: list_files問題解決 | debug-expert | 3日 |
| Task 1.2: エラーレポート構造化 | rust-developer | 2日 |
| Task 1.3: Capability検証 | rust-developer | 2日 |
| Task 1.4: タイムアウト設定 | rust-developer | 1日 |
| Task 1.5: 統合テスト | test-engineer | 2日 |
| **合計** | | **10日** |

---

## 2. 作業スケジュール

### Week 1: バグ修正とコア機能実装

```
Day 1 (11/18 月):
  AM: 環境準備、list_files問題の調査開始
  PM: 問題の再現と原因特定

Day 2 (11/19 火):
  AM: list_files問題の修正実装
  PM: 修正のテストとレビュー

Day 3 (11/20 水):
  AM: list_files問題の最終確認
  PM: エラーレポート構造化の設計

Day 4 (11/21 木):
  AM: エラーレポート構造化の実装
  PM: エラーハンドリングのテスト

Day 5 (11/22 金):
  AM: Capability検証機能の設計
  PM: 週次レビュー、進捗確認
```

### Week 2: 追加機能と統合テスト

```
Day 6 (11/25 月):
  AM: Capability検証機能の実装
  PM: Capability警告のテスト

Day 7 (11/26 火):
  AM: タイムアウト設定機能の実装
  PM: 設定機能のテスト

Day 8 (11/27 水):
  AM: 統合テスト準備
  PM: 全機能の統合テスト実施

Day 9 (11/28 木):
  AM: リグレッションテスト
  PM: バグ修正、最終調整

Day 10 (12/01 金):
  AM: リリースノート作成、最終レビュー
  PM: v0.3.1 リリース
```

---

## 3. タスク詳細

### Task 1.1: list_filesツール問題の完全解決

**期間**: Day 1-3 (11/18-11/20)
**担当**: debug-expert
**優先度**: Critical

#### 3.1.1 問題の再現と調査

**Step 1: 問題の再現**

```bash
# テスト環境の準備
cd C:\Users\takah\work\my_mcp_server\mcp_inspector_mcp

# デバッグモードでの実行
RUST_LOG=debug cargo run --release

# 別ターミナルでClaude Desktopから以下を実行
# tools_call("screening-server", "list_files", {"directory": "."})
```

**期待される症状**:
```
Error: "No result received from client-side tool execution."
```

**Step 2: 詳細ログの追加**

`src/client/stdio_client.rs` に詳細なトレーシングログを追加：

```rust
// 追加するログポイント

// 1. ツール呼び出し開始時
tracing::info!("=== TOOL CALL START ===");
tracing::info!("Tool: {}", name);
tracing::info!("Arguments: {:?}", arguments);
let start_time = std::time::Instant::now();

// 2. リクエスト送信直前
tracing::debug!("Sending CallToolRequest to server process");
tracing::debug!("Request params: {:?}", request_params);

// 3. 応答待ち
tracing::debug!("Waiting for server response...");

// 4. 応答受信時
let elapsed = start_time.elapsed();
tracing::info!("Response received in {:?}", elapsed);
tracing::debug!("Response data: {:?}", response);

// 5. タイムアウト時
tracing::error!("Tool call timeout after {:?}", elapsed);
tracing::error!("Server process alive: {}", is_server_alive());
```

**Step 3: 原因の特定**

調査すべき項目：

1. **タイムアウト設定**
   - 現在のタイムアウト値を確認
   - list_files処理時間を測定
   - タイムアウトが発生しているか確認

2. **サーバープロセスの状態**
   - プロセスがクラッシュしていないか
   - プロセスがハングしていないか
   - stderrにエラーが出力されていないか

3. **レスポンスの受信**
   - レスポンスが送信されているか
   - レスポンスのフォーマットが正しいか
   - バッファリング問題がないか

**調査結果の記録**:

```markdown
# list_files問題調査結果

## 再現環境
- OS: Windows
- MCP Inspector: v0.3.0
- screening-server: rmcp 0.8.5

## 症状
[詳細な症状の記述]

## 原因
[特定された原因]

## 根本原因分析
[5 Whys分析など]

## 修正方針
[修正アプローチの説明]
```

#### 3.1.2 修正の実装

**パターンA: タイムアウト問題の場合**

`src/client/stdio_client.rs`:

```rust
use tokio::time::{timeout, Duration};

pub async fn call_tool(
    &self,
    name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value> {
    // 既存のコード...

    // タイムアウト時間を延長（5秒 → 30秒）
    let timeout_duration = Duration::from_millis(
        std::env::var("MCP_TOOL_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30000) // デフォルト30秒
    );

    tracing::debug!("Tool call timeout set to {:?}", timeout_duration);

    // タイムアウト付きで実行
    let result = timeout(timeout_duration, async {
        service.call_tool(request_params).await
    }).await;

    match result {
        Ok(Ok(response)) => {
            tracing::info!("Tool call successful");
            Ok(response)
        }
        Ok(Err(e)) => {
            tracing::error!("Tool call failed: {}", e);
            Err(e.into())
        }
        Err(_) => {
            tracing::error!("Tool call timed out after {:?}", timeout_duration);
            Err(anyhow::anyhow!(
                "Tool '{}' execution timed out after {}ms",
                name,
                timeout_duration.as_millis()
            ))
        }
    }
}
```

**パターンB: サーバープロセス監視の追加**

```rust
// プロセス生存確認機能の追加
async fn is_server_process_alive(&self) -> bool {
    // Windows向けの実装
    #[cfg(target_os = "windows")]
    {
        if let Some(child) = self.child_process.lock().await.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => false, // プロセス終了済み
                Ok(None) => true,     // プロセス実行中
                Err(_) => false,      // エラー
            }
        } else {
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Linux/macOS向けの実装
        // 類似のロジック
        true
    }
}

// ツール呼び出し失敗時にプロセス状態を確認
if let Err(e) = result {
    let server_alive = self.is_server_process_alive().await;
    tracing::error!("Server process alive: {}", server_alive);

    if !server_alive {
        return Err(anyhow::anyhow!(
            "Tool '{}' failed: Server process has terminated. {}",
            name,
            e
        ));
    }
}
```

**パターンC: バッファリング問題の対処**

```rust
// stdout/stderrのフラッシュを明示的に実行
async fn ensure_flush(&mut self) -> Result<()> {
    if let Some(stdin) = &mut self.stdin {
        stdin.flush().await?;
    }
    Ok(())
}
```

#### 3.1.3 テストケースの作成

`tests/integration/list_files_test.rs`:

```rust
#[cfg(test)]
mod list_files_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_files_success() {
        // 正常系: カレントディレクトリのファイル一覧
        let result = call_tool(
            "screening-server",
            "list_files",
            json!({"directory": "."}),
        ).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains_key("files"));
    }

    #[tokio::test]
    async fn test_list_files_large_directory() {
        // 大量のファイルがあるディレクトリ
        let result = call_tool(
            "screening-server",
            "list_files",
            json!({"directory": "C:\\Windows"}),
        ).await;

        // タイムアウトしないことを確認
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_files_timeout_handling() {
        // タイムアウト設定を短くして意図的にタイムアウトさせる
        std::env::set_var("MCP_TOOL_TIMEOUT_MS", "100");

        let result = call_tool(
            "screening-server",
            "list_files",
            json!({"directory": "C:\\"}),
        ).await;

        // タイムアウトエラーが適切に処理されることを確認
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("timeout"));
    }
}
```

#### 3.1.4 ドキュメント更新

`docs/troubleshooting/list_files_issue.md`:

```markdown
# list_files ツール問題と解決策

## 問題の概要
v0.3.0において、`list_files`ツールが"No result received"エラーで失敗する問題がありました。

## 原因
[調査で判明した原因]

## 解決策
v0.3.1で以下の修正を実施：
1. タイムアウト時間を30秒に延長
2. 環境変数でのタイムアウト設定サポート
3. サーバープロセス生存確認の追加
4. より詳細なエラーメッセージ

## 使用方法
タイムアウトをカスタマイズする場合：
```bash
MCP_TOOL_TIMEOUT_MS=60000 mcp-inspector tools_call ...
```

## 今後の改善
- 処理中の進捗表示
- キャンセル機能の追加
```

**成果物**:
- [ ] 問題調査レポート
- [ ] 修正済みコード
- [ ] テストケース
- [ ] トラブルシューティングドキュメント

---

### Task 1.2: エラーレポートの構造化

**期間**: Day 3-4 (11/20-11/21)
**担当**: rust-developer
**優先度**: High

#### 3.2.1 エラー型の定義

新しいファイル `src/error.rs` を作成：

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

/// ツール実行エラーの詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolExecutionError {
    /// タイムアウトエラー
    Timeout {
        tool_name: String,
        elapsed_ms: u64,
        configured_timeout_ms: u64,
        server_alive: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },

    /// サーバープロセスのクラッシュ
    ServerCrash {
        tool_name: String,
        exit_code: Option<i32>,
        stderr: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_log: Option<String>,
    },

    /// 不正なレスポンス
    InvalidResponse {
        tool_name: String,
        received: String,
        expected_format: String,
        parse_error: String,
    },

    /// ネットワーク/通信エラー
    CommunicationError {
        tool_name: String,
        details: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },

    /// サーバー側のエラー
    ServerError {
        tool_name: String,
        error_message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_code: Option<i32>,
    },

    /// その他のエラー
    Other {
        tool_name: String,
        message: String,
    },
}

impl ToolExecutionError {
    /// ユーザーフレンドリーなエラーメッセージを生成
    pub fn user_message(&self) -> String {
        match self {
            Self::Timeout {
                tool_name,
                elapsed_ms,
                configured_timeout_ms,
                server_alive,
                suggestion
            } => {
                let mut msg = format!(
                    "Tool '{}' timed out after {}ms (configured timeout: {}ms)",
                    tool_name, elapsed_ms, configured_timeout_ms
                );

                if !server_alive {
                    msg.push_str("\nServer process has terminated unexpectedly.");
                }

                if let Some(sug) = suggestion {
                    msg.push_str(&format!("\nSuggestion: {}", sug));
                }

                msg
            }

            Self::ServerCrash {
                tool_name,
                exit_code,
                stderr,
                ..
            } => {
                format!(
                    "Server crashed while executing tool '{}'\nExit code: {:?}\nError output: {}",
                    tool_name,
                    exit_code,
                    if stderr.is_empty() { "(none)" } else { stderr }
                )
            }

            Self::InvalidResponse {
                tool_name,
                parse_error,
                ..
            } => {
                format!(
                    "Tool '{}' returned invalid response: {}",
                    tool_name, parse_error
                )
            }

            Self::CommunicationError {
                tool_name,
                details,
                suggestion
            } => {
                let mut msg = format!(
                    "Communication error while calling tool '{}': {}",
                    tool_name, details
                );

                if let Some(sug) = suggestion {
                    msg.push_str(&format!("\nSuggestion: {}", sug));
                }

                msg
            }

            Self::ServerError {
                tool_name,
                error_message,
                ..
            } => {
                format!(
                    "Server error in tool '{}': {}",
                    tool_name, error_message
                )
            }

            Self::Other { tool_name, message } => {
                format!("Error in tool '{}': {}", tool_name, message)
            }
        }
    }

    /// JSON形式でのシリアライズ
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            serde_json::json!({
                "type": "serialization_error",
                "message": "Failed to serialize error"
            })
        })
    }
}

impl fmt::Display for ToolExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ToolExecutionError {}

/// エラーレスポンスの構造
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ToolExecutionError,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: ToolExecutionError) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }
}
```

#### 3.2.2 エラーハンドリングの統一

`src/client/stdio_client.rs` の修正：

```rust
use crate::error::{ToolExecutionError, ErrorResponse};

pub async fn call_tool(
    &self,
    name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, ErrorResponse> {
    let start_time = std::time::Instant::now();
    let timeout_ms = self.config.timeout_ms;

    // タイムアウト付きで実行
    let result = timeout(
        Duration::from_millis(timeout_ms),
        self.call_tool_inner(name, arguments)
    ).await;

    match result {
        Ok(Ok(response)) => Ok(response),

        Ok(Err(e)) => {
            // 内部エラーを構造化エラーに変換
            let error = if e.to_string().contains("server terminated") {
                ToolExecutionError::ServerCrash {
                    tool_name: name.to_string(),
                    exit_code: self.get_exit_code().await,
                    stderr: self.get_stderr_output().await,
                    last_log: None,
                }
            } else {
                ToolExecutionError::Other {
                    tool_name: name.to_string(),
                    message: e.to_string(),
                }
            };

            Err(ErrorResponse::new(error))
        }

        Err(_) => {
            // タイムアウト
            let elapsed_ms = start_time.elapsed().as_millis() as u64;
            let server_alive = self.is_server_process_alive().await;

            let error = ToolExecutionError::Timeout {
                tool_name: name.to_string(),
                elapsed_ms,
                configured_timeout_ms: timeout_ms,
                server_alive,
                suggestion: Some(format!(
                    "Try increasing timeout with environment variable: MCP_TOOL_TIMEOUT_MS={}",
                    timeout_ms * 2
                )),
            };

            Err(ErrorResponse::new(error))
        }
    }
}
```

#### 3.2.3 server/mod.rs での統合

```rust
use crate::error::{ErrorResponse, ToolExecutionError};

async fn handle_tools_call(
    inspector: Arc<InspectorServer>,
    params: ToolCallParams,
) -> Result<CallToolResult, McpError> {
    match inspector.call_tool(&params.server, &params.tool_name, params.arguments).await {
        Ok(result) => Ok(CallToolResult::success(result)),

        Err(error_response) => {
            // 構造化されたエラーをJSON-RPCエラーとして返す
            let error_json = error_response.error.to_json();

            Ok(CallToolResult {
                content: vec![ContentItem::Text {
                    text: serde_json::to_string_pretty(&error_json)
                        .unwrap_or_else(|_| error_response.error.user_message()),
                }],
                is_error: true,
            })
        }
    }
}
```

#### 3.2.4 テストケース

`tests/error_handling_test.rs`:

```rust
#[tokio::test]
async fn test_timeout_error_structure() {
    std::env::set_var("MCP_TOOL_TIMEOUT_MS", "100");

    let result = call_tool("screening-server", "slow_tool", json!({})).await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // エラー型の確認
    assert!(matches!(error.error, ToolExecutionError::Timeout { .. }));

    // JSONシリアライズの確認
    let json = error.error.to_json();
    assert_eq!(json["type"], "Timeout");
    assert!(json["elapsed_ms"].as_u64().unwrap() >= 100);
    assert!(json["suggestion"].is_string());
}

#[tokio::test]
async fn test_server_crash_error() {
    // サーバークラッシュのシミュレーション
    let result = simulate_server_crash("screening-server", "crash_tool").await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    assert!(matches!(error.error, ToolExecutionError::ServerCrash { .. }));
}
```

**成果物**:
- [ ] src/error.rs（新規作成）
- [ ] 既存コードの修正
- [ ] テストケース
- [ ] エラー一覧ドキュメント

---

### Task 1.3: Capability検証と警告機能

**期間**: Day 5-6 (11/22, 11/25)
**担当**: rust-developer
**優先度**: Medium

#### 3.3.1 Capability検証ロジックの実装

新しいファイル `src/validation/capability.rs` を作成：

```rust
use serde::{Deserialize, Serialize};

/// Capability検証の警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityWarning {
    pub severity: WarningSeverity,
    pub category: WarningCategory,
    pub message: String,
    pub details: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningCategory {
    ToolsMismatch,
    ResourcesMismatch,
    PromptsMismatch,
    ProtocolVersionMismatch,
    Other,
}

/// 実際に発見された機能
#[derive(Debug, Default)]
pub struct DiscoveredCapabilities {
    pub tools_count: usize,
    pub resources_count: usize,
    pub prompts_count: usize,
}

/// Capabilityの検証
pub fn validate_capabilities(
    server_name: &str,
    reported: &ServerCapabilities,
    discovered: &DiscoveredCapabilities,
) -> Vec<CapabilityWarning> {
    let mut warnings = Vec::new();

    // Tools capability mismatch
    if !reported.tools.supported && discovered.tools_count > 0 {
        warnings.push(CapabilityWarning {
            severity: WarningSeverity::Warning,
            category: WarningCategory::ToolsMismatch,
            message: format!(
                "Server '{}' reports tools as not supported, but {} tools were discovered",
                server_name, discovered.tools_count
            ),
            details: format!(
                "Reported: tools.supported = false\nDiscovered: {} tools",
                discovered.tools_count
            ),
            recommendation: Some(
                "This is likely a server-side capability reporting issue. \
                 The tools are functional despite the capability report.".to_string()
            ),
        });
    }

    // Resources capability mismatch
    if !reported.resources.supported && discovered.resources_count > 0 {
        warnings.push(CapabilityWarning {
            severity: WarningSeverity::Warning,
            category: WarningCategory::ResourcesMismatch,
            message: format!(
                "Server '{}' reports resources as not supported, but {} resources were discovered",
                server_name, discovered.resources_count
            ),
            details: format!(
                "Reported: resources.supported = false\nDiscovered: {} resources",
                discovered.resources_count
            ),
            recommendation: Some(
                "The resources are functional despite the capability report.".to_string()
            ),
        });
    }

    // Prompts capability mismatch
    if !reported.prompts.supported && discovered.prompts_count > 0 {
        warnings.push(CapabilityWarning {
            severity: WarningSeverity::Warning,
            category: WarningCategory::PromptsMismatch,
            message: format!(
                "Server '{}' reports prompts as not supported, but {} prompts were discovered",
                server_name, discovered.prompts_count
            ),
            details: format!(
                "Reported: prompts.supported = false\nDiscovered: {} prompts",
                discovered.prompts_count
            ),
            recommendation: Some(
                "The prompts are functional despite the capability report.".to_string()
            ),
        });
    }

    warnings
}

/// 警告の表示フォーマット
impl CapabilityWarning {
    pub fn format_for_display(&self) -> String {
        let icon = match self.severity {
            WarningSeverity::Info => "ℹ️ ",
            WarningSeverity::Warning => "⚠️ ",
            WarningSeverity::Error => "❌",
        };

        let mut output = format!("{} {}\n", icon, self.message);
        output.push_str(&format!("\n{}\n", self.details));

        if let Some(rec) = &self.recommendation {
            output.push_str(&format!("\nRecommendation: {}\n", rec));
        }

        output
    }
}
```

#### 3.3.2 server_inspect への統合

`src/handlers/server_inspect.rs` の修正：

```rust
use crate::validation::capability::{validate_capabilities, DiscoveredCapabilities};

pub async fn handle_server_inspect(
    inspector: Arc<InspectorServer>,
    server_name: String,
) -> Result<InspectResult> {
    // 既存のserver_inspect処理...
    let capabilities = inspector.get_capabilities(&server_name).await?;

    // 実際の機能を発見
    let mut discovered = DiscoveredCapabilities::default();

    if let Ok(tools) = inspector.list_tools(&server_name).await {
        discovered.tools_count = tools.len();
    }

    if let Ok(resources) = inspector.list_resources(&server_name).await {
        discovered.resources_count = resources.len();
    }

    if let Ok(prompts) = inspector.list_prompts(&server_name).await {
        discovered.prompts_count = prompts.len();
    }

    // Capabilityを検証
    let warnings = validate_capabilities(&server_name, &capabilities, &discovered);

    // 結果にwarningsを追加
    let mut result = InspectResult {
        server_name,
        capabilities,
        // ... 他のフィールド
    };

    if !warnings.is_empty() {
        result.validation_warnings = Some(warnings);
    }

    Ok(result)
}
```

#### 3.3.3 出力フォーマット

```json
{
  "server_name": "screening-server",
  "connection_status": "connected",
  "capabilities": {
    "tools": {
      "supported": false
    }
  },
  "validation_warnings": [
    {
      "severity": "warning",
      "category": "tools_mismatch",
      "message": "Server 'screening-server' reports tools as not supported, but 8 tools were discovered",
      "details": "Reported: tools.supported = false\nDiscovered: 8 tools",
      "recommendation": "This is likely a server-side capability reporting issue. The tools are functional despite the capability report."
    }
  ]
}
```

**成果物**:
- [ ] src/validation/capability.rs（新規作成）
- [ ] server_inspect統合コード
- [ ] テストケース
- [ ] ドキュメント

---

### Task 1.4: タイムアウト設定のカスタマイズ

**期間**: Day 7 (11/26)
**担当**: rust-developer
**優先度**: Medium

#### 3.4.1 設定構造の定義

`src/config.rs` に追加：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// ツール実行のタイムアウト（ミリ秒）
    #[serde(default = "default_tool_timeout")]
    pub tool_timeout_ms: u64,

    /// サーバー接続のタイムアウト（ミリ秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,

    /// リトライ回数
    #[serde(default)]
    pub retry_count: u32,

    /// タイムアウト時の自動リトライ
    #[serde(default)]
    pub auto_retry_on_timeout: bool,
}

fn default_tool_timeout() -> u64 { 30000 }
fn default_connection_timeout() -> u64 { 5000 }

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            tool_timeout_ms: default_tool_timeout(),
            connection_timeout_ms: default_connection_timeout(),
            retry_count: 0,
            auto_retry_on_timeout: false,
        }
    }
}

impl ExecutionConfig {
    /// 環境変数から設定を読み込む
    pub fn from_env() -> Self {
        Self {
            tool_timeout_ms: std::env::var("MCP_TOOL_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_tool_timeout),

            connection_timeout_ms: std::env::var("MCP_CONNECTION_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_connection_timeout),

            retry_count: std::env::var("MCP_RETRY_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            auto_retry_on_timeout: std::env::var("MCP_AUTO_RETRY")
                .ok()
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}
```

#### 3.4.2 config.jsonでの設定サポート

`.inspector/config.json` の拡張：

```json
{
  "servers": [
    {
      "name": "screening-server",
      "transport": "stdio",
      "command": "C:\\Users\\takah\\work\\my_mcp_server\\screening_server\\target\\release\\screening_server.exe",
      "args": [],
      "env": {}
    }
  ],
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 1,
    "auto_retry_on_timeout": false
  }
}
```

#### 3.4.3 ドキュメント

`docs/configuration/timeouts.md`:

```markdown
# タイムアウト設定

## 概要
mcp-inspectorでは、ツール実行と接続のタイムアウトをカスタマイズできます。

## 設定方法

### 1. 環境変数
```bash
# ツール実行タイムアウト（ミリ秒）
export MCP_TOOL_TIMEOUT_MS=60000

# 接続タイムアウト（ミリ秒）
export MCP_CONNECTION_TIMEOUT_MS=10000

# リトライ回数
export MCP_RETRY_COUNT=1

# タイムアウト時の自動リトライ
export MCP_AUTO_RETRY=true
```

### 2. config.json
```json
{
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 1,
    "auto_retry_on_timeout": false
  }
}
```

## デフォルト値
- ツール実行タイムアウト: 30秒 (30000ms)
- 接続タイムアウト: 5秒 (5000ms)
- リトライ回数: 0
- 自動リトライ: 無効

## 推奨設定

### 開発環境
```json
{
  "tool_timeout_ms": 60000,
  "retry_count": 2
}
```

### 本番環境
```json
{
  "tool_timeout_ms": 30000,
  "retry_count": 1
}
```

### CI/CD環境
```json
{
  "tool_timeout_ms": 120000,
  "retry_count": 0
}
```
```

**成果物**:
- [ ] 設定構造の実装
- [ ] 環境変数サポート
- [ ] config.jsonサポート
- [ ] ドキュメント

---

### Task 1.5: Phase 1 統合テストとリリース

**期間**: Day 8-10 (11/27-12/01)
**担当**: test-engineer, release-manager
**優先度**: Critical

#### 3.5.1 統合テストスイート

`tests/integration/phase1_integration.rs`:

```rust
/// Phase 1 統合テストスイート
#[cfg(test)]
mod phase1_integration_tests {
    use super::*;

    /// Setup: screening-serverが起動していること
    async fn setup() -> TestContext {
        // テスト環境の準備
        TestContext::new("screening-server").await
    }

    #[tokio::test]
    async fn test_list_files_fixed() {
        let ctx = setup().await;

        // list_filesが正常動作することを確認
        let result = ctx.call_tool("list_files", json!({
            "directory": "."
        })).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains_key("files"));

        // パフォーマンス確認（30秒以内）
        assert!(ctx.last_elapsed_ms() < 30000);
    }

    #[tokio::test]
    async fn test_error_reporting() {
        let ctx = setup().await;

        // タイムアウトエラーのテスト
        std::env::set_var("MCP_TOOL_TIMEOUT_MS", "100");
        let result = ctx.call_tool("slow_operation", json!({})).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        // 構造化されたエラーであることを確認
        assert!(error.contains_key("type"));
        assert_eq!(error["type"], "Timeout");
        assert!(error.contains_key("elapsed_ms"));
        assert!(error.contains_key("suggestion"));
    }

    #[tokio::test]
    async fn test_capability_validation() {
        let ctx = setup().await;

        // server_inspectを実行
        let result = ctx.server_inspect().await;

        assert!(result.is_ok());
        let inspect = result.unwrap();

        // 警告が含まれていることを確認
        assert!(inspect.contains_key("validation_warnings"));
        let warnings = inspect["validation_warnings"].as_array().unwrap();

        // tools_mismatch警告があることを確認
        let has_tools_warning = warnings.iter().any(|w| {
            w["category"] == "tools_mismatch"
        });
        assert!(has_tools_warning);
    }

    #[tokio::test]
    async fn test_custom_timeout() {
        let ctx = setup().await;

        // カスタムタイムアウトが機能することを確認
        ctx.set_timeout(60000);

        let result = ctx.call_tool("list_files", json!({
            "directory": "C:\\"
        })).await;

        // 60秒以内に完了することを確認
        assert!(result.is_ok() || ctx.last_elapsed_ms() >= 60000);
    }

    #[tokio::test]
    async fn test_regression_hello_world() {
        let ctx = setup().await;

        // v0.3.0で動作していた機能が継続することを確認
        let result = ctx.call_tool("hello_world", json!({
            "name": "雄大"
        })).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["message"], "Hello, 雄大!");
    }

    #[tokio::test]
    async fn test_regression_echo() {
        let ctx = setup().await;

        let result = ctx.call_tool("echo", json!({
            "message": "テストメッセージ"
        })).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response["echoed"], "テストメッセージ");
    }

    #[tokio::test]
    async fn test_performance_no_regression() {
        let ctx = setup().await;

        // 10回連続実行してパフォーマンスを測定
        let mut total_ms = 0u64;
        for _ in 0..10 {
            let result = ctx.call_tool("hello_world", json!({})).await;
            assert!(result.is_ok());
            total_ms += ctx.last_elapsed_ms();
        }

        let avg_ms = total_ms / 10;

        // 平均100ms以下（v0.3.0と同等以上）
        assert!(avg_ms < 100, "Performance regression detected: avg {}ms", avg_ms);
    }
}
```

#### 3.5.2 リグレッションテスト

`tests/regression/v0_3_0.rs`:

```rust
/// v0.3.0で動作していた全機能のリグレッションテスト
#[cfg(test)]
mod v0_3_0_regression {
    #[tokio::test]
    async fn test_all_tools_working() {
        let tools = vec![
            "hello_world",
            "echo",
            "get_project_info",
            "list_files",
        ];

        for tool in tools {
            let result = call_tool("screening-server", tool, json!({})).await;
            assert!(result.is_ok() || result.is_err_with_valid_structure());
        }
    }

    #[tokio::test]
    async fn test_japanese_support() {
        let test_cases = vec![
            ("雄大", "Hello, 雄大!"),
            ("テスト", "Hello, テスト!"),
            ("中葉", "Hello, 中葉!"),
        ];

        for (input, expected) in test_cases {
            let result = call_tool("screening-server", "hello_world", json!({
                "name": input
            })).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap()["message"], expected);
        }
    }
}
```

#### 3.5.3 リリースチェックリスト

`RELEASE_CHECKLIST_v0.3.1.md`:

```markdown
# Release Checklist - v0.3.1

## コード品質
- [ ] 全ユニットテストがパス
- [ ] 全統合テストがパス
- [ ] リグレッションテストがパス
- [ ] cargo clippy で警告なし
- [ ] cargo fmt でフォーマット済み

## 機能確認
- [ ] list_filesツールが正常動作
- [ ] エラーメッセージが構造化されている
- [ ] Capability警告が表示される
- [ ] カスタムタイムアウトが機能する
- [ ] v0.3.0の全機能が継続動作

## パフォーマンス
- [ ] 主要ツールの応答時間 < 100ms
- [ ] パフォーマンス劣化なし（±10%以内）

## ドキュメント
- [ ] CHANGELOG.md 更新
- [ ] README.md 更新（必要に応じて）
- [ ] API仕様書更新（必要に応じて）
- [ ] トラブルシューティングガイド追加

## ビルド
- [ ] cargo build --release 成功
- [ ] Windows バイナリ生成確認
- [ ] バイナリサイズ確認（< 10MB）
- [ ] バイナリ動作確認

## リリース準備
- [ ] バージョン番号更新（Cargo.toml）
- [ ] リリースノート作成
- [ ] タグ作成（v0.3.1）
- [ ] GitHub Release作成

## 事後確認
- [ ] ユーザーテスト実施
- [ ] フィードバック収集
- [ ] 次フェーズへの引き継ぎ事項まとめ
```

#### 3.5.4 リリースノート

`RELEASE_NOTES_v0.3.1.md`:

```markdown
# Release Notes - v0.3.1

**リリース日**: 2025-12-01
**コードネーム**: Stability
**ステータス**: Production Ready

## 概要

v0.3.1は、v0.3.0で報告されたバグを完全に解決し、エラーハンドリングを大幅に改善した安定版です。三井情報株式会社の中葉雄大様によるテストで指摘された全ての問題に対応しました。

## 主な変更点

### 🐛 バグ修正

1. **list_filesツール無応答問題の解決** (#Critical)
   - タイムアウト時間を30秒に延長
   - サーバープロセス生存確認機能を追加
   - より詳細なエラーメッセージ

2. **エラーレポートの構造化** (#High)
   - すべてのエラーが構造化された形式で報告される
   - エラーの種類を明確に区別（Timeout, ServerCrash, InvalidResponse等）
   - ユーザーフレンドリーなエラーメッセージと改善提案

3. **Capability検証と警告** (#Medium)
   - サーバーが報告するCapabilityと実際の機能の矛盾を検出
   - 矛盾がある場合は警告を表示
   - server_inspect出力に検証結果を含める

### ✨ 新機能

1. **タイムアウトのカスタマイズ**
   - 環境変数でタイムアウトを設定可能
   - config.jsonでタイムアウトを設定可能
   - ツール実行、接続、リトライの各タイムアウトを個別設定

2. **改善されたエラーハンドリング**
   - すべてのエラーがJSON形式で構造化
   - タイムスタンプとリクエストID付与
   - エラーの種類に応じた適切な提案

### 📚 ドキュメント

- トラブルシューティングガイド追加
- タイムアウト設定ドキュメント追加
- エラーメッセージ一覧追加

## アップグレード方法

### 環境変数を使用する場合

```bash
# タイムアウトを60秒に設定
export MCP_TOOL_TIMEOUT_MS=60000

# mcp-inspectorを実行
mcp-inspector tools_call screening-server list_files --args '{"directory":"."}'
```

### config.jsonを使用する場合

```json
{
  "servers": [...],
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000
  }
}
```

## 互換性

- **後方互換性**: v0.3.0の全機能が継続動作
- **推奨**: Claude Desktopを再起動して最新バイナリを使用
- **動作確認済み**: rmcp 0.8.5

## 既知の問題

なし（v0.3.1時点）

## 謝辞

本リリースは、三井情報株式会社の中葉雄大様による詳細なテストレポートに基づいて実現しました。包括的なテストとフィードバックに深く感謝いたします。

## 次のステップ

Phase 2（v0.3.2）でドキュメント整備を実施予定です。
詳細は [開発計画書](docs/DEVELOPMENT_PLAN.md) をご参照ください。

---

**フィードバック**: GitHub Issuesまでお寄せください
**ダウンロード**: [GitHub Releases](https://github.com/nakata5577/mcp_inspector_mcp/releases/tag/v0.3.1)
```

**成果物**:
- [ ] 統合テストスイート
- [ ] リグレッションテスト
- [ ] リリースチェックリスト完了
- [ ] リリースノート
- [ ] v0.3.1タグとリリース

---

## 4. 開発環境準備

### 4.1 必要なツール

```bash
# Rust toolchain
rustc --version  # 1.70以上

# 依存ライブラリの確認
cd C:\Users\takah\work\my_mcp_server\mcp_inspector_mcp
cargo check

# テスト環境
cd C:\Users\takah\work\my_mcp_server\screening_server
cargo build --release
```

### 4.2 ブランチ戦略

```bash
# Phase 1開発用ブランチの作成
git checkout -b phase1/v0.3.1

# 各タスク用のfeatureブランチ
git checkout -b feature/fix-list-files
git checkout -b feature/structured-errors
git checkout -b feature/capability-validation
git checkout -b feature/timeout-config
```

### 4.3 プロジェクト管理

- GitHub Projectsでタスク管理
- 毎日の進捗をIssueにコメント
- 週次レビューミーティング（金曜日）

---

## 5. テスト計画

### 5.1 ユニットテスト

**カバレッジ目標**: 70%以上

```bash
# テストの実行
cargo test

# カバレッジ計測（tarpaulin使用）
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 5.2 統合テスト

**テスト対象サーバー**: screening-server

```bash
# screening-serverの起動
cd C:\Users\takah\work\my_mcp_server\screening_server
cargo run --release

# 統合テストの実行
cd C:\Users\takah\work\my_mcp_server\mcp_inspector_mcp
cargo test --test integration
```

### 5.3 パフォーマンステスト

**ベンチマーク基準**: v0.3.0と同等以上

```bash
# ベンチマークテストの実行
cargo bench

# 主要ツールの応答時間測定
./scripts/benchmark_tools.sh
```

### 5.4 リグレッションテスト

**v0.3.0の全機能が継続動作することを確認**

```bash
cargo test --test regression_v0_3_0
```

---

## 6. リリース手順

### 6.1 リリース前チェック

```bash
# 1. すべてのテストをパス
cargo test --all

# 2. Clippy チェック
cargo clippy -- -D warnings

# 3. フォーマットチェック
cargo fmt --check

# 4. リリースビルド
cargo build --release

# 5. バイナリ動作確認
./target/release/mcp_inspector_mcp.exe --version
```

### 6.2 バージョン更新

```toml
# Cargo.toml
[package]
name = "mcp_inspector_mcp"
version = "0.3.1"  # ← 更新
```

```markdown
# CHANGELOG.md に追加
## [0.3.1] - 2025-12-01

### Fixed
- list_filesツールの無応答問題を解決
- エラーメッセージを構造化

### Added
- Capability検証と警告機能
- タイムアウトのカスタマイズ機能

### Changed
- デフォルトタイムアウトを30秒に延長
```

### 6.3 Git操作

```bash
# 変更のコミット
git add .
git commit -m "feat(v0.3.1): Phase 1 完了 - 安定化改善

- list_filesツール問題の解決
- エラーレポートの構造化
- Capability検証と警告
- タイムアウト設定のカスタマイズ

Tested-by: Yudai Nakaba <nakaba-takahiro@mki.co.jp>

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"

# masterにマージ
git checkout master
git merge phase1/v0.3.1

# タグ作成
git tag -a v0.3.1 -m "Release v0.3.1 - Stability Improvements"

# プッシュ
git push origin master
git push origin v0.3.1
```

### 6.4 GitHub Release作成

1. GitHub Releasesページにアクセス
2. "Draft a new release"をクリック
3. タグ: v0.3.1
4. タイトル: "v0.3.1 - Stability Improvements"
5. 説明: RELEASE_NOTES_v0.3.1.md の内容をコピー
6. バイナリを添付
7. "Publish release"をクリック

---

## 7. リスク対応

### 7.1 高リスク項目

**Risk 1: list_files問題が複雑で解決に時間がかかる**

**対策**:
- Day 1に集中調査
- Day 2までに原因特定できない場合はエスカレーション
- 最悪の場合、Phase 1を1週間延長

**Risk 2: テスト時に新たなバグが発見される**

**対策**:
- 早期テストの徹底
- Critical Bugは最優先対応
- ホットフィックスブランチの準備

### 7.2 進捗モニタリング

**デイリースタンドアップ**:
- 毎日10:00に進捗確認
- 完了タスク、進行中タスク、ブロッカーを報告

**週次レビュー**:
- 毎週金曜日17:00にレビューミーティング
- 次週の計画調整

---

## 8. 成功基準の確認

### チェックリスト

- [ ] list_filesツールが95%以上の成功率
- [ ] すべてのエラーが構造化された形式
- [ ] Capability矛盾時に警告表示
- [ ] カスタムタイムアウトが機能
- [ ] v0.3.0の全機能が継続動作
- [ ] パフォーマンス劣化なし
- [ ] テストカバレッジ70%以上
- [ ] すべてのドキュメント更新完了

---

## 9. Phase 2への引き継ぎ

Phase 1完了後、以下をPhase 2に引き継ぎ：

1. **改善点リスト**
   - Phase 1で気づいた問題
   - ユーザーフィードバック

2. **ドキュメント要望**
   - よくある質問
   - トラブルシューティング事例

3. **技術的負債**
   - Phase 1で対応しきれなかった問題
   - リファクタリング候補

---

**Phase 1 詳細計画書 終了**

本計画に従って開発を進め、v0.3.1の安定版リリースを目指します。

**次のアクション**: 環境準備とTask 1.1（list_files問題調査）の開始

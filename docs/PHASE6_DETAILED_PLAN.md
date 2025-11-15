# Phase 6 詳細計画書: 高度な検査機能

**文書バージョン**: 1.0
**作成日**: 2025-11-15
**対象フェーズ**: Phase 6 - 高度な検査機能
**作成者**: Solution Architect

---

## 目次

1. [エグゼクティブサマリー](#1-エグゼクティブサマリー)
2. [技術要件定義](#2-技術要件定義)
3. [アーキテクチャ設計](#3-アーキテクチャ設計)
4. [詳細技術設計](#4-詳細技術設計)
5. [実装計画](#5-実装計画)
6. [テスト戦略](#6-テスト戦略)
7. [リスク分析と緩和策](#7-リスク分析と緩和策)
8. [完了基準](#8-完了基準)
9. [成果物リスト](#9-成果物リスト)
10. [スケジュール](#10-スケジュール)

---

## 1. エグゼクティブサマリー

### 1.1 Phase 6の目的とビジネス価値

Phase 6では、MCP Inspector MCPサーバーに**高度な検査機能**を追加し、対象サーバーの詳細な状態監視とデバッグ支援を実現します。これにより、開発者はMCPサーバーの健全性、設定、ログ出力を包括的に把握できるようになります。

**ビジネス価値**:
- **運用監視の強化**: サーバーのヘルスチェックにより、本番環境での安定運用を支援
- **デバッグ効率の向上**: ログ検査機能により、問題の早期発見と迅速な解決を実現
- **可視性の向上**: サーバー設定情報の取得により、構成管理と診断を容易化

### 1.2 主要な実装機能（3つ）

#### 機能1: サーバー設定検査（Server Inspection）
- サーバー情報（名前、バージョン、プロトコルバージョン）の取得
- サーバー機能（capabilities）の詳細表示
- 接続状態の確認

**優先度**: 最高（最もシンプルで、他機能の基盤となる）

#### 機能2: ヘルスチェック機能（Health Check）
- サーバーのヘルスステータス取得
- レスポンスタイム測定
- エラー率算出

**優先度**: 高（本番環境での運用監視に必須）

#### 機能3: Logging検査（Logging/Tracing Inspection）
- 対象サーバーのログメッセージ監視
- ログレベル設定の取得（可能であれば）
- ログメッセージの収集と保存

**優先度**: 中（rmcp 0.8.5の制約により実装範囲が限定的）

### 1.3 総推定工数とスケジュール

**総推定工数**: 29-42時間

| サブフェーズ | 工数 | 期間 |
|------------|------|------|
| Phase 6.1: サーバー設定検査 | 8-12時間 | 1-2週間 |
| Phase 6.2: ヘルスチェック機能 | 6-10時間 | 1週間 |
| Phase 6.3: Logging検査 | 15-20時間 | 2-3週間 |
| **合計** | **29-42時間** | **4-6週間** |

**推奨実装順序**:
1. Phase 6.1（サーバー設定検査）→ 最もシンプル、他機能の基盤
2. Phase 6.2（ヘルスチェック）→ 本番運用での価値が高い
3. Phase 6.3（Logging検査）→ 技術的に最も複雑

### 1.4 期待される成果

**技術的成果**:
- 新規ツール3-4個の追加
- 新規データモデル5-7個の実装
- 包括的なテストカバレッジ（単体テスト + 統合テスト）

**ビジネス的成果**:
- デバッグ効率の30%向上（ログ検査により問題特定が高速化）
- 本番環境での安定性向上（ヘルスチェックによる早期問題検出）
- 開発者体験の向上（サーバー情報の可視化）

---

## 2. 技術要件定義

### 2.1 MCPプロトコル調査

#### 2.1.1 rmcp 0.8.5でのサポート状況

**調査結果（2025-11-15時点）**:

| 機能 | MCPプロトコル仕様 | rmcp 0.8.5サポート | 実装可能性 |
|------|-------------------|-------------------|-----------|
| **サーバー情報取得** | `initialize`レスポンス | ✅ 完全サポート | ✅ 高 |
| **サーバー機能取得** | `ServerCapabilities` | ✅ 完全サポート | ✅ 高 |
| **ログメッセージ通知** | `notifications/message` | ✅ 部分サポート | ⚠️ 中 |
| **ログレベル設定** | `logging/setLevel` | ❌ 未サポート | ❌ 低 |
| **ピンク機能** | `ping`メソッド | ✅ 完全サポート | ✅ 高 |

**重要な発見**:

1. **rmcp 0.8.5にLogging型が存在**:
   ```rust
   // rmcp::model::LoggingLevel
   pub enum LoggingLevel {
       Debug, Info, Notice, Warning,
       Error, Critical, Alert, Emergency,
   }

   // rmcp::model::LoggingMessageNotificationParam
   pub struct LoggingMessageNotificationParam {
       pub level: LoggingLevel,
       pub logger: Option<String>,
       pub data: Value,
   }
   ```

2. **ServerCapabilitiesの構造**（推定）:
   ```rust
   pub struct ServerCapabilities {
       pub tools: Option<ToolsCapability>,
       pub resources: Option<ResourcesCapability>,
       pub prompts: Option<PromptsCapability>,
       pub logging: Option<LoggingCapability>,
       // その他の機能...
   }
   ```

3. **InitializeResultにサーバー情報が含まれる**:
   ```rust
   pub struct InitializeResult {
       pub protocol_version: ProtocolVersion,
       pub server_info: Implementation,
       pub capabilities: ServerCapabilities,
       // ...
   }

   pub struct Implementation {
       pub name: String,
       pub version: String,
       // ...
   }
   ```

#### 2.1.2 MCPプロトコル仕様の確認

**MCPプロトコル 2024-11-05仕様**:

1. **Logging機能**:
   - サーバーはログメッセージを`notifications/message`として送信可能
   - クライアントはログレベルの最小値を設定可能（`logging/setLevel`）
   - サーバーは`logging` capabilityを宣言する必要あり

2. **Ping機能**:
   - クライアントからサーバーへの疎通確認
   - サーバーは空のレスポンスを返す
   - レスポンスタイム測定に利用可能

3. **Initialize機能**:
   - プロトコルバージョン、サーバー情報、機能のネゴシエーション
   - 接続の最初に必ず実行される

#### 2.1.3 実装可能性の評価

| 機能 | 評価 | 理由 |
|------|------|------|
| **サーバー情報取得** | ✅ 完全実装可能 | `InitializeResult`から取得可能 |
| **ヘルスチェック** | ✅ 完全実装可能 | `ping`メソッド利用 |
| **ログメッセージ監視** | ⚠️ 部分実装可能 | 通知の受信は可能だが、Transport層での監視が必要 |
| **ログレベル設定** | ❌ 実装困難 | rmcp 0.8.5にクライアント側APIが不明 |

**結論**: Phase 6.1と6.2は完全実装可能。Phase 6.3は部分実装となる。

### 2.2 システム要件

#### 2.2.1 機能要件

**FR-6.1: サーバー設定検査**
- **FR-6.1.1**: サーバー名、バージョン、プロトコルバージョンを取得
- **FR-6.1.2**: サーバーの機能（tools, resources, prompts, logging等）を取得
- **FR-6.1.3**: 接続状態（connected/disconnected）を表示

**FR-6.2: ヘルスチェック**
- **FR-6.2.1**: pingメソッドでサーバーの疎通確認
- **FR-6.2.2**: レスポンスタイムを測定（ミリ秒単位）
- **FR-6.2.3**: 過去のエラー履歴からエラー率を算出
- **FR-6.2.4**: ヘルスステータス（Healthy/Degraded/Unhealthy）を判定

**FR-6.3: Logging検査**
- **FR-6.3.1**: ログメッセージ通知を受信して保存
- **FR-6.3.2**: ログレベル、タイムスタンプ、メッセージ本文を記録
- **FR-6.3.3**: ログ検索機能（サーバー名、レベル、時間範囲でフィルタ）
- **FR-6.3.4**: （オプション）ログレベル設定の取得

#### 2.2.2 非機能要件

**NFR-6.1: パフォーマンス**
- サーバー情報取得: 100ms以内
- ヘルスチェック: 500ms以内（ネットワーク遅延を除く）
- ログメッセージ保存: 10ms以内（非同期処理）

**NFR-6.2: 信頼性**
- ネットワークエラー時の適切なエラーハンドリング
- タイムアウト設定（デフォルト: 5秒）
- 部分的な障害時の継続動作（1サーバーの障害が他に影響しない）

**NFR-6.3: スケーラビリティ**
- ログメッセージ: 10,000件/サーバーまで保存可能
- 複数サーバーの同時監視: 10サーバーまで
- メモリ使用量: ログ10,000件あたり10-20MB以下

**NFR-6.4: 保守性**
- コードカバレッジ: 80%以上
- ドキュメント: 全公開APIにrustdoc形式のドキュメント
- エラーメッセージ: 開発者が問題を特定できる詳細なメッセージ

#### 2.2.3 制約条件

**技術的制約**:
1. rmcp 0.8.5の制約に従う（ログレベル設定APIが不明）
2. MCPプロトコル 2024-11-05仕様に準拠
3. 既存のアーキテクチャ（Phase 1-5）との整合性を保つ

**運用制約**:
1. ログ保存はPhase 5の永続化バックエンド（sled）を利用
2. 既存のサーバー設定ファイル（TOML）形式を踏襲
3. 後方互換性を維持（既存ツールへの影響なし）

---

## 3. アーキテクチャ設計

### 3.1 全体アーキテクチャ

#### 3.1.1 Phase 5後半までのアーキテクチャ（現状）

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Inspector MCP Server                │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            Server Layer (src/server/mod.rs)         │   │
│  │  - tools_list, tools_call                           │   │
│  │  - resources_list, resources_read                   │   │
│  │  - prompts_list, prompts_get                        │   │
│  │  - sampling_logs                                     │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│  ┌────────────────▼────────────────────────────────────┐   │
│  │        Service Layer (src/services/)                │   │
│  │  - InspectorService (inspector.rs)                  │   │
│  │  - SamplingLogger (sampling_logger.rs)              │   │
│  │  - ResponseCache (response_cache.rs)                │   │
│  │  - LoggerBackend (logger_backend.rs)                │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│  ┌────────────────▼────────────────────────────────────┐   │
│  │         Client Layer (src/client/)                  │   │
│  │  - ClientManager (manager.rs)                       │   │
│  │  - StdioClient (stdio_client.rs)                    │   │
│  │  - MonitoringTransport (monitoring_transport.rs)    │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│                   ▼                                          │
│            Target MCP Servers                                │
└─────────────────────────────────────────────────────────────┘
```

#### 3.1.2 Phase 6追加後のアーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Inspector MCP Server                │
│                                                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            Server Layer (src/server/mod.rs)         │   │
│  │  [既存ツール]                                        │   │
│  │  - tools_list, tools_call, resources_*, prompts_*   │   │
│  │  - sampling_logs                                     │   │
│  │  [新規ツール - Phase 6]                              │   │
│  │  - server_inspect       (6.1: サーバー設定検査)      │   │
│  │  - health_check         (6.2: ヘルスチェック)        │   │
│  │  - logging_messages     (6.3: ログメッセージ取得)    │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│  ┌────────────────▼────────────────────────────────────┐   │
│  │        Service Layer (src/services/)                │   │
│  │  [既存サービス]                                      │   │
│  │  - InspectorService                                 │   │
│  │  - SamplingLogger, ResponseCache                    │   │
│  │  [新規サービス - Phase 6]                            │   │
│  │  - ServerInfoService    (6.1: サーバー情報管理)      │   │
│  │  - HealthChecker        (6.2: ヘルスチェック)        │   │
│  │  - LoggingInspector     (6.3: ログ検査)              │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│  ┌────────────────▼────────────────────────────────────┐   │
│  │         Models Layer (src/models/)                  │   │
│  │  [新規モデル - Phase 6]                              │   │
│  │  - server_info.rs       (サーバー情報)               │   │
│  │  - health.rs            (ヘルスチェック)             │   │
│  │  - logging_inspection.rs (ログ検査)                  │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│  ┌────────────────▼────────────────────────────────────┐   │
│  │         Client Layer (src/client/)                  │   │
│  │  - ClientManager                                    │   │
│  │  - StdioClient (拡張: server_info取得, ping)        │   │
│  │  - MonitoringTransport (拡張: ログ通知受信)         │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                          │
│                   ▼                                          │
│            Target MCP Servers                                │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 モジュール設計

#### 3.2.1 新規モジュール一覧

| モジュール | ファイルパス | 役割 | 依存関係 |
|-----------|-------------|------|---------|
| **ServerInfoService** | `src/services/server_info_service.rs` | サーバー情報の取得と管理 | ClientManager |
| **HealthChecker** | `src/services/health_checker.rs` | ヘルスチェック実行とステータス判定 | ClientManager |
| **LoggingInspector** | `src/services/logging_inspector.rs` | ログメッセージの収集と検索 | LoggerBackend |
| **ServerInfoModels** | `src/models/server_info.rs` | サーバー情報のデータ構造 | - |
| **HealthModels** | `src/models/health.rs` | ヘルスチェックのデータ構造 | - |
| **LoggingInspectionModels** | `src/models/logging_inspection.rs` | ログ検査のデータ構造 | - |

#### 3.2.2 既存モジュールの拡張

| モジュール | 追加機能 | 理由 |
|-----------|---------|------|
| **InspectorService** | `server_inspect`, `health_check`, `logging_messages`メソッド | 新規ツールのエントリーポイント |
| **StdioClient** | `get_server_info`, `ping`メソッド | サーバー情報取得とピンク実行 |
| **MonitoringTransport** | ログ通知受信処理 | ログメッセージの監視 |
| **InspectorServer** | 新規ツール定義 | MCPサーバーとしてツールを公開 |

### 3.3 データフロー図

#### 3.3.1 サーバー設定検査のデータフロー

```
AIエージェント
    │
    │ 1. server_inspect(server="target-server")
    │
    ▼
InspectorServer (tools)
    │
    │ 2. call server_inspect
    │
    ▼
InspectorService
    │
    │ 3. get_server_info("target-server")
    │
    ▼
ServerInfoService
    │
    │ 4. request server info
    │
    ▼
ClientManager → StdioClient
    │
    │ 5. MCP: initialize (if not connected)
    │
    ▼
Target MCP Server
    │
    │ 6. InitializeResult
    │    {
    │      protocol_version: "2024-11-05",
    │      server_info: { name, version },
    │      capabilities: { tools, resources, ... }
    │    }
    │
    ▼
StdioClient (cache server_info)
    │
    │ 7. return ServerInfo
    │
    ▼
ServerInfoService
    │
    │ 8. format response
    │
    ▼
InspectorService → InspectorServer → AIエージェント
    │
    │ ServerInspectResponse {
    │   server_name: "target-server",
    │   protocol_version: "2024-11-05",
    │   server_info: { name: "...", version: "..." },
    │   capabilities: { ... },
    │   connection_status: "connected"
    │ }
```

#### 3.3.2 ヘルスチェックのデータフロー

```
AIエージェント
    │
    │ 1. health_check(server="target-server")
    │
    ▼
InspectorServer (tools)
    │
    │ 2. call health_check
    │
    ▼
InspectorService
    │
    │ 3. perform_health_check("target-server")
    │
    ▼
HealthChecker
    │
    │ 4. measure ping response time
    │
    ▼
ClientManager → StdioClient
    │
    │ 5. start_time = now()
    │ 6. MCP: ping
    │
    ▼
Target MCP Server
    │
    │ 7. pong (empty response)
    │
    ▼
StdioClient
    │
    │ 8. elapsed = now() - start_time
    │
    ▼
HealthChecker
    │
    │ 9. calculate error_rate from history
    │ 10. determine status (Healthy/Degraded/Unhealthy)
    │
    ▼
InspectorService → InspectorServer → AIエージェント
    │
    │ HealthCheckResponse {
    │   server_name: "target-server",
    │   status: "Healthy",
    │   response_time_ms: 45,
    │   last_check: "2025-11-15T10:30:00Z",
    │   error_count: 0,
    │   error_rate: 0.0
    │ }
```

#### 3.3.3 Logging検査のデータフロー

```
Target MCP Server
    │
    │ 1. notifications/message (logging)
    │    {
    │      level: "info",
    │      logger: "app.service",
    │      data: { message: "Request processed" }
    │    }
    │
    ▼
MonitoringTransport (receive notification)
    │
    │ 2. detect logging message
    │
    ▼
LoggingInspector
    │
    │ 3. create LogEntry
    │    {
    │      timestamp: "2025-11-15T10:30:00Z",
    │      server_name: "target-server",
    │      level: "Info",
    │      logger: "app.service",
    │      message: "Request processed"
    │    }
    │
    │ 4. save to LoggerBackend
    │
    ▼
PersistentLogger (sled DB)


[別フロー: ログ取得]

AIエージェント
    │
    │ 1. logging_messages(server="target-server", level="info")
    │
    ▼
InspectorServer (tools)
    │
    │ 2. call logging_messages
    │
    ▼
InspectorService
    │
    │ 3. get_logging_messages("target-server", level, limit)
    │
    ▼
LoggingInspector
    │
    │ 4. query LoggerBackend
    │
    ▼
PersistentLogger (sled DB)
    │
    │ 5. return LogEntry[]
    │
    ▼
LoggingInspector
    │
    │ 6. format response
    │
    ▼
InspectorService → InspectorServer → AIエージェント
    │
    │ LoggingMessagesResponse {
    │   server_name: "target-server",
    │   messages: [ ... ],
    │   total_count: 150
    │ }
```

---

## 4. 詳細技術設計

### 4.1 Phase 6.1: サーバー設定検査

#### 4.1.1 新規ツール: `server_inspect`

**ツール定義**:

```rust
/// Parameters for server_inspect tool
#[derive(Deserialize, JsonSchema)]
struct ServerInspectParams {
    /// Name of the MCP server to inspect
    server: String,
}

#[tool(
    name = "server_inspect",
    description = "指定されたMCPサーバーの設定情報を取得します（サーバー名、バージョン、プロトコルバージョン、機能等）"
)]
async fn server_inspect(
    &self,
    params: Parameters<ServerInspectParams>,
) -> Result<CallToolResult, McpError> {
    let result = self
        .inspector
        .server_inspect(&params.0.server)
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("Failed to inspect server: {}", e).into(),
            data: None,
        })?;

    let json_result = serde_json::to_value(&result).map_err(|e| McpError {
        code: ErrorCode(-32603),
        message: format!("JSON serialization error: {}", e).into(),
        data: None,
    })?;

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json_result)
            .unwrap_or_else(|_| json_result.to_string()),
    )]))
}
```

#### 4.1.2 データ構造（`src/models/server_info.rs`）

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Request for server inspection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerInspectRequest {
    pub server: String,
}

/// Response from server inspection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerInspectResponse {
    /// Server name from configuration
    pub server_name: String,

    /// MCP protocol version (e.g., "2024-11-05")
    pub protocol_version: String,

    /// Server implementation information
    pub server_info: ServerImplementation,

    /// Server capabilities
    pub capabilities: ServerCapabilitiesInfo,

    /// Connection status
    pub connection_status: ConnectionStatus,
}

/// Server implementation details
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerImplementation {
    /// Server implementation name
    pub name: String,

    /// Server implementation version
    pub version: String,
}

/// Server capabilities information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerCapabilitiesInfo {
    /// Tools capability
    pub tools: bool,

    /// Resources capability
    pub resources: bool,

    /// Prompts capability
    pub prompts: bool,

    /// Sampling capability
    pub sampling: bool,

    /// Logging capability
    pub logging: bool,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}
```

#### 4.1.3 サービス実装（`src/services/server_info_service.rs`）

```rust
use crate::client::ClientManager;
use crate::models::{
    ConnectionStatus, ServerCapabilitiesInfo, ServerImplementation, ServerInspectResponse,
};
use anyhow::{Context, Result};
use std::sync::Arc;

/// Service for inspecting server information
pub struct ServerInfoService {
    client_manager: Arc<ClientManager>,
}

impl ServerInfoService {
    pub fn new(client_manager: Arc<ClientManager>) -> Self {
        Self { client_manager }
    }

    /// Get comprehensive server information
    pub async fn get_server_info(&self, server_name: &str) -> Result<ServerInspectResponse> {
        // Get client (will connect if not already connected)
        let client = self
            .client_manager
            .get_client(server_name)
            .await
            .context("Failed to get client")?;

        // Get server info from InitializeResult (cached in client)
        let init_result = client
            .get_init_result()
            .await
            .context("Failed to get initialization result")?;

        // Check connection status
        let connection_status = if client.is_connected().await {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        // Extract server implementation info
        let server_info = ServerImplementation {
            name: init_result.server_info.name.to_string(),
            version: init_result.server_info.version.to_string(),
        };

        // Extract capabilities
        let capabilities = ServerCapabilitiesInfo {
            tools: init_result.capabilities.tools.is_some(),
            resources: init_result.capabilities.resources.is_some(),
            prompts: init_result.capabilities.prompts.is_some(),
            sampling: init_result.capabilities.sampling.is_some(),
            logging: init_result.capabilities.logging.is_some(),
        };

        Ok(ServerInspectResponse {
            server_name: server_name.to_string(),
            protocol_version: format!("{:?}", init_result.protocol_version),
            server_info,
            capabilities,
            connection_status,
        })
    }
}
```

#### 4.1.4 StdioClientの拡張

```rust
// src/client/stdio_client.rs

use rmcp::model::InitializeResult;

impl StdioClient {
    /// Get cached initialization result
    ///
    /// Returns the InitializeResult from the MCP handshake.
    /// If not connected, this will return an error.
    pub async fn get_init_result(&self) -> Result<InitializeResult> {
        let guard = self.service.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to server"))?;

        // Get peer_info which contains InitializeResult
        let peer_info = service.peer_info();

        Ok(peer_info.clone())
    }

    /// Check if client is connected
    pub async fn is_connected(&self) -> bool {
        let guard = self.service.lock().await;
        guard.is_some()
    }
}
```

#### 4.1.5 InspectorServiceへの統合

```rust
// src/services/inspector.rs

use crate::services::ServerInfoService;

impl InspectorService {
    pub fn new(config: InspectorConfig) -> anyhow::Result<Self> {
        // ... existing code ...

        let server_info_service = Arc::new(ServerInfoService::new(
            Arc::clone(&client_manager)
        ));

        Ok(Self {
            client_manager,
            sampling_logger,
            response_cache,
            server_info_service, // 追加
        })
    }

    /// Get server information
    pub async fn server_inspect(&self, server_name: &str) -> Result<ServerInspectResponse> {
        self.server_info_service
            .get_server_info(server_name)
            .await
    }
}
```

#### 4.1.6 実装ファイル一覧

| ファイル | 種類 | 行数（推定） | 内容 |
|---------|------|-------------|------|
| `src/models/server_info.rs` | 新規 | 80-100 | データ構造定義 |
| `src/services/server_info_service.rs` | 新規 | 100-120 | サービス実装 |
| `src/client/stdio_client.rs` | 修正 | +30 | `get_init_result`, `is_connected`メソッド追加 |
| `src/services/inspector.rs` | 修正 | +20 | `server_inspect`メソッド追加 |
| `src/server/mod.rs` | 修正 | +40 | `server_inspect`ツール定義 |

#### 4.1.7 技術的課題と解決策

**課題1: InitializeResultのキャッシュ方法**
- **問題**: `peer_info()`の戻り値の型が不明
- **解決策**: rmcpのドキュメントを確認。必要に応じてInitializeResultをStdioClient内部で保持

**課題2: ServerCapabilitiesの構造**
- **問題**: rmcp 0.8.5の`ServerCapabilities`の正確な構造が不明
- **解決策**: 実装時にrmcpのソースコードを確認し、適切にマッピング

**課題3: 接続状態の判定**
- **問題**: `is_connected()`の実装方法
- **解決策**: `service: Arc<Mutex<Option<RunningService>>>`の`Option`の有無で判定

#### 4.1.8 実装工数

| タスク | 工数 |
|-------|------|
| データ構造定義 | 2時間 |
| ServerInfoService実装 | 3時間 |
| StdioClient拡張 | 2時間 |
| InspectorServiceへの統合 | 1時間 |
| ツール定義 | 1時間 |
| 単体テスト | 2時間 |
| 統合テスト | 1時間 |
| **合計** | **12時間** |

### 4.2 Phase 6.2: ヘルスチェック機能

#### 4.2.1 新規ツール: `health_check`

**ツール定義**:

```rust
/// Parameters for health_check tool
#[derive(Deserialize, JsonSchema)]
struct HealthCheckParams {
    /// Name of the MCP server to health check
    server: String,
}

#[tool(
    name = "health_check",
    description = "指定されたMCPサーバーのヘルスチェックを実行します（疎通確認、レスポンスタイム測定、エラー率算出）"
)]
async fn health_check(
    &self,
    params: Parameters<HealthCheckParams>,
) -> Result<CallToolResult, McpError> {
    let result = self
        .inspector
        .health_check(&params.0.server)
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("Failed to perform health check: {}", e).into(),
            data: None,
        })?;

    let json_result = serde_json::to_value(&result).map_err(|e| McpError {
        code: ErrorCode(-32603),
        message: format!("JSON serialization error: {}", e).into(),
        data: None,
    })?;

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json_result)
            .unwrap_or_else(|_| json_result.to_string()),
    )]))
}
```

#### 4.2.2 データ構造（`src/models/health.rs`）

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Request for health check
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthCheckRequest {
    pub server: String,
}

/// Response from health check
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HealthCheckResponse {
    /// Server name
    pub server_name: String,

    /// Health status
    pub status: HealthStatus,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Timestamp of last check (RFC3339 format)
    pub last_check: String,

    /// Total error count in history
    pub error_count: u64,

    /// Error rate (0.0-1.0)
    pub error_rate: f64,

    /// Details about the health check
    pub details: Option<String>,
}

/// Health status of a server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum HealthStatus {
    /// Server is healthy (response time < 500ms, error rate < 5%)
    Healthy,

    /// Server is degraded (response time < 2000ms, error rate < 20%)
    Degraded,

    /// Server is unhealthy (response time >= 2000ms or error rate >= 20%)
    Unhealthy,
}

/// Internal structure to track health history
#[derive(Debug, Clone)]
pub struct HealthHistory {
    /// Server name
    pub server_name: String,

    /// Circular buffer of recent check results (success/failure)
    pub recent_checks: Vec<HealthCheckResult>,

    /// Maximum history size
    pub max_history: usize,
}

/// Result of a single health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Success or failure
    pub success: bool,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Error message (if failed)
    pub error_message: Option<String>,
}
```

#### 4.2.3 サービス実装（`src/services/health_checker.rs`）

```rust
use crate::client::ClientManager;
use crate::models::{
    HealthCheckResponse, HealthCheckResult, HealthHistory, HealthStatus,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Service for health checking MCP servers
pub struct HealthChecker {
    client_manager: Arc<ClientManager>,
    /// Health history for each server
    history: Arc<RwLock<HashMap<String, HealthHistory>>>,
    /// Maximum history size per server
    max_history: usize,
}

impl HealthChecker {
    pub fn new(client_manager: Arc<ClientManager>) -> Self {
        Self {
            client_manager,
            history: Arc::new(RwLock::new(HashMap::new())),
            max_history: 100, // Keep last 100 checks per server
        }
    }

    /// Perform health check on a server
    pub async fn check_health(&self, server_name: &str) -> Result<HealthCheckResponse> {
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        // Attempt to ping the server
        let client = self
            .client_manager
            .get_client(server_name)
            .await
            .context("Failed to get client")?;

        let ping_result = client.ping().await;
        let elapsed = start_time.elapsed();
        let response_time_ms = elapsed.as_millis() as u64;

        // Record the result
        let check_result = HealthCheckResult {
            timestamp,
            success: ping_result.is_ok(),
            response_time_ms,
            error_message: ping_result.as_ref().err().map(|e| e.to_string()),
        };

        // Update history
        self.update_history(server_name, check_result.clone()).await;

        // Calculate error rate
        let (error_count, error_rate) = self.calculate_error_rate(server_name).await;

        // Determine health status
        let status = Self::determine_status(response_time_ms, error_rate);

        Ok(HealthCheckResponse {
            server_name: server_name.to_string(),
            status,
            response_time_ms,
            last_check: timestamp.to_rfc3339(),
            error_count,
            error_rate,
            details: check_result.error_message,
        })
    }

    /// Update health history for a server
    async fn update_history(&self, server_name: &str, result: HealthCheckResult) {
        let mut history = self.history.write().await;

        let server_history = history.entry(server_name.to_string()).or_insert_with(|| {
            HealthHistory {
                server_name: server_name.to_string(),
                recent_checks: Vec::new(),
                max_history: self.max_history,
            }
        });

        server_history.recent_checks.push(result);

        // Maintain circular buffer
        if server_history.recent_checks.len() > server_history.max_history {
            server_history.recent_checks.remove(0);
        }
    }

    /// Calculate error rate from history
    async fn calculate_error_rate(&self, server_name: &str) -> (u64, f64) {
        let history = self.history.read().await;

        let server_history = match history.get(server_name) {
            Some(h) => h,
            None => return (0, 0.0),
        };

        if server_history.recent_checks.is_empty() {
            return (0, 0.0);
        }

        let total_checks = server_history.recent_checks.len() as u64;
        let error_count = server_history
            .recent_checks
            .iter()
            .filter(|c| !c.success)
            .count() as u64;

        let error_rate = error_count as f64 / total_checks as f64;

        (error_count, error_rate)
    }

    /// Determine health status based on metrics
    fn determine_status(response_time_ms: u64, error_rate: f64) -> HealthStatus {
        // Unhealthy: response time >= 2000ms or error rate >= 20%
        if response_time_ms >= 2000 || error_rate >= 0.2 {
            return HealthStatus::Unhealthy;
        }

        // Degraded: response time >= 500ms or error rate >= 5%
        if response_time_ms >= 500 || error_rate >= 0.05 {
            return HealthStatus::Degraded;
        }

        // Healthy
        HealthStatus::Healthy
    }

    /// Get health history for a server (for testing/debugging)
    #[cfg(test)]
    pub async fn get_history(&self, server_name: &str) -> Option<HealthHistory> {
        let history = self.history.read().await;
        history.get(server_name).cloned()
    }
}
```

#### 4.2.4 StdioClientのping実装

```rust
// src/client/stdio_client.rs

impl StdioClient {
    /// Ping the server to check connectivity
    ///
    /// Returns Ok(()) if the server responds successfully.
    pub async fn ping(&self) -> Result<()> {
        let guard = self.service.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to server"))?;

        // Call ping method
        service
            .ping()
            .await
            .context("Ping failed")?;

        Ok(())
    }
}
```

#### 4.2.5 InspectorServiceへの統合

```rust
// src/services/inspector.rs

use crate::services::HealthChecker;

impl InspectorService {
    pub fn new(config: InspectorConfig) -> anyhow::Result<Self> {
        // ... existing code ...

        let health_checker = Arc::new(HealthChecker::new(
            Arc::clone(&client_manager)
        ));

        Ok(Self {
            client_manager,
            sampling_logger,
            response_cache,
            server_info_service,
            health_checker, // 追加
        })
    }

    /// Perform health check on a server
    pub async fn health_check(&self, server_name: &str) -> Result<HealthCheckResponse> {
        self.health_checker
            .check_health(server_name)
            .await
    }
}
```

#### 4.2.6 実装ファイル一覧

| ファイル | 種類 | 行数（推定） | 内容 |
|---------|------|-------------|------|
| `src/models/health.rs` | 新規 | 100-120 | データ構造定義 |
| `src/services/health_checker.rs` | 新規 | 150-180 | ヘルスチェック実装 |
| `src/client/stdio_client.rs` | 修正 | +15 | `ping`メソッド追加 |
| `src/services/inspector.rs` | 修正 | +15 | `health_check`メソッド追加 |
| `src/server/mod.rs` | 修正 | +35 | `health_check`ツール定義 |

#### 4.2.7 技術的課題と解決策

**課題1: pingメソッドの実装**
- **問題**: rmcp 0.8.5でのpingメソッドの呼び出し方が不明
- **解決策**: rmcpのドキュメントとサンプルコードを確認。`service.ping()`で呼び出し可能と推定

**課題2: ヘルス履歴の保存方法**
- **問題**: メモリベースか永続化か
- **解決策**: 初期実装はメモリベース。将来的に永続化も検討（オプション）

**課題3: ヘルスステータスの判定基準**
- **問題**: 閾値の決定
- **解決策**:
  - Healthy: response_time < 500ms, error_rate < 5%
  - Degraded: response_time < 2000ms, error_rate < 20%
  - Unhealthy: response_time >= 2000ms or error_rate >= 20%

#### 4.2.8 実装工数

| タスク | 工数 |
|-------|------|
| データ構造定義 | 2時間 |
| HealthChecker実装 | 4時間 |
| StdioClient ping実装 | 1時間 |
| InspectorServiceへの統合 | 1時間 |
| ツール定義 | 1時間 |
| 単体テスト | 2時間 |
| 統合テスト | 1時間 |
| **合計** | **12時間** |

**実際の工数範囲**: 10-14時間（楽観値10時間、悲観値14時間）

### 4.3 Phase 6.3: Logging検査

#### 4.3.1 新規ツール: `logging_messages`

**ツール定義**:

```rust
/// Parameters for logging_messages tool
#[derive(Deserialize, JsonSchema)]
struct LoggingMessagesParams {
    /// Name of the MCP server
    server: String,

    /// Minimum log level to retrieve (Debug, Info, Warning, Error, etc.)
    #[serde(default)]
    level: Option<String>,

    /// Maximum number of messages to return
    #[serde(default)]
    limit: Option<usize>,

    /// Start time (RFC3339 format)
    #[serde(default)]
    since: Option<String>,
}

#[tool(
    name = "logging_messages",
    description = "指定されたMCPサーバーのログメッセージを取得します"
)]
async fn logging_messages(
    &self,
    params: Parameters<LoggingMessagesParams>,
) -> Result<CallToolResult, McpError> {
    let request = LoggingMessagesRequest {
        server: params.0.server.clone(),
        level: params.0.level.clone(),
        limit: params.0.limit.unwrap_or(100),
        since: params.0.since.clone(),
    };

    let result = self
        .inspector
        .logging_messages(request)
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("Failed to get logging messages: {}", e).into(),
            data: None,
        })?;

    let json_result = serde_json::to_value(&result).map_err(|e| McpError {
        code: ErrorCode(-32603),
        message: format!("JSON serialization error: {}", e).into(),
        data: None,
    })?;

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json_result)
            .unwrap_or_else(|_| json_result.to_string()),
    )]))
}
```

#### 4.3.2 データ構造（`src/models/logging_inspection.rs`）

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Request for logging messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingMessagesRequest {
    /// Server name
    pub server: String,

    /// Minimum log level (Debug, Info, Notice, Warning, Error, Critical, Alert, Emergency)
    pub level: Option<String>,

    /// Maximum number of messages to return
    pub limit: usize,

    /// Start time (RFC3339 format)
    pub since: Option<String>,
}

/// Response with logging messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingMessagesResponse {
    /// Server name
    pub server_name: String,

    /// Log messages
    pub messages: Vec<LogEntry>,

    /// Total count of messages matching the criteria
    pub total_count: usize,
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    /// Timestamp (RFC3339 format)
    pub timestamp: String,

    /// Server name
    pub server_name: String,

    /// Log level
    pub level: LogLevel,

    /// Logger name (optional)
    pub logger: Option<String>,

    /// Log message or data
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl From<rmcp::model::LoggingLevel> for LogLevel {
    fn from(level: rmcp::model::LoggingLevel) -> Self {
        match level {
            rmcp::model::LoggingLevel::Debug => LogLevel::Debug,
            rmcp::model::LoggingLevel::Info => LogLevel::Info,
            rmcp::model::LoggingLevel::Notice => LogLevel::Notice,
            rmcp::model::LoggingLevel::Warning => LogLevel::Warning,
            rmcp::model::LoggingLevel::Error => LogLevel::Error,
            rmcp::model::LoggingLevel::Critical => LogLevel::Critical,
            rmcp::model::LoggingLevel::Alert => LogLevel::Alert,
            rmcp::model::LoggingLevel::Emergency => LogLevel::Emergency,
        }
    }
}
```

#### 4.3.3 サービス実装（`src/services/logging_inspector.rs`）

```rust
use crate::models::{LogEntry, LogLevel, LoggingMessagesRequest, LoggingMessagesResponse};
use crate::services::LoggerBackend;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Service for inspecting logging messages
pub struct LoggingInspector {
    /// Logger backend for storing log entries
    logger_backend: Arc<dyn LoggerBackend>,
}

impl LoggingInspector {
    pub fn new(logger_backend: Arc<dyn LoggerBackend>) -> Self {
        Self { logger_backend }
    }

    /// Add a log entry (called by MonitoringTransport)
    pub fn add_log_entry(&self, entry: LogEntry) -> Result<()> {
        // Store in logger backend using server name as key
        // Note: We'll need to extend LoggerBackend to support log entries
        // For now, we can serialize LogEntry to JSON and store as SamplingLogEntry

        // TODO: Implement proper storage
        // This is a placeholder - actual implementation will depend on
        // how we extend LoggerBackend

        Ok(())
    }

    /// Get logging messages
    pub fn get_logging_messages(
        &self,
        request: LoggingMessagesRequest,
    ) -> Result<LoggingMessagesResponse> {
        // Parse level filter
        let level_filter = request.level.as_ref().and_then(|l| {
            match l.to_lowercase().as_str() {
                "debug" => Some(LogLevel::Debug),
                "info" => Some(LogLevel::Info),
                "notice" => Some(LogLevel::Notice),
                "warning" => Some(LogLevel::Warning),
                "error" => Some(LogLevel::Error),
                "critical" => Some(LogLevel::Critical),
                "alert" => Some(LogLevel::Alert),
                "emergency" => Some(LogLevel::Emergency),
                _ => None,
            }
        });

        // Parse time filter
        let since_filter = request.since.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s).ok()
        });

        // Query logger backend
        // TODO: Implement actual query logic
        // This is a placeholder

        let messages = Vec::new(); // TODO: Get from storage
        let total_count = 0; // TODO: Get from storage

        Ok(LoggingMessagesResponse {
            server_name: request.server.clone(),
            messages,
            total_count,
        })
    }
}
```

#### 4.3.4 MonitoringTransportの拡張（ログ通知受信）

```rust
// src/client/monitoring_transport.rs

use rmcp::model::LoggingMessageNotificationParam;
use crate::models::{LogEntry, LogLevel};
use crate::services::LoggingInspector;

impl<T: Transport> MonitoringTransport<T> {
    /// Check if message is a logging notification
    fn is_logging_notification(message: &JsonRpcMessage) -> bool {
        message.method.as_ref().map_or(false, |m| {
            m == "notifications/message"
        })
    }

    /// Handle logging notification
    async fn handle_logging_notification(
        &self,
        message: &JsonRpcMessage,
    ) {
        if let Some(params) = &message.params {
            // Parse logging notification
            if let Ok(notification) = serde_json::from_value::<LoggingMessageNotificationParam>(params.clone()) {
                // Create LogEntry
                let entry = LogEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    server_name: self.server_name.clone(),
                    level: LogLevel::from(notification.level),
                    logger: notification.logger.clone(),
                    message: notification.data.to_string(),
                };

                // Store log entry
                if let Some(logging_inspector) = &self.logging_inspector {
                    if let Err(e) = logging_inspector.add_log_entry(entry) {
                        tracing::warn!(
                            server = self.server_name.as_str(),
                            error = ?e,
                            "Failed to store log entry"
                        );
                    }
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl<T: Transport> Transport for MonitoringTransport<T> {
    async fn receive(&mut self) -> Result<JsonRpcMessage> {
        let message = self.inner.receive().await?;

        // Check for logging notification
        if Self::is_logging_notification(&message) {
            self.handle_logging_notification(&message).await;
        }

        Ok(message)
    }
}
```

#### 4.3.5 LoggerBackendの拡張（オプション）

**選択肢1: 既存のLoggerBackendを拡張**

```rust
// src/services/logger_backend.rs

pub trait LoggerBackend: Send + Sync + std::fmt::Debug {
    // Existing methods for sampling logs
    fn add_log(&self, server_name: &str, entry: SamplingLogEntry) -> Result<()>;
    fn get_logs(
        &self,
        server_name: &str,
        limit: Option<usize>,
        status: Option<SamplingStatus>,
    ) -> Result<Vec<SamplingLogEntry>>;
    fn clear_logs(&self, server_name: &str) -> Result<()>;

    // New methods for logging inspection
    fn add_log_message(&self, entry: LogEntry) -> Result<()>;
    fn get_log_messages(
        &self,
        server_name: &str,
        level: Option<LogLevel>,
        limit: usize,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntry>>;
}
```

**選択肢2: 別のストレージを使用**

ログメッセージ専用のストレージを作成（`LogMessageStorage`）

**推奨**: 選択肢1（既存のLoggerBackendを拡張）を採用。実装の一貫性とコード再利用性が高い。

#### 4.3.6 実装ファイル一覧

| ファイル | 種類 | 行数（推定） | 内容 |
|---------|------|-------------|------|
| `src/models/logging_inspection.rs` | 新規 | 120-150 | データ構造定義 |
| `src/services/logging_inspector.rs` | 新規 | 150-200 | ログ検査実装 |
| `src/client/monitoring_transport.rs` | 修正 | +80 | ログ通知受信処理 |
| `src/services/logger_backend.rs` | 修正 | +30 | ログメッセージ用メソッド追加 |
| `src/services/persistent_logger.rs` | 修正 | +100 | ログメッセージ保存実装 |
| `src/services/memory_logger.rs` | 修正 | +80 | ログメッセージ保存実装 |
| `src/services/inspector.rs` | 修正 | +20 | `logging_messages`メソッド追加 |
| `src/server/mod.rs` | 修正 | +45 | `logging_messages`ツール定義 |

#### 4.3.7 技術的課題と解決策

**課題1: ログ通知の受信タイミング**
- **問題**: `notifications/message`はいつ送られてくるか不明
- **解決策**: MonitoringTransportの`receive()`メソッドで常時監視。非同期で受信

**課題2: ログストレージの設計**
- **問題**: SamplingログとLoggingログを同じストレージに保存するか分離するか
- **解決策**: 同じLoggerBackendを使用し、キー設計で区別
  - Samplingログ: `sampling:{server_name}:{timestamp}`
  - Loggingログ: `logging:{server_name}:{timestamp}`

**課題3: ログレベル設定の実装**
- **問題**: rmcp 0.8.5でのクライアント側`logging/setLevel` APIが不明
- **解決策**: Phase 6.3では実装を見送り、将来的にrmcpがサポートした際に追加

**課題4: 大量のログメッセージの処理**
- **問題**: ログが大量に送信された場合のパフォーマンス
- **解決策**:
  - 非同期処理による即座のレスポンス
  - ローテーション機能によるメモリ管理
  - バッチ処理の検討（将来）

#### 4.3.8 実装工数

| タスク | 工数 |
|-------|------|
| データ構造定義 | 3時間 |
| LoggingInspector実装 | 4時間 |
| MonitoringTransport拡張 | 4時間 |
| LoggerBackend拡張 | 3時間 |
| PersistentLogger/MemoryLogger拡張 | 5時間 |
| InspectorServiceへの統合 | 1時間 |
| ツール定義 | 1時間 |
| 単体テスト | 4時間 |
| 統合テスト | 3時間 |
| **合計** | **28時間** |

**実際の工数範囲**: 20-32時間（楽観値20時間、悲観値32時間）

### 4.4 Phase 6全体の技術スタック

| レイヤー | 技術 | 用途 |
|---------|------|------|
| **プロトコル** | MCP 2024-11-05 | MCPプロトコル仕様 |
| **SDK** | rmcp 0.8.5 | Rust MCP SDK |
| **非同期ランタイム** | tokio 1.x | 非同期処理 |
| **シリアライゼーション** | serde, serde_json | JSON処理 |
| **エラーハンドリング** | anyhow | エラー管理 |
| **ロギング** | tracing | デバッグログ |
| **時刻処理** | chrono | タイムスタンプ管理 |
| **ストレージ** | sled (既存) | ログ永続化 |

---

## 5. 実装計画

### 5.1 タスク分解

#### 5.1.1 Phase 6.1: サーバー設定検査

| タスクID | タスク名 | 種類 | 工数 | 依存関係 |
|---------|---------|------|------|---------|
| 6.1.1 | データ構造定義（`server_info.rs`） | 開発 | 2時間 | - |
| 6.1.2 | ServerInfoService実装 | 開発 | 3時間 | 6.1.1 |
| 6.1.3 | StdioClient拡張（`get_init_result`, `is_connected`） | 開発 | 2時間 | - |
| 6.1.4 | InspectorServiceへの統合 | 開発 | 1時間 | 6.1.2, 6.1.3 |
| 6.1.5 | ツール定義（`server_inspect`） | 開発 | 1時間 | 6.1.4 |
| 6.1.6 | 単体テスト作成 | テスト | 2時間 | 6.1.2, 6.1.3 |
| 6.1.7 | 統合テスト作成 | テスト | 1時間 | 6.1.5 |
| 6.1.8 | ドキュメント作成 | ドキュメント | 1時間 | 6.1.7 |

**合計**: 13時間

#### 5.1.2 Phase 6.2: ヘルスチェック機能

| タスクID | タスク名 | 種類 | 工数 | 依存関係 |
|---------|---------|------|------|---------|
| 6.2.1 | データ構造定義（`health.rs`） | 開発 | 2時間 | - |
| 6.2.2 | HealthChecker実装 | 開発 | 4時間 | 6.2.1 |
| 6.2.3 | StdioClient ping実装 | 開発 | 1時間 | - |
| 6.2.4 | InspectorServiceへの統合 | 開発 | 1時間 | 6.2.2, 6.2.3 |
| 6.2.5 | ツール定義（`health_check`） | 開発 | 1時間 | 6.2.4 |
| 6.2.6 | 単体テスト作成 | テスト | 2時間 | 6.2.2, 6.2.3 |
| 6.2.7 | 統合テスト作成 | テスト | 1時間 | 6.2.5 |
| 6.2.8 | ドキュメント作成 | ドキュメント | 1時間 | 6.2.7 |

**合計**: 13時間

#### 5.1.3 Phase 6.3: Logging検査

| タスクID | タスク名 | 種類 | 工数 | 依存関係 |
|---------|---------|------|------|---------|
| 6.3.1 | データ構造定義（`logging_inspection.rs`） | 開発 | 3時間 | - |
| 6.3.2 | LoggingInspector実装 | 開発 | 4時間 | 6.3.1 |
| 6.3.3 | MonitoringTransport拡張（ログ通知受信） | 開発 | 4時間 | 6.3.2 |
| 6.3.4 | LoggerBackend拡張 | 開発 | 3時間 | 6.3.1 |
| 6.3.5 | PersistentLogger/MemoryLogger拡張 | 開発 | 5時間 | 6.3.4 |
| 6.3.6 | InspectorServiceへの統合 | 開発 | 1時間 | 6.3.2 |
| 6.3.7 | ツール定義（`logging_messages`） | 開発 | 1時間 | 6.3.6 |
| 6.3.8 | 単体テスト作成 | テスト | 4時間 | 6.3.2, 6.3.5 |
| 6.3.9 | 統合テスト作成 | テスト | 3時間 | 6.3.7 |
| 6.3.10 | ドキュメント作成 | ドキュメント | 1時間 | 6.3.9 |

**合計**: 29時間

### 5.2 実装順序

**推奨順序**: Phase 6.1 → Phase 6.2 → Phase 6.3

**理由**:
1. **Phase 6.1が最もシンプル**: 他機能への依存が少なく、基盤となる
2. **Phase 6.2は単独実装可能**: Phase 6.1の成果を活用しつつ、独立した機能
3. **Phase 6.3が最も複雑**: Transport層の拡張が必要で、技術的リスクが高い

**並列実装の可能性**:
- Phase 6.1とPhase 6.2は並列実装可能（依存関係が少ない）
- Phase 6.3は単独で実装を推奨（複雑度が高く、集中が必要）

### 5.3 工数見積もり

#### 5.3.1 詳細工数（標準値）

| フェーズ | 開発 | テスト | ドキュメント | 合計 |
|---------|------|-------|------------|------|
| Phase 6.1 | 9時間 | 3時間 | 1時間 | 13時間 |
| Phase 6.2 | 9時間 | 3時間 | 1時間 | 13時間 |
| Phase 6.3 | 21時間 | 7時間 | 1時間 | 29時間 |
| **合計** | **39時間** | **13時間** | **3時間** | **55時間** |

#### 5.3.2 工数見積もり（三点見積もり）

| フェーズ | 楽観値 | 標準値 | 悲観値 | 期待値 |
|---------|-------|-------|-------|-------|
| Phase 6.1 | 10時間 | 13時間 | 16時間 | 13時間 |
| Phase 6.2 | 10時間 | 13時間 | 16時間 | 13時間 |
| Phase 6.3 | 20時間 | 29時間 | 38時間 | 29時間 |
| **合計** | **40時間** | **55時間** | **70時間** | **55時間** |

**期待値の計算**: (楽観値 + 4 × 標準値 + 悲観値) / 6

**最終推奨工数**: 50-60時間（バッファ含む）

**注**: FUTURE_EXTENSION_PLAN.mdの見積もり（29-42時間）より多い理由:
- 詳細設計により、実装の複雑度が明確化
- テストとドキュメントの工数を明示的に含めた
- LoggerBackend拡張など、当初想定していなかった作業を追加

---

## 6. テスト戦略

### 6.1 単体テスト

#### 6.1.1 Phase 6.1: サーバー設定検査

**テスト対象**: `ServerInfoService`

**テストケース**:

```rust
// tests/unit/server_info_service_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_server_info_success() {
        // Mock ClientManager with test data
        let client_manager = create_mock_client_manager();
        let service = ServerInfoService::new(client_manager);

        let result = service.get_server_info("test-server").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.server_name, "test-server");
        assert_eq!(response.protocol_version, "2024-11-05");
        assert!(response.capabilities.tools);
    }

    #[tokio::test]
    async fn test_get_server_info_not_connected() {
        // Mock ClientManager that fails to connect
        let client_manager = create_failing_mock_client_manager();
        let service = ServerInfoService::new(client_manager);

        let result = service.get_server_info("nonexistent-server").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_status_detection() {
        // Test Connected status
        // Test Disconnected status
        // Test Error status
    }
}
```

**テスト項目**:
1. ✅ サーバー情報取得成功
2. ✅ 未接続サーバーへのエラーハンドリング
3. ✅ 接続状態の正確な検出
4. ✅ ServerCapabilitiesの正確な抽出
5. ✅ プロトコルバージョンの正確な取得

#### 6.1.2 Phase 6.2: ヘルスチェック

**テスト対象**: `HealthChecker`

**テストケース**:

```rust
// tests/unit/health_checker_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_success() {
        let client_manager = create_mock_client_manager();
        let checker = HealthChecker::new(client_manager);

        let result = checker.check_health("test-server").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, HealthStatus::Healthy);
        assert!(response.response_time_ms < 500);
    }

    #[tokio::test]
    async fn test_health_status_degraded() {
        // Mock slow response (500-2000ms)
        let result = checker.check_health("slow-server").await;
        assert_eq!(result.unwrap().status, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_status_unhealthy() {
        // Mock very slow response (>2000ms) or errors
        let result = checker.check_health("unhealthy-server").await;
        assert_eq!(result.unwrap().status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_error_rate_calculation() {
        // Perform multiple checks with some failures
        // Verify error_rate is calculated correctly
    }

    #[tokio::test]
    async fn test_history_circular_buffer() {
        // Perform 150 checks (max_history = 100)
        // Verify only last 100 are kept
    }
}
```

**テスト項目**:
1. ✅ ヘルスチェック成功（Healthy）
2. ✅ 低下状態の検出（Degraded）
3. ✅ 不健全状態の検出（Unhealthy）
4. ✅ エラー率の正確な計算
5. ✅ 履歴の循環バッファ動作
6. ✅ レスポンスタイム測定の精度

#### 6.1.3 Phase 6.3: Logging検査

**テスト対象**: `LoggingInspector`

**テストケース**:

```rust
// tests/unit/logging_inspector_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_log_entry() {
        let logger_backend = create_mock_logger_backend();
        let inspector = LoggingInspector::new(logger_backend);

        let entry = create_test_log_entry();
        let result = inspector.add_log_entry(entry);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_logging_messages_with_level_filter() {
        // Add logs with different levels
        // Query with level filter
        // Verify only matching logs are returned
    }

    #[tokio::test]
    async fn test_get_logging_messages_with_time_filter() {
        // Add logs with different timestamps
        // Query with 'since' filter
        // Verify only logs after specified time are returned
    }

    #[tokio::test]
    async fn test_log_level_conversion() {
        // Test conversion from rmcp::model::LoggingLevel to LogLevel
    }
}
```

**テスト項目**:
1. ✅ ログエントリの追加
2. ✅ レベルフィルタリング
3. ✅ 時刻フィルタリング
4. ✅ 件数制限
5. ✅ ログレベル変換の正確性
6. ✅ 複数サーバーのログ分離

### 6.2 統合テスト

#### 6.2.1 Phase 6統合テストシナリオ

**テストファイル**: `tests/phase6_integration_test.rs`

**テストシナリオ**:

```rust
// tests/phase6_integration_test.rs

#[tokio::test]
async fn test_server_inspect_end_to_end() {
    // 1. Start mock MCP server
    let mock_server = start_mock_server().await;

    // 2. Create InspectorService with config pointing to mock server
    let config = create_test_config();
    let service = InspectorService::new(config).unwrap();

    // 3. Call server_inspect
    let response = service.server_inspect("mock-server").await;

    // 4. Verify response
    assert!(response.is_ok());
    let result = response.unwrap();
    assert_eq!(result.server_name, "mock-server");
    assert!(result.capabilities.tools);

    // 5. Stop mock server
    mock_server.stop().await;
}

#[tokio::test]
async fn test_health_check_end_to_end() {
    // Similar structure to above
    // Test ping response time measurement
}

#[tokio::test]
async fn test_logging_messages_end_to_end() {
    // 1. Start mock MCP server that sends logging notifications
    // 2. Create InspectorService
    // 3. Wait for logging notifications to be received
    // 4. Query logging_messages
    // 5. Verify messages are stored and retrieved correctly
}

#[tokio::test]
async fn test_multiple_servers_concurrent() {
    // Test inspecting multiple servers concurrently
    // Verify no interference between servers
}

#[tokio::test]
async fn test_error_handling_server_down() {
    // Test behavior when target server is down
    // Verify appropriate error messages
}
```

**テスト項目**:
1. ✅ server_inspectのエンドツーエンド動作
2. ✅ health_checkのエンドツーエンド動作
3. ✅ logging_messagesのエンドツーエンド動作
4. ✅ 複数サーバーの同時検査
5. ✅ サーバーダウン時のエラーハンドリング
6. ✅ ログ通知の受信と保存
7. ✅ キャッシュの動作（該当する場合）
8. ✅ 永続化の動作確認

#### 6.2.2 モックサーバーの実装

Phase 4で実装済みの`mock_sampling_server`を拡張:

```rust
// tests/mock_server/mod.rs

pub struct MockServer {
    // ... existing fields ...

    /// Send logging notification to client
    pub async fn send_logging_notification(&self, level: LoggingLevel, message: &str) {
        // Implementation
    }

    /// Configure server capabilities
    pub fn with_capabilities(&mut self, capabilities: ServerCapabilities) {
        // Implementation
    }
}
```

### 6.3 パフォーマンステスト

#### 6.3.1 レスポンスタイムテスト

```rust
#[tokio::test]
async fn test_server_inspect_performance() {
    let service = create_test_service().await;
    let start = Instant::now();

    let _ = service.server_inspect("test-server").await;

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100, "server_inspect took {:?}", elapsed);
}

#[tokio::test]
async fn test_health_check_performance() {
    let service = create_test_service().await;
    let start = Instant::now();

    let _ = service.health_check("test-server").await;

    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "health_check took {:?}", elapsed);
}
```

#### 6.3.2 負荷テスト

```rust
#[tokio::test]
async fn test_concurrent_health_checks() {
    let service = Arc::new(create_test_service().await);
    let mut tasks = Vec::new();

    // Spawn 10 concurrent health checks
    for i in 0..10 {
        let service_clone = Arc::clone(&service);
        tasks.push(tokio::spawn(async move {
            service_clone.health_check(&format!("server-{}", i)).await
        }));
    }

    // Wait for all tasks
    for task in tasks {
        assert!(task.await.is_ok());
    }
}
```

#### 6.3.3 メモリ使用量テスト

```rust
#[tokio::test]
async fn test_logging_memory_usage() {
    let service = create_test_service().await;

    // Add 10,000 log entries
    for i in 0..10_000 {
        let entry = create_test_log_entry(i);
        service.logging_inspector.add_log_entry(entry).unwrap();
    }

    // Verify memory usage is within acceptable range
    // (This is a placeholder - actual implementation would measure memory)
}
```

### 6.4 テスト実行計画

#### 6.4.1 テスト環境

**必要な環境**:
- Rust toolchain (stable)
- テスト用モックサーバー
- 一時ファイルシステム（tempfile crateを使用）

#### 6.4.2 テスト実行コマンド

```bash
# 全テスト実行
cargo test

# Phase 6の単体テストのみ
cargo test --lib server_info_service
cargo test --lib health_checker
cargo test --lib logging_inspector

# Phase 6の統合テストのみ
cargo test --test phase6_integration_test

# カバレッジ測定（tarpaulin使用）
cargo tarpaulin --out Html --output-dir coverage
```

#### 6.4.3 合格基準

| メトリクス | 目標値 |
|----------|-------|
| **テストカバレッジ** | 80%以上 |
| **単体テスト合格率** | 100% |
| **統合テスト合格率** | 100% |
| **パフォーマンステスト** | 全項目が目標値以内 |
| **メモリリーク** | なし |

---

## 7. リスク分析と緩和策

### 7.1 技術リスク

#### リスク7.1.1: rmcp APIの制約

**影響度**: 高
**発生確率**: 中

**リスク詳細**:
- rmcp 0.8.5で想定しているAPIが実際には存在しない、または異なる実装になっている可能性
- 特に`InitializeResult`の取得方法、`ping`メソッドの呼び出し方が不明確

**緩和策**:
1. **事前調査の徹底**: 実装前にrmcpのドキュメントとサンプルコードを詳細に確認
2. **プロトタイプ実装**: 小規模なプロトタイプで動作確認
3. **代替案の準備**: APIが想定と異なる場合の代替実装パターンを用意
4. **rmcpコミュニティとの連携**: 不明点はGitHub Issueで質問

**コンティンジェンシープラン**:
- `InitializeResult`が取得できない場合: 接続時にキャッシュする独自実装
- `ping`メソッドがない場合: `tools_list`など軽量なメソッドで代替

#### リスク7.1.2: ログ通知の受信タイミング

**影響度**: 中
**発生確率**: 中

**リスク詳細**:
- `notifications/message`（ログ通知）がいつ、どのように送信されるか不明
- MonitoringTransportでの受信処理が正しく動作しない可能性

**緩和策**:
1. **モックサーバーでの検証**: 制御可能なモックサーバーで通知送信をテスト
2. **段階的実装**: まず受信処理を実装し、次に保存処理を追加
3. **ロギングの強化**: 受信したメッセージの詳細ログを出力して動作確認

**コンティンジェンシープラン**:
- ログ通知が受信できない場合: Phase 6.3の機能を「将来実装予定」として延期

#### リスク7.1.3: LoggerBackend拡張の複雑度

**影響度**: 中
**発生確率**: 低

**リスク詳細**:
- 既存のLoggerBackendを拡張する際、互換性を損なう可能性
- PersistentLoggerとMemoryLoggerの両方に変更が必要で、工数増加のリスク

**緩和策**:
1. **インターフェース設計の慎重化**: 既存メソッドに影響を与えない設計
2. **テストの充実**: 既存機能のリグレッションテストを実施
3. **段階的実装**: まずMemoryLoggerで実装・テスト後、PersistentLoggerに展開

**コンティンジェンシープラン**:
- 拡張が困難な場合: 別のストレージ（`LogMessageStorage`）を新規作成

### 7.2 スケジュールリスク

#### リスク7.2.1: 工数見積もりの誤差

**影響度**: 中
**発生確率**: 中

**リスク詳細**:
- 実装中に想定外の技術的困難が発生し、工数が超過する可能性
- 特にPhase 6.3（Logging検査）は複雑度が高く、工数超過リスクが大きい

**緩和策**:
1. **バッファの確保**: 見積もりに20%のバッファを追加
2. **マイルストーンの設定**: 各サブフェーズで進捗を確認
3. **早期の問題検出**: デイリーでの進捗確認と課題の早期共有

**コンティンジェンシープラン**:
- 工数超過が明確になった時点で、Phase 6.3の一部機能（ログレベル設定など）を次フェーズに延期

#### リスク7.2.2: 依存タスクの遅延

**影響度**: 低
**発生確率**: 低

**リスク詳細**:
- Phase 6.1の遅延がPhase 6.2、6.3に影響する可能性
- ただし、各フェーズの依存関係は比較的独立しているため、影響は限定的

**緩和策**:
1. **並列実装の検討**: Phase 6.1と6.2は並列実装可能
2. **優先順位の明確化**: Phase 6.1を最優先で完了させる

### 7.3 品質リスク

#### リスク7.3.1: テストカバレッジ不足

**影響度**: 中
**発生確率**: 低

**リスク詳細**:
- 時間的制約により、十分なテストが実施できない可能性
- エッジケースのテストが漏れる可能性

**緩和策**:
1. **テスト駆動開発**: 実装前にテストケースを設計
2. **カバレッジツールの活用**: `cargo tarpaulin`でカバレッジを可視化
3. **クリティカルパスの優先**: エラーハンドリングなど重要な部分を優先的にテスト

**コンティンジェンシープラン**:
- カバレッジが80%未満の場合: リリース前に追加のテスト作成期間を確保

#### リスク7.3.2: パフォーマンス劣化

**影響度**: 低
**発生確率**: 低

**リスク詳細**:
- ログ通知の処理などで、既存機能のパフォーマンスが低下する可能性

**緩和策**:
1. **パフォーマンステストの実施**: 各フェーズでベンチマークを測定
2. **非同期処理の活用**: ブロッキング処理を最小化
3. **プロファイリング**: 必要に応じて`cargo flamegraph`でボトルネック特定

**コンティンジェンシープラン**:
- パフォーマンス劣化が検出された場合: 最適化作業を追加（工数+5-10時間）

### 7.4 運用リスク

#### リスク7.4.1: 対象サーバーの多様性

**影響度**: 中
**発生確率**: 中

**リスク詳細**:
- 様々なMCPサーバー実装との互換性確保が困難
- 一部のサーバーでは期待通りに動作しない可能性

**緩和策**:
1. **複数サーバーでのテスト**: 公式サンプルサーバーなど複数で動作確認
2. **エラーハンドリングの充実**: 予期しない動作にも対応できる設計
3. **ドキュメント化**: 既知の制限事項をREADMEに明記

**コンティンジェンシープラン**:
- 特定サーバーで問題が発生した場合: トラブルシューティングガイドを作成

---

## 8. 完了基準

### 8.1 機能完了基準

#### 8.1.1 Phase 6.1: サーバー設定検査

- [ ] `server_inspect`ツールが正常に動作する
- [ ] サーバー名、バージョン、プロトコルバージョンが正確に取得できる
- [ ] ServerCapabilities（tools, resources, prompts, logging等）が正確に表示される
- [ ] 接続状態（Connected/Disconnected）が正確に判定される
- [ ] エラー時に適切なエラーメッセージが返される

#### 8.1.2 Phase 6.2: ヘルスチェック

- [ ] `health_check`ツールが正常に動作する
- [ ] pingによる疎通確認が成功する
- [ ] レスポンスタイムが正確に測定される（ミリ秒単位）
- [ ] エラー率が正確に算出される
- [ ] ヘルスステータス（Healthy/Degraded/Unhealthy）が適切に判定される
- [ ] ヘルス履歴が正しく保存される

#### 8.1.3 Phase 6.3: Logging検査

- [ ] `logging_messages`ツールが正常に動作する
- [ ] ログ通知が正しく受信される
- [ ] ログエントリが正確に保存される（タイムスタンプ、レベル、メッセージ）
- [ ] レベルフィルタリングが正常に機能する
- [ ] 時刻フィルタリングが正常に機能する
- [ ] 件数制限が正常に機能する
- [ ] 複数サーバーのログが分離して保存される

### 8.2 品質基準

#### 8.2.1 テスト合格基準

- [ ] **単体テスト**: 全テストが合格（Phase 6.1: 5件以上、6.2: 6件以上、6.3: 6件以上）
- [ ] **統合テスト**: 全テストが合格（8件以上）
- [ ] **テストカバレッジ**: 80%以上
- [ ] **パフォーマンステスト**: 全項目が目標値以内
  - server_inspect: < 100ms
  - health_check: < 500ms
  - logging_messages: < 50ms

#### 8.2.2 コード品質基準

- [ ] **cargo check**: エラーなし
- [ ] **cargo clippy**: 警告なし（または正当化されたwarning）
- [ ] **cargo fmt**: フォーマット準拠
- [ ] **rustdoc**: 全公開APIにドキュメントコメント
- [ ] **エラーハンドリング**: 全エラーケースに適切な処理

#### 8.2.3 ドキュメント基準

- [ ] **README.md**: Phase 6機能の説明追加
- [ ] **CHANGELOG.md**: Phase 6の変更履歴記載
- [ ] **APIドキュメント**: rustdocで生成されたドキュメントが完全
- [ ] **使用例**: 各ツールの使用例をREADMEまたは別ドキュメントに記載
- [ ] **トラブルシューティング**: 既知の問題と解決策を記載

### 8.3 リリース基準

#### 8.3.1 必須条件

- [ ] 全機能完了基準を満たす
- [ ] 全品質基準を満たす
- [ ] 統合テストが全て合格
- [ ] 既存機能（Phase 1-5）への影響がない（リグレッションテスト合格）

#### 8.3.2 推奨条件

- [ ] 複数の公式MCPサーバーで動作確認
- [ ] パフォーマンスベンチマークの実施と記録
- [ ] セキュリティレビューの実施
- [ ] ユーザーフィードバックの収集（可能であれば）

---

## 9. 成果物リスト

### 9.1 実装ファイル

#### 9.1.1 新規ファイル

| ファイルパス | 種類 | 行数（推定） | 説明 |
|-------------|------|------------|------|
| `src/models/server_info.rs` | モデル | 80-100 | サーバー情報のデータ構造 |
| `src/models/health.rs` | モデル | 100-120 | ヘルスチェックのデータ構造 |
| `src/models/logging_inspection.rs` | モデル | 120-150 | ログ検査のデータ構造 |
| `src/services/server_info_service.rs` | サービス | 100-120 | サーバー情報サービス |
| `src/services/health_checker.rs` | サービス | 150-180 | ヘルスチェックサービス |
| `src/services/logging_inspector.rs` | サービス | 150-200 | ログ検査サービス |

**合計新規ファイル**: 6ファイル、約700-870行

#### 9.1.2 修正ファイル

| ファイルパス | 修正内容 | 追加行数（推定） |
|-------------|---------|---------------|
| `src/client/stdio_client.rs` | `get_init_result`, `is_connected`, `ping`メソッド追加 | +45 |
| `src/client/monitoring_transport.rs` | ログ通知受信処理追加 | +80 |
| `src/services/logger_backend.rs` | ログメッセージ用メソッド追加 | +30 |
| `src/services/persistent_logger.rs` | ログメッセージ保存実装 | +100 |
| `src/services/memory_logger.rs` | ログメッセージ保存実装 | +80 |
| `src/services/inspector.rs` | 新規メソッド3つ追加、フィールド3つ追加 | +55 |
| `src/server/mod.rs` | 新規ツール3つ定義 | +120 |

**合計修正**: 7ファイル、約510行追加

#### 9.1.3 総実装規模

- **新規ファイル**: 6ファイル、700-870行
- **修正ファイル**: 7ファイル、約510行追加
- **総追加行数**: 約1,210-1,380行

### 9.2 テストファイル

#### 9.2.1 単体テストファイル

| ファイルパス | 内容 | 行数（推定） |
|-------------|------|------------|
| `tests/unit/server_info_service_test.rs` | ServerInfoServiceのテスト | 150-200 |
| `tests/unit/health_checker_test.rs` | HealthCheckerのテスト | 200-250 |
| `tests/unit/logging_inspector_test.rs` | LoggingInspectorのテスト | 200-250 |

**合計**: 3ファイル、約550-700行

#### 9.2.2 統合テストファイル

| ファイルパス | 内容 | 行数（推定） |
|-------------|------|------------|
| `tests/phase6_integration_test.rs` | Phase 6統合テスト | 400-500 |

**合計**: 1ファイル、約400-500行

#### 9.2.3 テスト総規模

- **単体テスト**: 3ファイル、550-700行
- **統合テスト**: 1ファイル、400-500行
- **総テスト行数**: 約950-1,200行

### 9.3 ドキュメントファイル

| ファイルパス | 内容 | 行数（推定） |
|-------------|------|------------|
| `docs/PHASE6_DETAILED_PLAN.md` | 本ドキュメント | 3,000+ |
| `docs/PHASE6_COMPLETION_REPORT.md` | Phase 6完了レポート（実装後） | 500-800 |
| `README.md` | Phase 6機能の説明追加 | +100-150 |
| `CHANGELOG.md` | Phase 6の変更履歴 | +50-100 |

**合計**: 4ファイル、約3,650-4,050行

### 9.4 成果物サマリー

| カテゴリ | ファイル数 | 行数（推定） |
|---------|----------|------------|
| **実装ファイル** | 13 | 1,210-1,380 |
| **テストファイル** | 4 | 950-1,200 |
| **ドキュメント** | 4 | 3,650-4,050 |
| **合計** | 21 | 5,810-6,630 |

---

## 10. スケジュール

### 10.1 フェーズ別タイムライン

#### 10.1.1 全体スケジュール（週単位）

```
Week 1: Phase 6.1 (サーバー設定検査)
├─ Day 1-2: データ構造定義、ServerInfoService実装
├─ Day 3: StdioClient拡張、統合
└─ Day 4-5: テスト、ドキュメント

Week 2: Phase 6.2 (ヘルスチェック)
├─ Day 1-2: データ構造定義、HealthChecker実装
├─ Day 3: StdioClient ping実装、統合
└─ Day 4-5: テスト、ドキュメント

Week 3-4: Phase 6.3 (Logging検査)
├─ Week 3 Day 1-3: データ構造、LoggingInspector、MonitoringTransport拡張
├─ Week 3 Day 4-5: LoggerBackend拡張
├─ Week 4 Day 1-2: PersistentLogger/MemoryLogger拡張、統合
└─ Week 4 Day 3-5: テスト、ドキュメント

Week 5: 総合テスト・リリース準備
├─ Day 1-2: 統合テスト、パフォーマンステスト
├─ Day 3: ドキュメント整備
├─ Day 4: リグレッションテスト
└─ Day 5: リリース準備、バッファ
```

#### 10.1.2 詳細スケジュール（日単位）

**Phase 6.1: サーバー設定検査（Week 1）**

| 日 | タスク | 工数 | 完了基準 |
|----|-------|------|---------|
| Day 1 | データ構造定義（server_info.rs） | 2時間 | ✅ コンパイル成功 |
| Day 1-2 | ServerInfoService実装 | 3時間 | ✅ 基本機能動作 |
| Day 2 | StdioClient拡張 | 2時間 | ✅ get_init_result, is_connected動作 |
| Day 3 | InspectorServiceへの統合 | 1時間 | ✅ server_inspect呼び出し可能 |
| Day 3 | ツール定義 | 1時間 | ✅ MCPツールとして公開 |
| Day 4 | 単体テスト | 2時間 | ✅ 全テスト合格 |
| Day 4 | 統合テスト | 1時間 | ✅ E2Eテスト合格 |
| Day 5 | ドキュメント作成 | 1時間 | ✅ README更新 |

**Phase 6.2: ヘルスチェック（Week 2）**

| 日 | タスク | 工数 | 完了基準 |
|----|-------|------|---------|
| Day 1 | データ構造定義（health.rs） | 2時間 | ✅ コンパイル成功 |
| Day 1-2 | HealthChecker実装 | 4時間 | ✅ ヘルスチェック動作 |
| Day 2 | StdioClient ping実装 | 1時間 | ✅ ping動作 |
| Day 3 | InspectorServiceへの統合 | 1時間 | ✅ health_check呼び出し可能 |
| Day 3 | ツール定義 | 1時間 | ✅ MCPツールとして公開 |
| Day 4 | 単体テスト | 2時間 | ✅ 全テスト合格 |
| Day 4 | 統合テスト | 1時間 | ✅ E2Eテスト合格 |
| Day 5 | ドキュメント作成 | 1時間 | ✅ README更新 |

**Phase 6.3: Logging検査（Week 3-4）**

| 日 | タスク | 工数 | 完了基準 |
|----|-------|------|---------|
| W3 D1 | データ構造定義（logging_inspection.rs） | 3時間 | ✅ コンパイル成功 |
| W3 D1-2 | LoggingInspector実装 | 4時間 | ✅ 基本機能動作 |
| W3 D2-3 | MonitoringTransport拡張 | 4時間 | ✅ ログ通知受信 |
| W3 D4 | LoggerBackend拡張 | 3時間 | ✅ 新メソッド動作 |
| W3 D4-5 | PersistentLogger拡張 | 3時間 | ✅ ログメッセージ保存 |
| W4 D1 | MemoryLogger拡張 | 2時間 | ✅ ログメッセージ保存 |
| W4 D1 | InspectorServiceへの統合 | 1時間 | ✅ logging_messages呼び出し可能 |
| W4 D2 | ツール定義 | 1時間 | ✅ MCPツールとして公開 |
| W4 D2-3 | 単体テスト | 4時間 | ✅ 全テスト合格 |
| W4 D3-4 | 統合テスト | 3時間 | ✅ E2Eテスト合格 |
| W4 D5 | ドキュメント作成 | 1時間 | ✅ README更新 |

**Week 5: 総合テスト・リリース準備**

| 日 | タスク | 工数 | 完了基準 |
|----|-------|------|---------|
| Day 1 | 統合テスト（全体） | 4時間 | ✅ 全シナリオ合格 |
| Day 2 | パフォーマンステスト | 4時間 | ✅ 全項目が目標値以内 |
| Day 3 | ドキュメント整備 | 4時間 | ✅ 全ドキュメント完成 |
| Day 4 | リグレッションテスト | 4時間 | ✅ 既存機能に影響なし |
| Day 5 | リリース準備、バッファ | 4時間 | ✅ リリース可能状態 |

### 10.2 マイルストーン

| マイルストーン | 日付 | 完了基準 |
|--------------|------|---------|
| **M1: Phase 6.1完了** | Week 1終了時 | server_inspectツールが動作、全テスト合格 |
| **M2: Phase 6.2完了** | Week 2終了時 | health_checkツールが動作、全テスト合格 |
| **M3: Phase 6.3完了** | Week 4終了時 | logging_messagesツールが動作、全テスト合格 |
| **M4: Phase 6全体完了** | Week 5終了時 | 全機能完了、リリース準備完了 |

### 10.3 クリティカルパス

**クリティカルパス**: Phase 6.3（Logging検査）

**理由**:
- 最も工数が大きい（29時間）
- 技術的複雑度が最も高い
- MonitoringTransport、LoggerBackendなど複数モジュールへの影響が大きい

**クリティカルパス管理**:
1. Phase 6.3に十分な時間を確保（2週間）
2. 技術的課題の早期検出と解決
3. 必要に応じて並列タスクの活用（例: MemoryLoggerとPersistentLoggerの並列実装）
4. バッファ時間の確保（Week 5）

### 10.4 リソース配分

**想定リソース**: 1名のrust-developer（フルタイム）

**日あたりの稼働時間**: 6-8時間（実装・テスト）

**週あたりの工数**: 30-40時間

**総期間**: 5週間（4週間実装 + 1週間総合テスト）

**バッファ**: Week 5（総合テスト週）に含まれる

### 10.5 進捗管理

#### 10.5.1 デイリー進捗確認

- 毎日の終わりに進捗を確認
- 遅延が発生した場合は即座に対策を検討
- ブロッカーがあれば早期にエスカレーション

#### 10.5.2 週次レビュー

- 各週の終わりにマイルストーン達成を確認
- 次週の計画を調整
- リスクの再評価

#### 10.5.3 進捗報告

**PMへの報告内容**:
- 完了したタスク
- 進行中のタスク
- ブロッカーと課題
- 次週の予定

---

## 付録

### A. 用語集

| 用語 | 説明 |
|------|------|
| **MCP** | Model Context Protocol - AIエージェントとツール間の通信プロトコル |
| **rmcp** | Rust MCP SDK - MCPプロトコルのRust実装 |
| **InitializeResult** | MCP接続時のハンドシェイクで返されるサーバー情報 |
| **ServerCapabilities** | サーバーがサポートする機能のリスト |
| **Ping** | サーバーの疎通確認メソッド |
| **Logging Notification** | サーバーからクライアントへのログメッセージ通知 |
| **Health Status** | サーバーの健全性ステータス（Healthy/Degraded/Unhealthy） |
| **LoggerBackend** | ログ保存の抽象化インターフェース |

### B. 参考リソース

#### B.1 公式ドキュメント

- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp/0.8.5/)
- [rmcp GitHub Repository](https://github.com/modelcontextprotocol/rmcp)

#### B.2 技術リソース

- [tokio Documentation](https://tokio.rs/)
- [serde Documentation](https://serde.rs/)
- [anyhow Documentation](https://docs.rs/anyhow/)
- [chrono Documentation](https://docs.rs/chrono/)

#### B.3 プロジェクト内ドキュメント

- `docs/FUTURE_EXTENSION_PLAN.md` - Phase 4以降の計画
- `docs/PHASE5_COMPLETION_REPORT.md` - Phase 5完了レポート
- `docs/PHASE4_COMPLETION_REPORT.md` - Phase 4完了レポート
- `README.md` - プロジェクトREADME

### C. 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|----------|---------|-------|
| 2025-11-15 | 1.0 | 初版作成 | Solution Architect |

---

## まとめ

本Phase 6詳細計画書は、MCP Inspector MCP Serverの高度な検査機能実装のための包括的なロードマップを提供します。

**重要なポイント**:

1. **段階的アプローチ**: Phase 6.1（サーバー設定検査）→ 6.2（ヘルスチェック）→ 6.3（Logging検査）の順で実装
2. **技術的実現可能性**: rmcp 0.8.5の制約を考慮し、実装可能な範囲を明確化
3. **品質重視**: 包括的なテスト戦略とドキュメント整備により、高品質な実装を保証
4. **リスク管理**: 技術リスクと緩和策を明示し、コンティンジェンシープランを用意

**期待される成果**:

- 3つの新規ツール（`server_inspect`, `health_check`, `logging_messages`）
- 約1,200-1,400行の新規実装コード
- 約950-1,200行のテストコード
- 80%以上のテストカバレッジ
- 包括的なドキュメント

この計画書に従って実装を進めることで、MCP Inspectorは対象サーバーの詳細な状態監視とデバッグ支援が可能な、より強力なツールへと進化します。

---

**発行者**: Solution Architect
**承認者**: （Project Manager承認待ち）
**発行日**: 2025-11-15
**バージョン**: 1.0

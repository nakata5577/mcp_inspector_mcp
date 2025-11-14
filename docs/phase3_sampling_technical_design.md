# Phase 3: Sampling機能 技術設計書

## 1. エグゼクティブサマリー

### 設計結論
**推奨アプローチ: オプションC（ロギング専用）+ 将来的なオプションA拡張の余地**

Sampling機能はMCPプロトコルにおいて**サーバー→クライアント方向の通信**を実現する特殊な機能です。本プロジェクトのアーキテクチャおよびrmcp 0.8.5の制約を考慮した結果、Phase 3では**ロギング専用の実装**を推奨します。

### 主要な技術的制約
1. rmcp 0.8.5の`ServerHandler`トレイトには**Sampling関連メソッドが存在しない**
2. mcp_inspector_mcpは**MCPサーバー**として動作するため、Samplingリクエストを受信する標準的な手段がない
3. Samplingは対象サーバーから発信されるが、それを転送する仕組みがプロトコル上定義されていない

---

## 2. Sampling機能の技術的分析

### 2.1 MCPプロトコルにおけるSampling

#### 通信方向の違い
```
通常のツール呼び出し:
AIエージェント (Client) → mcp_inspector_mcp (Server) → 対象サーバー (Server)

Samplingの場合:
対象サーバー (Server) → ??? → AIエージェント (Client) → LLM
```

#### rmcp APIのSampling関連型
```rust
// リクエスト
pub struct CreateMessageRequestParam {
    pub messages: Vec<SamplingMessage>,
    pub model_preferences: Option<ModelPreferences>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: u32,
    // ... その他のパラメータ
}

// レスポンス
pub struct CreateMessageResult {
    pub model: String,
    pub stop_reason: Option<String>,
    pub message: SamplingMessage,
}

// メッセージ
pub struct SamplingMessage {
    pub role: Role,      // User | Assistant
    pub content: Content,
}
```

### 2.2 rmcp 0.8.5 APIの制約

#### ServerHandlerトレイトの分析
```rust
pub trait ServerHandler {
    fn initialize(...) -> InitializeResult;
    fn list_tools(...) -> ListToolsResult;
    fn call_tool(...) -> CallToolResult;
    fn list_resources(...) -> ListResourcesResult;
    fn read_resource(...) -> ReadResourceResult;
    fn list_prompts(...) -> ListPromptsResult;
    fn get_prompt(...) -> GetPromptResult;
    // ... その他
    // ❌ create_messageやsampling関連のメソッドは存在しない
}
```

**重要な発見**: `ServerHandler`にはSamplingリクエストを受信・処理するメソッドが**一切定義されていない**。

#### クライアント側API
Samplingは本来、**MCPクライアント**が実装すべき機能です：
```rust
// クライアント側でLLMにリクエストを送る
service.create_message(CreateMessageRequestParam { ... }).await
```

### 2.3 アーキテクチャ上の課題

#### 現在のアーキテクチャ
```
┌─────────────────┐
│  AIエージェント  │ (MCPクライアント)
└────────┬────────┘
         │ tools_list, tools_call, etc.
         ↓
┌─────────────────────────┐
│ mcp_inspector_mcp       │ (MCPサーバー)
│ - InspectorServer       │
│ - InspectorService      │
│ - ClientManager         │
└────────┬────────────────┘
         │ list_tools, call_tool, etc.
         ↓
┌─────────────────────────┐
│ 対象MCPサーバー          │
│ (fundamental_analysis)  │
└─────────────────────────┘
```

#### Sampling時の問題
1. **対象サーバー**がSamplingリクエストを発行しても、それを受け取る仕組みがない
2. mcp_inspector_mcpは**サーバー**として動作するため、クライアント側のSampling APIを使えない
3. rmcpのTransportレイヤーで双方向通信は可能だが、`ServerHandler`でハンドリングできない

---

## 3. 実装オプションの評価

### オプションA: Samplingリクエストの転送（理想形）

#### 概要
対象サーバーからのSamplingリクエストを受信し、AIエージェントに転送して、レスポンスを返却する。

#### アーキテクチャ
```
対象サーバー
  ↓ sampling/createMessage
mcp_inspector_mcp (中継)
  ↓ カスタム通知 or ツール呼び出し
AIエージェント
  ↓ LLM呼び出し
  ↓ レスポンス
mcp_inspector_mcp
  ↓ レスポンス返却
対象サーバー
```

#### 実装方法
1. **カスタム通知の利用**
   - 対象サーバーからのSamplingリクエストを検出
   - AIエージェントへカスタム通知として送信
   - 課題: 標準MCPプロトコル外の拡張が必要

2. **ツールとしての公開**
   - `sampling_request`というツールを提供
   - AIエージェントがツールを呼び出すことでLLMにアクセス
   - 課題: 非同期性の問題（サーバーが待機できない）

#### 技術的課題
- **rmcp APIの制約**: `ServerHandler`で受信できない
- **プロトコルの制約**: MCPプロトコルにSampling転送の仕様がない
- **非同期性**: 対象サーバーは同期的にレスポンスを待つが、AIエージェントへの転送は非同期
- **実装複雑度**: 高い（推定20-30時間）

#### 評価
- **実現可能性**: △（技術的には可能だが、標準外の拡張が必要）
- **メンテナンス性**: △（rmcpアップデートで影響を受ける可能性）
- **実用性**: ○（実現できれば最も有用）

### オプションB: テスト用モックレスポンス

#### 概要
対象サーバーからのSamplingリクエストを受信し、固定のモックレスポンスを返す。

#### 実装方法
```rust
// 疑似コード
async fn handle_sampling_request(req: CreateMessageRequestParam) -> CreateMessageResult {
    CreateMessageResult {
        model: "mock-model".to_string(),
        stop_reason: Some("endTurn".to_string()),
        message: SamplingMessage {
            role: Role::Assistant,
            content: Content::text("[Mock Response] This is a test response"),
        },
    }
}
```

#### 技術的課題
- **同じ課題**: `ServerHandler`で受信できない
- **実用性の欠如**: モックレスポンスでは実際の動作確認ができない

#### 評価
- **実現可能性**: △（オプションAと同じ技術的課題）
- **メンテナンス性**: ○（シンプル）
- **実用性**: ✗（テスト以外の用途がない）

### オプションC: ロギングのみ（推奨）

#### 概要
対象サーバーがSamplingリクエストを発行した場合、それを検出してログに記録する。実際のLLM呼び出しは行わない。

#### 実装方法
1. **Transport層での監視**
   - rmcpのTransport層で双方向通信を監視
   - `sampling/createMessage`メソッドを検出
   - リクエスト内容をログに出力

2. **新規ツール: sampling_log**
   ```rust
   #[tool(description = "サーバーが発行したSamplingリクエストのログを取得")]
   async fn sampling_log(&self, params: SamplingLogParams) -> Result<Json<SamplingLogResult>> {
       // 記録されたSamplingリクエストの履歴を返す
   }
   ```

#### データ構造
```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SamplingLogEntry {
    pub timestamp: String,
    pub server_name: String,
    pub request: CreateMessageRequestParam,
    pub status: SamplingStatus, // Detected | Ignored | Error
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SamplingLogResult {
    pub entries: Vec<SamplingLogEntry>,
    pub total_count: usize,
}
```

#### 技術的課題
- **Transport層の拡張**: 低レベルのメッセージ監視が必要
- **ログストレージ**: メモリまたはファイルベースのログ保存

#### 評価
- **実現可能性**: ○（Transport層で実装可能）
- **メンテナンス性**: ○（既存アーキテクチャへの影響が小さい）
- **実用性**: ○（デバッグ・監視目的で有用）

---

## 4. 推奨アプローチ

### Phase 3での実装: オプションC（ロギング専用）

#### 推奨理由
1. **技術的実現可能性が高い**: Transport層での監視は可能
2. **低リスク**: 既存機能への影響がない
3. **段階的な拡張が可能**: 将来的にオプションAに拡張できる
4. **実用性**: Samplingの使用状況を把握できる

#### 実装範囲
- Samplingリクエストの検出とロギング
- ログ取得用ツール（`sampling_log`）の提供
- 対象サーバーへのエラーレスポンス返却（"Not Implemented"）

---

## 5. データモデル設計

### 5.1 モデル定義

#### src/models/sampling.rs
```rust
use rmcp::model::{CreateMessageRequestParam, CreateMessageResult, SamplingMessage};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Samplingリクエストのログエントリ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SamplingLogEntry {
    /// タイムスタンプ (ISO 8601)
    pub timestamp: String,

    /// サーバー名
    pub server_name: String,

    /// リクエスト内容
    pub request: CreateMessageRequestParam,

    /// ステータス
    pub status: SamplingStatus,

    /// エラーメッセージ（存在する場合）
    pub error_message: Option<String>,
}

/// Samplingリクエストのステータス
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SamplingStatus {
    /// 検出済み
    Detected,

    /// 未実装のため無視
    NotImplemented,

    /// エラー発生
    Error,
}

/// sampling_logツールのパラメータ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SamplingLogParams {
    /// サーバー名でフィルタ（オプション）
    pub server: Option<String>,

    /// 最大件数（デフォルト: 100）
    pub limit: Option<usize>,
}

/// sampling_logツールのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SamplingLogResult {
    /// ログエントリのリスト
    pub entries: Vec<SamplingLogEntry>,

    /// 総件数
    pub total_count: usize,
}
```

### 5.2 既存モデルとの整合性

Phase 1/2で確立されたパターンに準拠：
- `JsonSchema`トレイトの実装
- serde の `Serialize`/`Deserialize`
- ドキュメントコメントの記載

---

## 6. アーキテクチャ設計

### 6.1 コンポーネント構成

```
┌─────────────────────────────────────────────┐
│           InspectorServer                   │
│  (MCPサーバー、ツール定義)                   │
│  - tools_list, tools_call                   │
│  - resources_list, resources_read           │
│  - prompts_list, prompts_get                │
│  - sampling_log (NEW)                       │
└────────────────┬────────────────────────────┘
                 │
┌────────────────┴────────────────────────────┐
│         InspectorService                    │
│  (ビジネスロジック)                          │
│  - inspect_tools, call_tool                 │
│  - inspect_resources, read_resource         │
│  - inspect_prompts, get_prompt              │
│  - get_sampling_logs (NEW)                  │
└────────────────┬────────────────────────────┘
                 │
┌────────────────┴────────────────────────────┐
│         ClientManager                       │
│  (対象サーバー接続管理)                      │
│  - get_or_create_client                     │
│  - list_clients                             │
└────────────────┬────────────────────────────┘
                 │
┌────────────────┴────────────────────────────┐
│         StdioClient (拡張)                  │
│  (Transport層でのメッセージ監視)             │
│  - send / receive のラッピング              │
│  - "sampling/createMessage" の検出          │
│  - SamplingLogger への通知 (NEW)            │
└────────────────┬────────────────────────────┘
                 │
┌────────────────┴────────────────────────────┐
│         SamplingLogger (NEW)                │
│  (ログ管理)                                  │
│  - log_sampling_request                     │
│  - get_logs                                 │
│  - clear_logs                               │
└─────────────────────────────────────────────┘
```

### 6.2 新規コンポーネント: SamplingLogger

#### src/services/sampling_logger.rs
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::sampling::{SamplingLogEntry, SamplingStatus};
use rmcp::model::CreateMessageRequestParam;

/// Samplingリクエストのロガー
#[derive(Clone)]
pub struct SamplingLogger {
    logs: Arc<RwLock<Vec<SamplingLogEntry>>>,
    max_entries: usize,
}

impl SamplingLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_entries,
        }
    }

    /// Samplingリクエストを記録
    pub async fn log_request(
        &self,
        server_name: String,
        request: CreateMessageRequestParam,
        status: SamplingStatus,
        error_message: Option<String>,
    ) {
        let mut logs = self.logs.write().await;

        let entry = SamplingLogEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            server_name,
            request,
            status,
            error_message,
        };

        logs.push(entry);

        // 最大件数を超えたら古いものを削除
        if logs.len() > self.max_entries {
            logs.drain(0..logs.len() - self.max_entries);
        }
    }

    /// ログを取得
    pub async fn get_logs(
        &self,
        server: Option<String>,
        limit: Option<usize>,
    ) -> Vec<SamplingLogEntry> {
        let logs = self.logs.read().await;

        let filtered: Vec<_> = logs
            .iter()
            .filter(|entry| {
                server.as_ref().map_or(true, |s| &entry.server_name == s)
            })
            .cloned()
            .collect();

        let limit = limit.unwrap_or(100).min(filtered.len());
        filtered.into_iter().rev().take(limit).collect()
    }

    /// ログをクリア
    pub async fn clear_logs(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
    }
}
```

### 6.3 StdioClientの拡張

#### 基本方針
Transport層でメッセージを監視し、`sampling/createMessage`を検出する。

#### 実装上の課題
rmcpのTransport層は抽象化されており、低レベルでのメッセージ監視は**困難**です。以下の代替案を検討：

**代替案1: カスタムTransportの実装**
- rmcpの`Transport`トレイトを実装
- メッセージの送受信をラップして監視
- 複雑度が高い（推定10-15時間）

**代替案2: 未実装のまま残す**
- Phase 3では`sampling_log`ツールのみ実装
- ログは常に空を返す
- 将来的なrmcpアップデートを待つ

**推奨: 代替案2**
- rmcp 0.8.5の制約により、Transport層の監視は技術的に困難
- 将来のrmcpバージョンでSampling対応が追加される可能性がある
- ツールのインターフェースのみ実装しておき、将来の拡張に備える

---

## 7. 実装手順（最小限のMVP）

### Phase 3.1: データモデル定義（2-3時間）
1. `src/models/sampling.rs` 作成
2. `SamplingLogEntry`, `SamplingStatus`, `SamplingLogParams`, `SamplingLogResult` 定義
3. `src/models/mod.rs` に追加

### Phase 3.2: SamplingLogger実装（2-3時間）
1. `src/services/sampling_logger.rs` 作成
2. ログ管理機能実装（メモリベース）
3. 単体テスト作成

### Phase 3.3: InspectorServiceへの統合（2-3時間）
1. `InspectorService`に`SamplingLogger`フィールド追加
2. `get_sampling_logs`メソッド実装
3. 初期化処理の更新

### Phase 3.4: MCPツール追加（2-3時間）
1. `InspectorServer`に`sampling_log`ツール追加
2. `#[tool]`マクロでの定義
3. パラメータバリデーション

### Phase 3.5: テストと検証（2-3時間）
1. ユニットテスト作成
2. MCP Inspector CLIでのテスト
3. ドキュメント更新

### Phase 3.6: Transport監視（将来の拡張）
**現時点では実装しない**
- rmcpのアップデート待ち
- または別のアプローチの検討

---

## 8. 技術的課題とリスク

### 8.1 主要な課題

#### 課題1: rmcp APIの制約
- **内容**: `ServerHandler`にSampling関連メソッドがない
- **影響度**: 高
- **対策**: ロギング専用の実装で回避（Phase 3）

#### 課題2: Transport層の監視
- **内容**: 低レベルのメッセージ監視が困難
- **影響度**: 中
- **対策**: 将来の拡張として残す

#### 課題3: プロトコル上の制約
- **内容**: MCPプロトコルにSampling転送の標準仕様がない
- **影響度**: 高
- **対策**: 現時点では対応不可

### 8.2 リスク評価

| リスク | 確率 | 影響 | 対策 |
|--------|------|------|------|
| rmcpアップデートでAPI変更 | 中 | 中 | 最小限の実装で影響を抑制 |
| Transport監視の実装困難 | 高 | 低 | Phase 3では非実装 |
| 実用性の不足 | 低 | 中 | ロギング機能だけでも有用 |

---

## 9. 見積もり

### Phase 3 MVP（ロギング専用）
| フェーズ | 作業内容 | 時間 |
|---------|---------|------|
| 3.1 | データモデル定義 | 2-3h |
| 3.2 | SamplingLogger実装 | 2-3h |
| 3.3 | InspectorService統合 | 2-3h |
| 3.4 | MCPツール追加 | 2-3h |
| 3.5 | テストと検証 | 2-3h |
| **合計** | | **10-15時間** |

### 将来の拡張（Transport監視）
| フェーズ | 作業内容 | 時間 |
|---------|---------|------|
| 3.6 | カスタムTransport実装 | 8-12h |
| 3.7 | メッセージ監視ロジック | 4-6h |
| 3.8 | エラーハンドリング | 2-3h |
| **合計** | | **14-21時間** |

---

## 10. 結論と次のステップ

### 10.1 Phase 3の実装方針

**採用アプローチ**: オプションC（ロギング専用）

**理由**:
1. rmcp 0.8.5の技術的制約により、完全なSampling転送は困難
2. ロギング機能だけでも、デバッグ・監視目的で有用
3. 将来的な拡張の余地を残せる
4. 低リスク・低工数で実装可能

### 10.2 実装する機能
- ✅ `SamplingLogEntry`等のデータモデル
- ✅ `SamplingLogger`サービス
- ✅ `sampling_log`ツール
- ❌ Transport層でのメッセージ監視（将来の拡張）

### 10.3 将来の拡張可能性
1. **rmcpアップデート待ち**: 将来のバージョンでSampling対応が追加される可能性
2. **カスタムTransport**: 技術的には可能だが、工数が大きい
3. **プロキシモード**: mcp_inspector_mcpをプロキシとして動作させる（大規模な改修）

### 10.4 次のステップ
1. 本設計書のレビュー
2. Phase 3.1（データモデル定義）の着手
3. 段階的な実装とテスト

---

## 付録A: rmcp API調査結果

### ServerHandlerトレイトのメソッド一覧
```rust
pub trait ServerHandler {
    fn ping(...);
    fn initialize(...);
    fn complete(...);
    fn set_level(...);
    fn get_prompt(...);
    fn list_prompts(...);
    fn list_resources(...);
    fn list_resource_templates(...);
    fn read_resource(...);
    fn subscribe(...);
    fn unsubscribe(...);
    fn call_tool(...);
    fn list_tools(...);
    fn on_cancelled(...);
    fn on_progress(...);
    fn on_initialized(...);
    fn on_roots_list_changed(...);
    fn get_info(...);
}
```

**結論**: Sampling関連のメソッドは一切存在しない。

### ServiceExtトレイト（クライアント側）
```rust
pub trait ServiceExt<R: ServiceRole> {
    fn serve(...);
    fn serve_with_ct(...);
    fn into_dyn(...);
}
```

クライアント側でサーバーを起動するための拡張トレイト。Samplingの直接的なメソッドは含まれない。

---

## 付録B: 参考資料

- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp crate documentation](https://docs.rs/rmcp/0.8.5/)
- mcp_inspector_mcp Phase 1/2 実装

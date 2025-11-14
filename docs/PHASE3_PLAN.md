# Phase 3 実装計画書: Sampling機能のロギング

## 1. 概要

### 1.1 Phase 3の目的

Phase 3では、MCP Inspector MCPサーバーに**Samplingリクエストのロギング機能**を追加し、対象MCPサーバーのSampling動作を監視・デバッグできるようにします。

- `sampling_logs` - Samplingリクエストのログ取得

**注意:** Phase 3では完全なSampling転送機能ではなく、**ロギング専用の実装**を行います。これはrmcp 0.8.5の技術的制約によるもので、詳細は技術設計書を参照してください。

### 1.2 Phase 1/2との関連性

**Phase 1（完了）:**
- Tools検査機能（tools_list, tools_call）

**Phase 2（完了）:**
- Resources検査機能（resources_list, resources_read）
- Prompts検査機能（prompts_list, prompts_get）

**Phase 3（本フェーズ）:**
- Samplingロギング機能（sampling_logs）

Phase 3はPhase 1/2の確立されたアーキテクチャパターンを踏襲し、新機能を追加します。

---

## 2. 新規追加機能仕様

### 2.1 sampling_logs

対象MCPサーバーからのSamplingリクエストのログを取得します。

#### 引数
| パラメータ | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `server` | string | ✅ | 対象のMCPサーバー名 |
| `limit` | integer | ❌ | 取得するログの最大件数（デフォルト: 100） |
| `status` | string | ❌ | フィルタするステータス（"all", "success", "failed"、デフォルト: "all"） |

#### 戻り値
```rust
SamplingLogsResponse {
    server: String,                     // サーバー名
    logs: Vec<SamplingLogEntry>,        // ログエントリのリスト
    total_count: usize                  // 総ログ数
}

SamplingLogEntry {
    timestamp: String,                  // タイムスタンプ（ISO 8601形式）
    server_name: String,                // サーバー名
    request_id: String,                 // リクエストID（UUID）
    model_preferences: ModelPreferences,// モデル設定
    system_prompt: Option<String>,      // システムプロンプト
    messages: Vec<SamplingMessage>,     // メッセージリスト
    max_tokens: u32,                    // 最大トークン数
    status: SamplingStatus,             // ステータス
    error_message: Option<String>       // エラーメッセージ
}

ModelPreferences {
    hints: Vec<ModelHint>,              // モデルヒント
    cost_priority: Option<f64>,         // コスト優先度
    speed_priority: Option<f64>,        // 速度優先度
    intelligence_priority: Option<f64>  // 知能優先度
}

ModelHint {
    name: Option<String>                // モデル名のヒント
}

SamplingMessage {
    role: String,                       // ロール（"user", "assistant"）
    content: SamplingContent           // メッセージ内容
}

SamplingContent {
    content_type: String,               // "text" or "image"
    text: Option<String>,               // テキストコンテンツ
    data: Option<String>,               // 画像データ（Base64）
    mime_type: Option<String>           // MIMEタイプ
}

SamplingStatus = "Pending" | "Success" | "Failed"
```

#### 使用例
```bash
# 全てのSamplingログを取得
> tools/call sampling_logs '{"server": "fundamental_analysis"}'

# 最新10件のログを取得
> tools/call sampling_logs '{"server": "fundamental_analysis", "limit": 10}'

# 失敗したリクエストのみ取得
> tools/call sampling_logs '{"server": "fundamental_analysis", "status": "failed"}'
```

---

## 3. データ構造設計

### 3.1 models/request.rs に追加する型

```rust
use serde::{Deserialize, Serialize};

/// Samplingログ取得のリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogsRequest {
    pub server: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_limit() -> usize {
    100
}

fn default_status() -> String {
    "all".to_string()
}
```

### 3.2 models/response.rs に追加する型

```rust
use serde::{Deserialize, Serialize};

/// Samplingログのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogsResponse {
    pub server: String,
    pub logs: Vec<SamplingLogEntry>,
    pub total_count: usize,
}

/// Samplingログエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogEntry {
    pub timestamp: String,
    pub server_name: String,
    pub request_id: String,
    pub model_preferences: ModelPreferences,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub messages: Vec<SamplingMessage>,
    pub max_tokens: u32,
    pub status: SamplingStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// モデル設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    #[serde(default)]
    pub hints: Vec<ModelHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f64>,
}

/// モデルヒント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Samplingメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    pub role: String,
    pub content: SamplingContent,
}

/// Samplingコンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Samplingステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingStatus {
    Pending,
    Success,
    Failed,
}
```

### 3.3 新規ファイル: src/services/sampling_logger.rs

```rust
use std::sync::{Arc, Mutex};
use anyhow::Result;
use crate::models::{SamplingLogEntry, SamplingStatus};

/// Samplingロガー（メモリベース）
#[derive(Debug, Clone)]
pub struct SamplingLogger {
    logs: Arc<Mutex<Vec<SamplingLogEntry>>>,
    max_logs: usize,
}

impl SamplingLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            max_logs,
        }
    }

    /// ログエントリを追加
    pub fn add_log(&self, entry: SamplingLogEntry) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(entry);

        // 最大件数を超えたら古いログを削除
        if logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    /// ログを取得（フィルタリング付き）
    pub fn get_logs(&self, server_name: &str, limit: usize, status: &str) -> Vec<SamplingLogEntry> {
        let logs = self.logs.lock().unwrap();

        logs.iter()
            .filter(|log| log.server_name == server_name)
            .filter(|log| {
                match status {
                    "success" => matches!(log.status, SamplingStatus::Success),
                    "failed" => matches!(log.status, SamplingStatus::Failed),
                    _ => true,
                }
            })
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 総ログ数を取得
    pub fn count_logs(&self, server_name: &str) -> usize {
        let logs = self.logs.lock().unwrap();
        logs.iter().filter(|log| log.server_name == server_name).count()
    }
}
```

---

## 4. 実装フェーズ

### Phase 3.1: データモデル実装

**タスク:**
- `src/models/request.rs`に`SamplingLogsRequest`を追加
- `src/models/response.rs`にSampling関連のレスポンス型を追加
- `src/models/mod.rs`で型をエクスポート

**実装ファイル:**
- `src/models/request.rs`
- `src/models/response.rs`
- `src/models/mod.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- `cargo check` がパス
- `cargo clippy` で警告なし
- すべての型にSerialize/Deserialize実装

---

### Phase 3.2: SamplingLogger実装

**タスク:**
- 新規ファイル`src/services/sampling_logger.rs`を作成
- メモリベースのログ管理システムを実装
- スレッドセーフな実装（Arc<Mutex<>>使用）
- ログのフィルタリング機能

**実装ファイル:**
- `src/services/sampling_logger.rs`
- `src/services/mod.rs`（エクスポート追加）

**見積もり:** 3-4時間

**チェックポイント:**
- ログの追加・取得が正常に動作
- フィルタリング機能が正しく動作
- スレッドセーフ性の確保

**技術的考慮事項:**
```rust
// 最大ログ数の管理
const DEFAULT_MAX_LOGS: usize = 1000;

// ログのローテーション
if logs.len() > self.max_logs {
    logs.remove(0);  // FIFO方式
}
```

---

### Phase 3.3: サービスレイヤー実装

**タスク:**
- `InspectorService`に`SamplingLogger`フィールドを追加
- `sampling_logs`メソッドを実装
- コンストラクタでSamplingLoggerを初期化

**実装ファイル:**
- `src/services/inspector.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- `SamplingLogger`の統合が正しく動作
- ログ取得のビジネスロジックが適切

**実装例:**
```rust
pub struct InspectorService {
    client: Arc<Mutex<Box<dyn McpClient>>>,
    sampling_logger: SamplingLogger,
}

impl InspectorService {
    pub fn new(client: Box<dyn McpClient>) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            sampling_logger: SamplingLogger::new(1000),
        }
    }

    pub async fn sampling_logs(&self, request: SamplingLogsRequest) -> Result<SamplingLogsResponse> {
        let logs = self.sampling_logger.get_logs(
            &request.server,
            request.limit,
            &request.status,
        );
        let total_count = self.sampling_logger.count_logs(&request.server);

        Ok(SamplingLogsResponse {
            server: request.server,
            logs,
            total_count,
        })
    }
}
```

---

### Phase 3.4: サーバーツール実装

**タスク:**
- `InspectorServer`に`#[tool]`マクロで`sampling_logs`ツールを追加
- パラメータ型の定義

**実装ファイル:**
- `src/server/mod.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- ツールがMCP Inspectorから呼び出し可能
- 引数の検証が適切
- Phase 1/2の実装パターンと一貫性

**実装例:**
```rust
#[tool(description = "対象MCPサーバーからのSamplingリクエストのログを取得します")]
async fn sampling_logs(&self, request: SamplingLogsRequest) -> Result<SamplingLogsResponse> {
    let mut service = self.create_service(&request.server).await?;
    service.sampling_logs(request).await
}
```

---

### Phase 3.5: テストと検証

**タスク:**
- ユニットテストの作成
- MCP Inspector CLI での手動テスト
- エラーケースの確認

**テスト環境:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --release --method tools/call \
  --tool-name sampling_logs \
  --tool-arg server=fundamental_analysis
```

**見積もり:** 2-3時間

**チェックポイント:**
- すべてのツールが正常に動作
- エラーハンドリングが適切
- `cargo test` がパス

---

## 5. 技術的考慮事項

### 5.1 メモリ管理

**課題:**
- ログが無制限に増加するとメモリを圧迫

**対策:**
- 最大ログ数（DEFAULT_MAX_LOGS = 1000）を設定
- FIFO方式でログをローテーション
- 設定可能な最大ログ数

**実装例:**
```rust
pub fn add_log(&self, entry: SamplingLogEntry) {
    let mut logs = self.logs.lock().unwrap();
    logs.push(entry);

    // ログローテーション
    if logs.len() > self.max_logs {
        logs.drain(0..logs.len() - self.max_logs);
    }
}
```

---

### 5.2 スレッドセーフ性

**課題:**
- 複数のリクエストが同時にログにアクセスする可能性

**対策:**
- `Arc<Mutex<Vec<SamplingLogEntry>>>`でスレッドセーフを保証
- Mutexのロック時間を最小化

**実装パターン:**
```rust
pub fn get_logs(&self, ...) -> Vec<SamplingLogEntry> {
    let logs = self.logs.lock().unwrap();
    // ロック中は最小限の処理のみ
    logs.iter()
        .filter(...)
        .cloned()
        .collect()
}
```

---

### 5.3 将来の拡張性

**現在の実装（Phase 3）:**
- メモリベースのログ管理
- 基本的なフィルタリング

**将来の拡張可能性:**
- 永続化（ファイルまたはデータベース）
- より高度なフィルタリング（時間範囲、メッセージ内容など）
- ログのエクスポート機能
- Transport層での実際のSampling監視

**設計での配慮:**
```rust
// Trait化により将来的な置き換えが容易
pub trait Logger {
    fn add_log(&self, entry: SamplingLogEntry);
    fn get_logs(&self, ...) -> Vec<SamplingLogEntry>;
}

// メモリベース実装
pub struct MemoryLogger { ... }
impl Logger for MemoryLogger { ... }

// 将来のファイルベース実装
pub struct FileLogger { ... }
impl Logger for FileLogger { ... }
```

---

## 6. テストプラン

### 6.1 テストケース1: sampling_logs（空のログ）

**目的:** ログが存在しない場合の動作確認

**手順:**
```bash
> tools/call sampling_logs '{"server": "fundamental_analysis"}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "logs": [],
  "total_count": 0
}
```

---

### 6.2 テストケース2: ログのフィルタリング

**目的:** ステータスフィルタが正しく動作することを確認

**手順:**
```bash
# 失敗したリクエストのみ
> tools/call sampling_logs '{"server": "fundamental_analysis", "status": "failed"}'

# 成功したリクエストのみ
> tools/call sampling_logs '{"server": "fundamental_analysis", "status": "success"}'
```

**期待される結果:**
- 指定したステータスのログのみが返される
- total_countは全体の数を返す

---

### 6.3 テストケース3: limit パラメータ

**目的:** limit パラメータが正しく動作することを確認

**手順:**
```bash
> tools/call sampling_logs '{"server": "fundamental_analysis", "limit": 10}'
```

**期待される結果:**
- 最新10件のログが返される
- total_countは全体の数を返す

---

### 6.4 ユニットテスト

**実装ファイル:** `src/services/sampling_logger.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_logs() {
        let logger = SamplingLogger::new(10);

        let entry = SamplingLogEntry {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            server_name: "test_server".to_string(),
            // ... 他のフィールド
        };

        logger.add_log(entry.clone());

        let logs = logger.get_logs("test_server", 10, "all");
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_log_rotation() {
        let logger = SamplingLogger::new(5);

        // 10個のログを追加
        for i in 0..10 {
            let entry = create_test_entry(i);
            logger.add_log(entry);
        }

        let logs = logger.get_logs("test_server", 100, "all");
        assert_eq!(logs.len(), 5);  // 最大5件まで保持
    }

    #[test]
    fn test_status_filter() {
        let logger = SamplingLogger::new(100);

        logger.add_log(create_entry_with_status(SamplingStatus::Success));
        logger.add_log(create_entry_with_status(SamplingStatus::Failed));

        let success_logs = logger.get_logs("test_server", 10, "success");
        assert_eq!(success_logs.len(), 1);

        let failed_logs = logger.get_logs("test_server", 10, "failed");
        assert_eq!(failed_logs.len(), 1);
    }
}
```

---

## 7. 成功基準

Phase 3の完了は、以下の基準をすべて満たすことで判断します：

### 7.1 機能要件
- ✅ `sampling_logs`ツールが実装されている
- ✅ ログの追加・取得が正常に動作する
- ✅ フィルタリング機能（limit、status）が動作する
- ✅ エラーハンドリングが適切に実装されている

### 7.2 品質要件
- ✅ `cargo check` がエラーなしでパス
- ✅ `cargo clippy` が警告なしでパス
- ✅ `cargo test` がすべてパス
- ✅ `cargo build --release` が成功

### 7.3 テスト要件
- ✅ MCP Inspector CLIでのすべてのテストケースが成功
- ✅ ユニットテストが実装され、すべてパス
- ✅ エラーケースが適切に処理される

### 7.4 ドキュメント要件
- ✅ ツールの使用例がドキュメント化されている
- ✅ エラーメッセージが明確で理解しやすい

---

## 8. 見積もり

| フェーズ | タスク | 見積もり時間 |
|---------|--------|------------|
| Phase 3.1 | データモデル実装 | 2-3時間 |
| Phase 3.2 | SamplingLogger実装 | 3-4時間 |
| Phase 3.3 | サービスレイヤー実装 | 2-3時間 |
| Phase 3.4 | サーバーツール実装 | 2-3時間 |
| Phase 3.5 | テストと検証 | 2-3時間 |
| **合計** | - | **11-16時間** |

**備考:**
- Phase 1/2の実装パターンを踏襲するため、学習コストは最小限
- 技術的な複雑性は低く、リスクも低い
- 予期しない問題が発生した場合は+20%のバッファを見込む

---

## 9. リスクと対策

### リスク1: Sampling機能の実用性

**影響度:** 中
**発生確率:** 高

**対策:**
- Phase 3はロギング専用の実装であることを明確に文書化
- 将来的な完全実装への基盤として位置づける
- ユーザーに期待値を正しく伝える

---

### リスク2: メモリ使用量の増加

**影響度:** 中
**発生確率:** 低

**対策:**
- 最大ログ数（1000件）を設定
- ログローテーションを実装
- 設定可能な最大ログ数

---

### リスク3: Transport層の監視が困難

**影響度:** 高
**発生確率:** 高

**対策:**
- Phase 3ではTransport層の監視は実装しない
- 基本的なロギングインフラのみを構築
- 将来のrmcpアップデートまたはカスタムTransport実装を待つ

---

## 10. 依存関係

### 10.1 前提条件

| 項目 | ステータス | 備考 |
|------|----------|------|
| Phase 1 完了 | ✅ | tools_list, tools_call 実装済み |
| Phase 2 完了 | ✅ | resources/prompts機能実装済み |
| rmcp 0.8.5 | ✅ | Cargo.tomlで依存関係設定済み |
| fundamental_analysis MCP | ✅ | テスト用サーバーとして使用 |
| MCP Inspector CLI | ✅ | 検証環境として使用 |

### 10.2 技術スタック

- **Rust:** 1.70以上
- **rmcp:** 0.8.5
- **tokio:** 非同期ランタイム
- **anyhow:** エラーハンドリング
- **serde:** シリアライゼーション

---

## 11. 次のステップ

Phase 3完了後、以下のタスクを実施します：

### 11.1 ドキュメント更新
- [ ] README.md にPhase 3機能を追加
- [ ] 使用例セクションに新ツールの例を追加
- [ ] 技術設計書のアーカイブ

### 11.2 実地テスト
- [ ] Claude Desktop での統合テスト
- [ ] fundamental_analysis MCP との実運用テスト

### 11.3 Phase 4 準備
- [ ] Phase 4計画書の作成（ロギング・モニタリング機能の拡張）
- [ ] パフォーマンス最適化の検討
- [ ] 永続化ストレージの検討

---

## 12. 制限事項と今後の展望

### 12.1 Phase 3の制限事項

Phase 3の実装には以下の制限があります：

1. **実際のSampling監視は行わない**
   - Transport層でのメッセージ監視が技術的に困難
   - rmcp 0.8.5の制約

2. **ログは手動で追加される**
   - 自動的なログ記録は未実装
   - テスト目的のみの機能

3. **メモリベースのストレージ**
   - 再起動するとログが消失
   - 永続化は未実装

### 12.2 将来の展望

Phase 3の基盤の上に、将来的に以下の機能を追加できます：

**Phase 4以降の候補:**
- Transport層でのSampling監視（カスタムTransport実装）
- ログの永続化（ファイルまたはデータベース）
- より高度なフィルタリングと検索機能
- ログのエクスポート機能（JSON、CSV）
- 統計情報とダッシュボード

---

## 13. 参考資料

### 13.1 公式ドキュメント

- **MCP Protocol Specification - Sampling**
  https://modelcontextprotocol.io/docs/concepts/sampling

- **rmcp Documentation**
  https://docs.rs/rmcp/0.8.5/

- **MCP Inspector**
  https://github.com/modelcontextprotocol/inspector

### 13.2 関連リソース

- **Phase 1/2 実装**
  `c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/src/`

- **fundamental_analysis MCP**
  `c:/Users/takah/work/my_mcp_server/fundamental_analysis/`

- **Phase 3 技術設計書**
  `c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/docs/phase3_sampling_technical_design.md`

---

## 14. 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|----------|---------|-------|
| 2025-11-14 | 1.0 | 初版作成 | Claude Code |

---

**以上がPhase 3実装計画書です。この計画に従って実装を進めることで、MCP Inspector MCPサーバーにSamplingロギング機能を追加できます。**

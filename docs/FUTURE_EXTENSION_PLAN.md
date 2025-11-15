# MCP Inspector MCP Server 拡張計画書
## Phase 4以降の戦略的ロードマップ

**文書バージョン**: 1.0
**作成日**: 2025-11-15
**対象**: Phase 4以降の機能拡張

---

## 1. エグゼクティブサマリー

### 1.1 ビジョン

MCP Inspector MCP Serverを**最も包括的で実用的なMCPデバッギングツール**にすることを目標とします。AIエージェントが他のMCPサーバーを完全に検査・監視・デバッグできる環境を提供します。

### 1.2 現在の状況

#### 実装済み機能
- ✅ **Phase 1**: Tools検査（tools_list, tools_call）
- ✅ **Phase 2**: Resources & Prompts検査（resources_*, prompts_*）
- ✅ **Phase 3**: Samplingログインフラ（sampling_logs）

#### Phase 3の重要な制限事項
Phase 3では**ロギングインフラのみ**を実装し、実際のTransport層でのSampling監視は未実装です。これはrmcp 0.8.5の技術的制約によるものです。

### 1.3 主要マイルストーン

| フェーズ | 主な目的 | 優先度 | 推定工数 |
|---------|---------|-------|---------|
| **Phase 4** | Transport層Sampling監視 | 最高 | 20-30時間 |
| **Phase 5** | ログ永続化とパフォーマンス最適化 | 高 | 15-25時間 |
| **Phase 6** | 高度な検査機能（Logging/Tracing） | 中 | 25-35時間 |
| **Phase 7** | 開発者体験向上（Dashboard/Export） | 中 | 30-40時間 |
| **Phase 8** | エコシステム統合 | 低 | 20-30時間 |

---

## 2. Phase 4: Transport層Sampling監視（詳細設計）

### 2.1 問題の明確化

#### 現在の制限事項

**技術的課題:**
- rmcp 0.8.5の`ServerHandler`トレイトにSampling関連メソッドが存在しない
- MCPプロトコル上、Samplingは「サーバー→クライアント」方向の通信
- mcp_inspector_mcpは「サーバー」として動作するため、Sampling requestを自然に受信できない

**アーキテクチャ上の課題:**
```
対象サーバー (Sampling発信元)
    ↓ sampling/createMessage
    ❌ 受信方法が存在しない
mcp_inspector_mcp (MCPサーバー)
    ↓ 転送方法が存在しない
AIエージェント (MCPクライアント)
```

### 2.2 技術的アプローチの評価

#### オプションA: rmcp SDKアップデート待ち

**概要:**
rmcpのアップデートで`ServerHandler`にSampling関連メソッドが追加されるまで待つ。

**メリット:**
- 標準的なアプローチで実装可能
- 保守性が高い
- 将来のrmcpアップデートに自動的に対応

**デメリット:**
- アップデート時期が不明（数ヶ月〜数年）
- rmcpチームの開発方針に依存
- 機能提供までの遅延

**推奨度:** ⭐⭐⭐⭐☆
**実装工数:** 0時間（待機のみ） + 統合作業5-10時間

**アクション:**
1. rmcpリポジトリをウォッチし、Sampling関連のIssue/PRを監視
2. rmcpコミュニティでSampling対応の要望を提出
3. 定期的に最新バージョンをチェック

---

#### オプションB: カスタムTransport実装

**概要:**
rmcpの`Transport`トレイトを独自実装し、メッセージを低レベルで監視する。

**アーキテクチャ:**
```rust
pub trait Transport {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()>;
    async fn receive(&mut self) -> Result<JsonRpcMessage>;
}

pub struct MonitoringTransport {
    inner: Box<dyn Transport>,
    sampling_logger: Arc<SamplingLogger>,
}

impl Transport for MonitoringTransport {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        // メッセージを監視
        if message.method == Some("sampling/createMessage") {
            // ログに記録
            self.sampling_logger.add_log(...).await;
        }
        self.inner.send(message).await
    }

    async fn receive(&mut self) -> Result<JsonRpcMessage> {
        let message = self.inner.receive().await?;
        // 受信メッセージも監視可能
        Ok(message)
    }
}
```

**実装手順:**
1. `src/client/monitoring_transport.rs`を作成
2. `Transport`トレイトを実装
3. JSON-RPCメッセージのパース処理を追加
4. `sampling/createMessage`メソッドの検出ロジック
5. `SamplingLogger`への統合
6. `StdioClient`で`MonitoringTransport`を使用

**メリット:**
- rmcp SDKに依存しない独立実装
- メッセージレベルでの完全な制御が可能
- 他のメソッドの監視にも応用可能

**デメリット:**
- 低レベル実装が必要（複雑度高）
- rmcpの内部構造変更に影響を受ける可能性
- テストが困難（統合テストが必須）

**推奨度:** ⭐⭐⭐☆☆
**実装工数:** 15-25時間

**技術的リスク:**
- rmcp内部のJSON-RPC実装との互換性維持
- エラーハンドリングの複雑化
- デバッグの困難性

---

#### オプションC: プロキシモード実装

**概要:**
mcp_inspector_mcpを「プロキシサーバー」として動作させ、AIエージェント↔対象サーバー間の全通信を中継する。

**アーキテクチャ:**
```
AIエージェント (Client)
    ↓ MCP Protocol
mcp_inspector_mcp (Proxy Server + Proxy Client)
    ↓ 全メッセージを中継
対象サーバー (Server)
```

**動作フロー:**
1. AIエージェントは mcp_inspector_mcp に接続
2. mcp_inspector_mcp は対象サーバーへの接続を確立
3. すべてのMCPメッセージ（tools, resources, prompts, sampling）を双方向に中継
4. 通信内容をログとして記録

**実装手順:**
1. プロキシモードの設計（新規アーキテクチャ）
2. 双方向メッセージ転送ロジックの実装
3. Sampling含むすべてのメソッドの中継
4. ログ記録機能の統合
5. 設定ファイルでのモード切替

**メリット:**
- Sampling含むすべてのMCP機能を完全に監視可能
- プロトコルレベルでの透明性が高い
- 新しいMCP機能にも自動対応

**デメリット:**
- 大規模なアーキテクチャ変更が必要
- 既存の実装（Phase 1-3）の大幅な修正が必要
- レイテンシの増加（中継オーバーヘッド）
- 複雑度が非常に高い

**推奨度:** ⭐⭐☆☆☆
**実装工数:** 40-60時間

**技術的課題:**
- 既存のInspectorServerとの統合方法
- エラー伝播の複雑化
- パフォーマンス最適化が必須

---

#### オプションD: モック/シミュレーション機能

**概要:**
Transport層の監視は諦め、代わりに「Sampling動作をシミュレートする機能」を提供する。

**実装内容:**
```rust
#[tool(description = "対象サーバーがSamplingを使用すると仮定した場合のシミュレーション")]
async fn simulate_sampling(
    &self,
    request: SimulateSamplingRequest
) -> Result<SimulateSamplingResponse> {
    // テスト用のSamplingリクエストを生成
    // ログに記録
    // モックレスポンスを返す
}
```

**メリット:**
- 実装が非常にシンプル
- テスト・デバッグ用途には十分
- 技術的リスクが低い

**デメリット:**
- 実際のSampling通信は監視できない
- 実用性が限定的
- Phase 3からの実質的な進展がない

**推奨度:** ⭐☆☆☆☆
**実装工数:** 5-10時間

---

### 2.3 推奨アプローチ

**最終推奨: オプションA（rmcp SDKアップデート待ち） + オプションB（カスタムTransport）のハイブリッド戦略**

#### 段階的実装計画

**フェーズ 4.1: 調査とコミュニティ連携（1-2週間）**
- rmcpリポジトリのIssueトラッカーでSampling対応状況を確認
- コミュニティでSampling監視の需要を提起
- rmcp開発チームとのコミュニケーション

**フェーズ 4.2: カスタムTransportの実験的実装（2-3週間）**
- `MonitoringTransport`のプロトタイプ実装
- 基本的なメッセージ監視機能の検証
- Sampling検出ロジックの実装

**フェーズ 4.3: 統合とテスト（1-2週間）**
- `StdioClient`への統合
- 統合テストの作成
- エラーケースの網羅的な検証

**フェーズ 4.4: ドキュメント化（1週間）**
- 技術設計書の更新
- 使用例の追加
- トラブルシューティングガイド

#### 判断基準
- **rmcpアップデートが6ヶ月以内に見込める場合**: オプションA（待機）
- **rmcpアップデートの見通しが立たない場合**: オプションB（カスタムTransport）を実装

---

### 2.4 技術設計: カスタムTransport実装

#### ファイル構成
```
src/
├── client/
│   ├── mod.rs
│   ├── stdio_client.rs
│   ├── monitoring_transport.rs  (NEW)
│   └── message_inspector.rs     (NEW)
```

#### コア実装: MonitoringTransport

```rust
use rmcp::transport::{Transport, JsonRpcMessage};
use crate::services::SamplingLogger;
use std::sync::Arc;
use anyhow::Result;

/// メッセージ監視機能を持つTransportラッパー
pub struct MonitoringTransport<T: Transport> {
    /// 実際の通信を行う内部Transport
    inner: T,

    /// Samplingログを記録するロガー
    sampling_logger: Arc<SamplingLogger>,

    /// 対象サーバー名（ログ記録用）
    server_name: String,
}

impl<T: Transport> MonitoringTransport<T> {
    pub fn new(
        inner: T,
        sampling_logger: Arc<SamplingLogger>,
        server_name: String,
    ) -> Self {
        Self {
            inner,
            sampling_logger,
            server_name,
        }
    }

    /// メッセージがSamplingリクエストかどうかを判定
    fn is_sampling_request(message: &JsonRpcMessage) -> bool {
        message.method.as_ref().map_or(false, |m| {
            m == "sampling/createMessage"
        })
    }

    /// Samplingリクエストをログに記録
    async fn log_sampling_request(&self, message: &JsonRpcMessage) {
        if let Some(params) = &message.params {
            // JSON-RPCパラメータからSamplingリクエストをパース
            if let Ok(request) = serde_json::from_value::<CreateMessageRequestParam>(params.clone()) {
                let entry = SamplingLogEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    server_name: self.server_name.clone(),
                    request_id: message.id.clone().unwrap_or_default(),
                    model_preferences: request.model_preferences.unwrap_or_default(),
                    system_prompt: request.system_prompt,
                    messages: convert_messages(request.messages),
                    max_tokens: request.max_tokens,
                    status: SamplingStatus::Pending,
                    error_message: None,
                };

                self.sampling_logger.add_log(entry);
            }
        }
    }
}

#[async_trait::async_trait]
impl<T: Transport> Transport for MonitoringTransport<T> {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        // 送信前にメッセージを検査
        if Self::is_sampling_request(&message) {
            self.log_sampling_request(&message).await;
        }

        // 実際の送信
        self.inner.send(message).await
    }

    async fn receive(&mut self) -> Result<JsonRpcMessage> {
        // メッセージを受信
        let message = self.inner.receive().await?;

        // 受信メッセージの検査（レスポンスの監視など）
        // 必要に応じて実装

        Ok(message)
    }
}
```

#### StdioClientの更新

```rust
use crate::client::MonitoringTransport;
use crate::services::SamplingLogger;

pub struct StdioClient {
    config: ServerConfig,
    service: Arc<Mutex<Option<RunningService<RoleClient, ()>>>>,
    sampling_logger: Arc<SamplingLogger>,  // 追加
}

impl StdioClient {
    pub fn new(config: ServerConfig, sampling_logger: Arc<SamplingLogger>) -> Self {
        Self {
            config,
            service: Arc::new(Mutex::new(None)),
            sampling_logger,
        }
    }

    async fn connect(&self) -> Result<()> {
        let mut guard = self.service.lock().await;

        if guard.is_some() {
            return Ok(());
        }

        let config = self.config.clone();

        // 基本的なTransportを作成
        let base_transport = TokioChildProcess::new(
            Command::new(&config.params.command).configure(|cmd| {
                cmd.args(&config.params.args);
                for (key, value) in &config.params.env {
                    cmd.env(key, value);
                }
            }),
        )?;

        // MonitoringTransportでラップ
        let monitoring_transport = MonitoringTransport::new(
            base_transport,
            Arc::clone(&self.sampling_logger),
            config.name.clone(),
        );

        // サービスを起動
        let service = ()
            .serve(monitoring_transport)
            .await?;

        *guard = Some(service);
        Ok(())
    }
}
```

---

### 2.5 実装見積もり

#### 詳細タスク

| タスク | 説明 | 見積工数 |
|-------|------|---------|
| **4.1 調査** | rmcp調査、コミュニティ連携 | 4-6時間 |
| **4.2 Transport実装** | MonitoringTransport実装 | 8-12時間 |
| **4.3 メッセージパース** | JSON-RPC処理、Samplingパース | 4-6時間 |
| **4.4 統合** | StdioClientへの統合 | 3-5時間 |
| **4.5 テスト** | ユニット・統合テスト | 5-8時間 |
| **4.6 ドキュメント** | 技術設計書、使用例 | 3-4時間 |
| **合計** | | **27-41時間** |

---

### 2.6 リスク分析と緩和策

#### リスク1: rmcp内部構造の変更

**影響度:** 高
**発生確率:** 中

**緩和策:**
- rmcpのバージョンを固定（0.8.5）
- アップデート時には慎重な検証を実施
- Transport実装を独立したモジュールに分離

#### リスク2: Samplingメッセージの検出失敗

**影響度:** 高
**発生確率:** 低

**緩和策:**
- 包括的な統合テストの実装
- エラーログの詳細化
- フォールバック処理の実装

#### リスク3: パフォーマンス劣化

**影響度:** 中
**発生確率:** 低

**緩和策:**
- メッセージ検査処理の最適化
- 非同期処理の活用
- ベンチマークテストの実施

---

## 3. Phase 5: ログ永続化とパフォーマンス最適化

### 3.1 ログ永続化戦略

#### 現在の状態

**制限事項:**
- メモリベースのログ保存（`Arc<Mutex<Vec<SamplingLogEntry>>>`）
- サーバー再起動でログが消失
- 最大ログ数の制限（1000件）

#### 永続化オプション比較

| オプション | メリット | デメリット | 推奨度 |
|-----------|---------|-----------|-------|
| **SQLite** | 高機能、SQL使用可、トランザクション | 依存関係増加、セットアップ必要 | ⭐⭐⭐⭐☆ |
| **sled** | Rust製、組み込み型、高速 | キーバリュー型のみ | ⭐⭐⭐⭐⭐ |
| **JSON Lines** | シンプル、人間可読 | 検索性能低、スケールしない | ⭐⭐⭐☆☆ |
| **bincode** | 高速、小サイズ | 人間不可読、バージョン管理困難 | ⭐⭐☆☆☆ |

#### 推奨アプローチ: sled（Rust製組み込みDB）

**選定理由:**
- 依存関係がシンプル（Pure Rust）
- 高速な読み書き性能
- トランザクション対応
- 設定ファイル不要
- サイズが小さい

**アーキテクチャ:**
```rust
use sled::Db;

pub struct PersistentSamplingLogger {
    db: Db,
    max_logs: usize,
}

impl PersistentSamplingLogger {
    pub fn new(db_path: &str, max_logs: usize) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self { db, max_logs })
    }

    pub fn add_log(&self, entry: SamplingLogEntry) -> Result<()> {
        // エントリをシリアライズ
        let key = format!("{}:{}", entry.server_name, entry.timestamp);
        let value = serde_json::to_vec(&entry)?;

        // DBに保存
        self.db.insert(key.as_bytes(), value)?;

        // ログローテーション処理
        self.rotate_logs(&entry.server_name)?;

        Ok(())
    }

    pub fn get_logs(
        &self,
        server_name: &str,
        limit: usize,
        status: &str,
    ) -> Result<Vec<SamplingLogEntry>> {
        let prefix = format!("{}:", server_name);

        let logs: Vec<SamplingLogEntry> = self.db
            .scan_prefix(prefix.as_bytes())
            .filter_map(|result| {
                let (_, value) = result.ok()?;
                let entry: SamplingLogEntry = serde_json::from_slice(&value).ok()?;
                Some(entry)
            })
            .filter(|entry| match status {
                "success" => entry.status == SamplingStatus::Success,
                "failed" => entry.status == SamplingStatus::Failed,
                _ => true,
            })
            .take(limit)
            .collect();

        Ok(logs)
    }

    fn rotate_logs(&self, server_name: &str) -> Result<()> {
        let prefix = format!("{}:", server_name);
        let count = self.db.scan_prefix(prefix.as_bytes()).count();

        if count > self.max_logs {
            // 古いログを削除
            let to_remove = count - self.max_logs;
            for (i, result) in self.db.scan_prefix(prefix.as_bytes()).enumerate() {
                if i >= to_remove {
                    break;
                }
                if let Ok((key, _)) = result {
                    self.db.remove(key)?;
                }
            }
        }

        Ok(())
    }
}
```

#### 実装手順

**Phase 5.1: Loggerトレイトの抽出（3-5時間）**
```rust
pub trait LoggerBackend {
    fn add_log(&self, entry: SamplingLogEntry) -> Result<()>;
    fn get_logs(&self, server_name: &str, limit: usize, status: &str) -> Result<Vec<SamplingLogEntry>>;
    fn count_logs(&self, server_name: &str) -> Result<usize>;
}

pub struct MemoryLogger { /* 既存実装 */ }
pub struct PersistentLogger { /* sled実装 */ }
```

**Phase 5.2: sled統合（5-8時間）**
- Cargo.tomlに`sled`依存追加
- `PersistentLogger`実装
- ログローテーション処理
- エラーハンドリング

**Phase 5.3: 設定ベースの切替（2-3時間）**
```toml
[logging]
backend = "persistent"  # または "memory"
db_path = "./data/logs.db"
max_logs = 10000
```

**Phase 5.4: マイグレーション機能（3-5時間）**
- メモリ→永続化への移行ツール
- データのエクスポート/インポート

---

### 3.2 データ保持ポリシー

#### ログローテーション戦略

**基本ポリシー:**
- **デフォルト最大件数:** 10,000件/サーバー
- **ローテーション方式:** FIFO（古いものから削除）
- **時間ベース削除:** オプション（30日以前のログを削除）

**設定例:**
```toml
[logging]
max_logs_per_server = 10000
retention_days = 30  # オプション
auto_cleanup = true  # 定期的なクリーンアップ
```

**実装:**
```rust
pub struct RetentionPolicy {
    pub max_logs: usize,
    pub retention_days: Option<u32>,
}

impl PersistentLogger {
    pub async fn cleanup_old_logs(&self, policy: &RetentionPolicy) -> Result<()> {
        if let Some(days) = policy.retention_days {
            let cutoff = Utc::now() - Duration::days(days as i64);

            for result in self.db.iter() {
                let (key, value) = result?;
                let entry: SamplingLogEntry = serde_json::from_slice(&value)?;

                if let Ok(timestamp) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                    if timestamp < cutoff {
                        self.db.remove(key)?;
                    }
                }
            }
        }

        Ok(())
    }
}
```

---

### 3.3 パフォーマンス最適化

#### 3.3.1 接続プーリング

**課題:**
現在、ツール呼び出しごとに新しいMCP接続を確立している可能性があり、オーバーヘッドが大きい。

**解決策: 接続プール実装**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ClientPool {
    clients: Arc<RwLock<HashMap<String, Arc<StdioClient>>>>,
    config_loader: Arc<ConfigLoader>,
}

impl ClientPool {
    pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config_loader,
        }
    }

    /// クライアントを取得（なければ作成）
    pub async fn get_or_create(&self, server_name: &str) -> Result<Arc<StdioClient>> {
        // 読み取りロックで既存クライアントを確認
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(server_name) {
                if client.is_connected().await {
                    return Ok(Arc::clone(client));
                }
            }
        }

        // 書き込みロックで新規作成
        let mut clients = self.clients.write().await;

        // ダブルチェック（他のタスクが作成済みの可能性）
        if let Some(client) = clients.get(server_name) {
            if client.is_connected().await {
                return Ok(Arc::clone(client));
            }
        }

        // 新規クライアント作成
        let config = self.config_loader.get_server_config(server_name)?;
        let client = Arc::new(StdioClient::new(config, self.sampling_logger.clone()));
        client.connect().await?;

        clients.insert(server_name.to_string(), Arc::clone(&client));

        Ok(client)
    }

    /// 不要なクライアントをクリーンアップ
    pub async fn cleanup(&self) {
        let mut clients = self.clients.write().await;

        clients.retain(|_, client| {
            // 接続チェックは非同期なので簡易的に保持
            // 実際にはタイムアウト機能などが必要
            true
        });
    }
}
```

**実装工数:** 8-12時間

---

#### 3.3.2 キャッシング戦略

**キャッシュ対象:**
1. **ツール一覧** - 頻繁に変更されない
2. **リソース一覧** - サーバー起動後は安定
3. **プロンプト一覧** - 静的な情報

**実装:**
```rust
use std::time::{Duration, Instant};

pub struct CachedResponse<T> {
    data: T,
    cached_at: Instant,
    ttl: Duration,
}

pub struct ResponseCache {
    tools: Arc<RwLock<HashMap<String, CachedResponse<Vec<ToolInfo>>>>>,
    resources: Arc<RwLock<HashMap<String, CachedResponse<Vec<ResourceInfo>>>>>,
    prompts: Arc<RwLock<HashMap<String, CachedResponse<Vec<PromptInfo>>>>>,
}

impl ResponseCache {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_tools(&self, server: &str) -> Option<Vec<ToolInfo>> {
        let cache = self.tools.read().await;

        if let Some(cached) = cache.get(server) {
            if cached.cached_at.elapsed() < cached.ttl {
                return Some(cached.data.clone());
            }
        }

        None
    }

    pub async fn set_tools(&self, server: String, tools: Vec<ToolInfo>, ttl: Duration) {
        let mut cache = self.tools.write().await;

        cache.insert(server, CachedResponse {
            data: tools,
            cached_at: Instant::now(),
            ttl,
        });
    }

    pub async fn invalidate(&self, server: &str) {
        let mut tools = self.tools.write().await;
        let mut resources = self.resources.write().await;
        let mut prompts = self.prompts.write().await;

        tools.remove(server);
        resources.remove(server);
        prompts.remove(server);
    }
}
```

**キャッシュ無効化戦略:**
- TTLベース（デフォルト: 5分）
- 明示的な無効化API提供
- サーバー再接続時に自動無効化

**実装工数:** 6-10時間

---

#### 3.3.3 並列処理の改善

**現在の課題:**
複数ツール呼び出しが直列実行される可能性

**解決策:**
```rust
use tokio::task::JoinSet;

impl InspectorService {
    /// 複数サーバーのツール一覧を並列取得
    pub async fn list_tools_batch(
        &self,
        servers: Vec<String>,
    ) -> Result<HashMap<String, Vec<ToolInfo>>> {
        let mut tasks = JoinSet::new();

        for server in servers {
            let service = self.clone();
            tasks.spawn(async move {
                let tools = service.list_tools(&server).await?;
                Ok::<_, anyhow::Error>((server, tools))
            });
        }

        let mut results = HashMap::new();

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((server, tools))) => {
                    results.insert(server, tools);
                }
                Ok(Err(e)) => {
                    tracing::warn!("Failed to list tools: {:?}", e);
                }
                Err(e) => {
                    tracing::error!("Task join error: {:?}", e);
                }
            }
        }

        Ok(results)
    }
}
```

**実装工数:** 4-6時間

---

### 3.4 Phase 5 実装見積もり

| タスク | 見積工数 |
|-------|---------|
| 5.1 Loggerトレイト抽出 | 3-5時間 |
| 5.2 sled統合 | 5-8時間 |
| 5.3 設定ベース切替 | 2-3時間 |
| 5.4 マイグレーション | 3-5時間 |
| 5.5 接続プール実装 | 8-12時間 |
| 5.6 キャッシング実装 | 6-10時間 |
| 5.7 並列処理改善 | 4-6時間 |
| 5.8 テスト・ドキュメント | 4-6時間 |
| **合計** | **35-55時間** |

---

## 4. Phase 6: 高度な検査機能

### 4.1 Logging/Tracing検査

**目的:**
対象サーバーが出力するログやトレース情報を監視・検査する。

#### 機能仕様

**新規ツール: logging/list**
```rust
#[tool(description = "対象サーバーのロギング設定を取得")]
async fn logging_list(&self, request: LoggingListRequest) -> Result<LoggingListResponse> {
    // MCP logging/listメソッドを呼び出し
}
```

**新規ツール: logging/setLevel**
```rust
#[tool(description = "対象サーバーのログレベルを設定")]
async fn logging_set_level(&self, request: SetLogLevelRequest) -> Result<SetLogLevelResponse> {
    // MCP logging/setLevelメソッドを呼び出し
}
```

**ログ収集機能:**
- Transportレベルで`logging/message`通知を監視
- ログエントリをデータベースに保存
- ログ検索・フィルタリング機能

**実装工数:** 15-20時間

---

### 4.2 サーバー設定検査

**機能仕様:**

**新規ツール: server/inspect**
```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ServerInspectResponse {
    pub server_name: String,
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub runtime_info: RuntimeInfo,
}

pub struct ServerCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
    pub sampling: bool,
    pub logging: bool,
}

pub struct RuntimeInfo {
    pub pid: Option<u32>,
    pub uptime_seconds: u64,
    pub connection_status: String,
}
```

**実装工数:** 8-12時間

---

### 4.3 ヘルスチェック機能

**機能仕様:**

**新規ツール: health/check**
```rust
#[tool(description = "対象サーバーのヘルスチェック")]
async fn health_check(&self, request: HealthCheckRequest) -> Result<HealthCheckResponse> {
    // ping送信
    // レスポンスタイム測定
    // エラー率算出
}

pub struct HealthCheckResponse {
    pub server_name: String,
    pub status: HealthStatus,  // Healthy | Degraded | Unhealthy
    pub response_time_ms: u64,
    pub last_check: String,
    pub error_count: u64,
    pub error_rate: f64,
}
```

**実装工数:** 6-10時間

---

### 4.4 Phase 6 実装見積もり

| タスク | 見積工数 |
|-------|---------|
| 6.1 Logging検査機能 | 15-20時間 |
| 6.2 サーバー設定検査 | 8-12時間 |
| 6.3 ヘルスチェック | 6-10時間 |
| 6.4 テスト・ドキュメント | 5-8時間 |
| **合計** | **34-50時間** |

---

## 5. Phase 7: 開発者体験向上

### 5.1 Webダッシュボード

**技術スタック評価:**

| フレームワーク | メリット | デメリット | 推奨度 |
|--------------|---------|-----------|-------|
| **Tauri** | Rustベース、小サイズ、デスクトップアプリ | フロントエンド技術必要 | ⭐⭐⭐⭐⭐ |
| **Leptos** | Full-stack Rust、WASM | 学習曲線、エコシステム未熟 | ⭐⭐⭐⭐☆ |
| **Axum + HTMX** | シンプル、サーバーサイド | モダンUI困難 | ⭐⭐⭐☆☆ |

#### 推奨: Tauri + React

**アーキテクチャ:**
```
┌─────────────────────────────┐
│    Tauri Desktop App        │
│  ┌────────────────────────┐ │
│  │   React Frontend       │ │
│  │  - ダッシュボード       │ │
│  │  - リアルタイムログ     │ │
│  │  - ツール実行UI        │ │
│  └──────────┬─────────────┘ │
│             │ IPC            │
│  ┌──────────▼─────────────┐ │
│  │   Rust Backend         │ │
│  │  - InspectorService    │ │
│  │  - WebSocket server    │ │
│  └────────────────────────┘ │
└─────────────────────────────┘
```

**主要機能:**
1. **サーバー一覧表示** - 接続状態、ヘルスステータス
2. **リアルタイムログビューア** - Samplingログ、通信ログ
3. **ツール実行UI** - GUIでツール呼び出し
4. **パフォーマンスモニタリング** - レスポンスタイム、エラー率

**実装工数:** 40-60時間

---

### 5.2 エクスポート/インポート機能

**機能仕様:**

**新規ツール: logs/export**
```rust
#[tool(description = "ログをエクスポート")]
async fn logs_export(&self, request: LogsExportRequest) -> Result<LogsExportResponse> {
    // JSON、CSV、JSON Linesフォーマットでエクスポート
}

pub struct LogsExportRequest {
    pub server: String,
    pub format: ExportFormat,  // Json | Csv | JsonLines
    pub filter: Option<LogFilter>,
}

pub struct LogsExportResponse {
    pub file_path: String,
    pub entry_count: usize,
    pub file_size_bytes: u64,
}
```

**新規ツール: logs/import**
```rust
#[tool(description = "ログをインポート")]
async fn logs_import(&self, request: LogsImportRequest) -> Result<LogsImportResponse> {
    // エクスポートされたログを再インポート
}
```

**実装工数:** 10-15時間

---

### 5.3 設定プロファイル管理

**機能:**
- 複数の設定プロファイルの管理
- プロファイル切替機能
- デフォルト設定のエクスポート/インポート

**実装例:**
```toml
[profiles.development]
logging.backend = "memory"
cache.ttl_seconds = 60

[profiles.production]
logging.backend = "persistent"
logging.db_path = "./data/prod_logs.db"
cache.ttl_seconds = 300
```

**実装工数:** 8-12時間

---

### 5.4 Phase 7 実装見積もり

| タスク | 見積工数 |
|-------|---------|
| 7.1 Tauriダッシュボード | 40-60時間 |
| 7.2 エクスポート/インポート | 10-15時間 |
| 7.3 設定プロファイル | 8-12時間 |
| 7.4 テスト・ドキュメント | 8-12時間 |
| **合計** | **66-99時間** |

---

## 6. Phase 8: エコシステム統合

### 6.1 IDEプラグイン

**対象IDE:**
- VS Code (優先度: 高)
- JetBrains IDEs (優先度: 中)

**VS Code拡張機能:**
```typescript
// extension.ts
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    // MCP Inspector コマンド
    let inspectTool = vscode.commands.registerCommand(
        'mcp-inspector.inspectTool',
        async () => {
            // ツール一覧を取得してQuickPickで表示
        }
    );

    // ログビューア
    let logViewer = vscode.window.createTreeView('mcpInspectorLogs', {
        treeDataProvider: new LogTreeDataProvider()
    });

    context.subscriptions.push(inspectTool, logViewer);
}
```

**実装工数:** 25-35時間

---

### 6.2 CI/CD統合

**GitHub Actions ワークフロー例:**
```yaml
name: MCP Server Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install MCP Inspector
        run: cargo install --path .

      - name: Test MCP Server
        run: |
          # MCP Inspectorでツール一覧取得
          mcp-inspector tools-list --server my-server

          # 特定のツールをテスト
          mcp-inspector tools-call --server my-server \
            --tool test_tool --args '{"param": "value"}'
```

**実装工数:** 8-12時間

---

### 6.3 Phase 8 実装見積もり

| タスク | 見積工数 |
|-------|---------|
| 8.1 VS Code拡張 | 25-35時間 |
| 8.2 CI/CD統合ガイド | 8-12時間 |
| 8.3 ドキュメント・サンプル | 5-8時間 |
| **合計** | **38-55時間** |

---

## 7. 技術評価

### 7.1 rmcp SDKトラッキング

**監視項目:**
- rmcpリポジトリのリリースノート
- Sampling関連のIssue/PR
- API変更のアナウンス

**アクション:**
```bash
# GitHub CLI でIssue監視
gh repo view modelcontextprotocol/rmcp --web

# RSS フィードで更新を監視
# https://github.com/modelcontextprotocol/rmcp/releases.atom
```

---

### 7.2 代替SDKの評価

現時点では rmcp が公式SDKであり、代替選択肢は限定的。

**将来的な選択肢:**
- TypeScript SDK（mcp package）
- Python SDK（検討中）

---

### 7.3 データベースオプション

#### sled vs SQLite 詳細比較

| 項目 | sled | SQLite |
|------|------|--------|
| **言語** | Pure Rust | C |
| **セットアップ** | ゼロ設定 | マイグレーション必要 |
| **クエリ** | KV操作のみ | SQL使用可 |
| **パフォーマンス** | 非常に高速 | 高速 |
| **トランザクション** | ○ | ○ |
| **バイナリサイズ** | 小 | 中 |
| **エコシステム** | 小 | 非常に大 |

**最終推奨:** sled（シンプルさとRustエコシステムとの親和性）

---

### 7.4 UIフレームワーク評価

#### Tauri vs Leptos vs Axum+HTMX

**Tauri:**
- デスクトップアプリとして配布可能
- Rustバックエンド + 任意のフロントエンド
- バイナリサイズが小さい

**Leptos:**
- Full-stack Rust
- WASMベース
- 学習曲線が急

**Axum + HTMX:**
- サーバーサイドレンダリング
- シンプルな実装
- モダンUIは困難

**最終推奨:** Tauri（配布の容易さと柔軟性）

---

## 8. 実装ロードマップ

### 8.1 全体タイムライン

```
Phase 4: Transport層Sampling監視
├─ 4.1 調査・コミュニティ連携 (Week 1)
├─ 4.2 カスタムTransport実装 (Week 2-3)
└─ 4.3 統合・テスト (Week 4)
   推定: 4週間 (27-41時間)

Phase 5: ログ永続化・最適化
├─ 5.1 Loggerトレイト抽出 (Week 5)
├─ 5.2 sled統合 (Week 5-6)
├─ 5.3 接続プール (Week 6-7)
└─ 5.4 キャッシング (Week 7-8)
   推定: 4週間 (35-55時間)

Phase 6: 高度な検査機能
├─ 6.1 Logging検査 (Week 9-10)
├─ 6.2 サーバー設定検査 (Week 11)
└─ 6.3 ヘルスチェック (Week 11-12)
   推定: 4週間 (34-50時間)

Phase 7: 開発者体験向上
├─ 7.1 Tauriダッシュボード (Week 13-16)
├─ 7.2 エクスポート/インポート (Week 17)
└─ 7.3 設定プロファイル (Week 17-18)
   推定: 6週間 (66-99時間)

Phase 8: エコシステム統合
├─ 8.1 VS Code拡張 (Week 19-21)
└─ 8.2 CI/CD統合 (Week 22)
   推定: 4週間 (38-55時間)

総推定: 22週間 (約5.5ヶ月)
```

---

### 8.2 Phase 4優先事項

**最優先タスク:**
1. ✅ rmcp SDKのSampling対応状況調査
2. ✅ コミュニティでのフィードバック収集
3. ✅ MonitoringTransportプロトタイプ実装

**判断ポイント:**
- **2週間以内**: rmcp開発状況を確認 → アップデート予定あり/なし判断
- **アップデート予定あり**: オプションA（待機）を選択
- **アップデート予定なし**: オプションB（カスタムTransport）実装開始

---

### 8.3 Phase 5以降の優先順位

**高優先度:**
- Phase 5.1-5.2: ログ永続化（実用性が高い）
- Phase 5.5: 接続プール（パフォーマンス改善）

**中優先度:**
- Phase 6.1: Logging検査（デバッグに有用）
- Phase 7.2: エクスポート機能（データ分析に必要）

**低優先度:**
- Phase 7.1: Webダッシュボード（CLIで十分な場合は延期）
- Phase 8: エコシステム統合（コア機能が安定してから）

---

## 9. リスク評価と緩和

### 9.1 技術リスク

#### リスク1: rmcp SDK制約

**影響度:** 最高
**発生確率:** 高

**緩和策:**
- カスタムTransport実装の準備
- rmcpコミュニティとの積極的な連携
- 代替アプローチの継続的な検討

---

#### リスク2: プロトコル変更

**影響度:** 高
**発生確率:** 中

**緩和策:**
- MCPプロトコル仕様の定期的な確認
- 後方互換性の維持
- バージョン管理の徹底

---

#### リスク3: パフォーマンス問題

**影響度:** 中
**発生確率:** 中

**緩和策:**
- 早期のベンチマーク実施
- プロファイリングツールの活用
- 段階的な最適化

---

### 9.2 リソースリスク

#### リスク4: 開発時間の見積もり誤差

**影響度:** 中
**発生確率:** 高

**緩和策:**
- 各Phaseに20%のバッファを追加
- MVP優先のアプローチ
- 定期的な進捗レビュー

---

#### リスク5: 技術的負債の蓄積

**影響度:** 中
**発生確率:** 中

**緩和策:**
- コードレビューの徹底
- リファクタリング時間の確保
- テストカバレッジの維持

---

### 9.3 互換性リスク

#### リスク6: 破壊的変更

**影響度:** 高
**発生確率:** 低

**緩和策:**
- セマンティックバージョニングの採用
- 非推奨期間の設定
- マイグレーションガイドの提供

---

## 10. 成功基準

### 10.1 Phase 4成功基準

- ✅ Samplingリクエストを90%以上の確率で検出
- ✅ 既存機能（Phase 1-3）への影響なし
- ✅ パフォーマンス劣化5%以内
- ✅ すべてのテストがパス

---

### 10.2 Phase 5成功基準

- ✅ ログが永続化され、再起動後も保持される
- ✅ 100,000件のログでもパフォーマンス低下なし
- ✅ 接続プールにより初回呼び出し以降50%高速化
- ✅ キャッシュヒット率80%以上

---

### 10.3 Phase 6-8成功基準

- ✅ すべての主要MCP機能が検査可能
- ✅ Webダッシュボードで直感的な操作が可能
- ✅ VS Code拡張が10,000ダウンロードを達成
- ✅ CI/CD統合により自動テストが実現

---

## 11. 次のステップ

### 11.1 即座に着手すべきタスク

1. **rmcp調査（Week 1）**
   - rmcpリポジトリのIssue/PR確認
   - コミュニティDiscordでの情報収集
   - Sampling対応のタイムライン確認

2. **技術検証（Week 1-2）**
   - MonitoringTransportのプロトタイプ実装
   - JSON-RPCメッセージパースの検証
   - 統合テスト環境の構築

3. **設計文書の作成（Week 2）**
   - Phase 4詳細技術設計書
   - API仕様書
   - テスト計画書

---

### 11.2 ドキュメント整備

必要なドキュメント:
- [ ] Phase 4 技術設計書（詳細版）
- [ ] API リファレンス
- [ ] コントリビューターガイド
- [ ] アーキテクチャ決定記録（ADR）

---

### 11.3 コミュニティ連携

**アクション:**
- rmcp GitHubリポジトリでFeature Requestを作成
- MCPコミュニティDiscordで議論を開始
- 技術ブログ記事の執筆（知見の共有）

---

## 12. 付録

### 12.1 用語集

| 用語 | 説明 |
|------|------|
| **MCP** | Model Context Protocol |
| **Sampling** | サーバーがLLMにリクエストを送る機能 |
| **Transport** | MCPの通信層 |
| **rmcp** | Rust MCP SDK |
| **sled** | Rust製組み込みデータベース |

---

### 12.2 参考リソース

#### 公式ドキュメント
- [MCP Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp/)
- [Sampling Specification](https://modelcontextprotocol.io/docs/concepts/sampling)

#### 技術リソース
- [Tauri Documentation](https://tauri.app/)
- [sled Database](https://github.com/spacejam/sled)
- [tokio Async Runtime](https://tokio.rs/)

#### コミュニティ
- [MCP Discord](https://discord.gg/mcp)
- [rmcp GitHub](https://github.com/modelcontextprotocol/rmcp)

---

### 12.3 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|----------|---------|-------|
| 2025-11-15 | 1.0 | 初版作成 | Solution Architect Agent |

---

## まとめ

本拡張計画書は、MCP Inspector MCP Serverを最も包括的なMCPデバッギングツールにするための戦略的ロードマップです。

**重要なポイント:**

1. **Phase 4は最優先事項** - Sampling監視の実現が最も価値が高い
2. **段階的なアプローチ** - MVPから始め、段階的に拡張
3. **技術的柔軟性** - rmcp SDKの進展に応じて戦略を調整
4. **実用性重視** - 開発者が実際に使いたくなる機能を優先

**推奨される実装順序:**
```
Phase 4 → Phase 5（ログ永続化のみ） → Phase 6.1（Logging検査）
→ Phase 5（最適化） → Phase 7.2（エクスポート） → その他
```

この計画に従って実装を進めることで、MCP Inspectorは業界標準のデバッギングツールとなるでしょう。

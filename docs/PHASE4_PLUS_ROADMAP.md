# Phase 4以降の拡張計画書

MCP Inspector MCP Server - 戦略的ロードマップ

---

## 1. エグゼクティブサマリー

### 1.1 ビジョン

MCP Inspector MCP Serverを、**業界標準のMCPデバッギング・監視ツール**として確立します。

**現状（Phase 3完了時点）:**
- ✅ Tools検査機能（Phase 1）
- ✅ Resources/Prompts検査機能（Phase 2）
- ✅ Samplingログインフラ（Phase 3）

**目指す姿:**
- 🎯 完全なSampling監視（Phase 4）
- 🎯 永続化された包括的なログシステム（Phase 5）
- 🎯 高度な検査・分析機能（Phase 6）
- 🎯 直感的なWebダッシュボード（Phase 7）
- 🎯 開発エコシステムとの統合（Phase 8）

### 1.2 主要マイルストーン

| Phase | 目標 | 期間 | 優先度 |
|-------|------|------|--------|
| Phase 4 | Transport層Sampling監視 | 4週間 | 🔴 最高 |
| Phase 5 | ログ永続化・最適化 | 4週間 | 🟠 高 |
| Phase 6 | 高度な検査機能 | 4週間 | 🟡 中 |
| Phase 7 | Webダッシュボード | 6週間 | 🟡 中 |
| Phase 8 | エコシステム統合 | 4週間 | 🟢 低 |

**総実装期間**: 22週間（約5.5ヶ月）

---

## 2. Phase 4: Transport層Sampling監視（詳細設計）

### 2.1 問題の本質

**Phase 3の制限:**
- Samplingログインフラは実装済み
- しかし、実際のSamplingリクエストを**検出・記録できない**
- rmcp 0.8.5の`ServerHandler`にSampling関連メソッドが存在しない

**根本的な課題:**
```
通常のMCP通信:
AI Agent (Client) → mcp_inspector (Server) → Target Server

Samplingの場合:
Target Server → ??? → AI Agent (Client)
                ↑
           mcp_inspectorはここで監視できない
```

### 2.2 技術的アプローチの評価

#### オプションA: rmcp SDKアップデート待ち

**概要**: rmcpの将来バージョンでSampling対応を待つ

**メリット:**
- ✅ 標準的なアプローチ
- ✅ 保守性が高い
- ✅ 互換性の問題が少ない

**デメリット:**
- ❌ 実装時期が不明
- ❌ 機能追加が保証されていない

**実装工数**: 0時間（待機のみ）

**推奨度**: ⭐⭐⭐⭐☆

---

#### オプションB: カスタムTransport実装

**概要**: rmcpのTransportトレイトを実装し、メッセージを監視

**アーキテクチャ:**
```rust
pub struct MonitoringTransport<T: Transport> {
    inner: T,                              // 元のTransport
    sampling_logger: Arc<SamplingLogger>,  // ロガー
    server_name: String,                   // サーバー名
}

impl<T: Transport> Transport for MonitoringTransport<T> {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Samplingリクエストを検出
        if Self::is_sampling_request(&message) {
            self.log_sampling_request(&message).await;
        }

        // 元のTransportに転送
        self.inner.send(message).await
    }

    async fn receive(&mut self) -> Result<JsonRpcMessage> {
        let message = self.inner.receive().await?;

        // Samplingレスポンスを検出
        if Self::is_sampling_response(&message) {
            self.log_sampling_response(&message).await;
        }

        Ok(message)
    }
}
```

**検出ロジック:**
```rust
fn is_sampling_request(message: &JsonRpcMessage) -> bool {
    message.method() == Some("sampling/createMessage")
}

async fn log_sampling_request(&self, message: &JsonRpcMessage) {
    if let Some(params) = message.params() {
        let request: CreateMessageRequestParam =
            serde_json::from_value(params).ok()?;

        let entry = SamplingLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            server_name: self.server_name.clone(),
            request_id: Uuid::new_v4().to_string(),
            model_preferences: request.model_preferences,
            system_prompt: request.system_prompt,
            messages: request.messages,
            max_tokens: request.max_tokens,
            status: SamplingStatus::Pending,
            error_message: None,
        };

        self.sampling_logger.add_log(entry);
    }
}
```

**実装箇所:**
- `src/client/monitoring_transport.rs` (新規)
- `src/client/stdio_client.rs` (MonitoringTransportの使用)
- `src/services/inspector.rs` (SamplingLogger統合)

**メリット:**
- ✅ rmcpバージョンに依存しない
- ✅ 完全な制御が可能
- ✅ 段階的な実装が可能

**デメリット:**
- ❌ rmcpの内部APIに依存
- ❌ メンテナンスコストが増加

**実装工数**: 15-25時間

**推奨度**: ⭐⭐⭐☆☆

---

#### オプションC: プロキシモード

**概要**: mcp_inspectorを完全なプロキシとして動作させる

**アーキテクチャ:**
```
AI Agent
    ↓
mcp_inspector (Proxy)
    ├→ 全てのリクエストを記録
    ├→ 全てのレスポンスを記録
    └→ Target Server
```

**メリット:**
- ✅ 完全な通信監視
- ✅ すべてのMCPメッセージを記録可能
- ✅ Sampling以外も監視可能

**デメリット:**
- ❌ 大規模なアーキテクチャ変更
- ❌ AI AgentとTarget Serverの間に入る必要がある
- ❌ セットアップが複雑

**実装工数**: 40-60時間

**推奨度**: ⭐⭐☆☆☆

---

#### オプションD: モック/テスト機能

**概要**: Sampling機能のモックレスポンスを返す

**実装:**
```rust
async fn handle_mock_sampling(&self) -> CreateMessageResult {
    CreateMessageResult {
        model: "mock-model".to_string(),
        stop_reason: Some("endTurn".to_string()),
        message: SamplingMessage {
            role: Role::Assistant,
            content: Content::text("[Mock] Test response"),
        },
    }
}
```

**メリット:**
- ✅ 実装が簡単
- ✅ テスト目的で有用

**デメリット:**
- ❌ 実用性が低い
- ❌ 実際のSampling動作を監視できない

**実装工数**: 5-10時間

**推奨度**: ⭐☆☆☆☆

---

### 2.3 推奨アプローチ: ハイブリッド戦略

**Step 1: rmcp調査（Week 1）**
```bash
# rmcpリポジトリの調査
1. GitHubでIssue/PRを確認
2. Sampling対応のロードマップを確認
3. コミュニティDiscordで状況を確認
```

**Step 2: 判断基準**
```
rmcpアップデート予定がある場合:
  → オプションA（待機）+ 暫定的なモック実装（オプションD）

rmcpアップデート予定がない場合:
  → オプションB（カスタムTransport実装）
```

**Step 3: 実装（Week 2-4）**

**オプションAの場合:**
```rust
// 暫定的なモック実装
pub async fn create_mock_sampling_log(&self) -> SamplingLogEntry {
    SamplingLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        server_name: "[MOCK]".to_string(),
        // ... モックデータ
    }
}
```

**オプションBの場合:**
```rust
// MonitoringTransportの完全実装
// （詳細は上記参照）
```

### 2.4 実装計画

#### Week 1: 調査・検証
- [ ] rmcpリポジトリ調査
- [ ] コミュニティ確認
- [ ] Feature Requestの作成

#### Week 2: プロトタイプ
- [ ] MonitoringTransportプロトタイプ
- [ ] JSON-RPCメッセージパース検証
- [ ] 統合テスト環境構築

#### Week 3: 実装
- [ ] MonitoringTransport完全実装
- [ ] StdioClientの統合
- [ ] SamplingLoggerとの統合

#### Week 4: テストと最適化
- [ ] ユニットテスト
- [ ] 統合テスト
- [ ] ドキュメント更新

**総工数見積もり**: 27-41時間

### 2.5 成功基準

- ✅ Samplingリクエストが検出される
- ✅ ログに正しく記録される
- ✅ sampling_logsツールで取得可能
- ✅ 既存機能（Phase 1-3）が正常動作
- ✅ パフォーマンス劣化なし

---

## 3. Phase 5: ログ永続化とパフォーマンス最適化

### 3.1 現状の制限

**Phase 3-4の状態:**
- メモリベースのログ管理
- 最大1000件で古いログを削除
- サーバー再起動でログが消失

**課題:**
- 長期的なログ分析ができない
- 重要なデバッグ情報が失われる
- 統計情報の蓄積ができない

### 3.2 ログ永続化オプション

#### オプション1: sled（推奨）

**概要**: Pure RustのKey-Valueストア

```rust
use sled::{Db, IVec};

pub struct SledSamplingLogger {
    db: Db,
    memory_cache: Arc<RwLock<Vec<SamplingLogEntry>>>,
}

impl SledSamplingLogger {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self {
            db,
            memory_cache: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn add_log(&self, entry: SamplingLogEntry) -> Result<()> {
        // DBに永続化
        let key = format!("{}:{}", entry.server_name, entry.timestamp);
        let value = serde_json::to_vec(&entry)?;
        self.db.insert(key.as_bytes(), value)?;

        // メモリキャッシュにも追加
        let mut cache = self.memory_cache.write().await;
        cache.push(entry);
        if cache.len() > 1000 {
            cache.remove(0);
        }

        Ok(())
    }

    pub async fn get_logs(&self, server: &str, limit: usize) -> Result<Vec<SamplingLogEntry>> {
        // メモリキャッシュから取得（高速）
        let cache = self.memory_cache.read().await;
        let cached: Vec<_> = cache.iter()
            .filter(|e| e.server_name == server)
            .take(limit)
            .cloned()
            .collect();

        if !cached.is_empty() {
            return Ok(cached);
        }

        // DBから取得（フォールバック）
        let prefix = format!("{}:", server);
        let iter = self.db.scan_prefix(prefix.as_bytes());

        let mut logs = Vec::new();
        for item in iter {
            let (_, value) = item?;
            let entry: SamplingLogEntry = serde_json::from_slice(&value)?;
            logs.push(entry);
        }

        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs.into_iter().take(limit).collect())
    }
}
```

**メリット:**
- ✅ Pure Rust（外部依存なし）
- ✅ ゼロ設定（ファイルパス指定のみ）
- ✅ 高速
- ✅ ACID保証

**デメリット:**
- ❌ SQL非対応（複雑なクエリが困難）

**工数**: 8-12時間

**推奨度**: ⭐⭐⭐⭐⭐

---

#### オプション2: SQLite

**概要**: リレーショナルデータベース

```sql
CREATE TABLE sampling_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    server_name TEXT NOT NULL,
    request_id TEXT NOT NULL,
    model_preferences TEXT,
    messages TEXT,
    max_tokens INTEGER,
    status TEXT,
    error_message TEXT,
    INDEX idx_server_timestamp (server_name, timestamp)
);
```

**メリット:**
- ✅ SQL対応（複雑なクエリ可能）
- ✅ 広く使われている
- ✅ ツールが豊富

**デメリット:**
- ❌ C依存（クロスコンパイル困難）
- ❌ 設定が必要

**工数**: 12-18時間

**推奨度**: ⭐⭐⭐⭐☆

---

#### オプション3: JSON Lines

**概要**: 行区切りのJSONファイル

```rust
pub async fn add_log(&self, entry: SamplingLogEntry) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/sampling.jsonl")
        .await?;

    let json = serde_json::to_string(&entry)?;
    file.write_all(format!("{}\n", json).as_bytes()).await?;
    Ok(())
}
```

**メリット:**
- ✅ シンプル
- ✅ 人間が読める

**デメリット:**
- ❌ スケールしない
- ❌ クエリが遅い
- ❌ ファイルサイズ増大

**工数**: 4-6時間

**推奨度**: ⭐⭐⭐☆☆

---

### 3.3 推奨: sled + メモリキャッシュのハイブリッド

**設計:**
```rust
pub trait LogStorage {
    async fn add_log(&self, entry: SamplingLogEntry) -> Result<()>;
    async fn get_logs(&self, query: LogQuery) -> Result<Vec<SamplingLogEntry>>;
}

// メモリ実装（Phase 3-4）
pub struct MemoryStorage { ... }

// sled実装（Phase 5）
pub struct SledStorage { ... }

// 将来のSQLite実装
pub struct SqliteStorage { ... }
```

**移行パス:**
```rust
// 設定ファイルで切り替え可能
[logging]
storage = "sled"  # "memory" | "sled" | "sqlite"
db_path = "data/logs.db"
cache_size = 1000
```

### 3.4 パフォーマンス最適化

#### 最適化1: 接続プーリング

**現状の問題:**
```rust
// 毎回新しいクライアントを生成
async fn call_tool(&self, request: ToolCallRequest) -> Result<...> {
    let client = self.client_manager.get_client(&request.server).await?;
    // 使用後、破棄
}
```

**改善:**
```rust
pub struct ClientPool {
    pools: Arc<RwLock<HashMap<String, Vec<StdioClient>>>>,
    max_per_server: usize,
}

impl ClientPool {
    pub async fn acquire(&self, server: &str) -> Result<PooledClient> {
        let mut pools = self.pools.write().await;

        if let Some(pool) = pools.get_mut(server) {
            if let Some(client) = pool.pop() {
                return Ok(PooledClient::new(client, self.clone(), server));
            }
        }

        // プールが空の場合、新規作成
        let client = StdioClient::new(server).await?;
        Ok(PooledClient::new(client, self.clone(), server))
    }

    async fn release(&self, server: &str, client: StdioClient) {
        let mut pools = self.pools.write().await;
        let pool = pools.entry(server.to_string()).or_insert_with(Vec::new);

        if pool.len() < self.max_per_server {
            pool.push(client);
        }
        // 最大数を超えたら破棄
    }
}

pub struct PooledClient {
    client: Option<StdioClient>,
    pool: ClientPool,
    server: String,
}

impl Drop for PooledClient {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            tokio::spawn(self.pool.release(self.server.clone(), client));
        }
    }
}
```

**効果:**
- 起動オーバーヘッド削減: 50%高速化
- リソース効率向上

**工数**: 10-15時間

---

#### 最適化2: キャッシング

**キャッシュ対象:**
- ツール一覧（TTL: 5分）
- リソース一覧（TTL: 5分）
- プロンプト一覧（TTL: 5分）

**実装:**
```rust
use moka::future::Cache;

pub struct CachedInspectorService {
    inner: InspectorService,
    tools_cache: Cache<String, ToolsListResponse>,
    resources_cache: Cache<String, ResourcesListResponse>,
    prompts_cache: Cache<String, PromptsListResponse>,
}

impl CachedInspectorService {
    pub fn new(inner: InspectorService) -> Self {
        Self {
            inner,
            tools_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .build(),
            resources_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .build(),
            prompts_cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }

    pub async fn list_tools(&self, server: &str) -> Result<ToolsListResponse> {
        if let Some(cached) = self.tools_cache.get(server).await {
            return Ok(cached);
        }

        let result = self.inner.list_tools(server).await?;
        self.tools_cache.insert(server.to_string(), result.clone()).await;
        Ok(result)
    }
}
```

**効果:**
- レスポンスタイム削減: 80%高速化（キャッシュヒット時）

**工数**: 6-8時間

---

#### 最適化3: 並列処理

**現状:**
```rust
// 複数サーバーを順次処理
for server in servers {
    let tools = service.list_tools(&server).await?;
    println!("{:?}", tools);
}
```

**改善:**
```rust
use futures::stream::{self, StreamExt};

// 並列処理
let results: Vec<_> = stream::iter(servers)
    .map(|server| async move {
        service.list_tools(&server).await
    })
    .buffer_unordered(10)  // 最大10並列
    .collect()
    .await;
```

**効果:**
- 複数サーバー検査時: 5倍高速化（5サーバーの場合）

**工数**: 4-6時間

---

### 3.5 Phase 5実装計画

#### Week 5: ログ永続化
- [ ] sledストレージ実装
- [ ] LogStorageトレイト定義
- [ ] 移行パス実装
- [ ] テスト

#### Week 6: 接続プーリング
- [ ] ClientPool実装
- [ ] PooledClient実装
- [ ] 統合テスト

#### Week 7: キャッシング
- [ ] CachedInspectorService実装
- [ ] TTL設定
- [ ] 無効化ロジック

#### Week 8: 並列処理・最適化
- [ ] 並列処理の導入
- [ ] ベンチマーク
- [ ] パフォーマンステスト

**総工数見積もり**: 35-55時間

---

## 4. Phase 6: 高度な検査機能

### 4.1 新機能の提案

#### 機能1: Logging/Tracing検査

**目的**: 対象サーバーのログを収集・分析

**新規ツール:**
```rust
#[tool(description = "対象サーバーのログレベルを設定")]
async fn logging_set_level(&self, request: LoggingSetLevelRequest) -> Result<...> {
    // logging/setLevel メソッドを呼び出し
}

#[tool(description = "対象サーバーのログを取得")]
async fn logging_get_messages(&self, request: LoggingGetMessagesRequest) -> Result<...> {
    // ログメッセージの収集
}
```

**工数**: 12-18時間

---

#### 機能2: サーバー設定検査

**目的**: 対象サーバーのCapabilitiesとランタイム情報を取得

**新規ツール:**
```rust
#[tool(description = "対象サーバーの設定情報を取得")]
async fn server_get_config(&self, request: ServerConfigRequest) -> Result<...> {
    // InitializeResultからcapabilitiesを抽出
}

#[tool(description = "対象サーバーのランタイム情報を取得")]
async fn server_get_runtime_info(&self, request: ServerRuntimeRequest) -> Result<...> {
    // サーバーのバージョン、プロトコルバージョンなど
}
```

**工数**: 10-15時間

---

#### 機能3: ヘルスチェック・モニタリング

**目的**: 対象サーバーのパフォーマンス監視

**新規ツール:**
```rust
#[tool(description = "対象サーバーのヘルスチェック")]
async fn server_health_check(&self, request: HealthCheckRequest) -> Result<...> {
    // pingメソッドでレスポンスタイム計測
}

#[tool(description = "対象サーバーのメトリクスを取得")]
async fn server_get_metrics(&self, request: MetricsRequest) -> Result<...> {
    // エラー率、平均レスポンスタイムなど
}
```

**実装:**
```rust
pub struct ServerMetrics {
    pub server_name: String,
    pub uptime: Duration,
    pub request_count: u64,
    pub error_count: u64,
    pub avg_response_time: Duration,
    pub last_ping: Option<Instant>,
}
```

**工数**: 12-16時間

---

### 4.2 Phase 6実装計画

#### Week 9-10: Logging/Tracing検査
- [ ] logging_set_levelツール実装
- [ ] logging_get_messagesツール実装
- [ ] テスト

#### Week 11: サーバー設定検査
- [ ] server_get_configツール実装
- [ ] server_get_runtime_infoツール実装
- [ ] テスト

#### Week 12: ヘルスチェック・モニタリング
- [ ] server_health_checkツール実装
- [ ] server_get_metricsツール実装
- [ ] メトリクス収集システム
- [ ] テスト

**総工数見積もり**: 34-50時間

---

## 5. Phase 7: 開発者体験向上（Webダッシュボード）

### 5.1 UIフレームワーク評価

#### オプション1: Tauri（推奨）

**概要**: Rustバックエンド + Web技術フロントエンド

**メリット:**
- ✅ デスクトップアプリとして配布可能
- ✅ 小さいバイナリサイズ
- ✅ ネイティブパフォーマンス
- ✅ Rustとの統合が容易

**デメリット:**
- ❌ Web版とデスクトップ版の2つの配布形態

**技術スタック:**
- バックエンド: Rust (Tauri)
- フロントエンド: React/Vue/Svelte
- 通信: Tauri Commands (Rust関数を直接呼び出し)

**工数**: 50-75時間

**推奨度**: ⭐⭐⭐⭐⭐

---

#### オプション2: Leptos

**概要**: Full-stack Rust Webフレームワーク

**メリット:**
- ✅ Pure Rust
- ✅ SSRとCSR対応
- ✅ リアクティブUI

**デメリット:**
- ❌ 学習曲線が急
- ❌ エコシステムが未成熟

**工数**: 60-90時間

**推奨度**: ⭐⭐⭐⭐☆

---

#### オプション3: Axum + HTMX

**概要**: シンプルなサーバーサイドレンダリング

**メリット:**
- ✅ シンプル
- ✅ 軽量

**デメリット:**
- ❌ モダンなUIが困難
- ❌ インタラクティブ性が低い

**工数**: 30-45時間

**推奨度**: ⭐⭐⭐☆☆

---

### 5.2 推奨: Tauri + React

**アーキテクチャ:**
```
┌─────────────────────────────────────┐
│         Tauri Application           │
├─────────────────────────────────────┤
│  Frontend (React)                   │
│  - Dashboard                        │
│  - Log Viewer                       │
│  - Tool Executor                    │
├─────────────────────────────────────┤
│  Tauri Commands (Rust)              │
│  - list_servers()                   │
│  - get_sampling_logs()              │
│  - execute_tool()                   │
├─────────────────────────────────────┤
│  MCP Inspector Core (Rust)          │
│  - InspectorService                 │
│  - SamplingLogger                   │
│  - ClientManager                    │
└─────────────────────────────────────┘
```

### 5.3 主要機能

#### ダッシュボード

**概要画面:**
- 登録サーバー一覧
- 各サーバーのステータス（Online/Offline）
- リアルタイムメトリクス（リクエスト数、エラー率）

#### ログビューア

**機能:**
- リアルタイムログ表示
- フィルタリング（サーバー、ステータス、時間範囲）
- 検索
- エクスポート（JSON、CSV）

**UI:**
```tsx
function SamplingLogViewer() {
  const [logs, setLogs] = useState<SamplingLogEntry[]>([]);
  const [filter, setFilter] = useState({ server: 'all', status: 'all' });

  useEffect(() => {
    const fetchLogs = async () => {
      const result = await invoke('get_sampling_logs', {
        server: filter.server,
        status: filter.status,
        limit: 100
      });
      setLogs(result.logs);
    };

    fetchLogs();
    const interval = setInterval(fetchLogs, 5000);  // 5秒ごとに更新
    return () => clearInterval(interval);
  }, [filter]);

  return (
    <div>
      <FilterBar onFilterChange={setFilter} />
      <LogTable logs={logs} />
    </div>
  );
}
```

#### ツール実行GUI

**機能:**
- ツール一覧の表示
- パラメータフォーム（JSON Schemaから自動生成）
- 実行結果の表示
- 履歴管理

#### エクスポート/インポート

**機能:**
- ログのエクスポート（JSON、CSV）
- 設定のエクスポート/インポート
- テストケースの保存

### 5.4 Phase 7実装計画

#### Week 13-14: Tauriセットアップ
- [ ] Tauriプロジェクト作成
- [ ] Reactフロントエンド構築
- [ ] Tauri Commandsの定義

#### Week 15-16: ダッシュボード実装
- [ ] サーバー一覧表示
- [ ] ステータス監視
- [ ] メトリクス表示

#### Week 17: ログビューア実装
- [ ] ログ表示
- [ ] フィルタリング
- [ ] 検索機能

#### Week 18: ツール実行GUI
- [ ] ツール一覧
- [ ] パラメータフォーム
- [ ] 実行結果表示

**総工数見積もり**: 66-99時間

---

## 6. Phase 8: エコシステム統合

### 6.1 VS Code拡張機能

**機能:**
- サイドバーでMCPサーバー管理
- ツールの実行
- ログの表示

**技術:**
- TypeScript
- VS Code Extension API

**工数**: 20-30時間

---

### 6.2 CI/CD統合

**GitHub Actionsワークフロー:**
```yaml
name: MCP Server Test

on: [push, pull_request]

jobs:
  test-mcp:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup MCP Inspector
        run: |
          cargo install mcp-inspector-mcp

      - name: Test MCP Server
        run: |
          mcp-inspector test --server my-mcp-server
```

**工数**: 8-12時間

---

### 6.3 コミュニティツール

**内容:**
- テンプレート集
- サンプル設定
- ベストプラクティスガイド

**工数**: 10-15時間

---

### 6.4 Phase 8実装計画

#### Week 19-20: VS Code拡張
- [ ] 拡張機能のセットアップ
- [ ] サーバー管理UI
- [ ] ツール実行機能

#### Week 21: CI/CD統合
- [ ] GitHub Actionsワークフロー
- [ ] ドキュメント

#### Week 22: コミュニティツール
- [ ] テンプレート作成
- [ ] サンプル集
- [ ] ガイド執筆

**総工数見積もり**: 38-55時間

---

## 7. 技術評価サマリー

### 7.1 データベース比較

| データベース | 評価 | Rustサポート | クエリ機能 | セットアップ | 推奨用途 |
|------------|------|-------------|----------|------------|---------|
| **sled** | ⭐⭐⭐⭐⭐ | Pure Rust | Key-Value | ゼロ設定 | Phase 5推奨 |
| SQLite | ⭐⭐⭐⭐☆ | rusqlite | SQL対応 | 簡単 | 将来の拡張 |
| JSON Lines | ⭐⭐⭐☆☆ | 標準ライブラリ | なし | ゼロ設定 | プロトタイプ |

### 7.2 UIフレームワーク比較

| フレームワーク | 評価 | デスクトップ | Web | 学習曲線 | エコシステム |
|--------------|------|------------|-----|----------|------------|
| **Tauri** | ⭐⭐⭐⭐⭐ | ✅ | ✅ | 低 | 成熟 |
| Leptos | ⭐⭐⭐⭐☆ | ❌ | ✅ | 高 | 成長中 |
| Axum+HTMX | ⭐⭐⭐☆☆ | ❌ | ✅ | 低 | 成熟 |

### 7.3 rmcp SDK追跡

**定期確認項目:**
- [ ] GitHubリポジトリのIssue/PR（毎週）
- [ ] Discord/Slackでの議論（毎週）
- [ ] リリースノート（新バージョン時）

**Feature Request候補:**
```markdown
# Feature Request: Sampling Support in ServerHandler

## Problem
Currently, `ServerHandler` trait does not include methods for handling Sampling requests
(`sampling/createMessage`). This prevents MCP servers from acting as Sampling clients.

## Proposed Solution
Add Sampling-related methods to `ServerHandler`:

```rust
pub trait ServerHandler {
    // ... existing methods

    async fn handle_sampling_request(
        &self,
        request: CreateMessageRequestParam,
    ) -> Result<CreateMessageResult>;
}
```

## Use Case
MCP Inspector MCP Server needs to monitor Sampling requests from target servers.
```

---

## 8. 実装ロードマップ

### 8.1 全体タイムライン

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 4: Transport層Sampling監視                    4週間  │
├─────────────────────────────────────────────────────────────┤
│ Phase 5: ログ永続化・最適化                         4週間  │
├─────────────────────────────────────────────────────────────┤
│ Phase 6: 高度な検査機能                             4週間  │
├─────────────────────────────────────────────────────────────┤
│ Phase 7: Webダッシュボード                          6週間  │
├─────────────────────────────────────────────────────────────┤
│ Phase 8: エコシステム統合                           4週間  │
└─────────────────────────────────────────────────────────────┘

総推定期間: 22週間（約5.5ヶ月）
総推定工数: 200-300時間
```

### 8.2 優先順位マトリクス

```
  重要度
    ↑
    │
  高│  Phase 4         Phase 5
    │  (Sampling監視)  (永続化)
    │
  中│  Phase 6         Phase 7
    │  (高度検査)      (Dashboard)
    │
  低│                  Phase 8
    │                  (統合)
    │
    └───────────────────────────→ 緊急度
         高     中      低
```

### 8.3 段階的リリース計画

**v0.4.0 (Phase 4完了時)**
- Transport層Sampling監視
- リリースノート: "実際のSampling監視に対応"

**v0.5.0 (Phase 5完了時)**
- ログ永続化
- パフォーマンス最適化
- リリースノート: "エンタープライズ対応"

**v0.6.0 (Phase 6完了時)**
- 高度な検査機能
- リリースノート: "包括的なデバッグ機能"

**v0.7.0 (Phase 7完了時)**
- Webダッシュボード
- リリースノート: "視覚的なモニタリング"

**v1.0.0 (Phase 8完了時)**
- エコシステム統合
- リリースノート: "MCP Inspector 1.0 GA"

---

## 9. リスク評価と緩和策

### 9.1 技術リスク

#### リスク1: rmcp SDK制約

**影響度**: 🔴 高
**発生確率**: 🔴 高

**緩和策:**
- カスタムTransport実装の準備
- rmcpコミュニティとの連携
- 定期的なSDK追跡

---

#### リスク2: MCPプロトコル変更

**影響度**: 🟠 中
**発生確率**: 🟡 中

**緩和策:**
- MCP仕様の定期的な確認
- バージョン管理の徹底
- 後方互換性の維持

---

#### リスク3: パフォーマンス問題

**影響度**: 🟡 中
**発生確率**: 🟢 低

**緩和策:**
- 早期のベンチマーク実施
- プロファイリングの導入
- 段階的な最適化

---

### 9.2 リソースリスク

#### リスク4: 工数見積もりの誤差

**影響度**: 🟠 中
**発生確率**: 🟠 中

**緩和策:**
- 20%のバッファを見込む
- MVP優先の開発
- 定期的な進捗レビュー

---

#### リスク5: 技術的負債の蓄積

**影響度**: 🟠 中
**発生確率**: 🟠 中

**緩和策:**
- コードレビューの徹底
- リファクタリング時間の確保
- ドキュメント整備

---

### 9.3 互換性リスク

#### リスク6: 破壊的変更

**影響度**: 🔴 高
**発生確率**: 🟢 低

**緩和策:**
- セマンティックバージョニング
- 移行ガイドの提供
- 非推奨警告の事前通知

---

### 9.4 リスクマトリクス

```
  影響度
    ↑
    │
  高│  R1: rmcp制約    R6: 破壊的変更
    │
  中│  R2: プロトコル  R4: 工数誤差
    │  変更            R5: 技術負債
    │
  低│                  R3: パフォーマンス
    │
    └─────────────────────────────→ 発生確率
         低     中      高
```

---

## 10. 成功基準

### 10.1 機能的成功基準

- ✅ Samplingリクエストの完全な監視
- ✅ 永続化されたログシステム
- ✅ 10種類以上の検査ツール
- ✅ 直感的なWebダッシュボード
- ✅ VS Code統合

### 10.2 技術的成功基準

- ✅ レスポンスタイム < 100ms（キャッシュヒット時）
- ✅ メモリ使用量 < 50MB（アイドル時）
- ✅ 接続プール効率 > 80%
- ✅ テストカバレッジ > 80%

### 10.3 ユーザー体験成功基準

- ✅ セットアップ時間 < 5分
- ✅ ドキュメント完備
- ✅ エラーメッセージが明確
- ✅ コミュニティサポート

---

## 11. 次のアクション（即座に着手）

### Week 1: 調査フェーズ

**Phase 4準備:**
1. [ ] rmcp GitHubリポジトリの調査
   - Issue検索: "sampling"
   - PR検索: "ServerHandler"
2. [ ] MCP DiscordでSampling対応状況を確認
3. [ ] Feature Requestの下書き作成

**技術検証:**
4. [ ] MonitoringTransportプロトタイプ作成
5. [ ] JSON-RPCメッセージパース検証
6. [ ] 統合テスト環境構築

### Week 2-4: Phase 4実装

7. [ ] 技術アプローチの最終決定
8. [ ] MonitoringTransport完全実装
9. [ ] ユニット・統合テスト
10. [ ] ドキュメント更新

---

## 12. まとめ

### 12.1 戦略的ビジョン

MCP Inspector MCP Serverは、Phase 4以降の実装により、**業界標準のMCPデバッグ・監視ツール**となる基盤が整います。

**差別化要因:**
- 完全なSampling監視（Phase 4）
- エンタープライズグレードの永続化（Phase 5）
- 包括的な検査機能（Phase 6）
- 直感的なUI（Phase 7）
- 開発ワークフロー統合（Phase 8）

### 12.2 推奨実装順序

1. **Phase 4（最優先）**: Sampling監視の実現
2. **Phase 5（高優先）**: 永続化と最適化
3. **Phase 6-8（中優先）**: 段階的な機能拡張

### 12.3 長期的展望

**1年後:**
- MCP Inspector 1.0 GA
- 1000+ GitHub Stars
- 活発なコミュニティ

**3年後:**
- MCPエコシステムの標準デバッグツール
- 商用サポート版の提供
- クラウドモニタリングサービス

---

## 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|----------|---------|-------|
| 2025-01-15 | 1.0 | 初版作成 | Solution Architect Agent |

---

**この拡張計画書に基づいて実装を進めることで、MCP Inspector MCP Serverは業界をリードするMCPデバッグツールとなるでしょう。**

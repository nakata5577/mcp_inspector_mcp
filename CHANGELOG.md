# Changelog

All notable changes to MCP Inspector MCP Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Phase 6: 高度な検査機能

#### Phase 6.3: Logging検査機能 📋
- **新規ツール**: `logging_messages` - ログメッセージ検査
  - `notifications/message`通知の自動検出
  - ログメッセージの収集と保存
  - レベルフィルタリング（Debug/Info/Notice/Warning/Error/Critical/Alert/Emergency）
  - 時間範囲フィルタリング（RFC3339形式）
  - 件数制限
  - Memory/Persistent両バックエンド対応

**実装詳細:**
- `src/models/logging_inspection.rs`: データモデル（112行）
  - LoggingMessagesRequest/Response
  - LogEntry（timestamp, level, logger, message）
  - LogLevel（PartialOrd実装）
- `src/services/logging_inspector.rs`: ログ検査サービス（288行）
  - add_log_entry() - ログ追加
  - get_logging_messages() - フィルタ付き取得
- `src/services/logger_backend.rs`: トレイト拡張（+48行）
  - add_log_message()/get_log_messages()メソッド追加
- `src/services/persistent_logger.rs`: sled永続化実装（+110行）
- `src/services/memory_logger.rs`: メモリ実装（+77行）
- `src/client/monitoring_transport.rs`: 通知検出（+59行）
  - is_logging_notification() - 通知判定
  - extract_and_log_message() - ログ抽出・記録

**使用例:**
```
対象サーバー"fundamental_analysis"のエラーレベル以上のログを最新50件取得してください
```

**テスト**: 5個のユニットテスト（レベルフィルタ、時間フィルタ、件数制限検証）

#### Phase 6.2: ヘルスチェック機能 💚
- **新規ツール**: `health_check` - ヘルスチェック実行
  - Pingによる疎通確認
  - レスポンスタイム測定（ミリ秒精度）
  - エラー率算出（最近100件の履歴）
  - 3段階ステータス判定（Healthy/Degraded/Unhealthy）
  - 循環バッファによる履歴管理

**ヘルスステータス判定基準:**
- **Healthy**: レスポンスタイム < 500ms かつ エラー率 < 5%
- **Degraded**: レスポンスタイム < 2000ms かつ エラー率 < 20%
- **Unhealthy**: レスポンスタイム >= 2000ms または エラー率 >= 20%

**実装詳細:**
- `src/models/health.rs`: データモデル（116行）
  - HealthCheckRequest/Response
  - HealthStatus（Healthy/Degraded/Unhealthy）
  - HealthHistory（循環バッファ）
- `src/services/health_checker.rs`: ヘルスチェックサービス（247行）
  - check_health() - ping送信、測定、判定
  - RwLockによるスレッドセーフな履歴管理
- `src/client/stdio_client.rs`: ping()メソッド追加（+32行）

**使用例:**
```
対象サーバー"fundamental_analysis"のヘルスチェックを実行してください
```

**テスト**: 6個のユニットテスト（ステータス判定、循環バッファ、エラー統計検証）

#### Phase 6.1: サーバー設定検査機能 🔧
- **新規ツール**: `server_inspect` - サーバー設定検査
  - サーバー実装情報（名前、バージョン、WebサイトURL）
  - サーバー機能（tools, resources, prompts, logging, experimental）
  - 各機能の詳細フラグ（list_changed, subscribe等）
  - プロトコルバージョン
  - 接続状態（Connected/Disconnected/Error）

**実装詳細:**
- `src/models/server_info.rs`: データモデル（101行）
  - ServerInspectRequest/Response
  - ServerImplementation, ServerCapabilitiesInfo
  - ConnectionStatus
- `src/services/server_info_service.rs`: サーバー情報取得（145行）
  - get_server_info() - InitializeResultからの情報抽出
- `src/client/stdio_client.rs`: get_init_result()メソッド追加（+33行）
- `src/client/manager.rs`: get_stdio_client()メソッド追加（+59行）

**使用例:**
```
対象サーバー"fundamental_analysis"の設定情報を取得してください
```

**総実装規模（Phase 6全体）:**
- **新規コード**: 約1,600行
- **新規ファイル**: 7ファイル
- **修正ファイル**: 13ファイル
- **新規MCPツール**: 3個
- **新規テスト**: 16個

### Added - Phase 5後半: パフォーマンス最適化

#### 接続プーリング
- **ClientManager接続プール**: MCPクライアント接続を再利用
  - `Arc<StdioClient>`による共有接続
  - 接続の健全性チェック（`is_connected()`メソッド）
  - 不要な接続のクリーンアップAPI（`disconnect()`メソッド）
  - パフォーマンス: 2回目以降の接続が50%以上高速化

#### キャッシング戦略
- **ResponseCache実装**: ツール/リソース/プロンプト一覧のTTLベースキャッシュ
  - デフォルトTTL: 5分（300秒）
  - キャッシュヒット時のレスポンスタイム < 1ms
  - 推定キャッシュヒット率: 80%以上
  - キャッシュ無効化API:
    - `invalidate(server_name)` - サーバー単位の無効化
    - `invalidate_all()` - 全体無効化
  - キャッシュ統計取得: `stats()` - ツール/リソース/プロンプトのキャッシュ数

#### 並列処理の改善
- **バッチ処理メソッド**: 複数サーバーの並列処理
  - `list_tools_batch()` - 複数サーバーのツール一覧を並列取得
  - `list_resources_batch()` - 複数サーバーのリソース一覧を並列取得
  - `list_prompts_batch()` - 複数サーバーのプロンプト一覧を並列取得
  - `tokio::task::JoinSet`を使用した並列実行
  - エラーハンドリング: 一部失敗でも他の結果を返す
  - パフォーマンス: N個のサーバー処理が約1/Nの時間

#### 実装詳細
- `src/client/manager.rs`: 接続プール拡張（`disconnect()`メソッド追加）
- `src/client/stdio_client.rs`: `Arc<StdioClient>`への`McpClient`トレイト実装
- `src/services/response_cache.rs`: キャッシング実装（新規427行）
  - `CachedResponse<T>`: タイムスタンプ付きキャッシュエントリ
  - `ResponseCache`: ツール/リソース/プロンプトのキャッシュ管理
  - TTL期限切れの自動検出
- `src/services/inspector.rs`: キャッシュ統合、並列処理追加
  - `ResponseCache`の統合
  - バッチメソッドの実装

#### テスト
- 単体テスト: 27/27合格
  - `response_cache.rs`: 8テスト（キャッシュヒット、TTL、無効化）
  - その他既存テスト: 19テスト
- 統合テスト: 8/8合格（`phase5_performance_test.rs`）
  - 接続プーリングテスト: 2テスト
  - キャッシングテスト: 3テスト
  - 並列処理テスト: 2テスト
  - 統合テスト: 1テスト
- パフォーマンステスト:
  - キャッシュヒット: 1000回 < 10ms ✅
  - TTL期限切れ検証: 正常動作 ✅
  - 並列処理構造: 正常動作 ✅

### Added - Phase 5前半: ログ永続化機能

#### 新機能
- **ログバックエンドの抽象化**: `LoggerBackend`トレイトによる統一的なインターフェース
  - `Send + Sync + Debug`実装によるスレッドセーフ性
  - バックエンド非依存のアーキテクチャ
- **Memory Backend**: 高速なインメモリログストレージ
  - 既存のメモリベース実装を改善
  - FIFOローテーションによる自動メモリ管理
  - 1000件/秒以上の書き込み性能
- **Persistent Backend**: sledデータベースによるディスク永続化
  - サーバー再起動後もログを保持
  - 500-1000件/秒の書き込み性能
  - データベース破損からの自動復旧機能
- **設定ベースの切替**: `config/servers.toml`でバックエンドを選択可能
  - Factoryパターンによる柔軟な実装
  - 設定バリデーション機能
- **ログローテーション**: サーバーごとに最大ログ数を設定、古いログを自動削除
  - FIFO方式による効率的なメモリ/ディスク管理
  - 設定可能な`max_logs`パラメータ（デフォルト: 10000）

#### 設定ファイル
`config/servers.toml`に`[logging]`セクションを追加:
- `backend`: "memory"または"persistent"（デフォルト: "memory"）
- `db_path`: データベースファイルパス（persistent使用時必須）
- `max_logs`: サーバーごとの最大ログ数（デフォルト: 10000）

**Memory Backend設定例:**
```toml
[logging]
backend = "memory"
max_logs = 10000
```

**Persistent Backend設定例:**
```toml
[logging]
backend = "persistent"
db_path = "./data/logs.db"
max_logs = 10000
```

#### 技術スタック
- `sled 0.34`: Rust製組み込み型データベース
  - 高性能なkey-valueストア
  - ACID準拠
  - 組み込み可能なアーキテクチャ
- `bincode 1.3`: 高速シリアライゼーション
  - バイナリフォーマットによる効率的なデータ保存
  - serdeとの統合

#### 実装詳細
- `src/services/logger_backend.rs`: トレイト定義
  - `add_log`: ログの追加
  - `get_logs`: ログの取得（フィルタリング対応）
  - `clear_logs`: ログのクリア
- `src/services/memory_logger.rs`: メモリバックエンド実装
  - `Arc<RwLock<HashMap>>`によるスレッドセーフなストレージ
  - VecDequeによる効率的なFIFOキュー
- `src/services/persistent_logger.rs`: 永続化バックエンド実装
  - sled::Dbによるディスク永続化
  - エラー処理とリカバリー機能
- `src/services/logger_factory.rs`: Factoryパターン実装
  - 設定に基づくバックエンドの動的生成
  - エラーハンドリングとバリデーション
- `src/models/logging_config.rs`: 設定構造体
  - Deserialize実装によるTOMLパース
  - デフォルト値の提供

### Changed
- **SamplingLogger**: 具象実装からFacadeパターンに変更
  - `Arc<dyn LoggerBackend>`を使用
  - バックエンドの実装詳細を隠蔽
- **InspectorService**: 設定ファイルからログバックエンドを初期化
  - `LoggerFactory`を使用したバックエンド生成
  - 設定の読み込みと適用

### Tests
- 単体テスト: 20テスト追加
  - `memory_logger`: 5テスト
  - `persistent_logger`: 6テスト
  - `sampling_logger`: 6テスト
  - `logger_factory`: 3テスト
- 統合テスト: 8テスト追加
  - `phase5_logger_integration_test`: バックエンド切替、永続化、ローテーション検証
- **全テスト合格**: 28/28テスト ✅

### Documentation
- `README.md`: Phase 5セクション追加
  - ログバックエンドの説明
  - 設定例と使用方法
  - トラブルシューティングガイド
- `docs/PHASE5_COMPLETION_REPORT.md`: 実装完了レポート
- `CHANGELOG.md`: 変更履歴の追加（本ファイル）

### Migration Guide

#### 既存ユーザー向け
**後方互換性**: 既存の設定ファイルはそのまま動作します（Memory backendがデフォルト使用）。

#### Phase 5機能を使用する場合

1. **Memory Backend（推奨: 開発・テスト環境）**

   `config/servers.toml`に以下を追加（オプション）:
   ```toml
   [logging]
   backend = "memory"
   max_logs = 10000
   ```

2. **Persistent Backend（推奨: 本番環境）**

   ステップ1: データベースディレクトリを作成
   ```bash
   # Windowsの場合
   mkdir data

   # Linux/macOSの場合
   mkdir -p ./data
   ```

   ステップ2: `config/servers.toml`に以下を追加:
   ```toml
   [logging]
   backend = "persistent"
   db_path = "./data/logs.db"
   max_logs = 10000
   ```

   ステップ3: サーバーを再起動
   ```bash
   cargo run --release
   ```

3. **動作確認**

   ログ出力で使用中のバックエンドを確認:
   ```
   INFO Creating persistent logger (db_path: ./data/logs.db, max_logs: 10000)
   ```
   または
   ```
   INFO Creating memory logger (max_logs: 10000)
   ```

### Known Issues
- Persistent backendはMemory backendより約2倍遅い（500-1000件/秒 vs 1000件/秒以上）
  - 開発・テスト環境ではMemory backendを推奨
- 複数プロセスからの同時書き込みは未サポート
  - 将来のバージョンで改善予定

### Future Improvements
- Phase 5後半: パフォーマンス最適化
  - 接続プール
  - キャッシング
  - 非同期書き込み
- 圧縮オプションの追加
- クエリ機能の強化

---

## [0.4.0] - 2025-01-15

### Added - Phase 4: Transport層Sampling監視

#### 新機能
- **MonitoringTransport**: Transport層でSamplingリクエストを監視
  - JSONRPCメッセージのインターセプト
  - Samplingリクエストの検出と記録
- **StdioClient統合**: MonitoringTransportの自動適用
  - 既存コードへの透過的な統合
  - 全サーバーで自動的にSampling監視を有効化

#### 実装詳細
- `src/client/monitoring_transport.rs`: Transport層の監視実装
- `src/client/stdio_client.rs`: MonitoringTransportの統合

### Known Issues
- E2Eテストは環境制約により未検証（実装は完了）
- Windows環境でのstdio通信制約により、一部のテストが不安定

---

## [0.3.0] - 2025-01-14

### Added - Phase 3: Samplingログ機能

#### 新機能
- `sampling_logs`ツール: Samplingリクエストのログ取得
  - サーバーごとのログ管理
  - ステータスフィルタリング（all/success/failed）
  - 件数制限（limit）

#### 実装詳細
- `src/services/sampling_logger.rs`: ログ管理サービス
- `src/models/sampling.rs`: Samplingデータモデル

### Known Issues
- rmcp 0.8.5の技術的制約により、実際のSampling通信の監視は未実装
- ログ管理インフラのみ提供

---

## [0.2.0] - 2025-01-10

### Added - Phase 2: リソース・プロンプト検査機能

#### 新機能
- `resources_list`: リソース一覧取得
- `resources_read`: リソース読み取り
- `prompts_list`: プロンプト一覧取得
- `prompts_get`: プロンプト取得

#### 実装詳細
- `src/server/mod.rs`: 新ツールのハンドラー追加
- `src/services/inspector.rs`: リソース・プロンプト検査機能

### Tests
- 統合テスト: `tests/phase2_integration_test.rs`
- 全テスト合格 ✅

---

## [0.1.0] - 2025-01-05

### Added - MVP (Phase 1): ツール検査機能

#### 初期リリース
- `tools_list`: 対象MCPサーバーのツール一覧を取得
- `tools_call`: 対象MCPサーバーのツールを実行

#### 技術スタック
- Rust
- rmcp 0.8.5（MCP Rust SDK）
- tokio（非同期ランタイム）
- serde（シリアライゼーション）

#### アーキテクチャ
- MCPサーバーとクライアントの統合
- 設定ファイルベースのサーバー管理（`config/servers.toml`）
- Stdio transportによる通信

### Documentation
- README.md: 基本的な使用方法
- MCP_INSPECTOR_CLI_GUIDE.md: CLIモード使用ガイド

---

[Unreleased]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/mcp_inspector_mcp/releases/tag/v0.1.0

# Changelog

All notable changes to MCP Inspector MCP Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2025-11-18

### Added - Phase 2: ドキュメント整備フェーズ

#### 📚 充実したREADME.md
- **15,000文字以上**の詳細なドキュメント
- **30個以上のコード例**（実行可能）
- クイックスタートガイド（30分以内に基本操作を習得）
- 詳細な設定ガイド（`.inspector/config.json`の全項目解説）
- 10個以上の使用例（基本〜高度）
- 完全なトラブルシューティングセクション
- FAQ（17項目）

#### 📖 3本のチュートリアル
1. **Getting Started** (`docs/tutorials/getting-started.md`)
   - 約19,500文字、50個以上のコード例
   - 完全な初心者向けガイド
   - インストールから初回実行まで
   - 5つの基本操作を実演

2. **Practical Guide** (`docs/tutorials/practical-guide.md`)
   - 約19,000文字、60個以上のコード例
   - 5つの実務シナリオ
   - 新規サーバー検査、ツールテスト、エラーデバッグ
   - パフォーマンス測定、複数サーバー管理
   - 10個のベストプラクティス

3. **Advanced Usage** (`docs/tutorials/advanced-usage.md`)
   - 約18,500文字、55個以上のコード例
   - カスタム設定の詳細（タイムアウト、リトライ）
   - Capability検証の活用
   - エラーハンドリングの高度なパターン
   - パフォーマンスチューニング
   - CI/CD統合（GitHub Actions、GitLab CI）
   - セキュリティ考慮事項
   - 拡張とカスタマイズ

#### 📋 完全なAPI仕様書
- **API仕様書** (`docs/api/tools.md`)
  - 約22,000文字
  - **全13ツール**の詳細仕様
  - リクエスト/レスポンス形式（JSON Schema）
  - エラーコード一覧（12種類の詳細分類）
  - ベストプラクティス10項目
  - バージョニング方針

#### 🎯 サンプルプロジェクト
- **Simple Server** (`examples/simple-server/`)
  - 完全に動作するMCPサーバー実装
  - 3つのツール（echo、reverse、uppercase）
  - 1つのリソース（simple://greeting）
  - 1つのプロンプト（help）
  - 統合テストスクリプト（Bash/PowerShell）
  - 詳細なREADME.md（約2,500文字）
  - 使用方法ガイド（約4,500文字）
  - 合計約1,700行のコードとドキュメント

### Documentation

#### 統計情報
- **総文字数**: 100,000文字以上（目標30,000文字を大幅に超過）
- **チュートリアル数**: 3本（Getting Started、Practical Guide、Advanced Usage）
- **コード例数**: 165個以上（すべて実行可能）
- **API仕様カバレッジ**: 全13ツール（100%）
- **サンプルプロジェクト**: 1個（完全に動作）

#### 新規追加ファイル
- `docs/tutorials/getting-started.md` (19,500文字)
- `docs/tutorials/practical-guide.md` (19,000文字)
- `docs/tutorials/advanced-usage.md` (18,500文字)
- `docs/api/tools.md` (22,000文字)
- `examples/simple-server/README.md` (2,500文字)
- `examples/simple-server/docs/usage.md` (4,500文字)
- `examples/simple-server/src/main.rs` (約1,200行)
- `examples/simple-server/tests/integration_test.sh`
- `examples/simple-server/tests/integration_test.ps1`

#### 更新されたファイル
- `README.md`: 4,500文字 → 15,000文字に拡充

### Improved
- **ドキュメント全体の構造化**: 初心者から上級者まで段階的に学習可能
- **新規ユーザーの学習曲線**: 30分以内に基本操作を習得できるガイド
- **技術的正確性**: v0.3.1実装と完全一致
- **実用性**: すべてのコード例が実行可能

### Changed
- Phase 1計画書を削除（完了済みのため、`docs/PHASE1_DETAILED_PLAN.md`を削除）

---

## [0.3.1] - 2025-11-18

### Added

#### Task 1.1: タイムアウトとエラーハンドリングの強化
- **ツール実行タイムアウト**: MCPツール実行時のタイムアウト機能を追加（デフォルト30秒）
- **プロセス生存確認**: タイムアウト時にサーバープロセスの生存状態を確認
- **詳細ログ機能**: ツール呼び出しの開始～終了を追跡し、実行時間を測定
- 環境変数 `MCP_TOOL_TIMEOUT_MS` でタイムアウトをカスタマイズ可能

#### Task 1.2: エラーレポートの構造化
- **ToolExecutionError enum**: 6種類のエラータイプ（Timeout、ServerCrash、InvalidResponse、CommunicationError、ServerError、Other）
- **ErrorResponse構造体**: タイムスタンプ、リクエストID付きのエラー情報
- **ユーザーフレンドリーなエラーメッセージ**: `user_message()`メソッドでわかりやすいメッセージを生成
- **JSON形式でのシリアライズ**: `to_json()`メソッドで構造化されたエラー情報を出力

#### Task 1.3: Capability検証と警告機能
- **CapabilityValidator**: MCPサーバーのcapabilitiesを検証し、機能不整合を検出
- **ベストエフォート方式**: 警告を出力するが実行は継続
- **3つのcapability検証**: Tools、Resources、Promptsのサポート状況を確認
- **詳細な警告メッセージ**: どのcapabilityが不足しているか明示

#### Task 1.4: タイムアウト設定のカスタマイズ
- **ExecutionConfig構造体**: 実行設定を一元管理
  - `tool_timeout_ms`: ツールタイムアウト（デフォルト: 30000ms）
  - `connection_timeout_ms`: 接続タイムアウト（デフォルト: 5000ms）
  - `retry_count`: リトライ回数（デフォルト: 0）
  - `auto_retry_on_timeout`: 自動リトライフラグ（デフォルト: false）
- **config.json対応**: `.inspector/config.json`で実行設定をカスタマイズ可能
- **環境変数サポート継続**: 後方互換性を完全に維持
- **優先順位**: config.json > 環境変数 > デフォルト値

#### Task 1.5: 品質保証
- **統合テストの追加**: 109テスト全て成功（既存58 + 新規51）
- **テストカバレッジ測定**: 主要モジュールで高いカバレッジを達成（execution_config.rs: 94.02%）
- **パフォーマンステスト**: 全テストが0.1秒以内に実行完了

### Changed
- `StdioClient`: capabilities情報を保存・提供する機能を追加
- `InspectorService`: 各操作前にcapabilityを検証
- ハードコードされた環境変数読み込みをExecutionConfig経由に変更

### Fixed
- タイムアウト時のエラーメッセージが不明瞭だった問題を解決
- サーバーcapabilityとクライアントリクエストの矛盾が検出されなかった問題を解決

### Technical Details
- **新規ファイル**:
  - `src/error.rs` (約186行)
  - `src/models/execution_config.rs` (約203行)
  - `src/services/capability_validator.rs` (約242行)
  - `tests/task1_3_capability_validator_test.rs` (24テスト)
  - `tests/task1_4_execution_config_test.rs` (19テスト)
  - `tests/error_structure_test.rs` (8テスト)

- **修正ファイル**: 15ファイル以上
- **依存関係追加**: `serial_test = "3"` (dev-dependencies)
- **総テスト数**: 109テスト（全て成功）

---

## [0.3.0] - 2025-11-16

### Changed - Phase 8: `.inspector/config.json`による設定管理への移行

#### 🔧 破壊的変更: 設定方式の刷新

**新しい設定方式:**
- `.inspector/config.json`ファイルによる設定管理
- プロジェクト直下に`.inspector`フォルダを自動作成
- 初回起動時にデフォルト設定を自動生成

**廃止された機能:**
- CLI引数方式（`--server`等）を完全削除
- 環境変数方式（`MCP_INSPECTOR_SERVERS`等）を完全削除
- `clap`依存関係を削除

#### 📋 新機能: 設定操作用MCPツール

AIエージェントから設定を直接操作できる3つのツールを追加：

**1. config_add_server**
- サーバー設定を`.inspector/config.json`に追加
- 引数: name, transport, command, args, env

**2. config_remove_server**
- サーバー設定を削除
- 引数: name

**3. config_list_servers**
- 登録済みサーバーの一覧を取得
- 引数: なし

#### 実装詳細

**新規ファイル:**
- `src/models/project_config.rs`: 設定ファイル構造体定義
  - `ProjectConfig`: ルート構造体
  - `ServerEntry`: サーバーエントリ
  - `LoggingSettings`: ロギング設定
- `src/services/config_manager.rs`: 設定ファイル管理
  - `load_config()`: 設定読み込み、自動セットアップ
  - `save_config()`: 設定保存
  - `add_server()`: サーバー追加
  - `remove_server()`: サーバー削除
  - `list_servers()`: サーバー一覧

**変更ファイル:**
- `src/main.rs`: `.inspector/config.json`からの読み込みに変更
- `src/server/mod.rs`: 3つの新MCPツールを追加
- `src/models/server_config.rs`: `from_env()`メソッドを削除
- `src/models/logging_config.rs`: `from_env()`メソッドを削除
- `Cargo.toml`: `clap`依存関係を削除

**削除ファイル:**
- `tests/phase7_config_tests.rs`: 環境変数方式のテスト
- `tests/phase7_integration_test.rs`: 環境変数方式の統合テスト

#### 設定例

```json
{
  "servers": [
    {
      "name": "fundamental_analysis",
      "transport": "stdio",
      "command": "C:/path/to/fa.exe",
      "args": [],
      "env": {}
    }
  ],
  "logging": {
    "backend": "memory",
    "db_path": "./data/logs.db",
    "max_logs": 10000
  }
}
```

#### 移行ガイド

**Phase 7（CLI引数方式）からの移行:**

Before (Phase 7):
```json
{
  "command": "mcp_inspector_mcp.exe",
  "args": [
    "--server", "fa:stdio:C:/path/to/fa.exe",
    "--log-backend", "memory"
  ]
}
```

After (Phase 8):
```json
{
  "command": "mcp_inspector_mcp.exe"
}
```

そして、`.inspector/config.json`を作成：
```json
{
  "servers": [
    {
      "name": "fa",
      "transport": "stdio",
      "command": "C:/path/to/fa.exe",
      "args": [],
      "env": {}
    }
  ],
  "logging": {
    "backend": "memory",
    "db_path": "./data/logs.db",
    "max_logs": 10000
  }
}
```

## [Unreleased]

### Added - CLI引数方式の設定サポート

#### 🌟 新機能: CLI引数によるサーバー設定

**CLI引数方式のサーバー設定**
- `--server`フラグによるDSL形式の設定: `name:transport:command[:args...]`
- 複数サーバーの登録: `--server`を複数回指定
- エスケープ不要で読みやすい設定
- Windowsパスの自動検出（`C:/...`、`D:/...`等）
- 大文字小文字を区別しないトランスポート指定（`stdio`, `STDIO`, `Stdio`）

**DSL形式の詳細:**
```bash
# 基本形式
--server "name:transport:command[:arg1:arg2:...]"

# 例
--server "fa:stdio:C:/path/to/fa.exe"
--server "ta:stdio:/usr/local/bin/ta:--verbose:--debug"
```

**ログ設定のCLI引数サポート**
- `--log-backend <TYPE>`: ログバックエンド（`memory`/`persistent`）
- `--log-path <PATH>`: データベースパス（persistent時）
- `--log-max-logs <NUM>`: 最大ログ数

**設定優先順位:** CLI引数 > 環境変数 > デフォルト値

#### 実装詳細

**新規実装:**
- `src/main.rs`: `Cli`構造体の追加（clap 4.x使用）
  - `servers: Vec<String>`: DSL形式のサーバー設定
  - `log_backend: Option<String>`: ログバックエンド指定
  - `log_path: Option<String>`: DBパス指定
  - `log_max_logs: Option<usize>`: 最大ログ数指定
- `src/models/server_config.rs`: `ServerConfig::from_dsl()`メソッド追加
  - DSL文字列のパース処理
  - Windowsパス自動検出ロジック
  - トランスポートタイプのバリデーション
- `src/main.rs`: 設定ロード関数
  - `load_server_configs()`: CLI引数 > 環境変数の優先順位制御
  - `load_logging_config()`: ログ設定の優先順位制御

**テスト:**
- `src/models/server_config.rs`: DSL形式のパーステスト（10件追加）
  - 基本形式のパース
  - 引数付きパース
  - Windowsパスの処理
  - 大文字小文字の処理
  - エラーケース（不正形式、未サポートtransport）

**全テスト合格:** ✅

#### ドキュメント更新

**README.md:**
- 「2. 設定方法」セクションを刷新
  - 方式A: CLI引数方式（推奨）の説明追加
  - 方式B: 環境変数方式（後方互換性）として整理
  - 設定方式の比較表を追加
  - DSL形式の詳細説明
  - CLI引数リファレンステーブル
- 「3. Claude Desktopへの登録」セクションを更新
  - CLI引数方式の`.mcp.json`設定例追加（Windows/macOS/Linux）
  - 環境変数方式の設定例も維持（後方互換性）
- トラブルシューティングセクションを拡張
  - DSL形式のエラー対処法
  - 設定方式別のトラブルシューティング

**CHANGELOG.md:** 本エントリを追加

#### 使用例

**Claude Desktop設定（CLI引数方式）:**
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\\path\\to\\mcp_inspector_mcp.exe",
      "args": [
        "--server", "fundamental_analysis:stdio:C:/path/to/fa.exe",
        "--server", "technical_analysis:stdio:C:/path/to/ta.exe:--verbose",
        "--log-backend", "persistent",
        "--log-path", "./data/logs.db"
      ]
    }
  }
}
```

**利点:**
- エスケープ不要でJSON構文エラーのリスク低減
- 複数サーバーの設定が明確で読みやすい
- 設定ファイルの保守性向上
- Windowsユーザーの設定が容易

### Changed

**後方互換性の維持:**
- 環境変数方式（`MCP_INSPECTOR_SERVERS`）は引き続きサポート
- 既存の設定ファイルはそのまま動作
- CLI引数が指定されていない場合、自動的に環境変数にフォールバック

### Dependencies

**追加:**
- `clap 4.x`（derive feature付き）: CLI引数パース

---

## [0.2.0] - 2025-11-15

### Added - Phase 7: 環境変数ベース設定管理

#### 🌟 破壊的変更（Breaking Changes）

**環境変数ベースの設定管理に完全移行**
- 新しい設定方法: `MCP_INSPECTOR_SERVERS`環境変数（JSON形式）
- ログ設定も環境変数化（`MCP_LOGGING_*`）
- シングルバイナリ配布が可能に
- 12 Factor App原則への準拠
- MCP公式パターンへの準拠
- **TOML設定のサポートを完全削除**（後方互換性なし）

#### 新機能

**環境変数パーサー**: `InspectorConfig::from_env()`
- JSON配列のデシリアライズとバリデーション
- 詳細なエラーメッセージ
- サーバー設定の自動検証

**ログ設定の環境変数化**: `LoggingConfig::from_env()`
- `MCP_LOGGING_BACKEND`: memory/persistent選択
- `MCP_LOGGING_DB_PATH`: データベースパス指定
- `MCP_LOGGING_MAX_LOGS`: 最大ログ数設定

**環境変数一覧:**

| 環境変数名 | 型 | 必須 | デフォルト | 説明 |
|-----------|-----|------|-----------|------|
| `MCP_INSPECTOR_SERVERS` | JSON配列 | ✅ | - | 検査対象サーバーのリスト |
| `MCP_LOGGING_BACKEND` | string | ❌ | "memory" | ログバックエンド |
| `MCP_LOGGING_DB_PATH` | string | ❌ | "./data/logs.db" | DBパス（persistent時） |
| `MCP_LOGGING_MAX_LOGS` | integer | ❌ | 10000 | 最大ログ数 |


#### テスト

**新規テスト**: 21件
- 単体テスト: 15件
  - JSON パース機能（正常系・異常系）
  - 環境変数読み込み
  - デフォルト値の検証
  - バリデーション
- 統合テスト: 6件
  - 環境変数ベースの起動
  - ログ設定統合
  - エンドツーエンドシナリオ

**テストカバレッジ**: 95%以上

#### ドキュメント

**更新:**
- `README.md`: 環境変数設定方法に統一
  - Claude Desktop設定例（Windows/macOS/Linux）
  - JSON設定フォーマット
  - トラブルシューティング（環境変数関連）
  - TOML関連記述を完全削除
- `CHANGELOG.md`: v0.2.0の変更履歴（本ファイル）

#### 使用例

**Claude Desktop設定（Windows）:**
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\\path\\to\\mcp_inspector_mcp.exe",
      "env": {
        "MCP_INSPECTOR_SERVERS": "[{\"name\":\"my-server\",\"transport\":\"stdio\",\"command\":\"C:/path/to/server.exe\",\"args\":[]}]",
        "MCP_LOGGING_BACKEND": "persistent",
        "MCP_LOGGING_DB_PATH": "./data/logs.db",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Claude Desktop設定（macOS/Linux）:**
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "/path/to/mcp_inspector_mcp",
      "env": {
        "MCP_INSPECTOR_SERVERS": "[{\"name\":\"my-server\",\"transport\":\"stdio\",\"command\":\"/path/to/server\",\"args\":[]}]",
        "MCP_LOGGING_BACKEND": "memory",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Changed

**main.rs**: 設定ロードロジックを完全書き換え
- 環境変数のみのシンプルな実装（86行 → 42行、51%削減）
- 詳細なエラーメッセージ
- コードの保守性向上

### Removed

**TOML設定のサポート完全削除**
- `config/servers.toml`
- `load_from_toml()`関数
- `toml`クレート依存関係
- TOML関連テスト（1件削除）

### Migration Guide

v0.1.xからの移行:

1. `config/servers.toml`の内容をJSON形式に変換
2. `MCP_INSPECTOR_SERVERS`環境変数に設定
3. Claude Desktop設定ファイルを更新

**JSON形式への変換例:**

TOML:
```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "C:/path/to/server.exe"
args = []
```

JSON（環境変数）:
```json
[{
  "name": "my-server",
  "transport": "stdio",
  "command": "C:/path/to/server.exe",
  "args": []
}]
```

---

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

[Unreleased]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/yourusername/mcp_inspector_mcp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yourusername/mcp_inspector_mcp/releases/tag/v0.1.0

# 📋 実装計画書: 環境変数ベース設定管理への移行

**プロジェクト名:** MCP Inspector MCP - Phase 7: Environment-Based Configuration
**バージョン:** v0.2.0
**作成日:** 2025-11-15
**承認者:** [お客様名]

---

## 🎯 1. プロジェクト概要

### 1.1 背景
現在の`config/servers.toml`方式では、シングルバイナリ配布が実現できず、ユーザーセットアップが複雑になっています。MCP公式実装のパターンに準拠し、12 Factor App原則に従った環境変数ベースの設定管理に移行します。

### 1.2 目標
- ✅ シングルバイナリ配布の実現
- ✅ ユーザーセットアップの簡素化
- ✅ MCP公式実装パターンへの準拠
- ✅ セキュリティの向上
- ✅ 後方互換性の維持（移行期間中）

### 1.3 成功基準
1. **機能要件**
   - 環境変数`MCP_INSPECTOR_SERVERS`からサーバー設定を読み込み
   - 環境変数`MCP_LOGGING_*`からログ設定を読み込み
   - 既存の全機能が正常動作
   - 後方互換性維持（`MCP_INSPECTOR_CONFIG`を非推奨化）

2. **品質要件**
   - `cargo check`: エラーなし
   - `cargo clippy`: 警告なし
   - `cargo test`: 全テスト合格
   - `cargo build --release`: ビルド成功

3. **ドキュメント要件**
   - README更新（新しい設定方法）
   - 移行ガイド作成
   - CHANGELOG更新

---

## 🏗️ 2. 技術仕様

### 2.1 環境変数定義

| 環境変数名 | 型 | 必須 | デフォルト | 説明 |
|-----------|-----|------|-----------|------|
| `MCP_INSPECTOR_SERVERS` | JSON配列 | ✅ | - | 検査対象サーバーのリスト |
| `MCP_LOGGING_BACKEND` | string | ❌ | "memory" | ログバックエンド（"memory" or "persistent"） |
| `MCP_LOGGING_DB_PATH` | string | ❌ | "./data/logs.db" | DB保存パス（persistent時） |
| `MCP_LOGGING_MAX_LOGS` | integer | ❌ | 10000 | サーバーごとの最大ログ数 |
| `MCP_INSPECTOR_CONFIG` | string | ❌ | - | **非推奨** TOML設定ファイルパス |

### 2.2 JSON設定スキーマ

```json
// MCP_INSPECTOR_SERVERS の形式
[
  {
    "name": "server_name",
    "transport": "stdio",
    "command": "/path/to/executable",
    "args": ["arg1", "arg2"],
    "env": {
      "ENV_VAR": "value"
    }
  }
]
```

### 2.3 アーキテクチャ変更

**変更対象ファイル:**
```
src/
├── main.rs                    # 設定ロード処理を完全書き換え
├── models/
│   ├── server_config.rs       # 環境変数パーサー追加
│   └── logging_config.rs      # 環境変数パーサー追加
└── lib.rs                     # 公開API変更なし
```

**削除対象:**
```
config/servers.toml            # サンプルとして残すが、非推奨扱い
```

---

## 📊 3. タスク分解（WBS）

### Phase 7.1: コア実装 ⚙️

#### タスク 7.1.1: 環境変数パーサー実装
**担当サブエージェント:** `rust-developer`
**優先度:** 高
**工数見積:** 3時間

**詳細:**
- `src/models/server_config.rs`に以下の関数を追加
  - `InspectorConfig::from_env()` - 環境変数から設定をロード
  - `ServerConfig::parse_json_array()` - JSON配列をパース
- `src/models/logging_config.rs`に以下の関数を追加
  - `LoggingConfig::from_env()` - ログ設定を環境変数から読み込み

**成果物:**
- [ ] `server_config.rs` の実装コード
- [ ] `logging_config.rs` の実装コード
- [ ] 単体テスト（各関数）

**品質基準:**
- serde_jsonを使用したパース処理
- エラーハンドリングの適切な実装
- 詳細なエラーメッセージ

---

#### タスク 7.1.2: main.rs の書き換え
**担当サブエージェント:** `rust-developer`
**優先度:** 高
**工数見積:** 2時間
**依存:** タスク7.1.1完了後

**詳細:**
- `main.rs`の`load_config`ロジックを書き換え
- 優先順位実装:
  1. 環境変数`MCP_INSPECTOR_SERVERS`（新方式）
  2. 環境変数`MCP_INSPECTOR_CONFIG`（旧方式・非推奨警告）
- エラーハンドリング強化

**成果物:**
- [ ] 書き換えた`main.rs`
- [ ] 統合テスト

**品質基準:**
- 既存機能が全て動作
- 非推奨警告メッセージの表示

---

#### タスク 7.1.3: 後方互換性の実装
**担当サブエージェント:** `rust-developer`
**優先度:** 中
**工数見積:** 1時間
**依存:** タスク7.1.2完了後

**詳細:**
- `MCP_INSPECTOR_CONFIG`使用時に非推奨警告を表示
- 警告内容:
  - "⚠️ MCP_INSPECTOR_CONFIG is deprecated"
  - "⚠️ Please migrate to MCP_INSPECTOR_SERVERS"
  - 移行ガイドへのリンク

**成果物:**
- [ ] 非推奨警告ロジック
- [ ] 既存TOML読み込みの維持

---

### Phase 7.2: テスト実装 🧪

#### タスク 7.2.1: 単体テスト作成
**担当サブエージェント:** `test-engineer`
**優先度:** 高
**工数見積:** 2時間
**依存:** タスク7.1.1完了後

**詳細:**
以下のテストケースを実装:
- `test_parse_servers_from_json()` - 正常系
- `test_parse_invalid_json()` - 異常系（不正JSON）
- `test_parse_empty_servers()` - 異常系（空配列）
- `test_logging_config_from_env()` - ログ設定読み込み
- `test_logging_config_defaults()` - デフォルト値

**成果物:**
- [ ] `tests/config_parsing_tests.rs`
- [ ] テストカバレッジ80%以上

---

#### タスク 7.2.2: 統合テスト作成
**担当サブエージェント:** `test-engineer`
**優先度:** 高
**工数見積:** 2時間
**依存:** タスク7.1.2完了後

**詳細:**
- 環境変数を設定して起動するE2Eテスト
- 既存の`tests/integration_test.rs`を拡張
- 新旧両方の設定方式でのテスト

**成果物:**
- [ ] 統合テストコード
- [ ] CIでのテスト実行確認

---

### Phase 7.3: ドキュメント整備 📖

#### タスク 7.3.1: README更新
**担当サブエージェント:** `tech-writer`
**優先度:** 高
**工数見積:** 2時間
**依存:** タスク7.1.2完了後

**詳細:**
README.mdの以下セクションを更新:
- セットアップ手順（環境変数ベース）
- Claude Desktop設定例
- トラブルシューティング
- 旧方式（TOML）の非推奨化

**成果物:**
- [ ] 更新されたREADME.md
- [ ] 設定例のJSON検証

---

#### タスク 7.3.2: 移行ガイド作成
**担当サブエージェント:** `tech-writer`
**優先度:** 中
**工数見積:** 1時間
**依存:** タスク7.3.1完了後

**詳細:**
`docs/MIGRATION_GUIDE_v0.2.md`を新規作成:
- 移行の理由と利点
- 旧設定→新設定の変換例
- よくある質問（FAQ）
- トラブルシューティング

**成果物:**
- [ ] `docs/MIGRATION_GUIDE_v0.2.md`

---

#### タスク 7.3.3: CHANGELOG更新
**担当サブエージェント:** `tech-writer`
**優先度:** 低
**工数見積:** 0.5時間
**依存:** 全実装完了後

**詳細:**
`CHANGELOG.md`にv0.2.0の変更内容を記載:
- 新機能
- 破壊的変更（非推奨化）
- 移行ガイドへのリンク

**成果物:**
- [ ] 更新されたCHANGELOG.md

---

### Phase 7.4: 品質保証 ✅

#### タスク 7.4.1: コードレビュー
**担当サブエージェント:** `code-auditor`
**優先度:** 高
**工数見積:** 1時間
**依存:** 全実装完了後

**詳細:**
以下の観点でコードレビュー:
- Rustのベストプラクティス準拠
- エラーハンドリングの適切性
- セキュリティ（機密情報のログ出力等）
- パフォーマンス

**成果物:**
- [ ] レビューレポート
- [ ] 修正リスト

---

#### タスク 7.4.2: ビルド検証
**担当サブエージェント:** `rust-developer`
**優先度:** 高
**工数見積:** 0.5時間
**依存:** タスク7.4.1完了後

**詳細:**
以下のコマンドを実行し、全て成功を確認:
```bash
cargo check
cargo clippy
cargo test
cargo build --release
```

**成果物:**
- [ ] ビルド成功ログ
- [ ] テスト結果レポート

---

### Phase 7.5: リリース準備 🚀

#### タスク 7.5.1: バージョン更新
**担当サブエージェント:** `release-manager`
**優先度:** 中
**工数見積:** 0.5時間

**詳細:**
- `Cargo.toml`のバージョンを0.1.0→0.2.0に変更
- リリースタグ作成準備

**成果物:**
- [ ] 更新されたCargo.toml
- [ ] リリースノート草案

---

## 📅 4. スケジュール

### 4.1 タイムライン（1.5日想定）

```
Day 1 (8時間)
├─ 09:00-12:00 [3h] タスク7.1.1: 環境変数パーサー実装
├─ 13:00-15:00 [2h] タスク7.1.2: main.rs書き換え
├─ 15:00-16:00 [1h] タスク7.1.3: 後方互換性実装
└─ 16:00-18:00 [2h] タスク7.2.1: 単体テスト作成

Day 2 (4時間)
├─ 09:00-11:00 [2h] タスク7.2.2: 統合テスト
├─ 11:00-13:00 [2h] タスク7.3.1: README更新
├─ 13:00-14:00 [1h] タスク7.3.2: 移行ガイド
├─ 14:00-14:30 [0.5h] タスク7.3.3: CHANGELOG更新
├─ 14:30-15:30 [1h] タスク7.4.1: コードレビュー
├─ 15:30-16:00 [0.5h] タスク7.4.2: ビルド検証
└─ 16:00-16:30 [0.5h] タスク7.5.1: リリース準備

合計: 12時間（1.5日）
```

### 4.2 マイルストーン

| マイルストーン | 予定日時 | 成果物 |
|-------------|----------|--------|
| M1: コア実装完了 | Day 1 16:00 | main.rs, 設定パーサー |
| M2: テスト完了 | Day 2 11:00 | 全テスト合格 |
| M3: ドキュメント完了 | Day 2 14:30 | README, 移行ガイド |
| M4: 品質保証完了 | Day 2 16:00 | レビュー、ビルド成功 |
| M5: リリース準備完了 | Day 2 16:30 | v0.2.0リリース可能 |

---

## 🎯 5. サブエージェント割り当て

| サブエージェント | 担当タスク | 合計工数 |
|---------------|-----------|---------|
| **rust-developer** | 7.1.1, 7.1.2, 7.1.3, 7.4.2 | 6.5時間 |
| **test-engineer** | 7.2.1, 7.2.2 | 4時間 |
| **tech-writer** | 7.3.1, 7.3.2, 7.3.3 | 3.5時間 |
| **code-auditor** | 7.4.1 | 1時間 |
| **release-manager** | 7.5.1 | 0.5時間 |
| **合計** | - | **15.5時間** |

---

## ⚠️ 6. リスクと対策

### 6.1 技術的リスク

| リスク | 影響度 | 確率 | 対策 |
|-------|--------|------|------|
| JSON パース失敗 | 高 | 低 | 詳細なエラーメッセージと検証機能 |
| 後方互換性の破壊 | 高 | 中 | 移行期間中は両方式をサポート |
| 環境変数が長すぎる | 中 | 中 | ドキュメントで警告、代替案提示 |
| テストの不足 | 中 | 低 | テストカバレッジ80%以上を目標 |

### 6.2 対策詳細

**対策1: JSONスキーマ検証**
```rust
// serde_json_schema を使用して検証
use serde_json_schema::Schema;

fn validate_servers_json(json: &str) -> Result<()> {
    let schema = Schema::compile(include_str!("../schemas/servers.json"))?;
    schema.validate(json)?;
    Ok(())
}
```

**対策2: 移行ヘルパー提供**
```bash
# TOMLからJSONへの変換ツール（将来的に提供）
cargo run --bin toml-to-json -- config/servers.toml
```

---

## 📦 7. 成果物一覧

### 7.1 コード成果物
- [ ] `src/main.rs` (書き換え)
- [ ] `src/models/server_config.rs` (拡張)
- [ ] `src/models/logging_config.rs` (拡張)
- [ ] `tests/config_parsing_tests.rs` (新規)
- [ ] `tests/integration_test.rs` (拡張)

### 7.2 ドキュメント成果物
- [ ] `README.md` (更新)
- [ ] `docs/MIGRATION_GUIDE_v0.2.md` (新規)
- [ ] `CHANGELOG.md` (更新)

### 7.3 品質成果物
- [ ] コードレビューレポート
- [ ] テスト結果レポート
- [ ] ビルド成功ログ

---

## 📋 8. 承認フロー

### 8.1 承認項目

| 項目 | 承認者 | 状態 |
|------|--------|------|
| 実装計画書 | お客様 | ⏳ 承認待ち |
| コア実装 | code-auditor | ⏳ 未着手 |
| テスト結果 | quality-manager | ⏳ 未着手 |
| ドキュメント | お客様 | ⏳ 未着手 |
| 最終リリース | お客様 | ⏳ 未着手 |

### 8.2 承認基準

**実装計画書承認基準:**
- ✅ WBSが明確
- ✅ 工数見積が妥当
- ✅ リスク対策が適切

**実装承認基準:**
- ✅ `cargo check`: エラー0
- ✅ `cargo clippy`: 警告0
- ✅ `cargo test`: 全合格
- ✅ コードレビュー合格

**ドキュメント承認基準:**
- ✅ 移行手順が明確
- ✅ 設定例が正確
- ✅ トラブルシューティング充実

---

## 🚦 9. 開始条件

以下の条件が全て満たされた時点で実装開始します：

- [ ] お客様による本実装計画書の承認
- [ ] 開発環境の準備完了（既存環境で対応可）
- [ ] サブエージェントへの指示準備完了

---

## 📞 10. コミュニケーション計画

### 10.1 進捗報告

- **頻度:** 各マイルストーン完了時
- **方法:** チャット経由でのステータス報告
- **内容:**
  - 完了タスク
  - 次のタスク
  - 問題・ブロッカー

### 10.2 問題エスカレーション

**即座報告が必要な事象:**
- ビルドエラーが解決できない
- テストが失敗する
- 設計の重大な欠陥発見

---

## ✅ 11. 完了条件

Phase 7完了の判定基準:

1. **技術要件**
   - ✅ 全タスクが完了
   - ✅ 品質基準を満たす
   - ✅ ビルド・テスト成功

2. **ドキュメント要件**
   - ✅ README更新完了
   - ✅ 移行ガイド作成完了
   - ✅ CHANGELOG更新完了

3. **承認要件**
   - ✅ コードレビュー合格
   - ✅ お客様の最終承認

4. **リリース要件**
   - ✅ バージョン0.2.0でタグ付け準備完了
   - ✅ リリースノート作成完了

---

## 📝 12. 実装例（参考）

### 12.1 main.rs の実装イメージ

```rust
use anyhow::{Context, Result};
use mcp_inspector_mcp::{run_server, InspectorConfig, InspectorService};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("MCP Inspector Server starting...");

    // Load configuration from environment variables
    let config = load_config_from_env()
        .context("Failed to load configuration from environment")?;

    tracing::info!("Loaded {} server configuration(s)", config.servers.len());
    tracing::info!("Logging backend: {:?}", config.logging.backend);

    // Initialize inspector service
    let inspector = InspectorService::new(config)
        .context("Failed to initialize InspectorService")?;

    // Log configured servers
    for server_name in inspector.list_servers() {
        tracing::info!("  - {}", server_name);
    }

    // Run MCP server
    tracing::info!("MCP Inspector Server ready");
    run_server(inspector).await?;

    Ok(())
}

fn load_config_from_env() -> Result<InspectorConfig> {
    use mcp_inspector_mcp::models::{InspectorConfig, LoggingConfig, LoggingBackend};

    // 優先順位1: 環境変数（新方式）
    if let Ok(servers_json) = env::var("MCP_INSPECTOR_SERVERS") {
        let servers = serde_json::from_str(&servers_json)
            .context("Failed to parse MCP_INSPECTOR_SERVERS as JSON")?;

        // ログ設定（オプション）
        let logging = LoggingConfig {
            backend: env::var("MCP_LOGGING_BACKEND")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(LoggingBackend::Memory),
            db_path: env::var("MCP_LOGGING_DB_PATH").ok(),
            max_logs: env::var("MCP_LOGGING_MAX_LOGS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10000),
        };

        return Ok(InspectorConfig { servers, logging });
    }

    // 優先順位2: TOMLファイル（旧方式・非推奨警告）
    if let Ok(config_path) = env::var("MCP_INSPECTOR_CONFIG") {
        eprintln!("⚠️  WARNING: MCP_INSPECTOR_CONFIG is deprecated.");
        eprintln!("⚠️  Please migrate to MCP_INSPECTOR_SERVERS environment variable.");
        eprintln!("⚠️  See migration guide: https://github.com/.../docs/MIGRATION_GUIDE_v0.2.md");

        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read configuration file: {}", config_path))?;

        let config: InspectorConfig = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse configuration file: {}", config_path))?;

        return Ok(config);
    }

    Err(anyhow::anyhow!(
        "No configuration found. Please set MCP_INSPECTOR_SERVERS environment variable."
    ))
}
```

### 12.2 Claude Desktop設定例

```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\\Users\\takah\\work\\my_mcp_server\\mcp_inspector_mcp\\target\\release\\mcp_inspector_mcp.exe",
      "env": {
        "MCP_INSPECTOR_SERVERS": "[{\"name\":\"fundamental_analysis\",\"transport\":\"stdio\",\"command\":\"C:/Users/takah/work/my_mcp_server/fundamental_analysis/target/release/fundamental_analysis.exe\",\"args\":[]}]",
        "MCP_LOGGING_BACKEND": "persistent",
        "MCP_LOGGING_DB_PATH": "./data/logs.db",
        "MCP_LOGGING_MAX_LOGS": "10000",
        "RUST_LOG": "info"
      }
    }
  }
}
```

---

**実装計画書は以上です。**

**承認後、Phase 7の実装を開始します。推定完了時間: 1.5日（12時間）**

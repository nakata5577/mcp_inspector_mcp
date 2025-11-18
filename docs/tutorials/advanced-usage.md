# Advanced Usage: MCP Inspector MCPの高度な使い方

**所要時間**: 約1〜2時間
**対象**: 上級ユーザー
**前提**: [Getting Started](getting-started.md)と[Practical Guide](practical-guide.md)を完了していること

---

## 目次

1. [このガイドについて](#このガイドについて)
2. [カスタム設定](#カスタム設定)
3. [Capability検証の活用](#capability検証の活用)
4. [エラーハンドリング](#エラーハンドリング)
5. [パフォーマンスチューニング](#パフォーマンスチューニング)
6. [CI/CD統合](#cicd統合)
7. [セキュリティ考慮事項](#セキュリティ考慮事項)
8. [拡張とカスタマイズ](#拡張とカスタマイズ)

---

## このガイドについて

### 対象読者

このガイドは以下のような方を対象としています:

- MCP Inspector MCPの基本操作と実務スキルを習得済みの方
- 本番環境でMCPサーバーを運用している方
- パフォーマンス最適化やセキュリティ強化が必要な方
- CI/CD環境に組み込みたい方
- MCP Inspector MCPをカスタマイズしたい開発者

### 高度な機能の概要

このガイドでカバーする高度な機能:

1. **カスタム設定** - タイムアウト、リトライ、キャッシュTTLの詳細設定
2. **Capability検証** - サーバーの機能を事前に確認し、安全に実行
3. **エラーハンドリング** - 構造化エラーレポートとカスタムハンドラ
4. **パフォーマンスチューニング** - 接続プーリング、キャッシング、並列処理の最適化
5. **CI/CD統合** - 自動テスト、GitHub Actionsとの連携
6. **セキュリティ** - 認証、暗号化、脆弱性対策
7. **拡張** - プラグイン開発、カスタムツールの追加

### 前提条件

- [Getting Started](getting-started.md)チュートリアルを完了
- [Practical Guide](practical-guide.md)チュートリアルを完了
- Rustの基本的な知識（カスタマイズを行う場合）
- JSONの深い理解

---

## カスタム設定

### 実行設定（execution_config）の詳細

`.inspector/config.json`の`execution_config`セクションで、ツール実行時の動作を細かく制御できます。

#### 完全な設定例

```json
{
  "servers": [
    {
      "name": "production_server",
      "transport": "stdio",
      "command": "/path/to/production/server",
      "args": ["--production"],
      "env": {
        "LOG_LEVEL": "info",
        "MAX_CONNECTIONS": "100"
      }
    }
  ],
  "logging": {
    "backend": "persistent",
    "db_path": "./data/production_logs.db",
    "max_logs": 100000
  },
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

#### 設定項目の詳細

##### 1. tool_timeout_ms（ツール実行タイムアウト）

**デフォルト値**: `30000`（30秒）
**環境変数**: `MCP_TOOL_TIMEOUT_MS`
**単位**: ミリ秒

**用途別の推奨値**:

| ツールの種類 | 推奨値 | 理由 |
|------------|--------|------|
| 軽量計算（加算、文字列操作） | 5,000ms | 即座に完了する処理 |
| API呼び出し（REST API） | 30,000ms | ネットワーク遅延を考慮 |
| データベースクエリ | 60,000ms | 複雑なクエリに対応 |
| データ分析（統計処理） | 120,000ms | CPU集約的な処理 |
| 機械学習推論 | 300,000ms | モデルの読み込みと推論 |

**設定例（機械学習サーバー）**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 300000
  }
}
```

##### 2. connection_timeout_ms（接続タイムアウト）

**デフォルト値**: `5000`（5秒）
**環境変数**: `MCP_CONNECTION_TIMEOUT_MS`
**単位**: ミリ秒

**用途別の推奨値**:

| サーバーの種類 | 推奨値 | 理由 |
|--------------|--------|------|
| ローカルサーバー | 5,000ms | 高速起動を期待 |
| リモートサーバー | 15,000ms | ネットワーク遅延を考慮 |
| 重量級サーバー（大量依存関係） | 30,000ms | 初期化に時間がかかる |

**設定例（リモートサーバー）**:
```json
{
  "execution_config": {
    "connection_timeout_ms": 15000
  }
}
```

##### 3. retry_count（リトライ回数）

**デフォルト値**: `0`（リトライしない）
**環境変数**: `MCP_RETRY_COUNT`
**範囲**: 0〜5

**推奨値**:
- **開発環境**: `0`（エラーを即座に検出）
- **本番環境**: `2`〜`3`（一時的な障害に対応）

**設定例（本番環境）**:
```json
{
  "execution_config": {
    "retry_count": 2
  }
}
```

**リトライの動作**:
1. 初回実行が失敗
2. 1秒待機
3. 2回目の実行
4. 失敗した場合、さらに2秒待機
5. 3回目の実行（`retry_count: 2`の場合）

##### 4. auto_retry_on_timeout（タイムアウト時の自動リトライ）

**デフォルト値**: `false`
**環境変数**: `MCP_AUTO_RETRY`

**推奨値**:
- **開発環境**: `false`（タイムアウトを明確に検出）
- **本番環境**: `true`（一時的なネットワーク遅延に対応）

**設定例（本番環境）**:
```json
{
  "execution_config": {
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

**注意**: `retry_count`が`0`の場合、この設定は無視されます。

### タイムアウトのカスタマイズ

#### シナリオ1: 長時間実行ツールへの対応

**問題**: データ分析ツールが30秒を超えることがある

**解決策**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 180000,
    "connection_timeout_ms": 10000
  }
}
```

#### シナリオ2: 高速レスポンスが必要な環境

**問題**: 開発環境で素早くエラーを検出したい

**解決策**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 10000,
    "connection_timeout_ms": 3000,
    "retry_count": 0
  }
}
```

#### シナリオ3: 不安定なネットワーク環境

**問題**: リモートサーバーへの接続が不安定

**解決策**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 30000,
    "retry_count": 3,
    "auto_retry_on_timeout": true
  }
}
```

### 環境変数の活用

設定ファイルを変更せずに、環境変数で動作をカスタマイズできます。

**Windows（PowerShell）**:
```powershell
$env:MCP_TOOL_TIMEOUT_MS = "120000"
$env:MCP_RETRY_COUNT = "2"
.\mcp_inspector_mcp.exe
```

**macOS/Linux（Bash）**:
```bash
export MCP_TOOL_TIMEOUT_MS=120000
export MCP_RETRY_COUNT=2
./mcp_inspector_mcp
```

**優先順位**: `config.json` > 環境変数 > デフォルト値

**注意**: Claude Desktop経由で使用する場合、環境変数は適用されません（`config.json`を使用してください）。

### ログバックエンドの詳細設定

#### Persistent Backendの最適化

**パフォーマンスチューニング**:
```json
{
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs.db",
    "max_logs": 50000
  }
}
```

**max_logsの推奨値**:

| 環境 | 推奨値 | ストレージサイズ（概算） |
|------|--------|----------------------|
| 開発環境 | 10,000 | 5〜10MB |
| ステージング環境 | 50,000 | 25〜50MB |
| 本番環境 | 100,000〜500,000 | 50MB〜2.5GB |

**ストレージ容量の計算**:
```
ストレージサイズ ≈ max_logs × 0.5KB〜1KB（sled圧縮後）
```

**例**: `max_logs: 100000` → 約50〜100MB

#### ログローテーション

**手動ローテーション（定期的に実行）**:
```bash
# Windowsの場合
move .inspector\logs.db .inspector\logs_backup_20251118.db

# macOS/Linuxの場合
mv ./data/logs.db ./data/logs_backup_$(date +%Y%m%d).db
```

MCP Inspector MCPを再起動すると、新しい`logs.db`が自動生成されます。

---

## Capability検証の活用

### Capabilityとは

**Capability**は、MCPサーバーがサポートしている機能を示すメタデータです。

**主要なCapability**:

| Capability | 説明 | 例 |
|-----------|------|-----|
| `tools` | ツール実行機能 | `tools_list`, `tools_call` |
| `resources` | リソース提供機能 | `resources_list`, `resources_read` |
| `prompts` | プロンプトテンプレート機能 | `prompts_list`, `prompts_get` |
| `logging` | ログ通知機能 | `notifications/message` |
| `experimental` | 実験的機能 | 将来の機能 |
| `completions` | 補完機能 | オートコンプリート |

### Capability検証の実行

**Claude Desktopでの指示**:
```
"production_server"のCapabilityを詳しく教えてください
```

**期待される応答**:
```json
{
  "server_name": "production_server",
  "capabilities": {
    "logging": true,
    "experimental": false,
    "completions": false,
    "prompts": {
      "supported": true,
      "list_changed": false
    },
    "resources": {
      "supported": true,
      "subscribe": false,
      "list_changed": false
    },
    "tools": {
      "supported": true,
      "list_changed": false
    }
  }
}
```

### Capability検証のベストプラクティス

#### 1. 事前確認によるエラー回避

**問題**: サポートしていない機能を呼び出してエラーが発生

**解決策**: Capabilityを事前に確認してから実行

**実装例（疑似コード）**:
```javascript
// 1. Capabilityを確認
const capabilities = await server_inspect("my_server");

// 2. サポート確認
if (capabilities.capabilities.tools.supported) {
  // 3. ツールを実行
  await tools_call("my_server", "my_tool", {});
} else {
  console.error("Server does not support tools");
}
```

#### 2. ベストエフォート実行

MCP Inspector MCPは、Capabilityが`false`でも**警告を表示した上で実行を試みます**。

**警告メッセージ例**:
```
WARN Warning: Server 'my_server' does not support tools capability,
but attempting to call tool 'my_tool' anyway (best-effort mode)
```

**ベストエフォート実行の理由**:
- サーバーが正しくCapabilityを報告していない場合がある
- 実験的機能をテストする場合

**推奨**: 本番環境では、Capabilityが`true`のもののみ使用

#### 3. list_changedフラグの活用

**list_changedの意味**:
- `true`: ツール/リソース/プロンプトの一覧が動的に変化する
- `false`: 一覧が固定（キャッシュ有効）

**キャッシュ戦略**:
```json
{
  "tools": {
    "supported": true,
    "list_changed": false
  }
}
```

→ ツール一覧は変化しないため、**キャッシュを長時間保持**可能

```json
{
  "tools": {
    "supported": true,
    "list_changed": true
  }
}
```

→ ツール一覧が動的に変化するため、**キャッシュを短時間で無効化**

### Capability検証の実務例

#### 例1: 条件分岐による安全な実行

**シナリオ**: ログ機能がサポートされている場合のみログを取得

**Claude Desktopでの指示**:
```
"my_server"のCapabilityを確認して、
ログ機能がサポートされていればログメッセージを取得してください
```

**内部的な動作**:
1. `server_inspect`でCapabilityを確認
2. `logging: true`なら`logging_messages`を実行
3. `logging: false`なら「ログ機能は非サポート」と報告

#### 例2: 複数サーバーのCapability比較

**シナリオ**: 複数のサーバーのうち、プロンプト機能を持つものだけ選択

**Claude Desktopでの指示**:
```
["server1", "server2", "server3"]の3つのサーバーのCapabilityを確認して、
プロンプト機能をサポートしているサーバーのみプロンプト一覧を取得してください
```

---

## エラーハンドリング

### エラー構造の理解

MCP Inspector MCPは、Phase 7（v0.3.1）で構造化されたエラーレポート機能を実装しました。

#### エラーの種類

**1. Timeout（タイムアウト）**

```json
{
  "error": {
    "type": "Timeout",
    "tool_name": "long_running_tool",
    "elapsed_ms": 30500,
    "configured_timeout_ms": 30000,
    "suggestion": "Consider increasing tool_timeout_ms in config.json"
  }
}
```

**対処法**:
- `execution_config.tool_timeout_ms`を延長
- ツールの最適化（不要な処理を削減）

**2. ServerCrash（サーバークラッシュ）**

```json
{
  "error": {
    "type": "ServerCrash",
    "message": "Server process terminated unexpectedly",
    "exit_code": 1,
    "suggestion": "Check server logs for errors"
  }
}
```

**対処法**:
- サーバーのログを確認（`RUST_LOG=debug`で実行）
- サーバーのバージョンを更新
- メモリ不足の確認（`top`, `htop`コマンド）

**3. ConnectionError（接続エラー）**

```json
{
  "error": {
    "type": "ConnectionError",
    "message": "Failed to connect to server",
    "details": {
      "server": "my_server",
      "command": "/path/to/server",
      "error": "No such file or directory"
    },
    "suggestion": "Verify command path in config.json"
  }
}
```

**対処法**:
- `command`のパスが正しいか確認
- 実行権限があるか確認（`chmod +x`）
- 依存関係が揃っているか確認

**4. ToolNotFound（ツールが見つからない）**

```json
{
  "error": {
    "type": "ToolNotFound",
    "tool_name": "non_existent_tool",
    "available_tools": ["tool1", "tool2", "tool3"],
    "suggestion": "Use tools_list to see available tools"
  }
}
```

**対処法**:
- `tools_list`で利用可能なツールを確認
- ツール名のスペルミスを確認

**5. InvalidArguments（無効な引数）**

```json
{
  "error": {
    "type": "InvalidArguments",
    "tool_name": "calculate_sum",
    "message": "Missing required argument: 'a'",
    "schema": {
      "type": "object",
      "properties": {
        "a": {"type": "number"},
        "b": {"type": "number"}
      },
      "required": ["a", "b"]
    },
    "suggestion": "Provide required arguments according to schema"
  }
}
```

**対処法**:
- `input_schema`で必須引数を確認
- 引数の型が正しいか確認（`number`, `string`, `boolean`等）

### エラーコードの一覧

| エラーコード | 説明 | HTTP相当 |
|------------|------|---------|
| `Timeout` | タイムアウト | 504 Gateway Timeout |
| `ServerCrash` | サーバークラッシュ | 502 Bad Gateway |
| `ConnectionError` | 接続エラー | 503 Service Unavailable |
| `ToolNotFound` | ツールが見つからない | 404 Not Found |
| `InvalidArguments` | 無効な引数 | 400 Bad Request |
| `CapabilityNotSupported` | 機能が非サポート | 501 Not Implemented |
| `InternalError` | 内部エラー | 500 Internal Server Error |

### カスタムエラーハンドラの実装

**ユースケース**: エラー発生時に自動的にSlack通知を送信

**実装例（スクリプト）**:

```bash
#!/bin/bash
# error_handler.sh

# MCP Inspector MCPを実行し、エラーを検出
OUTPUT=$(./mcp_inspector_mcp tools_call my_server my_tool 2>&1)

# エラーチェック
if echo "$OUTPUT" | grep -q '"error"'; then
  # Slack通知
  curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
    -H 'Content-Type: application/json' \
    -d "{\"text\": \"MCP Error: $OUTPUT\"}"
fi
```

**実務での活用**:
- CIパイプラインでのエラー通知
- 本番環境のモニタリング
- 自動リカバリースクリプト

### リトライロジックのカスタマイズ

**デフォルトのリトライ戦略**:
1. 初回実行失敗
2. 1秒待機
3. 2回目実行
4. 2秒待機
5. 3回目実行

**カスタマイズ例（指数バックオフ）**:

将来のバージョンで以下のような設定を検討中:
```json
{
  "execution_config": {
    "retry_count": 3,
    "retry_strategy": "exponential_backoff",
    "initial_backoff_ms": 1000,
    "max_backoff_ms": 30000
  }
}
```

**現バージョンの回避策**:
スクリプトでリトライロジックを実装:

```bash
#!/bin/bash
# retry_logic.sh

MAX_RETRIES=3
BACKOFF=1

for i in $(seq 1 $MAX_RETRIES); do
  OUTPUT=$(./mcp_inspector_mcp tools_call my_server my_tool)

  if ! echo "$OUTPUT" | grep -q '"error"'; then
    echo "$OUTPUT"
    exit 0
  fi

  echo "Retry $i/$MAX_RETRIES after ${BACKOFF}s"
  sleep $BACKOFF
  BACKOFF=$((BACKOFF * 2))
done

echo "Failed after $MAX_RETRIES retries"
exit 1
```

---

## パフォーマンスチューニング

### 接続プーリングの最適化

**Phase 5（v0.2.0）で実装された接続プーリング**:
- 2回目以降の接続が**50%以上高速化**
- 自動的な接続健全性チェック
- 切断時の自動再接続

**パフォーマンス測定**:

```bash
# 初回接続（キャッシュミス）
time ./mcp_inspector_mcp tools_list my_server
# 実行時間: 150ms

# 2回目の接続（キャッシュヒット）
time ./mcp_inspector_mcp tools_list my_server
# 実行時間: 60ms（60%短縮）
```

**最適化のヒント**:
- 頻繁にアクセスするサーバーは接続を維持
- 長時間未使用の接続は自動的にクリーンアップされる

### キャッシュの最適化

**現在のキャッシュ設定**:
- **TTL**: 5分（固定）
- **対象**: `tools_list`, `resources_list`, `prompts_list`
- **無効化**: サーバー再接続時

**キャッシュヒット率の測定**:

ログで確認:
```bash
RUST_LOG=debug ./mcp_inspector_mcp
```

**期待されるログ**:
```
DEBUG Cache hit for tools_list on server 'my_server'
DEBUG Cache miss for tools_list on server 'my_server', fetching...
```

**推定キャッシュヒット率**: 80%以上（実務での測定値）

**キャッシュの手動無効化**:

現在、手動でのキャッシュ無効化機能は未実装です。回避策:
1. MCP Inspector MCPを再起動
2. サーバーへの接続を切断・再接続

**将来のバージョン**（検討中）:
```json
{
  "execution_config": {
    "cache_ttl_ms": 300000,
    "cache_enabled": true
  }
}
```

### 並列処理の最適化

**バッチメソッドの活用**:

**単一サーバー処理**:
```bash
# 3サーバーを順次処理
time1=$(./mcp_inspector_mcp tools_list server1)  # 120ms
time2=$(./mcp_inspector_mcp tools_list server2)  # 120ms
time3=$(./mcp_inspector_mcp tools_list server3)  # 120ms
# 合計: 360ms
```

**並列処理**:
```bash
# 3サーバーを並列処理
./mcp_inspector_mcp list_tools_batch server1,server2,server3
# 合計: 130ms（約64%短縮）
```

**パフォーマンス向上率**:
- N個のサーバー → 約**1/N**の時間

**並列処理のベストプラクティス**:
1. **10サーバー以上**: バッチメソッド必須
2. **3〜10サーバー**: バッチメソッド推奨
3. **1〜2サーバー**: 単一処理でOK

### メモリ管理

**メモリ使用量の監視**:

**Windows（タスクマネージャー）**:
1. タスクマネージャーを開く
2. `mcp_inspector_mcp.exe`を探す
3. メモリ使用量を確認

**macOS/Linux（top/htop）**:
```bash
# topコマンド
top -p $(pgrep mcp_inspector_mcp)

# htopコマンド（より見やすい）
htop -p $(pgrep mcp_inspector_mcp)
```

**メモリ使用量の最適化**:

**1. Memory Backendの最適化**:
```json
{
  "logging": {
    "backend": "memory",
    "max_logs": 1000
  }
}
```

**メモリ使用量の目安**:
- `max_logs: 1000` → 約1〜2MB
- `max_logs: 10000` → 約10〜20MB
- `max_logs: 100000` → 約100〜200MB

**2. Persistent Backendへの切り替え**:
```json
{
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs.db",
    "max_logs": 100000
  }
}
```

**メモリ削減効果**: 約**50〜70%削減**（ディスクに保存）

### ベンチマーク

**ツール実行のベンチマーク**:

```bash
# 100回実行して平均時間を測定
for i in {1..100}; do
  time ./mcp_inspector_mcp tools_list my_server
done | grep real | awk '{sum+=$2} END {print sum/NR}'
```

**期待される結果**:
- 初回: 150ms
- 2〜100回目: 60ms（キャッシュヒット）
- 平均: 約62ms

**パフォーマンス目標**:

| 操作 | 目標時間 | 実測値（v0.3.1） |
|------|---------|----------------|
| 接続確立（初回） | < 200ms | 150ms ✅ |
| 接続確立（2回目以降） | < 100ms | 60ms ✅ |
| tools_list（キャッシュミス） | < 500ms | 120ms ✅ |
| tools_list（キャッシュヒット） | < 10ms | 1ms ✅ |
| health_check | < 100ms | 45ms ✅ |

---

## CI/CD統合

### 自動テストへの組み込み

**ユースケース**: MCPサーバーのリリース前に自動テストを実行

#### Step 1: テストスクリプトの作成

**test_mcp_server.sh**:
```bash
#!/bin/bash
# test_mcp_server.sh

set -e

SERVER_NAME="my_server"
MCP_INSPECTOR="./mcp_inspector_mcp"

echo "=== MCP Server Integration Test ==="

# 1. ヘルスチェック
echo "1. Health Check..."
HEALTH=$($MCP_INSPECTOR health_check $SERVER_NAME)
STATUS=$(echo $HEALTH | jq -r '.status')

if [ "$STATUS" != "healthy" ]; then
  echo "FAILED: Server is not healthy"
  exit 1
fi
echo "PASSED: Server is healthy"

# 2. ツール一覧の取得
echo "2. Tools List..."
TOOLS=$($MCP_INSPECTOR tools_list $SERVER_NAME)
TOOL_COUNT=$(echo $TOOLS | jq '.tools | length')

if [ "$TOOL_COUNT" -lt 1 ]; then
  echo "FAILED: No tools found"
  exit 1
fi
echo "PASSED: Found $TOOL_COUNT tools"

# 3. 特定ツールの実行テスト
echo "3. Tool Execution..."
RESULT=$($MCP_INSPECTOR tools_call $SERVER_NAME calculate_sum '{"a": 5, "b": 3}')
SUM=$(echo $RESULT | jq -r '.result.sum')

if [ "$SUM" != "8" ]; then
  echo "FAILED: Expected 8, got $SUM"
  exit 1
fi
echo "PASSED: Tool execution successful"

echo "=== All tests passed ==="
```

**実行**:
```bash
chmod +x test_mcp_server.sh
./test_mcp_server.sh
```

#### Step 2: GitHub Actionsとの連携

**.github/workflows/mcp_test.yml**:
```yaml
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
    - uses: actions/checkout@v3

    # Rustのインストール
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    # MCPサーバーのビルド
    - name: Build MCP Server
      run: |
        cd my_mcp_server
        cargo build --release

    # MCP Inspector MCPのビルド
    - name: Build MCP Inspector MCP
      run: |
        cd mcp_inspector_mcp
        cargo build --release

    # 設定ファイルの作成
    - name: Create Config
      run: |
        mkdir -p mcp_inspector_mcp/target/release/.inspector
        cat > mcp_inspector_mcp/target/release/.inspector/config.json <<EOF
        {
          "servers": [
            {
              "name": "my_server",
              "transport": "stdio",
              "command": "${{ github.workspace }}/my_mcp_server/target/release/my_mcp_server"
            }
          ],
          "logging": {
            "backend": "memory",
            "max_logs": 1000
          }
        }
        EOF

    # テスト実行
    - name: Run Tests
      run: |
        cd mcp_inspector_mcp/target/release
        chmod +x test_mcp_server.sh
        ./test_mcp_server.sh
```

#### Step 3: GitLab CIとの連携

**.gitlab-ci.yml**:
```yaml
stages:
  - build
  - test

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/my_mcp_server
      - target/release/mcp_inspector_mcp

test:
  stage: test
  image: rust:latest
  dependencies:
    - build
  script:
    # 設定ファイルの作成
    - mkdir -p .inspector
    - |
      cat > .inspector/config.json <<EOF
      {
        "servers": [
          {
            "name": "my_server",
            "transport": "stdio",
            "command": "$CI_PROJECT_DIR/target/release/my_mcp_server"
          }
        ],
        "logging": {
          "backend": "memory",
          "max_logs": 1000
        }
      }
      EOF
    # テスト実行
    - chmod +x test_mcp_server.sh
    - ./test_mcp_server.sh
```

### スクリプト化

**複数サーバーの一括テスト**:

**test_all_servers.sh**:
```bash
#!/bin/bash
# test_all_servers.sh

SERVERS=("server1" "server2" "server3" "server4")
FAILED=0

for SERVER in "${SERVERS[@]}"; do
  echo "Testing $SERVER..."

  if ./test_mcp_server.sh $SERVER; then
    echo "✅ $SERVER passed"
  else
    echo "❌ $SERVER failed"
    FAILED=$((FAILED + 1))
  fi
done

if [ $FAILED -gt 0 ]; then
  echo "Failed: $FAILED/$[${#SERVERS[@]}] servers"
  exit 1
else
  echo "All servers passed!"
  exit 0
fi
```

### パフォーマンス回帰テスト

**benchmark.sh**:
```bash
#!/bin/bash
# benchmark.sh

SERVER_NAME="my_server"
MCP_INSPECTOR="./mcp_inspector_mcp"
ITERATIONS=100

echo "Running benchmark with $ITERATIONS iterations..."

# ツール実行のベンチマーク
START=$(date +%s%3N)
for i in $(seq 1 $ITERATIONS); do
  $MCP_INSPECTOR tools_call $SERVER_NAME calculate_sum '{"a": 5, "b": 3}' > /dev/null
done
END=$(date +%s%3N)

TOTAL_TIME=$((END - START))
AVG_TIME=$((TOTAL_TIME / ITERATIONS))

echo "Total time: ${TOTAL_TIME}ms"
echo "Average time per request: ${AVG_TIME}ms"

# 閾値チェック（100ms以内）
if [ $AVG_TIME -gt 100 ]; then
  echo "FAILED: Average time exceeds 100ms threshold"
  exit 1
else
  echo "PASSED: Performance within acceptable range"
  exit 0
fi
```

---

## セキュリティ考慮事項

### 認証・認可

**現在の制限**:
- MCP Inspector MCP自体には認証機能がありません
- MCPプロトコルはローカル通信を前提としています

**推奨セキュリティモデル**:

#### 1. ネットワーク分離

**本番環境**:
```
[AIエージェント] → [MCP Inspector MCP] → [信頼されたMCPサーバー]
     ↑ ローカル通信のみ                ↑ ローカル通信のみ
```

**外部からのアクセス禁止**:
- ファイアウォールでポートを閉じる
- VPN経由でのアクセスのみ許可

#### 2. 実行権限の制限

**Windowsの場合**:
- MCP Inspector MCPを非管理者権限で実行
- サーバーごとに専用ユーザーを作成

**macOS/Linuxの場合**:
```bash
# 専用ユーザーの作成
sudo useradd -m -s /bin/bash mcp_user

# 実行権限の設定
sudo chown mcp_user:mcp_user ./mcp_inspector_mcp
sudo chmod 750 ./mcp_inspector_mcp

# 専用ユーザーで実行
sudo -u mcp_user ./mcp_inspector_mcp
```

### セキュアな設定

#### 機密情報の管理

**❌ 悪い例**（設定ファイルに直接記載）:
```json
{
  "servers": [
    {
      "name": "api_server",
      "command": "/path/to/server",
      "env": {
        "API_KEY": "sk-1234567890abcdef"
      }
    }
  ]
}
```

**✅ 良い例**（環境変数で管理）:

**1. 環境変数ファイルの作成**（`.env`）:
```bash
# .env
API_KEY=sk-1234567890abcdef
DATABASE_PASSWORD=super_secret_password
```

**2. `.gitignore`に追加**:
```gitignore
.env
.inspector/config.json
```

**3. スクリプトで環境変数を読み込み**:
```bash
#!/bin/bash
# run_mcp_inspector.sh

# 環境変数を読み込み
set -a
source .env
set +a

# MCP Inspector MCPを実行
./mcp_inspector_mcp
```

**注意**: 現在、MCP Inspector MCPは環境変数の展開機能（`${API_KEY}`形式）をサポートしていません。将来のバージョンで対応予定です。

#### ログの保護

**1. ログファイルのアクセス権限**:

**macOS/Linux**:
```bash
# Persistent Backendのログファイルを保護
chmod 600 ./data/logs.db
chown mcp_user:mcp_user ./data/logs.db
```

**Windows**:
```powershell
# アクセス制御リスト（ACL）の設定
icacls .\data\logs.db /inheritance:r /grant:r "mcp_user:F"
```

**2. 機密情報のフィルタリング**:

ログに機密情報（API key、パスワード等）が含まれないよう、サーバー側で適切にフィルタリングしてください。

### 脆弱性対策

#### 1. 依存関係の定期的な更新

**Rustの更新**:
```bash
rustup update
```

**依存関係の監査**:
```bash
cargo audit
```

**脆弱性が見つかった場合**:
```bash
# Cargo.lockを更新
cargo update

# 再ビルド
cargo build --release
```

#### 2. 入力検証

**MCP Inspector MCPが行っている検証**:
- サーバー名の存在確認
- ツール名の存在確認
- 引数のスキーマ検証（MCPサーバー側）

**追加の検証（推奨）**:
- コマンドインジェクション対策（`command`の検証）
- パストラバーサル対策（`db_path`の検証）

**例**（疑似コード）:
```rust
fn validate_server_command(command: &str) -> Result<(), Error> {
    // コマンドに危険な文字が含まれていないか確認
    if command.contains(";") || command.contains("&&") {
        return Err(Error::InvalidCommand);
    }
    Ok(())
}
```

#### 3. サンドボックス化

**Docker化による分離**:

**Dockerfile**:
```dockerfile
FROM rust:1.70-slim

WORKDIR /app

# MCP Inspector MCPのビルド
COPY . .
RUN cargo build --release

# 非rootユーザーで実行
RUN useradd -m mcp_user
USER mcp_user

CMD ["./target/release/mcp_inspector_mcp"]
```

**実行**:
```bash
docker build -t mcp_inspector .
docker run -v $(pwd)/.inspector:/app/.inspector mcp_inspector
```

---

## 拡張とカスタマイズ

### プラグイン開発（将来機能）

**v0.4.0以降で検討中の機能**:

**プラグインインターフェース**（構想）:
```rust
pub trait McpInspectorPlugin {
    fn name(&self) -> &str;
    fn on_tool_call(&self, server: &str, tool: &str, args: &Value) -> Result<(), Error>;
    fn on_error(&self, error: &Error) -> Result<(), Error>;
}
```

**使用例**（構想）:
```rust
struct SlackNotificationPlugin;

impl McpInspectorPlugin for SlackNotificationPlugin {
    fn name(&self) -> &str {
        "slack_notification"
    }

    fn on_error(&self, error: &Error) -> Result<(), Error> {
        // Slackに通知を送信
        send_slack_message(&format!("Error: {:?}", error))?;
        Ok(())
    }
}
```

### カスタムツールの追加

**ユースケース**: 独自の検査ツールを追加

**実装手順**:

#### Step 1: ツール定義の追加

**src/server/mod.rs**を編集:
```rust
// 新しいツールの定義
let custom_tool = Tool::new(
    "custom_inspect",
    "Custom inspection tool for advanced users",
    json!({
        "type": "object",
        "properties": {
            "server": {
                "type": "string",
                "description": "Server name"
            },
            "options": {
                "type": "object",
                "description": "Custom options"
            }
        },
        "required": ["server"]
    }),
);
```

#### Step 2: ハンドラーの実装

**src/services/inspector.rs**に追加:
```rust
pub async fn custom_inspect(
    server: &str,
    options: Option<Value>,
) -> Result<Value, Error> {
    // カスタム検査ロジック
    let client = get_client(server).await?;

    // 独自の検査処理
    let result = perform_custom_inspection(client, options).await?;

    Ok(json!({
        "server": server,
        "custom_result": result
    }))
}
```

#### Step 3: ビルドと実行

```bash
cargo build --release
./target/release/mcp_inspector_mcp
```

### コントリビューション

**貢献の方法**:

1. **Issue報告**:
   - バグの詳細な再現手順
   - 期待される動作と実際の動作
   - 環境情報（OS、Rustバージョン等）

2. **機能要望**:
   - ユースケースの説明
   - 期待される動作
   - 代替手段の有無

3. **プルリクエスト**:
   - `feat:`, `fix:`, `docs:` 等のプレフィックスを使用
   - 単体テストを追加
   - `cargo fmt`と`cargo clippy`をパス

**プルリクエストの例**:
```bash
# フォーク
git clone https://github.com/yourusername/mcp_inspector_mcp.git
cd mcp_inspector_mcp

# ブランチ作成
git checkout -b feat/add-custom-tool

# 変更を実装
# ...

# テスト
cargo test
cargo clippy

# コミット
git add .
git commit -m "feat: Add custom inspection tool"

# プッシュ
git push origin feat/add-custom-tool

# GitHubでプルリクエストを作成
```

---

## まとめ

このガイドでは、以下の高度な機能を習得しました:

✅ **カスタム設定**: タイムアウト、リトライ、環境変数の詳細設定
✅ **Capability検証**: サーバーの機能を事前確認し、安全に実行
✅ **エラーハンドリング**: 構造化エラーとカスタムハンドラ
✅ **パフォーマンスチューニング**: 接続プーリング、キャッシング、並列処理
✅ **CI/CD統合**: GitHub Actions、GitLab CIとの連携
✅ **セキュリティ**: 認証、機密情報管理、脆弱性対策
✅ **拡張**: プラグイン開発とカスタムツールの追加

**実務での活用例**:
- 本番環境での高可用性運用
- 大規模なMCPサーバー群の効率的な管理
- セキュアな環境での安全な運用
- CI/CDパイプラインへの統合

**さらに学ぶ**:
- [MCP公式ドキュメント](https://modelcontextprotocol.io/)
- [rmcp（Rust MCP SDK）](https://docs.rs/rmcp/)
- [GitHubリポジトリ](https://github.com/yourusername/mcp_inspector_mcp)

---

**最終更新**: 2025-11-18
**バージョン**: v0.3.1
**フィードバック**: [GitHub Issues](https://github.com/yourusername/mcp_inspector_mcp/issues)

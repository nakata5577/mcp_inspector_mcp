# デバッグモードガイド

**対象バージョン**: v0.4.0+
**最終更新**: 2025-11-20

---

## 目次

- [概要](#概要)
- [デバッグモードの有効化](#デバッグモードの有効化)
- [整形表示](#整形表示)
- [タイミング情報](#タイミング情報)
- [ログファイル出力](#ログファイル出力)
- [実践的な使用例](#実践的な使用例)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

デバッグモードは、MCP Inspector MCPの強力な機能で、MCPサーバーとの通信内容を詳細に可視化します。開発時のデバッグやトラブルシューティング、パフォーマンス分析に役立ちます。

### デバッグモードで提供される機能

- **詳細ログ出力**: すべての JSONRPC メッセージを記録
- **整形表示**: リクエスト/レスポンスを見やすく整形
- **カラー表示**: 成功/エラー/警告を色分けで直感的に表示
- **タイミング情報**: 高精度タイマーによる実行時間計測
- **ログファイル出力**: デバッグログの永続化とローテーション

### いつデバッグモードを使うべきか

- **開発中**: 新しいMCPツールの実装とテスト
- **デバッグ時**: 予期しないエラーや動作の調査
- **パフォーマンス分析**: レスポンスタイムの測定
- **統合テスト**: 複雑なワークフローの検証

---

## デバッグモードの有効化

デバッグモードを有効にする方法は3つあります。

### 方法1: コマンドラインフラグ（推奨）

最もシンプルで直接的な方法です。

```bash
mcp_inspector_mcp --verbose
```

**利点**:
- 一時的な有効化に最適
- 設定ファイルの変更不要
- コマンド実行時に即座に反映

### 方法2: 環境変数

セッション全体でデバッグモードを有効にしたい場合に便利です。

**Windows (PowerShell):**
```powershell
$env:MCP_VERBOSE = "true"
mcp_inspector_mcp
```

**Windows (コマンドプロンプト):**
```cmd
set MCP_VERBOSE=true
mcp_inspector_mcp
```

**macOS/Linux:**
```bash
export MCP_VERBOSE=true
mcp_inspector_mcp
```

**利点**:
- 複数のコマンド実行で共通設定を使える
- シェルスクリプトに組み込みやすい
- CI/CD環境での制御が容易

### 方法3: 設定ファイル

恒久的にデバッグモードを有効にする場合に使用します。

`.inspector/config.json` に以下を追加:

```json
{
  "servers": [...],
  "logging": {...},
  "execution_config": {
    "verbose": true
  }
}
```

**利点**:
- 恒久的な設定
- プロジェクト全体での標準設定
- 設定の共有が容易

### 優先順位

設定の優先順位は以下の通りです:

```
CLI引数（--verbose） > 環境変数（MCP_VERBOSE） > 設定ファイル（config.json） > デフォルト値（false）
```

---

## 整形表示

デバッグモードでは、JSONRPCメッセージが見やすく整形されて表示されます。

### リクエストの表示

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📤 REQUEST  [2025-11-20 10:30:45.123]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Tool: tools/call
Request ID: req-abc123

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate_rsi",
    "arguments": {
      "symbol": "AAPL",
      "period": 14
    }
  }
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### レスポンスの表示（成功時）

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📥 RESPONSE [2025-11-20 10:30:45.456] (333ms)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Request ID: req-abc123
Status: ✅ Success

{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"symbol\":\"AAPL\",\"rsi\":65.3}"
      }
    ]
  }
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### レスポンスの表示（エラー時）

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📥 RESPONSE [2025-11-20 10:30:45.456] (333ms)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Request ID: req-abc123
Status: ❌ Error

{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid parameters",
    "data": {
      "reason": "Symbol 'INVALID' not found"
    }
  }
}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### カラー表示

デバッグモードでは、状態に応じて色分けされて表示されます:

- **緑 (✅)**: 成功したリクエスト
- **赤 (❌)**: エラーが発生したリクエスト
- **黄 (⚠️)**: 警告メッセージ
- **青 (ℹ️)**: 情報メッセージ

**カラー表示の無効化**:

環境変数で無効化できます:

```bash
export NO_COLOR=1
mcp_inspector_mcp --verbose
```

### 大きなペイロードのトランケート

デフォルトでは、4KB以上のペイロードは自動的にトランケートされます。

**トランケートの例**:

```
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"data\":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20, ..."
      }
    ]
  }
}

... (truncated 15234 bytes) ...
```

**トランケート無効化**:

```bash
export MCP_TRUNCATE_PAYLOADS=false
mcp_inspector_mcp --verbose
```

---

## タイミング情報

デバッグモードでは、各操作の実行時間が高精度で計測されます。

### タイムスタンプ形式

- **ISO 8601形式**: `2025-11-20T10:30:45.123456Z`
- **マイクロ秒精度**: 6桁の小数点精度

### 経過時間の表示

レスポンスには、リクエスト送信からレスポンス受信までの経過時間が表示されます:

```
📥 RESPONSE [2025-11-20 10:30:45.456] (333ms)
```

- **`(333ms)`**: 経過時間（ミリ秒）

### タイミング分析の例

**高速な操作（< 100ms）**:
```
📥 RESPONSE [2025-11-20 10:30:45.045] (45ms)
```

**通常の操作（100-500ms）**:
```
📥 RESPONSE [2025-11-20 10:30:45.234] (234ms)
```

**遅い操作（> 500ms）**:
```
📥 RESPONSE [2025-11-20 10:30:46.123] (1123ms) ⚠️ SLOW
```

**タイムアウト（> 設定値）**:
```
📥 RESPONSE [2025-11-20 10:31:15.001] (30001ms) ❌ TIMEOUT
```

---

## ログファイル出力

デバッグログをファイルに永続化することができます。

### ログファイル出力の有効化

```bash
export RUST_LOG=debug
mcp_inspector_mcp --verbose 2>&1 | tee debug.log
```

**Windows (PowerShell):**
```powershell
$env:RUST_LOG = "debug"
mcp_inspector_mcp --verbose 2>&1 | Tee-Object -FilePath debug.log
```

### ログレベル

| レベル | 説明 | 用途 |
|--------|------|------|
| `error` | エラーのみ | 本番環境 |
| `warn` | 警告以上 | 本番環境 |
| `info` | 情報以上 | 開発環境 |
| `debug` | デバッグ以上 | デバッグ |
| `trace` | すべて | 詳細デバッグ |

**例**:

```bash
# エラーのみ
export RUST_LOG=error
mcp_inspector_mcp --verbose

# デバッグ情報
export RUST_LOG=debug
mcp_inspector_mcp --verbose

# 特定モジュールのみ
export RUST_LOG=mcp_inspector_mcp::services::inspector=trace
mcp_inspector_mcp --verbose
```

### 構造化ログ（JSON形式）

JSON形式のログ出力も可能です:

```bash
export RUST_LOG=debug
export RUST_LOG_FORMAT=json
mcp_inspector_mcp --verbose > debug.json
```

**JSON形式の例**:

```json
{
  "timestamp": "2025-11-20T10:30:45.123456Z",
  "level": "INFO",
  "target": "mcp_inspector_mcp::services::inspector",
  "message": "tools_list completed",
  "fields": {
    "server": "fundamental_analysis",
    "elapsed_ms": 222,
    "cache_hit": false
  }
}
```

### ログローテーション

ログファイルが大きくなりすぎないよう、日付ベースのローテーションが推奨されます。

**macOS/Linux:**
```bash
# logrotateを使用
cat > /etc/logrotate.d/mcp_inspector << 'EOF'
/var/log/mcp_inspector/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
EOF
```

---

## 実践的な使用例

### 例1: 新しいツールのデバッグ

新しく実装したMCPツールの動作を確認する場合:

```bash
# デバッグモード有効化
mcp_inspector_mcp --verbose

# Claude Desktopから対象ツールを実行
# ターミナルにリクエスト/レスポンスが表示される
```

**観察ポイント**:
- リクエストのパラメータが正しいか
- レスポンスの形式が期待通りか
- エラーメッセージが適切か
- レスポンスタイムが許容範囲か

### 例2: タイムアウト問題の調査

ツール実行がタイムアウトする問題を調査する場合:

```bash
# タイムアウトを延長＋デバッグモード
export MCP_TOOL_TIMEOUT_MS=60000
export RUST_LOG=debug
mcp_inspector_mcp --verbose 2>&1 | tee timeout_debug.log

# 対象ツールを実行
# ログファイルで詳細を確認
```

**確認ポイント**:
- どの時点でタイムアウトしたか
- サーバーからのレスポンスがあったか
- ネットワーク遅延の可能性
- サーバー側の処理時間

### 例3: エラーメッセージの詳細調査

エラーが発生した場合の詳細調査:

```bash
# トレースレベルで詳細ログ
export RUST_LOG=trace
mcp_inspector_mcp --verbose 2>&1 | tee error_trace.log

# エラーを再現
# ログファイルでスタックトレースを確認
```

**確認ポイント**:
- エラーの発生箇所
- エラーメッセージの詳細
- スタックトレース
- 関連する状態変化

### 例4: パフォーマンス分析

複数のツール実行のパフォーマンスを比較する場合:

```bash
# デバッグモード＋時間計測
export RUST_LOG=info
mcp_inspector_mcp --verbose 2>&1 | grep -E "RESPONSE.*ms"

# 複数のツールを実行
# 経過時間を比較
```

**分析例**:
```
📥 RESPONSE [2025-11-20 10:30:45.123] (123ms) - tools_list
📥 RESPONSE [2025-11-20 10:30:46.234] (1111ms) - calculate_rsi
📥 RESPONSE [2025-11-20 10:30:46.345] (111ms) - health_check
```

→ `calculate_rsi`が他のツールより遅いことが分かる

---

## トラブルシューティング

### デバッグモードが有効にならない

**症状**: `--verbose`フラグを使ってもデバッグ情報が表示されない

**原因と対策**:

1. **環境変数の確認**:
   ```bash
   echo $MCP_VERBOSE
   # 出力: true
   ```

2. **ログレベルの確認**:
   ```bash
   echo $RUST_LOG
   # 出力: debug または trace
   ```

3. **設定ファイルの確認**:
   ```bash
   cat .inspector/config.json | grep verbose
   # "verbose": true であることを確認
   ```

### カラー表示が乱れる

**症状**: カラーコードが文字として表示される

**原因**: ターミナルがANSIカラーコードに対応していない

**対策**:

```bash
# カラー表示を無効化
export NO_COLOR=1
mcp_inspector_mcp --verbose
```

### ログファイルが作成されない

**症状**: ログファイルへの出力が機能しない

**原因と対策**:

1. **リダイレクトの確認**:
   ```bash
   # 標準エラー出力もリダイレクト
   mcp_inspector_mcp --verbose 2>&1 | tee debug.log
   ```

2. **書き込み権限の確認**:
   ```bash
   # ログディレクトリの権限確認
   ls -la logs/

   # 必要に応じて権限変更
   chmod 755 logs/
   ```

### 大きなペイロードが見づらい

**症状**: トランケートされてレスポンス全体が見えない

**対策**:

```bash
# トランケート無効化
export MCP_TRUNCATE_PAYLOADS=false
mcp_inspector_mcp --verbose

# または、JSON形式でログファイルに出力
export RUST_LOG_FORMAT=json
mcp_inspector_mcp --verbose > debug.json
```

---

## 参考リンク

- [README.md](../../README.md): 全体的な使い方
- [バッチテストガイド](./batch-testing.md): テスト自動化
- [パフォーマンスモニタリングガイド](./performance-monitoring.md): 性能分析
- [CHANGELOG.md](../../CHANGELOG.md): 変更履歴

---

**最終更新**: 2025-11-20
**対象バージョン**: v0.4.0+

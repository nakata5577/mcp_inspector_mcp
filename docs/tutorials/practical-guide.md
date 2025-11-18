# Practical Guide: 実務で使えるMCP Inspector MCP

**所要時間**: 約1時間
**対象**: 基本操作を習得したユーザー
**前提**: [Getting Started](getting-started.md)を完了していること

---

## 目次

1. [このガイドについて](#このガイドについて)
2. [実務シナリオ1: 新しいサーバーの検査](#実務シナリオ1-新しいサーバーの検査)
3. [実務シナリオ2: ツールの詳細テスト](#実務シナリオ2-ツールの詳細テスト)
4. [実務シナリオ3: エラーのデバッグ](#実務シナリオ3-エラーのデバッグ)
5. [実務シナリオ4: パフォーマンス確認](#実務シナリオ4-パフォーマンス確認)
6. [実務シナリオ5: 複数サーバーの管理](#実務シナリオ5-複数サーバーの管理)
7. [ベストプラクティス](#ベストプラクティス)
8. [次のステップ](#次のステップ)

---

## このガイドについて

### 対象読者

このガイドは以下のような方を対象としています:

- MCP Inspector MCPの基本操作を習得済みの方
- 実務でMCPサーバーを開発・運用している方
- 効率的なデバッグ・テスト手法を学びたい方
- 複数のMCPサーバーを管理している方

### 学習内容

5つの実務シナリオを通じて、以下のスキルを習得します:

1. **新規サーバーの体系的な検査方法** - 初めて触るサーバーを効率的に理解
2. **ツールの詳細テスト** - パラメータのバリデーションとエラーハンドリング
3. **エラーのデバッグ手法** - ログ分析と問題の特定
4. **パフォーマンス測定** - レスポンスタイムとエラー率の監視
5. **複数サーバーの効率的な管理** - バッチ処理と設定の切り替え

### 前提条件

- [Getting Started](getting-started.md)チュートリアルを完了していること
- MCP Inspector MCPがClaude Desktopに登録済みであること
- 少なくとも1つのMCPサーバーが`.inspector/config.json`に登録されていること

---

## 実務シナリオ1: 新しいサーバーの検査

### シナリオ概要

**問題**: チームメイトが開発したMCPサーバーを初めて使う際、どのような機能があるか分からない

**解決策**: MCP Inspector MCPを使って体系的に検査し、サーバーの全体像を把握する

**所要時間**: 約10分

### Step 1: サーバーの基本情報を取得

まず、サーバーが正常に接続できるか、どのような実装情報を持っているかを確認します。

**Claude Desktopでの指示**:
```
"new_server"の詳細情報を取得してください。
サーバーのバージョン、サポートしている機能、プロトコルバージョンを教えてください。
```

**期待される応答例**:
```json
{
  "server_name": "new_server",
  "implementation": {
    "name": "financial-data-server",
    "title": "Financial Data Analysis Server",
    "version": "2.1.0",
    "website_url": "https://example.com"
  },
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
  },
  "connection_status": "connected",
  "protocol_version": "2024-11-05",
  "instructions": null
}
```

**このレスポンスから分かること**:
- ✅ サーバー名: `financial-data-server`
- ✅ バージョン: `2.1.0`
- ✅ サポート機能: Tools, Resources, Prompts, Logging
- ✅ MCPプロトコル: `2024-11-05`（最新）

### Step 2: ヘルスチェックで動作確認

サーバーが正常に動作しているか、レスポンスタイムを確認します。

**Claude Desktopでの指示**:
```
"new_server"のヘルスチェックを実行してください
```

**期待される応答例**:
```json
{
  "server_name": "new_server",
  "status": "healthy",
  "response_time_ms": 32,
  "last_check": "2025-11-18T14:30:00Z",
  "error_count": 0,
  "error_rate": 0.0
}
```

**判定基準**:
- **Healthy** (32ms): 優秀なパフォーマンス ✅
- **Degraded** (500ms〜2000ms): 注意が必要 ⚠️
- **Unhealthy** (2000ms以上): 問題あり ❌

### Step 3: 提供ツールの全体像を把握

サーバーがどのようなツールを提供しているか、一覧を取得します。

**Claude Desktopでの指示**:
```
"new_server"が提供しているツールの一覧を取得してください
```

**期待される応答例**:
```json
{
  "server": "new_server",
  "tools": [
    {
      "name": "get_stock_price",
      "description": "指定された銘柄の現在株価を取得します",
      "input_schema": {
        "type": "object",
        "properties": {
          "ticker": {
            "type": "string",
            "description": "ティッカーシンボル（例: AAPL）"
          }
        },
        "required": ["ticker"]
      }
    },
    {
      "name": "calculate_moving_average",
      "description": "移動平均を計算します",
      "input_schema": {
        "type": "object",
        "properties": {
          "ticker": {
            "type": "string",
            "description": "ティッカーシンボル"
          },
          "period": {
            "type": "integer",
            "description": "期間（日数）",
            "default": 20
          }
        },
        "required": ["ticker"]
      }
    }
  ]
}
```

**分析のポイント**:
1. **ツール数**: 2つ（小規模なサーバー）
2. **必須引数**: `ticker`が両方のツールで必須
3. **オプション引数**: `calculate_moving_average`の`period`はオプション（デフォルト20）

### Step 4: リソースとプロンプトの確認

**Claude Desktopでの指示**:
```
"new_server"のリソース一覧とプロンプト一覧を取得してください
```

**期待される応答例（リソース）**:
```json
{
  "server": "new_server",
  "resources": [
    {
      "uri": "file:///data/sp500_list.csv",
      "name": "S&P 500 List",
      "description": "S&P 500構成銘柄のリスト",
      "mime_type": "text/csv"
    }
  ]
}
```

**期待される応答例（プロンプト）**:
```json
{
  "server": "new_server",
  "prompts": [
    {
      "name": "analyze_stock",
      "description": "株式分析プロンプト",
      "arguments": [
        {
          "name": "ticker",
          "description": "ティッカーシンボル",
          "required": true
        }
      ]
    }
  ]
}
```

### Step 5: 検査結果のまとめ

**検査完了チェックリスト**:
- ✅ サーバーの基本情報を確認（名前、バージョン、capabilities）
- ✅ ヘルスチェックで動作確認（レスポンスタイム測定）
- ✅ 提供ツールの一覧を取得（2ツール）
- ✅ リソースとプロンプトを確認（1リソース、1プロンプト）

**次のアクション**:
- 各ツールを実際に実行して動作確認（シナリオ2）
- ログメッセージを確認してエラーがないか調査（シナリオ3）

---

## 実務シナリオ2: ツールの詳細テスト

### シナリオ概要

**問題**: ツールが期待通り動作するか、エッジケースでエラーハンドリングが適切か確認したい

**解決策**: 正常系と異常系のテストケースを実行し、動作を検証する

**所要時間**: 約15分

### Step 1: 正常系のテスト

まず、有効なパラメータでツールを実行します。

**Claude Desktopでの指示**:
```
"new_server"の"get_stock_price"ツールを使って、
ティッカーシンボル"AAPL"の株価を取得してください
```

**期待される応答例**:
```json
{
  "server": "new_server",
  "tool_name": "get_stock_price",
  "result": {
    "ticker": "AAPL",
    "price": 178.45,
    "currency": "USD",
    "timestamp": "2025-11-18T14:35:00Z"
  }
}
```

**検証ポイント**:
- ✅ 正常にレスポンスが返ってきた
- ✅ データ形式が正しい（price、currency、timestamp）
- ✅ レスポンスタイムが許容範囲内（< 1秒）

### Step 2: オプション引数のテスト

オプション引数の動作を確認します。

**Claude Desktopでの指示**:
```
"new_server"の"calculate_moving_average"ツールを使って、
ティッカーシンボル"AAPL"の移動平均を計算してください。
期間は50日でお願いします。
```

**期待される応答例**:
```json
{
  "server": "new_server",
  "tool_name": "calculate_moving_average",
  "result": {
    "ticker": "AAPL",
    "period": 50,
    "moving_average": 175.23,
    "calculation_date": "2025-11-18"
  }
}
```

**検証ポイント**:
- ✅ オプション引数`period`が正しく渡された（50）
- ✅ デフォルト値ではなく指定値が使われた

### Step 3: デフォルト値のテスト

オプション引数を省略した場合の動作を確認します。

**Claude Desktopでの指示**:
```
"new_server"の"calculate_moving_average"ツールを使って、
ティッカーシンボル"AAPL"の移動平均を計算してください。
期間は指定しません（デフォルト値を使用）。
```

**期待される応答例**:
```json
{
  "server": "new_server",
  "tool_name": "calculate_moving_average",
  "result": {
    "ticker": "AAPL",
    "period": 20,
    "moving_average": 177.89,
    "calculation_date": "2025-11-18"
  }
}
```

**検証ポイント**:
- ✅ デフォルト値（20）が使用された
- ✅ ツールが正常に動作した

### Step 4: 異常系のテスト（無効な引数）

無効な引数を渡した場合のエラーハンドリングを確認します。

**Claude Desktopでの指示**:
```
"new_server"の"get_stock_price"ツールを使って、
ティッカーシンボル"INVALID_TICKER_XXXXXXX"の株価を取得してください
```

**期待される応答例（エラー）**:
```json
{
  "server": "new_server",
  "tool_name": "get_stock_price",
  "error": {
    "code": "TICKER_NOT_FOUND",
    "message": "Ticker symbol 'INVALID_TICKER_XXXXXXX' not found",
    "details": {
      "ticker": "INVALID_TICKER_XXXXXXX",
      "suggestion": "Please check the ticker symbol and try again"
    }
  }
}
```

**検証ポイント**:
- ✅ 適切なエラーメッセージが返された
- ✅ エラーコードが明確（`TICKER_NOT_FOUND`）
- ✅ 修正のための提案が含まれている

### Step 5: 異常系のテスト（タイムアウト）

長時間実行されるツールのタイムアウト動作を確認します。

**事前準備**: タイムアウトを短く設定（テスト用）

`.inspector/config.json`を編集:
```json
{
  "servers": [...],
  "logging": {...},
  "execution_config": {
    "tool_timeout_ms": 5000
  }
}
```

**Claude Desktopでの指示**:
```
"new_server"の"calculate_moving_average"ツールを使って、
ティッカーシンボル"AAPL"、期間365日で計算してください
```

**期待される応答例（タイムアウト）**:
```json
{
  "error": {
    "type": "Timeout",
    "tool_name": "calculate_moving_average",
    "elapsed_ms": 5050,
    "configured_timeout_ms": 5000,
    "suggestion": "Consider increasing tool_timeout_ms in config.json"
  }
}
```

**検証ポイント**:
- ✅ タイムアウトが正しく検出された
- ✅ 経過時間が記録されている
- ✅ 解決策の提案が含まれている

### テスト結果のまとめ

**テスト完了チェックリスト**:
- ✅ 正常系: 有効な引数でツールが動作
- ✅ オプション引数: 指定値とデフォルト値の両方をテスト
- ✅ 異常系（無効引数）: エラーメッセージが適切
- ✅ 異常系（タイムアウト）: タイムアウト検出が機能

---

## 実務シナリオ3: エラーのデバッグ

### シナリオ概要

**問題**: サーバーがエラーを返すが、原因が分からない

**解決策**: ログメッセージを収集・分析し、問題を特定する

**所要時間**: 約15分

### Step 1: エラーの発生

あるツールを実行したところ、エラーが発生しました。

**Claude Desktopでの指示**:
```
"new_server"の"get_stock_price"ツールを使って、
ティッカーシンボル"TSLA"の株価を取得してください
```

**エラーレスポンス**:
```json
{
  "server": "new_server",
  "tool_name": "get_stock_price",
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Failed to fetch stock price"
  }
}
```

**問題**: エラーメッセージが抽象的で、原因が分からない

### Step 2: ログメッセージの取得

サーバーから送信されているログメッセージを確認します。

**Claude Desktopでの指示**:
```
"new_server"のログメッセージを最新50件取得してください。
エラーレベル以上のログに絞ってください。
```

**期待される応答例**:
```json
{
  "server_name": "new_server",
  "messages": [
    {
      "timestamp": "2025-11-18T14:45:20Z",
      "server_name": "new_server",
      "level": "error",
      "logger": "app.api_client",
      "message": "Failed to connect to external API: Connection refused"
    },
    {
      "timestamp": "2025-11-18T14:45:20Z",
      "server_name": "new_server",
      "level": "error",
      "logger": "app.stock_price_service",
      "message": "Stock price fetch failed for ticker TSLA: API connection error"
    },
    {
      "timestamp": "2025-11-18T14:45:15Z",
      "server_name": "new_server",
      "level": "warning",
      "logger": "app.api_client",
      "message": "API rate limit approaching: 95/100 requests"
    }
  ],
  "total_count": 3
}
```

### Step 3: ログの分析

ログメッセージから問題を特定します。

**分析結果**:
1. **根本原因**: 外部API接続エラー（`Connection refused`）
2. **影響範囲**: `get_stock_price`ツールが動作しない
3. **副次的問題**: APIレート制限に近づいている（95/100）

**考えられる原因**:
- 外部APIサーバーがダウンしている
- ネットワーク接続の問題
- APIキーの認証エラー
- ファイアウォールによるブロック

### Step 4: 詳細ログの取得（時間範囲指定）

特定の時間範囲のログを詳しく調査します。

**Claude Desktopでの指示**:
```
"new_server"のログメッセージを取得してください。
2025-11-18T14:45:00Z以降のログで、すべてのレベルを含めてください。
```

**期待される応答例**:
```json
{
  "server_name": "new_server",
  "messages": [
    {
      "timestamp": "2025-11-18T14:45:00Z",
      "level": "info",
      "logger": "app.main",
      "message": "Received request for stock price: ticker=TSLA"
    },
    {
      "timestamp": "2025-11-18T14:45:01Z",
      "level": "debug",
      "logger": "app.api_client",
      "message": "Connecting to https://api.example.com/stocks/TSLA"
    },
    {
      "timestamp": "2025-11-18T14:45:20Z",
      "level": "error",
      "logger": "app.api_client",
      "message": "Failed to connect to external API: Connection refused"
    }
  ],
  "total_count": 3
}
```

**詳細な時系列**:
1. `14:45:00` - リクエスト受信
2. `14:45:01` - API接続試行
3. `14:45:20` - 接続失敗（19秒経過）

**新たな発見**: 接続タイムアウトが発生している（19秒）

### Step 5: 解決策の実施

**問題の特定**:
- 外部APIサーバーへの接続タイムアウト
- APIサーバーが応答していない可能性

**解決アクション**:
1. サーバー管理者に外部API接続状況を確認
2. APIエンドポイントのURLが正しいか確認
3. ネットワーク接続をテスト（`curl`等）
4. 一時的な問題の場合、リトライ設定を追加

**リトライ設定の追加** (`.inspector/config.json`):
```json
{
  "execution_config": {
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

### デバッグ完了チェックリスト

- ✅ エラーメッセージを確認
- ✅ ログメッセージを取得（エラーレベル）
- ✅ ログを分析し、根本原因を特定
- ✅ 時系列でログを追跡
- ✅ 解決策を実施

---

## 実務シナリオ4: パフォーマンス確認

### シナリオ概要

**問題**: サーバーのレスポンスタイムが遅い気がするが、定量的に測定したい

**解決策**: ヘルスチェックとツール実行を定期的に行い、パフォーマンスを監視する

**所要時間**: 約10分

### Step 1: ベースラインの測定

まず、現在のパフォーマンスを測定します。

**Claude Desktopでの指示**:
```
"new_server"のヘルスチェックを実行してください
```

**応答例**:
```json
{
  "server_name": "new_server",
  "status": "healthy",
  "response_time_ms": 120,
  "last_check": "2025-11-18T15:00:00Z",
  "error_count": 2,
  "error_rate": 0.02
}
```

**ベースライン記録**:
- レスポンスタイム: 120ms
- エラー率: 2%（許容範囲内）
- ステータス: Healthy

### Step 2: 負荷テスト（複数回実行）

複数回実行して、パフォーマンスの変動を確認します。

**Claude Desktopでの指示**:
```
"new_server"の"get_stock_price"ツールを5回連続で実行してください。
ティッカーシンボルは"AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"を使ってください。
```

**応答例（5回分）**:
```
1回目: 実行時間 85ms - 成功
2回目: 実行時間 92ms - 成功
3回目: 実行時間 88ms - 成功
4回目: 実行時間 1250ms - 成功（遅い）
5回目: 実行時間 91ms - 成功
```

**分析**:
- 平均レスポンスタイム: 321ms（(85+92+88+1250+91)/5）
- 4回目だけ異常に遅い（1250ms）
- キャッシュの影響または一時的なネットワーク遅延の可能性

### Step 3: キャッシュの影響を確認

同じ引数で複数回実行し、キャッシュの効果を測定します。

**Claude Desktopでの指示**:
```
"new_server"の"get_stock_price"ツールを3回連続で実行してください。
すべて同じティッカーシンボル"AAPL"を使ってください。
```

**応答例**:
```
1回目: 実行時間 320ms - キャッシュミス
2回目: 実行時間 2ms - キャッシュヒット
3回目: 実行時間 1ms - キャッシュヒット
```

**分析**:
- ✅ キャッシュが正常に機能している
- ✅ キャッシュヒット時は1〜2ms（160倍高速化）
- ✅ キャッシュミス時は320ms

### Step 4: エラー率の監視

定期的にヘルスチェックを実行し、エラー率の推移を確認します。

**Claude Desktopでの指示**:
```
"new_server"のヘルスチェックを再度実行してください
```

**応答例（5分後）**:
```json
{
  "server_name": "new_server",
  "status": "degraded",
  "response_time_ms": 1850,
  "last_check": "2025-11-18T15:05:00Z",
  "error_count": 15,
  "error_rate": 0.15
}
```

**変化の検出**:
- ⚠️ ステータスが`healthy` → `degraded`に悪化
- ⚠️ レスポンスタイムが120ms → 1850msに増加（15倍）
- ⚠️ エラー率が2% → 15%に増加

**アラート**: パフォーマンスが低下しています！

### Step 5: 問題の調査

ログメッセージを確認し、パフォーマンス低下の原因を探ります。

**Claude Desktopでの指示**:
```
"new_server"のログメッセージを最新100件取得してください。
警告レベル以上のログに絞ってください。
```

**応答例**:
```json
{
  "messages": [
    {
      "timestamp": "2025-11-18T15:04:00Z",
      "level": "warning",
      "logger": "app.database",
      "message": "Database connection pool exhausted: 95/100 connections in use"
    },
    {
      "timestamp": "2025-11-18T15:04:30Z",
      "level": "error",
      "logger": "app.api_client",
      "message": "API rate limit exceeded: 429 Too Many Requests"
    }
  ]
}
```

**原因の特定**:
1. データベース接続プールが枯渇（95/100）
2. 外部APIのレート制限に到達（429エラー）

**推奨アクション**:
- データベース接続プールを拡大
- APIリクエストの頻度を制限
- キャッシュのTTLを延長して外部API呼び出しを削減

### パフォーマンス測定のまとめ

**測定結果**:
- ✅ ベースライン: 120ms, エラー率2%
- ⚠️ 5分後: 1850ms, エラー率15%（degraded）
- ✅ キャッシュ有効: 1〜2ms（キャッシュヒット時）

**推奨モニタリング頻度**:
- **開発環境**: 1時間ごと
- **本番環境**: 5分ごと

---

## 実務シナリオ5: 複数サーバーの管理

### シナリオ概要

**問題**: 10個以上のMCPサーバーを管理しており、すべてのツールを確認するのに時間がかかる

**解決策**: バッチ処理機能を使って並列に情報を取得する

**所要時間**: 約15分

### Step 1: サーバー一覧の確認

まず、現在登録されているサーバーを確認します。

**Claude Desktopでの指示**:
```
MCP Inspectorに登録されているすべてのサーバーの一覧を取得してください
```

**応答例**:
```json
{
  "servers": [
    {
      "name": "financial_analysis",
      "transport": "stdio",
      "command": "C:/servers/financial_analysis.exe"
    },
    {
      "name": "technical_analysis",
      "transport": "stdio",
      "command": "C:/servers/technical_analysis.exe"
    },
    {
      "name": "news_aggregator",
      "transport": "stdio",
      "command": "C:/servers/news_aggregator.exe"
    },
    {
      "name": "risk_assessment",
      "transport": "stdio",
      "command": "C:/servers/risk_assessment.exe"
    }
  ]
}
```

**登録サーバー数**: 4つ

### Step 2: 全サーバーのツール一覧を並列取得

バッチメソッドを使って、すべてのサーバーのツール一覧を一度に取得します。

**Claude Desktopでの指示**:
```
["financial_analysis", "technical_analysis", "news_aggregator", "risk_assessment"]
の4つのサーバーのツール一覧を一度に取得してください
```

**内部で実行されるツール**:
```json
{
  "name": "list_tools_batch",
  "arguments": {
    "servers": [
      "financial_analysis",
      "technical_analysis",
      "news_aggregator",
      "risk_assessment"
    ]
  }
}
```

**応答例**:
```json
{
  "results": [
    {
      "server": "financial_analysis",
      "tools": [
        {"name": "calculate_dcf", "description": "DCF法による評価"},
        {"name": "get_financial_ratios", "description": "財務比率取得"}
      ]
    },
    {
      "server": "technical_analysis",
      "tools": [
        {"name": "calculate_rsi", "description": "RSI計算"},
        {"name": "calculate_macd", "description": "MACD計算"}
      ]
    },
    {
      "server": "news_aggregator",
      "tools": [
        {"name": "fetch_news", "description": "ニュース取得"},
        {"name": "analyze_sentiment", "description": "感情分析"}
      ]
    },
    {
      "server": "risk_assessment",
      "tools": [
        {"name": "calculate_var", "description": "VaR計算"},
        {"name": "stress_test", "description": "ストレステスト"}
      ]
    }
  ],
  "errors": []
}
```

**パフォーマンス比較**:
- **順次実行**: 4サーバー × 平均120ms = 480ms
- **並列実行**: 約130ms（約**70%短縮**）

### Step 3: 全サーバーのリソース一覧を並列取得

同様に、リソース一覧もバッチで取得します。

**Claude Desktopでの指示**:
```
["financial_analysis", "technical_analysis", "news_aggregator", "risk_assessment"]
の4つのサーバーのリソース一覧を一度に取得してください
```

**応答例**:
```json
{
  "results": [
    {
      "server": "financial_analysis",
      "resources": [
        {"uri": "file:///data/sp500.csv", "name": "S&P 500 List"}
      ]
    },
    {
      "server": "technical_analysis",
      "resources": []
    },
    {
      "server": "news_aggregator",
      "resources": [
        {"uri": "file:///data/sources.json", "name": "News Sources"}
      ]
    },
    {
      "server": "risk_assessment",
      "resources": []
    }
  ],
  "errors": []
}
```

### Step 4: 新しいサーバーの追加（AIエージェント経由）

新しいサーバーをAIエージェントから直接追加します。

**Claude Desktopでの指示**:
```
"sentiment_analysis"という名前で"C:/servers/sentiment_analysis.exe"を
MCP Inspectorに登録してください
```

**内部で実行されるツール**:
```json
{
  "name": "config_add_server",
  "arguments": {
    "name": "sentiment_analysis",
    "transport": "stdio",
    "command": "C:/servers/sentiment_analysis.exe",
    "args": [],
    "env": {}
  }
}
```

**応答**:
```
サーバー"sentiment_analysis"を正常に追加しました。
設定ファイル.inspector/config.jsonに保存されました。
```

**注意**: Claude Desktopを再起動する必要はありません（設定はリアルタイムで反映されます）

### Step 5: 不要なサーバーの削除

使わなくなったサーバーを削除します。

**Claude Desktopでの指示**:
```
"news_aggregator"をMCP Inspectorから削除してください
```

**内部で実行されるツール**:
```json
{
  "name": "config_remove_server",
  "arguments": {
    "name": "news_aggregator"
  }
}
```

**応答**:
```
サーバー"news_aggregator"を正常に削除しました。
```

### 複数サーバー管理のまとめ

**効率化のポイント**:
- ✅ バッチメソッド（`list_tools_batch`, `list_resources_batch`）で並列処理
- ✅ AIエージェントから設定を直接操作（`config_add_server`, `config_remove_server`）
- ✅ 手動でファイル編集する必要がない

**管理作業の時間短縮**:
- 従来: 各サーバーを順次操作（4サーバー × 2分 = 8分）
- 改善後: バッチ処理（1回 × 30秒 = 30秒）
- **削減率: 約94%**

---

## ベストプラクティス

### 1. 定期的なヘルスチェック

**推奨頻度**:
- **開発環境**: 1時間ごと
- **ステージング環境**: 15分ごと
- **本番環境**: 5分ごと

**自動化の方法**:
```bash
# cron（Linuxの場合）
*/5 * * * * /path/to/health_check_script.sh
```

### 2. ログの適切な保存

**Memory Backend vs Persistent Backend**:

| 項目 | Memory | Persistent |
|------|--------|-----------|
| **速度** | 高速（1000件/秒） | 普通（500-1000件/秒） |
| **永続化** | ❌（再起動で消失） | ✅（ディスク保存） |
| **推奨環境** | 開発・テスト | 本番・長期保存 |

**設定例（本番環境）**:
```json
{
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs.db",
    "max_logs": 50000
  }
}
```

### 3. エラーログのフィルタリング

**開発時の推奨**:
- すべてのレベルを確認（`debug`以上）

**本番環境の推奨**:
- `warning`以上のみ監視
- `error`以上は即座にアラート

### 4. タイムアウト設定の最適化

**ツールの種類別**:

| ツール種類 | 推奨タイムアウト |
|-----------|----------------|
| **軽量な計算** | 5秒 |
| **API呼び出し** | 30秒（デフォルト） |
| **データ分析** | 60〜120秒 |
| **機械学習推論** | 180秒以上 |

**設定例（長時間実行ツール）**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 180000,
    "retry_count": 1
  }
}
```

### 5. キャッシュの活用

**キャッシュ有効なケース**:
- ✅ ツール一覧（変化が少ない）
- ✅ リソース一覧（静的データ）
- ✅ プロンプト一覧（固定テンプレート）

**キャッシュ無効なケース**:
- ❌ リアルタイムデータ（株価、為替）
- ❌ ユーザー固有データ
- ❌ 時間依存データ

### 6. バッチ処理の活用

**効率的なケース**:
- 複数サーバーの一覧取得
- 定期的なモニタリング
- 初回検査（新規サーバー）

**使用例**:
```
["server1", "server2", "server3"]の3つのサーバーの
ツール一覧を一度に取得してください
```

### 7. エラーハンドリング

**推奨パターン**:
1. エラー発生 → ログメッセージ取得
2. 根本原因の特定
3. リトライ設定の追加
4. 必要に応じてタイムアウト延長

**リトライ設定の例**:
```json
{
  "execution_config": {
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

### 8. 環境変数の管理

**機密情報の扱い**:
- ❌ `.inspector/config.json`に直接記載しない
- ✅ 環境変数で管理
- ✅ `.gitignore`に`.inspector/config.json`を追加

**設定例**:
```json
{
  "servers": [
    {
      "name": "api_server",
      "command": "/path/to/server",
      "env": {
        "API_KEY": "${API_KEY}"
      }
    }
  ]
}
```

**注意**: 現在、環境変数の展開機能は未実装です。将来のバージョンで対応予定。

### 9. 設定のバージョン管理

**推奨方法**:
- `.inspector/config.json`をGitで管理
- 機密情報は別ファイル（`.inspector/secrets.json`）に分離
- `.inspector/secrets.json`を`.gitignore`に追加

### 10. ドキュメント化

**サーバーごとに記録すべき情報**:
- 目的と責任範囲
- 提供ツールの一覧
- 依存関係（外部API等）
- エラーパターンと対処法

---

## 次のステップ

実務シナリオをマスターしたら、さらに高度な機能を学びましょう！

### Advanced Usage（高度な使い方）

**学習内容**:
- カスタム設定の詳細（タイムアウト、リトライ、キャッシュTTL）
- Capability検証の活用
- エラーハンドリングの高度なパターン
- CI/CD統合（自動テスト、GitHub Actions）
- セキュリティ考慮事項（認証、暗号化）
- 拡張とカスタマイズ

**所要時間**: 約1〜2時間
**ドキュメント**: [docs/tutorials/advanced-usage.md](advanced-usage.md)

---

## まとめ

このガイドでは、以下の実務スキルを習得しました:

✅ **シナリオ1**: 新しいサーバーの体系的な検査方法
✅ **シナリオ2**: ツールの詳細テスト（正常系・異常系）
✅ **シナリオ3**: ログ分析によるエラーのデバッグ
✅ **シナリオ4**: パフォーマンス測定と監視
✅ **シナリオ5**: 複数サーバーの効率的な管理

✅ **ベストプラクティス**: 10個の実務で役立つテクニック

**実務での活用例**:
- 新規MCPサーバーの導入前検証
- 本番環境でのヘルスモニタリング
- エラー発生時の迅速なデバッグ
- 複数プロジェクトの一元管理

**次のステップ**: [Advanced Usage](advanced-usage.md)で高度な機能を習得しましょう！

---

**最終更新**: 2025-11-18
**バージョン**: v0.3.1
**フィードバック**: [GitHub Issues](https://github.com/yourusername/mcp_inspector_mcp/issues)

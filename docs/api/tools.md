# MCP Inspector MCP Server - API仕様書

**バージョン**: v0.3.1
**最終更新**: 2025-11-18
**プロトコル**: Model Context Protocol (MCP) 2024-11-05

---

## 目次

1. [概要](#概要)
2. [全ツール一覧](#全ツール一覧)
3. [ツール詳細仕様](#ツール詳細仕様)
   - [Tools検査機能](#tools検査機能)
   - [Resources検査機能](#resources検査機能)
   - [Prompts検査機能](#prompts検査機能)
   - [Sampling検査機能](#sampling検査機能)
   - [サーバー検査機能](#サーバー検査機能)
   - [設定管理機能](#設定管理機能)
4. [共通仕様](#共通仕様)
5. [エラーコード一覧](#エラーコード一覧)
6. [ベストプラクティス](#ベストプラクティス)
7. [バージョニング](#バージョニング)

---

## 概要

MCP Inspector MCP Serverは、他のMCPサーバーをデバッグ・検査するための専用MCPサーバーです。AIエージェント（Claude Desktopなど）から、対象MCPサーバーのツール実行、ヘルスチェック、ログ取得などが行えます。

### アーキテクチャ

```
AIエージェント (Claude Desktop)
    ↓ MCP Protocol
本MCPサーバー (mcp_inspector_mcp)
    ↓ MCP Protocol
対象MCPサーバー (fundamental_analysis, filesystem, etc.)
```

### プロトコル情報

- **プロトコル名**: Model Context Protocol (MCP)
- **プロトコルバージョン**: 2024-11-05
- **トランスポート**: stdio（標準入出力）
- **エンコーディング**: JSON-RPC 2.0

---

## 全ツール一覧

MCP Inspector MCP Serverは以下の13ツールを提供します。

| カテゴリ | ツール名 | 説明 | 主な用途 |
|---------|----------|------|---------|
| **Tools検査** | `tools_list` | ツール一覧取得 | 対象サーバーの提供機能確認 |
| | `tools_call` | ツール実行 | 対象サーバーのツールをテスト実行 |
| **Resources検査** | `resources_list` | リソース一覧取得 | 対象サーバーのデータソース確認 |
| | `resources_read` | リソース読み取り | 対象サーバーのデータ取得 |
| **Prompts検査** | `prompts_list` | プロンプト一覧取得 | 対象サーバーのプロンプトテンプレート確認 |
| | `prompts_get` | プロンプト取得 | プロンプトテンプレートの詳細取得 |
| **Sampling検査** | `sampling_logs` | Samplingログ取得 | AI推論リクエストのログ調査 |
| **サーバー検査** | `server_inspect` | サーバー設定検査 | Capability、バージョン情報確認 |
| | `health_check` | ヘルスチェック | 疎通確認、パフォーマンス測定 |
| | `logging_messages` | ログメッセージ取得 | サーバーログの収集・分析 |
| **設定管理** | `config_add_server` | サーバー追加 | 検査対象サーバーの登録 |
| | `config_remove_server` | サーバー削除 | 検査対象サーバーの削除 |
| | `config_list_servers` | サーバー一覧 | 登録済みサーバーの確認 |

---

## ツール詳細仕様

### Tools検査機能

#### tools_list

対象MCPサーバーが提供するツールの一覧を取得します。

**説明**
指定されたMCPサーバーに接続し、`tools/list`プロトコルを使用してツール一覧を取得します。取得したツール情報には、名前、説明、入力スキーマが含まれます。

**用途**
- 対象サーバーが提供する機能の確認
- ツールの入力スキーマの調査
- APIドキュメント生成の基礎データ収集

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名（`.inspector/config.json`に登録済みのもの） | - |

**リクエストJSONの例**
```json
{
  "name": "tools_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `tools` | array | ツール情報の配列 |
| `tools[].name` | string | ツール名 |
| `tools[].description` | string | ツールの説明（オプション） |
| `tools[].input_schema` | object | JSONスキーマ形式の入力仕様（オプション） |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "tools": [
    {
      "name": "get_financial_ratios",
      "description": "企業の財務比率を取得します",
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
      "name": "calculate_dcf",
      "description": "DCF法による企業価値評価",
      "input_schema": {
        "type": "object",
        "properties": {
          "ticker": {
            "type": "string"
          },
          "growth_rate": {
            "type": "number"
          }
        },
        "required": ["ticker"]
      }
    }
  ]
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to list tools: Server 'unknown_server' not found in configuration"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、接続失敗）
- `-32602`: Invalid params（パラメータ不正）

**使用例**

基本的な使用例:
```
"fundamental_analysis"サーバーのツール一覧を取得してください
```

**注意事項**
- キャッシュTTL: 5分（2回目以降のリクエストは高速）
- タイムアウト: デフォルト30秒（`execution_config.tool_timeout_ms`で変更可能）
- サーバーが`tools` capabilityをサポートしていない場合、警告が出力されますがベストエフォートで実行されます

---

#### tools_call

対象MCPサーバーの特定のツールを実行します。

**説明**
指定されたMCPサーバーに接続し、`tools/call`プロトコルを使用してツールを実行します。実行結果はJSON形式で返却されます。

**用途**
- デバッグ時のツール動作確認
- 統合テストの実行
- 本番環境でのツール機能検証

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `tool_name` | string | ✅ | 実行するツール名 | - |
| `arguments` | object | ❌ | ツールに渡す引数（JSONオブジェクト） | `{}` |

**リクエストJSONの例**
```json
{
  "name": "tools_call",
  "arguments": {
    "server": "fundamental_analysis",
    "tool_name": "get_financial_ratios",
    "arguments": {
      "ticker": "AAPL"
    }
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `tool_name` | string | 実行したツール名 |
| `result` | any | ツールの実行結果（ツール依存） |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "tool_name": "get_financial_ratios",
  "result": {
    "ticker": "AAPL",
    "ratios": {
      "pe_ratio": 28.5,
      "pb_ratio": 42.1,
      "debt_to_equity": 1.73,
      "roe": 0.147
    }
  }
}
```

**エラー時のレスポンスJSONの例**

タイムアウトエラー:
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to call tool: Timeout after 30000ms",
    "data": {
      "type": "Timeout",
      "tool_name": "slow_operation",
      "elapsed_ms": 30500,
      "configured_timeout_ms": 30000,
      "suggestion": "Increase tool_timeout_ms in execution_config"
    }
  }
}
```

Capabilityエラー（警告として実行継続）:
```json
{
  "warning": "Server 'xxx' does not support tools capability, but attempting to call tool 'yyy'",
  "server": "xxx",
  "tool_name": "yyy",
  "result": { ... }
}
```

**エラーコード**
- `-32603`: Internal error（接続失敗、実行失敗、タイムアウト、サーバークラッシュ）
- `-32602`: Invalid params（パラメータ不正、引数のJSON解析失敗）

**使用例**

基本的な使用例:
```
"fundamental_analysis"サーバーの"get_financial_ratios"ツールを
引数 {"ticker": "AAPL"} で実行してください
```

高度な使用例（複雑な引数）:
```json
{
  "name": "tools_call",
  "arguments": {
    "server": "technical_analysis",
    "tool_name": "backtest_strategy",
    "arguments": {
      "symbol": "AAPL",
      "start_date": "2024-01-01",
      "end_date": "2024-12-31",
      "strategy": {
        "type": "moving_average_crossover",
        "short_period": 50,
        "long_period": 200
      }
    }
  }
}
```

**注意事項**
- **タイムアウト**: デフォルト30秒（長時間実行が必要な場合は`execution_config.tool_timeout_ms`で延長）
- **リトライ**: `execution_config.retry_count`と`auto_retry_on_timeout`で設定可能
- **引数の型**: Claude Desktopは引数をJSON文字列として送信する場合があります。自動的にパースされます
- **エラーハンドリング**: v0.3.1で強化され、詳細なエラー情報が返却されます

---

### Resources検査機能

#### resources_list

対象MCPサーバーが提供するリソースの一覧を取得します。

**説明**
指定されたMCPサーバーに接続し、`resources/list`プロトコルを使用してリソース一覧を取得します。

**用途**
- 対象サーバーが公開しているデータの確認
- リソースURIの取得
- データソースの調査

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |

**リクエストJSONの例**
```json
{
  "name": "resources_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `resources` | array | リソース情報の配列 |
| `resources[].uri` | string | リソースURI |
| `resources[].name` | string | リソース名（オプション） |
| `resources[].description` | string | リソースの説明（オプション） |
| `resources[].mime_type` | string | MIMEタイプ（オプション） |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "resources": [
    {
      "uri": "file:///data/sp500_list.csv",
      "name": "S&P 500 List",
      "description": "S&P 500構成銘柄のリスト",
      "mime_type": "text/csv"
    },
    {
      "uri": "file:///data/financial_reports/AAPL_2024Q4.json",
      "name": "AAPL 2024 Q4 Report",
      "description": "Appleの2024年Q4決算レポート",
      "mime_type": "application/json"
    }
  ]
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to list resources: Connection failed"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、接続失敗）
- `-32602`: Invalid params（パラメータ不正）

**使用例**
```
"fundamental_analysis"サーバーのリソース一覧を取得してください
```

**注意事項**
- キャッシュTTL: 5分
- サーバーが`resources` capabilityをサポートしていない場合、空配列が返却されます

---

#### resources_read

対象MCPサーバーの特定のリソースを読み込みます。

**説明**
指定されたMCPサーバーに接続し、`resources/read`プロトコルを使用してリソースのコンテンツを取得します。

**用途**
- リソースの内容確認
- データの取得と解析
- ファイルやデータベースの読み取り

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `uri` | string | ✅ | リソースのURI（`resources_list`で取得） | - |

**リクエストJSONの例**
```json
{
  "name": "resources_read",
  "arguments": {
    "server": "fundamental_analysis",
    "uri": "file:///data/sp500_list.csv"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `uri` | string | リソースURI |
| `contents` | array | コンテンツの配列 |
| `contents[].uri` | string | コンテンツのURI |
| `contents[].mime_type` | string | MIMEタイプ（オプション） |
| `contents[].text` | string | テキストコンテンツ（テキストの場合） |
| `contents[].blob` | string | Base64エンコードされたバイナリデータ（バイナリの場合） |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "uri": "file:///data/sp500_list.csv",
  "contents": [
    {
      "uri": "file:///data/sp500_list.csv",
      "mime_type": "text/csv",
      "text": "Symbol,Name,Sector\nAAPL,Apple Inc.,Technology\nMSFT,Microsoft Corporation,Technology\n...",
      "blob": null
    }
  ]
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to read resource: Resource not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（リソース未存在、読み取り失敗）
- `-32602`: Invalid params（パラメータ不正、URI不正）

**使用例**
```
"fundamental_analysis"サーバーの"file:///data/sp500_list.csv"リソースを読み込んでください
```

**注意事項**
- 大きなリソースの場合、タイムアウトに注意
- バイナリデータは`blob`フィールドにBase64エンコードされて返却されます

---

### Prompts検査機能

#### prompts_list

対象MCPサーバーが提供するプロンプトテンプレートの一覧を取得します。

**説明**
指定されたMCPサーバーに接続し、`prompts/list`プロトコルを使用してプロンプトテンプレート一覧を取得します。

**用途**
- 対象サーバーが提供するプロンプトテンプレートの確認
- プロンプトの引数仕様の調査

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |

**リクエストJSONの例**
```json
{
  "name": "prompts_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `prompts` | array | プロンプト情報の配列 |
| `prompts[].name` | string | プロンプト名 |
| `prompts[].description` | string | プロンプトの説明（オプション） |
| `prompts[].arguments` | array | 引数定義の配列 |
| `prompts[].arguments[].name` | string | 引数名 |
| `prompts[].arguments[].description` | string | 引数の説明（オプション） |
| `prompts[].arguments[].required` | boolean | 必須かどうか |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "prompts": [
    {
      "name": "analyze_company",
      "description": "企業の財務分析を行うプロンプト",
      "arguments": [
        {
          "name": "ticker",
          "description": "ティッカーシンボル",
          "required": true
        },
        {
          "name": "analysis_type",
          "description": "分析タイプ（fundamental/technical）",
          "required": false
        }
      ]
    }
  ]
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to list prompts: Connection failed"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、接続失敗）
- `-32602`: Invalid params（パラメータ不正）

**使用例**
```
"fundamental_analysis"サーバーのプロンプトテンプレート一覧を取得してください
```

**注意事項**
- キャッシュTTL: 5分
- サーバーが`prompts` capabilityをサポートしていない場合、空配列が返却されます

---

#### prompts_get

対象MCPサーバーの特定のプロンプトテンプレートを取得します。

**説明**
指定されたMCPサーバーに接続し、`prompts/get`プロトコルを使用してプロンプトの詳細を取得します。引数を渡すことで、動的にプロンプトを生成できます。

**用途**
- プロンプトテンプレートの内容確認
- 引数を渡してプロンプトを生成
- AIエージェントへのプロンプト提供

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `name` | string | ✅ | プロンプト名 | - |
| `arguments` | object | ❌ | プロンプトに渡す引数（文字列のキー・バリュー） | `{}` |

**リクエストJSONの例**
```json
{
  "name": "prompts_get",
  "arguments": {
    "server": "fundamental_analysis",
    "name": "analyze_company",
    "arguments": {
      "ticker": "AAPL",
      "analysis_type": "fundamental"
    }
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `name` | string | プロンプト名 |
| `messages` | array | メッセージの配列 |
| `messages[].role` | string | ロール（`user`, `assistant`等） |
| `messages[].content` | object | メッセージコンテンツ |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "name": "analyze_company",
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Analyze AAPL company using fundamental analysis. Focus on financial ratios, growth potential, and competitive advantages."
      }
    }
  ]
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to get prompt: Prompt 'unknown_prompt' not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（プロンプト未存在、生成失敗）
- `-32602`: Invalid params（パラメータ不正、必須引数不足）

**使用例**
```
"fundamental_analysis"サーバーの"analyze_company"プロンプトを
引数 {"ticker": "AAPL"} で取得してください
```

**注意事項**
- 引数はすべて文字列型（`HashMap<String, String>`）
- 必須引数が不足している場合、エラーが返却されます

---

### Sampling検査機能

#### sampling_logs

対象MCPサーバーからのSamplingリクエストのログを取得します。

**説明**
対象MCPサーバーがAIエージェントに対して発行したSamplingリクエスト（AI推論依頼）のログを取得します。

**⚠️ 重要な制限事項**: rmcp 0.8.5の技術的制約により、実際のSampling通信の監視は未実装です。ログ管理インフラのみ提供しており、将来のバージョンで完全実装予定です。

**用途**
- Samplingリクエストの履歴調査
- AI推論パラメータの確認
- デバッグ用途（将来実装）

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `limit` | integer | ❌ | 取得するログの最大件数 | `100` |
| `status` | string | ❌ | フィルタするステータス（`all`, `success`, `failed`） | `"all"` |

**リクエストJSONの例**
```json
{
  "name": "sampling_logs",
  "arguments": {
    "server": "fundamental_analysis",
    "limit": 10,
    "status": "all"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server` | string | 対象サーバー名 |
| `logs` | array | Samplingログエントリの配列 |
| `logs[].id` | string | ログエントリの一意ID |
| `logs[].timestamp` | string | タイムスタンプ（ISO 8601形式） |
| `logs[].status` | string | ステータス（`pending`, `success`, `failed`） |
| `logs[].messages` | array | Samplingリクエストのメッセージ |
| `logs[].model_preferences` | object | モデル選択のヒント（オプション） |
| `logs[].system_prompt` | string | システムプロンプト（オプション） |
| `logs[].max_tokens` | integer | 最大トークン数（オプション） |
| `logs[].error` | string | エラーメッセージ（失敗時） |
| `logs[].response` | string | レスポンス内容（成功時） |
| `total_count` | integer | 総ログ数 |

**成功時のレスポンスJSONの例**
```json
{
  "server": "fundamental_analysis",
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2025-11-18T12:00:00Z",
      "status": "pending",
      "messages": [
        {
          "role": "user",
          "content": {
            "type": "text",
            "text": "Analyze AAPL financial data"
          }
        }
      ],
      "model_preferences": {
        "hints": [{"name": "claude-3-opus"}],
        "intelligence_priority": 0.8
      },
      "system_prompt": null,
      "max_tokens": 1024,
      "error": null,
      "response": null
    }
  ],
  "total_count": 1
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to get sampling logs: Server not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、ログバックエンド失敗）
- `-32602`: Invalid params（パラメータ不正、limitが負数）

**使用例**
```
"fundamental_analysis"サーバーのSamplingログを最新10件取得してください
```

**注意事項**
- **現在未実装**: 実際のSampling通信は監視されません。ログ管理インフラのみ提供
- ログバックエンド（Memory/Persistent）によって永続性が異なります
- `max_logs`（デフォルト10,000件）を超えると古いログから削除されます

---

### サーバー検査機能

#### server_inspect

対象MCPサーバーの設定情報と機能を詳細に取得します。

**説明**
指定されたMCPサーバーに接続し、初期化情報（`initialize`プロトコル）を取得します。サーバーのバージョン、サポートしている機能（Capability）、プロトコルバージョンなどが含まれます。

**用途**
- サーバーの機能確認（Tools/Resources/Prompts等）
- プロトコルバージョンの確認
- 接続状態の診断
- サーバー情報のドキュメント化

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |

**リクエストJSONの例**
```json
{
  "name": "server_inspect",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server_name` | string | サーバー名 |
| `implementation` | object | 実装情報 |
| `implementation.name` | string | 実装名 |
| `implementation.title` | string | タイトル（オプション） |
| `implementation.version` | string | バージョン |
| `implementation.website_url` | string | WebサイトURL（オプション） |
| `capabilities` | object | Capability情報 |
| `capabilities.logging` | boolean | ログ機能のサポート |
| `capabilities.experimental` | boolean | 実験的機能のサポート |
| `capabilities.completions` | boolean | 補完機能のサポート |
| `capabilities.prompts` | object | プロンプト機能の詳細 |
| `capabilities.prompts.supported` | boolean | プロンプトのサポート |
| `capabilities.prompts.list_changed` | boolean | 変更通知のサポート |
| `capabilities.resources` | object | リソース機能の詳細 |
| `capabilities.resources.supported` | boolean | リソースのサポート |
| `capabilities.resources.subscribe` | boolean | 購読機能のサポート |
| `capabilities.resources.list_changed` | boolean | 変更通知のサポート |
| `capabilities.tools` | object | ツール機能の詳細 |
| `capabilities.tools.supported` | boolean | ツールのサポート |
| `capabilities.tools.list_changed` | boolean | 変更通知のサポート |
| `connection_status` | string | 接続状態（`connected`, `disconnected`, `error`） |
| `protocol_version` | string | プロトコルバージョン（オプション） |
| `instructions` | string | サーバーからの指示（オプション） |

**成功時のレスポンスJSONの例**
```json
{
  "server_name": "fundamental_analysis",
  "implementation": {
    "name": "fundamental-analysis-server",
    "title": "Financial Analysis Server",
    "version": "1.2.3",
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

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to inspect server: Connection timeout"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、接続失敗、初期化失敗）
- `-32602`: Invalid params（パラメータ不正）

**使用例**
```
"fundamental_analysis"サーバーの設定情報を取得してください
```

**注意事項**
- 接続プーリングにより、2回目以降は高速（50%以上高速化）
- Capabilityをチェックすることで、サーバーがサポートしていない機能への呼び出しを避けられます

---

#### health_check

対象MCPサーバーのヘルスチェックを実行し、疎通確認とパフォーマンス測定を行います。

**説明**
指定されたMCPサーバーに`ping`プロトコルを送信し、レスポンスタイムを測定します。最近の履歴からエラー率を計算し、ヘルスステータス（Healthy/Degraded/Unhealthy）を判定します。

**用途**
- サーバーの疎通確認
- レスポンスタイムの測定
- エラー率の監視
- 本番環境のヘルスモニタリング

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |

**リクエストJSONの例**
```json
{
  "name": "health_check",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server_name` | string | サーバー名 |
| `status` | string | ヘルスステータス（`healthy`, `degraded`, `unhealthy`） |
| `response_time_ms` | integer | レスポンスタイム（ミリ秒） |
| `last_check` | string | 最終チェック時刻（RFC3339形式） |
| `error_count` | integer | 最近のエラー回数 |
| `error_rate` | number | エラー率（0.0〜1.0） |
| `details` | string | 詳細情報（エラー時） |

**成功時のレスポンスJSONの例**
```json
{
  "server_name": "fundamental_analysis",
  "status": "healthy",
  "response_time_ms": 45,
  "last_check": "2025-11-18T10:30:00Z",
  "error_count": 0,
  "error_rate": 0.0,
  "details": null
}
```

**エラー時のレスポンスJSONの例**

Unhealthyステータス:
```json
{
  "server_name": "fundamental_analysis",
  "status": "unhealthy",
  "response_time_ms": 2500,
  "last_check": "2025-11-18T10:35:00Z",
  "error_count": 25,
  "error_rate": 0.25,
  "details": "Response time exceeded threshold (2000ms)"
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、接続失敗）
- `-32602`: Invalid params（パラメータ不正）

**ステータス判定基準**

| ステータス | 条件 |
|----------|------|
| `healthy` | レスポンスタイム < 500ms かつ エラー率 < 5% |
| `degraded` | レスポンスタイム < 2000ms かつ エラー率 < 20% |
| `unhealthy` | レスポンスタイム >= 2000ms または エラー率 >= 20% |

**使用例**
```
"fundamental_analysis"サーバーのヘルスチェックを実行してください
```

**注意事項**
- 履歴は最近100件の循環バッファで管理
- エラー率は最近100件のチェック結果から計算
- 定期的に実行することで、サーバーの健全性をモニタリングできます

---

#### logging_messages

対象MCPサーバーから送信されるログメッセージを取得します。

**説明**
対象MCPサーバーが送信する`notifications/message`プロトコルのログメッセージを取得します。ログレベル、時刻でフィルタリングできます。

**用途**
- サーバーログの収集
- エラーログの調査
- デバッグ情報の取得
- 本番環境のログモニタリング

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `level` | string | ❌ | 最小ログレベル（`debug`, `info`, `warning`, `error`等） | すべて |
| `limit` | integer | ❌ | 取得するログの最大件数 | `100` |
| `since` | string | ❌ | 開始時刻（RFC3339形式） | - |

**リクエストJSONの例**
```json
{
  "name": "logging_messages",
  "arguments": {
    "server": "fundamental_analysis",
    "level": "error",
    "limit": 50
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `server_name` | string | サーバー名 |
| `messages` | array | ログエントリの配列 |
| `messages[].timestamp` | string | タイムスタンプ（RFC3339形式） |
| `messages[].server_name` | string | サーバー名 |
| `messages[].level` | string | ログレベル |
| `messages[].logger` | string | ロガー名/コンポーネント名（オプション） |
| `messages[].message` | string | ログメッセージ |
| `total_count` | integer | 返却されたログ数 |

**成功時のレスポンスJSONの例**
```json
{
  "server_name": "fundamental_analysis",
  "messages": [
    {
      "timestamp": "2025-11-18T10:25:30Z",
      "server_name": "fundamental_analysis",
      "level": "error",
      "logger": "app.api_client",
      "message": "Failed to fetch data from external API: timeout"
    },
    {
      "timestamp": "2025-11-18T10:22:15Z",
      "server_name": "fundamental_analysis",
      "level": "error",
      "logger": "app.database",
      "message": "Database connection lost, retrying..."
    }
  ],
  "total_count": 2
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to get logging messages: Server not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未登録、ログバックエンド失敗）
- `-32602`: Invalid params（パラメータ不正、limitが負数、since形式不正）

**ログレベル一覧**

| レベル | 優先度 | 説明 |
|--------|--------|------|
| `debug` | 1 | デバッグメッセージ |
| `info` | 2 | 情報メッセージ |
| `notice` | 3 | 通知メッセージ |
| `warning` | 4 | 警告メッセージ |
| `error` | 5 | エラーメッセージ |
| `critical` | 6 | 致命的エラー |
| `alert` | 7 | アラート |
| `emergency` | 8 | 緊急事態 |

**使用例**

基本的な使用例:
```
"fundamental_analysis"サーバーのエラーレベル以上のログを最新50件取得してください
```

時刻フィルタリング:
```json
{
  "name": "logging_messages",
  "arguments": {
    "server": "fundamental_analysis",
    "level": "warning",
    "limit": 100,
    "since": "2025-11-18T00:00:00Z"
  }
}
```

**注意事項**
- `level`パラメータは最小レベルを指定（例: `error`を指定すると`error`以上のログが返却）
- ログバックエンド（Memory/Persistent）によって永続性が異なります
- サーバーが`logging` capabilityをサポートしていない場合、空配列が返却されます
- `max_logs`（デフォルト10,000件）を超えると古いログから削除されます

---

### 設定管理機能

#### config_add_server

サーバー設定を`.inspector/config.json`に追加します。

**説明**
新しいMCPサーバーの設定を追加します。AIエージェントから直接設定を操作できます。

**用途**
- 新しい検査対象サーバーの登録
- AIエージェントからの動的な設定追加

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `name` | string | ✅ | サーバー識別名（一意である必要がある） | - |
| `transport` | string | ✅ | トランスポートタイプ（現在は`stdio`のみ） | - |
| `command` | string | ✅ | 実行可能ファイルのフルパス | - |
| `args` | array | ❌ | コマンドライン引数 | `[]` |
| `env` | object | ❌ | 環境変数（キー・バリューのマップ） | `{}` |

**リクエストJSONの例**
```json
{
  "name": "config_add_server",
  "arguments": {
    "name": "my_new_server",
    "transport": "stdio",
    "command": "C:/path/to/server.exe",
    "args": ["--verbose"],
    "env": {
      "API_KEY": "your-api-key-here",
      "LOG_LEVEL": "debug"
    }
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `success` | boolean | 成功したかどうか |
| `message` | string | 成功メッセージ |

**成功時のレスポンスJSONの例**
```json
{
  "success": true,
  "message": "Server 'my_new_server' added successfully"
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to add server: Server 'my_new_server' already exists"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー名重複、設定ファイル書き込み失敗）
- `-32602`: Invalid params（パラメータ不正、nameが空文字列）

**使用例**
```
"my_new_server"という名前で"C:/path/to/server.exe"をMCP Inspectorに登録してください
```

**注意事項**
- サーバー名は一意である必要があります（重複する場合エラー）
- Windowsの場合、パスの`\`を`\\`にエスケープする必要があります
- 追加後、該当サーバーへの接続が可能になります

---

#### config_remove_server

サーバー設定を`.inspector/config.json`から削除します。

**説明**
登録済みのMCPサーバー設定を削除します。

**用途**
- 不要になったサーバーの削除
- 設定のクリーンアップ

**リクエスト形式**

| パラメータ名 | 型 | 必須 | 説明 | デフォルト値 |
|------------|-----|------|------|------------|
| `name` | string | ✅ | サーバー識別名 | - |

**リクエストJSONの例**
```json
{
  "name": "config_remove_server",
  "arguments": {
    "name": "my_new_server"
  }
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `success` | boolean | 成功したかどうか |
| `message` | string | 成功メッセージ |

**成功時のレスポンスJSONの例**
```json
{
  "success": true,
  "message": "Server 'my_new_server' removed successfully"
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to remove server: Server 'unknown_server' not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（サーバー未存在、設定ファイル書き込み失敗）
- `-32602`: Invalid params（パラメータ不正、nameが空文字列）

**使用例**
```
"my_new_server"をMCP Inspectorから削除してください
```

**注意事項**
- 削除後、該当サーバーへの接続はできなくなります
- 削除は永続的です（元に戻すには`config_add_server`で再登録）

---

#### config_list_servers

登録済みサーバーの一覧を`.inspector/config.json`から取得します。

**説明**
現在登録されているすべてのMCPサーバー設定を取得します。

**用途**
- 登録済みサーバーの確認
- 設定の確認

**リクエスト形式**

パラメータなし

**リクエストJSONの例**
```json
{
  "name": "config_list_servers",
  "arguments": {}
}
```

**レスポンス形式**

| フィールド名 | 型 | 説明 |
|------------|-----|------|
| `servers` | array | サーバー設定の配列 |
| `servers[].name` | string | サーバー識別名 |
| `servers[].transport` | string | トランスポートタイプ |
| `servers[].command` | string | 実行可能ファイルのパス |
| `servers[].args` | array | コマンドライン引数 |
| `servers[].env` | object | 環境変数 |
| `total_count` | integer | 登録済みサーバー数 |

**成功時のレスポンスJSONの例**
```json
{
  "servers": [
    {
      "name": "fundamental_analysis",
      "transport": "stdio",
      "command": "C:/projects/fa/target/release/fa.exe",
      "args": [],
      "env": {
        "API_KEY": "your-api-key-here"
      }
    },
    {
      "name": "technical_analysis",
      "transport": "stdio",
      "command": "C:/projects/ta/target/release/ta.exe",
      "args": ["--verbose"],
      "env": {}
    }
  ],
  "total_count": 2
}
```

**エラー時のレスポンスJSONの例**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to list servers: Configuration file not found"
  }
}
```

**エラーコード**
- `-32603`: Internal error（設定ファイル読み込み失敗）

**使用例**
```
MCP Inspectorに登録されているサーバーの一覧を教えてください
```

**注意事項**
- 環境変数に機密情報が含まれる場合があるため、共有時は注意してください

---

## 共通仕様

### リクエスト/レスポンスの共通ヘッダー

すべてのツールはJSON-RPC 2.0形式で通信します。

**リクエストフォーマット**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "tools_list",
    "arguments": {
      "server": "fundamental_analysis"
    }
  }
}
```

**レスポンスフォーマット（成功時）**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{...}"
      }
    ]
  }
}
```

**レスポンスフォーマット（エラー時）**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Failed to list tools: ...",
    "data": null
  }
}
```

### 認証方式

現在、認証機能は実装されていません。将来的にAPIキーやOAuth2のサポートを検討しています。

### エラーハンドリングの共通仕様

すべてのツールは以下のエラーハンドリング方針に従います。

1. **構造化されたエラーレポート**: v0.3.1でエラー情報が詳細化されました
2. **タイムアウト検出**: タイムアウト時は経過時間と設定値を返却
3. **Capability検証**: サーバーがサポートしていない機能へのアクセス時は警告を出力してベストエフォート実行
4. **エラータイプの分類**: Timeout, ServerCrash, ConnectionFailed等

**エラーレスポンスの`data`フィールド（v0.3.1以降）**
```json
{
  "error": {
    "code": -32603,
    "message": "Failed to call tool: Timeout",
    "data": {
      "type": "Timeout",
      "tool_name": "slow_operation",
      "elapsed_ms": 30500,
      "configured_timeout_ms": 30000,
      "suggestion": "Increase tool_timeout_ms in execution_config"
    }
  }
}
```

---

## エラーコード一覧

以下はMCP Inspector MCPで使用されるエラーコードの一覧です。

| エラーコード | 名前 | 説明 | 対処方法 |
|------------|------|------|---------|
| `-32700` | Parse error | JSONのパースエラー | リクエストのJSON形式を確認 |
| `-32600` | Invalid Request | 不正なリクエスト | リクエストフォーマットを確認 |
| `-32601` | Method not found | 指定されたメソッドが見つからない | ツール名を確認 |
| `-32602` | Invalid params | パラメータが不正 | パラメータの型と必須項目を確認 |
| `-32603` | Internal error | 内部エラー | エラーメッセージの詳細を確認 |

### `-32603` Internal errorの詳細分類

`-32603`は汎用的な内部エラーコードですが、`message`フィールドで詳細な原因を特定できます。

| メッセージパターン | 原因 | 対処方法 |
|------------------|------|---------|
| `Server '...' not found in configuration` | サーバーが未登録 | `config_add_server`で登録、または`config_list_servers`で確認 |
| `Failed to connect to server: ...` | 接続失敗 | サーバーのコマンドパスと起動確認 |
| `Timeout after ...ms` | タイムアウト | `execution_config.tool_timeout_ms`を延長 |
| `Server process terminated unexpectedly` | サーバークラッシュ | サーバーログを確認、バージョン確認 |
| `Configuration file not found` | 設定ファイルが見つからない | `.inspector/config.json`を作成 |
| `Server '...' already exists` | サーバー名重複 | 異なる名前を使用 |
| `JSON serialization error: ...` | JSONシリアライズエラー | レスポンスが不正（サーバー側の問題） |

---

## ベストプラクティス

API呼び出し時のベストプラクティス10項目を以下に示します。

### 1. キャッシュの活用

ツール一覧、リソース一覧、プロンプト一覧は5分間キャッシュされます。繰り返しアクセスする場合は、キャッシュが有効活用されます。

**推奨**: 初回に`tools_list`等を実行し、2回目以降は高速なレスポンスを享受してください。

### 2. タイムアウトの適切な設定

長時間実行されるツールを呼び出す場合、タイムアウトを延長してください。

**推奨設定**:
```json
{
  "execution_config": {
    "tool_timeout_ms": 120000,
    "connection_timeout_ms": 10000
  }
}
```

### 3. エラーハンドリングの実装

すべてのツール呼び出しでエラーハンドリングを実装してください。

**推奨**: エラーレスポンスの`data`フィールドを活用して詳細な診断を行う。

### 4. Capability検証

サーバーの`capabilities`を事前に確認し、サポートしていない機能へのアクセスを避けてください。

**推奨フロー**:
1. `server_inspect`でcapabilityを取得
2. `capabilities.tools.supported`が`true`の場合のみ`tools_list`や`tools_call`を実行

### 5. ログレベルの適切な選択

`logging_messages`ツールでは、必要なログレベルのみを取得してください。

**推奨**: 本番環境では`warning`以上、開発環境では`debug`以上

### 6. ログバックエンドの選択

用途に応じてログバックエンドを選択してください。

- **開発・テスト**: Memory Backend（高速）
- **本番環境**: Persistent Backend（永続化）

### 7. ヘルスチェックの定期実行

本番環境では定期的にヘルスチェックを実行してください。

**推奨頻度**: 1分〜5分ごと

### 8. 接続プーリングの恩恵を受ける

同じサーバーへの繰り返しアクセスでは、接続プーリングにより高速化されます。

**推奨**: サーバーを切り替えずに連続してツールを呼び出す

### 9. リトライの設定

ネットワークが不安定な環境では、リトライ設定を有効にしてください。

**推奨設定**:
```json
{
  "execution_config": {
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

### 10. セキュリティ対策

設定ファイルに機密情報を含める場合、アクセス権限を制限してください。

**推奨**:
- `.inspector/config.json`を`.gitignore`に追加
- 環境変数で機密情報を管理（将来実装予定）
- Persistent Backendのデータベースファイルのアクセス権限を制限

---

## バージョニング

### APIバージョン管理の方針

MCP Inspector MCPは、セマンティックバージョニング（Semantic Versioning）を採用しています。

**バージョンフォーマット**: `MAJOR.MINOR.PATCH`

- **MAJOR**: 互換性のない変更
- **MINOR**: 後方互換性のある機能追加
- **PATCH**: 後方互換性のあるバグ修正

### 後方互換性の保証

- **MINOR**および**PATCH**バージョンアップでは、既存のツールAPIは変更されません
- 新しいツールの追加は**MINOR**バージョンアップで行われます
- **MAJOR**バージョンアップでは、非推奨化されたAPIが削除される可能性があります

### プロトコルバージョン

MCP Inspector MCPは、Model Context Protocol (MCP) 2024-11-05に準拠しています。

将来的にMCPプロトコルのバージョンアップがあった場合、本サーバーも対応する予定です。

### バージョン履歴

| バージョン | リリース日 | 主な変更点 |
|----------|-----------|-----------|
| v0.3.1 | 2025-11-18 | エラーハンドリング強化、Capability検証、タイムアウト詳細レポート |
| v0.3.0 | 2025-11-15 | `.inspector/config.json`方式への移行、設定管理ツール追加 |
| v0.2.0 | 2025-11-10 | サーバー検査機能、パフォーマンス最適化 |
| v0.1.0 | 2025-11-01 | 初回リリース（Tools/Resources/Prompts/Sampling検査） |

---

**ドキュメント情報**

- **文字数**: 約22,000文字
- **ツール数**: 13ツール
- **エラーコード数**: 主要5コード + 詳細分類7パターン
- **最終更新**: 2025-11-18
- **バージョン**: v0.3.1

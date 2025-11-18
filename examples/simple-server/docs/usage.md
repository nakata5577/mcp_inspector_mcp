# Simple MCP Server - 詳細な使用方法

このドキュメントでは、Simple MCP Serverの詳細な使用方法、カスタマイズ方法、拡張のヒントについて説明します。

## 目次

1. [MCPプロトコルの基礎](#mcpプロトコルの基礎)
2. [各機能の詳細](#各機能の詳細)
3. [カスタマイズ方法](#カスタマイズ方法)
4. [拡張のヒント](#拡張のヒント)
5. [ベストプラクティス](#ベストプラクティス)

## MCPプロトコルの基礎

### JSON-RPC 2.0

MCPはJSON-RPC 2.0をベースとしています。基本的なメッセージ構造は以下の通りです:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method_name",
  "params": {}
}
```

### 必須メソッド

すべてのMCPサーバーは以下のメソッドを実装する必要があります:

- `initialize` - サーバーの初期化と機能の通知
- `tools/list` - 利用可能なツールのリスト
- `tools/call` - ツールの実行
- `resources/list` - 利用可能なリソースのリスト
- `resources/read` - リソースの読み取り
- `prompts/list` - 利用可能なプロンプトのリスト
- `prompts/get` - プロンプトの取得

## 各機能の詳細

### 1. Initialize

サーバーの初期化と機能の通知を行います。

**リクエスト**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "example-client",
      "version": "1.0.0"
    }
  }
}
```

**レスポンス**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {},
      "resources": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "simple-server",
      "version": "0.1.0"
    }
  }
}
```

### 2. Tools

#### Echo Tool

入力された文字列をそのまま返します。

**使用例**:
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"echo","arguments":{"message":"こんにちは、MCP！"}}}' | cargo run
```

**実装のポイント**:
- 入力検証: `message`フィールドが必須
- エラーハンドリング: 不正な入力に対して適切なエラーを返す

**拡張アイデア**:
- タイムスタンプの追加
- メッセージの履歴保存
- フォーマット変換（JSON、XML、YAMLなど）

#### Reverse Tool

文字列を逆順にします。

**使用例**:
```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"reverse","arguments":{"text":"Hello, World!"}}}' | cargo run
```

**実装のポイント**:
- Unicode文字の正しい処理
- 絵文字やサロゲートペアへの対応

**拡張アイデア**:
- 単語単位での逆順
- 行単位での逆順
- バイナリデータの逆順

#### Uppercase Tool

文字列を大文字に変換します。

**使用例**:
```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"uppercase","arguments":{"text":"hello, world!"}}}' | cargo run
```

**実装のポイント**:
- ロケールに依存しない変換
- ASCII以外の文字への対応

**拡張アイデア**:
- Lowercaseツールの追加
- Title Caseツールの追加
- カスタム変換ルールのサポート

### 3. Resources

#### Greeting Resource

挨拶メッセージとサーバー情報を提供します。

**使用例**:
```bash
echo '{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"simple://greeting"}}' | cargo run
```

**実装のポイント**:
- URIスキームの定義（`simple://`）
- MIMEタイプの指定
- 複数のリソースタイプのサポート

**拡張アイデア**:
- ファイルベースのリソース
- データベースから取得するリソース
- 動的に生成されるリソース
- テンプレートエンジンの統合

### 4. Prompts

#### Help Prompt

サーバーの使い方を説明します。

**使用例**:
```bash
echo '{"jsonrpc":"2.0","id":5,"method":"prompts/get","params":{"name":"help"}}' | cargo run
```

**実装のポイント**:
- 会話形式のメッセージ（user/assistant）
- 複数のメッセージの連鎖
- 動的な内容生成

**拡張アイデア**:
- パラメータ化されたプロンプト
- プロンプトのバージョン管理
- 多言語対応のプロンプト

## カスタマイズ方法

### 新しいツールの追加

新しいツールを追加するには、以下の手順を実行します:

#### 1. ツールリストに追加

`handle_tools_list`メソッドに新しいツールの定義を追加:

```rust
{
    "name": "my_custom_tool",
    "description": "カスタムツールの説明",
    "inputSchema": {
        "type": "object",
        "properties": {
            "param1": {
                "type": "string",
                "description": "パラメータ1の説明"
            }
        },
        "required": ["param1"]
    }
}
```

#### 2. ツール実装の追加

`handle_tools_call`メソッドに新しいツールの処理を追加:

```rust
"my_custom_tool" => {
    let param1 = arguments["param1"].as_str().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params: param1 is required".to_string(),
        data: None,
    })?;

    // カスタム処理
    let result = process_custom_tool(param1);

    json!({
        "content": [
            {
                "type": "text",
                "text": result
            }
        ]
    })
}
```

### 新しいリソースの追加

#### 1. リソースリストに追加

`handle_resources_list`メソッドに新しいリソースを追加:

```rust
{
    "uri": "simple://my_resource",
    "name": "my_resource",
    "description": "カスタムリソースの説明",
    "mimeType": "application/json"
}
```

#### 2. リソース読み取りの追加

`handle_resources_read`メソッドに新しいリソースの処理を追加:

```rust
"simple://my_resource" => Ok(json!({
    "contents": [
        {
            "uri": "simple://my_resource",
            "mimeType": "application/json",
            "text": "リソースの内容"
        }
    ]
}))
```

### 新しいプロンプトの追加

#### 1. プロンプトリストに追加

`handle_prompts_list`メソッドに新しいプロンプトを追加:

```rust
{
    "name": "my_prompt",
    "description": "カスタムプロンプトの説明",
    "arguments": [
        {
            "name": "topic",
            "description": "トピック",
            "required": true
        }
    ]
}
```

#### 2. プロンプト取得の追加

`handle_prompts_get`メソッドに新しいプロンプトの処理を追加:

```rust
"my_prompt" => {
    let topic = params["arguments"]["topic"].as_str().unwrap_or("一般");

    Ok(json!({
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!("{}について教えてください", topic)
                }
            }
        ]
    }))
}
```

## 拡張のヒント

### 1. 永続化の追加

SQLiteやファイルシステムを使用してデータを永続化:

```rust
use rusqlite::{Connection, Result as SqliteResult};

struct SimpleMcpServer {
    db: Connection,
}

impl SimpleMcpServer {
    fn new() -> SqliteResult<Self> {
        let db = Connection::open("simple_server.db")?;
        Ok(Self { db })
    }
}
```

### 2. 非同期処理の活用

重い処理を非同期で実行:

```rust
async fn handle_heavy_processing(&self, data: String) -> Result<String, JsonRpcError> {
    tokio::task::spawn_blocking(move || {
        // 重い処理
        process_data(data)
    })
    .await
    .map_err(|e| JsonRpcError {
        code: -32603,
        message: format!("Processing failed: {}", e),
        data: None,
    })
}
```

### 3. 設定ファイルの読み込み

外部設定ファイルからの設定読み込み:

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Config {
    server_name: String,
    version: String,
    timeout_seconds: u64,
}

impl SimpleMcpServer {
    fn load_config() -> Result<Config> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
```

### 4. ログ記録の追加

構造化ログの実装:

```rust
use tracing::{info, error, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt::init();

    info!("Simple MCP Server starting");

    // サーバー処理

    info!("Simple MCP Server stopped");
    Ok(())
}
```

### 5. メトリクスの収集

パフォーマンスメトリクスの収集:

```rust
use std::time::Instant;

struct Metrics {
    request_count: u64,
    total_duration: Duration,
}

impl SimpleMcpServer {
    fn handle_request_with_metrics(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let start = Instant::now();
        let response = self.handle_request(request);
        let duration = start.elapsed();

        self.metrics.request_count += 1;
        self.metrics.total_duration += duration;

        response
    }
}
```

## ベストプラクティス

### 1. エラーハンドリング

- すべてのエラーケースを処理する
- 適切なエラーコードとメッセージを返す
- エラーの詳細情報を提供する

```rust
fn validate_input(input: &str) -> Result<(), JsonRpcError> {
    if input.is_empty() {
        return Err(JsonRpcError {
            code: -32602,
            message: "Input cannot be empty".to_string(),
            data: Some(json!({"field": "input"})),
        });
    }
    Ok(())
}
```

### 2. 入力検証

- すべての入力を検証する
- 型安全性を確保する
- サニタイゼーションを実施する

```rust
fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}
```

### 3. パフォーマンス

- 不必要なクローンを避ける
- 適切なデータ構造を選択する
- キャッシュを活用する

```rust
use std::collections::HashMap;

struct CachedServer {
    cache: HashMap<String, String>,
}

impl CachedServer {
    fn get_or_compute(&mut self, key: &str) -> String {
        self.cache
            .entry(key.to_string())
            .or_insert_with(|| expensive_computation(key))
            .clone()
    }
}
```

### 4. テスト

- ユニットテストを書く
- 統合テストを書く
- エッジケースをテストする

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_tool() {
        let result = reverse_string("Hello");
        assert_eq!(result, "olleH");
    }

    #[test]
    fn test_empty_input() {
        let result = reverse_string("");
        assert_eq!(result, "");
    }
}
```

### 5. ドキュメンテーション

- コードコメントを書く
- API仕様を文書化する
- 使用例を提供する

```rust
/// 文字列を逆順にします。
///
/// # Arguments
///
/// * `text` - 逆順にする文字列
///
/// # Returns
///
/// 逆順にされた文字列
///
/// # Example
///
/// ```
/// let result = reverse_string("Hello");
/// assert_eq!(result, "olleH");
/// ```
fn reverse_string(text: &str) -> String {
    text.chars().rev().collect()
}
```

## まとめ

このドキュメントでは、Simple MCP Serverの詳細な使用方法とカスタマイズ方法について説明しました。

基本的な機能を理解したら、独自のツール、リソース、プロンプトを追加して、あなたのニーズに合わせてカスタマイズしてください。

より高度な機能や質問がある場合は、MCPの公式仕様書やRustのドキュメントを参照してください。

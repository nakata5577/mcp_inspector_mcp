# Mock Sampling Server

テスト用MCPサーバー。Samplingリクエスト（`sampling/createMessage`）を送信する機能を提供します。

## 目的

`MonitoringTransport`がSamplingリクエストを正しく検出・記録することを検証するためのモックサーバーです。

## ビルド

```bash
cd tests/mock_sampling_server
cargo build --release
```

## 実行

```bash
cargo run --release
```

## 提供ツール

### trigger_sampling

Samplingリクエストを送信するテストツール。

**パラメータ:**
- `message` (string): テストメッセージ（LLMに送信するユーザーメッセージ）

**動作:**
1. 指定されたメッセージで`sampling/createMessage`リクエストを構築
2. ホストに対してSamplingリクエストを送信
3. MonitoringTransportがこのリクエストを検出してログに記録

## 使用例

### MCP Inspectorから使用

```bash
# mcp_inspector_mcpからモックサーバーを検査
cargo run -- --config config/servers.toml

# trigger_samplingツールを実行
# （InspectorのCLIまたはMCP Inspector GUIから）
tools_call --server mock_sampling --tool trigger_sampling --arguments '{"message":"Hello, test!"}'
```

### MCP Inspector CLIから直接使用

```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --release --manifest-path tests/mock_sampling_server/Cargo.toml
```

その後、以下のコマンドでツールを実行:

```
tools/call trigger_sampling {"message": "Hello, this is a test!"}
```

## テスト検証ポイント

1. **Samplingリクエスト送信**: `trigger_sampling`ツールが正常に実行される
2. **MonitoringTransport検出**: `sampling/createMessage`メソッドが検出される
3. **ログ記録**: SamplingLoggerにリクエストが記録される
4. **sampling_logs取得**: Inspector経由でログが取得できる

## 技術仕様

- **Rust**: 2021 edition
- **rmcp**: 0.8.5
- **Transport**: stdio (child process)
- **Sampling API**: `CreateMessageRequestParam`を使用

## 制限事項

- テスト専用のモックサーバーです
- 実際のLLM呼び出しは実装していません（ホスト側で処理される想定）
- Samplingリクエストの送信機能のみを提供します

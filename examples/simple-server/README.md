# Simple MCP Server - サンプルプロジェクト

このサンプルは、MCP (Model Context Protocol) サーバーの基本的な実装例です。シンプルで理解しやすいコードで、MCPの主要な機能を実演します。

## 概要

Simple MCP Serverは以下の機能を提供します:

### ツール (Tools)
1. **echo** - 入力された文字列をそのまま返す
2. **reverse** - 入力された文字列を逆順にする
3. **uppercase** - 入力された文字列を大文字に変換する

### リソース (Resources)
- **simple://greeting** - 挨拶メッセージとサーバー情報を提供

### プロンプト (Prompts)
- **help** - サーバーの使い方を説明するヘルプメッセージ

## 前提条件

このサンプルを実行するには、以下が必要です:

- Rust 1.70.0 以上 (推奨: 最新の安定版)
- Cargo (Rustに同梱)
- MCP Inspector MCP (オプション: 詳細な検査用)

## セットアップ手順

### 1. 依存関係のインストール

```bash
# プロジェクトディレクトリに移動
cd examples/simple-server

# 依存関係をビルド
cargo build
```

### 2. 設定ファイルの確認

`.inspector/config.json` ファイルに以下の設定が含まれていることを確認:

```json
{
  "servers": {
    "simple-server": {
      "command": "cargo",
      "args": ["run"],
      "timeout_seconds": 30
    }
  }
}
```

## 実行方法

### 基本的な実行

サーバーを起動するには:

```bash
cargo run
```

サーバーは標準入力からJSON-RPCメッセージを受け取り、標準出力に応答を返します。

### 手動でのテスト

#### 1. Initialize

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run
```

期待される出力:
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

#### 2. ツールのリスト取得

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run
```

#### 3. echoツールの呼び出し

```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello, MCP!"}}}' | cargo run
```

期待される出力:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Hello, MCP!"
      }
    ]
  }
}
```

#### 4. reverseツールの呼び出し

```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"reverse","arguments":{"text":"Hello"}}}' | cargo run
```

期待される出力:
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "olleH"
      }
    ]
  }
}
```

#### 5. uppercaseツールの呼び出し

```bash
echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"uppercase","arguments":{"text":"hello"}}}' | cargo run
```

期待される出力:
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "HELLO"
      }
    ]
  }
}
```

#### 6. リソースの読み取り

```bash
echo '{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"simple://greeting"}}' | cargo run
```

#### 7. プロンプトの取得

```bash
echo '{"jsonrpc":"2.0","id":7,"method":"prompts/get","params":{"name":"help"}}' | cargo run
```

## テスト方法

### 自動統合テスト

#### Bash (Linux/macOS/Git Bash)

```bash
cd tests
chmod +x integration_test.sh
./integration_test.sh
```

#### PowerShell (Windows)

```powershell
cd tests
.\integration_test.ps1
```

統合テストは以下をチェックします:
- サーバーのビルドが成功すること
- Initialize が正しく応答すること
- 全ツールが正常に動作すること
- リソースが読み取れること
- プロンプトが取得できること

### MCP Inspector MCPを使用したテスト

MCP Inspector MCPを使用すると、より詳細な検査が可能です:

```bash
# MCP Inspector MCPのルートディレクトリから
cd examples/simple-server
cargo run --bin mcp-inspector -- inspect simple-server
```

## ディレクトリ構造

```
examples/simple-server/
├── README.md                    # このファイル
├── src/
│   └── main.rs                  # MCPサーバー実装
├── Cargo.toml                   # 依存関係定義
├── .inspector/
│   └── config.json              # MCP Inspector設定
├── tests/
│   ├── integration_test.sh      # Bash統合テスト
│   └── integration_test.ps1     # PowerShell統合テスト
└── docs/
    └── usage.md                 # 詳細な使用方法
```

## トラブルシューティング

### ビルドエラー

**問題**: `cargo build` が失敗する

**解決策**:
1. Rustのバージョンを確認: `rustc --version`
2. 最新版に更新: `rustup update`
3. 依存関係をクリーン: `cargo clean && cargo build`

### サーバーが応答しない

**問題**: サーバーが起動するが応答がない

**解決策**:
1. JSON-RPCメッセージの形式を確認
2. 標準エラー出力を確認してログを見る
3. タイムアウト設定を増やす (`.inspector/config.json`)

### テストスクリプトが失敗する

**問題**: 統合テストが失敗する

**解決策**:
1. `cargo build` を手動で実行して成功を確認
2. 各コマンドを個別に実行してどこで失敗するか特定
3. JSON出力を確認して期待値と比較

### パーミッションエラー (Linux/macOS)

**問題**: `permission denied` エラー

**解決策**:
```bash
chmod +x tests/integration_test.sh
```

## 次のステップ

このサンプルを理解したら、以下を試してみてください:

1. **新しいツールの追加**
   - 独自の文字列処理ツールを実装
   - 数学演算ツールを追加
   - ファイル操作ツールを追加

2. **リソースの拡張**
   - ファイルベースのリソースを追加
   - 動的なリソースを実装
   - 複数のリソースタイプをサポート

3. **エラーハンドリングの強化**
   - カスタムエラータイプを定義
   - より詳細なエラーメッセージを提供
   - リトライロジックを実装

4. **パフォーマンスの最適化**
   - 非同期処理の活用
   - キャッシュの実装
   - バッチ処理のサポート

## 参考資料

- [MCP仕様](https://spec.modelcontextprotocol.io/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- [Tokio公式ドキュメント](https://tokio.rs/)
- [Serde公式ドキュメント](https://serde.rs/)

## ライセンス

このサンプルコードは、MCP Inspector MCPプロジェクトと同じライセンスで提供されます。

## サポート

質問や問題が発生した場合は、MCP Inspector MCPのIssueトラッカーで報告してください。

# MCP Inspector MCP Server

MCP Inspectorの機能をMCPサーバーとして提供し、AIエージェントから他のMCPサーバーをデバッグ・検査できるようにするツールです。

## 概要

このプロジェクトは、[MCP Inspector](https://github.com/modelcontextprotocol/inspector)のCLIモードの機能をMCPサーバー化したものです。AIエージェント（Claude Desktopなど）から、他のMCPサーバーのツール一覧取得や実行が可能になります。

```
AIエージェント (Claude Desktop)
    ↓ MCP Protocol
本MCPサーバー (mcp_inspector_mcp)
    ↓ MCP Protocol
対象MCPサーバー (fundamental_analysis, filesystem, etc.)
```

## 機能

### MVP (Phase 1) - 現在実装済み
- ✅ `tools_list` - 対象MCPサーバーのツール一覧を取得
- ✅ `tools_call` - 対象MCPサーバーのツールを実行

### Phase 2 (計画中)
- ⏳ `resources_list` - リソース一覧取得
- ⏳ `resources_read` - リソース読み取り
- ⏳ `prompts_list` - プロンプト一覧取得
- ⏳ `prompts_get` - プロンプト取得

## セットアップ

### 1. ビルド

```bash
cargo build --release
```

### 2. 設定ファイルの作成

`config/servers.toml`で検査対象のMCPサーバーを定義します：

```toml
[[servers]]
name = "my-server"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]

[servers.env]
# 環境変数があれば追加
```

### 3. Claude Desktopへの登録

Claude Desktopの設定ファイル（`claude_desktop_config.json`）に追加：

**Windows:**
`%APPDATA%\Claude\claude_desktop_config.json`

**macOS:**
`~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\path\to\mcp_inspector_mcp\target\release\mcp_inspector_mcp.exe",
      "args": [],
      "env": {
        "MCP_INSPECTOR_CONFIG": "C:\path\to\mcp_inspector_mcp\config\servers.toml"
      }
    }
  }
}
```

**注意:** Windowsの場合、パスの`\`を`\`にエスケープしてください。

## 使用方法

Claude Desktopから以下のように使用します：

### ツール一覧の取得

```
対象サーバー"fundamental_analysis"のツール一覧を取得してください
```

内部的には以下のツールが呼ばれます：
```json
{
  "name": "tools_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

### ツールの実行

```
対象サーバー"fundamental_analysis"のツール"calculate_rsi"を
引数 {"symbol": "AAPL", "period": 14} で実行してください
```

内部的には以下のツールが呼ばれます：
```json
{
  "name": "tools_call",
  "arguments": {
    "server": "fundamental_analysis",
    "tool_name": "calculate_rsi",
    "arguments": {
      "symbol": "AAPL",
      "period": 14
    }
  }
}
```

## 提供するツール

### tools_list

対象MCPサーバーが提供するツールの一覧を取得します。

**引数:**
- `server` (string, required): 対象サーバー名（config/servers.tomlで定義）

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "tools": [
    {
      "name": "tool_name",
      "description": "Tool description",
      "input_schema": { ... }
    }
  ]
}
```

### tools_call

対象MCPサーバーの特定のツールを実行します。

**引数:**
- `server` (string, required): 対象サーバー名
- `tool_name` (string, required): 実行するツール名
- `arguments` (object, optional): ツールに渡す引数

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "tool_name": "calculate_rsi",
  "result": { ... }
}
```

## トラブルシューティング

### サーバーが見つからない

**エラー:** "Server not found: xxx"

**解決策:**
1. `config/servers.toml`に対象サーバーが定義されているか確認
2. サーバー名のスペルミスがないか確認

### 接続エラー

**エラー:** "Failed to connect to server: xxx"

**解決策:**
1. 対象サーバーのコマンドパスが正しいか確認
2. 対象サーバーが正常に起動できるか、直接実行して確認
   ```bash
   cargo run --release --manifest-path ../fundamental_analysis/Cargo.toml
   ```
3. 環境変数が正しく設定されているか確認

### ログの確認

環境変数`RUST_LOG`を設定すると詳細なログが出力されます：

```bash
RUST_LOG=debug cargo run --release
```

## 技術スタック

- **Rust** - システムプログラミング言語
- **rmcp 0.8.5** - MCP Rust SDK
- **tokio** - 非同期ランタイム
- **serde** - シリアライゼーション/デシリアライゼーション

## アーキテクチャ

```
src/
├── main.rs              # エントリーポイント
├── lib.rs               # ライブラリルート
├── server/              # MCPサーバー実装
│   └── mod.rs          # ツール定義とハンドラー
├── client/              # MCPクライアント実装
│   ├── mod.rs
│   ├── manager.rs      # クライアント管理
│   └── stdio_client.rs # Stdioトランスポート
├── services/            # ビジネスロジック
│   ├── mod.rs
│   └── inspector.rs    # Inspector機能
└── models/              # データ構造
    ├── mod.rs
    ├── error.rs        # エラー型
    ├── request.rs      # リクエスト型
    ├── response.rs     # レスポンス型
    └── server_config.rs # サーバー設定
```

## 開発

### コードチェック

```bash
# コンパイルチェック
cargo check

# リンターチェック
cargo clippy

# テスト実行
cargo test
```

### デバッグ実行

```bash
RUST_LOG=debug cargo run
```

## ライセンス

MIT License

## 参考

- [Model Context Protocol](https://modelcontextprotocol.io/)
- [MCP Inspector](https://github.com/modelcontextprotocol/inspector)
- [rmcp - Rust MCP SDK](https://docs.rs/rmcp/)

# Getting Started with MCP Inspector MCP

**所要時間**: 約30分
**対象**: 完全な初心者
**目標**: インストールから初回実行まで、基本的な使い方を習得する

---

## 目次

1. [前提条件](#前提条件)
2. [インストール](#インストール)
3. [初期設定](#初期設定)
4. [初回実行](#初回実行)
5. [基本操作](#基本操作)
6. [トラブルシューティング](#トラブルシューティング)
7. [次のステップ](#次のステップ)

---

## 前提条件

MCP Inspector MCPを使い始める前に、以下の環境を準備してください。

### 必要なソフトウェア

#### 1. Rustツールチェイン（必須）

MCP Inspector MCPはRust言語で書かれているため、Rustのインストールが必要です。

**Windows:**
1. [rustup公式サイト](https://rustup.rs/)にアクセス
2. `rustup-init.exe`をダウンロードして実行
3. インストーラーの指示に従い、デフォルト設定でインストール
4. PowerShellまたはコマンドプロンプトを再起動

**macOS/Linux:**
```bash
# ターミナルで以下のコマンドを実行
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# PATHを有効化
source $HOME/.cargo/env
```

**インストール確認:**
```bash
rustc --version
# 出力例: rustc 1.70.0 (xxxxxxx 2023-xx-xx)

cargo --version
# 出力例: cargo 1.70.0 (xxxxxxx 2023-xx-xx)
```

**最低バージョン**: Rust 1.70以上

#### 2. Claude Desktop（推奨）

AIエージェントからMCP Inspector MCPを使用するには、MCP対応のクライアントが必要です。

- **公式サイト**: https://claude.ai/
- **対応OS**: Windows, macOS, Linux

**注意**: Claude Desktop以外のMCP対応クライアントでも使用できますが、このチュートリアルではClaude Desktopを前提とします。

#### 3. 検査対象のMCPサーバー（任意）

MCP Inspector MCPで検査する対象のMCPサーバーが必要です。
自作のMCPサーバーがない場合は、[MCP Servers Collection](https://github.com/modelcontextprotocol/servers)から選択できます。

**簡易テスト用**: このチュートリアルでは、まず設定だけを確認し、後で実際のサーバーを追加します。

### 環境要件

| 項目 | 要件 |
|------|------|
| **OS** | Windows 10/11, macOS 10.15+, Linux（主要ディストリビューション） |
| **メモリ** | 最低512MB、推奨1GB以上 |
| **ディスク** | 約100MB（ビルド時は追加で1GB程度） |
| **ネットワーク** | インターネット接続（初回ビルド時に依存関係をダウンロード） |

### 事前知識

- **必須**: なし（このチュートリアルで基本から説明します）
- **推奨**: コマンドラインの基本操作（cd, mkdir, 等）
- **あると便利**: JSON形式の基本的な理解

---

## インストール

### Step 1: リポジトリのクローン

まず、MCP Inspector MCPのソースコードを取得します。

```bash
# 任意のディレクトリに移動（例: Windowsの場合）
cd C:\Users\YourName\projects

# macOS/Linuxの場合
cd ~/projects

# リポジトリをクローン
git clone https://github.com/yourusername/mcp_inspector_mcp.git

# ディレクトリに移動
cd mcp_inspector_mcp
```

**Gitがない場合**:
- [Git公式サイト](https://git-scm.com/)からダウンロードしてインストール
- または、GitHubからZIPファイルをダウンロードして解凍

### Step 2: ビルド

Rustのビルドシステム（Cargo）を使ってプロジェクトをビルドします。

```bash
# リリースビルド（最適化有効）
cargo build --release
```

**初回ビルドの注意点**:
- 初回は依存関係のダウンロードとコンパイルで**5〜10分**かかります
- インターネット接続が必要です
- 進行状況が表示されるので、完了まで待ちましょう

**ビルド成功の確認**:
```bash
# Windowsの場合
ls target\release\mcp_inspector_mcp.exe

# macOS/Linuxの場合
ls -l target/release/mcp_inspector_mcp
```

実行可能ファイルが存在すれば成功です。

### Step 3: 動作確認

ビルドが成功したら、簡単な動作確認をします。

```bash
# Windowsの場合
.\target\release\mcp_inspector_mcp.exe --help

# macOS/Linuxの場合
./target/release/mcp_inspector_mcp --help
```

**期待される出力**:
```
MCP Inspector MCP Server

USAGE:
    mcp_inspector_mcp [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

ヘルプメッセージが表示されればインストール成功です！

---

## 初期設定

### Step 1: 設定ファイルの自動生成

初回起動時、設定ファイル`.inspector/config.json`が自動生成されます。

```bash
# Windowsの場合
cd target\release
.\mcp_inspector_mcp.exe

# macOS/Linuxの場合
cd target/release
./mcp_inspector_mcp
```

起動したら、すぐに `Ctrl+C` で停止してください。

**生成された設定ファイル**:
```
target/release/.inspector/config.json
```

### Step 2: 設定ファイルの確認

生成された設定ファイルを開いてみましょう。

**Windows（メモ帳で開く）**:
```bash
notepad .inspector\config.json
```

**macOS/Linux（標準エディタで開く）**:
```bash
cat .inspector/config.json
# または
nano .inspector/config.json
```

**デフォルトの設定内容**:
```json
{
  "servers": [],
  "logging": {
    "backend": "memory",
    "db_path": "./data/logs.db",
    "max_logs": 10000
  }
}
```

**設定項目の説明**:
- `servers`: 検査対象のMCPサーバーのリスト（現在は空）
- `logging.backend`: ログの保存方法（`memory`=メモリ内、`persistent`=ディスク）
- `logging.db_path`: ログデータベースのパス（persistent時のみ使用）
- `logging.max_logs`: サーバーごとの最大ログ保存件数

### Step 3: MCPサーバーの追加

検査対象のMCPサーバーを設定ファイルに追加します。

**例: 自作のMCPサーバーを追加する場合**

`.inspector/config.json`を編集します:

**Windows版の設定例**:
```json
{
  "servers": [
    {
      "name": "my_first_server",
      "transport": "stdio",
      "command": "C:\\Users\\YourName\\projects\\my_mcp_server\\target\\release\\my_mcp_server.exe",
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

**macOS/Linux版の設定例**:
```json
{
  "servers": [
    {
      "name": "my_first_server",
      "transport": "stdio",
      "command": "/Users/yourname/projects/my_mcp_server/target/release/my_mcp_server",
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

**設定項目の詳細**:

| 項目 | 必須 | 説明 | 例 |
|------|------|------|-----|
| `name` | ✅ | サーバーの識別名（一意） | `"my_first_server"` |
| `transport` | ✅ | トランスポートタイプ（現在は`stdio`のみ） | `"stdio"` |
| `command` | ✅ | 実行可能ファイルのフルパス | `"C:\\path\\to\\server.exe"` |
| `args` | ❌ | コマンドライン引数（配列） | `["--verbose", "--debug"]` |
| `env` | ❌ | 環境変数（オブジェクト） | `{"API_KEY": "xxx"}` |

**重要なポイント**:
1. **Windows**: パスの`\`は`\\`にエスケープが必要
2. **name**: 一意である必要があります（重複すると動作しません）
3. **command**: フルパス（絶対パス）で指定

### Step 4: Claude Desktopへの登録

MCP Inspector MCPをClaude Desktopから使えるようにします。

#### 設定ファイルの場所

**Windows**:
```
%APPDATA%\Claude\claude_desktop_config.json
```
実際のパス例: `C:\Users\YourName\AppData\Roaming\Claude\claude_desktop_config.json`

**macOS**:
```
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Linux**:
```
~/.config/Claude/claude_desktop_config.json
```

#### 設定ファイルの編集

**Windows版の設定例**:
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\\Users\\YourName\\projects\\mcp_inspector_mcp\\target\\release\\mcp_inspector_mcp.exe"
    }
  }
}
```

**macOS/Linux版の設定例**:
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "/Users/yourname/projects/mcp_inspector_mcp/target/release/mcp_inspector_mcp"
    }
  }
}
```

**既に他のMCPサーバーが登録されている場合**:
```json
{
  "mcpServers": {
    "existing-server": {
      "command": "/path/to/existing/server"
    },
    "mcp-inspector": {
      "command": "/path/to/mcp_inspector_mcp"
    }
  }
}
```

#### Claude Desktopの再起動

設定を反映させるため、**Claude Desktopを完全に終了して再起動**してください。

**Windows**:
1. タスクバーのClaude Desktopアイコンを右クリック
2. 「終了」をクリック
3. Claude Desktopを再度起動

**macOS**:
1. `Cmd + Q` でClaude Desktopを完全終了
2. Claude Desktopを再度起動

**Linux**:
1. Claude Desktopを終了
2. Claude Desktopを再度起動

---

## 初回実行

Claude Desktopから初めてMCP Inspector MCPを使ってみましょう。

### Step 1: サーバー一覧の確認

Claude Desktopのチャット画面で以下のように指示します。

**プロンプト例**:
```
mcp-inspectorに登録されているサーバーの一覧を教えてください
```

**期待される応答**:
```
登録されているサーバー:
1. my_first_server
   - Transport: stdio
   - Command: C:\Users\YourName\projects\my_mcp_server\target\release\my_mcp_server.exe
```

**内部で実行されているツール**:
- `config_list_servers` - 設定ファイルからサーバー一覧を取得

**もし空の結果が返ってきた場合**:
- `.inspector/config.json`の`servers`配列が空である可能性があります
- [Step 3: MCPサーバーの追加](#step-3-mcpサーバーの追加)に戻って設定を確認してください

### Step 2: サーバーの検査

登録したサーバーの詳細情報を取得してみましょう。

**プロンプト例**:
```
"my_first_server"の詳細情報を取得してください
```

**期待される応答**:
```json
{
  "server_name": "my_first_server",
  "implementation": {
    "name": "my-mcp-server",
    "version": "1.0.0"
  },
  "capabilities": {
    "tools": {
      "supported": true
    },
    "resources": {
      "supported": false
    },
    "prompts": {
      "supported": false
    }
  },
  "connection_status": "connected",
  "protocol_version": "2024-11-05"
}
```

**内部で実行されているツール**:
- `server_inspect` - サーバーのcapabilityと設定情報を取得

**このレスポンスから分かること**:
- サーバーが正常に接続されている
- サポートしている機能（Tools, Resources, Prompts）
- MCPプロトコルのバージョン

### Step 3: ヘルスチェック

サーバーの健康状態を確認します。

**プロンプト例**:
```
"my_first_server"のヘルスチェックを実行してください
```

**期待される応答**:
```json
{
  "server_name": "my_first_server",
  "status": "healthy",
  "response_time_ms": 45,
  "last_check": "2025-11-18T12:00:00Z",
  "error_count": 0,
  "error_rate": 0.0
}
```

**内部で実行されているツール**:
- `health_check` - Pingによる疎通確認とパフォーマンス測定

**ステータスの意味**:
- **healthy**: レスポンスタイム < 500ms かつ エラー率 < 5%
- **degraded**: レスポンスタイム < 2000ms かつ エラー率 < 20%
- **unhealthy**: レスポンスタイム >= 2000ms または エラー率 >= 20%

---

## 基本操作

### 1. ツール一覧の取得（tools_list）

サーバーが提供しているツールの一覧を取得します。

**プロンプト例**:
```
"my_first_server"が提供しているツールの一覧を取得してください
```

**期待される応答例**:
```json
{
  "server": "my_first_server",
  "tools": [
    {
      "name": "calculate_sum",
      "description": "2つの数値を足し算します",
      "input_schema": {
        "type": "object",
        "properties": {
          "a": {
            "type": "number",
            "description": "1つ目の数値"
          },
          "b": {
            "type": "number",
            "description": "2つ目の数値"
          }
        },
        "required": ["a", "b"]
      }
    }
  ]
}
```

**この情報から分かること**:
- ツール名: `calculate_sum`
- 説明: 2つの数値を足し算する
- 必要な引数: `a`と`b`（両方とも必須）

### 2. ツールの実行（tools_call）

取得したツール一覧から、実際にツールを実行してみます。

**プロンプト例**:
```
"my_first_server"の"calculate_sum"ツールを使って、5と3を足し算してください
```

**期待される応答例**:
```json
{
  "server": "my_first_server",
  "tool_name": "calculate_sum",
  "result": {
    "sum": 8
  }
}
```

**Claude Desktopが内部で実行していること**:
1. ユーザーの指示を解釈
2. `tools_call`ツールを呼び出し
3. 以下のパラメータを渡す:
   ```json
   {
     "server": "my_first_server",
     "tool_name": "calculate_sum",
     "arguments": {
       "a": 5,
       "b": 3
     }
   }
   ```

### 3. リソース一覧の取得（resources_list）

サーバーが提供しているリソースを確認します。

**プロンプト例**:
```
"my_first_server"が提供しているリソースの一覧を教えてください
```

**期待される応答例**:
```json
{
  "server": "my_first_server",
  "resources": [
    {
      "uri": "file:///data/sample.txt",
      "name": "Sample Data",
      "description": "サンプルデータファイル",
      "mime_type": "text/plain"
    }
  ]
}
```

**リソースとは**:
- AIが参照できるデータ（ファイル、データベース等）
- URIで識別される
- ツールとは異なり、実行ではなく読み取り専用

### 4. リソースの読み取り（resources_read）

リソースの内容を読み込みます。

**プロンプト例**:
```
"my_first_server"の"file:///data/sample.txt"リソースを読み込んでください
```

**期待される応答例**:
```json
{
  "server": "my_first_server",
  "uri": "file:///data/sample.txt",
  "contents": [
    {
      "uri": "file:///data/sample.txt",
      "mime_type": "text/plain",
      "text": "This is sample data.\nLine 2.\nLine 3.",
      "blob": null
    }
  ]
}
```

### 5. プロンプトテンプレート一覧の取得（prompts_list）

サーバーが提供しているプロンプトテンプレートを確認します。

**プロンプト例**:
```
"my_first_server"が提供しているプロンプトテンプレートの一覧を取得してください
```

**期待される応答例**:
```json
{
  "server": "my_first_server",
  "prompts": [
    {
      "name": "greeting",
      "description": "挨拶メッセージを生成",
      "arguments": [
        {
          "name": "name",
          "description": "挨拶する相手の名前",
          "required": true
        }
      ]
    }
  ]
}
```

---

## トラブルシューティング

### エラー1: "Server not found"

**エラーメッセージ**:
```
Server not found: my_first_server
```

**原因**:
- `.inspector/config.json`に該当サーバーが登録されていない
- サーバー名のスペルミス

**解決方法**:
1. `.inspector/config.json`を開く
2. `servers`配列に該当サーバーが存在するか確認
3. サーバー名が正確に一致しているか確認（大文字小文字を含む）

### エラー2: "Failed to connect to server"

**エラーメッセージ**:
```
Failed to connect to server: my_first_server
```

**原因**:
- サーバーの実行可能ファイルのパスが間違っている
- サーバーが起動できない（依存関係の欠如、権限不足等）

**解決方法**:
1. コマンドラインから直接サーバーを実行して動作確認:
   ```bash
   # Windowsの場合
   C:\path\to\my_mcp_server.exe

   # macOS/Linuxの場合
   /path/to/my_mcp_server
   ```
2. エラーメッセージを確認し、問題を解決
3. `.inspector/config.json`のパスが正確か確認

### エラー3: "Connection timeout"

**エラーメッセージ**:
```
Connection timeout after 5000ms
```

**原因**:
- サーバーの起動が遅い
- サーバーが応答していない

**解決方法**:
1. `.inspector/config.json`でタイムアウトを延長:
   ```json
   {
     "servers": [...],
     "logging": {...},
     "execution_config": {
       "connection_timeout_ms": 15000
     }
   }
   ```
2. Claude Desktopを再起動

### エラー4: "Tool execution timeout"

**エラーメッセージ**:
```json
{
  "error": {
    "type": "Timeout",
    "tool_name": "slow_operation",
    "elapsed_ms": 30500
  }
}
```

**原因**:
- ツールの実行時間が30秒（デフォルト）を超えている

**解決方法**:
1. `.inspector/config.json`でツール実行タイムアウトを延長:
   ```json
   {
     "execution_config": {
       "tool_timeout_ms": 120000
     }
   }
   ```
2. Claude Desktopを再起動

### 問題5: Claude Desktopで認識されない

**症状**:
- Claude Desktopでmcp-inspectorのツールが表示されない

**解決方法**:
1. Claude Desktopの設定ファイルを再確認:
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
2. JSON形式が正しいか確認（[JSONLint](https://jsonlint.com/)等で検証）
3. パスが正確か確認（フルパス、エスケープ処理）
4. Claude Desktopを**完全に終了して再起動**

---

## 次のステップ

基本操作を習得したら、次のレベルに進みましょう！

### 1. Practical Guide（実践ガイド）

実務で使える実践的なテクニックを学べます:
- 複数サーバーの管理方法
- デバッグのベストプラクティス
- ログ分析とトラブルシューティング
- パフォーマンス測定とチューニング

**所要時間**: 約1時間
**ドキュメント**: [docs/tutorials/practical-guide.md](practical-guide.md)

### 2. Advanced Usage（高度な使い方）

上級者向けの高度な機能を学べます:
- カスタム設定の詳細
- Capability検証の活用
- CI/CD統合
- セキュリティ考慮事項

**所要時間**: 約1〜2時間
**ドキュメント**: [docs/tutorials/advanced-usage.md](advanced-usage.md)

### 3. コミュニティに参加

- **Issue報告**: バグや機能要望を報告
- **ディスカッション**: 質問や提案を共有
- **コントリビューション**: プルリクエストを送信

**GitHubリポジトリ**: https://github.com/yourusername/mcp_inspector_mcp

---

## まとめ

このチュートリアルでは、以下を学びました:

✅ Rustのインストールとビルド方法
✅ `.inspector/config.json`の基本的な設定
✅ Claude Desktopへの登録方法
✅ 基本的なツールの使い方（tools_list, tools_call, health_check）
✅ よくあるエラーの解決方法

**次のステップ**: [Practical Guide](practical-guide.md)で実践的なテクニックを習得しましょう！

---

**最終更新**: 2025-11-18
**バージョン**: v0.3.1
**フィードバック**: [GitHub Issues](https://github.com/yourusername/mcp_inspector_mcp/issues)

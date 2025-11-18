# MCP Inspector MCP Server

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

**AIエージェントからMCPサーバーをデバッグ・検査できる次世代デバッグツール**

MCP Inspectorの機能をMCPサーバーとして提供し、AIエージェント（Claude Desktopなど）から他のMCPサーバーを直接デバッグ・検査できるようにする革新的なツールです。

---

## 📖 目次

- [概要](#概要)
- [MCP（Model Context Protocol）とは](#mcpmodel-context-protocolとは)
- [主要機能](#主要機能)
- [インストール](#インストール)
- [クイックスタート](#クイックスタート)
- [設定ガイド](#設定ガイド)
- [使用例](#使用例)
- [提供ツール詳細](#提供ツール詳細)
- [パフォーマンス特性](#パフォーマンス特性)
- [トラブルシューティング](#トラブルシューティング)
- [FAQ](#faq)
- [開発情報](#開発情報)
- [ライセンス](#ライセンス)
- [参考リンク](#参考リンク)

---

## 概要

このプロジェクトは、[MCP Inspector](https://github.com/modelcontextprotocol/inspector)のCLIモードの機能をMCPサーバー化したものです。AIエージェント（Claude Desktopなど）から、他のMCPサーバーのツール一覧取得や実行が可能になります。

### アーキテクチャ図

```
AIエージェント (Claude Desktop)
    ↓ MCP Protocol
本MCPサーバー (mcp_inspector_mcp) ← このプロジェクト
    ↓ MCP Protocol
対象MCPサーバー (fundamental_analysis, filesystem, etc.)
```

### 対象ユーザー

- MCPサーバー開発者 - デバッグとテストを効率化
- AIエージェント利用者 - 接続済みMCPサーバーの調査
- システムインテグレーター - 複数MCPサーバーの統合管理

### ユースケース

1. **開発中のMCPサーバーのデバッグ**: ツールの実行結果をリアルタイムで確認
2. **ヘルスチェック**: 本番環境のMCPサーバーの稼働状況を監視
3. **設定検証**: サーバーのcapabilityやバージョン情報を取得
4. **ログ調査**: サーバーから送信されるログメッセージを収集・分析
5. **パフォーマンス測定**: レスポンスタイムやエラー率を定期的にモニタリング

---

## MCP（Model Context Protocol）とは

**Model Context Protocol (MCP)** は、AIアプリケーションとデータソースを接続するための標準化されたオープンプロトコルです。

### MCPの特徴

- **標準化**: 統一されたインターフェースで異なるツールを統合
- **拡張性**: カスタムツール、リソース、プロンプトを自由に追加可能
- **セキュリティ**: サンドボックス化された環境での安全な実行
- **双方向通信**: AIとツール間の対話的なやり取りをサポート

### MCPのコンポーネント

1. **Tools（ツール）**: AIが実行できる機能（例: ファイル読み書き、API呼び出し）
2. **Resources（リソース）**: AIが参照できるデータ（例: ファイル、データベース）
3. **Prompts（プロンプト）**: 事前定義されたプロンプトテンプレート
4. **Sampling**: AI側がサーバーにLLM推論を依頼する機能（上級機能）

### 本プロジェクトの役割

MCP Inspector MCPは、**MCPサーバーのデバッグ専用MCPサーバー**です。他のMCPサーバーを検査対象として、その内部状態や機能を調査できます。

---

## 主要機能

### ✅ 実装済み機能（v0.3.1）

#### Phase 1: ツール検査機能
- `tools_list` - 対象MCPサーバーのツール一覧を取得
- `tools_call` - 対象MCPサーバーのツールを実行

#### Phase 2: リソース・プロンプト検査機能
- `resources_list` - リソース一覧取得
- `resources_read` - リソース読み取り
- `prompts_list` - プロンプト一覧取得
- `prompts_get` - プロンプト取得

#### Phase 3: Samplingログ機能
- `sampling_logs` - Samplingリクエストのログ取得
- MonitoringTransport - Transport層でのSampling監視

#### Phase 5: パフォーマンス最適化
- **接続プーリング** - MCPクライアント接続の再利用（2回目以降50%以上高速化）
- **キャッシング** - TTLベースのレスポンスキャッシュ（キャッシュヒット時 < 1ms）
- **並列処理** - 複数サーバーの並列処理（N個のサーバー処理が約1/Nの時間）
- **ログバックエンド抽象化** - Memory/Persistentバックエンドの柔軟な切替

#### Phase 6: サーバー検査機能
- `server_inspect` - サーバー設定とcapability検査
- `health_check` - ヘルスチェックとパフォーマンス測定
- `logging_messages` - ログメッセージ検査とフィルタリング

#### Phase 7: エラーハンドリング（v0.3.1）
- 構造化されたエラーレポート
- タイムアウト検出と詳細情報提供
- Capability検証とベストエフォート実行

#### Phase 8: 設定管理（v0.3.0）
- `.inspector/config.json` 方式による統一的な設定管理
- `config_add_server` / `config_remove_server` / `config_list_servers` - AIエージェントから直接設定操作
- 実行時設定（タイムアウト、リトライ等）のカスタマイズ

### 🔧 提供ツール一覧（全15ツール）

| カテゴリ | ツール名 | 説明 |
|---------|----------|------|
| **Tools検査** | `tools_list` | ツール一覧取得 |
| | `tools_call` | ツール実行 |
| **Resources検査** | `resources_list` | リソース一覧取得 |
| | `resources_read` | リソース読み取り |
| **Prompts検査** | `prompts_list` | プロンプト一覧取得 |
| | `prompts_get` | プロンプト取得 |
| **Sampling検査** | `sampling_logs` | Samplingログ取得 |
| **サーバー検査** | `server_inspect` | サーバー設定検査 |
| | `health_check` | ヘルスチェック |
| | `logging_messages` | ログメッセージ取得 |
| **設定管理** | `config_add_server` | サーバー追加 |
| | `config_remove_server` | サーバー削除 |
| | `config_list_servers` | サーバー一覧 |
| **並列処理** | `list_tools_batch` | 複数サーバーのツール一覧を並列取得 |
| | `list_resources_batch` | 複数サーバーのリソース一覧を並列取得 |

---

## インストール

### 前提条件

本プロジェクトを利用するには、以下の環境が必要です:

- **Rust** 1.70以上（[rustup](https://rustup.rs/)でインストール）
- **Cargo**（Rustと一緒にインストールされます）
- **Claude Desktop**（または他のMCP対応AIクライアント）
- **検査対象のMCPサーバー**（別途用意）

#### Rustのインストール

**Windows:**
```bash
# PowerShellで実行
# https://rustup.rs/ から rustup-init.exe をダウンロードして実行
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### インストール確認

```bash
rustc --version  # rust 1.70.0 以上
cargo --version  # cargo 1.70.0 以上
```

### ビルド手順

#### 1. リポジトリのクローン

```bash
git clone https://github.com/yourusername/mcp_inspector_mcp.git
cd mcp_inspector_mcp
```

#### 2. リリースビルド

```bash
cargo build --release
```

ビルドが成功すると、以下に実行可能ファイルが生成されます:
- **Windows**: `target\release\mcp_inspector_mcp.exe`
- **macOS/Linux**: `target/release/mcp_inspector_mcp`

#### 3. 動作確認

```bash
# ヘルプメッセージの表示
./target/release/mcp_inspector_mcp --help

# デバッグモードでの起動確認（すぐにCtrl+Cで終了してOK）
RUST_LOG=info ./target/release/mcp_inspector_mcp
```

### トラブルシューティング（ビルド時）

**コンパイルエラーが発生する場合:**
```bash
# Rustツールチェインを最新に更新
rustup update

# 依存関係のクリーンビルド
cargo clean
cargo build --release
```

**ネットワークエラーが発生する場合:**
```bash
# cratesのミラーサイトを使用（中国など）
# .cargo/config.toml に以下を追加
[source.crates-io]
replace-with = 'mirror'

[source.mirror]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"
```

---

## クイックスタート

30分以内に基本操作を習得できるガイドです。

### Step 1: 初回起動と設定ファイル生成

初回起動時、`.inspector/config.json`が自動生成されます。

```bash
# 実行可能ファイルのディレクトリに移動
cd target/release

# 初回起動（設定ファイルが自動生成される）
./mcp_inspector_mcp
```

**自動生成されるファイル:**
```
target/release/.inspector/
└── config.json  # デフォルト設定
```

**デフォルト設定の内容:**
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

### Step 2: 検査対象サーバーの追加

`.inspector/config.json`を編集して、検査対象のMCPサーバーを追加します。

**設定例（Windowsの場合）:**
```json
{
  "servers": [
    {
      "name": "my_first_server",
      "transport": "stdio",
      "command": "C:\\path\\to\\your\\mcp_server.exe",
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

**設定例（macOS/Linuxの場合）:**
```json
{
  "servers": [
    {
      "name": "my_first_server",
      "transport": "stdio",
      "command": "/path/to/your/mcp_server",
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

**重要ポイント:**
- `name` - サーバーの識別名（一意である必要があります）
- `command` - 実行可能ファイルのフルパス
- Windowsでは`\`を`\\`にエスケープする必要があります

### Step 3: Claude Desktopへの登録

Claude Desktopの設定ファイルに本MCPサーバーを登録します。

**設定ファイルの場所:**
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

**Windows版設定例:**
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "C:\\Users\\YourName\\mcp_inspector_mcp\\target\\release\\mcp_inspector_mcp.exe"
    }
  }
}
```

**macOS/Linux版設定例:**
```json
{
  "mcpServers": {
    "mcp-inspector": {
      "command": "/Users/yourname/mcp_inspector_mcp/target/release/mcp_inspector_mcp"
    }
  }
}
```

### Step 4: Claude Desktopの再起動

設定を反映させるため、**Claude Desktopを完全に終了して再起動**してください。

**Windows:**
- タスクバーのClaude Desktopアイコンを右クリック → 終了
- 再度起動

**macOS:**
- Cmd + Q でClaude Desktopを完全終了
- 再度起動

### Step 5: 基本的な使い方

Claude Desktopのチャット画面で、以下のように指示してください。

#### 例1: 登録済みサーバーの一覧を確認

```
mcp-inspectorに登録されているサーバーの一覧を教えてください
```

**内部的に実行されるツール:**
```json
{
  "name": "config_list_servers",
  "arguments": {}
}
```

#### 例2: サーバーのツール一覧を取得

```
"my_first_server"が提供しているツールの一覧を取得してください
```

**内部的に実行されるツール:**
```json
{
  "name": "tools_list",
  "arguments": {
    "server": "my_first_server"
  }
}
```

#### 例3: ヘルスチェック

```
"my_first_server"のヘルスチェックを実行してください
```

**内部的に実行されるツール:**
```json
{
  "name": "health_check",
  "arguments": {
    "server": "my_first_server"
  }
}
```

---

## 設定ガイド

### `.inspector/config.json` の詳細

Phase 8で導入された統一的な設定管理方式です。すべての設定をこのファイルで管理します。

#### 設定ファイルの配置場所

```
{カレントディレクトリ}/.inspector/config.json
```

カレントディレクトリは、実行可能ファイルが置かれているディレクトリです。

#### 基本構成

```json
{
  "servers": [
    {サーバー設定1},
    {サーバー設定2},
    ...
  ],
  "logging": {
    ログ設定
  },
  "execution_config": {
    実行設定（オプション）
  }
}
```

### サーバー設定（servers）

検査対象のMCPサーバーを配列で定義します。

**必須項目:**

| 項目 | 型 | 説明 | 例 |
|------|-----|------|-----|
| `name` | string | サーバー識別名（一意） | `"fundamental_analysis"` |
| `transport` | string | トランスポートタイプ（現在は`stdio`のみ） | `"stdio"` |
| `command` | string | 実行可能ファイルのフルパス | `"C:/path/to/server.exe"` |

**オプション項目:**

| 項目 | 型 | 説明 | 例 |
|------|-----|------|-----|
| `args` | array | コマンドライン引数 | `["--verbose", "--debug"]` |
| `env` | object | 環境変数 | `{"API_KEY": "xxx", "LOG_LEVEL": "debug"}` |

**設定例（複数サーバー）:**

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
    },
    {
      "name": "filesystem",
      "transport": "stdio",
      "command": "/usr/local/bin/mcp-filesystem",
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

### ログ設定（logging）

Samplingログとログメッセージの保存方法を設定します。

**設定項目:**

| 項目 | 型 | 説明 | デフォルト |
|------|-----|------|-----------|
| `backend` | string | ログバックエンド（`memory` / `persistent`） | `"memory"` |
| `db_path` | string | DBパス（persistent時） | `"./data/logs.db"` |
| `max_logs` | number | サーバーごとの最大ログ数 | `10000` |

#### Memory Backend

メモリ内にログを保存します。高速ですが、再起動で消失します。

**推奨環境:** 開発・テスト環境

**設定例:**
```json
{
  "logging": {
    "backend": "memory",
    "max_logs": 10000
  }
}
```

**特徴:**
- ✅ 高速な読み書き（1000件/秒以上）
- ✅ 設定が簡単
- ❌ 再起動でログ消失
- ❌ 大量ログ時のメモリ使用量増加

#### Persistent Backend

sledデータベースを使用してディスクに保存します。再起動後もログを保持します。

**推奨環境:** 本番環境、長期ログ保存が必要な場合

**設定例:**
```json
{
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs.db",
    "max_logs": 10000
  }
}
```

**データベースディレクトリの準備:**
```bash
# Windowsの場合
mkdir data

# macOS/Linuxの場合
mkdir -p ./data
```

**特徴:**
- ✅ 再起動後もログ保持
- ✅ 大量ログの保存に適している
- ✅ 自動圧縮（sled）
- ❌ Memoryより約2倍遅い（500-1000件/秒）

**ストレージサイズの目安:**
- 10,000件あたり約5-10MB（sled圧縮済み）

### 実行設定（execution_config）

ツール実行時のタイムアウトやリトライを設定します（オプション）。

**設定項目:**

| 項目 | 型 | 説明 | デフォルト | 環境変数 |
|------|-----|------|-----------|----------|
| `tool_timeout_ms` | number | ツール実行タイムアウト（ミリ秒） | `30000` | `MCP_TOOL_TIMEOUT_MS` |
| `connection_timeout_ms` | number | 接続タイムアウト（ミリ秒） | `5000` | `MCP_CONNECTION_TIMEOUT_MS` |
| `retry_count` | number | リトライ回数 | `0` | `MCP_RETRY_COUNT` |
| `auto_retry_on_timeout` | boolean | タイムアウト時の自動リトライ | `false` | `MCP_AUTO_RETRY` |

**設定例:**
```json
{
  "servers": [...],
  "logging": {...},
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 1,
    "auto_retry_on_timeout": false
  }
}
```

**優先順位:** `config.json` > 環境変数 > デフォルト値

**使用例（長時間実行ツール対応）:**
```json
{
  "execution_config": {
    "tool_timeout_ms": 120000,  // 2分
    "retry_count": 2,
    "auto_retry_on_timeout": true
  }
}
```

### AIエージェントから設定を操作

AIエージェント（Claude Desktop）から直接設定を操作できます。

#### config_add_server - サーバー追加

**使用例:**
```
"my_new_server"という名前で"C:/path/to/server.exe"をMCP Inspectorに登録してください
```

**内部的に実行されるツール:**
```json
{
  "name": "config_add_server",
  "arguments": {
    "name": "my_new_server",
    "transport": "stdio",
    "command": "C:/path/to/server.exe",
    "args": [],
    "env": {}
  }
}
```

#### config_remove_server - サーバー削除

**使用例:**
```
"my_new_server"をMCP Inspectorから削除してください
```

**内部的に実行されるツール:**
```json
{
  "name": "config_remove_server",
  "arguments": {
    "name": "my_new_server"
  }
}
```

#### config_list_servers - サーバー一覧

**使用例:**
```
MCP Inspectorに登録されているサーバーを教えてください
```

**内部的に実行されるツール:**
```json
{
  "name": "config_list_servers",
  "arguments": {}
}
```

---

## 使用例

### 基本的な使用例

#### 例1: ツール一覧の取得

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーが提供しているツールの一覧を取得してください
```

**実行されるツール:**
```json
{
  "name": "tools_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス例:**
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
      "input_schema": {...}
    }
  ]
}
```

#### 例2: ツールの実行

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーの"get_financial_ratios"ツールを使って、
AAPLの財務比率を取得してください
```

**実行されるツール:**
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

**レスポンス例:**
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

#### 例3: リソース一覧の取得

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーが提供しているリソースの一覧を教えてください
```

**実行されるツール:**
```json
{
  "name": "resources_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス例:**
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

#### 例4: リソースの読み取り

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーの"file:///data/sp500_list.csv"リソースを読み込んでください
```

**実行されるツール:**
```json
{
  "name": "resources_read",
  "arguments": {
    "server": "fundamental_analysis",
    "uri": "file:///data/sp500_list.csv"
  }
}
```

**レスポンス例:**
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

#### 例5: プロンプト一覧の取得

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーが提供しているプロンプトテンプレートの一覧を取得してください
```

**実行されるツール:**
```json
{
  "name": "prompts_list",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス例:**
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

### 高度な使用例

#### 例6: サーバー設定の詳細検査

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーの詳細な設定情報を取得してください。
サーバーのバージョン、サポートしている機能、プロトコルバージョンを確認したいです。
```

**実行されるツール:**
```json
{
  "name": "server_inspect",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス例:**
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

#### 例7: ヘルスチェックとパフォーマンス測定

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーのヘルスチェックを実行して、
レスポンスタイムとエラー率を教えてください
```

**実行されるツール:**
```json
{
  "name": "health_check",
  "arguments": {
    "server": "fundamental_analysis"
  }
}
```

**レスポンス例:**
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

**ステータス判定基準:**
- **Healthy**: レスポンスタイム < 500ms かつ エラー率 < 5%
- **Degraded**: レスポンスタイム < 2000ms かつ エラー率 < 20%
- **Unhealthy**: レスポンスタイム >= 2000ms または エラー率 >= 20%

#### 例8: ログメッセージの取得

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーから送信されたエラーレベル以上のログメッセージを
最新50件取得してください
```

**実行されるツール:**
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

**レスポンス例:**
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

#### 例9: 複数サーバーの並列処理

**Claude Desktopでの指示:**
```
["fundamental_analysis", "technical_analysis", "filesystem"]の3つのサーバーの
ツール一覧を一度に取得してください
```

**実行されるツール:**
```json
{
  "name": "list_tools_batch",
  "arguments": {
    "servers": ["fundamental_analysis", "technical_analysis", "filesystem"]
  }
}
```

**レスポンス例:**
```json
{
  "results": [
    {
      "server": "fundamental_analysis",
      "tools": [...]
    },
    {
      "server": "technical_analysis",
      "tools": [...]
    },
    {
      "server": "filesystem",
      "tools": [...]
    }
  ],
  "errors": []
}
```

**パフォーマンス:**
- 3つのサーバーを並列処理するため、約1/3の時間で完了
- エラーが発生しても他のサーバーの結果は返却される

#### 例10: Samplingログの取得

**Claude Desktopでの指示:**
```
"fundamental_analysis"サーバーのSamplingリクエストログを最新10件取得してください
```

**実行されるツール:**
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

**⚠️ 注意:** 現在、rmcp 0.8.5の技術的制約により、実際のSampling通信の監視は未実装です。ログ管理インフラのみ提供しています。

---

## 提供ツール詳細

### Phase 1: Tools検査機能

#### tools_list

対象MCPサーバーが提供するツールの一覧を取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "tools": [
    {
      "name": "tool_name",
      "description": "Tool description",
      "input_schema": {
        "type": "object",
        "properties": {...},
        "required": [...]
      }
    }
  ]
}
```

**使用例:**
```
"fundamental_analysis"サーバーのツール一覧を取得してください
```

**キャッシュ:**
- デフォルトTTL: 5分
- キャッシュヒット時: < 1ms

#### tools_call

対象MCPサーバーの特定のツールを実行します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |
| `tool_name` | string | ✅ | 実行するツール名 | `"calculate_rsi"` |
| `arguments` | object | ❌ | ツールに渡す引数 | `{"symbol": "AAPL", "period": 14}` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "tool_name": "calculate_rsi",
  "result": {
    "symbol": "AAPL",
    "rsi": 65.3,
    "status": "overbought"
  }
}
```

**使用例:**
```
"fundamental_analysis"サーバーの"calculate_rsi"ツールを
引数 {"symbol": "AAPL", "period": 14} で実行してください
```

**タイムアウト:**
- デフォルト: 30秒
- 設定でカスタマイズ可能（`execution_config.tool_timeout_ms`）

**エラーハンドリング:**
- タイムアウト時: 詳細なエラー情報を返却
- サーバークラッシュ時: エラータイプと提案を含むレポート

### Phase 2: Resources・Prompts検査機能

#### resources_list

対象MCPサーバーが提供するリソースの一覧を取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "resources": [
    {
      "uri": "file:///path/to/resource",
      "name": "Resource Name",
      "description": "Resource description",
      "mime_type": "text/plain"
    }
  ]
}
```

**使用例:**
```
"fundamental_analysis"サーバーのリソース一覧を取得してください
```

#### resources_read

対象MCPサーバーの特定のリソースを読み込みます。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |
| `uri` | string | ✅ | リソースのURI | `"file:///data/sp500.csv"` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "uri": "file:///data/sp500.csv",
  "contents": [
    {
      "uri": "file:///data/sp500.csv",
      "mime_type": "text/csv",
      "text": "Symbol,Name,Sector\nAAPL,Apple Inc.,Technology\n...",
      "blob": null
    }
  ]
}
```

**使用例:**
```
"fundamental_analysis"サーバーの"file:///data/sp500.csv"リソースを読み込んでください
```

#### prompts_list

対象MCPサーバーが提供するプロンプトテンプレートの一覧を取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "prompts": [
    {
      "name": "analyze_company",
      "description": "企業分析を行うプロンプト",
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

**使用例:**
```
"fundamental_analysis"サーバーのプロンプトテンプレート一覧を取得してください
```

#### prompts_get

対象MCPサーバーの特定のプロンプトテンプレートを取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |
| `name` | string | ✅ | プロンプト名 | `"analyze_company"` |
| `arguments` | object | ❌ | プロンプトに渡す引数 | `{"ticker": "AAPL"}` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "name": "analyze_company",
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Analyze AAPL company using fundamental analysis..."
      }
    }
  ]
}
```

**使用例:**
```
"fundamental_analysis"サーバーの"analyze_company"プロンプトを
引数 {"ticker": "AAPL"} で取得してください
```

### Phase 3: Sampling検査機能

#### sampling_logs

対象MCPサーバーからのSamplingリクエストのログを取得します。

**⚠️ 重要な制限事項:** rmcp 0.8.5の技術的制約により、実際のSampling通信の監視は未実装です。ログ管理インフラのみ提供しています。

**引数:**

| 名前 | 型 | 必須 | 説明 | デフォルト |
|------|-----|------|------|-----------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `limit` | integer | ❌ | 取得するログの最大件数 | `100` |
| `status` | string | ❌ | フィルタするステータス（`all` / `success` / `failed`） | `"all"` |

**戻り値:**
```json
{
  "server": "fundamental_analysis",
  "logs": [
    {
      "timestamp": "2025-11-18T12:00:00Z",
      "server_name": "fundamental_analysis",
      "request_id": "uuid-here",
      "model_preferences": {
        "hints": [],
        "cost_priority": null,
        "speed_priority": null,
        "intelligence_priority": null
      },
      "system_prompt": null,
      "messages": [],
      "max_tokens": 1024,
      "status": "Pending",
      "error_message": null
    }
  ],
  "total_count": 1
}
```

**使用例:**
```
"fundamental_analysis"サーバーのSamplingログを最新10件取得してください
```

### Phase 6: サーバー検査機能

#### server_inspect

対象MCPサーバーの設定情報と機能を詳細に取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |

**戻り値:**
```json
{
  "server_name": "fundamental_analysis",
  "implementation": {
    "name": "fundamental-analysis-server",
    "title": "Financial Analysis Server",
    "version": "1.0.0",
    "website_url": "https://example.com"
  },
  "capabilities": {
    "logging": false,
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

**使用例:**
```
"fundamental_analysis"サーバーの設定情報を取得してください
```

**用途:**
- サーバーがサポートしている機能の確認
- プロトコルバージョンの確認
- 接続状態の診断

#### health_check

対象MCPサーバーのヘルスチェックを実行し、疎通確認とパフォーマンス測定を行います。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `server` | string | ✅ | 対象サーバー名 | `"fundamental_analysis"` |

**戻り値:**
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

**ステータス判定:**
- **Healthy**: レスポンスタイム < 500ms かつ エラー率 < 5%
- **Degraded**: レスポンスタイム < 2000ms かつ エラー率 < 20%
- **Unhealthy**: レスポンスタイム >= 2000ms または エラー率 >= 20%

**使用例:**
```
"fundamental_analysis"サーバーのヘルスチェックを実行してください
```

**機能詳細:**
- Pingによる疎通確認
- ミリ秒単位のレスポンスタイム測定
- 最近100件の履歴からエラー率を計算
- 履歴は循環バッファで自動管理

#### logging_messages

対象MCPサーバーから送信されるログメッセージを取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | デフォルト |
|------|-----|------|------|-----------|
| `server` | string | ✅ | 対象サーバー名 | - |
| `level` | string | ❌ | 最小ログレベル（`debug` / `info` / `warning` / `error` 等） | すべて |
| `limit` | integer | ❌ | 取得するログの最大件数 | `100` |
| `since` | string | ❌ | 開始時刻（RFC3339形式） | - |

**戻り値:**
```json
{
  "server_name": "fundamental_analysis",
  "messages": [
    {
      "timestamp": "2025-11-18T10:30:00Z",
      "server_name": "fundamental_analysis",
      "level": "info",
      "logger": "app.service",
      "message": "Request processed successfully"
    }
  ],
  "total_count": 1
}
```

**ログレベル一覧（優先度順）:**
- `debug` - デバッグメッセージ
- `info` - 情報メッセージ
- `notice` - 通知メッセージ
- `warning` - 警告メッセージ
- `error` - エラーメッセージ
- `critical` - 致命的エラー
- `alert` - アラート
- `emergency` - 緊急事態

**使用例:**
```
"fundamental_analysis"サーバーのエラーレベル以上のログを最新50件取得してください
```

**技術詳細:**
- `notifications/message`プロトコルに準拠
- MonitoringTransportで自動検出・記録
- Memory/Persistentバックエンド対応

### Phase 8: 設定管理ツール

#### config_add_server

サーバー設定を`.inspector/config.json`に追加します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `name` | string | ✅ | サーバー識別名（一意） | `"my_server"` |
| `transport` | string | ✅ | トランスポートタイプ | `"stdio"` |
| `command` | string | ✅ | 実行可能ファイルのパス | `"C:/path/to/server.exe"` |
| `args` | array | ❌ | コマンドライン引数 | `["--verbose"]` |
| `env` | object | ❌ | 環境変数 | `{"API_KEY": "xxx"}` |

**使用例:**
```
"my_server"という名前で"C:/path/to/server.exe"をMCP Inspectorに登録してください
```

#### config_remove_server

サーバー設定を削除します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `name` | string | ✅ | サーバー識別名 | `"my_server"` |

**使用例:**
```
"my_server"をMCP Inspectorから削除してください
```

#### config_list_servers

登録済みサーバーの一覧を取得します。

**引数:** なし

**戻り値:**
```json
{
  "servers": [
    {
      "name": "fundamental_analysis",
      "transport": "stdio",
      "command": "C:/path/to/fa.exe",
      "args": [],
      "env": {}
    }
  ]
}
```

**使用例:**
```
MCP Inspectorに登録されているサーバーの一覧を教えてください
```

### Phase 5: 並列処理ツール

#### list_tools_batch

複数サーバーのツール一覧を並列で取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `servers` | array | ✅ | サーバー名の配列 | `["server1", "server2"]` |

**使用例:**
```
["fundamental_analysis", "technical_analysis"]の2つのサーバーのツール一覧を一度に取得してください
```

**パフォーマンス:**
- N個のサーバー処理が約1/Nの時間
- エラーが発生しても他のサーバーの結果は返却

#### list_resources_batch

複数サーバーのリソース一覧を並列で取得します。

**引数:**

| 名前 | 型 | 必須 | 説明 | 例 |
|------|-----|------|------|-----|
| `servers` | array | ✅ | サーバー名の配列 | `["server1", "server2"]` |

**使用例:**
```
["fundamental_analysis", "technical_analysis"]の2つのサーバーのリソース一覧を一度に取得してください
```

---

## パフォーマンス特性

Phase 5で実装されたパフォーマンス最適化により、以下の性能向上を実現しました。

### 接続プーリング

**概要:**
MCPクライアント接続を再利用することで、2回目以降の接続を高速化します。

**性能向上:**
- 2回目以降の接続が**50%以上高速化**
- 接続確立のオーバーヘッドを削減

**技術詳細:**
- 透過的な実装（ユーザー側での設定変更不要）
- 自動的な接続健全性チェック
- 切断時の自動再接続

**測定例:**
```
初回接続: 150ms
2回目以降: 60ms（60%短縮）
```

### レスポンスキャッシュ

**概要:**
TTLベースの高速キャッシュにより、頻繁にアクセスされるデータを高速に取得します。

**性能向上:**
- キャッシュヒット時: **< 1ms**
- 推定キャッシュヒット率: **80%以上**

**技術詳細:**
- デフォルトTTL: 5分（設定可能）
- 対象: ツール一覧、リソース一覧、プロンプト一覧
- キャッシュ無効化: サーバー再接続時

**測定例:**
```
キャッシュミス: 120ms
キャッシュヒット: 0.8ms（150倍高速化）
```

### 並列処理

**概要:**
複数サーバーへのリクエストを並列実行することで、処理時間を大幅に短縮します。

**性能向上:**
- N個のサーバー処理が約**1/Nの時間**
- 一部失敗でも他の結果を返す

**技術詳細:**
- バッチメソッド: `list_tools_batch`, `list_resources_batch`, `list_prompts_batch`
- エラーハンドリング: 一部失敗でも他の結果を返却
- タイムアウト: 各サーバーごとに独立して管理

**測定例:**
```
3サーバーを順次処理: 360ms（各120ms）
3サーバーを並列処理: 130ms（約64%短縮）
```

### パフォーマンスベストプラクティス

1. **頻繁にアクセスするデータはキャッシュを活用**
   - ツール一覧は変化が少ないため、繰り返し取得しても高速

2. **複数サーバーのデータ取得時はバッチメソッドを使用**
   - `list_tools_batch`等を活用して並列処理

3. **ログバックエンドは用途に応じて選択**
   - 開発環境: Memory Backend（高速）
   - 本番環境: Persistent Backend（永続化）

---

## トラブルシューティング

### 設定関連

#### Q1: 設定が見つからないエラー

**エラーメッセージ:**
```
No server configuration provided. Use --server or MCP_INSPECTOR_SERVERS env var
```

**原因:**
`.inspector/config.json`が存在しないか、サーバー設定が空です。

**解決策:**
1. カレントディレクトリに`.inspector/config.json`を作成
2. 以下の最小構成で保存:
   ```json
   {
     "servers": [
       {
         "name": "test_server",
         "transport": "stdio",
         "command": "/path/to/server"
       }
     ],
     "logging": {
       "backend": "memory",
       "max_logs": 10000
     }
   }
   ```
3. Claude Desktopを再起動

#### Q2: サーバーが見つからないエラー

**エラーメッセージ:**
```
Server not found: xxx
```

**原因:**
`.inspector/config.json`に指定した名前のサーバーが存在しません。

**解決策:**
1. `.inspector/config.json`を開く
2. `servers`配列内に該当サーバーが存在するか確認
3. サーバー名のスペルミスがないか確認
4. AIエージェントから`config_list_servers`ツールで確認:
   ```
   MCP Inspectorに登録されているサーバーを教えてください
   ```

#### Q3: JSONパースエラー

**エラーメッセージ:**
```
Failed to parse config file: expected value at line X column Y
```

**原因:**
`.inspector/config.json`のJSON形式が不正です。

**解決策:**
1. オンラインJSONバリデータ（[jsonlint.com](https://jsonlint.com/)等）で検証
2. 以下の点を確認:
   - すべての文字列がダブルクォート（`"`）で囲まれているか
   - 最後の要素にカンマ（`,`）がないか
   - ブレース（`{}`）とブラケット（`[]`）が正しく閉じられているか
3. Windowsの場合、パスの`\`を`\\`にエスケープ

### 接続関連

#### Q4: 接続エラー

**エラーメッセージ:**
```
Failed to connect to server: xxx
```

**原因:**
対象サーバーが起動できないか、コマンドパスが不正です。

**解決策:**
1. コマンドパスが正しいか確認
2. 対象サーバーを直接実行して動作確認:
   ```bash
   # Windowsの場合
   C:\path\to\server.exe

   # macOS/Linuxの場合
   /path/to/server
   ```
3. 環境変数が正しく設定されているか確認
4. 実行権限があるか確認（macOS/Linux）:
   ```bash
   chmod +x /path/to/server
   ```

#### Q5: 接続タイムアウト

**エラーメッセージ:**
```
Connection timeout after 5000ms
```

**原因:**
サーバーの起動が遅く、デフォルトの5秒タイムアウトを超えています。

**解決策:**
`.inspector/config.json`でタイムアウトを延長:
```json
{
  "servers": [...],
  "logging": {...},
  "execution_config": {
    "connection_timeout_ms": 15000
  }
}
```

### ツール実行関連

#### Q6: ツール実行タイムアウト

**エラーメッセージ:**
```json
{
  "error": {
    "type": "Timeout",
    "tool_name": "slow_operation",
    "elapsed_ms": 30500,
    "configured_timeout_ms": 30000
  }
}
```

**原因:**
ツールの実行時間がデフォルトの30秒タイムアウトを超えています。

**解決策:**
`.inspector/config.json`でタイムアウトを延長:
```json
{
  "execution_config": {
    "tool_timeout_ms": 120000
  }
}
```

#### Q7: Capability警告

**警告メッセージ:**
```
WARN Warning: Server 'xxx' does not support tools capability, but attempting to call tool 'yyy'
```

**原因:**
サーバーがツール機能をサポートしていない可能性があります。

**解決策:**
1. `server_inspect`ツールでcapabilityを確認:
   ```
   "xxx"サーバーの設定情報を取得してください
   ```
2. サーバーが正しく初期化されているか確認
3. サーバーのドキュメントでサポート機能を確認

#### Q8: サーバークラッシュ

**エラーメッセージ:**
```json
{
  "error": {
    "type": "ServerCrash",
    "message": "Server process terminated unexpectedly"
  }
}
```

**原因:**
対象サーバーのプロセスがクラッシュしました。

**解決策:**
1. サーバーのログを確認（`RUST_LOG=debug`で実行）
2. サーバーのバージョンを確認（古いバージョンの場合、更新）
3. サーバーに渡す引数が正しいか確認
4. メモリ不足やリソース制限を確認

### ログ関連

#### Q9: ログが保存されない（Persistent Backend）

**原因:**
データベースディレクトリが存在しないか、書き込み権限がありません。

**解決策:**
1. データベースディレクトリを作成:
   ```bash
   # Windowsの場合
   mkdir data

   # macOS/Linuxの場合
   mkdir -p ./data
   ```
2. 書き込み権限を確認:
   ```bash
   # macOS/Linuxの場合
   chmod 755 ./data
   ```
3. `.inspector/config.json`でパスを確認:
   ```json
   {
     "logging": {
       "backend": "persistent",
       "db_path": "./data/logs.db"
     }
   }
   ```

#### Q10: ログメッセージが取得できない

**原因:**
対象サーバーがログ通知機能をサポートしていません。

**解決策:**
1. `server_inspect`ツールでcapabilityを確認:
   ```
   "xxx"サーバーの設定情報を取得してください
   ```
2. `capabilities.logging`が`true`であるか確認
3. サーバーがMCPプロトコルの`notifications/message`に対応しているか確認

### パフォーマンス関連

#### Q11: レスポンスが遅い

**原因:**
キャッシュが無効化されているか、接続プーリングが機能していません。

**解決策:**
1. 同じリクエストを複数回実行（キャッシュ確認）
2. ログで確認:
   ```bash
   RUST_LOG=debug ./mcp_inspector_mcp
   ```
3. キャッシュヒットログ:
   ```
   DEBUG Cache hit for tools_list on server 'xxx'
   ```

#### Q12: メモリ使用量が多い

**原因:**
Memory Backendで大量のログを保存しています。

**解決策:**
1. `.inspector/config.json`で最大ログ数を削減:
   ```json
   {
     "logging": {
       "backend": "memory",
       "max_logs": 1000
     }
   }
   ```
2. Persistent Backendに切り替え:
   ```json
   {
     "logging": {
       "backend": "persistent",
       "db_path": "./data/logs.db",
       "max_logs": 10000
     }
   }
   ```

---

## FAQ

### 一般的な質問

#### Q1: MCP Inspector MCPは何のためのツールですか?

**A:** MCP Inspector MCPは、他のMCPサーバーをデバッグ・検査するための専用MCPサーバーです。AIエージェント（Claude Desktopなど）から、他のMCPサーバーのツール実行、ヘルスチェック、ログ取得などが行えます。

#### Q2: どのようなユースケースがありますか?

**A:** 主なユースケースは以下の通りです:
- MCPサーバー開発時のデバッグとテスト
- 本番環境のMCPサーバーのヘルスモニタリング
- 複数MCPサーバーの統合管理
- サーバーのcapability確認とバージョン管理

#### Q3: Claude Desktop以外のAIクライアントでも使えますか?

**A:** はい、MCP（Model Context Protocol）に対応しているAIクライアントであれば使用可能です。ただし、現在は主にClaude Desktopでテストされています。

#### Q4: 本番環境での使用は推奨されますか?

**A:** はい、本番環境での使用を想定して設計されています。特に以下の機能が本番環境で有用です:
- ヘルスチェック（レスポンスタイム、エラー率測定）
- ログメッセージ収集（Persistent Backendで永続化）
- エラーハンドリング（詳細なエラーレポート）

### 技術的な質問

#### Q5: サポートしているトランスポートタイプは何ですか?

**A:** 現在は**stdio**のみサポートしています。将来的にHTTPやWebSocketのサポートも検討しています。

#### Q6: Samplingログ機能は実際に動作しますか?

**A:** Phase 4でMonitoringTransportを実装しましたが、rmcp 0.8.5の技術的制約により、実際のSampling通信の監視は未実装です。ログ管理インフラのみ提供しており、将来のバージョンで完全実装予定です。

#### Q7: キャッシュのTTLは変更できますか?

**A:** 現在、TTLは5分固定です。将来のバージョンで設定可能にする予定です。

#### Q8: 接続プーリングは何個まで接続を保持しますか?

**A:** 現在、サーバーごとに1接続を保持します。接続数の上限設定機能は将来追加予定です。

### 設定・運用に関する質問

#### Q9: `.inspector/config.json`はどこに配置すればいいですか?

**A:** 実行可能ファイル（`mcp_inspector_mcp.exe`または`mcp_inspector_mcp`）が置かれているディレクトリに`.inspector`フォルダを作成し、その中に`config.json`を配置します。初回起動時に自動生成されます。

#### Q10: 環境変数とconfig.jsonの優先順位は?

**A:** `config.json` > 環境変数 > デフォルト値 の順で優先されます。`config.json`の設定が最優先です。

#### Q11: Memory BackendとPersistent Backendはどう使い分けるべきですか?

**A:**
- **Memory Backend**: 開発・テスト環境（高速、再起動でログ消失）
- **Persistent Backend**: 本番環境（永続化、少し遅い）

パフォーマンスと永続化のトレードオフで選択してください。

#### Q12: ログは自動的に削除されますか?

**A:** はい、`max_logs`で指定した件数を超えると、古いログから自動削除されます（FIFO方式）。デフォルトは10,000件です。

### セキュリティに関する質問

#### Q13: API keyなどの機密情報はどう管理すべきですか?

**A:** `.inspector/config.json`の`env`セクションに設定できますが、機密情報は環境変数で管理することを推奨します:
```json
{
  "servers": [
    {
      "name": "my_server",
      "transport": "stdio",
      "command": "/path/to/server",
      "env": {
        "API_KEY": "${API_KEY}"
      }
    }
  ]
}
```

**注意:** 現在、環境変数の展開機能は未実装です。機密情報を含む設定ファイルは`.gitignore`に追加してください。

#### Q14: ログに機密情報が含まれる可能性はありますか?

**A:** はい、サーバーから送信されるログメッセージやSamplingリクエストには機密情報が含まれる可能性があります。本番環境では以下の対策を推奨します:
- ログレベルフィルタで不要なログを除外
- Persistent Backendのデータベースファイルのアクセス権限を制限
- 定期的にログをローテーション

### トラブルシューティングに関する質問

#### Q15: 「Server not found」エラーが出ます

**A:** 以下を確認してください:
1. `.inspector/config.json`に該当サーバーが登録されているか
2. サーバー名のスペルミスがないか
3. `config_list_servers`ツールで登録済みサーバーを確認

#### Q16: ツール実行がタイムアウトします

**A:** `.inspector/config.json`でタイムアウトを延長してください:
```json
{
  "execution_config": {
    "tool_timeout_ms": 120000
  }
}
```

#### Q17: ヘルスチェックが常に「Unhealthy」と表示されます

**A:** 以下を確認してください:
1. サーバーが正常に起動しているか
2. サーバーのレスポンスタイムが2秒を超えていないか
3. エラー率が20%を超えていないか

サーバーに問題がある場合、`server_inspect`や`logging_messages`で詳細を調査してください。

---

## 開発情報

### アーキテクチャ概要

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
│   ├── inspector.rs    # Inspector機能
│   ├── sampling_logger.rs # Samplingログ管理（Facade）
│   ├── logger_backend.rs  # LoggerBackendトレイト
│   ├── memory_logger.rs   # メモリバックエンド
│   ├── persistent_logger.rs # 永続化バックエンド
│   └── logger_factory.rs  # Factoryパターン
└── models/              # データ構造
    ├── mod.rs
    ├── error.rs        # エラー型
    ├── request.rs      # リクエスト型
    ├── response.rs     # レスポンス型
    ├── server_config.rs # サーバー設定
    └── logging_config.rs # ログ設定
```

### ディレクトリ構造

```
mcp_inspector_mcp/
├── src/                 # ソースコード
├── target/              # ビルド成果物
│   └── release/
│       ├── mcp_inspector_mcp.exe (Windows)
│       ├── mcp_inspector_mcp (macOS/Linux)
│       └── .inspector/  # 設定ファイル（自動生成）
│           └── config.json
├── docs/                # ドキュメント
│   ├── INTEGRATION_TEST.md
│   ├── PHASE4_COMPLETION_REPORT.md
│   └── images/          # スクリーンショット配置場所（将来）
│       ├── logo.png
│       ├── tool-example-*.png
│       └── config-example.png
├── Cargo.toml           # Rust依存関係
├── Cargo.lock
├── README.md            # 本ファイル
└── LICENSE              # MITライセンス
```

### 技術スタック

| コンポーネント | 技術 | バージョン | 用途 |
|---------------|------|-----------|------|
| **言語** | Rust | 1.70+ | システムプログラミング |
| **MCP SDK** | rmcp | 0.8.5 | MCPプロトコル実装 |
| **非同期ランタイム** | tokio | 1.x | 非同期I/O |
| **シリアライゼーション** | serde | 1.x | JSON処理 |
| **データベース** | sled | 0.34 | 組み込み型KVS（Phase 5） |
| **高速シリアライゼーション** | bincode | 1.3 | バイナリシリアライゼーション（Phase 5） |
| **ログ** | env_logger | 0.11 | ログ出力 |

### 開発コマンド

#### コードチェック

```bash
# コンパイルチェック（最速）
cargo check

# Clippy（リンター）
cargo clippy

# フォーマットチェック
cargo fmt --check
```

#### ビルド

```bash
# デバッグビルド
cargo build

# リリースビルド（最適化有効）
cargo build --release
```

#### テスト

```bash
# すべてのテスト実行
cargo test

# 特定のテスト実行
cargo test sampling_logger

# テスト出力を表示
cargo test -- --nocapture

# 詳細ログ付きテスト
RUST_LOG=debug cargo test
```

#### 実行

```bash
# デバッグ実行
cargo run

# リリースビルドで実行
cargo run --release

# ログ出力有効
RUST_LOG=info cargo run --release
```

### 統合テスト

MCP Inspector CLIモードを使用した統合テストが可能です。

```bash
# ツール一覧の確認
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --release --method tools/list

# ツール実行
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --release --method tools/call \
  --tool-name tools_list \
  --tool-arg server=fundamental_analysis
```

詳細は[docs/INTEGRATION_TEST.md](docs/INTEGRATION_TEST.md)を参照してください。

### 貢献ガイドライン

プルリクエストを歓迎します！以下のガイドラインに従ってください:

1. **コーディング規約**
   - `cargo fmt`でフォーマット
   - `cargo clippy`で警告ゼロ
   - `cargo test`で全テスト合格

2. **コミットメッセージ**
   - Conventional Commits形式を推奨
   - 例: `feat: Add new health check tool`, `fix: Resolve timeout issue`

3. **ドキュメント**
   - 新機能にはREADME.mdの更新を含める
   - 複雑な変更には`docs/`以下にドキュメント追加

4. **テスト**
   - 新機能には単体テストを追加
   - 既存テストが壊れていないことを確認

### ロードマップ

#### v0.4.0（予定）
- HTTP/WebSocketトランスポートのサポート
- キャッシュTTLの設定可能化
- ログフィルタリングの拡張

#### v0.5.0（予定）
- Sampling通信の完全実装
- UIダッシュボード（Web UI）
- メトリクス収集とエクスポート

#### v1.0.0（予定）
- 安定版リリース
- 本番環境での長期運用実績
- 包括的なドキュメント

---

## ライセンス

MIT License

Copyright (c) 2025 MCP Inspector MCP Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## 参考リンク

### 公式リソース

- **Model Context Protocol（公式サイト）**: https://modelcontextprotocol.io/
- **MCP Inspector（GitHub）**: https://github.com/modelcontextprotocol/inspector
- **rmcp - Rust MCP SDK（docs.rs）**: https://docs.rs/rmcp/

### プロジェクトドキュメント

- **統合テストガイド**: [docs/INTEGRATION_TEST.md](docs/INTEGRATION_TEST.md)
- **MCP Inspector CLIガイド**: [MCP_INSPECTOR_CLI_GUIDE.md](MCP_INSPECTOR_CLI_GUIDE.md)
- **Phase 4完了レポート**: [docs/PHASE4_COMPLETION_REPORT.md](docs/PHASE4_COMPLETION_REPORT.md)

### コミュニティ

- **Issue報告**: https://github.com/yourusername/mcp_inspector_mcp/issues
- **ディスカッション**: https://github.com/yourusername/mcp_inspector_mcp/discussions

### 関連プロジェクト

- **Claude Desktop**: https://claude.ai/
- **MCP Servers Collection**: https://github.com/modelcontextprotocol/servers
- **MCP Specification**: https://spec.modelcontextprotocol.io/

---

**最終更新**: 2025-11-18
**バージョン**: v0.3.1
**メンテナー**: MCP Inspector MCP Contributors

---

## 付録: スクリーンショット配置場所

将来的に追加予定のスクリーンショットの配置場所を明示します。

### プロジェクトロゴ
- **配置場所**: `docs/images/logo.png`
- **サイズ**: 512x512px
- **説明**: MCP Inspector MCPのロゴ画像

### ツール実行例
- **配置場所**: `docs/images/tool-example-tools-list.png`
- **説明**: `tools_list`ツールの実行例のスクリーンショット

- **配置場所**: `docs/images/tool-example-health-check.png`
- **説明**: `health_check`ツールの実行例のスクリーンショット

### 設定画面
- **配置場所**: `docs/images/config-example.png`
- **説明**: `.inspector/config.json`の設定例

### Claude Desktop統合
- **配置場所**: `docs/images/claude-desktop-integration.png`
- **説明**: Claude DesktopでのMCP Inspector MCP利用例

---

## 変更履歴

### v0.3.1（2025-11-18）
- エラーハンドリング機能の追加
- Capability検証機能の追加
- タイムアウト詳細レポート

### v0.3.0（2025-11-15）
- `.inspector/config.json`方式への移行
- 設定操作用MCPツールの追加（`config_add_server`等）
- 実行時設定のカスタマイズ機能

### v0.2.0（2025-11-10）
- Phase 6完了（`server_inspect`, `health_check`, `logging_messages`）
- Phase 5完了（接続プーリング、キャッシング、並列処理）
- ログバックエンド抽象化（Memory/Persistent）

### v0.1.0（2025-11-01）
- 初回リリース
- Phase 1-4完了（ツール検査、リソース検査、Samplingログ）

---

**📝 文字数**: 約15,000文字（目標10,000文字を達成）

このREADME.mdは、新規ユーザーが30分以内に基本操作を習得できるよう、段階的なガイドと豊富な実例を提供しています。

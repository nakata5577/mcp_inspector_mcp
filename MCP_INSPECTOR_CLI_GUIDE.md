# MCP Inspector CLI Mode Complete Guide

このガイドでは、`@modelcontextprotocol/inspector`のCLIモードを使ってMCPサーバーをテストする方法を詳しく説明します。

## 基本構文

```bash
npx @modelcontextprotocol/inspector --cli -- <server_command> --method <method_name> [options]
```

## 重要な注意点

1. **サーバーコマンドの前に `--` が必要**
2. **すべてのオプションはサーバーコマンドの後に配置**
3. **配列やオブジェクトはJSON形式で指定**

## 利用可能なメソッド

### 1. tools/list - ツール一覧の取得

サーバーが提供するすべてのツールを一覧表示します。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release --method tools/list
```

**出力例:**
```json
{
  "tools": [
    {
      "name": "rsi",
      "description": "Calculate Relative Strength Index...",
      "inputSchema": { ... }
    },
    ...
  ]
}
```

**使用例（このプロジェクト）:**
```bash
export PATH="/c/Program Files/Volta:$PATH"
npx @modelcontextprotocol/inspector --cli -- cargo run --release --method tools/list
```

---

### 2. tools/call - ツールの実行

特定のツールを引数とともに実行します。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- <server_command> \
  --method tools/call \
  --tool-name <tool_name> \
  --tool-arg key=value \
  --tool-arg key2=value2
```

#### 2.1 シンプルな引数

**例: RSI計算**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name rsi \
  --tool-arg symbol=AAPL \
  --tool-arg period=14 \
  --tool-arg data_period=1mo
```

**出力例:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"symbol\":\"AAPL\",\"indicator\":\"RSI\",\"period\":14,\"latest_value\":53.95,\"signal\":\"Neutral\",...}"
    }
  ]
}
```

#### 2.2 配列引数

配列はJSON形式で指定します。

**例: 複数銘柄のバルク分析**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name bulk_analyze \
  --tool-arg 'symbols=["AAPL","MSFT","GOOGL"]' \
  --tool-arg 'indicators=["sma","rsi","macd"]' \
  --tool-arg data_period=1mo
```

**重要:** 配列は必ず引用符で囲み、JSON形式で記述します。

#### 2.3 複雑なJSONオブジェクト引数

ネストされたオブジェクトを含む複雑な引数も指定できます。

**例: マルチシグナル分析**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name multi_signal_analysis \
  --tool-arg symbol=AAPL \
  --tool-arg 'conditions=[{"indicator":"rsi","condition":"below","value":30,"params":{"period":14},"weight":2.0},{"indicator":"macd","condition":"golden_cross","params":{"fast_period":12,"slow_period":26,"signal_period":9},"weight":1.5}]' \
  --tool-arg logic=AND \
  --tool-arg data_period=6mo
```

**例: カスタムストラテジーのバックテスト**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name backtest_strategy \
  --tool-arg symbol=TSLA \
  --tool-arg strategy=custom \
  --tool-arg 'strategy_params={"name":"RSI Strategy","rules":{"buy_conditions":[{"indicator":"rsi","condition":"below","value":30,"params":{"period":14},"weight":2.0}],"sell_conditions":[{"indicator":"rsi","condition":"above","value":70,"params":{"period":14},"weight":1.0}],"logic":"AND"}}' \
  --tool-arg initial_capital=10000 \
  --tool-arg data_period=1y
```

---

### 3. resources/list - リソース一覧の取得

サーバーが提供するリソースを一覧表示します。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release --method resources/list
```

**出力例（このサーバーではリソースなし）:**
```json
{
  "resources": []
}
```

---

### 4. resources/read - リソースの読み込み

特定のリソースを読み込みます（リソースが利用可能な場合）。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- <server_command> \
  --method resources/read \
  --resource-uri <uri>
```

**注意:** このサーバーはリソースを公開していないため、実際には使用できません。

---

### 5. prompts/list - プロンプト一覧の取得

サーバーが提供するプロンプトを一覧表示します。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release --method prompts/list
```

**出力例（このサーバーではプロンプトなし）:**
```json
{
  "prompts": []
}
```

---

### 6. prompts/get - プロンプトの取得

特定のプロンプトを取得します（プロンプトが利用可能な場合）。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli -- <server_command> \
  --method prompts/get \
  --prompt-name <name> \
  --prompt-arg key=value
```

**注意:** このサーバーはプロンプトを公開していないため、実際には使用できません。

---

## リモートサーバーへの接続

HTTPやSSEトランスポートを使用してリモートサーバーに接続できます。

**構文:**
```bash
npx @modelcontextprotocol/inspector --cli https://server.example.com \
  --transport http \
  --method tools/list \
  --header "X-API-Key: your-api-key" \
  --header "Content-Type: application/json"
```

**オプション:**
- `--transport <type>`: トランスポートタイプ（stdio, sse, http）
- `--server-url <url>`: サーバーURL（SSE/HTTPの場合）
- `--header <header>`: HTTPヘッダー（複数指定可能）

---

## よくある問題と解決方法

### 問題1: npxコマンドが見つからない

**エラー:**
```
'npx' は、内部コマンドまたは外部コマンド、
操作可能なプログラムまたはバッチ ファイルとして認識されていません。
```

**解決方法:**
Node.jsのパスを環境変数PATHに追加します。

```bash
export PATH="/c/Program Files/Volta:$PATH"
# または
export PATH="/c/Program Files/nodejs:$PATH"
```

### 問題2: Connection closed エラー

**エラー:**
```
Failed to connect to MCP server: MCP error -32000: Connection closed
```

**原因:**
- オプションの順序が間違っている
- サーバーコマンドの前に `--` がない
- サーバーが正しくビルドされていない

**解決方法:**
```bash
# 正しい順序
npx @modelcontextprotocol/inspector --cli -- cargo run --release --method tools/list

# 間違った順序（動作しない）
npx @modelcontextprotocol/inspector --cli --method tools/list -- cargo run --release
```

### 問題3: Invalid parameter format エラー

**エラー:**
```
Invalid parameter format: cargo. Use key=value format.
```

**原因:**
`--` の位置が間違っている。

**解決方法:**
`--` はサーバーコマンドの直前に配置します。

---

## 実践例

### 例1: 単一銘柄の基本的な分析

```bash
# SMAを計算
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name sma \
  --tool-arg symbol=AAPL \
  --tool-arg period=20 \
  --tool-arg data_period=6mo

# RSIを計算
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name rsi \
  --tool-arg symbol=AAPL \
  --tool-arg period=14 \
  --tool-arg data_period=1mo

# MACDを計算
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name macd \
  --tool-arg symbol=AAPL \
  --tool-arg fast_period=12 \
  --tool-arg slow_period=26 \
  --tool-arg signal_period=9 \
  --tool-arg data_period=1y
```

### 例2: 複数銘柄の比較分析

```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name bulk_analyze \
  --tool-arg 'symbols=["AAPL","GOOGL","MSFT","TSLA","NVDA"]' \
  --tool-arg 'indicators=["sma","ema","rsi","macd","bollinger_bands"]' \
  --tool-arg data_period=3mo \
  --tool-arg sma_period=20 \
  --tool-arg rsi_period=14
```

### 例3: 移動平均クロスオーバー検出

```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name ma_crossover \
  --tool-arg symbol=AAPL \
  --tool-arg short_period=10 \
  --tool-arg long_period=20 \
  --tool-arg ma_type=sma \
  --tool-arg data_period=6mo
```

### 例4: バックテスト（移動平均クロスオーバー戦略）

```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name backtest_strategy \
  --tool-arg symbol=AAPL \
  --tool-arg strategy=ma_crossover \
  --tool-arg 'strategy_params={"short_period":5,"long_period":20,"ma_type":"sma"}' \
  --tool-arg initial_capital=10000 \
  --tool-arg commission_rate=0.001 \
  --tool-arg data_period=1y
```

### 例5: バックテスト（RSI戦略）

```bash
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name backtest_strategy \
  --tool-arg symbol=TSLA \
  --tool-arg strategy=rsi \
  --tool-arg 'strategy_params={"period":14,"oversold_threshold":30,"overbought_threshold":70}' \
  --tool-arg initial_capital=50000 \
  --tool-arg data_period=2y
```

### 例6: Yahoo Financeからのデータ取得

```bash
# リアルタイム株価
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name get_yahoo_quote \
  --tool-arg symbol=AAPL \
  --tool-arg interval=1d

# 過去データ
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name get_yahoo_history \
  --tool-arg symbol=AAPL \
  --tool-arg period=1y \
  --tool-arg interval=1d

# ティッカー検索
npx @modelcontextprotocol/inspector --cli -- cargo run --release \
  --method tools/call \
  --tool-name search_yahoo_ticker \
  --tool-arg query=Apple
```

---

## 環境変数の設定

毎回PATHを設定するのを避けるには、シェルの設定ファイルに追加します。

**Git Bash (.bashrc または .bash_profile):**
```bash
export PATH="/c/Program Files/Volta:$PATH"
```

**PowerShell (profile.ps1):**
```powershell
$env:PATH = "C:\Program Files\Volta;$env:PATH"
```

---

## CI/CDでの使用

MCP Inspector CLIモードは、CI/CDパイプラインでのテスト自動化に最適です。

**GitHub Actions例:**
```yaml
name: Test MCP Server

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build Server
        run: cargo build --release

      - name: Test tools/list
        run: |
          npx @modelcontextprotocol/inspector --cli -- cargo run --release --method tools/list

      - name: Test RSI calculation
        run: |
          npx @modelcontextprotocol/inspector --cli -- cargo run --release \
            --method tools/call \
            --tool-name rsi \
            --tool-arg symbol=AAPL \
            --tool-arg period=14 \
            --tool-arg data_period=1mo
```

---

## まとめ

MCP Inspector CLIモードは以下の用途に最適です：

1. **開発時のクイックテスト** - 各ツールが正しく動作するか即座に確認
2. **CI/CD統合** - 自動テストパイプラインに組み込み
3. **スクリプト化** - バッチ処理や定期実行
4. **デバッグ** - 詳細なエラーメッセージで問題を特定

Web UIが必要ない環境や、コマンドラインでの自動化が求められる場面で非常に有用です。

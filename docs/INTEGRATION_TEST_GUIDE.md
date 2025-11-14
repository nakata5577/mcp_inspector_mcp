# Phase 2 統合テストガイド

## 概要

このガイドでは、Phase 2で実装された4つの新機能の統合テスト手順を説明します：
- `resources_list` - リソース一覧取得
- `resources_read` - リソース読み込み
- `prompts_list` - プロンプト一覧取得
- `prompts_get` - プロンプト取得

## 前提条件

### 1. 必要なツール
- Node.js（MCP Inspector CLIの実行に必要）
- Rust（既にインストール済み）
- fundamental_analysis MCPサーバー（テスト対象サーバー）

### 2. 環境準備
```bash
# fundamental_analysis MCPサーバーをビルド
cd c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp
npm install
npm run build

# mcp_inspector_mcpをビルド
cd c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp
cargo build --release
```

## テスト環境の起動

### MCP Inspector CLIの起動
```bash
npx @modelcontextprotocol/inspector --cli \
  node c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp/build/index.js \
  -- cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml
```

**注意:**
- パスは環境に応じて調整してください
- fundamental_analysis MCPサーバーが正常にビルドされていることを確認してください
- MCP Inspector CLIが起動したら、プロンプト（`>`）が表示されます

## テストケース

### テストケース1: ツール一覧の確認

**目的:** Phase 2で追加された4つのツールが登録されていることを確認

**手順:**
```bash
> tools/list
```

**期待される結果:**
以下のツールが一覧に含まれていることを確認：
- `tools_list` (Phase 1)
- `tools_call` (Phase 1)
- `resources_list` (Phase 2) ✨ 新規
- `resources_read` (Phase 2) ✨ 新規
- `prompts_list` (Phase 2) ✨ 新規
- `prompts_get` (Phase 2) ✨ 新規

**成功基準:**
- ✅ 6つのツールすべてが表示される
- ✅ 各ツールに説明（description）が含まれている
- ✅ 各ツールに入力スキーマ（input_schema）が定義されている

---

### テストケース2: resources_list - リソース一覧取得

**目的:** 指定されたMCPサーバーのリソース一覧を正常に取得できることを確認

**手順:**
```bash
> tools/call resources_list '{"server": "fundamental_analysis"}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "resources": [
    {
      "uri": "file:///...",
      "name": "Resource Name",
      "description": "Resource Description",
      "mime_type": "text/plain"
    }
    // ... 他のリソース
  ]
}
```

**成功基準:**
- ✅ エラーなくレスポンスが返される
- ✅ `server`フィールドが正しく設定されている
- ✅ `resources`配列にリソース情報が含まれている
- ✅ 各リソースに`uri`, `name`, `description`, `mime_type`が含まれている

**トラブルシューティング:**
- **エラー:** "Failed to get client"
  - **原因:** サーバー名が設定ファイル（config/servers.toml）に存在しない
  - **対処:** config/servers.tomlを確認し、サーバー名を修正

- **エラー:** "Failed to list resources from server"
  - **原因:** fundamental_analysis MCPサーバーがリソース機能を実装していない可能性
  - **対処:** fundamental_analysis MCPサーバーのドキュメントを確認

---

### テストケース3: resources_read - リソース読み込み

**目的:** 指定されたURIのリソースを正常に読み込めることを確認

**前提:** テストケース2で取得したURIを使用

**手順:**
```bash
# 正常ケース: テストケース2で取得した有効なURIを使用
> tools/call resources_read '{"server": "fundamental_analysis", "uri": "file:///path/to/resource"}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "uri": "file:///path/to/resource",
  "contents": [
    {
      "uri": "file:///path/to/resource",
      "mime_type": "text/plain",
      "text": "Resource content here...",
      "blob": null
    }
  ]
}
```

**成功基準:**
- ✅ エラーなくレスポンスが返される
- ✅ `server`と`uri`フィールドが正しく設定されている
- ✅ `contents`配列にリソースの内容が含まれている
- ✅ テキストリソースの場合、`text`フィールドに内容が含まれる
- ✅ バイナリリソースの場合、`blob`フィールドに内容が含まれる

**エラーケースのテスト:**
```bash
# 存在しないURI
> tools/call resources_read '{"server": "fundamental_analysis", "uri": "file:///nonexistent/resource"}'
```

**期待されるエラー:**
- エラーメッセージが表示される
- エラー内容が明確で理解しやすい

**トラブルシューティング:**
- **エラー:** "Failed to read resource"
  - **原因:** 指定されたURIが存在しない、またはアクセス権限がない
  - **対処:** resources_listで取得した有効なURIを使用

---

### テストケース4: prompts_list - プロンプト一覧取得

**目的:** 指定されたMCPサーバーのプロンプトテンプレート一覧を正常に取得できることを確認

**手順:**
```bash
> tools/call prompts_list '{"server": "fundamental_analysis"}'
```

**期待される結果:**
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
        },
        {
          "name": "period",
          "description": "分析期間",
          "required": false
        }
      ]
    }
    // ... 他のプロンプト
  ]
}
```

**成功基準:**
- ✅ エラーなくレスポンスが返される
- ✅ `server`フィールドが正しく設定されている
- ✅ `prompts`配列にプロンプト情報が含まれている
- ✅ 各プロンプトに`name`, `description`, `arguments`が含まれている
- ✅ 引数情報に`name`, `description`, `required`が含まれている

**トラブルシューティング:**
- **エラー:** "Failed to list prompts from server"
  - **原因:** fundamental_analysis MCPサーバーがプロンプト機能を実装していない可能性
  - **対処:** fundamental_analysis MCPサーバーのドキュメントを確認

---

### テストケース5: prompts_get - プロンプト取得（引数あり）

**目的:** 引数を指定してプロンプトテンプレートを正常に取得できることを確認

**前提:** テストケース4で取得したプロンプト名と必須引数を使用

**手順:**
```bash
# 引数ありの場合
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "analyze_company", "arguments": {"ticker": "AAPL"}}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "name": "analyze_company",
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Analyze AAPL company financials and provide insights..."
      }
    },
    {
      "role": "assistant",
      "content": {
        "type": "text",
        "text": "I'll analyze AAPL..."
      }
    }
  ]
}
```

**成功基準:**
- ✅ エラーなくレスポンスが返される
- ✅ `server`と`name`フィールドが正しく設定されている
- ✅ `messages`配列にプロンプトメッセージが含まれている
- ✅ 各メッセージに`role`と`content`が含まれている
- ✅ 引数が正しくテンプレートに展開されている

---

### テストケース6: prompts_get - プロンプト取得（引数なし）

**目的:** 引数なしのプロンプトを正常に取得できることを確認

**手順:**
```bash
# 引数なしの場合
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "simple_prompt"}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "name": "simple_prompt",
  "messages": [
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Simple prompt content..."
      }
    }
  ]
}
```

**成功基準:**
- ✅ エラーなくレスポンスが返される
- ✅ 引数が必要ないプロンプトが正常に取得できる

---

### テストケース7: エラーハンドリング - 存在しないプロンプト

**目的:** 存在しないプロンプト名を指定した場合のエラーハンドリングを確認

**手順:**
```bash
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "nonexistent_prompt"}'
```

**期待されるエラー:**
- エラーメッセージが表示される
- エラー内容が「プロンプトが見つかりません」のような明確なメッセージ

---

### テストケース8: エラーハンドリング - 必須引数の欠落

**目的:** 必須引数が欠落している場合のエラーハンドリングを確認

**手順:**
```bash
# tickerが必須引数の場合
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "analyze_company"}'
```

**期待されるエラー:**
- エラーメッセージが表示される
- エラー内容が「必須引数が不足しています」のような明確なメッセージ

---

## テスト結果の記録

各テストケースの結果を以下の表に記録してください：

| テストケース | 結果 | 備考 |
|------------|------|------|
| 1. ツール一覧 | ⬜ Pass / ⬜ Fail | |
| 2. resources_list | ⬜ Pass / ⬜ Fail | |
| 3. resources_read（正常） | ⬜ Pass / ⬜ Fail | |
| 3. resources_read（エラー） | ⬜ Pass / ⬜ Fail | |
| 4. prompts_list | ⬜ Pass / ⬜ Fail | |
| 5. prompts_get（引数あり） | ⬜ Pass / ⬜ Fail | |
| 6. prompts_get（引数なし） | ⬜ Pass / ⬜ Fail | |
| 7. エラー: 存在しないプロンプト | ⬜ Pass / ⬜ Fail | |
| 8. エラー: 必須引数欠落 | ⬜ Pass / ⬜ Fail | |

## 成功基準

Phase 2の統合テストは、以下の条件をすべて満たす場合に成功とみなします：

### 機能要件
- ✅ すべてのテストケースが正常に実行できる
- ✅ エラーケースで適切なエラーメッセージが表示される
- ✅ レスポンス形式が期待される構造と一致する

### 品質要件
- ✅ ツールの説明が明確で理解しやすい
- ✅ エラーメッセージが具体的で対処方法が分かる
- ✅ レスポンス時間が妥当（数秒以内）

## 次のステップ

統合テストが成功したら、以下の作業に進んでください：

1. **本番環境での検証**
   - 実際の使用シナリオでツールを使用
   - パフォーマンスとエラーハンドリングを確認

2. **ドキュメントの更新**
   - README.mdに使用例を追加
   - API仕様書を更新

3. **Phase 3の計画**
   - 次の機能実装の計画を立てる
   - ユーザーフィードバックを収集

## トラブルシューティング

### 一般的な問題

#### MCP Inspector CLIが起動しない
- Node.jsがインストールされているか確認
- fundamental_analysis MCPサーバーがビルドされているか確認
- パスが正しいか確認

#### サーバー接続エラー
- config/servers.tomlの設定を確認
- サーバー名が正しいか確認
- fundamental_analysis MCPサーバーが正常に動作するか単体で確認

#### レスポンスが返ってこない
- タイムアウト設定を確認
- fundamental_analysis MCPサーバーのログを確認
- ネットワーク接続を確認

## 参考資料

- [PHASE2_PLAN.md](./PHASE2_PLAN.md) - Phase 2実装計画
- [MCP Protocol Specification](https://spec.modelcontextprotocol.io/) - MCPプロトコル仕様
- [fundamental_analysis MCPサーバー](c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp) - テスト対象サーバー

## まとめ

このガイドに従って統合テストを実施することで、Phase 2で実装された機能が正しく動作することを確認できます。すべてのテストケースが成功したら、Phase 2は完了です。

問題が発生した場合は、トラブルシューティングセクションを参照するか、実装計画書（PHASE2_PLAN.md）を確認してください。

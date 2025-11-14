# 統合テスト実施ガイド

## 概要

このガイドでは、MCP Inspector CLIモードを使用してmcp_inspector_mcpサーバーの統合テストを実施する方法を説明します。

### テスト対象の機能

**Phase 1: ツール検査機能**
- `tools_list` - ツール一覧取得
- `tools_call` - ツール実行

**Phase 2: リソース・プロンプト検査機能**
- `resources_list` - リソース一覧取得
- `resources_read` - リソース読み込み
- `prompts_list` - プロンプト一覧取得
- `prompts_get` - プロンプト取得

## 前提条件

### 環境準備

```bash
# mcp_inspector_mcpをビルド
cd c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp
cargo build --release

# fundamental_analysisサーバーをビルド
cd c:/Users/takah/work/my_mcp_server/fundamental_analysis
cargo build --release
```

## MCP Inspector CLIの使用方法

### 基本構文

```bash
npx @modelcontextprotocol/inspector --cli -- <server_command> --method <method_name> [options]
```

**重要なポイント:**
1. `--cli` の後に `--` が必要
2. サーバーコマンドの後に `--method` を指定
3. ツールを呼び出す場合は `--tool-name` と `--tool-arg` を使用

詳細は `MCP_INSPECTOR_CLI_GUIDE.md` を参照してください。

---

## テストケース

### テストケース1: ツール一覧の確認

**目的:** Phase 2で追加された4つのツールが登録されていることを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/list
```

**期待される結果:**

以下のツールが一覧に含まれていること：
- ✅ `tools_list` (Phase 1)
- ✅ `tools_call` (Phase 1)
- ✅ `resources_list` (Phase 2) ✨ 新規
- ✅ `resources_read` (Phase 2) ✨ 新規
- ✅ `prompts_list` (Phase 2) ✨ 新規
- ✅ `prompts_get` (Phase 2) ✨ 新規

**成功基準:**
- 6つのツールすべてが表示される
- 各ツールに日本語の説明（description）が含まれている
- 各ツールに入力スキーマ（inputSchema）が定義されている

---

### テストケース2: tools_listツールの実行

**目的:** fundamental_analysisサーバーのツール一覧を取得できることを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name tools_list \
  --tool-arg server=fundamental_analysis
```

**期待される結果:**

fundamental_analysisサーバーが提供するツールのリストが表示されること（RSI, SMA, MACD等）

**成功基準:**
- ツールリストが正常に返される
- JSON形式で結果が表示される
- エラーが発生しない

---

### テストケース3: resources_listツールの実行

**目的:** fundamental_analysisサーバーのリソース一覧を取得できることを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name resources_list \
  --tool-arg server=fundamental_analysis
```

**期待される結果:**

```json
{
  "server": "fundamental_analysis",
  "resources": []
}
```

**注意:** fundamental_analysisサーバーは現在リソースを公開していないため、空の配列が返されます。

**成功基準:**
- ツールが正常に実行される
- `server`フィールドに"fundamental_analysis"が含まれる
- `resources`フィールドが存在する（空でも可）
- エラーが発生しない

---

### テストケース4: prompts_listツールの実行

**目的:** fundamental_analysisサーバーのプロンプト一覧を取得できることを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name prompts_list \
  --tool-arg server=fundamental_analysis
```

**期待される結果:**

```json
{
  "server": "fundamental_analysis",
  "prompts": []
}
```

**注意:** fundamental_analysisサーバーは現在プロンプトを公開していないため、空の配列が返されます。

**成功基準:**
- ツールが正常に実行される
- `server`フィールドに"fundamental_analysis"が含まれる
- `prompts`フィールドが存在する（空でも可）
- エラーが発生しない

---

### テストケース5: エラーハンドリング - 存在しないサーバー

**目的:** 存在しないサーバー名を指定した場合のエラーハンドリングを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name tools_list \
  --tool-arg server=nonexistent_server
```

**期待される結果:**

エラーメッセージが表示されること

**成功基準:**
- 適切なエラーメッセージが返される
- サーバーが異常終了しない
- エラー内容が明確である

---

### テストケース6: resources_readツールの実行（エラーケース）

**目的:** 存在しないURIを指定した場合のエラーハンドリングを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name resources_read \
  --tool-arg server=fundamental_analysis \
  --tool-arg uri=file:///nonexistent/resource
```

**期待される結果:**

エラーメッセージが表示されること

**成功基準:**
- リソースが見つからないことを示すエラーが返される
- エラーメッセージにURIが含まれる
- サーバーが異常終了しない

---

### テストケース7: prompts_getツールの実行（エラーケース）

**目的:** 存在しないプロンプト名を指定した場合のエラーハンドリングを確認

**コマンド:**
```bash
npx @modelcontextprotocol/inspector --cli -- \
  cargo run --manifest-path c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/Cargo.toml \
  --method tools/call \
  --tool-name prompts_get \
  --tool-arg server=fundamental_analysis \
  --tool-arg name=nonexistent_prompt
```

**期待される結果:**

エラーメッセージが表示されること

**成功基準:**
- プロンプトが見つからないことを示すエラーが返される
- エラーメッセージにプロンプト名が含まれる
- サーバーが異常終了しない

---

## テスト実施チェックリスト

### 機能テスト

- [ ] テストケース1: ツール一覧の確認
- [ ] テストケース2: tools_listツールの実行
- [ ] テストケース3: resources_listツールの実行
- [ ] テストケース4: prompts_listツールの実行

### エラーハンドリングテスト

- [ ] テストケース5: 存在しないサーバー
- [ ] テストケース6: resources_readエラーケース
- [ ] テストケース7: prompts_getエラーケース

### 品質確認

- [ ] すべてのツールが正常に動作する
- [ ] エラーメッセージが明確で理解しやすい
- [ ] JSON形式の出力が整形されている
- [ ] パフォーマンスが許容範囲内（各コマンド10秒以内）

---

## トラブルシューティング

### エラー: "Method is required"

**原因:** コマンド構文が間違っている

**解決方法:**
```bash
# 正しい構文
npx @modelcontextprotocol/inspector --cli -- cargo run ... --method tools/list

# 間違った構文
npx @modelcontextprotocol/inspector --cli cargo run ... --method tools/list  # -- が欠落
```

### エラー: "Connection closed"

**原因:** サーバーが正しくビルドされていない、またはパスが間違っている

**解決方法:**
1. サーバーを再ビルド: `cargo build --release`
2. パスを確認: `--manifest-path` が正しいか確認
3. サーバー単体で動作確認: `cargo run`

### エラー: npxコマンドが見つからない

**原因:** Node.jsがインストールされていない、またはPATHが通っていない

**解決方法:**
```bash
# Git Bashの場合
export PATH="/c/Program Files/Volta:$PATH"

# または
export PATH="/c/Program Files/nodejs:$PATH"
```

---

## 次のステップ

### テスト完了後

1. **テスト結果の記録**
   - すべてのテストケースの結果を記録
   - 問題があれば詳細を記録

2. **README.md更新**
   - Phase 2の機能説明を追加
   - 使用例を追加

3. **実地テスト**
   - Claude Desktopでの動作確認
   - 他のMCPサーバーでのテスト

### 参考資料

- **詳細なCLI使用方法:** `MCP_INSPECTOR_CLI_GUIDE.md`
- **Phase 2実装計画:** `docs/PHASE2_PLAN.md`
- **Phase 2テストレポート:** `docs/PHASE2_TEST_REPORT.md`

---

**テスト実施日:** _____________________
**実施者:** _____________________
**結果:** ✅ 成功 / ❌ 失敗 / ⚠️ 部分的成功

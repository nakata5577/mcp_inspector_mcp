# Phase 2 実装計画書: Resources と Prompts 機能の追加

## 1. 概要

### 1.1 Phase 2の目的

Phase 2では、MCP Inspector MCPサーバーに以下の4つの新規ツールを追加し、MCPサーバーの**Resources（リソース）**と**Prompts（プロンプト）**機能を検査できるようにします。

- `resources_list` - サーバーが提供するリソースの一覧取得
- `resources_read` - 特定のリソースの内容読み込み
- `prompts_list` - サーバーが提供するプロンプトテンプレートの一覧取得
- `prompts_get` - 特定のプロンプトテンプレートの取得

### 1.2 Phase 1との関連性

Phase 1では`tools_list`と`tools_call`を実装し、以下の基盤を構築しました：

- **StdioClient** - MCPサーバーとの通信基盤
- **InspectorService** - ビジネスロジック層
- **InspectorServer** - MCPサーバーツールの公開層
- **データモデル** - リクエスト/レスポンス型の定義

Phase 2はこの確立されたアーキテクチャパターンを踏襲し、新機能を追加します。

---

## 2. 新規追加ツール仕様

### 2.1 resources_list

MCPサーバーが提供するリソースの一覧を取得します。

#### 引数
| パラメータ | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `server` | string | ✅ | 対象のMCPサーバー名 |

#### 戻り値
```rust
ResourcesListResponse {
    server: String,           // サーバー名
    resources: Vec<ResourceInfo>  // リソース情報のリスト
}

ResourceInfo {
    uri: String,                    // リソースのURI
    name: Option<String>,           // 人間が読みやすいリソース名
    description: Option<String>,    // リソースの説明
    mime_type: Option<String>       // MIMEタイプ（例: "text/plain", "image/png"）
}
```

#### 使用例
```bash
> tools/call resources_list '{"server": "fundamental_analysis"}'
```

---

### 2.2 resources_read

特定のリソースの内容を読み込みます。

#### 引数
| パラメータ | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `server` | string | ✅ | 対象のMCPサーバー名 |
| `uri` | string | ✅ | 読み込むリソースのURI |

#### 戻り値
```rust
ResourceReadResponse {
    server: String,                      // サーバー名
    uri: String,                         // リソースのURI
    contents: Vec<ResourceContent>       // リソースの内容（複数可）
}

ResourceContent {
    uri: String,                    // コンテンツのURI
    mime_type: Option<String>,      // MIMEタイプ
    text: Option<String>,           // テキストコンテンツ
    blob: Option<String>            // バイナリコンテンツ（Base64エンコード済み）
}
```

#### 使用例
```bash
> tools/call resources_read '{"server": "fundamental_analysis", "uri": "file:///path/to/resource.txt"}'
```

---

### 2.3 prompts_list

MCPサーバーが提供するプロンプトテンプレートの一覧を取得します。

#### 引数
| パラメータ | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `server` | string | ✅ | 対象のMCPサーバー名 |

#### 戻り値
```rust
PromptsListResponse {
    server: String,             // サーバー名
    prompts: Vec<PromptInfo>    // プロンプト情報のリスト
}

PromptInfo {
    name: String,                       // プロンプト名
    description: Option<String>,        // プロンプトの説明
    arguments: Vec<PromptArgument>      // プロンプト引数の定義
}

PromptArgument {
    name: String,                   // 引数名
    description: Option<String>,    // 引数の説明
    required: bool                  // 必須かどうか
}
```

#### 使用例
```bash
> tools/call prompts_list '{"server": "fundamental_analysis"}'
```

---

### 2.4 prompts_get

特定のプロンプトテンプレートを取得し、引数を適用した結果を返します。

#### 引数
| パラメータ | 型 | 必須 | 説明 |
|-----------|------|------|------|
| `server` | string | ✅ | 対象のMCPサーバー名 |
| `name` | string | ✅ | プロンプト名 |
| `arguments` | Map<String, String> | ❌ | プロンプトに渡す引数 |

#### 戻り値
```rust
PromptGetResponse {
    server: String,                 // サーバー名
    name: String,                   // プロンプト名
    messages: Vec<PromptMessage>    // プロンプトメッセージのリスト
}

PromptMessage {
    role: String,               // ロール（"user", "assistant", "system"）
    content: PromptContent      // メッセージの内容（テキストまたは画像）
}

PromptContent {
    // 実際の型は rmcp::PromptContent に準拠
    // テキストまたは画像（Base64）を含む
}
```

#### 使用例
```bash
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "analyze_company", "arguments": {"ticker": "AAPL"}}'
```

---

## 3. データ構造設計

### 3.1 models/request.rs に追加する型

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// リソース一覧取得のリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListRequest {
    pub server: String,
}

/// リソース読み込みのリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadRequest {
    pub server: String,
    pub uri: String,
}

/// プロンプト一覧取得のリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListRequest {
    pub server: String,
}

/// プロンプト取得のリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGetRequest {
    pub server: String,
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, String>,
}
```

### 3.2 models/response.rs に追加する型

```rust
use serde::{Deserialize, Serialize};

/// リソース一覧のレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResponse {
    pub server: String,
    pub resources: Vec<ResourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// リソース読み込みのレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadResponse {
    pub server: String,
    pub uri: String,
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,  // Base64エンコード済み
}

/// プロンプト一覧のレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListResponse {
    pub server: String,
    pub prompts: Vec<PromptInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// プロンプト取得のレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGetResponse {
    pub server: String,
    pub name: String,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: serde_json::Value,  // rmcp::PromptContentから変換
}
```

---

## 4. 実装フェーズ

### Phase 2.1: データモデル実装

**タスク:**
- `src/models/request.rs`に新規リクエスト型を追加
- `src/models/response.rs`に新規レスポンス型を追加
- `src/models/mod.rs`で型をエクスポート

**実装ファイル:**
- `src/models/request.rs`
- `src/models/response.rs`
- `src/models/mod.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- `cargo check` がパス
- `cargo clippy` で警告なし
- すべての型にSerialize/Deserialize実装

---

### Phase 2.2: クライアント機能拡張

**タスク:**
- `StdioClient`に以下のメソッドを追加：
  - `list_resources(&mut self, server: &str) -> Result<Vec<ResourceInfo>>`
  - `read_resource(&mut self, server: &str, uri: &str) -> Result<Vec<ResourceContent>>`
  - `list_prompts(&mut self, server: &str) -> Result<Vec<PromptInfo>>`
  - `get_prompt(&mut self, server: &str, name: &str, arguments: HashMap<String, String>) -> Result<Vec<PromptMessage>>`
- `rmcp::Resource`, `rmcp::Prompt` 型から自前の型への変換実装

**実装ファイル:**
- `src/client/stdio.rs`

**見積もり:** 3-4時間

**チェックポイント:**
- 各メソッドがrmcp APIを正しく呼び出す
- エラーハンドリングが適切
- 型変換が正しく動作

**技術的考慮事項:**
```rust
// rmcp::Resource から ResourceInfo への変換例
impl From<rmcp::Resource> for ResourceInfo {
    fn from(resource: rmcp::Resource) -> Self {
        ResourceInfo {
            uri: resource.uri,
            name: resource.name,
            description: resource.description,
            mime_type: resource.mime_type,
        }
    }
}
```

---

### Phase 2.3: サービスレイヤー実装

**タスク:**
- `InspectorService`に以下のメソッドを追加：
  - `resources_list(&mut self, request: ResourcesListRequest) -> Result<ResourcesListResponse>`
  - `resources_read(&mut self, request: ResourceReadRequest) -> Result<ResourceReadResponse>`
  - `prompts_list(&mut self, request: PromptsListRequest) -> Result<PromptsListResponse>`
  - `prompts_get(&mut self, request: PromptGetRequest) -> Result<PromptGetResponse>`
- 各メソッドでStdioClientを呼び出し、結果をレスポンス型に変換

**実装ファイル:**
- `src/services/inspector.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- ビジネスロジックの適切な実装
- エラーハンドリングの一貫性
- Phase 1パターンとの統一感

---

### Phase 2.4: サーバーツール実装

**タスク:**
- `InspectorServer`に`#[tool]`マクロで4つのツールを追加：
  - `resources_list`
  - `resources_read`
  - `prompts_list`
  - `prompts_get`
- 各ツールメソッドで`InspectorService`を呼び出し

**実装ファイル:**
- `src/server.rs`

**見積もり:** 2-3時間

**チェックポイント:**
- ツールがMCP Inspectorから呼び出し可能
- 引数の検証が適切
- Phase 1の実装パターンと一貫性

**実装例:**
```rust
#[tool(description = "指定されたMCPサーバーが提供するリソースの一覧を取得します")]
async fn resources_list(&self, request: ResourcesListRequest) -> Result<ResourcesListResponse> {
    let mut service = self.create_service(&request.server).await?;
    service.resources_list(request).await
}
```

---

### Phase 2.5: テストと検証

**タスク:**
- MCP Inspector CLI での手動テスト
- エラーケースの確認
- ドキュメントの検証

**テスト環境:**
```bash
npx @modelcontextprotocol/inspector --cli \
  node c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp/build/index.js \
  -- cargo run
```

**見積もり:** 3-4時間

**チェックポイント:**
- すべてのツールが正常に動作
- エラーハンドリングが適切
- `cargo test` がパス

---

## 5. 技術的考慮事項

### 5.1 URIハンドリング

**課題:**
- リソースURIは多様な形式をサポート（`file://`, `http://`, カスタムスキーム）
- 相対パス/絶対パスの適切な処理

**対策:**
- URIの検証は基本的にrmcp側に委ねる
- 自前の検証は最小限に留める
- エラーメッセージでURIの形式を明示

**実装例:**
```rust
pub async fn read_resource(&mut self, uri: &str) -> Result<Vec<ResourceContent>> {
    // rmcpがURI検証を行うため、ここでは最小限の検証のみ
    if uri.is_empty() {
        return Err(anyhow::anyhow!("URI cannot be empty"));
    }

    let response = self.client.read_resource(uri).await?;
    // ... 変換処理
}
```

---

### 5.2 バイナリデータ

**課題:**
- リソースコンテンツはテキストとバイナリの両方をサポート
- バイナリデータはBase64エンコーディングが必要

**対策:**
- rmcpが提供する`blob`フィールドは既にBase64エンコード済み
- 自前でのエンコード/デコードは不要
- `text`と`blob`の排他的な使用を保証

**実装例:**
```rust
impl From<rmcp::ResourceContent> for ResourceContent {
    fn from(content: rmcp::ResourceContent) -> Self {
        ResourceContent {
            uri: content.uri,
            mime_type: content.mime_type,
            text: content.text,
            blob: content.blob,  // 既にBase64エンコード済み
        }
    }
}
```

---

### 5.3 エラーハンドリング

**考慮すべきエラーケース:**

| エラーケース | 発生場所 | 対応方法 |
|------------|---------|---------|
| 存在しないリソース | `resources_read` | 404相当のエラーメッセージ |
| 存在しないプロンプト | `prompts_get` | 404相当のエラーメッセージ |
| 無効なURI | `resources_read` | URI形式エラーを明示 |
| 必須引数の欠落 | `prompts_get` | 欠落している引数名を明示 |
| サーバー接続エラー | 全ツール | 接続エラーを明確に通知 |

**実装パターン:**
```rust
pub async fn resources_read(&mut self, request: ResourceReadRequest) -> Result<ResourceReadResponse> {
    let contents = self.client.read_resource(&request.uri)
        .await
        .context(format!("Failed to read resource: {}", request.uri))?;

    Ok(ResourceReadResponse {
        server: request.server,
        uri: request.uri,
        contents,
    })
}
```

---

## 6. テストプラン

### 6.1 テストケース1: resources_list

**目的:** リソース一覧の取得が正常に動作することを確認

**手順:**
```bash
# MCP Inspector CLI を起動
npx @modelcontextprotocol/inspector --cli \
  node c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp/build/index.js \
  -- cargo run

# ツール一覧を確認
> tools/list

# resources_list を呼び出し
> tools/call resources_list '{"server": "fundamental_analysis"}'
```

**期待される結果:**
```json
{
  "server": "fundamental_analysis",
  "resources": [
    {
      "uri": "file:///path/to/resource",
      "name": "Example Resource",
      "description": "A sample resource",
      "mime_type": "text/plain"
    }
  ]
}
```

---

### 6.2 テストケース2: resources_read

**目的:** リソースの読み込みが正常に動作することを確認

**手順:**
```bash
# resources_list で取得したURIを使用
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
      "text": "Resource content here",
      "blob": null
    }
  ]
}
```

**エラーケース:**
```bash
# 存在しないURI
> tools/call resources_read '{"server": "fundamental_analysis", "uri": "file:///nonexistent"}'
# 期待: エラーメッセージ
```

---

### 6.3 テストケース3: prompts_list

**目的:** プロンプト一覧の取得が正常に動作することを確認

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
        }
      ]
    }
  ]
}
```

---

### 6.4 テストケース4: prompts_get

**目的:** プロンプトの取得が正常に動作することを確認

**手順:**
```bash
# 引数ありの場合
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "analyze_company", "arguments": {"ticker": "AAPL"}}'

# 引数なしの場合
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "simple_prompt"}'
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
        "text": "Analyze AAPL company..."
      }
    }
  ]
}
```

**エラーケース:**
```bash
# 存在しないプロンプト
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "nonexistent"}'

# 必須引数の欠落
> tools/call prompts_get '{"server": "fundamental_analysis", "name": "analyze_company"}'
```

---

### 6.5 ユニットテスト

**実装ファイル:** `src/services/inspector.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resources_list_request_format() {
        let request = ResourcesListRequest {
            server: "test_server".to_string(),
        };

        assert_eq!(request.server, "test_server");
    }

    #[tokio::test]
    async fn test_resource_read_request_format() {
        let request = ResourceReadRequest {
            server: "test_server".to_string(),
            uri: "file:///test".to_string(),
        };

        assert_eq!(request.uri, "file:///test");
    }

    // ... 他のテストケース
}
```

---

## 7. 成功基準

Phase 2の完了は、以下の基準をすべて満たすことで判断します：

### 7.1 機能要件
- ✅ 4つの新ツール（resources_list, resources_read, prompts_list, prompts_get）が実装されている
- ✅ すべてのツールがMCP Inspector CLIから呼び出し可能
- ✅ エラーハンドリングが適切に実装されている
- ✅ 型変換が正しく動作している

### 7.2 品質要件
- ✅ `cargo check` がエラーなしでパス
- ✅ `cargo clippy` が警告なしでパス
- ✅ `cargo test` がすべてパス
- ✅ `cargo build --release` が成功

### 7.3 テスト要件
- ✅ MCP Inspector CLIでのすべてのテストケースが成功
- ✅ エラーケースが適切に処理される
- ✅ fundamental_analysis MCPサーバーとの統合テストが成功

### 7.4 ドキュメント要件
- ✅ 各ツールの使用例がドキュメント化されている
- ✅ エラーメッセージが明確で理解しやすい

---

## 8. 見積もり

| フェーズ | タスク | 見積もり時間 |
|---------|--------|------------|
| Phase 2.1 | データモデル実装 | 2-3時間 |
| Phase 2.2 | クライアント機能拡張 | 3-4時間 |
| Phase 2.3 | サービスレイヤー実装 | 2-3時間 |
| Phase 2.4 | サーバーツール実装 | 2-3時間 |
| Phase 2.5 | テストと検証 | 3-4時間 |
| **合計** | - | **12-17時間** |

**備考:**
- 見積もりは開発経験に基づく標準的な時間
- Phase 1の実装パターンを踏襲するため、学習コストは最小限
- 予期しない問題が発生した場合は+20%のバッファを見込む

---

## 9. リスクと対策

### リスク1: rmcp API の理解不足

**影響度:** 高
**発生確率:** 中

**対策:**
- Phase 2.2 の開始前に [docs.rs/rmcp](https://docs.rs/rmcp/0.8.5/) で詳細確認
- `rmcp::Resource`, `rmcp::Prompt` の実装を読む
- 不明点があればサンプルコードを作成して動作確認

---

### リスク2: バイナリデータ処理の複雑さ

**影響度:** 中
**発生確率:** 低

**対策:**
- rmcpが提供するBase64エンコード済みデータをそのまま使用
- Phase 1のJSON処理パターンを踏襲
- テストケースでバイナリリソースの読み込みを検証

---

### リスク3: URI処理の複雑さ

**影響度:** 中
**発生確率:** 中

**対策:**
- URI検証はrmcp側に委ねる
- エラーメッセージで問題のあるURIを明示
- 標準的なURI形式（file://, http://）でテスト

---

### リスク4: fundamental_analysis MCPサーバーの仕様変更

**影響度:** 低
**発生確約:** 低

**対策:**
- fundamental_analysis MCPサーバーのバージョンを固定
- テスト前にサーバーの動作を確認
- 代替のテストサーバーも準備

---

## 10. 依存関係

### 10.1 前提条件

| 項目 | ステータス | 備考 |
|------|----------|------|
| Phase 1 完了 | ✅ | tools_list, tools_call 実装済み |
| rmcp 0.8.5 | ✅ | Cargo.tomlで依存関係設定済み |
| fundamental_analysis MCP | ✅ | テスト用サーバーとして使用 |
| MCP Inspector CLI | ✅ | 検証環境として使用 |

### 10.2 技術スタック

- **Rust:** 1.70以上
- **rmcp:** 0.8.5
- **tokio:** 非同期ランタイム
- **anyhow:** エラーハンドリング
- **serde:** シリアライゼーション

---

## 11. 次のステップ

Phase 2完了後、以下のタスクを実施します：

### 11.1 ドキュメント更新
- [ ] README.md に Resources と Prompts 機能を追加
- [ ] 使用例セクションに新ツールの例を追加
- [ ] アーキテクチャ図の更新

### 11.2 実地テスト
- [ ] Claude Desktop での統合テスト
- [ ] fundamental_analysis MCP との実運用テスト
- [ ] 他のMCPサーバー（filesystem, github等）でのテスト

### 11.3 Phase 3 準備
- [ ] Phase 3 計画書の作成（Sampling機能の追加）
- [ ] リファクタリングの検討
- [ ] パフォーマンス最適化の検討

---

## 12. 参考資料

### 12.1 公式ドキュメント

- **MCP Protocol Specification**
  https://modelcontextprotocol.io/

- **rmcp Documentation**
  https://docs.rs/rmcp/0.8.5/

- **MCP Inspector**
  https://github.com/modelcontextprotocol/inspector

### 12.2 関連リソース

- **Phase 1 実装**
  `c:/Users/takah/work/my_mcp_server/mcp_inspector_mcp/src/`

- **fundamental_analysis MCP**
  `c:/Users/takah/work/my_mcp_server/fundamental_analysis_mcp/`

### 12.3 学習リソース

- **MCPの概念理解**
  MCP Protocol Specification - Resources
  MCP Protocol Specification - Prompts

- **rmcpの使い方**
  docs.rs/rmcp - Resource API
  docs.rs/rmcp - Prompt API

---

## 13. 変更履歴

| 日付 | バージョン | 変更内容 | 作成者 |
|------|----------|---------|-------|
| 2025-11-14 | 1.0 | 初版作成 | Claude Code |

---

## 付録A: データフロー図

```
[MCP Inspector CLI]
        ↓
  tools/call resources_list
        ↓
[InspectorServer]
  #[tool] resources_list
        ↓
[InspectorService]
  resources_list()
        ↓
[StdioClient]
  list_resources()
        ↓
[rmcp::Client]
  resources/list
        ↓
[Target MCP Server]
  (fundamental_analysis)
        ↓
[rmcp::Resource]
        ↓
[ResourceInfo (変換)]
        ↓
[ResourcesListResponse]
        ↓
[JSON serialization]
        ↓
[MCP Inspector CLI 表示]
```

---

## 付録B: エラーコード一覧

| コード | 説明 | 発生場所 |
|--------|------|---------|
| E001 | サーバー接続エラー | StdioClient |
| E002 | 無効なURI形式 | resources_read |
| E003 | リソースが見つからない | resources_read |
| E004 | プロンプトが見つからない | prompts_get |
| E005 | 必須引数の欠落 | prompts_get |
| E006 | JSON変換エラー | 全ツール |

---

**以上がPhase 2実装計画書です。この計画に従って実装を進めることで、MCP Inspector MCPサーバーにResources と Prompts 検査機能を追加できます。**

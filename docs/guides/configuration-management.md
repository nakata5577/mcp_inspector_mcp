# 構成管理ガイド

**対象バージョン**: v0.4.0+
**最終更新**: 2025-11-20

---

## 目次

- [概要](#概要)
- [プロファイル機能](#プロファイル機能)
- [インポート/エクスポート](#インポートエクスポート)
- [テンプレート機能](#テンプレート機能)
- [実践的な使用例](#実践的な使用例)
- [ベストプラクティス](#ベストプラクティス)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

構成管理拡張機能により、環境別の設定管理が容易になりました。プロファイル、インポート/エクスポート、テンプレート機能を使用して、開発・ステージング・本番環境の設定を効率的に管理できます。

### 構成管理の特徴

- **プロファイル機能**: 環境別設定の切り替え
- **インポート/エクスポート**: 設定の移植性向上
- **テンプレート機能**: 設定の標準化
- **差分表示**: 設定変更の可視化
- **設定検証**: 設定ファイルの正確性チェック

### ユースケース

- **環境切り替え**: 開発/ステージング/本番の切り替え
- **設定のバックアップ**: 定期的な設定のエクスポート
- **チーム間の共有**: 設定ファイルの共有
- **標準化**: テンプレートによる設定の統一
- **マイグレーション**: 環境間の設定移行

---

## プロファイル機能

プロファイル機能により、環境別の設定を簡単に切り替えられます。

### プリセットプロファイル

| プロファイル | 用途 | 特徴 |
|------------|------|------|
| `default` | デフォルト | 標準設定 |
| `dev` | 開発環境 | デバッグログ有効、短いタイムアウト |
| `staging` | ステージング | 本番に近い設定、詳細ログ |
| `prod` | 本番環境 | 最適化された設定、エラーログのみ |

### プロファイルの作成

プロファイル設定ファイルは`.inspector/`ディレクトリに配置します。

**ファイル命名規則**:
```
config.{profile}.json
```

**例**:
- `config.dev.json` - 開発環境
- `config.staging.json` - ステージング環境
- `config.prod.json` - 本番環境

### プロファイル設定例

#### 開発環境 (config.dev.json)

```json
{
  "servers": [
    {
      "name": "test_server",
      "transport": "stdio",
      "command": "C:/dev/test_server.exe",
      "args": ["--verbose"],
      "env": {}
    }
  ],
  "logging": {
    "backend": "memory",
    "max_logs": 1000
  },
  "execution_config": {
    "tool_timeout_ms": 10000,
    "connection_timeout_ms": 3000,
    "retry_count": 0,
    "verbose": true
  }
}
```

#### ステージング環境 (config.staging.json)

```json
{
  "servers": [
    {
      "name": "staging_server",
      "transport": "stdio",
      "command": "C:/staging/server.exe",
      "args": [],
      "env": {
        "ENVIRONMENT": "staging"
      }
    }
  ],
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs_staging.db",
    "max_logs": 5000
  },
  "execution_config": {
    "tool_timeout_ms": 30000,
    "connection_timeout_ms": 5000,
    "retry_count": 1
  }
}
```

#### 本番環境 (config.prod.json)

```json
{
  "servers": [
    {
      "name": "prod_server",
      "transport": "stdio",
      "command": "C:/production/server.exe",
      "args": [],
      "env": {
        "ENVIRONMENT": "production"
      }
    }
  ],
  "logging": {
    "backend": "persistent",
    "db_path": "./data/logs_prod.db",
    "max_logs": 10000
  },
  "execution_config": {
    "tool_timeout_ms": 60000,
    "connection_timeout_ms": 10000,
    "retry_count": 2,
    "verbose": false
  }
}
```

### プロファイルの使用

#### コマンドラインフラグ

```bash
# プロファイル指定で起動
mcp_inspector_mcp --profile dev

# 本番環境で起動
mcp_inspector_mcp --profile prod
```

#### 環境変数

```bash
# 環境変数で指定（推奨: CI/CD環境）
export MCP_PROFILE=staging
mcp_inspector_mcp

# Windows (PowerShell)
$env:MCP_PROFILE = "prod"
mcp_inspector_mcp
```

### プロファイル管理コマンド

#### プロファイル一覧表示

```bash
mcp_inspector_mcp --list-profiles
```

**出力例**:
```
Available Profiles:
  - default (active)
  - dev
  - staging
  - prod
```

#### プロファイル検証

```bash
# プロファイル設定の検証
mcp_inspector_mcp --validate-profile prod
```

**出力例**:
```
✅ Profile 'prod' is valid
  - Servers: 1
  - Logging: persistent
  - Execution Config: configured
```

---

## インポート/エクスポート

設定のインポート/エクスポート機能により、設定の移植性が向上します。

### エクスポート

#### JSON形式

```bash
# 現在の設定をエクスポート
mcp_inspector_mcp config export --output config_backup.json

# 特定のプロファイルをエクスポート
mcp_inspector_mcp --profile prod config export --output config_prod_backup.json
```

**出力例 (config_backup.json)**:
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
  ],
  "logging": {
    "backend": "memory",
    "db_path": "./data/logs.db",
    "max_logs": 10000
  },
  "execution_config": {
    "tool_timeout_ms": 30000,
    "connection_timeout_ms": 5000,
    "retry_count": 0
  }
}
```

#### YAML形式

```bash
# YAML形式でエクスポート
mcp_inspector_mcp config export --output config_backup.yaml --format yaml
```

**出力例 (config_backup.yaml)**:
```yaml
servers:
  - name: fundamental_analysis
    transport: stdio
    command: C:/path/to/fa.exe
    args: []
    env: {}

logging:
  backend: memory
  db_path: ./data/logs.db
  max_logs: 10000

execution_config:
  tool_timeout_ms: 30000
  connection_timeout_ms: 5000
  retry_count: 0
```

### インポート

#### dry-run モード

インポート前に変更内容を確認できます。

```bash
# dry-run で差分表示
mcp_inspector_mcp config import --input config_backup.json --dry-run
```

**出力例**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Configuration Diff
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  servers:
    - name: fundamental_analysis
-     command: C:/old/path/fa.exe
+     command: C:/new/path/fa.exe

  execution_config:
-   tool_timeout_ms: 30000
+   tool_timeout_ms: 60000

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Changes: 2 modified fields
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Dry-run mode: No changes were made.
Run without --dry-run to apply changes.
```

#### 実際のインポート

```bash
# 設定をインポート
mcp_inspector_mcp config import --input config_backup.json
```

**出力例**:
```
✅ Configuration imported successfully
  - 2 fields modified
  - Config file updated: .inspector/config.json
```

### 設定バリデーション

```bash
# 設定ファイルの検証
mcp_inspector_mcp config validate --input config_backup.json
```

**出力例**:
```
✅ Configuration is valid
  - Servers: 1
  - Logging: configured
  - Execution Config: configured
  - No errors found
```

---

## テンプレート機能

テンプレート機能により、標準化された設定を簡単に適用できます。

### プリセットテンプレート

| テンプレート | 用途 | 特徴 |
|------------|------|------|
| `minimal` | 最小構成 | 1サーバー、memory logging |
| `development` | 開発環境 | 詳細ログ、短いタイムアウト |
| `production` | 本番環境 | 最適化設定、persistent logging |
| `ci` | CI/CD | 高速、fail-fast |

### テンプレート一覧表示

```bash
mcp_inspector_mcp config template list
```

**出力例**:
```
Available Templates:
  Preset Templates:
    - minimal: Minimal configuration with single server
    - development: Development environment configuration
    - production: Production environment configuration
    - ci: CI/CD optimized configuration

  Custom Templates:
    - my_team_template: Custom team configuration
```

### テンプレート表示

```bash
# テンプレート内容を表示
mcp_inspector_mcp config template show --template production
```

**出力例**:
```yaml
servers:
  - name: example_server
    transport: stdio
    command: /path/to/server
    args: []
    env: {}

logging:
  backend: persistent
  db_path: ./data/logs.db
  max_logs: 10000

execution_config:
  tool_timeout_ms: 60000
  connection_timeout_ms: 10000
  retry_count: 2
```

### テンプレート適用

```bash
# プロファイルにテンプレートを適用
mcp_inspector_mcp config template apply --template production --profile prod
```

**出力例**:
```
✅ Template 'production' applied to profile 'prod'
  - Profile file created: .inspector/config.prod.json
  - You can now use: mcp_inspector_mcp --profile prod
```

### カスタムテンプレート作成

```bash
# 現在のプロファイルからテンプレート作成
mcp_inspector_mcp config template create --name my_template --from-profile dev
```

**出力例**:
```
✅ Custom template 'my_template' created
  - Template saved to: .inspector/templates/my_template.json
  - Use with: mcp_inspector_mcp config template apply --template my_template --profile {name}
```

### カスタムテンプレート削除

```bash
# カスタムテンプレート削除
mcp_inspector_mcp config template delete --name my_template
```

**出力例**:
```
✅ Custom template 'my_template' deleted
```

---

## 実践的な使用例

### 例1: 開発環境から本番環境への移行

開発環境で検証した設定を本番環境に移行します。

```bash
# 1. 開発環境の設定をエクスポート
mcp_inspector_mcp --profile dev config export --output dev_config.json

# 2. 設定を編集（サーバーパスの変更など）
# dev_config.json を手動編集

# 3. 本番環境にインポート（dry-run）
mcp_inspector_mcp --profile prod config import --input dev_config.json --dry-run

# 4. 差分を確認後、実際にインポート
mcp_inspector_mcp --profile prod config import --input dev_config.json

# 5. 本番環境で起動
mcp_inspector_mcp --profile prod
```

### 例2: チーム間での設定共有

チーム内で標準的な設定を共有します。

```bash
# 1. チーム標準テンプレート作成
mcp_inspector_mcp config template create --name team_standard --from-profile dev

# 2. テンプレートファイルをGitにコミット
git add .inspector/templates/team_standard.json
git commit -m "Add team standard configuration template"
git push

# 3. 他のメンバーがテンプレートを適用
git pull
mcp_inspector_mcp config template apply --template team_standard --profile dev
```

### 例3: CI/CD環境の設定

CI/CD環境で異なるプロファイルを使用します。

**GitHub Actions (.github/workflows/ci.yml)**:

```yaml
name: CI Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install MCP Inspector
        run: cargo install --path .

      - name: Set up CI profile
        run: |
          mcp_inspector_mcp config template apply --template ci --profile ci

      - name: Run tests
        run: |
          mcp_inspector_mcp --profile ci test run --suite tests/ci_test.yaml
```

### 例4: 定期的な設定バックアップ

設定を定期的にバックアップします。

```bash
# crontab -e で以下を追加
# 毎日午前2時にバックアップ
0 2 * * * /path/to/mcp_inspector_mcp config export --output /backup/config_$(date +\%Y\%m\%d).json

# 7日以上古いバックアップを削除
0 3 * * * find /backup -name "config_*.json" -mtime +7 -delete
```

---

## ベストプラクティス

### 1. プロファイル命名規則

- **環境名を使用**: dev、staging、prod
- **チーム名を含める**: team_a_dev、team_b_prod
- **用途を明確に**: load_test、ci、debug

### 2. 設定のバージョン管理

```bash
# Gitで設定ファイルを管理
git add .inspector/config.*.json
git commit -m "Update staging configuration"

# 機密情報は環境変数で管理
# config.json内ではプレースホルダーを使用
```

### 3. テンプレートの活用

- **標準テンプレート作成**: チーム標準設定をテンプレート化
- **環境別テンプレート**: dev、staging、prod用のテンプレート
- **用途別テンプレート**: testing、debugging、monitoring用

### 4. dry-run の活用

インポート前に必ず差分を確認します。

```bash
# 常にdry-runで確認
mcp_inspector_mcp config import --input config.json --dry-run

# 確認後にインポート
mcp_inspector_mcp config import --input config.json
```

### 5. 設定の検証

設定変更後は必ず検証します。

```bash
# 設定検証
mcp_inspector_mcp config validate --input config.json

# プロファイル検証
mcp_inspector_mcp --validate-profile prod
```

---

## トラブルシューティング

### プロファイルが見つからない

**症状**: `--profile dev`を指定してもエラーになる

**原因**: プロファイルファイルが存在しない

**対策**:

```bash
# プロファイル一覧を確認
mcp_inspector_mcp --list-profiles

# プロファイルファイルを作成
mcp_inspector_mcp config template apply --template development --profile dev

# 手動で作成
cp .inspector/config.json .inspector/config.dev.json
```

### インポートが失敗する

**症状**: `config import`を実行するとエラーになる

**原因**: 設定ファイルの形式が不正

**対策**:

```bash
# 設定ファイルのバリデーション
mcp_inspector_mcp config validate --input config.json

# JSON構文チェック（jqツール使用）
cat config.json | jq .

# YAML構文チェック
yamllint config.yaml
```

### 差分が表示されない

**症状**: `--dry-run`で差分が表示されない

**原因**: インポートする設定が現在の設定と同じ

**対策**:

```bash
# 現在の設定をエクスポート
mcp_inspector_mcp config export --output current.json

# 差分を手動で確認
diff current.json import.json
```

---

## 参考リンク

- [README.md](../../README.md): 全体的な使い方
- [デバッグモードガイド](./debug-mode.md): デバッグ方法
- [バッチテストガイド](./batch-testing.md): テスト自動化
- [パフォーマンスモニタリングガイド](./performance-monitoring.md): 性能分析
- [プロファイル例](../../examples/profiles/): サンプルプロファイル

---

**最終更新**: 2025-11-20
**対象バージョン**: v0.4.0+

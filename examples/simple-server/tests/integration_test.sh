#!/bin/bash
# Simple MCP Server 統合テストスクリプト

set -e

# カラー出力定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ログ関数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

log_info "Simple MCP Server 統合テスト開始"
log_info "プロジェクトディレクトリ: $PROJECT_DIR"

# ステップ1: ビルド
log_info "ステップ1: サーバーのビルド"
cd "$PROJECT_DIR"
if cargo build 2>&1; then
    log_info "✓ ビルド成功"
else
    log_error "✗ ビルド失敗"
    exit 1
fi

# ステップ2: MCP Inspector MCPの確認
log_info "ステップ2: MCP Inspector MCPの確認"
INSPECTOR_DIR="$(cd "$PROJECT_DIR/../.." && pwd)"
log_info "Inspector ディレクトリ: $INSPECTOR_DIR"

if [ ! -f "$INSPECTOR_DIR/Cargo.toml" ]; then
    log_error "MCP Inspector MCPが見つかりません"
    exit 1
fi

# ステップ3: サーバーの基本動作確認
log_info "ステップ3: サーバーの基本動作確認"

# サーバーを起動してinitializeリクエストを送信
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --quiet 2>/dev/null)

if echo "$TEST_RESULT" | grep -q "simple-server"; then
    log_info "✓ Initialize成功"
else
    log_error "✗ Initialize失敗"
    echo "レスポンス: $TEST_RESULT"
    exit 1
fi

# ステップ4: ツールのテスト
log_info "ステップ4: ツールのテスト"

# tools/list
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "echo"; then
    log_info "✓ tools/list成功"
else
    log_error "✗ tools/list失敗"
    exit 1
fi

# echo ツール
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello, MCP!"}}}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "Hello, MCP!"; then
    log_info "✓ echo ツール成功"
else
    log_error "✗ echo ツール失敗"
    exit 1
fi

# reverse ツール
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"reverse","arguments":{"text":"Hello"}}}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "olleH"; then
    log_info "✓ reverse ツール成功"
else
    log_error "✗ reverse ツール失敗"
    exit 1
fi

# uppercase ツール
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"uppercase","arguments":{"text":"hello"}}}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "HELLO"; then
    log_info "✓ uppercase ツール成功"
else
    log_error "✗ uppercase ツール失敗"
    exit 1
fi

# ステップ5: リソースのテスト
log_info "ステップ5: リソースのテスト"

# resources/list
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":6,"method":"resources/list"}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "greeting"; then
    log_info "✓ resources/list成功"
else
    log_error "✗ resources/list失敗"
    exit 1
fi

# resources/read
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"simple://greeting"}}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "こんにちは"; then
    log_info "✓ resources/read成功"
else
    log_error "✗ resources/read失敗"
    exit 1
fi

# ステップ6: プロンプトのテスト
log_info "ステップ6: プロンプトのテスト"

# prompts/list
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":8,"method":"prompts/list"}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "help"; then
    log_info "✓ prompts/list成功"
else
    log_error "✗ prompts/list失敗"
    exit 1
fi

# prompts/get
TEST_RESULT=$(echo '{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"help"}}' | cargo run --quiet 2>/dev/null)
if echo "$TEST_RESULT" | grep -q "使い方"; then
    log_info "✓ prompts/get成功"
else
    log_error "✗ prompts/get失敗"
    exit 1
fi

# ステップ7: 完了
log_info "========================================="
log_info "全てのテストが成功しました！"
log_info "========================================="
log_info "テスト済み機能:"
log_info "  - Initialize"
log_info "  - Tools (echo, reverse, uppercase)"
log_info "  - Resources (greeting)"
log_info "  - Prompts (help)"
log_info ""
log_info "次のステップ:"
log_info "  1. MCP Inspector MCPを使用して詳細な検査を実行"
log_info "  2. カスタムツールを追加して機能を拡張"
log_info "  3. エラーハンドリングを強化"

exit 0

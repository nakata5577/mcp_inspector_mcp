# パフォーマンスモニタリングガイド

**対象バージョン**: v0.4.0+
**最終更新**: 2025-11-20

---

## 目次

- [概要](#概要)
- [メトリクス収集](#メトリクス収集)
- [統計集計](#統計集計)
- [レポート生成](#レポート生成)
- [ボトルネック検出](#ボトルネック検出)
- [実践的な使用例](#実践的な使用例)
- [ベストプラクティス](#ベストプラクティス)
- [トラブルシューティング](#トラブルシューティング)

---

## 概要

パフォーマンスモニタリング機能は、MCPサーバーのリアルタイムパフォーマンスデータを収集・分析する機能です。レスポンスタイム、スループット、エラー率などのメトリクスを自動収集し、ボトルネックを早期に発見できます。

### パフォーマンスモニタリングの特徴

- **リアルタイムメトリクス収集**: 継続的なパフォーマンスデータ収集
- **時間窓ベース統計**: 1分/5分/15分の移動平均
- **多形式レポート**: Console/JSON/HTML形式
- **ボトルネック自動検出**: 性能問題の早期発見
- **低オーバーヘッド**: メトリクス収集のオーバーヘッド < 3%

### ユースケース

- **開発時**: 新機能のパフォーマンス検証
- **テスト時**: 負荷テスト結果の分析
- **本番環境**: リアルタイム性能監視
- **トラブルシューティング**: 性能問題の原因特定
- **最適化**: パフォーマンス改善の効果測定

---

## メトリクス収集

パフォーマンスモニタリングでは、以下のメトリクスを自動収集します。

### 収集されるメトリクス

#### 1. レスポンスタイム

| メトリクス | 説明 | 単位 |
|-----------|------|------|
| `avg_response_time` | 平均レスポンスタイム | ミリ秒 |
| `min_response_time` | 最小レスポンスタイム | ミリ秒 |
| `max_response_time` | 最大レスポンスタイム | ミリ秒 |
| `p50_response_time` | 50パーセンタイル | ミリ秒 |
| `p95_response_time` | 95パーセンタイル | ミリ秒 |
| `p99_response_time` | 99パーセンタイル | ミリ秒 |

#### 2. スループット

| メトリクス | 説明 | 単位 |
|-----------|------|------|
| `throughput` | リクエスト数/秒 | req/s |
| `total_requests` | 総リクエスト数 | 個 |

#### 3. エラー率

| メトリクス | 説明 | 単位 |
|-----------|------|------|
| `error_rate` | エラー率 | % |
| `total_errors` | 総エラー数 | 個 |

#### 4. 同時実行数

| メトリクス | 説明 | 単位 |
|-----------|------|------|
| `concurrent_requests` | 現在の同時実行リクエスト数 | 個 |
| `max_concurrent` | 最大同時実行数 | 個 |

### メトリクス収集の開始

```bash
# メトリクス収集開始
mcp_inspector_mcp monitor start

# 特定のサーバーのみ監視
mcp_inspector_mcp monitor start --server fundamental_analysis
```

### メトリクス収集の停止

```bash
# メトリクス収集停止
mcp_inspector_mcp monitor stop
```

### メトリクスのリセット

```bash
# 収集したメトリクスをリセット
mcp_inspector_mcp monitor reset
```

---

## 統計集計

時間窓ベースの統計集計により、短期/中期/長期のトレンドを分析できます。

### 時間窓

| 時間窓 | 説明 | 用途 |
|-------|------|------|
| 1分 | 直近1分間の統計 | リアルタイム監視 |
| 5分 | 直近5分間の統計 | 短期トレンド |
| 15分 | 直近15分間の統計 | 中期トレンド |

### 統計項目

#### 移動平均

各時間窓での平均値を計算します。

```bash
# 統計情報の表示
mcp_inspector_mcp monitor stats
```

**出力例**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Performance Statistics
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Time Window: 1 minute
  Avg Response Time: 245ms
  Throughput: 12.5 req/s
  Error Rate: 0.8%

Time Window: 5 minutes
  Avg Response Time: 267ms
  Throughput: 11.2 req/s
  Error Rate: 1.2%

Time Window: 15 minutes
  Avg Response Time: 289ms
  Throughput: 10.8 req/s
  Error Rate: 1.5%
```

#### パーセンタイル

レスポンスタイムの分布を理解するためのパーセンタイル値です。

- **P50（中央値）**: 50%のリクエストがこの時間以下
- **P95**: 95%のリクエストがこの時間以下
- **P99**: 99%のリクエストがこの時間以下

**P95/P99の重要性**:
- P50だけでは外れ値が見えない
- P95/P99で worst case のパフォーマンスを把握
- ユーザー体験の質を測定

---

## レポート生成

パフォーマンスレポートは3つの形式で生成できます。

### Console形式

ターミナルでの表示用です。

```bash
mcp_inspector_mcp monitor report
```

**出力例**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Performance Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated: 2025-11-20T10:30:45Z
Duration: 15 minutes

Summary:
  Total Requests: 10,245
  Total Errors: 123 (1.2%)
  Avg Response Time: 289ms
  Throughput: 11.4 req/s

Response Time Distribution:
  Min: 45ms
  P50: 234ms
  P95: 567ms
  P99: 892ms
  Max: 1234ms

Top 5 Slowest Tools:
  1. calculate_dcf (avg: 1123ms, count: 245)
  2. get_financial_ratios (avg: 567ms, count: 1234)
  3. calculate_rsi (avg: 234ms, count: 3456)
  4. tools_list (avg: 123ms, count: 2345)
  5. health_check (avg: 45ms, count: 2965)

Errors by Type:
  Timeout: 78 (63.4%)
  ServerError: 32 (26.0%)
  CommunicationError: 13 (10.6%)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### JSON形式

プログラムで処理しやすい形式です。

```bash
mcp_inspector_mcp monitor report --format json --output metrics.json
```

**出力例 (metrics.json)**:
```json
{
  "generated_at": "2025-11-20T10:30:45Z",
  "duration_seconds": 900,
  "summary": {
    "total_requests": 10245,
    "total_errors": 123,
    "error_rate": 0.012,
    "avg_response_time_ms": 289,
    "throughput_rps": 11.4
  },
  "response_time": {
    "min_ms": 45,
    "p50_ms": 234,
    "p95_ms": 567,
    "p99_ms": 892,
    "max_ms": 1234
  },
  "slowest_tools": [
    {
      "tool": "calculate_dcf",
      "avg_response_time_ms": 1123,
      "count": 245
    }
  ],
  "errors_by_type": {
    "Timeout": 78,
    "ServerError": 32,
    "CommunicationError": 13
  }
}
```

### HTML形式

ブラウザでの可視化用です。グラフ付きのインタラクティブなレポートが生成されます。

```bash
mcp_inspector_mcp monitor report --format html --output performance.html
```

**HTMLレポートの特徴**:
- **レスポンシブデザイン**: モバイルでも見やすい
- **インタラクティブグラフ**: Chart.js による可視化
- **ダークモード対応**: 目に優しい表示
- **XSS対策**: 安全なHTMLレンダリング

**含まれる情報**:
- サマリーダッシュボード
- レスポンスタイムヒストグラム
- 時系列グラフ（レスポンスタイム、スループット、エラー率）
- ツール別統計
- エラー分析
- ボトルネック一覧

---

## ボトルネック検出

パフォーマンスモニタリングは、以下の問題を自動検出します。

### 検出されるボトルネック

#### 1. 遅いツール

平均レスポンスタイムが閾値を超えるツールを検出します。

**検出条件**:
- 平均レスポンスタイム > 1000ms

**例**:
```
⚠️ Slow Tool Detected
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Tool: calculate_dcf
Avg Response Time: 1123ms (threshold: 1000ms)
Recommendation: Optimize calculation or increase timeout
```

#### 2. エラー率の急上昇

エラー率が急激に上昇したことを検出します。

**検出条件**:
- エラー率 > 5%

**例**:
```
❌ High Error Rate Detected
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: fundamental_analysis
Error Rate: 8.5% (threshold: 5%)
Primary Error Type: Timeout (78 occurrences)
Recommendation: Check server load and network connectivity
```

#### 3. 高い同時実行数

同時実行リクエスト数が上限に近づいたことを検出します。

**検出条件**:
- 同時実行数 > 80% of max

**例**:
```
⚠️ High Concurrency Detected
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Current Concurrent Requests: 85
Max Concurrent Requests: 100
Utilization: 85%
Recommendation: Increase connection pool size or rate limit requests
```

### ボトルネック分析の実行

```bash
# ボトルネック検出を実行
mcp_inspector_mcp monitor analyze --detect-bottlenecks
```

**出力例**:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Bottleneck Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️ 2 bottlenecks detected

1. Slow Tool: calculate_dcf
   - Avg Response Time: 1123ms
   - Recommendation: Optimize calculation algorithm

2. High Error Rate: fundamental_analysis
   - Error Rate: 8.5%
   - Recommendation: Check server logs for errors

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 実践的な使用例

### 例1: 開発中のパフォーマンス検証

新機能を開発中、パフォーマンスが許容範囲内か検証します。

```bash
# 1. メトリクス収集開始
mcp_inspector_mcp monitor start

# 2. 新機能を実行（Claude Desktopから）
# ...

# 3. レポート生成
mcp_inspector_mcp monitor report

# 4. ボトルネック検出
mcp_inspector_mcp monitor analyze --detect-bottlenecks
```

**判定基準**:
- 平均レスポンスタイム < 500ms
- P95レスポンスタイム < 1000ms
- エラー率 < 5%

### 例2: 負荷テスト結果の分析

負荷テストを実行し、結果を分析します。

```bash
# 1. メトリクス収集開始
mcp_inspector_mcp monitor start

# 2. 負荷テスト実行（別ターミナル）
for i in {1..1000}; do
  # ツール実行
done

# 3. HTMLレポート生成
mcp_inspector_mcp monitor report --format html --output load_test_results.html

# 4. ブラウザでレポート確認
open load_test_results.html  # macOS
start load_test_results.html  # Windows
```

**分析ポイント**:
- レスポンスタイムの推移
- スループットの飽和点
- エラー発生のタイミング
- 同時実行数のピーク

### 例3: 本番環境の継続監視

本番環境で継続的にパフォーマンスを監視します。

```bash
# 1. バックグラウンドでメトリクス収集
nohup mcp_inspector_mcp monitor start > /dev/null 2>&1 &

# 2. 定期的にレポート生成（cronで設定）
# crontab -e
# */15 * * * * /path/to/mcp_inspector_mcp monitor report --format json --output /var/log/mcp/metrics_$(date +\%Y\%m\%d_\%H\%M).json

# 3. アラート設定（閾値超過時に通知）
mcp_inspector_mcp monitor analyze --detect-bottlenecks --alert-email admin@example.com
```

### 例4: パフォーマンス改善の効果測定

パフォーマンス最適化の効果を測定します。

```bash
# Before: 最適化前のパフォーマンス測定
mcp_inspector_mcp monitor reset
mcp_inspector_mcp monitor start
# テスト実行...
mcp_inspector_mcp monitor report --format json --output before.json

# After: 最適化後のパフォーマンス測定
mcp_inspector_mcp monitor reset
mcp_inspector_mcp monitor start
# テスト実行...
mcp_inspector_mcp monitor report --format json --output after.json

# 比較分析
diff before.json after.json
```

---

## ベストプラクティス

### 1. 適切な収集期間

- **短期（1-5分）**: リアルタイム監視、問題発見
- **中期（15-60分）**: トレンド分析、負荷テスト
- **長期（数時間）**: 本番環境監視、容量計画

### 2. メトリクスのリセット

テスト前にメトリクスをリセットし、正確な測定を行います。

```bash
mcp_inspector_mcp monitor reset
```

### 3. レポートの保存

定期的にレポートを保存し、履歴として残します。

```bash
# タイムスタンプ付きでレポート保存
mcp_inspector_mcp monitor report --format json \
  --output "metrics_$(date +%Y%m%d_%H%M%S).json"
```

### 4. ボトルネック対策

検出されたボトルネックに対して、適切な対策を講じます。

| ボトルネック | 対策 |
|------------|------|
| 遅いツール | アルゴリズム最適化、キャッシング、タイムアウト延長 |
| 高エラー率 | ログ調査、サーバー再起動、設定見直し |
| 高同時実行 | 接続プール拡大、レートリミット導入 |

### 5. アラート設定

重要なメトリクスにアラートを設定します。

```bash
# 閾値設定例
export MCP_ALERT_RESPONSE_TIME_MS=1000
export MCP_ALERT_ERROR_RATE=0.05
export MCP_ALERT_CONCURRENT_REQUESTS=80

mcp_inspector_mcp monitor analyze --detect-bottlenecks
```

---

## トラブルシューティング

### メトリクスが収集されない

**症状**: `monitor start`を実行してもメトリクスが記録されない

**原因と対策**:

1. **メトリクス収集が有効か確認**:
   ```bash
   mcp_inspector_mcp monitor status
   ```

2. **権限の確認**:
   ```bash
   # メトリクス保存先の権限確認
   ls -la /var/log/mcp/
   ```

3. **ログの確認**:
   ```bash
   export RUST_LOG=debug
   mcp_inspector_mcp monitor start
   ```

### HTMLレポートが開けない

**症状**: HTMLレポートをブラウザで開くとエラーになる

**原因**: ファイルパスや権限の問題

**対策**:

```bash
# ファイルが存在するか確認
ls -la performance.html

# 絶対パスで開く
open "$(pwd)/performance.html"  # macOS
start "$(pwd)/performance.html"  # Windows
```

### メモリ使用量が多い

**症状**: メトリクス収集中にメモリ使用量が増加する

**対策**:

```bash
# メトリクスの保存期間を短縮
export MCP_METRICS_RETENTION_MINUTES=15

# 定期的にメトリクスをリセット
mcp_inspector_mcp monitor reset
```

---

## 参考リンク

- [README.md](../../README.md): 全体的な使い方
- [デバッグモードガイド](./debug-mode.md): デバッグ方法
- [バッチテストガイド](./batch-testing.md): テスト自動化
- [構成管理ガイド](./configuration-management.md): 設定管理

---

**最終更新**: 2025-11-20
**対象バージョン**: v0.4.0+

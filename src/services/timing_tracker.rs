use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// タイミング情報
#[derive(Debug, Clone)]
pub struct TimingInfo {
    /// 開始時刻
    pub start_time: DateTime<Local>,
    /// 開始時点（経過時間計測用）
    pub start_instant: Instant,
    /// 終了時刻（オプション）
    pub end_time: Option<DateTime<Local>>,
    /// 経過時間（ミリ秒、オプション）
    pub elapsed_ms: Option<u128>,
}

/// タイミングトラッカー
///
/// リクエスト/レスポンスのタイミング情報を記録・管理します。
pub struct TimingTracker {
    timings: Arc<Mutex<HashMap<String, TimingInfo>>>,
}

impl TimingTracker {
    /// 新しいタイミングトラッカーを作成
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    ///
    /// let tracker = TimingTracker::new();
    /// ```
    pub fn new() -> Self {
        Self {
            timings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// タイマーを開始
    ///
    /// # Arguments
    /// * `request_id` - リクエストID
    ///
    /// # Returns
    /// 開始時刻
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    ///
    /// let tracker = TimingTracker::new();
    /// let start_time = tracker.start_timer("req-123");
    /// ```
    pub fn start_timer(&self, request_id: &str) -> DateTime<Local> {
        let start_time = Local::now();
        let start_instant = Instant::now();

        let timing_info = TimingInfo {
            start_time,
            start_instant,
            end_time: None,
            elapsed_ms: None,
        };

        let mut timings = self.timings.lock().unwrap();
        timings.insert(request_id.to_string(), timing_info);

        start_time
    }

    /// タイマーを停止して経過時間を計算
    ///
    /// # Arguments
    /// * `request_id` - リクエストID
    ///
    /// # Returns
    /// 経過時間（ミリ秒）と終了時刻のタプル
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let tracker = TimingTracker::new();
    /// tracker.start_timer("req-123");
    /// thread::sleep(Duration::from_millis(10));
    /// let (elapsed_ms, end_time) = tracker.stop_timer("req-123").unwrap();
    /// assert!(elapsed_ms >= 10);
    /// ```
    pub fn stop_timer(&self, request_id: &str) -> Option<(u128, DateTime<Local>)> {
        let end_time = Local::now();
        let mut timings = self.timings.lock().unwrap();

        if let Some(timing_info) = timings.get_mut(request_id) {
            let elapsed = timing_info.start_instant.elapsed();
            let elapsed_ms = elapsed.as_millis();

            timing_info.end_time = Some(end_time);
            timing_info.elapsed_ms = Some(elapsed_ms);

            Some((elapsed_ms, end_time))
        } else {
            None
        }
    }

    /// 経過時間を取得（タイマーを停止せずに）
    ///
    /// # Arguments
    /// * `request_id` - リクエストID
    ///
    /// # Returns
    /// 現時点での経過時間（ミリ秒）
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let tracker = TimingTracker::new();
    /// tracker.start_timer("req-123");
    /// thread::sleep(Duration::from_millis(10));
    /// let elapsed_ms = tracker.get_elapsed("req-123").unwrap();
    /// assert!(elapsed_ms >= 10);
    /// ```
    pub fn get_elapsed(&self, request_id: &str) -> Option<u128> {
        let timings = self.timings.lock().unwrap();

        timings.get(request_id).map(|timing_info| {
            if let Some(elapsed_ms) = timing_info.elapsed_ms {
                // 既に停止している場合は記録された経過時間を返す
                elapsed_ms
            } else {
                // まだ停止していない場合は現在の経過時間を計算
                timing_info.start_instant.elapsed().as_millis()
            }
        })
    }

    /// タイミング情報を取得
    ///
    /// # Arguments
    /// * `request_id` - リクエストID
    ///
    /// # Returns
    /// タイミング情報（存在する場合）
    pub fn get_timing_info(&self, request_id: &str) -> Option<TimingInfo> {
        let timings = self.timings.lock().unwrap();
        timings.get(request_id).cloned()
    }

    /// タイミング情報をクリア
    ///
    /// # Arguments
    /// * `request_id` - リクエストID
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    ///
    /// let tracker = TimingTracker::new();
    /// tracker.start_timer("req-123");
    /// tracker.clear("req-123");
    /// assert!(tracker.get_elapsed("req-123").is_none());
    /// ```
    pub fn clear(&self, request_id: &str) {
        let mut timings = self.timings.lock().unwrap();
        timings.remove(request_id);
    }

    /// 全てのタイミング情報をクリア
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    ///
    /// let tracker = TimingTracker::new();
    /// tracker.start_timer("req-1");
    /// tracker.start_timer("req-2");
    /// tracker.clear_all();
    /// assert!(tracker.get_elapsed("req-1").is_none());
    /// assert!(tracker.get_elapsed("req-2").is_none());
    /// ```
    pub fn clear_all(&self) {
        let mut timings = self.timings.lock().unwrap();
        timings.clear();
    }

    /// タイムスタンプを整形
    ///
    /// # Arguments
    /// * `timestamp` - タイムスタンプ
    ///
    /// # Returns
    /// 整形された文字列（YYYY-MM-DD HH:MM:SS.mmm）
    ///
    /// # Example
    /// ```
    /// use mcp_inspector_mcp::services::timing_tracker::TimingTracker;
    /// use chrono::Local;
    ///
    /// let tracker = TimingTracker::new();
    /// let timestamp = Local::now();
    /// let formatted = TimingTracker::format_timestamp(timestamp);
    /// assert!(formatted.contains('-'));
    /// assert!(formatted.contains(':'));
    /// ```
    pub fn format_timestamp(timestamp: DateTime<Local>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }
}

impl Default for TimingTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timing_tracker_creation() {
        let tracker = TimingTracker::new();
        assert!(tracker.get_elapsed("non-existent").is_none());
    }

    #[test]
    fn test_start_timer() {
        let tracker = TimingTracker::new();
        let start_time = tracker.start_timer("req-123");

        assert!(tracker.get_elapsed("req-123").is_some());

        let timing_info = tracker.get_timing_info("req-123").unwrap();
        assert_eq!(timing_info.start_time, start_time);
        assert!(timing_info.end_time.is_none());
        assert!(timing_info.elapsed_ms.is_none());
    }

    #[test]
    fn test_stop_timer() {
        let tracker = TimingTracker::new();
        tracker.start_timer("req-123");

        thread::sleep(Duration::from_millis(10));

        let result = tracker.stop_timer("req-123");
        assert!(result.is_some());

        let (elapsed_ms, _end_time) = result.unwrap();
        assert!(elapsed_ms >= 10);

        let timing_info = tracker.get_timing_info("req-123").unwrap();
        assert!(timing_info.end_time.is_some());
        assert_eq!(timing_info.elapsed_ms, Some(elapsed_ms));
    }

    #[test]
    fn test_stop_nonexistent_timer() {
        let tracker = TimingTracker::new();
        let result = tracker.stop_timer("non-existent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_elapsed_running() {
        let tracker = TimingTracker::new();
        tracker.start_timer("req-123");

        thread::sleep(Duration::from_millis(10));

        let elapsed_ms = tracker.get_elapsed("req-123").unwrap();
        assert!(elapsed_ms >= 10);
    }

    #[test]
    fn test_get_elapsed_stopped() {
        let tracker = TimingTracker::new();
        tracker.start_timer("req-123");

        thread::sleep(Duration::from_millis(10));

        let (stopped_elapsed, _) = tracker.stop_timer("req-123").unwrap();

        // 停止後に取得した経過時間は、停止時の経過時間と一致する
        let elapsed_ms = tracker.get_elapsed("req-123").unwrap();
        assert_eq!(elapsed_ms, stopped_elapsed);
    }

    #[test]
    fn test_clear() {
        let tracker = TimingTracker::new();
        tracker.start_timer("req-123");

        assert!(tracker.get_elapsed("req-123").is_some());

        tracker.clear("req-123");
        assert!(tracker.get_elapsed("req-123").is_none());
    }

    #[test]
    fn test_clear_all() {
        let tracker = TimingTracker::new();
        tracker.start_timer("req-1");
        tracker.start_timer("req-2");
        tracker.start_timer("req-3");

        assert!(tracker.get_elapsed("req-1").is_some());
        assert!(tracker.get_elapsed("req-2").is_some());
        assert!(tracker.get_elapsed("req-3").is_some());

        tracker.clear_all();

        assert!(tracker.get_elapsed("req-1").is_none());
        assert!(tracker.get_elapsed("req-2").is_none());
        assert!(tracker.get_elapsed("req-3").is_none());
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = Local::now();
        let formatted = TimingTracker::format_timestamp(timestamp);

        // フォーマットが正しいかチェック
        assert!(formatted.contains('-'));
        assert!(formatted.contains(':'));
        assert!(formatted.contains('.'));

        // 長さをチェック（YYYY-MM-DD HH:MM:SS.mmm は23文字）
        assert!(formatted.len() >= 23);
    }
}

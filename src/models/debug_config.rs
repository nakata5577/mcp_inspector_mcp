use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// ログレベルの列挙型
///
/// アプリケーション全体で使用されるログレベルを定義します。
/// tracingクレートのログレベルに対応しています。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// TRACE: 最も詳細なログレベル
    Trace,
    /// DEBUG: デバッグ情報のログレベル
    Debug,
    /// INFO: 通常の情報のログレベル
    Info,
    /// WARN: 警告のログレベル
    Warn,
    /// ERROR: エラーのログレベル
    Error,
}

impl LogLevel {
    /// LogLevelをtracingのLevelに変換
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// ログファイルのローテーションポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPolicy {
    /// 日次ローテーション
    Daily,
    /// 時間毎のローテーション
    Hourly,
    /// ローテーションなし
    Never,
}

impl RotationPolicy {
    /// RotationPolicyをtracing_appenderのRotationに変換
    pub fn to_tracing_rotation(&self) -> tracing_appender::rolling::Rotation {
        match self {
            RotationPolicy::Daily => tracing_appender::rolling::Rotation::DAILY,
            RotationPolicy::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            RotationPolicy::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}

/// ログ設定
///
/// アプリケーション全体のログ設定を管理します。
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// ログレベル
    pub level: LogLevel,
    /// ログファイルへの出力を有効化
    pub output_to_file: bool,
    /// ログファイルのパス
    pub log_file_path: PathBuf,
    /// ログファイルのローテーションポリシー
    pub log_file_rotation: RotationPolicy,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            output_to_file: false,
            log_file_path: PathBuf::from("logs"),
            log_file_rotation: RotationPolicy::Daily,
        }
    }
}

/// グローバルなログ設定
static LOG_CONFIG: Lazy<Mutex<LogConfig>> = Lazy::new(|| Mutex::new(LogConfig::default()));

/// グローバルなデバッグモードフラグ
///
/// アプリケーション全体でデバッグモードの状態を管理します。
/// `once_cell::sync::Lazy`を使用してスレッドセーフな初期化を保証します。
pub static VERBOSE_MODE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// デバッグモードを有効化
///
/// # Example
/// ```
/// use mcp_inspector_mcp::models::debug_config;
///
/// debug_config::enable_verbose_mode();
/// assert!(debug_config::is_verbose_mode());
/// ```
pub fn enable_verbose_mode() {
    VERBOSE_MODE.store(true, Ordering::SeqCst);
}

/// デバッグモードを無効化
///
/// # Example
/// ```
/// use mcp_inspector_mcp::models::debug_config;
///
/// debug_config::disable_verbose_mode();
/// assert!(!debug_config::is_verbose_mode());
/// ```
pub fn disable_verbose_mode() {
    VERBOSE_MODE.store(false, Ordering::SeqCst);
}

/// デバッグモードが有効かどうかをチェック
///
/// # Returns
/// デバッグモードが有効な場合は`true`、そうでない場合は`false`
///
/// # Example
/// ```
/// use mcp_inspector_mcp::models::debug_config;
///
/// if debug_config::is_verbose_mode() {
///     println!("Debug mode is enabled");
/// }
/// ```
pub fn is_verbose_mode() -> bool {
    VERBOSE_MODE.load(Ordering::SeqCst)
}

/// ログ設定を取得
///
/// # Returns
/// 現在のログ設定のクローン
///
/// # Example
/// ```
/// use mcp_inspector_mcp::models::debug_config;
///
/// let config = debug_config::get_log_config();
/// println!("Current log level: {:?}", config.level);
/// ```
pub fn get_log_config() -> LogConfig {
    LOG_CONFIG.lock().unwrap().clone()
}

/// ログ設定を更新
///
/// # Arguments
/// * `config` - 新しいログ設定
///
/// # Example
/// ```
/// use mcp_inspector_mcp::models::debug_config::{self, LogConfig, LogLevel, RotationPolicy};
/// use std::path::PathBuf;
///
/// let new_config = LogConfig {
///     level: LogLevel::Debug,
///     output_to_file: true,
///     log_file_path: PathBuf::from("logs"),
///     log_file_rotation: RotationPolicy::Daily,
/// };
/// debug_config::set_log_config(new_config);
/// ```
pub fn set_log_config(config: LogConfig) {
    *LOG_CONFIG.lock().unwrap() = config;
}

/// Verboseモード時のログレベルを設定
///
/// この関数は、verboseモードが有効な場合にログレベルをDEBUGに設定します。
///
/// # Arguments
/// * `enable_file_output` - ログファイル出力を有効化するかどうか
pub fn configure_verbose_logging(enable_file_output: bool) {
    let mut config = LOG_CONFIG.lock().unwrap();
    config.level = LogLevel::Debug;
    config.output_to_file = enable_file_output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbose_mode_toggle() {
        // 初期状態はfalse
        disable_verbose_mode();
        assert!(!is_verbose_mode());

        // 有効化
        enable_verbose_mode();
        assert!(is_verbose_mode());

        // 無効化
        disable_verbose_mode();
        assert!(!is_verbose_mode());
    }

    #[test]
    fn test_verbose_mode_idempotent() {
        // 複数回有効化しても問題ない
        enable_verbose_mode();
        enable_verbose_mode();
        assert!(is_verbose_mode());

        // 複数回無効化しても問題ない
        disable_verbose_mode();
        disable_verbose_mode();
        assert!(!is_verbose_mode());
    }

    #[test]
    fn test_log_level_to_tracing() {
        assert_eq!(LogLevel::Trace.to_tracing_level(), tracing::Level::TRACE);
        assert_eq!(LogLevel::Debug.to_tracing_level(), tracing::Level::DEBUG);
        assert_eq!(LogLevel::Info.to_tracing_level(), tracing::Level::INFO);
        assert_eq!(LogLevel::Warn.to_tracing_level(), tracing::Level::WARN);
        assert_eq!(LogLevel::Error.to_tracing_level(), tracing::Level::ERROR);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("trace".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("Info".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("warn".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("error".parse::<LogLevel>(), Ok(LogLevel::Error));
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_rotation_policy_to_tracing() {
        assert_eq!(
            RotationPolicy::Daily.to_tracing_rotation(),
            tracing_appender::rolling::Rotation::DAILY
        );
        assert_eq!(
            RotationPolicy::Hourly.to_tracing_rotation(),
            tracing_appender::rolling::Rotation::HOURLY
        );
        assert_eq!(
            RotationPolicy::Never.to_tracing_rotation(),
            tracing_appender::rolling::Rotation::NEVER
        );
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert!(!config.output_to_file);
        assert_eq!(config.log_file_path, PathBuf::from("logs"));
        assert_eq!(config.log_file_rotation, RotationPolicy::Daily);
    }

    #[test]
    fn test_get_set_log_config() {
        let new_config = LogConfig {
            level: LogLevel::Debug,
            output_to_file: true,
            log_file_path: PathBuf::from("test_logs"),
            log_file_rotation: RotationPolicy::Hourly,
        };

        set_log_config(new_config.clone());
        let retrieved = get_log_config();

        assert_eq!(retrieved.level, LogLevel::Debug);
        assert!(retrieved.output_to_file);
        assert_eq!(retrieved.log_file_path, PathBuf::from("test_logs"));
        assert_eq!(retrieved.log_file_rotation, RotationPolicy::Hourly);
    }

    #[test]
    fn test_configure_verbose_logging() {
        configure_verbose_logging(true);
        let config = get_log_config();
        assert_eq!(config.level, LogLevel::Debug);
        assert!(config.output_to_file);

        configure_verbose_logging(false);
        let config = get_log_config();
        assert_eq!(config.level, LogLevel::Debug);
        assert!(!config.output_to_file);
    }

    #[test]
    fn test_log_config_clone() {
        let config1 = LogConfig {
            level: LogLevel::Warn,
            output_to_file: true,
            log_file_path: PathBuf::from("test"),
            log_file_rotation: RotationPolicy::Never,
        };

        let config2 = config1.clone();
        assert_eq!(config1.level, config2.level);
        assert_eq!(config1.output_to_file, config2.output_to_file);
        assert_eq!(config1.log_file_path, config2.log_file_path);
        assert_eq!(config1.log_file_rotation, config2.log_file_rotation);
    }
}

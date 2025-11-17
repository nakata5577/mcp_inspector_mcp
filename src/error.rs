use serde::{Deserialize, Serialize};
use std::fmt;

/// ツール実行エラーの詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolExecutionError {
    /// タイムアウトエラー
    Timeout {
        tool_name: String,
        elapsed_ms: u64,
        configured_timeout_ms: u64,
        server_alive: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },

    /// サーバープロセスのクラッシュ
    ServerCrash {
        tool_name: String,
        exit_code: Option<i32>,
        stderr: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_log: Option<String>,
    },

    /// 不正なレスポンス
    InvalidResponse {
        tool_name: String,
        received: String,
        expected_format: String,
        parse_error: String,
    },

    /// ネットワーク/通信エラー
    CommunicationError {
        tool_name: String,
        details: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestion: Option<String>,
    },

    /// サーバー側のエラー
    ServerError {
        tool_name: String,
        error_message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_code: Option<i32>,
    },

    /// その他のエラー
    Other {
        tool_name: String,
        message: String,
    },
}

impl ToolExecutionError {
    /// ユーザーフレンドリーなエラーメッセージを生成
    pub fn user_message(&self) -> String {
        match self {
            Self::Timeout {
                tool_name,
                elapsed_ms,
                configured_timeout_ms,
                server_alive,
                suggestion,
            } => {
                let mut msg = format!(
                    "Tool '{}' timed out after {}ms (configured timeout: {}ms)",
                    tool_name, elapsed_ms, configured_timeout_ms
                );

                if !server_alive {
                    msg.push_str("\nServer process has terminated unexpectedly.");
                }

                if let Some(sug) = suggestion {
                    msg.push_str(&format!("\nSuggestion: {}", sug));
                }

                msg
            }

            Self::ServerCrash {
                tool_name,
                exit_code,
                stderr,
                ..
            } => {
                format!(
                    "Server crashed while executing tool '{}'\nExit code: {:?}\nError output: {}",
                    tool_name,
                    exit_code,
                    if stderr.is_empty() { "(none)" } else { stderr }
                )
            }

            Self::InvalidResponse {
                tool_name,
                parse_error,
                ..
            } => {
                format!(
                    "Tool '{}' returned invalid response: {}",
                    tool_name, parse_error
                )
            }

            Self::CommunicationError {
                tool_name,
                details,
                suggestion,
            } => {
                let mut msg = format!(
                    "Communication error while calling tool '{}': {}",
                    tool_name, details
                );

                if let Some(sug) = suggestion {
                    msg.push_str(&format!("\nSuggestion: {}", sug));
                }

                msg
            }

            Self::ServerError {
                tool_name,
                error_message,
                ..
            } => {
                format!(
                    "Server error in tool '{}': {}",
                    tool_name, error_message
                )
            }

            Self::Other { tool_name, message } => {
                format!("Error in tool '{}': {}", tool_name, message)
            }
        }
    }

    /// JSON形式でのシリアライズ
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            serde_json::json!({
                "type": "serialization_error",
                "message": "Failed to serialize error"
            })
        })
    }
}

impl fmt::Display for ToolExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ToolExecutionError {}

/// エラーレスポンスの構造
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ToolExecutionError,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: ToolExecutionError) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }
}

use rmcp::model::ServerCapabilities;
use tracing::{debug, warn};

/// Capability検証結果
#[derive(Debug, Clone)]
pub enum CapabilityValidationResult {
    /// 検証成功
    Valid,
    /// 警告あり（実行は継続）
    Warning { message: String },
    /// Capability情報なし（検証スキップ）
    Unavailable,
}

/// Capability検証器
///
/// MCPサーバーのcapabilities（サポート機能）を検証し、
/// クライアントのリクエストとの矛盾を検出して警告を生成します。
pub struct CapabilityValidator {
    capabilities: Option<ServerCapabilities>,
}

impl CapabilityValidator {
    /// 新規作成
    ///
    /// # Arguments
    /// * `capabilities` - サーバーのcapabilities情報（Noneの場合は検証スキップ）
    pub fn new(capabilities: Option<ServerCapabilities>) -> Self {
        Self { capabilities }
    }

    /// ツール実行前の検証
    ///
    /// サーバーがtools capabilityをサポートしているかを確認します。
    /// サポートしていない場合は警告を生成しますが、実行は継続します（ベストエフォート）。
    ///
    /// # Arguments
    /// * `tool_name` - 実行しようとしているツール名
    ///
    /// # Returns
    /// 検証結果（Valid, Warning, またはUnavailable）
    pub fn validate_tools_call(&self, tool_name: &str) -> CapabilityValidationResult {
        match &self.capabilities {
            None => {
                debug!("No capability information available, skipping tools validation");
                CapabilityValidationResult::Unavailable
            }
            Some(caps) => {
                if let Some(_tools_cap) = &caps.tools {
                    // tools capabilityが存在する場合はサポートされている
                    CapabilityValidationResult::Valid
                } else {
                    let msg = format!(
                        "Warning: Server does not declare tools capability, but attempting to call tool '{}'",
                        tool_name
                    );
                    warn!("{}", msg);
                    CapabilityValidationResult::Warning { message: msg }
                }
            }
        }
    }

    /// リソース取得前の検証
    ///
    /// サーバーがresources capabilityをサポートしているかを確認します。
    /// サポートしていない場合は警告を生成しますが、実行は継続します（ベストエフォート）。
    ///
    /// # Arguments
    /// * `uri` - 取得しようとしているリソースURI
    ///
    /// # Returns
    /// 検証結果（Valid, Warning, またはUnavailable）
    pub fn validate_resources_read(&self, uri: &str) -> CapabilityValidationResult {
        match &self.capabilities {
            None => {
                debug!("No capability information available, skipping resources validation");
                CapabilityValidationResult::Unavailable
            }
            Some(caps) => {
                if let Some(_resources_cap) = &caps.resources {
                    // resources capabilityが存在する場合はサポートされている
                    CapabilityValidationResult::Valid
                } else {
                    let msg = format!(
                        "Warning: Server does not declare resources capability, but attempting to read resource '{}'",
                        uri
                    );
                    warn!("{}", msg);
                    CapabilityValidationResult::Warning { message: msg }
                }
            }
        }
    }

    /// プロンプト取得前の検証
    ///
    /// サーバーがprompts capabilityをサポートしているかを確認します。
    /// サポートしていない場合は警告を生成しますが、実行は継続します（ベストエフォート）。
    ///
    /// # Arguments
    /// * `prompt_name` - 取得しようとしているプロンプト名
    ///
    /// # Returns
    /// 検証結果（Valid, Warning, またはUnavailable）
    pub fn validate_prompts_get(&self, prompt_name: &str) -> CapabilityValidationResult {
        match &self.capabilities {
            None => {
                debug!("No capability information available, skipping prompts validation");
                CapabilityValidationResult::Unavailable
            }
            Some(caps) => {
                if let Some(_prompts_cap) = &caps.prompts {
                    // prompts capabilityが存在する場合はサポートされている
                    CapabilityValidationResult::Valid
                } else {
                    let msg = format!(
                        "Warning: Server does not declare prompts capability, but attempting to get prompt '{}'",
                        prompt_name
                    );
                    warn!("{}", msg);
                    CapabilityValidationResult::Warning { message: msg }
                }
            }
        }
    }

    /// Capability情報のログ出力
    ///
    /// デバッグ目的で、サーバーのcapability情報をログに出力します。
    ///
    /// # Arguments
    /// * `server_name` - サーバー名（ログメッセージに含める）
    pub fn log_capabilities(&self, server_name: &str) {
        match &self.capabilities {
            None => {
                debug!("Server '{}': No capability information available", server_name);
            }
            Some(caps) => {
                debug!("Server '{}' capabilities:", server_name);
                if let Some(tools) = &caps.tools {
                    debug!("  - Tools: declared (list_changed: {:?})", tools.list_changed);
                } else {
                    debug!("  - Tools: not declared");
                }
                if let Some(resources) = &caps.resources {
                    debug!(
                        "  - Resources: declared (subscribe: {:?}, list_changed: {:?})",
                        resources.subscribe, resources.list_changed
                    );
                } else {
                    debug!("  - Resources: not declared");
                }
                if let Some(prompts) = &caps.prompts {
                    debug!("  - Prompts: declared (list_changed: {:?})", prompts.list_changed);
                } else {
                    debug!("  - Prompts: not declared");
                }
                if let Some(logging) = &caps.logging {
                    debug!("  - Logging: declared ({:?})", logging);
                } else {
                    debug!("  - Logging: not declared");
                }
                if let Some(experimental) = &caps.experimental {
                    debug!("  - Experimental: declared ({:?})", experimental);
                } else {
                    debug!("  - Experimental: not declared");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のServerCapabilitiesを生成するヘルパー関数
    fn create_capabilities_with_tools() -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(rmcp::model::ToolsCapability {
                list_changed: Some(false),
            }),
            resources: None,
            prompts: None,
            logging: None,
            experimental: None,
            completions: None,
        }
    }

    fn create_capabilities_without_tools() -> ServerCapabilities {
        ServerCapabilities {
            tools: None,
            resources: None,
            prompts: None,
            logging: None,
            experimental: None,
            completions: None,
        }
    }

    #[test]
    fn test_validate_tools_call_with_capability() {
        let validator = CapabilityValidator::new(Some(create_capabilities_with_tools()));
        let result = validator.validate_tools_call("test_tool");

        match result {
            CapabilityValidationResult::Valid => {
                // 期待通り
            }
            _ => panic!("Expected Valid result"),
        }
    }

    #[test]
    fn test_validate_tools_call_without_capability() {
        let validator = CapabilityValidator::new(Some(create_capabilities_without_tools()));
        let result = validator.validate_tools_call("test_tool");

        match result {
            CapabilityValidationResult::Warning { message } => {
                assert!(message.contains("test_tool"));
                assert!(message.contains("does not declare tools capability"));
            }
            _ => panic!("Expected Warning result"),
        }
    }

    #[test]
    fn test_validate_tools_call_unavailable() {
        let validator = CapabilityValidator::new(None);
        let result = validator.validate_tools_call("test_tool");

        match result {
            CapabilityValidationResult::Unavailable => {
                // 期待通り
            }
            _ => panic!("Expected Unavailable result"),
        }
    }
}

pub mod error;
pub mod request;
pub mod response;
pub mod server_config;

pub use error::{InspectorError, Result};
pub use request::{
    PromptGetRequest, PromptsListRequest, ResourceReadRequest, ResourcesListRequest,
    ToolCallRequest, ToolsListRequest,
};
pub use response::{
    PromptArgument, PromptGetResponse, PromptInfo, PromptMessage, PromptsListResponse,
    ResourceContent, ResourceInfo, ResourceReadResponse, ResourcesListResponse, ToolCallResponse,
    ToolInfo, ToolsListResponse,
};
pub use server_config::{ConnectionParams, InspectorConfig, ServerConfig, TransportType};

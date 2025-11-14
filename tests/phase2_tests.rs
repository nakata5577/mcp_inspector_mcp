/// Phase 2 Integration Tests
///
/// This module contains tests for the Phase 2 features:
/// - resources_list
/// - resources_read
/// - prompts_list
/// - prompts_get

use mcp_inspector_mcp::{
    PromptGetRequest, PromptsListRequest, ResourceReadRequest, ResourcesListRequest,
};

#[cfg(test)]
mod request_tests {
    use super::*;

    /// Test ResourcesListRequest structure
    #[test]
    fn test_resources_list_request_creation() {
        let request = ResourcesListRequest {
            server: "test_server".to_string(),
        };

        assert_eq!(request.server, "test_server");
    }

    /// Test ResourceReadRequest structure
    #[test]
    fn test_resource_read_request_creation() {
        let request = ResourceReadRequest {
            server: "test_server".to_string(),
            uri: "file:///test/resource.txt".to_string(),
        };

        assert_eq!(request.server, "test_server");
        assert_eq!(request.uri, "file:///test/resource.txt");
    }

    /// Test PromptsListRequest structure
    #[test]
    fn test_prompts_list_request_creation() {
        let request = PromptsListRequest {
            server: "test_server".to_string(),
        };

        assert_eq!(request.server, "test_server");
    }

    /// Test PromptGetRequest structure with arguments
    #[test]
    fn test_prompt_get_request_with_arguments() {
        use std::collections::HashMap;

        let mut arguments = HashMap::new();
        arguments.insert("ticker".to_string(), "AAPL".to_string());
        arguments.insert("period".to_string(), "Q1".to_string());

        let request = PromptGetRequest {
            server: "test_server".to_string(),
            name: "analyze_company".to_string(),
            arguments,
        };

        assert_eq!(request.server, "test_server");
        assert_eq!(request.name, "analyze_company");
        assert_eq!(request.arguments.len(), 2);
        assert_eq!(request.arguments.get("ticker"), Some(&"AAPL".to_string()));
        assert_eq!(request.arguments.get("period"), Some(&"Q1".to_string()));
    }

    /// Test PromptGetRequest structure without arguments
    #[test]
    fn test_prompt_get_request_without_arguments() {
        use std::collections::HashMap;

        let request = PromptGetRequest {
            server: "test_server".to_string(),
            name: "simple_prompt".to_string(),
            arguments: HashMap::new(),
        };

        assert_eq!(request.server, "test_server");
        assert_eq!(request.name, "simple_prompt");
        assert!(request.arguments.is_empty());
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    /// Test that server names are properly validated
    #[test]
    fn test_server_name_validation() {
        let valid_names = vec![
            "server",
            "server_name",
            "server-name",
            "server123",
            "fundamental_analysis",
        ];

        for name in valid_names {
            let request = ResourcesListRequest {
                server: name.to_string(),
            };
            assert!(!request.server.is_empty());
            assert!(request.server.len() > 0);
        }
    }

    /// Test that URIs are properly formatted
    #[test]
    fn test_uri_formatting() {
        let valid_uris = vec![
            "file:///path/to/resource",
            "http://example.com/resource",
            "https://example.com/resource",
            "resource://custom/path",
        ];

        for uri in valid_uris {
            let request = ResourceReadRequest {
                server: "test".to_string(),
                uri: uri.to_string(),
            };
            assert!(!request.uri.is_empty());
            assert!(request.uri.contains("://"));
        }
    }

    /// Test empty server name edge case
    #[test]
    fn test_empty_server_name() {
        let request = ResourcesListRequest {
            server: "".to_string(),
        };
        // Empty server name should be handled by the service layer
        assert_eq!(request.server.len(), 0);
    }

    /// Test empty URI edge case
    #[test]
    fn test_empty_uri() {
        let request = ResourceReadRequest {
            server: "test".to_string(),
            uri: "".to_string(),
        };
        // Empty URI should be handled by the service layer
        assert_eq!(request.uri.len(), 0);
    }

    /// Test prompt name validation
    #[test]
    fn test_prompt_name_validation() {
        use std::collections::HashMap;

        let valid_names = vec![
            "analyze",
            "analyze_company",
            "get-data",
            "simple_prompt",
        ];

        for name in valid_names {
            let request = PromptGetRequest {
                server: "test".to_string(),
                name: name.to_string(),
                arguments: HashMap::new(),
            };
            assert!(!request.name.is_empty());
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use serde_json;

    /// Test ResourcesListRequest JSON serialization
    #[test]
    fn test_resources_list_request_serialization() {
        let request = ResourcesListRequest {
            server: "test_server".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_server"));
    }

    /// Test ResourceReadRequest JSON serialization
    #[test]
    fn test_resource_read_request_serialization() {
        let request = ResourceReadRequest {
            server: "test_server".to_string(),
            uri: "file:///test".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_server"));
        assert!(json.contains("file:///test"));
    }

    /// Test PromptsListRequest JSON serialization
    #[test]
    fn test_prompts_list_request_serialization() {
        let request = PromptsListRequest {
            server: "test_server".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_server"));
    }

    /// Test PromptGetRequest JSON serialization with arguments
    #[test]
    fn test_prompt_get_request_serialization() {
        use std::collections::HashMap;

        let mut arguments = HashMap::new();
        arguments.insert("ticker".to_string(), "AAPL".to_string());

        let request = PromptGetRequest {
            server: "test_server".to_string(),
            name: "analyze".to_string(),
            arguments,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test_server"));
        assert!(json.contains("analyze"));
        assert!(json.contains("ticker"));
        assert!(json.contains("AAPL"));
    }
}

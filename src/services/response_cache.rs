use crate::models::{PromptInfo, ResourceInfo, ToolInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A cached response with TTL (Time To Live)
#[derive(Clone, Debug)]
struct CachedResponse<T> {
    data: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T> CachedResponse<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            cached_at: Instant::now(),
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

/// Cache for MCP server responses
///
/// This cache stores responses from MCP servers with a TTL-based expiration policy.
/// It supports caching for:
/// - Tool lists
/// - Resource lists
/// - Prompt lists
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use mcp_inspector_mcp::services::ResponseCache;
///
/// let cache = ResponseCache::new(Duration::from_secs(300)); // 5 minutes TTL
///
/// // Set cache
/// cache.set_tools("server1".to_string(), vec![], Duration::from_secs(300)).await;
///
/// // Get cache
/// if let Some(tools) = cache.get_tools("server1").await {
///     println!("Found cached tools: {:?}", tools);
/// }
/// ```
pub struct ResponseCache {
    tools: Arc<RwLock<HashMap<String, CachedResponse<Vec<ToolInfo>>>>>,
    resources: Arc<RwLock<HashMap<String, CachedResponse<Vec<ResourceInfo>>>>>,
    prompts: Arc<RwLock<HashMap<String, CachedResponse<Vec<PromptInfo>>>>>,
    default_ttl: Duration,
}

impl ResponseCache {
    /// Create a new ResponseCache with the default TTL
    ///
    /// # Arguments
    ///
    /// * `default_ttl` - The default time-to-live for cached responses
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Get the default TTL
    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    // ========== Tools Cache ==========

    /// Get cached tools for a server
    ///
    /// Returns `Some(Vec<ToolInfo>)` if a valid cache entry exists,
    /// or `None` if the cache is missing or expired.
    ///
    /// # Arguments
    ///
    /// * `server` - The server name to look up
    pub async fn get_tools(&self, server: &str) -> Option<Vec<ToolInfo>> {
        let cache = self.tools.read().await;

        if let Some(cached) = cache.get(server) {
            if cached.is_valid() {
                tracing::debug!(server = server, "Cache hit for tools");
                return Some(cached.data.clone());
            } else {
                tracing::debug!(server = server, "Cache expired for tools");
            }
        } else {
            tracing::debug!(server = server, "Cache miss for tools");
        }

        None
    }

    /// Set cached tools for a server
    ///
    /// # Arguments
    ///
    /// * `server` - The server name
    /// * `tools` - The tools to cache
    /// * `ttl` - The time-to-live for this cache entry
    pub async fn set_tools(&self, server: String, tools: Vec<ToolInfo>, ttl: Duration) {
        let mut cache = self.tools.write().await;

        cache.insert(server.clone(), CachedResponse::new(tools, ttl));

        tracing::debug!(
            server = server.as_str(),
            ttl_secs = ttl.as_secs(),
            "Cached tools"
        );
    }

    // ========== Resources Cache ==========

    /// Get cached resources for a server
    ///
    /// Returns `Some(Vec<ResourceInfo>)` if a valid cache entry exists,
    /// or `None` if the cache is missing or expired.
    ///
    /// # Arguments
    ///
    /// * `server` - The server name to look up
    pub async fn get_resources(&self, server: &str) -> Option<Vec<ResourceInfo>> {
        let cache = self.resources.read().await;

        if let Some(cached) = cache.get(server) {
            if cached.is_valid() {
                tracing::debug!(server = server, "Cache hit for resources");
                return Some(cached.data.clone());
            } else {
                tracing::debug!(server = server, "Cache expired for resources");
            }
        } else {
            tracing::debug!(server = server, "Cache miss for resources");
        }

        None
    }

    /// Set cached resources for a server
    ///
    /// # Arguments
    ///
    /// * `server` - The server name
    /// * `resources` - The resources to cache
    /// * `ttl` - The time-to-live for this cache entry
    pub async fn set_resources(&self, server: String, resources: Vec<ResourceInfo>, ttl: Duration) {
        let mut cache = self.resources.write().await;

        cache.insert(server.clone(), CachedResponse::new(resources, ttl));

        tracing::debug!(
            server = server.as_str(),
            ttl_secs = ttl.as_secs(),
            "Cached resources"
        );
    }

    // ========== Prompts Cache ==========

    /// Get cached prompts for a server
    ///
    /// Returns `Some(Vec<PromptInfo>)` if a valid cache entry exists,
    /// or `None` if the cache is missing or expired.
    ///
    /// # Arguments
    ///
    /// * `server` - The server name to look up
    pub async fn get_prompts(&self, server: &str) -> Option<Vec<PromptInfo>> {
        let cache = self.prompts.read().await;

        if let Some(cached) = cache.get(server) {
            if cached.is_valid() {
                tracing::debug!(server = server, "Cache hit for prompts");
                return Some(cached.data.clone());
            } else {
                tracing::debug!(server = server, "Cache expired for prompts");
            }
        } else {
            tracing::debug!(server = server, "Cache miss for prompts");
        }

        None
    }

    /// Set cached prompts for a server
    ///
    /// # Arguments
    ///
    /// * `server` - The server name
    /// * `prompts` - The prompts to cache
    /// * `ttl` - The time-to-live for this cache entry
    pub async fn set_prompts(&self, server: String, prompts: Vec<PromptInfo>, ttl: Duration) {
        let mut cache = self.prompts.write().await;

        cache.insert(server.clone(), CachedResponse::new(prompts, ttl));

        tracing::debug!(
            server = server.as_str(),
            ttl_secs = ttl.as_secs(),
            "Cached prompts"
        );
    }

    // ========== Invalidation ==========

    /// Invalidate all cached data for a specific server
    ///
    /// This removes all cached tools, resources, and prompts for the given server.
    ///
    /// # Arguments
    ///
    /// * `server` - The server name to invalidate
    pub async fn invalidate(&self, server: &str) {
        let mut tools = self.tools.write().await;
        let mut resources = self.resources.write().await;
        let mut prompts = self.prompts.write().await;

        let removed_tools = tools.remove(server).is_some();
        let removed_resources = resources.remove(server).is_some();
        let removed_prompts = prompts.remove(server).is_some();

        if removed_tools || removed_resources || removed_prompts {
            tracing::debug!(server = server, "Invalidated cache");
        }
    }

    /// Invalidate all cached data for all servers
    ///
    /// This clears the entire cache.
    pub async fn invalidate_all(&self) {
        let mut tools = self.tools.write().await;
        let mut resources = self.resources.write().await;
        let mut prompts = self.prompts.write().await;

        let total_removed = tools.len() + resources.len() + prompts.len();

        tools.clear();
        resources.clear();
        prompts.clear();

        tracing::debug!(entries_removed = total_removed, "Invalidated all caches");
    }

    /// Get cache statistics
    ///
    /// Returns a tuple of (tools_count, resources_count, prompts_count)
    pub async fn stats(&self) -> (usize, usize, usize) {
        let tools = self.tools.read().await;
        let resources = self.resources.read().await;
        let prompts = self.prompts.read().await;

        (tools.len(), resources.len(), prompts.len())
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        // Default TTL: 5 minutes
        Self::new(Duration::from_secs(300))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tools_cache_hit() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let tools = vec![ToolInfo {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({})),
        }];

        cache
            .set_tools(
                "server1".to_string(),
                tools.clone(),
                Duration::from_secs(60),
            )
            .await;

        let cached_tools = cache.get_tools("server1").await;
        assert!(cached_tools.is_some());
        assert_eq!(cached_tools.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_tools_cache_miss() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let cached_tools = cache.get_tools("nonexistent").await;
        assert!(cached_tools.is_none());
    }

    #[tokio::test]
    async fn test_tools_cache_expiration() {
        let cache = ResponseCache::new(Duration::from_millis(10));

        let tools = vec![ToolInfo {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({})),
        }];

        cache
            .set_tools(
                "server1".to_string(),
                tools.clone(),
                Duration::from_millis(10),
            )
            .await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        let cached_tools = cache.get_tools("server1").await;
        assert!(cached_tools.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_server() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let tools = vec![ToolInfo {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({})),
        }];

        cache
            .set_tools(
                "server1".to_string(),
                tools.clone(),
                Duration::from_secs(60),
            )
            .await;

        cache.invalidate("server1").await;

        let cached_tools = cache.get_tools("server1").await;
        assert!(cached_tools.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let tools = vec![ToolInfo {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: Some(serde_json::json!({})),
        }];

        cache
            .set_tools(
                "server1".to_string(),
                tools.clone(),
                Duration::from_secs(60),
            )
            .await;
        cache
            .set_tools(
                "server2".to_string(),
                tools.clone(),
                Duration::from_secs(60),
            )
            .await;

        let (tools_count, _, _) = cache.stats().await;
        assert_eq!(tools_count, 2);

        cache.invalidate_all().await;

        let (tools_count, _, _) = cache.stats().await;
        assert_eq!(tools_count, 0);
    }

    #[tokio::test]
    async fn test_resources_cache() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let resources = vec![ResourceInfo {
            uri: "test://resource".to_string(),
            name: Some("Test Resource".to_string()),
            description: Some("A test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
        }];

        cache
            .set_resources(
                "server1".to_string(),
                resources.clone(),
                Duration::from_secs(60),
            )
            .await;

        let cached = cache.get_resources("server1").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_prompts_cache() {
        let cache = ResponseCache::new(Duration::from_secs(60));

        let prompts = vec![PromptInfo {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: vec![],
        }];

        cache
            .set_prompts(
                "server1".to_string(),
                prompts.clone(),
                Duration::from_secs(60),
            )
            .await;

        let cached = cache.get_prompts("server1").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }
}

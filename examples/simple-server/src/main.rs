//! Simple MCP Server Example
//!
//! このサンプルは、基本的なMCP (Model Context Protocol) サーバーの実装例です。
//! 3つのツール、1つのリソース、1つのプロンプトを提供します。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

// JSON-RPC メッセージ構造
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCPサーバーのメイン構造体
struct SimpleMcpServer;

impl SimpleMcpServer {
    /// リクエストを処理してレスポンスを生成
    fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request.params),
            "resources/list" => self.handle_resources_list(),
            "resources/read" => self.handle_resources_read(request.params),
            "prompts/list" => self.handle_prompts_list(),
            "prompts/get" => self.handle_prompts_get(request.params),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    /// Initialize ハンドラ
    fn handle_initialize(&self, _params: Option<Value>) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "simple-server",
                "version": "0.1.0"
            }
        }))
    }

    /// Tools List ハンドラ
    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "tools": [
                {
                    "name": "echo",
                    "description": "入力された文字列をそのまま返します",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "エコーする文字列"
                            }
                        },
                        "required": ["message"]
                    }
                },
                {
                    "name": "reverse",
                    "description": "入力された文字列を逆順にして返します",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "逆順にする文字列"
                            }
                        },
                        "required": ["text"]
                    }
                },
                {
                    "name": "uppercase",
                    "description": "入力された文字列を大文字に変換して返します",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "大文字にする文字列"
                            }
                        },
                        "required": ["text"]
                    }
                }
            ]
        }))
    }

    /// Tools Call ハンドラ
    fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: params is required".to_string(),
            data: None,
        })?;

        let name = params["name"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: name is required".to_string(),
            data: None,
        })?;

        let arguments = params["arguments"].clone();

        let result = match name {
            "echo" => {
                let message = arguments["message"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid params: message is required".to_string(),
                    data: None,
                })?;
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": message
                        }
                    ]
                })
            }
            "reverse" => {
                let text = arguments["text"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid params: text is required".to_string(),
                    data: None,
                })?;
                let reversed: String = text.chars().rev().collect();
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": reversed
                        }
                    ]
                })
            }
            "uppercase" => {
                let text = arguments["text"].as_str().ok_or_else(|| JsonRpcError {
                    code: -32602,
                    message: "Invalid params: text is required".to_string(),
                    data: None,
                })?;
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": text.to_uppercase()
                        }
                    ]
                })
            }
            _ => {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", name),
                    data: None,
                })
            }
        };

        Ok(result)
    }

    /// Resources List ハンドラ
    fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "resources": [
                {
                    "uri": "simple://greeting",
                    "name": "greeting",
                    "description": "挨拶メッセージを提供します",
                    "mimeType": "text/plain"
                }
            ]
        }))
    }

    /// Resources Read ハンドラ
    fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: params is required".to_string(),
            data: None,
        })?;

        let uri = params["uri"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: uri is required".to_string(),
            data: None,
        })?;

        match uri {
            "simple://greeting" => Ok(json!({
                "contents": [
                    {
                        "uri": "simple://greeting",
                        "mimeType": "text/plain",
                        "text": "こんにちは！Simple MCP Serverへようこそ。\n\n利用可能なツール:\n- echo: メッセージをエコーします\n- reverse: 文字列を逆順にします\n- uppercase: 文字列を大文字にします\n\nぜひお試しください！"
                    }
                ]
            })),
            _ => Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown resource: {}", uri),
                data: None,
            }),
        }
    }

    /// Prompts List ハンドラ
    fn handle_prompts_list(&self) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "prompts": [
                {
                    "name": "help",
                    "description": "Simple MCP Serverのヘルプメッセージを表示します",
                    "arguments": []
                }
            ]
        }))
    }

    /// Prompts Get ハンドラ
    fn handle_prompts_get(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: params is required".to_string(),
            data: None,
        })?;

        let name = params["name"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Invalid params: name is required".to_string(),
            data: None,
        })?;

        match name {
            "help" => Ok(json!({
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Simple MCP Serverの使い方を教えてください。"
                        }
                    },
                    {
                        "role": "assistant",
                        "content": {
                            "type": "text",
                            "text": "Simple MCP Serverは、以下の機能を提供しています:\n\n【ツール】\n1. echo - 入力された文字列をそのまま返します\n   使用例: {\"message\": \"Hello, World!\"}\n\n2. reverse - 入力された文字列を逆順にします\n   使用例: {\"text\": \"Hello\"}\n\n3. uppercase - 入力された文字列を大文字に変換します\n   使用例: {\"text\": \"hello\"}\n\n【リソース】\n- simple://greeting - 挨拶メッセージを提供します\n\n【プロンプト】\n- help - このヘルプメッセージを表示します\n\n各機能は MCP Inspector MCP を使ってテストできます。"
                        }
                    }
                ]
            })),
            _ => Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown prompt: {}", name),
                data: None,
            }),
        }
    }
}

/// メイン関数：標準入出力でJSON-RPCメッセージを処理
#[tokio::main]
async fn main() -> Result<()> {
    let server = SimpleMcpServer;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // 標準エラー出力にログ出力（デバッグ用）
    eprintln!("Simple MCP Server started");

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read line from stdin")?;

        // 空行はスキップ
        if line.trim().is_empty() {
            continue;
        }

        // JSON-RPCリクエストをパース
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        eprintln!("Received request: method={}", request.method);

        // リクエストを処理
        let response = server.handle_request(request);

        // レスポンスをJSON形式で標準出力に書き込み
        let response_json = serde_json::to_string(&response)
            .context("Failed to serialize response")?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;

        eprintln!("Sent response");
    }

    eprintln!("Simple MCP Server stopped");
    Ok(())
}

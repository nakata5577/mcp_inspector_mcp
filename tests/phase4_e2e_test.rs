/// Phase 4 End-to-End Test
///
/// このテストは、モックサーバーの`trigger_sampling`ツールを使用して、
/// MonitoringTransportがSamplingリクエストを検出し、SamplingLoggerに記録することを検証します。
///
/// テスト手順:
/// 1. mcp_inspector_mcpをクライアントとして起動
/// 2. mock_samplingサーバーに接続
/// 3. tools_listでモックサーバーのツール一覧を取得
/// 4. sampling_logsで初期ログが空であることを確認
/// 5. tools_callでtrigger_samplingを実行
/// 6. sampling_logsでログが記録されたことを確認
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// MCPクライアントのヘルパー構造体
struct McpClient {
    child: Child,
    writer: Arc<Mutex<std::process::ChildStdin>>,
    #[allow(dead_code)]
    reader_thread: Option<thread::JoinHandle<Vec<String>>>,
}

impl McpClient {
    /// mcp_inspector_mcpを起動
    fn new() -> Result<Self> {
        let mut child = Command::new("cargo")
            .args(["run", "--release", "--quiet"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        // stderrのログを別スレッドで読み取り（ブロッキング防止）
        let stderr = child.stderr.take().expect("Failed to get stderr");
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                eprintln!("[SERVER] {}", line);
            }
        });

        let writer = Arc::new(Mutex::new(stdin));

        // stdoutの読み取りスレッドを起動
        let reader_thread = Some(thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut collected = Vec::new();

            for line in reader.lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    println!("[RESPONSE] {}", line);
                    collected.push(line);
                }
            }

            collected
        }));

        // サーバーの起動を待つ
        thread::sleep(Duration::from_secs(2));

        Ok(Self {
            child,
            writer,
            reader_thread,
        })
    }

    /// JSONRPCリクエストを送信
    fn send_request(&self, id: u64, method: &str, params: Value) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", request)?;
        writer.flush()?;

        // レスポンスの処理時間を待つ
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    /// サーバーを初期化
    fn initialize(&self) -> Result<()> {
        self.send_request(
            1,
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }),
        )?;

        // initialized通知を送信
        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", initialized)?;
        writer.flush()?;

        thread::sleep(Duration::from_secs(1));

        Ok(())
    }

    /// ツールを呼び出す
    fn call_tool(&self, id: u64, tool_name: &str, arguments: Value) -> Result<()> {
        self.send_request(
            id,
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments
            }),
        )
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
#[ignore] // 手動実行用（`cargo test --test phase4_e2e_test -- --ignored`で実行）
fn test_phase4_end_to_end() -> Result<()> {
    println!("\n=== Phase 4 End-to-End Test ===\n");

    // Step 1: mcp_inspector_mcpクライアントを起動
    println!("Step 1: Starting mcp_inspector_mcp client...");
    let client = McpClient::new()?;
    println!("✅ Client started\n");

    // Step 2: 初期化
    println!("Step 2: Initializing MCP connection...");
    client.initialize()?;
    println!("✅ Initialized\n");

    // Step 3: tools_listでモックサーバーのツール一覧を取得
    println!("Step 3: Listing tools from mock_sampling server...");
    client.call_tool(
        3,
        "tools_list",
        json!({
            "server": "mock_sampling"
        }),
    )?;
    thread::sleep(Duration::from_secs(2));
    println!("✅ Tools listed (check output above for trigger_sampling)\n");

    // Step 4: sampling_logsで初期ログを確認（空であるべき）
    println!("Step 4: Checking initial sampling logs (should be empty)...");
    client.call_tool(
        4,
        "sampling_logs",
        json!({
            "server": "mock_sampling"
        }),
    )?;
    thread::sleep(Duration::from_secs(2));
    println!("✅ Initial logs checked (should show total_count: 0)\n");

    // Step 5: trigger_samplingを実行（重要）
    println!("Step 5: Triggering sampling request...");
    client.call_tool(
        5,
        "tools_call",
        json!({
            "server": "mock_sampling",
            "tool_name": "trigger_sampling",
            "arguments": {
                "message": "Hello from Phase 4 end-to-end test!"
            }
        }),
    )?;
    thread::sleep(Duration::from_secs(3));
    println!("✅ Sampling request triggered\n");

    // Step 6: sampling_logsでログが記録されたことを確認（最重要）
    println!("Step 6: Checking sampling logs after trigger (CRITICAL TEST)...");
    client.call_tool(
        6,
        "sampling_logs",
        json!({
            "server": "mock_sampling"
        }),
    )?;
    thread::sleep(Duration::from_secs(2));
    println!("✅ Final logs checked\n");

    println!("\n=== Test Complete ===");
    println!("Please verify the following in the output above:");
    println!("1. Step 3: trigger_sampling tool is listed");
    println!("2. Step 4: total_count is 0");
    println!("3. Step 5: Sampling request sent successfully");
    println!(
        "4. Step 6: total_count > 0 and log entry contains 'Hello from Phase 4 end-to-end test!'"
    );
    println!("\nIf Step 6 shows total_count > 0, Phase 4 is SUCCESSFUL! ✅\n");

    // クリーンアップのため少し待つ
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

#[test]
#[ignore] // 手動実行用
fn test_multiple_sampling_requests() -> Result<()> {
    println!("\n=== Multiple Sampling Requests Test ===\n");

    let client = McpClient::new()?;
    client.initialize()?;

    // 初期ログ確認
    println!("Checking initial logs...");
    client.call_tool(1, "sampling_logs", json!({"server": "mock_sampling"}))?;
    thread::sleep(Duration::from_secs(1));

    // 複数回trigger_samplingを実行
    for i in 1..=3 {
        println!("\nSending sampling request #{}...", i);
        client.call_tool(
            100 + i,
            "tools_call",
            json!({
                "server": "mock_sampling",
                "tool_name": "trigger_sampling",
                "arguments": {
                    "message": format!("Test message #{}", i)
                }
            }),
        )?;
        thread::sleep(Duration::from_secs(2));
    }

    // 最終ログ確認
    println!("\nChecking final logs (should show 3 entries)...");
    client.call_tool(200, "sampling_logs", json!({"server": "mock_sampling"}))?;
    thread::sleep(Duration::from_secs(2));

    println!("\n=== Test Complete ===");
    println!("Expected: total_count should be 3\n");

    thread::sleep(Duration::from_secs(2));

    Ok(())
}

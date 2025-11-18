# Simple MCP Server Integration Test (PowerShell)

$ErrorActionPreference = "Continue"

# Color output functions
function Log-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Log-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Log-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

# Get project directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

Log-Info "Simple MCP Server integration test started"
Log-Info "Project directory: $ProjectDir"

# Step 1: Build
Log-Info "Step 1: Building server"
Set-Location $ProjectDir

$BuildOutput = cargo build 2>&1
if ($LASTEXITCODE -eq 0) {
    Log-Info "Build successful"
}
else {
    Log-Error "Build failed"
    Write-Host $BuildOutput
    exit 1
}

# Step 2: Check MCP Inspector MCP
Log-Info "Step 2: Checking MCP Inspector MCP"
$InspectorDir = Split-Path -Parent (Split-Path -Parent $ProjectDir)
Log-Info "Inspector directory: $InspectorDir"

if (-not (Test-Path "$InspectorDir\Cargo.toml")) {
    Log-Error "MCP Inspector MCP not found"
    exit 1
}

# Step 3: Basic server test
Log-Info "Step 3: Testing basic server functionality"

# Initialize request
$TestRequest = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null

if ($TestResult -match "simple-server") {
    Log-Info "Initialize successful"
}
else {
    Log-Error "Initialize failed"
    Write-Host "Response: $TestResult"
    exit 1
}

# Step 4: Tools test
Log-Info "Step 4: Testing tools"

# tools/list
$TestRequest = '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "echo") {
    Log-Info "tools/list successful"
}
else {
    Log-Error "tools/list failed"
    exit 1
}

# echo tool
$TestRequest = '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello, MCP!"}}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "Hello, MCP!") {
    Log-Info "echo tool successful"
}
else {
    Log-Error "echo tool failed"
    exit 1
}

# reverse tool
$TestRequest = '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"reverse","arguments":{"text":"Hello"}}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "olleH") {
    Log-Info "reverse tool successful"
}
else {
    Log-Error "reverse tool failed"
    exit 1
}

# uppercase tool
$TestRequest = '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"uppercase","arguments":{"text":"hello"}}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "HELLO") {
    Log-Info "uppercase tool successful"
}
else {
    Log-Error "uppercase tool failed"
    exit 1
}

# Step 5: Resources test
Log-Info "Step 5: Testing resources"

# resources/list
$TestRequest = '{"jsonrpc":"2.0","id":6,"method":"resources/list"}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "greeting") {
    Log-Info "resources/list successful"
}
else {
    Log-Error "resources/list failed"
    exit 1
}

# resources/read
$TestRequest = '{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"simple://greeting"}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "greeting") {
    Log-Info "resources/read successful"
}
else {
    Log-Error "resources/read failed"
    exit 1
}

# Step 6: Prompts test
Log-Info "Step 6: Testing prompts"

# prompts/list
$TestRequest = '{"jsonrpc":"2.0","id":8,"method":"prompts/list"}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "help") {
    Log-Info "prompts/list successful"
}
else {
    Log-Error "prompts/list failed"
    exit 1
}

# prompts/get
$TestRequest = '{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"help"}}'
$TestResult = $TestRequest | cargo run --quiet 2>$null
if ($TestResult -match "help") {
    Log-Info "prompts/get successful"
}
else {
    Log-Error "prompts/get failed"
    exit 1
}

# Step 7: Complete
Log-Info "========================================="
Log-Info "All tests passed!"
Log-Info "========================================="
Log-Info "Tested features:"
Log-Info "  - Initialize"
Log-Info "  - Tools: echo, reverse, uppercase"
Log-Info "  - Resources: greeting"
Log-Info "  - Prompts: help"
Log-Info ""
Log-Info "Next steps:"
Log-Info "  1. Use MCP Inspector MCP for detailed inspection"
Log-Info "  2. Add custom tools to extend functionality"
Log-Info "  3. Enhance error handling"

exit 0

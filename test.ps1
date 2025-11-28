# MTCP 测试脚本 (PowerShell)
# 用法: .\test.ps1

Write-Host "=== MTCP 测试脚本 ===" -ForegroundColor Green

# 检查是否已编译
if (-not (Test-Path "target\release\mtcp.exe")) {
    Write-Host "正在编译项目..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "编译失败！" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n步骤 1: 启动后端服务器 (端口 9000)" -ForegroundColor Cyan
Write-Host "在新终端运行: python example_backend.py"
Write-Host "按任意键继续..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

Write-Host "`n步骤 2: 启动 MTCP 服务端 (端口 8000)" -ForegroundColor Cyan
Write-Host "在新终端运行: " -ForegroundColor Yellow
Write-Host '$env:RUST_LOG="info"; .\target\release\mtcp.exe server -c config.server.toml' -ForegroundColor White
Write-Host "按任意键继续..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

Write-Host "`n步骤 3: 启动 MTCP 客户端 (端口 7000)" -ForegroundColor Cyan
Write-Host "在新终端运行: " -ForegroundColor Yellow
Write-Host '$env:RUST_LOG="info"; .\target\release\mtcp.exe client -c config.client.toml' -ForegroundColor White
Write-Host "按任意键继续..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

Write-Host "`n步骤 4: 测试连接" -ForegroundColor Cyan
Write-Host "正在通过 MTCP 访问后端服务..." -ForegroundColor Yellow

try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:7000" -TimeoutSec 5
    Write-Host "`n成功！响应状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "响应内容:" -ForegroundColor Green
    Write-Host $response.Content
} catch {
    Write-Host "`n测试失败: $_" -ForegroundColor Red
    Write-Host "请确保所有服务都已启动" -ForegroundColor Yellow
}

Write-Host "`n=== 测试完成 ===" -ForegroundColor Green

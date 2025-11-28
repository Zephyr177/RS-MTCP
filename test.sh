#!/bin/bash
# MTCP 测试脚本 (Bash)
# 用法: ./test.sh

echo "=== MTCP 测试脚本 ==="

# 检查是否已编译
if [ ! -f "target/release/mtcp" ]; then
    echo "正在编译项目..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "编译失败！"
        exit 1
    fi
fi

echo ""
echo "步骤 1: 启动后端服务器 (端口 9000)"
echo "在新终端运行: python3 example_backend.py"
echo "按 Enter 继续..."
read

echo ""
echo "步骤 2: 启动 MTCP 服务端 (端口 8000)"
echo "在新终端运行:"
echo "RUST_LOG=info ./target/release/mtcp server -c config.server.toml"
echo "按 Enter 继续..."
read

echo ""
echo "步骤 3: 启动 MTCP 客户端 (端口 7000)"
echo "在新终端运行:"
echo "RUST_LOG=info ./target/release/mtcp client -c config.client.toml"
echo "按 Enter 继续..."
read

echo ""
echo "步骤 4: 测试连接"
echo "正在通过 MTCP 访问后端服务..."

if command -v curl &> /dev/null; then
    curl -v http://127.0.0.1:7000
    if [ $? -eq 0 ]; then
        echo ""
        echo "成功！"
    else
        echo ""
        echo "测试失败！请确保所有服务都已启动"
    fi
else
    echo "未找到 curl 命令，请手动测试: curl http://127.0.0.1:7000"
fi

echo ""
echo "=== 测试完成 ==="

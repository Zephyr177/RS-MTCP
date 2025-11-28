# MTCP 使用指南

## 快速开始

### 1. 编译项目

```bash
cargo build --release
```

### 2. 准备配置文件

项目已包含两个示例配置文件：
- `config.server.toml` - 服务端配置
- `config.client.toml` - 客户端配置

**重要**: 所有 IP 地址和端口都需要根据你的实际环境修改！

### 3. 启动后端服务

首先需要一个后端服务用于测试。可以使用提供的 Python 示例：

```bash
python example_backend.py
```

这会在 `127.0.0.1:9000` 启动一个简单的 HTTP 服务器。

### 4. 启动 MTCP 服务端

```bash
# Windows
set RUST_LOG=info
target\release\mtcp.exe server -c config.server.toml

# Linux/Mac
RUST_LOG=info ./target/release/mtcp server -c config.server.toml
```

服务端会：
- 监听 `0.0.0.0:8000` 等待 MTCP 客户端连接
- 将流量转发到 `127.0.0.1:9000` (后端服务)

### 5. 启动 MTCP 客户端

在另一个终端：

```bash
# Windows
set RUST_LOG=info
target\release\mtcp.exe client -c config.client.toml

# Linux/Mac
RUST_LOG=info ./target/release/mtcp client -c config.client.toml
```

客户端会：
- 建立 4 个到服务端的 TCP 连接（连接池）
- 监听 `127.0.0.1:7000` 等待应用程序连接

### 6. 测试连接

现在可以通过客户端访问后端服务：

```bash
# 使用 curl
curl http://127.0.0.1:7000

# 或使用浏览器访问
# http://127.0.0.1:7000
```

## 配置详解

### 服务端配置参数

```toml
[server]
listen_ip = "0.0.0.0"           # 监听所有网卡
listen_port = 8000              # MTCP 协议端口
backend_ip = "127.0.0.1"        # 后端服务地址
backend_port = 9000             # 后端服务端口
connection_pool_size = 4        # 连接池大小（建议 2-8）
buffer_size = 65536             # 64KB 缓冲区
```

### 客户端配置参数

```toml
[client]
local_listen_ip = "127.0.0.1"   # 本地监听地址
local_listen_port = 7000        # 应用程序连接此端口
server_ip = "127.0.0.1"         # MTCP 服务器地址
server_port = 8000              # MTCP 服务器端口
connection_pool_size = 4        # 必须与服务端一致
buffer_size = 65536             # 必须与服务端一致
enable_zero_rtt = true          # 启用预连接池
```

## 实际部署场景

### 场景 1: 本地测试

```
应用 -> 客户端(127.0.0.1:7000) -> 服务端(127.0.0.1:8000) -> 后端(127.0.0.1:9000)
```

使用默认配置即可。

### 场景 2: 远程服务器

假设：
- 远程服务器 IP: `203.0.113.10`
- 远程后端服务: `203.0.113.10:80`

**服务端配置** (部署在 203.0.113.10):
```toml
[server]
listen_ip = "0.0.0.0"
listen_port = 8000
backend_ip = "127.0.0.1"
backend_port = 80
connection_pool_size = 8
buffer_size = 131072
```

**客户端配置** (本地):
```toml
[client]
local_listen_ip = "127.0.0.1"
local_listen_port = 7000
server_ip = "203.0.113.10"
server_port = 8000
connection_pool_size = 8
buffer_size = 131072
enable_zero_rtt = true
```

### 场景 3: 内网穿透

客户端在内网，服务端在公网：

**服务端** (公网 IP: 198.51.100.5):
```toml
[server]
listen_ip = "0.0.0.0"
listen_port = 8000
backend_ip = "10.0.0.100"      # 内网服务器
backend_port = 3306            # 例如 MySQL
connection_pool_size = 4
buffer_size = 65536
```

**客户端** (内网):
```toml
[client]
local_listen_ip = "127.0.0.1"
local_listen_port = 3306
server_ip = "198.51.100.5"
server_port = 8000
connection_pool_size = 4
buffer_size = 65536
enable_zero_rtt = true
```

然后本地应用连接 `127.0.0.1:3306` 即可。

## 性能优化建议

### 连接池大小

- **低延迟网络** (局域网): 2-4 个连接
- **高延迟网络** (跨国): 4-8 个连接
- **高丢包网络**: 8-16 个连接

### 缓冲区大小

- **小文件传输**: 32KB (32768)
- **一般用途**: 64KB (65536)
- **大文件传输**: 128KB-256KB (131072-262144)

### 0-RTT 设置

- **频繁短连接**: 启用 (enable_zero_rtt = true)
- **长连接应用**: 可选
- **资源受限**: 禁用 (enable_zero_rtt = false)

## 监控和调试

### 启用详细日志

```bash
# 信息级别
RUST_LOG=info ./mtcp server -c config.toml

# 调试级别
RUST_LOG=debug ./mtcp server -c config.toml

# 仅 MTCP 模块
RUST_LOG=mtcp=debug ./mtcp server -c config.toml
```

### 日志输出说明

- `Connection X established`: 连接池中的连接已建立
- `New stream X requested`: 新的逻辑流创建
- `Data for stream X: Y bytes`: 数据传输
- `Stream X closed`: 流关闭

## 故障排查

### 客户端无法连接服务端

1. 检查服务端是否启动
2. 检查防火墙规则
3. 验证 IP 和端口配置
4. 查看服务端日志

### 连接建立但无数据传输

1. 检查后端服务是否运行
2. 验证 backend_ip 和 backend_port
3. 查看服务端日志中的错误信息

### 性能不如预期

1. 增加 connection_pool_size
2. 调整 buffer_size
3. 检查网络带宽和延迟
4. 确认没有其他瓶颈（CPU、磁盘 I/O）

## 安全建议

1. **生产环境**: 建议在 MTCP 外层添加 TLS/SSL 加密
2. **防火墙**: 限制 MTCP 端口只允许特定 IP 访问
3. **认证**: 可扩展协议添加认证机制
4. **监控**: 记录所有连接和异常行为

## 限制和注意事项

- 不支持 UDP 协议
- 需要客户端和服务端同时部署
- 连接池大小需要两端一致
- 不保证消息顺序（在不同连接间）
- 需要稳定的网络连接

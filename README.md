# MTCP - Multi-TCP Connection Aggregator

用户态 MTCP 实现，通过聚合多个 TCP 连接提升单连接速度，支持预连接池实现 0-RTT。

## 功能特性

- **连接聚合**: 将多个 TCP 连接聚合成单个逻辑连接
- **负载均衡**: 自动在多个物理连接间分配数据
- **0-RTT**: 通过预建立连接池实现零往返时延
- **完整配置**: 所有参数可配置，无硬编码默认值
- **双端支持**: 需要客户端和服务端同时部署

## 架构

```
应用程序 <-> MTCP客户端 <=[多个TCP连接]=> MTCP服务端 <-> 后端服务
```

## 编译

```bash
cargo build --release
```

## 配置

### 服务端配置 (config.server.toml)

```toml
mode = "server"

[server]
listen_ip = "0.0.0.0"          # MTCP 监听地址
listen_port = 8000              # MTCP 监听端口
backend_ip = "127.0.0.1"        # 后端服务地址
backend_port = 9000             # 后端服务端口
connection_pool_size = 4        # 连接池大小
buffer_size = 65536             # 缓冲区大小
```

### 客户端配置 (config.client.toml)

```toml
mode = "client"

[client]
local_listen_ip = "127.0.0.1"   # 本地监听地址
local_listen_port = 7000        # 本地监听端口
server_ip = "127.0.0.1"         # MTCP 服务器地址
server_port = 8000              # MTCP 服务器端口
connection_pool_size = 4        # 连接池大小
buffer_size = 65536             # 缓冲区大小
enable_zero_rtt = true          # 启用 0-RTT
```

## 使用方法

### 启动服务端

```bash
# 使用默认配置文件
./target/release/mtcp server -c config.server.toml

# 或使用自定义配置
./target/release/mtcp server -c /path/to/your/config.toml
```

### 启动客户端

```bash
# 使用默认配置文件
./target/release/mtcp client -c config.client.toml

# 或使用自定义配置
./target/release/mtcp client -c /path/to/your/config.toml
```

### 启用日志

```bash
# 设置日志级别
RUST_LOG=info ./target/release/mtcp server -c config.server.toml
RUST_LOG=debug ./target/release/mtcp client -c config.client.toml
```

## 工作原理

### 连接聚合

客户端建立多个到服务端的 TCP 连接，形成连接池。当应用程序连接到客户端时：

1. 客户端为该连接分配一个 stream_id
2. 数据通过轮询方式分配到不同的物理连接
3. 服务端根据 stream_id 重组数据并转发到后端

### 0-RTT 实现

启用 `enable_zero_rtt` 后：

1. 客户端启动时预建立所有连接
2. 应用程序连接时无需等待 TCP 握手
3. 数据可立即通过已建立的连接发送

### 协议格式

每个消息包含：
- 消息长度 (4 字节)
- 消息类型 (1 字节)
- Stream ID (4 字节，数据消息)
- 数据长度 (4 字节，数据消息)
- 数据内容

## 性能调优

- `connection_pool_size`: 增加连接数可提升吞吐量，但会增加资源消耗
- `buffer_size`: 更大的缓冲区可减少系统调用，但增加内存使用
- `enable_zero_rtt`: 启用可减少延迟，但会保持持久连接

## 示例场景

### 加速 HTTP 代理

```bash
# 服务端 (部署在远程服务器)
./mtcp server -c config.server.toml
# listen_ip = "0.0.0.0", listen_port = 8000
# backend_ip = "127.0.0.1", backend_port = 80

# 客户端 (本地)
./mtcp client -c config.client.toml
# local_listen_ip = "127.0.0.1", local_listen_port = 7000
# server_ip = "remote.server.com", server_port = 8000

# 应用程序连接到 127.0.0.1:7000
curl http://127.0.0.1:7000
```

## 注意事项

- 所有 IP 和端口必须在配置文件中明确指定
- 客户端和服务端必须同时运行才能工作
- 确保防火墙允许相应端口的连接
- 建议在生产环境使用 TLS 加密（需额外实现）

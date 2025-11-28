# MTCP 快速入门

## 5 分钟快速测试

### 1. 编译项目

```bash
cargo build --release
```

### 2. 准备 4 个终端窗口

#### 终端 1: 后端服务 (端口 9000)

```bash
python example_backend.py
```

输出应该显示:
```
后端服务器监听在 127.0.0.1:9000
```

#### 终端 2: MTCP 服务端 (端口 8000)

Windows:
```powershell
$env:RUST_LOG="info"
.\target\release\mtcp.exe server -c config.server.toml
```

Linux/Mac:
```bash
RUST_LOG=info ./target/release/mtcp server -c config.server.toml
```

输出应该显示:
```
Server listening on 0.0.0.0:8000
Backend target: 127.0.0.1:9000
```

#### 终端 3: MTCP 客户端 (端口 7000)

Windows:
```powershell
$env:RUST_LOG="info"
.\target\release\mtcp.exe client -c config.client.toml
```

Linux/Mac:
```bash
RUST_LOG=info ./target/release/mtcp client -c config.client.toml
```

输出应该显示:
```
Initializing client with 4 connections to 127.0.0.1:8000
Connection 0 established to 127.0.0.1:8000
Connection 1 established to 127.0.0.1:8000
Connection 2 established to 127.0.0.1:8000
Connection 3 established to 127.0.0.1:8000
0-RTT enabled: Pre-established connection pool ready
Client listening on 127.0.0.1:7000
```

#### 终端 4: 测试客户端

```bash
curl http://127.0.0.1:7000
```

或在浏览器打开: `http://127.0.0.1:7000`

### 3. 观察日志

你应该能看到：

**客户端日志**:
```
New local connection from 127.0.0.1:xxxxx
Handling stream 1
```

**服务端日志**:
```
New MTCP connection from 127.0.0.1:xxxxx
New stream 1 requested
```

**后端日志**:
```
新连接来自 ('127.0.0.1', xxxxx)
收到请求:
GET / HTTP/1.1
```

### 4. 成功！

如果你看到 HTML 响应，说明 MTCP 工作正常！

## 配置自己的服务

### 修改服务端配置

编辑 `config.server.toml`:

```toml
[server]
listen_ip = "0.0.0.0"          # 改为你的服务器 IP
listen_port = 8000              # 改为你想要的端口
backend_ip = "127.0.0.1"        # 改为你的后端服务 IP
backend_port = 9000             # 改为你的后端服务端口
connection_pool_size = 4        # 根据需要调整
buffer_size = 65536
```

### 修改客户端配置

编辑 `config.client.toml`:

```toml
[client]
local_listen_ip = "127.0.0.1"   # 本地监听地址
local_listen_port = 7000        # 应用程序连接的端口
server_ip = "127.0.0.1"         # MTCP 服务器的 IP
server_port = 8000              # MTCP 服务器的端口
connection_pool_size = 4        # 必须与服务端一致
buffer_size = 65536
enable_zero_rtt = true
```

## 常见问题

### Q: 客户端无法连接服务端

**A**: 检查：
1. 服务端是否已启动
2. 防火墙是否允许端口 8000
3. IP 地址是否正确

### Q: 连接成功但无响应

**A**: 检查：
1. 后端服务是否运行
2. backend_ip 和 backend_port 是否正确
3. 查看服务端日志的错误信息

### Q: 性能没有提升

**A**: 尝试：
1. 增加 connection_pool_size (例如 8 或 16)
2. 增加 buffer_size (例如 131072)
3. 确保网络是瓶颈而不是 CPU 或磁盘

### Q: 如何停止服务

**A**: 在各个终端按 `Ctrl+C`

## 下一步

- 阅读 [README.md](README.md) 了解更多功能
- 阅读 [USAGE.md](USAGE.md) 了解详细使用方法
- 阅读 [ARCHITECTURE.md](ARCHITECTURE.md) 了解架构设计

## 生产环境部署

1. 使用 systemd 或其他进程管理器
2. 配置日志轮转
3. 设置监控和告警
4. 考虑添加 TLS 加密
5. 配置防火墙规则

示例 systemd 服务文件 (mtcp-server.service):

```ini
[Unit]
Description=MTCP Server
After=network.target

[Service]
Type=simple
User=mtcp
WorkingDirectory=/opt/mtcp
Environment="RUST_LOG=info"
ExecStart=/opt/mtcp/target/release/mtcp server -c /opt/mtcp/config.server.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

启用服务:
```bash
sudo systemctl enable mtcp-server
sudo systemctl start mtcp-server
sudo systemctl status mtcp-server
```

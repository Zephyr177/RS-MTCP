#!/usr/bin/env python3
"""
简单的 HTTP 后端服务器用于测试 MTCP
运行: python example_backend.py
"""

import socket
import threading

def handle_client(client_socket):
    try:
        request = client_socket.recv(4096).decode('utf-8')
        print(f"收到请求:\n{request[:200]}")
        
        response = """HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Connection: close

<!DOCTYPE html>
<html>
<head><title>MTCP 测试</title></head>
<body>
<h1>MTCP 后端服务器</h1>
<p>这是通过 MTCP 聚合连接传输的响应！</p>
<p>当前时间: """ + str(__import__('datetime').datetime.now()) + """</p>
</body>
</html>
"""
        client_socket.sendall(response.encode('utf-8'))
    except Exception as e:
        print(f"错误: {e}")
    finally:
        client_socket.close()

def main():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(('127.0.0.1', 9000))
    server.listen(5)
    print("后端服务器监听在 127.0.0.1:9000")
    
    while True:
        client, addr = server.accept()
        print(f"新连接来自 {addr}")
        thread = threading.Thread(target=handle_client, args=(client,))
        thread.start()

if __name__ == '__main__':
    main()

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use log::{info, error};
use bytes::Bytes;
use crate::config::ClientConfig;
use crate::connection_pool::ConnectionPool;
use crate::protocol::Message;

pub struct Client {
    config: ClientConfig,
    pool: Arc<ConnectionPool>,
    next_stream_id: Arc<AtomicU32>,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing client with {} connections to {}:{}", 
              config.connection_pool_size, config.server_ip, config.server_port);

        let pool = ConnectionPool::new(
            &config.server_ip,
            config.server_port,
            config.connection_pool_size,
        ).await?;

        let pool = Arc::new(pool);
        pool.listen_all().await;

        if config.enable_zero_rtt {
            info!("0-RTT enabled: Pre-established connection pool ready");
        }

        Ok(Client {
            config,
            pool,
            next_stream_id: Arc::new(AtomicU32::new(1)),
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("{}:{}", 
            self.config.local_listen_ip, 
            self.config.local_listen_port
        )).await?;

        info!("Client listening on {}:{}", 
              self.config.local_listen_ip, 
              self.config.local_listen_port);

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("New local connection from {}", addr);

            let stream_id = self.next_stream_id.fetch_add(1, Ordering::SeqCst);
            let pool = self.pool.clone();
            let buffer_size = self.config.buffer_size;

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, stream_id, pool, buffer_size).await {
                    error!("Connection error for stream {}: {}", stream_id, e);
                }
            });
        }
    }

    async fn handle_connection(
        local_stream: TcpStream,
        stream_id: u32,
        pool: Arc<ConnectionPool>,
        buffer_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Handling stream {}", stream_id);

        pool.send_message(Message::NewStream { stream_id }).await?;

        let mut rx = pool.start_receiving(stream_id).await;

        let (mut local_read, mut local_write) = tokio::io::split(local_stream);
        let pool_clone = pool.clone();

        let read_task = tokio::spawn(async move {
            let mut buf = vec![0u8; buffer_size];
            loop {
                match local_read.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = Bytes::copy_from_slice(&buf[..n]);
                        if let Err(e) = pool_clone.send_message(Message::Data { stream_id, data }).await {
                            error!("Failed to send data: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Read error: {}", e);
                        break;
                    }
                }
            }
            let _ = pool_clone.send_message(Message::CloseStream { stream_id }).await;
        });

        let write_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = local_write.write_all(&data).await {
                    error!("Write error: {}", e);
                    break;
                }
            }
        });

        let _ = tokio::try_join!(read_task, write_task);
        pool.stop_receiving(stream_id).await;

        info!("Stream {} closed", stream_id);
        Ok(())
    }
}

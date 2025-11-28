use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::collections::HashMap;
use log::{info, error, debug};
use bytes::Bytes;
use crate::config::ServerConfig;
use crate::protocol::Message;

pub struct Server {
    config: ServerConfig,
    streams: Arc<Mutex<HashMap<u32, mpsc::Sender<Bytes>>>>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Server {
            config,
            streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!("{}:{}", 
            self.config.listen_ip, 
            self.config.listen_port
        )).await?;

        info!("Server listening on {}:{}", 
              self.config.listen_ip, 
              self.config.listen_port);
        info!("Backend target: {}:{}", 
              self.config.backend_ip, 
              self.config.backend_port);

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("New MTCP connection from {}", addr);

            let streams = self.streams.clone();
            let backend_ip = self.config.backend_ip.clone();
            let backend_port = self.config.backend_port;
            let buffer_size = self.config.buffer_size;

            tokio::spawn(async move {
                if let Err(e) = Self::handle_mtcp_connection(
                    socket, 
                    streams, 
                    backend_ip, 
                    backend_port,
                    buffer_size
                ).await {
                    error!("MTCP connection error: {}", e);
                }
            });
        }
    }

    async fn handle_mtcp_connection(
        mut mtcp_stream: TcpStream,
        streams: Arc<Mutex<HashMap<u32, mpsc::Sender<Bytes>>>>,
        backend_ip: String,
        backend_port: u16,
        buffer_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Ok(l) = mtcp_stream.read_u32().await {
            let len = l as usize;

            let mut buf = vec![0u8; len];
            mtcp_stream.read_exact(&mut buf).await?;

            let msg = Message::decode(Bytes::from(buf))?;

            match msg {
                Message::NewStream { stream_id } => {
                    info!("New stream {} requested", stream_id);
                    
                    let backend_stream = TcpStream::connect(
                        format!("{}:{}", backend_ip, backend_port)
                    ).await?;
                    
                    let (tx, rx) = mpsc::channel(100);
                    streams.lock().await.insert(stream_id, tx);

                    let streams_clone = streams.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_backend_stream(
                            backend_stream,
                            stream_id,
                            rx,
                            streams_clone,
                            buffer_size,
                        ).await {
                            error!("Backend stream {} error: {}", stream_id, e);
                        }
                    });
                }
                Message::Data { stream_id, data } => {
                    debug!("Data for stream {}: {} bytes", stream_id, data.len());
                    let streams_lock = streams.lock().await;
                    if let Some(tx) = streams_lock.get(&stream_id) {
                        let _ = tx.send(data).await;
                    }
                }
                Message::CloseStream { stream_id } => {
                    info!("Close stream {} requested", stream_id);
                    streams.lock().await.remove(&stream_id);
                }
                Message::Heartbeat => {
                    debug!("Heartbeat received");
                }
            }
        }

        Ok(())
    }

    async fn handle_backend_stream(
        backend_stream: TcpStream,
        stream_id: u32,
        mut rx: mpsc::Receiver<Bytes>,
        streams: Arc<Mutex<HashMap<u32, mpsc::Sender<Bytes>>>>,
        buffer_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (mut backend_read, mut backend_write) = tokio::io::split(backend_stream);

        let read_task = tokio::spawn(async move {
            let mut buf = vec![0u8; buffer_size];
            loop {
                match backend_read.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let _data = Bytes::copy_from_slice(&buf[..n]);
                        // Send back to client via MTCP (would need connection reference)
                        debug!("Read {} bytes from backend for stream {}", n, stream_id);
                    }
                    Err(e) => {
                        error!("Backend read error: {}", e);
                        break;
                    }
                }
            }
        });

        let write_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = backend_write.write_all(&data).await {
                    error!("Backend write error: {}", e);
                    break;
                }
            }
        });

        let _ = tokio::try_join!(read_task, write_task);
        streams.lock().await.remove(&stream_id);

        info!("Backend stream {} closed", stream_id);
        Ok(())
    }
}

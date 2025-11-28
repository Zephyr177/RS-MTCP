use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::collections::HashMap;
use log::{info, error, debug};
use bytes::Bytes;
use crate::protocol::Message;

pub struct ConnectionPool {
    connections: Vec<Arc<Mutex<TcpStream>>>,
    next_conn: Arc<Mutex<usize>>,
    tx_channels: Arc<Mutex<HashMap<u32, mpsc::Sender<Bytes>>>>,
}

impl ConnectionPool {
    pub async fn new(server_ip: &str, server_port: u16, pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();
        
        for i in 0..pool_size {
            match TcpStream::connect(format!("{}:{}", server_ip, server_port)).await {
                Ok(stream) => {
                    info!("Connection {} established to {}:{}", i, server_ip, server_port);
                    connections.push(Arc::new(Mutex::new(stream)));
                }
                Err(e) => {
                    error!("Failed to establish connection {}: {}", i, e);
                    return Err(Box::new(e));
                }
            }
        }

        Ok(ConnectionPool {
            connections,
            next_conn: Arc::new(Mutex::new(0)),
            tx_channels: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn send_message(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = msg.encode();
        let conn_count = self.connections.len();
        
        let mut next = self.next_conn.lock().await;
        let conn_idx = *next;
        *next = (*next + 1) % conn_count;
        drop(next);

        let conn = self.connections[conn_idx].clone();
        let mut stream = conn.lock().await;
        
        stream.write_u32(encoded.len() as u32).await?;
        stream.write_all(&encoded).await?;
        stream.flush().await?;
        
        debug!("Sent message via connection {}", conn_idx);
        Ok(())
    }

    pub async fn start_receiving(&self, stream_id: u32) -> mpsc::Receiver<Bytes> {
        let (tx, rx) = mpsc::channel(100);
        self.tx_channels.lock().await.insert(stream_id, tx);
        rx
    }

    pub async fn stop_receiving(&self, stream_id: u32) {
        self.tx_channels.lock().await.remove(&stream_id);
    }

    pub async fn listen_all(&self) {
        for (idx, conn) in self.connections.iter().enumerate() {
            let conn = conn.clone();
            let tx_channels = self.tx_channels.clone();
            
            tokio::spawn(async move {
                loop {
                    let mut stream = conn.lock().await;
                    
                    let len = match stream.read_u32().await {
                        Ok(l) => l as usize,
                        Err(e) => {
                            error!("Connection {} read error: {}", idx, e);
                            break;
                        }
                    };

                    let mut buf = vec![0u8; len];
                    if let Err(e) = stream.read_exact(&mut buf).await {
                        error!("Connection {} read payload error: {}", idx, e);
                        break;
                    }
                    drop(stream);

                    match Message::decode(Bytes::from(buf)) {
                        Ok(Message::Data { stream_id, data }) => {
                            let channels = tx_channels.lock().await;
                            if let Some(tx) = channels.get(&stream_id) {
                                let _ = tx.send(data).await;
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to decode message: {}", e);
                        }
                    }
                }
            });
        }
    }
}

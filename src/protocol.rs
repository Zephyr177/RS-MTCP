use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;

#[derive(Debug, Clone)]
pub enum Message {
    Data { stream_id: u32, data: Bytes },
    NewStream { stream_id: u32 },
    CloseStream { stream_id: u32 },
    Heartbeat,
}

impl Message {
    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();
        
        match self {
            Message::Data { stream_id, data } => {
                buf.put_u8(0x01); // Type: Data
                buf.put_u32(*stream_id);
                buf.put_u32(data.len() as u32);
                buf.put_slice(data);
            }
            Message::NewStream { stream_id } => {
                buf.put_u8(0x02); // Type: NewStream
                buf.put_u32(*stream_id);
            }
            Message::CloseStream { stream_id } => {
                buf.put_u8(0x03); // Type: CloseStream
                buf.put_u32(*stream_id);
            }
            Message::Heartbeat => {
                buf.put_u8(0x04); // Type: Heartbeat
            }
        }
        
        buf.freeze()
    }

    pub fn decode(mut data: Bytes) -> io::Result<Self> {
        if data.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty data"));
        }

        let msg_type = data.get_u8();
        
        match msg_type {
            0x01 => {
                if data.remaining() < 8 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete data message"));
                }
                let stream_id = data.get_u32();
                let len = data.get_u32() as usize;
                
                if data.remaining() < len {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete payload"));
                }
                
                let payload = data.split_to(len);
                Ok(Message::Data { stream_id, data: payload })
            }
            0x02 => {
                if data.remaining() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete new stream message"));
                }
                let stream_id = data.get_u32();
                Ok(Message::NewStream { stream_id })
            }
            0x03 => {
                if data.remaining() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Incomplete close stream message"));
                }
                let stream_id = data.get_u32();
                Ok(Message::CloseStream { stream_id })
            }
            0x04 => Ok(Message::Heartbeat),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown message type")),
        }
    }
}

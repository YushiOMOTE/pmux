use bytes::{Buf, BufMut, BytesMut};
use tokio::net::TcpStream;
use tokio::prelude::*;

use crate::error::Result;

// [u16][u8...]
#[derive(Debug)]
pub struct Header {
    pub addr: String,
}

pub async fn read_header(conn: &mut TcpStream) -> Result<Header> {
    let mut buf = [0; 2];
    conn.read_exact(&mut buf).await?;
    let mut buf = BytesMut::from(&buf as &[u8]);
    let len = buf.get_u16_le();

    let mut buf = vec![0; len as usize];
    conn.read_exact(&mut buf).await?;
    let addr = String::from_utf8(buf)?;
    Ok(Header::new(addr))
}

pub async fn write_header(conn: &mut TcpStream, header: Header) -> Result<()> {
    let mut buf = BytesMut::new();
    buf.put_u16_le(header.addr.len() as u16);
    buf.extend_from_slice(header.addr.as_bytes());
    conn.write_all(&buf).await?;
    Ok(())
}

impl Header {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

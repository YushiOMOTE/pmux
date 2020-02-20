use anyhow::anyhow;
use bytes::{Buf, BufMut, BytesMut};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::TcpStream;
use tokio::prelude::*;

use crate::error::Result;

#[derive(Debug)]
pub struct Header {
    pub addr: SocketAddr,
}

pub async fn read_header(conn: &mut TcpStream) -> Result<Header> {
    let mut buf = [0; 6];
    conn.read_exact(&mut buf).await?;
    let mut buf = BytesMut::from(&buf as &[u8]);
    let addr = Ipv4Addr::new(buf.get_u8(), buf.get_u8(), buf.get_u8(), buf.get_u8());
    let addr = SocketAddr::V4(SocketAddrV4::new(addr, buf.get_u16_le()));
    Ok(Header::new(addr))
}

pub async fn write_header(conn: &mut TcpStream, header: Header) -> Result<()> {
    let mut buf = BytesMut::new();
    match header.addr {
        SocketAddr::V4(addr) => {
            for octet in addr.ip().octets().iter() {
                buf.put_u8(*octet);
            }
            buf.put_u16_le(addr.port());
        }
        SocketAddr::V6(_) => return Err(anyhow!("Ip v6 is not supported")),
    }
    conn.write_all(&buf).await?;
    Ok(())
}

impl Header {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

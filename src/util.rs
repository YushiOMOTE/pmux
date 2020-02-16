use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;

pub fn localhost(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
}

pub fn peer(stream: &TcpStream) -> String {
    stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or("<unknown>".into())
}

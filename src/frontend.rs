use log::*;
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener};
use tokio::stream::StreamExt;
use futures::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::Decoder;

use crate::codec::{Codec, Message};
use crate::error::{Error, Result};
use crate::options::Frontend;
use crate::util::{localhost, peer};

async fn frontend_handle(mut client: TcpStream, backend: SocketAddr, src_port: u16, dst_port: u16) -> Result<()> {
    let addr = peer(&client);

    let backend = TcpStream::connect(backend).await?;
    info!("[{}: {}->{}] Connected to backend {}", addr, src_port, dst_port, peer(&backend));

    let mut backend = Codec::new().framed(backend);

    loop {
        let mut buf = [0; 1024];

        tokio::select! {
            item = backend.next() => match item {
                Some(item) => {
                    let item = item?;
                    client.write_all(&item.payload).await?
                },
                None => {
                    info!("[{}: {}->{}] Connection closed by backend", addr, src_port, dst_port);
                    break;
                },
            },
            size = client.read(&mut buf) => {
                let size = size?;
                if size == 0 {
                    info!("[{}: {}->{}] Connection closed by client", addr, src_port, dst_port);
                    break;
                }
                let msg = Message::new(dst_port, &buf[0..size]);
                backend.send(msg).await?;
            }
        }
    }

    Ok(())
}

async fn start_rule(backend: SocketAddr, src_port: u16, dst_port: u16) -> Result<()> {
    let mut listener = TcpListener::bind(localhost(src_port)).await?;

    loop {
        let (client, _) = listener.accept().await?;

        tokio::spawn(async move {
            let addr = peer(&client);

            info!("[{}: {}->{}] Connection accepted", addr, src_port, dst_port);

            if let Err(e) = frontend_handle(client, backend, src_port, dst_port).await {
                error!("[{}: {}->{}] Error occurred: {}", addr, src_port, dst_port, e);
            }
        });
    }
}

pub async fn frontend(opt: &Frontend) -> Result<()> {
    let mut handles = vec![];

    for rule in &opt.rules {
        let mut tokens = rule.split(':');
        let src_port = match tokens.next() {
            Some(src) => {
                src.parse().map_err(|e| Error::RuleError(format!("Invalid src port: {}", e)))?
            }
            None => return Err(Error::RuleError("Invalid tokens".into()))
        };
        let dst_port: u16 = match tokens.next() {
            Some(dst) => {
                dst.parse().map_err(|e| Error::RuleError(format!("Invalid dst port: {}", e)))?
            }
            None => src_port
        };

        info!("[{}->{}] Starting redirection", src_port, dst_port);

        let backend = opt.backend.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = start_rule(backend, src_port, dst_port).await {
                error!("[{}->{}] Error occurred: {}", src_port, dst_port, e);
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}

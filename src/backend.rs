use log::*;
use tokio::net::{TcpStream, TcpListener};
use tokio::stream::StreamExt;
use futures::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::Decoder;

use crate::codec::{Codec, Message};
use crate::error::Result;
use crate::options::Backend;
use crate::util::{localhost, peer};

async fn handle_client(client: TcpStream) -> Result<()> {
    let addr = peer(&client);
    let mut client = Codec::new().framed(client);

    let (port, mut conn) = match client.next().await {
        Some(item) => {
            let item = item?;
            let port = item.port;
            let addr = localhost(port);
            let conn = TcpStream::connect(addr).await?;
            if item.payload.len() != 0 {
                warn!("[{}] Initial message should not contain payload", addr);
            }
            (port, conn)
        },
        None => {
            info!("[{}] Connection closed without any message", addr);
            return Ok(())
        }
    };

    info!("[{}] Redirect to {}", addr, peer(&conn));

    loop {
        let mut buf = [0; 1024];

        tokio::select! {
            item = client.next() => match item {
                Some(item) => {
                    let item = item?;
                    conn.write_all(&item.payload).await?
                },
                None => {
                    info!("[{}] Connection closed by client", addr);
                    break;
                },
            },
            size = conn.read(&mut buf) => {
                let size = size?;
                if size == 0 {
                    info!("[{}] Connection closed by destination", addr);
                    break;
                }
                let msg = Message::new(port, &buf[0..size]);
                client.send(msg).await?;
            }
        }
    }

    Ok(())
}

pub async fn backend(opt: &Backend) -> Result<()> {
    info!("Starting backend: {:?}", opt);

    let mut listener = TcpListener::bind(&opt.bind).await?;

    loop {
        let (client, _) = listener.accept().await?;

        tokio::spawn(async move {
            let addr = peer(&client);

            info!("[{}] Connection accepted", addr);

            if let Err(e) = handle_client(client).await {
                error!("[{}] Error occurred: {}", addr, e);
            }
        });
    }
}

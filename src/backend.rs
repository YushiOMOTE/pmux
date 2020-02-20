use anyhow::anyhow;
use log::*;
use std::net::SocketAddr;
use tokio::io::{copy, split};
use tokio::net::{TcpListener, TcpStream};

use crate::error::Result;
use crate::options::Backend;
use crate::proto::read_header;

async fn handle(mut cli: TcpStream, cli_addr: &SocketAddr) -> Result<()> {
    let header = read_header(&mut cli).await?;

    let srv = TcpStream::connect(header.addr).await?;

    let (mut cr, mut cw) = split(cli);
    let (mut sr, mut sw) = split(srv);

    let up = copy(&mut cr, &mut sw);
    let down = copy(&mut sr, &mut cw);

    tokio::select! {
        r = up => {
            r.map_err(|e| anyhow!("[{}] Cannot forward to server: {}", cli_addr, e))?;
        },
        r = down => {
            r.map_err(|e| anyhow!("[{}] Cannot forward to client: {}", cli_addr, e))?;
        },
    }

    info!("[{}] Finished forwarding", cli_addr);

    Ok(())
}

pub async fn backend(opt: &Backend) -> Result<()> {
    info!("Starting backend: {:?}", opt);

    let mut listener = TcpListener::bind(&opt.bind).await?;

    loop {
        let (cli, addr) = listener.accept().await?;

        tokio::spawn(async move {
            info!("[{}] Connection accepted", addr);

            if let Err(e) = handle(cli, &addr).await {
                error!("[{}] Error occurred: {}", addr, e);
            }
        });
    }
}

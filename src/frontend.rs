use anyhow::anyhow;
use log::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{copy, split};
use tokio::net::{TcpListener, TcpStream};

// use crate::codec::{Codec, Message};
use crate::error::{Error, Result};
use crate::options::Frontend;
use crate::proto::{write_header, Header};
use crate::util::localhost;

async fn frontend_handle(
    cli: TcpStream,
    cli_addr: &SocketAddr,
    backend_addr: &SocketAddr,
    src_port: u16,
    dst: &SocketAddr,
) -> Result<()> {
    let mut backend = TcpStream::connect(backend_addr).await?;
    info!(
        "[{}: {}->{}] Connected to backend {}",
        cli_addr, src_port, dst, backend_addr,
    );

    write_header(&mut backend, Header::new(dst.clone())).await?;

    let (mut br, mut bw) = split(backend);
    let (mut cr, mut cw) = split(cli);

    let up = copy(&mut cr, &mut bw);
    let down = copy(&mut br, &mut cw);

    tokio::select! {
        r = up => {
            r.map_err(|e| anyhow!("[{}] Cannot forward to backend: {}", cli_addr, e))?;
        },
        r = down => {
            r.map_err(|e| anyhow!("[{}] Cannot forward to client: {}", cli_addr, e))?;
        },
    }

    info!("[{}] Finished forwarding", cli_addr);

    Ok(())
}

async fn start_rule(backend: &SocketAddr, src_port: u16, dst: &SocketAddr) -> Result<()> {
    let mut listener = TcpListener::bind(localhost(src_port)).await?;

    loop {
        let (cli, cli_addr) = listener.accept().await?;

        let backend_addr = backend.clone();
        let dst = dst.clone();

        tokio::spawn(async move {
            info!("[{}: {}->{}] Connection accepted", cli_addr, src_port, dst);

            if let Err(e) = frontend_handle(cli, &cli_addr, &backend_addr, src_port, &dst).await {
                error!(
                    "[{}: {}->{}] Error occurred: {}",
                    cli_addr, src_port, dst, e
                );
            }
        });
    }
}

pub async fn frontend(opt: &Frontend) -> Result<()> {
    let mut handles = vec![];

    for rule in &opt.rules {
        let mut tokens = rule.split(':');
        let src_port = match tokens.next() {
            Some(src) => src
                .parse()
                .map_err(|e| anyhow!("Invalid src port: {}", e))?,
            None => return Err(anyhow!("Invalid tokens")),
        };
        let dst: SocketAddr = match tokens.next() {
            Some(dst) => {
                let port = dst
                    .parse()
                    .map_err(|e| Error::RuleError(format!("Invalid dst port: {}", e)))?;
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
            }
            None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), src_port),
        };

        info!("[{}->{}] Starting redirection", src_port, dst);

        let backend = opt.backend.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = start_rule(&backend, src_port, &dst).await {
                error!("[{}->{}] Error occurred: {}", src_port, dst, e);
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}

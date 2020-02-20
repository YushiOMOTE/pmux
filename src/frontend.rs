use anyhow::anyhow;
use log::*;
use std::net::SocketAddr;
use tokio::io::{copy, split};
use tokio::net::{TcpListener, TcpStream};

use crate::error::Result;
use crate::options::Frontend;
use crate::proto::{write_header, Header};
use crate::util::localhost;

async fn frontend_handle(
    cli: TcpStream,
    cli_addr: &SocketAddr,
    backend_addr: &SocketAddr,
    src_port: u16,
    dst: &str,
) -> Result<()> {
    let mut backend = TcpStream::connect(backend_addr).await?;
    info!(
        "[{}: {}->{}] Connected to backend {}",
        cli_addr, src_port, dst, backend_addr,
    );

    write_header(&mut backend, Header::new(dst.into())).await?;

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

async fn start_rule(backend: &SocketAddr, src_port: u16, dst: &str) -> Result<()> {
    let mut listener = TcpListener::bind(localhost(src_port)).await?;

    loop {
        let (cli, cli_addr) = listener.accept().await?;

        let backend_addr = backend.clone();
        let dst = dst.to_string();

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

fn parse_rule(rule: &str) -> Result<(u16, String)> {
    let mut tokens = rule.splitn(2, ':');

    let src_port = match tokens.next() {
        Some(src) => src
            .parse()
            .map_err(|e| anyhow!("Invalid src port: {}", e))?,
        None => return Err(anyhow!("Invalid tokens")),
    };

    let dst = match tokens.next() {
        Some(dst) => {
            if dst.contains(':') {
                dst.to_owned()
            } else {
                let dst_port: u16 = dst
                    .parse()
                    .map_err(|e| anyhow!("Invalid dst port: {}", e))?;
                format!("127.0.0.1:{}", dst_port)
            }
        }
        None => format!("127.0.0.1:{}", src_port),
    };

    Ok((src_port, dst))
}

pub async fn frontend(opt: &Frontend) -> Result<()> {
    let mut handles = vec![];

    for rule in &opt.rules {
        let (src_port, dst) = parse_rule(&rule)?;

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

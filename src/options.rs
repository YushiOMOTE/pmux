use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Cmd,
}

#[derive(StructOpt, Debug)]
pub struct Frontend {
    /// Port mapping rules
    #[structopt(short = "r", long = "rule")]
    pub rules: Vec<String>,
    /// Backend address
    #[structopt(name = "backend")]
    pub backend: SocketAddr,
}

#[derive(StructOpt, Debug)]
pub struct Backend {
    /// Address to bind
    #[structopt(name = "bind")]
    pub bind: SocketAddr,
}

#[derive(StructOpt, Debug)]
pub enum Cmd {
    /// Run as frontend
    Frontend(Frontend),
    /// Run as backend
    Backend(Backend),
}

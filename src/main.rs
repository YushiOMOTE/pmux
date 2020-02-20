use log::*;
use structopt::StructOpt;

mod backend;
mod error;
mod frontend;
mod options;
mod proto;
mod util;

use crate::backend::backend;
use crate::error::Result;
use crate::frontend::frontend;
use crate::options::{Cmd, Opt};

async fn run(opt: &Opt) -> Result<()> {
    match &opt.cmd {
        Cmd::Frontend(o) => frontend(&o).await,
        Cmd::Backend(o) => backend(&o).await,
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + 'static>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();

    if let Err(e) = run(&opt).await {
        error!("{}", e);
    }

    Ok(())
}

use structopt::StructOpt;

mod codec;
mod error;
mod util;
mod options;
mod backend;
mod frontend;

use crate::options::{Opt, Cmd};
use crate::error::Result;
use crate::backend::backend;
use crate::frontend::frontend;

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

    run(&opt).await.map_err(|e| Box::new(e))?;

    Ok(())
}

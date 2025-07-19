use std::path::Path;
use std::net::SocketAddr;

use clap::Parser;

mod logger;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    address: SocketAddr,
}

fn setup_logger<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>
{
    let logger = logger::Logger::from_path(path)?;
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .map_err(|err| err.into())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logger("bb.log")?;
    bb_server::main(args.address).await
}

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use clap::Parser;
use tokio::runtime::Builder;

mod logger;

#[derive(Parser, Debug)]
struct Args {
    /// the ip address and port the server should run on
    /// in a format of `<address>:<port>`
    #[arg(short, long)]
    address: SocketAddr,

    /// define the amount of worker threads that will be used
    /// to handle different connection and tasks
    #[arg(short, long, default_value_t = 8)]
    threads: usize,

    /// configure the path to the server log file
    /// by default use the CWD/bb.log
    #[arg(long)]
    log: Option<PathBuf>,
}

fn setup_logger<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let logger = logger::Logger::from_path(path)?;
    log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(log::LevelFilter::Info))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logger(args.log.unwrap_or("threet.log".into()))?;

    let runtime = Builder::new_multi_thread()
        .worker_threads(args.threads)
        .thread_name("3T-tokio-worker-thread")
        // lower the event interval just a tad because chatting users
        // may not always be active, so it is better to be responsive more to the
        // active users
        .event_interval(40)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(threet_server::main(args.address))
}

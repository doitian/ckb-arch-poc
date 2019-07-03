mod args;
mod chain;
mod debug_console;

use args::Args;
use chain::ChainService;
use debug_console::DebugConsole;
use futures::future::Future;

fn main() {
    if app().is_err() {
        std::process::exit(113);
    }
}

fn app() -> Result<(), ExidCode> {
    pretty_env_logger::init_timed();

    let args = Args::parse()?;

    let (chain, chain_service) = ChainService::spawn();
    let debug_console = DebugConsole::bind(&args.bind)?;

    chain.info().wait().expect("get info");

    tokio::run(chain_service.join(debug_console).map(|_| ()));

    Ok(())
}

struct ExidCode;

impl From<std::io::Error> for ExidCode {
    fn from(e: std::io::Error) -> Self {
        eprintln!("error: {}", e);
        ExidCode
    }
}

impl From<()> for ExidCode {
    fn from(_: ()) -> Self {
        ExidCode
    }
}

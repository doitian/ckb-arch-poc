mod args;
mod debug_console;

use args::Args;
use debug_console::DebugConsole;
use log::{error, info};
use tokio::prelude::*;

fn main() {
    if app().is_err() {
        std::process::exit(113);
    }
}

fn app() -> Result<(), ExidCode> {
    pretty_env_logger::init_timed();

    let args = Args::parse()?;
    let debug_console = DebugConsole::bind(&args.bind)?;

    tokio::run(debug_console.for_each(|cmd| {
        info!(target: "ckb_arch_poc::debug_console", "{}", cmd);
        tokio::spawn(
            cmd.reply(cmd.body().to_string())
                .map(|_| ())
                .map_err(|err| error!("reply error: {}", err)),
        );
        Ok(())
    }));
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

mod args;
mod debug_console;

use args::Args;
use debug_console::DebugConsole;

fn main() {
    if app().is_err() {
        std::process::exit(113);
    }
}

fn app() -> Result<(), ExidCode> {
    pretty_env_logger::init_timed();

    let args = Args::parse()?;
    tokio::run(DebugConsole::bind(&args.bind)?);

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

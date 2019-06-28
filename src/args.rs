use std::net::SocketAddr;

pub struct Args {
    pub bind: SocketAddr,
}

impl Args {
    pub fn parse() -> Result<Args, ()> {
        let mut args = std::env::args().skip(1);

        let bind = args
            .next()
            .unwrap_or("127.0.0.1:12345".to_string())
            .parse()
            .map_err(|err|
                eprintln!("invalid bind address: {}", err)
            )?;

        if args.next().is_none() {
            Ok(Args { bind })
        } else {
            eprintln!("usage: ckb-arch-poc [bind]");
            Err(())
        }
    }
}

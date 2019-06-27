use log::{error, info};
use tokio::io::copy;
use tokio::net::TcpListener;
use tokio::prelude::*;

fn main() {
    pretty_env_logger::init_timed();

    let mut args = std::env::args().skip(1);

    let addr = args
        .next()
        .unwrap_or("127.0.0.1:12345".to_string())
        .parse()
        .expect("valid bind address");

    if args.next().is_some() {
        eprintln!("ckb-arch-poc [bind]");
        std::process::exit(127);
    }

    info!("listen on {}", addr);
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    let server = listener
        .incoming()
        .map_err(|e| error!("accept failed = {:?}", e))
        .for_each(|sock| {
            // Split up the reading and writing parts of the
            // socket.
            let (reader, writer) = sock.split();

            // A future that echos the data and returns how
            // many bytes were copied...
            let bytes_copied = copy(reader, writer);

            let inspect = bytes_copied
                .map(|amount| info!("wrote {:} bytes", amount.0))
                .map_err(|err| error!("IO error {:?}", err));

            tokio::spawn(inspect)
        });

    tokio::run(server);
}

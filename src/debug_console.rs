use log::error;
use std::io;
use std::net::SocketAddr;
use tokio::{
    codec::{Decoder, LinesCodec},
    net::{tcp::Incoming, TcpListener},
    prelude::*,
};

pub struct DebugConsole {
    incoming: Incoming,
}

impl DebugConsole {
    pub fn bind(addr: &SocketAddr) -> io::Result<DebugConsole> {
        let listener = TcpListener::bind(&addr)?;

        Ok(DebugConsole {
            incoming: listener.incoming(),
        })
    }

    pub fn run(self) -> impl Future<Item = (), Error = io::Error> {
        self.incoming.for_each(|socket| {
            let (tx, rx) = LinesCodec::new().framed(socket).split();
            tokio::spawn(
                tx.send_all(rx)
                    .map(|_| ())
                    .map_err(|err| error!("send all: {}", err)),
            );
            Ok(())
        })
    }
}

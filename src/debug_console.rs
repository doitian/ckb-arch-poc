use log::error;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::{
    codec::{Decoder, LinesCodec},
    net::{tcp::Incoming, TcpListener},
    prelude::*,
    sync::mpsc,
};

pub struct DebugConsole {
    incoming: Incoming,
    commands_tx: mpsc::Sender<String>,
    commands_rx: mpsc::Receiver<String>,
}

const CHANNEL_BOUND: usize = 64;

impl DebugConsole {
    pub fn bind(addr: &SocketAddr) -> io::Result<DebugConsole> {
        let listener = TcpListener::bind(&addr)?;
        let (commands_tx, commands_rx) = mpsc::channel(CHANNEL_BOUND);

        Ok(DebugConsole {
            incoming: listener.incoming(),
            commands_tx,
            commands_rx,
        })
    }
}

impl DebugConsole {
    fn on_connection(&mut self, socket: TcpStream) {
        let tx = self
            .commands_tx
            .clone()
            .sink_map_err(|err| error!("commands tx error: {}", err));
        let lines = LinesCodec::new()
            .framed(socket)
            .map_err(|err| error!("read command error: {}", err));

        tokio::spawn(tx.send_all(lines).map(|_| ()));
    }
}

impl Stream for DebugConsole {
    type Item = String;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        loop {
            match self
                .incoming
                .poll()
                .map_err(|err| error!("accept new connection error: {}", err))?
            {
                Async::Ready(Some(socket)) => self.on_connection(socket),
                _ => break,
            }
        }

        self.commands_rx
            .poll()
            .map_err(|err| error!("commands rx error: {}", err))
    }
}

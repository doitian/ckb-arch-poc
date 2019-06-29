use log::error;
use std::io;
use std::net::SocketAddr;
use tokio::{
    codec::{Decoder, LinesCodec},
    net::{tcp::Incoming, TcpListener, TcpStream},
    prelude::*,
    sync::mpsc,
};
use std::fmt;

pub struct DebugConsole {
    incoming: Incoming,
    requests_tx: mpsc::Sender<DebugConsoleRequest>,
    requests_rx: mpsc::Receiver<DebugConsoleRequest>,
}

pub struct DebugConsoleRequest {
    body: String,
    responses_tx: mpsc::Sender<String>,
}

const CHANNEL_BOUND: usize = 64;

impl DebugConsole {
    pub fn bind(addr: &SocketAddr) -> io::Result<DebugConsole> {
        let listener = TcpListener::bind(&addr)?;
        let (requests_tx, requests_rx) = mpsc::channel(CHANNEL_BOUND);

        Ok(DebugConsole {
            incoming: listener.incoming(),
            requests_tx,
            requests_rx,
        })
    }
}

impl DebugConsole {
    fn on_connection(&mut self, socket: TcpStream) {
        let (responses_tx, responses_rx) = mpsc::channel(CHANNEL_BOUND);
        let (socket_tx, socket_rx) = LinesCodec::new().framed(socket).split();

        let requests_tx = self
            .requests_tx
            .clone()
            .sink_map_err(|err| error!("requests tx error: {}", err));
        let responses_rx = responses_rx.map_err(|err| error!("responses rx error: {}", err));
        let socket_tx = socket_tx.sink_map_err(|err| error!("send response error: {}", err));
        let socket_rx = socket_rx.map(move |body| DebugConsoleRequest {
            body,
            responses_tx: responses_tx.clone(),
        }).map_err(|err| error!("receive request error: {}", err));

        tokio::spawn(requests_tx.send_all(socket_rx).map(|_| ()));
        tokio::spawn(socket_tx.send_all(responses_rx).map(|_| ()));
    }
}

impl Stream for DebugConsole {
    type Item = DebugConsoleRequest;
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

        self.requests_rx
            .poll()
            .map_err(|err| error!("requests rx error: {}", err))
    }
}

impl DebugConsoleRequest {
    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn reply(&self, response: String) {
        tokio::spawn(self.responses_tx.clone().send(response).map(|_| ()).map_err(|err| error!("reply error: {}", err)));
    }
}

impl fmt::Display for DebugConsoleRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_tuple("DebugConsoleRequest")
            .field(&self.body)
            .finish()
    }
}


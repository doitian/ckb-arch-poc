use std::io;
use std::net::SocketAddr;
use tokio::{
    codec::Framed,
    codec::{Decoder, LinesCodec},
    net::{tcp::Incoming, TcpListener, TcpStream},
    prelude::*,
};

pub struct DebugConsole {
    incoming: Incoming,
}

pub type DebugConsolePeer = Framed<TcpStream, LinesCodec>;

impl DebugConsole {
    pub fn bind(addr: &SocketAddr) -> io::Result<DebugConsole> {
        let listener = TcpListener::bind(&addr)?;

        Ok(DebugConsole {
            incoming: listener.incoming(),
        })
    }
}

impl Stream for DebugConsole {
    type Item = DebugConsolePeer;
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.incoming.poll()? {
            Async::Ready(Some(socket)) => Ok(Async::Ready(Some(LinesCodec::new().framed(socket)))),
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

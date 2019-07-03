use log::error;
use std::io;
use std::net::SocketAddr;
use tokio::{
    codec::{Decoder, LinesCodec},
    net::{tcp::Incoming, TcpListener, TcpStream},
    prelude::*,
    sync::mpsc,
};

pub struct DebugConsole {
    incoming: Incoming,
    requests_queue_sender: mpsc::Sender<Request>,
    requests_queue_receiver: mpsc::Receiver<Request>,
}

struct Request {
    body: String,
    responses_queue_sender: mpsc::Sender<String>,
}

const CHANNEL_BOUND: usize = 64;

impl DebugConsole {
    pub fn bind(addr: &SocketAddr) -> io::Result<DebugConsole> {
        let listener = TcpListener::bind(&addr)?;
        let (requests_queue_sender, requests_queue_receiver) = mpsc::channel(CHANNEL_BOUND);

        Ok(DebugConsole {
            incoming: listener.incoming(),
            requests_queue_sender,
            requests_queue_receiver,
        })
    }

    fn on_connection(&self, socket: TcpStream, requests_queue_sender: mpsc::Sender<Request>) {
        let (responses_queue_sender, responses_queue_receiver) = mpsc::channel(CHANNEL_BOUND);
        let (socket_sender, socket_receiver) = LinesCodec::new().framed(socket).split();

        tokio::spawn(
            socket_receiver
                .map(move |body| {
                    let responses_queue_sender = responses_queue_sender.clone();
                    Request {
                        body,
                        responses_queue_sender,
                    }
                })
                .map_err(|err| error!("socket receiver error: {}", err))
                .forward(
                    requests_queue_sender
                        .sink_map_err(|err| error!("requests queue sender error: {}", err)),
                )
                .map(|_| ()),
        );

        tokio::spawn(
            responses_queue_receiver
                .map_err(|err| error!("responses queue receiver error: {}", err))
                .forward(socket_sender.sink_map_err(|err| error!("socket sender error: {}", err)))
                .map(|_| ()),
        );
    }

    fn on_request(&self, request: Request) {
        tokio::spawn(
            request
                .responses_queue_sender
                .clone()
                .send(request.body)
                .map_err(|err| error!("responses queue sender error: {}", err))
                .map(|_| ()),
        );
    }
}

impl Future for DebugConsole {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            match self.incoming.poll() {
                Ok(Async::Ready(Some(socket))) => {
                    self.on_connection(socket, self.requests_queue_sender.clone())
                }
                Err(err) => error!("accept error: {}", err),
                _ => break,
            }
        }

        loop {
            match self.requests_queue_receiver.poll() {
                Ok(Async::Ready(Some(request))) => self.on_request(request),
                Ok(Async::Ready(None)) => return Ok(Async::Ready(())),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => error!("requests queue receiver error: {}", err),
            }
        }
    }
}

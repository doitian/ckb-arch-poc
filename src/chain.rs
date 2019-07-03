use log::{error, trace};
use tokio::{
    prelude::*,
    sync::mpsc::{channel, error::SendError, Receiver, Sender},
};

const CHANNEL_BOUND: usize = 1024;

#[derive(Debug)]
pub enum ChainCommand {
    Info,
}

pub struct ChainService {
    commands_queue_receiver: Receiver<ChainCommand>,
}

#[derive(Clone)]
pub struct ChainHandle {
    commands_queue_sender: Sender<ChainCommand>,
}

impl ChainService {
    pub fn spawn() -> (ChainHandle, ChainService) {
        let (commands_queue_sender, commands_queue_receiver) = channel(CHANNEL_BOUND);

        (
            ChainHandle {
                commands_queue_sender,
            },
            ChainService {
                commands_queue_receiver,
            },
        )
    }

    fn on_command(&mut self, command: ChainCommand) -> Option<Result<Async<()>, ()>> {
        trace!("receive command: {:?}", command);

        None
    }
}

impl ChainHandle {
    pub fn info(self) -> impl Future<Item = Self, Error = SendError> {
        self.commands_queue_sender
            .send(ChainCommand::Info)
            .map(|sender| ChainHandle {
                commands_queue_sender: sender,
            })
    }
}

impl Future for ChainService {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            match self.commands_queue_receiver.poll() {
                Ok(Async::Ready(Some(command))) => {
                    if let Some(result) = self.on_command(command) {
                        return result;
                    }
                }
                Ok(Async::Ready(None)) => return Ok(Async::Ready(())),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => {
                    error!("commands queue receiver error: {}", err);
                    return Err(());
                }
            }
        }
    }
}

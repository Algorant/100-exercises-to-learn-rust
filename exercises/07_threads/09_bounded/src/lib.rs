// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

impl TicketStoreClient {
    pub fn insert(
        &self,
        draft: TicketDraft,
    ) -> Result<TicketId, Box<dyn std::error::Error + Send>> {
        let (response_sender, response_receiver) = sync_channel(1);

        let command = Command::Insert {
            draft,
            response_channel: response_sender,
        };

        self.sender
            .send(command)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        response_receiver
            .recv()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, Box<dyn std::error::Error + Send>> {
        let (response_sender, response_receiver) = sync_channel(1);

        let command = Command::Get {
            id,
            response_channel: response_sender,
        };

        self.sender
            .send(command)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        response_receiver
            .recv()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<Result<TicketId, Box<dyn std::error::Error + Send>>>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Result<Option<Ticket>, Box<dyn std::error::Error + Send>>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                let _ = response_channel.send(Ok(id));
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id).cloned();
                let _ = response_channel.send(Ok(ticket));
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}

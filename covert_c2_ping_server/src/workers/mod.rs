use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use self::ping::Transaction;

pub mod ping;
pub mod session;
pub mod web;

pub type WorkerHandles = (
    UnboundedSender<Transaction>,
    UnboundedReceiver<Transaction>,
    JoinHandle<()>,
    JoinHandle<()>,
);

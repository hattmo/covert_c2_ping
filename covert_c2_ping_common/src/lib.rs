use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Deserialized from multiple covert packets
#[derive(Serialize, Deserialize, Debug)]
pub enum PingMessage {
    DataMessage(Vec<u8>),
    SleepMessage(Duration),
    InitMessage(Arch, String),
    CloseMessage
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Arch {
    i686,
    X86_64,
}

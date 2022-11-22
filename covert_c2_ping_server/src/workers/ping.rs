use anyhow::{anyhow, bail, Result};
use nfq::{Queue, Verdict};
use pnet_datalink::{
    linux::{self, Config as LinuxConfig},
    Channel::Ethernet as EthernetChan,
    MacAddr, NetworkInterface,
};
use pnet_packet::{
    ethernet::{EtherTypes, MutableEthernetPacket},
    icmp::{
        echo_reply::{IcmpCodes, MutableEchoReplyPacket},
        echo_request::EchoRequestPacket,
        IcmpTypes,
    },
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    Packet,
};

use std::{fmt::Display, net::Ipv4Addr};
use tokio::{
    sync::mpsc::{self},
    task::{self},
};

use crate::GLOBAL_CONF;

use super::WorkerHandles;

#[derive(Debug)]
pub struct Transaction {
    pub src_mac: [u8; 6],
    pub dst_mac: [u8; 6],
    pub src_ip: Ipv4Addr,
    pub dst_ip: Ipv4Addr,
    pub sequence_number: u16,
    pub identifier: u16,
    pub data: Vec<u8>,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PingTransaction -- IP: {} -> {} , MAC: {:?} -> {:?} Sqn: {} Data: {:?}",
            self.src_ip, self.dst_ip, self.src_mac, self.dst_mac, self.sequence_number, self.data
        )
    }
}

pub fn start_workers() -> Result<WorkerHandles> {
    let interfaces = linux::interfaces();
    let interface = interfaces
        .iter()
        .find(|i| i.name == GLOBAL_CONF.i)
        .expect("Interface not found")
        .clone();
    let mac = interface
        .mac
        .ok_or_else(|| anyhow!("No mac addr on interface."))?
        .octets();

    let (upstream_sender, upstream_receiver) = mpsc::unbounded_channel::<Transaction>();
    let (downstream_sender, downstream_receiver) = mpsc::unbounded_channel::<Transaction>();

    let ping_recv_h = task::spawn_blocking(move || {
        if let Err(err) = ping_recv(upstream_sender, interface.index, mac) {
            tracing::info!(error = ?err, "Receive job ended");
        };
    });
    let ping_send_h = task::spawn_blocking(move || {
        if let Err(err) = ping_send(downstream_receiver, interface) {
            tracing::info!(error = ?err, "Send job ended");
        };
    });
    Ok((
        downstream_sender,
        upstream_receiver,
        ping_recv_h,
        ping_send_h,
    ))
}

#[tracing::instrument(skip_all)]
fn ping_recv(
    upstream_sender: mpsc::UnboundedSender<Transaction>,
    iface_num: u32,
    dst_mac: [u8; 6],
) -> Result<()> {
    let mut q = Queue::open()?;
    q.bind(42)?;
    tracing::info!("Listening on queue");

    loop {
        let data = match recv_iteration(&mut q, iface_num, dst_mac) {
            Err(IterationError::Recoverable(reason)) => {
                tracing::info!("{}", reason);
                continue;
            }
            Err(IterationError::Unrecoverable(reason)) => {
                tracing::info!("{}", reason);
                break;
            }
            Ok(data) => data,
        };
        if upstream_sender.send(data).is_err() {
            tracing::info!("Upstream channel closed stopping recv task");
            break;
        };
    }
    Ok(())
}
enum IterationError {
    Recoverable(String),
    Unrecoverable(String),
}
#[tracing::instrument(skip_all)]
fn recv_iteration(
    q: &mut Queue,
    iface_num: u32,
    dst_mac: [u8; 6],
) -> Result<Transaction, IterationError> {
    tracing::info!("Waiting for packet");
    let mut msq = q.recv().or(Err(IterationError::Unrecoverable(
        "Net Filter Queue closed".to_owned(),
    )))?;
    let payload = msq.get_payload().to_vec();
    if iface_num != msq.get_indev() {
        return Err(IterationError::Recoverable("Wrong interface".to_owned()));
    }
    let addr = msq
        .get_hw_addr()
        .ok_or_else(|| IterationError::Recoverable("No mac addr on packet".to_owned()))?
        .to_vec();
    if addr.len() != 6 {
        return Err(IterationError::Recoverable(
            "Wrong size of mac addr".to_owned(),
        ));
    };
    tracing::info!(
        "Packet received ip_packet={:x?} mac_addr={:x?}",
        payload,
        addr
    );
    let ip_packet = Ipv4Packet::new(&payload)
        .ok_or_else(|| IterationError::Recoverable("Malformed Ip packet".to_owned()))?;
    let icmp_packet = EchoRequestPacket::new(ip_packet.payload())
        .ok_or_else(|| IterationError::Recoverable("Malformed Icmp packet".to_owned()))?;
    tracing::info!(payload=?icmp_packet.payload());
    msq.set_verdict(Verdict::Drop);
    q.verdict(msq)
        .map_err(|_| IterationError::Unrecoverable("Net Filter Queue closed".to_owned()))?;

    Ok(Transaction {
        src_mac: addr.try_into().expect("Should be the right size"),
        dst_mac,
        src_ip: ip_packet.get_source(),
        dst_ip: ip_packet.get_destination(),
        sequence_number: icmp_packet.get_sequence_number(),
        identifier: icmp_packet.get_identifier(),
        data: icmp_packet.payload().to_vec(),
    })
}

#[tracing::instrument(skip_all)]
fn ping_send(
    mut downstream_reciever: mpsc::UnboundedReceiver<Transaction>,
    iface: NetworkInterface,
) -> Result<()> {
    match linux::channel(&iface, LinuxConfig::default()) {
        Ok(EthernetChan(mut sender, _)) => loop {
            tracing::info!("Waiting to send ping");
            let ping = match downstream_reciever.blocking_recv() {
                Some(ping) => ping,
                None => {
                    bail!("Downstream channel closed stopping send task");
                }
            };
            tracing::info!("Got ping message to send");
            match send_iteration(&ping) {
                Ok(data) => {
                    tracing::info!("Sending ping");
                    sender.send_to(&data, Option::None);
                }
                Err(IterationError::Recoverable(reason)) => {
                    tracing::info!("{}", reason);
                    continue;
                }
                Err(IterationError::Unrecoverable(reason)) => {
                    tracing::info!("{}", reason);
                    break;
                }
            };
        },
        Ok(_) => {
            bail!("This device is not an ethernet device");
        }
        Err(err) => {
            bail!(err);
        }
    }
    Ok(())
}

fn send_iteration(ping: &Transaction) -> Result<Vec<u8>, IterationError> {
    let icmp_buf = vec![0u8; MutableEchoReplyPacket::minimum_packet_size() + ping.data.len()];
    let mut icmp_p = MutableEchoReplyPacket::owned(icmp_buf)
        .ok_or_else(|| IterationError::Recoverable("Failed to allocate reply".to_owned()))?;
    icmp_p.set_icmp_type(IcmpTypes::EchoReply);
    icmp_p.set_icmp_code(IcmpCodes::NoCode);
    icmp_p.set_identifier(ping.identifier);
    icmp_p.set_sequence_number(ping.sequence_number);
    icmp_p.set_payload(&ping.data);
    icmp_p.set_checksum(pnet_packet::util::checksum(icmp_p.packet(), 1));

    let icmp_payload = icmp_p.packet();
    let packet_size = MutableIpv4Packet::minimum_packet_size() + icmp_payload.len();
    let ip_buf = vec![0u8; packet_size];
    let mut ip_p = MutableIpv4Packet::owned(ip_buf)
        .ok_or_else(|| IterationError::Recoverable("Failed to allocate reply".to_owned()))?;
    ip_p.set_version(4);
    ip_p.set_header_length(5);
    ip_p.set_dscp(24);
    ip_p.set_ecn(0);
    ip_p.set_total_length(
        packet_size
            .try_into()
            .map_err(|_| IterationError::Recoverable("Packet Size too large".to_owned()))?,
    );
    ip_p.set_identification(0);
    ip_p.set_flags(0);
    ip_p.set_fragment_offset(0);
    ip_p.set_ttl(120);
    ip_p.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip_p.set_source(ping.src_ip);
    ip_p.set_destination(ping.dst_ip);
    ip_p.set_payload(icmp_payload);
    ip_p.set_checksum(ipv4::checksum(&ip_p.to_immutable()));

    let ip_payload = ip_p.packet();
    let eth_buf = vec![0u8; MutableEthernetPacket::minimum_packet_size() + ip_payload.len()];
    let mut eth_p = MutableEthernetPacket::owned(eth_buf)
        .ok_or_else(|| IterationError::Recoverable("Failed to allocate reply".to_owned()))?;
    eth_p.set_source(MacAddr::from(ping.src_mac));
    eth_p.set_destination(MacAddr::from(ping.dst_mac));
    eth_p.set_ethertype(EtherTypes::Ipv4);
    eth_p.set_payload(ip_payload);
    Ok(eth_p.packet().to_vec())
}

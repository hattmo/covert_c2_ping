use covert_c2_ping_common::{PingMessage, SessionData};
use covert_server::{CSFrameRead, CSFrameWrite};

use tokio::{net::TcpStream, select, sync::mpsc, task};

use crate::{CHANNEL, SESSIONS};

pub async fn session_worker(connection: TcpStream, id: u16, arch: String) {
    let (sender, mut receiver) = mpsc::unbounded_channel::<()>();
    SESSIONS
        .lock()
        .await
        .insert(id, (sender, SessionData::new(&arch)));
    let (mut read_tcp, mut write_tcp) = connection.into_split();

    let ts_reader = task::spawn(async move {
        loop {
            match read_tcp.read_frame().await {
                Ok(data) => {
                    CHANNEL
                        .lock()
                        .await
                        .put_message(PingMessage::DataMessage(data), id);
                }
                Err(_) => {
                    tracing::info!("Session closed with TS");
                    break;
                }
            };
        }
    });

    let ts_writer = task::spawn(async move {
        loop {
            if let Some(_) = receiver.recv().await {
                if let Some(mess) = CHANNEL.lock().await.get_message(id) {
                    match mess {
                        PingMessage::DataMessage(data) => {
                            if let Err(_) = write_tcp.write_frame(&data).await {
                                tracing::info!("Session closed with TS");
                                break;
                            };
                        }
                        PingMessage::SleepMessage(_) => {
                            //NOOP
                        }
                        PingMessage::CloseMessage => {
                            tracing::info!("Close message received");
                            break;
                        }
                    }
                }
            } else {
                tracing::info!("Session closed with downstream");
                break;
            }
        }
    });
    select! {
        _ = ts_reader => {}
        _ = ts_writer => {}
    };
    SESSIONS.lock().await.remove(&id);
}

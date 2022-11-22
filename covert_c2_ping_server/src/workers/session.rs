use covert_c2_ping_common::{PingMessage, SessionData};
use covert_server::{CSFrameRead, CSFrameWrite};

use tokio::{net::TcpStream, select, sync::mpsc, task};

use crate::{CHANNEL, SESSIONS};

pub async fn worker(connection: TcpStream, id: u16, arch: String) {
    let (sender, mut receiver) = mpsc::unbounded_channel::<()>();
    SESSIONS
        .lock()
        .await
        .insert(id, (sender, SessionData::new(&arch)));
    let (mut read_tcp, mut write_tcp) = connection.into_split();

    let ts_reader = task::spawn(async move {
        loop {
            if let Ok(data) = read_tcp.read_frame().await {
                CHANNEL
                    .lock()
                    .await
                    .put_message(PingMessage::DataMessage(data), id);
            } else {
                tracing::info!("Session closed with TS");
                break;
            };
        }
    });

    let ts_writer = task::spawn(async move {
        loop {
            if receiver.recv().await.is_some() {
                if let Some(mess) = CHANNEL.lock().await.get_message(id) {
                    match mess {
                        PingMessage::DataMessage(data) => {
                            if write_tcp.write_frame(&data).await.is_err() {
                                tracing::info!("Session closed with TS");
                                break;
                            };
                        }
                        PingMessage::CloseMessage => {
                            tracing::info!("Close message received");
                            break;
                        }
                        _ => {
                            //NOOP
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

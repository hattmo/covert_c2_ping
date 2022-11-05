use crate::edit_agent::EditAgent;
use covert_c2_ping_common::{AgentSessions, DeleteAgent, SessionData};
use gloo::{net::http::Request, timers::callback::Interval};
use itertools::Itertools;
use js_sys::Date;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_effect, use_state, Callback, Html, UseStateHandle};

#[function_component(AgentList)]
pub fn agent_list() -> Html {
    let sessions: UseStateHandle<AgentSessions> = use_state(|| {
        let mut initial = HashMap::new();
        if cfg!(debug_assertions) {
            initial.insert(2, SessionData::new("x86"));
            initial.insert(9, SessionData::new("x86"));
            initial.insert(1, SessionData::new("x86"));
            initial.insert(7, SessionData::new("x86"));
        }
        initial
    });

    let edit_visible: UseStateHandle<Option<u16>> = use_state(|| Some(1));

    let fragments: Html = sessions.iter().sorted_by_key(|(id,_)|**id)
        .map(|(id, data)| {
            let id = id.clone();
            let last_checkin = data
                .last_checkin
                .clone()
                .map(|then|{
                    let mut result = Date::now() - then;
                    result /= 1000.0;
                    result = result.floor();
                    if result < 0.0 {0.0} else {result}
                })
                .and_then(|t| Some(format!("{} sec ago", t)))
                .unwrap_or("Never".to_owned());

            let delete_cb = Callback::from(move |_|{
                spawn_local(async move {
                    Request::delete("/api/agents").json(&DeleteAgent{agentid:id}).unwrap().send().await.unwrap();
                });
            });
            let edit = edit_visible.clone();
            let edit_cb = Callback::from(move |_|{
                edit.set(Some(id));
            });

            html!(<key={id}>
                    <div>{id}</div>
                    <div>{data.arch.clone()}</div>
                    <div>{data.host.clone().map(|v|v.to_string()).unwrap_or("Unknown".to_owned())}</div>
                    <div>{last_checkin}</div>
                    <div><span class="clickable" onclick={delete_cb}>{"🗑️"}</span><span class="clickable" onclick={edit_cb}>{"🛠️"}</span></div>
                  </>)
        })
        .collect();

    use_effect(|| {
        let interval = Interval::new(1000, move || {
            let sessions = sessions.clone();
            spawn_local(async move {
                log::info!("Called interval");
                if let Ok(res) = Request::get("/api/agents").send().await {
                    if let Ok(new_sessions) = res.json::<AgentSessions>().await {
                        sessions.set(new_sessions);
                    }
                }
            });
        });
        return move || {
            drop(interval);
        };
    });

    let editor = if edit_visible.is_some() {
        html!(<EditAgent id = {edit_visible}/>)
    } else {
        html!(<></>)
    };

    html!(<><div class="agent_list">
            <div>{"ID"}</div>
            <div>{"Arch"}</div>
            <div>{"From"}</div>
            <div>{"Last Check-In"}</div>
            <div/>
            {fragments}
          </div>{editor}</>)
}

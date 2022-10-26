use covert_c2_ping_common::{AgentSessions, SessionData};
use js_sys::Date;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, use_state, Html, UseStateHandle};

#[function_component(AgentList)]
pub fn agent_list() -> Html {
    let sessions: UseStateHandle<AgentSessions> = use_state(|| {
        let mut initial = HashMap::new();
        if cfg!(debug_assertions) {
            initial.insert(1, SessionData::new("x86"));
        }
        initial
    });

    let mut temp: Vec<_> = sessions.iter().collect();
    temp.sort_by_key(|(id, _)| **id);

    let fragments: Html = temp
        .iter()
        .map(|(id, data)| {
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
            html!(<key={**id}>
                    <div>{id}</div>
                    <div>{data.arch.clone()}</div>
                    <div>{data.host.clone().map(|v|v.to_string()).unwrap_or("Unknown".to_owned())}</div>
                    <div>{last_checkin}</div>
                    <div><span>{"🗑️"}</span><span>{"🛠️"}</span></div>
                  </>)
        })
        .collect();

    let _timer = use_state(|| {
        gloo::timers::callback::Interval::new(1000, move || {
            let sessions = sessions.clone();
            spawn_local(async move {
                if let Ok(res) = gloo::net::http::Request::get("/api/agents").send().await {
                    if let Ok(new_sessions) = res.json::<AgentSessions>().await {
                        sessions.clone().set(new_sessions);
                    }
                }
            });
        })
    });

    html!(<div class="agent_list">
            <div>{"ID"}</div>
            <div>{"Arch"}</div>
            <div>{"Host"}</div>
            <div>{"Last Check-In"}</div>
            <div/>
            {fragments}
          </div>)
}

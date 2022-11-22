use covert_c2_ping_common::NewAgent;
use gloo::{
    file::{File, ObjectUrl},
    net::http::Request,
    utils::document,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlAnchorElement, HtmlInputElement};
use yew::{events::Event, function_component, prelude::*, use_state, Callback};

#[function_component(NewAgentForm)]
pub fn new_agent() -> Html {
    let is_64 = use_state(|| true);
    let arch_change = {
        let is_64 = is_64.clone();
        Callback::from(move |e: Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .map_or(true, |ele| ele.value() == "x86_64");
            is_64.set(val);
        })
    };

    let host = use_state(|| "localhost".to_owned());
    let host_change = {
        let host = host.clone();
        Callback::from(move |e: Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .map(|ele| ele.value())
                .unwrap_or_default();
            host.set(val);
        })
    };

    let pipe = use_state(|| "my_pipe".to_owned());
    let pipe_change = {
        let pipe = pipe.clone();
        Callback::from(move |e: Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .map(|ele| ele.value())
                .unwrap_or_default();
            pipe.set(val);
        })
    };
    let sleep = use_state(|| 2u64);
    let sleep_change = {
        let sleep = sleep.clone();
        Callback::from(move |e: Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .and_then(|ele| ele.value().parse::<u64>().ok())
                .unwrap_or(2u64);
            sleep.set(val);
        })
    };
    let new_agent = NewAgent {
        host: host.to_string(),
        pipe: pipe.to_string(),
        sleep: *sleep,
        arch: if *is_64 {
            "x64".to_owned()
        } else {
            "x86".to_owned()
        },
    };
    let generate_payload = {
        Callback::from(move |_| {
            let new_agent = new_agent.clone();
            spawn_local(async move {
                log::warn!("Sending");
                let res = Request::post("/api/agents")
                    .json(&new_agent)
                    .unwrap()
                    .send()
                    .await
                    .unwrap();
                if !res.ok() {
                    return;
                }
                let res_payload = res.binary().await.unwrap();
                let file = File::new("Payload.exe", res_payload.as_slice());
                let a: HtmlAnchorElement = document().create_element("a").unwrap().unchecked_into();

                let url = ObjectUrl::from(file);
                a.set_href(&url);
                a.set_download("Payload.exe");
                a.click();
                log::warn!("Sent");
            });
        })
    };

    html! {
    <div class="new_agent_form">
        <label for="host">{"Arch"}</label>
        <div onchange={arch_change}>
            <input checked={*is_64} type="radio" name="arch" value="x86_64" id="x86_64"/>
            <label for="x86_64">{"x86_64"}</label>
            <input checked={!*is_64} type="radio" name="arch" value="i686" id="i686"/>
            <label for="i686">{"i686"}</label>
        </div>

        <label for="host">{"Host"}</label>
        <input onchange={host_change} id="host" type="text" value={(*host).clone()}/>
        <label for="pipe">{"Pipe"}</label>
        <input onchange={pipe_change} id="pipe" type="text" value={(*pipe).clone()}/>
        <label for="sleep">{"Sleep (sec)"}</label>
        <input onchange={sleep_change} id="sleep" type="number" value={sleep.to_string()}/>
        <input class="new_agent_submit" type="submit" value="Create" onclick={generate_payload}/>
    </div>
    }
}

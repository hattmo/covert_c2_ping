use covert_c2_ping_common::PatchAgent;
use gloo::net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, MouseEvent};
use yew::{
    function_component, html, html::onchange, use_state, Callback, Properties, UseStateHandle,
};

#[derive(Properties, PartialEq)]
pub struct EditAgentProps {
    pub id: UseStateHandle<Option<u16>>,
}

#[function_component(EditAgent)]
pub fn edit_agent(props: &EditAgentProps) -> Html {
    let id = props.id.unwrap();
    let edit = props.id.clone();
    let close = {
        let edit = edit.clone();
        Callback::from(move |_| {
            edit.set(None);
        })
    };
    let do_nothing = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
    });
    let sleep = use_state(|| 2u64);
    let sleep_change = {
        let sleep = sleep.clone();
        Callback::from(move |e: onchange::Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .and_then(|ele| ele.value().parse::<u64>().ok())
                .unwrap_or(2u64);
            sleep.set(val);
        })
    };

    let on_submit = {
        let edit = edit.clone();
        let sleep = *sleep.clone();
        Callback::from(move |_| {
            edit.set(None);
            spawn_local(async move {
                Request::patch("/api/agents")
                    .json(&PatchAgent {
                        agentid: id,
                        sleep: Some(sleep),
                    })
                    .unwrap()
                    .send()
                    .await.unwrap();
            });
        })
    };
    html!(<div class="background" onclick={close}><div class="modal" onclick={do_nothing}><label for="sleep">{"sleep:"}</label><input onchange={sleep_change} id="sleep" type="number" value={sleep.to_string()}/><input type="submit" value="edit" onclick={on_submit}/></div></div>)
}

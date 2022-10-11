use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{events::Event, prelude::*, Callback};

#[function_component(Root)]
fn root() -> Html {
    let is_64 = use_state(|| true);
    let arch_change = {
        let is_64 = is_64.clone();
        Callback::from(move |e: Event| {
            let val = e
                .target()
                .and_then(|v| v.dyn_into::<HtmlInputElement>().ok())
                .and_then(|ele| Some(ele.value() == "x86_64"))
                .unwrap_or(true);
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
                .and_then(|ele| Some(ele.value()))
                .unwrap_or(String::default());
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
                .and_then(|ele| Some(ele.value()))
                .unwrap_or(String::default());
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
    let generate_payload = {
        Callback::from(move |_| {
            spawn_local(async {
                let out = Foo {
                    bar: "Hi".to_owned(),
                    foo: 4,
                };
                let blah = JsValue::from_serde(&out).unwrap();
                let res = gloo::net::http::Request::post("/api/agents")
                    .body(blah)
                    .send()
                    .await
                    .unwrap();
            })
        })
    };

    html! {
    <div>
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
        <label for="sleep">{"Sleep"}</label>
        <input onchange={sleep_change} id="sleep" type="number" value={sleep.to_string()}/>
        <input type="submit" value="Create" onclick={generate_payload}/>
    </div>
    }
}

#[derive(Serialize)]
struct Foo {
    foo: u8,
    bar: String,
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Root>();
}

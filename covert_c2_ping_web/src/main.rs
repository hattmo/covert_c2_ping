#![deny(clippy::all, clippy::pedantic)]
mod agent_list;
mod edit_agent;
mod new_agent;
use agent_list::AgentList;
use new_agent::NewAgentForm;
use yew::{function_component, html};

#[function_component(Root)]
fn root() -> Html {
    html! {
    <div>
        <AgentList/>
        <NewAgentForm/>
    </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Root>();
}

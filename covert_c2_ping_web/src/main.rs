use yew::prelude::*;

#[function_component(Root)]
fn root() -> Html {
    html! {"Hello world"}
}

fn main() {
    yew::start_app::<Root>();
}

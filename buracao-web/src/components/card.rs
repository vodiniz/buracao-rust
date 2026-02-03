use crate::utils::assets::get_card_path;
use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(into)] id: String,

    #[prop(optional, into)] width: String,

    // Removemos 'optional' para evitar ambiguidade. O Signal<Option> é claro.
    #[prop(into)] selection_group: Signal<Option<usize>>,

    // IMPORTANTE: 'default = None' obriga o tipo a ser Option<Callback>.
    #[prop(default = None)] on_click: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let is_selected = move || selection_group.get().is_some();

    let path = get_card_path(&id);
    let width_style = if width.is_empty() {
        "auto".to_string()
    } else {
        width
    };

    view! {
        <img
            src=path
            style:width=width_style
            style:border=move || if is_selected() { "3px solid yellow" } else { "none" }
            style:border-radius="8px"
            style:margin-top=move || if is_selected() { "-20px" } else { "0" }
            style:transition="all 0.2s ease"
            style:cursor="pointer"
            on:click=move |e| {
                if let Some(cb) = on_click {
                    cb.run(e);
                }
            }
        />
    }
}

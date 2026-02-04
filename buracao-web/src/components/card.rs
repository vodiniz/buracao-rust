use crate::utils::assets::get_card_path;
use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(into)] id: String,

    // CORREÇÃO: Substituído MaybeSignal por Signal
    #[prop(into, default = "auto".to_string().into())] width: Signal<String>,

    #[prop(into, default = "PaperCards1.1".to_string())] theme: String,
    #[prop(into)] selection_group: Signal<Option<usize>>,
    #[prop(default = None)] on_click: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let is_selected = move || selection_group.get().is_some();

    let path = move || get_card_path(&id, &theme);

    view! {
        <img
            src=path
            // Signal também usa .get()
            style:width=move || width.get()
            style:border=move || if is_selected() { "3px solid yellow" } else { "none" }
            style:border-radius="6px"
            style:transform=move || if is_selected() { "translateY(-20px)" } else { "translateY(0)" }
            style:transition="transform 0.2s ease, margin 0.2s ease"
            style:cursor="pointer"
            on:click=move |e| {
                if let Some(cb) = on_click {
                    cb.run(e);
                }
            }
        />
    }
}

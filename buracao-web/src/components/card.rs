use crate::utils::assets::get_card_path;
use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(into)] id: Signal<String>,
    #[prop(into, default = "auto".to_string().into())] width: Signal<String>,
    // Ajustei o default para garantir, verifique se a pasta é essa mesmo
    #[prop(into, default = "PaperCards".to_string())] theme: String,
    #[prop(into)] selection_group: Signal<Option<usize>>,
    #[prop(default = None)] on_click: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    // O get_card_path precisa receber o tema correto
    let path = move || get_card_path(&id.get(), &theme);
    let is_selected = move || selection_group.get().is_some();

    view! {
        <img
            src=path // Se o tema estiver errado, esse path gera um link quebrado (404)
            style:width=move || width.get()
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

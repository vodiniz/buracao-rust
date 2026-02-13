use crate::utils::assets::get_card_path;
use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(into)] id: Signal<String>,
    #[prop(into, default = "auto".to_string().into())] width: Signal<String>,
    #[prop(into, default = "PaperCards".to_string())] theme: String,
    #[prop(into)] selection_group: Signal<Option<usize>>,
    #[prop(default = None)] on_click: Option<Callback<web_sys::MouseEvent>>,

    // --- NOVOS PROPS PARA DRAG ---
    #[prop(default = false)] draggable: bool,
    #[prop(default = None)] on_drag_start: Option<Callback<web_sys::DragEvent>>,
    #[prop(default = None)] on_drag_over: Option<Callback<web_sys::DragEvent>>,
    #[prop(default = None)] on_drop: Option<Callback<web_sys::DragEvent>>,
) -> impl IntoView {
    let path = move || get_card_path(&id.get(), &theme);
    let is_selected = move || selection_group.get().is_some();

    view! {
        <img
            src=path
            style:width=move || width.get()
            style:border=move || if is_selected() { "3px solid yellow" } else { "none" }
            style:border-radius="8px"
            style:margin-top=move || if is_selected() { "-20px" } else { "0" }
            style:transition="all 0.2s ease"
            // Se for draggable, cursor deve ser grab
            style:cursor=if draggable { "grab" } else { "pointer" }

            // --- ATRIBUTOS DE DRAG ---
            draggable=draggable.to_string()

            on:click=move |e| {
                if let Some(cb) = on_click { cb.run(e); }
            }

            // Eventos de Drag
            on:dragstart=move |e| {
                if let Some(cb) = on_drag_start { cb.run(e); }
            }
            on:dragover=move |e| {
                if let Some(cb) = on_drag_over { cb.run(e); }
            }
            on:drop=move |e| {
                if let Some(cb) = on_drop { cb.run(e); }
            }
        />
    }
}

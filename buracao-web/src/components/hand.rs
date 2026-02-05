use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn Hand(
    #[prop(into)] cartas: Signal<Vec<Carta>>,
    #[prop(into)] selected_indices: RwSignal<HashSet<usize>>,
    #[prop(into, default = "100px".to_string().into())] card_width: Signal<String>,
    #[prop(into, default = "PaperCards1.1".to_string())] theme: String, // Confira se o default aqui bate com sua pasta
) -> impl IntoView {
    let toggle_selection = move |index: usize| {
        selected_indices.update(|set| {
            if set.contains(&index) {
                set.remove(&index);
            } else {
                set.insert(index);
            }
        });
    };

    view! {
        <div style="display: flex; justify-content: center; padding: 20px; overflow-x: auto; min-height: 160px;">
            <For
                each=move || cartas.get().into_iter().enumerate()
                key=|(i, _carta)| *i
                children=move |(index, carta)| {
                    let is_selected = move || selected_indices.get().contains(&index);
                    let selection_state = move || if is_selected() { Some(1) } else { None };

                    // --- CORREÇÃO AQUI ---
                    // Precisamos clonar o tema para passar para cada carta individualmente
                    let theme_str = theme.clone();
                    let width_signal = card_width;

                    view! {
                        <div style="margin-right: -40px; transition: margin-right 0.2s;"
                             style:margin-right=move || if index == cartas.with(|c| c.len()) - 1 { "0px" } else { "-40px" }
                        >
                            <Card
                                id=carta_para_asset(&carta)
                                width=width_signal

                                // AQUI ESTAVA FALTANDO:
                                theme=theme_str

                                selection_group=Signal::derive(selection_state)
                                on_click=Some(Callback::new(move |_: web_sys::MouseEvent| toggle_selection(index)))
                            />
                        </div>
                    }
                }
            />
        </div>
    }
}

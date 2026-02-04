use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn Hand(
    #[prop(into)] cartas: Signal<Vec<Carta>>,
    selected_indices: RwSignal<HashSet<usize>>,

    // CORREÇÃO: Substituído MaybeSignal por Signal
    #[prop(into, default = "100px".to_string().into())] card_width: Signal<String>,

    #[prop(into, default = "PaperCards1.1".to_string())] theme: String,
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
        <div style="
            display: flex;
            justify-content: flex-start;
            align-items: flex-end;
            padding: 60px 10px 10px 10px;
            overflow-x: auto;
            min-height: 220px;
            width: 100%;
            scrollbar-color: rgba(255,255,255,0.5) rgba(0,0,0,0.2);
            scrollbar-width: thin;
        ">
            <For
                each=move || cartas.get().into_iter().enumerate()
                key=|(i, _carta)| *i
                children=move |(index, carta)| {
                    let is_selected = move || selected_indices.get().contains(&index);
                    let selection_state = move || if is_selected() { Some(1) } else { None };
                    let total_len = move || cartas.with(|c| c.len());

                    // No Leptos 0.7, Signal é Copy, então nem precisaria do clone explicito às vezes,
                    // mas mantemos para clareza ou se o compilador reclamar de move no loop.
                    let width_signal = card_width;
                    let theme_str = theme.clone();

                    view! {
                        <div
                            style="transition: margin-right 0.1s;"
                            style:margin-right=move || if index == total_len() - 1 { "0px" } else { "-40px" }
                        >
                            <Card
                                id=carta_para_asset(&carta)
                                width=width_signal // Passando o Signal
                                theme=theme_str
                                selection_group=Signal::derive(selection_state)
                                on_click=Some(Callback::new(move |_: web_sys::MouseEvent| toggle_selection(index)))
                            />
                        </div>
                    }
                }
            />

            <div style="min-width: 50px; height: 1px; flex-shrink: 0;"></div>
        </div>
    }
}

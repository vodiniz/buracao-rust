use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn Hand(
    // MUDANÇA 1: De Signal para RwSignal (Permite Leitura e Escrita)
    #[prop(into)] cartas: RwSignal<Vec<Carta>>,
    #[prop(into)] selected_indices: RwSignal<HashSet<usize>>,
    #[prop(into, default = "100px".to_string().into())] card_width: Signal<String>,
    #[prop(into, default = "PaperCards1.1".to_string())] theme: String,
) -> impl IntoView {
    let (dragged_idx, set_dragged_idx) = signal(Option::<usize>::None);

    let handle_drag_start = move |ev: web_sys::DragEvent, idx: usize| {
        set_dragged_idx.set(Some(idx));
        if let Some(dt) = ev.data_transfer() {
            dt.set_effect_allowed("move");
        }
    };

    let handle_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
    };

    let handle_drop = move |ev: web_sys::DragEvent, target_idx: usize| {
        ev.prevent_default();

        if let Some(source_idx) = dragged_idx.get() {
            if source_idx != target_idx {
                // Agora cartas.update funciona pois é um RwSignal
                cartas.update(|c| {
                    if source_idx < c.len() && target_idx < c.len() {
                        let carta = c.remove(source_idx);
                        c.insert(target_idx, carta);
                    }
                });
                selected_indices.update(|s| s.clear());
            }
        }
        set_dragged_idx.set(None);
    };

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
                key=|(i, carta)| format!("{}-{}", i, carta_para_asset(carta))
                children=move |(index, carta)| {
                    let is_selected = move || selected_indices.get().contains(&index);
                    let selection_state = move || if is_selected() { Some(1) } else { None };
                    let theme_str = theme.clone();
                    let width_signal = card_width;

                    view! {
                        <div
                            style="margin-right: -40px; transition: margin-right 0.2s;"
                            // Pequena correção: usamos 'with' para não clonar o vetor inteiro só pra pegar o len
                            style:margin-right=move || if index == cartas.with(|c| c.len()) - 1 { "0px" } else { "-40px" }
                        >
                            <Card
                                id=carta_para_asset(&carta)
                                width=width_signal
                                theme=theme_str
                                draggable=true
                                on_drag_start=Some(Callback::new(move |e| handle_drag_start(e, index)))
                                on_drag_over=Some(Callback::new(handle_drag_over))
                                on_drop=Some(Callback::new(move |e| handle_drop(e, index)))
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

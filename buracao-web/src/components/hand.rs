use crate::components::card::Card;
use crate::game::state::CartaIdentificada;
use crate::utils::mappers::carta_para_asset;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn Hand(
    #[prop(into)] cartas: RwSignal<Vec<CartaIdentificada>>,
    #[prop(into)] selected_ids: RwSignal<HashSet<u32>>,
    #[prop(into, default = "100px".to_string().into())] card_width: Signal<String>,
    #[prop(into, default = "PaperCards1.1".to_string())] theme: String,
    #[prop(into, optional)] shake_trigger: Option<Signal<usize>>,
) -> impl IntoView {
    // --- ESTILOS CSS ---
    let styles = view! {
        <style>
            "@keyframes flyIn {
                0% { opacity: 0; transform: translateY(-50px) scale(0.8); }
                100% { opacity: 1; transform: translateY(0) scale(1); }
            }
            @keyframes shakeCard {
                0%, 100% { transform: translateX(0); }
                20% { transform: translateX(-5px) rotate(-2deg); }
                40% { transform: translateX(5px) rotate(2deg); }
                60% { transform: translateX(-5px) rotate(-2deg); }
                80% { transform: translateX(5px) rotate(2deg); }
            }

            /* Estado base: visível e estável */
            .card-wrapper {
                opacity: 1;
                transform: none;
                transition: margin-right 0.2s;
                margin-right: -40px; /* Padrão: sobreposição */
            }
            .card-wrapper:last-child {
                margin-right: 0px !important;
            }

            /* Estado de entrada (só roda uma vez) */
            .card-enter {
                animation: flyIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
            }

            /* Estado de erro */
            .card-shake {
                animation: shakeCard 0.4s ease-in-out;
                filter: sepia(1) hue-rotate(-50deg) saturate(3);
                /* Forçamos opacidade 1 para garantir que não suma na troca de classe */
                opacity: 1 !important; 
            }
            "
        </style>
    };

    // --- LÓGICA DE SHAKE GLOBAL ---
    // Monitora o sinal do pai e ativa o estado local de shake
    let (is_shaking, set_shaking) = signal(false);

    if let Some(trigger) = shake_trigger {
        Effect::new(move |_| {
            trigger.track();
            if trigger.get() > 0 {
                set_shaking.set(true);
                set_timeout(
                    move || set_shaking.set(false),
                    std::time::Duration::from_millis(400),
                );
            }
        });
    }

    // --- DRAG & DROP (Mantido) ---
    let (dragged_idx, set_dragged_idx) = signal(Option::<usize>::None);
    let handle_drag_start = move |ev: web_sys::DragEvent, idx: usize| {
        set_dragged_idx.set(Some(idx));
        if let Some(dt) = ev.data_transfer() {
            dt.set_effect_allowed("move");
        }
    };
    let handle_drag_over = move |ev: web_sys::DragEvent| ev.prevent_default();
    let handle_drop = move |ev: web_sys::DragEvent, target_idx: usize| {
        ev.prevent_default();
        if let Some(source_idx) = dragged_idx.get() {
            if source_idx != target_idx {
                cartas.update(|c| {
                    if source_idx < c.len() && target_idx < c.len() {
                        let carta = c.remove(source_idx);
                        c.insert(target_idx, carta);
                    }
                });
                selected_ids.update(|s| s.clear());
            }
        }
        set_dragged_idx.set(None);
    };

    let toggle_selection = move |id: u32| {
        selected_ids.update(|set| {
            if set.contains(&id) {
                set.remove(&id);
            } else {
                set.insert(id);
            }
        });
    };

    view! {
        {styles}
        <div style="display: flex; justify-content: center; padding: 20px; overflow-x: auto; min-height: 160px;">
            <For
                each=move || cartas.get().into_iter().enumerate()
                // Usamos o ID único visual como chave.
                // Se a carta muda de índice (drag), o ID não muda, então o DOM é preservado.
                key=|(i, item)| item.id

                children=move |(index, item)| {
                    // Lógica de entrada temporária (mantida para compras novas)
                    let (is_entering, set_entering) = signal(true);
                    set_timeout(move || { set_entering.set(false); }, std::time::Duration::from_millis(400));

                    let id_atual = item.id;
                    let carta_real = item.carta;

                    let is_selected = move || selected_ids.get().contains(&id_atual);
                    let selection_state = move || if is_selected() { Some(1) } else { None };
                    let theme_str = theme.clone();
                    let width_signal = card_width;


                    view! {
                        <div
                            class=move || {
                                let mut classes = "card-wrapper".to_string();
                                if is_entering.get() { classes.push_str(" card-enter"); }
                                if is_shaking.get() && is_selected() { classes.push_str(" card-shake"); }
                                classes
                            }
                            // Usamos with para acessar tamanho sem clonar
                        >
                            <Card
                                id=carta_para_asset(&carta_real)
                                width=width_signal
                                theme=theme_str
                                draggable=true
                                on_drag_start=Some(Callback::new(move |e| handle_drag_start(e, index)))
                                on_drag_over=Some(Callback::new(handle_drag_over))
                                on_drop=Some(Callback::new(move |e| handle_drop(e, index)))
                                selection_group=Signal::derive(selection_state)
                                on_click=Some(Callback::new(move |_: web_sys::MouseEvent| toggle_selection(id_atual)))
                            />
                        </div>
                    }
                }
            />
        </div>
    }
}

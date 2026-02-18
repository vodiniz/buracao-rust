use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;

#[component]
pub fn Board(
    #[prop(into)] lixo: Signal<Option<Carta>>,
    #[prop(into)] lixo_selecionado: Signal<bool>,
    #[prop(into)] theme: String,
    #[prop(into)] card_width: Signal<String>,
    #[prop(into)] qtd_monte: Signal<u32>,
    #[prop(into)] qtd_lixo: Signal<u32>,
    #[prop(into)] verso_monte: Signal<String>,
    #[prop(default = None)] on_click_deck: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = None)] on_click_trash: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    let no_selection = Signal::derive(|| Option::<usize>::None);
    let selection_lixo_visual = Signal::derive(move || {
        if lixo_selecionado.get() {
            Some(1)
        } else {
            None
        }
    });

    let theme_monte = theme.clone();
    let theme_lixo = theme.clone();

    // Destaque visual do lixo quando muda
    let (highlight_trash, set_highlight_trash) = signal(false);
    let lixo_id = Signal::derive(move || lixo.get().map(|c| format!("{}{}", c.valor, c.naipe)));

    Effect::new(move |prev_id: Option<Option<String>>| {
        let curr_id = lixo_id.get();

        // Só ativa o brilho se:
        // 1. Não é a primeira execução (prev_id.is_some())
        // 2. O ID da carta mudou em relação ao anterior
        if let Some(last) = prev_id {
            if last != curr_id {
                set_highlight_trash.set(true);
                set_timeout(
                    move || set_highlight_trash.set(false),
                    std::time::Duration::from_millis(500),
                );
            }
        }

        // Retorna o atual para ser o 'prev' da próxima vez
        curr_id
    });

    view! {
        <div style="
            display: flex; gap: 40px; justify-content: center; align-items: center; 
            padding: 20px; background-color: rgba(0,0,0,0.15); border-radius: 20px;
            box-shadow: inset 0 0 20px rgba(0,0,0,0.2);
        ">
            // --- MONTE (STACKED EFFECT) ---
            <div
                on:click=move |e| if let Some(cb) = on_click_deck { cb.run(e) }
                style="text-align: center; position: relative; cursor: pointer; transition: transform 0.1s;"
                style:active="transform: scale(0.95);"
            >
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">
                    "Monte (" {move || qtd_monte.get()} ")"
                </span>

                <div style="position: relative; display: inline-block;">
                    // Cartas "falsas" embaixo para dar volume
                    <div style="position: absolute; top: -4px; left: -2px; transform: rotate(-2deg); filter: brightness(0.7);">
                        <Card id=verso_monte.clone() width=card_width theme=theme_monte.clone() selection_group=no_selection />
                    </div>
                    <div style="position: absolute; top: -2px; left: 1px; transform: rotate(1deg); filter: brightness(0.85);">
                        <Card id=verso_monte.clone() width=card_width theme=theme_monte.clone() selection_group=no_selection />
                    </div>

                    // Carta do topo (Real)
                    <div style="position: relative; z-index: 2; box-shadow: 0 5px 15px rgba(0,0,0,0.5);">
                        <Card id=verso_monte width=card_width theme=theme_monte selection_group=no_selection />
                    </div>
                </div>
            </div>

            // --- LIXO ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">
                    "Lixo (" {move || qtd_lixo.get()} ")"
                </span>

                {move || match lixo.get() {
                    Some(carta) => view! {
                        <div style=move || {
                            let glow = if highlight_trash.get() { "0 0 20px 5px rgba(255, 235, 59, 0.6)" } else { "none" };
                            let scale = if highlight_trash.get() { "scale(1.1)" } else { "scale(1.0)" };
                            format!("
                                opacity: 1.0; 
                                transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275); 
                                box-shadow: {}; 
                                transform: {};
                                border-radius: 8px;
                            ", glow, scale)
                        }>
                            <Card
                                id=carta_para_asset(&carta)
                                width=card_width
                                theme=theme_lixo.clone()
                                selection_group=selection_lixo_visual
                                on_click=on_click_trash
                            />
                        </div>
                    }.into_any(),
                    None => view! {
                        <div style=move || format!("
                            width: {}; 
                            height: calc({} * 1.45); 
                            border: 2px dashed rgba(255,255,255,0.2); 
                            border-radius: 8px;
                            display: flex; align-items: center; justify-content: center;
                            color: rgba(255,255,255,0.3); font-size: 11px;
                        ", card_width.get(), card_width.get())>
                            "Vazio"
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

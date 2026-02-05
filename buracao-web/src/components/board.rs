use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;

#[component]
pub fn Board(
    #[prop(into)] lixo: Signal<Option<Carta>>,
    #[prop(into)] lixo_selecionado: Signal<bool>,
    // 1. Recebe o tema do App corretamente aqui
    #[prop(into)] theme: String,
    #[prop(into)] card_width: Signal<String>,
    #[prop(into)] qtd_monte: Signal<u32>,
    #[prop(into)] qtd_lixo: Signal<u32>,
    #[prop(into)] verso_monte: Signal<String>,
    #[prop(default = None)] on_click_deck: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = None)] on_click_trash: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    // Helpers
    // Usamos tipagem explícita no None para ajudar o compilador
    let no_selection = Signal::derive(|| Option::<usize>::None);
    let selection_lixo_visual = Signal::derive(move || {
        if lixo_selecionado.get() {
            Some(1)
        } else {
            None
        }
    });

    // 2. PREPARAÇÃO DO TEMA (CRUCIAL):
    // Precisamos clonar a string para passar para dois componentes diferentes (Monte e Lixo).
    // Se não fizermos isso e não passarmos no <Card>, ele usa o padrão "PaperCards1.1".
    let theme_monte = theme.clone();
    let theme_lixo = theme.clone();

    view! {
        <div style="
            display: flex; 
            gap: 20px; 
            justify-content: center; 
            align-items: center; 
            padding: 15px;
            background-color: rgba(0,0,0,0.2);
            border-radius: 16px;
        ">
            // --- MONTE ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">
                    "Monte (" {move || qtd_monte.get()} ")"
                </span>
                <Card
                    id=verso_monte
                    width=card_width

                    // A CORREÇÃO ESTÁ AQUI:
                    theme=theme_monte

                    selection_group=no_selection
                    on_click=on_click_deck
                />
            </div>

            // --- LIXO ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">
                    "Lixo (" {move || qtd_lixo.get()} ")"
                </span>

                {move || match lixo.get() {
                    Some(carta) => view! {
                        <div style="opacity: 1.0; transition: opacity 0.3s;">
                            <Card
                                id=carta_para_asset(&carta)
                                width=card_width

                                // A CORREÇÃO NO LIXO TAMBÉM:
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
                            border: 2px dashed rgba(255,255,255,0.3); 
                            border-radius: 8px;
                            display: flex; align-items: center; justify-content: center;
                            color: rgba(255,255,255,0.5); font-size: 11px;
                            transition: width 0.1s, height 0.1s;
                        ", card_width.get(), card_width.get())>
                            "Vazio"
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

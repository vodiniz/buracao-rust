use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;

#[component]
pub fn Board(
    #[prop(into)] lixo: Signal<Option<Carta>>,
    #[prop(into)] lixo_selecionado: Signal<bool>,
    #[prop(default = None)] on_click_deck: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = None)] on_click_trash: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into, default = "/assets/cards/PaperCards1.1".to_string())] theme: String,

    // IMPORTANTE: Recebe Signal<String> para reagir às mudanças
    #[prop(into, default = "90px".to_string().into())] card_width: Signal<String>,
) -> impl IntoView {
    let no_selection = Signal::derive(|| None);
    let selection_lixo_visual = Signal::derive(move || {
        if lixo_selecionado.get() {
            Some(1)
        } else {
            None
        }
    });

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
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">"Monte"</span>
                <Card
                    id="back_r".to_string()
                    // Passa o sinal direto. O Card já sabe lidar com ele.
                    width=card_width
                    selection_group=no_selection
                    on_click=on_click_deck
                    theme=theme.clone()
                />
            </div>

            // --- LIXO ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 11px; margin-bottom: 5px; display: block; opacity: 0.7;">"Lixo"</span>
                {move || match lixo.get() {
                    Some(carta) => view! {
                        <div style="opacity: 1.0; transition: opacity 0.3s;">
                            <Card
                                id=carta_para_asset(&carta)
                                // Passa o sinal direto para a carta do lixo
                                width=card_width
                                selection_group=selection_lixo_visual
                                on_click=on_click_trash
                                theme=theme.clone()
                            />
                        </div>
                    }.into_any(),
                    None => view! {
                        // --- CORREÇÃO DO VAZIO ---
                        // Usamos `style=move ||` para recalcular o CSS quando card_width mudar.
                        <div style=move || format!("
                            width: {}; 
                            height: calc({} * 1.45); /* 1.45 mantém a proporção padrão de carta */
                            border: 2px dashed rgba(255,255,255,0.3); 
                            border-radius: 8px;
                            display: flex; align-items: center; justify-content: center;
                            color: rgba(255,255,255,0.5); font-size: 11px;
                            transition: width 0.1s, height 0.1s;
                        ", card_width.get(), card_width.get())> // <--- .get() aqui é crucial
                            "Vazio"
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

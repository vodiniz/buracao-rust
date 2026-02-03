use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::baralho::Carta;
use leptos::prelude::*;

#[component]
pub fn Board(
    #[prop(into)] lixo: Signal<Option<Carta>>,
    #[prop(default = None)] on_click_deck: Option<Callback<web_sys::MouseEvent>>,
    #[prop(default = None)] on_click_trash: Option<Callback<web_sys::MouseEvent>>,
) -> impl IntoView {
    // Helper para passar "Nenhuma seleção" para as cartas do tabuleiro
    let no_selection = Signal::derive(|| None);

    view! {
        <div style="
            display: flex; 
            gap: 40px; 
            justify-content: center; 
            align-items: center; 
            margin: 20px 0;
            padding: 20px;
            background-color: rgba(0,0,0,0.1);
            border-radius: 16px;
        ">
            // --- MONTE DE COMPRAS ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 12px; margin-bottom: 5px; display: block;">"Monte"</span>
                <Card
                    id="back_r".to_string()
                    width="100px".to_string()
                    // CORREÇÃO: Passamos o sinal vazio
                    selection_group=no_selection
                    on_click=on_click_deck
                />
            </div>

            // --- LIXO ---
            <div style="text-align: center;">
                <span style="color: white; font-size: 12px; margin-bottom: 5px; display: block;">"Lixo"</span>
                {move || match lixo.get() {
                    Some(carta) => view! {
                        <div style="opacity: 1.0; transition: opacity 0.3s;">
                            <Card
                                id=carta_para_asset(&carta)
                                width="100px".to_string()
                                // CORREÇÃO: Passamos o sinal vazio
                                selection_group=no_selection
                                on_click=on_click_trash
                            />
                        </div>
                    }.into_any(),
                    None => view! {
                        <div style="
                            width: 100px; 
                            height: 140px; 
                            border: 2px dashed rgba(255,255,255,0.3); 
                            border-radius: 8px;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            color: rgba(255,255,255,0.5);
                            font-size: 12px;
                        ">
                            "Vazio"
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

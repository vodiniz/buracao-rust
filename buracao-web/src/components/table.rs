use crate::components::card::Card;
use crate::utils::mappers::{carta_para_asset, organizar_para_exibicao};
use buracao_core::acoes::DetalheJogo;
use buracao_core::baralho::Carta; // <--- Importante
use leptos::prelude::*;

#[component]
pub fn Table(
    #[prop(into)] titulo: String,
    #[prop(into)] jogos: Signal<Vec<DetalheJogo>>,
    // NOVA PROP: Recebe os 3 vermelhos. Default vazio para não quebrar se não passar.
    #[prop(into, default = Signal::derive(|| vec![]))] tres_vermelhos: Signal<Vec<Carta>>,
    #[prop(optional, into, default = "/assets/cards/PaperCards1.1".to_string())] theme: String,
    #[prop(default = None)] on_click: Option<Callback<usize>>,
    #[prop(into, default = "80px".to_string().into())] card_width: Signal<String>,
) -> impl IntoView {
    let interativo = on_click.is_some();

    let theme_body = theme.clone();
    let theme_footer = theme.clone();

    view! {
        <div style="
            flex: 1; 
            border: 1px solid rgba(255,255,255,0.1); 
            border-radius: 12px; 
            min-width: 300px; 
            background: rgba(0,0,0,0.15);
            display: flex;
            flex-direction: column; /* Organiza Header, Body, Footer verticalmente */
            height: 100%; 
            max-height: 50vh; 
            overflow: hidden; /* O Container pai não scrola, quem scrola é o miolo */
        ">
            // --- HEADER (FIXO) ---
            <h3 style="
                margin: 0;
                padding: 10px;
                color: #ffeb3b; 
                font-size: 14px; 
                text-transform: uppercase; 
                border-bottom: 1px solid rgba(255,255,255,0.1);
                background: rgba(0,0,0,0.2);
                text-align: center;
                flex-shrink: 0;
            ">
                {titulo}
            </h3>

            // --- BODY (JOGOS - COM SCROLL) ---
            <div style="
                flex: 1; /* Ocupa todo espaço disponível */
                overflow-y: auto; /* Scroll apenas aqui */
                padding: 10px;
                scrollbar-width: thin;
                scrollbar-color: rgba(255,255,255,0.3) transparent;
            ">
                <div style="display: flex; flex-wrap: wrap; gap: 15px; align-content: flex-start; justify-content: center;">
                    {move || {
                        let lista_jogos = jogos.get();
                        if lista_jogos.is_empty() {
                            view! {
                                <div style="width: 100%; text-align: center; padding: 20px; color: rgba(255,255,255,0.3); font-style: italic; font-size: 12px;">
                                    "Nenhum jogo"
                                </div>
                            }.into_any()
                        } else {
                            lista_jogos.into_iter().enumerate().map(|(idx, jogo)| {
                                let cb_local = on_click;
                                let cartas_visuais = organizar_para_exibicao(&jogo.cartas);
                                let theme_local = theme_body.clone();

                                view! {
                                    <div
                                        on:click=move |_| {
                                            if let Some(cb) = cb_local {
                                                cb.run(idx);
                                            }
                                        }
                                        style=move || {
                                            let cursor = if interativo { "pointer" } else { "default" };
                                            format!("
                                                background: rgba(0,0,0,0.2); 
                                                padding: 8px; 
                                                border-radius: 10px;
                                                display: flex;
                                                align-items: center;
                                                cursor: {};
                                                transition: all 0.2s;
                                                border: 1px solid rgba(255,255,255,0.05);
                                                min-height: 90px;
                                            ", cursor)
                                        }
                                    >
                                        {cartas_visuais.into_iter().map(|c| {
                                            view! {
                                                <div style="margin-right: -35px;">
                                                    <Card
                                                        id=carta_para_asset(&c)
                                                        width=card_width // <--- Tamanho dinâmico
                                                        selection_group=Signal::derive(|| None)
                                                        theme=theme_local.clone()
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}

                                        <div style="width: 35px;"></div>
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>
            </div>

            // --- FOOTER (3 VERMELHOS - FIXO EMBAIXO) ---
            {move || {
                let tres = tres_vermelhos.get();
                let theme_local = theme_footer.clone();
                if !tres.is_empty() {
                    view! {
                        <div style="
                            padding: 5px 10px;
                            background: rgba(0,0,0,0.3);
                            border-top: 1px solid rgba(255,255,255,0.1);
                            display: flex;
                            align-items: center;
                            gap: 10px;
                            flex-shrink: 0;
                            min-height: 40px;
                        ">
                            <span style="font-size: 10px; color: #ff5252; font-weight: bold; text-transform: uppercase;">
                            </span>
                            <div style="display: flex; gap: 5px;">
                                {tres.into_iter().map(|c| {
                                    view! {
                                        <Card
                                            id=carta_para_asset(&c)
                                            width="35px".to_string() // <--- Tamanho reduzido
                                            selection_group=Signal::derive(|| None)
                                            theme=theme_local.clone()
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }}
        </div>
    }
}

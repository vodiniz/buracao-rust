use crate::components::card::Card;
use crate::utils::mappers::{
    analisar_status_canastra, carta_para_asset, organizar_para_exibicao, StatusCanastra,
};
use buracao_core::acoes::DetalheJogo;
use buracao_core::baralho::Carta;
use leptos::prelude::*;

#[component]
pub fn Table(
    #[prop(into)] titulo: String,
    #[prop(into)] jogos: Signal<Vec<DetalheJogo>>,
    #[prop(into, default = Signal::derive(|| vec![]))] tres_vermelhos: Signal<Vec<Carta>>,
    #[prop(default = None)] on_click: Option<Callback<usize>>,
    #[prop(optional, into, default = "PaperCards".to_string())] theme: String,
    #[prop(optional, into, default = false)] is_my_team: bool,

    #[prop(into, default = "80px".to_string().into())] card_width: Signal<String>,
) -> impl IntoView {
    let interativo = on_click.is_some();
    // Clones para usar nas closures
    let theme_body = theme.clone();
    let theme_footer = theme.clone();

    // Lógica da Borda da Mesa
    let border_style = if is_my_team {
        "2px solid #ffeb3b" // Verde mais grosso se for meu time
    } else {
        "1px solid rgba(255,255,255,0.1)" // Borda sutil se for inimigo
    };

    let box_shadow = if is_my_team {
        "0 0 15px rgba(255, 235, 59, 0.4)" // Glow
    } else {
        "none"
    };

    view! {
        <div style=format!("
            flex: 1; 
            border: {}; 
            box-shadow: {};
            border-radius: 12px; 
            min-width: 300px; 
            background: rgba(0,0,0,0.15);
            display: flex;
            flex-direction: column;
            height: 100%; 
            max-height: 50vh; 
            overflow: hidden;
            transition: all 0.3s ease;
        ", border_style, box_shadow)>
            // --- HEADER ---
            <h3 style="
                margin: 0; padding: 10px; color: #ffeb3b; font-size: 14px; 
                text-transform: uppercase; border-bottom: 1px solid rgba(255,255,255,0.1);
                background: rgba(0,0,0,0.2); text-align: center; flex-shrink: 0;
            ">
                {titulo}
            </h3>

            // --- BODY ---
            <div style="
                flex: 1; overflow-y: auto; padding: 10px;
                scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.3) transparent;
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

                                let mut cartas_visuais = organizar_para_exibicao(&jogo.cartas);
                                let status = analisar_status_canastra(&jogo.cartas);

                                let mut index_rotacionado: Option<usize> = None;
                                let mut index_escuro: Option<usize> = None;

                                match status {
                                    StatusCanastra::Real => {
                                        if !cartas_visuais.is_empty() {
                                            let carta_base = cartas_visuais.remove(0);
                                            let meio = cartas_visuais.len() / 2;
                                            cartas_visuais.insert(meio, carta_base);
                                            index_rotacionado = Some(meio);
                                        }
                                    },
                                    StatusCanastra::Suja => {
                                        if !cartas_visuais.is_empty() {
                                            let carta_base = cartas_visuais.remove(0);
                                            let meio = cartas_visuais.len() / 2;
                                            cartas_visuais.insert(meio, carta_base);

                                            index_rotacionado = Some(meio);
                                            index_escuro = Some(meio);
                                        }
                                    },
                                    StatusCanastra::Normal => {}
                                }

                                let theme_local = theme_body.clone();
                                let total_cartas = cartas_visuais.len();
                                // CORREÇÃO 2: Largura clonada para evitar problema de move
                                let width_local = card_width;

                                view! {
                                    <div
                                        on:click=move |_| { if let Some(cb) = cb_local { cb.run(idx); } }
                                        // 3. CORREÇÃO DO ESPAÇO EXTRA:
                                        // - Adicionado 'width: fit-content' para a caixa abraçar as cartas
                                        // - Padding lateral reduzido para ficar justo
                                        style=move || {
                                            let cursor = if interativo { "pointer" } else { "default" };
                                            format!("
                                                background: rgba(0,0,0,0.2); 
                                                padding: 8px 8px 8px 8px; 
                                                border-radius: 10px;
                                                display: inline-flex; 
                                                align-items: center; 
                                                cursor: {}; 
                                                transition: all 0.2s;
                                                border: 1px solid rgba(255,255,255,0.05); 
                                                min-height: 90px;
                                                /* width: fit-content; REMOVIDO pois inline-flex resolve */
                                            ", cursor)
                                        }
                                    >
                                        {cartas_visuais.into_iter().enumerate().map(|(c_idx, c)| {
                                            let eh_rotacionada = index_rotacionado == Some(c_idx);
                                            let eh_escura = index_escuro == Some(c_idx);
                                            let eh_ultima = c_idx == total_cartas - 1;

                                            // Clones para o loop interno
                                            let theme_card = theme_local.clone();
                                            let w_card = width_local;

                                            view! {
                                                <div
                                                    style="transition: all 0.3s ease-out;"
                                                    style:display="flex"
                                                    style:justify-content="center"
                                                    style:align-items="center"
                                                    style:width=w_card
                                                    style:margin-right=if eh_ultima { "0px" } else { "-45px" }
                                                    style:transform=if eh_rotacionada { "rotate(90deg) translateY(-15px)" } else { "none" }
                                                    style:filter=if eh_escura { "brightness(0.4) sepia(1) hue-rotate(-50deg) saturate(3)" } else { "none" }
                                                    style:z-index=if eh_rotacionada { "10" } else { "auto" }
                                                >
                                                    <Card
                                                        id=carta_para_asset(&c)
                                                        width=w_card
                                                        // CORREÇÃO 3: Tipagem explícita no None para ajudar o compilador
                                                        selection_group=Signal::derive(|| Option::<usize>::None)
                                                        theme=theme_card
                                                        // Sem on_click -> vira "Fantasma" (pointer-events: none),
                                                        // mas o wrapper pai (div acima) captura o clique. Perfeito.
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}

                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>
            </div>

            // --- FOOTER (Três Vermelhos) ---
            {move || {
                let tres = tres_vermelhos.get();
                let theme_local = theme_footer.clone();
                if !tres.is_empty() {
                    view! {
                        <div style="padding: 5px 10px; background: rgba(0,0,0,0.3); border-top: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center; gap: 10px; flex-shrink: 0; min-height: 40px;">
                            <div style="display: flex; gap: 5px;">
                                {tres.into_iter().map(|c| {
                                    view! {
                                        <Card
                                            id=carta_para_asset(&c)
                                            width="35px".to_string()
                                            selection_group=Signal::derive(|| Option::<usize>::None)
                                            theme=theme_local.clone()
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                } else { view! {}.into_any() }
            }}
        </div>
    }
}

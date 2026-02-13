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

    // CORREÇÃO CRÍTICA: Armazenar Strings em StoredValue para permitir clones baratos
    // dentro de múltiplos closures (Fn) sem consumir a variável original (FnOnce).
    let theme_store = StoredValue::new(theme);
    let titulo_store = StoredValue::new(titulo);

    // Estilos calculados
    let border_style = if is_my_team {
        "2px solid #ffeb3b"
    } else {
        "1px solid rgba(255,255,255,0.1)"
    };

    let box_shadow = if is_my_team {
        "0 0 15px rgba(255, 235, 59, 0.4)"
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
                // Acessamos o valor guardado
                {titulo_store.get_value()}
            </h3>

            // --- BODY (Jogos na Mesa) ---
            <div style="
                flex: 1; overflow-y: auto; padding: 10px;
                scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.3) transparent;
            ">
                <div style="display: flex; flex-wrap: wrap; gap: 15px; align-content: flex-start; justify-content: center;">

                    <Show
                        when=move || !jogos.get().is_empty()
                        fallback=|| view! {
                            <div style="width: 100%; text-align: center; padding: 20px; color: rgba(255,255,255,0.3); font-style: italic; font-size: 12px;">
                                "Nenhum jogo"
                            </div>
                        }
                    >
                        <For
                            each=move || jogos.get().into_iter().enumerate()
                            key=|(idx, jogo)| format!("{}-{}", idx, jogo.id)
                            children=move |(idx, jogo)| {
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

                                let total_cartas = cartas_visuais.len();
                                let cb_local = on_click;
                                let width_local = card_width; // Signal é Copy

                                view! {
                                    <div
                                        on:click=move |_| { if let Some(cb) = cb_local { cb.run(idx); } }
                                        style=move || {
                                            let cursor = if interativo { "pointer" } else { "default" };
                                            format!("
                                                background: rgba(0,0,0,0.2); 
                                                padding: 8px; 
                                                border-radius: 10px;
                                                display: inline-flex; 
                                                align-items: center; 
                                                cursor: {}; 
                                                transition: all 0.2s;
                                                border: 1px solid rgba(255,255,255,0.05); 
                                                min-height: 90px;
                                            ", cursor)
                                        }
                                    >
                                        {cartas_visuais.into_iter().enumerate().map(|(c_idx, c)| {
                                            let eh_rotacionada = index_rotacionado == Some(c_idx);
                                            let eh_escura = index_escuro == Some(c_idx);
                                            let eh_ultima = c_idx == total_cartas - 1;

                                            // CLONE LIMPO: Pegamos do Store a cada iteração
                                            let theme_local = theme_store.get_value();
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
                                                        selection_group=Signal::derive(|| Option::<usize>::None)
                                                        theme=theme_local
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }
                            }
                        />
                    </Show>
                </div>
            </div>

            // --- FOOTER (Três Vermelhos) ---
            <Show
                when=move || !tres_vermelhos.get().is_empty()
                fallback=|| ()
            >
                <div style="padding: 5px 10px; background: rgba(0,0,0,0.3); border-top: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; justify-content: center; gap: 10px; flex-shrink: 0; min-height: 40px;">
                    <div style="display: flex; gap: 5px;">
                        <For
                            each=move || tres_vermelhos.get()
                            key=|c| format!("{}{}", c.valor, c.naipe)
                            children=move |c| {
                                // CLONE LIMPO: Pegamos do Store
                                let theme_local = theme_store.get_value();
                                view! {
                                    <Card
                                        id=carta_para_asset(&c)
                                        width="35px".to_string()
                                        selection_group=Signal::derive(|| Option::<usize>::None)
                                        theme=theme_local
                                    />
                                }
                            }
                        />
                    </div>
                </div>
            </Show>
        </div>
    }
}

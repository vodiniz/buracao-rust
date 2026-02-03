use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::acoes::DetalheJogo;
use leptos::prelude::*;

#[component]
pub fn Table(
    #[prop(into)] jogos_time_a: Signal<Vec<DetalheJogo>>,
    #[prop(into)] jogos_time_b: Signal<Vec<DetalheJogo>>,
    // Simplificamos: Usamos #[prop(into)] para aceitar tanto Callback quanto Option<Callback>
    #[prop(into, optional)] on_click_jogo_a: Option<Callback<usize>>,
) -> impl IntoView {
    let handle_click = move |idx: usize| {
        if let Some(cb) = on_click_jogo_a {
            cb.run(idx);
        }
    };

    let render_lado = move |titulo: String, jogos: Vec<DetalheJogo>, interativo: bool| {
        view! {
            <div style="flex: 1; border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; padding: 10px; margin: 5px; min-width: 300px;">
                <h3 style="margin-top: 0; color: #ffeb3b; font-size: 14px; text-transform: uppercase;">{titulo}</h3>

                <div style="display: flex; flex-wrap: wrap; gap: 15px;">
                    {if jogos.is_empty() {
                        view! { <span style="font-size: 12px; color: rgba(255,255,255,0.4);">"Nenhum jogo baixado"</span> }.into_any()
                    } else {
                        jogos.into_iter().enumerate().map(|(idx, jogo)| {
                            view! {
                                <div
                                    on:click=move |_| if interativo { handle_click(idx) }
                                    style="
                                        background: rgba(0,0,0,0.2); 
                                        padding: 5px; 
                                        border-radius: 8px;
                                        display: flex;
                                        align-items: center;
                                        cursor: pointer;
                                        transition: background 0.2s;
                                        border: 1px solid transparent;
                                    "
                                    // Removemos os eventos mouseenter/mouseleave que causavam erro
                                    // Adicionei uma borda transparente acima para estrutura
                                >
                                    {jogo.cartas.into_iter().map(|c| {
                                        view! {
                                            <div style="margin-right: -25px;">
                                                <Card
                                                    id=carta_para_asset(&c)
                                                    width="60px".to_string()
                                                    selection_group=Signal::derive(|| None)
                                                />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                    <div style="width: 25px;"></div>
                                </div>
                            }
                        }).collect::<Vec<_>>().into_any()
                    }}
                </div>
            </div>
        }
    };

    view! {
        <div style="width: 100%; display: flex; flex-wrap: wrap; justify-content: space-between; gap: 10px; margin-bottom: 20px;">
            {move || render_lado("Mesa Time A".to_string(), jogos_time_a.get(), true)}
            {move || render_lado("Mesa Time B".to_string(), jogos_time_b.get(), false)}
        </div>
    }
}

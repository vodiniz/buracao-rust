use crate::components::card::Card;
use crate::utils::mappers::carta_para_asset;
use buracao_core::acoes::DetalheJogo;
use leptos::prelude::*;

#[component]
pub fn Table(
    #[prop(into)] jogos_time_a: Signal<Vec<DetalheJogo>>,
    #[prop(into)] jogos_time_b: Signal<Vec<DetalheJogo>>,

    // CORREÇÃO AQUI:
    // Trocamos #[prop(optional)] por #[prop(default = None)]
    // Isso diz ao Leptos: "Espere receber um Option<Callback>. Se não receber nada, use None."
    // Assim, os tipos batem com o que você criou no app.rs (cb_a e cb_b).
    #[prop(default = None)] on_click_jogo_a: Option<Callback<usize>>,
    #[prop(default = None)] on_click_jogo_b: Option<Callback<usize>>,
) -> impl IntoView {
    let render_lado = move |titulo: String,
                            jogos: Vec<DetalheJogo>,
                            callback: Option<Callback<usize>>| {
        let interativo = callback.is_some();

        view! {
            <div style="flex: 1; border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; padding: 10px; margin: 5px; min-width: 300px;">
                <h3 style="margin-top: 0; color: #ffeb3b; font-size: 14px; text-transform: uppercase;">{titulo}</h3>

                <div style="display: flex; flex-wrap: wrap; gap: 15px;">
                    {if jogos.is_empty() {
                        view! { <span style="font-size: 12px; color: rgba(255,255,255,0.4);">"Nenhum jogo baixado"</span> }.into_any()
                    } else {
                        jogos.into_iter().enumerate().map(|(idx, jogo)| {
                            // Clona o callback para mover para dentro da closure
                            let cb_local = callback;

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
                                            padding: 5px; 
                                            border-radius: 8px;
                                            display: flex;
                                            align-items: center;
                                            cursor: {};
                                            transition: background 0.2s;
                                            border: 1px solid transparent;
                                        ", cursor)
                                    }
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
            {move || render_lado("Mesa Time A".to_string(), jogos_time_a.get(), on_click_jogo_a)}
            {move || render_lado("Mesa Time B".to_string(), jogos_time_b.get(), on_click_jogo_b)}
        </div>
    }
}

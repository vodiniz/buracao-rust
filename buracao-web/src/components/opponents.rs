use crate::components::card::Card;
use leptos::prelude::*;

#[component]
pub fn OpponentHand(
    #[prop(optional, into, default = "/assets/cards/PaperCards1.1".to_string())] theme: String,
    posicao: &'static str, // "top", "left", "right"
    #[prop(into)] qtd_cartas: usize,
) -> impl IntoView {
    // Define a rotação e layout baseada na posição
    let style_container = match posicao {
        "left" => "display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;",
        "right" => "display: flex; flex-direction: column; align-items: center; height: 100%; justify-content: center;",
        _ => "display: flex; flex-direction: row; justify-content: center; width: 100%;", // top
    };

    let style_card = match posicao {
        "left" => "margin-top: -80px; transform: rotate(90deg);",
        "right" => "margin-top: -80px; transform: rotate(-90deg);",
        _ => "margin-right: -40px;", // top
    };

    view! {
        <div style=style_container>
            <div style="background: rgba(0,0,0,0.5); padding: 5px; border-radius: 8px; color: white; font-size: 12px; margin-bottom: 5px;">
                {format!("{} cartas", qtd_cartas)}
            </div>

            // Renderiza apenas versos
            {
                (0..qtd_cartas).map(|i| view! {
                    <div style=move || if i == 0 { style_card.replace("margin-top: -80px;", "").replace("margin-right: -40px;", "") } else { style_card.to_string() }>
                        <Card
                            id="back_r".to_string() // Verso Vermelho
                            width="80px".to_string()
                            selection_group=Signal::derive(|| None)
                            theme=theme.clone()
                        />
                    </div>
                }).collect::<Vec<_>>()
            }
        </div>
    }
}

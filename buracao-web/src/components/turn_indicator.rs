use leptos::prelude::*;
use std::collections::HashMap;

#[component]
fn TurnDot(
    #[prop(into)] is_me: Signal<bool>,
    #[prop(into)] is_turn: Signal<bool>,
    rgb: (u8, u8, u8),
    #[prop(into)] label: Signal<String>,
) -> impl IntoView {
    let (r, g, b) = rgb;

    view! {
        <div
            // O texto do tooltip (demora ~1s para aparecer no navegador)
            title=move || label.get()
            style=move || {
                let turn = is_turn.get();
                let me = is_me.get();

                let alpha = if turn { 1.0 } else { 0.3 };
                let bg_color = format!("rgba({}, {}, {}, {})", r, g, b, alpha);

                let border = if me { "3px solid #ffeb3b" } else { "3px solid transparent" };

                let shadow_alpha = if turn { 0.8 } else { 0.0 };
                // Aumentei o blur da sombra para acompanhar o tamanho maior
                let shadow = format!("0 0 25px rgba({}, {}, {}, {})", r, g, b, shadow_alpha);

                let transform = if turn { "scale(1.25)" } else { "scale(1.0)" };

                format!("
                    width: 40px;  
                    height: 40px; 
                    border-radius: 50%;
                    box-sizing: border-box;
                    background-color: {};
                    border: {};
                    box-shadow: {};
                    transform: {};
                    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
                    cursor: pointer;
                ", bg_color, border, shadow, transform)
            }
        ></div>
    }
}

#[component]
pub fn TurnIndicator(
    #[prop(into)] my_id: Signal<u32>,
    #[prop(into)] current_turn: Signal<u32>,
    names: ReadSignal<HashMap<u32, String>>,
    cards_count: ReadSignal<Vec<usize>>,
) -> impl IntoView {
    // Cores Base em RGB
    let blue_rgb = (41, 182, 246); // #29b6f6
    let orange_rgb = (255, 152, 0); // #ff9800
    let green_rgb = (76, 175, 80); // #4caf50
    let red_rgb = (244, 67, 54); // #f44336

    // Helper para criar o texto do label dinamicamente
    let make_label = move |target_id: u32| {
        Signal::derive(move || {
            let all_names = names.get();
            let all_counts = cards_count.get();

            let name = all_names
                .get(&target_id)
                .cloned()
                .unwrap_or(format!("Jogador {}", target_id));
            let count = all_counts.get(target_id as usize).copied().unwrap_or(0);

            format!("{}\n{} Cartas", name, count)
        })
    };

    view! {
        <div style="
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            grid-template-rows: 1fr 1fr 1fr;
            gap: 20px;    /* Aumentado o gap para espalhar as bolinhas */
            width: 180px; /* Aumentado de 80px para 180px (Tamanho aproximado do Monte+Lixo) */
            height: 180px;/* Aumentado de 80px para 180px (Para manter circular) */
            align-items: center;
            justify-items: center;
        ">
            // TOPO (Norte) - Jogador 2 (Azul)
            <div style="grid-column: 2; grid-row: 1;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 2)
                        is_turn=Signal::derive(move || current_turn.get() == 2)
                        rgb=blue_rgb
                        label=make_label(2)
                    />
                }}
            </div>

            // ESQUERDA (Oeste) - Jogador 3 (Laranja)
            <div style="grid-column: 1; grid-row: 2;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 3)
                        is_turn=Signal::derive(move || current_turn.get() == 3)
                        rgb=orange_rgb
                        label=make_label(3)
                    />
                }}
            </div>

            // DIREITA (Leste) - Jogador 1 (Verde)
            <div style="grid-column: 3; grid-row: 2;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 1)
                        is_turn=Signal::derive(move || current_turn.get() == 1)
                        rgb=green_rgb
                        label=make_label(1)
                    />
                }}
            </div>

            // BAIXO (Sul) - Jogador 0 (Vermelho)
            <div style="grid-column: 2; grid-row: 3;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 0)
                        is_turn=Signal::derive(move || current_turn.get() == 0)
                        rgb=red_rgb
                        label=make_label(0)
                    />
                }}
            </div>
        </div>
    }
}

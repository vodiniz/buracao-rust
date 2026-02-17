use leptos::prelude::*;
use std::collections::HashMap;

#[component]
fn TurnDot(
    #[prop(into)] is_me: Signal<bool>,
    #[prop(into)] is_turn: Signal<bool>,
    rgb: (u8, u8, u8),
    #[prop(into)] label: Signal<String>,
    #[prop(into)] count: Signal<usize>, // <--- NOVO: Recebe o número de cartas
) -> impl IntoView {
    let (r, g, b) = rgb;

    view! {
        <div
            // O tooltip continua mostrando o nome completo ao passar o mouse
            title=move || label.get()
            style=move || {
                let turn = is_turn.get();
                let me = is_me.get();

                let alpha = if turn { 1.0 } else { 0.6 }; // Aumentei um pouco a opacidade base para ler o número
                let bg_color = format!("rgba({}, {}, {}, {})", r, g, b, alpha);

                let border = if me { "3px solid #ffeb3b" } else { "3px solid transparent" };

                let shadow_alpha = if turn { 0.8 } else { 0.0 };
                let shadow = format!("0 0 25px rgba({}, {}, {}, {})", r, g, b, shadow_alpha);

                let transform = if turn { "scale(1.25)" } else { "scale(1.0)" };

                // CSS ATUALIZADO: Flexbox para centralizar e Fonte Charmosa
                format!("
                    width: 45px;  
                    height: 45px; 
                    border-radius: 50%;
                    box-sizing: border-box;
                    background-color: {};
                    border: {};
                    box-shadow: {};
                    transform: {};
                    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
                    
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    
                    color: white;
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    font-weight: bold;
                    font-size: 1.1rem;
                    text-shadow: 0px 1px 3px rgba(0,0,0,0.8);
                    user-select: none;
                ", bg_color, border, shadow, transform)
            }
        >
            // Exibe o número no centro
            {move || count.get()}
        </div>
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

    // Helper para criar o texto do Tooltip (Nome + Cartas)
    let make_label = move |target_id: u32| {
        Signal::derive(move || {
            let all_names = names.get();
            let all_counts = cards_count.get();

            let name = all_names
                .get(&target_id)
                .cloned()
                .unwrap_or(format!("Jogador {}", target_id));
            let count = all_counts.get(target_id as usize).copied().unwrap_or(0);

            format!("{} ({} Cartas)", name, count)
        })
    };

    // Helper para pegar APENAS o número de cartas (para o centro da bolinha)
    let get_count = move |target_id: u32| {
        Signal::derive(move || {
            cards_count
                .get()
                .get(target_id as usize)
                .copied()
                .unwrap_or(0)
        })
    };

    view! {
        <div style="
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            grid-template-rows: 1fr 1fr 1fr;
            gap: 20px;    
            width: 180px; 
            height: 180px;
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
                        count=get_count(2) // Passando o count
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
                        count=get_count(3)
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
                        count=get_count(1)
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
                        count=get_count(0)
                    />
                }}
            </div>
        </div>
    }
}

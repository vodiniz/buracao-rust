use leptos::prelude::*;
use std::collections::HashMap;

#[component]
fn TurnDot(
    #[prop(into)] is_me: Signal<bool>,
    #[prop(into)] is_turn: Signal<bool>,
    rgb: (u8, u8, u8), // Cor do time/posição (borda)
    #[prop(into)] label: Signal<String>,
    #[prop(into)] count: Signal<usize>,
    #[prop(into)] name_seed: Signal<String>, // <--- NOVO: Semente para o Avatar
) -> impl IntoView {
    let (r, g, b) = rgb;

    // Gera a URL do Avatar baseada no nome (determinístico)
    // Estilos legais do DiceBear: 'adventurer', 'lorelei', 'notionists', 'fun-emoji'
    let avatar_src = move || {
        let seed = name_seed.get();
        // Usando 'adventurer' pois tem rostos expressivos e funciona bem pequeno
        format!("https://api.dicebear.com/9.x/adventurer/svg?seed={}&backgroundColor=b6e3f4,c0aede,d1d4f9", seed)
    };

    view! {
        <div
            title=move || label.get()
            style=move || {
                let turn = is_turn.get();
                let me = is_me.get();

                // Borda colorida indica o time/posição
                // Se for o turno, a borda brilha. Se sou eu, borda dourada extra.
                let border_color = if me { "gold" } else {
                    &format!("rgb({},{},{})", r, g, b)
                };

                let border_width = if turn { "4px" } else { "2px" };
                let shadow = if turn {
                    format!("0 0 15px 2px rgba({},{},{}, 0.8)", r, g, b)
                } else {
                    "0 4px 6px rgba(0,0,0,0.3)".to_string()
                };

                let transform = if turn { "scale(1.35)" } else { "scale(1.0)" };
                let opacity = if turn { "1.0" } else { "0.85" };

                format!("
                    width: 55px;  
                    height: 55px; 
                    border-radius: 50%;
                    background-color: #222; /* Fundo caso imagem falhe */
                    border: {} solid {};
                    box-shadow: {};
                    transform: {};
                    opacity: {};
                    transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
                    position: relative;
                    cursor: default;
                ", border_width, border_color, shadow, transform, opacity)
            }
        >
            // 1. O Avatar (Imagem de fundo)
            <img
                src=avatar_src
                alt="Avatar"
                style="
                    width: 100%; 
                    height: 100%; 
                    border-radius: 50%; 
                    object-fit: cover;
                "
            />

            // 2. Contador de Cartas (Badge flutuante)
            <div style="
                position: absolute;
                bottom: -5px;
                right: -5px;
                background: rgba(0, 0, 0, 0.85);
                color: white;
                font-size: 11px;
                font-weight: bold;
                padding: 2px 6px;
                border-radius: 10px;
                border: 1px solid rgba(255,255,255,0.2);
                box-shadow: 0 2px 4px rgba(0,0,0,0.5);
                font-family: sans-serif;
            ">
                {move || count.get()}
            </div>
        </div>
    }
}

#[component]
pub fn TurnIndicator(
    #[prop(into)] my_id: Signal<u32>,
    #[prop(into)] current_turn: Signal<u32>,
    #[prop(into)] names: Signal<HashMap<u32, String>>,
    #[prop(into)] cards_count: Signal<Vec<usize>>,
) -> impl IntoView {
    // Cores (Mantidas para a borda)
    let blue_rgb = (41, 182, 246);
    let orange_rgb = (255, 152, 0);
    let green_rgb = (76, 175, 80);
    let red_rgb = (244, 67, 54);

    // Helper: Pega o nome "puro" para gerar o avatar
    let get_name_raw = move |target_id: u32| {
        Signal::derive(move || {
            names
                .get()
                .get(&target_id)
                .cloned()
                .unwrap_or(format!("Player{}", target_id))
        })
    };

    // Helper: Texto do Tooltip
    let make_label = move |target_id: u32| {
        Signal::derive(move || {
            let n = names.get();
            let c = cards_count.get();
            let name = n
                .get(&target_id)
                .cloned()
                .unwrap_or(format!("Jogador {}", target_id));
            let count = c.get(target_id as usize).copied().unwrap_or(0);
            format!("{} ({} Cartas)", name, count)
        })
    };

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
            gap: 15px;    
            width: 200px; 
            height: 200px;
            align-items: center;
            justify-items: center;
            /* Um fundo sutil para conectar os jogadores visualmente, opcional */
            background: radial-gradient(circle, rgba(255,255,255,0.05) 0%, transparent 70%);
            border-radius: 50%;
        ">
            // TOPO (Norte) - Jogador 2
            <div style="grid-column: 2; grid-row: 1;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 2)
                        is_turn=Signal::derive(move || current_turn.get() == 2)
                        rgb=blue_rgb
                        label=make_label(2)
                        count=get_count(2)
                        name_seed=get_name_raw(2) // Passa o nome para o avatar
                    />
                }}
            </div>

            // ESQUERDA (Oeste) - Jogador 3
            <div style="grid-column: 1; grid-row: 2;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 3)
                        is_turn=Signal::derive(move || current_turn.get() == 3)
                        rgb=orange_rgb
                        label=make_label(3)
                        count=get_count(3)
                        name_seed=get_name_raw(3)
                    />
                }}
            </div>

            // DIREITA (Leste) - Jogador 1
            <div style="grid-column: 3; grid-row: 2;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 1)
                        is_turn=Signal::derive(move || current_turn.get() == 1)
                        rgb=green_rgb
                        label=make_label(1)
                        count=get_count(1)
                        name_seed=get_name_raw(1)
                    />
                }}
            </div>

            // BAIXO (Sul) - Jogador 0
            <div style="grid-column: 2; grid-row: 3;">
                {move || view! {
                    <TurnDot
                        is_me=Signal::derive(move || my_id.get() == 0)
                        is_turn=Signal::derive(move || current_turn.get() == 0)
                        rgb=red_rgb
                        label=make_label(0)
                        count=get_count(0)
                        name_seed=get_name_raw(0)
                    />
                }}
            </div>
        </div>
    }
}

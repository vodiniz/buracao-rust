use leptos::prelude::*;

// 1. Função Pura: Define a cor baseada no ID (não muda)
fn get_player_color(id: u32) -> &'static str {
    match id % 4 {
        0 => "#f44336", // Vermelho
        1 => "#4caf50", // Verde
        2 => "#29b6f6", // Azul
        3 => "#ff9800", // Laranja
        _ => "white",
    }
}

// 2. Sub-componente para a Bolinha (Resolve o erro de closure)
// Recebe um Signal ou bool para reatividade limpa
#[component]
fn TurnDot(#[prop(into)] is_active: Signal<bool>, player_id: u32) -> impl IntoView {
    let base_color = get_player_color(player_id);

    view! {
        <div
            title=format!("Jogador {}", player_id)
            style=move || {
                let active = is_active.get();
                // Lógica visual: Se ativo, brilha; se inativo, fica fosco
                let opacity = if active { "1.0" } else { "0.3" };
                let border = if active { "2px solid #ffeb3b" } else { "2px solid rgba(0,0,0,0.2)" };
                let transform = if active { "scale(1.3)" } else { "scale(1.0)" };
                let shadow = if active { format!("0 0 15px {}", base_color) } else { "none".to_string() };

                format!("
                    width: 14px;
                    height: 14px;
                    border-radius: 50%;
                    background-color: {};
                    border: {};
                    box-shadow: {};
                    transform: {};
                    opacity: {};
                    transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
                ", base_color, border, shadow, transform, opacity)
            }
        ></div>
    }
}

// 3. Componente Principal
#[component]
pub fn TurnIndicator(
    #[prop(into)] my_id: Signal<u32>,
    #[prop(into)] current_turn: Signal<u32>,
) -> impl IntoView {
    view! {
        <div style="
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            grid-template-rows: 1fr 1fr 1fr;
            gap: 8px;
            width: 70px;
            height: 70px;
            align-items: center;
            justify-items: center;
        ">
            // --- TOPO: Parceiro (Eu + 2) ---
            <div style="grid-column: 2; grid-row: 1;">
                {move || {
                    let eu = my_id.get();
                    let parceiro_id = (eu + 2) % 4;
                    // Usamos Signal::derive para passar a reatividade para o sub-componente
                    view! {
                        <TurnDot
                            is_active=Signal::derive(move || current_turn.get() == parceiro_id)
                            player_id=parceiro_id
                        />
                    }
                }}
            </div>

            // --- ESQUERDA: Anterior (Eu + 3) ---
            // Sentido anti-horário de "quem jogou antes" na esquerda
            <div style="grid-column: 1; grid-row: 2;">
                {move || {
                    let eu = my_id.get();
                    let anterior_id = (eu + 3) % 4;
                    view! {
                        <TurnDot
                            is_active=Signal::derive(move || current_turn.get() == anterior_id)
                            player_id=anterior_id
                        />
                    }
                }}
            </div>

            // --- DIREITA: Próximo (Eu + 1) ---
            <div style="grid-column: 3; grid-row: 2;">
                {move || {
                    let eu = my_id.get();
                    let proximo_id = (eu + 1) % 4;
                    view! {
                        <TurnDot
                            is_active=Signal::derive(move || current_turn.get() == proximo_id)
                            player_id=proximo_id
                        />
                    }
                }}
            </div>

            // --- BAIXO: Eu Mesmo ---
            <div style="grid-column: 2; grid-row: 3;">
                {move || {
                    let eu = my_id.get();
                    view! {
                        <TurnDot
                            is_active=Signal::derive(move || current_turn.get() == eu)
                            player_id=eu
                        />
                    }
                }}
            </div>
        </div>
    }
}

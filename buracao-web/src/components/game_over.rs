use leptos::ev;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn GameOverModal(
    #[prop(into)] show: RwSignal<bool>,
    #[prop(into)] data: RwSignal<Option<(u8, i32, i32, String)>>,
    #[prop(into)] my_id: Signal<u32>,
    #[prop(into)] nomes: Signal<HashMap<u32, String>>,
) -> impl IntoView {
    // Adiciona um listener global para a tecla Escape
    let window_event = window_event_listener(ev::keydown, move |ev| {
        if ev.key() == "Escape" && show.get() {
            show.set(false);
        }
    });
    let styles = view! {
        <style>
            "@keyframes confetti-fall {
                0% { transform: translateY(-100vh) rotate(0deg); opacity: 1; }
                100% { transform: translateY(100vh) rotate(720deg); opacity: 0; }
            }
            .confetti {
                position: absolute;
                top: 0;
                width: 10px;
                height: 10px;
                animation: confetti-fall 4s linear infinite;
            }"
        </style>
    };

    let render_confetti = move || {
        (0..50).map(|i| {
            let left = format!("{}%", (rand::random::<f32>() * 100.0));
            let delay = format!("{}s", (rand::random::<f32>() * 3.0));
            let color = if i % 3 == 0 { "#ff5252" } else if i % 3 == 1 { "#448aff" } else { "#ffd700" };
            view! { <div class="confetti" style=format!("left: {}; animation-delay: {}; background: {};", left, delay, color)></div> }
        }).collect::<Vec<_>>()
    };

    let get_team_names = move |team_idx: u8| {
        let n = nomes.get();
        if team_idx == 0 {
            let p1 = n.get(&0).cloned().unwrap_or("Jog. 1".to_string());
            let p2 = n.get(&2).cloned().unwrap_or("Jog. 3".to_string());
            format!("{} / {}", p1, p2)
        } else {
            let p1 = n.get(&1).cloned().unwrap_or("Jog. 2".to_string());
            let p2 = n.get(&3).cloned().unwrap_or("Jog. 4".to_string());
            format!("{} / {}", p1, p2)
        }
    };

    view! {
        {styles}
        <Show when=move || show.get() fallback=|| ()>
            <div
                on:click=move |_| show.set(false)
                style="
                position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
                background: rgba(0,0,0,0.9); z-index: 3000;
                display: flex; justify-content: center; align-items: center;
                backdrop-filter: blur(10px);
                overflow: hidden;
            ">
                // Só renderiza confetes se o usuário ganhou
                {move || {
                    let is_winner = data.get().map(|(v, ..)| (my_id.get() % 2) as u8 == v).unwrap_or(false);
                    if is_winner {
                        view! { <div style="position: absolute; width: 100%; height: 100%; pointer-events: none;">{render_confetti()}</div> }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }
                }}

                {move || {
                    if let Some((vencedor, pts_a, pts_b, _motivo)) = data.get() {
                        let meu_time = (my_id.get() % 2) as u8;
                        let ganhei = meu_time == vencedor;

                        // Alteração de Título e Emojis conforme Vitória ou Derrota
                        let titulo = if ganhei { "VITÓRIA!" } else { "DERROTA" };
                        let cor_titulo = if ganhei { "#ffd700" } else { "#ff5252" };
                        let emoji = if ganhei { "🏆" } else { "😔" };
                        let sub_texto = if ganhei { "Parabéns pela partida!" } else { "Não foi dessa vez..." };

                        view! {
                            <div style="
                                background: linear-gradient(135deg, #1a1a1a 0%, #222 100%);
                                border: 2px solid rgba(255,255,255,0.1);
                                padding: 50px;
                                border-radius: 24px;
                                text-align: center;
                                box-shadow: 0 30px 60px rgba(0,0,0,0.8);
                                color: white;
                                min-width: 550px;
                                transform: scale(1.1);
                                font-family: 'Segoe UI', system-ui, sans-serif;
                                position: relative;
                                z-index: 3001;
                            ">
                                <div style="font-size: 80px; margin-bottom: 20px; filter: drop-shadow(0 5px 15px rgba(0,0,0,0.5));">{emoji}</div>
                                <h1 style=format!("color: {}; font-size: 3.5rem; margin: 0; text-transform: uppercase; letter-spacing: 4px; font-weight: 900;", cor_titulo)>
                                    {titulo}
                                </h1>
                                <p style="color: #888; font-size: 1.2rem; margin-top: 10px; margin-bottom: 40px; font-weight: 500;">
                                    {sub_texto}
                                </p>

                                <div style="display: flex; justify-content: space-around; background: rgba(255,255,255,0.03); padding: 25px; border-radius: 16px; margin-bottom: 30px; border: 1px solid rgba(255,255,255,0.05);">

                                    // COLUNA TIME A
                                    <div style=move || format!("display: flex; flex-direction: column; align-items: center; opacity: {};", if vencedor == 0 { "1.0" } else { "0.5" })>
                                        <span style="font-size: 0.85rem; color: #90caf9; font-weight: 800; text-transform: uppercase; margin-bottom: 8px; letter-spacing: 1px;">
                                            {get_team_names(0)}
                                        </span>
                                        <span style=format!("font-size: 3rem; font-weight: 900; color: {};", if vencedor == 0 { "#fff" } else { "#aaa" })>{pts_a}</span>
                                        {if vencedor == 0 { view! { <span style="font-size: 10px; color: #ffd700; margin-top: 5px;">"VENCEDOR"</span> }.into_any() } else { view! { <div /> }.into_any() }}
                                    </div>

                                    <div style="width: 1px; background: rgba(255,255,255,0.1); margin: 0 20px;"></div>

                                    // COLUNA TIME B
                                    <div style=move || format!("display: flex; flex-direction: column; align-items: center; opacity: {};", if vencedor == 1 { "1.0" } else { "0.5" })>
                                        <span style="font-size: 0.85rem; color: #ffcc80; font-weight: 800; text-transform: uppercase; margin-bottom: 8px; letter-spacing: 1px;">
                                            {get_team_names(1)}
                                        </span>
                                        <span style=format!("font-size: 3rem; font-weight: 900; color: {};", if vencedor == 1 { "#fff" } else { "#aaa" })>{pts_b}</span>
                                        {if vencedor == 1 { view! { <span style="font-size: 10px; color: #ffd700; margin-top: 5px;">"VENCEDOR"</span> }.into_any() } else { view! { <div /> }.into_any() }}
                                    </div>
                                </div>

                                <div style="display: flex; align-items: center; justify-content: center; gap: 10px; color: #555;">
                                    <div class="loader" style="width: 14px; height: 14px; border: 2px solid #333; border-top: 2px solid #888; border-radius: 50%; animation: spin 1s linear infinite;"></div>
                                    <span style="font-size: 0.9rem; font-weight: 600; letter-spacing: 0.5px;">
                                        "REINICIANDO MESA..."
                                    </span>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div style="color: white; font-weight: bold;">"Aguardando resultados..."</div> }.into_any()
                    }
                }}
            </div>
        </Show>
        <style>
            "@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }"
        </style>
    }
}

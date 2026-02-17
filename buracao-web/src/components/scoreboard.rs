use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn Scoreboard(
    #[prop(into)] pontuacao_a: Signal<i32>,
    #[prop(into)] pontuacao_b: Signal<i32>,
    #[prop(into)] historico_a: Signal<Vec<i32>>,
    #[prop(into)] historico_b: Signal<Vec<i32>>,
    #[prop(into)] nomes: Signal<HashMap<u32, String>>,
    #[prop(into)] my_id: Signal<u32>,
) -> impl IntoView {
    // Estado para controlar se o modal está aberto
    let (show_history, set_show_history) = signal(false);

    // Estado local para efeito de hover/click (opcional, mas purista em Rust)
    let (is_pressed, set_pressed) = signal(false);

    view! {
        // --- PLACAR RESUMIDO ---
        <div
            on:click=move |_| set_show_history.set(true)
            // Eventos para simular o efeito de clique visualmente
            on:mousedown=move |_| set_pressed.set(true)
            on:mouseup=move |_| set_pressed.set(false)
            on:mouseleave=move |_| set_pressed.set(false)

            style=move || format!("
                cursor: pointer;
                display: flex;
                flex-direction: column;
                background: rgba(0, 0, 0, 0.6);
                padding: 10px 15px;
                border-radius: 8px;
                color: white;
                font-family: sans-serif;
                min-width: 140px;
                box-shadow: 0 4px 6px rgba(0,0,0,0.3);
                border: 1px solid rgba(255,255,255,0.1);
                transition: transform 0.1s;
                transform: scale({});
            ", if is_pressed.get() { "0.95" } else { "1.0" }) // <--- Correção aqui: estilo dinâmico

            title="Clique para ver o histórico detalhado"
        >
            <div style="font-size: 10px; text-transform: uppercase; letter-spacing: 1.5px; color: #aaa; margin-bottom: 8px; text-align: center; font-weight: bold;">
                "PLACAR (VER DETALHES)"
            </div>

            {move || {
                let id = my_id.get();
                let p_a = pontuacao_a.get();
                let p_b = pontuacao_b.get();

                let sou_time_a = id.is_multiple_of(2);
                let cor_meu = "#90caf9";
                let cor_inimigo = "#ffcc80";

                let (pontos_meu, pontos_inimigo) = if sou_time_a {
                    (p_a, p_b)
                } else {
                    (p_b, p_a)
                };

                view! {
                    <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 5px; margin-bottom: 5px;">
                        <span style=format!("color: {}; font-weight: bold; font-size: 11px;", cor_meu)>"MEU TIME"</span>
                        <span style="font-size: 18px; font-weight: bold;">{pontos_meu}</span>
                    </div>

                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span style=format!("color: {}; font-weight: bold; font-size: 11px;", cor_inimigo)>"INIMIGO"</span>
                        <span style="font-size: 18px; font-weight: bold;">{pontos_inimigo}</span>
                    </div>
                }
            }}
        </div>

        // --- MODAL DE HISTÓRICO ---
        <Show when=move || show_history.get()>
            <ScoreHistoryModal
                historico_a=historico_a
                historico_b=historico_b
                nomes=nomes
                on_close=Callback::new(move |_| set_show_history.set(false))
            />
        </Show>
    }
}

// --- SUB-COMPONENTE DO MODAL ---
#[component]
fn ScoreHistoryModal(
    historico_a: Signal<Vec<i32>>,
    historico_b: Signal<Vec<i32>>,
    nomes: Signal<HashMap<u32, String>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let get_team_names = move |is_team_a: bool| {
        let n = nomes.get();
        if is_team_a {
            let p1 = n.get(&0).cloned().unwrap_or("P1".to_string());
            let p2 = n.get(&2).cloned().unwrap_or("P3".to_string());
            format!("{} & {}", p1, p2)
        } else {
            let p1 = n.get(&1).cloned().unwrap_or("P2".to_string());
            let p2 = n.get(&3).cloned().unwrap_or("P4".to_string());
            format!("{} & {}", p1, p2)
        }
    };

    // Estado para o botão de fechar (hover)
    let (hover_btn, set_hover_btn) = signal(false);

    view! {
        <div
            style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); display: flex; justify-content: center; align-items: center; z-index: 1000;"
            on:click=move |_| on_close.run(())
        >
            <div
                style="background: #1e1e1e; padding: 20px; border-radius: 12px; min-width: 300px; max-width: 90%; color: white; border: 1px solid #333; box-shadow: 0 10px 25px rgba(0,0,0,0.5);"
                // CORREÇÃO DO ERRO E0282: Tipando explicitamente o evento
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
            >
                <h3 style="margin-top: 0; text-align: center; border-bottom: 1px solid #444; padding-bottom: 10px;">
                    "Histórico da Partida"
                </h3>

                <div style="max_height: 60vh; overflow-y: auto;">
                    <table style="width: 100%; border-collapse: collapse; font-size: 14px;">
                        <thead>
                            <tr style="color: #aaa; font-size: 12px; text-transform: uppercase;">
                                <th style="padding: 8px; text-align: center;">"Rodada"</th>
                                <th style="padding: 8px; text-align: center; color: #90caf9;">
                                    <div>"TIME A"</div>
                                    <div style="font-size: 9px; opacity: 0.7;">{move || get_team_names(true)}</div>
                                </th>
                                <th style="padding: 8px; text-align: center; color: #ffcc80;">
                                    <div>"TIME B"</div>
                                    <div style="font-size: 9px; opacity: 0.7;">{move || get_team_names(false)}</div>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let ha = historico_a.get();
                                let hb = historico_b.get();
                                let max_len = ha.len().max(hb.len());

                                (0..max_len).map(|i| {
                                    let score_a = ha.get(i).copied().unwrap_or(0);
                                    let score_b = hb.get(i).copied().unwrap_or(0);

                                    let color_a = if score_a < 0 { "#ff5252" } else { "#fff" };
                                    let color_b = if score_b < 0 { "#ff5252" } else { "#fff" };

                                    view! {
                                        <tr style="border-bottom: 1px solid #333;">
                                            <td style="padding: 8px; text-align: center; color: #666;">{i + 1}</td>
                                            <td style=format!("padding: 8px; text-align: center; color: {}; font-weight: bold;", color_a)>
                                                {score_a}
                                            </td>
                                            <td style=format!("padding: 8px; text-align: center; color: {}; font-weight: bold;", color_b)>
                                                {score_b}
                                            </td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()
                            }}
                        </tbody>
                        <tfoot>
                            <tr style="background: #252525; font-weight: bold;">
                                <td style="padding: 10px; text-align: center;">"TOTAL"</td>
                                <td style="padding: 10px; text-align: center; color: #90caf9;">
                                    {move || historico_a.get().iter().sum::<i32>()}
                                </td>
                                <td style="padding: 10px; text-align: center; color: #ffcc80;">
                                    {move || historico_b.get().iter().sum::<i32>()}
                                </td>
                            </tr>
                        </tfoot>
                    </table>
                </div>

                <button
                    on:click=move |_| on_close.run(())
                    // CORREÇÃO: Usando Signals para hover em vez de JS string
                    on:mouseover=move |_| set_hover_btn.set(true)
                    on:mouseout=move |_| set_hover_btn.set(false)
                    style=move || format!("
                        margin-top: 15px; 
                        width: 100%; 
                        padding: 10px; 
                        background: {}; 
                        color: white; 
                        border: none; 
                        border-radius: 4px; 
                        cursor: pointer; 
                        font-weight: bold;
                        transition: background 0.2s;
                    ", if hover_btn.get() { "#444" } else { "#333" })
                >
                    "FECHAR"
                </button>
            </div>
        </div>
    }
}

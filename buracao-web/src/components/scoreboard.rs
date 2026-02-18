use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn Scoreboard(
    #[prop(into)] pontuacao_a: Signal<i32>,
    #[prop(into)] pontuacao_b: Signal<i32>,
    // --- NOVOS CAMPOS QUE FALTAVAM ---
    #[prop(into)] historico_a: Signal<Vec<i32>>,
    #[prop(into)] historico_b: Signal<Vec<i32>>,
    #[prop(into)] nomes: Signal<HashMap<u32, String>>,
    // ---------------------------------
    #[prop(into)] my_id: Signal<u32>,
    #[prop(into)] on_click_expand: Callback<()>,
) -> impl IntoView {
    let (is_pressed, set_pressed) = signal(false);

    view! {
        <div
            on:click=move |_| on_click_expand.run(())
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
                user-select: none;
            ", if is_pressed.get() { "0.95" } else { "1.0" })
            title="Clique ou pressione 'P' para ver o histórico"
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

                let (pontos_meu, pontos_inimigo) = if sou_time_a { (p_a, p_b) } else { (p_b, p_a) };

                view! {
                    <>
                        <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 5px; margin-bottom: 5px;">
                            <span style=format!("color: {}; font-weight: bold; font-size: 11px;", cor_meu)>"MEU TIME"</span>
                            <span style="font-size: 18px; font-weight: bold;">{pontos_meu}</span>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style=format!("color: {}; font-weight: bold; font-size: 11px;", cor_inimigo)>"INIMIGO"</span>
                            <span style="font-size: 18px; font-weight: bold;">{pontos_inimigo}</span>
                        </div>
                    </>
                }
            }}
        </div>
    }
}

// MUDANÇA IMPORTANTE: Adicione 'pub' aqui para podermos usar no App.rs
#[component]
pub fn ScoreHistoryModal(
    #[prop(into)] historico_a: Signal<Vec<i32>>,
    #[prop(into)] historico_b: Signal<Vec<i32>>,
    #[prop(into)] nomes: Signal<HashMap<u32, String>>,
    #[prop(into)] my_id: Signal<u32>,
    #[prop(default = 1.0)] scale: f64,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    // ... (O código interno do Modal permanece IDÊNTICO ao que você já tem) ...
    // ... Apenas certifique-se de que ele está neste arquivo ...

    // (Vou omitir o corpo para economizar espaço, mantenha o seu código atual do Modal aqui)
    // Se precisar do código do modal novamente, me avise.

    let (hover_btn, set_hover_btn) = signal(false);
    let sou_time_a = move || my_id.get().is_multiple_of(2);

    // Helper de nomes (igual ao anterior)
    let get_names_for_team = move |is_team_a: bool| {
        let n = nomes.get();
        if is_team_a {
            let p1 = n.get(&0).cloned().unwrap_or("Jog. 1".to_string());
            let p2 = n.get(&2).cloned().unwrap_or("Jog. 3".to_string());
            format!("{} & {}", p1, p2)
        } else {
            let p1 = n.get(&1).cloned().unwrap_or("Jog. 2".to_string());
            let p2 = n.get(&3).cloned().unwrap_or("Jog. 4".to_string());
            format!("{} & {}", p1, p2)
        }
    };

    view! {
         <div
            style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.85); display: flex; justify-content: center; align-items: center; z-index: 2000; backdrop-filter: blur(5px);"
            on:click=move |_| on_close.run(())
        >
            <div
                style=move || format!("
                    background: #1e1e1e; 
                    padding: 30px; 
                    border-radius: 16px; 
                    min-width: 600px; 
                    max-width: 90vw; 
                    color: white; 
                    border: 1px solid #444; 
                    box-shadow: 0 20px 50px rgba(0,0,0,0.8);
                    transform: scale({}); 
                    transform-origin: center;
                    font-family: 'Segoe UI', sans-serif;
                ", scale)
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
            >
                // ... (Conteúdo da tabela igual ao anterior) ...
                // Se precisar do conteúdo exato da tabela, eu repito aqui,
                // mas você pode manter o que já funcionou no passo anterior.
                <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #444; padding-bottom: 15px; margin-bottom: 15px;">
                    <h2 style="margin: 0; font-size: 20px; letter-spacing: 1px; color: white;">"HISTÓRICO DA PARTIDA"</h2>
                    <div style="font-size: 12px; color: #888;">"DETALHAMENTO POR RODADA"</div>
                </div>
                 <div style="max-height: 60vh; overflow-y: auto; padding-right: 5px;">
                    <table style="width: 100%; border-collapse: separate; border-spacing: 0 4px;">
                        <thead>
                            <tr style="font-size: 12px; text-transform: uppercase; color: #aaa;">
                                <th style="padding: 10px; text-align: center; width: 50px;">"RODADA"</th>
                                <th style="padding: 10px; text-align: center; color: #90caf9; background: rgba(144, 202, 249, 0.05); border-radius: 6px 0 0 6px;">
                                    <div style="font-size: 0.9em; opacity: 0.7;">"MEU TIME"</div>
                                    <div style="font-size: 1.1em; font-weight: bold;">{move || get_names_for_team(sou_time_a())}</div>
                                </th>
                                <th style="padding: 10px; text-align: center; color: #ffcc80; background: rgba(255, 204, 128, 0.05); border-radius: 0 6px 6px 0;">
                                    <div style="font-size: 0.9em; opacity: 0.7;">"INIMIGO"</div>
                                    <div style="font-size: 1.1em; font-weight: bold;">{move || get_names_for_team(!sou_time_a())}</div>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                             {move || {
                                let (hist_meu, historico_inimigo) = if sou_time_a() { (historico_a.get(), historico_b.get()) } else { (historico_b.get(), historico_a.get()) };
                                let max_len = hist_meu.len().max(historico_inimigo.len());
                                if max_len == 0 { return vec![view! { <tr><td colspan="3" style="text-align: center; padding: 20px; color: #666; font-style: italic;">"Nenhuma rodada finalizada ainda."</td></tr> }.into_any()]; }

                                (0..max_len).map(|i| {
                                    let pts_meu = hist_meu.get(i).copied().unwrap_or(0);
                                    let pts_inimigo = historico_inimigo.get(i).copied().unwrap_or(0);
                                    let color_meu = if pts_meu < 0 { "#ff5252" } else { "#fff" };
                                    let color_inimigo = if pts_inimigo < 0 { "#ff5252" } else { "#fff" };
                                    let bg_row = if i % 2 == 0 { "rgba(255,255,255,0.03)" } else { "transparent" };
                                    view! {
                                        <tr style=format!("background: {};", bg_row)>
                                            <td style="padding: 12px; text-align: center; color: #666; font-weight: bold;">{i + 1}</td>
                                            <td style=format!("padding: 12px; text-align: center; color: {}; font-weight: bold; font-size: 1.1em; border-right: 1px solid #333;", color_meu)>{if pts_meu > 0 { format!("+{}", pts_meu) } else { format!("{}", pts_meu) }}</td>
                                            <td style=format!("padding: 12px; text-align: center; color: {}; font-weight: bold; font-size: 1.1em;", color_inimigo)>{if pts_inimigo > 0 { format!("+{}", pts_inimigo) } else { format!("{}", pts_inimigo) }}</td>
                                        </tr>
                                    }.into_any()
                                }).collect::<Vec<_>>()
                            }}
                        </tbody>
                        <tfoot>
                            <tr style="background: #333; height: 50px;">
                                <td style="padding: 10px; text-align: center; font-weight: bold; border-radius: 0 0 0 8px;">"TOTAL"</td>
                                <td style="padding: 10px; text-align: center; color: #90caf9; font-size: 1.4em; font-weight: 800;">{move || { let h = if sou_time_a() { historico_a.get() } else { historico_b.get() }; h.iter().sum::<i32>() }}</td>
                                <td style="padding: 10px; text-align: center; color: #ffcc80; font-size: 1.4em; font-weight: 800; border-radius: 0 0 8px 0;">{move || { let h = if sou_time_a() { historico_b.get() } else { historico_a.get() }; h.iter().sum::<i32>() }}</td>
                            </tr>
                        </tfoot>
                    </table>
                </div>
                 <button on:click=move |_| on_close.run(()) on:mouseover=move |_| set_hover_btn.set(true) on:mouseout=move |_| set_hover_btn.set(false)
                    style=move || format!("margin-top: 20px; width: 100%; padding: 12px; background: {}; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: bold; font-size: 14px; transition: background 0.2s; text-transform: uppercase;", if hover_btn.get() { "#444" } else { "#2a2a2a" })
                >"Fechar"</button>
            </div>
        </div>
    }
}

use leptos::prelude::*;

#[component]
pub fn Scoreboard(
    #[prop(into)] pontuacao_a: Signal<i32>,
    #[prop(into)] pontuacao_b: Signal<i32>,
) -> impl IntoView {
    view! {
        <div style="
            display: flex;
            flex-direction: column;
            background: rgba(0, 0, 0, 0.6);
            padding: 10px 15px;
            border-radius: 8px;
            color: white;
            font-family: sans-serif;
            min-width: 120px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
            border: 1px solid rgba(255,255,255,0.1);
        ">
            <div style="font-size: 12px; text-transform: uppercase; letter-spacing: 1px; color: #aaa; margin-bottom: 5px; text-align: center;">
                "Placar"
            </div>

            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.2); padding-bottom: 5px; margin-bottom: 5px;">
                <span style="color: #90caf9; font-weight: bold;">"Time A"</span>
                <span style="font-size: 18px; font-weight: bold;">{pontuacao_a}</span>
            </div>

            <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="color: #ffcc80; font-weight: bold;">"Time B"</span>
                <span style="font-size: 18px; font-weight: bold;">{pontuacao_b}</span>
            </div>
        </div>
    }
}

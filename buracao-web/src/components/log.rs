use crate::game::state::LogEntry;
use leptos::html::Div;
use leptos::prelude::*;

#[component]
pub fn GameLog(
    #[prop(into)] log: Signal<Vec<LogEntry>>,
    #[prop(default = "200px")] height: &'static str,
) -> impl IntoView {
    // Referência para a área de rolagem (para fazermos auto-scroll)
    let scroll_ref = NodeRef::<Div>::new();

    // Efeito: Sempre que o log mudar, rola para o final
    Effect::new(move |_| {
        log.track(); // Monitora mudanças
        if let Some(div) = scroll_ref.get() {
            // request_animation_frame garante que o DOM atualizou antes de rolar
            request_animation_frame(move || {
                div.set_scroll_top(div.scroll_height());
            });
        }
    });

    view! {
        <div style=format!("
            display: flex;
            flex-direction: column;
            background: rgba(0, 0, 0, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 8px;
            width: 320px;
            height: {};
            color: rgba(255, 255, 255, 0.9);
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            backdrop-filter: blur(4px);
            box-shadow: 0 4px 6px rgba(0,0,0,0.2);
            pointer-events: auto; /* Permite rolar/clicar */
        ", height)>

            // Cabeçalho
            <div style="
                padding: 6px 10px;
                background: rgba(0,0,0,0.4);
                border-bottom: 1px solid rgba(255,255,255,0.05);
                font-weight: bold;
                color: #888;
                border-radius: 8px 8px 0 0;
                display: flex;
                justify-content: space-between;
                align-items: center;
            ">
                <span>"GAME LOG"</span>
            </div>

            // Lista de Mensagens
            <div
                node_ref=scroll_ref
                style="
                    flex: 1;
                    overflow-y: auto;
                    padding: 8px;
                    display: flex;
                    flex-direction: column;
                    gap: 4px;
                    scrollbar-width: thin;
                    scrollbar-color: #555 transparent;
                "
            >
                <For
                    each=move || log.get().into_iter().enumerate()
                    key=|(i, _)| *i
                    children=move |(_, entry)| {
                        // Cores baseadas no tipo de mensagem
                        let color = if entry.is_error { "#ff5252" }
                                   else if entry.is_success { "#ffd700" }
                                   else { "#ddd" };

                        // Fundo leve para erros
                        let bg = if entry.is_error { "rgba(255, 82, 82, 0.1)" } else { "transparent" };

                        view! {
                            <div style=format!("
                                display: flex; 
                                gap: 8px; 
                                align-items: flex-start; 
                                background: {}; 
                                padding: 2px 4px; 
                                border-radius: 4px;
                            ", bg)>
                                // Timestamp (Cinza e fixo)
                                <span style="color: #666; font-size: 0.9em; min-width: 55px; flex-shrink: 0;">
                                    {format!("[{}]", entry.time)}
                                </span>

                                // Mensagem
                                <span style=format!("color: {}; word-break: break-word;", color)>
                                    {entry.msg}
                                </span>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

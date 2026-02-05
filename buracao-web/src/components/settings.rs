use leptos::prelude::*;

#[component]
pub fn SettingsModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,

    // RwSignal permite leitura e escrita direta (Two-way binding)
    current_theme_path: RwSignal<String>,
    card_scale: RwSignal<f32>,
) -> impl IntoView {
    view! {
        {move || if show.get() {
            view! {
                <div style="
                    position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
                    background: rgba(0,0,0,0.8); z-index: 2000;
                    display: flex; justify-content: center; align-items: center;
                    backdrop-filter: blur(3px);
                ">
                    <div style="
                        background: #1e1e1e; padding: 25px; border-radius: 12px; width: 320px;
                        color: white; border: 1px solid #444; box-shadow: 0 10px 30px black;
                    ">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #444; padding-bottom: 10px;">
                            <h3 style="margin: 0; font-size: 18px;">"Configurações"</h3>
                            <button on:click=move |_| on_close.run(()) style="background: none; border: none; color: #aaa; font-size: 20px; cursor: pointer;">"✕"</button>
                        </div>

                        // --- TEMA ---
                        <div style="margin-bottom: 20px;">
                            <label style="display: block; font-size: 13px; color: #aaa; margin-bottom: 8px;">"Estilo das Cartas"</label>
                            <select
                                on:change=move |ev| current_theme_path.set(event_target_value(&ev))
                                prop:value=move || current_theme_path.get()
                                style="width: 100%; padding: 10px; background: #333; color: white; border: 1px solid #555; border-radius: 6px; outline: none;"
                            >
                                // O value deve ser o caminho da pasta dentro de public/
                                <option value="/assets/cards/PaperCards">"Clássico (Papel)"</option>
                                <option value="/assets/cards/Kortit">"Kortit"</option>
                                <option value="/assets/cards/PixelCards">"Pixel Cards"</option>

                                // Exemplo futuro (se você criar a pasta):
                                // <option value="/assets/cards/PixelArt">"Pixel Art"</option>
                            </select>
                        </div>

                        // --- TAMANHO ---
                        <div style="margin-bottom: 25px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                                <label style="font-size: 13px; color: #aaa;">"Tamanho na Mão"</label>
                                <span style="font-size: 12px; color: #ffeb3b;">{move || format!("{:.0}%", card_scale.get() * 100.0)}</span>
                            </div>
                            <input
                                type="range" min="0.8" max="1.5" step="0.01"
                                prop:value=move || card_scale.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f32>() { card_scale.set(val); }
                                }
                                style="width: 100%; cursor: pointer;"
                            />
                        </div>

                        <div style="text-align: right;">
                            <button
                                on:click=move |_| on_close.run(())
                                style="background: #2e7d32; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-weight: bold;"
                            >
                                "Salvar"
                            </button>
                        </div>
                    </div>
                </div>
            }.into_any()
        } else {
            view! {}.into_any()
        }}
    }
}

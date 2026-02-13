use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys::HtmlAudioElement;

#[component]
pub fn SettingsModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,

    // Sinais Globais (RwSignal) que controlam o jogo
    current_theme_path: RwSignal<String>,
    card_scale: RwSignal<f32>,
    volume: RwSignal<f64>,
) -> impl IntoView {
    // --- 1. ESTADOS LOCAIS (RASCUNHO) ---
    // Criamos sinais locais para segurar os valores enquanto o usuário edita.
    // Usamos o valor atual como inicial.
    let (draft_theme, set_draft_theme) = signal(current_theme_path.get_untracked());
    let (draft_scale, set_draft_scale) = signal(card_scale.get_untracked());
    let (draft_volume, set_draft_volume) = signal(volume.get_untracked());

    // --- 2. SINCRONIZAÇÃO AO ABRIR ---
    // Sempre que o modal for aberto (show vira true), resetamos o rascunho
    // com os valores reais atuais do jogo.
    Effect::new(move |_| {
        if show.get() {
            set_draft_theme.set(current_theme_path.get_untracked());
            set_draft_scale.set(card_scale.get_untracked());
            set_draft_volume.set(volume.get_untracked());
        }
    });

    // --- 3. AÇÃO DE SALVAR ---
    let salvar_alteracoes = move |_| {
        // Aplica os valores do rascunho nas variáveis globais
        current_theme_path.set(draft_theme.get());
        card_scale.set(draft_scale.get());
        volume.set(draft_volume.get());

        // Salva volume no LocalStorage
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item("buraco_volume", &draft_volume.get().to_string());
            }
        }

        // Fecha o modal
        on_close.run(());
    };

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

                        // --- VOLUME (Edita draft_volume) ---
                        <div style="margin-bottom: 20px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                                <label style="font-size: 13px; color: #aaa;">"Volume Notificação"</label>
                                <span style="font-size: 12px; color: #ffeb3b;">{move || format!("{:.0}%", draft_volume.get() * 100.0)}</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <input
                                    type="range" min="0" max="1" step="0.1"
                                    prop:value=move || draft_volume.get()
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev).parse::<f64>().unwrap_or(0.5);
                                        set_draft_volume.set(val);
                                    }
                                    style="width: 100%; cursor: pointer;"
                                />
                                <button
                                    style="background: #ffeb3b; border: none; border-radius: 50%; width: 24px; height: 24px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 12px;"
                                    title="Testar som"
                                >
                                    "🔊"
                                </button>
                            </div>
                        </div>

                        // --- TEMA (Edita draft_theme) ---
                        <div style="margin-bottom: 20px;">
                            <label style="display: block; font-size: 13px; color: #aaa; margin-bottom: 8px;">"Estilo das Cartas"</label>

                            // O PROBLEMA DO TRAVAMENTO GERALMENTE OCORRE AQUI
                            // Ao usar prop:value ligado ao draft, garantimos que o select siga o estado local
                            <select
                                on:change=move |ev| set_draft_theme.set(event_target_value(&ev))
                                prop:value=move || draft_theme.get()
                                style="width: 100%; padding: 10px; background: #333; color: white; border: 1px solid #555; border-radius: 6px; outline: none;"
                            >
                                <option value="/assets/cards/PaperCards">"Clássico (Papel)"</option>
                                <option value="/assets/cards/Kortit">"Kortit"</option>
                                <option value="/assets/cards/PixelCards">"Pixel Cards"</option>
                            </select>
                        </div>

                        // --- TAMANHO (Edita draft_scale) ---
                        <div style="margin-bottom: 25px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                                <label style="font-size: 13px; color: #aaa;">"Tamanho na Mão"</label>
                                <span style="font-size: 12px; color: #ffeb3b;">{move || format!("{:.0}%", draft_scale.get() * 100.0)}</span>
                            </div>
                            <input
                                type="range" min="0.8" max="1.5" step="0.01"
                                prop:value=move || draft_scale.get()
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f32>() {
                                        set_draft_scale.set(val);
                                    }
                                }
                                style="width: 100%; cursor: pointer;"
                            />
                        </div>

                        <div style="text-align: right;">
                            <button
                                on:click=salvar_alteracoes
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

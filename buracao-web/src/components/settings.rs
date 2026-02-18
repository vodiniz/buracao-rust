use crate::components::shortcut_manager::KeyBindings;
use leptos::prelude::*;

// --- SUB-COMPONENTE AUXILIAR ---
// Encapsula a lógica do input para evitar repetição e garantir reatividade correta
#[component]
fn KeyInput(
    #[prop(into)] value: Signal<String>, // Recebe um sinal (reativo)
    #[prop(into)] set_value: Callback<String>, // Recebe a função de update
) -> impl IntoView {
    view! {
        <input
            type="text"
            prop:value=value // O prop:value agora ouve o sinal automaticamente
            maxlength="1"
            style="width: 40px; text-align: center; background: #333; border: 1px solid #555; color: #ffeb3b; border-radius: 4px; padding: 4px; text-transform: uppercase;"
            on:input=move |ev| {
                let val = event_target_value(&ev).to_lowercase();
                set_value.run(val);
            }
        />
    }
}

// --- COMPONENTE PRINCIPAL ---
#[component]
pub fn SettingsModal(
    #[prop(into)] show: RwSignal<bool>,
    current_theme_path: RwSignal<String>,
    card_scale: RwSignal<f32>,
    volume: RwSignal<f64>,
    key_bindings: RwSignal<KeyBindings>,
) -> impl IntoView {
    // --- LOCAL STATE ---
    let draft_theme = RwSignal::new(String::new());
    let draft_scale = RwSignal::new(1.0);
    let draft_volume = RwSignal::new(0.8);
    let draft_keys = RwSignal::new(KeyBindings::default());

    // --- SYNC ON OPEN ---
    Effect::new(move |_| {
        if show.get() {
            draft_theme.set(current_theme_path.get_untracked());
            draft_scale.set(card_scale.get_untracked());
            draft_volume.set(volume.get_untracked());
            if let Some(keys) = key_bindings.try_get_untracked() {
                draft_keys.set(keys);
            }
        }
    });

    // --- SAVE ACTION ---
    let salvar = move |_| {
        current_theme_path.set(draft_theme.get());
        card_scale.set(draft_scale.get());
        volume.set(draft_volume.get());
        key_bindings.set(draft_keys.get());

        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item("buraco_volume", &draft_volume.get().to_string());
                if let Ok(json) = serde_json::to_string(&draft_keys.get()) {
                    let _ = storage.set_item("buraco_keys", &json);
                }
            }
        }

        show.set(false);
    };

    view! {
        <div
            style=move || {
                let display = if show.get() { "flex" } else { "none" };
                format!("display: {}; position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); z-index: 2000; justify-content: center; align-items: center; backdrop-filter: blur(3px);", display)
            }
            on:click=move |_| show.set(false)
        >
            <div
                style="background: #1e1e1e; padding: 25px; border-radius: 12px; width: 340px; color: white; border: 1px solid #444; box-shadow: 0 10px 30px black; max-height: 90vh; overflow-y: auto;"
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
            >
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #444; padding-bottom: 10px;">
                    <h3 style="margin: 0; font-size: 18px;">"Configurações"</h3>
                    <button
                        on:click=move |_| show.set(false)
                        style="background: none; border: none; color: #aaa; font-size: 20px; cursor: pointer;"
                    >"✕"</button>
                </div>

                // VOLUME
                <div style="margin-bottom: 20px;">
                    <label style="font-size: 13px; color: #aaa;">"Volume"</label>
                    <input type="range" min="0" max="1" step="0.1"
                        prop:value=move || draft_volume.get()
                        on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse() { draft_volume.set(v); } }
                        style="width: 100%; cursor: pointer;"
                    />
                </div>

                // TEMA
                <div style="margin-bottom: 20px;">
                     <label style="display: block; font-size: 13px; color: #aaa; margin-bottom: 8px;">"Estilo das Cartas"</label>
                     <select
                        on:change=move |ev| draft_theme.set(event_target_value(&ev))
                        prop:value=move || draft_theme.get()
                        style="width: 100%; padding: 10px; background: #333; color: white; border: 1px solid #555; border-radius: 6px; outline: none;"
                    >
                        <option value="/assets/cards/PaperCards">"Clássico (Papel)"</option>
                        <option value="/assets/cards/Kortit">"Kortit"</option>
                        <option value="/assets/cards/PixelCards">"Pixel Cards"</option>
                    </select>
                </div>

                // ESCALA
                <div style="margin-bottom: 25px;">
                    <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Tamanho na Mão"</label>
                        <span style="font-size: 12px; color: #ffeb3b;">{move || format!("{:.0}%", draft_scale.get() * 100.0)}</span>
                    </div>
                    <input type="range" min="0.8" max="1.5" step="0.01"
                        prop:value=move || draft_scale.get()
                        on:input=move |ev| { if let Ok(val) = event_target_value(&ev).parse::<f32>() { draft_scale.set(val); } }
                        style="width: 100%; cursor: pointer;"
                    />
                </div>

                // ATALHOS - Usando o novo componente KeyInput
                <div style="margin-bottom: 20px;">
                    <h4 style="font-size: 14px; color: #fff; margin-bottom: 10px; border-bottom: 1px dashed #444; padding-bottom: 5px;">"Atalhos de Teclado"</h4>

                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Comprar Monte"</label>
                        <KeyInput
                            // Signal::derive cria um contexto reativo, resolvendo o warning
                            value=Signal::derive(move || draft_keys.get().comprar_monte)
                            set_value=Callback::new(move |v| draft_keys.update(|d| d.comprar_monte = v))
                        />
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Descartar"</label>
                        <KeyInput
                            value=Signal::derive(move || draft_keys.get().descartar)
                            set_value=Callback::new(move |v| draft_keys.update(|d| d.descartar = v))
                        />
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Pegar Lixo"</label>
                        <KeyInput
                            value=Signal::derive(move || draft_keys.get().comprar_lixo)
                            set_value=Callback::new(move |v| draft_keys.update(|d| d.comprar_lixo = v))
                        />
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Organizar"</label>
                        <KeyInput
                            value=Signal::derive(move || draft_keys.get().organizar)
                            set_value=Callback::new(move |v| draft_keys.update(|d| d.organizar = v))
                        />
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <label style="font-size: 13px; color: #aaa;">"Ver Placar"</label>
                        <KeyInput
                            value=Signal::derive(move || draft_keys.get().placar)
                            set_value=Callback::new(move |v| draft_keys.update(|d| d.placar = v))
                        />
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                            <label style="font-size: 13px; color: #aaa;">"Separar Jogo"</label>
                            <KeyInput
                                value=Signal::derive(move || draft_keys.get().separar)
                                set_value=Callback::new(move |v| draft_keys.update(|d| d.separar = v))
                            />
                        </div>
                    </div>

                <div style="text-align: right;">
                    <button
                        on:click=salvar
                        style="background: #2e7d32; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-weight: bold;"
                    >"Salvar"</button>
                </div>
            </div>
        </div>
    }
}

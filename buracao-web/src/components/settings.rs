use crate::components::shortcut_manager::KeyBindings;
use leptos::prelude::*;

#[component]
pub fn SettingsModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,

    // Sinais Globais
    current_theme_path: RwSignal<String>,
    card_scale: RwSignal<f32>,
    volume: RwSignal<f64>,
    key_bindings: RwSignal<KeyBindings>,
) -> impl IntoView {
    // --- ESTADOS LOCAIS (RASCUNHO) ---
    let (draft_theme, set_draft_theme) = signal(current_theme_path.get_untracked());
    let (draft_scale, set_draft_scale) = signal(card_scale.get_untracked());
    let (draft_volume, set_draft_volume) = signal(volume.get_untracked());
    let (draft_keys, set_draft_keys) = signal(key_bindings.get_untracked());

    // --- SINCRONIZAÇÃO AO ABRIR ---
    Effect::new(move |_| {
        if show.get() {
            set_draft_theme.set(current_theme_path.get_untracked());
            set_draft_scale.set(card_scale.get_untracked());
            set_draft_volume.set(volume.get_untracked());
            set_draft_keys.set(key_bindings.get_untracked());
        }
    });

    // --- AÇÃO DE SALVAR ---
    let salvar_alteracoes = move |_| {
        current_theme_path.set(draft_theme.get());
        card_scale.set(draft_scale.get());
        volume.set(draft_volume.get());

        let novas_teclas = draft_keys.get();
        key_bindings.set(novas_teclas.clone());

        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item("buraco_volume", &draft_volume.get().to_string());
                if let Ok(json_keys) = serde_json::to_string(&novas_teclas) {
                    let _ = storage.set_item("buraco_keys", &json_keys);
                }
            }
        }
        on_close.run(());
    };

    // Helper para inputs de tecla
    let render_key_input = move |label: &'static str,
                                 value: String,
                                 field_setter: Box<dyn Fn(String)>| {
        view! {
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <label style="font-size: 13px; color: #aaa;">{label}</label>
                <input
                    type="text"
                    value=value
                    maxlength="1"
                    style="width: 40px; text-align: center; background: #333; border: 1px solid #555; color: #ffeb3b; border-radius: 4px; padding: 4px; text-transform: uppercase;"
                    on:input=move |ev| {
                        let val = event_target_value(&ev).to_lowercase();
                        field_setter(val);
                    }
                />
            </div>
        }
    };

    view! {
        <Show when=move || show.get() fallback=|| ()>
            <div style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.8); z-index: 2000; display: flex; justify-content: center; align-items: center; backdrop-filter: blur(3px);">
                <div style="background: #1e1e1e; padding: 25px; border-radius: 12px; width: 340px; color: white; border: 1px solid #444; box-shadow: 0 10px 30px black; max-height: 90vh; overflow-y: auto;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; border-bottom: 1px solid #444; padding-bottom: 10px;">
                        <h3 style="margin: 0; font-size: 18px;">"Configurações"</h3>
                        <button on:click=move |_| on_close.run(()) style="background: none; border: none; color: #aaa; font-size: 20px; cursor: pointer;">"✕"</button>
                    </div>

                    <div style="margin-bottom: 20px;">
                        <label style="font-size: 13px; color: #aaa;">"Volume"</label>
                        <input type="range" min="0" max="1" step="0.1" prop:value=move || draft_volume.get() on:input=move |ev| { set_draft_volume.set(event_target_value(&ev).parse().unwrap_or(0.5)); } style="width: 100%; cursor: pointer;" />
                    </div>

                    <div style="margin-bottom: 20px;">
                        <label style="display: block; font-size: 13px; color: #aaa; margin-bottom: 8px;">"Estilo das Cartas"</label>
                        <select on:change=move |ev| set_draft_theme.set(event_target_value(&ev)) prop:value=move || draft_theme.get() style="width: 100%; padding: 10px; background: #333; color: white; border: 1px solid #555; border-radius: 6px; outline: none;">
                            <option value="/assets/cards/PaperCards">"Clássico (Papel)"</option>
                            <option value="/assets/cards/Kortit">"Kortit"</option>
                            <option value="/assets/cards/PixelCards">"Pixel Cards"</option>
                        </select>
                    </div>

                    <div style="margin-bottom: 25px;">
                        <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
                            <label style="font-size: 13px; color: #aaa;">"Tamanho na Mão"</label>
                            <span style="font-size: 12px; color: #ffeb3b;">{move || format!("{:.0}%", draft_scale.get() * 100.0)}</span>
                        </div>
                        <input type="range" min="0.8" max="1.5" step="0.01" prop:value=move || draft_scale.get() on:input=move |ev| { if let Ok(val) = event_target_value(&ev).parse::<f32>() { set_draft_scale.set(val); } } style="width: 100%; cursor: pointer;" />
                    </div>

                    <div style="margin-bottom: 20px;">
                        <h4 style="font-size: 14px; color: #fff; margin-bottom: 10px; border-bottom: 1px dashed #444; padding-bottom: 5px;">"Atalhos de Teclado"</h4>
                        {move || {
                            // Pegamos o objeto uma vez
                            let k = draft_keys.get();
                            view! {
                                <div>
                                    {render_key_input("Comprar Monte", k.comprar_monte.clone(), Box::new(move |v| set_draft_keys.update(|d| d.comprar_monte = v)))}
                                    {render_key_input("Descartar Seleção", k.descartar.clone(), Box::new(move |v| set_draft_keys.update(|d| d.descartar = v)))}
                                    {render_key_input("Pegar Lixo", k.comprar_lixo.clone(), Box::new(move |v| set_draft_keys.update(|d| d.comprar_lixo = v)))}
                                    {render_key_input("Organizar Mão", k.organizar.clone(), Box::new(move |v| set_draft_keys.update(|d| d.organizar = v)))}
                                    {render_key_input("Ver Placar", k.placar.clone(), Box::new(move |v| set_draft_keys.update(|d| d.placar = v)))}
                                </div>
                            }
                        }}
                    </div>

                    <div style="text-align: right;">
                        <button on:click=salvar_alteracoes style="background: #2e7d32; color: white; border: none; padding: 10px 20px; border-radius: 6px; cursor: pointer; font-weight: bold;">"Salvar"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

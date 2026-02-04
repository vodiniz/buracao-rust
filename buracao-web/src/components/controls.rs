use leptos::prelude::*;
use web_sys::MouseEvent; // <--- Importante

#[component]
pub fn GameControls(
    // ESTADOS
    #[prop(into)] lixo_selecionado: Signal<bool>,
    #[prop(into)] tem_jogos_preparados: Signal<bool>,

    // AÇÕES: Agora tipadas explicitamente como MouseEvent para compatibilidade
    #[prop(into)] on_descartar: Callback<MouseEvent>,
    #[prop(into)] on_separar: Callback<MouseEvent>,
    #[prop(into)] on_ordenar: Callback<MouseEvent>,
    #[prop(into)] on_confirmar_lixo: Callback<MouseEvent>,
    #[prop(into)] on_cancelar_lixo: Callback<MouseEvent>,
    #[prop(into)] on_confirmar_baixa: Callback<MouseEvent>,
) -> impl IntoView {
    view! {
        <div style="
            display: flex; 
            flex-direction: column; 
            gap: 12px; 
            padding: 15px; 
            background: rgba(0,0,0,0.4); 
            border-radius: 12px; 
            border: 1px solid rgba(255,255,255,0.1);
            min-width: 200px;
            backdrop-filter: blur(5px);
        ">

            // --- 1. MODO LIXO ATIVO ---
            {move || if lixo_selecionado.get() {
                view! {
                    <div style="background: rgba(255, 193, 7, 0.15); padding: 10px; border-radius: 8px; border: 1px solid #ffc107; text-align: center;">
                        <span style="display: block; color: #ffc107; font-size: 11px; margin-bottom: 8px; text-transform: uppercase; font-weight: bold; letter-spacing: 1px;">
                            "Modo Compra de Lixo"
                        </span>
                        <div style="display: flex; gap: 8px; justify-content: center;">
                            <button
                                on:click=move |ev| on_confirmar_lixo.run(ev)
                                style="flex: 1; background: #ffc107; color: black; border: none; padding: 8px; border-radius: 4px; font-weight: bold; cursor: pointer; font-size: 12px;"
                            >
                                "CONFIRMAR"
                            </button>
                            <button
                                on:click=move |ev| on_cancelar_lixo.run(ev)
                                style="flex: 1; background: transparent; border: 1px solid #ffc107; color: #ffc107; padding: 8px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                            >
                                "CANCELAR"
                            </button>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}

            // --- 2. MODO BAIXAR JOGOS ---
            {move || if tem_jogos_preparados.get() && !lixo_selecionado.get() {
                view! {
                    <button
                        on:click=move |ev| on_confirmar_baixa.run(ev)
                        style="
                            background: linear-gradient(45deg, #2e7d32, #43a047);
                            color: white; border: none; padding: 12px; 
                            border-radius: 6px; font-weight: bold; cursor: pointer; 
                            width: 100%; box-shadow: 0 4px 6px rgba(0,0,0,0.2);
                            font-size: 13px; letter-spacing: 0.5px;
                        "
                    >
                        "BAIXAR JOGOS SEPARADOS"
                    </button>
                }.into_any()
            } else {
                view! {}.into_any()
            }}

            // --- 3. AÇÕES PADRÃO ---
            <div style="display: flex; gap: 10px;">
                <button
                    on:click=move |ev| on_separar.run(ev)
                    style="flex: 1; padding: 10px; background-color: #0288d1; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: bold; font-size: 13px;"
                    title="Mover cartas selecionadas para área de separação"
                >
                    "Separar"
                </button>

                <button
                    on:click=move |ev| on_descartar.run(ev)
                    style="flex: 1; padding: 10px; background-color: #d32f2f; color: white; border: none; border-radius: 6px; cursor: pointer; font-weight: bold; font-size: 13px;"
                    title="Descartar carta selecionada e passar a vez"
                >
                    "Descartar"
                </button>
            </div>

            <button
                on:click=move |ev| on_ordenar.run(ev)
                style="
                    padding: 8px; font-size: 11px; background-color: rgba(255,255,255,0.1); 
                    color: #ddd; border: 1px solid rgba(255,255,255,0.2); 
                    border-radius: 15px; cursor: pointer; margin-top: 5px;
                    transition: all 0.2s;
                "
            >
                "Ordenar Minha Mão"
            </button>
        </div>
    }
}

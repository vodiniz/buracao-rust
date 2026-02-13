use leptos::prelude::*; // Importante para eventos do DOM

#[component]
pub fn LoginScreen(
    // Callback que devolve uma tupla (Nome, Sala) para o pai
    #[prop(into)] on_enter: Callback<(String, String)>,
) -> impl IntoView {
    let (nome, set_nome) = signal("".to_string());
    let (sala, set_sala) = signal("SALA-1".to_string());

    let enviar = move |_| {
        let n = nome.get();
        let s = sala.get();

        if !n.trim().is_empty() && !s.trim().is_empty() {
            // AQUI ESTÁ A MÁGICA: Passa os dados para o pai (App)
            on_enter.run((n, s));
        }
    };

    view! {
        <div style="
            height: 100vh; display: flex; align-items: center; justify-content: center;
            background: #1b5e20; font-family: sans-serif; color: white;
        ">
            <div style="
                background: rgba(0,0,0,0.5); padding: 40px; border-radius: 15px;
                border: 2px solid #4caf50; width: 300px; display: flex; flex-direction: column; gap: 15px;
                box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            ">
                <h1 style="text-align: center; color: #ffeb3b; margin-top: 0;">"Buracão Web"</h1>

                <div>
                    <label>"Seu Nick:"</label>
                    <input
                        type="text"
                        prop:value=nome
                        on:input=move |e| set_nome.set(event_target_value(&e))
                        style="width: 100%; padding: 10px; margin-top: 5px; border-radius: 5px; border: none; box-sizing: border-box;"
                        placeholder="Ex: Vitor"
                    />
                </div>

                <div>
                    <label>"Código da Sala:"</label>
                    <input
                        type="text"
                        prop:value=sala
                        on:input=move |e| set_sala.set(event_target_value(&e).to_uppercase())
                        style="width: 100%; padding: 10px; margin-top: 5px; border-radius: 5px; border: none;box-sizing: border-box;"
                        placeholder="Ex: SALA-1"
                    />
                </div>

                <button
                    on:click=enviar
                    style="
                        width: 100%;
                        margin-top: 15px; 
                        padding: 12px; 
                        background: #ffeb3b; 
                        color: black;
                        font-weight: bold; 
                        border: none; 
                        border-radius: 5px; 
                        cursor: pointer;
                        font-size: 1.1rem; 
                        transition: transform 0.1s;
                        box-sizing: border-box; /* Garante que o padding não estoure a largura */
                    "
                >
                    "ENTRAR"
                </button>
            </div>
        </div>
    }
}

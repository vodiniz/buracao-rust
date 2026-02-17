use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct KeyBindings {
    pub comprar_monte: String,
    pub descartar: String,
    pub comprar_lixo: String, // Adicionei para completude (ex: 'T' de Trash ou 'X')
    pub organizar: String,
    pub placar: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            comprar_monte: "c".to_string(),
            descartar: "l".to_string(), // "L" conforme seu pedido
            comprar_lixo: "x".to_string(),
            organizar: "o".to_string(),
            placar: "p".to_string(),
        }
    }
}

#[component]
pub fn ShortcutManager(
    #[prop(into)] bindings: Signal<KeyBindings>,
    #[prop(into)] on_buy_deck: Callback<()>,
    #[prop(into)] on_discard: Callback<()>,
    #[prop(into)] on_buy_trash: Callback<()>,
    #[prop(into)] on_sort: Callback<()>,
    #[prop(into)] on_toggle_scoreboard: Callback<()>,
) -> impl IntoView {
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key().to_lowercase();
        let binds = bindings.get();

        // 1. Ignorar se o usuário estiver digitando em um input
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                let tag = el.tag_name().to_lowercase();
                if tag == "input" || tag == "textarea" || tag == "select" {
                    return;
                }
            }
        }

        // 2. Mapeamento
        if key == binds.comprar_monte {
            on_buy_deck.run(());
        } else if key == binds.descartar {
            on_discard.run(());
        } else if key == binds.comprar_lixo {
            on_buy_trash.run(());
        } else if key == binds.organizar {
            on_sort.run(());
        } else if key == binds.placar {
            on_toggle_scoreboard.run(());
        }
    };

    // Registra o listener na janela global
    window_event_listener(ev::keydown, handle_keydown);
}

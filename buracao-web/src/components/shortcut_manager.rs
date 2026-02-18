use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct KeyBindings {
    pub comprar_monte: String,
    pub descartar: String,
    pub comprar_lixo: String,
    pub organizar: String,
    pub placar: String,
    pub separar: String, // <--- NOVO CAMPO
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            comprar_monte: "c".to_string(),
            descartar: "l".to_string(),
            comprar_lixo: "x".to_string(),
            organizar: "o".to_string(),
            placar: "p".to_string(),
            separar: "s".to_string(), // <--- VALOR PADRÃO
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
    #[prop(into)] on_separate: Callback<()>, // <--- NOVA PROP (CALLBACK)
    #[prop(into)] on_toggle_scoreboard: Callback<()>,
) -> impl IntoView {
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let binds = match bindings.try_get() {
            Some(b) => b,
            None => return,
        };

        let key = ev.key().to_lowercase();

        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                let tag = el.tag_name().to_lowercase();
                if tag == "input" || tag == "textarea" || tag == "select" {
                    return;
                }
            }
        }

        if key == binds.comprar_monte.to_lowercase() {
            on_buy_deck.run(());
        } else if key == binds.descartar.to_lowercase() {
            on_discard.run(());
        } else if key == binds.comprar_lixo.to_lowercase() {
            on_buy_trash.run(());
        } else if key == binds.organizar.to_lowercase() {
            on_sort.run(());
        } else if key == binds.separar.to_lowercase() {
            // <--- LÓGICA DO NOVO ATALHO
            on_separate.run(());
        } else if key == binds.placar.to_lowercase() {
            on_toggle_scoreboard.run(());
        }
    };

    window_event_listener(ev::keydown, handle_keydown);
}

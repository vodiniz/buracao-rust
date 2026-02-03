use leptos::prelude::*;

// 1. Declarando os módulos do projeto
// Isso torna as pastas components, utils e state acessíveis em todo o projeto
pub mod app;
pub mod components;
pub mod state; // Se ainda não criou os arquivos dentro de state, comente essa linha
pub mod utils;

// Importando o componente principal
use app::App;

fn main() {
    // Integração com logs do navegador (console.log)
    // Se der erro, ele mostra no Console do Inspecionar Elemento (F12)
    console_error_panic_hook::set_once();

    // Inicia o App no elemento <body> do HTML
    mount_to_body(|| {
        view! {
            <App/>
        }
    });
}

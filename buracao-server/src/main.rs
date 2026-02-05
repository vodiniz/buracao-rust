mod handler;
mod state;

use std::path::Path;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("🔥 INICIANDO SERVIDOR BURACO (PORTA 8080)...");

    let jogo = state::inicializar_jogo();
    let clientes = state::inicializar_clientes();

    let jogo_filter = warp::any().map(move || jogo.clone());
    let clientes_filter = warp::any().map(move || clientes.clone());

    // --- 1. ROTA DO WEBSOCKET ---
    // Acessível apenas via ws://localhost:8080/buracao
    let game_ws_route = warp::path("buraco")
        .and(warp::ws())
        .and(jogo_filter)
        .and(clientes_filter)
        .map(|ws: warp::ws::Ws, jogo, clientes| {
            ws.on_upgrade(move |socket| handler::handle_connection(socket, jogo, clientes))
        });

    // --- 2. DESCOBRIR ONDE ESTÁ O SITE ---
    let lugares_provaveis = vec!["./dist", "../buracao-web/dist", "./buracao-web/dist"];

    let mut static_path = "./dist".to_string();
    let mut encontrou = false;

    for caminho in lugares_provaveis {
        if Path::new(caminho).join("index.html").exists() {
            static_path = caminho.to_string();
            encontrou = true;
            break;
        }
    }

    if encontrou {
        println!("✅ Site encontrado em: '{}'", static_path);
    } else {
        println!("❌ AVISO CRÍTICO: 'index.html' não encontrado!");
        println!("   Rode 'trunk build --release' na pasta buracao-web.");
    }

    // --- 3. CONFIGURAÇÃO DO SITE (SPA) ---

    // A. Arquivos estáticos (JS, CSS, WASM, Imagens)
    // Se o navegador pedir /style.css, o Warp procura na pasta static_path
    let assets = warp::fs::dir(static_path.clone());

    // B. O Arquivo Index (Fallback)
    // Se o navegador pedir / (Raiz) ou uma rota que não é arquivo (ex: /sala/1),
    // entregamos o index.html para o Leptos resolver no frontend.
    let index_file_path = Path::new(&static_path).join("index.html");
    let spa_fallback = warp::fs::file(index_file_path);

    // Ordem importante: Tenta servir arquivo exato -> Se falhar, serve index.html
    let site_route = assets.or(spa_fallback);

    // --- 4. COMBINAÇÃO ---
    // Websocket tem prioridade. Se não for WS, tenta site.
    let routes = game_ws_route.or(site_route);

    println!("🚀 Server rodando em http://0.0.0.0:8080");
    println!("   (Acesse http://localhost:8080 no navegador)");

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

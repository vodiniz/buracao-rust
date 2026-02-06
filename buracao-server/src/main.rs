mod handler;
mod state;

use std::path::Path;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("🔥 INICIANDO SERVIDOR BURACO COM LOBBIES (PORTA 8080)...");

    // 1. INICIALIZAÇÃO DO ESTADO GLOBAL (O "HOTEL" DE SALAS)
    // Agora 'global_state' contém um HashMap de salas, não um jogo único.
    let global_state = state::inicializar_servidor();

    // Cria um filtro do Warp para injetar esse estado em cada conexão
    let state_filter = warp::any().map(move || global_state.clone());

    // 2. ROTA DO WEBSOCKET
    // Quando alguém acessa ws://localhost:8080/buraco
    let game_ws_route =
        warp::path("buraco")
            .and(warp::ws())
            .and(state_filter)
            .map(|ws: warp::ws::Ws, state| {
                // Passa a conexão e o estado global para o handler gerenciar o login
                ws.on_upgrade(move |socket| handler::handle_connection(socket, state))
            });

    // 3. DESCOBRIR ONDE ESTÁ O SITE (FRONTEND)
    // Mantive sua lógica robusta de procurar a pasta 'dist'
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
        println!("❌ AVISO: 'index.html' não encontrado. O site não vai carregar.");
    }

    // 4. CONFIGURAÇÃO DE ARQUIVOS ESTÁTICOS (SPA)

    // A. Serve arquivos reais (js, css, imagens)
    let assets = warp::fs::dir(static_path.clone());

    // B. Fallback para SPA (Single Page Application)
    // Se a rota não for arquivo nem websocket (ex: /sala/amigos), entrega o index.html
    let index_file_path = Path::new(&static_path).join("index.html");
    let spa_fallback = warp::fs::file(index_file_path);

    // Combina: Tenta arquivo -> Se falhar, entrega index.html
    let site_route = assets.or(spa_fallback);

    // 5. JUNTAR TUDO E RODAR
    // Ordem de prioridade: WebSocket > Arquivos do Site
    let routes = game_ws_route.or(site_route);

    println!("🚀 Server rodando em http://0.0.0.0:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

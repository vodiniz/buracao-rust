use axum::{routing::get, Router};
use buracao_core::{Carta, MsgCliente, Naipe, Valor};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Inicializa logs
    tracing_subscriber::fmt::init();

    // Teste rápido para ver se o CORE está funcionando
    let carta_teste = Carta {
        naipe: Naipe::Copas,
        valor: Valor::Dois,
    };
    println!("--- TESTE DO CORE ---");
    println!(
        "Carta criada: {:?} de {:?}",
        carta_teste.valor, carta_teste.naipe
    );
    println!("É coringa? {}", carta_teste.eh_coringa());
    println!("Pontos: {}", carta_teste.pontos());

    // Simula um JSON chegando do frontend
    let json_entrada = r#"{ "acao": "ComprarBaralho" }"#;
    let msg: MsgCliente = serde_json::from_str(json_entrada).unwrap();
    println!("Recebido via JSON simulado: {:?}", msg);
    println!("---------------------");

    // Configura rotas básicas do Axum
    let app = Router::new().route("/", get(|| async { "Servidor Buraco Online! 🃏" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Servidor ouvindo em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

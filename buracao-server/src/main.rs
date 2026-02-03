use buracao_core::acoes::{AcaoJogador, MsgServidor};
use buracao_core::estado::EstadoJogo;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::mpsc;
use warp::Filter;

// Tipo para controlar o jogo compartilhado (Thread-safe)
type JogoCompartilhado = Arc<tokio::sync::RwLock<EstadoJogo>>;
type Clientes = Arc<DashMap<usize, mpsc::UnboundedSender<warp::ws::Message>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    println!("🔥 INICIANDO SERVIDOR BURACO...");

    let mut estado_inicial = EstadoJogo::new();
    println!("🎲 Embaralhando e distribuindo cartas...");
    estado_inicial.dar_cartas();

    let jogo = Arc::new(tokio::sync::RwLock::new(estado_inicial));
    let clientes: Clientes = Arc::new(DashMap::new());

    let jogo_filter = warp::any().map(move || jogo.clone());
    let clientes_filter = warp::any().map(move || clientes.clone());

    let routes = warp::path("buraco")
        .and(warp::ws())
        .and(jogo_filter)
        .and(clientes_filter)
        .map(|ws: warp::ws::Ws, jogo, clientes| {
            ws.on_upgrade(move |socket| handle_connection(socket, jogo, clientes))
        });

    println!("🚀 Server rodando em ws://127.0.0.1:3030/buraco");

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket, jogo: JogoCompartilhado, clientes: Clientes) {
    let my_connection_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    println!("Novo cliente conectado: ID Conexão {}", my_connection_id);

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    clientes.insert(my_connection_id, tx);

    // --- ENVIAR ESTADO INICIAL ---
    {
        let id_jogador_jogo = ((my_connection_id - 1) % 4) as u32;
        let estado = jogo.read().await;

        if let Some(sender) = clientes.get(&my_connection_id) {
            // 1. Envia Boas Vindas (Para o cliente saber quem ele é)
            let msg_bv = MsgServidor::BoasVindas {
                id_jogador: id_jogador_jogo,
            };
            if let Ok(json) = serde_json::to_string(&msg_bv) {
                let _ = sender.send(warp::ws::Message::text(json));
            }

            // 2. Envia Estado Inicial (Cartas)
            let visao = estado.gerar_visao_para_jogador(id_jogador_jogo);
            // ENVELOPA NO ENUM
            let msg_estado = MsgServidor::Estado(visao);

            if let Ok(json) = serde_json::to_string(&msg_estado) {
                let _ = sender.send(warp::ws::Message::text(json));
            }
        }
    }

    // --- TAREFA DE ENVIO (Server -> Cliente) ---
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(_e) = user_ws_tx.send(msg).await {
                break;
            }
        }
    });

    // --- TAREFA DE RECEBIMENTO (Cliente -> Server) ---
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Erro de conexão com cliente {}: {}", my_connection_id, e);
                break;
            }
        };

        let texto = match msg.to_str() {
            Ok(t) => t,
            Err(_) => continue,
        };

        println!("Cliente {} enviou: {}", my_connection_id, texto);

        if let Ok(acao) = serde_json::from_str::<AcaoJogador>(texto) {
            let id_jogador_jogo = ((my_connection_id - 1) % 4) as u32;

            let mut estado = jogo.write().await;
            let resultado = estado.realizar_acao(id_jogador_jogo, acao);

            match resultado {
                Ok(msg) => {
                    // --- NOVO: ENVIAR NOTIFICAÇÃO SÓ PARA QUEM JOGOU ---
                    if let Some(sender) = clientes.get(&my_connection_id) {
                        let msg_notif = MsgServidor::Notificacao(msg);
                        if let Ok(json_notif) = serde_json::to_string(&msg_notif) {
                            let _ = sender.send(warp::ws::Message::text(json_notif));
                        }
                    }
                    // --- BROADCAST DO ESTADO ---
                    for cliente in clientes.iter() {
                        let conn_id_destino = *cliente.key();
                        let sender = cliente.value();
                        let id_destino_jogo = ((conn_id_destino - 1) % 4) as u32;

                        let visao = estado.gerar_visao_para_jogador(id_destino_jogo);

                        // CORREÇÃO: Usar MsgServidor::Estado
                        let msg_envelope = MsgServidor::Estado(visao);

                        if let Ok(json_visao) = serde_json::to_string(&msg_envelope) {
                            let _ = sender.send(warp::ws::Message::text(json_visao));
                        }
                    }

                    // --- CHECK DE FIM DE JOGO ---
                    if estado.partida_encerrada {
                        // Define vencedor
                        let vencedor = if estado.pontuacao_a >= estado.pontuacao_b {
                            0
                        } else {
                            1
                        };

                        // A. Monta mensagem FimDeJogo
                        let msg_fim = MsgServidor::FimDeJogo {
                            vencedor_time: vencedor,
                            pontos_a: estado.pontuacao_a,
                            pontos_b: estado.pontuacao_b,
                            motivo: "Pontuação Alcançada".to_string(),
                        };

                        // B. Avisa todo mundo
                        if let Ok(json_fim) = serde_json::to_string(&msg_fim) {
                            for cliente in clientes.iter() {
                                let _ = cliente
                                    .value()
                                    .send(warp::ws::Message::text(json_fim.clone()));
                            }
                        }

                        // C. Cronômetro (Usando Notificacao)
                        for i in (1..=15).rev() {
                            let msg_timer =
                                MsgServidor::Notificacao(format!("Nova partida em {}...", i));

                            if let Ok(json_timer) = serde_json::to_string(&msg_timer) {
                                for cliente in clientes.iter() {
                                    let _ = cliente
                                        .value()
                                        .send(warp::ws::Message::text(json_timer.clone()));
                                }
                            }
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }

                        // D. Reseta
                        estado.resetar_jogo();

                        // E. Envia Início e Novo Estado
                        let msg_inicio =
                            MsgServidor::Notificacao("=== 🎲 NOVA PARTIDA ===\n".to_string());
                        let json_inicio = serde_json::to_string(&msg_inicio).unwrap_or_default();

                        for cliente in clientes.iter() {
                            let conn_id = *cliente.key();
                            let sender = cliente.value();

                            // Avisa texto
                            let _ = sender.send(warp::ws::Message::text(json_inicio.clone()));

                            // Manda cartas novas (Envelopado)
                            let id_player = ((conn_id - 1) % 4) as u32;
                            let nova_visao = estado.gerar_visao_para_jogador(id_player);
                            let msg_estado = MsgServidor::Estado(nova_visao); // Envelope

                            if let Ok(json) = serde_json::to_string(&msg_estado) {
                                let _ = sender.send(warp::ws::Message::text(json));
                            }
                        }
                    }
                }
                Err(e) => {
                    // CORREÇÃO: Usar MsgServidor::Erro
                    if let Some(sender) = clientes.get(&my_connection_id) {
                        let msg_erro = MsgServidor::Erro(format!("{}", e));
                        if let Ok(json_erro) = serde_json::to_string(&msg_erro) {
                            let _ = sender.send(warp::ws::Message::text(json_erro));
                        }
                    }
                }
            }
        }
    }

    println!("Cliente {} desconectou", my_connection_id);
    clientes.remove(&my_connection_id);
}

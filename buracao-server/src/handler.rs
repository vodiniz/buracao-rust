use crate::state::{GlobalState, Room};
use buracao_core::acoes::{AcaoJogador, MsgServidor};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration; // Importação para o Timer
use tokio::sync::{RwLock, mpsc};
use warp::ws::{Message, WebSocket};

#[derive(Deserialize, Debug)]
struct MensagemLogin {
    device_id: String,
    nome: String,
    sala: String,
}

// Struct para enviar a lista de nomes ao Frontend
#[derive(Serialize)]
struct EventoNomes {
    tipo: String, // "NomesJogadores"
    mapa: std::collections::HashMap<u32, String>,
}

pub async fn handle_connection(ws: WebSocket, global_state: GlobalState) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Tarefa para encaminhar mensagens do servidor -> cliente
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    println!("⏳ Nova conexão... aguardando Login.");

    let login_data: MensagemLogin = match ws_rx.next().await {
        Some(Ok(msg)) => match msg.to_str() {
            Ok(texto) => match serde_json::from_str::<MensagemLogin>(texto) {
                Ok(dados) => dados,
                Err(_) => {
                    println!("❌ JSON inválido recebido.");
                    return;
                }
            },
            Err(_) => return,
        },
        _ => return,
    };

    println!(
        "🔑 Login na sala '{}': {} ({})",
        login_data.sala, login_data.nome, login_data.device_id
    );

    // 2. ENCONTRA OU CRIA A SALA
    let room_ref: Arc<RwLock<Room>>;

    {
        let mut server = global_state.write().await;

        if let Some(existing_room) = server.rooms.get(&login_data.sala) {
            room_ref = existing_room.clone();
        } else {
            println!("🏠 Criando SALA NOVA: {}", login_data.sala);
            // IMPORTANTE: Room::new() já chama EstadoJogo::new() que gerencia a criação correta (Teste ou Padrão)
            // Não chamamos mais dar_cartas() aqui manualmente.
            let r = Room::new();
            let new_room = Arc::new(RwLock::new(r));

            server
                .rooms
                .insert(login_data.sala.clone(), new_room.clone());
            room_ref = new_room;
        }
    }

    let my_player_id: u32;

    // 3. REGISTRA O JOGADOR NA SALA
    {
        let mut room = room_ref.write().await;

        if let Some(&id) = room.sessions.get(&login_data.device_id) {
            println!("🔄 Reconexão detectada: ID {}", id);
            my_player_id = id;
        } else {
            let next_id = room.sessions.len() as u32;
            if next_id >= 4 {
                let _ = tx.send(Message::text(r#"{"erro": "Sala cheia!"}"#));
                return;
            }
            my_player_id = next_id;
            room.sessions
                .insert(login_data.device_id.clone(), my_player_id);
        }

        // Atualiza nome e canal
        room.player_names
            .insert(my_player_id, login_data.nome.clone());
        room.clients.insert(my_player_id, tx.clone());

        // Envia estado inicial
        let visao = room.game_state.gerar_visao_para_jogador(my_player_id);
        if let Ok(msg) = serde_json::to_string(&MsgServidor::Estado(visao)) {
            let _ = tx.send(Message::text(msg));
        }

        // Envia lista de nomes atualizada
        let evento_nomes = EventoNomes {
            tipo: "NomesJogadores".to_string(),
            mapa: room.player_names.clone(),
        };

        if let Ok(json_nomes) = serde_json::to_string(&evento_nomes) {
            for client_tx in room.clients.values() {
                let _ = client_tx.send(Message::text(json_nomes.clone()));
            }
        }
    }

    // 4. LOOP DO JOGO
    while let Some(Ok(msg)) = ws_rx.next().await {
        let texto = match msg.to_str() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let acao: AcaoJogador = match serde_json::from_str(texto) {
            Ok(a) => a,
            Err(_) => continue,
        };

        let mut room = room_ref.write().await;
        let resultado = room.game_state.realizar_acao(my_player_id, acao);

        match resultado {
            Ok(msg_sucesso) => {
                // 1. Broadcast do Estado ATUAL (Mostra a batida na mesa)
                for (pid, client_tx) in room.clients.iter() {
                    let visao = room.game_state.gerar_visao_para_jogador(*pid);
                    let envelope = MsgServidor::Estado(visao);
                    if let Ok(json) = serde_json::to_string(&envelope) {
                        let _ = client_tx.send(Message::text(json));
                    }
                }

                // 2. Notificação de sucesso para quem jogou
                if let Ok(json) = serde_json::to_string(&MsgServidor::Notificacao(msg_sucesso)) {
                    let _ = tx.send(Message::text(json));
                }

                // --- LÓGICA DO TIMER DE 15 SEGUNDOS (BATIDA) ---
                if room.game_state.partida_encerrada {
                    let nome_vencedor = room
                        .player_names
                        .get(&my_player_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Jogador {}", my_player_id));

                    println!("🏆 {} BATEU! Iniciando contagem...", nome_vencedor);

                    // Clona referência para a Task
                    let room_clone_timer = room_ref.clone();

                    // Dispara Task independente (não bloqueia o loop)
                    tokio::spawn(async move {
                        // Aviso Inicial
                        broadcast_msg(
                            &room_clone_timer,
                            format!("🏆 {} BATEU! Reiniciando em 15s...", nome_vencedor),
                        )
                        .await;

                        // Contagem Regressiva
                        for i in (1..=15).rev() {
                            broadcast_msg(
                                &room_clone_timer,
                                format!("Reiniciando Jogo em {}s...", i),
                            )
                            .await;

                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }

                        // Reset Final
                        broadcast_msg(&room_clone_timer, "🔄 REINICIANDO AGORA!".to_string()).await;

                        {
                            let mut r = room_clone_timer.write().await;

                            // AQUI acontece a mágica: chama o reset (que verifica MODO_TESTE se configurado)
                            r.game_state.resetar_jogo();

                            // Manda as cartas novas para todos
                            for (pid, client_tx) in r.clients.iter() {
                                let visao = r.game_state.gerar_visao_para_jogador(*pid);
                                let envelope = MsgServidor::Estado(visao);
                                if let Ok(json) = serde_json::to_string(&envelope) {
                                    let _ = client_tx.send(Message::text(json));
                                }
                            }
                        }
                    });
                }
            }
            Err(erro) => {
                if let Ok(json) = serde_json::to_string(&MsgServidor::Erro(erro)) {
                    let _ = tx.send(Message::text(json));
                }
            }
        }
    }

    println!(
        "❌ Conexão encerrada: Sala {}, Jogador {}",
        login_data.sala, my_player_id
    );
}

// Helper seguro para Broadcast
async fn broadcast_msg(room_ref: &Arc<RwLock<Room>>, texto: String) {
    let room = room_ref.read().await;
    // Usamos if let Ok para evitar unwrap() em produção
    if let Ok(json_msg) = serde_json::to_string(&MsgServidor::Notificacao(texto)) {
        for client_tx in room.clients.values() {
            let _ = client_tx.send(Message::text(json_msg.clone()));
        }
    }
}

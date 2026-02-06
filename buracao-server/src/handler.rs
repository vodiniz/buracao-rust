use crate::state::{GlobalState, Room};
use buracao_core::acoes::{AcaoJogador, MsgServidor};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use warp::ws::{Message, WebSocket};

#[derive(Deserialize, Debug)]
struct MensagemLogin {
    tipo: String,
    device_id: String,
    nome: String,
    sala: String,
}

pub async fn handle_connection(ws: WebSocket, global_state: GlobalState) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Tarefa para encaminhar mensagens do servidor -> cliente
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(_) = ws_tx.send(msg).await {
                break;
            }
        }
    });

    println!("⏳ Nova conexão... aguardando Login.");

    // 1. ESPERA O LOGIN (Handshake)
    let login_data: MensagemLogin;

    if let Some(result) = ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(texto) = msg.to_str() {
                if let Ok(dados) = serde_json::from_str::<MensagemLogin>(texto) {
                    login_data = dados;
                } else {
                    println!("❌ JSON inválido recebido.");
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        return;
    }

    println!(
        "🔑 Login na sala '{}': {} ({})",
        login_data.sala, login_data.nome, login_data.device_id
    );

    // 2. ENCONTRA OU CRIA A SALA
    let room_ref: Arc<RwLock<Room>>;

    {
        let mut server = global_state.write().await;

        // Verifica se a sala já existe ou cria nova
        if let Some(existing_room) = server.rooms.get(&login_data.sala) {
            room_ref = existing_room.clone();
        } else {
            println!("🏠 Criando SALA NOVA: {}", login_data.sala);
            let mut r = Room::new();
            r.game_state.dar_cartas();
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
            // Novo jogador
            let next_id = room.sessions.len() as u32;
            if next_id >= 4 {
                let _ = tx.send(Message::text(r#"{"erro": "Sala cheia!"}"#));
                return;
            }
            my_player_id = next_id;
            // Salva na sessão para o futuro
            room.sessions
                .insert(login_data.device_id.clone(), my_player_id);
        }

        // SEMPRE atualiza o canal de comunicação (para novos e reconexões)
        // O socket antigo morreu, este 'tx' é o novo.
        room.clients.insert(my_player_id, tx.clone());

        // Envia estado inicial imediato
        let visao = room.game_state.gerar_visao_para_jogador(my_player_id);
        if let Ok(msg) = serde_json::to_string(&MsgServidor::Estado(visao)) {
            let _ = tx.send(Message::text(msg));
        }
    }

    // 4. LOOP DO JOGO
    while let Some(result) = ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(texto) = msg.to_str() {
                if let Ok(acao) = serde_json::from_str::<AcaoJogador>(texto) {
                    let mut room = room_ref.write().await;

                    let resultado = room.game_state.realizar_acao(my_player_id, acao);

                    // Lógica de resposta e broadcast
                    match resultado {
                        Ok(msg_sucesso) => {
                            // 1. Broadcast do Estado para TODOS
                            for (pid, client_tx) in room.clients.iter() {
                                let visao = room.game_state.gerar_visao_para_jogador(*pid);
                                let envelope = MsgServidor::Estado(visao);
                                if let Ok(json) = serde_json::to_string(&envelope) {
                                    let _ = client_tx.send(Message::text(json));
                                }
                            }
                            // 2. Notificação de sucesso só para quem jogou
                            if let Ok(json) =
                                serde_json::to_string(&MsgServidor::Notificacao(msg_sucesso))
                            {
                                let _ = tx.send(Message::text(json));
                            }
                        }
                        Err(erro) => {
                            // Erro só para quem jogou
                            if let Ok(json) = serde_json::to_string(&MsgServidor::Erro(erro)) {
                                let _ = tx.send(Message::text(json));
                            }
                        }
                    }
                }
            }
        }
    }

    println!(
        "❌ Conexão encerrada: Sala {}, Jogador {}",
        login_data.sala, my_player_id
    );
}

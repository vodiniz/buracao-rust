use crate::state::{GlobalState, Room};
use buracao_core::acoes::{AcaoJogador, MsgServidor};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize}; // Adicionado Serialize
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use warp::ws::{Message, WebSocket};

#[derive(Deserialize, Debug)]
struct MensagemLogin {
    // tipo: String,
    device_id: String,
    nome: String,
    sala: String,
}

// --- NOVO: Struct para enviar a lista de nomes ao Frontend ---
// Isso permite que o frontend saiba que o ID 0 é "Vitor", o ID 1 é "João", etc.
#[derive(Serialize)]
struct EventoNomes {
    tipo: String, // Será sempre "NomesJogadores"
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

        // --- NOVO: SALVAR O NOME DO JOGADOR ---
        // Sempre atualiza o nome (caso ele tenha mudado no login)
        room.player_names
            .insert(my_player_id, login_data.nome.clone());

        // SEMPRE atualiza o canal de comunicação (para novos e reconexões)
        room.clients.insert(my_player_id, tx.clone());

        // Envia estado inicial imediato
        let visao = room.game_state.gerar_visao_para_jogador(my_player_id);
        if let Ok(msg) = serde_json::to_string(&MsgServidor::Estado(visao)) {
            let _ = tx.send(Message::text(msg));
        }

        // --- NOVO: ENVIAR LISTA DE NOMES PARA TODOS ---
        // Como entrou gente (ou reconectou), avisamos a sala inteira quem é quem.
        let evento_nomes = EventoNomes {
            tipo: "NomesJogadores".to_string(),
            mapa: room.player_names.clone(),
        };

        if let Ok(json_nomes) = serde_json::to_string(&evento_nomes) {
            // Manda para todos os clientes conectados na sala
            for client_tx in room.clients.values() {
                let _ = client_tx.send(Message::text(json_nomes.clone()));
            }
        }
    }

    // 4. LOOP DO JOGO
    while let Some(Ok(msg)) = ws_rx.next().await {
        let texto = match msg.to_str() {
            Ok(t) => t,
            Err(_) => continue, // ignora mensagens não-texto / inválidas
        };

        let acao: AcaoJogador = match serde_json::from_str(texto) {
            Ok(a) => a,
            Err(_) => continue, // ignora JSON inválido
        };

        let mut room = room_ref.write().await;

        let resultado = room.game_state.realizar_acao(my_player_id, acao);

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
                if let Ok(json) = serde_json::to_string(&MsgServidor::Notificacao(msg_sucesso)) {
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

    println!(
        "❌ Conexão encerrada: Sala {}, Jogador {}",
        login_data.sala, my_player_id
    );
}

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

        // --- CLONE PARA O LOG ---
        // Precisamos clonar porque 'realizar_acao' consome 'acao'
        let acao_para_log = acao.clone();

        let mut room = room_ref.write().await;
        let resultado = room.game_state.realizar_acao(my_player_id, acao);

        match resultado {
            Ok(_msg_sucesso) => {
                // --- 1. IDENTIFICA O NOME ---
                let nome_jogador = room
                    .player_names
                    .get(&my_player_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Jogador {}", my_player_id));

                // --- 2. GERA TEXTO DESCRITIVO ---
                // Transforma a ação técnica em texto legível para o Log
                let texto_log = match &acao_para_log {
                    AcaoJogador::ComprarBaralho => format!("{} comprou do monte.", nome_jogador),
                    AcaoJogador::ComprarLixo { .. } => format!("{} pegou o lixo!", nome_jogador),
                    AcaoJogador::BaixarJogos { jogos } => {
                        format!("{} baixou {} novos jogos.", nome_jogador, jogos.len())
                    }
                    AcaoJogador::Ajuntar { .. } => format!("{} ajuntou cartas.", nome_jogador),
                    AcaoJogador::Descartar { .. } => format!("{} descartou.", nome_jogador),
                    AcaoJogador::Mensagem { texto } => format!("{}: {}", nome_jogador, texto),
                };

                // --- 3. BROADCAST DO LOG (NOTIFICAÇÃO) ---
                // Envia para TODOS os clientes na sala
                if let Ok(json_notif) = serde_json::to_string(&MsgServidor::Notificacao(texto_log))
                {
                    for client_tx in room.clients.values() {
                        let _ = client_tx.send(Message::text(json_notif.clone()));
                    }
                }

                // --- 4. BROADCAST DO ESTADO (VISUAL) ---
                for (pid, client_tx) in room.clients.iter() {
                    let visao = room.game_state.gerar_visao_para_jogador(*pid);
                    let envelope = MsgServidor::Estado(visao);
                    if let Ok(json) = serde_json::to_string(&envelope) {
                        let _ = client_tx.send(Message::text(json));
                    }
                }

                // --- LÓGICA DO TIMER DE 15 SEGUNDOS (BATIDA) ---
                if room.game_state.partida_encerrada {
                    let nome_vencedor = room
                        .player_names
                        .get(&my_player_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Jogador {}", my_player_id));

                    let (pts_a, pts_b) = (room.game_state.pontuacao_a, room.game_state.pontuacao_b);

                    // 1. AVALIAÇÃO DO FIM DO JOGO (SÓ MANDA A TELA SE TIVER 5000+)
                    if pts_a >= 5000 || pts_b >= 5000 {
                        let time_vencedor = if pts_a >= pts_b { 0 } else { 1 };

                        let msg_fim = MsgServidor::FimDeJogo {
                            vencedor_time: time_vencedor,
                            pontos_a: pts_a,
                            pontos_b: pts_b,
                            motivo: "Pontuação máxima atingida!".to_string(),
                        };

                        // Se atingiu 5000, envia o modal para a tela de todos
                        if let Ok(json) = serde_json::to_string(&msg_fim) {
                            for client_tx in room.clients.values() {
                                let _ = client_tx.send(Message::text(json.clone()));
                            }
                        }
                    }

                    // 2. BROADCAST DE BATIDA NA RODADA (Sempre acontece, via Game Log)
                    println!("🏆 {} BATEU! Iniciando contagem...", nome_vencedor);

                    let room_clone_timer = room_ref.clone();
                    let nome_vencedor_log = nome_vencedor.clone();

                    tokio::spawn(async move {
                        let alguem_bateu = {
                            let r = room_clone_timer.read().await;
                            r.game_state.maos.iter().any(|mao| mao.is_empty())
                        };

                        let mensagem_alerta = if alguem_bateu {
                            format!("🏆 {} BATEU! Reiniciando em 15s...", nome_vencedor_log)
                        } else {
                            "📭 O BARALHO ESGOTOU! Reiniciando em 15s...".to_string()
                        };

                        // Aviso inicial
                        broadcast_msg(&room_clone_timer, mensagem_alerta).await;

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

                            // O Core decide se zera pontos (5000) ou só começa nova rodada
                            r.game_state.resetar_jogo();

                            // Manda o novo estado (cartas limpas) para todos
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
                // Erro vai apenas para quem tentou jogar
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
    if let Ok(json_msg) = serde_json::to_string(&MsgServidor::Notificacao(texto)) {
        for client_tx in room.clients.values() {
            let _ = client_tx.send(Message::text(json_msg.clone()));
        }
    }
}

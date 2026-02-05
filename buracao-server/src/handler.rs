use crate::state::{Clientes, JogoCompartilhado, NEXT_USER_ID};
use buracao_core::acoes::{AcaoJogador, MsgServidor};
use futures::{SinkExt, StreamExt};
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

pub async fn handle_connection(ws: WebSocket, jogo: JogoCompartilhado, clientes: Clientes) {
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
            // 1. Envia Boas Vindas
            let msg_bv = MsgServidor::BoasVindas {
                id_jogador: id_jogador_jogo,
            };
            if let Ok(json) = serde_json::to_string(&msg_bv) {
                let _ = sender.send(Message::text(json));
            }

            // 2. Envia Estado Inicial
            let visao = estado.gerar_visao_para_jogador(id_jogador_jogo);
            let msg_estado = MsgServidor::Estado(visao);

            if let Ok(json) = serde_json::to_string(&msg_estado) {
                let _ = sender.send(Message::text(json));
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

        if msg.is_close() {
            break;
        }

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
                    // Notificação individual
                    if let Some(sender) = clientes.get(&my_connection_id) {
                        let msg_notif = MsgServidor::Notificacao(msg);
                        if let Ok(json_notif) = serde_json::to_string(&msg_notif) {
                            let _ = sender.send(Message::text(json_notif));
                        }
                    }
                    // Broadcast Estado
                    for cliente in clientes.iter() {
                        let conn_id_destino = *cliente.key();
                        let sender = cliente.value();
                        let id_destino_jogo = ((conn_id_destino - 1) % 4) as u32;

                        let visao = estado.gerar_visao_para_jogador(id_destino_jogo);
                        let msg_envelope = MsgServidor::Estado(visao);

                        if let Ok(json_visao) = serde_json::to_string(&msg_envelope) {
                            let _ = sender.send(Message::text(json_visao));
                        }
                    }

                    // Check Fim de Jogo
                    if estado.partida_encerrada {
                        let vencedor = if estado.pontuacao_a >= estado.pontuacao_b {
                            0
                        } else {
                            1
                        };
                        let msg_fim = MsgServidor::FimDeJogo {
                            vencedor_time: vencedor,
                            pontos_a: estado.pontuacao_a,
                            pontos_b: estado.pontuacao_b,
                            motivo: "Pontuação Alcançada".to_string(),
                        };

                        if let Ok(json_fim) = serde_json::to_string(&msg_fim) {
                            for cliente in clientes.iter() {
                                let _ = cliente.value().send(Message::text(json_fim.clone()));
                            }
                        }

                        // Cronômetro
                        for i in (1..=15).rev() {
                            let msg_timer =
                                MsgServidor::Notificacao(format!("Nova partida em {}...", i));
                            if let Ok(json_timer) = serde_json::to_string(&msg_timer) {
                                for cliente in clientes.iter() {
                                    let _ = cliente.value().send(Message::text(json_timer.clone()));
                                }
                            }
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }

                        // Reseta e Reinicia
                        estado.resetar_jogo();
                        let msg_inicio =
                            MsgServidor::Notificacao("=== 🎲 NOVA PARTIDA ===\n".to_string());
                        let json_inicio = serde_json::to_string(&msg_inicio).unwrap_or_default();

                        for cliente in clientes.iter() {
                            let conn_id = *cliente.key();
                            let sender = cliente.value();
                            let _ = sender.send(Message::text(json_inicio.clone()));

                            let id_player = ((conn_id - 1) % 4) as u32;
                            let nova_visao = estado.gerar_visao_para_jogador(id_player);
                            let msg_estado = MsgServidor::Estado(nova_visao);
                            if let Ok(json) = serde_json::to_string(&msg_estado) {
                                let _ = sender.send(Message::text(json));
                            }
                        }
                    }
                }
                Err(e) => {
                    if let Some(sender) = clientes.get(&my_connection_id) {
                        let msg_erro = MsgServidor::Erro(format!("{}", e));
                        if let Ok(json_erro) = serde_json::to_string(&msg_erro) {
                            let _ = sender.send(Message::text(json_erro));
                        }
                    }
                }
            }
        }
    }

    println!("Cliente {} desconectou", my_connection_id);
    clientes.remove(&my_connection_id);
}

use super::state::{GameState, LogEntry};
use crate::components::notification::{Toast, ToastType};
use crate::utils::helper::{get_current_time, reconciliar_mao};
use crate::utils::mappers::verso_para_asset;
use buracao_core::acoes::{DetalheJogo, MsgServidor};

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};
use wasm_bindgen_futures::JsFuture;

#[derive(serde::Deserialize)]
struct EventoNomes {
    mapa: HashMap<u32, String>,
}

/// Inicializa a conexão WebSocket e retorna o canal de envio (Sender)
pub fn setup_websocket(
    state: GameState,
    device_id: String,
) -> Option<mpsc::UnboundedSender<String>> {
    if !state.in_game.get_untracked() {
        return None;
    }

    let (tx, mut rx) = mpsc::unbounded();

    spawn_local(async move {
        // 1. Determina a URL
        let ws_url = {
            let location = window().location();
            let protocol = if location.protocol().unwrap() == "https:" {
                "wss"
            } else {
                "ws"
            };
            let host = location.host().unwrap();
            if host.contains("3000") {
                "ws://127.0.0.1:8080/buraco".to_string()
            } else {
                format!("{}://{}/buraco", protocol, host)
            }
        };

        // 2. Tenta conectar
        let ws = match WebSocket::open(&ws_url) {
            Ok(ws) => ws,
            Err(e) => {
                leptos::logging::error!("Erro WS: {:?}", e);
                state.status_jogo.set("Erro na conexão!".to_string());
                return;
            }
        };

        let (mut write, mut read) = ws.split();
        state
            .status_jogo
            .set("Conectado! Aguardando jogo...".to_string());

        // 3. Envia Login Imediato
        let login_msg = serde_json::json!({
            "tipo": "Login",
            "device_id": device_id,
            "nome": state.player_name.get_untracked(),
            "sala": state.room_code.get_untracked()
        });

        if write
            .send(Message::Text(login_msg.to_string()))
            .await
            .is_err()
        {
            state.status_jogo.set("Erro ao autenticar".to_string());
            return;
        }

        // 4. Loop de Envio (Canal -> WebSocket)
        spawn_local(async move {
            while let Some(msg_json) = rx.next().await {
                let _ = write.send(Message::Text(msg_json)).await;
            }
        });

        // 5. Loop de Recebimento (WebSocket -> Lógica)
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                // Tenta fazer parse como MsgServidor (Estado/Erro/Notificação)
                if let Ok(msg_servidor) = serde_json::from_str::<MsgServidor>(&text) {
                    processar_mensagem(state, msg_servidor);
                }
                // Tenta fazer parse como EventoNomes (Lista de jogadores)
                else if let Ok(evento) = serde_json::from_str::<EventoNomes>(&text) {
                    state.mapa_nomes.set(evento.mapa);
                }
            }
        }

        state.status_jogo.set("Desconectado.".to_string());
    });

    Some(tx)
}

/// Processa a mensagem recebida e atualiza os sinais do GameState
fn processar_mensagem(state: GameState, msg: MsgServidor) {
    match msg {
        MsgServidor::BoasVindas { .. } => {
            leptos::logging::log!("👋 Boas vindas recebidas");
        }
        MsgServidor::Estado(visao) => {
            // A. Limpa seleção para evitar bugs visuais (índices inválidos)
            state.selected_indices.update(|s| s.clear());

            // B. Som de Compra (Se qtd monte diminuiu)
            let monte_antigo = state.qtd_monte.get_untracked();
            let monte_novo = visao.qtd_monte;
            if monte_novo < monte_antigo {
                tocar_som(state, false); // false = som de compra
            }

            // C. Lógica de Highlight (Comparar mesas antigas com novas)
            let mut changed_ids = HashSet::new();

            // Helper local para detectar mudanças
            let detect_changes = |old: &[DetalheJogo], new: &[DetalheJogo]| -> Vec<u32> {
                let mut mudou = Vec::new();
                let old_map: HashMap<u32, usize> =
                    old.iter().map(|j| (j.id, j.cartas.len())).collect();
                for jogo in new {
                    match old_map.get(&jogo.id) {
                        None => mudou.push(jogo.id), // Jogo novo
                        Some(&old_len) => {
                            if jogo.cartas.len() > old_len {
                                mudou.push(jogo.id); // Jogo cresceu
                            }
                        }
                    }
                }
                mudou
            };

            let changes_a = detect_changes(&state.mesa_a.get_untracked(), &visao.mesa_time_a);
            for id in changes_a {
                changed_ids.insert(id);
            }

            let changes_b = detect_changes(&state.mesa_b.get_untracked(), &visao.mesa_time_b);
            for id in changes_b {
                changed_ids.insert(id);
            }

            if !changed_ids.is_empty() {
                state
                    .highlighted_games
                    .update(|set| set.extend(changed_ids.clone()));
                let ids_to_clear = changed_ids;

                // Timeout para remover o brilho
                set_timeout(
                    move || {
                        state.highlighted_games.update(|set| {
                            for id in ids_to_clear {
                                set.remove(&id);
                            }
                        });
                    },
                    std::time::Duration::from_secs(2),
                );
            }

            // D. Atualização dos Dados Principais
            state.minha_mao.update(|mao_atual| {
                *mao_atual = reconciliar_mao(mao_atual, visao.minha_mao);
            });

            state.lixo_topo.set(visao.lixo);
            state.meu_id.set(visao.meu_id);
            state.qtd_cartas_jogadores.set(visao.qtd_cartas_jogadores);
            state.mesa_a.set(visao.mesa_time_a);
            state.mesa_b.set(visao.mesa_time_b);
            state.pontuacao_a.set(visao.pontuacao_a);
            state.pontuacao_b.set(visao.pontuacao_b);

            // Novos campos de Histórico
            state.historico_a.set(visao.historico_pontos_a);
            state.historico_b.set(visao.historico_pontos_b);

            state.tres_vermelhos_a.set(visao.tres_vermelho_time_a);
            state.tres_vermelhos_b.set(visao.tres_vermelho_time_b);
            state.sou_o_jogador_da_vez.set(visao.posso_jogar);

            // E. Som de "Sua Vez"
            let turno_antigo = state.turno_atual_id.get_untracked();
            let turno_novo = visao.turno_atual;
            let sou_eu = visao.meu_id;
            state.turno_atual_id.set(turno_novo);

            if turno_novo == sou_eu && turno_antigo != sou_eu {
                tocar_som(state, true); // true = som de turno
                add_toast(state, "Sua vez de jogar!".to_string(), ToastType::Info);

                state.game_log.update(|log| {
                    log.push(LogEntry {
                        time: get_current_time(),
                        msg: "--- SUA VEZ ---".to_string(),
                        is_error: false,
                        is_success: true, // Usa cor de destaque
                    });
                });
            }

            state.status_jogo.set(format!("Rodada {}", visao.rodada));

            // F. Limpeza de estados voláteis (Sucesso)
            state.jogos_preparados.set(Vec::new());
            state.ajuntes_lixo_preparados.set(Vec::new());
            state.lixo_selecionado.set(false);

            state.qtd_monte.set(monte_novo);
            state.qtd_lixo.set(visao.qtd_lixo);
            state
                .verso_monte
                .set(verso_para_asset(visao.verso_topo).to_string());
        }
        MsgServidor::Erro(e) => {
            add_toast(state, format!("ERRO: {}", e), ToastType::Error);

            // Limpa seleção
            state.selected_indices.update(|s| s.clear());

            // Recuperação Otimista: Devolve cartas preparadas para a mão se falhou
            let jogos_pendentes = state.jogos_preparados.get_untracked();
            if !jogos_pendentes.is_empty() {
                state.minha_mao.update(|mao| {
                    for jogo in jogos_pendentes {
                        mao.extend(jogo);
                    }
                    mao.sort();
                });
                state.jogos_preparados.set(Vec::new());
            }

            state.game_log.update(|log| {
                log.push(LogEntry {
                    time: get_current_time(),
                    msg: format!("ERRO: {}", e),
                    is_error: true,
                    is_success: false, // Usa cor de destaque
                });
            });
        }
        MsgServidor::Notificacao(n) => {
            add_toast(state, n.clone(), ToastType::Info);
            state.game_log.update(|log| {
                log.push(LogEntry {
                    time: get_current_time(),
                    msg: n.clone(),
                    is_error: false,
                    is_success: n.contains("bateu") || n.contains("Vencedor"), // Pinta de dourado se for vitória
                });
                if log.len() > 50 {
                    log.remove(0);
                }
            });
        }
        MsgServidor::FimDeJogo { vencedor_time, .. } => {
            state
                .status_jogo
                .set(format!("Vencedor: Time {}", vencedor_time));
            state.selected_indices.update(|s| s.clear());
        }
    }
}

// --- HELPERS INTERNOS ---

fn add_toast(state: GameState, msg: String, tipo: ToastType) {
    let id = state.next_toast_id.get_untracked();
    state.next_toast_id.set(id + 1);

    state.toasts.update(|t| {
        t.push(Toast {
            id,
            message: msg,
            toast_type: tipo,
        })
    });

    // Remove toast após 4s
    set_timeout(
        move || {
            state.toasts.update(|t| t.retain(|toast| toast.id != id));
        },
        std::time::Duration::from_secs(4),
    );
}

fn tocar_som(state: GameState, is_turn: bool) {
    let vol = state.volume.get_untracked();
    if vol <= 0.0 {
        return;
    }

    let element = if is_turn {
        state.my_turn_audio_ref.get()
    } else {
        state.draw_audio_ref.get()
    };

    if let Some(audio) = element {
        audio.set_volume(vol);
        audio.set_current_time(0.0);

        match audio.play() {
            Ok(promise) => {
                spawn_local(async move {
                    if let Err(e) = JsFuture::from(promise).await {
                        leptos::logging::warn!("⚠️ Som bloqueado: {:?}", e);
                    }
                });
            }
            Err(e) => leptos::logging::error!("❌ Erro ao tocar som: {:?}", e),
        }
    }
}

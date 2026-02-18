use super::state::{CartaIdentificada, GameState, LogEntry};
use crate::components::notification::{Toast, ToastType};
use crate::utils::helper::get_current_time;
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

// --- LÓGICA DE RECONCILIAÇÃO INTELIGENTE ---
// Compara a mão atual com a nova do servidor. Se a carta já existe, mantém o ID antigo.
// Isso impede que o Leptos destrua e recrie o elemento DOM, mantendo a animação fluida.
fn atualizar_mao_preservando_ids(
    state: GameState,
    nova_mao_server: Vec<buracao_core::baralho::Carta>,
) -> Vec<CartaIdentificada> {
    let mao_atual = state.minha_mao.get_untracked();
    let mut nova_mao_identificada = Vec::new();

    // Pool de cartas existentes para tentar reutilizar IDs
    let mut pool: Vec<CartaIdentificada> = mao_atual;

    for carta_server in nova_mao_server {
        // Verifica se já temos essa carta na mão local (comparação por valor/naipe)
        if let Some(idx) = pool.iter().position(|c| c.carta == carta_server) {
            // Achou! Remove do pool para evitar duplicatas e reusa o objeto com ID antigo
            nova_mao_identificada.push(pool.remove(idx));
        } else {
            // Carta nova (ex: acabou de comprar): Gera um ID visual novo
            let novo_id = state.unique_card_counter.get_untracked() + 1;
            state.unique_card_counter.set(novo_id);

            nova_mao_identificada.push(CartaIdentificada {
                id: novo_id,
                carta: carta_server,
            });
        }
    }

    nova_mao_identificada
}

/// Processa a mensagem recebida e atualiza os sinais do GameState
fn processar_mensagem(state: GameState, msg: MsgServidor) {
    match msg {
        MsgServidor::BoasVindas { .. } => {
            leptos::logging::log!("👋 Boas vindas recebidas");
        }
        MsgServidor::Estado(visao) => {
            // A. Limpa seleção para evitar bugs visuais (índices inválidos)
            state.selected_ids.update(|s| s.clear());

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

            // D. Atualização dos Dados Principais (COM PRESERVAÇÃO DE IDs)
            let nova_mao = atualizar_mao_preservando_ids(state, visao.minha_mao);
            state.minha_mao.set(nova_mao);

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
            state.selected_ids.update(|s| s.clear());

            // Recuperação Otimista: Devolve cartas preparadas para a mão se falhou
            // Gera novos IDs para garantir unicidade
            let jogos_pendentes = state.jogos_preparados.get_untracked();
            if !jogos_pendentes.is_empty() {
                state.minha_mao.update(|mao| {
                    let mut current_id = state.unique_card_counter.get_untracked();
                    for jogo in jogos_pendentes {
                        for carta in jogo {
                            current_id += 1;
                            mao.push(CartaIdentificada {
                                id: current_id,
                                carta,
                            });
                        }
                    }
                    // Atualiza o contador global
                    state.unique_card_counter.set(current_id);

                    // Ordenação opcional (se quiser manter organizado após erro)
                    // mao.sort_by(|a, b| a.carta.cmp(&b.carta));
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
            state.selected_ids.update(|s| s.clear());
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

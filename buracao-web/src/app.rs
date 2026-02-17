use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::html::Audio;
use leptos::prelude::window;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json;
use std::collections::{HashMap, HashSet};
use wasm_bindgen_futures::JsFuture;

use crate::components::board::Board;
use crate::components::controls::GameControls;
use crate::components::hand::Hand;
use crate::components::login::LoginScreen;
use crate::components::notification::{NotificationToast, Toast, ToastType};
use crate::components::scoreboard::{ScoreHistoryModal, Scoreboard};
use crate::components::settings::SettingsModal;
use crate::components::shortcut_manager::{KeyBindings, ShortcutManager};
use crate::components::table::Table;
use crate::components::turn_indicator::TurnIndicator;
use crate::utils::helper::reconciliar_mao;

use crate::utils::assets::get_card_path;
use crate::utils::mappers::{carta_para_asset, verso_para_asset};

use buracao_core::acoes::{AcaoJogador, DetalheJogo, MsgServidor};
use buracao_core::baralho::Carta;

#[derive(serde::Deserialize, Debug, Clone)]
struct EventoNomes {
    mapa: std::collections::HashMap<u32, String>,
}

#[component]
fn CardImage(
    carta: buracao_core::baralho::Carta,
    #[prop(default = "50px")] width: &'static str,
    theme: String,
) -> impl IntoView {
    let id = carta_para_asset(&carta);
    let src = get_card_path(&id, &theme);
    view! { <img src=src style=format!("width: {}; height: auto;", width) /> }
}

fn get_or_create_device_id() -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .ok()
        .flatten()
        .expect("no local storage");

    if let Ok(Some(id)) = storage.get_item("buraco_device_id") {
        id
    } else {
        let new_id = format!("user_{}", rand::random::<u32>());
        let _ = storage.set_item("buraco_device_id", &new_id);
        new_id
    }
}

const SOUND_PATH: &str = "/assets/audio/my_turn_xylophone.wav";

#[component]
pub fn App() -> impl IntoView {
    let (turno_atual_id, set_turno_atual_id) = signal(0_u32);
    let minha_mao = RwSignal::new(Vec::<Carta>::new());
    let (lixo_topo, set_lixo_topo) = signal(Option::<Carta>::None);
    let (jogos_preparados, set_jogos_preparados) = signal(Vec::<Vec<Carta>>::new());
    let (ajuntes_lixo_preparados, set_ajuntes_lixo_preparados) =
        signal(Vec::<(u32, Vec<Carta>)>::new());
    let (mesa_a, set_mesa_a) = signal(Vec::<DetalheJogo>::new());
    let (mesa_b, set_mesa_b) = signal(Vec::<DetalheJogo>::new());
    let (pontuacao_a, set_pontuacao_a) = signal(0);
    let (pontuacao_b, set_pontuacao_b) = signal(0);
    let (tres_vermelhos_a, set_tres_vermelhos_a) = signal(Vec::<Carta>::new());
    let (tres_vermelhos_b, set_tres_vermelhos_b) = signal(Vec::<Carta>::new());
    let (meu_id, set_meu_id) = signal(0_u32);
    let (status_jogo, set_status_jogo) = signal("Conectando...".to_string());
    let (sou_o_jogador_da_vez, set_sou_o_jogador_da_vez) = signal(false);
    let (lixo_selecionado, set_lixo_selecionado) = signal(false);
    let selected_indices = RwSignal::new(HashSet::new());
    let (ws_sender, set_ws_sender) = signal(Option::<mpsc::UnboundedSender<String>>::None);

    let (show_settings, set_show_settings) = signal(false);
    let current_theme = RwSignal::new("/assets/cards/PaperCards".to_string());
    let card_scale = RwSignal::new(1.0);
    let hand_card_width =
        Signal::derive(move || format!("{}px", (100.0 * card_scale.get()) as i32));

    let (qtd_monte, set_qtd_monte) = signal(0_u32);
    let (qtd_lixo, set_qtd_lixo) = signal(0_u32);

    let board_width = Signal::derive(move || format!("{}px", (90.0 * card_scale.get()) as i32));
    let table_width = Signal::derive(move || format!("{}px", (80.0 * card_scale.get()) as i32));

    let (verso_monte, set_verso_monte) = signal("back_r".to_string());
    let (toasts, set_toasts) = signal(Vec::<Toast>::new());
    let next_toast_id = StoredValue::new(0_usize);

    let (in_game, set_in_game) = signal(false);
    let (player_name, set_player_name) = signal("".to_string());
    let (room_code, set_room_code) = signal("".to_string());
    let device_id = StoredValue::new(get_or_create_device_id());

    let (mapa_nomes, set_mapa_nomes) = signal(std::collections::HashMap::<u32, String>::new());
    let (qtd_cartas_jogadores, set_qtd_cartas_jogadores) = signal(Vec::<usize>::new());

    let my_turn_audio_ref = NodeRef::<Audio>::new();
    let draw_audio_ref = NodeRef::<Audio>::new();

    let (highlighted_games, set_highlighted_games) = signal(HashSet::<u32>::new());
    let (historico_a, set_historico_a) = signal(Vec::new());
    let (historico_b, set_historico_b) = signal(Vec::new());

    // --- ATALHOS ---
    let initial_keys = if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(Some(json)) = storage.get_item("buraco_keys") {
                serde_json::from_str(&json).unwrap_or_default()
            } else {
                KeyBindings::default()
            }
        } else {
            KeyBindings::default()
        }
    } else {
        KeyBindings::default()
    };
    let key_bindings = RwSignal::new(initial_keys);
    let (show_history, set_show_history) = signal(false);

    let add_toast = move |msg: String, tipo: ToastType| {
        let id = next_toast_id.get_value();
        next_toast_id.set_value(id + 1);
        set_toasts.update(|t| {
            t.push(Toast {
                id,
                message: msg,
                toast_type: tipo,
            })
        });
        set_timeout(
            move || {
                set_toasts.update(|t| t.retain(|toast| toast.id != id));
            },
            std::time::Duration::from_secs(4),
        );
    };

    let ao_entrar = Callback::new(move |(nome, sala): (String, String)| {
        minha_mao.set(Vec::new());
        set_mesa_a.set(Vec::new());
        set_mesa_b.set(Vec::new());
        set_jogos_preparados.set(Vec::new());
        set_status_jogo.set("Conectando à sala...".to_string());
        set_player_name.set(nome);
        set_room_code.set(sala);
        set_in_game.set(true);
    });

    let acao_sair = move |_| {
        set_in_game.set(false);
        let _ = window().location().reload();
    };

    let acao_resetar = move |_| {
        let window = window();
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("buraco_device_id");
        }
        let _ = window.location().reload();
    };

    let volume = RwSignal::new(0.8);

    Effect::new(move |_| {
        if let Some(win) = web_sys::window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(vol_str)) = storage.get_item("buraco_volume") {
                    if let Ok(val) = vol_str.parse::<f64>() {
                        volume.set(val);
                    }
                }
            }
        }
    });

    let tocar_som_sua_vez = move || {
        let vol = volume.get_untracked();
        if vol <= 0.0 {
            return;
        }
        if let Some(audio_element) = my_turn_audio_ref.get() {
            audio_element.set_volume(vol);
            audio_element.set_current_time(0.0);
            match audio_element.play() {
                Ok(promise) => {
                    spawn_local(async move {
                        if let Err(e) = JsFuture::from(promise).await {
                            leptos::logging::warn!("⚠️ [SOM] Bloqueado: {:?}", e);
                        }
                    });
                }
                Err(e) => leptos::logging::error!("❌ [SOM] Erro: {:?}", e),
            }
        }
    };

    let tocar_som_compra = move || {
        let vol = volume.get_untracked();
        if vol <= 0.0 {
            return;
        }
        if let Some(audio_element) = draw_audio_ref.get() {
            audio_element.set_volume(vol);
            audio_element.set_current_time(0.0);
            match audio_element.play() {
                Ok(promise) => {
                    spawn_local(async move {
                        if let Err(e) = JsFuture::from(promise).await {
                            leptos::logging::warn!("⚠️ Som de compra bloqueado: {:?}", e);
                        }
                    });
                }
                Err(e) => leptos::logging::error!("❌ Erro ao tocar som de compra: {:?}", e),
            }
        }
    };

    Effect::new(move |_| {
        if !in_game.get() {
            return;
        }
        let (tx, mut rx) = mpsc::unbounded();
        set_ws_sender.set(Some(tx));

        spawn_local(async move {
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

            let ws = match WebSocket::open(&ws_url) {
                Ok(ws) => ws,
                Err(e) => {
                    leptos::logging::error!("Erro WS: {:?}", e);
                    set_status_jogo.set("Erro na conexão!".to_string());
                    return;
                }
            };

            let (mut write, mut read) = ws.split();
            set_status_jogo.set("Conectado! Aguardando jogo...".to_string());

            let login_msg = serde_json::json!({
                "tipo": "Login",
                "device_id": device_id.get_value(),
                "nome": player_name.get_untracked(),
                "sala": room_code.get_untracked()
            });

            if write
                .send(Message::Text(login_msg.to_string()))
                .await
                .is_err()
            {
                set_status_jogo.set("Erro ao autenticar".to_string());
                return;
            }

            spawn_local(async move {
                while let Some(msg_json) = rx.next().await {
                    let _ = write.send(Message::Text(msg_json)).await;
                }
            });

            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(msg_servidor) = serde_json::from_str::<MsgServidor>(&text) {
                        match msg_servidor {
                            MsgServidor::BoasVindas { .. } => {
                                leptos::logging::log!("👋 Boas vindas")
                            }
                            MsgServidor::Estado(visao) => {
                                selected_indices.update(|s| s.clear());

                                let monte_antigo = qtd_monte.get_untracked();
                                let monte_novo = visao.qtd_monte;
                                if monte_novo < monte_antigo {
                                    tocar_som_compra();
                                }

                                let mut changed_ids = HashSet::new();
                                let detect_changes =
                                    |old: &[DetalheJogo], new: &[DetalheJogo]| -> Vec<u32> {
                                        let mut mudou = Vec::new();
                                        let old_map: HashMap<u32, usize> =
                                            old.iter().map(|j| (j.id, j.cartas.len())).collect();
                                        for jogo in new {
                                            match old_map.get(&jogo.id) {
                                                None => {
                                                    mudou.push(jogo.id);
                                                }
                                                Some(&old_len) => {
                                                    if jogo.cartas.len() > old_len {
                                                        mudou.push(jogo.id);
                                                    }
                                                }
                                            }
                                        }
                                        mudou
                                    };

                                let changes_a =
                                    detect_changes(&mesa_a.get_untracked(), &visao.mesa_time_a);
                                for id in changes_a {
                                    changed_ids.insert(id);
                                }
                                let changes_b =
                                    detect_changes(&mesa_b.get_untracked(), &visao.mesa_time_b);
                                for id in changes_b {
                                    changed_ids.insert(id);
                                }

                                if !changed_ids.is_empty() {
                                    set_highlighted_games
                                        .update(|set| set.extend(changed_ids.clone()));
                                    let ids_to_clear = changed_ids;
                                    set_timeout(
                                        move || {
                                            set_highlighted_games.update(|set| {
                                                for id in ids_to_clear {
                                                    set.remove(&id);
                                                }
                                            });
                                        },
                                        std::time::Duration::from_secs(2),
                                    );
                                }

                                minha_mao.update(|mao_atual| {
                                    *mao_atual = reconciliar_mao(mao_atual, visao.minha_mao);
                                });
                                set_lixo_topo.set(visao.lixo);
                                set_meu_id.set(visao.meu_id);
                                set_qtd_cartas_jogadores.set(visao.qtd_cartas_jogadores);
                                set_mesa_a.set(visao.mesa_time_a);
                                set_mesa_b.set(visao.mesa_time_b);
                                set_pontuacao_a.set(visao.pontuacao_a);
                                set_pontuacao_b.set(visao.pontuacao_b);
                                set_historico_a.set(visao.historico_pontos_a);
                                set_historico_b.set(visao.historico_pontos_b);
                                set_tres_vermelhos_a.set(visao.tres_vermelho_time_a);
                                set_tres_vermelhos_b.set(visao.tres_vermelho_time_b);
                                set_sou_o_jogador_da_vez.set(visao.posso_jogar);

                                let turno_antigo = turno_atual_id.get_untracked();
                                let turno_novo = visao.turno_atual;
                                let sou_eu = visao.meu_id;
                                set_turno_atual_id.set(turno_novo);

                                if turno_novo == sou_eu && turno_antigo != sou_eu {
                                    tocar_som_sua_vez();
                                    add_toast("Sua vez de jogar!".to_string(), ToastType::Info);
                                }

                                set_status_jogo.set(format!("Rodada {}", visao.rodada));
                                set_jogos_preparados.set(Vec::new());
                                set_ajuntes_lixo_preparados.set(Vec::new());
                                set_lixo_selecionado.set(false);
                                set_qtd_monte.set(monte_novo);
                                set_qtd_lixo.set(visao.qtd_lixo);
                                set_verso_monte.set(verso_para_asset(visao.verso_topo).to_string());
                            }
                            MsgServidor::Erro(e) => {
                                add_toast(format!("ERRO: {}", e), ToastType::Error);
                                selected_indices.update(|s| s.clear());
                                let jogos_pendentes = jogos_preparados.get();
                                if !jogos_pendentes.is_empty() {
                                    minha_mao.update(|mao| {
                                        for jogo in jogos_pendentes {
                                            mao.extend(jogo);
                                        }
                                        mao.sort();
                                    });
                                    set_jogos_preparados.set(Vec::new());
                                }
                            }
                            MsgServidor::Notificacao(n) => add_toast(n, ToastType::Info),
                            MsgServidor::FimDeJogo { vencedor_time, .. } => {
                                set_status_jogo.set(format!("Vencedor: Time {}", vencedor_time));
                                selected_indices.update(|s| s.clear());
                            }
                        }
                    } else if let Ok(evento) = serde_json::from_str::<EventoNomes>(&text) {
                        set_mapa_nomes.set(evento.mapa);
                    }
                }
            }
            set_status_jogo.set("Desconectado.".to_string());
        });
    });

    let enviar_acao = move |acao: AcaoJogador| {
        if let Some(sender) = ws_sender.get_untracked() {
            let json = serde_json::to_string(&acao).unwrap();
            let _ = sender.unbounded_send(json);
        }
    };

    // --- AÇÕES REFORMULADAS: 0-arity (move ||) ---
    // Isso permite serem chamadas tanto pelo teclado quanto pelo mouse,
    // apenas ignorando os argumentos nos callbacks.

    let acao_descartar = move || {
        let indices = selected_indices.get();
        if indices.len() != 1 {
            window()
                .alert_with_message("Selecione apenas 1 carta para descartar!")
                .unwrap();
            return;
        }
        let idx = *indices.iter().next().unwrap();
        let carta_opt = minha_mao.with(|cartas| cartas.get(idx).cloned());
        if let Some(carta) = carta_opt {
            enviar_acao(AcaoJogador::Descartar { carta });
            selected_indices.update(|s| s.clear());
        }
    };

    let acao_separar = move || {
        let mao_atual = minha_mao.get();
        let indices_set = selected_indices.get();
        if indices_set.len() < 3 {
            return;
        }

        let (sel_com_idx, resto_com_idx): (Vec<_>, Vec<_>) = mao_atual
            .into_iter()
            .enumerate()
            .partition(|(i, _)| indices_set.contains(i));
        let cartas_para_baixar: Vec<Carta> = sel_com_idx.into_iter().map(|(_, c)| c).collect();
        let nova_mao: Vec<Carta> = resto_com_idx.into_iter().map(|(_, c)| c).collect();

        set_jogos_preparados.update(|jogos| jogos.push(cartas_para_baixar));
        minha_mao.set(nova_mao);
        selected_indices.update(|s| s.clear());
    };

    let acao_devolver = move |idx_jogo_preparado: usize| {
        let mut jogo_removido = None;
        set_jogos_preparados.update(|jogos| {
            if idx_jogo_preparado < jogos.len() {
                jogo_removido = Some(jogos.remove(idx_jogo_preparado));
            }
        });
        if let Some(cartas) = jogo_removido {
            minha_mao.update(|mao| {
                mao.extend(cartas);
                mao.sort();
            });
        }
    };

    let acao_confirmar_baixa = move || {
        let jogos = jogos_preparados.get();
        if jogos.is_empty() {
            return;
        }
        enviar_acao(AcaoJogador::BaixarJogos { jogos });
    };

    let acao_comprar_monte = move || {
        enviar_acao(AcaoJogador::ComprarBaralho);
    };

    let confirmar_compra_lixo = move || {
        if !lixo_selecionado.get() {
            return;
        }
        let ajuntes_guardados = ajuntes_lixo_preparados.get();
        let mut novos_jogos = jogos_preparados.get();
        let indices = selected_indices.get();
        if !indices.is_empty() {
            let cartas_soltas: Vec<Carta> = minha_mao.with(|mao| {
                indices
                    .iter()
                    .filter_map(|&i| mao.get(i).cloned())
                    .collect()
            });
            novos_jogos.push(cartas_soltas);
        }

        if ajuntes_guardados.is_empty() && novos_jogos.is_empty() {
            let _ = window().alert_with_message(
                "Para pegar o lixo, faça um jogo novo ou ajunte em um existente.",
            );
            return;
        }

        enviar_acao(AcaoJogador::ComprarLixo {
            novos_jogos,
            cartas_em_jogos_existentes: ajuntes_guardados,
        });
        set_lixo_selecionado.set(false);
        set_ajuntes_lixo_preparados.set(Vec::new());
        selected_indices.update(|s| s.clear());
    };

    let toggle_lixo_selecao = move |_: web_sys::MouseEvent| {
        set_lixo_selecionado.update(|v| *v = !*v);
    };

    let acao_ajuntar = move |idx_jogo_mesa: usize| {
        let indices = selected_indices.get();
        if indices.is_empty() {
            let _ = window().alert_with_message("Selecione cartas da mão primeiro para ajuntar!");
            return;
        }
        let cartas_selecionadas: Vec<Carta> = minha_mao.with(|mao| {
            indices
                .iter()
                .filter_map(|&i| mao.get(i).cloned())
                .collect()
        });
        let sou_time_a = meu_id.get() % 2 == 0;
        let id_jogo_real = if sou_time_a {
            mesa_a.with(|m| m.get(idx_jogo_mesa).map(|jogo| jogo.id))
        } else {
            mesa_b.with(|m| m.get(idx_jogo_mesa).map(|jogo| jogo.id))
        };

        if let Some(id_real) = id_jogo_real {
            if lixo_selecionado.get() {
                let ajunte_do_lixo = vec![(id_real, cartas_selecionadas)];
                let jogos_novos_guardados = jogos_preparados.get();
                enviar_acao(AcaoJogador::ComprarLixo {
                    novos_jogos: jogos_novos_guardados,
                    cartas_em_jogos_existentes: ajunte_do_lixo,
                });
                set_lixo_selecionado.set(false);
                selected_indices.update(|s| s.clear());
            } else {
                enviar_acao(AcaoJogador::Ajuntar {
                    indice_jogo: id_real,
                    cartas: cartas_selecionadas,
                });
                selected_indices.update(|s| s.clear());
            }
        }
    };

    let e_minha_vez = move || sou_o_jogador_da_vez.get();
    let acao_organizar = move || {
        minha_mao.update(|mao| mao.sort());
    };

    view! {
        <Show when=move || in_game.get() fallback=move || view! { <LoginScreen on_enter=ao_entrar /> }>
            <div style=move || { let bg = if e_minha_vez() { "#388e3c" } else { "#1b5e20" }; format!("background-color: {}; height: 100vh; display: flex; flex-direction: column; font-family: sans-serif; color: white; overflow: hidden; transition: background-color 0.5s;", bg) }>
                <div style="flex-shrink: 0; background: rgba(0,0,0,0.2); padding: 10px 20px; display: flex; justify-content: space-between; align-items: center; box-shadow: 0 2px 4px rgba(0,0,0,0.2);">
                    <div style="display: flex; flex-direction: column; align-items: flex-start;">
                        <h1 style="margin: 0; font-size: 1.5rem; line-height: 1.2;">"Buracão Web"</h1>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <small style="opacity: 0.8; font-size: 0.85rem;">
                                {move || { let id = meu_id.get(); let time = if id % 2 == 0 { "Time A" } else { "Time B" }; format!("Meu ID: {} ({})", id, time) }}
                            </small>
                            <button on:click=move |_| set_show_settings.set(true) title="Configurações" style="background: transparent; border: none; cursor: pointer; font-size: 1.2rem; padding: 0; opacity: 0.7;">"⚙️"</button>
                        </div>
                        <div style="display: flex; flex-direction: row; gap: 5px; margin-left: 0px; margin-top: 5px;">
                            <button on:click=acao_sair title="Sair da sala (Mantém ID)" style="background: #d32f2f; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;">"Sair"</button>
                            <button on:click=acao_resetar title="Apagar sessão e gerar novo ID" style="background: #455a64; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;">"Novo ID"</button>
                        </div>
                    </div>
                    <div style="text-align: right; display: flex; gap: 20px; align-items: center;">
                        <div style="text-align: right;">
                            <strong style="color: #ffeb3b; font-size: 1.1rem; text-shadow: 1px 1px 2px black; display: block;">
                                {move || {
                                    let status = status_jogo.get();
                                    let vez_id = turno_atual_id.get();
                                    let eu = meu_id.get();
                                    let nomes = mapa_nomes.get();
                                    let texto_vez = if vez_id == eu { "SUA VEZ!".to_string() } else {
                                        let nome = nomes.get(&vez_id).cloned().unwrap_or(format!("Jogador {}", vez_id));
                                        format!("Vez de {}", nome)
                                    };
                                    format!("{}. {}", status, texto_vez)
                                }}
                            </strong>
                        </div>
                        <Scoreboard
                            pontuacao_a=pontuacao_a
                            pontuacao_b=pontuacao_b
                            my_id=meu_id
                            // Ao clicar no placar pequeno, setamos o sinal GLOBAL para true
                            on_click_expand=Callback::new(move |_| set_show_history.set(true))
                        />
                    </div>
                </div>

                <ShortcutManager
                    bindings=key_bindings
                    on_buy_deck=Callback::new(move |_| acao_comprar_monte())
                    on_discard=Callback::new(move |_| acao_descartar())
                    on_buy_trash=Callback::new(move |_| { set_lixo_selecionado.set(true); confirmar_compra_lixo(); })
                    on_sort=Callback::new(move |_| acao_organizar())
                    on_toggle_scoreboard=Callback::new(move |_| set_show_history.update(|v| *v = !*v))
                />

                // --- MODAL CENTRALIZADO ---
                // Agora o App controla quando ele aparece.
                <Show when=move || show_history.get() fallback=|| ()>
                    <ScoreHistoryModal
                        historico_a=historico_a
                        historico_b=historico_b
                        nomes=mapa_nomes
                        my_id=meu_id // Passamos o ID para ele saber as cores
                        scale=1.2    // Pode ajustar o zoom aqui
                        on_close=Callback::new(move |_| set_show_history.set(false))
                    />
                </Show>

                <div style="flex: 1; display: flex; flex-direction: row; justify-content: space-between; align-items: flex-start; padding: 20px; gap: 20px; overflow-y: auto;">
                    {move || {
                        let sou_time_a = meu_id.get() % 2 == 0;
                        let cb = if sou_time_a { Some(Callback::new(acao_ajuntar)) } else { None };
                        let titulo = if sou_time_a { "MEU TIME" } else { "TIME INIMIGO" };
                        view! { <Table titulo=titulo.to_string() jogos=mesa_a tres_vermelhos=tres_vermelhos_a on_click=cb theme=current_theme.get() card_width=table_width is_my_team=sou_time_a highlighted_ids=highlighted_games /> }
                    }}
                    <div style="display: flex; flex-direction: column; align-items: center; gap: 20px; flex-shrink: 0; margin-top: 40px;">
                        <Board lixo=lixo_topo lixo_selecionado=lixo_selecionado on_click_deck=Some(Callback::new(move |_| acao_comprar_monte())) on_click_trash=Some(Callback::new(toggle_lixo_selecao)) theme=current_theme.get() card_width=board_width qtd_monte=qtd_monte qtd_lixo=qtd_lixo verso_monte=verso_monte />
                        <div style="background: rgba(0,0,0,0.2); padding: 10px; border-radius: 50%; border: 1px solid rgba(255,255,255,0.1);">
                            <TurnIndicator my_id=meu_id current_turn=turno_atual_id names=mapa_nomes cards_count=qtd_cartas_jogadores />
                        </div>
                    </div>
                    {move || {
                        let sou_time_b = meu_id.get() % 2 != 0;
                        let cb = if sou_time_b { Some(Callback::new(acao_ajuntar)) } else { None };
                        let titulo = if sou_time_b { "MEU TIME" } else { "TIME INIMIGO" };
                        view! { <Table titulo=titulo.to_string() jogos=mesa_b tres_vermelhos=tres_vermelhos_b on_click=cb theme=current_theme.get() card_width=table_width is_my_team=sou_time_b highlighted_ids=highlighted_games /> }
                    }}
                </div>

                <div style="flex-shrink: 0; background: linear-gradient(to top, rgba(0,0,0,0.9) 20%, transparent); padding-bottom: 20px; position: relative; z-index: 10;">
                    <Show when=move || !jogos_preparados.get().is_empty() fallback=|| ()>
                        <div style="display: flex; justify-content: center; margin-bottom: 10px;">
                            <div style="background: rgba(0,0,0,0.5); padding: 10px; border-radius: 10px; border: 1px dashed #ffeb3b; text-align: center;">
                                <h4 style="margin: 0 0 10px 0; color: #ffeb3b; font-size: 12px;">"Jogos a Baixar"</h4>
                                <div style="display: flex; gap: 10px;">
                                    {move || jogos_preparados.get().into_iter().enumerate().map(|(idx, cartas)| {
                                        view! {
                                            <div on:click=move |_| acao_devolver(idx) style="cursor: pointer; display: flex; transform: scale(0.8);">
                                                {cartas.into_iter().map(|c| view! { <CardImage carta=c width="40px" theme=current_theme.get() /> }).collect::<Vec<_>>()}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <button on:click=move |_| acao_confirmar_baixa() style="margin-top: 5px; background: #2e7d32; color: white; border: none; padding: 5px 15px; border-radius: 4px; cursor: pointer;">"Confirmar"</button>
                            </div>
                        </div>
                    </Show>
                    <div style="display: flex; align-items: flex-end; gap: 20px; width: 100%; overflow: hidden; padding: 0 20px;">
                        <div style="flex-shrink: 0; margin-bottom: 20px;">
                            <GameControls lixo_selecionado=lixo_selecionado tem_jogos_preparados=Signal::derive(move || !jogos_preparados.get().is_empty())
                                on_descartar=Callback::new(move |_| acao_descartar())
                                on_separar=Callback::new(move |_| acao_separar())
                                on_ordenar=Callback::new(move |_| acao_organizar())
                                on_confirmar_lixo=Callback::new(move |_| confirmar_compra_lixo())
                                on_confirmar_baixa=Callback::new(move |_| acao_confirmar_baixa())
                                on_cancelar_lixo=Callback::new(move |_| { set_lixo_selecionado.set(false); set_ajuntes_lixo_preparados.set(Vec::new()); selected_indices.update(|s| s.clear()); }) />
                        </div>
                        <div style="flex-grow: 1; min-width: 0;">
                            {move || { let _mao = minha_mao.get(); view! { <Hand cartas=minha_mao card_width=hand_card_width theme=current_theme.get() selected_indices=selected_indices /> } }}
                        </div>
                        <SettingsModal show=show_settings on_close=Callback::new(move |_| set_show_settings.set(false)) current_theme_path=current_theme card_scale=card_scale volume=volume key_bindings=key_bindings />
                    </div>
                </div>
                <NotificationToast toasts=toasts />
                <audio node_ref=my_turn_audio_ref src=SOUND_PATH style="display: none;" />
                <audio node_ref=draw_audio_ref src="/assets/audio/draw1.ogg" style="display: none;" prop:preload="auto" />
            </div>
        </Show>
    }
}

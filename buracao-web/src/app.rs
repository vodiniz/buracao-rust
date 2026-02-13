use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::html::Audio;
use leptos::prelude::window;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json;
use std::collections::HashSet;
use std::time::Duration;
use wasm_bindgen_futures::JsFuture;

use crate::components::board::Board;
use crate::components::controls::GameControls;
use crate::components::hand::Hand;
use crate::components::login::LoginScreen;
use crate::components::notification::{NotificationToast, Toast, ToastType};
use crate::components::scoreboard::Scoreboard;
use crate::components::settings::SettingsModal;
use crate::components::table::Table;
use crate::components::turn_indicator::TurnIndicator;

use crate::utils::assets::get_card_path;
use crate::utils::mappers::{carta_para_asset, verso_para_asset};

use buracao_core::acoes::{AcaoJogador, DetalheJogo, MsgServidor};
use buracao_core::baralho::Carta;

#[derive(serde::Deserialize, Debug, Clone)]
struct EventoNomes {
    // tipo: String, // Não precisamos mapear o tipo aqui pois já filtramos antes
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
    let (minha_mao, set_minha_mao) = signal(Vec::<Carta>::new());
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

    // --- ESTADOS DE CONFIGURAÇÃO ---
    let (show_settings, set_show_settings) = signal(false);
    let current_theme = RwSignal::new("/assets/cards/PaperCards".to_string());
    let card_scale = RwSignal::new(1.0);
    let hand_card_width =
        Signal::derive(move || format!("{}px", (100.0 * card_scale.get()) as i32));

    let (qtd_monte, set_qtd_monte) = signal(0_u32);
    let (qtd_lixo, set_qtd_lixo) = signal(0_u32);

    // --- SINAIS DERIVADOS DE TAMANHO ---
    let board_width = Signal::derive(move || format!("{}px", (90.0 * card_scale.get()) as i32));
    let table_width = Signal::derive(move || format!("{}px", (80.0 * card_scale.get()) as i32));

    let (verso_monte, set_verso_monte) = signal("back_r".to_string());

    let (toasts, set_toasts) = signal(Vec::<Toast>::new());
    let next_toast_id = StoredValue::new(0_usize);

    // --- ESTADO DO LOGIN ---
    let (in_game, set_in_game) = signal(false); // false = Tela de Login, true = Jogo
    let (player_name, set_player_name) = signal("".to_string());
    let (room_code, set_room_code) = signal("".to_string());
    let device_id = StoredValue::new(get_or_create_device_id());

    // NOVO: Mapa de Nomes para traduzir IDs
    let (mapa_nomes, set_mapa_nomes) = signal(std::collections::HashMap::<u32, String>::new());

    let (qtd_cartas_jogadores, set_qtd_cartas_jogadores) = signal(Vec::<usize>::new());

    let audio_ref = NodeRef::<Audio>::new();

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
        set_minha_mao.set(Vec::new());
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

    // 1. CONFIGURAÇÃO DE VOLUME
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

    // FUNÇÃO DE TOCAR SOM COM LOGS (Robusta com NodeRef)
    let tocar_som_sua_vez = move || {
        let vol = volume.get_untracked();

        if vol <= 0.0 {
            return;
        }

        // Tenta pegar o elemento <audio> que está no HTML
        if let Some(audio_element) = audio_ref.get() {
            // Configurações básicas
            audio_element.set_volume(vol);
            audio_element.set_current_time(0.0); // Reinicia o som se já estiver tocando

            // O .play() retorna Result<Promise, JsValue>
            match audio_element.play() {
                Ok(promise) => {
                    spawn_local(async move {
                        // JsFuture transforma a Promise do JS em algo que o Rust entende
                        if let Err(e) = JsFuture::from(promise).await {
                            // Erro comum: O usuário ainda não interagiu com a página
                            leptos::logging::warn!("⚠️ [SOM] Bloqueado pelo navegador: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    leptos::logging::error!("❌ [SOM] Erro ao chamar .play(): {:?}", e);
                }
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

            leptos::logging::log!("Tentando conectar no WebSocket em: {}", ws_url);
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

            leptos::logging::log!(">>> ENVIANDO LOGIN: {}", login_msg.to_string());

            if let Err(e) = write.send(Message::Text(login_msg.to_string())).await {
                leptos::logging::error!("Falha crítica ao enviar login: {:?}", e);
                set_status_jogo.set("Erro ao autenticar".to_string());
                return;
            }

            spawn_local(async move {
                while let Some(msg_json) = rx.next().await {
                    if let Err(e) = write.send(Message::Text(msg_json)).await {
                        leptos::logging::error!("Falha envio: {:?}", e);
                    }
                }
            });

            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    // TENTATIVA 1: É uma mensagem padrão do jogo?
                    if let Ok(msg_servidor) = serde_json::from_str::<MsgServidor>(&text) {
                        match msg_servidor {
                            MsgServidor::BoasVindas { .. } => {
                                // Apenas ignora ou loga, já que estamos usando a mensagem de Estado para sincronizar
                                leptos::logging::log!("👋 Boas vindas recebidas");
                            }
                            MsgServidor::Estado(visao) => {
                                // 1. Atualiza dados básicos
                                set_minha_mao.set(visao.minha_mao);
                                set_lixo_topo.set(visao.lixo);
                                set_meu_id.set(visao.meu_id);

                                set_qtd_cartas_jogadores.set(visao.qtd_cartas_jogadores);

                                set_mesa_a.set(visao.mesa_time_a);
                                set_mesa_b.set(visao.mesa_time_b);

                                set_pontuacao_a.set(visao.pontuacao_a);
                                set_pontuacao_b.set(visao.pontuacao_b);

                                set_tres_vermelhos_a.set(visao.tres_vermelho_time_a);
                                set_tres_vermelhos_b.set(visao.tres_vermelho_time_b);

                                set_sou_o_jogador_da_vez.set(visao.posso_jogar);

                                // --- LÓGICA CORRIGIDA DE SOM E TURNO ---

                                // 1. Captura o estado ANTIGO (antes de atualizar o sinal)
                                let turno_antigo = turno_atual_id.get_untracked();

                                // 2. Pega os dados NOVOS
                                let turno_novo = visao.turno_atual;
                                let sou_eu = visao.meu_id;

                                // 3. AGORA SIM: Atualiza o estado visual para o novo
                                set_turno_atual_id.set(turno_novo);

                                // 4. Verifica se houve MUDANÇA para a MINHA vez
                                if turno_novo == sou_eu {
                                    if turno_antigo != sou_eu {
                                        leptos::logging::log!(
                                            "🔔 [SOM] Mudança de turno detectada ({} -> {}). Tocando!",
                                            turno_antigo,
                                            turno_novo
                                        );
                                        tocar_som_sua_vez();
                                        add_toast("Sua vez de jogar!".to_string(), ToastType::Info);
                                    } else {
                                        leptos::logging::log!(
                                            "ℹ️ [SOM] Já era minha vez. Silêncio."
                                        );
                                    }
                                } else {
                                    leptos::logging::log!(
                                        "zzz [SOM] Vez do jogador {}.",
                                        turno_novo
                                    );
                                }

                                // ---------------------------------------

                                // Atualiza texto de status (com número da rodada apenas)
                                // O "Vez de..." agora é calculado no view!
                                set_status_jogo.set(format!("Rodada {}", visao.rodada));

                                set_jogos_preparados.set(Vec::new());

                                set_qtd_monte.set(visao.qtd_monte);
                                set_qtd_lixo.set(visao.qtd_lixo);

                                let nome_arquivo = verso_para_asset(visao.verso_topo);
                                set_verso_monte.set(nome_arquivo.to_string());
                            }
                            MsgServidor::Erro(e) => {
                                add_toast(format!("ERRO: {}", e), ToastType::Error);
                                selected_indices.update(|s| s.clear());

                                let jogos_pendentes = jogos_preparados.get();
                                if !jogos_pendentes.is_empty() {
                                    set_minha_mao.update(|mao| {
                                        for jogo in jogos_pendentes {
                                            mao.extend(jogo);
                                        }
                                        mao.sort();
                                    });
                                    set_jogos_preparados.set(Vec::new());
                                }

                                // Não precisamos mais do set_timeout para corrigir o texto,
                                // pois o view! é reativo e recalcula tudo automaticamente.
                            }
                            MsgServidor::Notificacao(n) => {
                                add_toast(n, ToastType::Info);
                            }
                            MsgServidor::FimDeJogo { vencedor_time, .. } => {
                                set_status_jogo.set(format!("Vencedor: Time {}", vencedor_time));
                            }
                        }
                    }
                    // TENTATIVA 2: É a lista de nomes?
                    else if let Ok(evento) = serde_json::from_str::<EventoNomes>(&text) {
                        leptos::logging::log!("👥 [NOMES] Recebi lista: {:?}", evento.mapa);
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

    let acao_descartar = move |_| {
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

    let reset_preparacao = move || {
        let preparados = jogos_preparados.get();
        if preparados.is_empty() {
            return;
        }

        set_minha_mao.update(|mao| {
            for jogo in preparados {
                mao.extend(jogo);
            }
            mao.sort();
        });
        set_jogos_preparados.set(Vec::new());
    };

    let acao_separar = move |_| {
        let mao_atual = minha_mao.get();
        let indices_set = selected_indices.get();

        if indices_set.len() < 3 {
            return;
        }

        let (cartas_selecionadas_com_idx, cartas_restantes_com_idx): (Vec<_>, Vec<_>) = mao_atual
            .into_iter()
            .enumerate()
            .partition(|(i, _)| indices_set.contains(i));

        let cartas_para_baixar: Vec<Carta> = cartas_selecionadas_com_idx
            .into_iter()
            .map(|(_, c)| c)
            .collect();
        let nova_mao: Vec<Carta> = cartas_restantes_com_idx
            .into_iter()
            .map(|(_, c)| c)
            .collect();

        set_jogos_preparados.update(|jogos| {
            jogos.push(cartas_para_baixar);
        });

        set_minha_mao.set(nova_mao);
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
            set_minha_mao.update(|mao| {
                mao.extend(cartas);
                mao.sort();
            });
        }
    };

    let acao_confirmar_baixa = move |_| {
        let jogos = jogos_preparados.get();

        if jogos.is_empty() {
            return;
        }

        enviar_acao(AcaoJogador::BaixarJogos { jogos });
    };

    let acao_comprar_monte = move |_| {
        enviar_acao(AcaoJogador::ComprarBaralho);
    };

    let confirmar_compra_lixo = move |_| {
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
            novos_jogos: novos_jogos,
            cartas_em_jogos_existentes: ajuntes_guardados,
        });

        set_lixo_selecionado.set(false);
        set_jogos_preparados.set(Vec::new());
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
                set_ajuntes_lixo_preparados.set(Vec::new());
                set_jogos_preparados.set(Vec::new());
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

    let acao_organizar = move |_: web_sys::MouseEvent| {
        set_minha_mao.update(|mao| {
            mao.sort();
        });
    };

    view! {
        <Show
        when=move || in_game.get()
        fallback=move || view! { <LoginScreen on_enter=ao_entrar /> }
        >
            <div style=move || {
                let bg = if e_minha_vez() { "#388e3c" } else { "#1b5e20" };
                format!("background-color: {}; height: 100vh; display: flex; flex-direction: column; font-family: sans-serif; color: white; overflow: hidden; transition: background-color 0.5s;", bg)
            }>
                // --- HEADER ---
                <div style="
                    flex-shrink: 0;
                    background: rgba(0,0,0,0.2);
                    padding: 10px 20px;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.2);
                ">
                    // LADO ESQUERDO
                    <div style="display: flex; flex-direction: column; align-items: flex-start;">
                        <h1 style="margin: 0; font-size: 1.5rem; line-height: 1.2;">"Buracão Web"</h1>
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <small style="opacity: 0.8; font-size: 0.85rem;">
                                {move || {
                                    let id = meu_id.get();
                                    let time = if id % 2 == 0 { "Time A" } else { "Time B" };
                                    format!("Meu ID: {} ({})", id, time)
                                }}
                            </small>
                            <button
                                on:click=move |_| set_show_settings.set(true)
                                title="Configurações"
                                style="background: transparent; border: none; cursor: pointer; font-size: 1.2rem; padding: 0; opacity: 0.7;"
                            >
                                "⚙️"
                            </button>
                        </div>

                        // NOVOS BOTÕES
                        <div style="display: flex; flex-direction: row; gap: 5px; margin-left: 0px; margin-top: 5px;">
                            <button
                                on:click=acao_sair
                                title="Sair da sala (Mantém ID)"
                                style="background: #d32f2f; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;"
                            >
                                "Sair"
                            </button>
                            <button
                                on:click=acao_resetar
                                title="Apagar sessão e gerar novo ID"
                                style="background: #455a64; color: white; border: none; border-radius: 4px; padding: 5px 10px; cursor: pointer; font-size: 0.8rem;"
                            >
                                "Novo ID"
                            </button>
                        </div>
                    </div>

                    // LADO DIREITO: Status e Placar (ATUALIZADO)
                    <div style="text-align: right; display: flex; gap: 20px; align-items: center;">
                        <div style="text-align: right;">
                            <strong style="color: #ffeb3b; font-size: 1.1rem; text-shadow: 1px 1px 2px black; display: block;">
                                {move || {
                                    let status = status_jogo.get();
                                    // Pega o número da rodada (vem do set_status_jogo ou usa fallback)
                                    // Assumindo que status_jogo é "Rodada X"

                                    let vez_id = turno_atual_id.get();
                                    let eu = meu_id.get();
                                    let nomes = mapa_nomes.get();

                                    let texto_vez = if vez_id == eu {
                                        "SUA VEZ!".to_string()
                                    } else {
                                        // Busca o nome no mapa, ou usa o ID como fallback
                                        let nome = nomes.get(&vez_id).cloned().unwrap_or(format!("Jogador {}", vez_id));
                                        format!("Vez de {}", nome)
                                    };

                                    // Formatação final: "Rodada X. Vez de Fulano"
                                    format!("{}. {}", status, texto_vez)
                                }}
                            </strong>
                        </div>
                        <Scoreboard pontuacao_a=pontuacao_a pontuacao_b=pontuacao_b />
                    </div>
                </div>

                // --- 2. ÁREA CENTRAL (Mesas e Board) ---
                <div style="
                    flex: 1;
                    display: flex;
                    flex-direction: row;
                    justify-content: space-between;
                    align-items: flex-start;
                    padding: 20px;
                    gap: 20px;
                    overflow-y: auto;
                ">
                    // MESA TIME A
                    {move || {
                        let sou_time_a = meu_id.get() % 2 == 0;
                        let cb = if sou_time_a { Some(Callback::new(acao_ajuntar)) } else { None };
                        let titulo = if sou_time_a { "MEU TIME" } else { "TIME INIMIGO" };
                        view! {
                            <Table
                                titulo=titulo.to_string()
                                jogos=mesa_a
                                tres_vermelhos=tres_vermelhos_a
                                on_click=cb
                                theme=current_theme.get()
                                card_width=table_width
                                is_my_team=sou_time_a
                            />
                        }
                    }}

                    // --- COLUNA CENTRAL (Board + Indicador) ---
                    <div style="
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 20px;
                        flex-shrink: 0;
                        margin-top: 40px;
                    ">
                        <Board
                            lixo=lixo_topo
                            lixo_selecionado=lixo_selecionado
                            on_click_deck=Some(Callback::new(move |_| acao_comprar_monte(())))
                            on_click_trash=Some(Callback::new(toggle_lixo_selecao))
                            theme=current_theme.get()
                            card_width=board_width
                            qtd_monte=qtd_monte
                            qtd_lixo=qtd_lixo
                            verso_monte=verso_monte
                        />

                        // O Indicador agora está DENTRO da coluna central
                        <div style="
                            background: rgba(0,0,0,0.2);
                            padding: 10px;
                            border-radius: 50%;
                            border: 1px solid rgba(255,255,255,0.1);
                        ">
                            <TurnIndicator
                                my_id=meu_id
                                current_turn=turno_atual_id
                                names=mapa_nomes
                                cards_count=qtd_cartas_jogadores
                            />
                        </div>
                    </div>

                    // MESA TIME B
                    {move || {
                        let sou_time_b = meu_id.get() % 2 != 0;
                        let cb = if sou_time_b { Some(Callback::new(acao_ajuntar)) } else { None };
                        let titulo = if sou_time_b { "MEU TIME" } else { "TIME INIMIGO" };
                        view! {
                            <Table
                                titulo=titulo.to_string()
                                jogos=mesa_b
                                tres_vermelhos=tres_vermelhos_b
                                on_click=cb
                                theme=current_theme.get()
                                card_width=table_width
                                is_my_team=sou_time_b
                            />
                        }
                    }}
                </div>

                // --- 3. ÁREA INFERIOR (Mão e Ações) ---
                <div style="
                    flex-shrink: 0;
                    background: linear-gradient(to top, rgba(0,0,0,0.9) 20%, transparent);
                    padding-bottom: 20px;
                    position: relative;
                    z-index: 10;
                ">
                    // ÁREA DE PREPARAÇÃO
                    {move || {
                        let jogos = jogos_preparados.get();
                        if !jogos.is_empty() {
                            view! {
                                <div style="display: flex; justify-content: center; margin-bottom: 10px;">
                                    <div style="background: rgba(0,0,0,0.5); padding: 10px; border-radius: 10px; border: 1px dashed #ffeb3b; text-align: center;">
                                        <h4 style="margin: 0 0 10px 0; color: #ffeb3b; font-size: 12px;">"Jogos a Baixar"</h4>
                                        <div style="display: flex; gap: 10px;">
                                            {jogos.into_iter().enumerate().map(|(idx, cartas)| {
                                                view! {
                                                    <div on:click=move |_| acao_devolver(idx) style="cursor: pointer; display: flex; transform: scale(0.8);">
                                                        {cartas.into_iter().map(|c| view! {
                                                            <CardImage
                                                                carta=c
                                                                width="40px"
                                                                theme=current_theme.get()
                                                            />
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        <button on:click=move |ev| acao_confirmar_baixa(ev) style="margin-top: 5px; background: #2e7d32; color: white; border: none; padding: 5px 15px; border-radius: 4px; cursor: pointer;">
                                            "Confirmar"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}

                    // CONTAINER FLEX: CONTROLES + MÃO
                    <div style="display: flex; align-items: flex-end; gap: 20px; width: 100%; overflow: hidden; padding: 0 20px;">
                        // CONTROLES
                        <div style="flex-shrink: 0; margin-bottom: 20px;">
                            <GameControls
                                lixo_selecionado=lixo_selecionado
                                tem_jogos_preparados=Signal::derive(move || !jogos_preparados.get().is_empty())
                                on_descartar=Callback::new(acao_descartar)
                                on_separar=Callback::new(acao_separar)
                                on_ordenar=Callback::new(acao_organizar)
                                on_confirmar_lixo=Callback::new(confirmar_compra_lixo)
                                on_confirmar_baixa=Callback::new(acao_confirmar_baixa)
                                on_cancelar_lixo=Callback::new(move |_| {
                                    set_lixo_selecionado.set(false);
                                    set_ajuntes_lixo_preparados.set(Vec::new());
                                    selected_indices.update(|s| s.clear());
                                })
                            />
                        </div>

                        // MÃO
                        <div style="flex-grow: 1; min-width: 0;">
                            {move || {
                                let _mao = minha_mao.get();
                                view! {
                                    <Hand
                                        cartas=minha_mao
                                        card_width=hand_card_width
                                        theme=current_theme.get()
                                        selected_indices=selected_indices
                                    />
                                }
                            }}
                        </div>

                        // SETTINGS
                        <SettingsModal
                            show=show_settings
                            on_close=Callback::new(move |_| set_show_settings.set(false))
                            current_theme_path=current_theme
                            card_scale=card_scale
                            volume=volume
                        />
                    </div>
                </div>

                <NotificationToast toasts=toasts />
                <audio
                    node_ref=audio_ref
                    src=SOUND_PATH
                    style="display: none;"
                />
            </div>
        </Show>
    }
}

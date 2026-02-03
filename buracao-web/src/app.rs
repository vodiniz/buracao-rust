use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::prelude::window;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json;
use std::collections::HashSet;

use crate::components::board::Board;
use crate::components::hand::Hand;
use crate::components::table::Table;
use crate::utils::assets::get_card_path;

use crate::utils::mappers::carta_para_asset;

// CORREÇÃO: Usar DetalheJogo aqui também
use buracao_core::acoes::{AcaoJogador, DetalheJogo, MsgServidor};
use buracao_core::baralho::Carta;

#[component]
fn CardImage(carta: Carta, #[prop(default = "50px")] width: &'static str) -> impl IntoView {
    let src = carta_para_asset(&carta);
    view! {
        <img
            src=src
            style=format!("width: {}; height: auto; display: block; border-radius: 4px;", width)
            alt="carta"
        />
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (minha_mao, set_minha_mao) = signal(Vec::<Carta>::new());
    let (lixo_topo, set_lixo_topo) = signal(Option::<Carta>::None);

    let (jogos_preparados, set_jogos_preparados) = signal(Vec::<Vec<Carta>>::new());

    // CORREÇÃO: O tipo do sinal agora é DetalheJogo
    let (mesa_a, set_mesa_a) = signal(Vec::<DetalheJogo>::new());
    let (mesa_b, set_mesa_b) = signal(Vec::<DetalheJogo>::new());

    let (meu_id, set_meu_id) = signal(0_u32);
    let (status_jogo, set_status_jogo) = signal("Conectando...".to_string());

    let selected_indices = RwSignal::new(HashSet::new());
    let (ws_sender, set_ws_sender) = signal(Option::<mpsc::UnboundedSender<String>>::None);

    Effect::new(move |_| {
        let (tx, mut rx) = mpsc::unbounded();
        set_ws_sender.set(Some(tx));

        spawn_local(async move {
            let ws_url = "ws://127.0.0.1:3030/buraco";
            let ws = match WebSocket::open(ws_url) {
                Ok(ws) => ws,
                Err(e) => {
                    leptos::logging::error!("Erro WS: {:?}", e);
                    set_status_jogo.set("Erro na conexão!".to_string());
                    return;
                }
            };

            let (mut write, mut read) = ws.split();
            set_status_jogo.set("Conectado! Aguardando jogo...".to_string());

            spawn_local(async move {
                while let Some(msg_json) = rx.next().await {
                    if let Err(e) = write.send(Message::Text(msg_json)).await {
                        leptos::logging::error!("Falha envio: {:?}", e);
                    }
                }
            });

            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    match serde_json::from_str::<MsgServidor>(&text) {
                        Ok(MsgServidor::Estado(visao)) => {
                            set_minha_mao.set(visao.minha_mao);
                            set_lixo_topo.set(visao.lixo);
                            set_meu_id.set(visao.meu_id);

                            // Agora os tipos batem (Vec<DetalheJogo>)
                            set_mesa_a.set(visao.mesa_time_a);
                            set_mesa_b.set(visao.mesa_time_b);

                            let turno = if visao.posso_jogar {
                                "SUA VEZ!"
                            } else {
                                "Aguarde..."
                            };
                            set_status_jogo.set(format!("Rodada {}. {}", visao.rodada, turno));
                        }
                        Ok(MsgServidor::Erro(e)) => set_status_jogo.set(format!("ERRO: {}", e)),
                        Ok(MsgServidor::Notificacao(n)) => {
                            leptos::logging::log!("Notificação: {}", n)
                        }
                        Ok(MsgServidor::FimDeJogo { vencedor_time, .. }) => {
                            set_status_jogo.set(format!("Vencedor: Time {}", vencedor_time));
                        }
                        _ => {}
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
            // CORREÇÃO: window() agora está importado corretamente
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

    let acao_separar = move |_| {
        // Pega os dados atuais (clonando para garantir estabilidade)
        let mao_atual = minha_mao.get();
        let indices_set = selected_indices.get(); // Supõe que seja um HashSet<usize>

        // LOG: Abra o console (F12) para ver isso
        leptos::logging::log!("Indices selecionados: {:?}", indices_set);
        leptos::logging::log!("Tamanho da mão: {}", mao_atual.len());

        if indices_set.len() < 3 {
            leptos::logging::log!("Menos de 3 cartas selecionadas, cancelando.");
            return;
        }

        // 1. Separação robusta usando partition
        // O `partition` divide o vetor em dois baseados na condição (true vai para esquerda, false para direita)
        let (cartas_selecionadas_com_idx, cartas_restantes_com_idx): (Vec<_>, Vec<_>) = mao_atual
            .into_iter()
            .enumerate()
            .partition(|(i, _)| indices_set.contains(i));

        // 2. Removemos os índices (que usamos apenas para separar) e ficamos só com as Cartas
        let cartas_para_baixar: Vec<Carta> = cartas_selecionadas_com_idx
            .into_iter()
            .map(|(_, c)| c)
            .collect();
        let nova_mao: Vec<Carta> = cartas_restantes_com_idx
            .into_iter()
            .map(|(_, c)| c)
            .collect();

        // LOG: Conferir se separou certo
        leptos::logging::log!(
            "Baixando {} cartas. Restam {} na mão.",
            cartas_para_baixar.len(),
            nova_mao.len()
        );

        // 3. Atualiza os sinais
        set_jogos_preparados.update(|jogos| {
            jogos.push(cartas_para_baixar);
        });

        set_minha_mao.set(nova_mao);

        // 4. Limpa a seleção
        selected_indices.update(|s| s.clear());
    };
    // 2. Permite desfazer se o usuário errou (clica no jogo preparado e ele volta pra mão)
    let acao_devolver = move |idx_jogo_preparado: usize| {
        // Variável para armazenar o jogo que vamos remover
        let mut jogo_removido = None;

        // Atualiza a lista removendo o item e salvando na variável acima
        set_jogos_preparados.update(|jogos| {
            if idx_jogo_preparado < jogos.len() {
                jogo_removido = Some(jogos.remove(idx_jogo_preparado));
            }
        });

        // Se conseguimos remover, devolvemos para a mão
        if let Some(cartas) = jogo_removido {
            set_minha_mao.update(|mao| {
                mao.extend(cartas);
                mao.sort(); // Reordena para ficar bonito
            });
        }
    };

    // 3. Essa é a que FINALMENTE envia pro servidor (substitui a parte de rede da antiga acao_baixar)
    let acao_confirmar_baixa = move |_| {
        let jogos = jogos_preparados.get();

        if jogos.is_empty() {
            return;
        }

        // Envia TODOS os jogos preparados de uma vez
        enviar_acao(AcaoJogador::BaixarJogos { jogos });

        // Limpa a preparação (o servidor vai mandar o novo estado da mão depois)
        set_jogos_preparados.set(Vec::new());
    };

    let acao_comprar_monte = move |_| {
        enviar_acao(AcaoJogador::ComprarBaralho);
    };

    let acao_comprar_lixo = move |_| {
        // Envia vetores vazios.
        // Se o servidor exigir que use a carta AGORA, isso retornará erro "Regra violada".
        // Se o servidor for flexível, ele entregará o lixo.
        enviar_acao(AcaoJogador::ComprarLixo {
            novos_jogos: vec![],
            cartas_em_jogos_existentes: vec![],
        });
    };

    // AÇÃO: AJUNTAR (Colocar carta em jogo existente)
    // Recebe o índice do jogo clicado na Table
    let acao_ajuntar = move |idx_jogo: usize| {
        let indices_set = selected_indices.get();

        if indices_set.is_empty() {
            // Se clicar no jogo sem selecionar nada, não faz nada
            return;
        }

        // Converte índices selecionados em Vetor de Cartas
        let cartas_para_ajuntar: Vec<Carta> = minha_mao.with(|mao| {
            indices_set
                .iter()
                .filter_map(|&idx| mao.get(idx).cloned())
                .collect()
        });

        // Envia a ação Ajuntar
        enviar_acao(AcaoJogador::Ajuntar {
            indice_jogo: idx_jogo as u32, // Convertendo usize -> u32
            cartas: cartas_para_ajuntar,
        });

        // Limpa a seleção da mão
        selected_indices.update(|s| s.clear());
    };

    let e_minha_vez = move || {
        let texto = status_jogo.get();
        texto.contains("SUA VEZ") // Jeito "rápido" baseada na string que montamos antes
    };

    let acao_organizar = move |_: web_sys::MouseEvent| {
        set_minha_mao.update(|mao| {
            // Aqui você usa o seu método de ordenar (seja o sort nativo ou sua função do core)
            mao.sort();
        });
    };

    view! {
        // 1. DIV PRINCIPAL (Container Verde)
        <div style=move || {
            let bg = if e_minha_vez() { "#388e3c" } else { "#1b5e20" };
            format!("background-color: {}; min-height: 100vh; display: flex; flex-direction: column; font-family: sans-serif; color: white; overflow-x: hidden; transition: background-color 0.5s;", bg)
        }>
            // HEADER
            <div style="background: rgba(0,0,0,0.2); padding: 15px; display: flex; justify-content: space-between; align_items: center; box-shadow: 0 2px 4px rgba(0,0,0,0.2);">
                <h1 style="margin: 0; font-size: 1.5rem;">"Buracão Web"</h1>
                <div style="text-align: right;">
                    <strong style="color: #ffeb3b; font-size: 1.2rem;">{move || status_jogo.get()}</strong>
                    <br/>
                    <small>"Meu ID: " {move || meu_id.get()}</small>
                </div>
            </div>

            // 2. ÁREA CENTRAL
            <div style="flex: 1; display: flex; flex-direction: column; justify-content: flex-start; align-items: center; padding: 10px;">

                // MESA
                <Table
                    jogos_time_a=mesa_a
                    jogos_time_b=mesa_b
                    on_click_jogo_a=Callback::new(acao_ajuntar)
                />

                // TABULEIRO
                <Board
                    lixo=lixo_topo
                    on_click_deck=Some(Callback::new(move |_| acao_comprar_monte(())))
                    on_click_trash=Some(Callback::new(move |_| acao_comprar_lixo(())))
                />

                // ÁREA DE PREPARAÇÃO (Jogos a Baixar)
                {move || {
                    let jogos = jogos_preparados.get();
                    if !jogos.is_empty() {
                        view! {
                            <div style="background: rgba(0,0,0,0.3); width: 90%; margin: 10px 0; padding: 10px; border-radius: 10px; border: 2px dashed #ffeb3b; text-align: center;">
                                <h4 style="margin: 0 0 10px 0; color: #ffeb3b;">"Jogos a Baixar"</h4>

                                <div style="display: flex; gap: 20px; flex-wrap: wrap; justify-content: center; margin-bottom: 10px;">
                                    {jogos.into_iter().enumerate().map(|(idx, cartas)| {
                                        view! {
                                            <div
                                                on:click=move |_| acao_devolver(idx)
                                                style="background: rgba(255,255,255,0.1); padding: 5px; border-radius: 8px; cursor: pointer; display: flex;"
                                                title="Devolver para mão"
                                            >
                                            {cartas.into_iter().map(|c| view! { <CardImage carta=c /> }).collect::<Vec<_>>()}

                                                // Espaçador para o final do jogo
                                                <div style="width: 20px;"></div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                <button
                                    on:click=acao_confirmar_baixa
                                    style="background: #2e7d32; color: white; border: none; padding: 10px 30px; border-radius: 5px; cursor: pointer; font-weight: bold;"
                                >
                                    "CONFIRMAR BAIXA"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}

                // BOTÕES DE AÇÃO
                <div style="display: flex; gap: 20px; margin-top: 10px;">
                     <button
                        on:click=acao_descartar
                        style="padding: 12px 30px; font-size: 16px; font-weight: bold; background-color: #e53935; color: white; border: none; border-radius: 25px; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.3);"
                    >
                        "Descartar"
                     </button>

                     <button
                        on:click=acao_separar
                        style="padding: 12px 30px; font-size: 16px; font-weight: bold; background-color: #0288d1; color: white; border: none; border-radius: 25px; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.3);"
                    >
                        "Separar Jogo"
                     </button>
                </div>
            </div>

            // 3. MÃO DO JOGADOR
            <div style="background: linear-gradient(to top, rgba(0,0,0,0.8), transparent); padding-bottom: 20px;">
                <div style="display: flex; justify-content: center; margin-bottom: 5px;">
                     <button
                        on:click=acao_organizar
                        style="padding: 5px 15px; font-size: 12px; background-color: #fbc02d; color: #333; border: none; border-radius: 15px; cursor: pointer; font-weight: bold;"
                     >
                        "Ordenar Cartas"
                     </button>
                </div>

                {move || {
                    let _mao = minha_mao.get();
                    view! {
                        <Hand
                            cartas=minha_mao
                            selected_indices=selected_indices
                            card_width="110px"
                        />
                    }
                }}
            </div>
        </div>
    }
}

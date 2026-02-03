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

use crate::utils::mappers::{carta_para_asset, carta_para_asset_path};

// CORREÇÃO: Usar DetalheJogo aqui também
use buracao_core::acoes::{AcaoJogador, DetalheJogo, MsgServidor};
use buracao_core::baralho::Carta;

#[component]
fn CardImage(carta: Carta, #[prop(default = "50px")] width: &'static str) -> impl IntoView {
    let src = carta_para_asset_path(&carta);

    // Clonamos o src para usar no log e no tratamento de erro sem "roubar"
    // o valor que vai para a tag <img>
    let src_error = src.clone();

    view! {
        <img
            src=src.clone() // Aqui o src original é movido
            style=format!("width: {}; height: auto; display: block; border-radius: 4px;", width)
            alt=format!("{}", src.clone())
            on:error=move |_ev| {
                // Usamos a cópia clonada aqui
                leptos::logging::error!("Falha ao carregar imagem da carta: {}", src_error);
            }
        />
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (minha_mao, set_minha_mao) = signal(Vec::<Carta>::new());
    let (lixo_topo, set_lixo_topo) = signal(Option::<Carta>::None);

    let (jogos_preparados, set_jogos_preparados) = signal(Vec::<Vec<Carta>>::new());

    let (ajuntes_lixo_preparados, set_ajuntes_lixo_preparados) =
        signal(Vec::<(u32, Vec<Carta>)>::new());

    // CORREÇÃO: O tipo do sinal agora é DetalheJogo
    let (mesa_a, set_mesa_a) = signal(Vec::<DetalheJogo>::new());
    let (mesa_b, set_mesa_b) = signal(Vec::<DetalheJogo>::new());

    let (meu_id, set_meu_id) = signal(0_u32);
    let (status_jogo, set_status_jogo) = signal("Conectando...".to_string());

    let (lixo_selecionado, set_lixo_selecionado) = signal(false);

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
                            // 1. SUCESSO: O servidor aceitou a jogada (ou alguém jogou).
                            // Atualizamos a mão oficial vinda do servidor.
                            set_minha_mao.set(visao.minha_mao);
                            set_lixo_topo.set(visao.lixo);
                            set_meu_id.set(visao.meu_id);
                            set_mesa_a.set(visao.mesa_time_a);
                            set_mesa_b.set(visao.mesa_time_b);

                            let turno = if visao.posso_jogar {
                                "SUA VEZ!"
                            } else {
                                "Aguarde..."
                            };
                            set_status_jogo.set(format!("Rodada {}. {}", visao.rodada, turno));

                            // CORREÇÃO AQUI:
                            // Se recebemos um estado novo, significa que nossa jogada foi processada
                            // ou o jogo mudou. Podemos limpar a área de preparação com segurança.
                            set_jogos_preparados.set(Vec::new());
                        }
                        Ok(MsgServidor::Erro(e)) => {
                            set_status_jogo.set(format!("ERRO: {}", e));

                            // CORREÇÃO AQUI:
                            // Se deu erro, pegamos o que estava "preparado" e devolvemos para a mão manualmente,
                            // pois o servidor não enviou uma nova mão.
                            let jogos_pendentes = jogos_preparados.get();
                            if !jogos_pendentes.is_empty() {
                                set_minha_mao.update(|mao| {
                                    for jogo in jogos_pendentes {
                                        mao.extend(jogo);
                                    }
                                    mao.sort(); // Reordena
                                });
                                set_jogos_preparados.set(Vec::new()); // Agora sim limpamos
                                leptos::logging::log!(
                                    "Erro recebido: Cartas devolvidas para a mão."
                                );
                            }
                        }
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

    let reset_preparacao = move || {
        let preparados = jogos_preparados.get();
        if preparados.is_empty() {
            return;
        }

        set_minha_mao.update(|mao| {
            for jogo in preparados {
                mao.extend(jogo);
            }
            mao.sort(); // Mantém a mão organizada
        });
        set_jogos_preparados.set(Vec::new());
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

    // 3. Essa é a que FINALMENTE envia pro servidor
    let acao_confirmar_baixa = move |_| {
        let jogos = jogos_preparados.get();

        if jogos.is_empty() {
            return;
        }

        // Envia TODOS os jogos preparados de uma vez
        enviar_acao(AcaoJogador::BaixarJogos { jogos });

        // REMOVIDO: set_jogos_preparados.set(Vec::new());
        // NÃO limpe aqui. Deixe o WebSocket decidir se limpa (sucesso) ou devolve (erro).
    };

    let acao_comprar_monte = move |_| {
        enviar_acao(AcaoJogador::ComprarBaralho);
    };

    let confirmar_compra_lixo = move |_| {
        if !lixo_selecionado.get() {
            return;
        }

        // 1. Pega os ajuntes que preparamos clicando na mesa
        let ajuntes_guardados = ajuntes_lixo_preparados.get();

        // 2. Pega novos jogos que preparamos no botão "Separar"
        let mut novos_jogos = jogos_preparados.get();

        // 3. (Opcional) Se tiver cartas selecionadas na mão AGORA, tenta usar como jogo novo
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

        // 4. Envia tudo junto!
        enviar_acao(AcaoJogador::ComprarLixo {
            novos_jogos: novos_jogos,
            cartas_em_jogos_existentes: ajuntes_guardados, // <--- AQUI VÃO OS AJUNTES
        });

        // 5. Reset Geral
        set_lixo_selecionado.set(false);
        set_jogos_preparados.set(Vec::new());
        set_ajuntes_lixo_preparados.set(Vec::new()); // Limpa memória
        selected_indices.update(|s| s.clear());
    };

    // Apenas marca visualmente que queremos usar o lixo
    let toggle_lixo_selecao = move |_| {
        set_lixo_selecionado.update(|v| *v = !*v);
    };

    // AÇÃO: AJUNTAR (Colocar carta em jogo existente)
    // Recebe o índice do jogo clicado na Table
    // Recebe o índice VISUAL (0, 1, 2...) do jogo clicado na tabela
    let acao_ajuntar = move |idx_jogo_mesa: usize| {
        // 1. Validação Básica: Tem cartas selecionadas?
        let indices = selected_indices.get();
        if indices.is_empty() {
            let _ = window().alert_with_message("Selecione cartas da mão primeiro para ajuntar!");
            return;
        }

        // 2. Transforma índices da mão em Cartas Reais
        let cartas_selecionadas: Vec<Carta> = minha_mao.with(|mao| {
            indices
                .iter()
                .filter_map(|&i| mao.get(i).cloned())
                .collect()
        });

        // 3. Descobrir o ID REAL do jogo (Backend usa u32, Frontend usa índice usize)
        // Precisamos saber se sou Time A (0, 2) ou Time B (1, 3) para olhar na mesa certa
        let sou_time_a = meu_id.get() % 2 == 0;

        let id_jogo_real = if sou_time_a {
            mesa_a.with(|m| m.get(idx_jogo_mesa).map(|jogo| jogo.id))
        } else {
            mesa_b.with(|m| m.get(idx_jogo_mesa).map(|jogo| jogo.id))
        };

        // Se encontrou o jogo (segurança contra cliques fantasmas)
        if let Some(id_real) = id_jogo_real {
            // --- DECISÃO DE FLUXO ---

            if lixo_selecionado.get() {
                // MODO 1: ESTOU TENTANDO PEGAR O LIXO
                // Não envia nada ao servidor ainda. Apenas guarda na memória.
                set_ajuntes_lixo_preparados.update(|lista| {
                    lista.push((id_real, cartas_selecionadas));
                });

                // Feedback visual e limpeza
                leptos::logging::log!("Ajunte preparado (Lixo) no jogo ID: {}", id_real);
                selected_indices.update(|s| s.clear());
            } else {
                // MODO 2: JOGO NORMAL (Vez do jogador)
                // Envia imediatamente para o servidor processar
                enviar_acao(AcaoJogador::Ajuntar {
                    indice_jogo: id_real,
                    cartas: cartas_selecionadas,
                });

                // Limpa a seleção
                selected_indices.update(|s| s.clear());
            }
        } else {
            leptos::logging::error!("Jogo não encontrado no índice visual {}", idx_jogo_mesa);
        }
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
                    {move || {
                        let sou_time_a = meu_id.get() % 2 == 0;

                        // Definimos as variáveis explicitamente como Option<Callback<usize>>
                        let (cb_a, cb_b): (Option<Callback<usize>>, Option<Callback<usize>>) = if sou_time_a {
                            (Some(Callback::new(acao_ajuntar)), None)
                        } else {
                            (None, Some(Callback::new(acao_ajuntar)))
                        };

                        view! {
                            <Table
                                jogos_time_a=mesa_a
                                jogos_time_b=mesa_b
                                on_click_jogo_a=cb_a
                                on_click_jogo_b=cb_b
                            />
                        }
                    }}

            // TABULEIRO
                            <Board
                                lixo=lixo_topo
                                lixo_selecionado=lixo_selecionado // <--- Passando o sinal
                                on_click_deck=Some(Callback::new(move |_| acao_comprar_monte(())))
                                on_click_trash=Some(Callback::new(toggle_lixo_selecao)) // <--- Mudamos a ação aqui
                            />

            // ÁREA DE CONFIRMAÇÃO DO LIXO
                            {move || {
                                if lixo_selecionado.get() {
                                    let qtd_ajuntes = ajuntes_lixo_preparados.get().len();

                                    view! {
                                        <div style="margin: 10px; padding: 10px; background: rgba(255, 193, 7, 0.2); border: 1px solid #ffc107; border-radius: 8px; text-align: center; backdrop-filter: blur(5px);">
                                            <strong style="color: #ffc107; display: block; margin-bottom: 5px;">"MODO: COMPRAR LIXO"</strong>

                                            <ul style="text-align: left; color: white; font-size: 12px; list-style: none; padding: 0;">
                                                <li>"1. Selecione cartas e clique na Mesa para ajuntar."</li>
                                                <li>"2. Ou selecione cartas e clique em Confirmar para baixar novo jogo."</li>
                                            </ul>

                                            {if qtd_ajuntes > 0 {
                                                view! { <div style="color: #69f0ae; margin: 5px 0;">{format!("{} ajunte(s) preparado(s) na mesa!", qtd_ajuntes)}</div> }.into_any()
                                            } else {
                                                view! {}.into_any()
                                            }}

                                            <div style="display: flex; gap: 10px; justify-content: center; margin-top: 10px;">
                                                <button
                                                    on:click=confirmar_compra_lixo
                                                    style="background: #ffc107; color: black; border: none; padding: 8px 16px; border-radius: 4px; font-weight: bold; cursor: pointer;"
                                                >
                                                    "CONFIRMAR TUDO"
                                                </button>
                                                <button
                                                    on:click=move |_| {
                                                        // Cancelar limpa tudo
                                                        set_lixo_selecionado.set(false);
                                                        set_ajuntes_lixo_preparados.set(Vec::new());
                                                        selected_indices.update(|s| s.clear());
                                                    }
                                                    style="background: transparent; color: #ffc107; border: 1px solid #ffc107; padding: 8px 16px; border-radius: 4px; cursor: pointer;"
                                                >
                                                    "Cancelar"
                                                </button>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }
                            }}

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

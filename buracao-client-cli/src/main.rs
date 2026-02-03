use buracao_core::acoes::{AcaoJogador, MsgServidor, VisaoJogador};
use buracao_core::baralho::{Carta, Naipe, Valor}; // Importei Valor e Naipe explicitamente
use futures_util::{SinkExt, StreamExt};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

// --- ESTRUTURA PARA CONTROLAR O ESTADO E A MENSAGEM ---
struct ContextoLocal {
    visao: Option<VisaoJogador>,
    ultima_mensagem: String,
}

// Estado compartilhado agora guarda o contexto inteiro
type EstadoCompartilhado = Arc<Mutex<ContextoLocal>>;

#[tokio::main]
async fn main() {
    // 1. Conexão
    let connect_addr = "ws://127.0.0.1:3030/buraco";
    let url = Url::parse(connect_addr).unwrap();

    println!("Conectando em {}...", connect_addr);

    let (ws_stream, _) = connect_async(url.as_str())
        .await
        .expect("Falha ao conectar no Server");

    println!("✅ Conectado! Aguardando dados do jogo...");

    let (mut write, mut read) = ws_stream.split();

    // 2. Inicializa Estado com mensagem padrão
    let estado_atual: EstadoCompartilhado = Arc::new(Mutex::new(ContextoLocal {
        visao: None,
        ultima_mensagem: String::from("Conectado. Aguardando partida..."),
    }));

    let estado_clone_rede = estado_atual.clone();

    // --- TAREFA 1: RECEBER DADOS DO SERVIDOR ---
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                // Tenta deserializar o ENUM MsgServidor
                match serde_json::from_str::<MsgServidor>(&text) {
                    Ok(mensagem_servidor) => {
                        let mut guarda = estado_clone_rede.lock().unwrap();

                        match mensagem_servidor {
                            // 1. Atualização de Estado (Cartas, Mesa, etc)
                            MsgServidor::Estado(visao) => {
                                guarda.visao = Some(visao);
                                // Redesenha mantendo a mensagem que já estava lá
                                desenhar_jogo(&guarda.visao, &guarda.ultima_mensagem);
                            }

                            // 2. Notificações (Salva a msg e redesenha)
                            MsgServidor::Notificacao(txt) => {
                                guarda.ultima_mensagem = format!("🔔 {}", txt);
                                desenhar_jogo(&guarda.visao, &guarda.ultima_mensagem);
                            }

                            // 3. Erros de Regra
                            MsgServidor::Erro(erro_txt) => {
                                guarda.ultima_mensagem = format!("❌ ERRO: {}", erro_txt);
                                desenhar_jogo(&guarda.visao, &guarda.ultima_mensagem);
                            }

                            // 4. Boas vindas
                            MsgServidor::BoasVindas { id_jogador } => {
                                guarda.ultima_mensagem =
                                    format!("👋 Bem-vindo! Você é o Jogador ID: {}", id_jogador);
                                // Apenas printa, pois ainda não tem "Visao" pra desenhar
                                println!("\n{}", guarda.ultima_mensagem);
                            }

                            // 5. Fim de Jogo
                            MsgServidor::FimDeJogo { motivo, .. } => {
                                guarda.ultima_mensagem =
                                    format!("🏆 FIM DE JOGO! Motivo: {}", motivo);
                                desenhar_jogo(&guarda.visao, &guarda.ultima_mensagem);
                                // Não damos exit aqui para permitir ler a mensagem final
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "⚠️ Erro ao processar mensagem do servidor: {} | Msg: {}",
                            e, text
                        );
                    }
                }
            }
        }
    });

    // --- TAREFA 2: LER TECLADO E ENVIAR COMANDOS ---
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let _ = reader.read_line(&mut line).await;
        let input = line.trim();

        if input.is_empty() {
            continue;
        }

        // Recupera a mão do jogador do estado compartilhado de forma segura
        let mao_jogador = {
            let guarda = estado_atual.lock().unwrap();
            if let Some(ref visao) = guarda.visao {
                visao.minha_mao.clone()
            } else {
                Vec::new()
            }
        };

        let acao = processar_input(input, &mao_jogador);

        match acao {
            Some(a) => {
                let json = serde_json::to_string(&a).unwrap();
                if let Err(_) = write.send(Message::Text(json.into())).await {
                    println!("Erro ao enviar comando. Servidor caiu?");
                    break;
                }
            }
            None => {
                // Se o comando for inválido, apenas restaura o prompt
                // Opcional: Poderíamos redesenhar a tela aqui também
                print!("Comando > ");
                io::stdout().flush().unwrap();
            }
        }
    }
}

// --- PARSER DE COMANDOS (IGUAL AO ANTERIOR) ---
fn processar_input(input: &str, mao: &Vec<Carta>) -> Option<AcaoJogador> {
    let partes: Vec<&str> = input.split_whitespace().collect();

    if partes.is_empty() {
        return None;
    }

    match partes[0] {
        "c" => Some(AcaoJogador::ComprarBaralho),

        "l" => {
            // Sintaxe: l a <id> <idxs> / b <idxs> ...
            let resto_linha = partes[1..].join(" ");
            let grupos: Vec<&str> = resto_linha.split('/').collect();

            let mut novos_jogos_vec: Vec<Vec<Carta>> = Vec::new();
            let mut cartas_em_jogos_existentes_vec: Vec<(u32, Vec<Carta>)> = Vec::new();

            // 0 = indefinido, 1 = baixando (b), 2 = ajuntando (a)
            let mut ultimo_modo = 0;

            for grupo in grupos {
                let itens: Vec<&str> = grupo.trim().split_whitespace().collect();
                if itens.is_empty() {
                    continue;
                }

                let primeiro = itens[0];
                let indices_comecam_em;

                if primeiro == "b" {
                    ultimo_modo = 1;
                    indices_comecam_em = 1;
                } else if primeiro == "a" {
                    ultimo_modo = 2;
                    indices_comecam_em = 1;
                } else {
                    indices_comecam_em = 0;
                }

                match ultimo_modo {
                    1 => {
                        // Baixar
                        let indices = ler_varios_indices(&itens[indices_comecam_em..]);
                        let cartas: Vec<Carta> = indices
                            .iter()
                            .filter_map(|&i| mao.get(i).cloned())
                            .collect();
                        if !cartas.is_empty() {
                            novos_jogos_vec.push(cartas);
                        }
                    }
                    2 => {
                        // Ajuntar
                        let mut offset_idx = indices_comecam_em;
                        let mut id_jogo_alvo: Option<u32> = None;

                        if primeiro == "a" {
                            if let Some(id) = itens.get(1).and_then(|s| s.parse::<u32>().ok()) {
                                id_jogo_alvo = Some(id);
                                offset_idx += 1;
                            }
                        }

                        if let Some(id) = id_jogo_alvo {
                            let indices = ler_varios_indices(&itens[offset_idx..]);
                            let cartas: Vec<Carta> = indices
                                .iter()
                                .filter_map(|&i| mao.get(i).cloned())
                                .collect();
                            if !cartas.is_empty() {
                                cartas_em_jogos_existentes_vec.push((id, cartas));
                            }
                        }
                    }
                    _ => {}
                }
            }

            if novos_jogos_vec.is_empty() && cartas_em_jogos_existentes_vec.is_empty() {
                None
            } else {
                Some(AcaoJogador::ComprarLixo {
                    novos_jogos: novos_jogos_vec,
                    cartas_em_jogos_existentes: cartas_em_jogos_existentes_vec,
                })
            }
        }

        "d" => {
            if let Some(idx) = ler_indice(partes.get(1)) {
                if idx < mao.len() {
                    Some(AcaoJogador::Descartar {
                        carta: mao[idx].clone(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        "b" => {
            let argumentos = &partes[1..];
            let mut todos_os_jogos: Vec<Vec<Carta>> = Vec::new();
            let mut indices_jogo_atual: Vec<usize> = Vec::new();

            for item in argumentos {
                if *item == "/" {
                    if !indices_jogo_atual.is_empty() {
                        let cartas_jogo: Vec<Carta> = indices_jogo_atual
                            .iter()
                            .filter_map(|&i| mao.get(i).cloned())
                            .collect();
                        if !cartas_jogo.is_empty() {
                            todos_os_jogos.push(cartas_jogo);
                        }
                        indices_jogo_atual.clear();
                    }
                } else if let Ok(idx) = item.parse::<usize>() {
                    indices_jogo_atual.push(idx);
                }
            }
            if !indices_jogo_atual.is_empty() {
                let cartas_jogo: Vec<Carta> = indices_jogo_atual
                    .iter()
                    .filter_map(|&i| mao.get(i).cloned())
                    .collect();
                if !cartas_jogo.is_empty() {
                    todos_os_jogos.push(cartas_jogo);
                }
            }

            if !todos_os_jogos.is_empty() {
                Some(AcaoJogador::BaixarJogos {
                    jogos: todos_os_jogos,
                })
            } else {
                None
            }
        }

        "a" => {
            if let Some(id_jogo_parsed) = partes.get(1).and_then(|s| s.parse::<u32>().ok()) {
                let indices = ler_varios_indices(&partes[2..]);
                let cartas_ajunte: Vec<Carta> = indices
                    .iter()
                    .filter_map(|&i| mao.get(i).cloned())
                    .collect();

                if !cartas_ajunte.is_empty() {
                    Some(AcaoJogador::Ajuntar {
                        indice_jogo: id_jogo_parsed,
                        cartas: cartas_ajunte,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        "sair" => std::process::exit(0),
        _ => None,
    }
}

// --- DISPLAY (AGORA RECEBE A MENSAGEM) ---

fn desenhar_jogo(visao_opt: &Option<VisaoJogador>, mensagem_sistema: &str) {
    // 1. Limpa tela
    print!("\x1B[2J\x1B[1;1H");

    // Se ainda não tiver visão, mostra só a msg e sai
    let visao = match visao_opt {
        Some(v) => v,
        None => {
            println!("=== BURACO ONLINE (Conectando) ===");
            println!("\n>> {}\n", mensagem_sistema);
            return;
        }
    };

    println!("=== BURACO ONLINE (Eu sou ID: {}) ===", visao.meu_id);

    // 2. IMPRIME A MENSAGEM DO SERVIDOR EM DESTAQUE
    // Isso garante que "Comprou carta X" apareça aqui
    if !mensagem_sistema.is_empty() {
        println!("--------------------------------------------------");
        println!(" {}", mensagem_sistema);
        println!("--------------------------------------------------");
    }

    println!("{}", formatar_placar(visao));

    // MESA A
    println!("MESA TIME A:");
    if visao.mesa_time_a.is_empty() {
        println!("  (Mesa Vazia)");
    } else {
        for jogo in &visao.mesa_time_a {
            let visual = organizar_para_exibicao(&jogo.cartas);
            print!("  ID [{}]: ", jogo.id);
            for c in visual {
                print!("{} ", c);
            }
            println!("");
        }
    }

    // MESA B
    println!("MESA TIME B:");
    if visao.mesa_time_b.is_empty() {
        println!("  (Mesa Vazia)");
    } else {
        for jogo in &visao.mesa_time_b {
            let visual = organizar_para_exibicao(&jogo.cartas);
            print!("  ID [{}]: ", jogo.id);
            for c in visual {
                print!("{} ", c);
            }
            println!("");
        }
    }

    println!("--------------------------------------------------");

    // LIXO
    match &visao.lixo {
        Some(carta) => println!("LIXO: [{}]", carta),
        None => println!("LIXO: Vazio"),
    }

    println!("--------------------------------------------------");

    // TRES VERMELHOS (opcional, se tiver no struct)
    // println!("Três A: {:?} | Três B: {:?}", visao.tres_vermelho_time_a, visao.tres_vermelho_time_b);

    // VEZ E MÃO
    if visao.turno_atual == visao.meu_id {
        println!(">>> VEZ DO JOGADOR {} (É VOCÊ!) <<<", visao.turno_atual);
    } else {
        println!(">>> VEZ DO JOGADOR {} <<<", visao.turno_atual);
    }

    println!("SUA MÃO:");
    for (i, c) in visao.minha_mao.iter().enumerate() {
        // Cor cinza para o índice, normal para a carta
        print!("\x1b[90m{}:\x1b[0m[{}] ", i, c);
    }
    println!("\n");

    println!("COMANDOS: c (comprar), d <idx> (descartar), b (baixar), a (ajuntar), l (lixo)");
    print!("\nComando > ");
    io::stdout().flush().unwrap();
}

// --- FUNÇÕES AUXILIARES ---

fn ler_indice(s: Option<&&str>) -> Option<usize> {
    s.and_then(|x| x.parse::<usize>().ok())
}

fn ler_varios_indices(slice: &[&str]) -> Vec<usize> {
    slice
        .iter()
        .filter_map(|s| s.parse::<usize>().ok())
        .collect()
}

fn formatar_placar(visao: &VisaoJogador) -> String {
    format!(
        "PLACAR: Time A [{}] x [{}] Time B",
        visao.pontuacao_a, visao.pontuacao_b
    )
}

fn organizar_para_exibicao(cartas: &[Carta]) -> Vec<Carta> {
    if cartas.is_empty() {
        return vec![];
    }

    let naipe_dominante = descobrir_naipe_dominante(cartas);
    let mut naturais = Vec::new();
    let mut curingas = Vec::new();

    for c in cartas {
        if c.valor == Valor::Joker {
            curingas.push(c.clone());
        } else if c.valor == Valor::Dois && Some(c.naipe) != naipe_dominante {
            curingas.push(c.clone());
        } else {
            naturais.push(c.clone());
        }
    }

    if naturais.is_empty() {
        let mut t = curingas;
        t.sort_by_key(|c| c.valor.indice_sequencia());
        return t;
    }

    naturais.sort_by_key(|c| c.valor.indice_sequencia());

    // Verifica se é Trinca (lavadeira) ou Sequência
    if naturais.first().unwrap().valor == naturais.last().unwrap().valor {
        let mut r = naturais;
        r.append(&mut curingas);
        return r;
    }

    // Lógica visual de Sequência (inserindo curingas nos buracos)
    let mut resultado = Vec::new();
    let mut anterior = naturais[0].clone();
    resultado.push(anterior.clone());

    for carta_atual in naturais.into_iter().skip(1) {
        let idx_ant = anterior.valor.indice_sequencia();
        let idx_atual = carta_atual.valor.indice_sequencia();
        let gap = if idx_atual > idx_ant {
            idx_atual - idx_ant
        } else {
            0
        };

        if gap > 1 {
            for _ in 0..(gap - 1) {
                if let Some(curinga) = curingas.pop() {
                    resultado.push(curinga);
                }
            }
        }
        resultado.push(carta_atual.clone());
        anterior = carta_atual;
    }
    resultado.append(&mut curingas);
    resultado
}

fn descobrir_naipe_dominante(cartas: &[Carta]) -> Option<Naipe> {
    use std::collections::HashMap;
    let mut contagem = HashMap::new();
    for c in cartas {
        if c.valor != Valor::Joker {
            *contagem.entry(c.naipe).or_insert(0) += 1;
        }
    }
    contagem
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(n, _)| n)
}

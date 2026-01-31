use std::io::{self, Write};
// use std::str::FromStr; // Não estamos usando explicitamente, o parse resolve

// Importa tudo do core
use buracao_core::acoes::AcaoJogador;
use buracao_core::baralho::Carta;
use buracao_core::estado::EstadoJogo;

fn main() {
    // 1. Inicia o jogo
    let mut jogo = EstadoJogo::new();
    jogo.dar_cartas();
    let mut mensagem_erro = String::new();
    let mut mensagem_sucesso = String::new();

    // Loop principal do jogo
    loop {
        // Limpa a tela (funciona em terminais Linux/Mac/WSL e CMD modernos)
        print!("\x1B[2J\x1B[1;1H");

        println!("=== BURACO RUST CLI ===");
        println!(
            "Rodada: {} | Baralho: {} cartas",
            jogo.rodada,
            jogo.baralho.restantes()
        );

        // Placar
        println!(
            "Placar: Time A [{}] x [{}] Time B",
            jogo.pontuacao_a, jogo.pontuacao_b
        );
        println!("--------------------------------------------------");

        // Mostra a Mesa (Jogos baixados)
        println!("MESA TIME A:");
        exibir_mesa(&jogo.jogos_time_a);
        println!("MESA TIME B:");
        exibir_mesa(&jogo.jogos_time_b);

        println!("--------------------------------------------------");

        // Mostra o Lixo
        if let Some(topo) = jogo.lixo.last() {
            println!("LIXO ({} cartas): Topo -> [{}]", jogo.lixo.len(), topo);
        } else {
            println!("LIXO: Vazio");
        }

        println!("--------------------------------------------------");

        let str_tres = jogo
            .tres_vermelhos_time_a
            .iter()
            .map(|c| c.to_string()) // Usa o seu Display (emoji)
            .collect::<Vec<String>>()
            .join(" "); // Junta: "3♥️ 3♦️"

        println!("Três Vermelhos time A: [{}]", str_tres);

        let str_tres = jogo
            .tres_vermelhos_time_b
            .iter()
            .map(|c| c.to_string()) // Usa o seu Display (emoji)
            .collect::<Vec<String>>()
            .join(" "); // Junta: "3♥️ 3♦️"

        println!("Três Vermelhos time B: [{}]", str_tres);

        println!("--------------------------------------------------");

        // Identifica quem joga agora
        let id_jogador = jogo.turno_atual;
        println!(">>> VEZ DO JOGADOR {} <<<", id_jogador);

        // Se houver mensagem de erro da jogada anterior
        if !mensagem_erro.is_empty() {
            println!("\n!! AVISO: {} !!\n", mensagem_erro);
            mensagem_erro.clear();
        }

        // Mostra a mão do jogador com ÍNDICES
        let mao = &jogo.maos[id_jogador as usize];
        print!("SUA MÃO: ");
        for (i, carta) in mao.iter().enumerate() {
            print!("[{}: {}] ", i, carta);
        }
        println!("\n");

        // Menu de comandos atualizado
        println!("COMANDOS:");
        println!("  c           -> Comprar do Baralho");
        println!("  l           -> Comprar do Lixo");
        println!("  d <idx>     -> Descartar carta");
        println!("  b <idx...>  -> Baixar jogos. Use '/' para separar.");
        println!("                 Ex: 'b 0 1 2 / 5 6 7' baixa dois jogos de uma vez.");
        println!("  a <id> <idx...> -> Ajuntar em jogo existente");
        println!("  sair        -> Encerra");
        print!("\nDigite o comando: ");
        io::stdout().flush().unwrap();

        // Lê input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let partes: Vec<&str> = input.trim().split_whitespace().collect();

        if partes.is_empty() {
            continue;
        }

        // Mapeia comandos para o Enum AcaoJogador
        let acao = match partes[0] {
            "c" => Some(AcaoJogador::ComprarBaralho),

            "l" => {
                // Aqui usamos os campos exatos do seu Enum
                Some(AcaoJogador::ComprarLixo {
                    novos_jogos: vec![],
                    cartas_em_jogos_existentes: vec![],
                })
            }

            "d" => {
                if let Some(idx) = ler_indice(partes.get(1)) {
                    if idx < mao.len() {
                        Some(AcaoJogador::Descartar {
                            carta: mao[idx].clone(),
                        })
                    } else {
                        mensagem_erro = "Índice inválido".to_string();
                        None
                    }
                } else {
                    mensagem_erro = "Use: d <numero_da_carta>".to_string();
                    None
                }
            }

            "b" => {
                // Ex: b 0 1 2 / 4 5 6

                let argumentos = &partes[1..];
                let mut todos_os_jogos: Vec<Vec<Carta>> = Vec::new();
                let mut indices_jogo_atual: Vec<usize> = Vec::new();

                for item in argumentos {
                    if *item == "/" {
                        // Achou um separador: processa o jogo acumulado até agora
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
                    } else {
                        // É um número (índice)
                        if let Ok(idx) = item.parse::<usize>() {
                            indices_jogo_atual.push(idx);
                        }
                    }
                }

                // Processa o último grupo (que não tem "/" no final)
                if !indices_jogo_atual.is_empty() {
                    let cartas_jogo: Vec<Carta> = indices_jogo_atual
                        .iter()
                        .filter_map(|&i| mao.get(i).cloned())
                        .collect();
                    if !cartas_jogo.is_empty() {
                        todos_os_jogos.push(cartas_jogo);
                    }
                }

                // Envia para o Core
                if !todos_os_jogos.is_empty() {
                    Some(AcaoJogador::BaixarJogos {
                        jogos: todos_os_jogos,
                    })
                } else {
                    mensagem_erro = "Nenhuma carta válida identificada.".to_string();
                    None
                }
            }
            "a" => {
                // 'a <id_jogo> <indices_cartas>'
                // Ex: a 0 1 -> coloca a carta do indice 1 da mão no jogo de id 0
                if let Some(id_jogo_parsed) = partes.get(1).and_then(|s| s.parse::<u32>().ok()) {
                    let indices = ler_varios_indices(&partes[2..]);
                    let cartas_ajunte: Vec<Carta> = indices
                        .iter()
                        .filter_map(|&i| mao.get(i).cloned())
                        .collect();

                    if !cartas_ajunte.is_empty() {
                        Some(AcaoJogador::Ajuntar {
                            indice_jogo: id_jogo_parsed, // Campo corrigido para coincidir com o Enum
                            cartas: cartas_ajunte,
                        })
                    } else {
                        mensagem_erro = "Nenhuma carta selecionada.".to_string();
                        None
                    }
                } else {
                    mensagem_erro = "Use: a <id_jogo> <indices_cartas...>".to_string();
                    None
                }
            }

            "sair" => break,
            _ => {
                mensagem_erro = "Comando desconhecido".to_string();
                None
            }
        };

        // Executa a ação no Core
        if let Some(acao_real) = acao {
            let resultado = jogo.realizar_acao(id_jogador, acao_real);

            match resultado {
                Ok(msg) => {
                    // Guarda a mensagem para o próximo loop
                    mensagem_sucesso = msg;
                }
                Err(e) => {
                    mensagem_erro = format!("ERRO DE REGRA: {}", e);
                }
            }
        }
    }
}

// --- Funções Auxiliares de Display e Parse ---

fn exibir_mesa(jogos: &std::collections::HashMap<u32, Vec<Carta>>) {
    if jogos.is_empty() {
        println!("  (Mesa Vazia)");
        return;
    }
    let mut lista: Vec<_> = jogos.iter().collect();
    lista.sort_by_key(|(id, _)| *id);

    for (id, cartas) in lista {
        // AQUI ESTÁ O TRUQUE: Chamamos a função de organizar antes de imprimir
        let cartas_visuais = organizar_para_exibicao(cartas);

        print!("  ID [{}]: ", id);
        for c in cartas_visuais {
            print!("{} ", c);
        }

        if cartas.len() >= 7 {
            print!(" (Canastra!)");
        }
        println!("");
    }
}

fn organizar_para_exibicao(cartas: &[Carta]) -> Vec<Carta> {
    if cartas.is_empty() {
        return vec![];
    }

    // 1. Descobre o naipe dominante (o que aparece mais vezes)
    // Isso serve para saber se um '2' é carta natural ou curinga trocado
    let naipe_dominante = descobrir_naipe_dominante(cartas);

    // 2. Separa Naturals vs Curingas
    let mut naturais = Vec::new();
    let mut curingas = Vec::new();

    for c in cartas {
        if c.valor == buracao_core::baralho::Valor::Joker {
            // Joker é sempre curinga
            curingas.push(c.clone());
        } else if c.valor == buracao_core::baralho::Valor::Dois && Some(c.naipe) != naipe_dominante
        {
            // '2' de naipe diferente da sequência é Curinga!
            curingas.push(c.clone());
        } else {
            // Pode ser carta normal ou um '2' do mesmo naipe (que pode ser curinga ou não)
            // Para simplificar a visualização, se for do mesmo naipe, tratamos como natural
            // a menos que cause duplicidade, mas vamos manter simples.
            naturais.push(c.clone());
        }
    }

    // Se só tem curingas, retorna tudo junto
    if naturais.is_empty() {
        let mut tudo = curingas;
        tudo.sort_by_key(|c| c.valor.indice_sequencia()); // Ordena os curingas entre si se quiser
        return tudo;
    }

    // 3. Ordena as cartas naturais pelo valor (As=1 ... Rei=13)
    naturais.sort_by_key(|c| c.valor.indice_sequencia());

    // 4. Verifica se é TRINCA (ex: K♠️ K♣️ K♦️)
    // Se o primeiro e último têm o mesmo valor, é lavadeira/trinca.
    // Nesse caso, o curinga fica no fim.
    if naturais.first().unwrap().valor == naturais.last().unwrap().valor {
        let mut resultado = naturais;
        resultado.append(&mut curingas);
        return resultado;
    }

    // 5. LÓGICA DE SEQUÊNCIA (Preencher Buracos)
    let mut resultado = Vec::new();

    // Pega a primeira
    let mut anterior = naturais[0].clone();
    resultado.push(anterior.clone());

    // Itera sobre o restante das naturais
    for carta_atual in naturais.into_iter().skip(1) {
        let idx_ant = anterior.valor.indice_sequencia();
        let idx_atual = carta_atual.valor.indice_sequencia();

        // Detecta o buraco. Ex: 10 (idx 10) e Q (idx 12). Diferença = 2.
        let gap = if idx_atual > idx_ant {
            idx_atual - idx_ant
        } else {
            0
        };

        if gap > 1 {
            // Precisa preencher (gap - 1) espaços
            // Ex: Entre 10 e Q (gap 2), cabe 1 carta.
            let precisa = gap - 1;
            for _ in 0..precisa {
                if let Some(curinga) = curingas.pop() {
                    resultado.push(curinga);
                }
            }
        }

        resultado.push(carta_atual.clone());
        anterior = carta_atual;
    }

    // 6. Se ainda sobrou curinga (ex: sequência terminou aberta, ou curinga é o ás), põe no final
    // Nota: Para visualização perfeita (ex: curinga antes do 3), a lógica seria mais complexa,
    // mas colocar no final funciona para 99% dos casos de leitura.
    resultado.append(&mut curingas);

    resultado
}

// Helper para contar naipes
fn descobrir_naipe_dominante(cartas: &[Carta]) -> Option<buracao_core::baralho::Naipe> {
    use std::collections::HashMap;
    let mut contagem = HashMap::new();

    for c in cartas {
        // Ignora Joker e (opcionalmente) o 2 para contagem de naipe,
        // mas contar o 2 ajuda se a sequencia for pura de copas.
        if c.valor != buracao_core::baralho::Valor::Joker {
            *contagem.entry(c.naipe).or_insert(0) += 1;
        }
    }

    // Retorna o naipe com maior contagem
    contagem
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(naipe, _)| naipe)
}
fn ler_indice(s: Option<&&str>) -> Option<usize> {
    s.and_then(|x| x.parse::<usize>().ok())
}

fn ler_varios_indices(slice: &[&str]) -> Vec<usize> {
    slice
        .iter()
        .filter_map(|s| s.parse::<usize>().ok())
        .collect()
}

use buracao_core::baralho::{Carta, Naipe, Valor, Verso};
use std::collections::HashMap;

pub fn carta_para_asset(carta: &Carta) -> String {
    let valor_str = match carta.valor {
        Valor::As => "a",
        Valor::Dois => "2",
        Valor::Tres => "3",
        Valor::Quatro => "4",
        Valor::Cinco => "5",
        Valor::Seis => "6",
        Valor::Sete => "7",
        Valor::Oito => "8",
        Valor::Nove => "9",
        Valor::Dez => "10", // Mudamos de "t" para "10"
        Valor::Valete => "j",
        Valor::Dama => "q",
        Valor::Rei => "k",
        Valor::Joker => return "joker_r".to_string(),
    };

    let naipe_str = match carta.naipe {
        Naipe::Paus => "c",
        Naipe::Ouros => "d",
        Naipe::Copas => "h",
        Naipe::Espadas => "s",
        Naipe::Nenhum => "joker",
    };

    if carta.valor == Valor::Joker {
        return "joker_b".to_string();
    }

    format!("{}_{}", naipe_str, valor_str)
}

pub enum StatusCanastra {
    Normal, // Menos de 7 cartas
    Real,   // 7+ cartas, sem coringuinha (2), permite Joker
    Suja,   // 7+ cartas, com coringuinha (2)
}

pub fn analisar_status_canastra(cartas: &[Carta]) -> StatusCanastra {
    if cartas.len() < 7 {
        return StatusCanastra::Normal;
    }

    // A regra da Suja é: ter um 2 (Coringuinha)
    // Nota: O método eh_coringa() que você mencionou verifica se é Valor::Dois
    let tem_coringuinha = cartas.iter().any(|c| c.valor == Valor::Dois);

    if tem_coringuinha {
        StatusCanastra::Suja
    } else {
        // Se tem 7+ e não tem 2, é Real (mesmo se tiver Joker, conforme sua regra)
        StatusCanastra::Real
    }
}

pub fn verso_para_asset(verso: Option<Verso>) -> String {
    match verso {
        // CORREÇÃO: Usando os nomes exatos do seu Enum (Red/Blue)
        Some(Verso::Blue) => "back_b".to_string(),
        Some(Verso::Red) => "back_r".to_string(),

        // Fallback caso seja None
        None => "back_r".to_string(),
    }
}

// pub fn carta_para_asset_path(carta: &Carta) -> String {
//     let id_arquivo = match carta.valor {
//         Valor::Joker => "joker_r".to_string(), // Coringa Vermelho
//         _ => {
//             let valor_str = match carta.valor {
//                 Valor::As => "a",
//                 Valor::Valete => "j",
//                 Valor::Dama => "q",
//                 Valor::Rei => "k",
//                 Valor::Dez => "10",
//                 v => {
//                     // Para valores numéricos, usamos a representação simples
//                     // Assumindo que seu enum Valor possa ser convertido ou comparado
//                     match v {
//                         Valor::Dois => "2",
//                         Valor::Tres => "3",
//                         Valor::Quatro => "4",
//                         Valor::Cinco => "5",
//                         Valor::Seis => "6",
//                         Valor::Sete => "7",
//                         Valor::Oito => "8",
//                         Valor::Nove => "9",
//                         _ => "unknown",
//                     }
//                 }
//             };
//
//             let naipe_str = match carta.naipe {
//                 Naipe::Paus => "c",
//                 Naipe::Ouros => "d",
//                 Naipe::Copas => "h",
//                 Naipe::Espadas => "s",
//                 Naipe::Nenhum => "joker",
//             };
//
//             format!("{}_{}", naipe_str, valor_str)
//         }
//     };
//
//     get_card_path(&id_arquivo)
// }

pub fn organizar_para_exibicao(cartas: &[Carta]) -> Vec<Carta> {
    if cartas.is_empty() {
        return vec![];
    }

    // 1. Descobre o naipe (mantido para compatibilidade, embora não crítico para essa lógica visual específica)
    let naipe_dominante = descobrir_naipe_dominante(cartas);

    let mut naturais = Vec::new();
    let mut curingas = Vec::new();

    // 2. CLASSIFICAÇÃO RIGOROSA (A CORREÇÃO DO BUG VISUAL)
    // Tratamos TODOS os 2 como curingas para fins de ordenação visual.
    // Isso impede que um 2 "natural" seja ordenado numericamente antes do 3 ou 4,
    // o que causaria o bug visual de aparecer no meio ou início errado.
    for c in cartas {
        if c.valor == Valor::Joker || c.valor == Valor::Dois {
            curingas.push(c.clone());
        } else {
            naturais.push(c.clone());
        }
    }

    // Helper: Ás vale 14 para ficar no topo
    let valor_visual = |c: &Carta| -> u8 {
        match c.valor {
            Valor::As => 14,
            _ => c.valor.indice_sequencia(),
        }
    };

    // Se só tem curingas, retorna eles
    if naturais.is_empty() {
        let mut t = curingas;
        t.sort_by_key(|c| valor_visual(c));
        return t;
    }

    // 3. Ordena os naturais (Ex: 8, 9, 10, J)
    naturais.sort_by_key(|c| valor_visual(c));

    // Verifica Lavadeira (Trinca de valores iguais)
    if naturais.len() >= 2 && naturais.first().unwrap().valor == naturais.last().unwrap().valor {
        let mut r = naturais;
        r.append(&mut curingas);
        return r;
    }

    // 4. PREENCHIMENTO DE LACUNAS (BURACOS)
    let mut resultado = Vec::new();
    let mut anterior = naturais[0].clone();
    resultado.push(anterior.clone());

    for carta_atual in naturais.into_iter().skip(1) {
        let idx_ant = valor_visual(&anterior);
        let idx_atual = valor_visual(&carta_atual);

        let gap = if idx_atual > idx_ant {
            idx_atual - idx_ant
        } else {
            0
        };

        // Se gap > 1 (Ex: tem 8 e 10, gap é 2), precisamos de (2-1) = 1 coringa no meio
        if gap > 1 {
            let buracos_para_tapar = gap - 1;
            for _ in 0..buracos_para_tapar {
                if let Some(curinga) = curingas.pop() {
                    resultado.push(curinga);
                }
            }
        }

        resultado.push(carta_atual.clone());
        anterior = carta_atual;
    }

    // 5. LÓGICA INTELIGENTE DE EXTENSÃO (FIM vs COMEÇO)
    // Aqui garantimos que ele cresça para o topo (até o Ás) e só depois vá para o início.

    if !curingas.is_empty() {
        // Pega o valor da última carta visualmente inserida
        let ultima_carta = resultado.last().unwrap();
        let mut valor_ultimo = valor_visual(ultima_carta);

        // A. Tenta crescer para a DIREITA (até bater no Ás/14)
        // Ex: Se temos [J, Q] e 2 curingas -> Vira [J, Q, 2(K), 2(A)]
        let mut i = 0;
        while i < curingas.len() {
            if valor_ultimo < 14 {
                // Adiciona ao final
                resultado.push(curingas[i].clone());
                valor_ultimo += 1;
                i += 1;
            } else {
                // Bateu no teto (Ás), interrompe. O resto vai pro começo.
                break;
            }
        }

        // Remove do vetor de curingas os que já usamos para crescer pra direita
        if i > 0 {
            curingas.drain(0..i);
        }

        // B. Se AINDA sobraram curingas (ou porque bateu no Ás, ou porque era sequência terminada em Ás)
        // Insere no começo (crescendo para a esquerda)
        // Ex: [Q, K, A] e sobrou 1 curinga -> Vira [2(J), Q, K, A]
        for c in curingas {
            resultado.insert(0, c);
        }
    }

    resultado
}

fn descobrir_naipe_dominante(cartas: &[Carta]) -> Option<Naipe> {
    let mut contagem = HashMap::new();
    for c in cartas {
        // Ignora Jokers e 2 para contagem de naipe dominante inicial
        if c.valor != Valor::Joker && c.valor != Valor::Dois {
            *contagem.entry(c.naipe).or_insert(0) += 1;
        }
    }

    // Se só tinha coringas/2, tenta contar com os 2
    if contagem.is_empty() {
        for c in cartas {
            if c.valor == Valor::Dois {
                *contagem.entry(c.naipe).or_insert(0) += 1;
            }
        }
    }

    contagem
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(n, _)| n)
}

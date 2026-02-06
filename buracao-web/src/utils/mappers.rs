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

    // HELPER: Ás vale 14 visualmente
    let valor_visual = |c: &Carta| -> u8 {
        match c.valor {
            Valor::As => 14,
            _ => c.valor.indice_sequencia(),
        }
    };

    if naturais.is_empty() {
        let mut t = curingas;
        t.sort_by_key(|c| valor_visual(c));
        return t;
    }

    naturais.sort_by_key(|c| valor_visual(c));

    // Trinca (lavadeira)
    if naturais.first().unwrap().valor == naturais.last().unwrap().valor {
        let mut r = naturais;
        r.append(&mut curingas);
        return r;
    }

    // Sequência (Lógica de preencher buracos)
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

    // --- NOVA LÓGICA DE POSICIONAMENTO DO CORINGA ---

    if !curingas.is_empty() {
        // Verifica a última carta colocada na sequência visual
        if let Some(ultima_carta) = resultado.last() {
            let valor_ultimo = valor_visual(ultima_carta);

            // REGRA: Se a sequência termina em Ás (14), não dá pra subir mais.
            // Então o coringa deve ir para o INÍCIO (antes do 10, ou antes do 4 se estiver completa).
            // Se termina em qualquer outra carta (ex: Rei, 10, 6), o coringa vai pro FINAL pra continuar crescendo.
            if valor_ultimo == 14 {
                // Insere todos os coringas restantes no começo
                for c in curingas {
                    resultado.insert(0, c);
                }
            } else {
                // Comportamento padrão: adiciona ao final
                resultado.append(&mut curingas);
            }
        } else {
            // Fallback (não deve acontecer pois resultado começa com 1 carta)
            resultado.append(&mut curingas);
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

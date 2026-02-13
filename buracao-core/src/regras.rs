use crate::baralho::{Carta, Valor};

pub fn validar_jogo(cartas: &[Carta]) -> bool {
    // Regra 1: Mínimo de 3 cartas
    if cartas.len() < 3 {
        return false;
    }

    // --- PASSO A: SEPARAR CORINGAS DE NATURAIS ---
    let mut naturais: Vec<&Carta> = Vec::new();
    let mut coringas_count = 0;

    for c in cartas {
        if c.eh_coringa() || c.eh_joker() {
            coringas_count += 1;
        } else {
            naturais.push(c);
        }
    }

    // Regra Buraco: Máximo 1 coringa por jogo (seja sequência ou trinca)
    if coringas_count > 1 {
        return false;
    }

    // Se só tem coringas, é inválido
    if naturais.is_empty() {
        return false;
    }

    // --- BIFURCAÇÃO: TRINCA DE ASES VS SEQUÊNCIA ---

    // Verifica se TODAS as cartas naturais são Ases
    let sao_todos_ases = naturais.iter().all(|c| c.valor == Valor::As);

    if sao_todos_ases {
        // === ROTA 1: VALIDAÇÃO DE TRINCA DE ASES (LAVADEIRA) ===
        // Regras da Lavadeira:
        // 1. Mínimo 3 cartas (já checado no início).
        // 2. Todos são Ases (já checado no if).

        return true; // É uma lavadeira válida!
    }

    // === ROTA 2: VALIDAÇÃO DE SEQUÊNCIA (O seu código original) ===

    // PASSO B (Sequência): Validar se todos têm o MESMO naipe
    let naipe_referencia = naturais[0].naipe;
    for c in &naturais {
        if c.naipe != naipe_referencia {
            return false; // Na sequência, naipes misturados são proibidos
        }
    }
    // PASSO C (Sequência): Ordenar e Verificar Lacunas
    naturais.sort_by_key(|c| c.valor_numerico_sequencia());

    let mut lacunas_para_preencher = 0;

    for i in 0..(naturais.len() - 1) {
        let valor_atual = naturais[i].valor_numerico_sequencia();
        let valor_proximo = naturais[i + 1].valor_numerico_sequencia();

        // Checagem de segurança
        if valor_atual == 0 || valor_proximo == 0 {
            return false;
        }

        // Duplicata em sequência é inválido (ex: 4-4-5)
        if valor_atual == valor_proximo {
            return false;
        }

        // Calcula o intervalo
        // Ex: 4 e 5 -> (5 - 4 - 1) = 0 lacunas
        // Ex: 4 e 6 -> (6 - 4 - 1) = 1 lacuna
        let diff = valor_proximo - valor_atual - 1;
        lacunas_para_preencher += diff;
    }

    // PASSO D: Temos coringas suficientes?
    coringas_count >= lacunas_para_preencher
}

pub fn tem_coringa(jogo: &[Carta]) -> bool {
    jogo.iter().any(|c| c.eh_coringa())
}
